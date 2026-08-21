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

/// The close-mark treatments carried only for the affordance taste gallery.
///
/// Production stays [`Off`]: the user has already decided the stable RIGHT
/// zone and hover-only lifetime, but not whether the mark reveals in one or two
/// stages or whether the active row participates. Keeping those answers behind
/// one explicit prototype switch lets the real live App render comparable
/// frames without quietly turning one candidate into shipped behaviour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ClosePrototype {
    Off,
    OneStageAll,
    OneStageSiblings,
    TwoStageAll,
    TwoStageSiblings,
}

impl ClosePrototype {
    fn includes_active(self) -> bool {
        matches!(self, Self::OneStageAll | Self::TwoStageAll)
    }

    fn two_stage(self) -> bool {
        matches!(self, Self::TwoStageAll | Self::TwoStageSiblings)
    }
}

/// Parse without reading process state so the complete prototype roster can be
/// law-tested without mutating the environment.
pub(super) fn parse_close_prototype(value: Option<&str>) -> ClosePrototype {
    match value {
        Some("one-all") => ClosePrototype::OneStageAll,
        Some("one-siblings") => ClosePrototype::OneStageSiblings,
        Some("two-all") => ClosePrototype::TwoStageAll,
        Some("two-siblings") => ClosePrototype::TwoStageSiblings,
        _ => ClosePrototype::Off,
    }
}

pub(super) fn close_prototype() -> ClosePrototype {
    parse_close_prototype(std::env::var("AWL_GUTTER_CLOSE_PROTOTYPE").ok().as_deref())
}

/// The folder-line treatments carried only for the same taste gallery.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FolderPrototype {
    Legacy,
    QuietBelow,
    HeadingAbove,
}

pub(super) fn parse_folder_prototype(value: Option<&str>) -> FolderPrototype {
    match value {
        Some("quiet-below") => FolderPrototype::QuietBelow,
        Some("heading-above") => FolderPrototype::HeadingAbove,
        _ => FolderPrototype::Legacy,
    }
}

pub(super) fn folder_prototype() -> FolderPrototype {
    parse_folder_prototype(std::env::var("AWL_GUTTER_FOLDER_PROTOTYPE").ok().as_deref())
}

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

/// THE CLOSE ZONE'S WIDTH, in LABEL rows — a square target at the row's right
/// edge, so the thing the pointer aims at is the size of the line it belongs to
/// rather than a width invented for it.
///
/// The RIGHT edge is where it has to be. Every row's ink is right-aligned
/// against the writing column, so the right edge is the one x every row of the
/// stack shares no matter how long its name is; a left-edge target would sit in
/// empty margin on a short name and over the text on a long one.
pub(super) const CLOSE_ZONE_ROWS: Rows = Rows(1.0);

/// WHAT A POINTER AT `px` OVER A ROW IS AIMING AT.
///
/// The row is ONE band with two meanings, not two controls: the close zone is a
/// square at the right edge and everything left of it — the whole rest of the
/// band, which is most of it — stays the switch target. The asymmetry is the
/// design decision. Switching is the frequent, forgiving act and gets the large
/// area; closing is rare and destructive and gets a small, deliberate one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowIntent {
    Switch,
    Close,
}

/// The close zone `[x, y, w, h]` inside a row's own planner band, clamped to the
/// band when the margin is narrower than one square.
///
/// Derived from the row rect rather than re-measured, for the same reason
/// [`plate_rect`] is: a target that computed its own geometry could drift from
/// the row it belongs to, and a close target that has drifted closes the wrong
/// file.
pub(super) fn close_zone(row_rect: [f32; 4]) -> [f32; 4] {
    let [x, y, w, h] = row_rect;
    let zone = (h * CLOSE_ZONE_ROWS.0).min(w.max(0.0));
    [x + w - zone, y, zone, h]
}

/// Classify a pointer x against a row's band. The one owner both the hit-test
/// and any future drawn affordance read, so what the pointer accepts and what
/// the reader is shown cannot disagree.
pub(super) fn row_intent(row_rect: [f32; 4], px: f32) -> RowIntent {
    let [zx, _, zw, _] = close_zone(row_rect);
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
            }
        })
        .collect()
}

/// The stack's rich-text spans in draw order, each carrying the ink it wears.
///
/// TWO AXES OF VALUE, both drawn from the block's existing two-step ladder
/// rather than a new ink: the ACTIVE row's name wants `muted` — the same ink
/// the single-file identity has always spent on the filename — and every other
/// row's is `faint`, so the reader's current file is the one that comes
/// forward. A row's LOCATION is `faint` throughout, quieter than the name it
/// qualifies on the row that matters.
///
/// The active row's name is drawn ON [`plate_rect`]'s own fill
/// (`theme::surface_selected()`), not on the bare margin — so `muted` alone is
/// only ever a DEFAULT, never the final answer. It is routed through
/// [`theme::selected_row_secondary_ink`], the SAME ink-legibility mechanism the
/// picker row's own secondary ink and the toast rim already use
/// (`render/chrome/overlay_rows.rs`, `overlay_visual_sel.rs`): the function
/// keeps `muted` wherever it already contrasts against the plate (every
/// ordinary world, byte-identical to before) and falls back to whichever of
/// the page's two poles reads better only where it does not — which is
/// Wagtail, where the plate fills at page-inverse (`base_content`) and `muted`
/// is the SAME page-inverse value, so the unrouted ink used to vanish into its
/// own plate (the sidecar-vs-pixels tripwire: `selected_index` reads correctly
/// while the row renders unreadable).
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
    prototype: ClosePrototype,
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
        let (parent, leaf) = line.text.split_at(line.parent_byte);
        if parent.is_empty() {
            out.push((format!("{lead}{leaf}"), name_ink));
        } else {
            out.push((format!("{lead}{parent}"), faint));
            out.push((leaf.to_string(), name_ink));
        }
        // The mark's text is ALWAYS present under a prototype, even when its
        // alpha is zero. That invisible shaped run is the reserved trailing
        // lane: revealing the × changes only ink, never the label's advances,
        // so hover cannot make the filename jump. With the prototype OFF the
        // run does not exist at all, leaving production and the one-file path
        // byte-identical.
        if prototype != ClosePrototype::Off {
            let shown = hover.filter(|hit| hit.row == row).and_then(|hit| {
                if line.active && !prototype.includes_active() {
                    return None;
                }
                if prototype.two_stage() && !hit.is_close() {
                    Some(faint)
                } else if line.active {
                    Some(theme::selected_row_secondary_ink(theme::surface_selected()).to_glyphon())
                } else {
                    Some(theme::muted().to_glyphon())
                }
            });
            out.push((
                "  ×".to_string(),
                shown.unwrap_or_else(|| glyphon::Color::rgba(0, 0, 0, 0)),
            ));
        }
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

/// THE PLATED ROWS of a block — at most one, and none at all when there is no
/// stack.
///
/// Read off the SAME [`GutterLayout::lines`] list the glyphs are laid from and
/// the SAME planner rows they sit on, so the plate cannot mark a different line
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
        .find_map(|(row, (text, kind))| {
            let gutter::GutterLine::File(at) = kind else {
                return None;
            };
            if !layout.files.get(at)?.active {
                return None;
            }
            let rect = *plan.rows.get(row)?;
            let ink_w = text.chars().count() as f32 * label_char_w;
            Some(plate_rect(rect, ink_w, pad_x))
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests;
