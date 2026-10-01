#!/usr/bin/env python3
"""Exercise verification dispatch and failure semantics without expensive editor runs."""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


def jobs(text: str) -> dict[str, str]:
    text = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))
    body = text.split("\njobs:\n", 1)[1]
    matches = list(re.finditer(r"^  ([a-z][a-z0-9-]*):$", body, re.M))
    return {
        match[1]: body[match.end():matches[i + 1].start() if i + 1 < len(matches) else len(body)]
        for i, match in enumerate(matches)
    }


def wiring_errors(ci: str, release: str, extended: str) -> list[str]:
    errors = []
    c, r, e = jobs(ci), jobs(release), jobs(extended)
    for job in ["linux", "mac", "web", "mac-live-probe"]:
        if job not in c or "continue-on-error:" in c[job]:
            errors.append(f"required CI job lost or tolerated: {job}")
    linux = c.get("linux", "")
    if linux.count("run: scripts/native-gate.sh") != 1 or "run: bash scripts/code-health.sh" in linux:
        errors.append("Linux must run one native gate and no duplicate complete health")
    for job, command in [
        ("linux", "run: scripts/release-profile-gate.sh"),
        ("web", "run: scripts/web-smoke.sh"),
        ("mac-live-probe", "run: scripts/ci-live-probe.sh 25"),
    ]:
        if command not in c.get(job, ""):
            errors.append(f"required command lost: {command}")
    for convention in ["mac", "linux"]:
        if f"env AWL_CONVENTION_FORCE={convention} cargo test -- --skip render::tests" not in c.get("mac", ""):
            errors.append(f"Mac convention lost: {convention}")
    if "cron: '17 3 * * *'" not in extended or "workflow_call:" not in extended:
        errors.append("overnight and pre-release triggers missing")
    broad = e.get("journeys", "")
    for command in [
        "os: [ubuntu-latest, macos-latest]",
        "run: scripts/verify.sh extended --bin target/release/awl --jobs 1",
        "--seconds 120",
    ]:
        if (command not in [line.strip() for line in broad.splitlines()]
                if command.startswith(("os:", "run:")) else command not in broad):
            errors.append(f"broader coverage lost: {command}")
    if "continue-on-error:" in broad:
        errors.append("broader findings must fail")
    if "uses: ./.github/workflows/extended-verification.yml" not in r.get("extended", ""):
        errors.append("release journey prerequisite lost")
    for job in ["mac", "linux"]:
        if "needs: [plan, extended]" not in r.get(job, ""):
            errors.append(f"release {job} no longer waits for journeys")
    for job, artifact in [
        ("mac", '$MOUNT/Awl.app/Contents/MacOS/awl'),
        ("linux", '$UNPACK/awl-$VERSION-linux-x86_64/awl'),
        ("linux", '$PWD/dist-linux/awl-$VERSION-linux-x86_64.AppImage'),
    ]:
        if f'scripts/release-launch-smoke.py --binary "{artifact}"' not in r.get(job, ""):
            errors.append(f"actual packaged launch lost: {artifact}")
    return errors


