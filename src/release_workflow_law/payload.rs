//! Laws for the exact public payload selected by release preparation.

use super::{read, without_comments};

pub(super) fn audit(preparer: &str) -> Vec<&'static str> {
    let preparer = without_comments(preparer);
    let mut failures = Vec::new();
    let manifest = "\"$TARBALL\" \"$APPIMAGE\" \"$DMG_ARM64\" \"$DMG_X86_64\" > SHA256SUMS";
    if !preparer.contains("$LINUX_DIR/$TARBALL")
        || !preparer.contains("$LINUX_DIR/$APPIMAGE")
        || !preparer.contains("$MAC_DIR/$DMG_ARM64")
        || !preparer.contains("$MAC_DIR/$DMG_X86_64")
        || !preparer.contains(manifest)
        || !preparer.contains("\"${SHA256[@]}\" -c SHA256SUMS")
        || preparer.contains("app.zip\" \"$OUT_DIR")
    {
        failures.push("prepared-payload-layout");
    }
    let size_roster = preparer
        .split_once("for public_file in")
        .and_then(|(_, rest)| rest.split_once("; do"))
        .map(|(roster, _)| roster)
        .unwrap_or_default();
    if !preparer.contains("MAX_BYTES=50000000")
        || ![
            "$LINUX_DIR/$TARBALL",
            "$LINUX_DIR/$APPIMAGE",
            "$MAC_DIR/$DMG_ARM64",
            "$MAC_DIR/$DMG_X86_64",
        ]
        .iter()
        .all(|name| size_roster.contains(name))
        || !preparer.contains("[ \"$public_bytes\" -ge \"$MAX_BYTES\" ]")
        || !preparer.contains("every public download must be under")
    {
        failures.push("all-public-download-size-boundary");
    }
    failures
}

#[test]
fn prepared_payload_audit_rejects_size_and_layout_regressions() {
    let _guard = crate::testlock::serial();
    let preparer = read("scripts/prepare-release-payload.sh");
    for (subject, replacement) in [
        ("MAX_BYTES=50000000", "MAX_BYTES=50000001"),
        (
            "[ \"$public_bytes\" -ge \"$MAX_BYTES\" ]",
            "[ \"$public_bytes\" -gt \"$MAX_BYTES\" ]",
        ),
        (
            "  \"$MAC_DIR/$DMG_X86_64\"; do",
            "  \"$MAC_DIR/$DMG_ARM64\"; do",
        ),
    ] {
        assert!(
            preparer.contains(subject),
            "mutation subject missing: {subject}"
        );
        let broken = preparer.replacen(subject, replacement, 1);
        let failures = audit(&broken);
        assert!(
            failures.contains(&"all-public-download-size-boundary"),
            "public size mutation {subject:?} escaped the audit: {failures:?}"
        );
    }

    let subject = "\"$TARBALL\" \"$APPIMAGE\" \"$DMG_ARM64\" \"$DMG_X86_64\" > SHA256SUMS";
    let broken = preparer.replacen(
        subject,
        "\"$TARBALL\" \"$APPIMAGE\" \"$DMG_ARM64\" > SHA256SUMS",
        1,
    );
    let failures = audit(&broken);
    assert!(
        failures.contains(&"prepared-payload-layout"),
        "payload mutation did not fail by layout: {failures:?}"
    );
}
