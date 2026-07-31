//! ITEM 116b — THE SUMMONED WORKSPACE'S TWO BOXES, AND THE RELOCATED DOCUMENT
//! VIEWPORT ONE OF THEM CAN BECOME.
//!
//! # Why this file exists
//!
//! awl has exactly ONE prose renderer — the document layer. A workspace whose
//! content region holds prose (DESIGN.md §5's "a timeline beside a comparison")
//! therefore cannot grow a second one: it has to move the one there is. That is
//! what [`TextPipeline::comparison_viewport`] is — the ONE owner of "where does
//! the document layer draw this frame", read by the four geometry owners every
//! document consumer already routes through ([`TextPipeline::column_left`],
//! [`TextPipeline::column_width`], `doc_top`, `doc_clip_band`). Nothing else in
//! the tree learns a second placement rule; ~45 existing call sites compose it
//! for free, exactly as item 84's content clip and the adaptive column already
//! do.
//!
//! # The bypass is module-private, and it is named
//!
//! Two ideas of "the writing column" now exist and they are NOT the same:
//!
//!   * the DOCUMENT's column — where prose is drawn, which relocates;
//!   * the PAGE's column on the canvas — the backdrop's own ground punch, its
//!     margin orientation surfaces and its draggable edges, which never do.
//!
//! [`TextPipeline::page_column_left`] / [`TextPipeline::page_column_width`] are
//! that second idea, `pub(super)`-scoped to `render::geometry`'s own module tree
//! and read by exactly one public seam, `TextPipeline::page_geometry`. Every
//! other consumer stays on `column_left`/`column_width` and follows the
//! document. A margin-orientation surface that must NOT follow is GATED off
//! instead of re-pointed (see `margin_orientation_yields`), because a surface
//! that answers "where am I in the document?" has nothing true to say while the
//! document on screen is a comparison of two other versions.
//!
//! # Two regions, one arithmetic
//!
//! [`WorkspaceRegions`] is the positional half of `workspace_geometry`, lifted
//! out so the row geometry and the comparison viewport read ONE derivation of
//! the card box, the primary column and the content pane. It is pure arithmetic
//! over already-measured inputs — no clones, no window resolution, no plan — so
//! `column_left()` can afford to ask it on every call the way it already affords
//! `adaptive_column_left`.

use super::*;
use super::workspace::{RAIL_GAP_CHARS, WORKSPACE_PAD};

/// The workspace's boxes for one frame: the card it fills, the PRIMARY (narrow)
/// column, the CONTENT pane beside it, and which of them this width draws.
///
/// Positional only, and deliberately blind to item 116a's `rows_are_primary`:
/// WHICH region carries the row-window is the caller's question, and asking it
/// here would put the same fact in two places.
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct WorkspaceRegions {
    /// `[x, y, w, h]` — the workspace surface itself.
    pub card: [f32; 4],
    /// `[x, w]` — the narrow navigation column (category labels, or a timeline).
    pub primary: [f32; 2],
    /// `[x, w]` — the wide content region (a settings list, or a comparison).
    pub pane: [f32; 2],
    /// Is there room for both regions at once? (`workspace_is_wide`.)
    pub wide: bool,
    /// Does the CONTENT region have focus this frame (`overlay_detail_focus`)?
    pub content_focused: bool,
}

impl WorkspaceRegions {
    /// Is the PRIMARY column on screen? Wide draws both; narrow stages, and the
    /// focus fact becomes which stage you are on (DESIGN.md §8).
    pub(in crate::render) fn primary_visible(&self) -> bool {
        self.wide || !self.content_focused
    }

    pub(in crate::render) fn content_visible(&self) -> bool {
        self.wide || self.content_focused
    }

    /// The y a region's own content begins at — below the workspace's search
    /// line and the query beat that follows it.
    ///
    /// Derived here rather than read off `OverlayRowPlan` because
    /// `comparison_viewport` is called from `column_left()`, which cannot afford
    /// to build a plan, and because a document viewport is not a ROW (item 174's
    /// arithmetic ban is about a candidate row's slot, which this is not). The
    /// two are held to agree by
    /// `render::tests::comparison_viewport_item116b::the_comparison_viewport_opens_on_the_same_line_the_rows_do`.
    fn content_top(&self, header_beat: f32) -> f32 {
        self.card[1] + WORKSPACE_PAD + header_beat
    }

