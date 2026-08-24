//! Reconfiguration methods; pure layout and hit-test math lives in [`super::geometry`].
use super::*;
impl TextPipeline {
    /// Re-tint every baked GPU pipeline from the ACTIVE theme AND, when the new
    /// world's effective display face differs from the one the document is shaped
    /// in, RESHAPE the whole document in the new family (the expensive half —
    /// see [`Self::sync_theme_colors`] for the split). Call this after switching
    /// the active theme; the next `prepare` re-uploads.
    pub fn sync_theme(&mut self) {
        self.sync_theme_colors();
        self.sync_theme_font(ShapeReach::Whole);
    }

    /// The O(1) COLOR half of a theme switch: re-tint the baked GPU pipelines
    /// from the ACTIVE theme, touching NO text shaping. The clear color and text
    /// inks read the active theme directly each frame, so this only needs to
    /// update the pipelines that cached a color at construction.
    ///
    /// Split so the theme-picker preview can re-color per arrow while DEFERRING
    /// font reshape until the selection settles — the theme-burst profile
    /// showed the reshape (plus
    /// the following frame's new-face prepare) dominating every preview step,
    /// while this half is microseconds. Every settled path (commit, revert,
    /// capture, tests) still goes through [`Self::sync_theme`], which runs both.
    pub fn sync_theme_colors(&mut self) {
        self.caret_pipeline.set_color(theme::primary().rgb_bytes());
        self.caret_trail_pipeline
            .set_color(theme::primary().rgb_bytes());
        self.caret_glyph_pipeline
            .set_color(theme::primary().rgb_bytes());
        self.selection_pipeline
            .set_color(theme::selection_document().rgba_bytes());
        self.sync_two_colour_pipelines();
        // Search matches: `theme::selection_document()` on an ordinary world, THE ONE
        // WAGTAIL HIGHLIGHT TEXTURE's pure white + dither density on a
        // one-bit world — see `search_match_rgba_bytes`/`wagtail_dither_density`.
        // A switch AWAY from a one-bit world must reset the density back to
        // `0.0`, not merely leave it stale, so both calls run unconditionally
        // every re-tint.
        self.match_pipeline.set_color(search_match_rgba_bytes());
        self.match_pipeline.set_dither(wagtail_dither_density());
        self.match_pipeline
            .set_dither_cell(wagtail_stipple_cell_px(self.dpi));
        self.wash_comment_pipeline
            .set_color(wash_rgba_bytes(crate::syntax::SynKind::Comment));
        self.wash_string_pipeline
            .set_color(wash_rgba_bytes(crate::syntax::SynKind::Str));
        self.wash_highlight_pipeline
            .set_color(highlight_wash_rgba_bytes());
        self.wash_highlight_pipeline
            .set_dither(wagtail_dither_density());
        self.wash_highlight_pipeline
            .set_dither_cell(wagtail_stipple_cell_px(self.dpi));
        // WYSIWYG value-step panel/pill: re-tint from `base_200` (O(1) — geometry
        // is theme-independent, so a theme switch re-tints without rebuilding).
        self.fence_panel_pipeline
            .set_color(theme::base_200().rgba_bytes());
        self.code_pill_pipeline
            .set_color(theme::base_200().rgba_bytes());
        // INLINE IMAGES: the calm missing-file placeholder quad re-tints from
        // `base_200` (O(1); the placeholder GEOMETRY is theme-independent, so the
        // picker preview re-tints for free). The placeholder label rides `muted`,
        // re-read at prepare time; the image textures are theme-independent.
        self.image_placeholder_pipeline
            .set_color(theme::base_200().rgba_bytes());
        // INLINE IMAGES: the caption scrim re-tints from the world's own GROUND
        // (`base_100`, part-alpha) — O(1), geometry theme-independent, so the picker
        // preview re-tints for free.
        self.image_scrim_pipeline
            .set_color(theme::image_reveal_scrim().rgba_bytes());
        // INLINE-IMAGE resize-handle hover grip: `muted`, the same
        // quiet-affordance ink the Chips ghost pills and the fold chevron
        // both spend — never `primary` (DESIGN's one accent stays the caret).
        self.image_handle_mark
            .set_color(theme::muted().rgba_bytes());
        self.table_rule_pipeline
            .set_color(theme::muted().rgba_bytes());
        self.fold_chevron_pipeline
            .set_color(theme::fold_afford_chevron_ink().rgba_bytes());
        self.panel_card
            .set_color(theme::pane_surface(effective_card_elevation()).rgba_bytes());
        self.panel_shadow.set_color(float_shadow_srgba());
        self.panel_border
            .set_color(theme::surface_selected().rgba_bytes());
        self.hud_shadow.set_color(float_shadow_srgba());
        self.hud_border
            .set_color(theme::surface_selected().rgba_bytes());
        self.hud_card.set_color(theme::base_300().rgba_bytes());
        self.wk_shadow.set_color(float_shadow_srgba());
        self.wk_border
            .set_color(theme::surface_selected().rgba_bytes());
        self.wk_card.set_color(theme::base_300().rgba_bytes());
        // FORMAT POPOVER's active-button wash re-tints with the world (a `base_200`
        // value step, never amber). O(1); geometry is theme-independent. The
        // popover's ELEVATION trio is the shared `float_*` pipelines (re-tinted
        // below, alongside the caret-preview panel / spell popup / search panel
        // that already ride them) — no dedicated popover elevation tokens anymore.
        self.popover_wash.set_color(theme::base_200().rgba_bytes());
        self.popover_hl_wash.set_color(highlight_wash_rgba_bytes());
        self.popover_hl_wash.set_dither(wagtail_dither_density());
        self.popover_hl_wash
            .set_dither_cell(wagtail_stipple_cell_px(self.dpi));
        self.popover_strike.set_color(strike_srgba_bytes());
        // FORMAT POPOVER hover ring: `muted`, the SAME hairline
        // stroke weight the Chips ghost pills ride (`BAR_OUTLINE_STROKE`) —
        // never a fill, so it reads whether or not the button underneath also
        // carries the active-state `base_200` wash. Set once (this pipeline
        // never switches mode).
        self.popover_hover_ring
            .set_color(theme::muted().rgba_bytes());
        self.popover_hover_ring
            .set_stroke(self.metrics.px(BAR_OUTLINE_STROKE));
        // WEB/LINUX MENU BAR: re-tint from the world's own tokens (O(1) — the bar/
        // dropdown GEOMETRY is theme-independent, so the theme-picker preview re-tints
        // it for free). Bar ground = a value step off the room (`base_200`); the open
        // title's highlight + the dropdown border = `surface_selected`; the dropdown
        // card = `base_300` (risen a step); the separator hairline = `muted`. NEVER
        // amber — figure/ground by value only (DESIGN §3/§4). The title/item text ink
        // (faint / muted / content) is re-read live at prepare time.
        self.menubar_bg.set_color(theme::base_200().rgba_bytes());
        self.menubar_hi
            .set_color(theme::selection_document().rgba_bytes());
        self.menu_drop_shadow.set_color(float_shadow_srgba());
        self.menu_drop_border
            .set_color(theme::surface_selected().rgba_bytes());
        self.menu_drop_card
            .set_color(theme::base_300().rgba_bytes());
        self.menu_drop_sep.set_color(theme::muted().rgba_bytes());
        self.panel_caret.set_color(theme::primary().rgb_bytes());
        self.caret_preview_pipeline
            .set_color(theme::primary().rgb_bytes());
        self.caret_preview_glyph_pipeline
            .set_color(theme::primary().rgb_bytes());
        self.float_shadow.set_color(float_shadow_srgba());
        self.float_border
            .set_color(theme::surface_selected().rgba_bytes());
        self.float_card.set_color(theme::base_300().rgba_bytes());
        self.overlay_rows
            .set_color(theme::selection_ui().rgba_bytes());
        self.overlay_bars
            .set_color(theme::surface_selected().rgba_bytes());
        self.overlay_spine.set_color(theme::muted().rgba_bytes());
        self.overlay_cross
            .set_color(theme::overlay_band_overlap().rgba_bytes());
        self.overlay_range_track
            .set_color(theme::faint().rgba_bytes());
        self.overlay_range_thumb
            .set_color(theme::muted().rgba_bytes());
        self.overlay_lens_underline
            .set_color(theme::base_content().rgba_bytes());
        self.spell_pipeline.set_color(theme::error().rgba_bytes());
        self.nit_pipeline.set_color(nit_underline_srgba());
        self.strike_pipeline.set_color(strike_srgba_bytes());
        self.link_underline_pipeline
            .set_color(link_underline_srgba_bytes());
        self.background_pipeline.set_gradient(background_desc());
        self.page_frame_pipeline
            .set_color(theme::page_frame_ink().rgba_bytes());
        self.placard_stipple
            .set_color(theme::placard_ink(theme::PlacardInk::Stipple).rgba_bytes());
        self.placard_stipple
            .set_dither(theme::placard_stipple_density());
    }

