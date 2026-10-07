//! Laws for the architecture-specific macOS release packages.

use super::scratch;

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
