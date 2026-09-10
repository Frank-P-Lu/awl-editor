//! THE BLOCKQUOTE PULL-QUOTE **PAIR** — every world draws BOTH marks.
//!
//! The hanging pull-quote used to draw only its opening mark, so every quote in
//! every world read permanently unclosed. The closing mark is its counterpart —
//! same display face, same [`super::super::layers`] scale, same [`theme::faint`]
//! value — but NOT its mirror: the opening mark still hangs in the writing
//! column's LEFT text-pad gutter, while the closing mark (2026-09 decision)
//! instead FOLLOWS the block's own last-row text, landing one gap past its own
//! ink and anchored to that row's own baseline, clamped inside the column at
//! the widest wrap rather than hanging in a fixed right gutter.
//!
//! # What is measured, and how
//!
//! A DIFFERENTIAL pair per world — the fixed document WITH its two blockquote
//! blocks against the SAME document with those lines (and the reference row)
//! blanked, line count and every row top preserved. The differential cancels
//! the page ground, the margin texture and whatever per-world pattern bleeds
//! under the column (Kite's warped grid, Paperbark's stripes), which a
//! same-image threshold cannot.
//!
//! The OPENING mark's band is still the fixed LEFT gutter (`column_left` ..
//! `text_left`) — unchanged from before this item. The CLOSING mark's band is
//! no longer a fixed gutter: its LEFT edge is read straight off the sealed
//! [`crate::render::LayoutReport`]'s own row geometry (`row.xs.last()`, the
//! same real ink-right the production path anchors to), taking the WIDER of
//! the block's own two rows' ink so a shorter row above can never be mistaken
//! for the mark; its RIGHT edge is the column's own right edge, generous
//! enough to hold the mark at any clamp. Every claim is asserted TWICE, and
//! the pairing is the point:
//!
//! * **PRESENCE** — a floor on the COUNT of ink pixels the mark contributes to
//!   its band. A contrast ratio alone gets *happier* as a treatment fades
//!   toward the page: a mark washed out to four bytes from the ground reports a
//!   better ratio than the shipped one while being invisible. The mark has to
//!   EXIST to pass here.
//! * **CONTRAST** — a RATIO of two quantities read from the same pair of
//!   frames: the mark's own peak ink deviation from the ground it sits on, as a
//!   share of the BODY TEXT's peak deviation from that same ground in the same
//!   capture. Both terms are rendered pixels, never an authored constant, so a
//!   backend that rounds differently moves them together.
//!
//! A THIRD claim is specific to the closing mark's new placement:
//!
//! * **ROW CONTAINMENT** — the sliver strictly ABOVE the closing row's own top
//!   (that row's band grown upward by one more row-height, minus the row
//!   itself) must carry NO mark ink in the same x-band. This is the law the
//!   reported "the 99 rode above the row and read as belonging to the row
//!   above" defect would fail: the fix anchors the mark's box to the row's own
//!   baseline and floors it at the row's own top, and this law is the pixel
//!   proof that the floor holds.
//!
//! The NEGATIVE CONTROL is checked FIRST, before any reading is taken off the
//! differential: the bands beside a NON-blockquote row (the reference row,
//! which the blank arm also blanks, so its text band genuinely differs) must
//! contribute NO ink at all. That is both the "no mark on non-blockquote
//! lines" law and the proof that the ruler is a ruler — a differential
//! reporting frame noise would satisfy every presence floor below it.
//!
//! The enrolment is the roster itself — every world in [`theme::THEMES`],
//! Cassowary included, whose `Theme` lives in its own module and is missed by
//! any grep over `worlds.rs`. Every failure names the world and the block that
//! produced it, so a shrinking sweep is visible rather than silent.

use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view_md};
use crate::theme;

const W: u32 = 1200;
const H: u32 = 900;
/// An ordinary prose column — the default a user actually sees.
const MEASURE: usize = 70;

/// The fixed document. Line 2 is the 26-letter BODY-INK reference row and the
/// negative control's own band; lines 4-5 are a TWO-line blockquote block;
/// line 7 is the DEGENERATE one-line block, where the opening and closing marks
/// share a row top and are told apart by x alone. The last row's text ("A
/// second quoted line.") is deliberately LONGER than the first ("A quoted
/// line.") so the closing mark's own x-band (anchored past the last row's ink)
/// can never overlap the first row's shorter text — see the module doc's
/// "WIDER of the block's own two rows" note.
const DOC_QUOTE: &str = "# Pull Quote Presence\n\nabcdefghijklmnopqrstuvwxyz\n\n\
                         > A quoted line.\n> A second quoted line.\n\n\
                         > One line alone.\n\nBody text after the quotes.\n";
