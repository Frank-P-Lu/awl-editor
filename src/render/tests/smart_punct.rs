//! Smart-punctuation display-only conceal: the painted en-dash/em-dash/
//! ellipsis substitute, the byte-identical on-caret reveal (real prose, not
//! syntax markup — must never dim like `Markup`), the row_geom reshape a
//! reveal toggle forces, and measured-advance agreement across every world.

use super::super::*;
use super::{headless_dqp, headless_pipeline, view};

mod pixels;

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
            "skipping wysiwyg_smart_punct_conceals_off_cursor_and_reveals_on: no wgpu adapter"
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
    let revealed_color = p.buffer.lines[0].attrs_list().get_span(dash_byte).color_opt;
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

/// Each substitute-glyph slot is that KIND's own real shaped body-text advance
/// in EVERY world's display face, not a fixed widest-glyph placeholder.
/// `SmartPunctGlyphs::append_areas` carries a `debug_assert!` for exactly
/// this; NO-WILDCARD over `theme::THEMES` through the real GPU prepare pass
/// is what actually exercises it (the `bare_url_ellipsis_slot_fits_the_real_glyph_in_every_world`
/// precedent this mirrors).
#[test]
fn smart_punct_advance_matches_the_real_glyph_in_every_world() {
    let _t = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping smart_punct_advance_matches_the_real_glyph_in_every_world: no wgpu adapter"
        );
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
    let kinds: Vec<_> = p
        .smart_punct_marks()
        .into_iter()
        .map(|(_, _, k, _)| k)
        .collect();
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

/// END-TO-END selection reveal: touching a smart-punctuation source range
/// restores every literal byte AND suppresses the separately-painted
/// substitute. The disjoint state on either side proves both subjects were
/// enrolled; a test that only inspected concealment could leave the ornament
/// painted over the revealed literal, while a mark-count-only test could leave
/// the literal collapsed underneath an absent ornament.
#[test]
fn smart_punct_selection_touch_suppresses_conceal_and_ornament_for_full_roster() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping smart_punct_selection_touch_suppresses_conceal_and_ornament_for_full_roster: \
             no wgpu adapter"
        );
        return;
    };
    let text = "range -- pause --- wait... now\nplain\npark\n";
    let offsets: Vec<_> = crate::markdown::SmartPunctKind::ALL
        .into_iter()
        .map(|kind| (kind, text.find(kind.literal()).unwrap()))
        .collect();

    let mut disjoint = view(text, 2, 0);
    disjoint.is_markdown = true;
    disjoint.selection = Some(((1, 0), (1, 1)));
    p.set_view(&disjoint);
    let disjoint_marks: Vec<_> = p
        .smart_punct_marks()
        .into_iter()
        .map(|(_, _, kind, _)| kind)
        .collect();
    assert_eq!(
        disjoint_marks,
        crate::markdown::SmartPunctKind::ALL,
        "disjoint selection must enroll one painted substitute per roster member"
    );
    for (kind, byte) in &offsets {
        assert!(
            p.concealed_at(0, *byte),
            "{kind:?}: disjoint selection must leave the source literal concealed"
        );
    }

    let mut touching = view(text, 2, 0);
    touching.is_markdown = true;
    touching.selection = Some(((0, 0), (0, 1)));
    p.set_view(&touching);
    for (kind, byte) in &offsets {
        assert!(
            !p.concealed_at(0, *byte),
            "{kind:?}: a selection touching the line must reveal the literal"
        );
    }
    assert!(
        p.smart_punct_marks().is_empty(),
        "a selection touching the line must suppress every substitute ornament"
    );

    p.set_view(&disjoint);
    assert_eq!(
        p.smart_punct_marks().len(),
        crate::markdown::SmartPunctKind::ALL.len(),
        "clearing the touching selection must restore every ornament"
    );
}

