use crate::render::ImageHandle;
use winit::window::CursorIcon;

/// Hover inputs derived from the app's shared hit-test geometry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CursorContext {
    pub dragging_edge: bool,
    pub dragging_text: bool,
    /// A summoned overlay (palette / picker / the spell-suggest panel) is
    /// open — its scrim covers the document, so the pointer is never "over
    /// text" while this is set, regardless of where it geometrically sits.
    pub overlay_open: bool,
    pub over_edge: bool,
    pub over_text: bool,
    pub over_clickable_overlay_row: bool,
    pub over_clickable_lens: bool,
    /// The pointer is over a CELL of the summoned INSERT-TABLE dimension
    /// picker's drawn grid — a click-to-pick affordance, the same signal
    /// class as a clickable overlay row. Computed from the grid's OWN
    /// hit-test (`TextPipeline::table_dims_cell_at`), the identical one the
    /// click path and the hover-preview path both read, so a hovered cell
    /// can never disagree with a clickable one.
    pub over_table_dims_cell: bool,
    pub over_query_input: bool,
    pub over_outline_row: bool,
    /// The pointer is over a WORKING-SET STACK ROW in the bottom-left margin
    /// identity block — a clickable click-to-switch (or click-to-close) target,
    /// same affordance class as a margin-outline row. Computed from the stack's
    /// OWN hit-test (`TextPipeline::gutter_stack_hit`), so a hovered row can
    /// never disagree with a clickable one.
    pub over_stack_row: bool,
    /// The pointer is over a CLICKABLE menu-bar TITLE or an open-dropdown ITEM (the
    /// awl-rendered WEB/LINUX menu bar — NOT an overlay). A title/item you can click to
    /// act earns the pointing hand, exactly like a picker row. Computed from the bar's
    /// OWN hit-test (`TextPipeline::menubar_hand_at`).
    pub over_menu_hand: bool,
    /// The pointer is over the menu bar's own strip OR an open dropdown's card, but NOT
    /// on a clickable title/item — dead chrome space, which reads as the plain ARROW
    /// (never the document I-beam beneath the bar). Ranked ABOVE `over_edge`/`over_text`
    /// (the bar covers them). Computed from `TextPipeline::over_menu_surface`.
    pub over_menu_bar: bool,
    pub over_case_toggle: bool,
    /// The pointer is over a clickable FIND or REPLACE FIELD CELL of the summoned
    /// find/replace panel — a text field, computed from the SAME `panel_hit` the
    /// press path and `over_case_toggle` use, so it can never disagree with where a
    /// click would land. Ranked with the case-toggle/menu-bar arms, above the
    /// document edge/text it covers.
    pub over_panel_field: bool,
    /// The pointer is over the find/replace panel's DEAD chrome (padding,
    /// inter-cell gaps) — not a field, not the case toggle. Ranked with
    /// `over_panel_field`/`over_case_toggle`, so the plain arrow wins there
    /// instead of the document's own affordance bleeding through the floating card.
    pub over_panel: bool,
    pub image_drag: Option<ImageHandle>,
    /// The pointer is hovering (not yet dragging) one of an inline image's resize
    /// EDGES/CORNERS — `Some(handle)` names which, whose glyph ([`image_handle_icon`])
    /// reads as the resize affordance (↔ for a side, ↕ for top/bottom, ⤡/⤢ for a
    /// corner), exactly like a page-column edge. Computed from the SAME images layout
    /// the `ImageQuadPipeline` draws (`TextPipeline::image_handle_at`), never a parallel
    /// geometry. Ranked with the page edge (below an open overlay's scrim, which covers
    /// the images). `None` when the pointer is over no image border.
    pub image_hover: Option<ImageHandle>,
    /// The pointer is over a clickable BUTTON of the summoned FORMAT POPOVER (the
    /// reveal-on-select format toolbar — NOT an overlay). A button you can click to
    /// apply a format earns the pointing hand, exactly like a picker row. Computed
    /// from the popover's OWN hit-test (`TextPipeline::popover_hit`); only ever set
    /// while the popover is up (and it never coexists with an open overlay/search).
    pub over_popover_button: bool,
    /// The pointer is over a REVEALED FOLD CHEVRON — a heading's own
    /// left-margin fold-toggle target, expanded or collapsed alike. A clickable
    /// affordance signal like an outline row, computed from the SAME hit-test the
    /// click handler uses (`TextPipeline::fold_chevron_hit`), so a hovered chevron
    /// can never disagree with a clickable one. Only ever set while no overlay is
    /// open (an overlay's scrim covers the document).
    pub over_fold_chevron: bool,
}