/// The SAME document with the reference row and both blockquote blocks blanked
/// — same line count, so every row top is unmoved and the differential is
/// purely "what those lines drew".
const DOC_BLANK: &str = "# Pull Quote Presence\n\n\n\n\n\n\n\n\nBody text after the quotes.\n";

const REF_ROW: &str = "abcdefghijklmnopqrstuvwxyz";
const BLOCK_FIRST_ROW: &str = "> A quoted line.";
const BLOCK_LAST_ROW: &str = "> A second quoted line.";
const LONE_ROW: &str = "> One line alone.";

/// Per-channel summed abs diff a pixel must clear to count as ink — well above
/// 8-bit quantization noise, well under a real glyph edge's step (the same
/// differential-oracle margin `ornament_scale.rs` uses).
const INK_DIFF_FLOOR: i32 = 24;

/// The PRESENCE floor: ink pixels a single hanging mark must contribute to its
/// own band. Calibrated the same way as every appearance floor in this repo:
/// measured against the live roster (`--nocapture` on this test prints
/// nothing, so this figure is read out of a real run, not authored blind), a
/// mark deleted or faded past [`INK_DIFF_FLOOR`] reads **0**, and the floor
/// sits with room for rasterization jitter on another backend.
const PRESENCE_FLOOR: usize = 40;

/// The CONTRAST floor, as a share of the same frame's own body-text ink
/// deviation — two rendered quantities from one pair of captures, never an
/// authored colour. The mark is deliberately quiet ([`theme::faint`]).
const FAINT_SHARE_FLOOR: f32 = 0.15;

/// The ROW-CONTAINMENT ceiling: ink pixels tolerated in the sliver strictly
/// above the closing row's own top, in the SAME x-band the mark itself is
/// measured in. A few pixels of anti-aliasing bleed at a row boundary is not
/// the defect; a mark whose whole body rides into the row above is — the
/// bound sits far under [`PRESENCE_FLOOR`] so the two claims cannot be
/// confused for one another.
const ABOVE_ROW_CEILING: usize = 8;

struct GutterInk {
    /// Ink pixels the differential found in this band.
    count: usize,
    /// Peak per-pixel summed-channel deviation from the ground beneath.
    peak: i32,
}

/// Differential ink inside a rectangular band: `[x0, x1) x [y0, y1)`.
fn band_ink(a: &[[u8; 4]], b: &[[u8; 4]], x0: u32, x1: u32, y0: u32, y1: u32) -> GutterInk {
    let mut count = 0usize;
    let mut peak = 0i32;
    for y in y0..y1.min(H) {
        for x in x0..x1.min(W) {
            let idx = (y * W + x) as usize;
            let (p, q) = (a[idx], b[idx]);
            let diff = (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>();
            if diff > INK_DIFF_FLOOR {
                count += 1;
                peak = peak.max(diff);
            }
        }
    }
    GutterInk { count, peak }
}

fn render_doc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut crate::render::TextPipeline,
    text: &str,
) -> (Vec<[u8; 4]>, crate::render::LayoutReport) {
    // Caret parked on the closing paragraph: the pull-quote is a BLOCK
    // affordance, not reveal-on-cursor, but keeping the caret off the quote
    // lines keeps the `> ` markers concealed in the measured arm.
    p.set_view(&view_md(text, 9, 0));
    p.prepare(device, queue, W, H).expect("prepare failed");
    let report = p.layout_report().expect("sealed frame is reportable");
    let (texture, tview) = offscreen(device, W, H);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl pull-quote-pair test encoder"),
    });
    p.render(&mut encoder, &tview).expect("render failed");
    queue.submit(Some(encoder.finish()));
    (read_pixels(device, queue, &texture, W, H), report)
}

/// One world's readings. `open`/`close` are the two bands of the TWO-line
/// block; `lone_*` the degenerate one-line block; `control_*` the bands
/// beside the (non-blockquote) reference row; `*_above` is the row-containment
/// reading for the closing mark alone (there is no equivalent claim for the
/// opening mark, which is untouched by this item).
struct WorldInk {
    name: &'static str,
    open: GutterInk,
    close: GutterInk,
    close_above: usize,
    lone_open: GutterInk,
    lone_close: GutterInk,
    lone_close_above: usize,
    control_left: usize,
    control_right: usize,
    body_peak: i32,
}

/// A row's OWN band from the sealed layout: `(top, bottom)`, no headroom.
fn row_band(report: &crate::render::LayoutReport, content: &str, world: &str) -> (u32, u32) {
    let row = report
        .rows
        .iter()
        .find(|r| r.content == content)
        .unwrap_or_else(|| panic!("{world}: row {content:?} not found in the sealed layout"));
    let top = row.top.max(0.0).round() as u32;
    let bot = (row.top + row.height).max(0.0).round() as u32;
    (top, bot)
}

