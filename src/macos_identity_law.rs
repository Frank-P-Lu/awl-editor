//! src/macos_identity_law.rs — the macOS PRODUCT-IDENTITY laws.
//!
//! macOS reads a live app's product identity out of its BUNDLE, not out of the
//! running process, so awl's identity contract lives in shell — the packaging
//! script's `Info.plist` writer and its `verify_bundle_identity`. Shell has no
//! type system and no compiler to keep it honest, and the surfaces it governs
//! (the menu bar, Stage Manager, ⌘-Tab, Finder) are all live-only: no test in
//! this crate can render a menu bar. These laws are the next best thing — they
//! assert the STRUCTURE of that contract, so the ways it has actually been
//! observed to break go red here instead of silently on someone's desktop.
//!
//! Measured on 2026-07-29, bare `target/release/awl` versus the
//! bundle `scripts/dev-app.sh` assembles:
//!
//! | surface | bare binary | dev bundle |
//! |---|---|---|
//! | menu bar | `awl` | `Awl` |
//! | Stage Manager | no icon at all | the canonical Awl icon |
//! | ⌘-Tab name | `awl` | `Awl` |
//! | ⌘-Tab / Dock icon | the active world | the active world |
//!
//! The bare binary's menu-bar name and Stage Manager icon are NOT fixable
//! without a bundle: AppKit takes the application-menu title from the main
//! bundle's `CFBundleName` and falls back to the process name when there is no
//! `Info.plist`, and Stage Manager resolves its icon through LaunchServices.
//! Neither consults anything the process can set at runtime, and the only way
//! to force them would be to spoof the process title — which lies about what is
//! running. The supported answer is the bundle; see `docs/platform.md`.

// Not wasm: these laws read the repo's shell scripts off a real filesystem, and
// the browser build has neither one. They are NOT gated to macOS, though — the
// contract they guard is edited from any host, and a Linux CI run should still
// catch a packaging script that drifts.
#![cfg(all(test, not(target_arch = "wasm32")))]

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?} is unreadable: {e}"))
}

/// The DISPLAYED name is capitalised and the TYPED name is lowercase, and the
/// packaging script writes both. Sweeping the plist keys rather than spot-
/// checking one: the bug this guards is an edit that "tidies up" the casing to
/// match the crate name, which would rename the app everywhere macOS shows it.
#[test]
fn the_bundle_plist_capitalises_the_product_and_keeps_the_command_lowercase() {
    let script = read("scripts/package-macos.sh");
    for (key, want) in [
        ("CFBundleName", "Awl"),
        ("CFBundleDisplayName", "Awl"),
        ("CFBundleExecutable", "awl"),
        ("CFBundleIconFile", "Awl.icns"),
    ] {
        let block = format!("<key>{key}</key>\n  <string>{want}</string>");
        assert!(
            script.contains(&block),
            "scripts/package-macos.sh must write {key} = {want:?}. The displayed \
             product name is capitalised everywhere macOS shows it; the \
             executable — and so the command a person types — stays lowercase \
             `awl`. Merging those two is what this law exists to catch."
        );
    }
}

/// ONE owner for the identity contract. The assembly path and the `--verify`
/// gate must call the same function, or the gate starts checking something the
/// build does not (and the build ships something the gate never saw).
#[test]
fn the_identity_contract_has_exactly_one_owner() {
    let script = read("scripts/package-macos.sh");
    assert_eq!(
        script.matches("verify_bundle_identity() {").count(),
        1,
        "scripts/package-macos.sh must define verify_bundle_identity exactly once"
    );
    assert!(
        script.matches("verify_bundle_identity \"").count() >= 2,
        "both the assembly path and --verify must route through \
         verify_bundle_identity; found {} call site(s)",
        script.matches("verify_bundle_identity \"").count()
    );
    // The bypass this replaced: a second, inline identity assertion that could
    // drift from the shared one.
    let inline = script.matches("PlistBuddy").count();
    assert_eq!(
        inline, 1,
        "PlistBuddy must be reached only from inside verify_bundle_identity \
         ({inline} occurrences) — a second inline plist check is the drift this \
         law exists to prevent"
    );
}

