//! The per-world thematic-break ORNAMENT is equalized UPWARD: every world's
//! [`theme::Theme::ornament_scale`] now draws its own glyph at the ink height
//! the roster's OWN tallest normalized rendering reaches, not at a shared
//! numeric tier. A user comparing two live screenshots (a chess-piece set
//! drawing noticeably smaller than a bar-glyph set, both dark worlds) found
//! the shared three-tier scale (`ORNAMENT_SCALE_ORNATE`/`FLEURON`/`GEOMETRIC`)
//! was blind to the axis a glyph's own SHAPE hides: a chess knight and five
//! solid Genjiko bars fill their em-box at very different ink-to-em ratios, so
//! two worlds on the SAME tier (both `GEOMETRIC`, 1.5) rendered at measured
//! ink-height/char-width ratios of 2.750 (Currawong's knight) and 3.721
//! (Mulga's bars) — the user's own rough screenshot estimate, ~2.6-2.9 vs
//! ~3.6, confirmed by real pixel arithmetic.
//!
//! # What is measured, and how
//!
//! Real ink, not requested font size: a DIFFERENTIAL pair (the fixed document
//! WITH its `---` rule line vs the SAME document with that line blanked) is
//! rendered per world, and the ornament's own ink bounding-box height is the
//! rows that differ between the two — cancelling the page ground, any margin
//! texture, and (per world) whatever background pattern bleeds under the
//! column, exactly the class of false-ink source a same-image background
//! threshold would catch on Kite's under-page-veiled warped grid. The row's
//! TOP/HEIGHT and the reference row's per-glyph advances both come off
//! [`TextPipeline::layout_report`] — the renderer's own sealed geometry, not a
//! second parallel computation of it. Char width is the world's OWN body font,
//! measured off a 26-letter `a`..`z` reference row in the SAME capture (never
//! [`crate::render::CHAR_WIDTH`], which is a nominal UI cell, not a per-font
//! measurement).
//!
//! Scope: the fixed document carries one thematic-break ornament run (`---`),
//! a deliberately narrowed scope — [`Theme::ornament_scale`] is a single
//! per-world dial shared by all three break glyphs (dash/star/underscore), so
//! equalizing the dash axis moves all three together but does not
//! independently equalize star/underscore's own ink-to-em ratios; that
//! residual is unmeasured here and is a candidate for a follow-up round.
//!
//! The TARGET is derived from the roster itself (the live maximum, recomputed
//! every run) rather than pinned to a named world, so a future taste retune
//! moves the target with it instead of leaving the law asserting yesterday's
//! number. [`TOLERANCE`]'s own doc records the real post-fix spread it must
//! clear and the non-vacuity headroom below it.
//!
//! # A world that opts OUT of the upward equalization
//!
//! The equalization direction is UPWARD only until a live taste verdict asks
//! for the opposite: Gumtree's own dash/star/underscore trio (the snake run,
//! the fish, the snail) was judged "a tad too tall" against the rest of its
//! own room, not against the roster, and was retuned smaller on that verdict
//! alone. Re-running the roster-wide floor after that retune would either
//! reject the very change the user asked for, or (worse) silently creep the
//! literal back up the next time this law's own failure message is read as
//! an instruction — the exact "law re-equalizes the value back up" trap.
//! [`GUMTREE_TASTE_FLOOR`] is the named escape hatch: one world's own floor,
//! pinned under its own live measurement, replacing the shared upward target
//! for that world alone. A relaxed floor is not by itself safe — it is only
//! as good as its companion [`ORNAMENT_PRESENCE_FLOOR`], an absolute minimum
//! that ignores every per-world exception and is checked against the FULL
//! live roster, never a named world: a size law with no presence floor is
//! satisfied by an ornament that keeps shrinking toward nothing, and this is
//! what stops that on Gumtree today or on whichever world earns the next
//! taste exception.

use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view_md};

const W: u32 = 1200;
const H: u32 = 800;
/// The page measure this law renders at — an ordinary prose column, matching
/// the default a user would actually see (`theme::worlds::DEFAULT_THEME`'s own
/// world renders at this same default measure).
const MEASURE: usize = 70;

/// The fixed document: a heading (so the WYSIWYG/markdown gate is genuinely
/// live), a 26-letter reference row for the world's own char width, and one
/// `---` thematic break — followed by generous blank padding so the
/// differential's BLANK counterpart never lets its own later content (the
/// closing body paragraph) bleed into the rule row's comparison band. Sixteen
/// blank rows (512px at the default 32px line height) clears every roster
/// scale's tallest measured row (the widest observed, Gumtree's snake run,
/// row-boxes at ~150px).
const DOC_WITH_RULE: &str = "# Ornament Measurement\n\nabcdefghijklmnopqrstuvwxyz\n\n---\n\n\n\n\n\
                              \n\n\n\n\n\n\n\n\n\n\n\nBody text after the break, for reference.\n";
const DOC_BLANK: &str = "# Ornament Measurement\n\nabcdefghijklmnopqrstuvwxyz\n\n\n\n\n\n\
                          \n\n\n\n\n\n\n\n\n\n\n\nBody text after the break, for reference.\n";

