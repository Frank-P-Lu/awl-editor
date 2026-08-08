//! **THE SEARCH PANEL'S PUBLISHED GEOMETRY, GRADED AGAINST THE INK AND THE
//! POINTER** — the device-level companion to [`crate::render::plan::panel_report`].
//!
//! Grading the report against the plan it was projected from would be a
//! tautology, so nothing here reads the projection twice. Each figure is graded
//! against something that never reads it:
//!
//! * a ROW BAND against the shaped `panel_buffer`'s own `line_top` — the y
//!   glyphon laid the row's glyphs out at, which is what the frame uploaded — and
//!   against the POINTER, probed at the band's own centre and 1.5px outside each
//!   edge;
//! * the `Aa` TOGGLE SPAN against the find row's total shaped advance (`line_w`,
//!   an accumulation glyphon keeps separately from the per-glyph `x`/`w` the
//!   span is seated on) and against the pointer at both ends, inside and out.
//!   **EXTENT, NOT ONLY ORIGIN**: a span pinned only by where it starts accepts
//!   any uniform shrinking, so the probe just OUTSIDE the published right edge
//!   must stop being the toggle, which is exactly what a halved width breaks;
//! * the CARD against the pointer's own accept/reject boundary on all four
//!   sides, and against the ink it is sized from.
//!
//! **THE MENU BAR IS AN AXIS.** Its reserve moves the card's origin, and the
//! interesting failure is a PARTIAL yield — a card that steps down while its rows
//! or its toggle stay put. So the arms are compared as a block: everything
//! vertical moves by one shared delta, everything horizontal does not move at
//! all.
//!
//! **THE ROW COUNT IS AN AXIS TOO.** A plain find panel shapes one row; the
//! replace state shapes three (field, replacement, key-hint), and only there can
//! a row-band step be wrong in a way one row cannot show.
//!
//! ⚠️ **THE NARROW-WINDOW ARM ASSERTS AGREEMENT, NEVER ONSCREEN-NESS.** This card
//! is seated from the window's right edge with no clamp of its own, so below
//! about 600 logical px its left edge genuinely goes negative. A law demanding a
//! non-negative x would be asserting a policy the product does not implement;
//! what must hold is that the report still describes wherever the card went.

use crate::render::TextPipeline;
use crate::render::chrome::PanelHit;
use crate::render::plan::PanelGeometry;
use crate::render::tests::{headless_dqp, view};

/// What the sweep graded, so a green run can be shown to have graded something.
#[derive(Default)]
struct Enrolled {
    rows: usize,
    toggles: usize,
    cells: usize,
    bars: std::collections::BTreeSet<bool>,
    row_counts: std::collections::BTreeSet<usize>,
}

/// A panel in one state, prepared for real on the shared device, plus the
/// geometry it published.
fn prepared(
    w: f32,
    h: f32,
    replace: bool,
    editing_replacement: bool,
) -> Option<(TextPipeline, PanelGeometry)> {
    prepared_at(w, h, replace, editing_replacement, 1.0)
}

/// The same, at an explicit device scale. ⚠️ **THE DPI IS AN AXIS AND EVERY
/// CAPTURE RUNS AT ONE VALUE OF IT** — this card's pad and outer margin are
/// unscaled constants while its row pitch is not, so a figure derived from the
/// pitch and a figure derived from the pad agree at exactly one scale unless the
/// arithmetic is right. A law that only ever runs at `1.0` cannot see that.
fn prepared_at(
    w: f32,
    h: f32,
    replace: bool,
    editing_replacement: bool,
    dpi: f32,
) -> Option<(TextPipeline, PanelGeometry)> {
    let (device, queue, mut p) = headless_dqp(w, h)?;
    p.set_dpi(dpi);
    let mut v = view("hello world\nhello again\n", 0, 0);
    v.search_active = true;
    v.search_query = "hello".into();
    v.search_matches = vec![((0, 0), (0, 5)), ((1, 0), (1, 5))];
    v.search_current = Some(0);
    v.search_replace_active = replace;
    v.search_replacement = if replace {
        "goodbye".into()
    } else {
        String::new()
    };
    v.search_editing_replacement = editing_replacement;
    p.set_view(&v);
    p.prepare(&device, &queue, w as u32, h as u32).ok()?;
    let g = p
        .panel_geometry()
        .expect("an active search must publish its panel");
    Some((p, g))
}

