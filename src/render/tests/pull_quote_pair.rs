//! THE BLOCKQUOTE PULL-QUOTE **PAIR** — every world draws BOTH marks.
//!
//! The hanging pull-quote used to draw only its opening mark, so every quote in
//! every world read permanently unclosed. The closing mark is its mirror: same
//! display face, same [`super::super::layers`] scale, same [`theme::faint`]
//! value, hung in the writing column's RIGHT text-pad gutter
//! (`geometry::pull_quote_right`) on the block's LAST visual row.
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
//! Every claim is asserted TWICE, and the pairing is the point:
//!
//! * **PRESENCE** — a floor on the COUNT of ink pixels the mark contributes to
//!   its gutter. A contrast ratio alone gets *happier* as a treatment fades
//!   toward the page: a mark washed out to four bytes from the ground reports a
//!   better ratio than the shipped one while being invisible. The mark has to
//!   EXIST to pass here.
//! * **CONTRAST** — a RATIO of two quantities read from the same pair of
//!   frames: the mark's own peak ink deviation from the ground it sits on, as a
//!   share of the BODY TEXT's peak deviation from that same ground in the same
//!   capture. Both terms are rendered pixels, never an authored constant, so a
//!   backend that rounds differently moves them together.
//!
//! The NEGATIVE CONTROL is checked FIRST, before any reading is taken off the
//! differential: the gutters beside a NON-blockquote row (the reference row,
//! which the blank arm also blanks, so its text band genuinely differs) must
//! contribute NO gutter ink at all. That is both the "no mark on
//! non-blockquote lines" law and the proof that the ruler is a ruler — a
//! differential reporting frame noise would satisfy every presence floor
//! below it.
//!
//! The enrolment is the roster itself ([`theme::THEMES`], twenty worlds
//! including Cassowary, which lives in its own module and is missed by any
//! grep over `worlds.rs`), and every failure names the world and the block that
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
/// share a row top and are told apart by x alone.
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
/// own gutter. Three figures calibrate it, per this repo's rule for an
/// appearance floor: the roster's TIGHTEST shipped reading is **84** (Mulga's
/// opening mark; the whole roster runs 84-271 across all four marks this law
/// scans), a mark deleted or faded past [`INK_DIFF_FLOOR`] reads **0**, and the
/// floor sits between them with room for rasterization jitter on another
/// backend. This is the assertion the item exists for: a contrast ratio alone
/// gets HAPPIER as a treatment fades toward the page, so the mark must first be
/// shown to EXIST.
const PRESENCE_FLOOR: usize = 45;

/// The CONTRAST floor, as a share of the same frame's own body-text ink
/// deviation — two rendered quantities from one pair of captures, never an
/// authored colour. The mark is deliberately quiet ([`theme::faint`]), so this
/// sits far below 1: the roster's tightest shipped share is **0.239**
/// (Mangrove, peak 139 against body 581) and its widest is 1.000 (Wagtail,
/// where both saturate).
const FAINT_SHARE_FLOOR: f32 = 0.15;

/// The pair is ONE glyph rotated, at one scale, in one value. Their ink-box
/// heights must agree to within this many rows — the roster's widest shipped
/// disagreement is **1** row (a rotated outline rasterizes a hair differently),
/// against boxes 10-20 rows tall, so halving the closing mark's scale moves it
/// far outside.
const PAIR_HEIGHT_SLOP: u32 = 2;

/// ...and their peak ink deviations must agree to within this share. The roster
/// ships **1.000** in every world — the two marks are drawn from the same
/// `attrs`, so any drift here is a second colour or a second alpha appearing.
const PAIR_VALUE_FLOOR: f32 = 0.85;

struct GutterInk {
    /// Ink pixels the differential found in this band.
    count: usize,
    /// Peak per-pixel summed-channel deviation from the ground beneath.
    peak: i32,
    /// Ink bounding-box height (rows), 0 when nothing was found.
    ink_h: u32,
}

/// Differential ink inside a rectangular band: `[x0, x1) x [y0, y1)`.
fn band_ink(a: &[[u8; 4]], b: &[[u8; 4]], x0: u32, x1: u32, y0: u32, y1: u32) -> GutterInk {
    let mut count = 0usize;
    let mut peak = 0i32;
    let (mut min_y, mut max_y) = (None::<u32>, None::<u32>);
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
                min_y = Some(min_y.map_or(y, |m| m.min(y)));
                max_y = Some(max_y.map_or(y, |m| m.max(y)));
            }
        }
    }
    GutterInk {
        count,
        peak,
        ink_h: match (min_y, max_y) {
            (Some(t), Some(b)) => b - t + 1,
            _ => 0,
        },
    }
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

/// One world's readings. `open`/`close` are the two gutters of the TWO-line
/// block; `lone_*` the degenerate one-line block; `control_*` the gutters
/// beside the (non-blockquote) reference row.
struct WorldInk {
    name: &'static str,
    open: GutterInk,
    close: GutterInk,
    lone_open: GutterInk,
    lone_close: GutterInk,
    control_left: usize,
    control_right: usize,
    body_peak: i32,
}