class Dispatch(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="awl-verification-law-")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "scripts").mkdir()
        (self.root / "bin").mkdir()
        shutil.copy(ROOT / "scripts/verify.sh", self.root / "scripts/verify.sh")
        self.env = dict(os.environ, PATH=f"{self.root}/bin:/usr/bin:/bin",
                        LAW_LOG=str(self.root / "calls.jsonl"), LAW_HEAD=str(self.root / "head"))
        (self.root / "head").write_text("frozen")
        fake = f"""#!{sys.executable}
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
if name == 'git':
    print(pathlib.Path(os.environ['LAW_HEAD']).read_text() if args == ['rev-parse', 'HEAD'] else os.environ.get('LAW_DIRTY', ''))
    sys.exit(0)
with open(os.environ['LAW_LOG'], 'a') as log: log.write(json.dumps([name, *args]) + '\\n')
if os.environ.get('LAW_MUTATE') == name: pathlib.Path(os.environ['LAW_HEAD']).write_text('changed')
if os.environ.get('LAW_FAIL') == name: sys.exit(7)
if name == 'cargo' and args[0] == 'test':
    count = 0 if 'MISSING' in args else 1
    print(f'test result: ok. {{count}} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')
"""
        for name in ["cargo", "git", "wasm-bindgen-test-runner"]:
            p = self.root / "bin" / name
            p.write_text(fake)
            p.chmod(0o755)
        for name in ["native-gate.sh", "web-smoke.sh", "release-profile-gate.sh", "code-health.sh"]:
            p = self.root / "scripts" / name
            p.write_text(fake)
            p.chmod(0o755)

    def run_verify(self, *args, **env):
        self.env.update(env)
        return subprocess.run(["bash", str(self.root / "scripts/verify.sh"), *args],
                              env=self.env, text=True, capture_output=True, timeout=15)

    def calls(self):
        p = self.root / "calls.jsonl"
        return [json.loads(line) for line in p.read_text().splitlines()] if p.exists() else []

    def test_fast_requires_real_selectors_before_running(self):
        for args in [("fast",), ("fast", "--skip"), ("unknown",), ("full", "buffer::")]:
            self.assertNotEqual(self.run_verify(*args).returncode, 0)
        self.assertEqual(self.calls(), [])

    def test_fast_runs_owners_and_selected_tests_without_full_health(self):
        result = self.run_verify("fast", "buffer::", "actions::")
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        self.assertEqual(calls[:2], [["cargo", "fmt", "--all", "--", "--check"],
                                    ["cargo", "check", "--all-targets"]])
        self.assertIn("println_audit", calls[2])
        self.assertIn("card::figures::tests::only_the_substitution_door_replaces_a_view_states_text", calls[2])
        self.assertIn("view_policy", calls[2])
        self.assertEqual(calls[-1], ["cargo", "test", "--bin", "awl", "--", "buffer::", "actions::"])
        self.assertIn("NOT a full", result.stdout)
        self.assertNotIn("code-health.sh", [call[0] for call in calls])

    def test_fast_rejects_empty_selection_and_propagates_compiler_failure(self):
        self.assertNotEqual(self.run_verify("fast", "MISSING").returncode, 0)
        (self.root / "calls.jsonl").unlink()
        self.assertEqual(self.run_verify("fast", "buffer::", LAW_FAIL="cargo").returncode, 7)
        self.assertEqual(len(self.calls()), 1)

    def test_opt_in_lint_retains_targeted_label(self):
        self.assertEqual(self.run_verify("fast", "--lint", "buffer::").returncode, 0)
        self.assertIn(["code-health.sh", "--clippy-only"], self.calls())

    def test_full_uses_existing_owners_once_and_freezes_all_phases(self):
        result = self.run_verify("full")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([call[0] for call in self.calls()],
                         ["native-gate.sh", "web-smoke.sh", "release-profile-gate.sh"])
        self.assertIn("commit=frozen", result.stdout)
        self.assertNotEqual(self.run_verify("full", LAW_MUTATE="web-smoke.sh").returncode, 0)

    def test_full_never_skips_wasm_or_continues_after_failure(self):
        (self.root / "bin/wasm-bindgen-test-runner").unlink()
        self.assertNotEqual(self.run_verify("full").returncode, 0)
        self.assertEqual(self.calls(), [])
        (self.root / "bin/wasm-bindgen-test-runner").write_text("#!/bin/sh\nexit 0\n")
        (self.root / "bin/wasm-bindgen-test-runner").chmod(0o755)
        self.assertEqual(self.run_verify("full", LAW_FAIL="web-smoke.sh").returncode, 7)
        self.assertEqual([call[0] for call in self.calls()], ["native-gate.sh", "web-smoke.sh"])


