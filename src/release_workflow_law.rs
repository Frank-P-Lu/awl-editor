//! Laws for the public release workflow.
//!
//! The signed macOS path lives in GitHub Actions and shell, outside Rust's
//! type system. These tests keep its safety properties enrolled in the normal
//! suite on every host and deliberately mutate each headline subject so a
//! commented-out or disconnected command cannot satisfy the audit.

#![cfg(all(test, not(target_arch = "wasm32")))]

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

fn without_comments(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn job<'a>(workflow: &'a str, name: &str) -> &'a str {
    let start_marker = format!("  {name}:\n");
    let start = workflow
        .find(&start_marker)
        .unwrap_or_else(|| panic!("release workflow has no {name:?} job"));
    let rest = &workflow[start + start_marker.len()..];
    let mut end = rest.len();
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.starts_with("  ") && !line.starts_with("    ") && line.trim_end().ends_with(':') {
            end = offset;
            break;
        }
        offset += line.len();
    }
    &rest[..end]
}

fn step<'a>(job: &'a str, name: &str) -> &'a str {
    let start_marker = format!("      - name: {name}\n");
    let start = job
        .find(&start_marker)
        .unwrap_or_else(|| panic!("release workflow job has no {name:?} step"));
    let rest = &job[start + start_marker.len()..];
    let end = rest.find("\n      - ").unwrap_or(rest.len());
    &rest[..end]
}

fn release_audit(
    workflow: &str,
    packager: &str,
    size_check: &str,
    preparer: &str,
) -> Vec<&'static str> {
    let workflow = without_comments(workflow);
    let packager = without_comments(packager);
    let mac = job(&workflow, "mac");
    let plan = job(&workflow, "plan");
    let prepare = job(&workflow, "prepare-release");
    let publish = job(&workflow, "publish");
    let signing_check = step(mac, "Check signing secrets");
    let mut failures = Vec::new();

    if mac.lines().any(|line| line.starts_with("    if:")) {
        failures.push("mac-tag-enrolment");
    }
    if !plan.contains("sign_macos: ${{ steps.plan.outputs.sign_macos }}")
        || !plan.contains("echo \"sign_macos=true\"")
        || !plan.contains("credentialed_macos_rehearsal")
        || !plan.contains("SIGN_MACOS=false")
    {
        failures.push("signing-plan");
    }
    if !signing_check.contains("if [ \"$SIGN_MACOS\" = \"true\" ]; then")
        || !signing_check.contains("macOS signing is required")
        || !signing_check.contains("exit 1")
    {
        failures.push("missing-credentials-fail-closed");
    }
    for (needle, label) in [
        (
            "codesign --force --options runtime --timestamp",
            "developer-id-signing",
        ),
        ("xcrun notarytool submit", "notarization"),
        ("xcrun stapler staple", "staple"),
        ("hdiutil verify", "dmg-verification"),
    ] {
        if !mac.contains(needle) {
            failures.push(label);
        }
    }
    for (needle, minimum, label) in [
        ("xcrun stapler validate", 2, "staple-validation"),
        ("spctl --assess --type execute", 2, "gatekeeper"),
        ("lipo -verify_arch arm64 x86_64", 2, "universal-binary"),
    ] {
        if mac.matches(needle).count() < minimum {
            failures.push(label);
        }
    }
    if !mac.contains("awl-${{ needs.plan.outputs.version }}-macos-universal.dmg")
        || !mac.contains("$DMG.sha256")
    {
        failures.push("versioned-mac-artifact");
    }
    if !prepare.contains("needs: [plan, mac, linux]") {
        failures.push("publication-depends-on-mac");
    }
    if prepare.lines().any(|line| line.starts_with("    if:"))
        || !prepare.contains("name: awl-linux")
        || !prepare.contains("name: awl-macos")
        || !prepare.contains("scripts/prepare-release-payload.sh")
        || !prepare.contains("name: awl-release-payload")
    {
        failures.push("dry-run-prepares-public-payload");
    }
    if !publish.contains("needs: [plan, prepare-release]")
        || !publish.contains("name: awl-release-payload")
        || !publish.contains("sha256sum -c SHA256SUMS")
        || !publish.contains("release/awl-${{ needs.plan.outputs.version }}-macos-universal.dmg")
    {
        failures.push("publish-checksum-attachment-parity");
    }
    if publish.contains("macos-universal.app.zip") {
        failures.push("public-app-zip");
    }
    if !publish.contains("if: needs.plan.outputs.is_release == 'true'")
        || !plan.contains("github.event_name }}\" = \"push\"")
        || !plan.contains("refs/tags/v")
    {
        failures.push("manual-run-cannot-publish");
    }
    if packager.matches("create_dmg() {").count() != 1
        || packager.matches("create_dmg \"").count() != 2
        || !packager.contains("--dmg-only) DMG_ONLY=1")
        || !mac.contains("package-macos.sh --dmg-only")
    {
        failures.push("one-dmg-owner");
    }
    if !mac.contains("scripts/check-macos-release-size.sh")
        || !size_check.contains("MAX_BYTES=50000000")
        || !size_check.contains("[ \"$DMG_BYTES\" -ge \"$MAX_BYTES\" ]")
        || !size_check.contains("APP_ZIP_BYTES")
        || size_check.contains("[ \"$APP_ZIP_BYTES\" -ge")
        || size_check.contains("[ \"$APP_ZIP_BYTES\" -gt")
    {
        failures.push("public-dmg-size-boundary");
    }
    if !preparer.contains("$LINUX_DIR/$TARBALL")
        || !preparer.contains("$LINUX_DIR/$APPIMAGE")
        || !preparer.contains("$MAC_DIR/$DMG")
        || !preparer.contains("\"$TARBALL\" \"$APPIMAGE\" \"$DMG\" > SHA256SUMS")
        || !preparer.contains("\"${SHA256[@]}\" -c SHA256SUMS")
        || preparer.contains("app.zip\" \"$OUT_DIR")
    {
        failures.push("prepared-payload-layout");
    }
    failures
}

