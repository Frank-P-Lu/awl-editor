use super::*;

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
