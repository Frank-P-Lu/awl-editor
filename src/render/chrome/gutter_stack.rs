//! THE MARGIN WORKING SET'S ROWS — how a list of open files becomes the text,
//! the ink and the one soft plate the bottom identity draws when more than one
//! file is open under the active project root.
//!
//! It lives beside [`super::gutter`] rather than inside it because the two own
//! different questions. `gutter` owns the BLOCK: which lines exist, where the
//! block sits, what it reports. This owns a ROW: how a file's location and name
//! share one line's width, which of the two ends yields when that width runs
//! out, and what marks the row you are actually reading.
//!
//! Nothing here decides WHETHER a stack is drawn — that is
//! [`crate::workingset::WorkingSet::stack_rows`]'s single answer, delivered as
//! an empty list. Every function here is a no-op on an empty list, which is what
//! keeps the single-file margin on exactly the path it had before this surface
//! existed.

use super::*;

/// Horizontal breath either side of a plated row's ink, in LABEL rows. The plate
/// hugs the ink rather than filling the margin's full width: the labels are
/// right-aligned against the writing column, so a band spanning `[0, avail]`
/// would be a wide slab whose left half marks nothing, and DESIGN §5 asks this
/// surface to hug the column exactly as the outline does.
pub(super) const PLATE_PAD_X: Rows = Rows(0.35);

/// THE PLATE'S HEIGHT, in LABEL rows, centred on the row it marks. Short of a
/// full row on purpose: adjacent plates must not meet, or a two-file stack reads
/// as one tall block with a seam rather than as two rows.
pub(super) const PLATE_HEIGHT_ROWS: Rows = Rows(0.86);

/// The plate's corner radius in device px — the same soft radius the overlay's
/// living selection band wears, so a selected row reads as the same kind of
/// object in the margin as it does in a picker.
pub(super) const PLATE_CORNER_PX: Physical = Physical(2.5);

/// **559:** the close-zone hover plate's own alpha, over the SAME
/// [`theme::surface_selected`] fill the active-row plate uses — a fraction of
/// it, not a second color, so the hover square reads as a quieter member of
/// the same family rather than a competing accent (DESIGN's "one accent"
/// still names the caret alone). Distinct from the active-row plate's own
/// opaque fill so a reader never mistakes "the pointer is here" for "this is
/// the open file".
pub(super) const CLOSE_HOVER_PLATE_ALPHA: u8 = 0x80;

/// THE CLOSE ZONE'S own UPPER BOUND, in LABEL rows — a square target sized to
/// the row it belongs to, so the thing the pointer aims at is the size of the
/// line it belongs to rather than a width invented for it. An UPPER bound,
/// not the zone's literal width: [`close_zone`] also holds the zone to the
/// mark's OWN drawn lane (`mark_w`), so a face whose row height reaches
/// further than two characters of its own real pitch never draws (or
/// accepts clicks over) a square wider than the × it marks — the 559 hover
/// plate reuses this exact rect, and a zone wider than the mark's own lane
/// would be a highlight sitting under the label's first letter.
///
/// Anchored on the row's own shaped INK, never a fixed x: the mark is a
/// LEADING span in a right-aligned line, so it sits wherever that row's own
/// name happens to end on the left — a short name leaves a wide ragged
/// margin the ink never reaches, a long one pushes the mark close to the
/// stack's own leading edge. [`close_zone`] derives that anchor exactly the
/// way [`plate_rect`] already derives the plate's own edge — from the row's
/// shaped width, never a position invented independently of it — so a name's
/// length can never make the click target and the drawn ink disagree.
pub(super) const CLOSE_ZONE_ROWS: Rows = Rows(1.0);

/// The pre-shaped close lane. EVERY row of the stack shapes this exact run
/// before anything else on the line, even a `More`/`Overflow` row that can
/// never reveal it and even while it is transparent: a LEADING span in a
/// right-aligned line grows the row's own shaped width into the ragged
/// margin a shorter-than-budget name already leaves empty, so revealing it
/// changes ink only — it never asks the label for room, unlike the trailing
/// shape this superseded. `pub(super)` because the single-file identity line
/// ([`super::gutter`]) shapes and draws the exact same lane through the
/// exact same text, rather than a second close mark of its own.
///
/// ONE space, not two: the reveal shows exactly `×` followed by a single
/// breath before the label, so the resting (unrevealed but still-reserved)
/// overhang left of a right-aligned name is no wider than the glyph the
/// hover state actually draws there. A second space was pure reserve for a
/// mark that only ever occupies one — the user's own measurement against a
/// sibling row's char width (~3.17 char-widths of left overhang against
/// well under one on the right) named it directly.
pub(super) const CLOSE_MARK_TEXT: &str = "× ";

