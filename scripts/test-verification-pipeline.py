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


def wiring_errors(ci: str, release: str, extended: str, verifier: str | None = None) -> list[str]:
    errors = []
    c, r, e = jobs(ci), jobs(release), jobs(extended)
    for job in ["linux", "mac", "web", "mac-live-probe"]:
        if job not in c or "continue-on-error:" in c[job]:
            errors.append(f"required CI job lost or tolerated: {job}")
    for roster in [c, e]:
        for name, body in roster.items():
            if "uses: dtolnay/rust-toolchain" in body:
                errors.append(f"floating or independent Rust toolchain: {name}")
    for name in ["linux", "mac", "web", "mac-live-probe"]:
        if "uses: ./.github/actions/project-rust" not in c.get(name, ""):
            errors.append(f"project Rust activation lost: {name}")
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
        ("linux", '$UNPACK/awl-$VERSION-linux-x86_64/awl'),
        ("linux", '$PWD/dist-linux/awl-$VERSION-linux-x86_64.AppImage'),
    ]:
        if f'scripts/release-launch-smoke.py --binary "{artifact}"' not in r.get(job, ""):
            errors.append(f"actual packaged launch lost: {artifact}")
    mac = r.get("mac", "")
    for artifact in ["macos-arm64.dmg", "macos-x86_64.dmg"]:
        if artifact not in mac:
            errors.append(f"native Mac artifact lost: {artifact}")
    if verifier is None:
        verifier = (ROOT / "scripts/verify-macos-release.sh").read_text()
    if 'scripts/verify-macos-release.sh dist-mac' not in mac:
        errors.append("shared Mac validation owner disconnected")
    for command in [
        'package-macos.sh" --verify "$MOUNT/Awl.app" "$ARCH"',
        'check-macos-release-size.sh" "$DIST/$DMG" "$DIST/$APP_ZIP"',
    ]:
        if command not in verifier:
            errors.append(f"native Mac package gate lost: {command}")
    for name in ("mac", "mac-staged"):
        body = r.get(name, "")
        for command in (
            'RECEIPTS="$(mktemp -d "$RUNNER_TEMP/awl-mac-verification.XXXXXX")"',
            'ci "$RECEIPTS"',
            'python3 scripts/macos-verification-receipt.py check',
            '--receipt-dir "$RECEIPTS" --dist dist-mac --version "$VERSION"',
            '--build-version "$BUILD_VERSION" --source "$(git rev-parse HEAD)"',
            '--launch ci',
        ):
            if command not in body:
                errors.append(f"fresh Mac completion consumer lost in {name}: {command}")
        if 'macos-verification-receipt.py check' in body and 'scripts/verify-macos-release.sh' in body:
            if body.index('macos-verification-receipt.py check') < body.index('scripts/verify-macos-release.sh'):
                errors.append(f"Mac receipt checked before producer in {name}")
    if "macos-universal" in release or "lipo -create" in mac:
        errors.append("Mac release must remain two native downloads, not one universal binary")
    return errors


