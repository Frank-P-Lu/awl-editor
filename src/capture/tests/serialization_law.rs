//! THE CAPTURE-SERIALIZATION LAWS (queue item 98) — the two structural rules
//! that retired the `sidecar_reports_doc_lang_and_per_script_font_resolution`
//! flake, tested at the purest seam each one reaches.
//!
//! The flake: that test did not open with `crate::testlock::serial()` (its nine
//! siblings in `i18n_fixtures.rs` all do), so it ran concurrently with the
//! hundreds of `theme::set_active` / `cycle` call sites the suite drives. Its
//! sidecar's `font.cjk` and `font.scripts.ja` were two INDEPENDENT walks of the
//! active world's font ladder, microseconds apart; a flip landing in that
//! window produced two different family names for the same `FontId::Ja` inside
//! one sidecar, failing the test's `cjk == scripts.ja` assertion. The window is
//! small, so it failed rarely — which is what made it corrosive: a flaky gate
//! teaches everyone to re-run until green.
//!
//! Adding the missing guard fixes that ONE test. Law 1 below then found SEVEN
//! more unguarded capture tests (six in `pickers_faceted`, one in `panels`) —
//! the class was wider than the symptom. These two laws close it:
//!
//!   1. **No unguarded capture.** Every capture path funnels through
//!      `sidecar::write_sidecar`, which now asserts the guard is held in a test
//!      build. A future capture test that forgets the guard fails loudly and by
//!      name on its first run, not statistically on someone else's merge.
//!   2. **One resolution per sidecar.** `font.cjk` and `font.scripts.ja` are
//!      now two views of ONE `ScriptFontReports` snapshot, so they cannot
//!      disagree even if the theme were flipped mid-write.
//!
//! (`sidecar`'s `cjk_json` / `scripts_json` / `assert_capture_is_serialized`
//! are `pub(super)` rather than private solely so these laws can test them at
//! the pure seam instead of through a whole GPU capture.)

use super::super::sidecar::{assert_capture_is_serialized, cjk_json, scripts_json};
use crate::render::ScriptFontReports;
use crate::theme::{ALL_FONT_IDS, FontId};

/// LAW 1 (pure seam): the capture-serialization check PASSES on a thread
/// holding `testlock::serial()` and PANICS on one that isn't. `currently_held`
/// is thread-local, so "a caller without the guard" is modelled exactly by a
/// spawned thread — which is also the real shape of the race (the flipping
/// test and the capturing test were two `cargo test` worker threads).
#[test]
fn an_unguarded_capture_is_a_hard_error() {
    let _tg = crate::testlock::serial();

    // Guarded (this thread): the check is a no-op.
    assert_capture_is_serialized();

    // Unguarded (another thread, which cannot inherit our thread-local hold):
    // the check must panic rather than let the capture proceed. It cannot
    // acquire the guard behind our back either — we hold it for this window.
    let unguarded = std::thread::spawn(|| {
        assert!(
            !crate::testlock::currently_held(),
            "a fresh thread holds nothing"
        );
        assert_capture_is_serialized();
    });
    let outcome = unguarded.join();
    assert!(
        outcome.is_err(),
        "an unguarded capture must panic: the law is what stops a future capture \
         test from silently racing every theme-flipping test (queue item 98)"
    );
}

