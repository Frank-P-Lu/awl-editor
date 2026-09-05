use super::*;

// ── THE CENSUS LAW ──────────────────────────────────────────────────────────
//
// The exact failure mode this module retires: a restore list sized to what
// one sweep happened to touch, rather than to the toggle roster's actual
// reach. Pinning `MiscPins`'s field count against an independent source-scan
// of every `Toggle::new(`/`CardFlag::new(` site means a NEW sticky global
// can't join the roster silently — the count moves, this law goes red, and
// the panic message says exactly where to add it.

/// Every file allowed to declare a `Toggle::new(`/`CardFlag::new(` site
/// WITHOUT a corresponding `MiscPins` field, and why. `page.rs`/`spell.rs`'s
/// `PAGE_ON`/`SPELLCHECK_ON` are the two exceptions: `SerialGuard` already
/// snapshots and restores them directly (predating this module, and load-
/// bearing enough — the page/measure pair, the spellcheck bool — to keep
/// their own named fields rather than folding into this catch-all).
const ALREADY_COVERED_ELSEWHERE: &[(&str, &str)] = &[
    ("page.rs", "PAGE_ON — a named field on SerialGuard directly"),
    (
        "spell.rs",
        "SPELLCHECK_ON — a named field on SerialGuard directly",
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum ScanState {
    Normal,
    AfterCfgTest,
    InSkippedBlock(i32),
}

/// Count top-level (non-`cfg(test)`-gated) occurrences of `needle` per file
/// under `root`. Mirrors `crate::toggle::tests::scan_file`'s cfg(test)-body
/// skip, so a test fixture's own `Toggle::new(`/`CardFlag::new(` (this file
/// included) never self-matches.
fn scan_dir(root: &std::path::Path, dir: &std::path::Path, needle: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, needle, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // Skip this file itself: its own panic-message text quotes both
        // scanned-for needles in string literals, which would otherwise
        // self-match (mirrors `crate::toggle::tests::scan_dir`'s identical
        // exclusion of `toggle.rs`).
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs")
            && path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                == Some("misc")
        {
            continue;
        }
        scan_file(root, &path, needle, out);
    }
}

fn scan_file(root: &std::path::Path, path: &std::path::Path, needle: &str, out: &mut Vec<String>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let name = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let mut state = ScanState::Normal;
    for line in text.lines() {
        let trimmed = line.trim_start();
        state = match state {
            ScanState::Normal => {
                if trimmed.starts_with("#[cfg(test)") || trimmed.starts_with("#[cfg(all(test") {
                    ScanState::AfterCfgTest
                } else {
                    if !trimmed.starts_with("//") && trimmed.contains(needle) {
                        out.push(name.clone());
                    }
                    ScanState::Normal
                }
            }
            ScanState::AfterCfgTest => {
                if trimmed.starts_with("#[") {
                    ScanState::AfterCfgTest
                } else if line.contains('{') {
                    let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        ScanState::Normal
                    } else {
                        ScanState::InSkippedBlock(d)
                    }
                } else if trimmed.ends_with(';') {
                    ScanState::Normal
                } else {
                    ScanState::AfterCfgTest
                }
            }
            ScanState::InSkippedBlock(depth) => {
                let d = depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if d <= 0 {
                    ScanState::Normal
                } else {
                    ScanState::InSkippedBlock(d)
                }
            }
        };
    }
}

/// Every declared `Toggle::new(` / `CardFlag::new(` site is either a named
/// `SerialGuard` field (the two on `ALREADY_COVERED_ELSEWHERE`) or a
/// [`MiscPins`] field. Non-vacuous by construction: this asserts the scan
/// finds EXACTLY the roster this module documents, so an added global (which
/// bumps the scan) and a removed one (which shrinks it) both move the count
/// and fail here — not just an added one.
#[test]
fn every_toggle_and_card_flag_site_is_covered_by_serial_guard_or_named_here() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut toggle_sites = Vec::new();
    scan_dir(&root, &root, "Toggle::new(", &mut toggle_sites);
    // `toggle.rs` itself declares no `Toggle::new(` outside its own tests
    // (which the cfg(test) skip already excludes), so no self-match to strip.
    let covered_by_name: Vec<&str> = ALREADY_COVERED_ELSEWHERE.iter().map(|(f, _)| *f).collect();
    let uncovered_toggles: Vec<_> = toggle_sites
        .iter()
        .filter(|f| !covered_by_name.contains(&f.as_str()))
        .collect();
    // The thirteen `MiscPins` toggle fields: debug, outline, menu_bar,
    // typewriter, nits, popover, file_visibility_all, reduced_motion,
    // code_ligatures, wysiwyg, inline_images, whichkey_force_shown,
    // ambient_motion_on.
    assert_eq!(
        uncovered_toggles.len(),
        13,
        "a `Toggle::new(` site appeared or vanished outside page.rs/spell.rs: {:?}. \
         Add (or remove) the matching field in testlock::misc::MiscPins — pins/restore/leaked \
         all need it — and update this count, or add the file to ALREADY_COVERED_ELSEWHERE \
         with a reason if SerialGuard already restores it by another name.",
        uncovered_toggles
    );

    let mut card_flag_sites = Vec::new();
    scan_dir(&root, &root, "CardFlag::new(", &mut card_flag_sites);
    // The four `MiscPins` CardFlag fields: about_open, lifetime_open,
    // streaks_open, peek_open.
    assert_eq!(
        card_flag_sites.len(),
        4,
        "a `CardFlag::new(` site appeared or vanished: {:?}. Add (or remove) the matching \
         field in testlock::misc::MiscPins and update this count.",
        card_flag_sites
    );

    for (file, reason) in ALREADY_COVERED_ELSEWHERE {
        assert!(
            toggle_sites.iter().any(|f| f == file),
            "{file} is on ALREADY_COVERED_ELSEWHERE ({reason}) but no longer declares a \
             Toggle::new( — remove it from the list instead of leaving it stale"
        );
    }
}