/// The real shaped ink-right x of `content`'s own row, read straight off the
/// sealed [`crate::render::LayoutReport`] — the same quantity the production
/// path (`TextPipeline::quote_close_row_end_x`) anchors the closing mark to,
/// but independently sourced here from the report rather than re-deriving the
/// production formula.
fn row_ink_right(report: &crate::render::LayoutReport, content: &str, world: &str) -> f32 {
    report
        .rows
        .iter()
        .find(|r| r.content == content)
        .unwrap_or_else(|| panic!("{world}: row {content:?} not found in the sealed layout"))
        .xs
        .last()
        .copied()
        .unwrap_or(0.0)
}

fn measure_world(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut crate::render::TextPipeline,
    name: &'static str,
) -> WorldInk {
    theme::set_active_by_name(name).unwrap();
    p.sync_theme();

    let (pix_quote, report) = render_doc(device, queue, p, DOC_QUOTE);
    let (pix_blank, blank_report) = render_doc(device, queue, p, DOC_BLANK);

    // The differential is only honest if the two arms put their shared rows in
    // the same place; the closing paragraph is present in both.
    let tail = "Body text after the quotes.";
    assert_eq!(
        row_band(&report, tail, name),
        row_band(&blank_report, tail, name),
        "{name}: blanking the quote lines moved the document — the differential \
         would be measuring reflow, not the marks"
    );

    let col_left = p.column_left().max(0.0).round() as u32;
    let col_right = (p.column_left() + p.column_width()).max(0.0).round() as u32;
    let text_left = p.text_left().max(0.0).round() as u32;
    let text_right = (p.text_left() + p.text_wrap_width()).max(0.0).round() as u32;
    assert!(
        col_left < text_left && text_right < col_right,
        "{name}: the writing column has no text-pad gutter for the OPEN mark \
         (column {col_left}..{col_right}, text {text_left}..{text_right})"
    );

    // OPEN: unchanged fixed left gutter. Its mark is still anchored to the
    // row's own TOP (untouched by this item) and so, exactly as before, rides
    // one row-height of headroom above that top — the window is grown to
    // match, using THIS world's own real `line_height` rather than a literal.
    let line_height = p.metrics.line_height.round().max(1.0) as u32;
    let (open_top, open_bot) = row_band(&report, BLOCK_FIRST_ROW, name);
    let (lone_top, lone_bot) = row_band(&report, LONE_ROW, name);
    let left = |y0: u32, y1: u32| {
        band_ink(
            &pix_quote,
            &pix_blank,
            col_left,
            text_left,
            y0.saturating_sub(line_height),
            y1,
        )
    };

    // CLOSE: the block's own last-row ink-right, guarded against the row
    // ABOVE's own (shorter) text ever entering the x-band.
    let close_x0 = row_ink_right(&report, BLOCK_FIRST_ROW, name)
        .max(row_ink_right(&report, BLOCK_LAST_ROW, name))
        .round() as u32;
    let lone_x0 = row_ink_right(&report, LONE_ROW, name).round() as u32;
    let (close_top, close_bot) = row_band(&report, BLOCK_LAST_ROW, name);
    let close = band_ink(
        &pix_quote, &pix_blank, close_x0, col_right, close_top, close_bot,
    );
    let close_above = band_ink(
        &pix_quote,
        &pix_blank,
        close_x0,
        col_right,
        close_top.saturating_sub(close_bot - close_top),
        close_top,
    )
    .count;
    let lone_close = band_ink(
        &pix_quote, &pix_blank, lone_x0, col_right, lone_top, lone_bot,
    );
    let lone_close_above = band_ink(
        &pix_quote,
        &pix_blank,
        lone_x0,
        col_right,
        lone_top.saturating_sub(lone_bot - lone_top),
        lone_top,
    )
    .count;

    // Reference row: the same "own ink-right, generous right bound" shape as
    // the close band, so the control is a fair rehearsal of the same ruler.
    let ref_ink_right = row_ink_right(&report, REF_ROW, name).round() as u32;
    let (ref_top, ref_bot) = row_band(&report, REF_ROW, name);

    WorldInk {
        name,
        open: left(open_top, open_bot),
        close,
        close_above,
        lone_open: left(lone_top, lone_bot),
        lone_close,
        lone_close_above,
        control_left: left(ref_top, ref_bot).count,
        control_right: band_ink(
            &pix_quote,
            &pix_blank,
            ref_ink_right,
            col_right,
            ref_top,
            ref_bot,
        )
        .count,
        // The reference row's own ink, measured in the SAME differential (the
        // blank arm blanks that row) — the body-ink yardstick the mark's
        // contrast is a share of.
        body_peak: band_ink(
            &pix_quote, &pix_blank, text_left, text_right, ref_top, ref_bot,
        )
        .peak,
    }
}

