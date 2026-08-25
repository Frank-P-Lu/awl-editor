//! THE SUMMONED WORKSPACE'S TWO BOXES, AND THE RELOCATED DOCUMENT
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
//! the tree learns a second placement rule; every existing call site composes it
//! for free, exactly as the content clip and adaptive column already do.
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
//! that second idea. They stay crate-render-private and their call sites are
//! ENUMERATED by
//! `render::tests::comparison_viewport`'s
//! `the_unrelocated_page_column_has_exactly_the_named_consumers`
//! — the two definitions, the one public seam `TextPipeline::page_geometry`,
//! and the page-resize hit-test, which reads the canvas edges in order to
//! decide it must not arm. Every other consumer stays on
//! `column_left`/`column_width` and follows the document. A margin-orientation
//! surface that must NOT follow is GATED off instead of re-pointed (see
//! `margin_orientation_yields`), because a surface that answers "where am I in
//! the document?" has nothing true to say while the document on screen is a
//! comparison of two other versions.
//!
//! # Two regions, one arithmetic
//!
//! [`WorkspaceRegions`] is the positional half of `workspace_geometry`, lifted
//! out so the row geometry and the comparison viewport read ONE derivation of
//! the card box, the primary column and the content pane. It is pure arithmetic
//! over already-measured inputs — no clones, no window resolution, no plan — so
//! `column_left()` can afford to ask it on every call the way it already affords
//! `adaptive_column_left`.

use super::workspace::RAIL_GAP_CHARS;
use super::*;
pub(in crate::render) use crate::render::plan::WorkspaceRegions;

impl TextPipeline {
    /// THE TWO REGIONS' BOXES — the one derivation `workspace_geometry` (rows,
    /// rail, hit-test) and [`Self::comparison_viewport`] (the relocated
    /// document) both read, so the prose in the content pane and the rows in the
    /// primary column can never disagree about where either region is.
    pub(in crate::render) fn workspace_regions(&self, width: u32) -> WorkspaceRegions {
        let cw = self.overlay_char_width();
        let margin = self.workspace_margin();
        let hpad = self.overlay_text_hpad();
        let rail_w = self.workspace_primary_w;
        let gap = RAIL_GAP_CHARS.0 * cw;

        let wide = self.workspace_is_wide(width);
        crate::render::plan::plan_workspace_regions(crate::render::plan::WorkspaceRegionsInput {
            canvas_w: width as f32,
            canvas_h: self.window_h,
            margin,
            top_reserve: self.menubar_reserve(),
            min_height: self.overlay_lh(),
            hpad,
            primary_w: rail_w,
            gap,
            wide,
            content_focused: self.overlay_detail_focus,
        })
    }

    /// **THE ONE OWNER OF WHERE THE DOCUMENT LAYER DRAWS**.
    ///
    /// `Some([x, y, w, h])` — the workspace CONTENT pane, when this frame's
    /// summoned workspace puts its own rows in the PRIMARY column
    /// (`WorkspaceShape::rows_are_primary`, the shape's single fact), that content
    /// region is on screen, and there is READ-ONLY COMPARISON PROSE to put in it
    /// (`overlay_comparison`). `None` on every other frame — including every frame
    /// of Settings, whose rows live in the pane.
    ///
    /// The payload gate is not belt-and-braces. The timeline shape can
    /// be up with nothing to compare (an empty history; a query that filters every
    /// version away), and on those frames the pushed text is the user's OWN
    /// document. Relocating it into the comparison's place would put the live
    /// document up as a third readable layer inside the workspace — three
    /// competing readable layers, which is what this whole composition removes.
    ///
    /// [`Self::column_left`], [`Self::column_width`], `doc_top` and
    /// `doc_clip_band` are the four readers; everything downstream of them —
    /// caret, selection, washes, wrap width, hit-test, the content clip — follows
    /// without knowing this exists.
    pub(in crate::render) fn comparison_viewport(&self) -> Option<[f32; 4]> {
        if !self.overlay_active
            || !self.overlay_rows_primary
            || !self.overlay_comparison
            || !self.overlay_is_workspace()
        {
            return None;
        }
        let frame = self.workspace_frame(self.window_w as u32);
        let header_band = crate::render::plan::header_band_height(
            frame.fit.header_rows,
            self.overlay_lh(),
            frame.fit.header_gap,
        );
        crate::render::plan::plan_comparison_viewport(
            frame.regions,
            true,
            header_band,
            frame.fit.pad,
        )
    }