class WorkflowWiring(unittest.TestCase):
    def test_real_wiring_and_mutations(self):
        ci = (ROOT / ".github/workflows/ci.yml").read_text()
        release = (ROOT / ".github/workflows/release.yml").read_text()
        extended = (ROOT / ".github/workflows/extended-verification.yml").read_text()
        self.assertEqual(wiring_errors(ci, release, extended), [])
        for text, bad in [
            (ci, ci.replace("run: scripts/native-gate.sh", "# run: scripts/native-gate.sh")),
            (ci, ci.replace("run: scripts/web-smoke.sh", "run: true")),
            (ci, ci.replace("run: scripts/native-gate.sh", "run: bash scripts/code-health.sh")),
            (ci, ci.replace("    name: linux (build + test)", "    continue-on-error: true\n    name: linux (build + test)")),
        ]:
            self.assertNotEqual(text, bad)
            self.assertTrue(wiring_errors(bad, release, extended))
        self.assertTrue(wiring_errors(ci, release.replace("needs: [plan, extended]", "needs: plan"), extended))
        self.assertTrue(wiring_errors(ci, release, extended.replace("--jobs 1", "--jobs 1 --worlds Saltpan")))
        self.assertTrue(wiring_errors(ci, release.replace("--binary", "--missing-binary"), extended))


class JourneyOracles(unittest.TestCase):
    def test_both_canvas_pairs_reject_unreachable_controls(self):
        spec = importlib.util.spec_from_file_location("pretag_journey_law", ROOT / "scripts/pretag-journeys.py")
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        roster = {journey.jid: journey for journey in module.JOURNEYS}
        self.assertEqual(len(module.STAGE_PAIRS), 2)
        for pair in module.STAGE_PAIRS:
            self.assertTrue(set(pair) <= roster.keys())
            self.assertEqual(roster[pair[0]].shape, roster[pair[1]].shape)
            for dpi in module.DPIS:
                sides = {(jid, dpi): {"overlay": {"workspace": True, "window": {"rows": []}}} for jid in pair}
                missing = module.Ledger()
                module.stage_presence(missing, "fixture", dpi, sides, pair)
                self.assertTrue(missing.failures)
                sides[(pair[1], dpi)]["overlay"]["window"]["rows"] = ["visible control"]
                reachable = module.Ledger()
                module.stage_presence(reachable, "fixture", dpi, sides, pair)
                self.assertEqual(reachable.failures, [])


class BuiltLaunch(unittest.TestCase):
    def test_actual_command_is_isolated_and_nonzero_or_timeout_fails(self):
        spec = importlib.util.spec_from_file_location("release_launch", ROOT / "scripts/release-launch-smoke.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory(prefix="awl-launch-law-") as folder:
            binary = Path(folder) / "built-app"
            evidence = Path(folder) / "evidence.json"
            binary.write_text(f"""#!{sys.executable}
import json, os, pathlib, sys
pathlib.Path({str(evidence)!r}).write_text(json.dumps({{'args': sys.argv[1:], 'cwd': os.getcwd(), 'data': os.environ['XDG_DATA_HOME']}}))
""")
            binary.chmod(0o755)
            self.assertEqual(module.launch(binary, 25, 5, True), 0)
            record = json.loads(evidence.read_text())
            self.assertEqual(record["args"][0], "--appimage-extract-and-run")
            # The isolated app mode accepts only these probe-owned arguments.
            self.assertEqual(record["args"][1:], ["--soak-gpu", "--soak-gpu-seconds", "25"])
            probe = (ROOT / "scripts/ci-live-probe.sh").read_text()
            self.assertIn('-- --soak-gpu --soak-gpu-seconds "${SECONDS_ARG}"', probe)
            # AppImage owns its first argument; Awl owns the remaining flags.
            roster = (ROOT / "src/main/args/flags/roster.rs").read_text()
            aliases = re.findall(r"^\s*\w+:\s*&\[([^]]*)\]", roster, re.M)
            registered = set(re.findall(r'"(--[a-z][a-z0-9-]*)"', " ".join(aliases)))
            used = {arg for arg in record["args"][1:] if arg.startswith("--")}
            self.assertEqual(used - registered, set())
            self.assertEqual((used | {"--notes-root"}) - registered, {"--notes-root"})
            self.assertNotEqual(record["data"], os.environ.get("XDG_DATA_HOME"))
            self.assertFalse(Path(record["cwd"]).exists())
            binary.write_text(f"#!{sys.executable}\nraise SystemExit(7)\n")
            self.assertEqual(module.launch(binary, 25, 5, False), 1)
            binary.write_text(f"#!{sys.executable}\nimport time\ntime.sleep(60)\n")
            self.assertEqual(module.launch(binary, 25, 0.1, False), 1)


if __name__ == "__main__":
    unittest.main()
