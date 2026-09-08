//! **THE FIND/REPLACE PANEL'S SELECTION BAND IS REAL INK, ON THE FOCUSED
//! ROW, INSIDE THE FIXED-WIDTH FIELD.**
//!
//! The panel's select-all verb arms a selection the writer is about to type
//! over. A mode that changes what the next keystroke does and shows nothing
//! is worse than the routing bug it came from, so the band is asserted as
//! PIXELS (a real difference against the identical frame with nothing
//! selected) and as GEOMETRY (its two edges resolve through the same shaped
//! glyph scan the amber caret rides, on the same row, and a SCROLLED field's
//! band clips to the visible window instead of painting past it).
//!
//! Both halves are needed: geometry alone is satisfied by a band drawn in the
//! page colour, and "some pixels changed" alone is satisfied by a band on the
//! wrong row.

use super::super::*;
use super::pixeldiff::Region;
use super::{headless_dqp, headless_pipeline, pixeldiff, view};

const FIND_LABEL_LEN: usize = "find    ".len();
const REPLACE_LABEL_LEN: usize = "replace ".len();
const QUERY: &str = "hello";
/// Deliberately MULTIBYTE, and a different CHAR length from [`QUERY`]. The two
/// row labels are the same width and both fields are padded to the same cell
/// count, so on two ASCII fixtures a band computed off the WRONG field is
/// arithmetically identical to the right one — the focus axis this file sweeps
/// stops being an axis at all. A CJK field makes byte offsets and char indices
/// disagree, which is the only thing that can tell the two rows apart.
const REPLACEMENT: &str = "日本語";

/// A panel view over a two-line document, focused on one of its two fields.
fn panel_view(replacement_focused: bool, query: &str, replacement: &str) -> ViewState {
    let mut v = view("hello\nhello\n", 0, 0);
    v.search_active = true;
    v.search_query = query.into();
    v.search_query_caret = query.chars().count();
    v.search_matches = vec![((0, 0), (0, 5)), ((1, 0), (1, 5))];
    v.search_current = Some(0);
    v.search_replace_active = true;
    v.search_replacement = replacement.into();
    v.search_replacement_caret = replacement.chars().count();
    v.search_editing_replacement = replacement_focused;
    v
}

/// **THE BAND IS VISIBLE, AND ONLY WHERE IT SHOULD BE.**
///
/// Two frames that differ in exactly one bit of state — the focused field's
/// selection — are compared. The find row must change; the replace row and
/// the document body beneath the card must not. The presence half is the
/// `differing` count itself: a band that drew nothing (or drew in the card's
/// own colour) leaves the two frames identical and fails here rather than
/// passing quietly.
#[test]
fn the_panel_selection_band_paints_only_the_focused_row() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_panel_selection_band_paints_only_the_focused_row: no wgpu adapter");
        return;
    };

    let bare = panel_view(false, QUERY, REPLACEMENT);
    p.set_view(&bare);
    p.prepare(&device, &queue, w, h).unwrap();
    let without = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

    let mut selected = panel_view(false, QUERY, REPLACEMENT);
    selected.search_field_selection = Some((0, QUERY.chars().count()));
    p.set_view(&selected);
    p.prepare(&device, &queue, w, h).unwrap();
    let with = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

    // Row bands, read from the panel's own row owner rather than guessed.
    let shape = p.panel_shape_text(w);
    let (card, _text_left, text_top, _caret_x) = p.panel_layout(
        w,
        shape.caret_byte,
        shape.caret_fallback_chars,
        shape.caret_row,
    );
    let lh = p.metrics.line_height;
    let row = |i: f32| Region::new(card[0], text_top + i * lh, card[2], lh);

    let find_row = pixeldiff::diff_region(&without, &with, w as i64, h as i64, row(0.0));
    assert!(
        find_row.differing > 0,
        "the selection band painted NOTHING on the focused find row — the verb \
         would arm a mode with no visible report (differing={}, max delta={})",
        find_row.differing,
        find_row.max_channel_delta
    );

    let replace_row = pixeldiff::diff_region(&without, &with, w as i64, h as i64, row(1.0));
    assert_eq!(
        replace_row.differing, 0,
        "the find field's band leaked onto the REPLACE row"
    );

    // Everything below the card is the parked document — untouched.
    let below = Region::new(
        0.0,
        card[1] + card[3] + 1.0,
        w as f32,
        h as f32 - (card[1] + card[3] + 1.0),
    );
    let doc = pixeldiff::diff_region(&without, &with, w as i64, h as i64, below);
    assert_eq!(
        doc.differing, 0,
        "the panel's band changed pixels in the document parked behind it"
    );
}

