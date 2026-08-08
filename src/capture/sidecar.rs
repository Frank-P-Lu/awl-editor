use super::opts::CaptureOpts;
use super::{CANVAS_HEIGHT, CANVAS_WIDTH, schema_held, schema_plain, schema_timeline};
use crate::render::{ScriptFontReports, TextPipeline, ViewState};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

#[inline]
// Test captures read shared globals; enforce the process-wide test lock here.
pub(super) fn assert_capture_is_serialized() {
    #[cfg(test)]
    assert!(
        crate::testlock::currently_held(),
        "capture law: a capture in a test build must hold `crate::testlock::serial()` \
         for its whole window — an unguarded capture races every theme-flipping test \
         and reports state it did not render. Add \
         `let _tg = crate::testlock::serial();` as the first line of the test."
    );
}
pub(super) struct CaretFrame {
    pub(super) t_ms: u32,
    pub(super) pos: (f32, f32),
    pub(super) target: (f32, f32),
    pub(super) settle: f32,
    pub(super) animating: bool,
    pub(super) scale: f32,
    pub(super) block_w: f32,
    pub(super) block_h: f32,
    pub(super) trail: Option<TrailReport>,
    pub(super) cosmetic: CosmeticReport,
}
pub(super) struct CosmeticReport {
    pub(super) present: bool,
    pub(super) length: f32,
    pub(super) vertical: bool,
    pub(super) held: bool,
    pub(super) alpha: f32,
    pub(super) sweep: f32,
    pub(super) tail: (f32, f32),
    pub(super) head: (f32, f32),
}
pub(super) struct TrailReport {
    pub(super) holding: bool,
    pub(super) length: f32,
    pub(super) tail: (f32, f32),
    pub(super) head: (f32, f32),
}
pub(super) fn write_sidecar(
    out_png: &Path,
    view: &ViewState,
    pipeline: &TextPipeline,
    opts: &CaptureOpts,
    caret: Option<&CaretFrame>,
) -> Result<()> {
    assert_capture_is_serialized();
    let json_path = out_png.with_extension("json");
    let text = &view.text;
    let (cursor_line, cursor_col) = (view.cursor_line, view.cursor_col);
    let first_lines: Vec<String> = text.lines().take(12).map(|s| s.to_string()).collect();
    let first_lines_json = first_lines
        .iter()
        .map(|l| json_string(l))
        .collect::<Vec<_>>()
        .join(", ");

    let search_cur = view
        .search_current
        .map(|i| i.to_string())
        .unwrap_or_else(|| "null".into());
    let active = crate::theme::active();
    let script_fonts = pipeline.script_font_reports();
    let caret_mode = match crate::caret::mode() {
        crate::caret::CaretMode::Block => "block",
        crate::caret::CaretMode::Morph => "morph",
        crate::caret::CaretMode::Ibeam => "ibeam",
    };
    let dictionary = crate::config::dictionary_name(crate::spell::active_variant());
    let spellcheck = crate::spell::spellcheck_on();
    let date_format = date_format_json();
    let syn_lang_json = match pipeline.syn_lang_report() {
        Some(name) => json_string(name),
        None => "null".to_string(),
    };
    let (schema, caret_extra) = caret_block(caret);
    debug_assert!(!schema.is_empty());
    let (font_zoom, font_size, line_height) = pipeline.effective_font_metrics();
    let json = super::scroll_sidecar::sidecar_format!(
        schema_json = json_string(&schema),
        driver = json_string(opts.driver.as_str()),
        semantic = opts.semantic_json(),
        caret_extra = caret_extra,
        cjk = cjk_json(&script_fonts),
        scripts = scripts_json(&script_fonts),
        doc_lang = doc_lang_json(pipeline),
        dict = json_string(dictionary),
        sp = spellcheck,
        date_format = date_format,
        debug = debug_json(pipeline),
        whichkey = whichkey_json(pipeline),
        hud = hud_json(pipeline),
        about = about_json(pipeline),
        lifetime = lifetime_json(pipeline),
        streaks = streaks_json(pipeline),
        peek = peek_json(pipeline),
        caret_preview = caret_preview_json(pipeline),
        wysiwyg = wysiwyg_json(pipeline),
        popover = popover_json(pipeline),
        tables = tables_json(pipeline),
        xray = xray_json(pipeline),
        images = images_json(pipeline),
        outline = outline_json(pipeline),
        menubar = menubar_json(pipeline),
        md_spans = span_array_json(&pipeline.md_report()),
        syn_lang = syn_lang_json,
        syn_spans = span_array_json(&pipeline.syn_report()),
        readout = readout_json(pipeline),
        gutter = gutter_json(pipeline),
        notice = notice_json(pipeline),
        dim_overlay = pipeline.dims_doc(),
        canvas = canvas_json(opts),
        ff = json_string(active.font),
        fz = font_zoom,
        fs = font_size,
        lh = line_height,
        ornament = json_string(active.ornament_face),
        tn = json_string(active.name),
        tf = json_string(active.font),
        tm = json_string(if active.dark { "dark" } else { "light" }),
        tb100 = json_string(&active.base_100.hex()),
        tp = json_string(&active.primary.hex()),
        thb = crate::markdown::heading_weight_bold(active.heading_bold, 2),
        cm = json_string(caret_mode),
        left = pipeline.text_left(),
        top = pipeline.text_origin_top(),
        page = page_json(pipeline),
        lc = pipeline.line_count(),
        scroll = super::scroll_sidecar::fields(view.scroll, pipeline),
        cl = cursor_line,
        cc = cursor_col,
        folds = folds_json(view),
        sel = selection_json(view),
        text_json = json_string(text),
        fl = first_lines_json,
        layout = super::layout_sidecar::from_pipeline(pipeline)?,
        sq = json_string(&view.search_query),
        sa = view.search_active,
        scs = view.search_case_sensitive,
        hc = view.search_matches.len(),
        cur = search_cur,
        ra = view.search_replace_active,
        rep = json_string(&view.search_replacement),
        er = view.search_editing_replacement,
        project = project_json(opts),
        overlay = overlay_json(opts, pipeline),
        buffers = buffers_json(opts, view),
        replay_skips = super::replay_sidecar::replay_skips_json(opts),
        diff = diff_json(opts),
    );

    let mut f = std::fs::File::create(&json_path)
        .with_context(|| format!("failed to create {}", json_path.display()))?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

/// THE CALM NOTICE block: `{ text, kind }`, or `null` when nothing is showing.
///
/// Read off the PIPELINE, not off `CaptureOpts` — the same field the notice
/// chrome shapes from. A sidecar that reported the fold's input rather than the
/// renderer's own copy could say "saved" about a frame that drew nothing, which
/// is the exact class of disagreement this block was added to make visible.
fn notice_json(pipeline: &TextPipeline) -> String {
    match pipeline.notice_report() {
        Some((text, kind)) => format!(
            "{{ \"text\": {}, \"kind\": {} }}",
            json_string(&text),
            json_string(kind.as_str())
        ),
        None => "null".to_string(),
    }
}

fn folds_json(view: &ViewState) -> String {
    let items: Vec<String> = view.folds.iter().map(|h| h.to_string()).collect();
    format!("[{}]", items.join(", "))
}

fn selection_json(view: &ViewState) -> String {
    match view.selection {
        Some(((l0, c0), (l1, c1))) => format!(
            "{{ \"start\": {{ \"line\": {l0}, \"col\": {c0} }}, \"end\": {{ \"line\": {l1}, \"col\": {c1} }} }}"
        ),
        None => "null".to_string(),
    }
}

fn diff_json(opts: &CaptureOpts) -> String {
    match &opts.diff {
        Some(d) => format!(
            "{{ \"active\": {}, \"label\": {}, \"struck\": {}, \"washed\": {}, \"modified\": {}, \"moved\": {}, \"folds\": {} }}",
            d.active,
            json_string(&d.label),
            d.struck,
            d.washed,
            d.modified,
            d.moved,
            d.folds
        ),
        None => "null".to_string(),
    }
}

fn buffers_json(opts: &CaptureOpts, view: &ViewState) -> String {
    match &opts.buffers {
        Some(b) => format!(
            "{{ \"open\": {}, \"active\": {} }}",
            b.open,
            json_string(&b.active)
        ),
        None => format!(
            "{{ \"open\": 1, \"active\": {} }}",
            json_string(&view.gutter_name)
        ),
    }
}

/// The sidecar's `project` object. `pub(super)` for one test-only consumer:
/// `capture::tests::capture_md_drift` reads the KEYS out of this writer's own
/// output, so CAPTURE.md's `project` row cannot claim a field set the file does
/// not have.
pub(super) fn project_json(opts: &CaptureOpts) -> String {
    match &opts.project {
        Some(p) => {
            let branch = p
                .branch
                .as_ref()
                .map(|b| json_string(b))
                .unwrap_or_else(|| "null".into());
            let opt_path = |p: &Option<std::path::PathBuf>| {
                p.as_ref()
                    .map(|v| json_string(&v.to_string_lossy()))
                    .unwrap_or_else(|| "null".into())
            };
            format!(
                "{{ \"root\": {}, \"name\": {}, \"branch\": {}, \"dirty\": {}, \"default_folder\": {}, \"workspace\": {}, \"keymap_flavor\": {} }}",
                json_string(&p.root.to_string_lossy()),
                json_string(&p.name),
                branch,
                p.dirty,
                opt_path(&p.default_folder),
                opt_path(&p.workspace),
                json_string(p.keymap_flavor),
            )
        }
        None => "null".to_string(),
    }
}

fn overlay_json(opts: &CaptureOpts, pipeline: &TextPipeline) -> String {
    let window = match pipeline.overlay_window_report() {
        Some((top, lines, sel_row, card_h, canvas_h)) => format!(
            "{{ \"top\": {top}, \"lines\": {lines}, \"sel_row\": {sel_row}, \"card_h\": {card_h}, \"canvas_h\": {canvas_h} }}"
        ),
        None => "null".to_string(),
    };
    match &opts.overlay {
        Some(o) => {
            let items = o
                .items
                .iter()
                .map(|i| json_string(i))
                .collect::<Vec<_>>()
                .join(", ");
            let bindings = o
                .bindings
                .iter()
                .map(|b| json_string(b))
                .collect::<Vec<_>>()
                .join(", ");
            let ranges = o
                .ranges
                .iter()
                .map(|r| match r {
                    Some(f) => format!("{f:.3}"),
                    None => "null".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            let git = o
                .git
                .iter()
                .map(|g| json_string(g))
                .collect::<Vec<_>>()
                .join(", ");
            let browse_dir = o
                .browse_dir
                .as_ref()
                .map(|d| json_string(d))
                .unwrap_or_else(|| "null".into());
            let return_to = o
                .return_to
                .map(json_string)
                .unwrap_or_else(|| "null".into());
            let capture = match &o.capture {
                Some(c) => {
                    let captured = c
                        .captured
                        .iter()
                        .map(|x| json_string(x))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{{ \"command\": {}, \"stage\": {}, \"chord_mode\": {}, \"captured\": [{}], \"prompt\": {} }}",
                        json_string(&c.command),
                        json_string(c.stage),
                        c.chord_mode,
                        captured,
                        json_string(&c.prompt),
                    )
                }
                None => "null".to_string(),
            };
            let spell_target = super::opts::spell_target_json(o.spell_target);
            let context_anchor = super::opts::context_anchor_json(o.context_anchor);
            let lens = o.lens.map(json_string).unwrap_or_else(|| "null".into());
            let lens_strip = o
                .lens_strip
                .iter()
                .map(|(label, active)| format!("[{}, {}]", json_string(label), active))
                .collect::<Vec<_>>()
                .join(", ");
            let sections = o
                .sections
                .iter()
                .map(|s| json_string(s))
                .collect::<Vec<_>>()
                .join(", ");
            let preview_id = o
                .preview_id
                .as_ref()
                .map(|p| json_string(p))
                .unwrap_or_else(|| "null".into());
            let preview_view = o
                .preview_view
                .map(json_string)
                .unwrap_or_else(|| "null".into());
            let empty = o
                .empty
                .as_ref()
                .map(|m| json_string(m))
                .unwrap_or_else(|| "null".into());
            format!(
                concat!(
                    "{{ \"active\": {}, \"mode\": {}, \"title\": {}, \"query\": {}, ",
                    "\"selected_index\": {}, \"browse_dir\": {}, \"return_to\": {}, ",
                    "\"spell_target\": {}, \"context_anchor\": {}, \"hint\": {}, ",
                    "\"notice\": {}, \"lens\": {}, ",
                    "\"workspace\": {}, \"lens_strip\": [{}], \"sections\": [{}], ",
                    "\"preview_id\": {}, \"preview_view\": {}, ",
                    "\"detail_focus\": {}, \"diff_scroll\": {}, ",
                    "\"show_hidden\": {}, \"capture\": {}, \"empty\": {}, \"window\": {}, ",
                    "\"items\": [{}], \"bindings\": [{}], \"ranges\": [{}], ",
                    "\"git\": [{}] }}",
                ),
                o.active,
                json_string(o.mode),
                json_string(o.title),
                json_string(&o.query),
                o.selected_index,
                browse_dir,
                return_to,
                spell_target,
                context_anchor,
                json_string(&o.hint),
                json_string(&o.notice),
                lens,
                o.workspace,
                lens_strip,
                sections,
                preview_id,
                preview_view,
                o.detail_focus,
                o.diff_scroll,
                o.show_hidden,
                capture,
                empty,
                window,
                items,
                bindings,
                ranges,
                git
            )
        }
        None => concat!(
            "{ \"active\": false, \"mode\": null, \"title\": null, \"query\": \"\", ",
            "\"selected_index\": null, \"browse_dir\": null, \"return_to\": null, ",
            "\"spell_target\": null, \"context_anchor\": null, \"hint\": null, ",
            "\"notice\": \"\", ",
            "\"lens\": null, \"workspace\": false, \"lens_strip\": [], ",
            "\"sections\": [], \"preview_id\": null, \"preview_view\": null, ",
            "\"detail_focus\": false, ",
            "\"diff_scroll\": 0, \"show_hidden\": false, \"capture\": null, ",
            "\"empty\": null, \"window\": null, \"items\": [], \"bindings\": [], ",
            "\"ranges\": [], \"git\": [] }",
        )
        .to_string(),
    }
}

fn canvas_json(opts: &CaptureOpts) -> String {
    let (canvas_w, canvas_h) = opts.canvas.unwrap_or((CANVAS_WIDTH, CANVAS_HEIGHT));
    match (opts.canvas, opts.dpi) {
        (None, None) => format!("{{ \"width\": {canvas_w}, \"height\": {canvas_h} }}"),
        _ => format!(
            "{{ \"width\": {canvas_w}, \"height\": {canvas_h}, \"dpi\": {} }}",
            opts.dpi.unwrap_or(1.0)
        ),
    }
}

fn page_json(pipeline: &TextPipeline) -> String {
    let (page_on, page_measure, col_left, col_w) = pipeline.page_geometry();
    let class = match pipeline.page_class() {
        crate::page::PageClass::Prose => "prose",
        crate::page::PageClass::Code => "code",
    };
    format!(
        "{{ \"on\": {}, \"measure\": {}, \"class\": \"{}\", \"column\": {{ \"left\": {}, \"width\": {} }}, \"background\": {}, \"ambient\": {} }}",
        page_on,
        page_measure,
        class,
        col_left,
        col_w,
        super::background_sidecar::background_json(
            pipeline.effective_background(),
            pipeline.lava_render_phase(),
            pipeline.warp_travel(),
        ),
        ambient_json(pipeline),
    )
}

fn ambient_json(pipeline: &TextPipeline) -> String {
    let ambient = crate::theme::active().render_caps.ambient;
    match ambient.stars_params() {
        None => format!("{{ \"style\": {} }}", json_string(ambient.as_str())),
        Some((tint, _cell, _density, _size, _peak, _floor)) => format!(
            "{{ \"style\": {}, \"tint\": {}, \"count\": {}, \"phase\": {} }}",
            json_string(ambient.as_str()),
            json_string(&tint.hex()),
            pipeline.stars_pipeline.instance_count(),
            pipeline.stars_render_phase(),
        ),
    }
}

fn date_format_json() -> String {
    let fmt = crate::dateformat::active_format();
    let (y, m, d) = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
    format!(
        "{{ \"format\": {}, \"example\": {} }}",
        json_string(fmt.config_name()),
        json_string(&fmt.format(y, m, d))
    )
}

fn wysiwyg_json(pipeline: &TextPipeline) -> String {
    let (on, concealed) = pipeline.wysiwyg_report();
    format!(
        "{{ \"on\": {on}, \"concealed\": {} }}",
        span_array_json(&concealed)
    )
}

fn popover_json(pipeline: &TextPipeline) -> String {
    let on = crate::popover::popover_on();
    match pipeline.popover_report() {
        Some((card, rows)) => {
            let card_json = format!("[{}, {}, {}, {}]", card[0], card[1], card[2], card[3]);
            let buttons: Vec<String> = rows
                .iter()
                .map(|(label, active, span)| {
                    format!(
                        "{{ \"label\": {}, \"active\": {active}, \"x0\": {}, \"x1\": {} }}",
                        json_string(label),
                        span[0],
                        span[1]
                    )
                })
                .collect();
            format!(
                "{{ \"on\": {on}, \"shown\": true, \"card\": {card_json}, \"buttons\": [{}] }}",
                buttons.join(", ")
            )
        }
        None => format!("{{ \"on\": {on}, \"shown\": false, \"card\": null, \"buttons\": [] }}"),
    }
}

fn outline_json(pipeline: &TextPipeline) -> String {
    let (on, headings, current, collapsed) = pipeline.outline_report();
    let body = headings
        .iter()
        .map(|(text, level, line)| {
            format!(
                "{{ \"text\": {}, \"level\": {level}, \"line\": {line} }}",
                json_string(text)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let ancestors = pipeline
        .outline_ancestors()
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let collapsed = collapsed
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let current = current
        .map(|c| c.to_string())
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{ \"on\": {on}, \"headings\": [{body}], \"current\": {current}, \"ancestors\": [{ancestors}], \"collapsed\": [{collapsed}] }}"
    )
}

fn menubar_json(pipeline: &TextPipeline) -> String {
    let (shown, open, titles) = pipeline.menubar_report();
    let items = titles
        .iter()
        .map(|t| json_string(t))
        .collect::<Vec<_>>()
        .join(", ");
    let open_json = open
        .map(|t| json_string(&t))
        .unwrap_or_else(|| "null".to_string());
    format!("{{ \"shown\": {shown}, \"open_menu\": {open_json}, \"items\": [{items}] }}")
}

fn tables_json(pipeline: &TextPipeline) -> String {
    let body = pipeline
        .tables_report()
        .iter()
        .map(|t| {
            let widths = t
                .col_widths
                .iter()
                .map(|w| format!("{w}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{{ \"range\": [{}, {}], \"rows\": {}, \"cols\": {}, \"col_widths\": [{}], \"revealed\": {} }}",
                t.range.0, t.range.1, t.rows, t.cols, widths, t.revealed
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", body)
}

fn xray_json(pipeline: &TextPipeline) -> String {
    match pipeline.xray_report() {
        Some((line, chars, pan)) => format!(
            "{{ \"active\": true, \"line\": {}, \"chars\": {}, \"pan\": {:.1} }}",
            line, chars, pan
        ),
        None => "{ \"active\": false, \"line\": null, \"chars\": null, \"pan\": null }".to_string(),
    }
}

fn images_json(pipeline: &TextPipeline) -> String {
    let body = pipeline
        .images_report()
        .iter()
        .map(|im| {
            let hint = im
                .width_hint
                .map(|h| h.to_string())
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{ \"range\": [{}, {}], \"line\": {}, \"path\": {}, \"width_hint\": {}, \"display_w\": {:.1}, \"display_h\": {:.1}, \"missing\": {}, \"revealed\": {} }}",
                im.range.0, im.range.1, im.line, json_string(&im.path), hint,
                im.display_w, im.display_h, im.missing, im.revealed
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", body)
}

fn span_array_json<S: AsRef<str>>(spans: &[(usize, usize, S)]) -> String {
    let body = spans
        .iter()
        .map(|(s, e, tag)| format!("[{}, {}, {}]", s, e, json_string(tag.as_ref())))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", body)
}

fn readout_json(pipeline: &TextPipeline) -> String {
    match pipeline.readout_report() {
        Some((words, reading_min, unit)) => {
            format!(
                "{{ \"words\": {words}, \"reading_min\": {reading_min}, \"unit\": {} }}",
                json_string(unit.tag())
            )
        }
        None => "null".to_string(),
    }
}

pub(super) fn cjk_json(fonts: &ScriptFontReports) -> String {
    script_font_json(fonts, crate::theme::FontId::Ja)
}

fn script_font_json(fonts: &ScriptFontReports, id: crate::theme::FontId) -> String {
    match fonts.get(id) {
        Some((family, bundled)) => {
            format!(
                "{{ \"family\": {}, \"bundled\": {bundled} }}",
                json_string(family)
            )
        }
        None => "null".to_string(),
    }
}

pub(super) fn scripts_json(fonts: &ScriptFontReports) -> String {
    use crate::theme::FontId;
    format!(
        "{{ \"ja\": {}, \"zh_hans\": {}, \"zh_hant\": {}, \"ko\": {} }}",
        script_font_json(fonts, FontId::Ja),
        script_font_json(fonts, FontId::ZhHans),
        script_font_json(fonts, FontId::ZhHant),
        script_font_json(fonts, FontId::Ko),
    )
}

fn doc_lang_json(pipeline: &TextPipeline) -> String {
    match pipeline.doc_lang_report() {
        Some(lang) => json_string(lang.code()),
        None => "null".to_string(),
    }
}

fn debug_json(pipeline: &TextPipeline) -> String {
    let perf = pipeline.debug_perf_report();
    let num_f = |v: Option<f32>| v.map_or("null".to_string(), |v| format!("{v}"));
    let num_u = |v: Option<u64>| v.map_or("null".to_string(), |v| format!("{v}"));
    let (autosave_state, autosave_since_s) = match perf.autosave {
        None => ("null".to_string(), "null".to_string()),
        Some(crate::debug::AutosaveState::Off) => ("\"off\"".to_string(), "null".to_string()),
        Some(crate::debug::AutosaveState::Held) => ("\"held\"".to_string(), "null".to_string()),
        Some(crate::debug::AutosaveState::Saved(since)) => ("\"saved\"".to_string(), num_u(since)),
    };
    format!(
        "{{ \"enabled\": {}, \"text\": {}, \"frame_ms\": {}, \"worst_ms\": {}, \"budget_ms\": {}, \"key_px_ms\": {}, \"redraws\": {}, \"still\": {}, \"autosave_state\": {}, \"autosave_since_s\": {} }}",
        crate::debug::debug_on(),
        json_string(&pipeline.debug_text()),
        num_f(perf.frame_ms),
        num_f(perf.worst_ms),
        num_f(perf.budget_ms),
        num_f(perf.key_px_ms),
        num_u(perf.redraws),
        perf.still,
        autosave_state,
        autosave_since_s,
    )
}

fn whichkey_json(pipeline: &TextPipeline) -> String {
    match pipeline.whichkey_report() {
        Some(rows) => {
            let items = rows
                .iter()
                .map(|(k, n)| format!("[{}, {}]", json_string(k), json_string(n)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ \"shown\": true, \"rows\": [{items}] }}")
        }
        None => "{ \"shown\": false, \"rows\": [] }".to_string(),
    }
}

fn hud_json(pipeline: &TextPipeline) -> String {
    let hud = pipeline.hud_report();
    let hud_words = match hud.words {
        Some((w, m, unit)) => format!(
            "\"words\": {w}, \"reading_min\": {m}, \"unit\": {}",
            json_string(unit.tag())
        ),
        None => "\"words\": null, \"reading_min\": null, \"unit\": null".to_string(),
    };
    let lang = match hud.lang {
        Some(l) => json_string(l.code()),
        None => "null".to_string(),
    };
    format!(
        "{{ \"held\": {}, {}, \"percent\": {}, \"lang\": {}, \"eol\": {}, \"saved\": {} }}",
        hud.held,
        hud_words,
        hud.percent,
        lang,
        json_string(hud.eol.label()),
        json_string(&hud.saved),
    )
}

fn lifetime_json(pipeline: &TextPipeline) -> String {
    let l = pipeline.lifetime_report();
    format!(
        "{{ \"open\": {}, \"characters\": {}, \"time_writing\": {}, \"files_touched\": {}, \"caret_travel\": {}, \"your_world\": {} }}",
        l.open,
        json_string(&l.chars),
        json_string(&l.writing),
        json_string(&l.files),
        json_string(&l.caret_travel),
        json_string(&l.world),
    )
}

fn about_json(pipeline: &TextPipeline) -> String {
    let checked = crate::updates::checked_line(pipeline.hud_update_checked())
        .map(|s| json_string(&s))
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{ \"open\": {}, \"checked\": {}, \"pending_crash\": {} }}",
        crate::about::about_open(),
        checked,
        pipeline.hud_pending_crash()
    )
}

fn streaks_json(pipeline: &TextPipeline) -> String {
    let s = pipeline.streaks_report();
    let cells = s
        .cells
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{ \"open\": {}, \"view\": \"{}\", \"streak\": {}, \"today_words\": {}, \"total_words\": {}, \"cells\": [{}] }}",
        s.open, s.view, s.streak, s.today_words, s.total_words, cells
    )
}

fn peek_json(pipeline: &TextPipeline) -> String {
    let p = pipeline.peek_report();
    let rows = p
        .rows
        .iter()
        .map(|r| {
            format!(
                "{{ \"chord\": {}, \"name\": {} }}",
                json_string(&r.chord),
                json_string(&r.name)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ \"open\": {}, \"rows\": [{}] }}", p.open, rows)
}

fn caret_preview_json(pipeline: &TextPipeline) -> String {
    match pipeline.caret_preview_panel_report() {
        Some((rect, text, beat, silhouette)) => format!(
            "{{ \"rect\": [{}, {}, {}, {}], \"text\": {}, \"beat\": {}, \"silhouette\": {} }}",
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            json_string(&text),
            beat,
            silhouette,
        ),
        None => "null".to_string(),
    }
}

fn gutter_json(pipeline: &TextPipeline) -> String {
    let (gutter_visible, gutter_name, gutter_project, gutter_changed) =
        match pipeline.gutter_report() {
            Some((name, project, changed)) => (true, name, project, changed),
            None => (false, String::new(), String::new(), false),
        };
    format!(
        "{{ \"visible\": {}, \"name\": {}, \"project\": {}, \"changed\": {} }}",
        gutter_visible,
        json_string(&gutter_name),
        json_string(&gutter_project),
        gutter_changed,
    )
}

fn caret_block(caret: Option<&CaretFrame>) -> (String, String) {
    match caret {
        Some(c) => {
            let (schema, trail_extra) = match &c.trail {
                Some(tr) => (
                    schema_held(),
                    format!(
                        ", \"trail\": {{ \"holding\": {h}, \"length\": {len}, \"tail\": {{ \"x\": {tlx}, \"y\": {tly} }}, \"head\": {{ \"x\": {hdx}, \"y\": {hdy} }} }}",
                        h = tr.holding,
                        len = tr.length,
                        tlx = tr.tail.0,
                        tly = tr.tail.1,
                        hdx = tr.head.0,
                        hdy = tr.head.1,
                    ),
                ),
                None => (schema_timeline(), String::new()),
            };
            let co = &c.cosmetic;
            let cosmetic_extra = format!(
                ", \"cosmetic_trail\": {{ \"present\": {pr}, \"length\": {len}, \"direction\": {dir}, \"held\": {hd}, \"alpha\": {al}, \"sweep\": {sw}, \"tail\": {{ \"x\": {tlx}, \"y\": {tly} }}, \"head\": {{ \"x\": {hdx}, \"y\": {hdy} }} }}",
                pr = co.present,
                len = co.length,
                dir = json_string(if co.vertical {
                    "vertical"
                } else {
                    "horizontal"
                }),
                hd = co.held,
                al = co.alpha,
                sw = co.sweep,
                tlx = co.tail.0,
                tly = co.tail.1,
                hdx = co.head.0,
                hdy = co.head.1,
            );
            (
                schema,
                format!(
                    ",\n  \"caret\": {{ \"t_ms\": {t}, \"pos\": {{ \"x\": {px}, \"y\": {py} }}, \"target\": {{ \"x\": {tx}, \"y\": {ty} }}, \"settle_factor\": {sf}, \"animating\": {an}, \"pop_scale\": {ps}, \"block\": {{ \"w\": {bw}, \"h\": {bh} }}{trail_extra}{cosmetic_extra} }}",
                    t = c.t_ms,
                    px = c.pos.0,
                    py = c.pos.1,
                    tx = c.target.0,
                    ty = c.target.1,
                    sf = c.settle,
                    an = c.animating,
                    ps = c.scale,
                    bw = c.block_w,
                    bh = c.block_h,
                    trail_extra = trail_extra,
                    cosmetic_extra = cosmetic_extra,
                ),
            )
        }
        None => (schema_plain(), String::new()),
    }
}

pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
