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
    include!("cursor_shape/tests/helpers.rs");

    mod basic;
    mod priority;
}
