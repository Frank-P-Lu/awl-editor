//! DECORATIVE GEOMETRY VERSUS THE CARET — the class laws.
//!
//! Two mechanisms in the document layer make room for something the reader is
//! meant to see, and a caret-adjacent treatment drawn from that room inherits
//! whatever the room was sized for rather than what is actually drawn in it:
//!
//!   * a ROW grown taller than its own glyphs (the heading ladder's decoupled
//!     [`crate::markdown::heading_row_lead`], a thematic break's
//!     `ornament_scale`, an inline image's absolute row, a wrapped table row);
//!   * a RUN's advances forced wider than the source (the three painted
//!     substitute families — smart punctuation, a tamed bare URL's "…", a
//!     footnote's superscript number).
//!
//! The first family already had two carve-outs (image rows and x-rayed table
//! rows both answer a BODY-height band) and the thematic break drops its room
//! entirely on reveal. What nobody had asked is the heading: its row grows by
//! SIZE and then again by a decoupled LEAD, and every band treatment was
//! scaled by the product. Measured before the fix, on the `###` rung where the
//! lead is largest and the size smallest, the selection band stood 43.15px over
//! type shaped at 27.6px — 34% taller than the row's own glyphs asked for, and
//! 11px of empty accent at DPI 1. The ink-box Block caret was already right (it
//! tracks the row's FONT size, never its full height including leading); the
//! selection band, the search wash, the code pill, the strike and link
//! underlines, the spell and nit underlines, the mono line cell and the
//! insertion bar were all on the other quantity. [`super::super::TextPipeline::caret_band_scale`]
//! is the one owner, and it now divides the lead back out.
//!
//! The advance family had both failure directions live at once. The bare-URL
//! ellipsis reserved `line_height * 0.9` against a real glyph 14.04–24.00px
//! wide across the roster — 20% to 105% of a hole for the mark to sit
//! left-aligned in. The footnote number reserved
//! `line_height * (0.34 + 0.20 * (digits - 1))`, which is NARROWER than a real
//! two-digit number on nearly a third of the roster (worst: Potoroo, `100`
//! shaping 30.36px into a 23.68px slot) — an overrun into the following prose, and a
//! `debug_assert!` failure in any debug build that opened a document with ten
//! footnotes. Both are now the substitute's OWN shaped advance, from the same
//! owner that shapes the ink.
//!
//! Enrolment is derived from [`crate::theme::THEMES`] throughout, never from a
//! named world — `Cassowary` lives in its own module, so a grep over
//! `worlds.rs` alone comes up one short of the roster.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, headless_pipeline, view, view_md};

const W: u32 = 1200;
const H: u32 = 800;

/// The heading rungs plus body, so every law here states which rung it is on.
const LEVELS: [u8; 4] = [0, 1, 2, 3];

/// One `Heading title` line per rung, a blank, then body — the caret parks on
/// the body line so the heading is off-caret in the states that need it.
fn heading_doc(level: u8) -> String {
    if level == 0 {
        "Heading title\n\nbody line here\n".to_string()
    } else {
        format!(
            "{} Heading title\n\nbody line here\n",
            "#".repeat(level as usize)
        )
    }
}

/// The RETIRED band quantity — the row's full height over the base line
/// height, lead and all. Kept as a named function so every law below can state
/// what it would have answered and prove the two are distinguishable, rather
/// than asserting the new answer against nothing.
fn retired_band_scale(level: u8) -> f32 {
    crate::markdown::heading_scale(level) * crate::markdown::heading_row_lead(level)
}

// ---------------------------------------------------------------------------
// The ROW family — a heading's decoupled lead stops at the row
// ---------------------------------------------------------------------------

