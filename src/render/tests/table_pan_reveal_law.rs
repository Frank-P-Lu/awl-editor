//! THE TABLE READING-PAN LAW: growing a selection through an already
//! horizontally-scrolled table must never move a row the selection hasn't
//! touched.
//!
//! `place_shaped_table` (`render/layers/table_grid.rs`) zeroes a table's
//! reading pan — the horizontal offset a prior wheel-scroll left on an
//! overflowing table (`TextPipeline::table_pan`) — whenever `TableMeta`
//! reports the table "revealed". `revealed` is a wide flag: it also picks
//! which row's grid cells get skipped for its raw-source x-ray float, and it
//! fires for EITHER the caret sitting inside the table OR any active
//! selection merely touching it (`wysiwyg_reveals`'s selection-reveal rule).
//! So the very first row a selection touched in a panned table snapped the
//! WHOLE grid — every column of every OTHER row, header included — flush
//! left, and a live drag's caret tracks the moving end of the selection, so a
//! drag that starts above the table and grows down into it keeps the caret
//! genuinely inside the table range for most of the gesture: the untouched
//! rows visibly jumped sideways under the user's eyes while they were
//! mid-select, read out as "neighbouring cells flicker while the band
//! settles".
//!
//! The fix narrows the pan-reset condition to `TableMeta::caret_inside` — the
//! caret's line is inside the table's range AND there is no active selection,
//! the plain single-caret editing case the pan-reset mechanism was always
//! meant for. An active selection, even one whose far end lands inside the
//! table, now leaves the table's reading pan exactly where the user left it.
//!
//! PRESENCE FLOOR, non-vacuous: [`pan_actually_moves_the_grid`] proves the
//! panned fixture's header row is NOT already flush left before any
//! selection exists (a floor that a "grid never reads `table_pan` at all"
//! regression could not satisfy by accident). Reverting `caret_inside` back
//! to plain `range.contains(&cursor_byte)` (dropping the
//! `&& self.selection.is_none()` half) reproduces the reported jump exactly:
//! [`growing_a_selection_into_a_panned_table_never_moves_the_other_rows`]
//! goes red — measured, at the moment the drag first reaches the table body,
//! 10110 of a 1200x32 header band's 38400 pixels differing (max channel
//! delta 208, on the Tawny world) against the frame just before the drag
//! entered it.

use super::super::*;
use super::pixeldiff::{Region, assert_identical, render_frame};
use super::{headless_dqp, view_md};

/// A table wide enough that its content overflows an ordinary 1200px canvas
/// (two ~110-char columns), with prose on both sides and four body rows —
/// enough rows to grow a selection through one at a time.
fn wide_table_fixture() -> String {
    // A varied (non-periodic) run of digits, not a single repeated
    // character: a repeated glyph can render pixel-identical to itself under
    // a whole-character pan shift, which would make the presence floor below
    // vacuous rather than proving the pan actually moved anything.
    let filler = || {
        (0..110)
            .map(|i| char::from(b'0' + (i % 10)))
            .collect::<String>()
    };
    format!(
        "Prose before the table.\n\
         \n\
         | {} | {} |\n\
         | --- | --- |\n\
         | one | two |\n\
         | three | four |\n\
         | five | six |\n\
         | seven | eight |\n\
         \n\
         Prose after the table.\n",
        filler(),
        filler()
    )
}

/// Doc lines of this fixture's table: 2 header, 3 divider, 4..=7 body rows.
const HEADER_LINE: usize = 2;
const DIVIDER_LINE: usize = 3;
const BODY_LINES: [usize; 4] = [4, 5, 6, 7];

/// Prepare `p` on the wide fixture with the caret parked outside the table
/// (line 0) and a real, non-zero reading pan pre-set on it — the state a
/// prior horizontal wheel-scroll gesture (`App::try_horizontal_table_pan`)
/// would leave behind. Returns the table's own byte range start (the
/// `table_pan` key) and the pan value actually applied after clamping.
fn seed_panned_table(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) -> usize {
    let text = wide_table_fixture();
    let v = view_md(&text, 0, 0);
    p.set_view(&v);
    p.prepare(device, queue, 1200, 800).unwrap();
    let table_start = p
        .tables_report()
        .first()
        .expect("one table laid out")
        .range
        .0;
    // A generous, clamped-in-bounds scroll — real enough that the grid
    // visibly reads from partway through its content when parked at it.
    p.table_pan = Some((table_start, 300.0));
    p.prepare(device, queue, 1200, 800).unwrap();
    table_start
}

