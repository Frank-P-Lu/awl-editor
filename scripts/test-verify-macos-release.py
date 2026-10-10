#!/usr/bin/env python3
"""Run the real verifier against disposable packages and fake platform commands."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
SOURCE = (ROOT / "scripts/verify-macos-release.sh").read_text()
COMMIT = "a" * 40
LAUNCH = '''  if [ "$LAUNCH" = local ]; then
    python3 "$ROOT/scripts/release-launch-smoke.py" --binary "$MOUNT/Awl.app/Contents/MacOS/awl" --local
  else
    python3 "$ROOT/scripts/release-launch-smoke.py" --binary "$MOUNT/Awl.app/Contents/MacOS/awl"
  fi'''
OLD_LAUNCH = '''  launch_args=()
  if [ "$LAUNCH" = local ]; then launch_args+=(--local); fi
  python3 "$ROOT/scripts/release-launch-smoke.py" --binary "$MOUNT/Awl.app/Contents/MacOS/awl" "${launch_args[@]}"'''


def executable(path: Path, text: str):
    path.write_text(text)
    path.chmod(0o755)


@unittest.skipUnless(sys.platform == "darwin", "requires the installed Mac Bash and PlistBuddy")
class MacVerifier(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="awl-verifier-law-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "fixture with spaces"
        scripts = self.root / "scripts"
        scripts.mkdir(parents=True)
        self.verifier = scripts / "verify-macos-release.sh"
        executable(self.verifier, SOURCE)
        (scripts / "macos-verification-receipt.py").write_text(
            (ROOT / "scripts/macos-verification-receipt.py").read_text())
        team = self.root / "assets/macos/release-team-id.txt"
        team.parent.mkdir(parents=True)
        team.write_text("SYNTHETIC\n")
        self.dist = self.root / "dist"
        self.dist.mkdir()
        # A previous run's full eight-file payload and success record are no proof
        # of completion by this invocation.
        for arch in ("arm64", "x86_64"):
            for suffix in ("dmg", "app.zip"):
                artifact = self.dist / f"awl-0.13.0-macos-{arch}.{suffix}"
                artifact.write_text("stale synthetic artifact\n")
                artifact.with_name(artifact.name + ".sha256").write_text("stale hash\n")
        self.stale_receipt = self.dist / "success.json"
        self.stale_receipt.write_text('{"success":true,"invocation":"old"}\n')
        self.fresh = self.root / "fresh caller receipt"
        self.fresh.mkdir()
        self.receipt = self.fresh / "success.json"
        self.log = self.root / "launches.jsonl"
        self.platform_log = self.root / "platform.log"
        fakebin = self.root / "bin"
        fakebin.mkdir()
        (fakebin / "bash").symlink_to("/bin/bash")
        (fakebin / "python3").symlink_to(sys.executable)
        executable(fakebin / "git", f"#!/bin/sh\nprintf '%s\\n' '{COMMIT}'\n")
        executable(fakebin / "hdiutil", '''#!/usr/bin/env python3
import os, pathlib, plistlib, sys
operation = sys.argv[1]
arch = pathlib.Path(sys.argv[-1]).name
if operation == "attach":
    arch = pathlib.Path(sys.argv[sys.argv.index("-mountpoint") + 1]).name
with open(os.environ["FIXTURE_PLATFORM_LOG"], "a") as log:
    log.write(operation + " " + arch + "\\n")
if operation == "detach" and arch == os.environ.get("FAIL_DETACH_ARCH"):
    raise SystemExit(23)
if operation == "attach":
    mount = pathlib.Path(sys.argv[sys.argv.index("-mountpoint") + 1])
    app = mount / "Awl.app/Contents"
    (app / "MacOS").mkdir(parents=True)
    (app / "MacOS/awl").write_text("synthetic executable")
    with (app / "Info.plist").open("wb") as out:
        plistlib.dump({"CFBundleShortVersionString": "0.13.0",
                      "CFBundleVersion": "42.0.0",
                      "CFBundleIdentifier": "dev.franklu.awl",
                      "AwlSourceCommit": os.environ["FIXTURE_COMMIT"]}, out)
elif sys.argv[1] not in ("verify", "detach"):
    raise SystemExit("unexpected hdiutil operation")
''')
        executable(fakebin / "ditto", '''#!/usr/bin/env python3
import os, pathlib, sys
arch = pathlib.Path(sys.argv[-2]).parent.name
with open(os.environ["FIXTURE_PLATFORM_LOG"], "a") as log:
    log.write("ditto " + arch + "\\n")
pathlib.Path(sys.argv[-1]).write_text("fresh synthetic zip")
''')
        for command in ("codesign", "xcrun", "spctl"):
            executable(fakebin / command, "#!/bin/sh\necho 'unexpected signing command' >&2\nexit 90\n")
        executable(scripts / "check-macos-release-size.sh", "#!/bin/sh\nexit 0\n")
        executable(scripts / "package-macos.sh", "#!/bin/sh\nexit 0\n")
        (scripts / "release-launch-smoke.py").write_text('''import json, os, pathlib, sys
args = sys.argv[1:]
arch = pathlib.Path(args[1]).parents[3].name
with open(os.environ["FIXTURE_LOG"], "a") as log:
    log.write(json.dumps({"arch": arch, "args": args}) + "\\n")
if arch == os.environ.get("FAIL_ARCH"):
    raise SystemExit(17)
''')
        self.env = {"PATH": f"{fakebin}:/usr/bin:/bin", "HOME": str(self.root),
                    "TMPDIR": str(self.root), "FIXTURE_COMMIT": COMMIT,
                    "FIXTURE_LOG": str(self.log), "FIXTURE_PLATFORM_LOG": str(self.platform_log),
                    "PYTHONDONTWRITEBYTECODE": "1"}
        # This intentionally simple consumer issues a *fresh* success receipt
        # only after the real verifier returns success under outer Bash -e.
        # It is a regression fixture, not the release workflow's receipt format.
        self.caller = self.root / "caller.sh"
        executable(self.caller, '''#!/bin/bash
set -e
if [ -n "${5:-}" ]; then
  "$1" "$2" 0.13.0 42.0.0 unsigned "$3" "$5"
  python3 "$6" check --receipt-dir "$5" --dist "$2" --version 0.13.0 \
    --build-version 42.0.0 --source "$FIXTURE_COMMIT" --signing unsigned --launch "$3"
else
  "$1" "$2" 0.13.0 42.0.0 unsigned "$3"
fi
printf '{"success":true}\\n' > "$4"
''')

    def run_verifier(self, context="ci", source=SOURCE, fail_arch=None, receipts=False, fail_detach_arch=None):
        executable(self.verifier, source)
        self.receipt.unlink(missing_ok=True)
        self.log.unlink(missing_ok=True)
        self.platform_log.unlink(missing_ok=True)
        env = dict(self.env)
        if fail_arch:
            env["FAIL_ARCH"] = fail_arch
        if fail_detach_arch:
            env["FAIL_DETACH_ARCH"] = fail_detach_arch
        command = ["/bin/bash", "-e", str(self.caller), str(self.verifier),
                   str(self.dist), context, str(self.receipt)]
        if receipts:
            self.protocol = self.root / "current protocol receipts"
            self.protocol.mkdir()
            command += [str(self.protocol), str(self.root / "scripts/macos-verification-receipt.py")]
        return subprocess.run(command, env=env, capture_output=True, text=True, timeout=20)

    def launches(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def assert_incomplete(self, result):
        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.receipt.exists())
        self.assertEqual(json.loads(self.stale_receipt.read_text())["invocation"], "old")

    def test_ci_and_local_arguments_and_both_architectures(self):
        for context in ("ci", "local"):
            with self.subTest(context=context):
                result = self.run_verifier(context)
                self.assertEqual(result.returncode, 0, result.stderr)
                launches = self.launches()
                self.assertEqual([item["arch"] for item in launches], ["arm64", "x86_64"])
                for item in launches:
                    self.assertEqual(item["args"][0], "--binary")
                    self.assertIn("fixture with spaces", item["args"][1])
                    self.assertEqual(item["args"][2:], ["--local"] if context == "local" else [])
                self.assertTrue(json.loads(self.receipt.read_text())["success"])
                for checksum in self.dist.glob("*.sha256"):
                    self.assertNotIn("stale", checksum.read_text())

    def test_second_architecture_failure_preserves_status_and_no_fresh_receipt(self):
        result = self.run_verifier(fail_arch="x86_64")
        self.assert_incomplete(result)
        self.assertEqual(result.returncode, 17)
        self.assertEqual([item["arch"] for item in self.launches()], ["arm64", "x86_64"])

    def test_nominal_early_success_is_not_completion(self):
        anchor = '  COMPLETED_ARCHES="$COMPLETED_ARCHES $ARCH"'
        self.assertEqual(SOURCE.count(anchor), 1)
        result = self.run_verifier(source=SOURCE.replace(anchor, anchor + '\n  exit 0'))
        self.assert_incomplete(result)
        self.assertIn("did not complete both architectures", result.stderr)
        self.assertEqual([item["arch"] for item in self.launches()], ["arm64"])

    def test_reintroduced_empty_array_cannot_authorize_receipt(self):
        self.assertEqual(SOURCE.count(LAUNCH), 1)
        result = self.run_verifier(source=SOURCE.replace(LAUNCH, OLD_LAUNCH))
        self.assert_incomplete(result)
        self.assertIn("unbound variable", result.stderr)
        self.assertEqual(self.launches(), [])

    def test_real_producer_and_outer_consumer_complete_both_architectures(self):
        result = self.run_verifier(receipts=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.receipt.exists())
        self.assertEqual({p.name for p in self.protocol.iterdir()}, {"arm64.json", "x86_64.json"})
        self.assertIn("both current architecture receipts and hashes passed", result.stdout)

    def test_failed_second_launch_leaves_one_receipt_and_no_success(self):
        result = self.run_verifier(receipts=True, fail_arch="x86_64")
        self.assert_incomplete(result)
        self.assertEqual({p.name for p in self.protocol.iterdir()}, {"arm64.json"})

    def test_outer_consumer_rejects_original_masked_exit_and_stale_payload(self):
        version = subprocess.check_output(["/bin/bash", "--version"], text=True)
        if "version 3.2." not in version:
            self.skipTest("original EXIT status masking requires Bash 3.2")
        start = SOURCE.index('  local result=$?')
        end = SOURCE.index('\n}\ntrap cleanup EXIT', start)
        original = SOURCE[:start] + '''  if [ -n "$MOUNT" ]; then hdiutil detach "$MOUNT" >/dev/null 2>&1 || true; fi
  rm -rf "$SCRATCH"''' + SOURCE[end:]
        result = self.run_verifier(source=original.replace(LAUNCH, OLD_LAUNCH), receipts=True)
        self.assert_incomplete(result)
        self.assertIn("unbound variable", result.stderr)
        self.assertIn("both current architecture receipts are required", result.stderr)
        self.assertEqual(self.launches(), [])
        self.assertEqual(list(self.protocol.iterdir()), [])

    def seed_matching_payload(self):
        # Model stale downloads whose bytes and checksums also match this run.
        for artifact in self.dist.glob("*.app.zip"):
            artifact.write_text("fresh synthetic zip")
        for artifact in self.dist.iterdir():
            if artifact.name.endswith((".dmg", ".app.zip")):
                digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
                artifact.with_name(artifact.name + ".sha256").write_text(
                    f"{digest}  {artifact.name}\n")

    def check_protocol(self):
        return subprocess.run(
            [sys.executable, str(self.root / "scripts/macos-verification-receipt.py"),
             "check", "--receipt-dir", str(self.protocol), "--dist", str(self.dist),
             "--version", "0.13.0", "--build-version", "42.0.0", "--source", COMMIT,
             "--signing", "unsigned", "--launch", "ci"],
            env=self.env, capture_output=True, text=True, timeout=20)

    def detach_failure(self, arch, source):
        self.seed_matching_payload()
        result = self.run_verifier(source=source, receipts=True, fail_detach_arch=arch)
        self.assert_incomplete(result)
        self.assertEqual(result.returncode, 23, result.stderr)
        trace = self.platform_log.read_text().splitlines()
        start = trace.index(f"ditto {arch}")
        self.assertEqual(trace[start:],
                         [f"ditto {arch}", f"detach {arch}", f"detach {arch}"])
        return {p.name for p in self.protocol.iterdir()}, self.check_protocol()

    def test_late_detach_failure_cannot_publish_completed_architecture_receipt(self):
        for arch in ("arm64", "x86_64"):
            with self.subTest(arch=arch):
                if hasattr(self, "protocol"):
                    for receipt in self.protocol.iterdir():
                        receipt.unlink()
                    self.protocol.rmdir()
                roster, checked = self.detach_failure(arch, SOURCE)
                self.assertEqual(roster, set() if arch == "arm64" else {"arm64.json"})
                self.assertNotEqual(checked.returncode, 0, checked.stdout)
                self.assertIn("both current architecture receipts are required", checked.stderr)

    def test_early_receipt_mutant_exposes_late_detach_stage_boundary(self):
        start = SOURCE.index('  if [ -n "$RECEIPT_DIR" ]; then', SOURCE.index('for ARCH'))
        end = SOURCE.index('\n  COMPLETED_ARCHES=', start)
        block = SOURCE[start:end]
        mutant = SOURCE[:start] + SOURCE[end:]
        anchor = '  hdiutil detach "$MOUNT"'
        self.assertEqual(mutant.count(anchor), 1)
        mutant = mutant.replace(anchor, block + "\n" + anchor)
        for arch in ("arm64", "x86_64"):
            with self.subTest(arch=arch):
                if hasattr(self, "protocol"):
                    for receipt in self.protocol.iterdir():
                        receipt.unlink()
                    self.protocol.rmdir()
                roster, checked = self.detach_failure(arch, mutant)
                self.assertIn(f"{arch}.json", roster, "early receipt must violate the stage law")
                if arch == "x86_64":
                    self.assertEqual(roster, {"arm64.json", "x86_64.json"})
                    self.assertEqual(checked.returncode, 0, checked.stderr)
                else:
                    self.assertEqual(roster, {"arm64.json"})
                    self.assertNotEqual(checked.returncode, 0, checked.stdout)
        # Producer failure still stops outer Bash -e; this mutant demonstrates
        # premature receipt publication, not a real workflow upload.

    def run_workflow_step(self, name, source=SOURCE):
        lines = (ROOT / ".github/workflows/release.yml").read_text().splitlines()
        start = next(i for i, line in enumerate(lines) if line.strip() == f"- name: {name}")
        run = next(i for i in range(start, len(lines)) if lines[i] == "        run: |")
        body = []
        for line in lines[run + 1:]:
            if not line.startswith("          "):
                break
            body.append(line[10:])
        self.assertTrue(body)
        executable(self.verifier, source)
        executable(self.caller, "#!/bin/bash\n" + "\n".join(body) +
                   '\nprintf success > "$FIXTURE_SUCCESS"\n')
        link = self.root / "dist-mac"
        if not link.exists():
            link.symlink_to(self.dist, target_is_directory=True)
        env = dict(self.env, VERSION="0.13.0", BUILD_VERSION="42.0.0",
                   SIGN_MACOS="false", RUNNER_TEMP=str(self.root), FIXTURE_SUCCESS=str(self.receipt))
        # The staged route's signature checks are mocked, never real signing.
        for command in ("codesign", "xcrun", "spctl"):
            executable(self.root / "bin" / command, "#!/bin/sh\nexit 0\n")
        (self.root / "scripts/verify-macos-signature.py").write_text("# synthetic signature verdict\n")
        self.receipt.unlink(missing_ok=True)
        return subprocess.run(["/bin/bash", "-e", "-o", "pipefail", str(self.caller)],
                              cwd=self.root, env=env, capture_output=True, text=True, timeout=20)

    def test_actual_workflow_steps_accept_complete_current_verification(self):
        for name in ("Validate and checksum macOS artifacts", "Independently validate actual staged DMGs"):
            with self.subTest(name=name):
                result = self.run_workflow_step(name)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(self.receipt.exists())
                self.assertIn("both current architecture receipts and hashes passed", result.stdout)
                self.assertEqual(list(self.root.glob("awl-mac-verification.*")), [])

    def test_actual_workflow_steps_reject_original_masked_exit(self):
        version = subprocess.check_output(["/bin/bash", "--version"], text=True)
        if "version 3.2." not in version:
            self.skipTest("original EXIT status masking requires Bash 3.2")
        start = SOURCE.index('  local result=$?')
        end = SOURCE.index('\n}\ntrap cleanup EXIT', start)
        original = SOURCE[:start] + '''  if [ -n "$MOUNT" ]; then hdiutil detach "$MOUNT" >/dev/null 2>&1 || true; fi
  rm -rf "$SCRATCH"''' + SOURCE[end:]
        for name in ("Validate and checksum macOS artifacts", "Independently validate actual staged DMGs"):
            with self.subTest(name=name):
                result = self.run_workflow_step(name, original.replace(LAUNCH, OLD_LAUNCH))
                self.assert_incomplete(result)
                self.assertIn("unbound variable", result.stderr)
                self.assertIn("both current architecture receipts are required", result.stderr)
                self.assertEqual(list(self.root.glob("awl-mac-verification.*")), [])

    def test_original_bug_red_control_on_installed_bash32(self):
        version = subprocess.check_output(["/bin/bash", "--version"], text=True)
        if "version 3.2." not in version:
            self.skipTest("original EXIT status masking requires Bash 3.2")
        # Restore the original trap and array expansion in a disposable copy.
        # This must demonstrate the exact false-success mechanism; if it stops
        # reproducing, the regression no longer witnesses the reported defect.
        start = SOURCE.index('  local result=$?')
        end = SOURCE.index('\n}\ntrap cleanup EXIT', start)
        original = SOURCE[:start] + '''  if [ -n "$MOUNT" ]; then hdiutil detach "$MOUNT" >/dev/null 2>&1 || true; fi
  rm -rf "$SCRATCH"''' + SOURCE[end:]
        result = self.run_verifier(source=original.replace(LAUNCH, OLD_LAUNCH))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("unbound variable", result.stderr)
        self.assertEqual(self.launches(), [])
        self.assertTrue(self.receipt.exists(), "old bug must falsely authorize fresh receipt")


if __name__ == "__main__":
    unittest.main(verbosity=2)