/// WHAT A POINTER AT `px` OVER A ROW IS AIMING AT.
///
/// The row is ONE band with two meanings, not two controls: the close zone
/// hugs the LEADING edge of the row's own shaped ink and everything after it
/// — the whole rest of the band, which is most of it — stays the switch
/// target. The asymmetry is the design decision. Switching is the frequent,
/// forgiving act and gets the large area; closing is rare and destructive and
/// gets a small, deliberate one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowIntent {
    Switch,
    Close,
}

/// The close zone `[x, y, w, h]` inside a row's own planner band, anchored on
/// the row's own shaped ink width `text_w` — chars × the LABEL char width,
/// counting the leading mark's own characters, the SAME quantity
/// [`plate_rect`]'s own caller measures a plate from — rather than a fixed x.
/// A right-aligned row's ink starts wherever its own name happens to end on
/// the left, so a target pinned to the band's own edge instead would sit in
/// empty margin on a short name and inside the label on a long one —
/// [`plate_rect`]'s own rationale, mirrored here for the pointer rather than
/// the fill.
///
/// `mark_w` is the mark's OWN drawn lane width (`CLOSE_MARK_TEXT.chars().count()
/// as f32 * label_char_w`, the caller's exact quantity), and it caps the zone
/// alongside the row-height square [`CLOSE_ZONE_ROWS`] names: without this
/// second bound, a mono face whose row height reaches further than the
/// mark's own two characters (a narrow-aspect face — [`CLOSE_ZONE_ROWS`]'s
/// own doc) would offer a full row-height square that reaches past the ×
/// into the label's own first glyph. The 559 hover plate draws exactly this
/// rect, so a zone wider than the mark's lane would be a highlight sitting
/// under text that is not the mark.
///
/// Clamped to the band on both sides: a maximal-width name can push
/// `text_w` past the band's own width (the mark yields off the canvas edge
/// there, `fit_rows`' own doc), and the zone still lands on the row's
/// leading edge rather than escaping the band to the left.
pub(super) fn close_zone(row_rect: [f32; 4], text_w: f32, mark_w: f32) -> [f32; 4] {
    let [x, y, w, h] = row_rect;
    let ink_left = (x + w - text_w).max(x);
    let available = (x + w - ink_left).max(0.0);
    let zone = (h * CLOSE_ZONE_ROWS.0).min(mark_w.max(0.0)).min(available);
    [ink_left, y, zone, h]
}

/// Classify a pointer x against a row's band. The one owner both the hit-test
/// and any future drawn affordance read, so what the pointer accepts and what
/// the reader is shown cannot disagree.
pub(super) fn row_intent(row_rect: [f32; 4], text_w: f32, mark_w: f32, px: f32) -> RowIntent {
    let [zx, _, zw, _] = close_zone(row_rect, text_w, mark_w);
    if px >= zx && px <= zx + zw {
        RowIntent::Close
    } else {
        RowIntent::Switch
    }
}

/// ONE FITTED ROW: the exact text drawn, where its quieter location half ends,
/// and whether it is the active file.
pub(super) struct StackLine {
    /// `[parent][leaf]` — the whole row, already inside the line's budget.
    pub text: String,
    /// Byte offset where the quieter parent ends and the leaf begins. `0` for a
    /// file sitting directly under the root, which draws no location at all.
    pub parent_byte: usize,
    pub active: bool,
    pub kind: crate::workingset::StackRowKind,
}

/// Fit each row to `budget` characters, THE LEAF FIRST.
///
/// The order is the decision. A row's job is to name a file; its location is
/// context. Fitting the location first and giving the leaf what survives would
/// let a deep path eat the filename — the reader would learn where a file they
/// can no longer identify lives. So the leaf takes the budget it needs through
/// the same one elision door every other margin label uses, and the parent is
/// offered what is left through [`crate::workingset::fit_parent`], which returns
/// nothing rather than a misleading fragment.
///
/// `budget` is the row's FULL per-line budget, spent entirely on the label —
/// the close mark is a LEADING span shaped separately, on top of whatever
/// this returns (`stack_spans`' own doc), never docked out of it. A name
/// long enough to spend the whole budget now right-aligns flush to the
/// stack's own edge; the mark grows the shaped line into the ragged margin a
/// shorter name already leaves empty instead of a lane held out of every
/// row's own width.
pub(super) fn fit_rows(rows: &[crate::workingset::StackRow], budget: usize) -> Vec<StackLine> {
    rows.iter()
        .map(|row| {
            let leaf = rowlayout::fit_primary(&row.leaf, budget);
            let left = budget.saturating_sub(leaf.chars().count());
            let parent = if row.parent.is_empty() {
                String::new()
            } else {
                crate::workingset::fit_parent(&row.parent, left).unwrap_or_default()
            };
            StackLine {
                parent_byte: parent.len(),
                text: format!("{parent}{leaf}"),
                active: row.active,
                kind: row.kind,
            }
        })
        .collect()
}

