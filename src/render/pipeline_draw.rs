//! GPU construction for [`super::TextPipeline`]. Per-frame preparation and render
//! composition live in their respective pipeline modules.

use super::*;
impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cache: &Cache,
        format: wgpu::TextureFormat,
    ) -> Self {
        // Baked pipeline colours are deliberately placeholders here. The one
        // retint owner, `sync_theme_colors`, runs before this constructor returns.
        const PLACEHOLDER_RGB: [u8; 3] = [0; 3];
        const PLACEHOLDER_RGBA: [u8; 4] = [0; 4];
        let mut font_system = build_font_system();

        let swash_cache = SwashCache::new();
        let viewport = Viewport::new(device, cache);
        let mut atlas = TextAtlas::new(device, queue, cache, format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let metrics = Metrics::new(1.0);
        let buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());

        let caret_pipeline = CaretPipeline::new(device, format, PLACEHOLDER_RGB);
        let caret_trail_pipeline = CaretPipeline::new(device, format, PLACEHOLDER_RGB);
        let caret_glyph_pipeline = CaretGlyphPipeline::new(device, queue, format, PLACEHOLDER_RGB);
        let background_pipeline = BackgroundPipeline::new(device, format, background_desc());
        let lava_pipeline = crate::lava::LavaPipeline::new(device, format);
        let sel_shader = crate::selection::selection_shader(device);
        let mut page_frame_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        page_frame_pipeline.set_dither(1.0);
        let stars_pipeline = SelectionPipeline::new(device, &sel_shader, format, [0, 0, 0, 0]);
        // SYNTAX WASH quads (under selection, over the ground): the warm band
        // behind prose comments + the green band behind dark-world strings. The
        // tints come from THE role style provider (`role_style_for`, via
        // `wash_rgba_bytes`); a role/world with no wash gets transparent bytes AND
        // zero instances, so nothing draws.
        let wash_comment_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let wash_string_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let mut wash_highlight_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        wash_highlight_pipeline.set_dither(wagtail_dither_density());
        wash_highlight_pipeline.set_dither_cell(wagtail_stipple_cell_px(1.0));
        let fence_panel_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let code_pill_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let image_pipeline = crate::image_pipeline::ImageQuadPipeline::new(device, format);
        let image_placeholder_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let image_scrim_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let image_placeholder_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        // Translucent selection highlight quads, drawn under the text. On a
        // one-bit world `prepare_selection_layer` uploads ZERO rects here
        // (the true-inverse-video `selection_invert` pipeline takes over
        // document selection entirely); elsewhere this tracks `selection_document`.
        let selection_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        // Search matches share the one-bit texture with `==highlight==` spans.
        let mut match_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        match_pipeline.set_dither(wagtail_dither_density());
        match_pipeline.set_dither_cell(wagtail_stipple_cell_px(1.0));
        let swap_ground = theme::base_300().rgba_bytes();
        let swap_ink = theme::base_content().rgba_bytes();
        let selection_invert =
            SelectionPipeline::new_two_colour(device, &sel_shader, format, swap_ground, swap_ink);
        // The caret owns a separate two-colour instance buffer.
        let caret_invert =
            SelectionPipeline::new_two_colour(device, &sel_shader, format, swap_ground, swap_ink);
        // Markdown ORNAMENTS (section-break fleuron): a quiet DIM glyph renderer,
        // sharing the atlas + viewport. One single-glyph buffer per break, shaped
        // centered in the writing column. Empty / parked for a non-markdown buffer so
        // a default capture stays byte-identical.
        let ornament_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        // THE FOLD CHEVRON — leaves the glyphon pipeline entirely (it must rotate a
        // quarter turn on fold/unfold; glyphon 0.11 has no transform). Two
        // `spine_segment` arms per mark, uploaded through `prepare_rotated`.
        let fold_chevron_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let table_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let table_rule_pipeline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let panel_card = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let panel_shadow = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let panel_border = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let blur = blur::BlurBackdrop::new(device, format);
        // Second text renderer for the panel string, sharing the atlas + viewport.
        let panel_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let placard_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let panel_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let panel_bind_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        // The summoned workspace's navigation rail shapes into its own
        // buffer: it is a column, not more lines of the card's own list.
        let workspace_rail_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let workspace_hint_measure_buffer =
            GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let placard_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let panel_caret = CaretPipeline::new(device, format, PLACEHOLDER_RGB);
        let caret_preview_pipeline = CaretPipeline::new(device, format, PLACEHOLDER_RGB);
        let caret_preview_glyph_pipeline =
            CaretGlyphPipeline::new(device, queue, format, PLACEHOLDER_RGB);
        let float_shadow = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let float_border = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let float_card = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let preview_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let preview_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        // The overlay's selected-row highlight: same rounded quad as document
        // selection, its OWN token (`selection_ui`, a value step off the surface
        // ramp; amber stays the caret's alone), re-set from that same owner
        // every `overlay_prepare_selection`.
        let overlay_quad = |color| SelectionPipeline::new(device, &sel_shader, format, color);
        let overlay_rows = overlay_quad(PLACEHOLDER_RGBA);
        let overlay_bars = overlay_quad(PLACEHOLDER_RGBA);
        // Seeded with `muted`; `overlay_prepare_selection` re-resolves the ink
        // from the live theme every frame (mirroring `notice_rim`'s seed), so
        // this only has to be a valid colour, never the right one.
        let footer_plate_rim = overlay_quad(theme::muted().rgba_bytes());
        let overlay_spine = overlay_quad(PLACEHOLDER_RGBA);
        let overlay_spine_selected = overlay_quad(theme::base_content().rgba_bytes());
        let overlay_lens_underline =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let overlay_facet_ghost =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let overlay_cross = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let overlay_range_track =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let overlay_range_thumb =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let mut placard_stipple =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        placard_stipple.set_dither(theme::placard_stipple_density());
        // The rotated secondary-location cue. Shares the same rotation
        // shader every world's data can reach; parked (zero instances) until
        // a `RotatedRail` world's frame actually prepares one.
        let rotated_label_pipeline =
            crate::rotated_label::RotatedLabelPipeline::new(device, format);
        let wordcount_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let wordcount_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let notice_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let notice_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        // Seeded with the TOAST plane; `prepare_notice` re-resolves the plane from
        // the live theme (and the notice's kind) on every frame, so this seed only
        // has to be a valid colour, never the right one.
        let notice_plate = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let notice_rim =
            SelectionPipeline::new(device, &sel_shader, format, theme::muted().rgba_bytes());
        let page_drag_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let page_drag_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let zoom_readout_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let zoom_readout_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let debug_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let debug_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let gutter_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let gutter_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let outline_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let outline_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let menubar_bg =
            SelectionPipeline::new(device, &sel_shader, format, theme::base_200().rgba_bytes());
        let menubar_hi = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let menubar_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let menubar_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let menu_drop_shadow =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let menu_drop_border =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let menu_drop_card = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let menu_drop_sep = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let menu_drop_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let menu_drop_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let menu_chord_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let menu_chord_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let hud_shadow = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let hud_border = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let hud_card = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let mut streak_cells = SelectionPipeline::new(
            device,
            &sel_shader,
            format,
            theme::base_content().rgba_bytes(),
        );
        streak_cells.set_corner(1.5);
        let hud_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let hud_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let wk_shadow = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let wk_border = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let wk_card = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let wk_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let wk_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        // FORMAT POPOVER: an active-button value-step wash + a button-label text
        // renderer. Its float-panel ELEVATION rides the shared `float_shadow`/
        // `float_border`/`float_card` quads (`prepare_float_panel`) — no dedicated
        // trio of its own; see `render.rs`'s field doc. Empty/off until a mouse
        // selection summons it (or the `AWL_POPOVER` capture probe).
        let popover_wash = SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        let mut popover_hl_wash =
            SelectionPipeline::new(device, &sel_shader, format, PLACEHOLDER_RGBA);
        popover_hl_wash.set_dither(wagtail_dither_density());
        popover_hl_wash.set_dither_cell(wagtail_stipple_cell_px(1.0));
        let popover_strike = SpellUnderlinePipeline::new(device, format, PLACEHOLDER_RGBA);
        let popover_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let popover_buffer = GlyphBuffer::new(&mut font_system, metrics.glyph_metrics());
        let spell_pipeline = SpellUnderlinePipeline::new(device, format, PLACEHOLDER_RGBA);
        let nit_pipeline = SpellUnderlinePipeline::new(device, format, PLACEHOLDER_RGBA);
        let strike_pipeline = SpellUnderlinePipeline::new(device, format, PLACEHOLDER_RGBA);
        let link_underline_pipeline = SpellUnderlinePipeline::new(device, format, PLACEHOLDER_RGBA);

        let mut me = Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer,
            caret_pipeline,
            caret_trail_pipeline,
            caret_glyph_pipeline,
            caret_mask_to: None,
            caret_mask_from: None,
            caret_from_key: None,
            caret_look: crate::caret::mode(),
            background_pipeline,
            lava_pipeline,
            lava_phase: crate::lava::LAVA_FROZEN_PHASE,
            warp_phase: crate::warpgrid::FROZEN_PHASE,
            lava_field_viewport: [0.0, 0.0],
            frost_seeds: Vec::new(),
            frost_seed_key: None,
            frost_seed_rebuilds: 0,
            stars_pipeline,
            stars_protos: Vec::new(),
            stars_proto_key: None,
            page_frame_pipeline,
            wash_comment_pipeline,
            wash_string_pipeline,
            wash_highlight_pipeline,
            fence_panel_pipeline,
            code_pill_pipeline,
            selection_pipeline,
            match_pipeline,
            selection_invert,
            caret_invert,
            ornament_renderer,
            fold_chevron_pipeline,
            table_renderer,
            table_rule_pipeline,
            panel_card,
            panel_shadow,
            panel_border,
            blur,
            blur_recompute: false,
            blur_sig: None,
            panel_renderer,
            placard_renderer,
            panel_buffer,
            panel_bind_buffer,
            placard_buffer,
            panel_caret,
            caret_preview_pipeline,
            caret_preview_glyph_pipeline,
            float_shadow,
            float_border,
            float_card,
            float_panel_model: None,
            preview_renderer,
            preview_buffer,
            spell_pipeline,
            nit_pipeline,
            strike_pipeline,
            link_underline_pipeline,
            caret: CaretAnim::new(),
            cursor_line: 0,
            cursor_col: 0,
            caret_affinity: crate::caret::Affinity::Downstream,
            scroll: ScrollPos::default(),
            metrics,
            dpi: 1.0,
            window_w: crate::capture::CANVAS_WIDTH as f32,
            window_h: crate::capture::CANVAS_HEIGHT as f32,
            selection: None,
            fold_tails: Vec::new(),
            folded_headings: Vec::new(),
            fold_chevron_turn: std::collections::HashMap::new(),
            hover_line: None,
            preedit: String::new(),
            misspelled: Vec::new(),
            spell_gen: 0,
            shaped_key: None,
            // The first `set_text` (HELLO_TEXT below) shapes with the active
            // theme's font and updates this; seed it to the active font so the
            // tracker is consistent before that first shape.
            shaped_font: theme::active().font,
            shaped_theme: theme::active_index(),
            last_conceal_cursor_line: None,
            last_conceal_selection: None,
            row_geom: rowgeom::RowGeom::new(),
            caret_line_glyphs: std::cell::RefCell::new(None),
            ornament_cache: rects::OrnamentCache::new(),
            table_report: std::cell::RefCell::new(Vec::new()),
            table_pan: None,
            xray: Vec::new(),
            image_base_dir: None,
            image_heights: Vec::new(),
            image_force: Vec::new(),
            image_report: std::cell::RefCell::new(Vec::new()),
            image_preview: None,
            image_preview_dirty: false,
            image_pipeline,
            image_placeholder_pipeline,
            image_scrim_pipeline,
            image_placeholder_renderer,
            #[cfg(not(target_arch = "wasm32"))]
            image_cache: image_cache::ImageCache::default(),
            squiggle_cache: rects::UnderlineCache::new(),
            nit_cache: rects::UnderlineCache::new(),
            wash_cache: rects::WashCache::new(),
            fence_panel_cache: rects::FencePanelCache::new(),
            table_grid_cache: layers::TableGridCache::new(),
            #[cfg(test)]
            last_table_cell_lines: std::cell::RefCell::new(Vec::new()),
            reshape_count: 0,
            shape_tail_owed: false,
            search_active: false,
            search_matches: Vec::new(),
            search_query: String::new(),
            search_current: None,
            search_case_sensitive: false,
            search_replace_active: false,
            search_replacement: String::new(),
            search_editing_replacement: false,
            search_query_caret: usize::MAX,
            search_replacement_caret: usize::MAX,
            overlay_rows,
            overlay_bars,
            footer_plate_rim,
            overlay_spine,
            overlay_spine_selected,
            overlay_lens_underline,
            overlay_facet_ghost,
            overlay_cross,
            overlay_range_track,
            overlay_range_thumb,
            placard_stipple,
            rotated_label_pipeline,
            rotated_location_mask: None,
            overlay_theme_underline: None,
            overlay_theme_facet_ghosts: Vec::new(),
            overlay_strip_tab_plates: Vec::new(),
            overlay_right_shown: false,
            diagonal_cluster: None,
            wordcount_renderer,
            wordcount_buffer,
            notice_renderer,
            notice_buffer,
            notice_plate,
            notice_rim,
            page_drag_renderer,
            page_drag_buffer,
            zoom_readout_renderer,
            zoom_readout_buffer,
            debug_renderer,
            debug_buffer,
            gutter_renderer,
            gutter_buffer,
            outline_renderer,
            outline_buffer,
            menubar_bg,
            menubar_hi,
            menubar_renderer,
            menubar_buffer,
            menu_drop_shadow,
            menu_drop_border,
            menu_drop_card,
            menu_drop_sep,
            menu_drop_renderer,
            menu_drop_buffer,
            menu_chord_renderer,
            menu_chord_buffer,
            menubar_boxes: Vec::new(),
            menubar_bar_h: 0.0,
            menu_drop_rect: None,
            menu_drop_rows: Vec::new(),
            menu_drop_menu: None,
            hud_shadow,
            hud_border,
            hud_card,
            streak_cells,
            hud_renderer,
            hud_buffer,
            wk_shadow,
            wk_border,
            wk_card,
            wk_renderer,
            wk_buffer,
            popover_wash,
            popover_hl_wash,
            popover_strike,
            popover_renderer,
            popover_buffer,
            popover_model: None,
            popover_geom: None,
            hud: HudDefaults::default(),
            streaks_view: None,
            peek_rows: Vec::new(),
            keybindings_tips: Vec::new(),
            wk: WhichKeyDefaults::default(),
            notice: String::new(),
            notice_drawn: String::new(),
            notice_kind: crate::actions::NoticeKind::default(),
            juice_live: false,
            overlay_enter_t: 1.0,
            overlay_band_from: 0.0,
            overlay_band_t: 1.0,
            overlay_band_last: None,
            band_ease_started: false,
            page_drag_readout: None,
            zoom_readout: None,
            debug: DebugDefaults::default(),
            debug_still: true,
            overlay_active: false,
            overlay_align: None,
            overlay_crisp: false,
            overlay_query: String::new(),
            overlay_query_caret: usize::MAX,
            overlay_title: "",
            overlay_row_path_splits: false,
            overlay_items: Vec::new(),
            overlay_empty: None,
            overlay_bindings: Vec::new(),
            overlay_ranges: Vec::new(),
            overlay_times: Vec::new(),
            overlay_git: Vec::new(),
            overlay_selected: 0,
            overlay_scroll: 0,
            overlay_window_rows: 12,
            overlay_hint: String::new(),
            overlay_lens: Vec::new(),
            overlay_sections: Vec::new(),
            overlay_location: None,
            overlay_spell: None,
            overlay_context_anchor: None,
            overlay_detail_focus: false,
            overlay_workspace: false,
            overlay_rows_primary: false,
            overlay_comparison: false,
            workspace_primary_w: 0.0,
            workspace_rail_buffer,
            workspace_hint_measure_buffer,
            workspace_rail_rows: Vec::new(),
            workspace_rail_placement: None,
            overlay_spell_w: 0.0,
            overlay_content_w: 0.0,
            roster_memo: [None; chrome::roster::ROSTER_SLOTS],
            caret_preview: None,
            caret_demo: crate::caret::CaretDemo::new(),
            caret_preview_mask_to: None,
            caret_preview_mask_from: None,
            caret_preview_from_key: None,
            gutter_name: String::new(),
            gutter_project: String::new(),
            gutter_changed: false,
            md_enabled: false,
            wysiwyg_latched: crate::markdown::wysiwyg_on(),
            inline_images_latched: crate::markdown::inline_images_on(),
            md_spans: Vec::new(),
            outline_headings: Vec::new(),
            last_outline_current: None,
            syn_lang: None,
            syn_spans: Vec::new(),
            doc_lang: None,
            doc_source: None,
            cjk_priority: crate::frontmatter::DEFAULT_CJK_PRIORITY.to_vec(),
            eol: crate::buffer::Eol::Lf,
            copy_pulse_t: 1.0,
        };
        me.set_text(HELLO_TEXT);
        me.sync_theme_colors();
        me
    }
}
