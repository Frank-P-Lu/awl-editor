//! WHERE THE WRITING COLUMN SITS — the pure page-column placement policy, carved
//! out of `geometry.rs` (which is three times the production ceiling) into its own
//! owner. Nothing here touches the GPU or `TextPipeline`: it is the window width,
//! the measure, the outline's rail appetite and the DISPLAY scale in, one `left`
//! and one `width` out, so the whole policy is unit-testable without a device.
//!
//! **EVERY authored length in this module is [`Logical`].** The page column lives in
//! [`page_column_advance`]'s ZOOM-STRIPPED space, so each pad resolves against the
//! DISPLAY scale alone (`dpi`) and never `Metrics::scale` — a pad that took the zoom
//! would move the margins as the user zoomed, which is the one thing the
//! zoom-independent column exists to prevent. A pad left UNRESOLVED is the sharper
//! failure: mixed with the already-scaled `outline_pref_px`/`gap` it made the
//! column's own left edge a function of the display, so a headed document at a
//! matched LOGICAL window sat at a different logical x on a 1x and a 2x screen and
//! the gutter's visibility boundary moved a whole `--measure` step with it.

use super::*;

pub const PAGE_MIN_PAD: Logical = TEXT_LEFT;
/// PAGE MODE column glyph ADVANCE (px): the char advance that DRIVES the page
/// column's pixel width — the base advance at zoom 1.0, still DPI-scaled, with the
/// user ZOOM divided back out. `char_width` is the LIVE (zoomed × DPI) advance
/// `metrics.char_width` (= `CHAR_WIDTH * zoom * dpi`); dividing by `zoom` recovers
/// `CHAR_WIDTH * dpi`, which depends on the DISPLAY only, never on the user zoom.
///
/// This is THE seam that DECOUPLES zoom from the page width: the column pixel width
/// (see [`column_width_for`]) is `measure * this`, so it tracks the WINDOW + the
/// settable measure but is INVARIANT under zoom. Zooming then only scales the glyph
/// metrics that SHAPE/wrap text INSIDE the fixed column — bigger glyphs, FEWER chars
/// per line, but the page surface + gutter + margins stay put. THIS is the space the
/// page's authored [`Logical`] pads resolve against too — the DISPLAY scale alone, never
/// `Metrics::scale` — so a pad holds its ratio to the page surface rather than to a
/// zoomed glyph, and the margins stay put under zoom exactly as the column does.
///
/// At zoom 1.0 (the deterministic capture path) this is an IDENTITY, so wide captures
/// stay byte-identical.
pub fn page_column_advance(char_width: f32, zoom: f32) -> f32 {
    if zoom > 0.0 {
        char_width / zoom
    } else {
        char_width
    }
}
/// PAGE MODE column WIDTH (px) for a given window width + ZOOM-INDEPENDENT glyph
/// advance (see [`page_column_advance`]) + page state + measure. The single source
/// of truth, factored out of [`TextPipeline::column_width`] so it is unit-testable
/// without a GPU device. NOTE: `char_width` here is the PAGE-COLUMN advance
/// ([`page_column_advance`]), NOT the live zoomed `metrics.char_width` — feeding the
/// zoom-stripped advance is what keeps the column (and its margins + gutter) constant
/// across zoom levels.
///
/// Edge-to-edge (`page_on == false`): the plain content width
/// `window - 2*NONPAGE_INSET` (a slightly wider side inset than page's collapse
/// floor, so a tad more ground shows). Page mode on, ONE responsive formula — no mode toggle,
/// smooth across a resize. The side margin is the GENEROUS [`page_min_margin`] when
/// the window has room for it, but it COLLAPSES toward the small uniform
/// [`PAGE_MIN_PAD`] as the measure crowds the width, so the column is:
///
/// ```text
/// margin = clamp((window - measure_px)/2, PAGE_MIN_PAD, page_min_margin(window))
/// column = min(measure_px, window - 2*margin)             // centered
/// ```
///
/// * WIDE window (room for the measure plus a generous band): the margin sits at the
///   generous `page_min_margin`, the column sits at the target measure
///   (`measure * char_width`), and the leftover splits into MARGINS — the gradient
///   pattern band and the gutter both show.
/// * NARROW window (the measure ≈ or exceeds the width): the margin collapses to the
///   small [`PAGE_MIN_PAD`] and the column FILLS the width minus that pad, so the
///   margins fall to ~0, the gutter + patterns auto-hide, and the page runs
///   effectively edge-to-edge.
///
/// EVERY authored length here — [`NONPAGE_INSET`], [`PAGE_MIN_PAD`], the
/// [`page_min_margin`] floor — is [`Logical`], resolved through `dpi`. Physical `window_w`
/// against a physical pad would make the whole formula a function of the DISPLAY.
pub fn column_width_for(
    window_w: f32,
    char_width: f32,
    page_on: bool,
    measure: usize,
    dpi: f32,
) -> f32 {
    let edge = (window_w - 2.0 * NONPAGE_INSET.px(dpi)).max(1.0);
    if !page_on {
        return edge;
    }
    let measure_px = measure as f32 * char_width;
    let margin =
        ((window_w - measure_px) * 0.5).clamp(PAGE_MIN_PAD.px(dpi), page_min_margin(window_w, dpi));
    let avail = (window_w - 2.0 * margin).max(1.0);
    measure_px.min(avail).max(1.0)
}