/// The stack's rich-text spans in draw order, each carrying the ink it wears.
///
/// ONE AXIS OF VALUE: the ACTIVE row's name comes forward, whether that row
/// is a file or a project heading — [`plate_rects`] plates a heading exactly
/// when it is the reader's current project, the same `active` field a File
/// row's own plate reads — and every other row's name is `faint`. A row's
/// LOCATION is `faint` throughout, quieter than the name it qualifies on the
/// row that matters.
///
/// An active row's name is drawn ON [`plate_rect`]'s own fill
/// (`theme::surface_selected()`), not on the bare margin — so the ladder's
/// plain `muted` default is never the answer here, active row or heading
/// alike. It is routed through [`theme::selected_row_secondary_ink`], the
/// SAME ink-legibility mechanism the picker row's own secondary ink and the
/// toast rim already use (`render/chrome/overlay_rows.rs`,
/// `overlay_visual_sel.rs`): the function keeps `muted` wherever it already
/// contrasts against the plate (every ordinary world) and falls back to
/// whichever of the page's two poles reads better only where it does not —
/// which is Wagtail, where the plate fills at page-inverse (`base_content`)
/// and `muted` is the SAME page-inverse value, so unrouted ink vanishes into
/// its own plate (the sidecar-vs-pixels tripwire: `selected_index` reads
/// correctly while the row renders unreadable).
///
/// Rows are joined by carrying a leading newline on the first span of every row
/// after the first, so an absent location cannot swallow a line break.
///
/// Bails before touching the theme at all when `lines` is empty — the
/// single-file margin's own no-op path, and it keeps that path from becoming
/// an unguarded reader of the process-global active world (`crate::testlock`)
/// for a frame that will never draw a plate to begin with.
pub(super) fn stack_spans(
    lines: &[StackLine],
    hover: Option<super::gutter_hit::GutterStackHit>,
) -> Vec<(String, glyphon::Color)> {
    if lines.is_empty() {
        return Vec::new();
    }
    let active_ink = theme::selected_row_secondary_ink(theme::surface_selected()).to_glyphon();
    let faint = theme::faint().to_glyphon();
    let mut out = Vec::with_capacity(lines.len() * 2);
    for (row, line) in lines.iter().enumerate() {
        let lead = if row == 0 { "" } else { "\n" };
        let name_ink = if line.active { active_ink } else { faint };
        // The mark's text is ALWAYS shaped FIRST for EVERY row kind, even
        // when its alpha is zero: a LEADING span in a right-aligned line
        // grows the row's shaped width into the ragged margin a
        // shorter-than-budget name already leaves empty, so revealing it
        // only ever changes ink, never the label's own advances — and it
        // carries the row-separating newline, since it is now the first
        // span every row pushes rather than whichever of parent/leaf
        // happened to be first under the old trailing order. `hover` can
        // only ever name a `File` or `Group` row (`stack_hit_from_plan`'s
        // own enrolment), so a `More`/`Overflow` row's mark is shaped but
        // permanently transparent — it grows the ragged margin without ever
        // being able to reveal.
        let shown = hover.filter(|hit| hit.row == row).map(|_| {
            if line.active {
                theme::selected_row_secondary_ink(theme::surface_selected()).to_glyphon()
            } else {
                theme::muted().to_glyphon()
            }
        });
        out.push((
            format!("{lead}{CLOSE_MARK_TEXT}"),
            shown.unwrap_or_else(|| glyphon::Color::rgba(0, 0, 0, 0)),
        ));
        let (parent, leaf) = line.text.split_at(line.parent_byte);
        if !parent.is_empty() {
            out.push((parent.to_string(), faint));
        }
        out.push((leaf.to_string(), name_ink));
    }
    out
}

