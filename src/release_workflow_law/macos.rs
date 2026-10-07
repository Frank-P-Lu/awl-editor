//! Laws for the architecture-specific macOS release packages.

use super::scratch;

pub(super) fn apple_bundle_versions_are_proved(plan: &str, mac: &str, packager: &str) -> bool {
    let validation_precedes_plist = packager
        .find("validate_macos_versions \"$AWL_VERSION\" \"$AWL_BUILD_VERSION\"")
        .zip(packager.find("cat > \"$CONTENTS/Info.plist\""))
        .is_some_and(|(validation, plist)| validation < plist);
    plan.contains("uses: actions/checkout@v7")
        && plan.contains("tomllib.loads")
        && plan.contains("[\"package\"][\"version\"]")
        && !plan.contains("0.0.0-dryrun")
        && mac.contains("AWL_BUILD_VERSION: ${{ github.run_number }}.0.0")
        && mac.contains("BUILD_VERSION: ${{ github.run_number }}.0.0")
        && mac.contains("Print :CFBundleShortVersionString")
        && mac.contains("Print :CFBundleVersion")
        && mac.contains("[ \"$SHORT_VERSION\" = \"$VERSION\" ]")
        && mac.contains("[ \"$ACTUAL_BUILD_VERSION\" = \"$BUILD_VERSION\" ]")
        && packager.contains("AWL_BUILD_VERSION=\"${AWL_BUILD_VERSION:-1.0.0}\"")
        && packager.contains("validate_macos_versions() {")
        && packager.contains("^[0-9]+\\.[0-9]+\\.[0-9]+$")
        && packager.contains("^[1-9][0-9]{0,3}\\.[0-9]{1,2}\\.[0-9]{1,2}$")
        && packager.contains("<string>${AWL_VERSION}</string>")
        && packager.contains("<string>${AWL_BUILD_VERSION}</string>")
        && validation_precedes_plist
}

#[test]
fn apple_bundle_version_law_rejects_each_regression() {
    let _guard = crate::testlock::serial();
    let workflow = super::without_comments(&super::read(".github/workflows/release.yml"));
    let packager = super::without_comments(&super::read("scripts/package-macos.sh"));
    let audit = |workflow: &str, packager: &str| {
        apple_bundle_versions_are_proved(
            super::job(workflow, "plan"),
            super::job(workflow, "mac"),
            packager,
        )
    };
    assert!(audit(&workflow, &packager));

    for (subject, replacement) in [
        ("tomllib.loads", "print_dryrun_version"),
        (
            "AWL_BUILD_VERSION: ${{ github.run_number }}.0.0",
            "AWL_BUILD_VERSION: ${{ needs.plan.outputs.version }}",
        ),
        (
            "[ \"$ACTUAL_BUILD_VERSION\" = \"$BUILD_VERSION\" ]",
            "[ -n \"$ACTUAL_BUILD_VERSION\" ]",
        ),
    ] {
        assert!(
            workflow.contains(subject),
            "mutation subject missing: {subject}"
        );
        let broken = workflow.replacen(subject, replacement, 1);
        assert!(
            !audit(&broken, &packager),
            "workflow mutation escaped: {subject}"
        );
    }

    for (subject, replacement) in [
        ("^[1-9][0-9]{0,3}\\.[0-9]{1,2}\\.[0-9]{1,2}$", "^[0-9.]+$"),
        (
            "<string>${AWL_BUILD_VERSION}</string>",
            "<string>${AWL_VERSION}</string>",
        ),
    ] {
        assert!(
            packager.contains(subject),
            "mutation subject missing: {subject}"
        );
        let broken = packager.replacen(subject, replacement, 1);
        assert!(
            !audit(&workflow, &broken),
            "packager mutation escaped: {subject}"
        );
    }
}

pub(super) fn native_architecture_split_is_proved(mac: &str, packager: &str) -> bool {
    for needle in [
        "target/aarch64-apple-darwin/release/awl dist-mac/arm64",
        "target/x86_64-apple-darwin/release/awl dist-mac/x86_64",
        "scripts/package-macos.sh --verify \"$MOUNT/Awl.app\" \"$ARCH\"",
        "awl-$VERSION-macos-$ARCH.dmg",
    ] {
        if !mac.contains(needle) {
            return false;
        }
    }
    packager.contains("lipo -archs \"$binary\"")
        && packager.contains("arm64|x86_64)")
        && packager.contains("if [ -n \"$expected\" ] && [ \"$archs\" != \"$expected\" ]")
        && !mac.contains("lipo -create")
        && !mac.contains("macos-universal")
}

#[cfg(unix)]
#[test]
fn macos_packager_accepts_only_one_honestly_named_native_architecture() {
    use std::os::unix::fs::PermissionsExt;

    let _guard = crate::testlock::serial();
    let dir = scratch("native-arch");
    let tools = dir.join("tools");
    std::fs::create_dir_all(&tools).expect("create fake tool directory");
    let lipo = tools.join("lipo");
    std::fs::write(
        &lipo,
        b"#!/bin/sh\n[ \"$1\" = -archs ] || exit 2\ncat \"$2\"\n",
    )
    .expect("write fake lipo");
    std::fs::set_permissions(&lipo, std::fs::Permissions::from_mode(0o755))
        .expect("make fake lipo executable");
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![tools.clone()];
    paths.extend(std::env::split_paths(&old_path));
    let path = std::env::join_paths(paths).expect("construct test PATH");
    let binary = dir.join("awl");
    let run = |roster: &str, expected: Option<&str>| {
        std::fs::write(&binary, roster).expect("write fake architecture roster");
        let mut command =
            std::process::Command::new(super::root().join("scripts/package-macos.sh"));
        command.arg("--print-arch").arg(&binary).env("PATH", &path);
        if let Some(expected) = expected {
            command.arg(expected);
        }
        command.output().expect("architecture query must run")
    };

    for arch in ["arm64", "x86_64"] {
        let output = run(arch, Some(arch));
        assert!(
            output.status.success(),
            "{arch} must be accepted: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), arch);
    }
    for (roster, expected) in [
        ("arm64 x86_64", None),
        ("arm64", Some("x86_64")),
        ("ppc", None),
    ] {
        let output = run(roster, expected);
        assert!(
            !output.status.success(),
            "roster {roster:?} with expected {expected:?} must be rejected"
        );
    }
}