pub fn column_left_for(
    window_w: f32,
    char_width: f32,
    page_on: bool,
    measure: usize,
    dpi: f32,
) -> f32 {
    if !page_on {
        return NONPAGE_INSET.px(dpi);
    }
    let w = column_width_for(window_w, char_width, page_on, measure, dpi);
    ((window_w - w) * 0.5).max(PAGE_MIN_PAD.px(dpi))
}

/// ADAPTIVE-COLUMN PLACEMENT — the width-pressure policy behind the persistent
/// margin OUTLINE's rail (`render/chrome/outline.rs`). On a WIDE window the
/// centered column already leaves the outline a comfortable margin, so this is
/// a pure passthrough to [`column_left_for`] — **byte-identical to the
/// pre-round column position**, the hard law this round is built around. Only
/// once the SYMMETRIC left margin can't seat the outline's own preferred rail
/// (`outline_pref_px`, itself derived from [`crate::render::rowlayout::
/// OUTLINE_MIN_CHARS`] — never a parallel magic number) does the column shift
/// RIGHT to grant it, and only ever right: the column's WIDTH (its measure) is
/// never touched, so the writing column keeps its exact character count either
/// way — only where it SITS moves. The rightward shift is itself capped so a
/// [`RIGHT_MARGIN_BREATH`] sliver survives on the right, even under pressure;
/// once that cap would leave LESS than the outline's rail needs, the formula
/// naturally settles back on the plain symmetric `column_left_for` position
/// (see the doc comment on the final `else` arm) — the same "column
/// re-centers" the outline's own too-narrow-to-bother hide floor
/// (`rowlayout::OUTLINE_MIN_CHARS`) already falls back to, so the shift
/// threshold and the hide threshold can never drift apart: both are read off
/// this ONE `left`.
///
/// **The NO-PAYOFF guard (bugfix — a shift must EARN its keep):** the NARROW
/// branch used to shift right whenever `symmetric_left < desired_left`,
/// capped only by the right margin's breathing floor — with NO check that the
/// CAPPED shift actually buys the outline enough room to clear its own
/// [`rowlayout::OUTLINE_MIN_CHARS`] hide floor. On a window whose total
/// margin sits just past `RIGHT_MARGIN_BREATH` but well short of the
/// outline's MINIMUM viable rail, that produced a column that visibly shifts
/// right — shrinking the right margin toward the breathing floor — while the
/// outline stays hidden regardless: a shift with no payoff. This is reachable
/// at ordinary measures: confirmed live, `--measure 80` then "Reset page
/// width" on a ~1100px-wide window snaps the measure to the 70-char prose
/// default and lands exactly here (`left` shifts from a plain-centered 16 to
/// a wasted 76, right margin pinned to the breathing floor, outline still
/// hidden). `outline_min_px` (the pixel counterpart of `outline_pref_px`,
/// derived from the SAME `OUTLINE_MIN_CHARS` `outline_layout` itself hides
/// below) lets this function check that BEFORE committing to any shift: if
/// even the fully-capped `max_left` would leave the outline below its own
/// minimum, this returns the plain `symmetric_left` instead — the column
/// re-centers, exactly like the pre-existing NARROWEST tier, rather than
/// paying an asymmetric margin for a rail that will never draw.
///
/// **The ENTRY RAMP (resize-jitter fix, 2026-07-12 — user-reported live
/// bug):** the no-payoff guard above is a window-independent constant
/// (`min_left`) meeting a window-dependent one (`symmetric_left`) at its own
/// boundary, so a bare binary guard is discontinuous there BY CONSTRUCTION —
/// confirmed via a 1px resize sweep at the default 70-char measure: a SINGLE
/// pixel of window width flipped `left` from 61 to 107 (a 46px jump) the
/// instant `max_left` first cleared `min_left`. The last [`RIGHT_MARGIN_BREATH`]
/// px of approach (reusing the existing breathing-room constant, not a new
/// magic number) now LERPs from `symmetric_left` up to `min_left` instead of
/// snapping, so the column glides into the rail regime — see the guard's own
/// implementation comment for the exact band math. Well outside the ramp
/// band the guard is unchanged (a bare recenter, no wasted shift).
///
/// `outline_wants` is the outline's WIDTH-INDEPENDENT gate (feature on, page
/// mode on, a markdown buffer with at least one heading —
/// `TextPipeline::outline_wants_rail`) — everything BUT the horizontal-room
/// question this function itself decides.
///
/// **THE WHOLE-PIXEL SNAP (subpixel-shimmer fix, 2026-07-13 — the second,
/// surviving half of the user's resize-jitter report):** the final left is
/// FLOORED to a whole PHYSICAL pixel before being returned — the one owner
/// every downstream reader (glyph origins via `text_left`, caret, selection,
/// washes, hit-test) composes, so they all shift together. Why: the symmetric
/// centered left is `(window_w − measure_px) / 2`, which moves in **0.5px
/// steps** as a live resize drags the window 1px at a time. Glyph draw
/// origins inherit that fraction (glyphon feeds `TextArea.left` into
/// cosmic-text's `PhysicalGlyph::physical`, whose `SubpixelBin` quantizes the
/// fractional x into a rasterization bin) — so every SECOND pixel of window
/// width re-rasterized the entire column at a flipped antialiasing phase.
/// Measured on real captures (fixture at `--measure 40`): widths 1200 vs
/// 1202 (left 312.0 → 313.0, a whole-pixel shift) rendered the glyph band
/// BYTE-IDENTICAL under a 1px translation, while 1200 vs 1201 (left 312.0 →
/// 312.5) differed in **4.4% of the band's bytes** — the visible vibration
/// during a drag even though the placement math is perfectly smooth. With
/// the floor, a 1px resize moves the column by exactly 0 or 1 whole px — AA
/// phase stable, drag reads as a solid column sliding. FLOOR (not round) so
/// the snap can only ever move the column LEFT of the raw policy position —
/// the right-margin breathing floor (`RIGHT_MARGIN_BREATH`) is never eaten
/// by the snap, and floor-of-monotone stays monotone so the entry ramp's
/// no-jump law is preserved. DPI: `window_w`/`char_width` are PHYSICAL px
/// here, so this snaps to whole physical pixels — on a 2x display that is
/// 0.5 LOGICAL px, exactly the raster grid the glyphs rasterize on. The
/// even-width reference captures (1200px canvas, measures 40/70/80 → lefts
/// 312/96/24, all integral) are byte-identical under the snap.
// Column placement keeps each policy input explicit at the single authoritative seam.
#[allow(clippy::too_many_arguments)]
pub fn adaptive_column_left(
    window_w: f32,
    char_width: f32,
    page_on: bool,
    measure: usize,
    outline_wants: bool,
    outline_pref_px: f32,
    outline_min_px: f32,
    gap: f32,
    dpi: f32,
) -> f32 {
    adaptive_column_left_raw(
        window_w,
        char_width,
        page_on,
        measure,
        outline_wants,
        outline_pref_px,
        outline_min_px,
        gap,
        dpi,
    )
    .floor()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn adaptive_column_left_raw(
    window_w: f32,
    char_width: f32,
    page_on: bool,
    measure: usize,
    outline_wants: bool,
    outline_pref_px: f32,
    outline_min_px: f32,
    gap: f32,
    dpi: f32,
) -> f32 {
    let symmetric_left = column_left_for(window_w, char_width, page_on, measure, dpi);
    if !page_on || !outline_wants {
        return symmetric_left;
    }
    let width = column_width_for(window_w, char_width, page_on, measure, dpi);
    let total_margin = (window_w - width).max(0.0);
    // The rail's LEFT INSET and the right margin's BREATH are the same authored
    // pad, resolved once here: `outline_pref_px`/`gap` arrive already scaled, so a
    // pad left logical would make the whole placement a function of the display.
    let left_pad = PAGE_MIN_PAD.px(dpi);
    let breath = RIGHT_MARGIN_BREATH.px(dpi);
    let desired_left = outline_pref_px + gap + left_pad;
    let min_left = outline_min_px + gap + left_pad;
    if symmetric_left >= desired_left {
        return symmetric_left;
    }
    let max_left = (total_margin - breath).max(0.0);
    if max_left < min_left {
        let ramp_lo = min_left - breath;
        if max_left <= ramp_lo {
            return symmetric_left;
        }
        let t = ((max_left - ramp_lo) / breath).clamp(0.0, 1.0);
        return (symmetric_left + t * (min_left - symmetric_left)).max(symmetric_left);
    }
    desired_left.min(max_left).max(symmetric_left)
}

pub const RIGHT_MARGIN_BREATH: Logical = PAGE_MIN_PAD;