/// The OS cursor glyph for a given inline-image resize HANDLE: a horizontal
/// ↔ for the left/right edges, a vertical ↕ for the top/bottom edges, and a
/// diagonal ⤡ (`NwseResize`, "\") / ⤢ (`NeswResize`, "/") for the corners along
/// each diagonal. THE single owner of the handle→glyph mapping — a no-wildcard
/// `match`, so a new [`ImageHandle`] variant fails to compile until it is mapped
/// here (the same exhaustive-sweep discipline as `cursor_icon_for` itself).
pub fn image_handle_icon(handle: ImageHandle) -> CursorIcon {
    match handle {
        ImageHandle::Left | ImageHandle::Right => CursorIcon::ColResize,
        ImageHandle::Top | ImageHandle::Bottom => CursorIcon::RowResize,
        ImageHandle::TopLeft | ImageHandle::BottomRight => CursorIcon::NwseResize,
        ImageHandle::TopRight | ImageHandle::BottomLeft => CursorIcon::NeswResize,
    }
}

/// THE priority decision: hover context -> OS cursor icon. Pure, so it is
/// exhaustively unit-testable without a window. Priority, highest first:
/// 1. an ACTIVE page-edge drag always wins — the resize glyph tracks the gesture
///    the user is literally performing, regardless of anything else;
/// 2. an ACTIVE image drag-resize wins next — the grabbed edge/corner's own glyph
///    ([`image_handle_icon`]: ↔ side, ↕ top/bottom, ⤡/⤢ corner) tracks that gesture
///    (the two active drags are mutually exclusive; the page-edge drag is arbitrarily
///    ordered first);
///    2b. an ACTIVE text-SELECTION drag wins next — the I-beam is pinned for the
///    whole gesture (mutually exclusive with the other two active drags: a
///    page-edge or image drag can't start while a text selection is being
///    dragged, and vice versa), so it never flickers to the arrow/hand and
///    back if the pointer strays outside the exact writing-column bounds
///    mid-drag;
/// 3. hovering a clickable menu-bar TITLE / dropdown ITEM gets the pointing HAND —
///    the awl-rendered web/Linux menu bar's clickable-affordance signal, ranked with
///    the other hands (the menu + a summoned overlay are mutually exclusive, so the
///    relative order among the hands never matters, only that a clickable menu
///    surface earns the hand);
///    3b. hovering ANY clickable overlay ROW, a clickable LENS-STRIP facet, *or*
///    a CELL of the INSERT-TABLE dimension picker's grid gets the pointing HAND —
///    the clickable-affordance signal, sitting ABOVE the generic overlay→arrow
///    rule (but still under an in-progress resize drag); the three never
///    geometrically overlap (a TableDims card carries no rows/strip, and vice
///    versa), so which one is set never matters, only that any is;
/// 4. hovering the overlay's editable QUERY-INPUT line gets the I-beam — it is
///    a text field, ranked above the generic overlay→arrow but below a row;
/// 5. any other part of a summoned overlay wins next — its scrim visually
///    covers everything beneath it, the page edge + images included → the plain arrow;
///    5b. dead menu-bar space (the bar strip / an open dropdown's card, off any clickable
///    title/item) → the plain arrow, ranked ABOVE the page edge + text it covers, so the
///    bar reads as chrome not the document beneath it;
///    5c. hovering the find/replace panel's `Aa` CASE-TOGGLE cell gets the pointing HAND —
///    a clickable-affordance signal like a picker row, ranked ABOVE the page edge + text
///    the floating panel covers (the panel is not an overlay, so this is one of three
///    arms that surface the panel's own affordances);
///    5d. hovering one of the panel's FIND/REPLACE FIELD CELLS gets the I-beam — a text
///    field, ranked with the case toggle (mutually exclusive with it: `panel_hit` names
///    exactly one cell per point);
///    5e. hovering the panel's own DEAD chrome (padding, inter-cell gaps) gets the plain
///    arrow, ranked with the other two panel arms, so the document's I-beam/edge never
///    bleeds through the floating card where it isn't a field or the toggle;
/// 6. hovering a page-column edge (not yet dragging) still beats plain text;
/// 7. hovering an inline image's resize EDGE/CORNER gets that handle's glyph — a
///    resize affordance like the page edge, ranked just under it (the page edge wins
///    where a full-width image's border meets the column edge);
/// 8. hovering a clickable MARGIN-OUTLINE row gets the pointing HAND — the same
///    click-to-jump affordance signal as a picker row, below the page edge (the
///    outline lives just inside the column, so the edge grab wins where they meet);
///    8b. hovering a REVEALED FOLD CHEVRON gets the pointing HAND too — its
///    click-to-toggle affordance, ranked with the outline row (the two never
///    geometrically overlap: the chevron sits in the leading pad, the outline
///    further left still, so which one is set never matters, only that either is);
///    8c. hovering a WORKING-SET STACK ROW (the bottom-left margin identity's
///    click-to-switch/close list) gets the pointing HAND too, ranked with the
///    outline row — a different margin surface, so the two never geometrically
///    overlap, only that either being set earns the hand;
/// 9. plain document text gets the I-beam;
/// 10. everywhere else (margins, scrim, gutter) is the plain arrow.
pub fn cursor_icon_for(ctx: CursorContext) -> CursorIcon {
    if ctx.dragging_edge {
        CursorIcon::ColResize
    } else if let Some(handle) = ctx.image_drag {
        image_handle_icon(handle)
    } else if ctx.dragging_text {
        CursorIcon::Text
    } else if ctx.over_popover_button {
        // The format popover's button — the pointing HAND (a clickable affordance),
        // ranked with the other hands (it never coexists with an overlay/menu, so
        // the relative order among the hands never matters, only that it earns one).
        CursorIcon::Pointer
    } else if ctx.over_menu_hand
        || ctx.over_clickable_overlay_row
        || ctx.over_clickable_lens
        || ctx.over_table_dims_cell
    {
        CursorIcon::Pointer
    } else if ctx.over_query_input {
        CursorIcon::Text
    } else if ctx.overlay_open || ctx.over_menu_bar {
        CursorIcon::Default
    } else if ctx.over_case_toggle {
        CursorIcon::Pointer
    } else if ctx.over_panel_field {
        CursorIcon::Text
    } else if ctx.over_panel {
        CursorIcon::Default
    } else if ctx.over_edge {
        CursorIcon::ColResize
    } else if let Some(handle) = ctx.image_hover {
        image_handle_icon(handle)
    } else if ctx.over_outline_row || ctx.over_fold_chevron || ctx.over_stack_row {
        CursorIcon::Pointer
    } else if ctx.over_text {
        CursorIcon::Text
    } else {
        CursorIcon::Default
    }
}