/// The find row's total shaped advance and its last-two-glyph population — the
/// INK side of the toggle-span grade. `line_w` is glyphon's own accumulated row
/// width, not the per-glyph `x + w` the span is seated on, so the two agreeing is
/// a real cross-check rather than the same number twice.
fn find_row_ink(p: &TextPipeline) -> (f32, usize) {
    for run in p.panel_buffer.layout_runs() {
        if run.line_i == 0 {
            return (run.line_w, run.glyphs.len());
        }
    }
    panic!("the find row must shape");
}

/// Every shaped row's `(index, line_top)` — the y glyphon actually laid each row
/// out at, read off the buffer the frame uploaded.
fn shaped_row_tops(p: &TextPipeline) -> Vec<(usize, f32)> {
    p.panel_buffer
        .layout_runs()
        .map(|run| (run.line_i, run.line_top))
        .collect()
}

/// Grade one prepared panel: the ink, then the pointer, on every published
/// The field a press in band `row` must resolve to — the contract `panel_hit`'s
/// own doc states (row 0 the find field, row 1 the replacement once it is
/// revealed, anything else inside the card a calm no-op). Out-of-range rows are
/// `Elsewhere` because a press above or below the bands is still in the card's
/// pad, and that is what makes a boundary probe meaningful.
fn want_at(row: i64, replace: bool) -> PanelHit {
    match row {
        0 => PanelHit::Find,
        1 if replace => PanelHit::Replace,
        _ => PanelHit::Elsewhere,
    }
}

/// Grade one prepared panel: the ink, then the pointer, on every published
/// figure. `label` names the cell in every failure.
fn grade(label: &str, p: &TextPipeline, g: &PanelGeometry, replace: bool, e: &mut Enrolled) {
    e.cells += 1;
    grade_card_and_rows(label, p, g, replace, e);
    grade_case_toggle(label, p, g, e);
}

