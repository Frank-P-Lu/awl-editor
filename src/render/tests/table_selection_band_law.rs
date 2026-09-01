//! THE TABLE-ROW SELECTION-BAND LAW.
//!
//! A GFM table row the active selection touches gets its raw pipe-delimited
//! source floated over the still-drawn grid, at its REAL shaped advances
//! (`TextPipeline::prepare_table_xray`, the X-RAY — see `render::XrayRow`'s own
//! doc). Before this file's fix, `range_rects`/`selection_rects`
//! (`rects.rs`) kept reading the row's CONCEALED doc geometry for that row's
//! width — the same zero-width collapse `ensure_wash_protos`'s table carve-out
//! already documents for the comment/highlight washes — while the row was
//! SIMULTANEOUSLY revealed and drawn from the x-ray's own real advances, so the
//! selection band's geometry and the visible ink disagreed: a hairline sliver
//! at the left margin beside a full-width row of revealed text. The fix reads
//! the row's OWN `XrayRow::glyph_xs` (`geometry::xray_x_span`, mirroring the
//! caret's pre-existing `xray_col_x` redirect in `col_x_and_advance_aff`)
//! whenever the selected line is x-rayed.
//!
//! PRESENCE FLOOR, non-vacuous: every width assertion below compares the
//! band's pixel width against the SAME `glyph_xs` the drawn x-ray float paints
//! from (never a value this file re-derives independently), plus a
//! [`REPORTED_SLIVER_CEILING_PX`] floor (for the selections wide enough to
//! make it a meaningful check) — so a band that regresses toward zero width,
//! or merely shrinks BACK toward the sliver without hitting exactly zero,
//! both fail. Reverting the
//! `xray_x_span` call sites in `rects.rs::range_rects` (falling back to the
//! concealed `row.xs`) is this file's own mutation proof: every test below
//! goes red, because the reverted code makes `rects[i][2]` collapse to
//! (near-)zero while `xray_ink_width` still reports the real, wide advance.
//!
//! CONTROL: [`raw_mode_table_selection_already_bands_full_row_width`] asserts
//! WYSIWYG-off selection (no conceal, no x-ray — the plain `rows_by_line`
//! path this file never touches) was ALREADY correct, so this fix's job is
//! narrowly the x-rayed case.

use super::super::*;
use super::{headless_dqp, view_md};

/// The reported bug's own observed ceiling: "per-row band slivers 5-8px wide
/// hugging the left margin". Any selection wide enough that its real,
/// multi-glyph ink cannot plausibly be mistaken for that hairline (several
/// characters, several of them full letters rather than a bare `|`/space)
/// clearing this is a second, independent signal beyond the exact-width
/// match below — a regression back to the concealed geometry would land
/// UNDER this ceiling, not merely fail the exact-match tolerance.
const REPORTED_SLIVER_CEILING_PX: f32 = 8.0;

/// The prose/table/prose fixture from the item's own headless repro:
/// `--keys "Down Down S-Down S-Down S-Down"` over this exact text lands
/// `selection` at line 3 col 2 -> line 5 col 2 (divider row partial, the full
/// "one|two" body row, and "three|four" up to the caret) — reproduced here
/// without a key replay by setting that same selection directly.
fn table_fixture() -> String {
    "Some prose before the table.\n\
     \n\
     | Alpha | Beta |\n\
     | --- | --- |\n\
     | one | two |\n\
     | three | four |\n\
     \n\
     Some prose after the table.\n"
        .to_string()
}

/// The pixel width of x-rayed table row `line`'s REVEALED SOURCE over char
/// columns `[a, b]`, read straight from the same `XrayRow::glyph_xs` the drawn
/// float paints (`table_grid::shape_table_xray_floats` /
/// `append_table_xrays`) — so a passing comparison against a selection band
/// means the band covers the ACTUAL painted glyphs, not a width this test
/// computed independently.
fn xray_ink_width(p: &TextPipeline, line: usize, a: usize, b: usize) -> f32 {
    let x =
        p.xray.iter().find(|x| x.line == line).unwrap_or_else(|| {
            panic!("doc line {line} is not x-rayed by this fixture's selection")
        });
    let n = x.glyph_xs.len().saturating_sub(1);
    let a = a.min(n);
    let b = b.min(n);
    x.glyph_xs[b] - x.glyph_xs[a]
}