    /// The y a region's own content ends at — the same bottom
    /// `workspace_rail_box` runs its column to.
    fn content_bottom(&self) -> f32 {
        self.card[1] + self.card[3] - WORKSPACE_PAD
    }
}

impl TextPipeline {
    /// THE TWO REGIONS' BOXES — the one derivation `workspace_geometry` (rows,
    /// rail, hit-test) and [`Self::comparison_viewport`] (the relocated
    /// document) both read, so the prose in the content pane and the rows in the
    /// primary column can never disagree about where either region is.
    pub(in crate::render) fn workspace_regions(&self, width: u32) -> WorkspaceRegions {
        let cw = self.overlay_char_width();
        let margin = self.workspace_margin();
        let hpad = self.overlay_text_hpad();
        let rail_w = self.workspace_rail_w;
        let gap = RAIL_GAP_CHARS * cw;

        let card_x = margin;
        let card_w = (width as f32 - 2.0 * margin).max(0.0);
        let card_y = margin + self.menubar_reserve();
        let card_h = (self.window_h - card_y - margin).max(self.overlay_lh());

        let interior = (card_w - 2.0 * hpad).max(0.0);
        let wide = self.workspace_is_wide(width);
        let (primary, pane) = match wide {
            true => (
                [card_x + hpad, rail_w],
                [
                    card_x + hpad + rail_w + gap,
                    (card_w - 2.0 * hpad - rail_w - gap).max(0.0),
                ],
            ),
            false => ([card_x + hpad, interior], [card_x + hpad, interior]),
        };

        WorkspaceRegions {
            card: [card_x, card_y, card_w, card_h],
            primary,
            pane,
            wide,
            content_focused: self.overlay_detail_focus,
        }
    }

    /// **THE ONE OWNER OF WHERE THE DOCUMENT LAYER DRAWS** (item 116b).
    ///
    /// `Some([x, y, w, h])` — the workspace CONTENT pane, when this frame's
    /// summoned workspace puts its own rows in the PRIMARY column
    /// (`WorkspaceShape::rows_are_primary`, item 116a's single fact) and that
    /// content region is on screen. `None` on every other frame, including every
    /// frame any `OverlayKind` can reach today: `workspace_shape` returns
    /// `Some(RailOverRows)` for Settings alone, whose rows live in the pane, and
    /// `None` for History until item 116d. So this is a structural relocation
    /// with **zero pixel change** — proven, not asserted, by the world × surface
    /// fingerprint matrix this item landed with.
    ///
    /// [`Self::column_left`], [`Self::column_width`], `doc_top` and
    /// `doc_clip_band` are the four readers; everything downstream of them —
    /// caret, selection, washes, wrap width, hit-test, the content clip — follows
    /// without knowing this exists.
    pub(in crate::render) fn comparison_viewport(&self) -> Option<[f32; 4]> {
        if !self.overlay_active || !self.overlay_rows_primary || !self.overlay_is_workspace() {
            return None;
        }
        let r = self.workspace_regions(self.window_w as u32);
        if !r.content_visible() {
            return None;
        }
        let top = r.content_top(self.workspace_header_beat());
        let h = r.content_bottom() - top;
        (r.pane[1] > 0.0 && h > 0.0).then_some([r.pane[0], top, r.pane[1], h])
    }

    /// The vertical run of the workspace's header — its `settings › query` search
    /// line plus the beat below it. One line, one owner, read by both regions.
    pub(in crate::render) fn workspace_header_beat(&self) -> f32 {
        self.overlay_lh() + self.overlay_header_gap()
    }

    /// **DOES A MARGIN-ORIENTATION SURFACE YIELD THIS FRAME?** The persistent
    /// chrome DESIGN.md §5 bounds to answering "where am I?" — the outline, the
    /// bottom-left gutter, the corner readouts, the page frame and the draggable
    /// page edges — all compose off the four relocated owners, and none of them
    /// has anything true to say while the document on screen is a read-only
    /// comparison of two versions the user is not editing. So they yield, exactly
    /// as item 34 already yields them to a summoned overlay.
    ///
    /// The outline and the gutter reach this conclusion through their own
    /// `overlay_active` gate, which strictly SUBSUMES this one (a comparison
    /// viewport requires `overlay_active`); the law
    /// `every_margin_orientation_surface_yields_to_a_relocated_document` proves
    /// that rather than trusting it, over the whole roster.
    pub(in crate::render) fn margin_orientation_yields(&self) -> bool {
        self.comparison_viewport().is_some()
    }
}