/// **THE BAND'S EDGES COME THROUGH THE SAME GLYPH SCAN THE CARET RIDES**, on
/// whichever row has focus — so a proportional world cannot leave the band
/// beside its own text. Both fields are driven, because the row offset is
/// exactly what the caret's own regression got wrong once already.
#[test]
fn the_band_spans_the_shaped_field_on_whichever_row_has_focus() {
    let _t = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    // A PROPORTIONAL world, so the shaped advance genuinely differs from the
    // char-pitch fallback and a band placed by the fallback is caught.
    crate::theme::set_active_by_name("Gumtree").unwrap();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping the_band_spans_the_shaped_field_on_whichever_row_has_focus: no wgpu adapter"
        );
        return;
    };
    let width = 1200u32;

    for (replacement_focused, label_len, row) in [
        (false, FIND_LABEL_LEN, 0.0_f32),
        (true, REPLACE_LABEL_LEN, 1.0_f32),
    ] {
        let field = if replacement_focused {
            REPLACEMENT
        } else {
            QUERY
        };
        let mut v = panel_view(replacement_focused, QUERY, REPLACEMENT);
        v.search_field_selection = Some((0, field.chars().count()));
        p.set_view(&v);

        let shape = p.panel_shape_text(width);
        assert_eq!(
            shape.caret_row, row,
            "focus on replacement={replacement_focused} must target row {row}"
        );
        let ((s_byte, s_chars), (e_byte, e_chars)) = shape
            .selection_span
            .expect("an armed field selection must produce a band span");
        assert_eq!(
            (s_byte, e_byte),
            (label_len, label_len + field.len()),
            "the band's byte offsets are LINE-relative, past this row's own label"
        );

        let (_card, text_left, _top, _caret_x) = p.panel_layout(
            width,
            shape.caret_byte,
            shape.caret_fallback_chars,
            shape.caret_row,
        );
        let x0 = p.panel_glyph_x(row, s_byte, s_chars, text_left);
        let x1 = p.panel_glyph_x(row, e_byte, e_chars, text_left);
        assert!(
            x1 > x0,
            "the band has no width on row {row} (x0={x0}, x1={x1})"
        );

        // INDEPENDENT ground truth: scan this row's glyphs for the field's own
        // first cell and the cell just past it, without the span arithmetic
        // under test.
        let mut want0 = None;
        let mut want1 = None;
        for run in p.panel_buffer.layout_runs() {
            if run.line_i != row as usize {
                continue;
            }
            for g in run.glyphs.iter() {
                if g.start == label_len {
                    want0 = Some(text_left + g.x);
                }
                if g.start == label_len + field.len() {
                    want1 = Some(text_left + g.x);
                }
            }
        }
        let want0 = want0.expect("the field's first glyph is on this row");
        let want1 = want1.expect("the cell past the field is on this row");
        assert!(
            (x0 - want0).abs() < 0.5 && (x1 - want1).abs() < 0.5,
            "band edges must be the SHAPED advances (got {x0}..{x1}, want {want0}..{want1}) \
             on row {row}"
        );
        // And not the hardcoded char-pitch fallback, which is where a
        // row-mismatched offset lands.
        let fallback = text_left + p.metrics.char_width * s_chars as f32;
        assert!(
            (x0 - fallback).abs() > 0.5,
            "on a proportional world the band start is NOT the char-pitch fallback \
             (x0={x0}, fallback={fallback}) on row {row}"
        );
    }
}

/// **A FIELD SCROLLED PAST ITS FIXED WIDTH CLIPS ITS BAND TO THE VISIBLE
/// WINDOW.** The panel's card geometry is invariant to what is typed
/// (`field_view_window`'s whole point), so a band derived from the FULL field
/// offsets would paint outside the card. The band's edges must be crossed by
/// the same window rule the caret is.
///
/// The companion is the short field: with nothing scrolled, the same span
/// starts at the field's first cell — otherwise "it clipped" would be true of
/// a band that always collapsed.
#[test]
fn a_scrolled_field_clips_its_band_to_the_visible_window() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_scrolled_field_clips_its_band_to_the_visible_window: no wgpu adapter"
        );
        return;
    };
    let width = 1200u32;
    let long: String = std::iter::repeat_n('q', 60).collect();

    let mut v = panel_view(false, &long, REPLACEMENT);
    v.search_field_selection = Some((0, long.chars().count()));
    p.set_view(&v);
    let shape = p.panel_shape_text(width);
    let ((s_byte, _), (e_byte, _)) = shape
        .selection_span
        .expect("a scrolled field still has a visible band");
    let visible_cells = e_byte - s_byte;
    assert!(
        visible_cells > 0 && visible_cells < long.len(),
        "a 60-char field must clip to the panel's fixed window, not paint all of \
         it (visible cells: {visible_cells})"
    );

    // A PARTIAL selection in the SAME scrolled field is what the window
    // crossing actually exists for: a full-field band clips to the same cells
    // whether or not the offset is subtracted, so a law that only ever selects
    // everything cannot see the rule it is testing. Chars 40..50 of a 60-char
    // field sit inside the trailing window and must land INSIDE the field's
    // cells, not at its start.
    let mut partial = panel_view(false, &long, REPLACEMENT);
    partial.search_field_selection = Some((40, 50));
    p.set_view(&partial);
    let shape = p.panel_shape_text(width);
    let ((s_byte, _), (e_byte, _)) = shape
        .selection_span
        .expect("a partial selection inside the window still has a band");
    assert!(
        s_byte > FIND_LABEL_LEN,
        "a partial selection in a SCROLLED field must be crossed by the same \
         window offset the caret is — it started at the field's first cell \
         (s_byte={s_byte}, label={FIND_LABEL_LEN}), which is where an \
         un-crossed raw offset lands"
    );
    assert_eq!(
        e_byte - s_byte,
        10,
        "the crossed span keeps the selection's own width"
    );

    // The short-field companion: nothing to clip, so the whole field spans.
    let mut short = panel_view(false, QUERY, REPLACEMENT);
    short.search_field_selection = Some((0, QUERY.chars().count()));
    p.set_view(&short);
    let shape = p.panel_shape_text(width);
    let ((s_byte, _), (e_byte, _)) = shape.selection_span.expect("the short field has a band");
    assert_eq!(
        (s_byte, e_byte),
        (FIND_LABEL_LEN, FIND_LABEL_LEN + QUERY.len()),
        "an unscrolled field's band spans the whole field"
    );

    // And with nothing selected there is no span at all — the state every
    // other panel frame is in.
    let none = panel_view(false, QUERY, REPLACEMENT);
    p.set_view(&none);
    assert!(
        p.panel_shape_text(width).selection_span.is_none(),
        "an unarmed field must produce no band"
    );
}
