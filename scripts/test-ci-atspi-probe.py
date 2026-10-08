#!/usr/bin/env python3
"""Exercise the real probe with synthetic GI and X11 boundaries; no user input."""
import contextlib
import importlib.util
import io
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True


class InterfaceUnavailable(Exception):
    pass


atspi = types.SimpleNamespace(
    Role=types.SimpleNamespace(ENTRY="entry"),
    StateType=types.SimpleNamespace(EDITABLE="editable", MULTI_LINE="multiline", FOCUSED="focused"),
    TextGranularity=types.SimpleNamespace(LINE="line"),
    init=lambda: 0,
    get_desktop=lambda _: object(),
)
gi = types.ModuleType("gi")
gi.require_version = lambda *args: None
repository = types.ModuleType("gi.repository")
repository.Atspi = atspi
repository.GLib = types.SimpleNamespace(Error=InterfaceUnavailable)
with patch.dict(sys.modules, {"gi": gi, "gi.repository": repository}):
    spec = importlib.util.spec_from_file_location("probe", Path(__file__).with_name("ci-atspi-probe.py"))
    probe = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(probe)


class Clock:
    now = 0.0

    def monotonic(self):
        return self.now

    time = monotonic

    def sleep(self, seconds):
        self.now += seconds


class Document:
    def __init__(self, fault=None, delayed=False):
        self.fault = fault
        self.delayed = delayed
        self.reads = 0

    def get_state_set(self):
        return types.SimpleNamespace(contains=lambda state: state != self.fault)

    def get_character_count(self):
        self.reads += 1
        if self.fault == "interface" or (self.delayed and self.reads < 3):
            raise InterfaceUnavailable("Text interface not registered")
        return len("".join(probe.EXPECTED_RUN_TEXT))

    def get_text_iface(self):
        raise InterfaceUnavailable("Fallback Text interface not registered")

    def get_text(self, start, end):
        return "wrong" if self.fault == "text" else "".join(probe.EXPECTED_RUN_TEXT)

    def get_child_count(self):
        return 1 if self.fault == "children" else 0

    def get_string_at_offset(self, offset, granularity):
        assert granularity == atspi.TextGranularity.LINE
        lines = probe.EXPECTED_RUN_TEXT
        for line in lines:
            if offset == 0:
                return types.SimpleNamespace(content="wrong" if self.fault == "line" else line)
            offset -= len(line)
        raise AssertionError("Unexpected line offset")

    def get_caret_offset(self):
        return 1 if self.fault == "caret" else 0

    def get_n_selections(self):
        return 1

    def get_selection(self, index):
        assert index == 0
        return (1, 2) if self.fault == "selection" else (0, 1)


class ProbeLaws(unittest.TestCase):
    def run_probe(self, document, expected=None, replacements=None):
        clock = Clock()
        proc = types.SimpleNamespace(pid=123, returncode=0, poll=lambda: None,
                                     terminate=lambda: None, wait=lambda **kwargs: None)
        out, err = io.StringIO(), io.StringIO()
        with tempfile.TemporaryDirectory(prefix="awl-probe-law-") as fixture:
            with contextlib.ExitStack() as stack:
                stack.enter_context(patch.object(sys, "argv", ["probe", "synthetic-binary"]))
                stack.enter_context(patch.object(probe, "time", clock))
                stack.enter_context(patch.object(probe.tempfile, "mkdtemp", return_value=fixture))
                stack.enter_context(patch.object(probe.subprocess, "Popen", return_value=proc))
                stack.enter_context(patch.object(probe, "set_bus_enabled"))
                stack.enter_context(patch.object(probe, "bump_bus_enabled"))
                stack.enter_context(patch.object(probe, "find_by_pid", return_value=object()))
                stack.enter_context(patch.object(probe, "find_role", side_effect=replacements)
                                    if replacements is not None else
                                    patch.object(probe, "find_role", return_value=document))
                keys = stack.enter_context(patch.object(probe, "run_xdotool", return_value=types.SimpleNamespace(stdout="123", stderr="")))
                stack.enter_context(contextlib.redirect_stdout(out))
                stack.enter_context(contextlib.redirect_stderr(err))
                if expected is None:
                    probe.main()
                    self.assertIn("ATSPI-PROBE PASS", out.getvalue())
                    self.assertIn("4 stable line runs", out.getvalue())
                    self.assertTrue(any("shift+Right" in call.args[0] for call in keys.call_args_list))
                else:
                    with self.assertRaises(SystemExit) as failure:
                        probe.main()
                    self.assertNotEqual(failure.exception.code, 0)
                    self.assertIn(expected, err.getvalue())
                    self.assertNotIn("ATSPI-PROBE PASS", out.getvalue())
        self.assertLessEqual(clock.now, 10.0)
        return clock.now

    def test_real_main_reaches_success_with_all_required_oracles(self):
        self.run_probe(Document())

    def test_every_missing_contract_is_an_explicit_failure(self):
        self.run_probe(None, "ROLE_ENTRY")
        for fault, message in [("editable", "EDITABLE"), ("multiline", "MULTI_LINE"),
                               ("focused", "FOCUSED"), ("text", "matching document text"),
                               ("children", "accessible children"), ("line", "line 0"),
                               ("caret", "caret_offset"), ("selection", "selection after")]:
            with self.subTest(fault=fault):
                self.run_probe(Document(fault), message)

    def test_final_refetched_document_must_supply_all_required_states(self):
        for fault, message in [("editable", "EDITABLE"), ("multiline", "MULTI_LINE"),
                               ("focused", "matching document text")]:
            with self.subTest(fault=fault):
                good, bad = Document(), Document(fault)
                self.run_probe(good, message, [good, good] + [bad] * 21)

    def test_delayed_discovery_and_focus_use_refreshed_handles(self):
        good = Document()
        self.assertGreater(self.run_probe(good, replacements=[None, good, Document("focused"), good, good]), 0)

    def test_transient_discovery_interface_error_is_retried(self):
        good = Document()
        self.assertGreater(self.run_probe(good, replacements=[InterfaceUnavailable("node registering"), good, good, good]), 0)

    def test_both_text_interfaces_can_register_late(self):
        self.assertGreater(self.run_probe(Document(delayed=True)), 0)

    def test_permanently_unavailable_interfaces_fail_at_deadline(self):
        self.assertEqual(self.run_probe(Document("interface"), "Fallback Text interface"), 10.0)

    def test_early_process_exit_is_not_a_readiness_timeout(self):
        proc = types.SimpleNamespace(returncode=17, poll=lambda: 17)
        with patch.object(probe, "_AWL_LOG_PATH", None), contextlib.redirect_stderr(io.StringIO()) as err:
            with self.assertRaises(SystemExit):
                probe.await_document(object(), proc, lambda _: True, 10, "ROLE_ENTRY")
        self.assertIn("exited early with code 17", err.getvalue())


if __name__ == "__main__":
    unittest.main()