/// The window a mark hung from `content`'s row occupies: that row's own band
/// GROWN by one row-height upward. The pull-quote is shaped at
/// `QUOTE_MARK_SCALE` x the body size inside a ONE-row box, so its ink rides
/// above its own row top — a band clipped to the row alone measures a fraction
/// of the opening mark and all of the closing one, and then "the pair shares a
/// scale" fails on an artefact of the ruler rather than on the product. The row
/// above is blank in both arms of the differential, so the extra headroom
/// contributes nothing of its own.
fn mark_band(report: &crate::render::LayoutReport, content: &str, world: &str) -> (u32, u32) {
    let row = report
        .rows
        .iter()
        .find(|r| r.content == content)
        .unwrap_or_else(|| panic!("{world}: row {content:?} not found in the sealed layout"));
    let top = (row.top - row.height).max(0.0).round() as u32;
    let bot = (row.top + row.height).max(0.0).round() as u32;
    (top, bot)
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
        mark_band(&report, tail, name),
        mark_band(&blank_report, tail, name),
        "{name}: blanking the quote lines moved the document — the differential \
         would be measuring reflow, not the marks"
    );

    let col_left = p.column_left().max(0.0).round() as u32;
    let col_right = (p.column_left() + p.column_width()).max(0.0).round() as u32;
    let text_left = p.text_left().max(0.0).round() as u32;
    let text_right = (p.text_left() + p.text_wrap_width()).max(0.0).round() as u32;
    assert!(
        col_left < text_left && text_right < col_right,
        "{name}: the writing column has no text-pad gutters to hang a mark in \
         (column {col_left}..{col_right}, text {text_left}..{text_right})"
    );

    let (open_top, open_bot) = mark_band(&report, BLOCK_FIRST_ROW, name);
    let (close_top, close_bot) = mark_band(&report, BLOCK_LAST_ROW, name);
    let (lone_top, lone_bot) = mark_band(&report, LONE_ROW, name);
    let (ref_top, ref_bot) = mark_band(&report, REF_ROW, name);

    let left = |y0, y1| band_ink(&pix_quote, &pix_blank, col_left, text_left, y0, y1);
    let right = |y0, y1| band_ink(&pix_quote, &pix_blank, text_right, col_right, y0, y1);

    WorldInk {
        name,
        open: left(open_top, open_bot),
        close: right(close_top, close_bot),
        lone_open: left(lone_top, lone_bot),
        lone_close: right(lone_top, lone_bot),
        control_left: left(ref_top, ref_bot).count,
        control_right: right(ref_top, ref_bot).count,
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
/// same-frame contrast ratio — with the non-blockquote gutters as the negative
/// control.
#[test]
fn every_world_draws_both_pull_quote_marks() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping every_world_draws_both_pull_quote_marks: no wgpu adapter");
        return;
    };
    // Held across every render AND every readback: the shared test GPU's
    // counters move on all of them, not only on the call that reached the
    // device first.
    let _g = crate::testlock::serial();
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

/// One world's four marks, in the order a reading must be believed: the ruler
/// first (the negative control), then each mark's PRESENCE and CONTRAST, then
/// the pair's agreement in scale and value.
fn assert_world(m: &WorldInk) {
    // NEGATIVE CONTROL: a non-blockquote row hangs nothing in either
    // gutter, even though its own text band differs between the two arms.
    assert_eq!(
        (m.control_left, m.control_right),
        (0, 0),
        "{}: the NON-blockquote reference row put ink in the gutters \
             (left {}, right {}) — either an ornament is drawing where no \
             blockquote is, or this differential is reporting frame noise \
             rather than marks",
        m.name,
        m.control_left,
        m.control_right
    );

    for (label, ink) in [
        ("multi-line block, opening mark (left gutter)", &m.open),
        ("multi-line block, closing mark (right gutter)", &m.close),
        ("one-line block, opening mark (left gutter)", &m.lone_open),
        ("one-line block, closing mark (right gutter)", &m.lone_close),
    ] {
        // PRESENCE: the mark must EXIST. A pure contrast floor is satisfied
        // by a mark that has faded to nothing.
        assert!(
            ink.count >= PRESENCE_FLOOR,
            "{}: {label} contributed only {} ink pixels to its gutter \
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

    // OPEN AND CLOSE SHARE VALUE AND SCALE, by arithmetic over the pixels:
    // one glyph rotated, one face, one scale, one faint value.
    for (label, a, b) in [
        ("multi-line block", &m.open, &m.close),
        ("one-line block", &m.lone_open, &m.lone_close),
    ] {
        let h_gap = a.ink_h.abs_diff(b.ink_h);
        assert!(
            h_gap <= PAIR_HEIGHT_SLOP,
            "{}: {label} pair disagrees in SCALE — opening ink box {} rows, \
                 closing {} rows ({h_gap} apart, slop {PAIR_HEIGHT_SLOP})",
            m.name,
            a.ink_h,
            b.ink_h
        );
        let v_share = a.peak.min(b.peak) as f32 / a.peak.max(b.peak) as f32;
        assert!(
            v_share >= PAIR_VALUE_FLOOR,
            "{}: {label} pair disagrees in VALUE — opening peak {}, closing \
                 peak {} (share {v_share:.3} under {PAIR_VALUE_FLOOR})",
            m.name,
            a.peak,
            b.peak
        );
    }
}