// ── ROUND-TRIP AND MUTATION-PROOF LAWS ──────────────────────────────────────

#[test]
fn pins_restore_round_trips_with_no_diff() {
    let _g = crate::testlock::serial();
    let before = pins();
    restore(&before);
    let after = pins();
    assert_eq!(
        leaked(&before, &after),
        Vec::<String>::new(),
        "restoring what pins() just read must be a no-op"
    );
}

#[test]
fn toggles_restore_cleans_up_an_auto_caret_mode_a_bare_capture_restore_pair_cannot() {
    // The exact shape this guard exists to replace: `let m = caret::mode(); …;
    // caret::set_mode(m);` cannot express "was auto", so it leaves the
    // override armed even when it round-trips the CONCRETE mode correctly.
    let _g = crate::testlock::serial();
    crate::caret::clear_override();
    assert!(crate::caret::is_auto(), "test fixture: start from auto");
    {
        let _restore = TogglesRestore::capture();
        crate::caret::set_mode(crate::caret::CaretMode::Block);
        assert!(!crate::caret::is_auto());
    }
    assert!(
        crate::caret::is_auto(),
        "TogglesRestore put the override back to auto, not to some concrete mode"
    );
}

#[test]
fn a_deliberately_dirty_misc_window_fails_and_restores_before_the_next_reader() {
    let before = {
        let _g = crate::testlock::serial();
        pins()
    };
    let leaked_result = std::panic::catch_unwind(|| {
        let _g = crate::testlock::serial();
        crate::debug::set_debug_on(!before.debug);
    });
    assert!(
        leaked_result.is_err(),
        "a checked window must reject dirty misc globals"
    );
    let _g = crate::testlock::serial();
    assert_eq!(
        pins(),
        before,
        "the failed misc window restores before its next reader enters"
    );
}

/// The headline case: flip EVERY field this module owns, let the window die,
/// and require a pristine exit — proving the guard sweeps the WHOLE roster,
/// not just the one field a future bug happens to name.
#[test]
fn every_misc_field_is_restored_not_just_the_one_that_bit_us() {
    let before = {
        let _g = crate::testlock::serial();
        pins()
    };
    let died = std::panic::catch_unwind(|| {
        let _g = crate::testlock::serial();
        crate::debug::set_debug_on(!before.debug);
        crate::outline::set_outline_on(!before.outline);
        crate::menubar::set_menu_bar_on(!before.menu_bar);
        crate::typewriter::set_typewriter_on(!before.typewriter);
        crate::nits::set_nits_on(!before.nits);
        crate::popover::set_popover_on(!before.popover);
        crate::file_visibility::set_all_on(!before.file_visibility_all);
        crate::motion::set_reduced(!before.reduced_motion);
        crate::render::set_code_ligatures_on(!before.code_ligatures);
        crate::markdown::set_wysiwyg_on(!before.wysiwyg);
        crate::markdown::set_inline_images_on(!before.inline_images);
        crate::whichkey::set_force_shown(!before.whichkey_force_shown);
        // Each discrete-valued (non-bool) field picks whichever alternative
        // ISN'T the ambient starting value, so the flip is guaranteed to
        // differ rather than accidentally round-tripping to the same value.
        crate::caret::set_mode(match before.caret_mode {
            Some(crate::caret::CaretMode::Ibeam) => crate::caret::CaretMode::Block,
            _ => crate::caret::CaretMode::Ibeam,
        });
        crate::about::set_open(!before.about_open);
        crate::lifetime::set_open(!before.lifetime_open);
        crate::streaks::set_open(!before.streaks_open);
        crate::peek::set_open(!before.peek_open);
        crate::hud::set_held(!before.hud_held);
        crate::menubar::set_open(if before.menu_dropdown_open == Some(0) {
            Some(1)
        } else {
            Some(0)
        });
        crate::spell::set_active_variant(match before.spell_variant {
            crate::spell::DictVariant::EnGb => crate::spell::DictVariant::EnUs,
            _ => crate::spell::DictVariant::EnGb,
        });
        crate::dateformat::set_active_format(match before.date_format {
            crate::dateformat::DateFormat::Iso => crate::dateformat::DateFormat::DdMmYy,
            _ => crate::dateformat::DateFormat::Iso,
        });
        crate::settings::set_scroll_sensitivity(before.scroll_sensitivity + 0.3);
        crate::warpgrid::set_ambient_motion_on(!before.ambient_motion_on);
        panic!("a fixture that flipped every misc global and died");
    });
    assert!(died.is_err());
    let _g = crate::testlock::serial();
    let after = pins();
    assert_eq!(
        leaked(&before, &after),
        Vec::<String>::new(),
        "every misc field is restored, including ones an unwinding fixture never reached its \
         own reset for"
    );
}

/// Mutation-prove the panic fires BY NAME: drop one field from the diff
/// (simulating a restore list that once again under-fills) and confirm the
/// guard's exit check would have caught it.
#[test]
fn leaked_names_the_field_a_narrower_restore_would_have_missed() {
    let base = pins();
    let mut dirty = base.clone();
    dirty.debug = !dirty.debug;
    let names = leaked(&base, &dirty);
    assert_eq!(
        names,
        vec![format!("debug: {:?} -> {:?}", base.debug, dirty.debug)],
        "leaked() must name the exact field that diverged"
    );
}