    /// **IS THE PUSHED TEXT A TRANSCRIPT RATHER THAN THE USER'S DOCUMENT?**
    ///
    /// A timeline workspace with a resolved payload substitutes the comparison's
    /// prose for the document's own text — the substitution is a view, and the
    /// buffer is never touched. This says the substitution is in force, which is
    /// NOT the same question as [`Self::comparison_viewport`]: the region can be
    /// off screen while the substitution stands, which is exactly the narrow
    /// stage with the timeline focused.
    pub(in crate::render) fn document_is_a_transcript(&self) -> bool {
        self.overlay_active
            && self.overlay_comparison
            && self.overlay_rows_primary
            && self.overlay_is_workspace()
    }

    /// **IS THE TRANSCRIPT PARKED THIS FRAME?** The substitution is in force but
    /// its region is not on screen, so there is nowhere the transcript belongs.
    ///
    /// It must then not be drawn AT ALL — not at the page column it would
    /// otherwise fall back to, and not into the offscreen capture the blur frosts.
    /// The narrow stage is where this bites: with the timeline focused, a drawn
    /// transcript reads as a ghost of a comparison the user is not looking at,
    /// and on a world whose surface is bare plates rather than a filled card it is
    /// the most prominent thing on screen.
    pub(in crate::render) fn transcript_parked(&self) -> bool {
        self.document_is_a_transcript() && self.comparison_viewport().is_none()
    }

    /// **THE DOCUMENT LAYER'S GLYPH CLIP.** Every text renderer the document
    /// layer owns uploads its `TextBounds` through this one door.
    ///
    /// Off a comparison it returns the caller's own bounds UNCHANGED, so every
    /// ordinary frame in the tree is byte-identical by construction. While the
    /// document is relocated it returns those bounds INTERSECTED with the region —
    /// which is what makes containment structural rather than incidental. It matters
    /// only once the comparison is composited: while the card was drawn OVER the
    /// document a glyph that escaped the region was hidden by the surface anyway.
    /// The content is now drawn AFTER the card, and an escaping glyph would land on
    /// the workspace's own face.
    ///
    /// The quads have their own owner — `content_clip` / `clip_rects_to_band` —
    /// and this is its glyph twin: the two resolve the same region from the same
    /// [`Self::comparison_viewport`].
    pub(in crate::render) fn clip_text_bounds(&self, bounds: TextBounds) -> TextBounds {
        let Some([x, y, w, h]) = self.comparison_viewport() else {
            return bounds;
        };
        TextBounds {
            left: bounds.left.max(x as i32),
            top: bounds.top.max(y as i32),
            right: bounds.right.min((x + w).ceil() as i32),
            bottom: bounds.bottom.min((y + h).ceil() as i32),
        }
    }

    /// **DOES A MARGIN-ORIENTATION SURFACE YIELD THIS FRAME?** The persistent
    /// chrome DESIGN.md §5 bounds to answering "where am I?" — the outline, the
    /// bottom-left gutter, the corner readouts, the page frame and the draggable
    /// page edges — all compose off the four relocated owners, and none of them
    /// has anything true to say while the document on screen is a read-only
    /// comparison of two versions the user is not editing. So they yield, exactly
    /// as they already yield to a summoned overlay.
    ///
    /// The question is whether the document LAYER is a transcript, not whether its
    /// region is on screen — those differ on the narrow stage, and a margin
    /// surface has just as little to say about a transcript nobody can see. Read
    /// against the region alone, the outline listed the TRANSCRIPT's headings in
    /// the frame beside a workspace showing no comparison at all.
    ///
    /// The outline and the gutter also reach this conclusion through their own
    /// `overlay_active` gate, which strictly SUBSUMES this one; the law
    /// `every_margin_orientation_surface_yields_to_a_relocated_document` proves
    /// that rather than trusting it, over the whole roster.
    pub(in crate::render) fn margin_orientation_yields(&self) -> bool {
        self.document_is_a_transcript()
    }

    /// WHAT THE BOTTOM-RIGHT "how much?" READOUT SAYS THIS FRAME — one owner,
    /// so the draw path and the margin-orientation law read the same
    /// sentence. Empty parks the label off-screen.
    ///
    /// Word count and reading time describe the user's OWN document.
    /// While the document layer is relocated into a read-only comparison
    /// ([`TextPipeline::margin_orientation_yields`]) that number would be about
    /// a transcript nobody is writing, so it yields with the rest of the margin
    /// family and returns the moment the workspace closes.
    pub(in crate::render) fn wordcount_readout_text(&self) -> String {
        match self.margin_orientation_yields() {
            true => String::new(),
            false => self.wordcount_text(),
        }
    }

    /// WHAT THE CALM NOTICE SAYS THIS FRAME — the [`Self::wordcount_readout_text`]
    /// twin. The notice is seated on the writing column's own bottom centre, so
    /// it travels with the document layer; read against a relocated comparison
    /// it would look like a message ABOUT that comparison.
    pub(in crate::render) fn notice_readout_text(&self) -> String {
        match self.margin_orientation_yields() {
            true => String::new(),
            false => self.notice.clone(),
        }
    }
}
