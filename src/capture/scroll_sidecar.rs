use crate::render::{ScrollPos, TextPipeline};

macro_rules! sidecar_format {
    ($($args:tt)*) => {
        format!(
            concat!(
                "{{\n",
                "  \"schema\": {schema_json},\n",
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
                "  \"search\": {{ \"query\": {sq}, \"active\": {sa}, ",
                "\"case_sensitive\": {scs}, \"hit_count\": {hc}, \"current\": {cur}, ",
                "\"replace_active\": {ra}, \"replacement\": {rep}, ",
                "\"editing_replacement\": {er} }},\n",
                "  \"project\": {project},\n",
                "  \"overlay\": {overlay},\n",
                "  \"buffers\": {buffers},\n",
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
        pipeline.scroll_top_px(scroll)
    )
}