const REF_ROW: &str = "abcdefghijklmnopqrstuvwxyz";
const RULE_ROW: &str = "---";

/// Per-channel summed abs diff a pixel must clear to count as ornament ink —
/// well above 8-bit quantization noise, well under a real glyph edge's step
/// (mirrors `zigzag_ground.rs::INK_FLOOR`'s own differential-oracle margin,
/// scaled up for RGB-only here vs that law's RGB scan too).
const INK_DIFF_FLOOR: i32 = 24;

fn render_doc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut crate::render::TextPipeline,
    text: &str,
) -> (Vec<[u8; 4]>, crate::render::LayoutReport) {
    p.set_view(&view_md(text, 0, 0));
    p.prepare(device, queue, W, H).expect("prepare failed");
    let report = p.layout_report().expect("sealed frame is reportable");
    let (texture, tview) = offscreen(device, W, H);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl ornament-scale test encoder"),
    });
    p.render(&mut encoder, &tview).expect("render failed");
    queue.submit(Some(encoder.finish()));
    let pixels = read_pixels(device, queue, &texture, W, H);
    (pixels, report)
}

/// The ornament's own ink bounding-box HEIGHT: rows, within the rule row's own
/// [top, top+height) band and the page COLUMN's own horizontal bounds (never
/// the margins — a margin ambient pattern, e.g. a starfield, can differ
/// between the two captures for reasons that have nothing to do with the
/// glyph), where the WITH-RULE and BLANK pixels differ past [`INK_DIFF_FLOOR`].
fn ornament_ink_height(
    pix_rule: &[[u8; 4]],
    pix_blank: &[[u8; 4]],
    top: u32,
    height: u32,
    col_left: u32,
    col_right: u32,
) -> u32 {
    let bot = (top + height).min(H);
    let mut min_y = None;
    let mut max_y = None;
    for y in top..bot {
        for x in col_left..col_right.min(W) {
            let idx = (y * W + x) as usize;
            let a = pix_rule[idx];
            let b = pix_blank[idx];
            let diff = (0..3)
                .map(|k| (a[k] as i32 - b[k] as i32).abs())
                .sum::<i32>();
            if diff > INK_DIFF_FLOOR {
                min_y = Some(min_y.map_or(y, |m: u32| m.min(y)));
                max_y = Some(max_y.map_or(y, |m: u32| m.max(y)));
            }
        }
    }
    match (min_y, max_y) {
        (Some(a), Some(b)) => b - a + 1,
        _ => 0,
    }
}

struct WorldRatio {
    name: &'static str,
    ratio: f32,
    ink_h: u32,
    char_w: f32,
}

fn measure_world(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut crate::render::TextPipeline,
    name: &'static str,
) -> WorldRatio {
    crate::theme::set_active_by_name(name).unwrap();
    p.sync_theme();

    let (pix_rule, report_rule) = render_doc(device, queue, p, DOC_WITH_RULE);
    let (pix_blank, _report_blank) = render_doc(device, queue, p, DOC_BLANK);

    let col_left = p.column_left().max(0.0).round() as u32;
    let col_w = p.column_width().max(0.0).round() as u32;
    let col_right = col_left + col_w;

    let ref_row = report_rule
        .rows
        .iter()
        .find(|r| r.content == REF_ROW)
        .unwrap_or_else(|| panic!("{name}: reference row {REF_ROW:?} not found in layout"));
    assert!(
        ref_row.xs.len() >= 2,
        "{name}: reference row shaped too few glyphs to measure a char width"
    );
    let char_w = (ref_row.xs[ref_row.xs.len() - 1] - ref_row.xs[0]) / 26.0;
    assert!(
        char_w > 0.0,
        "{name}: measured a non-positive char width {char_w}"
    );

    let rule_row = report_rule
        .rows
        .iter()
        .find(|r| r.content == RULE_ROW)
        .unwrap_or_else(|| panic!("{name}: thematic-break row {RULE_ROW:?} not found in layout"));
    let top = rule_row.top.max(0.0).round() as u32;
    let height = rule_row.height.max(0.0).round() as u32;

    let ink_h = ornament_ink_height(&pix_rule, &pix_blank, top, height, col_left, col_right);
    assert!(
        ink_h > 0,
        "{name}: the differential found NO ornament ink at all in the rule row's own \
         band — the row-box grew but nothing drew inside it, or the diff floor/column \
         bounds missed the glyph entirely"
    );

    WorldRatio {
        name,
        ratio: ink_h as f32 / char_w,
        ink_h,
        char_w,
    }
}

/// The tolerance band under the roster's own live target, for every world
/// still riding the shared upward equalization. `0.15` clears the real
/// measured spread among that group with real headroom (a law at the exact
/// measured minimum would go red on ordinary floating-point/rasterization
/// jitter) while staying far tighter than the ~53% spread the law was
/// written to catch (the shared-tier state: Wagtail's un-equalized 2.014
/// against Saltpan's 4.324). Gumtree no longer rides this band — see
/// [`GUMTREE_TASTE_FLOOR`] below — because its own live taste verdict moved
/// it deliberately smaller than the pack, not by drift.
const TOLERANCE: f32 = 0.15;

