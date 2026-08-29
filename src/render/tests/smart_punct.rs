//! Smart-punctuation display-only conceal: the painted en-dash/em-dash/
//! ellipsis substitute, the byte-identical on-caret reveal (real prose, not
//! syntax markup — must never dim like `Markup`), the row_geom reshape a
//! reveal toggle forces, and the reserved slot's fit across every world.

use super::super::*;
use super::{headless_dqp, headless_pipeline, view};

/// `wysiwyg_reveals` treats `SmartPunct` as LINE-scoped, exactly like
/// `Emphasis`/`Highlight`: it reveals only when the caret sits on its own
/// line (or a touching selection), never on a different line.
#[test]
fn wysiwyg_reveals_smart_punct_is_line_scoped() {
    use crate::markdown::ConcealKind;
    let range = 4..6;
    assert!(!super::spans::wysiwyg_reveals(
        ConcealKind::SmartPunct,
        true,
        0,
        &range,
        None
    ));
    assert!(super::spans::wysiwyg_reveals(
        ConcealKind::SmartPunct,
        false,
        4,
        &range,
        None
    ));
}

/// THE dimming carve-out: `md_attrs` treats `ConcealMarkup(SmartPunct)` as a
/// NO-OP transform — the same shape as `Heading`/`Highlight` — riding the
/// buffer's own ink unchanged, rather than falling into the "dim like markup"
/// bucket every other `ConcealMarkup` kind takes. This is what keeps the
/// caret's-own-line reveal BYTE-IDENTICAL to plain prose: a literal `--` in
/// today's editor (before this feature) carries no span at all and so no
/// color override; if `SmartPunct` fell into the dim bucket, landing the
/// caret on its line would visibly darken ordinary sentence punctuation.
#[test]
fn smart_punct_on_caret_never_dims_unlike_ordinary_markup() {
    let base = Attrs::new();
    let smart = md_attrs(
        &base,
        crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::SmartPunct),
    );
    assert_eq!(
        smart.color_opt, base.color_opt,
        "SmartPunct must be a no-op transform (rides the surrounding ink)"
    );

    // Contrast: an ORDINARY ConcealMarkup kind (e.g. Heading's '#' markers)
    // DOES dim — proving the carve-out above is a real subtraction from that
    // shared bucket, not a no-op change to the whole match.
    let heading_markup = md_attrs(
        &base,
        crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::Heading),
    );
    assert_ne!(
        heading_markup.color_opt, base.color_opt,
        "an ordinary ConcealMarkup kind must still dim, so the contrast is real"
    );
}