    /// Does the document carry any per-span text color that was BAKED from the
    /// theme palette and would go stale on a same-face world hop? Only such spans
    /// need the theme-driven re-bake: SYNTAX role tints and markdown MARKUP dim/style
    /// spans. Plain prose body text sets NO
    /// `color_opt` ([`Self::doc_attrs`]) and reads the live active ink each frame,
    /// so a color-less buffer must NOT pay a wasted reshape on a same-face switch.
    pub(super) fn has_baked_theme_colors(&self) -> bool {
        !self.syn_spans.is_empty() || !self.md_spans.is_empty()
    }

    /// Would [`Self::sync_theme_font`] actually re-shape — because the ACTIVE
    /// world's effective display face differs from the one the document is shaped
    /// in, OR its palette differs from the one the per-span colors were baked under
    /// ([`Self::shaped_theme`]) AND the document actually carries baked color spans?
    /// A restyle re-bakes BOTH the glyph shapes and the syntax/markdown span
    /// colors, so a same-FACE world hop still needs it when the palette changed on a
    /// buffer that bakes colors (else stale colors — the Magpie -> Bombora bug); a
    /// color-less prose buffer stays free (its ink reads live). Lets the live preview
    /// arm its settle-deferral only when a real restyle is pending.
    pub fn needs_theme_reshape(&self) -> bool {
        self.doc_family() != self.shaped_font
            || (theme::active_index() != self.shaped_theme && self.has_baked_theme_colors())
    }