/// The card's four sides against the pointer, and every published band against
/// the ink glyphon laid the row out at plus the pointer's own boundaries.
fn grade_card_and_rows(
    label: &str,
    p: &TextPipeline,
    g: &PanelGeometry,
    replace: bool,
    e: &mut Enrolled,
) {
    let [cx, cy, cw, ch] = g.card;
    assert!(
        cw > 40.0 && ch > 10.0,
        "{label}: a drawn card must have a real extent, got {cw}x{ch}"
    );

    // ---- the rows, against the ink glyphon laid them out at -----------------
    let tops = shaped_row_tops(p);
    assert_eq!(
        tops.len(),
        g.rows.len(),
        "{label}: the report must carry one band per SHAPED row, got {} bands \
         for {} runs",
        g.rows.len(),
        tops.len()
    );
    e.row_counts.insert(g.rows.len());
    for (band, (line_i, line_top)) in g.rows.iter().zip(tops.iter()) {
        e.rows += 1;
        assert_eq!(
            band.row, *line_i,
            "{label}: bands must be in draw order alongside the shaped runs"
        );
        // The band the pointer inverts and the y the ink was uploaded at are two
        // different owners; this is the whole agreement the block exists for.
        assert!(
            (band.top - (g.text_top + line_top)).abs() < 0.51,
            "{label}: row {} publishes top {} while its glyphs were laid out at \
             text_top {} + line_top {} = {}",
            band.row,
            band.top,
            g.text_top,
            line_top,
            g.text_top + line_top
        );
        assert!(
            band.h > 4.0,
            "{label}: row {} publishes a {}px band — a presence floor, because \
             every containment check below is satisfied by a band of zero",
            band.row,
            band.h
        );
        // Overlap with the card, never containment: nothing here steps outward
        // today, but a row is ink and the card is a rect around it, and the two
        // are separate owners.
        assert!(
            band.top + band.h > cy && band.top < cy + ch,
            "{label}: row {}'s band [{}, {}] does not meet the card's [{cy}, {}]",
            band.row,
            band.top,
            band.top + band.h,
            cy + ch
        );

        // ---- the pointer, across the band's own BOUNDARIES -----------------
        // Probing centres alone is not enough and this is measured, not assumed:
        // an inverse reading a 20%-wider pitch than the published band still maps
        // every centre to the right row, and survived exactly this law until the
        // probes moved to the edges. What pins the two owners together is the
        // TRANSITION — just inside a band is that row, and 1.5px past either edge
        // is already the neighbour.
        let mid_x = cx + cw * 0.5;
        for (dy, want) in [
            (0.5, want_at(band.row as i64, replace)),
            (band.h * 0.5, want_at(band.row as i64, replace)),
            (band.h - 0.5, want_at(band.row as i64, replace)),
            (-1.5, want_at(band.row as i64 - 1, replace)),
            (band.h + 1.5, want_at(band.row as i64 + 1, replace)),
        ] {
            let py = band.top + dy;
            assert_eq!(
                p.panel_hit(mid_x, py),
                Some(want),
                "{label}: a press at y {py} — the published band for row {} is \
                 [{}, {}] — must resolve to {want:?}",
                band.row,
                band.top,
                band.top + band.h
            );
        }
        let mid_y = band.top + band.h * 0.5;
        assert_eq!(
            p.panel_hit(cx - 1.5, mid_y),
            None,
            "{label}: 1.5px left of the published card edge must fall through to \
             the document"
        );
        assert_eq!(
            p.panel_hit(cx + cw + 1.5, mid_y),
            None,
            "{label}: 1.5px right of the published card edge must fall through"
        );
    }
    assert_eq!(
        p.panel_hit(cx + cw * 0.5, cy - 1.5),
        None,
        "{label}: 1.5px above the published card top must fall through"
    );
    assert_eq!(
        p.panel_hit(cx + cw * 0.5, cy + ch + 1.5),
        None,
        "{label}: 1.5px below the published card bottom must fall through"
    );
}

/// The `Aa` click target: its right end against the find row's own accumulated
/// ink width, and both ends against the pointer, inside and out.
fn grade_case_toggle(label: &str, p: &TextPipeline, g: &PanelGeometry, e: &mut Enrolled) {
    let (line_w, glyphs) = find_row_ink(p);
    assert!(
        glyphs >= 2,
        "{label}: the find row must shape the two glyphs the toggle is seated on"
    );
    let (x0, x1) = g
        .case_toggle
        .unwrap_or_else(|| panic!("{label}: a shaped find row must publish its Aa span"));
    e.toggles += 1;
    // `Aa` is the LAST span on the find row, so the toggle's right edge is that
    // row's own ink right edge — asserted against `line_w`, which glyphon
    // accumulates rather than derives from the two glyph advances the span reads.
    assert!(
        (x1 - (g.text_left + line_w)).abs() < 0.51,
        "{label}: the toggle ends at {x1} but the find row's ink ends at \
         text_left {} + line_w {line_w} = {}",
        g.text_left,
        g.text_left + line_w
    );
    assert!(
        x1 - x0 > 4.0,
        "{label}: the toggle publishes a {}px span — a click target needs a real \
         width, and every position check here is satisfied by a span of zero",
        x1 - x0
    );
    let row0_mid = g.rows[0].top + g.rows[0].h * 0.5;
    assert_eq!(
        p.panel_hit(x0 + 0.5, row0_mid),
        Some(PanelHit::CaseToggle),
        "{label}: just inside the published toggle's left edge must toggle case"
    );
    assert_eq!(
        p.panel_hit(x1 - 0.5, row0_mid),
        Some(PanelHit::CaseToggle),
        "{label}: just inside the published toggle's right edge must toggle case"
    );
    // THE EXTENT GRADE. A published span that had been uniformly shrunk still
    // contains its own probes; what it cannot do is stop being the toggle where
    // it says it stops.
    assert_eq!(
        p.panel_hit(x0 - 1.5, row0_mid),
        Some(PanelHit::Find),
        "{label}: 1.5px left of the published toggle must be the find field, not \
         the toggle — a published span wider than the real one fails here"
    );
    assert_eq!(
        p.panel_hit(x1 + 1.5, row0_mid),
        Some(PanelHit::Find),
        "{label}: 1.5px right of the published toggle must be the find field, not \
         the toggle — a published span narrower than the real one fails here"
    );
}