/// END-TO-END: off the caret's line a smart-punct run conceals to zero width
/// (transparent ink, the `concealed_at` oracle); on the caret's own line it
/// reveals — and its color is the plain default ink, never the muted "dim
/// the markup" color a heading's `#` or emphasis's `**` would carry on their
/// own caret-revealed line. NON-VACUITY for the reveal/conceal toggle itself:
/// commenting out the `code_block == 0` guard's `push_smart_punct_spans` call
/// in `parse.rs` (checked by hand while writing this law) makes `off` assert
/// false immediately, since no span exists to conceal at all.
#[test]
fn wysiwyg_smart_punct_conceals_off_cursor_and_reveals_byte_identical_on() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping wysiwyg_smart_punct_conceals_off_cursor_and_reveals_byte_identical_on: no wgpu adapter"
        );
        return;
    };
    // Line 0: prose with an en dash run. Line 1: plain prose (caret parks here).
    let text = "double -- dash here\nprose\n";
    let dash_byte = text.find("--").unwrap();

    let mut off = view(text, 1, 0);
    off.is_markdown = true;
    p.set_view(&off);
    assert!(
        p.concealed_at(0, dash_byte),
        "the en dash run conceals off the caret's line"
    );

    let mut on = view(text, 0, 0);
    on.is_markdown = true;
    p.set_view(&on);
    assert!(
        !p.concealed_at(0, dash_byte),
        "caret on the line reveals the literal '--'"
    );
    let revealed_color = p.buffer.lines[0]
        .attrs_list()
        .get_span(dash_byte)
        .color_opt;
    let plain_color = p.buffer.lines[0].attrs_list().get_span(0).color_opt;
    assert_eq!(
        revealed_color, plain_color,
        "the revealed '--' must ride the SAME ink as the surrounding plain prose \
         (never the muted 'dim the markup' color a heading/emphasis marker gets)"
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// The reveal toggle changes GLYPH ADVANCES, not just color (the
/// `refresh_rule_conceal` tripwire): moving the caret onto/off a smart-punct
/// line must reshape `row_geom`, not serve a stale cached layout. Compares
/// `VisualRow::xs` directly, the same shape `wysiwyg_selection_change_alone_invalidates_row_geom`
/// uses for the pre-existing kinds.
#[test]
fn wysiwyg_smart_punct_reveal_invalidates_row_geom() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping wysiwyg_smart_punct_reveal_invalidates_row_geom: no wgpu adapter");
        return;
    };
    let text = "an ellipsis... here\nprose\n";

    // Caret elsewhere: the run collapses to near-zero advance.
    let mut off = view(text, 1, 0);
    off.is_markdown = true;
    p.set_view(&off);
    let xs_off = p.visual_rows(0)[0].xs.clone();

    // Caret on the line: the run reveals at full width — row_geom must have
    // reshaped, not served the collapsed layout from the prior frame.
    let mut on = view(text, 0, 0);
    on.is_markdown = true;
    p.set_view(&on);
    let xs_on = p.visual_rows(0)[0].xs.clone();

    assert_ne!(
        xs_off.last(),
        xs_on.last(),
        "the row's total advance must differ between concealed and revealed \
         (collapsed vs literal '...' glyphs): off={xs_off:?} on={xs_on:?}"
    );

    // And back off again: re-conceals, proving the toggle round-trips rather
    // than sticking at whichever state happened to shape first.
    let mut off_again = view(text, 1, 0);
    off_again.is_markdown = true;
    p.set_view(&off_again);
    let xs_off_again = p.visual_rows(0)[0].xs.clone();
    assert_eq!(
        xs_off.last(),
        xs_off_again.last(),
        "re-conceal must reproduce the SAME collapsed advance, not drift"
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// The reserved substitute-glyph slot (`smart_punct_slot`) is a FIXED,
/// `line_height`-only formula shared by all three kinds — it has to cover the
/// widest real shaped glyph (the ellipsis) in EVERY world's own display face,
/// not just whichever world a hand-picked example happens to hit.
/// `SmartPunctGlyphs::append_areas` carries a `debug_assert!` for exactly
/// this; NO-WILDCARD over `theme::THEMES` through the real GPU prepare pass
/// is what actually exercises it (the `bare_url_ellipsis_slot_fits_the_real_glyph_in_every_world`
/// precedent this mirrors).
#[test]
fn smart_punct_slot_fits_the_real_glyph_in_every_world() {
    let _t = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping smart_punct_slot_fits_the_real_glyph_in_every_world: no wgpu adapter");
        return;
    };
    let w = 1200u32;
    let h = 800u32;
    // Caret parks on the blank line 1 so line 0's three runs all conceal and
    // paint their ornaments (reveal-on-cursor).
    let text = "two -- three --- ellipsis... now\n\n";
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut v = view(text, 1, 0);
        v.is_markdown = true;
        p.set_view(&v);
        assert_eq!(
            p.smart_punct_marks().len(),
            3,
            "{}: all three runs must produce a mark — this law would be testing nothing",
            t.name
        );
        // The `debug_assert!` inside `SmartPunctGlyphs::append_areas` IS this
        // law's real assertion; a slot too small for the real glyph panics
        // here.
        p.prepare(&device, &queue, w, h).unwrap();
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::markdown::set_wysiwyg_on(true);
}

/// `smart_punct_marks` re-derives WHICH glyph from the concealed span's own
/// raw bytes rather than carrying it separately — proven by checking each of
/// the three kinds resolves to a DISTINCT glyph in document order.
#[test]
fn smart_punct_marks_resolve_each_kind_to_its_own_glyph() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping smart_punct_marks_resolve_each_kind_to_its_own_glyph: no wgpu adapter");
        return;
    };
    let text = "two -- three --- ellipsis... now\nprose\n";
    let mut v = view(text, 1, 0);
    v.is_markdown = true;
    p.set_view(&v);
    let kinds: Vec<_> = p.smart_punct_marks().into_iter().map(|(_, _, k, _)| k).collect();
    assert_eq!(
        kinds,
        vec![
            crate::markdown::SmartPunctKind::EnDash,
            crate::markdown::SmartPunctKind::EmDash,
            crate::markdown::SmartPunctKind::Ellipsis,
        ],
        "each run resolves to its own distinct kind, in document order: {kinds:?}"
    );

    crate::markdown::set_wysiwyg_on(true);
}
