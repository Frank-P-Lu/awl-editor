use crate::render::{ScrollPos, TextPipeline};

macro_rules! sidecar_format {
    ($($args:tt)*) => {
        format!(
            concat!(
                "{{\n",
                "  \"schema\": {schema_json},\n",
                "  \"driver\": {driver},\n",
                "  \"semantic\": {semantic},\n",
                "  \"canvas\": {canvas},\n",
                "  \"font\": {{ \"family\": {ff}, \"zoom\": {fz}, \"size\": {fs}, ",
                "\"line_height\": {lh}, \"ornament\": {ornament}, \"cjk\": {cjk}, ",
                "\"scripts\": {scripts} }},\n",
                "  \"theme\": {{ \"name\": {tn}, \"font_family\": {tf}, \"mode\": {tm}, ",
                "\"base100\": {tb100}, \"primary\": {tp}, \"heading_bold\": {thb} }},\n",
                "  \"caret_mode\": {cm},\n",
                "  \"dictionary\": {dict},\n",
                "  \"spellcheck\": {sp},\n",
                "  \"date_format\": {date_format},\n",
                "  \"text_origin\": {{ \"left\": {left}, \"top\": {top} }},\n",
                "  \"page\": {page},\n",
                "  \"wysiwyg\": {wysiwyg},\n",
                "  \"popover\": {popover},\n",
                "  \"tables\": {tables},\n",
                "  \"xray\": {xray},\n",
                "  \"images\": {images},\n",
                "  \"outline\": {outline},\n",
                "  \"menubar\": {menubar},\n",
                "  \"doc_lang\": {doc_lang},\n",
                "  \"md_spans\": {md_spans},\n",
                "  \"syn_lang\": {syn_lang},\n",
                "  \"syn_spans\": {syn_spans},\n",
                "  \"readout\": {readout},\n",
                "  \"gutter\": {gutter},\n",
                "  \"notice\": {notice},\n",
                "  \"dim_overlay\": {dim_overlay},\n",
                "  \"debug\": {debug},\n",
                "  \"whichkey\": {whichkey},\n",
                "  \"hud\": {hud},\n",
                "  \"about\": {about},\n",
                "  \"lifetime\": {lifetime},\n",
                "  \"streaks\": {streaks},\n",
                "  \"peek\": {peek},\n",
                "  \"caret_preview\": {caret_preview},\n",
                "  \"line_count\": {lc},\n",
                "  {scroll},\n",
                "  \"cursor\": {{ \"line\": {cl}, \"col\": {cc} }},\n",
                "  \"folds\": {folds},\n",
                "  \"selection\": {sel},\n",
                "  \"text\": {text_json},\n",
                "  \"first_lines\": [{fl}],\n",
                "  \"layout\": {layout},\n",
                "  \"search\": {{ \"query\": {sq}, \"active\": {sa}, ",
                "\"case_sensitive\": {scs}, \"hit_count\": {hc}, \"current\": {cur}, ",
                "\"replace_active\": {ra}, \"replacement\": {rep}, ",
                "\"editing_replacement\": {er} }},\n",
                "  \"project\": {project},\n",
                "  \"overlay\": {overlay},\n",
                "  \"buffers\": {buffers},\n",
                "  \"replay_skips\": {replay_skips},\n",
                "  \"diff\": {diff}{caret_extra}\n",
                "}}\n",
            ),
            $($args)*
        )
    };
}
pub(super) use sidecar_format;

pub(super) fn fields(scroll: ScrollPos, pipeline: &TextPipeline) -> String {
    format!(
        "\"scroll_lines\": {},\n  \"scroll_px\": {},\n  \"scroll_top_px\": {}",
        scroll.row,
        scroll.px(),
        pipeline.rendered_scroll_top_px(scroll)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::capture::{CaptureOpts, capture_with};
    use crate::testscratch::ScratchDir;

    fn capture(
        dir: &std::path::Path,
        name: &str,
        text: &str,
        markdown: bool,
        scroll: ScrollPos,
    ) -> (Vec<u8>, serde_json::Value) {
        let mut buffer = Buffer::from_str(text);
        if markdown {
            buffer.set_path(dir.join(format!("{name}.md")));
        }
        let png = dir.join(format!("{name}.png"));
        capture_with(
            &png,
            &buffer,
            &CaptureOpts {
                scroll: Some(scroll),
                ..CaptureOpts::default()
            },
        )
        .expect("scroll capture requires a GPU adapter");
        let pixels = std::fs::read(&png).expect("PNG bytes");
        let json = std::fs::read_to_string(png.with_extension("json")).expect("sidecar");
        (
            pixels,
            serde_json::from_str(&json).expect("valid sidecar JSON"),
        )
    }

    #[test]
    fn subpixel_sidecar_keeps_semantics_but_reports_settled_pixels() {
        let _g = crate::testlock::serial();
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl_scroll_px_{}", std::process::id())),
        );
        let text = "visible raster witness abcdefghijklmnopqrstuvwxyz\n".repeat(80);
        let (zero_png, zero) = capture(&dir, "zero", &text, false, ScrollPos::default());
        let (sub_png, sub) = capture(&dir, "sub", &text, false, ScrollPos { row: 0, px_q: 17 });

        assert_eq!(sub["scroll_px"], serde_json::json!(0.265625));
        assert_eq!(sub["scroll_top_px"], serde_json::json!(0));
        assert_eq!(zero["scroll_top_px"], serde_json::json!(0));
        assert_eq!(
            zero_png, sub_png,
            "0:17 must render byte-identically to 0:0"
        );
        let image = image::load_from_memory(&zero_png).unwrap().to_rgba8();
        let ground = *image.get_pixel(0, 0);
        assert!(
            image.pixels().any(|pixel| *pixel != ground),
            "pixel identity witness must contain real glyph ink"
        );
    }

    #[test]
    fn table_sidecar_reports_rounded_rendered_top() {
        let _g = crate::testlock::serial();
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl_table_scroll_{}", std::process::id())),
        );
        let table = "| left | right |\n| --- | --- |\n| cell | value |\n\n".repeat(80);
        let (_, json) = capture(&dir, "table", &table, true, ScrollPos { row: 0, px_q: 48 });
        assert_eq!(json["scroll_px"], serde_json::json!(0.75));
        assert_eq!(
            json["scroll_top_px"],
            serde_json::json!(1),
            "table appearance coordinate follows whole-pixel render geometry"
        );
        assert!(
            json["tables"]
                .as_array()
                .is_some_and(|tables| !tables.is_empty()),
            "fixture must exercise the table render surface"
        );
    }
}