    /// Apply the editor view snapshot: text, cursor, scroll, zoom, selection,
    /// preedit. When a preedit (IME composition) is active it is spliced into the
    /// shaped text at the cursor so it renders with real glyphs; the caret is then
    /// placed at the preedit's end and an underline is drawn beneath it.
    pub fn set_view(&mut self, view: &ViewState) {
        // The diagonal cluster is measured from the current overlay's shaped
        // labels and controls. A new view invalidates that measurement before
        // the next overlay preparation can publish a replacement.
        self.diagonal_cluster = None;
        // Apply zoom first: if it changed, reset the glyphon buffer metrics and
        // re-shape so glyph layout matches the zoomed caret + selection rects. The
        // metrics fold in the display DPI (`self.dpi`, set by `set_dpi`) on top of
        // the user zoom, so the live page scales correctly on a HiDPI screen.
        let new_metrics = Metrics::with_dpi(view.zoom, self.dpi);
        let zoom_changed = (new_metrics.font_size - self.metrics.font_size).abs() > f32::EPSILON;
        self.metrics = new_metrics;
        if zoom_changed {
            self.buffer
                .set_metrics(&mut self.font_system, self.metrics.glyph_metrics());
            // The shaping height budget is in (zoomed) pixels, so a zoom change
            // must re-grow the buffer's shaping height to keep the WHOLE document
            // shaped (fewer rows fit per pixel at higher zoom). The wrap width is
            // recomputed from the PAGE-MODE column: zoom changed the glyph advance,
            // so a measure-derived column is wider/narrower in px and must re-wrap.
            let width = Some(self.text_wrap_width());
            let shape_h = self.full_shape_height();
            self.buffer
                .set_size(&mut self.font_system, width, Some(shape_h));
            // Row geometry is in (zoomed) line-height units, so the cached
            // total-visual-row count is stale after a zoom change.
            self.row_geom.invalidate();
        }
        let cursor_moved =
            view.cursor_line != self.cursor_line || view.cursor_col != self.cursor_col;
        let from_key = if cursor_moved {
            self.caret_inhabited_key()
        } else {
            self.caret_from_key
        };
        self.cursor_line = view.cursor_line;
        self.cursor_col = view.cursor_col;
        self.caret_affinity = view.caret_affinity;
        self.caret_from_key = from_key;
        // Re-latch the effective caret LOOK for this frame (see the field doc):
        // the anchor geometry below — including the spring target — reads the
        // latched value, one global read per frame. A live text-selection DRAG
        // overrides the configured look to the thin insertion BAR (the I-beam
        // form) for the duration of the drag — the drag bar. This is the
        // ONE seam that resolves the effective look, so every reader (geometry
        // AND the paint path, which read `self.caret_look`) sees the same form.
        self.caret_look = if view.selecting_drag {
            CaretMode::Ibeam
        } else {
            crate::caret::mode()
        };
        self.sync_view_fields(view);
        let md_changed = self.md_enabled != view.is_markdown;
        self.md_enabled = view.is_markdown;
        let syn_changed = self.syn_lang != view.syn_lang;
        self.syn_lang = view.syn_lang;
        let wysiwyg_changed = self.wysiwyg_latched != crate::markdown::wysiwyg_on();
        self.wysiwyg_latched = crate::markdown::wysiwyg_on();
        let inline_images_changed =
            self.inline_images_latched != crate::markdown::inline_images_on();
        self.inline_images_latched = crate::markdown::inline_images_on();
        let image_preview_dirty = std::mem::take(&mut self.image_preview_dirty);
        let render_flag_changed = wysiwyg_changed || inline_images_changed || image_preview_dirty;
        self.cjk_priority = view.cjk_priority.clone();
        // Shape the document text with any active preedit spliced in at the cursor.
        // This is the ONE place a reshape may happen; it is skipped when neither the
        // composed (text+preedit) string NOR the zoom changed, so cursor moves,
        // scrolling, selection changes, and spell-span refreshes are all free.
        let reshape_before = self.reshape_count;
        self.shape_with_preedit(
            &view.text,
            zoom_changed || md_changed || syn_changed || render_flag_changed,
        );
        // Did a reshape actually happen this push? (A text edit reshapes; a pure
        // cursor move / scroll / selection change does not.) Feeds the
        // reveal-on-cursor conceal rescan below, which a reshape must force since it
        // drops the per-line attrs.
        let reshaped = self.reshape_count != reshape_before;
        // HEADING SIZE: heading rows carry absolute per-span metrics, so we must
        // rebuild line attrs in two cases the incremental text path can't catch on
        // its own: (1) a ZOOM/DPI change rescales the body but not the absolute
        // heading metrics (gated to a heading doc so the common path pays nothing);
        // (2) the markdown gate FLIPPED on UNCHANGED text (the diff rebuilds no
        // lines, so stale md/heading attrs would linger).
        //
        // This MUST run before `set_caret_target` below (see the bug it fixed): the
        // caret's row-geometry reads (`cursor_row_height`/`caret_cell_top`, via
        // `visual_rows`/`row_geom`) walk the buffer's CURRENTLY-shaped runs, and on
        // a heading doc those runs are briefly INCONSISTENT right after
        // `shape_with_preedit` — body text reshaped at the new zoom, but the
        // heading line's absolute per-span pixel metrics are still the OLD size
        // until this restyle rescales them. Latching the caret's spring target
        // from that transient state (the old ordering) left the caret floating at
        // the heading row's PRE-zoom position, never catching up once the text
        // re-laid moments later — the amber block caret drifting off the glyphs on
        // a zoomed heading line. Computing the target AFTER the restyle reads the
        // one, final, settled geometry.
        let restyled = if md_changed
            || syn_changed
            || render_flag_changed
            || (zoom_changed && self.has_heading_lines())
        {
            self.restyle_all_lines();
            true
        } else {
            false
        };
        // WYSIWYG v1.1: a reveal/conceal toggle can change actual glyph GEOMETRY
        // now (the zero-width metrics override), not just color, so this MUST
        // also run before `set_caret_target` below — the EXACT same ordering bug
        // `restyled` above was already moved earlier to avoid: a pure cursor move
        // onto/off a concealable line (heading/emphasis/code/highlight) reshapes
        // that line's glyphs, and latching the caret's spring target from the
        // stale PRE-toggle geometry (the old ordering) would leave the caret one
        // step behind the just-revealed/concealed row until some unrelated event
        // caught it up. Calling it here settles the geometry first.
        self.refresh_rule_conceal(reshaped || restyled);
        self.set_caret_target(view.is_edit_move, view.held);
    }