class Dispatch(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix="awl-verification-law-")
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "scripts").mkdir()
        (self.root / "bin").mkdir()
        shutil.copy(ROOT / "scripts/verify.sh", self.root / "scripts/verify.sh")
        shutil.copy(ROOT / "scripts/project-rust.sh", self.root / "scripts/project-rust.sh")
        self.env = dict(os.environ, PATH=f"{self.root}/bin:/usr/bin:/bin",
                        LAW_LOG=str(self.root / "calls.jsonl"), LAW_HEAD=str(self.root / "head"))
        (self.root / "head").write_text("frozen")
        fake = f"""#!{sys.executable}
import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
if name == 'rustup':
    if args[0] == 'run':
        os.environ['LAW_RUSTUP_RUNTIME'] = 'ready'
        os.execvpe(args[2], args[2:], os.environ)
    print(pathlib.Path(sys.argv[0]).parent / 'cargo' if args == ['which', 'cargo'] else '1.99.0-test')
    sys.exit(0)
if name == 'git':
    print(pathlib.Path(os.environ['LAW_HEAD']).read_text() if args == ['rev-parse', 'HEAD'] else os.environ.get('LAW_DIRTY', ''))
    sys.exit(0)
if os.environ.get('LAW_REQUIRE_RUSTUP_RUNTIME') and not os.environ.get('LAW_RUSTUP_RUNTIME'): sys.exit(9)
with open(os.environ['LAW_LOG'], 'a') as log: log.write(json.dumps([name, *args]) + '\\n')
if os.environ.get('LAW_MUTATE') == name: pathlib.Path(os.environ['LAW_HEAD']).write_text('changed')
if os.environ.get('LAW_FAIL') == name: sys.exit(7)
if name == 'cargo' and args[0] == 'test':
    count = 0 if 'MISSING' in args else 1
    print(f'test result: ok. {{count}} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')
"""
        for name in ["cargo", "git", "rustup", "wasm-bindgen-test-runner"]:
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

    def test_child_tools_use_pin_even_when_path_contains_direct_old_cargo(self):
        old_bin = self.root / "old-bin"
        old_bin.mkdir()
        old_cargo = old_bin / "cargo"
        old_cargo.write_text("#!/bin/sh\nexit 17\n")
        old_cargo.chmod(0o755)
        self.env["PATH"] = f"{old_bin}:" + self.env["PATH"]
        self.assertEqual(self.run_verify("fast", "buffer::", LAW_REQUIRE_RUSTUP_RUNTIME="1").returncode, 0)
        helper = self.root / "scripts/project-rust.sh"
        text = helper.read_text().replace('PATH="${awl_project_rust_cargo%/*}:$PATH"', ':')
        helper.write_text(text)
        # rustup run alone leaves a child script's old cargo link on PATH.
        self.assertEqual(self.run_verify("fast", "buffer::", LAW_REQUIRE_RUSTUP_RUNTIME="1").returncode, 17)

    def test_project_activation_preserves_rustup_runtime_environment(self):
        self.assertEqual(self.run_verify("full", LAW_REQUIRE_RUSTUP_RUNTIME="1").returncode, 0)
        helper = self.root / "scripts/project-rust.sh"
        text = helper.read_text()
        text = text.replace('exec rustup run "${awl_project_rust_active%% *}" "$@"', 'exec "$@"')
        helper.write_text(text)
        # The old direct-binary launch misses the loader environment and fails.
        self.assertNotEqual(self.run_verify("full", LAW_REQUIRE_RUSTUP_RUNTIME="1").returncode, 0)

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
        self.assertTrue(wiring_errors(ci, release.replace("macos-arm64.dmg", "macos-universal.dmg"), extended))
        for lost in (
            'RECEIPTS="$(mktemp -d "$RUNNER_TEMP/awl-mac-verification.XXXXXX")"',
            'ci "$RECEIPTS"',
            'python3 scripts/macos-verification-receipt.py check',
            '--build-version "$BUILD_VERSION" --source "$(git rev-parse HEAD)"',
        ):
            # Both independent native Mac producer routes must retain the guard.
            for job in ("mac", "mac-staged"):
                header = f"\n  {job}:\n"
                start = release.index(header)
                next_job = re.search(r"\n  [a-z][a-z0-9-]*:\n", release[start + len(header):])
                end = start + len(header) + next_job.start() if next_job else len(release)
                body = release[start:end]
                self.assertIn(lost, body)
                bad = release[:start] + body.replace(lost, "true") + release[end:]
                self.assertNotEqual(bad, release)
                self.assertTrue(wiring_errors(ci, bad, extended))
        verifier = (ROOT / "scripts/verify-macos-release.sh").read_text()
        self.assertTrue(wiring_errors(ci, release, extended, verifier.replace('"$MOUNT/Awl.app" "$ARCH"', '"$MOUNT/Awl.app"')))
        self.assertTrue(wiring_errors(ci.replace("uses: ./.github/actions/project-rust", "uses: dtolnay/rust-toolchain@stable"), release, extended))


class ProjectToolchain(unittest.TestCase):
    def test_one_project_pin_keeps_components_targets_and_all_workflows(self):
        import tomllib
        pin = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]
        self.assertRegex(pin["channel"], r"^\d+\.\d+\.\d+$")
        self.assertTrue({"clippy", "rustfmt", "llvm-tools"} <= set(pin["components"]))
        self.assertIn("wasm32-unknown-unknown", pin["targets"])
        action = (ROOT / ".github/actions/project-rust/action.yml").read_text()
        self.assertIn("rustup show active-toolchain", action)
        self.assertIn("unset RUSTUP_TOOLCHAIN", action)
        self.assertIn("scripts/project-rust.sh rustc --version", action)
        self.assertNotIn("GITHUB_PATH", action)
        helper = (ROOT / "scripts/project-rust.sh").read_text()
        self.assertIn('exec rustup run "${awl_project_rust_active%% *}" "$@"', helper)
        self.assertIn("rustup which cargo", helper)
        self.assertNotIn("rustup default", helper)
        self.assertNotIn("rustup default", action)
        self.assertIn('rustup target add "${targets[@]}"', action)
        for workflow in ["ci", "extended-verification", "release", "deploy-web"]:
            text = (ROOT / f".github/workflows/{workflow}.yml").read_text()
            self.assertIn("uses: ./.github/actions/project-rust", text)
            self.assertNotIn("uses: dtolnay/rust-toolchain", text)
        release = (ROOT / ".github/workflows/release.yml").read_text()
        self.assertIn("targets: aarch64-apple-darwin,x86_64-apple-darwin", release)
        self.assertIn("COPY Cargo.toml Cargo.lock rust-toolchain.toml ./", (ROOT / "Dockerfile.linux").read_text())
        for script in (ROOT / "scripts").glob("*.sh"):
            self.assertNotIn("toolchains/stable-aarch64-apple-darwin/bin", script.read_text())


class JourneyOracles(unittest.TestCase):
    def test_native_inputs_preserve_text_shift_and_use_linux_document_end(self):
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location("pretag_native_inputs", ROOT / "scripts/pretag-journeys.py")
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        for convention, expected in [("mac", "s-Down s-p s-t s-, S-Down s e t"),
                                     ("linux", "C-End C-p C-t C-, S-Down s e t")]:
            with patch.dict(os.environ, AWL_CONVENTION_FORCE=convention):
                self.assertEqual(module.native_keys("s-Down s-p s-t s-, S-Down s e t"), expected)
                for journey in module.JOURNEYS:
                    module.native_keys(journey.keys)
        with patch.dict(os.environ, AWL_CONVENTION_FORCE="linux"):
            with self.assertRaises(KeyError):
                module.native_keys("s-unknown")
        with patch.dict(os.environ, AWL_CONVENTION_FORCE="unknown"):
            with self.assertRaises(ValueError):
                module.native_keys("s-p")

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