#[test]
fn published_panel_geometry_agrees_with_the_ink_and_the_pointer() {
    let _g = crate::testlock::serial();
    if crate::test_gpu::adapter_present() {
        // nothing to do: the sweep below builds its own pipelines
    } else {
        eprintln!(
            "skipping published_panel_geometry_agrees_with_the_ink_and_the_pointer: \
             no wgpu adapter"
        );
        return;
    }
    // `menu_bar`'s default is platform-dependent, so the ambient value is what
    // gets restored — never a `cfg!`, which reflects the host that compiled this.
    let ambient_bar = crate::menubar::menu_bar_on();
    let mut e = Enrolled::default();
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        e.bars.insert(bar);
        for (w, h) in [(1200.0_f32, 800.0_f32), (700.0, 520.0)] {
            for (replace, editing) in [(false, false), (true, false), (true, true)] {
                let Some((p, g)) = prepared(w, h, replace, editing) else {
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    eprintln!("skipping the panel geometry sweep: no wgpu adapter");
                    return;
                };
                grade(
                    &format!("bar={bar} {w}x{h} replace={replace} editing={editing}"),
                    &p,
                    &g,
                    replace,
                    &mut e,
                );
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);

    assert_eq!(e.bars.len(), 2, "both menu-bar arms must be swept");
    assert!(
        e.row_counts.contains(&1) && e.row_counts.len() > 1,
        "the sweep must cross the row-count boundary (a plain find panel shapes \
         one row, the replace state three), got {:?}",
        e.row_counts
    );
    assert!(
        e.cells >= 12 && e.rows >= 20 && e.toggles == e.cells,
        "the sweep graded {} cells, {} row bands and {} toggle spans — a green \
         run must be able to show what it enrolled",
        e.cells,
        e.rows,
        e.toggles
    );
}

/// Grade ONE prepared panel's caret centre-y against the three owners that never
/// read the placer, returning what the cell enrolled: the focused row, and that
/// row's published band height (the pitch the DPI arm compares).
fn grade_caret_cy(
    label: &str,
    p: &mut TextPipeline,
    g: &PanelGeometry,
    w: f32,
    replace: bool,
    editing: bool,
) -> (usize, f32) {
    let tops = shaped_row_tops(p);
    // The FOCUSED row, from the state this cell set rather than from the shaper —
    // so the shaper's own focus→row mapping is graded too, instead of being read
    // back out and compared to itself.
    let want_row = usize::from(editing);
    let caret_row = p.panel_shape_text(w as u32).caret_row;
    assert!(
        (caret_row - want_row as f32).abs() < 0.001,
        "{label}: the focused field is on row {want_row} but the shaper reports \
         caret_row {caret_row}"
    );
    let cy = p.panel_caret_cy(g.text_top, caret_row);

    // ---- against the PUBLISHED band -----------------------------------------
    let band = g
        .rows
        .iter()
        .find(|r| r.row == want_row)
        .unwrap_or_else(|| panic!("{label}: row {want_row} must be published"));
    assert!(
        (cy - (band.top + band.h * 0.5)).abs() < 0.01,
        "{label}: the caret centres at y {cy} while its row's published band \
         [{}, {}] centres at {}",
        band.top,
        band.top + band.h,
        band.top + band.h * 0.5
    );
    assert!(
        cy > band.top + 0.5 && cy < band.top + band.h - 0.5,
        "{label}: the caret centre {cy} is not strictly inside its own band \
         [{}, {}] — a centre that has escaped the row it names draws the caret \
         against the neighbouring field's ink",
        band.top,
        band.top + band.h
    );

    // ---- against the INK glyphon laid that row out at ------------------------
    let (line_top, line_h) = p
        .panel_buffer
        .layout_runs()
        .find(|run| run.line_i == want_row)
        .map(|run| (run.line_top, run.line_height))
        .unwrap_or_else(|| panic!("{label}: row {want_row} must shape"));
    assert!(
        (cy - (g.text_top + line_top + line_h * 0.5)).abs() < 0.51,
        "{label}: the caret centres at {cy} while row {want_row}'s glyphs were laid \
         out at text_top {} + line_top {line_top} with height {line_h}, whose \
         centre is {}",
        g.text_top,
        g.text_top + line_top + line_h * 0.5
    );
    assert!(
        tops.iter().any(|(i, _)| *i == want_row),
        "{label}: the shaped-row census must contain the focused row"
    );

    // ---- against the POINTER's own inverse -----------------------------------
    let [cx, _, cw, _] = g.card;
    assert_eq!(
        p.panel_hit(cx + cw * 0.5, cy),
        Some(want_at(want_row as i64, replace)),
        "{label}: a press at the caret's own centre-y {cy} must land in the field \
         the caret is editing"
    );
    (want_row, band.h)
}

/// **THE PANEL CARET SITS AT ITS FOCUSED ROW'S VERTICAL CENTRE**, graded against
/// three owners that never read the placer: the published band, the y glyphon laid
/// that row's glyphs out at, and the pointer's own inverse.
///
/// ⚠️ **THIS WAS THE ONE UNGRADED ARM OF THE PANEL'S ROW-BAND OWNER.** `band` is
/// graded by the sweep above and `row_at` by every pointer probe in it, but
/// `center` — the arm the caret rides — had no law, and the panel caret's `y` was
/// asserted nowhere in the tree: the caret-placement law next door sweeps its `x`
/// across begin/mid/end in both fields and never looks at its row. Changing the
/// `0.5` in `PanelRowBands::center` moved the amber caret off every row's centre
/// with the whole suite green.
///
/// The axes are the ones that can hide it: **both field arms** (the focused row is
/// 0 or 1, and a placer that ignored `caret_row` would pass on row 0 alone),
/// **both row counts** (one shaped row cannot show a wrong step), and **both
/// DPIs** (the pitch scales while the card's pad does not, so a centre expressed
/// against the wrong one of the two agrees at exactly one scale). The pitch is
/// asserted to have actually changed between the DPI arms, because an axis that
/// turned out to be a no-op is the failure that reads as coverage.
#[test]
fn the_panel_caret_centres_on_its_focused_rows_band_and_ink() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!(
            "skipping the_panel_caret_centres_on_its_focused_rows_band_and_ink: \
             no wgpu adapter"
        );
        return;
    }
    let mut cells = 0usize;
    let mut focused_rows = std::collections::BTreeSet::new();
    let mut row_counts = std::collections::BTreeSet::new();
    let mut pitches: Vec<(u32, f32)> = Vec::new();
    for dpi in [1.0_f32, 2.0] {
        for (w, h) in [(1200.0_f32, 800.0_f32), (700.0, 520.0)] {
            for (replace, editing) in [(false, false), (true, false), (true, true)] {
                let Some((mut p, g)) = prepared_at(w, h, replace, editing, dpi) else {
                    eprintln!("skipping the panel caret sweep: no wgpu adapter");
                    return;
                };
                let label = format!("dpi={dpi} {w}x{h} replace={replace} editing={editing}");
                let (row, pitch) = grade_caret_cy(&label, &mut p, &g, w, replace, editing);
                cells += 1;
                focused_rows.insert(row);
                row_counts.insert(g.rows.len());
                pitches.push((dpi.to_bits(), pitch));
            }
        }
    }

    assert_eq!(
        focused_rows,
        std::collections::BTreeSet::from([0, 1]),
        "both field arms must be swept — a placer that ignores caret_row passes on \
         row 0 alone, got {focused_rows:?}"
    );
    assert!(
        row_counts.contains(&1) && row_counts.len() > 1,
        "the sweep must cross the row-count boundary, got {row_counts:?}"
    );
    // THE DPI AXIS IS PROVED, NOT ASSUMED: if the pitch did not move, the second
    // arm graded the first arm's arithmetic a second time.
    let lo = pitches
        .iter()
        .filter(|(d, _)| *d == 1.0_f32.to_bits())
        .map(|(_, h)| *h)
        .fold(f32::INFINITY, f32::min);
    let hi = pitches
        .iter()
        .filter(|(d, _)| *d == 2.0_f32.to_bits())
        .map(|(_, h)| *h)
        .fold(0.0_f32, f32::max);
    assert!(
        hi > lo * 1.5,
        "the dpi arms graded the same row pitch ({lo} at dpi 1, {hi} at dpi 2), so \
         the scale axis this law claims to sweep is a no-op"
    );
    assert!(
        cells >= 12,
        "the sweep graded {cells} cells — a green run must be able to show what it \
         enrolled"
    );
}