    pub fn set_hover_line(&mut self, line: Option<usize>) -> bool {
        if self.hover_line == line {
            return false;
        }
        self.hover_line = line;
        true
    }

    /// Copy the plain (non-metric, non-caret-latch) editor view fields — scroll,
    /// selection/preedit, spell, search, overlay, and project status — into the
    /// renderer's mirror of the view snapshot.
    fn sync_view_fields(&mut self, view: &ViewState) {
        self.scroll = view.scroll;
        self.image_base_dir = view.doc_dir.clone();
        self.selection = view.selection;
        self.fold_tails = view.fold_tails.clone();
        self.folded_headings = view.folded_headings.clone();
        self.doc_source = view.doc_source.clone();
        self.preedit = view.preedit.clone();
        // Mirror the spell list ONLY when it actually changed (a rescan landing),
        // bumping its version so the cached squiggle protos rebuild; the common
        // cursor-move / scroll event keeps the mirror, the clone, AND the cache.
        if self.misspelled != view.misspelled {
            self.misspelled = view.misspelled.clone();
            self.spell_gen = self.spell_gen.wrapping_add(1);
        }
        self.search_active = view.search_active;
        self.search_matches = view.search_matches.clone();
        self.search_query = view.search_query.clone();
        self.search_current = view.search_current;
        self.search_case_sensitive = view.search_case_sensitive;
        self.search_replace_active = view.search_replace_active;
        self.search_replacement = view.search_replacement.clone();
        self.search_editing_replacement = view.search_editing_replacement;
        self.search_query_caret = view.search_query_caret;
        self.search_replacement_caret = view.search_replacement_caret;
        self.popover_model = view.popover.clone();
        // A summoned overlay appears + disappears INSTANTLY (no rise-in / sink-out
        // motion) on every CALM world: the overlay content syncs verbatim from the
        // view every frame, so a close snaps the card off the frame the App clears
        // its logical `self.overlay`. THE ONE exception is the MOTION-JUICE
        // entrance (FIRETAIL-MAXIMALIST-SHOWCASE round): on an OPEN flip
        // (false→true), a live-armed pipeline whose effective `MotionJuice`
        // asks for `SpringIn` kicks the ~200ms drop-in spring. Every headless
        // pipeline is unarmed (`juice_live` false — see `arm_live_juice`), so
        // this branch is STRUCTURALLY unreachable in a capture and the settled
        // state stays byte-identical; Reduce Motion folds the kick on the very
        // next step (`step_overlay_juice`). A CLOSE flip resets both animators
        // to settled so a stale mid-flight state can never greet a re-summon.
        let overlay_opened = view.overlay_active && !self.overlay_active;
        let overlay_closed = !view.overlay_active && self.overlay_active;
        self.overlay_active = view.overlay_active;
        self.overlay_align = view.overlay_align;
        if overlay_opened
            && self.juice_live
            && !crate::motion::reduced()
            && crate::render::effective_motion_juice().entrance == theme::OverlayEntrance::SpringIn
        {
            self.overlay_enter_t = 0.0;
        }
        if overlay_closed {
            self.overlay_enter_t = 1.0;
            self.overlay_band_t = 1.0;
            self.overlay_band_last = None;
            self.overlay_band_started_at = None;
            self.overlay_band_frame_now = None;
            self.overlay_band_pending_at = None;
        }
        self.overlay_crisp = view.overlay_crisp;
        self.overlay_query = view.overlay_query.clone();
        self.overlay_query_caret = view.overlay_query_caret;
        self.overlay_title = view.overlay_title;
        self.overlay_row_path_splits = view.overlay_row_path_splits;
        self.overlay_items = view.overlay_items.clone();
        self.overlay_empty = view.overlay_empty.clone();
        self.overlay_bindings = view.overlay_bindings.clone();
        self.overlay_ranges = view.overlay_ranges.clone();
        self.overlay_times = view.overlay_times.clone();
        self.overlay_git = view.overlay_git.clone();
        self.overlay_selected = view.overlay_selected;
        self.overlay_scroll = view.overlay_scroll;
        self.overlay_window_rows = view.overlay_window_rows;
        self.overlay_hint = view.overlay_hint.clone();
        self.overlay_lens = view.overlay_lens.clone();
        self.overlay_workspace = view.overlay_workspace;
        self.overlay_rows_primary = view.overlay_rows_primary;
        self.overlay_comparison = view.overlay_comparison;
        self.overlay_sections = view.overlay_sections.clone();
        self.overlay_location = view.overlay_location.clone();
        self.overlay_spell = view.overlay_spell;
        self.overlay_context_anchor = view.overlay_context_anchor;
        self.overlay_detail_focus = view.overlay_detail_focus;
        self.overlay_spell_w = if self.overlay_spell.is_some() {
            self.measure_spell_content_w()
        } else {
            0.0
        };
        // A RIGHT-ANCHORED takeover card shrinks to hug its content, so
        // measure the widest visible primary (+ optional secondary column, query
        // line, lens strip and footer) NOW, with a `&mut FontSystem` in hand. Gated
        // to the right-anchored takeover cards (the frozen anchor mirrors growth):
        // a left/center card, the contextual spell popup, or a closed overlay leaves
        // the cache `0.0`, so `overlay_desired_w` falls back to the fixed wide cap —
        // byte-identical. Reset FIRST so the provisional geometry the measurement
        // shapes into uses the wide cap (not last frame's hug width).
        self.overlay_content_w = 0.0;
        if self.overlay_active && self.overlay_spell.is_none() && self.overlay_right_anchored() {
            self.overlay_content_w = self.measure_overlay_content_w();
        }
        // The workspace rail's column is MEASURED, never estimated
        // from a mean character width: its labels are display-face words and the
        // column has to hold the widest of them exactly, because that same number
        // is the rail's clip, its mark rect and its pointer hit band. Same
        // `&mut FontSystem` window as the content-width measurement above; `0.0` for
        // every card that is not a workspace, so the geometry is untouched there.
        self.workspace_primary_w = 0.0;
        if self.overlay_active && self.overlay_is_workspace() {
            self.workspace_primary_w = self.measure_workspace_primary_w();
        }
        self.caret_preview = view.caret_preview;
        match view.caret_preview {
            Some(look) => self.caret_demo.mode = look,
            None => self.caret_demo.reset(),
        }
        self.document_active = view.document_active;
        self.gutter_name = view.gutter_name.clone();
        self.gutter_project = view.gutter_project.clone();
        self.gutter_changed = view.gutter_changed;
        self.gutter_files.clone_from(&view.gutter_files);
        // The awl-drawn menu bar's chord column reads these at prepare time
        // (`chrome::menubar::dropdown`), so a config reload or a live `keymap`
        // flavor toggle reaches the next frame the same way every other synced
        // field does — no separate invalidation needed.
        self.config_keys.clone_from(&view.config_keys);
        self.config_linux_keep.clone_from(&view.config_linux_keep);
        self.notice = view.notice.clone();
        self.notice_kind = view.notice_kind;
        self.eol = view.eol;
    }

