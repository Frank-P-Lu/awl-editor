use super::*;

#[allow(dead_code)]
pub fn visible_lines(height: f32) -> usize {
    visible_lines_z(height, LINE_HEIGHT)
}

pub fn visible_lines_z(height: f32, line_height: f32) -> usize {
    ((height - TEXT_TOP) / line_height).floor().max(1.0) as usize
}

#[allow(dead_code)]
pub fn clamp_scroll(scroll_lines: usize, cursor_line: usize, height: f32) -> usize {
    clamp_scroll_z(scroll_lines, cursor_line, height, LINE_HEIGHT)
}

/// Zoom-aware cursor-follow scroll clamp, in the NON-WRAP model where the scroll
/// unit is a logical line (== a visual row when nothing wraps). The live app now
/// does cursor-follow in VISUAL rows (using the cursor's wrap-aware visual row),
/// but this is retained as the documented non-wrap reference + tested invariant:
/// when nothing wraps, `cursor_line` IS the cursor's visual row, so this matches.
#[allow(dead_code)]
pub fn clamp_scroll_z(
    scroll_lines: usize,
    cursor_line: usize,
    height: f32,
    line_height: f32,
) -> usize {
    let rows = visible_lines_z(height, line_height);
    let mut scroll = scroll_lines;
    if cursor_line < scroll {
        scroll = cursor_line;
    } else if cursor_line >= scroll + rows {
        scroll = cursor_line + 1 - rows;
    }
    scroll
}

#[allow(dead_code)]
pub fn max_scroll(total_visual_rows: usize, height: f32, line_height: f32) -> usize {
    let visible = visible_lines_z(height, line_height);
    let base = total_visual_rows.saturating_sub(visible);
    if base == 0 {
        return 0;
    }
    let overscroll = visible.saturating_sub(OVERSCROLL_KEEP_ROWS);
    base + overscroll
}

/// Pixel -> text hit-test. Given a click at `(px, py)` in physical pixels, the
/// current `scroll_lines`, the zoom `metrics`, and the column's `left` edge,
/// return the (line, col) the click maps to.
/// `line = scroll + floor((py - TEXT_TOP) / line_height)`;
/// `col = round((px - left) / char_width)`, both clamped to be >= 0. `left` is
/// the centered PAGE-MODE column left (or `TEXT_LEFT` edge-to-edge). The caller
/// clamps `line`/`col` to the actual buffer (via `line_col_to_char`), since this
/// function does not know the document. Mirrors EXACTLY the layout math used to
/// place glyphs + the caret, so a click lands on the right glyph.
pub fn hit_test(
    px: f32,
    py: f32,
    scroll_lines: usize,
    metrics: &Metrics,
    left: f32,
) -> (usize, usize) {
    let rel_y = (py - TEXT_TOP).max(0.0);
    let line = scroll_lines + (rel_y / metrics.line_height).floor() as usize;
    let rel_x = (px - left).max(0.0);
    let col = (rel_x / metrics.char_width).round() as usize;
    (line, col)
}

pub const PAGE_MIN_PAD: f32 = TEXT_LEFT;

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
/// per line, but the page surface + gutter + margins stay put. (Previously the column
/// used the zoomed advance directly, so zooming IN grew `measure_px` past the window
/// cap and collapsed the margins — the gutter vanished. This strips the zoom.)
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

pub fn zoom_anchor_target_top(anchor_top: f32, anchor_py: f32, menubar: f32) -> f32 {
    TEXT_TOP + menubar + anchor_top - anchor_py
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
/// (Previously the cap was the generous `page_min_margin` even at the full measure;
/// that over-reserved on narrow windows and squeezed the text into a sliver. Letting
/// the margin collapse fixes that while leaving WIDE captures — where the measure
/// binds well inside the available width — byte-identical.)
pub fn column_width_for(window_w: f32, char_width: f32, page_on: bool, measure: usize) -> f32 {
    let edge = (window_w - 2.0 * NONPAGE_INSET).max(1.0);
    if !page_on {
        return edge;
    }
    let measure_px = measure as f32 * char_width;
    let margin = ((window_w - measure_px) * 0.5).clamp(PAGE_MIN_PAD, page_min_margin(window_w));
    let avail = (window_w - 2.0 * margin).max(1.0);
    measure_px.min(avail).max(1.0)
}

pub fn column_left_for(window_w: f32, char_width: f32, page_on: bool, measure: usize) -> f32 {
    if !page_on {
        return NONPAGE_INSET;
    }
    let w = column_width_for(window_w, char_width, page_on, measure);
    ((window_w - w) * 0.5).max(PAGE_MIN_PAD)
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
    left_pad: f32,
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
        left_pad,
    )
    .floor()
}

#[allow(clippy::too_many_arguments)]
fn adaptive_column_left_raw(
    window_w: f32,
    char_width: f32,
    page_on: bool,
    measure: usize,
    outline_wants: bool,
    outline_pref_px: f32,
    outline_min_px: f32,
    gap: f32,
    left_pad: f32,
) -> f32 {
    let symmetric_left = column_left_for(window_w, char_width, page_on, measure);
    if !page_on || !outline_wants {
        return symmetric_left;
    }
    let width = column_width_for(window_w, char_width, page_on, measure);
    let total_margin = (window_w - width).max(0.0);
    let desired_left = outline_pref_px + gap + left_pad;
    let min_left = outline_min_px + gap + left_pad;
    if symmetric_left >= desired_left {
        return symmetric_left;
    }
    let max_left = (total_margin - RIGHT_MARGIN_BREATH).max(0.0);
    if max_left < min_left {
        let ramp_lo = min_left - RIGHT_MARGIN_BREATH;
        if max_left <= ramp_lo {
            return symmetric_left;
        }
        let t = ((max_left - ramp_lo) / RIGHT_MARGIN_BREATH).clamp(0.0, 1.0);
        return (symmetric_left + t * (min_left - symmetric_left)).max(symmetric_left);
    }
    desired_left.min(max_left).max(symmetric_left)
}

pub const RIGHT_MARGIN_BREATH: f32 = PAGE_MIN_PAD;

/// BLOCKQUOTE pull-quote DROP-CAP x (px): the left origin of the big hanging
/// opening-quote mark. It hangs in the writing column's own left text-pad gutter —
/// its RIGHT edge a hair (`gap`) shy of `text_left` (the quote text's own left
/// edge, so the text clears it) — with its LEFT edge clamped to `column_left` so it
/// can NEVER spill back out of the page into the left margin, which belongs to the
/// OUTLINE alone. Pure so the placement law (`text ≥ right edge`, `left ≥
/// column_left`) is unit-testable without a GPU. `mark_w` is the mark's shaped
/// advance; `gap` the small clearance before the text.
pub(super) fn pull_quote_left(column_left: f32, text_left: f32, gap: f32, mark_w: f32) -> f32 {
    (text_left - gap - mark_w).max(column_left)
}

pub const PAGE_RESIZE_GRAB_PX: f32 = 6.0;

/// A glyph cell whose advance is below this fraction of `metrics.char_width` is
/// DEGENERATE — a collapsed / glyphless mid-line cell rather than a real narrow
/// glyph. The canonical case is the SPACE at a soft-wrap boundary: cosmic-text
/// collapses the trailing whitespace at the break, so the cell's two x boundaries
/// coincide at the row's right edge and its raw width is ~0 (the block-caret
/// "1px sliver" bug). [`TextPipeline::col_x_and_advance`] rescues such a cell to
/// the default `char_width`, exactly like its end-of-line fallback. The fraction
/// is deliberately tiny relative to any REAL advance — the narrowest genuine
/// glyphs (a proportional `i`/`l` ≈ 0.25em, even a hair space ≈ 0.1em) sit well
/// above it at every zoom (both sides scale with zoom × dpi), so only truly
/// collapsed cells are rescued and thin glyphs keep their exact advance.
pub(super) const DEGENERATE_CELL_FRAC: f32 = 0.1;

pub(super) fn xray_col_x(x: &crate::render::XrayRow, col: usize, char_width: f32) -> (f32, f32) {
    let n = x.glyph_xs.len().saturating_sub(1); // char count on the source row
    let c = col.min(n);
    let gx = x.glyph_xs.get(c).copied().unwrap_or(0.0) - x.pan;
    let advance = if c < n {
        (x.glyph_xs[c + 1] - x.glyph_xs[c]).max(char_width * DEGENERATE_CELL_FRAC)
    } else {
        char_width
    };
    (gx, advance)
}

