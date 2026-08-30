use super::*;

fn meta(revealed: bool) -> TableMeta {
    TableMeta {
        range: (0, 0),
        ncols: 1,
        aligns: vec![crate::markdown::ColAlign::None],
        sep_doc_line: 1,
        revealed,
        visible: true,
        grid_rows: vec![(0, vec![]), (2, vec![])],
    }
}

#[test]
fn revealed_table_row_excludes_every_decoration_kind() {
    let table = meta(true);
    for row in [0, 1, 2] {
        assert!(
            !table_decoration_visible(&table, &[row], row),
            "revealed row {row} must reject cells, separator, and pan geometry"
        );
    }
    assert!(
        table_decoration_visible(&table, &[0], 2),
        "other table rows keep their grid"
    );
    assert!(
        table_decoration_visible(&meta(false), &[1], 1),
        "unrevealed tables retain decorations"
    );
}

#[test]
fn empty_cell_affordance_presence_and_filled_identity_law() {
    let empty = String::new();
    let filled = String::from("東京");
    for _world in crate::theme::THEMES {
        assert!(empty_cell_affordance(Some(&empty)), "empty enrolled");
        assert!(empty_cell_affordance(None), "ragged empty enrolled");
        assert!(
            !empty_cell_affordance(Some(&filled)),
            "filled stays unchanged"
        );
    }
}

fn context(pan: Option<(usize, f32)>) -> TablePlacementContext {
    TablePlacementContext {
        text_left: 0.0,
        view_w: 20.0,
        line_height: 20.0,
        pad: 2.0,
        rule_thick: 1.0,
        pan_bar_thick: 1.0,
        width: 200,
        height: 200,
        content: glyphon::Color::rgb(255, 255, 255),
        muted: glyphon::Color::rgb(128, 128, 128),
        table_pan: pan,
    }
}

#[test]
fn placement_wires_revealed_guard_and_empty_rect_presence() {
    let shaped = TableGridShaped {
        col_x: vec![0.0],
        col_w: vec![60.0],
        cells: vec![],
        row_heights: vec![20.0, 20.0],
    };
    let mut table = meta(true);
    table.range = (7, 30);
    table.grid_rows = vec![(0, vec![String::new()]), (2, vec![String::new()])];
    let mut placed = TablePlacement {
        areas: vec![],
        rule_rects: vec![],
        empty_rects: vec![],
        reports: vec![],
        pan_writeback: None,
        drawn_lines: vec![],
    };
    place_shaped_table(
        &mut placed,
        &table,
        &shaped,
        &[1, 2],
        context(Some((7, 10.0))),
        &|line| line as f32 * 20.0,
        &|bounds| bounds,
    );
    assert_eq!(
        placed.rule_rects.len(),
        0,
        "revealed separator and final pan route through guard"
    );
    assert_eq!(
        placed.empty_rects.len(),
        1,
        "only the unrevealed empty cell emits a nonzero wash"
    );
    assert!(
        placed.empty_rects[0][2] > 0.0 && placed.empty_rects[0][3] > 0.0,
        "wash has pixel extent"
    );
    table.revealed = false;
    let mut filled = TablePlacement {
        areas: vec![],
        rule_rects: vec![],
        empty_rects: vec![],
        reports: vec![],
        pan_writeback: None,
        drawn_lines: vec![],
    };
    table.grid_rows[0].1[0] = "x".into();
    place_shaped_table(
        &mut filled,
        &table,
        &shaped,
        &[],
        context(None),
        &|line| line as f32 * 20.0,
        &|bounds| bounds,
    );
    assert_eq!(
        filled.empty_rects.len(),
        1,
        "content drops its own wash while sibling stays present"
    );
}
