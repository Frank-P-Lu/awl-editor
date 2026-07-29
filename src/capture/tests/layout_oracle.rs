//! Capture-artifact proof for the shaped-frame layout oracle.

use super::super::*;
use crate::testscratch::ScratchDir;

#[test]
fn sidecar_layout_rows_locate_wrap_caret_and_selection() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(20);
    crate::theme::set_active_by_name("Gumtree").unwrap();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_layout_oracle_{}", std::process::id())),
    );
    let png = dir.join("layout.png");
    let text = "iiiiWWWW proportional wrap witness ".repeat(10);
    let buffer = crate::buffer::Buffer::from_str(&text);
    capture_with(
        &png,
        &buffer,
        &CaptureOpts {
            selection: Some(((0, 2), (0, 60))),
            ..CaptureOpts::default()
        },
    )
    .expect("layout capture requires a GPU adapter");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();

    assert_eq!(json["schema"], serde_json::json!(schema_plain()));
    let rows = json["layout"]["rows"].as_array().expect("layout rows");
    assert!(rows.len() > 2, "narrow fixture must visibly wrap");
    assert!(
        rows.iter().enumerate().all(|(index, row)| {
            row["index"] == serde_json::json!(index)
                && row["line"] == serde_json::json!(0)
                && row["xs"].as_array().is_some_and(|xs| {
                    xs.len()
                        == row["end_col"].as_u64().unwrap() as usize
                            - row["start_col"].as_u64().unwrap() as usize
                            + 1
                })
                && row["height"].as_f64().is_some_and(|height| height > 0.0)
        }),
        "every artifact row carries its source span and shaped geometry"
    );
    let caret = &json["layout"]["caret"];
    assert_eq!(caret["line"], serde_json::json!(0));
    assert_eq!(caret["col"], serde_json::json!(0));
    let caret_row = caret["row"].as_u64().unwrap() as usize;
    assert_eq!(rows[caret_row]["start_col"], serde_json::json!(0));
    let selection = json["layout"]["selection"]
        .as_array()
        .expect("selection segments");
    assert!(
        selection.len() >= 2,
        "a cross-wrap selection must be located on each visual row"
    );
    assert!(selection.iter().all(|segment| {
        let row = segment["row"].as_u64().unwrap() as usize;
        row < rows.len() && segment["x1"].as_f64().unwrap() >= segment["x0"].as_f64().unwrap()
    }));
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}
