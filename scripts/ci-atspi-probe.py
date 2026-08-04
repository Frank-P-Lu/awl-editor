#!/usr/bin/env python3
"""AT-SPI2 bridge-liveness and structure probe (item 252, CI-only).

Grades the bridge itself, not something adjacent to it (the four vacuous laws
in items 237/244/247/248 all made that mistake). Launches the real awl binary
under a real X display, connects to the AT-SPI2 accessibility bus as an
ordinary assistive-technology client would, and asserts that the tree
`SemanticSnapshot` intends is actually there:

  - the application registers on the bus at all (a broken/disabled adapter
    means this alone never appears — that is the mutation-proof case);
  - a window (ROLE_FRAME);
  - inside it, the editable multiline document (ROLE_ENTRY, EDITABLE,
    MULTI_LINE, eventually FOCUSED);
  - item 218's STABLE LINE RUNS as its children, one per rope line plus the
    trailing empty run, each ROLE_STATIC with the exact expected text — a
    monolithic single-node document (the pre-218 shape) fails this exact
    check, which is the point: this confirms the RUN-BASED tree specifically,
    not merely that *a* tree exists;
  - a real caret/selection: the initial caret at offset 0, and — driven by an
    actual Shift+Right keypress delivered through X11, not synthesized — a
    live AT-SPI selection afterwards.

THIS IS NOT ITEM 251. It says nothing about what a screen reader user would
hear or how navigation feels — only that the bridge is live and shaped right.
Item 251 still needs a human at a real Linux desktop running Orca.

Exits non-zero, with a message naming exactly what was missing, on any
mismatch or timeout. Exit 0 only when every assertion below passed.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib  # noqa: E402

APP_TIMEOUT_S = 30.0
FOCUS_TIMEOUT_S = 10.0
SELECTION_TIMEOUT_S = 10.0
POLL_S = 0.5
MAX_DEPTH = 12

FIXTURE_LINES = ["line one", "line two", "line three"]
# item 218's shape: one run per rope line, PLUS a trailing empty run for the
# implied empty line after the fixture's final newline.
EXPECTED_RUN_TEXT = [line + "\n" for line in FIXTURE_LINES] + [""]


def fail(msg: str) -> None:
    print(f"ATSPI-PROBE FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def find_by_pid(node, pid, depth=0):
    try:
        if node.get_process_id() == pid:
            return node
    except GLib.Error:
        pass
    if depth >= MAX_DEPTH:
        return None
    try:
        count = node.get_child_count()
    except GLib.Error:
        return None
    for i in range(count):
        try:
            child = node.get_child_at_index(i)
        except GLib.Error:
            continue
        if child is None:
            continue
        found = find_by_pid(child, pid, depth + 1)
        if found is not None:
            return found
    return None


def find_role(node, role, depth=0):
    if node.get_role() == role:
        return node
    if depth >= MAX_DEPTH:
        return None
    try:
        count = node.get_child_count()
    except GLib.Error:
        return None
    for i in range(count):
        try:
            child = node.get_child_at_index(i)
        except GLib.Error:
            continue
        if child is None:
            continue
        found = find_role(child, role, depth + 1)
        if found is not None:
            return found
    return None


def text_of(node) -> str:
    """The run's text, across whichever GI Text-interface shape is live."""
    try:
        count = node.get_character_count()
        return node.get_text(0, count)
    except (AttributeError, GLib.Error):
        pass
    iface = node.get_text_iface()
    count = iface.get_character_count()
    return iface.get_text(0, count)


def caret_offset_of(node) -> int:
    try:
        return node.get_caret_offset()
    except (AttributeError, GLib.Error):
        return node.get_text_iface().get_caret_offset()


