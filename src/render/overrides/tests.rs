use super::*;
use crate::testscratch::ScratchDir;

#[test]
fn set_test_override_installs_a_whole_struct_directly() {
    let _g = crate::testlock::serial();
    set_test_override(RenderOverrides {
        card_anchor: Some(theme::CardAnchor::TopRight),
        list_style: Some(theme::ListStyle::Pane),
        ..Default::default()
    });
    assert_eq!(current().card_anchor, Some(theme::CardAnchor::TopRight));
    assert_eq!(current().list_style, Some(theme::ListStyle::Pane));
    // A reset override must not leak the previous struct's field into a
    // later reader.
    set_test_override(RenderOverrides::default());
    assert_eq!(current().card_anchor, None);
    assert_eq!(current().list_style, None);
}

// LAW: these env vars are read from exactly ONE place — `from_env`.
const KNOB_ENV_VARS: &[&str] = &[
    "AWL_OVERLAY_STYLE_FORCE",
    "AWL_OVERLAY_ALIGN",
    "AWL_OVERLAY_ANCHOR_FORCE",
    "AWL_CHROME_FACE_FORCE",
    "AWL_MOTION_FORCE",
    "AWL_OVERLAY_SLANT_FORCE",
    "AWL_OVERLAY_LIST_FORCE",
    "AWL_FACET_STYLE_FORCE",
    "AWL_PANE_SPLIT_FORCE",
    "AWL_OVERLAY_DENSITY_FORCE",
    "AWL_OVERLAY_MOTION_FORCE",
];

/// The one file allowed to name these vars — its path relative to `src`,
/// not just its basename, so an unrelated `mod.rs` elsewhere can't spoof it.
const OWNER: &str = "render/overrides/mod.rs";

fn scan_dir(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, usize, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // A `tests.rs` is pure test scaffolding (mirrors `println_audit::scan_dir`);
        // scanning it would make this checklist self-match its own const.
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        scan_file(base, &path, out);
    }
}

/// Mirrors `println_audit::scan_file`: skips `#[cfg(test)]`-gated bodies, so
/// a knob read guarded for tests doesn't count as a stray production read.
fn scan_file(
    base: &std::path::Path,
    path: &std::path::Path,
    out: &mut Vec<(String, usize, String)>,
) {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        AfterCfgTest,
        InSkippedBlock(i32),
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let name = path
        .strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let mut state = State::Normal;
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        state = match state {
            State::Normal => {
                if trimmed.starts_with("#[cfg(test)") || trimmed.starts_with("#[cfg(all(test") {
                    State::AfterCfgTest
                } else {
                    if !trimmed.starts_with("//") {
                        for var in KNOB_ENV_VARS {
                            let needle = format!("\"{var}\"");
                            if line.contains(&needle) {
                                out.push((name.clone(), i + 1, (*var).to_string()));
                            }
                        }
                    }
                    State::Normal
                }
            }
            State::AfterCfgTest => {
                if trimmed.starts_with("#[") {
                    State::AfterCfgTest // a stacked attribute; keep waiting
                } else if line.contains('{') {
                    let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        State::Normal
                    } else {
                        State::InSkippedBlock(d)
                    }
                } else if trimmed.ends_with(';') {
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
}

#[test]
fn render_overrides_env_read_law() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    scan_dir(&root, &root, &mut hits);

    let stray: Vec<_> = hits.iter().filter(|(f, _, _)| f != OWNER).collect();
    assert!(
        stray.is_empty(),
        "these AWL_*_FORCE/AWL_OVERLAY_ALIGN knobs must be read ONLY by \
         `RenderOverrides::from_env` in `{OWNER}` — a second read site is exactly \
         the two-code-paths-to-one-setting bug this module retired. offending lines:\n{}",
        stray
            .iter()
            .map(|(f, l, v)| format!("  {f}:{l}  ({v})"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Non-vacuous: all eleven var names (ten knobs; `card_anchor` reads
    // two) must actually be found in `from_env`, or this count drops.
    let owner_hits = hits.iter().filter(|(f, _, _)| f == OWNER).count();
    assert_eq!(
        owner_hits,
        KNOB_ENV_VARS.len(),
        "expected exactly one `from_env` read site per knob env var in \
         `{OWNER}`; found {owner_hits}"
    );
}

#[test]
fn scan_file_skips_comment_lines() {
    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl_render_overrides_law_test_{}",
        std::process::id()
    )));
    let path = dir.join("fixture.rs");
    std::fs::write(
        &path,
        "// mentions \"AWL_MOTION_FORCE\" in prose, not code\nfn f() {}\n",
    )
    .unwrap();
    let mut out = Vec::new();
    scan_file(&dir, &path, &mut out);
    assert!(
        out.is_empty(),
        "a comment line must not count as a code read"
    );
}
