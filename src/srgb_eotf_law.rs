//! src/srgb_eotf_law.rs — THE ONE-sRGB-EOTF LAW.
//!
//! Five production files each carried their own hand-rolled copy of the sRGB
//! electro-optical transfer function (the `0.04045` / `12.92` / `0.055` /
//! `1.055` / `2.4` constants, IEC 61966-2-1) before this file existed — plus a
//! sixth, undocumented as a duplicate of any of the other five, found by this
//! item's own scope check. All six now route through `theme::color`'s
//! `srgb_channel_to_linear` / `srgb_channel_to_linear_f32`, the tree's one
//! definition of the rule at each width a real call site needs (see that
//! module's own doc comment for why two widths, not one).
//!
//! This is the source-scan half of "make the bypass module-private, and add a
//! law with a no-wildcard match so a new member can't dodge the sweep" — the
//! shape of `embedded_docs_law.rs`'s `embed_owner_is_the_only_include_str_site`
//! and `println_audit.rs`'s cfg(test)-block-skipping scanner, applied to this
//! constant instead of a macro name.
//!
//! **WHAT THIS CANNOT SEE** (a source scan is a grep, not a compiler, and its
//! own configuration is therefore a hypothesis, not a guarantee — the same
//! admission `app/files/open/tests.rs`'s bundled-document law makes about
//! itself):
//! - A copy that retypes the breakpoint as `4.045e-2` or otherwise re-spells
//!   the literal is invisible: this scans for the exact text `0.04045`, not
//!   its numeric value.
//! - A copy reached through a renamed `use` or a local alias of the needle
//!   (unlikely for a float literal, but not impossible via a `const`) would
//!   still spell `0.04045` at its own definition site, so it would still be
//!   caught there — but a copy that imports awl's OWN owner and then
//!   re-derives a second, textually different formula from the same standard
//!   would not share this needle and would not be caught at all.
//! - Only `src/**/*.rs` is scanned. A shader (`.wgsl`) or build script
//!   performing this conversion independently is outside this law's reach.
//!
//! This file is also the one OWNER of the source-scan primitives every
//! needle-in-production law needs — the `src/**/*.rs` walk, the production/test
//! path predicate, and the two text strippers — so a second such law reuses them
//! instead of carrying its own copy of "what counts as production source".
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// Repo root (the crate manifest dir — the worktree/checkout root).
pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `*.rs` under `src/` (recursively), repo-relative with `/` separators.
pub(crate) fn src_rs_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk_rs(&root.join("src"), root, &mut out);
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
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
    }
}

/// A file this law does not hold to "no hand-rolled EOTF": any path with a
/// `tests` directory component, a `tests.rs` basename, or a `_test.rs` suffix
/// — mirroring `scripts/code-health.py`'s own `production()` predicate, so
/// this law's notion of "production" cannot silently diverge from the
/// ratchet's. Test-local oracles are EXPECTED to carry an independent copy of
/// the standard (see `theme/tests/clear.rs`'s own `linear_to_srgb`, and this
/// item's own per-call-site bit-identity tests) — that independence is the
/// point of a test oracle, not a violation of this law.
pub(crate) fn is_test_path(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    let name = parts.last().copied().unwrap_or("");
    parts.contains(&"tests") || name == "tests.rs" || name.ends_with("_test.rs")
}

/// The ONE production file allowed to spell the EOTF constants: the owner
/// itself. An exact path match (no prefix/glob), so a same-named file in a
/// different directory could never be mistaken for it.
const OWNER: &str = "src/theme/color.rs";

/// The needle: the sRGB EOTF's breakpoint constant. Chosen over `12.92` /
/// `1.055` / `2.4` because those are common enough numbers to risk an
/// unrelated false positive; `0.04045` is specific to this one curve and (in
/// this tree, verified while writing this law) never appears for any other
/// reason.
const NEEDLE: &str = "0.04045";

/// Strip `#[cfg(test)]`-gated blocks from `text`, mirroring
/// `println_audit::scan_file`'s state machine exactly (a stray `#[cfg(test)]`
/// fixture inside an otherwise-real file — this item's own per-call-site
/// bit-identity tests among them — must not itself trip a production-only
/// law). Returns the file's non-test text only.
pub(crate) fn strip_cfg_test_blocks(text: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        AfterCfgTest,
        InSkippedBlock(i32),
    }
    let mut state = State::Normal;
    let mut kept = String::with_capacity(text.len());
    for line in text.lines() {
        state = match state {
            State::Normal => {
                let t = line.trim_start();
                if t.starts_with("#[cfg(test)") || t.starts_with("#[cfg(all(test") {
                    State::AfterCfgTest
                } else {
                    kept.push_str(line);
                    kept.push('\n');
                    State::Normal
                }
            }
            State::AfterCfgTest => {
                let t = line.trim_start();
                if t.starts_with("#[") {
                    State::AfterCfgTest // a stacked attribute; keep waiting
                } else if line.contains('{') {
                    let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        State::Normal
                    } else {
                        State::InSkippedBlock(d)
                    }
                } else if line.trim_end().ends_with(';') {
                    State::Normal // a bare `mod tests;` declaration
                } else {
                    State::AfterCfgTest // a multi-line signature; keep waiting
                }
            }
            State::InSkippedBlock(depth) => {
                let d = depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if d <= 0 {
                    State::Normal
                } else {
                    State::InSkippedBlock(d)
                }
            }
        };
    }
    kept
}

/// Drop every line's comment text, cutting at its first `//`. A scan whose
/// needle is ARITHMETIC wants the code only: prose that describes a retired
/// spelling is documentation, not a bypass.
///
/// Deliberately naive about string literals — a `//` inside one truncates that
/// line early, which can only ever hide a needle written inside a string, never
/// a live expression.
pub(crate) fn strip_line_comments(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    for line in text.lines() {
        let code = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        kept.push_str(code);
        kept.push('\n');
    }
    kept
}

/// THE LAW. Every production `.rs` file under `src/`, other than the owner
/// itself, must not spell the sRGB EOTF's breakpoint constant outside a
/// `#[cfg(test)]` block. A seventh hand-rolled copy fails here by NAME.
#[test]
fn no_srgb_eotf_copy_outside_its_one_owner() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    let mut owner_seen = false;
    for rel in src_rs_files(&root) {
        if rel == "src/srgb_eotf_law.rs" {
            continue; // this law's own doc/prose names the needle
        }
        let text = fs::read_to_string(root.join(&rel)).expect("read src file");
        if rel == OWNER {
            owner_seen = text.contains(NEEDLE);
            continue; // the one sanctioned owner
        }
        if is_test_path(&rel) {
            continue; // test oracles are allowed their own independent copy
        }
        let production_text = strip_cfg_test_blocks(&text);
        if production_text.contains(NEEDLE) {
            offenders.push(rel);
        }
    }
    assert!(
        owner_seen,
        "sanity: {OWNER} must itself spell the EOTF breakpoint constant `{NEEDLE}`, or this \
         law's needle has drifted from the owner's actual text and is no longer checking \
         anything real"
    );
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "hand-rolled sRGB EOTF found outside its one owner ({OWNER}): {offenders:?} — route \
         through `theme::srgb_channel_to_linear` (f64) or `theme::srgb_channel_to_linear_f32` \
         (f32, the width every shader-side converter needs) instead"
    );
}