/// THE ACTIVE ROW'S PLATE `[x, y, w, h]`, or nothing when no row is plated.
///
/// `row_rect` is the active row's own band as the block planner laid it
/// (`[x, y, w, h]`, `w` the full right-aligned box), so the plate cannot drift
/// from the line it marks: it is derived from that rect, never re-measured from
/// the canvas. The ink is right-aligned inside the box, so the plate ends where
/// the box does and begins a pad short of where the text starts.
/// The RIGHT edge is the invariant, and it is fixed at one pad past the box on
/// every row — the same convention [`TextPipeline::gutter_frost_seeds`] already
/// uses for this block's halos, so the two treatments hug the writing column
/// identically instead of each hugging it their own way. Only the LEFT edge
/// yields, at the canvas edge, when a label is as wide as the margin can hold; a
/// plate that clamped its right edge instead would pull off the column exactly
/// on the longest names, which is where it is most needed.
pub(super) fn plate_rect(row_rect: [f32; 4], text_w: f32, pad_x: f32) -> [f32; 4] {
    let [x, y, w, h] = row_rect;
    let ink = text_w.min(w);
    let plate_h = h * PLATE_HEIGHT_ROWS.0;
    let right = x + w + pad_x;
    let left = (x + w - ink - pad_x).max(0.0);
    [left, y + (h - plate_h) * 0.5, right - left, plate_h]
}

/// THE ROW-DRAG INSERTION HAIRLINE's own thickness, device px, unscaled by
/// DPI — a crisp line at any zoom/density, the same convention
/// [`super::FLOAT_BORDER_RING_PX`] uses for the float-panel border.
pub(super) const DRAG_INDICATOR_THICKNESS_PX: Physical = Physical(2.0);

/// **THE ROW-DRAG'S OWN INSERTION-SLOT RECT** — a thin band spanning a row's
/// full width, straddling the boundary ABOVE the drawn file-row `file_row`
/// (`file_row == 0` sits above the FIRST file row; `file_row ==
/// layout.files.len()` sits below the LAST). `None` while no stack is drawn
/// at all (nothing to straddle).
///
/// `file_row` is in the SAME index space [`super::gutter_hit::GutterStackHit::row`]
/// uses (0-based over the drawn FILE rows only), which is NOT
/// `plan.rows`'s own index space (the whole block's lines, `changed`/
/// `project` included) — [`plate_rects`] bridges the same two spaces for the
/// active-row plate; this mirrors it rather than re-deriving the offset a
/// second way.
pub(super) fn drag_indicator_rect(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    file_row: usize,
    thickness_px: f32,
) -> Option<[f32; 4]> {
    if layout.files.is_empty() {
        return None;
    }
    let offset = layout.lines().len() - layout.files.len();
    if file_row == 0 {
        let &[x, y, w, _] = plan.rows.get(offset)?;
        return Some([x, y - thickness_px * 0.5, w, thickness_px]);
    }
    let above = offset + (file_row - 1).min(layout.files.len() - 1);
    let &[x, y, w, h] = plan.rows.get(above)?;
    Some([x, y + h - thickness_px * 0.5, w, thickness_px])
}

/// **515: THE PLATE MEANS THE ACTIVE FILE, AND NOTHING ELSE.** At most ONE
/// plate per frame, across the resting stack and the expanded panel alike —
/// never a Group heading, even the current project's own, and never the
/// single-file identity line's projection.
///
/// A heading that IS the current project keeps its distinct ink
/// ([`stack_spans`] still routes it through
/// [`theme::selected_row_secondary_ink`]) but draws no fill: the project
/// identity is stated once, by the gutter's own folder heading above the
/// block (or, once the panel draws headings itself, by that ink-marked
/// heading) — plating it too would state "you are in this project" a second
/// time in the same column the active file's own plate already occupies,
/// which is the exact double-selection a screenshot once caught (two purple
/// plates answering two different questions, "which file" and "which
/// project", read together as two selections).
///
/// Read off the SAME [`GutterLayout::lines`] list the glyphs are laid from and
/// the SAME planner rows they sit on, so a plate cannot mark a different line
/// than the one the reader is editing. Adding a line to the block (an affordance
/// appearing, the project line vanishing) moves the glyphs and the plate through
/// one shared index rather than two agreeing counts.
pub(super) fn plate_rects(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    label_char_w: f32,
    pad_x: f32,
) -> Vec<[f32; 4]> {
    layout
        .lines()
        .into_iter()
        .enumerate()
        .filter_map(|(row, (text, kind))| {
            let gutter::GutterLine::File(at) = kind else {
                return None;
            };
            let file = layout.files.get(at)?;
            if !file.active || !matches!(file.kind, crate::workingset::StackRowKind::File) {
                return None;
            }
            let rect = *plan.rows.get(row)?;
            // The shaped line BEGINS with the always-present close run: even
            // while transparent it participates in right alignment, growing
            // the row's own shaped width leftward, so the plate's measured
            // run includes it (a revealed × must never draw routed ink
            // outside its own fill — fatal in a one-bit world, black on
            // black, the same tripwire this file already names for the
            // label itself). Only a File line ever reaches here (the filter
            // above), so the lane is always present.
            let ink_w =
                (text.chars().count() + CLOSE_MARK_TEXT.chars().count()) as f32 * label_char_w;
            Some(plate_rect(rect, ink_w, pad_x))
        })
        .collect()
}