pub(super) fn xray_pan_for_caret(
    caret_x: f32,
    content_w: f32,
    view_w: f32,
    pad: f32,
    prev: f32,
) -> f32 {
    let max_pan = (content_w - view_w).max(0.0);
    if max_pan <= 0.0 {
        return 0.0;
    }
    let prev = prev.clamp(0.0, max_pan);
    let lo = prev + pad;
    let hi = prev + view_w - pad;
    let pan = if caret_x < lo {
        (caret_x - pad).max(0.0)
    } else if caret_x > hi {
        (caret_x - view_w + pad).min(max_pan)
    } else {
        prev
    };
    pan.clamp(0.0, max_pan)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResizeEdge {
    Left,
    Right,
}

pub fn page_boundary_hit(
    pointer_x: f32,
    column_left: f32,
    column_width: f32,
    tol: f32,
) -> Option<ResizeEdge> {
    let right = column_left + column_width;
    let dl = (pointer_x - column_left).abs();
    let dr = (pointer_x - right).abs();
    if dl <= tol && dl <= dr {
        Some(ResizeEdge::Left)
    } else if dr <= tol {
        Some(ResizeEdge::Right)
    } else {
        None
    }
}

/// THE ONE OWNER of "does a press/hover at `pointer_x` arm the page-width resize
/// affordance?" — the full decision behind [`TextPipeline::page_resize_edge_at`],
/// pulled out as a pure fn so the arming LAW is testable without a GPU pipeline. The
/// rule is exactly two clauses: page mode must be ON, and the pointer must be within
/// `tol` of a DRAWN column edge ([`page_boundary_hit`] against the column's real
/// `left`/`width`). There is DELIBERATELY no "collapsed page has no handle" gate —
/// that earlier taste guard (`left <= PAGE_MIN_PAD + 1.0 → None`) locked the user out
/// of dragging a widened-past-capacity column back inward (bug, 2026-07-15). A
/// collapsed column pins both edges at the [`PAGE_MIN_PAD`] margins, and those edges
/// stay grabbable so the width can be pulled back down ([`page_resize_measure_anchored`]
/// clamps the drag result to the settable band regardless).
pub fn page_resize_edge_hit(
    page_on: bool,
    column_left: f32,
    column_width: f32,
    pointer_x: f32,
    tol: f32,
) -> Option<ResizeEdge> {
    if !page_on {
        return None;
    }
    page_boundary_hit(pointer_x, column_left, column_width, tol)
}

/// CURSOR SHAPE — is `pointer_x` within a column's horizontal extent
/// (`column_left` .. `column_left + column_width`, inclusive of both edges)?
/// The membership counterpart to [`page_boundary_hit`]'s proximity test: pure,
/// so the "is the pointer over document TEXT" half of the context-aware OS
/// cursor (`cursor_shape::CursorContext::over_text`,
/// `TextPipeline::over_writing_column`) is unit-testable without a GPU.
pub fn in_writing_column(pointer_x: f32, column_left: f32, column_width: f32) -> bool {
    pointer_x >= column_left && pointer_x <= column_left + column_width
}

pub fn page_resize_measure_anchored(
    advance: f32,
    pointer_x: f32,
    anchor_x: f32,
    edge: ResizeEdge,
) -> usize {
    let width = match edge {
        ResizeEdge::Right => pointer_x - anchor_x,
        ResizeEdge::Left => anchor_x - pointer_x,
    };
    let width = width.max(1.0);
    let measure = if advance > 0.0 {
        (width / advance).round()
    } else {
        0.0
    };
    (measure.max(0.0) as usize).clamp(crate::page::MIN_MEASURE, crate::page::MAX_MEASURE)
}

pub const IMAGE_RESIZE_GRAB_PX: f32 = 12.0;

/// The MINIMUM display width (px) a drag can shrink an inline image to — a floor so
/// a drag can never collapse the image to nothing (and pairs with the fit-to-column
/// MAX, the text wrap width). Companion to [`crate::page::MIN_MEASURE`] for images.
///
/// TASTE TUNABLE (flagged for live review): the clamp floor. Matches the task's
/// stated `[64, column width]` band — a `|64` hint is the smallest an image can be
/// dragged to; the ceiling is the writing column width (fit-to-column).
pub const MIN_IMAGE_W: f32 = 64.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImageHandle {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Is `pointer` within `tol` px of an EDGE or CORNER of an image whose on-screen
/// rect is `image_rect` = `[left, top, w, h]`? Returns which handle (edge/corner)
/// the pointer grabs, CORNERS FIRST (a corner is the intersection of two edges, so
/// its diagonal grip wins over either edge where they meet). An edge only arms
/// within the perpendicular SPAN of the image (plus `tol` slop), so a pointer far
/// above/below the left edge never arms it. Pure — the caller supplies the rect from
/// the SAME images layout the `ImageQuadPipeline` draws + the sidecar reports (no
/// parallel geometry), and gates on the feature being on; this only does the border
/// proximity. The proximity counterpart to [`page_boundary_hit`], unit-testable
/// without a GPU.
pub fn image_handle_hit(
    pointer: (f32, f32),
    image_rect: [f32; 4],
    tol: f32,
) -> Option<ImageHandle> {
    let [left, top, w, h] = image_rect;
    let (px, py) = pointer;
    let right = left + w;
    let bottom = top + h;
    let near_l = (px - left).abs() <= tol;
    let near_r = (px - right).abs() <= tol;
    let near_t = (py - top).abs() <= tol;
    let near_b = (py - bottom).abs() <= tol;
    let in_x = px >= left - tol && px <= right + tol;
    let in_y = py >= top - tol && py <= bottom + tol;
    if near_l && near_t {
        Some(ImageHandle::TopLeft)
    } else if near_r && near_t {
        Some(ImageHandle::TopRight)
    } else if near_l && near_b {
        Some(ImageHandle::BottomLeft)
    } else if near_r && near_b {
        Some(ImageHandle::BottomRight)
    } else if near_l && in_y {
        Some(ImageHandle::Left)
    } else if near_r && in_y {
        Some(ImageHandle::Right)
    } else if near_t && in_x {
        Some(ImageHandle::Top)
    } else if near_b && in_x {
        Some(ImageHandle::Bottom)
    } else {
        None
    }
}

fn diagonal_width(gx: f32, gy: f32, w: f32, h: f32) -> f32 {
    let denom = w * w + h * h;
    if denom <= 0.0 {
        return gx.max(gy);
    }
    w * (gx * w + gy * h) / denom
}

/// The new DISPLAY WIDTH (px) an inline image gets from dragging one of its edges or
/// corners (`handle`) to `pointer`, given the image's PRESS-TIME on-screen `rect`
/// `[left, top, w, h]`. Direct manipulation, ALWAYS aspect-locked (only a width is
/// ever produced — the height rides the fixed aspect, so no distortion): the OPPOSITE
/// edge/corner is the fixed anchor and the grabbed one tracks the pointer.
///   * left/right edges — the pointer's `x` distance past the anchored edge drives.
///   * top/bottom edges — the pointer's `y` distance past the anchored edge drives,
///     converted to a width through the fixed aspect (`w/h`).
///   * corners — the diagonal projection ([`diagonal_width`]) of the pointer's growth
///     from the anchored corner drives.
///     Clamped to `[min, wrap]` and ADDITIONALLY to the width whose IMPLIED height
///     (at the rect's own fixed aspect) hits `max_h` — the SAME
///     [`super::spans::IMAGE_MAX_VIEWPORT_FRAC`]-scaled viewport ceiling
///     [`super::spans::image_display_size`] enforces on the undragged fit-to-column
///     size, so a drag can never grow an image past the height cap either. Never
///     below [`MIN_IMAGE_W`] and never past the writing-column `wrap` width (the
///     fit-to-column ceiling). A non-positive `max_h` disables that half of the
///     clamp (matches [`super::spans::image_display_size`]'s own escape hatch).
///     Pure, so the px→width mapping is unit-testable without a GPU.
pub fn image_resize_width(
    handle: ImageHandle,
    rect: [f32; 4],
    pointer: (f32, f32),
    wrap: f32,
    min: f32,
    max_h: f32,
) -> f32 {
    let [left, top, w, h] = rect;
    let (px, py) = pointer;
    let right = left + w;
    let bottom = top + h;
    let aspect = if h > 0.0 { w / h } else { 1.0 };
    let raw = match handle {
        ImageHandle::Right => px - left,
        ImageHandle::Left => right - px,
        ImageHandle::Bottom => (py - top) * aspect,
        ImageHandle::Top => (bottom - py) * aspect,
        ImageHandle::BottomRight => diagonal_width(px - left, py - top, w, h),
        ImageHandle::TopLeft => diagonal_width(right - px, bottom - py, w, h),
        ImageHandle::TopRight => diagonal_width(px - left, bottom - py, w, h),
        ImageHandle::BottomLeft => diagonal_width(right - px, py - top, w, h),
    };
    let height_ceil = if max_h > 0.0 {
        (max_h * aspect).max(min)
    } else {
        f32::INFINITY
    };
    raw.clamp(min, wrap.max(min).min(height_ceil))
}

/// Choose the visual row of `rows` that owns char column `col`. A column is owned
/// by the row whose `[start_col, end_col)` contains it; at a wrap boundary the
/// column equals both the previous row's `end_col` and the next row's
/// `start_col`, and the NEXT (lower) row wins — that is where the caret sits when
/// you move onto a wrapped continuation. Past the logical end-of-line (col ==
/// last row's end_col with no following row) the LAST row is used. `rows` is
/// never empty (see [`TextPipeline::visual_rows`]).
pub(super) fn pick_row(rows: &[VisualRow], col: usize) -> &VisualRow {
    &rows[pick_row_index(rows, col)]
}

/// [`pick_row`] with a caret wrap `affinity`. `Downstream` is byte-identical to
/// `pick_row` (the historical lower-row bias). `Upstream` resolves a SHARED wrap
/// boundary (`col` == a row's `end_col` that also opens the next row) to the UPPER
/// row instead — the row whose TRAILING edge is `col` — so a caret parked at the
/// visual-row end (right after C-e / End / Cmd-Right) renders on that row's right
/// edge, not the lower row's left. At any NON-boundary column exactly one row owns
/// `col`, so affinity is inert and this is identical to `pick_row`.
pub(super) fn pick_row_aff(
    rows: &[VisualRow],
    col: usize,
    affinity: crate::caret::Affinity,
) -> &VisualRow {
    &rows[pick_row_index_aff(rows, col, affinity)]
}

pub(super) fn pick_row_index_aff(
    rows: &[VisualRow],
    col: usize,
    affinity: crate::caret::Affinity,
) -> usize {
    if affinity == crate::caret::Affinity::Upstream {
        // Upstream affinity selects the visual row ending at the boundary.
        if let Some(i) = rows
            .iter()
            .position(|r| r.end_col == col && r.start_col < col)
        {
            return i;
        }
    }
    pick_row_index(rows, col)
}

/// The INDEX form of [`pick_row`]: the position within `rows` of the visual row
/// that owns char column `col`, with the identical wrap-boundary bias (the later
/// row wins at a boundary). Used by the visual-motion oracle to step to the
/// adjacent (up/down) row, while [`pick_row`] keeps returning the reference its
/// existing callers want. `rows` is never empty (see [`TextPipeline::visual_rows`]).
pub(super) fn pick_row_index(rows: &[VisualRow], col: usize) -> usize {
    // First, a row that strictly contains the column in its half-open span: this
    // also resolves the wrap boundary in favor of the later row (its start_col).
    for (i, r) in rows.iter().enumerate() {
        if col >= r.start_col && col < r.end_col {
            return i;
        }
    }
    rows.iter()
        .enumerate()
        .rev()
        .find(|(_, r)| col >= r.start_col)
        .map(|(i, _)| i)
        .unwrap_or(rows.len().saturating_sub(1))
}

/// The pixel `(x, width)` of a `[s, e)` char-column span on one visual `row`,
/// from that row's own x boundaries (`xs[s]`..`xs[e]`, offset by `text_left`). The
/// width is floored at `min_w` so a zero-width span still shows a sliver where the
/// caller wants one. `s`/`e` must already be clamped to the row's column count.
/// Shared by the squiggle / selection / preedit rect builders.
pub(super) fn row_x_span(
    row: &VisualRow,
    text_left: f32,
    s: usize,
    e: usize,
    min_w: f32,
) -> (f32, f32) {
    let xs_s = row.xs.get(s).copied().unwrap_or(0.0);
    let xs_e = row.xs.get(e).copied().unwrap_or(xs_s);
    let x = text_left + xs_s;
    let w = (xs_e - xs_s).max(min_w);
    (x, w)
}

/// Assemble ONE [`VisualRow`] from a shaped layout `run` of the logical line whose
/// text is `line_text` — the per-run body shared VERBATIM by
/// [`TextPipeline::visual_rows`] and [`TextPipeline::visual_rows_for_lines`], so
/// the two sources produce byte-identical rows. Gathers the run's glyph clusters,
/// maps its byte range onto the full line's char columns (`assemble_glyph_xs`
/// keys off the line text, so the returned vector is char_count+1 long; only
/// columns within this run's byte span carry real x's, the rest are
/// forward-filled — callers index it by GLOBAL char column and clamp to this
/// row's [start_col,end_col]), and carries the run's wrap-aware top/height.
pub(super) fn visual_row_from_run(
    line_text: &str,
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    char_width: f32,
) -> VisualRow {
    let mut clusters: Vec<(usize, usize, f32, f32)> = Vec::new();
    let mut byte_start = usize::MAX;
    let mut byte_end = 0usize;
    for g in run.glyphs.iter() {
        clusters.push((g.start, g.end, g.x, g.x + g.w));
        byte_start = byte_start.min(g.start);
        byte_end = byte_end.max(g.end);
    }
    if byte_start == usize::MAX {
        byte_start = 0;
        byte_end = 0;
    }
    let xs = assemble_glyph_xs(line_text, &clusters, char_width);
    let start_col = byte_col(line_text, byte_start);
    let end_col = byte_col(line_text, byte_end);
    VisualRow {
        line_top: run.line_top,
        line_height: run.line_height,
        start_col,
        end_col,
        xs,
    }
}

/// Build the per-CHAR x boundaries for a line from its shaped glyph CLUSTERS.
///
/// `clusters` are `(start_byte, end_byte, left_x, right_x)` tuples (byte ranges
/// into `line_text`, pixel x's relative to the text left). Returns `char_count+1`
/// boundaries: `xs[col]` is the left edge of the cell at char-column `col`, and
/// `xs[char_count]` is the right edge of the last glyph (end of line).
///
/// This is the core char<->byte + advance mapping for advance-aware layout, kept
/// as a pure free function so the CJK (multi-byte) behavior is unit-testable
/// without a GPU. `char_width` is the fixed-pitch fallback used for empty /
/// glyphless lines.
///
/// LIGATURE CLUSTERS — the general N-source-chars → M-glyphs case. A single
/// `(start_byte, end_byte)` cluster SPAN may be shaped by several glyphs, all
/// stamped with that SAME span:
///   * `M < N` — a TRUE ligature (`fi`/`fl`, or `->` on a `calt` mono) collapses
///     several source chars into ONE glyph carrying the whole span.
///   * `M = N` — Monaspace Xenon's AAT/`morx` "texture-healing" ligatures
///     (`=> != -> >= <= == ::`) emit one glyph PER source char but stamp EVERY
///     one with the SAME (start,end) span (unsuppressable by OpenType features).
///     Either way the fix is one rule: gather the whole GROUP of consecutive glyphs
///     that share a span, take its COMBINED advance `A = (max right x) − (min left
/// x)` across all `M` glyphs, and distribute the `(end − start)` source chars
///     EVENLY over it — char `i` sits at `group_left + (i − start) · A / (end −
/// start)`. Splitting one glyph's advance fairly across its chars (`M<N`) and
///     summing several glyphs' advances into a uniform grid (`M=N`) fall out of the
///     same formula. Taking only the FIRST glyph's advance (the old behavior)
///     collapsed a texture-healed `=>` to a half-pitch interior column, mismapping
///     the caret / selection / click on every Monaspace code line with an operator.
pub(super) fn assemble_glyph_xs(
    line_text: &str,
    clusters: &[(usize, usize, f32, f32)],
    char_width: f32,
) -> Vec<f32> {
    #[cfg(test)]
    GLYPH_X_ASSEMBLIES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let char_count = line_text.chars().count();
    let mut byte_to_col = vec![char_count; line_text.len() + 1];
    for (col, (b, _)) in line_text.char_indices().enumerate() {
        byte_to_col[b] = col;
    }
    byte_to_col[line_text.len()] = char_count;

    let mut xs = vec![f32::NAN; char_count + 1];
    let mut max_right = 0.0f32;
    let any = !clusters.is_empty();
    // Walk the glyph clusters, GROUPING consecutive glyphs that share the exact
    // same (start_byte, end_byte) span into one logical cluster (LTR shaping
    // emits a span's glyphs contiguously, so a linear scan finds the whole
    // group). The group's COMBINED advance — max right minus min left across ITS
    // glyphs — is what the source chars are spread over, so a texture-healed
    // ligature (several glyphs, one span) yields a uniform grid instead of the
    // first glyph's advance winning and halving the interior columns.
    let mut i = 0;
    while i < clusters.len() {
        let (start_b, end_b, _, _) = clusters[i];
        let mut group_left = f32::INFINITY;
        let mut group_right = f32::NEG_INFINITY;
        let mut j = i;
        while j < clusters.len() && clusters[j].0 == start_b && clusters[j].1 == end_b {
            group_left = group_left.min(clusters[j].2);
            group_right = group_right.max(clusters[j].3);
            j += 1;
        }
        i = j;

        let start_col = byte_to_col
            .get(start_b)
            .copied()
            .unwrap_or(char_count)
            .min(char_count);
        let end_col = byte_to_col
            .get(end_b)
            .copied()
            .unwrap_or(char_count)
            .min(char_count);
        max_right = max_right.max(group_right);
        if xs[start_col].is_nan() {
            xs[start_col] = group_left;
        }
        let span = end_col.saturating_sub(start_col).max(1);
        for k in 1..=span {
            let col = start_col + k;
            if col <= char_count {
                let frac = k as f32 / span as f32;
                let x = group_left + (group_right - group_left) * frac;
                if xs[col].is_nan() {
                    xs[col] = x;
                }
            }
        }
    }

    if !any {
        // Empty or unshaped line: fixed-pitch fallback so the caret cell and any
        // selection sliver still render where a Latin glyph would sit.
        return (0..=char_count).map(|c| c as f32 * char_width).collect();
    }

    if xs[0].is_nan() {
        xs[0] = 0.0;
    }
    for i in 1..xs.len() {
        if xs[i].is_nan() {
            xs[i] = xs[i - 1].max(max_right);
        }
    }
    if let Some(last) = xs.last_mut() {
        *last = last.max(max_right);
    }
    xs
}

#[cfg(test)]
static GLYPH_X_ASSEMBLIES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(super) fn reset_glyph_x_assembly_count() {
    GLYPH_X_ASSEMBLIES.store(0, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
pub(super) fn glyph_x_assembly_count() -> usize {
    GLYPH_X_ASSEMBLIES.load(std::sync::atomic::Ordering::Relaxed)
}

/// The char SPAN of the glyph CLUSTER (a `(start_byte, end_byte)` pair — one
/// entry per shaped glyph, the same clustering `assemble_glyph_xs` reads) that
/// owns byte `cur_byte` on `line_text`: `end_col - start_col`, clamped to at
/// least 1. `None` when no cluster in `clusters` owns `cur_byte`.
///
/// `1` is the overwhelmingly common case (one glyph per char); `>1` is a
/// LIGATURE — several chars shape into a single glyph (e.g. an "fi"/"ffi"
/// fixture on a font that ligates it). This is what
/// [`TextPipeline::caret_anchor_ink_box`](super::caret) reads to decide whether
/// a caret anchor may safely be replaced by its glyph's own ink box (a 1-char
/// cluster IS that glyph, one-to-one) or must keep the CELL math's fair linear
/// split (a multi-char cluster's cell already spreads one glyph's ink fairly
/// across the chars it covers).
///
/// Kept free + pure (no GPU / no live shaping), mirroring `assemble_glyph_xs`,
/// so the ligature-fallback decision is unit-testable with a SYNTHETIC
/// multi-char cluster — no bundled awl font actually ligates "fi"/"ffi" under
/// the current shaper (verified empirically across every world), so this is
/// the only way to exercise that branch.
pub(super) fn cluster_span_at(
    line_text: &str,
    clusters: &[(usize, usize)],
    cur_byte: usize,
) -> Option<usize> {
    for &(start_b, end_b) in clusters {
        if cur_byte >= start_b && cur_byte < end_b {
            let start_col = byte_col(line_text, start_b);
            let end_col = byte_col(line_text, end_b);
            return Some(end_col.saturating_sub(start_col).max(1));
        }
    }
    None
}

impl TextPipeline {
    pub(super) fn page_advance(&self) -> f32 {
        page_column_advance(self.metrics.char_width, self.metrics.zoom)
    }

    pub fn column_width(&self) -> f32 {
        column_width_for(
            self.window_w,
            self.page_advance(),
            crate::page::page_on(),
            crate::page::measure(),
        )
    }

    /// PAGE MODE: the LEFT edge (px) of the writing column — the ONE owner every
    /// downstream reader (caret/selection/washes, hit-test, the page-edge drag
    /// handle, the corner readouts, the margin outline + gutter) goes through, so
    /// the ADAPTIVE-COLUMN placement policy ([`adaptive_column_left`]) composes
    /// for free everywhere without a parallel geometry to keep in sync. WIDE: a
    /// byte-identical passthrough to [`column_left_for`]. NARROW + the margin
    /// outline wanting its rail ([`Self::outline_wants_rail`]): shifts right per
    /// [`adaptive_column_left`]'s pressure test. Zoom-independent (driven by
    /// [`Self::page_advance`]).
    pub fn column_left(&self) -> f32 {
        let label = crate::markdown::type_scale::LABEL;
        let char_width = self.page_advance();
        adaptive_column_left(
            self.window_w,
            char_width,
            crate::page::page_on(),
            crate::page::measure(),
            self.outline_wants_rail(),
            rowlayout::OUTLINE_PREFERRED_CHARS as f32 * self.metrics.char_width * label,
            rowlayout::OUTLINE_MIN_CHARS as f32 * self.metrics.char_width * label,
            self.metrics.char_width * crate::render::chrome::MARGIN_COLUMN_GAP_CHARS,
            crate::render::TEXT_LEFT,
        )
    }

    pub(in crate::render) fn outline_wants_rail(&self) -> bool {
        crate::outline::outline_on()
            && crate::page::page_on()
            && self.md_enabled
            && !self.outline_headings.is_empty()
    }

    /// DIRECT-MANIPULATION resize — is the pointer at `pointer_x` (physical px)
    /// hovering a DRAGGABLE page-column edge? True whenever page mode is ON and the
    /// pointer is within [`PAGE_RESIZE_GRAB_PX`] of a DRAWN column edge — including a
    /// COLLAPSED page whose margins sit at the [`PAGE_MIN_PAD`] floor. The edge is
    /// the affordance whether or not there is margin left to give: dragging INWARD
    /// from a collapsed column must still narrow the measure (else the user is locked
    /// out — the widen-past-capacity lockout bug, 2026-07-15). The pure proximity
    /// test is [`page_boundary_hit`]. The live app reads this to flip the OS cursor
    /// to a resize glyph and to decide whether a press begins a width drag instead of
    /// a text selection.
    pub fn page_resize_hover(&self, pointer_x: f32) -> bool {
        self.page_resize_edge_at(pointer_x).is_some()
    }

    pub fn page_resize_edge_at(&self, pointer_x: f32) -> Option<ResizeEdge> {
        page_resize_edge_hit(
            crate::page::page_on(),
            self.column_left(),
            self.column_width(),
            pointer_x,
            PAGE_RESIZE_GRAB_PX,
        )
    }

    pub fn over_writing_column(&self, pointer_x: f32) -> bool {
        in_writing_column(pointer_x, self.column_left(), self.column_width())
    }

    pub fn page_resize_measure_at(&self, pointer_x: f32, edge: ResizeEdge, anchor_x: f32) -> usize {
        page_resize_measure_anchored(self.page_advance(), pointer_x, anchor_x, edge)
    }

    /// INLINE-IMAGE DRAG-RESIZE (v2) — the DISPLAY WIDTH (px) an image gets from
    /// dragging its `handle` (edge/corner) to `pointer`, given its PRESS-TIME on-screen
    /// `rect` `[left, top, w, h]`: the pure [`image_resize_width`] clamped to
    /// `[MIN_IMAGE_W, text_wrap_width()]` AND the same viewport-height ceiling
    /// [`super::spans::image_display_size`] applies to the undragged fit-to-column
    /// size — a drag can grow an image no taller than [`super::spans::IMAGE_MAX_VIEWPORT_FRAC`]
    /// of the window. Mirrors [`Self::page_resize_measure_at`] — the app supplies the
    /// handle + press rect + pointer, the pipeline owns the column geometry (the
    /// fit-to-column wrap ceiling) and the window height, so no raw geometry leaks
    /// to the app.
    pub fn image_resize_width_at(
        &self,
        handle: ImageHandle,
        rect: [f32; 4],
        pointer: (f32, f32),
    ) -> f32 {
        let max_h = self.window_h * super::spans::IMAGE_MAX_VIEWPORT_FRAC;
        image_resize_width(
            handle,
            rect,
            pointer,
            self.text_wrap_width(),
            MIN_IMAGE_W,
            max_h,
        )
    }

    pub fn page_geometry(&self) -> (bool, usize, f32, f32) {
        (
            crate::page::page_on(),
            crate::page::measure(),
            self.column_left(),
            self.column_width(),
        )
    }

    pub fn page_class(&self) -> crate::page::PageClass {
        crate::page::PageClass::of_syntax(self.syn_lang)
    }

    pub(super) fn text_pad(&self) -> f32 {
        if crate::page::page_on() {
            self.metrics.char_width * PAGE_TEXT_PAD_CHARS
        } else {
            0.0
        }
    }

    /// The x where document text / caret / selection start: the page column's left
    /// edge plus the writing inset [`Self::text_pad`]. The page SURFACE still spans
    /// from `column_left`, so this inset reads as an inner margin. Public so the
    /// capture sidecar can report the TRUE text origin (not the surface edge).
    pub fn text_left(&self) -> f32 {
        self.column_left() + self.text_pad()
    }

    /// The soft-wrap width available to TEXT: the page column width minus the inset
    /// on BOTH sides, so the right margin mirrors the left. This is THE buffer wrap
    /// width (the invariant `sync_wrap_width` enforces); every wrap-setter uses it.
    pub(super) fn text_wrap_width(&self) -> f32 {
        (self.column_width() - 2.0 * self.text_pad()).max(1.0)
    }

    /// WEB/LINUX MENU BAR reserve (px): the vertical strip the awl-rendered menu bar
    /// occupies at the canvas top while it is shown, else `0.0`. The document is inset
    /// below this (folded into [`Self::doc_top`] + the pipeline `hit_test` + the scroll
    /// viewport), so the caret / selection / hit-test all shift together. Gated on
    /// `crate::menubar::menu_bar_on()` — DEFAULT OFF on macOS (the capture/test
    /// platform), so this is `0.0` there and every default frame is byte-identical;
    /// `--menu-bar` / a web/Linux launch turns it on. Keyed off the LABEL-scaled line
    /// height, matching the slim bar the renderer draws. Public so the capture sidecar
    /// can report the TRUE text-origin top (`TEXT_TOP + this`) when the bar is shown.
    pub fn menubar_reserve(&self) -> f32 {
        if crate::menubar::menu_bar_on() {
            crate::menubar::bar_height(
                self.metrics.line_height * crate::markdown::type_scale::LABEL,
            )
        } else {
            0.0
        }
    }

    pub(super) fn doc_top(&self) -> f32 {
        TEXT_TOP + self.menubar_reserve() - self.rendered_scroll_top_px(self.scroll)
    }

    pub(super) fn row_top_px(&self, row: usize) -> f32 {
        self.row_geom.top_px(&self.buffer, &self.metrics, row)
    }

    pub(super) fn row_height_px(&self, row: usize) -> f32 {
        self.row_geom.height_px(&self.buffer, &self.metrics, row)
    }

    pub(super) fn total_doc_height(&self) -> f32 {
        self.row_geom.total_height(&self.buffer, &self.metrics)
    }

    pub fn max_scroll_rows(&self, height: f32) -> usize {
        let total = self.total_visual_rows();
        if total == 0 {
            return 0;
        }
        let avail = (height - TEXT_TOP - self.menubar_reserve()).max(0.0);
        if self.total_doc_height() <= avail {
            return 0;
        }
        total.saturating_sub(OVERSCROLL_KEEP_ROWS)
    }

    /// Real shaped-glyph X boundaries for a logical `line`, in pixels RELATIVE to
    /// the text's left edge (TEXT_LEFT not yet added). The returned vector has one
    /// entry per CHAR boundary: `xs[col]` is the left edge of the glyph cell at
    /// char-column `col`, and `xs[char_count]` is the right edge of the last glyph
    /// (end of line). So a line of N chars yields N+1 boundaries.
    ///
    /// This is the SINGLE SOURCE OF TRUTH for horizontal placement under advance-
    /// aware layout: it reads the actual advances cosmic-text produced (full-width
    /// for CJK, the mono advance for Latin), so caret / hit-test / selection all
    /// land on the real glyph cells for mixed CJK + Latin text.
    ///
    /// cosmic-text glyphs carry BYTE ranges (`start`/`end`) into the line text;
    /// awl columns are CHAR indices. We walk the line's chars and, for each, take
    /// the left x of the glyph cluster covering that char's byte. Multi-char
    /// clusters (rare here) share the cluster's span linearly. Empty / glyphless
    /// lines fall back to CHAR_WIDTH so an empty line still has a sane caret cell.
    pub(super) fn line_glyph_xs(&self, line: usize) -> Vec<f32> {
        let Some(line_text) = self.buffer.lines.get(line).map(|l| l.text().to_string()) else {
            return vec![0.0];
        };
        let mut clusters: Vec<(usize, usize, f32, f32)> = Vec::new();
        let mut x_offset = 0.0f32;
        for run in self.buffer.layout_runs() {
            if run.line_i != line {
                // Runs arrive in document order (non-decreasing `line_i`), so once
                // we pass the target line no later run can own it — stop instead of
                // walking the rest of the document's runs. Byte-identical: only
                // non-matching trailing runs are skipped (same as `cursor_glyph_key_at`).
                if run.line_i > line {
                    break;
                }
                continue;
            }
            let mut run_max_right = 0.0f32;
            for g in run.glyphs.iter() {
                let left = g.x + x_offset;
                let right = g.x + g.w + x_offset;
                clusters.push((g.start, g.end, left, right));
                run_max_right = run_max_right.max(right);
            }
            x_offset = run_max_right.max(x_offset);
        }
        assemble_glyph_xs(&line_text, &clusters, self.metrics.char_width)
    }

    /// The visual rows (wrapped sub-lines) of logical `line`, in top-to-bottom
    /// order. Each [`VisualRow`] carries the row's wrap-aware top y RELATIVE to
    /// the buffer top (add [`Self::doc_top`] for an absolute pixel y), the byte
    /// range of the original line it covers, and that row's own per-char x
    /// boundaries (relative to TEXT_LEFT) so an overlay can be placed on the
    /// correct row horizontally too. When `line` has no shaped runs (empty /
    /// glyphless line) a single synthetic row is returned at the line's uniform
    /// `line * line_height` top, so callers still get a sane row.
    pub(super) fn visual_rows(&self, line: usize) -> Vec<VisualRow> {
        // SINGLE-SLOT MEMO (see `rowgeom::RowGeom`): the caret geometry reads the
        // cursor line's wrap rows ~4× per redraw, and each rebuild walks every shaped
        // run of the document. The memo is cleared only at a shaped-geometry seam
        // (reshape/zoom/restyle), never on a cursor move, so a hit is always valid —
        // a motion keeps the same shaped runs. Calls 2–4 (and idle glide frames, where
        // the cursor line is unchanged) clone the cached rows instead of rebuilding.
        if let Some(cached) = self.row_geom.cached_rows(line) {
            return cached;
        }
        let line_text = self
            .buffer
            .lines
            .get(line)
            .map(|l| l.text().to_string())
            .unwrap_or_default();
        let mut rows: Vec<VisualRow> = Vec::new();
        for run in self.buffer.layout_runs() {
            if run.line_i != line {
                // Runs arrive in document order (non-decreasing `line_i`), so once
                // we pass the target line no later run can own it — stop instead of
                // walking the rest of the document's runs. Byte-identical: only
                // non-matching trailing runs are skipped (same as `cursor_glyph_key_at`).
                if run.line_i > line {
                    break;
                }
                continue;
            }
            rows.push(visual_row_from_run(
                &line_text,
                &run,
                self.metrics.char_width,
            ));
        }
        if rows.is_empty() {
            // Empty / glyphless logical line: synthesize one row at the uniform
            // top so the caret / selection sliver still renders sanely. This is
            // the only path that falls back to `line * line_height` and it matches
            // the pre-wrap behavior exactly for a blank line.
            rows.push(self.synthetic_visual_row(line, &line_text));
        }
        self.row_geom.store_rows(line, &rows);
        rows
    }

    /// The [`VisualRow`]s of EVERY logical line in `lines`, built in ONE
    /// `layout_runs()` walk — the batched twin of [`Self::visual_rows`] for the
    /// spell-squiggle / nit-underline proto rebuilds, which need the rows of MANY
    /// lines at once. Calling `visual_rows` per line re-walks every shaped run of
    /// the document each time (O(lines × doc)); this walks the runs once and
    /// assembles rows only for the requested lines (O(doc + requested rows)).
    ///
    /// Per line the rows are IDENTICAL to `visual_rows(line)` — the same
    /// [`visual_row_from_run`] assembly per shaped run, and the same synthetic
    /// uniform-top fallback row for an empty / glyphless / out-of-range line — so
    /// geometry derived from either source is byte-identical. Does NOT touch the
    /// single-slot cursor-line memo (so the caret path's warm memo survives).
    pub(super) fn visual_rows_for_lines(
        &self,
        lines: &std::collections::BTreeSet<usize>,
    ) -> std::collections::HashMap<usize, Vec<VisualRow>> {
        let mut out: std::collections::HashMap<usize, Vec<VisualRow>> =
            std::collections::HashMap::with_capacity(lines.len());
        let Some(&max_line) = lines.iter().next_back() else {
            return out;
        };
        let mut cur: Option<(usize, String)> = None;
        for run in self.buffer.layout_runs() {
            if run.line_i > max_line {
                break; // document order: nothing later can be a requested line
            }
            if !lines.contains(&run.line_i) {
                continue;
            }
            if cur.as_ref().map(|(li, _)| *li) != Some(run.line_i) {
                let text = self
                    .buffer
                    .lines
                    .get(run.line_i)
                    .map(|l| l.text().to_string())
                    .unwrap_or_default();
                cur = Some((run.line_i, text));
            }
            let line_text = &cur.as_ref().unwrap().1;
            out.entry(run.line_i).or_default().push(visual_row_from_run(
                line_text,
                &run,
                self.metrics.char_width,
            ));
        }
        for &line in lines {
            if out.contains_key(&line) {
                continue;
            }
            let line_text = self
                .buffer
                .lines
                .get(line)
                .map(|l| l.text().to_string())
                .unwrap_or_default();
            out.insert(line, vec![self.synthetic_visual_row(line, &line_text)]);
        }
        out
    }

    /// The synthetic single [`VisualRow`] for an EMPTY / glyphless logical line —
    /// the shared fallback of [`Self::visual_rows`] and
    /// [`Self::visual_rows_for_lines`], at the uniform `line * line_height` top
    /// (the only remaining use of that pre-wrap formula).
    fn synthetic_visual_row(&self, line: usize, line_text: &str) -> VisualRow {
        let char_count = line_text.chars().count();
        let xs = assemble_glyph_xs(line_text, &[], self.metrics.char_width);
        VisualRow {
            line_top: line as f32 * self.metrics.line_height,
            line_height: self.metrics.line_height,
            start_col: 0,
            end_col: char_count,
            xs,
        }
    }

    /// LOCAL wrap rows of logical `line` — the O(line) twin of [`Self::visual_rows`]
    /// for the visual-line MOTION oracle. It reads ONLY that line's already-shaped
    /// [`cosmic_text::BufferLine::layout_opt`] (its `Vec<LayoutLine>`), so it does NOT
    /// walk the whole document's `layout_runs()` the way `visual_rows` does — the fix
    /// for the O(doc)-per-keypress cost when a motion targets a line the single-slot
    /// row memo hasn't cached (the destination line ± 1 every arrow press).
    ///
    /// The returned rows carry the SAME per-char `xs` + `start_col`/`end_col` as
    /// `visual_rows` (built from the identical glyph clusters, so the oracle's
    /// `pick_row_index` / `col_in_row` land on the identical column), but the
    /// `line_top` / `line_height` are NOT the doc-absolute wrap tops — the motion
    /// oracle only needs the horizontal + column geometry, never the absolute y.
    /// Callers that need the absolute row top (caret / selection / ornament
    /// placement) MUST keep using `visual_rows`.
    ///
    /// Falls back to `visual_rows(line)` when the line is unshaped / has no layout
    /// (an empty or not-yet-laid line), so the synthetic-row edge case stays exactly
    /// as before.
    pub(super) fn line_rows_local(&self, line: usize) -> Vec<VisualRow> {
        let Some(bline) = self.buffer.lines.get(line) else {
            return self.visual_rows(line);
        };
        let Some(layout) = bline.layout_opt() else {
            return self.visual_rows(line);
        };
        if layout.is_empty() {
            return self.visual_rows(line);
        }
        let line_text = bline.text().to_string();
        let mut rows: Vec<VisualRow> = Vec::with_capacity(layout.len());
        for lline in layout.iter() {
            let mut clusters: Vec<(usize, usize, f32, f32)> = Vec::new();
            let mut byte_start = usize::MAX;
            let mut byte_end = 0usize;
            for g in lline.glyphs.iter() {
                clusters.push((g.start, g.end, g.x, g.x + g.w));
                byte_start = byte_start.min(g.start);
                byte_end = byte_end.max(g.end);
            }
            if byte_start == usize::MAX {
                byte_start = 0;
                byte_end = 0;
            }
            let xs = assemble_glyph_xs(&line_text, &clusters, self.metrics.char_width);
            let start_col = byte_col(&line_text, byte_start);
            let end_col = byte_col(&line_text, byte_end);
            rows.push(VisualRow {
                // The motion oracle ignores these two; use benign placeholders (the
                // uniform line height) rather than the absolute wrap top this path
                // deliberately does NOT compute.
                line_top: 0.0,
                line_height: self.metrics.line_height,
                start_col,
                end_col,
                xs,
            });
        }
        if rows.is_empty() {
            return self.visual_rows(line);
        }
        rows
    }

    /// TOTAL number of VISUAL ROWS in the whole document (every soft-wrapped
    /// continuation counts as its own row). This is the unit the scroll offset is
    /// measured in: a doc whose logical lines wrap has MORE visual rows than
    /// logical lines, and scrolling must reach the last one.
    ///
    /// Rows are NOT a uniform height (a heading row is taller), so this is simply
    /// the COUNT of shaped runs (one per visual row), read from the row-geometry
    /// table. Requires the whole document to be shaped (see [`Self::set_size`] /
    /// [`Self::full_shape_height`]); an unshaped tail would undercount. Falls back
    /// to the logical line count if nothing is shaped (degenerate empty buffer).
    pub fn total_visual_rows(&self) -> usize {
        self.row_geom.total_visual_rows(&self.buffer, &self.metrics)
    }

    pub fn visual_row_of(&self, line: usize, col: usize) -> usize {
        self.visual_row_of_aff(line, col, crate::caret::Affinity::Downstream)
    }

    /// [`Self::visual_row_of`] with a caret wrap `affinity` — used by the
    /// cursor-FOLLOW scroll so the viewport tracks the row the caret VISUALLY sits
    /// on (an `Upstream` caret rides the UPPER row). `Downstream` (search-match /
    /// zoom-anchor callers) is byte-identical to `visual_row_of`.
    pub fn visual_row_of_aff(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> usize {
        let rows = self.visual_rows(line);
        let target = pick_row_aff(&rows, col, affinity).line_top;
        self.row_geom.containing_row_q(
            &self.buffer,
            &self.metrics,
            (target * ScrollPos::SUBPX as f32).round() as i64,
        )
    }

    /// Wrap-aware visual-row top y (absolute, scroll-applied) for the position at
    /// (`line`, char `col`). Picks the wrapped run whose char span contains `col`;
    /// at/after end-of-line it uses the LAST run of the line. Empty / glyphless
    /// lines fall back to the synthetic row from [`Self::visual_rows`] (which is
    /// at the uniform `line * line_height` top), so a blank line keeps a sane
    /// caret row. This is THE replacement for `doc_top() + line * line_height` in
    /// every overlay, so caret / selection / squiggles ride the real wrapped row.
    pub(super) fn visual_row_top(&self, line: usize, col: usize) -> f32 {
        self.visual_row_top_aff(line, col, crate::caret::Affinity::Downstream)
    }

    /// [`Self::visual_row_top`] with a caret wrap `affinity` — the ONLY seam the
    /// caret's own row-placement uses, so an `Upstream` caret at a shared boundary
    /// rides the UPPER row's top. `Downstream` (every other caller: selection
    /// popover, etc.) is byte-identical to `visual_row_top`.
    pub(super) fn visual_row_top_aff(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> f32 {
        let rows = self.visual_rows(line);
        self.doc_top() + pick_row_aff(&rows, col, affinity).line_top
    }

    /// Pixel x (relative to TEXT_LEFT) of the glyph boundary at char-column `col`
    /// on logical `line`, plus the advance width of the glyph cell starting there
    /// (full-width for CJK, mono for Latin). At end-of-line the advance falls back
    /// to CHAR_WIDTH so the caret keeps a visible cell past the last glyph, and a
    /// DEGENERATE mid-line cell (see [`DEGENERATE_CELL_FRAC`]) falls back the same
    /// way so the caret stays visible on a collapsed wrap-boundary space.
    pub(super) fn col_x_and_advance(&self, line: usize, col: usize) -> (f32, f32) {
        self.col_x_and_advance_aff(line, col, crate::caret::Affinity::Downstream)
    }

    /// [`Self::col_x_and_advance`] with a caret wrap `affinity` — the seam the
    /// caret's own X/advance use, so an `Upstream` caret at a shared boundary reads
    /// the UPPER row's own left-aligned x's (its RIGHT edge) instead of the lower
    /// row's leading x (~0). `Downstream` is byte-identical to `col_x_and_advance`.
    pub(super) fn col_x_and_advance_aff(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> (f32, f32) {
        if let Some(x) = self.xray.iter().find(|x| x.line == line) {
            return xray_col_x(x, col, self.metrics.char_width);
        }
        let rows = self.visual_rows(line);
        let row = pick_row_aff(&rows, col, affinity);
        let n = row.xs.len().saturating_sub(1); // char count on the logical line
        let c = col.min(n);
        let x = row.xs[c];
        let advance = if c < n {
            let raw = row.xs[c + 1] - row.xs[c];
            if raw < self.metrics.char_width * DEGENERATE_CELL_FRAC {
                // DEGENERATE cell: a mid-line column with (near-)coincident x
                // boundaries — no visible glyph owns it. The canonical case is the
                // SPACE at a soft-wrap boundary: cosmic-text collapses the trailing
                // whitespace at the break, so both its boundaries sit on the row's
                // right edge and the raw width is ~0 — which used to draw the block
                // caret as a ~1px SLIVER there. Fall back to the same default cell
                // the end-of-line branch uses, so the caret on the collapsed wrap
                // space reads exactly like the caret past the last glyph. Real
                // narrow glyphs (`i`, `l`, thin spaces) sit well above the
                // threshold and keep their true advance.
                self.metrics.char_width
            } else {
                raw
            }
        } else {
            self.metrics.char_width
        };
        (x, advance)
    }

    pub(super) fn cursor_row_height(&self) -> f32 {
        let rows = self.visual_rows(self.cursor_line);
        pick_row(&rows, self.cursor_col).line_height
    }

    pub(super) fn cursor_scale(&self) -> f32 {
        self.caret_band_scale(self.cursor_line, self.cursor_row_height())
            .max(1.0)
    }

    /// THE ONE OWNER of "how tall is the caret-height BAND on line `li`, as a
    /// multiple of the base line height" — shared by the resting caret
    /// ([`Self::cursor_scale`]) AND the selection / squiggle / nit row-band
    /// builders ([`super::TextPipeline::row_band_for`]), so the highlight over a
    /// character is always the SAME height the caret would draw there.
    ///
    /// `1.0` on body text; the heading scale (`row_height / line_height`, e.g. 1.6)
    /// on a heading row so a heading's selection is as tall as its glyphs. IMAGE
    /// LINE (the caption model, WYSIWYG on): `1.0` — a BODY-height band, NOT the
    /// tall reserved row. The revealed source is body-size and the caret sizes to
    /// it ([`Self::cursor_row_height`]'s doc); a row-scaled band would balloon into
    /// a char-wide × whole-image-height PILLAR (the reported selection bug). The
    /// band's vertical CENTRING still uses the full (tall) `row_height` at the call
    /// site, exactly where cosmic-text centres the source glyphs, so the body-height
    /// band lands ON the caption — the same anchor the caret + caption scrim use.
    pub(super) fn caret_band_scale(&self, li: usize, row_height: f32) -> f32 {
        if crate::markdown::wysiwyg_on() && self.line_is_inline_image(li) {
            return 1.0;
        }
        // THE X-RAY table row: the caret (or an active selection) rides the
        // FLOATED body-size source, not the (possibly tall, wrapped-cell) grid
        // row — so the band sizes to the source line, exactly like the image
        // caption model above.
        if self.xray.iter().any(|x| x.line == li) {
            return 1.0;
        }
        let lh = self.metrics.line_height;
        if lh > 0.0 { row_height / lh } else { 1.0 }
    }

    /// Char column on a shaped run whose caret cell contains `target_x`, snapped
    /// to a grapheme-cluster boundary. A pointer inside a cell resolves to the
    /// nearer edge of it, the natural caret placement.
    ///
    /// The snap is not redundant with the assembled-cell lookup: a shaper's glyph clusters
    /// are NOT always UAX #29 clusters. Thai `ก` + `ำ` (U+0E33 SARA AM) is one
    /// cluster that every world's face shapes as two glyph groups — a click in
    /// the middle of it named the column BETWEEN the consonant and its vowel
    /// sign, a position that does not exist on screen. Devanagari conjuncts split
    /// the same way on faces without the ligature.
    pub(super) fn col_in_run(&self, run: &glyphon::cosmic_text::LayoutRun, target_x: f32) -> usize {
        let line = run.line_i;
        let row_top = run.line_top;
        let raw = self
            .row_geom
            .with_cached_rows(line, |rows| {
                Self::col_in_assembled_row(rows, row_top, target_x)
            })
            .unwrap_or_else(|| {
                let rows = self.visual_rows(line);
                Self::col_in_assembled_row(&rows, row_top, target_x)
            });
        Self::cluster_col(run, raw, target_x)
    }

    /// Resolve x against the already-assembled visual row that owns `row_top`.
    /// `row_top` comes from the same shaped run that produced the row, so equality
    /// selects the exact wrap row rather than repeating the y-band policy.
    fn col_in_assembled_row(rows: &[VisualRow], row_top: f32, target_x: f32) -> usize {
        rows.iter()
            .find(|row| row.line_top == row_top)
            .map(|row| Self::col_in_row(row, target_x))
            .unwrap_or(0)
    }

    /// `raw` — a column the assembled caret cells landed on — resolved against the INK of
    /// the MULTI-CHAR cluster the pointer sits in: its left half answers with the
    /// cluster's start, its right half with the end.
    ///
    /// It has to be the whole cluster's ink and not one glyph's, because a cluster
    /// shaped as SEVERAL glyphs makes the per-glyph walk both wrong and jumpy: on
    /// `a😀\u{200d}😀b` (one cluster, three glyphs, all stamped with the same byte
    /// span) sweeping the pointer rightward answered start, end, start, end, so
    /// clicking the right half of the sequence put the caret BEFORE it.
    ///
    /// A cluster of ONE char is left to the assembled-cell answer, so every ASCII,
    /// CJK, precomposed, and ligature column reads the same geometry as its caret.
    fn cluster_col(run: &glyphon::cosmic_text::LayoutRun, raw: usize, target_x: f32) -> usize {
        let line_text = run.text;
        // An all-ASCII row (most rows, in most documents) has no multi-char
        // cluster to resolve — a CR-LF pair cannot occur, since the rope is pure
        // `\n` and a run's text excludes it. Answered without allocating, because
        // this runs on every pointer MOVE (hover and drag), not only on a press.
        if line_text.is_ascii() {
            return raw;
        }
        let chars: Vec<char> = line_text.chars().collect();
        let len = chars.len();
        let at = |i: usize| chars[i];
        let (back, fwd) = (
            crate::grapheme::snap_backward(raw, len, at),
            crate::grapheme::snap_forward(raw, len, at),
        );
        // Interior: exactly one candidate, the cluster holding `raw`. On a boundary:
        // the pointer is in the cluster on one side of it — its own ink says which.
        let mut spans = [None, None];
        if back != fwd {
            spans[0] = Some((back, fwd));
        } else {
            if raw < len {
                spans[0] = Some((raw, crate::grapheme::next_cluster_boundary(raw, len, at)));
            }
            if raw > 0 {
                spans[1] = Some((crate::grapheme::prev_cluster_boundary(raw, at), raw));
            }
        }
        let byte_of = |col: usize| -> usize {
            line_text
                .char_indices()
                .nth(col)
                .map(|(b, _)| b)
                .unwrap_or(line_text.len())
        };
        for (start, end) in spans.into_iter().flatten() {
            if end - start < 2 {
                continue;
            }
            let (first, last) = (byte_of(start), byte_of(end));
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for g in run
                .glyphs
                .iter()
                .filter(|g| g.start >= first && g.end <= last)
            {
                lo = lo.min(g.x);
                hi = hi.max(g.x + g.w);
            }
            if lo >= hi || target_x < lo || target_x >= hi {
                continue;
            }
            // In an RTL run the cluster's logical START sits at the RIGHT of its ink.
            let past_middle = target_x >= (lo + hi) * 0.5;
            return if past_middle == run.rtl { start } else { end };
        }
        // Nothing measurable: `fwd` IS `raw` whenever the walk landed on a boundary,
        // and otherwise the cluster's end is a real position where `raw` is not.
        fwd
    }

    /// Char column on a visual row whose cell contains `target_x` (relative to
    /// TEXT_LEFT). Searches only this row's `[start_col, end_col]` and snaps a
    /// position past a glyph's midpoint to the next gap (natural caret placement).
    /// A position past the row's last glyph maps to the row's end column. This is a
    /// pure, GPU-free analogue of [`Self::col_in_run`] (which walks a real
    /// cosmic-text run); it lands the caret nearest a target x on a known row,
    /// shared by the unit tests AND the visual-line motion oracle (which uses it to
    /// place the caret under the sticky goal-x after stepping rows).
    pub(super) fn col_in_row(row: &VisualRow, target_x: f32) -> usize {
        let mut col = row.end_col; // default: past last glyph on this row
        for c in row.start_col..row.end_col {
            let left = row.xs[c];
            let right = row.xs[c + 1];
            let mid = (left + right) * 0.5;
            if target_x < mid {
                col = c;
                break;
            } else if target_x < right {
                col = c + 1;
                break;
            }
        }
        col
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The RESPONSIVE page column: `min(measure_px, window - 2*margin)`, centered, with
    // the margin collapsing from the generous `page_min_margin` to the small
    // `PAGE_MIN_PAD` as the measure crowds the width. These exercise the pure formula
    // (no GPU, no page globals) across the WIDE / NARROW / transition regimes.
    const CW: f32 = CHAR_WIDTH; // 14.4

    #[test]
    fn wide_window_seats_centered_column_at_measure() {
        let measure_px = 40.0 * CW; // 576
        let w = column_width_for(1200.0, CW, true, 40);
        let left = column_left_for(1200.0, CW, true, 40);
        assert!(
            (w - measure_px).abs() < 1e-3,
            "wide: column == measure, got {w}"
        );
        assert!(
            (left - (1200.0 - measure_px) * 0.5).abs() < 1e-3,
            "wide: centered, got {left}"
        );
        assert!(
            left > page_min_margin(1200.0) - 1e-3,
            "wide leftover >= generous margin"
        );
    }

    #[test]
    fn narrow_window_fills_minus_small_pad() {
        for &win in &[300.0_f32, 400.0, 700.0] {
            let w = column_width_for(win, CW, true, 80); // 80-char measure ~1152px >> win
            let left = column_left_for(win, CW, true, 80);
            assert!(
                (w - (win - 2.0 * PAGE_MIN_PAD)).abs() < 1e-3,
                "narrow {win}: fills minus pad, got {w}"
            );
            assert!(
                (left - PAGE_MIN_PAD).abs() < 1e-3,
                "narrow {win}: left at small pad, got {left}"
            );
            assert!(
                w + 2.0 * left <= win + 1e-3,
                "narrow {win}: never overflows"
            );
        }
    }

    #[test]
    fn column_is_monotonic_and_never_overflows_across_a_resize() {
        let measure_px = 80.0 * CW;
        let mut prev = 0.0_f32;
        let mut w = 200.0;
        while w <= 2600.0 {
            let col = column_width_for(w, CW, true, 80);
            let left = column_left_for(w, CW, true, 80);
            assert!(
                col >= prev - 1e-3,
                "column must not shrink as window grows (w={w})"
            );
            assert!(
                col <= measure_px + 1e-3,
                "column never exceeds the measure (w={w})"
            );
            assert!(
                left >= PAGE_MIN_PAD - 1e-3,
                "always at least the small pad (w={w})"
            );
            assert!(
                col + 2.0 * left <= w + 1e-2,
                "never overflows the window (w={w})"
            );
            prev = col;
            w += 50.0;
        }
        assert!((column_width_for(2600.0, CW, true, 80) - measure_px).abs() < 1e-3);
    }

    #[test]
    fn wide_capture_is_byte_identical_to_the_old_cap() {
        let measure_px = 40.0 * CW; // 576
        assert!((column_width_for(1200.0, CW, true, 40) - measure_px).abs() < 1e-3);
        assert!((column_left_for(1200.0, CW, true, 40) - (1200.0 - measure_px) * 0.5).abs() < 1e-3);
    }

    #[test]
    fn page_off_is_edge_to_edge_unaffected() {
        assert!((column_left_for(1200.0, CW, false, 80) - NONPAGE_INSET).abs() < 1e-3);
        assert!(
            (column_width_for(1200.0, CW, false, 80) - (1200.0 - 2.0 * NONPAGE_INSET)).abs() < 1e-3
        );
        assert!(std::hint::black_box(NONPAGE_INSET) > PAGE_MIN_PAD);
    }

    fn outline_pref_px() -> f32 {
        rowlayout::OUTLINE_PREFERRED_CHARS as f32 * CW * crate::markdown::type_scale::LABEL
    }
    fn outline_min_px() -> f32 {
        rowlayout::OUTLINE_MIN_CHARS as f32 * CW * crate::markdown::type_scale::LABEL
    }
    fn margin_gap() -> f32 {
        CW * crate::render::chrome::MARGIN_COLUMN_GAP_CHARS
    }
    const ADAPTIVE_LEFT_PAD: f32 = TEXT_LEFT;

    #[test]
    fn adaptive_wide_window_is_byte_identical_to_symmetric() {
        let left = adaptive_column_left(
            1200.0,
            CW,
            true,
            40,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        let symmetric = column_left_for(1200.0, CW, true, 40);
        assert_eq!(left, symmetric, "wide: adaptive placement changes nothing");
    }

    #[test]
    fn adaptive_outline_not_wanted_never_shifts_even_when_narrow() {
        let left = adaptive_column_left(
            900.0,
            CW,
            true,
            40,
            false,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        let symmetric = column_left_for(900.0, CW, true, 40);
        assert_eq!(left, symmetric);
    }

    #[test]
    fn adaptive_page_off_never_shifts() {
        let left = adaptive_column_left(
            900.0,
            CW,
            false,
            40,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert_eq!(left, NONPAGE_INSET);
    }

    #[test]
    fn adaptive_narrow_window_shifts_right_and_grants_the_full_preferred_rail() {
        let win = 900.0;
        let measure = 40usize;
        let symmetric = column_left_for(win, CW, true, measure);
        let width = column_width_for(win, CW, true, measure);
        let pref = outline_pref_px();
        let min = outline_min_px();
        let gap = margin_gap();
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            pref,
            min,
            gap,
            ADAPTIVE_LEFT_PAD,
        );
        assert!(
            left > symmetric,
            "narrow: column shifts right, got {left} vs symmetric {symmetric}"
        );
        let avail = (left - gap) - ADAPTIVE_LEFT_PAD;
        assert!(
            (avail - pref).abs() < 1.0,
            "narrow: outline granted its full preferred rail (within the whole-pixel snap), avail={avail} pref={pref}"
        );
        assert_eq!(
            left,
            (pref + gap + ADAPTIVE_LEFT_PAD).floor(),
            "narrow: the granted left is exactly the snapped desired_left"
        );
        let total_margin = win - width;
        let right_margin = total_margin - left;
        assert!(
            right_margin >= RIGHT_MARGIN_BREATH - 1e-3,
            "narrow: right margin keeps its breathing floor, got {right_margin}"
        );
    }

    #[test]
    fn adaptive_narrow_shift_caps_at_the_right_margin_breathing_floor() {
        let win = 800.0;
        let measure = 40usize;
        let width = column_width_for(win, CW, true, measure);
        let total_margin = win - width;
        let symmetric = column_left_for(win, CW, true, measure);
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert!(
            left > symmetric,
            "still shifts right from the symmetric position"
        );
        let right_margin = total_margin - left;
        assert!(
            (right_margin - RIGHT_MARGIN_BREATH).abs() < 0.5,
            "capped exactly at the breathing floor, got {right_margin}"
        );
        let avail = (left - margin_gap()) - ADAPTIVE_LEFT_PAD;
        assert!(
            avail < outline_pref_px() - 1.0,
            "granted rail is LESS than the full preference (capped by the floor), avail={avail}"
        );
        assert!(
            (avail / (CW * crate::markdown::type_scale::LABEL)).floor()
                >= rowlayout::OUTLINE_MIN_CHARS as f32,
            "but still comfortably above the hard hide floor"
        );
    }

    #[test]
    fn adaptive_narrowest_window_recenters_instead_of_overshooting_the_right_margin() {
        let win = 300.0;
        let measure = 80usize; // way more than fits at 300px
        let symmetric = column_left_for(win, CW, true, measure);
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert_eq!(
            left, symmetric,
            "narrowest: no shift possible, column re-centers exactly"
        );
    }

    #[test]
    fn adaptive_no_payoff_shift_recenters_instead_of_shifting_for_a_hidden_outline() {
        let win = 1100.0;
        let measure = 70usize;
        let symmetric = column_left_for(win, CW, true, measure);
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert_eq!(
            left, symmetric,
            "a shift that can't clear the outline's own minimum rail must not happen at all"
        );
        let width = column_width_for(win, CW, true, measure);
        let total_margin = win - width;
        let old_max_left = (total_margin - RIGHT_MARGIN_BREATH).max(0.0);
        assert!(
            old_max_left > symmetric,
            "fixture: the old formula would have shifted"
        );
        let old_avail = (old_max_left - margin_gap()) - ADAPTIVE_LEFT_PAD;
        let label_char_w = CW * crate::markdown::type_scale::LABEL;
        let old_avail_chars = (old_avail / label_char_w).floor().max(0.0) as usize;
        assert!(
            old_avail_chars < rowlayout::OUTLINE_MIN_CHARS,
            "fixture: the old shift would still leave the outline below its hide floor"
        );
    }

    #[test]
    fn adaptive_threshold_boundary_resolves_to_wide_not_narrow() {
        let pref = outline_pref_px();
        let min = outline_min_px();
        let gap = margin_gap();
        let desired_left = pref + gap + ADAPTIVE_LEFT_PAD;
        let measure = 40usize;
        let measure_px = measure as f32 * CW;
        let win = measure_px + 2.0 * desired_left;
        let symmetric = column_left_for(win, CW, true, measure);
        assert!(
            (symmetric - desired_left).abs() < 1.0,
            "fixture: symmetric lands at desired_left, got {symmetric} vs {desired_left}"
        );
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            pref,
            min,
            gap,
            ADAPTIVE_LEFT_PAD,
        );
        assert!(
            (left - symmetric.floor()).abs() < 1e-3,
            "boundary resolves to WIDE (no shift) at the exact threshold: left={left} symmetric={symmetric}"
        );
    }

    #[test]
    fn adaptive_never_shrinks_the_column_only_moves_where_it_sits() {
        for &(win, measure) in &[(1200.0_f32, 40usize), (900.0, 40), (800.0, 40), (300.0, 80)] {
            let width = column_width_for(win, CW, true, measure);
            let left = adaptive_column_left(
                win,
                CW,
                true,
                measure,
                true,
                outline_pref_px(),
                outline_min_px(),
                margin_gap(),
                ADAPTIVE_LEFT_PAD,
            );
            assert!(
                left + width <= win + 1e-2,
                "shifted column must still fit the window (win={win} measure={measure}): left={left} width={width}"
            );
        }
    }

    #[test]
    fn adaptive_entry_ramp_is_continuous_no_more_46px_jump() {
        let pref = outline_pref_px();
        let min = outline_min_px();
        let gap = margin_gap();
        let mut prev: Option<f32> = None;
        for w in 1090..=1170 {
            let left = adaptive_column_left(
                w as f32,
                CW,
                true,
                70,
                true,
                pref,
                min,
                gap,
                ADAPTIVE_LEFT_PAD,
            );
            if let Some(p) = prev {
                let step = left - p;
                assert!(
                    step >= -1e-3,
                    "width {w}px: column_left decreased ({p} -> {left})"
                );
                assert!(
                    step <= 20.0,
                    "width {w}px: column_left jumped {step}px in a single pixel of resize ({p} -> {left}) — the jitter bug"
                );
            }
            prev = Some(left);
        }
    }

    #[test]
    fn adaptive_ramp_still_recenters_well_outside_the_ramp_band() {
        let win = 1100.0;
        let measure = 70usize;
        let symmetric = column_left_for(win, CW, true, measure);
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert_eq!(
            left, symmetric,
            "well outside the ramp band: still a bare recenter, no partial shift"
        );
    }

    #[test]
    fn adaptive_left_snaps_to_whole_physical_pixels_across_a_1px_sweep() {
        let pref = outline_pref_px();
        let min = outline_min_px();
        let gap = margin_gap();
        for wants in [false, true] {
            let mut prev: Option<f32> = None;
            for w in 1000..=1400u32 {
                let left = adaptive_column_left(
                    w as f32,
                    CW,
                    true,
                    70,
                    wants,
                    pref,
                    min,
                    gap,
                    ADAPTIVE_LEFT_PAD,
                );
                assert_eq!(
                    left,
                    left.floor(),
                    "width {w} (wants={wants}): left must be a whole physical pixel, got {left}"
                );
                if let (Some(p), false) = (prev, wants) {
                    let step = left - p;
                    assert!(
                        step == 0.0 || step == 1.0,
                        "width {w}: symmetric-regime left must step exactly 0 or 1 whole px per width px, got {step}"
                    );
                }
                prev = Some(left);
            }
        }
    }

    #[test]
    fn page_column_advance_strips_zoom_keeps_dpi() {
        for &dpi in &[1.0_f32, 2.0] {
            let base = CW * dpi;
            for &zoom in &[0.5_f32, 1.0, 1.6, 2.5, 3.0] {
                let live = CW * zoom * dpi; // == metrics.char_width
                let adv = page_column_advance(live, zoom);
                assert!(
                    (adv - base).abs() < 1e-3,
                    "zoom={zoom} dpi={dpi}: advance must be zoom-free"
                );
            }
        }
        // Zoom 1.0 (the deterministic capture path) is an exact identity.
        assert!((page_column_advance(CW, 1.0) - CW).abs() < 1e-6);
    }

    #[test]
    fn zooming_in_keeps_column_and_margins_constant_gutter_stays() {
        let window = 1200.0;
        let measure = 40; // narrow measure -> generous, clearly-present margins
        let base_adv = page_column_advance(CW, 1.0);
        let ref_w = column_width_for(window, base_adv, true, measure);
        let ref_left = column_left_for(window, base_adv, true, measure);
        assert!(
            ref_left > PAGE_MIN_PAD + 1.0,
            "fixture must have a visible margin/gutter"
        );
        for &zoom in &[0.5_f32, 1.0, 1.6, 2.5, 3.0] {
            let live = CW * zoom; // metrics.char_width at this zoom (dpi 1.0)
            let adv = page_column_advance(live, zoom);
            let w = column_width_for(window, adv, true, measure);
            let left = column_left_for(window, adv, true, measure);
            assert!(
                (w - ref_w).abs() < 1e-3,
                "zoom={zoom}: column px must not change (got {w}, want {ref_w})"
            );
            assert!(
                (left - ref_left).abs() < 1e-3,
                "zoom={zoom}: left margin must not change"
            );
            let right = window - left - w;
            let ref_right = window - ref_left - ref_w;
            assert!(
                (right - ref_right).abs() < 1e-3,
                "zoom={zoom}: right margin must not change"
            );
        }
    }

    #[test]
    fn hover_zone_arms_only_within_grab_px_of_an_edge() {
        let measure_px = 40.0 * CW; // 576
        let left = (1200.0 - measure_px) * 0.5; // 312
        let tol = PAGE_RESIZE_GRAB_PX;
        assert_eq!(
            page_boundary_hit(left, left, measure_px, tol),
            Some(ResizeEdge::Left)
        );
        assert_eq!(
            page_boundary_hit(left + tol - 0.5, left, measure_px, tol),
            Some(ResizeEdge::Left)
        );
        assert_eq!(
            page_boundary_hit(left + tol + 2.0, left, measure_px, tol),
            None
        );
        let right = left + measure_px; // 888
        assert_eq!(
            page_boundary_hit(right - 1.0, left, measure_px, tol),
            Some(ResizeEdge::Right)
        );
        assert_eq!(page_boundary_hit(600.0, left, measure_px, tol), None);
    }

    #[test]
    fn resize_affordance_arms_at_both_drawn_edges_in_every_page_on_cell() {
        // THE LOCKOUT LAW (bug, 2026-07-15): in page mode the resize affordance must
        // arm at BOTH drawn column edges for every measure × window — ESPECIALLY the
        // collapsed cells (column pinned at the PAGE_MIN_PAD margins) where the old
        // `left <= PAGE_MIN_PAD + 1.0 → None` guard killed the affordance and locked the
        // user out of dragging a widened-past-capacity column back inward. Drives the
        // ONE arming owner `page_resize_edge_hit` against the DRAWN geometry
        // (`column_left_for`/`column_width_for`), so a reintroduced collapse-guard fails
        // here. Pure — no GPU, no page globals.
        let tol = PAGE_RESIZE_GRAB_PX;
        let adv = CW; // zoom-stripped page-column advance
        let mut saw_collapsed = false;
        for &measure in &[20usize, 40, 70, 100, 140] {
            for &window in &[600.0f32, 900.0, 1200.0, 2400.0] {
                let left = column_left_for(window, adv, true, measure);
                let width = column_width_for(window, adv, true, measure);
                let right = left + width;
                let cell = format!("measure={measure} window={window}");

                assert_eq!(
                    page_resize_edge_hit(true, left, width, left, tol),
                    Some(ResizeEdge::Left),
                    "{cell}: left edge must arm",
                );
                assert_eq!(
                    page_resize_edge_hit(true, left, width, right, tol),
                    Some(ResizeEdge::Right),
                    "{cell}: right edge must arm",
                );
                assert!(
                    page_resize_edge_hit(true, left, width, left + tol - 0.5, tol).is_some(),
                    "{cell}: just inside the left edge must arm",
                );
                assert!(
                    page_resize_edge_hit(true, left, width, right - (tol - 0.5), tol).is_some(),
                    "{cell}: just inside the right edge must arm",
                );

                assert_eq!(
                    page_resize_edge_hit(false, left, width, left, tol),
                    None,
                    "{cell}: page off must not arm (left)",
                );
                assert_eq!(
                    page_resize_edge_hit(false, left, width, right, tol),
                    None,
                    "{cell}: page off must not arm (right)",
                );

                if left <= PAGE_MIN_PAD + 1.0 {
                    saw_collapsed = true;
                    assert!(
                        page_resize_edge_hit(true, left, width, left, tol).is_some()
                            && page_resize_edge_hit(true, left, width, right, tol).is_some(),
                        "{cell}: COLLAPSED column must keep both edges grabbable (the lockout fix)",
                    );
                }
            }
        }
        assert!(
            saw_collapsed,
            "grid must include collapsed cells or it can't prove the lockout fix",
        );
    }

    #[test]
    fn in_writing_column_is_true_inside_and_on_both_edges_false_outside() {
        let measure_px = 40.0 * CW; // 576
        let left = (1200.0 - measure_px) * 0.5; // 312
        let right = left + measure_px; // 888
        assert!(
            in_writing_column(left, left, measure_px),
            "exactly on the left edge counts as inside"
        );
        assert!(
            in_writing_column(right, left, measure_px),
            "exactly on the right edge counts as inside"
        );
        assert!(
            in_writing_column(600.0, left, measure_px),
            "dead center is inside"
        );
        assert!(
            !in_writing_column(left - 1.0, left, measure_px),
            "just past the left margin is outside"
        );
        assert!(
            !in_writing_column(right + 1.0, left, measure_px),
            "just past the right margin is outside"
        );
    }

    #[test]
    fn image_handle_hit_arms_the_right_zone_per_edge_and_corner() {
        let rect = [100.0_f32, 50.0, 300.0, 200.0];
        let tol = IMAGE_RESIZE_GRAB_PX;
        assert_eq!(
            image_handle_hit((100.0, 50.0), rect, tol),
            Some(ImageHandle::TopLeft)
        );
        assert_eq!(
            image_handle_hit((400.0, 50.0), rect, tol),
            Some(ImageHandle::TopRight)
        );
        assert_eq!(
            image_handle_hit((100.0, 250.0), rect, tol),
            Some(ImageHandle::BottomLeft)
        );
        assert_eq!(
            image_handle_hit((400.0, 250.0), rect, tol),
            Some(ImageHandle::BottomRight)
        );
        assert_eq!(
            image_handle_hit((100.0, 150.0), rect, tol),
            Some(ImageHandle::Left)
        );
        assert_eq!(
            image_handle_hit((400.0, 150.0), rect, tol),
            Some(ImageHandle::Right)
        );
        assert_eq!(
            image_handle_hit((250.0, 50.0), rect, tol),
            Some(ImageHandle::Top)
        );
        assert_eq!(
            image_handle_hit((250.0, 250.0), rect, tol),
            Some(ImageHandle::Bottom)
        );
        assert_eq!(
            image_handle_hit((400.0 - tol + 1.0, 250.0 - tol + 1.0), rect, tol),
            Some(ImageHandle::BottomRight)
        );
        assert_eq!(image_handle_hit((250.0, 150.0), rect, tol), None, "center");
        assert_eq!(
            image_handle_hit((100.0, 50.0 - tol - 5.0), rect, tol),
            None,
            "above the top-left, off both"
        );
        assert_eq!(
            image_handle_hit((1000.0, 1000.0), rect, tol),
            None,
            "far outside"
        );
    }

    #[test]
    fn image_resize_width_drives_per_handle_clamped_to_min_and_wrap() {
        let rect = [100.0_f32, 50.0, 300.0, 200.0];
        let (wrap, min) = (500.0_f32, MIN_IMAGE_W);
        let w = |h: ImageHandle, p: (f32, f32)| image_resize_width(h, rect, p, wrap, min, 0.0);
        assert!((w(ImageHandle::Right, (350.0, 150.0)) - 250.0).abs() < 1e-3);
        assert!((w(ImageHandle::Left, (200.0, 150.0)) - 200.0).abs() < 1e-3);
        assert!((w(ImageHandle::Bottom, (250.0, 150.0)) - 150.0).abs() < 1e-3);
        assert!((w(ImageHandle::Top, (250.0, 150.0)) - 150.0).abs() < 1e-3);
        assert!((w(ImageHandle::BottomRight, (100.0 + 150.0, 50.0 + 100.0)) - 150.0).abs() < 1e-3);
        assert!((w(ImageHandle::BottomRight, (400.0, 250.0)) - 300.0).abs() < 1e-3);
        assert!((w(ImageHandle::TopLeft, (100.0, 50.0)) - 300.0).abs() < 1e-3);
        assert!((w(ImageHandle::TopRight, (400.0, 50.0)) - 300.0).abs() < 1e-3);
        assert!((w(ImageHandle::BottomLeft, (100.0, 250.0)) - 300.0).abs() < 1e-3);
        assert!(
            w(ImageHandle::TopLeft, (60.0, 20.0)) > 300.0,
            "TopLeft out widens"
        );
        assert!(
            w(ImageHandle::TopLeft, (250.0, 150.0)) < 300.0,
            "TopLeft toward center narrows"
        );
        // Clamps: dragging way out clamps to wrap; way in clamps up to the floor.
        assert!((w(ImageHandle::Right, (5000.0, 150.0)) - wrap).abs() < 1e-3);
        assert!((w(ImageHandle::Right, (100.0, 150.0)) - min).abs() < 1e-3);
        // A degenerate wrap below the floor never inverts the clamp band.
        assert!(
            (image_resize_width(ImageHandle::Right, rect, (350.0, 150.0), 10.0, min, 0.0) - min)
                .abs()
                < 1e-3
        );
    }

    /// The viewport-height half of the clamp: a drag can never grow an image
    /// taller than `max_h`, even when the wrap width would otherwise allow it.
    #[test]
    fn image_resize_width_caps_at_the_viewport_height_ceiling() {
        let rect = [100.0_f32, 50.0, 300.0, 200.0];
        let (wrap, min) = (800.0_f32, MIN_IMAGE_W);
        let max_h = 150.0_f32;
        let w = image_resize_width(ImageHandle::Right, rect, (5000.0, 150.0), wrap, min, max_h);
        assert!((w - 225.0).abs() < 1e-3, "capped to height ceiling: {w}");
        // A max_h of 0 (unknown window height) disables the height half entirely —
        // dragging way out clamps to `wrap` instead.
        let w2 = image_resize_width(ImageHandle::Right, rect, (5000.0, 150.0), wrap, min, 0.0);
        assert!((w2 - wrap).abs() < 1e-3, "max_h<=0 disables the cap: {w2}");
        let w3 = image_resize_width(ImageHandle::Right, rect, (100.0, 150.0), wrap, min, max_h);
        assert!(
            (w3 - min).abs() < 1e-3,
            "floor still wins under a tight height cap: {w3}"
        );
    }

    #[test]
    fn page_drag_measure_is_monotonic_across_the_rail_hide_boundary() {
        let window = 1800.0;
        let pref = outline_pref_px();
        let min = outline_min_px();
        let gap = margin_gap();

        let rendered_right = |m: usize| {
            adaptive_column_left(window, CW, true, m, true, pref, min, gap, ADAPTIVE_LEFT_PAD)
                + column_width_for(window, CW, true, m)
        };
        let cliffs = (crate::page::MIN_MEASURE + 1..=crate::page::MAX_MEASURE)
            .any(|m| rendered_right(m) < rendered_right(m - 1));
        assert!(
            cliffs,
            "fixture must span the rail-hide cliff or it can't reproduce the bug"
        );

        let start = 100usize;
        let anchor = adaptive_column_left(
            window,
            CW,
            true,
            start,
            true,
            pref,
            min,
            gap,
            ADAPTIVE_LEFT_PAD,
        );

        let mut prev = page_resize_measure_anchored(CW, 1700.0, anchor, ResizeEdge::Right);
        let first = prev;
        for px in 1700..=1799 {
            let m = page_resize_measure_anchored(CW, px as f32, anchor, ResizeEdge::Right);
            assert!(
                m >= prev,
                "rightward drag must never shrink the measure: at pointer {px} got {m} after {prev}",
            );
            prev = m;
        }
        assert!(
            prev > first,
            "the sweep must climb, not sit pinned (got {first}..{prev})"
        );
        let right_anchor = 2000.0;
        let mut lprev = page_resize_measure_anchored(CW, 1900.0, right_anchor, ResizeEdge::Left);
        for px in (1400..=1900).rev() {
            let m = page_resize_measure_anchored(CW, px as f32, right_anchor, ResizeEdge::Left);
            assert!(
                m >= lprev,
                "leftward drag of the left edge must never shrink the measure"
            );
            lprev = m;
        }
    }

    #[test]
    fn page_drag_maps_one_advance_to_one_measure_not_two() {
        let start = 40usize;
        let left_anchor = 100.0;
        let at_press = left_anchor + start as f32 * CW; // the rendered right edge for `start`
        assert_eq!(
            page_resize_measure_anchored(CW, at_press, left_anchor, ResizeEdge::Right),
            start,
            "pressing the rendered edge must not snap the measure",
        );
        assert_eq!(
            page_resize_measure_anchored(CW, at_press + CW, left_anchor, ResizeEdge::Right),
            start + 1,
            "one advance of pointer travel is exactly one char",
        );
        let right_anchor = 2000.0;
        let left_press = right_anchor - start as f32 * CW;
        assert_eq!(
            page_resize_measure_anchored(CW, left_press, right_anchor, ResizeEdge::Left),
            start,
        );
        assert_eq!(
            page_resize_measure_anchored(CW, left_press - CW, right_anchor, ResizeEdge::Left),
            start + 1,
            "the left edge tracks 1:1 too (widen by dragging further from the anchor)",
        );
    }

    #[test]
    fn page_drag_is_symmetric_and_zoom_independent() {
        for &zoom in &[0.5_f32, 1.0, 2.0] {
            let adv = page_column_advance(CW * zoom, zoom); // == CW at dpi 1.0
            let left_anchor = 100.0;
            let right_anchor = 2000.0;
            let dist = 40.0 * CW; // 40 chars of travel from the anchor
            let m_right = page_resize_measure_anchored(
                adv,
                left_anchor + dist,
                left_anchor,
                ResizeEdge::Right,
            );
            let m_left = page_resize_measure_anchored(
                adv,
                right_anchor - dist,
                right_anchor,
                ResizeEdge::Left,
            );
            assert_eq!(m_right, 40, "zoom={zoom}: 40 chars of travel -> 40 chars");
            assert_eq!(
                m_left, m_right,
                "zoom={zoom}: left/right mirror to the same measure"
            );
            let wider = page_resize_measure_anchored(
                adv,
                left_anchor + dist + 200.0,
                left_anchor,
                ResizeEdge::Right,
            );
            let narrower = page_resize_measure_anchored(
                adv,
                left_anchor + dist - 200.0,
                left_anchor,
                ResizeEdge::Right,
            );
            assert!(
                wider > m_right && narrower < m_right,
                "zoom={zoom}: out widens, in narrows"
            );
        }
    }

    #[test]
    fn page_drag_clamps_to_the_settable_band() {
        let anchor = 100.0;
        assert_eq!(
            page_resize_measure_anchored(CW, 100_000.0, anchor, ResizeEdge::Right),
            crate::page::MAX_MEASURE,
        );
        assert_eq!(
            page_resize_measure_anchored(CW, anchor, anchor, ResizeEdge::Right),
            crate::page::MIN_MEASURE,
        );
        assert_eq!(
            page_resize_measure_anchored(CW, anchor - 500.0, anchor, ResizeEdge::Right),
            crate::page::MIN_MEASURE,
        );
        assert_eq!(
            page_resize_measure_anchored(0.0, 100_000.0, anchor, ResizeEdge::Right),
            crate::page::MIN_MEASURE,
        );
    }

    #[test]
    fn narrow_window_still_collapses_edge_to_edge_at_any_zoom() {
        let window = 360.0; // 40-char measure ~576px >> window -> collapse
        for &zoom in &[0.5_f32, 1.0, 1.6, 3.0] {
            let adv = page_column_advance(CW * zoom, zoom);
            let w = column_width_for(window, adv, true, 40);
            let left = column_left_for(window, adv, true, 40);
            assert!(
                (w - (window - 2.0 * PAGE_MIN_PAD)).abs() < 1e-3,
                "zoom={zoom}: fills minus pad"
            );
            assert!(
                (left - PAGE_MIN_PAD).abs() < 1e-3,
                "zoom={zoom}: collapses to the small pad"
            );
        }
    }
}
