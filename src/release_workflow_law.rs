//! Laws for the public release workflow.
//!
//! The signed macOS path lives in GitHub Actions and shell, outside Rust's
//! type system. These tests keep its safety properties enrolled in the normal
//! suite on every host and deliberately mutate each headline subject so a
//! commented-out or disconnected command cannot satisfy the audit.

#![cfg(all(test, not(target_arch = "wasm32")))]

use std::path::PathBuf;

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

fn release_audit(workflow: &str, packager: &str) -> Vec<&'static str> {
    let workflow = without_comments(workflow);
    let packager = without_comments(packager);
    let mac = job(&workflow, "mac");
    let plan = job(&workflow, "plan");
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
        ("MAX_BYTES=50000000", "download-size-cap"),
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
    if !publish.contains("needs: [plan, mac, linux]") {
        failures.push("publication-depends-on-mac");
    }
    if !publish.contains("name: awl-macos")
        || !publish.contains("mac/$DMG")
        || !publish.contains("sha256sum \"$TARBALL\" \"$APPIMAGE\" \"$DMG\"")
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
    failures
}

#[test]
fn signed_macos_publication_is_fail_closed() {
    let _guard = crate::testlock::serial();
    let failures = release_audit(
        &read(".github/workflows/release.yml"),
        &read("scripts/package-macos.sh"),
    );
    assert!(failures.is_empty(), "release workflow audit: {failures:?}");
}

#[test]
fn release_audit_rejects_each_headline_regression() {
    let _guard = crate::testlock::serial();
    let workflow = read(".github/workflows/release.yml");
    let packager = read("scripts/package-macos.sh");
    let mutations = [
        (
            "echo \"::error::macOS signing is required, but these credentials are missing:$missing\"\n              exit 1",
            "echo \"::error::macOS signing is required, but these credentials are missing:$missing\"\n              true",
            "missing-credentials-fail-closed",
        ),
        (
            "needs: [plan, mac, linux]",
            "needs: [plan, linux]",
            "publication-depends-on-mac",
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
            "MAX_BYTES=50000000",
            "MAX_BYTES=50000001",
            "download-size-cap",
        ),
        (
            "sha256sum \"$TARBALL\" \"$APPIMAGE\" \"$DMG\"",
            "sha256sum \"$TARBALL\" \"$APPIMAGE\"",
            "publish-checksum-attachment-parity",
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
        let failures = release_audit(&broken, &packager);
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
    let failures = release_audit(&workflow, &broken_packager);
    assert!(
        failures.contains(&"one-dmg-owner"),
        "packager mutation did not fail by owner: {failures:?}"
    );
}