/// **559: THE CLOSE-ZONE HOVER PLATE** — a soft square behind the × under the
/// LIVE pointer, drawn ONLY while the pointer sits inside that row's own
/// close zone ([`RowIntent::Close`]). Exactly [`close_zone`]'s own rect: the
/// zone the hit-test accepts and the plate the reader sees are the SAME
/// geometry (this function's whole reason to exist, over hand-placing a
/// second square), so a click can never land somewhere the highlight did not
/// promise — the "drawn-vs-accepted drift" `close_zone`'s own doc names.
///
/// Unlike [`plate_rects`], this is not limited to the ACTIVE row: any row's
/// (or the single-file identity's) close zone earns the hover plate, since
/// the affordance answers "what will clicking here do", not "which file is
/// open" — the active-row question [`plate_rects`] alone answers. `None`
/// off every other state: resting, hovering the row's SWITCH half, or no
/// stack/identity drawn at all.
pub(super) fn close_hover_plate_rect(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    label_char_w: f32,
    hover: Option<super::gutter_hit::GutterStackHit>,
) -> Option<[f32; 4]> {
    let hit = hover.filter(|h| h.is_close())?;
    let mark_chars = CLOSE_MARK_TEXT.chars().count();
    layout
        .lines()
        .into_iter()
        .enumerate()
        .find_map(|(line, (text, kind))| {
            let row = match kind {
                gutter::GutterLine::File(at) => at,
                // The single-file identity draws the same lone slot `hit.row ==
                // 0` names (`gutter_hit::stack_hit_from_plan`'s own doc).
                gutter::GutterLine::Name => 0,
                gutter::GutterLine::Project | gutter::GutterLine::Changed => return None,
            };
            if row != hit.row {
                return None;
            }
            let rect = *plan.rows.get(line)?;
            let ink_w = (text.chars().count() + mark_chars) as f32 * label_char_w;
            let mark_w = mark_chars as f32 * label_char_w;
            Some(close_zone(rect, ink_w, mark_w))
        })
}

impl TextPipeline {
    /// **559:** set + upload the close-zone hover plate, off the exact rect
    /// [`close_hover_plate_rect`] hands the hit-test. Lives here (not inline
    /// in `gutter.rs::prepare_gutter`, the shared block owner) so a new draw
    /// does not grow that function past its own ceiling; recomputes
    /// `label_char_w`/the corner radius itself, matching how
    /// `gutter_hit::gutter_hit_plan` already re-derives the same quantity
    /// from `self.metrics` rather than threading it through every caller.
    pub(super) fn prepare_close_hover_plate(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        layout: &GutterLayout,
        plan: &crate::render::plan::GutterStackPlan,
    ) {
        let label_char_w = self.metrics.char_width * crate::markdown::type_scale::LABEL;
        let rect = close_hover_plate_rect(layout, plan, label_char_w, self.gutter_stack_hover);
        let fill = theme::surface_selected();
        self.gutter_close_hover_plate.set_color(
            theme::Srgb::rgba(fill.r, fill.g, fill.b, CLOSE_HOVER_PLATE_ALPHA).rgba_bytes(),
        );
        self.gutter_close_hover_plate
            .set_corner(self.metrics.px_physical(PLATE_CORNER_PX));
        self.gutter_close_hover_plate.prepare(
            device,
            queue,
            width,
            height,
            &rect.into_iter().collect::<Vec<_>>(),
        );
    }
}

/// The active-row plate rects AND the row-drag insertion-hairline rect (at
/// most one each), both derived from the SAME planner rows — so a drag
/// indicator can never draw off a row the plate math disagrees about.
/// `drag_row` is `None` outside a live drag, which is the whole reason the
/// indicator half comes back empty.
pub(super) fn plates_and_drag_indicator(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    label_char_w: f32,
    pad_x: f32,
    indicator_thickness_px: f32,
    drag_row: Option<usize>,
) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let plates = plate_rects(layout, plan, label_char_w, pad_x);
    let indicator = drag_row
        .and_then(|row| drag_indicator_rect(layout, plan, row, indicator_thickness_px))
        .into_iter()
        .collect();
    (plates, indicator)
}

#[cfg(test)]
mod tests;
