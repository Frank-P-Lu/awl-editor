use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn replay_keys_runs_palette_chain_into_overlay() {
    let _guard = crate::testlock::serial();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p g o t o RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.journey.card().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Goto),
        "palette Enter on 'Go to file' chains into the Goto overlay",
    );
}

#[test]
fn replay_keys_drives_palette_guide_and_opens_the_guide_buffer() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p g u i d e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "the palette closed itself on accept, no overlay left open"
    );
    let expected = crate::guide::render(
        crate::convention::Convention::current(),
        crate::commands::Platform::current(),
    );
    assert_eq!(
        buffer.text(),
        expected,
        "the buffer now holds the token-rendered guide text"
    );
    assert!(
        !buffer.text().contains("{{key:"),
        "no raw chord token survives in the opened guide"
    );
    assert!(
        buffer.path().is_none(),
        "headless replay never writes/loads a real on-disk guide.md"
    );
}

/// The reference manual's in-app door — proven at
/// the purest reachable tier: a Rust assertion on the REAL `replay_keys`
/// door (the same one `--keys` drives), not just `replay::classify_for`'s
/// bucket. This is exactly the class of gap where an effect that
/// classifies `Applied` while the interpreter silently drops it would still
/// pass `the_harness_reach_map_matches_the_production_classifier` (it only
/// checks the CLASSIFICATION), so the buffer's actual resulting TEXT has to be
/// asserted here too, mirroring Guide's own test above (Reference carries no
/// `{{key:}}` chord tokens to substitute, unlike Guide, so the embedded text
/// opens verbatim — see `reference_doc.rs`'s module doc).
#[test]
fn replay_keys_drives_palette_reference_and_opens_the_reference_buffer() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p r e f e r e n c e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "the palette closed itself on accept, no overlay left open"
    );
    assert_eq!(
        buffer.text(),
        crate::reference_doc::REFERENCE_MD,
        "the buffer now holds the embedded reference text verbatim"
    );
    assert!(
        buffer.path().is_none(),
        "headless replay never writes/loads a real on-disk reference.md"
    );
}

#[test]
fn replay_keys_palette_filter_surfaces_the_plain_settings_row() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y m a p").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.journey.card().expect("the palette is still open");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Command);
    assert!(
        ov.item_strings().iter().any(|s| s == "Keymap"),
        "the union corpus surfaces the plain settings row: {:?}",
        ov.item_strings()
    );
}

#[test]
fn replay_keys_palette_filters_to_a_settings_row_and_toggles_it() {
    // THE UNION ROUND: Cmd-P → "keymap" filters to the SETTINGS row "Keymap"
    // (the union palette's Settings-category row, `Keymap`) → Enter signals
    // the SAME `Effect::SettingToggle{key:"keymap"}` the Settings menu's own
    // accept would, and CLOSES the palette (the palette's "activation closes
    // it" convention). Note the honest scope boundary: `Effect::SettingToggle`
    // is a documented headless no-op (see the `Effect` match above) — flipping
    // + persisting the live keymap flavor is the live App's job
    // (`App::toggle_keymap_flavor`, unit-tested there); this replay proves the
    // dispatch reaches the toggle EFFECT end-to-end through the real keymap +
    // fuzzy filter + accept seam, not that the flavor value itself flips in a
    // capture (which the architecture never claims for any settings toggle).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y m a p RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "activating a settings row closes the palette"
    );
}

#[test]
fn replay_keys_palette_sub_picker_stamps_command_breadcrumb() {
    // Cmd-P → "theme" filters to "Switch theme…" → Enter runs OpenThemeMenu, which
    // the worklist re-dispatches into the Theme picker STAMPED return_to = Command
    // (the palette re-dispatch breadcrumb seam). Serialize on the theme lock: the
    // picker reads/reverts the process-global active theme.
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("palette chained into the theme picker");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Theme);
    assert_eq!(
        res.journey.parked_kind(),
        Some(crate::overlay::OverlayKind::Command),
        "the palette is parked beneath a palette-opened sub-picker",
    );
    crate::theme::set_active(0);
}

#[test]
fn replay_keys_palette_theme_esc_pops_back_to_palette() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET Esc").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("Esc pops back to the palette, not the buffer");
    assert_eq!(
        ov.kind,
        crate::overlay::OverlayKind::Command,
        "back at the command palette"
    );
    assert_eq!(
        res.journey.parked_kind(),
        None,
        "single-level: the resumed palette parks nothing itself"
    );
    crate::theme::set_active(0);
}

#[test]
fn replay_keys_palette_theme_keep_closes_to_buffer_not_a_recent_menu() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "keeping a palette-launched theme lands in the buffer"
    );
    assert!(
        matches!(res.accept, Some((crate::overlay::OverlayKind::Theme, _))),
        "the theme keep still committed, got {:?}",
        res.accept
    );
    crate::theme::set_active(0);
}

/// THE BUG this round fixes, end-to-end through the REAL `--keys` replay
/// (the reported symptom's actual repro, not just the pure `apply_transition`
/// unit — see `actions::tests::overlay_drive::
/// caret_picker_cancel_from_auto_restores_auto_not_a_pin` for that
/// purer-seam sibling). Riding AUTO on a PROPORTIONAL world (Gumtree ->
/// Morph), merely OPENING the Caret-style picker from the palette and
/// backing out with Esc (no pick made) must be a true no-op: a LATER
/// switch to a MONO world must still resolve Block, exactly as auto
/// always would. Before the fix, the Cancel silently pinned the caret at
/// Morph (auto's momentary resolution on Gumtree), so Potoroo (mono)
/// stayed wrongly Morph.
#[test]
fn replay_keys_caret_picker_cancel_from_auto_does_not_pin_it() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    crate::caret::clear_override();
    crate::theme::set_active_by_name("Gumtree").unwrap();
    assert!(crate::caret::is_auto());
    assert_eq!(crate::caret::mode(), crate::caret::CaretMode::Morph);

    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("s-p C a r e t Space s t y l e RET Esc Esc s-t P o t o r o o RET")
            .unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "the whole journey lands back in the buffer"
    );
    assert_eq!(
        crate::theme::active().name,
        "Potoroo",
        "the theme switch landed"
    );

    assert!(
        crate::caret::is_auto(),
        "cancelling the caret picker must not pin auto"
    );
    assert_eq!(
        crate::caret::mode(),
        crate::caret::CaretMode::Block,
        "auto correctly resolves Block on the now-active mono world"
    );

    crate::caret::clear_override();
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}