fn sources() -> (String, String, String, String) {
    (
        read(".github/workflows/release.yml"),
        read("scripts/package-macos.sh"),
        read("scripts/check-macos-release-size.sh"),
        read("scripts/prepare-release-payload.sh"),
    )
}

#[test]
fn signed_macos_publication_is_fail_closed() {
    let _guard = crate::testlock::serial();
    let (workflow, packager, size_check, preparer) = sources();
    let failures = release_audit(&workflow, &packager, &size_check, &preparer);
    assert!(failures.is_empty(), "release workflow audit: {failures:?}");
}

#[test]
fn release_audit_rejects_each_headline_regression() {
    let _guard = crate::testlock::serial();
    let (workflow, packager, size_check, preparer) = sources();
    let mutations = [
        (
            concat!(
                "echo \"::error::macOS signing is required, ",
                "but these credentials are missing:$missing\"\n              exit 1"
            ),
            concat!(
                "echo \"::error::macOS signing is required, ",
                "but these credentials are missing:$missing\"\n              true"
            ),
            "missing-credentials-fail-closed",
        ),
        (
            "needs: [plan, mac, linux]",
            "needs: [plan, linux]",
            "publication-depends-on-mac",
        ),
        (
            "    needs: [plan, mac, linux]\n    runs-on: ubuntu-latest",
            concat!(
                "    needs: [plan, mac, linux]\n",
                "    if: needs.plan.outputs.is_release == 'true'\n",
                "    runs-on: ubuntu-latest"
            ),
            "dry-run-prepares-public-payload",
        ),
        (
            "xcrun stapler validate",
            "xcrun stapler check",
            "staple-validation",
        ),
        (
            "spctl --assess --type execute",
            "true # spctl removed",
            "gatekeeper",
        ),
        (
            "release/awl-${{ needs.plan.outputs.version }}-macos-universal.dmg",
            "release/Awl.dmg",
            "publish-checksum-attachment-parity",
        ),
    ];
    for (subject, replacement, expected) in mutations {
        assert!(
            workflow.contains(subject),
            "mutation subject missing: {subject}"
        );
        let broken = workflow.replacen(subject, replacement, 1);
        let failures = release_audit(&broken, &packager, &size_check, &preparer);
        assert!(
            failures.contains(&expected),
            "mutation {subject:?} did not fail as {expected:?}: {failures:?}"
        );
    }

    let subject = "--dmg-only) DMG_ONLY=1";
    assert!(
        packager.contains(subject),
        "mutation subject missing: {subject}"
    );
    let broken_packager = packager.replacen(subject, "--dmg-copy) DMG_ONLY=1", 1);
    let failures = release_audit(&workflow, &broken_packager, &size_check, &preparer);
    assert!(
        failures.contains(&"one-dmg-owner"),
        "packager mutation did not fail by owner: {failures:?}"
    );

    for (subject, replacement) in [
        ("MAX_BYTES=50000000", "MAX_BYTES=50000001"),
        (
            "[ \"$DMG_BYTES\" -ge \"$MAX_BYTES\" ]",
            "[ \"$DMG_BYTES\" -gt \"$MAX_BYTES\" ]",
        ),
    ] {
        let broken = size_check.replacen(subject, replacement, 1);
        let failures = release_audit(&workflow, &packager, &broken, &preparer);
        assert!(
            failures.contains(&"public-dmg-size-boundary"),
            "size mutation {subject:?} did not fail by boundary: {failures:?}"
        );
    }

    let subject = "\"$TARBALL\" \"$APPIMAGE\" \"$DMG\" > SHA256SUMS";
    let broken = preparer.replacen(subject, "\"$TARBALL\" \"$APPIMAGE\" > SHA256SUMS", 1);
    let failures = release_audit(&workflow, &packager, &size_check, &broken);
    assert!(
        failures.contains(&"prepared-payload-layout"),
        "payload mutation did not fail by layout: {failures:?}"
    );
}