/// PRESENCE FLOOR: the panned fixture's header row is NOT already flush left
/// — proving `table_pan` really does move the grid before the selection law
/// below is asked to hold it still. A "the grid ignores `table_pan`
/// entirely" regression would make the growing-selection law vacuously pass
/// (nothing ever moves, selected or not), so this floor must hold first.
#[test]
fn pan_actually_moves_the_grid() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping pan_actually_moves_the_grid: no wgpu adapter");
        return;
    };
    let was_spell = crate::spell::spellcheck_on();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    crate::spell::set_spellcheck_on(false);
    let w = 1200u32;
    let h = 800u32;

    seed_panned_table(&mut p, &device, &queue);
    let top = p.line_ornament_top(HEADER_LINE);
    let panned = render_frame(&mut p, &device, &queue, w, h);

    // The SAME fixture and caret, with no pan ever applied. `table_pan` is
    // pipeline-persistent state `set_view` does not touch — clear it
    // explicitly, or this "unpanned" render would just be the panned one
    // again.
    let text = wide_table_fixture();
    let v = view_md(&text, 0, 0);
    p.set_view(&v);
    p.table_pan = None;
    p.prepare(&device, &queue, w, h).unwrap();
    let unpanned = render_frame(&mut p, &device, &queue, w, h);

    let region = Region::new(0.0, top, w as f32, p.metrics.line_height);
    super::pixeldiff::assert_perceptibly_different(
        &unpanned,
        &panned,
        w as i64,
        h as i64,
        region,
        super::pixeldiff::DistinguishFloor::DEFAULT,
        "header row band, panned vs. unpanned (presence floor)",
    );

    crate::spell::set_spellcheck_on(was_spell);
}

/// THE LAW: dragging a selection row by row into and through an
/// already-panned table — the caret tracking the drag's moving end, exactly
/// as a live mouse drag leaves it — never moves a row's own grid cells that
/// the selection hasn't touched, at any step of the growth.
#[test]
fn growing_a_selection_into_a_panned_table_never_moves_the_other_rows() {
    let _t = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let _world = crate::theme::WorldPin::snapshot();
    // ONE pipeline reused across the roster (`p.sync_theme()` retints it per
    // world) — the same hoisted-pipeline shape `empty_table_wash_has_
    // rendered_presence_on_every_world` uses, rather than a fresh device per
    // world.
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping growing_a_selection_into_a_panned_table_never_moves_the_other_rows: \
             no wgpu adapter"
        );
        return;
    };
    let was_spell = crate::spell::spellcheck_on();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    crate::spell::set_spellcheck_on(false);
    let w = 1200u32;
    let h = 800u32;
    let text = wide_table_fixture();

    let mut enrolled = Vec::new();
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        let table_start = seed_panned_table(&mut p, &device, &queue);
        let mut prev_frame = render_frame(&mut p, &device, &queue, w, h);
        let mut prev_xray: Vec<usize> = Vec::new();

        // Grow the drag one body row at a time: (4,2)-(4,2), (4,2)-(5,2), ...
        // The caret (the drag's moving end) lands ON the growing endpoint —
        // the realistic live-drag state, not a caret parked elsewhere.
        for &end_line in &BODY_LINES {
            let mut v = view_md(&text, end_line, 2);
            v.selection = Some(((BODY_LINES[0], 2), (end_line, 2)));
            p.set_view(&v);
            // `table_pan` is pipeline-owned state (item-persistent, not part
            // of `ViewState`) and `set_view` does not touch it — re-affirm it
            // every step exactly as the earlier seed did, since a real wheel
            // scroll is a one-time gesture the drag doesn't repeat.
            p.table_pan = Some((table_start, 300.0));
            p.prepare(&device, &queue, w, h).unwrap();
            let xray: Vec<usize> = p.xray.iter().map(|x| x.line).collect();
            let frame = render_frame(&mut p, &device, &queue, w, h);

            for line in [HEADER_LINE, DIVIDER_LINE].into_iter().chain(BODY_LINES) {
                if prev_xray.contains(&line) || xray.contains(&line) {
                    continue; // this row IS the band (or just left it) — may legitimately change
                }
                let top = p.line_ornament_top(line);
                let region = Region::new(0.0, top, w as f32, p.metrics.line_height);
                assert_identical(
                    &prev_frame,
                    &frame,
                    w as i64,
                    h as i64,
                    region,
                    &format!(
                        "{}: doc line {line} (untouched by the selection before or after \
                         growing to end_line={end_line})",
                        world.name
                    ),
                );
            }
            prev_frame = frame;
            prev_xray = xray;
        }
        enrolled.push(world.name);
    }
    assert_eq!(
        enrolled.len(),
        crate::theme::THEMES.len(),
        "full roster enrolled: {enrolled:?}"
    );

    crate::spell::set_spellcheck_on(was_spell);
}
