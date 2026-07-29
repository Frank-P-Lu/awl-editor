//! tests/spawn_config_law.rs — THE SPAWNED-CHILD CONFIG LAW (queue item 93).
//!
//! THE BUG THIS LOCKS OUT. `config::config_path()` (`src/config/model.rs`)
//! resolves `--config` → `$AWL_CONFIG` → `$XDG_CONFIG_HOME/awl/config.toml` →
//! `$HOME/.config/awl/config.toml`. Every binary-spawning test in the suite
//! wrote `.env_remove("AWL_CONFIG")` believing it isolated the child; it does
//! the opposite — removing that variable is the ONE value that guarantees the
//! ladder walks on down into the DEVELOPER'S OWN dotfiles. A personal
//! `zoom = 1.500` in `~/.config/awl/config.toml` therefore rescaled every pixel
//! metric inside the spawned child, and `tests/bullet_blank_line_nit_pixels.rs`
//! (whose row band is computed from the sidecar's `font.line_height`, which AT
//! THE TIME reported the unscaled base constant) started addressing row 0
//! instead of row 1 and reporting 118 phantom "stray mark" pixels — red on the
//! developer's box, green in CI, with zero product change between them. Item 96
//! later corrected that sidecar field; the config-isolation law remains necessary
//! for every other sticky preference.
//!
//! TWO LAWS, ONE RULE ("a spawned child's config source is DECLARED, never
//! inherited"):
//!
//!   1. **Structural, no-wildcard**: `env!("CARGO_BIN_EXE_awl")` — the only way
//!      to name the binary — may appear in exactly ONE file,
//!      `tests/common/mod.rs`. Every test spawns through that owner, so a new
//!      test cannot reintroduce the idiom without deleting this law first.
//!   2. **Behavioural**: a child spawned through the owner renders BYTE-
//!      IDENTICALLY whether or not an ambient user config exists on the ladder
//!      below it. This is the assertion that would actually have caught item 93
//!      (the structural law only stops the shape, not the effect), and it is
//!      proven against a decoy config carrying the exact settings — `zoom` and
//!      `theme` — that bent the capture.

mod common;
use common::ScratchDir;

use std::path::Path;

/// A source file's CODE, with every `//`-comment cut away — the laws below are
/// about what a test DOES, and both of them describe the banned idioms in
/// prose, so a raw text scan would convict this very file. The cut is
/// string-aware (a `//` inside a string literal is not a comment), which is
/// enough Rust for a test suite that contains no block comments.
fn code_only(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let bytes = line.as_bytes();
        let mut in_str = false;
        let mut i = 0;
        let cut = loop {
            if i >= bytes.len() {
                break bytes.len();
            }
            match bytes[i] {
                b'\\' if in_str => i += 1,
                b'"' => in_str = !in_str,
                b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => break i,
                _ => {}
            }
            i += 1;
        };
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// Every `tests/*.rs` source, plus the shared module, as `(name, code)` —
/// comments stripped by [`code_only`].
fn test_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut out: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&dir)
        .expect("tests/ is readable")
        .flatten()
    {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "rs") {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let src = std::fs::read_to_string(&p).expect("test source reads");
            out.push((name, code_only(&src)));
        }
    }
    let owner = dir.join("common").join("mod.rs");
    let owner_src = std::fs::read_to_string(&owner).expect("the shared spawn module exists");
    out.push(("common/mod.rs".to_string(), code_only(&owner_src)));
    out.sort();
    out
}

/// The literal token that names the binary. Written here in pieces so this
/// file's own law does not trip over its own text.
fn bin_token() -> String {
    format!("env!(\"CARGO_BIN_EXE_{}\")", "awl")
}

/// The banned idiom, likewise assembled so this file can name it.
fn banned_removal() -> String {
    format!("env_remove(\"AWL_{}\")", "CONFIG")
}

#[test]
fn only_the_shared_module_names_the_awl_binary() {
    let token = bin_token();
    let mut offenders: Vec<String> = Vec::new();
    let mut owner_seen = false;
    for (name, text) in test_sources() {
        if !text.contains(&token) {
            continue;
        }
        if name == "common/mod.rs" {
            owner_seen = true;
        } else {
            offenders.push(name);
        }
    }
    assert!(
        owner_seen,
        "tests/common/mod.rs must be the one place that names the awl binary — \
         it no longer contains {token}, so the spawn owner has been gutted"
    );
    assert!(
        offenders.is_empty(),
        "these tests spawn the awl binary directly instead of through \
         `common::awl` / `common::awl_in_home`: {offenders:?}. A direct spawn \
         inherits the config ladder and can read the developer's own \
         ~/.config/awl/config.toml (item 93) — route it through the owner in \
         tests/common/mod.rs."
    );
}