/// Whether (and to what) the OS `set_cursor` call should actually fire, given
/// the previously CACHED icon and the freshly decided one. `None` means no
/// call: either nothing changed, or the OS pointer is currently HIDDEN
/// (typing auto-hide, `pointer_hide::PointerHide::Hidden`) so there is
/// nothing visible to update. The caller does NOT advance its cache in that
/// case either — so the OS's real last-set icon and the cache stay in lockstep
/// (an invariant: the cache always equals the last icon actually handed to
/// `set_cursor`), and the very next un-hide (a `CursorMoved` always recomputes
/// context before this check, and any mouse motion un-hides — see
/// `pointer_hide::on_mouse_move`) compares the FRESH desired icon against that
/// still-accurate cache and fires exactly once if it truly differs — landing
/// directly in the context-correct shape instead of a stale one from before
/// the hide. Mirrors `pointer_hide::os_visibility_change`'s "only call on an
/// actual boundary" discipline, one door over for the icon instead of the
/// visibility bit.
pub fn cursor_icon_change(prev: CursorIcon, next: CursorIcon, hidden: bool) -> Option<CursorIcon> {
    if hidden || prev == next {
        None
    } else {
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(
        dragging_edge: bool,
        overlay_open: bool,
        over_edge: bool,
        over_text: bool,
    ) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_image_drag(
        handle: ImageHandle,
        dragging_edge: bool,
        overlay_open: bool,
        over_text: bool,
    ) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open,
            over_edge: false,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: Some(handle),
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_image_handle(
        handle: ImageHandle,
        overlay_open: bool,
        over_edge: bool,
        over_text: bool,
    ) -> CursorContext {
        CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: Some(handle),
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    /// A context with the margin-outline-row flag set (no overlay — the outline is
    /// margin chrome, hidden behind an overlay's scrim, so the two never co-occur).
    fn ctx_outline(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: true,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    /// A context with the working-set STACK-ROW flag set (no overlay — the stack
    /// is margin chrome, hidden behind an overlay's scrim, so the two never
    /// co-occur, same as the outline above).
    fn ctx_stack_row(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: true,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_row(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: true,
            over_edge,
            over_text,
            over_clickable_overlay_row: true,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_lens(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: true,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: true,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    /// A context with the table-dims-cell flag set (no lens/row — a TableDims
    /// card carries neither).
    fn ctx_table_dims_cell(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: true,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: true,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_query(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: true,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: true,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    #[test]
    fn nothing_hovered_is_the_plain_arrow() {
        assert_eq!(
            cursor_icon_for(ctx(false, false, false, false)),
            CursorIcon::Default
        );
    }

    #[test]
    fn plain_document_text_is_the_i_beam() {
        assert_eq!(
            cursor_icon_for(ctx(false, false, false, true)),
            CursorIcon::Text
        );
    }

    #[test]
    fn hovering_the_page_edge_is_col_resize() {
        assert_eq!(
            cursor_icon_for(ctx(false, false, true, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn overlay_open_alone_is_the_plain_arrow() {
        assert_eq!(
            cursor_icon_for(ctx(false, true, false, false)),
            CursorIcon::Default
        );
    }

    #[test]
    fn dragging_the_edge_alone_is_col_resize() {
        assert_eq!(
            cursor_icon_for(ctx(true, false, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn dragging_text_alone_is_the_i_beam() {
        let mut c = ctx(false, false, false, false);
        c.dragging_text = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Text);
    }

    #[test]
    fn dragging_text_beats_wandering_off_the_writing_column() {
        // THE BUG THIS CLOSED: `over_text` alone is a pure x/y hit-test, so a
        // drag that strays outside the column (past the last line, into a
        // margin/outline row) used to fall through to whatever that spot
        // shows at rest — here, the outline's pointing hand. An active text
        // drag must win regardless.
        let mut c = ctx(false, false, false, false);
        c.dragging_text = true;
        c.over_outline_row = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Text);
    }

    #[test]
    fn an_active_edge_drag_still_beats_dragging_text() {
        let mut c = ctx(true, false, false, false);
        c.dragging_text = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::ColResize);
    }

    #[test]
    fn an_active_image_drag_still_beats_dragging_text() {
        let mut c = ctx(false, false, false, false);
        c.dragging_text = true;
        c.image_drag = Some(ImageHandle::Left);
        assert_eq!(cursor_icon_for(c), CursorIcon::ColResize);
    }

    #[test]
    fn dragging_text_beats_the_popover_hand_and_every_lower_tier() {
        let mut c = ctx(false, false, false, false);
        c.dragging_text = true;
        c.over_popover_button = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Text);
    }

    #[test]
    fn edge_hover_beats_text() {
        assert_eq!(
            cursor_icon_for(ctx(false, false, true, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn overlay_open_beats_text() {
        // The scrim covers the document -- a spot that would otherwise be
        // plain document text still reads as the plain arrow, never the I-beam.
        assert_eq!(
            cursor_icon_for(ctx(false, true, false, true)),
            CursorIcon::Default
        );
    }

    #[test]
    fn dragging_edge_beats_text() {
        assert_eq!(
            cursor_icon_for(ctx(true, false, false, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn overlay_open_beats_edge_hover() {
        // The scrim covers the page edge too -- a would-be edge hover behind
        // an open overlay never shows the resize glyph.
        assert_eq!(
            cursor_icon_for(ctx(false, true, true, false)),
            CursorIcon::Default
        );
    }

    #[test]
    fn dragging_edge_beats_overlay_open() {
        // An ACTIVE drag (button down, mid-gesture) always wins -- it is never
        // masked by a summoned overlay appearing mid-drag.
        assert_eq!(
            cursor_icon_for(ctx(true, true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn overlay_open_beats_edge_hover_and_text_together() {
        assert_eq!(
            cursor_icon_for(ctx(false, true, true, true)),
            CursorIcon::Default
        );
    }

    #[test]
    fn dragging_edge_beats_overlay_open_and_text_together() {
        assert_eq!(
            cursor_icon_for(ctx(true, true, false, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn dragging_edge_beats_every_other_flag_at_once() {
        assert_eq!(
            cursor_icon_for(ctx(true, true, true, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn any_clickable_overlay_row_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_row(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_non_row_overlay_region_is_the_arrow_never_the_hand() {
        assert_eq!(
            cursor_icon_for(ctx(false, true, false, false)),
            CursorIcon::Default
        );
    }

    #[test]
    fn clickable_row_beats_the_generic_overlay_arrow() {
        assert_eq!(
            cursor_icon_for(ctx_row(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn clickable_row_beats_a_would_be_edge_or_text_beneath_it() {
        // The scrim covers the document, so edge/text beneath a row never
        // surface -- the hand still wins with those flags also set.
        assert_eq!(
            cursor_icon_for(ctx_row(false, true, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn an_active_edge_drag_still_beats_the_clickable_row_hand() {
        assert_eq!(
            cursor_icon_for(ctx_row(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn dragging_edge_beats_the_row_hand_with_every_flag_at_once() {
        assert_eq!(
            cursor_icon_for(ctx_row(true, true, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn a_clickable_lens_facet_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_lens(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn clickable_lens_beats_the_generic_overlay_arrow() {
        assert_eq!(
            cursor_icon_for(ctx_lens(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn clickable_lens_beats_a_would_be_edge_or_text_beneath_it() {
        // The scrim covers the document, so edge/text beneath the strip never
        // surface -- the hand still wins with those flags also set.
        assert_eq!(
            cursor_icon_for(ctx_lens(false, true, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn an_active_edge_drag_still_beats_the_lens_hand() {
        assert_eq!(
            cursor_icon_for(ctx_lens(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn dragging_edge_beats_the_lens_hand_with_every_flag_at_once() {
        assert_eq!(
            cursor_icon_for(ctx_lens(true, true, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn the_row_hand_and_the_lens_hand_both_resolve_to_the_pointer_if_ever_set_together() {
        // The strip and the rows sit on different lines and never geometrically
        // overlap, but the priority is stated regardless: either flag alone (or
        // both) resolves to the hand -- neither out-ranks the other.
        let both = CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open: true,
            over_edge: false,
            over_text: false,
            over_clickable_overlay_row: true,
            over_clickable_lens: true,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        };
        assert_eq!(cursor_icon_for(both), CursorIcon::Pointer);
    }

    // --- the INSERT-TABLE dimension picker's own grid: a cell = the pointing hand ---

    #[test]
    fn a_table_dims_grid_cell_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_table_dims_cell(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_table_dims_cell_beats_the_generic_overlay_arrow() {
        // THE BUG THIS CLOSES: before the grid earned its own priority arm, a
        // hovered cell fell through to the generic `overlay_open` -> arrow rule
        // (item 5), reading a clickable cell as inert card chrome.
        assert_eq!(
            cursor_icon_for(ctx(false, true, false, false)),
            CursorIcon::Default,
            "sanity: overlay_open alone (no table-dims flag) is still the arrow"
        );
        assert_eq!(
            cursor_icon_for(ctx_table_dims_cell(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_table_dims_cell_beats_a_would_be_edge_or_text_beneath_it() {
        // The scrim covers the document, so edge/text beneath the card never
        // surface -- the hand still wins with those flags also set.
        assert_eq!(
            cursor_icon_for(ctx_table_dims_cell(false, true, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn an_active_edge_drag_still_beats_the_table_dims_cell_hand() {
        assert_eq!(
            cursor_icon_for(ctx_table_dims_cell(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn dragging_edge_beats_the_table_dims_cell_hand_with_every_flag_at_once() {
        assert_eq!(
            cursor_icon_for(ctx_table_dims_cell(true, true, true)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn the_overlay_query_input_line_is_the_i_beam() {
        assert_eq!(
            cursor_icon_for(ctx_query(false, false, false)),
            CursorIcon::Text
        );
    }

    #[test]
    fn query_input_beats_the_generic_overlay_arrow() {
        assert_eq!(
            cursor_icon_for(ctx_query(false, false, false)),
            CursorIcon::Text
        );
    }

    #[test]
    fn a_clickable_row_outranks_the_query_input_field() {
        // A row and the query line never geometrically overlap, but the priority
        // is stated regardless: a row (were both set) resolves to the hand.
        let both = CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open: true,
            over_edge: false,
            over_text: false,
            over_clickable_overlay_row: true,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: true,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        };
        assert_eq!(cursor_icon_for(both), CursorIcon::Pointer);
    }

    #[test]
    fn a_margin_outline_row_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_outline(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_margin_outline_row_beats_the_plain_text_beneath_it() {
        assert_eq!(
            cursor_icon_for(ctx_outline(false, false, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_working_set_stack_row_is_the_pointing_hand() {
        // THE BUG THIS CLOSES: a stack row is click-to-switch (and its close
        // zone click-to-close), same as an outline row, but a hover over it used
        // to fall all the way through to the plain arrow — a clickable row read
        // as inert margin. `over_stack_row` must earn the hand exactly like
        // `over_outline_row` does.
        assert_eq!(
            cursor_icon_for(ctx_stack_row(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_working_set_stack_row_beats_the_plain_text_beneath_it() {
        assert_eq!(
            cursor_icon_for(ctx_stack_row(false, false, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_format_popover_button_is_the_pointing_hand() {
        // Hovering a popover button reads as a clickable affordance — the pointing
        // hand, exactly like a picker row (the popover never coexists with an
        // overlay, so it just needs to earn the hand).
        let mut c = ctx(false, false, false, false);
        c.over_popover_button = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Pointer);
    }

    // --- the WEB/LINUX MENU BAR: title/item = hand, dead bar space = arrow --------

    fn ctx_menu_hand(over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: true,
            over_menu_bar: true, // the hand is always within the bar surface
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    fn ctx_menu_bar(over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: true,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    #[test]
    fn a_clickable_menu_title_or_item_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_menu_hand(false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn a_menu_title_hand_beats_the_text_and_edge_beneath_the_bar() {
        assert_eq!(
            cursor_icon_for(ctx_menu_hand(true, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn dead_menu_bar_space_is_the_plain_arrow_never_the_i_beam() {
        assert_eq!(
            cursor_icon_for(ctx_menu_bar(false, true)),
            CursorIcon::Default
        );
    }

    #[test]
    fn dead_menu_bar_space_beats_a_would_be_page_edge_beneath_it() {
        assert_eq!(
            cursor_icon_for(ctx_menu_bar(true, false)),
            CursorIcon::Default
        );
    }

    fn ctx_case_toggle(over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge: false,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: true,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: false,
        }
    }

    #[test]
    fn the_case_toggle_cell_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_case_toggle(false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn the_case_toggle_cell_beats_the_plain_text_beneath_the_floating_panel() {
        // The panel floats over the writing column; a hover on its Aa cell reads as
        // the clickable hand, never the document I-beam under the card.
        assert_eq!(
            cursor_icon_for(ctx_case_toggle(false, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn an_active_page_edge_drag_still_beats_the_case_toggle_hand() {
        let mut c = ctx_case_toggle(false, false);
        c.dragging_edge = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::ColResize);
    }

    #[test]
    fn a_panel_field_cell_is_the_i_beam_and_beats_the_column_beneath_it() {
        let mut c = ctx(false, false, false, false);
        c.over_panel_field = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Text);
        // The floating panel covers the column: the field cell still wins even
        // with the document's own edge/text flags also set beneath it.
        c.over_edge = true;
        c.over_text = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Text);
    }

    #[test]
    fn an_active_page_edge_drag_still_beats_the_panel_field_i_beam() {
        let mut c = ctx(false, false, false, false);
        c.over_panel_field = true;
        c.dragging_edge = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::ColResize);
    }

    #[test]
    fn panel_dead_chrome_is_the_plain_arrow_and_beats_the_column_beneath_it() {
        // THE BUG THIS CLOSES: dead panel chrome (padding, inter-cell gaps) used to
        // fall through to whatever the document underneath the floating card shows —
        // the column edge's resize glyph or the I-beam. It must read as the plain
        // arrow instead, exactly like dead menu-bar space over the same document.
        let mut c = ctx(false, false, false, true);
        c.over_panel = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Default);
        c.over_text = false;
        c.over_edge = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::Default);
    }

    #[test]
    fn an_active_page_edge_drag_still_beats_panel_dead_chrome() {
        let mut c = ctx(false, false, false, false);
        c.over_panel = true;
        c.dragging_edge = true;
        assert_eq!(cursor_icon_for(c), CursorIcon::ColResize);
    }

    #[test]
    fn the_page_edge_still_beats_a_margin_outline_row() {
        assert_eq!(
            cursor_icon_for(ctx_outline(false, true, false)),
            CursorIcon::ColResize
        );
        assert_eq!(
            cursor_icon_for(ctx_outline(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn the_page_edge_still_beats_a_working_set_stack_row() {
        assert_eq!(
            cursor_icon_for(ctx_stack_row(false, true, false)),
            CursorIcon::ColResize
        );
        assert_eq!(
            cursor_icon_for(ctx_stack_row(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn an_active_edge_drag_still_beats_the_query_input_i_beam() {
        assert_eq!(
            cursor_icon_for(ctx_query(true, false, false)),
            CursorIcon::ColResize
        );
    }

    /// A context with the fold-chevron flag set (no overlay — its scrim would cover
    /// the chevron, so the two never co-occur).
    fn ctx_fold_chevron(dragging_edge: bool, over_edge: bool, over_text: bool) -> CursorContext {
        CursorContext {
            dragging_edge,
            dragging_text: false,
            overlay_open: false,
            over_edge,
            over_text,
            over_clickable_overlay_row: false,
            over_clickable_lens: false,
            over_table_dims_cell: false,
            over_query_input: false,
            over_outline_row: false,
            over_stack_row: false,
            over_menu_hand: false,
            over_menu_bar: false,
            over_case_toggle: false,
            over_panel_field: false,
            over_panel: false,
            image_drag: None,
            image_hover: None,
            over_popover_button: false,
            over_fold_chevron: true,
        }
    }

    #[test]
    fn a_revealed_fold_chevron_is_the_pointing_hand() {
        assert_eq!(
            cursor_icon_for(ctx_fold_chevron(false, false, false)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn the_fold_chevron_hand_beats_the_plain_text_beneath_it() {
        assert_eq!(
            cursor_icon_for(ctx_fold_chevron(false, false, true)),
            CursorIcon::Pointer
        );
    }

    #[test]
    fn an_active_page_edge_drag_still_beats_the_fold_chevron_hand() {
        assert_eq!(
            cursor_icon_for(ctx_fold_chevron(true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn the_page_edge_still_beats_a_fold_chevron_where_they_meet() {
        assert_eq!(
            cursor_icon_for(ctx_fold_chevron(false, true, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn image_handle_icon_maps_each_edge_and_corner_to_its_glyph() {
        // The single owner of the handle->glyph mapping (a no-wildcard match). Sides
        // are ↔, top/bottom are ↕, the "\" diagonal is NwseResize, the "/" is NeswResize.
        assert_eq!(image_handle_icon(ImageHandle::Left), CursorIcon::ColResize);
        assert_eq!(image_handle_icon(ImageHandle::Right), CursorIcon::ColResize);
        assert_eq!(image_handle_icon(ImageHandle::Top), CursorIcon::RowResize);
        assert_eq!(
            image_handle_icon(ImageHandle::Bottom),
            CursorIcon::RowResize
        );
        assert_eq!(
            image_handle_icon(ImageHandle::TopLeft),
            CursorIcon::NwseResize
        );
        assert_eq!(
            image_handle_icon(ImageHandle::BottomRight),
            CursorIcon::NwseResize
        );
        assert_eq!(
            image_handle_icon(ImageHandle::TopRight),
            CursorIcon::NeswResize
        );
        assert_eq!(
            image_handle_icon(ImageHandle::BottomLeft),
            CursorIcon::NeswResize
        );
    }

    #[test]
    fn hovering_each_image_handle_reads_as_that_handles_glyph() {
        for (h, want) in [
            (ImageHandle::Left, CursorIcon::ColResize),
            (ImageHandle::Right, CursorIcon::ColResize),
            (ImageHandle::Top, CursorIcon::RowResize),
            (ImageHandle::Bottom, CursorIcon::RowResize),
            (ImageHandle::TopLeft, CursorIcon::NwseResize),
            (ImageHandle::BottomRight, CursorIcon::NwseResize),
            (ImageHandle::TopRight, CursorIcon::NeswResize),
            (ImageHandle::BottomLeft, CursorIcon::NeswResize),
        ] {
            assert_eq!(
                cursor_icon_for(ctx_image_handle(h, false, false, false)),
                want
            );
        }
    }

    #[test]
    fn dragging_each_image_handle_reads_as_that_handles_glyph() {
        for (h, want) in [
            (ImageHandle::Right, CursorIcon::ColResize),
            (ImageHandle::Bottom, CursorIcon::RowResize),
            (ImageHandle::BottomRight, CursorIcon::NwseResize),
            (ImageHandle::TopRight, CursorIcon::NeswResize),
        ] {
            assert_eq!(
                cursor_icon_for(ctx_image_drag(h, false, false, false)),
                want
            );
        }
    }

    #[test]
    fn an_image_handle_hover_beats_plain_text_beneath_it() {
        assert_eq!(
            cursor_icon_for(ctx_image_handle(
                ImageHandle::BottomRight,
                false,
                false,
                true
            )),
            CursorIcon::NwseResize
        );
    }

    #[test]
    fn an_open_overlay_scrim_beats_an_image_handle_hover() {
        // The overlay's scrim covers the images too — a would-be handle hover
        // behind an open overlay reads as the plain arrow, never the resize glyph.
        assert_eq!(
            cursor_icon_for(ctx_image_handle(ImageHandle::Right, true, false, false)),
            CursorIcon::Default
        );
    }

    #[test]
    fn a_page_edge_hover_beats_an_image_handle_hover() {
        assert_eq!(
            cursor_icon_for(ctx_image_handle(ImageHandle::Right, false, true, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn an_active_page_edge_drag_still_beats_an_active_image_drag() {
        assert_eq!(
            cursor_icon_for(ctx_image_drag(ImageHandle::BottomRight, true, false, false)),
            CursorIcon::ColResize
        );
    }

    #[test]
    fn an_active_image_drag_beats_an_open_overlay() {
        assert_eq!(
            cursor_icon_for(ctx_image_drag(ImageHandle::Top, false, true, false)),
            CursorIcon::RowResize
        );
    }

    // --- cursor_icon_change: the "only call on a change, never while hidden" seam

    #[test]
    fn icon_change_is_none_when_unchanged() {
        assert_eq!(
            cursor_icon_change(CursorIcon::Text, CursorIcon::Text, false),
            None
        );
    }

    #[test]
    fn icon_change_fires_on_an_actual_change() {
        assert_eq!(
            cursor_icon_change(CursorIcon::Default, CursorIcon::Text, false),
            Some(CursorIcon::Text)
        );
    }

    #[test]
    fn icon_change_is_suppressed_while_the_os_pointer_is_hidden() {
        assert_eq!(
            cursor_icon_change(CursorIcon::Default, CursorIcon::Text, true),
            None
        );
    }

    #[test]
    fn icon_change_resumes_correctly_the_instant_the_pointer_is_visible_again() {
        // Simulates the seam `App::sync_cursor_icon` rides: while hidden, the
        // caller does NOT advance its cache (`prev` stays the last genuinely
        // -drawn icon); the next un-hide call (hidden = false) then sees the
        // real prev-vs-next gap and fires exactly once, landing on the
        // correct shape rather than a stale one.
        assert_eq!(
            cursor_icon_change(CursorIcon::Default, CursorIcon::Text, true),
            None
        );
        assert_eq!(
            cursor_icon_change(CursorIcon::Default, CursorIcon::Text, false),
            Some(CursorIcon::Text)
        );
    }
}
