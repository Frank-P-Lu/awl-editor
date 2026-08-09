//! src/render/tests/settings_fixture_law.rs — ONE OWNER FOR THE render::tests
//! `SettingsValues` PROBE FIXTURE.
//!
//! Six files under `src/render/tests/` each hand-rolled their own
//! `crate::settings::SettingsValues { .. }` literal — byte-identical in every
//! field except `zoom`/`scroll_sensitivity`, which two of them legitimately
//! parameterize (`range_rail`'s rail-position sweep, `settings_row_reach_law`'s
//! off-default-zoom probe) — before `mod.rs`'s `settings_values(zoom,
//! scroll_sensitivity)` became the one owner. Every consumer kept its own
//! call-site signature (some zero-arg, one one-arg, one two-arg) as a
//! one-line wrapper routed through the owner, so none of their call sites
//! moved.
//!
//! This is the source-scan half of "make the bypass module-private, and add a
//! law with a no-wildcard match so a new member can't dodge the sweep" — the
//! shape of `srgb_eotf_law.rs`'s scanner, applied to this fixture's
//! construction site instead of a numeric constant, and scoped to this
//! directory instead of the whole tree since the fixture is test-only.
//!
//! **WHAT THIS CANNOT SEE** (a source scan is a grep, not a compiler):
//! - A copy reached by re-deriving the same field set under a different type
//!   alias or a renamed re-export of `SettingsValues` would not spell the
//!   needle this scans for.
//! - `fs::read_dir` has no wasm32-unknown-unknown counterpart (no OS
//!   filesystem there), the same reason `font_licence` gates itself
//!   the same way — so this only runs on the native test target.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

fn tests_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render/tests")
}

/// Every `*.rs` directly under `src/render/tests/` (recursively, so a future
/// reorg into a subdirectory is still swept), repo-relative with `/`
/// separators, EXCLUDING `mod.rs` itself — the one file allowed to construct
/// the fixture directly, since it IS the owner — and this law's own file
/// (whose doc comment and string literals spell the needle in prose).
fn candidate_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_rs(&tests_dir(), root, &mut out);
    out.sort();
    out
}

fn walk_rs(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, root, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "mod.rs" || file_name == "settings_fixture_law.rs" {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
}

/// The needle: a `SettingsValues` struct-literal construction. Matches both
/// the qualified (`crate::settings::SettingsValues {`) and bare
/// (`SettingsValues {`) spellings a `use` could shorten it to, so a future
/// copy can't dodge the sweep by importing the type first.
const NEEDLE: &str = "SettingsValues {";

/// A LINE carrying the needle that is a return-type declaration
/// (`fn foo(..) -> crate::settings::SettingsValues {`), not a struct-literal
/// construction — the six one-line wrappers this law enrolls all return the
/// type by name, so their signature line contains the needle textually
/// without constructing anything. `->` appears nowhere in a struct literal in
/// this tree, so its presence earlier on the same line is the discriminator.
fn is_return_type_line(line: &str, needle_at: usize) -> bool {
    line[..needle_at].contains("->")
}

/// THE LAW. Every file under `src/render/tests/` other than the directory's
/// own `mod.rs` must reach `SettingsValues` through the shared
/// `settings_values(zoom, scroll_sensitivity)` owner, never by constructing
/// the struct literal itself. A seventh hand-rolled copy fails here BY NAME.
#[test]
fn no_settings_values_literal_outside_render_tests_owner() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let files = candidate_files(&root);
    assert!(
        files.len() > 50,
        "sanity: src/render/tests/ scan found only {} files — the law's own \
         directory path has drifted from the real tree",
        files.len()
    );

    let mod_rs = fs::read_to_string(tests_dir().join("mod.rs")).expect("read render/tests/mod.rs");
    assert!(
        mod_rs.contains("fn settings_values(") && mod_rs.contains(NEEDLE),
        "sanity: render/tests/mod.rs must itself define `settings_values` and construct the \
         struct literal, or this law's needle has drifted from the owner's actual text and is \
         no longer checking anything real"
    );

    let mut offenders: Vec<String> = Vec::new();
    for rel in &files {
        let text = fs::read_to_string(root.join(rel)).expect("read render/tests file");
        let constructs = text.lines().any(|line| {
            line.find(NEEDLE)
                .is_some_and(|at| !is_return_type_line(line, at))
        });
        if constructs {
            offenders.push(rel.clone());
        }
    }
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "hand-rolled SettingsValues construction found outside render/tests/mod.rs's one owner: \
         {offenders:?} — route through `super::settings_values(zoom, scroll_sensitivity)` \
         instead of constructing the struct literal directly"
    );
}