/// THE HEADLINE LAW. Every world in the live roster draws BOTH pull-quote
/// marks, in both a multi-line and a one-line block, by pixel presence AND by a
/// same-frame contrast ratio — with the non-blockquote bands as the negative
/// control, and the closing mark additionally proved to stay off the row above
/// it.
#[test]
fn every_world_draws_both_pull_quote_marks() {
    // Taken BEFORE the device is reached and held past the drops below: the
    // shared test GPU's counters move on every render AND every readback, not
    // only on the call that reached the device first.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping every_world_draws_both_pull_quote_marks: no wgpu adapter");
        return;
    };
    let was_theme = theme::active().name;
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_wysiwyg = crate::markdown::wysiwyg_on();
    crate::page::set_page_on(true);
    crate::page::set_measure(MEASURE);
    crate::markdown::set_wysiwyg_on(true);

    let measured: Vec<WorldInk> = theme::THEMES
        .iter()
        .map(|t| measure_world(&device, &queue, &mut p, t.name))
        .collect();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    crate::markdown::set_wysiwyg_on(was_wysiwyg);
    theme::set_active_by_name(was_theme).unwrap();
    drop(p);
    drop(queue);
    drop(device);

    // NON-VACUITY OF THE ENROLMENT: the sweep is the live roster, and the
    // roster is derived, not named. A world silently dropped from the loop, or
    // a roster that shrank, fails here rather than passing by measuring less.
    assert_eq!(
        measured.len(),
        theme::THEMES.len(),
        "swept {} worlds against a live roster of {} — the enrolment shrank",
        measured.len(),
        theme::THEMES.len()
    );
    let swept: Vec<&str> = measured.iter().map(|m| m.name).collect();
    assert!(
        swept.contains(&"Cassowary"),
        "the sweep did not enrol Cassowary, whose Theme lives outside worlds.rs — \
         swept: {swept:?}"
    );

    for m in &measured {
        assert_world(m);
    }
}

/// One world's readings, in the order a reading must be believed: the ruler
/// first (the negative control), then each mark's PRESENCE and CONTRAST, then
/// the closing mark's own row-containment.
fn assert_world(m: &WorldInk) {
    // NEGATIVE CONTROL: a non-blockquote row puts no ink in either band, even
    // though its own text band differs between the two arms.
    assert_eq!(
        (m.control_left, m.control_right),
        (0, 0),
        "{}: the NON-blockquote reference row put ink in the bands \
             (left {}, right {}) — either an ornament is drawing where no \
             blockquote is, or this differential is reporting frame noise \
             rather than marks",
        m.name,
        m.control_left,
        m.control_right
    );

    for (label, ink) in [
        ("multi-line block, opening mark", &m.open),
        ("multi-line block, closing mark", &m.close),
        ("one-line block, opening mark", &m.lone_open),
        ("one-line block, closing mark", &m.lone_close),
    ] {
        // PRESENCE: the mark must EXIST. A pure contrast floor is satisfied
        // by a mark that has faded to nothing.
        assert!(
            ink.count >= PRESENCE_FLOOR,
            "{}: {label} contributed only {} ink pixels to its band \
                 (floor {PRESENCE_FLOOR}) — the mark is missing or has faded \
                 into the page",
            m.name,
            ink.count
        );
        // CONTRAST, as a share of this same capture's own body ink — two
        // rendered quantities, never an authored colour.
        assert!(
            m.body_peak > 0,
            "{}: the body-ink yardstick measured nothing, so no share is \
                 computable",
            m.name
        );
        let share = ink.peak as f32 / m.body_peak as f32;
        assert!(
            share >= FAINT_SHARE_FLOOR,
            "{}: {label} peaks at {} against body ink {} — share {share:.3} \
                 under the floor {FAINT_SHARE_FLOOR}; the mark is present but \
                 washed toward the page",
            m.name,
            ink.peak,
            m.body_peak
        );
    }

    // ROW CONTAINMENT: the closing mark's own row-above sliver, in the SAME
    // x-band, carries no more than a hair of anti-aliasing bleed. This is the
    // pixel proof against the reported "reads as belonging to the row above"
    // defect — it is satisfiable only if the mark's ink genuinely stays at or
    // below its own row's top.
    for (label, above) in [
        ("multi-line block, closing mark", m.close_above),
        ("one-line block, closing mark", m.lone_close_above),
    ] {
        assert!(
            above <= ABOVE_ROW_CEILING,
            "{}: {label} put {above} ink pixels in the sliver ABOVE its own \
                 row (ceiling {ABOVE_ROW_CEILING}) — it rides into the row \
                 above instead of closing its own line",
            m.name
        );
    }
}