/// **THE HEADLINE.** On every rung of the ladder and every world in the
/// roster: the ROW is still grown by size × lead (the decoration is intact —
/// this law is about who INHERITS it, so a "fix" that flattened
/// `heading_row_lead` to 1.0 fails here rather than passing trivially), while
/// the caret-adjacent band scale is the SIZE rung alone.
///
/// The two quantities are asserted to differ at every rung above body, so the
/// assertion below is a choice between two live answers rather than a
/// tautology.
#[test]
fn a_headings_row_lead_grows_the_row_and_never_the_caret_band() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping a_headings_row_lead_grows_the_row_and_never_the_caret_band: no adapter"
        );
        return;
    };
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let lh = p.metrics.line_height;
        for level in LEVELS {
            let doc = heading_doc(level);
            p.set_view(&view_md(&doc, 2, 0));
            p.prepare(&device, &queue, W, H).unwrap();
            let report = p.layout_report().expect("sealed frame is reportable");
            let row = report
                .rows
                .iter()
                .find(|r| r.logical_line == 0)
                .expect("the heading row is shaped");
            let size = crate::markdown::heading_scale(level);
            let lead = crate::markdown::heading_row_lead(level);

            // THE SUBJECT IS STILL THERE: the row really is grown by both
            // factors. Without this a lead of 1.0 would satisfy the band claim
            // below by deleting the decoration the law exists to police.
            assert!(
                (row.height / lh - size * lead).abs() < 1e-2,
                "{} h{level}: the row must still carry size x lead ({}x{lead}); got {}/{lh}",
                t.name,
                size,
                row.height
            );

            let band = p.caret_band_scale(0, row.height);
            assert!(
                (band - size).abs() < 1e-3,
                "{} h{level}: the caret band must be the heading's own SIZE rung {size}, \
                 not the row's full {} — the decoupled row lead is decoration no glyph \
                 occupies",
                t.name,
                retired_band_scale(level)
            );
            if level > 0 {
                // NON-VACUITY: at every rung above body the two candidate
                // answers are genuinely different numbers, so the assertion
                // above chose one.
                assert!(
                    (retired_band_scale(level) - size).abs() > 0.1,
                    "{} h{level}: the retired quantity {} and the size rung {size} must \
                     differ or this law is choosing between equals",
                    t.name,
                    retired_band_scale(level)
                );
            }
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * LEVELS.len(),
        "every (world, rung) cell must be graded — enrolment comes from THEMES, \
         which is {} worlds including the one in its own module",
        theme::THEMES.len()
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// Every band consumer's reading on ONE line of the fixture, in three families
/// keyed by how each one reads the band: `heights` take the band's height,
/// `fractions` sit at a fraction of it, `gaps` hang a fixed distance under it.
/// `slot` is 0 for the body row and 1 for the heading row, so one call fills
/// one half of each pair. Returns the row's own height alongside.
struct BandReadings {
    heights: Vec<(&'static str, [f32; 2])>,
    fractions: Vec<(&'static str, [f32; 2])>,
    gaps: Vec<(&'static str, [f32; 2])>,
    rows: [f32; 2],
}

impl BandReadings {
    fn new() -> Self {
        Self {
            heights: Vec::new(),
            fractions: Vec::new(),
            gaps: Vec::new(),
            rows: [0.0; 2],
        }
    }

    fn push(bucket: &mut Vec<(&'static str, [f32; 2])>, slot: usize, name: &'static str, v: f32) {
        if let Some(entry) = bucket.iter_mut().find(|(n, _)| *n == name) {
            entry.1[slot] = v;
        } else {
            let mut pair = [0.0; 2];
            pair[slot] = v;
            bucket.push((name, pair));
        }
    }

    fn take(&mut self, p: &mut TextPipeline, world: &str, slot: usize) {
        let report = p.layout_report().unwrap();
        let row = report.rows.iter().find(|r| r.logical_line == 0).unwrap();
        self.rows[slot] = row.height;
        // The band this row's treatments are all drawn from, read from the one
        // owner rather than reconstructed.
        let vrow = p.visual_rows(0).remove(0);
        let line_top = p.doc_top() + vrow.line_top;
        let (band_y, band_h) = p.row_caret_band(0, &vrow, line_top);
        Self::push(
            &mut self.heights,
            slot,
            "selection band height",
            p.selection_rects().first().map(|r| r[3]).unwrap_or(0.0),
        );
        Self::push(
            &mut self.heights,
            slot,
            "search wash height",
            p.search_match_rects().first().map(|r| r[3]).unwrap_or(0.0),
        );
        Self::push(
            &mut self.heights,
            slot,
            "inline-code pill height",
            p.code_pill_rects().first().map(|r| r[3]).unwrap_or(0.0),
        );
        Self::push(
            &mut self.fractions,
            slot,
            "strike line band fraction",
            p.strike_lines()
                .first()
                .map(|s| (s.y - band_y) / band_h)
                .unwrap_or(0.0),
        );
        Self::push(
            &mut self.gaps,
            slot,
            "spell squiggle gap under the band",
            p.spell_squiggles()
                .first()
                .map(|s| s.y - (band_y + band_h))
                .unwrap_or(f32::NAN),
        );
        // THE FOLLOWABLE UNDERLINE IS STRUCTURALLY ABSENT FROM A HEADING ROW,
        // so it cannot join the fraction family: pulldown-cmark stamps a link's
        // text inside an ATX heading as `MdKind::Heading`, never `LinkText`, so
        // `Bucket::LinkUnderline` never enrols it. Pinned rather than skipped
        // silently — if that ever changes, this sweep gains a consumer and has
        // to say so.
        let links = p.link_underlines().len();
        let want = usize::from(slot == 0);
        assert_eq!(
            links, want,
            "{world}: the followable underline draws on the body row and not on a \
             heading row (its text is stamped `Heading`, not `LinkText`); if that \
             changed, add it to this sweep's fraction family"
        );
    }
}

/// **THE CONSUMER SWEEP.** The scale above is only worth pinning if every
/// treatment drawn from it actually moved. Each consumer is measured on an
/// `###` line and on the identical bytes as body prose, in three families by
/// how each one reads the band:
///
///   * HEIGHT consumers (the selection band, the search wash, the inline-code
///     pill) take the band's height, so their heading value must be the SIZE
///     rung times their body value — never the retired row product;
///   * INSIDE-band consumers (the strike line) sit at a fraction of the band,
///     so that fraction must be the SAME on both rows;
///   * BELOW-band consumers (the spell squiggle) hang a fixed gap under the
///     band's bottom edge, so that gap must be the same on both rows.
///
/// Together the three say: every one of them rides the band, and the band is
/// the one law above pins. Each reading is the drawn geometry, not a
/// re-derivation of it.
#[test]
fn every_caret_band_consumer_grew_by_the_size_rung_alone() {
    let _t = crate::testlock::serial();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping every_caret_band_consumer_grew_by_the_size_rung_alone: no adapter");
        return;
    };
    let level = 3u8;
    let size = crate::markdown::heading_scale(level);
    // One line carrying every span family a band treatment can enrol: inline
    // code (pill), a strike run, a followable link (underline), a misspelling
    // (squiggle). The selection and the search wash take plain words.
    const TAIL: &str = "alpha `code` ~~gone~~ [lnk](u) wrongg\n\nbody\n";
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut readings = BandReadings::new();
        for (slot, prefix) in [(0usize, ""), (1usize, "### ")] {
            let doc = format!("{prefix}{TAIL}");
            let off = prefix.len();
            let mut v = view_md(&doc, 2, 0);
            v.selection = Some(((0, off), (0, off + 5)));
            v.search_matches = vec![((0, off + 6), (0, off + 12))];
            v.misspelled = vec![crate::spell::Misspelling {
                line: 0,
                start_col: off + 30,
                end_col: off + 36,
            }];
            p.set_view(&v);
            p.prepare(&device, &queue, W, H).unwrap();
            readings.take(&mut p, t.name, slot);
        }
        let rows = readings.rows;
        assert!(
            rows[1] > rows[0] + 1.0,
            "{}: the fixture's heading row ({}) must actually be taller than its body \
             twin ({}) or this sweep proves nothing",
            t.name,
            rows[1],
            rows[0]
        );
        for (name, [body, head]) in readings.heights {
            // PRESENCE: a treatment that drew nothing would satisfy any ratio
            // claim by being absent from both frames.
            assert!(
                body > 1.0 && head > 1.0,
                "{}: `{name}` must actually be drawn on BOTH rows (body={body}, \
                 heading={head}) or its ratio is a claim about two zeroes",
                t.name
            );
            let ratio = head / body;
            assert!(
                (ratio - size).abs() < 0.06,
                "{}: `{name}` on an h{level} must be {size}x its body value (the size \
                 rung), not {}x (the retired row product): got {ratio} ({head}/{body})",
                t.name,
                retired_band_scale(level)
            );
            graded += 1;
        }
        for (name, [body, head]) in readings.fractions {
            assert!(
                body > 0.01 && head > 0.01,
                "{}: `{name}` must be drawn on BOTH rows (body={body}, heading={head})",
                t.name
            );
            assert!(
                (head - body).abs() < 0.02,
                "{}: `{name}` must sit at the same fraction of the band on an h{level} \
                 ({head}) as on body ({body}) — it rides the band, so the band moving \
                 is the whole change",
                t.name
            );
            graded += 1;
        }
        for (name, [body, head]) in readings.gaps {
            assert!(
                body.is_finite() && head.is_finite(),
                "{}: `{name}` must be drawn on BOTH rows",
                t.name
            );
            assert!(
                (head - body).abs() < 0.6,
                "{}: `{name}` must be the same fixed gap on an h{level} ({head}) as on \
                 body ({body})",
                t.name
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * 5,
        "five consumers x the whole roster"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// **THE MONO LINE CELL AND THE INSERTION BAR**, the two CARET forms that read
/// [`super::super::TextPipeline::cursor_scale`] rather than an ink box. The
/// proportional Block caret already tracked the row's font size
/// (`caret_ink_box.rs`); these two were on the row's full height, so an `###`
/// caret stood 34% over its own type. Swept over the roster's mono-display
/// worlds, derived from `facepitch::mono_display_worlds` rather than named.
#[test]
fn the_row_scaled_caret_forms_track_the_headings_size_rung() {
    let _t = crate::testlock::serial();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the_row_scaled_caret_forms_track_the_headings_size_rung: no adapter");
        return;
    };
    let mono = super::facepitch::mono_display_worlds();
    assert!(
        !mono.is_empty(),
        "the mono-display roster must be non-empty or the line-cell arm is unswept"
    );
    let mut graded = 0usize;
    for t in theme::THEMES.iter().filter(|t| mono.contains(&t.name)) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        for mode in [CaretMode::Block, CaretMode::Ibeam, CaretMode::Morph] {
            crate::caret::set_mode(mode);
            let mut heights = Vec::new();
            for level in LEVELS {
                let text = format!("{} xx", "#".repeat(level as usize));
                let text = if level == 0 { "xx".to_string() } else { text };
                let mut v = view(&text, 0, text.len());
                v.is_markdown = true;
                p.set_view(&v);
                p.settle_caret();
                let (_cx, _cy, _w, h, ..) = p.caret_geometry();
                heights.push(h);
            }
            for level in [1u8, 2, 3] {
                let want = crate::markdown::heading_scale(level);
                let ratio = heights[level as usize] / heights[0];
                assert!(
                    (ratio - want).abs() < 0.05,
                    "{} {mode:?} h{level}: the row-scaled caret must be {want}x the body \
                     caret (the size rung), not {}x (the retired row product): got {ratio}",
                    t.name,
                    retired_band_scale(level)
                );
                graded += 1;
            }
        }
    }
    assert!(
        graded > 0,
        "at least one mono world x form x rung was graded"
    );
    crate::caret::set_mode(CaretMode::Block);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// How much looser than its own body row a heading's band may hug its ink. The
/// two ratios either side of it are both READ FROM THE FRAME — a backend that
/// rasterizes a glyph edge one row differently moves them together — but this
/// slack is authored, so it is calibrated from three measured figures at the
/// tightest cell on the roster (Saltpan `##`, DPI 1, this host's Metal):
///
///   * the shipped reading: band/ink `1.233` against a body `1.167`, `+0.066`;
///   * the reading the retired defect produces at that same cell: `1.533`,
///     `+0.366`;
///   * the floor between them: `0.18`.
///
/// It sits ~2.7x above the shipped worst case (about three rows of a 30px ink
/// box, so a one-pixel threshold difference on another rasterizer cannot reach
/// it) and ~2x under the defect, which still fails on the large majority of
/// the roster by at least `0.083`. A tighter `0.12` left the tightest cell one
/// pixel from red, which is the shape that has taken this repo's pixel laws
/// down on lavapipe before.
const BAND_HUG_ALLOWANCE: f32 = 0.18;

/// **REAL PIXELS.** The geometry laws above are arithmetic over the emitters;
/// this one is arithmetic over the frame. The selection band's DRAWN vertical
/// extent is recovered by differencing two rendered frames (a one-character
/// selection against a whole-line one, sampled to the right of that first
/// character so the revealed markup and the caret are identical in both), and
/// the heading's own glyph INK from the unselected frame's own pixels.
///
/// The floor is the PRODUCT'S OWN body behaviour on the same world, never an
/// authored constant: a heading's band may not hug its type any looser than
/// the body band hugs body type. Before the fix the `###` rung ran 1.48–1.67x
/// its ink against a body ratio of 1.12–1.30.
#[test]
fn a_headings_selection_band_hugs_its_ink_as_tightly_as_body_does() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping a_headings_selection_band_hugs_its_ink_as_tightly_as_body_does: none");
        return;
    };
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut ratios = [0.0f32; 4];
        for level in LEVELS {
            let doc = heading_doc(level);
            let render = |p: &mut TextPipeline, v: &ViewState| {
                p.set_view(v);
                p.prepare(&device, &queue, W, H).unwrap();
                let (texture, tview) = offscreen(&device, W, H);
                let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("awl decor-band test encoder"),
                });
                p.render(&mut enc, &tview).expect("render failed");
                queue.submit(Some(enc.finish()));
                read_pixels(&device, &queue, &texture, W, H)
            };
            let mut narrow = view_md(&doc, 2, 0);
            narrow.selection = Some(((0, 0), (0, 1)));
            let mut wide = view_md(&doc, 2, 0);
            wide.selection = Some(((0, 0), (0, 13)));
            let narrow_px = render(&mut p, &narrow);
            let wide_px = render(&mut p, &wide);
            p.set_view(&wide);
            p.prepare(&device, &queue, W, H).unwrap();
            let report = p.layout_report().unwrap();
            let row = report.rows.iter().find(|r| r.logical_line == 0).unwrap();
            let rect = p.selection_rects()[0];
            // Sample well right of the first selected character (so the two
            // frames' glyphs and caret are identical there) and inside the
            // row's own box.
            let x0 = (rect[0] + 50.0) as usize;
            let x1 = ((rect[0] + rect[2] - 2.0) as usize).min(W as usize);
            let y0 = row.top.max(0.0) as usize;
            let y1 = ((row.top + row.height) as usize).min(H as usize);
            let bg = narrow_px[10 * W as usize + 10];
            let (mut ink_lo, mut ink_hi) = (usize::MAX, 0usize);
            let (mut band_lo, mut band_hi) = (usize::MAX, 0usize);
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = y * W as usize + x;
                    let d_bg: i32 = (0..3)
                        .map(|c| (narrow_px[i][c] as i32 - bg[c] as i32).abs())
                        .sum();
                    if d_bg > 24 {
                        ink_lo = ink_lo.min(y);
                        ink_hi = ink_hi.max(y);
                    }
                    let d_sel: i32 = (0..3)
                        .map(|c| (narrow_px[i][c] as i32 - wide_px[i][c] as i32).abs())
                        .sum();
                    if d_sel > 24 {
                        band_lo = band_lo.min(y);
                        band_hi = band_hi.max(y);
                    }
                }
            }
            assert!(
                ink_hi > ink_lo && band_hi > band_lo,
                "{} h{level}: the sample window must find BOTH real glyph ink and a real \
                 drawn band (ink [{ink_lo},{ink_hi}], band [{band_lo},{band_hi}]) — a law \
                 that finds neither proves nothing",
                t.name
            );
            let ink_h = (ink_hi - ink_lo + 1) as f32;
            let band_h = (band_hi - band_lo + 1) as f32;
            ratios[level as usize] = band_h / ink_h;
        }
        for level in [1u8, 2, 3] {
            // The body row's own hug, plus the allowance below.
            let floor = ratios[0] + BAND_HUG_ALLOWANCE;
            assert!(
                ratios[level as usize] <= floor,
                "{} h{level}: the drawn selection band is {:.3}x the heading's own ink \
                 while the SAME world's body band is {:.3}x its own — a heading's band \
                 may not hug looser than body's",
                t.name,
                ratios[level as usize],
                ratios[0]
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * 3,
        "every world x heading rung is graded in pixels"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

// ---------------------------------------------------------------------------
// The ADVANCE family — a painted substitute reserves its own shaped width
// ---------------------------------------------------------------------------

/// The RETIRED footnote reservation: a `line_height` fraction that grew a
/// fixed step per digit. Named so the non-vacuity clause below can prove it
/// really did fail — the defect this fix removes, not a hypothetical one.
fn retired_footnote_slot(number: usize, line_height: f32) -> f32 {
    let digits = number.max(1).ilog10() as f32 + 1.0;
    line_height * (0.34 + (digits - 1.0) * 0.20)
}

/// The RETIRED bare-URL ellipsis reservation.
fn retired_ellipsis_slot(line_height: f32) -> f32 {
    line_height * 0.9
}

/// The numbers a real document reaches. `10` is the first two-digit footnote —
/// ten footnotes in one file, which is not an edge case.
const FOOTNOTE_NUMBERS: [usize; 8] = [1, 5, 9, 10, 42, 99, 100, 999];

/// **A FOOTNOTE'S SLOT COVERS ITS OWN SHAPED NUMBER, IN EVERY WORLD AND AT
/// EVERY MAGNITUDE.** Non-vacuity is the important half: the retired formula is
/// evaluated alongside and must be proved to UNDER-reserve somewhere in the
/// same sweep, so this law is pinned to a defect that existed rather than to a
/// rule nobody could break.
#[test]
fn a_footnote_slot_covers_its_own_shaped_number_in_every_world() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping a_footnote_slot_covers_its_own_shaped_number_in_every_world: none");
        return;
    };
    let mut retired_failures = Vec::new();
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let metrics = p.metrics;
        let family = p.shaped_font;
        for number in FOOTNOTE_NUMBERS {
            let slot = p.substitute_advances.footnote_slot(number);
            let (_, width) = crate::render::spans::shape_footnote_number(
                &mut p.font_system,
                metrics,
                family,
                number,
                theme::muted().to_glyphon(),
            );
            // PRESENCE: a zero-width "number" would satisfy any covering claim.
            assert!(
                width > 1.0,
                "{}: footnote {number} must shape to real ink ({width}px) or the covering \
                 claim below is about nothing",
                t.name
            );
            assert!(
                slot >= width - 0.01,
                "{}: footnote {number} shapes {width}px but reserves only {slot}px — the \
                 painted number overruns into the prose that follows it",
                t.name
            );
            if retired_footnote_slot(number, metrics.line_height) < width - 0.01 {
                retired_failures.push(format!("{} n={number}", t.name));
            }
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * FOOTNOTE_NUMBERS.len(),
        "every world x magnitude cell is graded"
    );
    assert!(
        !retired_failures.is_empty(),
        "NON-VACUITY: the retired line-height formula must be shown to under-reserve \
         somewhere in this same sweep, or this law is pinning a rule nothing ever broke"
    );
    eprintln!(
        "retired footnote formula under-reserved in {} of {graded} cells, e.g. {}",
        retired_failures.len(),
        retired_failures[0]
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// The END-TO-END half: a real document with a two-digit footnote, driven
/// through a real `prepare` on every world, so the `debug_assert!` inside
/// `FootnoteNumbers::append_areas` is actually reached. That assert is the
/// production check; an assert nothing executes is silent, which is exactly how
/// the overrun above survived. (Mirrors
/// `markdown::bare_url_ellipsis_slot_fits_the_real_glyph_in_every_world`.)
#[test]
fn a_ten_footnote_document_paints_every_number_inside_its_slot() {
    let _t = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping a_ten_footnote_document_paints_every_number_inside_its_slot: none");
        return;
    };
    let mut doc = String::new();
    for i in 1..=12 {
        doc.push_str(&format!("para {i} with a note[^n{i}] in it\n\n"));
    }
    for i in 1..=12 {
        doc.push_str(&format!("[^n{i}]: definition {i}\n"));
    }
    // The caret parks on the LAST definition line so the references above all
    // conceal and their numbers actually paint.
    let caret_line = doc.lines().count().saturating_sub(1);
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        p.set_view(&view_md(&doc, caret_line, 0));
        let marks = p.footnote_marks();
        assert!(
            marks.iter().any(|(_, _, number, _)| *number >= 10),
            "{}: the fixture must actually paint a two-digit footnote number — the \
             magnitude the retired slot could not hold",
            t.name
        );
        // The `debug_assert!` in `FootnoteNumbers::append_areas` IS the
        // assertion; a slot too small for the real ink panics inside here.
        p.prepare(&device, &queue, W, H).unwrap();
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// **THE BARE-URL "…" RESERVES EXACTLY THE GLYPH IT PAINTS**, and it is the
/// SAME measurement the smart-punctuation ellipsis reserves — one codepoint
/// painted for one reason, so it gets one number. Non-vacuity: the retired
/// `line_height * 0.9` is proved, in this same sweep, to differ from the real
/// advance on the roster, and the spread is printed.
#[test]
fn the_bare_url_ellipsis_reserves_exactly_the_glyph_it_paints() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the_bare_url_ellipsis_reserves_exactly_the_glyph_it_paints: none");
        return;
    };
    let mut worst_retired = 0.0f32;
    let mut worst_at = "";
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let metrics = p.metrics;
        let family = p.shaped_font;
        let slot = p.substitute_advances.ellipsis_slot();
        let (_, width) = crate::render::spans::shape_smart_punct_glyph(
            &mut p.font_system,
            metrics,
            family,
            crate::markdown::SmartPunctKind::Ellipsis,
            theme::muted().to_glyphon(),
        );
        assert!(
            width > 1.0,
            "{}: the ellipsis must shape to a real advance or this law reserves nothing",
            t.name
        );
        assert!(
            (slot - width).abs() < 0.01,
            "{}: the bare-URL tail must reserve EXACTLY the '…' it paints ({width}px), \
             got {slot}px",
            t.name
        );
        // The bare URL and the smart-punctuation roster wear the same glyph and
        // must not hold two opinions about its width.
        assert!(
            (slot
                - p.substitute_advances
                    .advance(crate::markdown::SmartPunctKind::Ellipsis))
            .abs()
                < f32::EPSILON,
            "{}: the tamed-URL ellipsis and the smart-punctuation ellipsis must be ONE \
             measurement",
            t.name
        );
        let retired_gap = retired_ellipsis_slot(metrics.line_height) - width;
        if retired_gap > worst_retired {
            worst_retired = retired_gap;
            worst_at = t.name;
        }
        graded += 1;
    }
    assert_eq!(graded, theme::THEMES.len(), "the whole roster is graded");
    assert!(
        worst_retired > 2.0,
        "NON-VACUITY: the retired line-height formula must be shown to over-reserve \
         somewhere in this sweep (worst gap {worst_retired}px at {worst_at})"
    );
    eprintln!("retired ellipsis formula over-reserved by up to {worst_retired:.2}px ({worst_at})");
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// **A SUBSTITUTE'S SLOT NEVER SCALES WITH THE ROW IT LANDS ON.** Every painted
/// substitute is shaped at BODY metrics whatever row it sits in, so a
/// reservation keyed on the row's own height was wrong twice over on a heading
/// line: it reserved a `#` row's 1.84x of room for a body-size mark. All three
/// families are read from the mark rosters the ornament layer paints from
/// (`footnote_marks` / `bare_url_marks` / `smart_punct_marks`), which carry the
/// reserved slot itself, so this is the number the document actually forced.
/// Smart punctuation was already correct here and is swept alongside the two
/// that were not — an unlawed correct member is one refactor from being a
/// defect.
#[test]
fn a_painted_substitutes_slot_never_scales_with_the_row_it_lands_on() {
    let _t = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping a_painted_substitutes_slot_never_scales_with_the_row_it_lands_on: none"
        );
        return;
    };
    let mut graded = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        for (label, tail) in [
            ("footnote", "aa[^1] bb"),
            ("bare url", "aa https://example.com/deep bb"),
            ("smart punct", "aa -- bb"),
        ] {
            let mut slots = [0.0f32; 2];
            let mut heights = [0.0f32; 2];
            for (slot, prefix) in [(0usize, ""), (1usize, "# ")] {
                let doc = format!("{prefix}{tail}\n\n[^1]: def\nbody\n");
                p.set_view(&view_md(&doc, 3, 0));
                p.prepare(&device, &queue, W, H).unwrap();
                let report = p.layout_report().unwrap();
                let row = report.rows.iter().find(|r| r.logical_line == 0).unwrap();
                heights[slot] = row.height;
                slots[slot] = match label {
                    "footnote" => p.footnote_marks().first().map(|m| m.3).unwrap_or(0.0),
                    "bare url" => p.bare_url_marks().first().map(|m| m.2).unwrap_or(0.0),
                    _ => p.smart_punct_marks().first().map(|m| m.3).unwrap_or(0.0),
                };
            }
            assert!(
                heights[1] > heights[0] + 1.0,
                "{} {label}: the heading row ({}) must actually be taller than the body \
                 row ({}) or this cell proves nothing",
                t.name,
                heights[1],
                heights[0]
            );
            // PRESENCE: the mark has to exist on BOTH rows, or "the slot did
            // not change" is a statement about two absent marks.
            assert!(
                slots[0] > 1.0 && slots[1] > 1.0,
                "{} {label}: the substitute must actually be marked on BOTH rows \
                 (body={}, heading={})",
                t.name,
                slots[0],
                slots[1]
            );
            assert!(
                (slots[1] - slots[0]).abs() < 0.01,
                "{} {label}: the reserved slot must be the same on a heading row \
                 ({}) as on a body row ({}) — the mark paints at body size either way",
                t.name,
                slots[1],
                slots[0]
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        theme::THEMES.len() * 3,
        "three families x the roster"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// **AN ORNAMENT-LESS RENDERER LEAVES ITS SOURCE VISIBLE, FOR ALL THREE
/// FAMILIES.** A table GRID cell shapes its own buffer and has no ornament
/// layer, so it passes `None` for the reserved advances — and a substitute
/// family that collapsed anyway would force a hole nothing ever paints into.
/// Smart punctuation already followed that rule and was already lawed; the
/// footnote number and the tamed bare URL's "…" now follow it too and are
/// lawed here. Enrolment is the PARSE's own answer over the cell, never a
/// hand-picked byte list, and the match is exhaustive so a fourth substitute
/// family joins this sweep by failing to compile.
#[test]
fn a_table_grid_cell_keeps_every_substitute_family_visible() {
    use crate::markdown::ConcealKind;
    let _t = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    crate::markdown::set_wysiwyg_on(true);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping a_table_grid_cell_keeps_every_substitute_family_visible: no adapter");
        return;
    };
    let cell = "note[^1] see https://example.com/deep and wait... now";
    let doc = format!("| Mark |\n| --- |\n| {cell} |\n\npark\n\n[^1]: def\n");
    let mut v = view(&doc, 4, 0);
    v.is_markdown = true;
    p.set_view(&v);
    p.prepare(&device, &queue, W, H).unwrap();
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
        .expect("the body cell carrying all three substitute families is shaped");
    let line = &body.lines[0];
    let mut seen: Vec<&'static str> = Vec::new();
    for (range, kind) in crate::markdown::spans(cell) {
        let crate::markdown::MdKind::ConcealMarkup(ck) = kind else {
            continue;
        };
        let family = match ck {
            ConcealKind::Footnote => "footnote",
            ConcealKind::BareUrl => "bare url",
            ConcealKind::SmartPunct => "smart punct",
            ConcealKind::Heading
            | ConcealKind::Emphasis
            | ConcealKind::Code
            | ConcealKind::Highlight
            | ConcealKind::Strikethrough
            | ConcealKind::Fence
            | ConcealKind::Frontmatter
            | ConcealKind::Table
            | ConcealKind::Image
            | ConcealKind::Link
            | ConcealKind::Blockquote => continue,
        };
        let attrs = line.attrs_list().get_span(range.start);
        // The product's own "is this concealed" predicate (`TextPipeline::concealed_at`):
        // a zero-ALPHA span. A `ConcealMarkup` run that is merely dim still carries
        // real ink, which is the visible state this law is about.
        assert!(
            attrs.color_opt.is_none_or(|c| c.a() != 0),
            "{family} at {range:?}: a grid cell has no ornament layer, so its source keeps \
             real ink rather than concealing to a hole nothing paints into (got {:?})",
            attrs.color_opt
        );
        assert!(
            attrs.metrics_opt.is_none(),
            "{family} at {range:?}: a grid cell's source keeps full body metrics — a collapse \
             here forces an advance nothing ever paints into"
        );
        if !seen.contains(&family) {
            seen.push(family);
        }
    }
    seen.sort_unstable();
    assert_eq!(
        seen,
        ["bare url", "smart punct"],
        "the two substitute families a grid cell can REACH must both be graded (found \
         {seen:?}) or this law grades only whichever one it happened to parse"
    );
    // THE THIRD FAMILY IS STRUCTURALLY OUT OF REACH HERE, pinned rather than
    // skipped silently. `cell_inline_attrs` parses the cell SUBSTRING alone, and
    // a footnote reference only becomes one when its definition is in the same
    // parse — so a cell's `[^1]` stays literal text and never reaches the
    // substitute door at all. The same bytes inside a whole document do enrol,
    // which is what makes this an isolation property rather than a broken
    // fixture. If cell parsing ever gains document context, this flips and the
    // family joins the sweep above.
    let cell_footnotes = crate::markdown::spans(cell)
        .iter()
        .filter(|(_, k)| {
            matches!(
                k,
                crate::markdown::MdKind::ConcealMarkup(ConcealKind::Footnote)
            )
        })
        .count();
    let doc_footnotes = crate::markdown::spans(&doc)
        .iter()
        .filter(|(_, k)| {
            matches!(
                k,
                crate::markdown::MdKind::ConcealMarkup(ConcealKind::Footnote)
            )
        })
        .count();
    assert_eq!(
        cell_footnotes, 0,
        "a grid cell parsed in isolation carries no footnote reference span"
    );
    assert!(
        doc_footnotes > 0,
        "the SAME bytes in a whole document must carry one, or the isolation claim \
         above is really a broken fixture"
    );
}

/// **THE CELL THAT WAS ALREADY RIGHT, NOW PINNED.** A nested list item's
/// leading-space RUN is widened by the world's `list_indent_scale` — an
/// advance grown for a decorative reason, with no glyph of its own. Unlike
/// every other member of this class the widened cell IS what the reader sees,
/// so the block caret sitting in it should be exactly that wide, and it is.
/// Enrolment is derived from the roster's own two tiers, never named: the WIDE
/// worlds must show the scaled cell and the PLAIN worlds the natural one, and
/// both tiers must be non-empty.
#[test]
fn the_block_caret_in_a_widened_list_indent_is_exactly_that_cell_wide() {
    let _t = crate::testlock::serial();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping the_block_caret_in_a_widened_list_indent_is_exactly_that_cell: none");
        return;
    };
    crate::caret::set_mode(CaretMode::Block);
    let (mut wide, mut plain) = (0usize, 0usize);
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let scale = theme::active().list_indent_scale;
        // The world's OWN natural space advance, shaped on a plain paragraph of
        // the same world — never a cross-world constant.
        let mut v = view_md("a a\n", 0, 0);
        v.is_markdown = true;
        p.set_view(&v);
        p.settle_caret();
        let natural = p.visual_rows(0)[0].xs[2] - p.visual_rows(0)[0].xs[1];
        assert!(
            natural > 0.5,
            "{}: a space must have a real advance",
            t.name
        );

        let mut v = view_md("- top\n  - nested\n", 1, 0);
        v.is_markdown = true;
        p.set_view(&v);
        p.settle_caret();
        let (_cx, _cy, w, ..) = p.caret_geometry();
        assert!(
            (w - natural * scale).abs() < 0.2,
            "{}: the block caret on a widened indent space must be exactly that cell \
             ({} x {scale}), got {w}",
            t.name,
            natural
        );
        if (scale - 1.0).abs() > 1e-3 {
            wide += 1;
        } else {
            plain += 1;
        }
    }
    assert!(
        wide > 0 && plain > 0,
        "both list-indent tiers must be represented in the roster (wide={wide}, \
         plain={plain}) or this law only ever sees one branch"
    );
    crate::caret::set_mode(CaretMode::Block);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// **ENROLMENT GUARD.** Every caret-adjacent band in the document layer must
/// come through the one owner, so a new treatment cannot ship on a private
/// quotient of the row height. Counts the call sites in the render source: the
/// band builder (`row_band_for`/`row_caret_band`) and `caret_band_scale`
/// itself. A new consumer changes this count and has to join the sweep above.
#[test]
fn every_document_band_still_comes_through_the_one_scale_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render");
    let mut sites = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("render source is readable") {
            let path = entry.expect("readable entry").path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "tests") {
                    continue; // laws may name the owner freely
                }
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("readable source");
            for (n, line) in text.lines().enumerate() {
                let call = line.contains("row_band_for(") || line.contains("row_caret_band(");
                if call && !line.trim_start().starts_with("//") && !line.contains("fn ") {
                    sites.push(format!(
                        "{}:{}",
                        path.file_name().unwrap().to_string_lossy(),
                        n + 1
                    ));
                }
            }
        }
    }
    sites.sort();
    assert_eq!(
        sites.len(),
        8,
        "the caret-band owner has {} call sites, not the 8 this file's sweep grades \
         ({sites:?}) — a new caret-adjacent treatment must be added to \
         `every_caret_band_consumer_grew_by_the_size_rung_alone` before this count moves",
        sites.len()
    );
}