/// END-TO-END table exception: ordinary prose conceals the three ASCII runs
/// and paints substitutes, but the real grid-cell buffers intentionally keep
/// the raw literals in the table area's live default ink because the table
/// painter has no per-cell smart-punctuation ornament layer. This reads the shaped buffer that
/// `prepare_table_grid` submits to the renderer, not a parser-only attrs list.
#[test]
fn table_cell_smart_punct_stays_literal_visible_without_an_ornament_layer() {
    let _w = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping table_cell_smart_punct_stays_literal_visible_without_an_ornament_layer: \
             no wgpu adapter"
        );
        return;
    };
    let cell = "range -- pause --- wait... now";
    let text = format!("| Mark |\n| --- |\n| {cell} |\n\npark\n");
    let mut v = view(&text, 4, 0);
    v.is_markdown = true;
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    assert!(
        p.table_cell_lines_drawn().contains(&2),
        "the body row must be submitted as real grid-cell text"
    );
    let cache = p.table_grid_cache.entries.borrow();
    let (_, grid) = cache.first().expect("one shaped table grid");
    let (_, _, body, _) = grid
        .cells
        .iter()
        .find(|(_, _, buffer, _)| buffer.lines.first().is_some_and(|line| line.text() == cell))
        .expect("body cell carrying the smart-punctuation roster is shaped");
    let line = &body.lines[0];
    for kind in crate::markdown::SmartPunctKind::ALL {
        let byte = cell.find(kind.literal()).unwrap();
        let attrs = line.attrs_list().get_span(byte);
        assert_eq!(
            attrs.color_opt, None,
            "{kind:?}: table-cell literal inherits live default ink (never a stale authored tint)"
        );
        assert!(
            attrs.metrics_opt.is_none(),
            "{kind:?}: table-cell literal keeps full body metrics (never collapsed conceal)"
        );
    }
}

fn smart_punct_body_advance(p: &mut TextPipeline, kind: crate::markdown::SmartPunctKind) -> f32 {
    let glyph_metrics = GlyphMetrics::new(p.metrics.font_size, p.metrics.line_height);
    let attrs = p.doc_attrs();
    let mut buffer = GlyphBuffer::new(&mut p.font_system, glyph_metrics);
    buffer.set_size(
        &mut p.font_system,
        Some(p.metrics.line_height * 2.0),
        Some(p.metrics.line_height),
    );
    buffer.set_text(
        &mut p.font_system,
        &kind.glyph().to_string(),
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut p.font_system, false);
    buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0f32, f32::max)
}

/// HEADLINE LAW — every smart-punctuation reserve uses the active world's
/// BODY shaping: family, effective weight, features, and full body metrics.
///
/// The ornament is an isolated run, so its independent control is the same
/// glyph shaped in isolation through `doc_attrs`; an inline Unicode control
/// would add contextual shaping that the ornament does not own. The roster
/// loop has no wildcard, so a new `SmartPunctKind` cannot silently escape.
#[test]
fn smart_punct_advances_use_each_worlds_body_shaping_no_wildcard() {
    let _w = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping smart_punct_advances_use_each_worlds_body_shaping_\
             no_wildcard: no wgpu adapter"
        );
        return;
    };

    let mut graded = 0usize;
    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for kind in crate::markdown::SmartPunctKind::ALL {
            use crate::markdown::SmartPunctKind;
            match kind {
                SmartPunctKind::EnDash | SmartPunctKind::EmDash | SmartPunctKind::Ellipsis => {}
            }
            let reserve = p.smart_punct_advances.advance(kind);
            let body = smart_punct_body_advance(&mut p, kind);
            assert_eq!(
                reserve, body,
                "{} {kind:?}: smart-punctuation reserve must use the world's exact body \
                 shaping; reserve={reserve:.6} body={body:.6}",
                world.name,
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * crate::markdown::SmartPunctKind::ALL.len()
    );
    crate::markdown::set_wysiwyg_on(true);
}