/// GUMTREE's own floor: its dash/star/underscore trio (the snake run, the
/// fish, the snail) is tuned smaller than the rest of the roster by a live
/// taste verdict, not by drift. The roster-wide UPWARD equalization above no
/// longer applies to this one world: it is smaller than the pack ON
/// PURPOSE, so it is measured against its OWN floor instead of
/// [`TOLERANCE`]'s roster-relative one — the fix that keeps this law from
/// re-equalizing the retune back up the next time it runs. Real headroom
/// under Gumtree's own live-measured ratio (3.487, at `ornament_scale` 3.95,
/// on this law's fixed document) so ordinary float/rasterization jitter
/// cannot flap it, while a further accidental shrink still fails here.
const GUMTREE_TASTE_FLOOR: f32 = 3.3;

/// The PRESENCE floor, independent of every per-world exception above: an
/// ornament ink-height/char-width ratio below this is not a smaller
/// drawing, it is one that has stopped drawing anything real. A per-world
/// taste floor (like Gumtree's) may relax the roster's UPWARD target, but it
/// may never relax past THIS ONE — checked against the FULL live roster
/// (`theme::THEMES`) in the loop below, never a named world, so a treatment
/// this weak on some FUTURE world fails here even before it earns its own
/// taste exception. Set comfortably below every world's current measured
/// ratio (including Gumtree's own 3.3 floor above) while staying well above
/// the pre-equalization broken state this law exists to catch (Wagtail's
/// un-equalized 2.014).
const ORNAMENT_PRESENCE_FLOOR: f32 = 2.5;

/// THE HEADLINE LAW — every world's ornament, EQUALIZED UPWARD.
///
/// Captures the fixed document across the FULL roster (`theme::THEMES`, never
/// a hand-picked subset), measures each world's own ornament ink height by
/// real differential pixel arithmetic, normalizes to that world's own
/// measured body char width, and pins the spread: every world's ratio must
/// land within [`TOLERANCE`] of the roster's live target (its own maximum —
/// derived from the roster, never a named pair), naming the offending world,
/// its ratio, and the floor it missed.
#[test]
fn every_world_ornament_ink_height_is_equalized_to_the_roster_target() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping every_world_ornament_ink_height_is_equalized_to_the_roster_target: \
             no wgpu adapter"
        );
        return;
    };
    // Held across every render AND the pixel readback below: the shared test
    // GPU's counters move on every prepare/render/read this closure performs,
    // not only on the call that first reached the device.
    let _g = crate::testlock::serial();
    let was_theme = crate::theme::active().name;
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(MEASURE);

    let measured: Vec<WorldRatio> = crate::theme::THEMES
        .iter()
        .map(|t| measure_world(&device, &queue, &mut p, t.name))
        .collect();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    crate::theme::set_active_by_name(was_theme).unwrap();
    drop(p);
    drop(queue);
    drop(device);

    // NON-VACUITY: a real roster, all twenty graded, real positive numbers —
    // a degenerate sweep (an empty roster, or every world silently skipped)
    // would otherwise satisfy every assertion below by grading nothing.
    assert_eq!(
        measured.len(),
        crate::theme::THEMES.len(),
        "graded a different count than the live roster"
    );
    assert_eq!(measured.len(), 20, "the roster is twenty worlds");
    for m in &measured {
        assert!(
            m.ratio.is_finite() && m.ratio > 0.0,
            "{}: non-finite or non-positive ratio {}",
            m.name,
            m.ratio
        );
    }

    let target = measured.iter().map(|m| m.ratio).fold(f32::MIN, f32::max);
    let roster_floor = target * (1.0 - TOLERANCE);

    for m in &measured {
        // The PRESENCE floor first, over the FULL roster with no per-world
        // exception — see `ORNAMENT_PRESENCE_FLOOR`'s own doc for why a
        // relaxed law needs this companion.
        assert!(
            m.ratio >= ORNAMENT_PRESENCE_FLOOR,
            "{}: ornament ink-height/char-width ratio {:.3} (ink {}px / char {:.3}px) \
             falls under the absolute presence floor {:.3} — the ornament has shrunk \
             toward nothing, not just smaller",
            m.name,
            m.ratio,
            m.ink_h,
            m.char_w,
            ORNAMENT_PRESENCE_FLOOR,
        );

        // Gumtree carries its own taste floor (retuned smaller on the user's
        // own live verdict); every other world stays on the roster's shared
        // upward-equalization floor.
        let (world_floor, floor_name) = if m.name == "Gumtree" {
            (GUMTREE_TASTE_FLOOR, "its own taste floor")
        } else {
            (roster_floor, "the roster target")
        };
        assert!(
            m.ratio >= world_floor,
            "{}: ornament ink-height/char-width ratio {:.3} (ink {}px / char {:.3}px) \
             falls under {} {:.3} — theme::worlds::{}'s ornament_scale needs another look",
            m.name,
            m.ratio,
            m.ink_h,
            m.char_w,
            floor_name,
            world_floor,
            m.name.to_uppercase(),
        );
    }
}