    pub fn set_dpi(&mut self, dpi: f32) {
        if (dpi - self.dpi).abs() < f32::EPSILON {
            return;
        }
        self.dpi = dpi;
        let stipple_cell = wagtail_stipple_cell_px(dpi);
        self.match_pipeline.set_dither_cell(stipple_cell);
        self.wash_highlight_pipeline.set_dither_cell(stipple_cell);
        self.popover_hl_wash.set_dither_cell(stipple_cell);
        self.metrics = Metrics::with_dpi(self.metrics.zoom, dpi);
        self.buffer
            .set_metrics(&mut self.font_system, self.metrics.glyph_metrics());
        let width = Some(self.text_wrap_width());
        let shape_h = self.full_shape_height();
        self.buffer
            .set_size(&mut self.font_system, width, Some(shape_h));
        self.row_geom.invalidate();
        // Heading spans and every WYSIWYG conceal carry an ABSOLUTE per-span line-height
        // the relayout above preserves, so the row pitch is stale until the attrs rebuild.
        self.restyle_all_lines();
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        // Width drives soft-wrap (text wraps to the viewport width). We manage
        // vertical scroll ourselves via the draw offset (`doc_top`), so the
        // buffer's own scroll stays at 0 and we never rely on it to clip.
        //
        // The HEIGHT we hand cosmic-text is NOT the window height: cosmic-text
        // only lays out (and yields from `layout_runs()`) the rows that fit in
        // the buffer's height starting at its scroll. To make scrolling, overlay
        // placement, and the total-visual-row count correct for a scrolled or
        // long wrapped document, the WHOLE document must be shaped — so we pass a
        // generous height that covers every visual row. These docs are small, so
        // shaping the whole buffer is cheap. The real window `height` only bounds
        // what we DRAW (via `TextBounds` in `prepare`), not what we shape — we keep it
        // only for the DEBUG panel's `viewport WxH` readout.
        self.window_h = height;
        // Record the real window width FIRST so the column geometry derives from
        // it; then wrap the text at the (possibly narrower, centered) COLUMN width
        // rather than the whole window — that is the centered writing measure.
        self.window_w = width;
        // Remember the buffer's CURRENT wrap size so we can tell whether this call
        // actually re-wraps (cosmic-text no-ops on an unchanged size).
        let before = self.buffer.size();
        let shape_h = self.full_shape_height();
        let wrap_w = self.text_wrap_width();
        self.buffer
            .set_size(&mut self.font_system, Some(wrap_w), Some(shape_h));
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        // A CHANGED wrap size re-laid the document's runs, so every row-geometry
        // cache (row tops/heights/total, the cursor-line VisualRow memo) is stale.
        // This is the LIVE window-resize / page-mode-toggle / page-width seam: the
        // following `prepare`'s `sync_wrap_width` sees the width already in sync and
        // skips its own invalidate, so without this the scroll math, caret row, and
        // hit-tests keep answering from the PRE-RESIZE geometry until the next text
        // edit. (The headless capture sets its size before the text, so this only
        // ever fires on a real geometry change — captures stay byte-identical.)
        let changed = |a: Option<f32>, b: Option<f32>| match (a, b) {
            (Some(x), Some(y)) => (x - y).abs() > 0.5,
            (None, None) => false,
            _ => true,
        };
        if changed(before.0, Some(wrap_w)) || changed(before.1, Some(shape_h)) {
            self.row_geom.invalidate();
        }
        if changed(before.0, Some(wrap_w)) {
            self.resync_table_layout_for_width();
        }
    }

    pub fn line_count(&self) -> usize {
        self.buffer.lines.len()
    }

    /// The effective document font metrics for this frame, in the same physical
    /// pixel coordinate space as [`Self::text_left`], [`Self::column_left`], and
    /// the rendered PNG. The capture sidecar reads this seam instead of the base
    /// constants so its geometry remains composable at every zoom/DPI scale.
    pub fn effective_font_metrics(&self) -> (f32, f32, f32) {
        (
            self.metrics.zoom,
            self.metrics.font_size,
            self.metrics.line_height,
        )
    }
}