fn xray_char_count(p: &TextPipeline, line: usize) -> usize {
    p.xray
        .iter()
        .find(|x| x.line == line)
        .map(|x| x.glyph_xs.len().saturating_sub(1))
        .unwrap_or_else(|| panic!("doc line {line} is not x-rayed by this fixture's selection"))
}

/// THE REPORTED BUG, REPRODUCED AND ASSERTED FIXED: the item's exact `--keys`
/// repro selection (divider row partial, full body row, partial body row) over
/// the prose/table/prose fixture. Every touched row's band width must equal
/// its revealed ink's width (plus the same trailing-selection eol pad the
/// plain text path grants a line that extends to its own end) and clear the
/// sliver floor — not the ~5-8px hairline the report photographed.
#[test]
fn table_selection_band_reproduces_the_report_and_covers_full_row_ink() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _g = crate::testlock::serial();
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping table_selection_band_reproduces_the_report_and_covers_full_row_ink: no \
             wgpu adapter"
        );
        return;
    };
    let text = table_fixture();
    let mut v = view_md(&text, 5, 2);
    v.selection = Some(((3, 2), (5, 2)));
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    assert_eq!(p.tables_report().len(), 1, "one table laid out");
    assert_eq!(
        p.xray_lines_report().len(),
        3,
        "all three selection-touched rows (divider, both body rows) are x-rayed: {:?}",
        p.xray_lines_report()
    );

    let eol_pad = p.metrics.char_width * 0.5;
    let rects = p.range_rects((3, 2), (5, 2));
    assert_eq!(
        rects.len(),
        3,
        "one band per selected table row (divider, body, body): {rects:?}"
    );

    // Divider row (doc line 3): partial, col 2 through its own end -> gets the
    // trailing eol pad (it does not reach the selection's last line).
    let n3 = xray_char_count(&p, 3) - 2;
    let want3 = xray_ink_width(&p, 3, 2, xray_char_count(&p, 3)) + eol_pad;
    assert!(
        (rects[0][2] - want3).abs() < 0.5,
        "divider row band width {} != revealed ink width {want3}",
        rects[0][2]
    );
    assert!(
        rects[0][2] > REPORTED_SLIVER_CEILING_PX,
        "divider row band is a sliver: {}px ({n3} chars selected)",
        rects[0][2]
    );

    // Body row "| one | two |" (doc line 4): a full middle line -> whole row +
    // eol pad.
    let n4 = xray_char_count(&p, 4);
    let want4 = xray_ink_width(&p, 4, 0, n4) + eol_pad;
    assert!(
        (rects[1][2] - want4).abs() < 0.5,
        "full body row band width {} != revealed ink width {want4}",
        rects[1][2]
    );
    assert!(
        rects[1][2] > REPORTED_SLIVER_CEILING_PX,
        "full body row band is a sliver: {}px ({n4} chars selected)",
        rects[1][2]
    );

    // Body row "| three | four |" (doc line 5): the selection's LAST line,
    // partial 0..2 -> no eol pad (the code's `extends_to_eol` is fixed false
    // for the last line regardless of where c1 lands). Only 2 narrow glyphs
    // (`|` and a space) are selected here, so the ceiling check above isn't a
    // meaningful discriminator at this width — the exact-match assertion
    // against `xray_ink_width` (the same real advances the drawn float
    // paints) is this case's non-vacuity proof instead: a regression to the
    // concealed geometry would fail that match, not merely dip under a floor
    // sized for wider selections.
    let want5 = xray_ink_width(&p, 5, 0, 2);
    assert!(
        (rects[2][2] - want5).abs() < 0.5,
        "partial last-row band width {} != revealed ink width {want5}",
        rects[2][2]
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// ENDPOINTS INSIDE CELLS: a selection that starts and ends INSIDE a single
/// cell's text (never touching the row's `|` edges) still bands under the
/// exact revealed substring, not the whole row and not a sliver.
#[test]
fn table_selection_band_endpoints_inside_a_cell() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _g = crate::testlock::serial();
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping table_selection_band_endpoints_inside_a_cell: no wgpu adapter");
        return;
    };
    let text = table_fixture();
    // Doc line 4 is "| one | two |": columns 8..11 are exactly "two", inside
    // the second cell, touching neither the leading nor the trailing pipe.
    let mut v = view_md(&text, 0, 0); // caret well outside the table
    v.selection = Some(((4, 8), (4, 11)));
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    assert_eq!(
        p.xray_lines_report()
            .iter()
            .find(|(l, _)| *l == 4)
            .map(|(_, s)| s.clone()),
        Some("| one | two |".to_string()),
        "doc line 4 floats its exact raw source: {:?}",
        p.xray_lines_report()
    );

    let rects = p.range_rects((4, 8), (4, 11));
    assert_eq!(
        rects.len(),
        1,
        "one band for the single touched row: {rects:?}"
    );
    let want = xray_ink_width(&p, 4, 8, 11);
    assert!(
        (rects[0][2] - want).abs() < 0.5,
        "cell-interior band width {} != revealed ink width {want}",
        rects[0][2]
    );
    // Real coverage of "two" (3 glyphs), not the whole row and not a sliver.
    let full_row = xray_ink_width(&p, 4, 0, xray_char_count(&p, 4));
    assert!(
        rects[0][2] > 5.0 && rects[0][2] < full_row - 5.0,
        "band ({}) covers only the touched cell text, not the whole row ({full_row}) and not \
         nothing",
        rects[0][2]
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// HEADER + DIVIDER ROWS, PARTIAL FIRST/LAST LINES: a selection starting
/// mid-header and ending mid-divider covers the header's markup and dashes —
/// rows that carry no ordinary "cell content" at all — and both still band
/// under their real revealed advances.
#[test]
fn table_selection_band_covers_header_and_divider_partial_first_last() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _g = crate::testlock::serial();
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping table_selection_band_covers_header_and_divider_partial_first_last: no \
             wgpu adapter"
        );
        return;
    };
    let text = table_fixture();
    // Doc line 2 "| Alpha | Beta |": col 2 is inside "Alpha" (skips the
    // leading "| "). Doc line 3 "| --- | --- |": col 5 is inside the second
    // dash run. Selection: mid-header -> mid-divider, caret on the divider.
    let mut v = view_md(&text, 3, 5);
    v.selection = Some(((2, 2), (3, 5)));
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let eol_pad = p.metrics.char_width * 0.5;
    let rects = p.range_rects((2, 2), (3, 5));
    assert_eq!(
        rects.len(),
        2,
        "one band each for header + divider: {rects:?}"
    );

    // Header (partial FIRST line, col 2 -> its own end): eol pad applies.
    let n_header = xray_char_count(&p, 2) - 2;
    let want_header = xray_ink_width(&p, 2, 2, xray_char_count(&p, 2)) + eol_pad;
    assert!(
        (rects[0][2] - want_header).abs() < 0.5,
        "header row band width {} != revealed ink width {want_header}",
        rects[0][2]
    );
    assert!(
        rects[0][2] > REPORTED_SLIVER_CEILING_PX,
        "header row band is a sliver: {}px ({n_header} chars selected)",
        rects[0][2]
    );

    // Divider (partial LAST line, 0 -> col 5): no eol pad.
    let want_divider = xray_ink_width(&p, 3, 0, 5);
    assert!(
        (rects[1][2] - want_divider).abs() < 0.5,
        "divider row band width {} != revealed ink width {want_divider}",
        rects[1][2]
    );
    assert!(
        rects[1][2] > REPORTED_SLIVER_CEILING_PX,
        "divider row band is a sliver: {}px (5 chars selected)",
        rects[1][2]
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// THE WRAPPED TALL-ROW CASE (`ensure_wash_protos`'s own doc: "in tables-v1
/// the source row was one line tall so the sliver was invisible; the
/// wrap-not-clip round's tall rows made it show"). A row whose CELL wraps
/// reserves a tall grid row, but the revealed SOURCE still floats as one
/// NON-WRAPPING line (`Wrap::None` in `prepare_table_xray`) — so the band
/// must cover the full source width despite the tall row, AND stay
/// BODY-height (never stretched to the tall row — `caret_band_scale`'s
/// pre-existing x-ray override), so this axis doesn't trade one bug for
/// another.
#[test]
fn table_selection_band_on_a_wrapped_tall_row_covers_full_width_at_body_height() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _g = crate::testlock::serial();
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    crate::page::set_measure(40);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping table_selection_band_on_a_wrapped_tall_row_covers_full_width_at_body_\
             height: no wgpu adapter"
        );
        return;
    };
    // A cell whose content far exceeds its column, forced to wrap onto several
    // lines and reserve a tall grid row (mirrors `wide_table_wraps_and_reserves_\
    // a_tall_row_while_a_short_row_does_not`).
    let long = "pale eucalyptus-green with a very long description that keeps going well past \
                any single column width so it is forced to wrap onto several lines";
    let text =
        format!("| World | Ground |\n|-------|--------|\n| Short | {long} |\n| Tiny | ok |\n");
    let mut v = view_md(&text, 0, 0); // caret outside the table
    v.selection = Some(((2, 0), (2, 8))); // touches only the tall row, "| Short "
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let heights = p.compute_table_layout(&text, &crate::markdown::spans(&text));
    let tall = heights[2].expect("the selection-touched row reserves a tall row");
    assert!(
        tall > p.metrics.line_height * 1.5,
        "fixture sanity: row 2 is genuinely tall ({tall}, lh {})",
        p.metrics.line_height
    );

    let rects = p.range_rects((2, 0), (2, 8));
    assert_eq!(rects.len(), 1, "one band for the tall row: {rects:?}");
    let want = xray_ink_width(&p, 2, 0, 8);
    assert!(
        (rects[0][2] - want).abs() < 0.5,
        "tall-row band width {} != revealed ink width {want} (not the tall row's own reserved \
         height/width)",
        rects[0][2]
    );
    assert!(
        rects[0][2] > REPORTED_SLIVER_CEILING_PX,
        "tall-row band is a sliver: {}px (8 chars selected)",
        rects[0][2]
    );
    // Height stays BODY-size (the caret band), not the tall reserved row.
    assert!(
        (rects[0][3] - p.metrics.caret_h).abs() < 0.5,
        "tall-row band height {} != body caret height {} — it must not balloon to the tall \
         row's reserved height",
        rects[0][3],
        p.metrics.caret_h
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// CONTROL, asserted already-correct (not part of this fix): with WYSIWYG
/// off, a table's raw `|`-delimited source is never concealed and there is no
/// x-ray — `range_rects` walks the SAME `rows_by_line` path plain prose uses,
/// which already measures real advances. A selection over a table row bands
/// its full row width exactly like any other line.
#[test]
fn raw_mode_table_selection_already_bands_full_row_width() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _g = crate::testlock::serial();
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(false);
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping raw_mode_table_selection_already_bands_full_row_width: no wgpu adapter"
        );
        return;
    };
    let text = table_fixture();
    let mut v = view_md(&text, 4, 0);
    v.selection = Some(((4, 0), (4, 13))); // "| one | two |" is 13 chars
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    assert!(
        p.xray_lines_report().is_empty(),
        "WYSIWYG off: no x-ray, no conceal at all — {:?}",
        p.xray_lines_report()
    );

    let rects = p.range_rects((4, 0), (4, 13));
    assert_eq!(rects.len(), 1, "one band for the selected row: {rects:?}");
    // Plain-text row width: the row's own concealed-free `xs` boundary at col
    // 13, read straight off `visual_rows` (the ordinary, never-concealed
    // path) — the SAME oracle the plain-prose selection laws in
    // `washes.rs` use, so this control has no dependency on the fix under
    // test.
    let row = &p.visual_rows(4)[0];
    let want = row.xs[13] - row.xs[0];
    assert!(
        (rects[0][2] - want).abs() < 0.5,
        "raw-mode row band width {} != plain row width {want}",
        rects[0][2]
    );
    assert!(
        rects[0][2] > REPORTED_SLIVER_CEILING_PX,
        "raw-mode row band is unexpectedly a sliver: {}px — the control itself is broken",
        rects[0][2]
    );

    crate::markdown::set_wysiwyg_on(true);
}