fn scratch(name: &str) -> crate::testscratch::ScratchDir {
    crate::testscratch::ScratchDir::new(std::env::temp_dir().join(format!(
        "awl-release-law-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )))
}

fn write_checksum(dir: &Path, name: &str) {
    let output = std::process::Command::new("shasum")
        .args(["-a", "256", name])
        .current_dir(dir)
        .output()
        .expect("shasum must run");
    assert!(output.status.success(), "shasum failed: {output:?}");
    std::fs::write(dir.join(format!("{name}.sha256")), output.stdout).expect("write checksum");
}

#[test]
fn only_the_public_dmg_has_a_hard_size_limit() {
    let _guard = crate::testlock::serial();
    let dir = scratch("size");
    std::fs::create_dir_all(&*dir).expect("create scratch");
    let dmg = dir.join("public.dmg");
    let zip = dir.join("workflow-only.app.zip");
    std::fs::File::create(&dmg)
        .and_then(|file| file.set_len(49_999_999))
        .expect("create under-limit DMG");
    std::fs::File::create(&zip)
        .and_then(|file| file.set_len(50_000_001))
        .expect("create oversized diagnostic ZIP");

    let accepted = std::process::Command::new(root().join("scripts/check-macos-release-size.sh"))
        .arg(&dmg)
        .arg(&zip)
        .output()
        .expect("size check must run");
    assert!(
        accepted.status.success(),
        "an oversized nonpublic ZIP must not reject an under-limit DMG: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );

    std::fs::File::create(&dmg)
        .and_then(|file| file.set_len(50_000_000))
        .expect("create boundary DMG");
    let rejected = std::process::Command::new(root().join("scripts/check-macos-release-size.sh"))
        .arg(&dmg)
        .arg(&zip)
        .output()
        .expect("size check must run");
    assert!(
        !rejected.status.success(),
        "a DMG at exactly 50,000,000 bytes must fail the strict under-limit law"
    );
}

#[test]
fn dry_run_preparation_builds_the_exact_public_payload() {
    let _guard = crate::testlock::serial();
    let dir = scratch("payload");
    let linux = dir.join("linux");
    let mac = dir.join("mac");
    let output = dir.join("release");
    std::fs::create_dir_all(&linux).expect("create linux input");
    std::fs::create_dir_all(&mac).expect("create mac input");
    let version = "9.8.7-test";
    let tarball = format!("awl-{version}-linux-x86_64.tar.gz");
    let appimage = format!("awl-{version}-linux-x86_64.AppImage");
    let dmg = format!("awl-{version}-macos-universal.dmg");
    std::fs::write(linux.join(&tarball), b"tarball").expect("write tarball");
    std::fs::write(linux.join(&appimage), b"appimage").expect("write AppImage");
    std::fs::write(mac.join(&dmg), b"dmg").expect("write DMG");
    write_checksum(&linux, &tarball);
    write_checksum(&linux, &appimage);
    write_checksum(&mac, &dmg);

    // This is intentionally larger than the public-download cap. Preparation
    // must ignore it because the app ZIP is a workflow receipt, not a release.
    std::fs::File::create(mac.join(format!("awl-{version}-macos-universal.app.zip")))
        .and_then(|file| file.set_len(50_000_001))
        .expect("create diagnostic ZIP");

    let prepared = std::process::Command::new(root().join("scripts/prepare-release-payload.sh"))
        .args([&linux, &mac, &output])
        .arg(version)
        .output()
        .expect("payload preparation must run");
    assert!(
        prepared.status.success(),
        "fake artifact preparation failed: {}",
        String::from_utf8_lossy(&prepared.stderr)
    );

    let mut names: Vec<_> = std::fs::read_dir(&output)
        .expect("read prepared output")
        .map(|entry| entry.expect("output entry").file_name())
        .collect();
    names.sort();
    let mut expected = vec![
        std::ffi::OsString::from("SHA256SUMS"),
        std::ffi::OsString::from(appimage),
        std::ffi::OsString::from(dmg),
        std::ffi::OsString::from(tarball),
    ];
    expected.sort();
    assert_eq!(
        names, expected,
        "prepared payload must contain exactly three public downloads plus SHA256SUMS"
    );
}
