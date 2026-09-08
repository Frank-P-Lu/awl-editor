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
