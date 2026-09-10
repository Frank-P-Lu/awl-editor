//! The bullet ornament's real-pixel reveal law — the pixel companion to
//! `markdown::selected_bullet_row_drops_the_depth_glyph_keeps_its_dash`:
//! a selection merely TOUCHING a nested bullet row, caret elsewhere, must draw
//! ZERO ornament ink in that row — exactly like the already-correct
//! caret-ON-the-row state — never the "two markers on one row" the bug
//! produced. Mirrors `rule_reveal_state`'s pixel half for the sibling legacy
//! (pre-`ConcealKind`) construct, and reuses `awl_marks_pixels`'s own
//! with/without-ornament technique rather than re-deriving one.
//!
//! # Isolating the ornament's own ink
//!
//! A raw before/after screenshot diff conflates the glyph with the
//! selection-band WASH (a whole-row background tint that changes shape with
//! the selected column range, present whether or not the bug fires) and, on
//! the caret-row control, the caret's own block — neither is "how wide is the
//! ornament". The established isolation (`awl_marks_pixels::
//! every_rule_ornament_and_existing_bullet_is_legible_at_its_real_size`) sidesteps
//! both: prepare ONE frame normally (`with`), then flip `md_enabled` off and
//! re-run ONLY `prepare_ornaments` — leaving the already-shaped text buffer
//! (wash, revealed/concealed dash, caret) completely untouched — before
//! rendering a second frame (`without`) with an EMPTIED ornament layer. The
//! diff between the two is exactly the ornament's own ink, whatever else is
//! on screen.

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};

const W: u32 = 1200;
const H: u32 = 800;

/// Depth 0/1/2/3 nested list, one construct per line — [`CHILD_LINE`] (depth
/// 1, "mid") is the row a reader actually notices doubling on (the board's
/// `--keys "C-n C-n S-Down S-Down"` repro lands here): at depth 0 the glyph
/// sits ON TOP of the dash and hides it, so the double-draw is real but
/// harder to see; at depth 1+ the indent moves the glyph clear of the dash
/// and the double-draw reads as two separate marks.
const DOC: &str = "- top\n  * mid\n    + deep\n      - deeper\n";
const CHILD_LINE: usize = 1;

/// The ornament layer's own ink width in [`CHILD_LINE`]'s row band, for
/// whatever `ViewState` the caller hands in — `0` when the ornament renderer
/// contributes nothing there. `md_enabled` is left `true` on `p` afterward so
/// callers can chain probes without re-deriving the reset.
fn ornament_ink_width(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    v: &ViewState,
) -> i64 {
    p.set_view(v);
    p.prepare(device, queue, W, H).unwrap();
    let row_top = p.line_ornament_top(CHILD_LINE);
    let (y0, y1) = (
        row_top as i64,
        (row_top + p.metrics.line_height).ceil() as i64,
    );
    let with = pixeldiff::render_frame(p, device, queue, W, H);
    p.md_enabled = false;
    p.prepare_ornaments(device, queue, W, H).unwrap();
    let without = pixeldiff::render_frame(p, device, queue, W, H);
    p.md_enabled = true;

    let w = W as i64;
    let (mut left, mut right) = (i64::MAX, i64::MIN);
    for y in y0..y1 {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if with[idx] != without[idx] {
                left = left.min(x);
                right = right.max(x);
            }
        }
    }
    if left > right { 0 } else { right - left + 1 }
}

/// THE HEADLINE LAW: a selection touching the child (depth-1) row draws ZERO
/// ornament ink — exactly like the already-correct caret-on-the-row control —
/// while a THIRD state (caret elsewhere, no selection at all — the ordinary
/// concealed case) proves the measurement technique actually finds real ink
/// when the ornament DOES draw (the presence floor a pure absence claim could
/// pass by deleting its own subject).
#[test]
fn selected_child_bullet_row_draws_no_ornament_ink() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping selected_child_bullet_row_draws_no_ornament_ink: no wgpu adapter");
        return;
    };

    // PRESENCE (non-vacuity): caret elsewhere entirely, no selection — the
    // ordinary concealed state, where the ornament genuinely draws. This is
    // the "unselected...lane" ink the headline claim is measured against —
    // and it must be non-zero, or a law that finds zero everywhere proves
    // nothing below.
    let concealed = view_md(DOC, 4, 0);
    let concealed_width = ornament_ink_width(&mut p, &device, &queue, &concealed);
    assert!(
        concealed_width > 0,
        "presence floor failed: the ordinary concealed state must draw real ornament ink \
         (got {concealed_width}px) or this law's zero-width claims below prove nothing"
    );

    // CONTROL: caret ON the child row, no selection — the pre-existing,
    // already-correct reveal-on-cursor state.
    let caret_on = view_md(DOC, CHILD_LINE, 0);
    let caret_on_width = ornament_ink_width(&mut p, &device, &queue, &caret_on);
    assert_eq!(
        caret_on_width, 0,
        "caret-on-the-row must draw zero ornament ink (control regression)"
    );

    // THE CLAIM: caret elsewhere (trailing blank line), a selection merely
    // TOUCHING the child row.
    let mut selected = view_md(DOC, 4, 0);
    selected.selection = Some(((CHILD_LINE, 0), (CHILD_LINE, 7)));
    let selected_width = ornament_ink_width(&mut p, &device, &queue, &selected);
    assert_eq!(
        selected_width, 0,
        "a selection merely touching the child row (caret elsewhere) must draw zero ornament \
         ink too — {selected_width}px means the depth glyph is still stacked over the now-\
         revealed dash"
    );
    assert!(
        selected_width <= caret_on_width,
        "selected-row ornament ink ({selected_width}px) must be no wider than the unselected \
         caret-row lane ({caret_on_width}px)"
    );
}

/// THE SKIP-GATE AXIS at pixel scale: the caret's own line never moves across
/// the sequence (parked on the trailing blank line throughout) — only the
/// selection changes, driven as real `set_view` calls in order rather than
/// one snapshot `ViewState`, so a cache keyed on caret-line movement alone
/// cannot serve a stale (still double-drawn) frame.
#[test]
fn selection_added_without_moving_the_caret_still_drops_the_ornament_ink() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping selection_added_without_moving_the_caret_still_drops_the_ornament_ink: no \
             wgpu adapter"
        );
        return;
    };
    let caret_line = 4; // the trailing blank line, held fixed throughout

    // Frame 1: caret parked, no selection — the child row's ornament ink is
    // present (same presence floor as the headline law, re-affirmed here as
    // this sequence's own starting point).
    let none_view = view_md(DOC, caret_line, 0);
    let width_before = ornament_ink_width(&mut p, &device, &queue, &none_view);
    assert!(
        width_before > 0,
        "sequence start: the child row must draw real ornament ink before any selection exists"
    );

    // Frame 2: SAME caret line, only a selection touching the child row is
    // now active — the exact sequence a Shift-click or `S-Down` drag
    // produces without moving the caret's own LINE.
    let mut touch_view = view_md(DOC, caret_line, 0);
    touch_view.selection = Some(((CHILD_LINE, 0), (CHILD_LINE, 7)));
    let width_after = ornament_ink_width(&mut p, &device, &queue, &touch_view);
    assert_eq!(
        width_after, 0,
        "caret's own line held fixed, selection added over the child row: ornament ink must \
         drop to zero, not linger from a caret-line-only cache"
    );
}