def selection_of(node):
    """(n_selections, (start, end) | None) across whichever GI shape is live."""
    try:
        n = node.get_n_selections()
        rng = node.get_selection(0) if n > 0 else None
    except (AttributeError, GLib.Error):
        iface = node.get_text_iface()
        n = iface.get_n_selections()
        rng = iface.get_selection(0) if n > 0 else None
    if rng is None:
        return n, None
    if isinstance(rng, tuple):
        return n, rng
    return n, (rng.start_offset, rng.end_offset)


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: ci-atspi-probe.py <path-to-awl-binary>")
    binary = sys.argv[1]

    fixture_dir = tempfile.mkdtemp(prefix="awl-atspi-fixture-")
    fixture_path = os.path.join(fixture_dir, "probe.md")
    with open(fixture_path, "w", encoding="utf-8") as f:
        f.write("".join(line + "\n" for line in FIXTURE_LINES))

    awl_log_path = os.path.join(fixture_dir, "awl.log")
    awl_log = open(awl_log_path, "wb")

    def dump_awl_log():
        awl_log.flush()
        try:
            with open(awl_log_path, "r", encoding="utf-8", errors="replace") as f:
                tail = f.readlines()[-60:]
            print("----- awl output (tail) -----", file=sys.stderr)
            sys.stderr.writelines(tail)
            print("------------------------------", file=sys.stderr)
        except OSError:
            pass

    print(f"ATSPI-PROBE: launching {binary} {fixture_path}")
    proc = subprocess.Popen(
        [binary, fixture_path],
        stdout=awl_log,
        stderr=subprocess.STDOUT,
        env=dict(os.environ),
    )

    try:
        rc = Atspi.init()
        if rc not in (0, 1):
            fail(f"Atspi.init() returned {rc}")

        desktop = Atspi.get_desktop(0)

        deadline = time.time() + APP_TIMEOUT_S
        app = None
        while time.time() < deadline:
            if proc.poll() is not None:
                dump_awl_log()
                fail(
                    f"awl exited early with code {proc.returncode} before the "
                    "AT-SPI app node ever appeared"
                )
            app = find_by_pid(desktop, proc.pid)
            if app is not None:
                break
            time.sleep(POLL_S)
        if app is None:
            dump_awl_log()
            fail(
                f"no AT-SPI application for pid {proc.pid} appeared under the "
                f"desktop within {APP_TIMEOUT_S}s — the bridge never registered "
                "with the accessibility bus"
            )

        frame = find_role(app, Atspi.Role.FRAME)
        if frame is None:
            dump_awl_log()
            fail("no ROLE_FRAME (window) under the awl application node")

        document = find_role(frame, Atspi.Role.ENTRY)
        if document is None:
            dump_awl_log()
            fail(
                "no ROLE_ENTRY under the frame — SemanticSnapshot's editable "
                "multiline document node did not cross the bridge"
            )

        state = document.get_state_set()
        if not state.contains(Atspi.StateType.EDITABLE):
            fail("document node is missing STATE_EDITABLE")
        if not state.contains(Atspi.StateType.MULTI_LINE):
            fail("document node is missing STATE_MULTI_LINE")

        focus_deadline = time.time() + FOCUS_TIMEOUT_S
        while (
            not state.contains(Atspi.StateType.FOCUSED)
            and time.time() < focus_deadline
        ):
            time.sleep(POLL_S)
            state = document.get_state_set()
        if not state.contains(Atspi.StateType.FOCUSED):
            fail(
                "document node never reports STATE_FOCUSED — focus did not "
                "cross the bridge"
            )

        run_count = document.get_child_count()
        if run_count != len(EXPECTED_RUN_TEXT):
            fail(
                f"document has {run_count} children, expected "
                f"{len(EXPECTED_RUN_TEXT)} stable line runs (item 218's shape) "
                "for the 3-line fixture — a monolithic single-node document "
                "(the pre-218 shape) would fail this exact check"
            )

        for i, want in enumerate(EXPECTED_RUN_TEXT):
            run = document.get_child_at_index(i)
            if run is None:
                fail(f"line run {i} is missing")
            if run.get_role() != Atspi.Role.STATIC:
                fail(
                    f"line run {i} has role {run.get_role_name()!r}, expected "
                    "'static' (accesskit's Role::TextRun -> AtspiRole::Static "
                    "mapping)"
                )
            got = text_of(run)
            if got != want:
                fail(
                    f"line run {i} text is {got!r}, expected {want!r} — item "
                    "218's per-line run text did not cross the bridge intact"
                )

        caret = caret_offset_of(document)
        if caret != 0:
            fail(
                f"document caret_offset is {caret} at launch, expected 0 — "
                "the Text interface is present but its initial value is wrong"
            )

        # Live selection: drive a real Shift+Right through X11 (not
        # synthesized on the AT-SPI side) and confirm it crosses the bridge.
        found = subprocess.run(
            ["xdotool", "search", "--name", "^awl - "],
            capture_output=True,
            text=True,
        )
        winids = [w for w in found.stdout.split() if w]
        if not winids:
            fail(
                "xdotool found no window titled 'awl - ...' to drive a live "
                f"selection into (stderr: {found.stderr.strip()!r})"
            )
        winid = winids[0]
        subprocess.run(["xdotool", "windowfocus", "--sync", winid], check=False)
        subprocess.run(
            ["xdotool", "key", "--window", winid, "--clearmodifiers", "shift+Right"],
            check=False,
        )

        sel_deadline = time.time() + SELECTION_TIMEOUT_S
        n_sel, sel_range = 0, None
        while time.time() < sel_deadline:
            n_sel, sel_range = selection_of(document)
            if sel_range is not None:
                break
            time.sleep(POLL_S)
        if sel_range is None:
            fail(
                "Shift+Right into the live window never produced an AT-SPI "
                "selection on the document node (n_selections stayed 0) — "
                "selection did not cross the bridge"
            )
        if tuple(sel_range) != (0, 1):
            fail(
                f"AT-SPI selection after Shift+Right is {tuple(sel_range)}, "
                "expected (0, 1)"
            )

        print(
            "ATSPI-PROBE PASS: awl registered with the AT-SPI2 bus; the frame, "
            f"the editable multiline document (focused), its {run_count} "
            "stable line runs with matching text, and a live keyboard-driven "
            "selection all crossed the bridge intact."
        )
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
        awl_log.close()


if __name__ == "__main__":
    main()