#[test]
fn no_test_scrubs_awl_config_outside_the_canary_door() {
    let banned = banned_removal();
    let offenders: Vec<String> = test_sources()
        .into_iter()
        .filter(|(name, text)| name != "common/mod.rs" && text.contains(&banned))
        .map(|(name, _)| name)
        .collect();
    assert!(
        offenders.is_empty(),
        "these tests remove $AWL_CONFIG: {offenders:?}. Removing it is not \
         isolation — it is the one value that makes config_path() fall through \
         to $XDG_CONFIG_HOME and then the developer's $HOME (item 93). PIN the \
         variable via `common::awl`, or take the canary door \
         `common::awl_in_home`, which owns the single legitimate removal."
    );
}

#[test]
fn the_owner_pins_the_config_variable_rather_than_removing_it() {
    let (_, owner) = test_sources()
        .into_iter()
        .find(|(name, _)| name == "common/mod.rs")
        .expect("the shared spawn module exists");
    let pin = format!(".env(\"AWL_{}\"", "CONFIG");
    assert!(
        owner.contains(&pin),
        "tests/common/mod.rs must SET $AWL_CONFIG (found no `{pin}`) — pinning \
         that rung is the whole mechanism that keeps a spawned child off the \
         developer's dotfiles (item 93)"
    );
}

/// A fresh, uniquely-named tempdir under the OS temp root, owned by a
/// [`ScratchDir`] guard that removes it on drop (queue item 168; this fixture
/// used to never remove it at all).
fn tmp_dir(tag: &str) -> ScratchDir {
    let dir =
        std::env::temp_dir().join(format!("awl-spawn-config-law-{tag}-{}", std::process::id()));
    ScratchDir::new(dir)
}

/// One capture through the spawn owner, with `$XDG_CONFIG_HOME` pointed at
/// `xdg` (the rung directly BELOW the pinned `$AWL_CONFIG`). Returns the PNG
/// bytes, or `None` if the box has no wgpu adapter.
fn capture_under_xdg(sandbox: &Path, xdg: &Path, doc: &Path, out: &Path) -> Option<Vec<u8>> {
    let output = common::awl(sandbox)
        .arg("--theme")
        .arg("Magpie")
        .arg("--screenshot")
        .arg(out)
        .arg(doc)
        .env("XDG_CONFIG_HOME", xdg)
        .env_remove("AWL_CJK_FORCE")
        .output()
        .expect("failed to spawn the awl binary");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() && stderr.contains("no wgpu adapter for headless capture") {
        return None;
    }
    assert!(
        output.status.success(),
        "capture failed: {}\n{stderr}",
        output.status
    );
    Some(std::fs::read(out).expect("capture PNG reads"))
}

#[test]
fn a_child_spawned_through_the_owner_ignores_an_ambient_user_config() {
    let root = tmp_dir("decoy");
    let doc = root.join("doc.md");
    std::fs::write(&doc, "- \n\nsomething\n").unwrap();

    // Rung 3 of the ladder, salted with the settings the developer's real
    // config carried when item 93 went red — `zoom`, which rescales every pixel
    // metric, plus `page_width`, which moves the column, and a `theme` the
    // explicit `--theme` flag out-ranks (there on purpose: the flag's
    // precedence is itself part of what the capture path relies on). If the pin
    // ever stops holding, this capture cannot come out the same as the clean one.
    let decoy = root.join("decoy-xdg");
    std::fs::create_dir_all(decoy.join("awl")).unwrap();
    std::fs::write(
        decoy.join("awl").join("config.toml"),
        "theme = \"Wagtail\"\nzoom = 1.500\npage_width = 40\n",
    )
    .unwrap();

    // Rung 3, empty — the control.
    let clean = root.join("clean-xdg");
    std::fs::create_dir_all(&clean).unwrap();

    let Some(with_decoy) = capture_under_xdg(&root, &decoy, &doc, &root.join("decoy.png")) else {
        eprintln!("skipping spawn_config_law capture: no wgpu adapter");
        return;
    };
    let without =
        capture_under_xdg(&root, &clean, &doc, &root.join("clean.png")).expect("adapter present");

    assert_eq!(
        with_decoy, without,
        "a capture spawned through `common::awl` changed when an ambient user \
         config appeared on $XDG_CONFIG_HOME — the $AWL_CONFIG pin is not \
         holding, and every pixel test in the suite is once again reading the \
         developer's personal theme/zoom (item 93)"
    );

    // MEANINGFULNESS GUARD. The equality above is vacuous unless those decoy
    // settings are ones awl would genuinely have obeyed. Put the identical TOML
    // on the rung the owner PINS (a second sandbox's own config file) and the
    // render must visibly change — proving the child reads config at all, that
    // this config in particular moves pixels, and therefore that the clean/decoy
    // match is the pin working rather than a config awl ignores.
    let obeying = root.join("obeying");
    std::fs::create_dir_all(&obeying).unwrap();
    std::fs::write(
        common::config_path_in(&obeying),
        "theme = \"Wagtail\"\nzoom = 1.500\npage_width = 40\n",
    )
    .unwrap();
    let obeyed = capture_under_xdg(&obeying, &clean, &doc, &root.join("obeyed.png"))
        .expect("adapter present");
    assert_ne!(
        obeyed, without,
        "the decoy config moved no pixels even when read — this law would pass \
         vacuously; give it settings that visibly change the render"
    );
}