/// **THE MENU BAR MOVES THE WHOLE BLOCK OR NONE OF IT.** The reserve is a
/// vertical inset, so every published y steps by ONE shared delta and no
/// published x moves at all. A partial yield — the card stepping down while its
/// rows or its click target stay put — is the failure this arm exists for, and it
/// is invisible to any single-arm check.
#[test]
fn a_shown_menu_bar_steps_the_whole_published_panel_by_one_delta() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!(
            "skipping a_shown_menu_bar_steps_the_whole_published_panel_by_one_delta: \
             no wgpu adapter"
        );
        return;
    }
    let ambient_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    let Some((off_p, off)) = prepared(1200.0, 800.0, true, false) else {
        crate::menubar::set_menu_bar_on(ambient_bar);
        return;
    };
    let reserve_off = off_p.menubar_reserve();
    crate::menubar::set_menu_bar_on(true);
    let Some((on_p, on)) = prepared(1200.0, 800.0, true, false) else {
        crate::menubar::set_menu_bar_on(ambient_bar);
        return;
    };
    let reserve_on = on_p.menubar_reserve();
    crate::menubar::set_menu_bar_on(ambient_bar);

    assert_eq!(reserve_off, 0.0, "the bar-off arm must reserve nothing");
    assert!(
        reserve_on > 1.0,
        "a shown bar must reserve a real strip, got {reserve_on}"
    );
    let delta = on.card[1] - off.card[1];
    assert!(
        delta > 1.0,
        "the card must yield to a shown bar, but its top moved {delta}"
    );
    assert!(
        (on.text_top - off.text_top - delta).abs() < 0.01,
        "the text origin stepped {} while the card stepped {delta}",
        on.text_top - off.text_top
    );
    assert_eq!(
        on.rows.len(),
        off.rows.len(),
        "the swept state must shape the same rows in both arms"
    );
    for (a, b) in off.rows.iter().zip(on.rows.iter()) {
        assert!(
            (b.top - a.top - delta).abs() < 0.01,
            "row {} stepped {} while the card stepped {delta} — a partial yield",
            a.row,
            b.top - a.top
        );
        assert!(
            (b.h - a.h).abs() < 0.01,
            "row {}'s band height changed with the menu bar",
            a.row
        );
    }
    assert!(
        (on.card[0] - off.card[0]).abs() < 0.01 && (on.text_left - off.text_left).abs() < 0.01,
        "a vertical reserve moved a horizontal figure: card x {} -> {}, text left \
         {} -> {}",
        off.card[0],
        on.card[0],
        off.text_left,
        on.text_left
    );
    let (a, b) = (
        off.case_toggle.expect("bar off publishes a toggle"),
        on.case_toggle.expect("bar on publishes a toggle"),
    );
    assert!(
        (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01,
        "the Aa click target moved horizontally with a vertical reserve: \
         {a:?} -> {b:?}"
    );
}