/// THE NON-OBVIOUS HALF, and the one no other test can see: assembling the
/// bundle is enough for the menu bar, but Stage Manager showed the GENERIC
/// blueprint tile until the bundle was registered with LaunchServices. A
/// locally built `.app` in a build directory is not somewhere LS has looked.
///
/// Dropping the `lsregister` call would silently regress Stage Manager to the
/// exact live-only defect, and nothing else in this repo would
/// notice — the surface is live-only and the script would still exit 0.
#[test]
fn the_dev_launch_registers_the_bundle_with_launchservices() {
    let script = read("scripts/dev-app.sh");
    assert!(
        script.contains("lsregister"),
        "scripts/dev-app.sh must register the assembled bundle with \
         LaunchServices. Without it Stage Manager shows the generic blueprint \
         tile instead of the Awl icon."
    );
    assert!(
        script.contains("-f \"$APP\""),
        "the LaunchServices registration must force-register the bundle it just \
         built (`lsregister -f \"$APP\"`)"
    );
}

/// The supported development launch goes through the SAME canonical packaging
/// metadata a release does — one Info.plist writer, one committed icon. A dev
/// script that grew its own plist would be a second source of product identity,
/// free to drift from the one that ships.
#[test]
fn the_dev_launch_reuses_the_release_packaging_metadata() {
    let script = read("scripts/dev-app.sh");
    assert!(
        script.contains("package-macos.sh"),
        "scripts/dev-app.sh must assemble through scripts/package-macos.sh"
    );
    for forbidden in ["<key>CFBundle", "Info.plist\" <<", "PlistBuddy"] {
        assert!(
            !script.contains(forbidden),
            "scripts/dev-app.sh must not write or read bundle metadata itself \
             (found {forbidden:?}) — package-macos.sh owns it"
        );
    }
}

/// The bare-binary limitation is DOCUMENTED, not silently advertised as
/// equivalent. If macOS requires a bundle
/// for a surface, the normal dev script uses that bundle and the limitation is
/// explicit.
#[test]
fn the_bare_binary_limitation_is_written_down() {
    let doc = read("docs/platform.md");
    assert!(
        doc.contains("dev-app.sh"),
        "docs/platform.md must name the supported macOS development launch"
    );
    for needle in ["Stage Manager", "menu bar"] {
        assert!(
            doc.contains(needle),
            "docs/platform.md must state which surfaces a bare binary cannot \
             satisfy (missing {needle:?})"
        );
    }
}

/// The canonical bundle icon the plist names is actually committed, and it is
/// the one `app_icon` calls canonical. Cheap, and it fails loudly if the asset
/// is ever moved without the packaging script following.
///
/// macOS-only because `app_icon` itself is: naming the constant rather than
/// re-spelling its path is the point, so this law follows it rather than
/// keeping a second copy of the string that could drift.
#[cfg(target_os = "macos")]
#[test]
fn the_named_bundle_icon_is_committed_where_the_script_looks() {
    let icon = root().join(crate::app_icon::CANONICAL_ICNS);
    assert!(
        icon.exists(),
        "{icon:?} is named by CFBundleIconFile but is not committed"
    );
    let script = read("scripts/package-macos.sh");
    assert!(
        script.contains(crate::app_icon::CANONICAL_ICNS),
        "scripts/package-macos.sh must copy {} into the bundle",
        crate::app_icon::CANONICAL_ICNS
    );
}

/// Both scripts stay executable and parse. A committed shell gate that cannot
/// run is a gate that silently never ran.
#[test]
fn the_packaging_scripts_are_executable_and_parse() {
    for rel in ["scripts/package-macos.sh", "scripts/dev-app.sh"] {
        let path = root().join(rel);
        assert!(path.exists(), "{rel} is missing");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
            assert!(mode & 0o111 != 0, "{rel} is not executable (mode {mode:o})");
        }
        assert!(
            bash_parses(&path),
            "{rel} is not valid bash (`bash -n` failed)"
        );
    }
}

fn bash_parses(path: &Path) -> bool {
    std::process::Command::new("bash")
        .arg("-n")
        .arg(path)
        .status()
        .map(|s| s.success())
        .unwrap_or(true) // no bash on this host: not this law's business
}
