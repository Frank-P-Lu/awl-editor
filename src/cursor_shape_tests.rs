use super::*;

fn ctx(dragging_edge: bool, overlay_open: bool, over_edge: bool, over_text: bool) -> CursorContext {
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
    // hovered cell fell through to the generic `overlay_open` -> arrow
    // rule, reading a clickable cell as inert card chrome.
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