/// LAW 1 (wired): the check is actually ON the real capture path, not merely
/// defined beside it. A REAL `capture_with` driven from a thread without the
/// guard must panic — proof that `write_sidecar` calls the check. Every capture
/// entry point (`modes`' plain + motion stills, `animated`'s timeline and held
/// steppers, `film`'s storyboard stepper, `frames`' sheet) writes its sidecar
/// through that one function, so none of them can dodge the law.
#[test]
fn the_real_capture_path_enforces_the_law() {
    if !super::adapter_available() {
        eprintln!("skipping the_real_capture_path_enforces_the_law: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl_capture_law_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("unguarded.png");

    let unguarded = std::thread::spawn({
        let png = png.clone();
        move || {
            let mut buf = crate::buffer::Buffer::from_str("hello\n");
            buf.set_path(png.with_extension("md"));
            let _ =
                crate::capture::capture_with(&png, &buf, &crate::capture::CaptureOpts::default());
        }
    });
    assert!(
        unguarded.join().is_err(),
        "a real capture taken without `testlock::serial()` must panic in write_sidecar"
    );
    assert!(
        !png.with_extension("json").exists(),
        "the law fires BEFORE the sidecar is written, so no torn sidecar reaches disk"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// LAW 2: `font.cjk` is LITERALLY the `font.scripts.ja` entry — one snapshot,
/// one formatter, so the equality the i18n contract asserts holds BY
/// CONSTRUCTION rather than by two ladder walks happening to agree. Uses a
/// synthetic `ScriptFontReports` with a distinctive `ja` family so the test
/// witnesses the plumbing rather than whatever this machine has installed.
#[test]
fn font_cjk_is_literally_the_scripts_ja_entry() {
    let fonts = ScriptFontReports {
        ja: Some(("Fixture Mincho JP", true)),
        zh_hans: Some(("Fixture SC", false)),
        zh_hant: None,
        ko: Some(("Fixture KR", false)),
    };
    let cjk = cjk_json(&fonts);
    assert_eq!(
        cjk,
        "{ \"family\": \"Fixture Mincho JP\", \"bundled\": true }"
    );
    assert_eq!(
        scripts_json(&fonts),
        format!(
            "{{ \"ja\": {cjk}, \"zh_hans\": {{ \"family\": \"Fixture SC\", \"bundled\": false }}, \
             \"zh_hant\": null, \"ko\": {{ \"family\": \"Fixture KR\", \"bundled\": false }} }}"
        ),
        "the scripts block's `ja` entry must be the SAME bytes as the cjk block"
    );

    // The degenerate case stays representable and shared: an unresolved `ja` is
    // `null` in BOTH blocks, never `null` in one and a family in the other.
    let none = ScriptFontReports::default();
    assert_eq!(cjk_json(&none), "null");
    assert_eq!(
        scripts_json(&none),
        "{ \"ja\": null, \"zh_hans\": null, \"zh_hant\": null, \"ko\": null }"
    );
}

/// LAW 2's SWEEP: every non-Latin `FontId` reaches the sidecar. `get`'s match is
/// wildcard-free, so a new variant fails to COMPILE there; this catches the
/// other half — a new variant that compiles (because `get` was updated) but was
/// never given a `font.scripts` key. Latin is the one deliberate absence: the
/// base doc attrs already shape in the world's own display face.
#[test]
fn every_non_latin_font_id_has_a_scripts_entry() {
    let named = |id: FontId| -> Option<&'static str> {
        match id {
            FontId::Latin => None,
            FontId::Ja => Some("ja"),
            FontId::ZhHans => Some("zh_hans"),
            FontId::ZhHant => Some("zh_hant"),
            FontId::Ko => Some("ko"),
        }
    };
    let fonts = ScriptFontReports {
        ja: Some(("A", false)),
        zh_hans: Some(("B", false)),
        zh_hant: Some(("C", false)),
        ko: Some(("D", false)),
    };
    let json = scripts_json(&fonts);
    for id in ALL_FONT_IDS {
        match named(id) {
            Some(key) => {
                assert!(
                    json.contains(&format!("\"{key}\":")),
                    "FontId::{id:?} must appear in font.scripts as \"{key}\" — got {json}"
                );
                assert!(
                    fonts.get(id).is_some(),
                    "ScriptFontReports::get must route FontId::{id:?} to its own field"
                );
            }
            None => assert!(
                !json.contains("\"latin\":"),
                "Latin is deliberately absent from font.scripts (no override span)"
            ),
        }
    }
}
