use super::*;
mod scroll;

impl App {
    pub(super) fn sync_view(&mut self, follow: bool) {
        if self.gpu.is_none() {
            return;
        }
        self.zoom_reflow.clear();
        let height = self.gpu.as_ref().unwrap().config.height as f32;
        debug_assert!(height.is_finite());
        let (cursor_line, cursor_col) = self.active.buffer.cursor_line_col();
        self.sync_spell_cache();
        // SAVE-FEEDBACK round: the window-title EDITED marker + the native
        // macOS titlebar dot, kept live WITHOUT re-titling every keystroke —
        // `sync_view` already runs on nearly every edit/cursor-move (gated on
        // the gpu-present check above, the cheapest honest hook), so compare
        // against `persistence`'s title cache and only call `update_title` (a
        // string format + a `set_title`/`set_document_edited` OS call) on an
        // ACTUAL clean↔dirty flip.
        if self.persistence.title_cache_stale(self.is_document_dirty()) {
            self.update_title();
        }
        // Schedule a debounced AUTO-SAVE for the active quick note when its text
        // changed. This lives ONLY here (the live windowed path, gated by the
        // gpu-present check above), so the headless capture/replay never auto-writes
        // — the determinism + no-fixture-mutation guarantee. The write fires in
        // `about_to_wait` after a quiet period.
        if self.active.buffer.is_unnamed_fresh()
            && self
                .persistence
                .note_write_owed(self.active.buffer.version())
        {
            let now = self.clock.now();
            self.persistence.arm_note_debounce(now);
        }
        // Arm the DOCUMENT AUTOSAVE idle timer (config-gated, default ON) when a
        // non-note buffer's text changed since its last write — a pathed document
        // tracks `doc_saved_version`, the no-path scratch its stash version.
        // Same determinism guarantee as the note arming above: this lives ONLY
        // under the gpu-present gate, so headless can never schedule a write.
        if self.config.autosave_on() && !self.active.buffer.is_unnamed_fresh() {
            let unsaved = if self.active.buffer.path().is_some() {
                self.active.extra.doc_saved_version != Some(self.active.buffer.version())
            } else {
                self.active.extra.scratch_saved_version != Some(self.active.buffer.version())
            };
            if unsaved {
                self.active.extra.doc_autosave_at = Some(self.clock.now());
            }
        }
        // DIFF-AS-PREVIEW: while the History picker is open, the page below the
        // card shows the WRITER'S DIFF of the current buffer vs the highlighted
        // row's version — derived here, at ViewState-build time, by overriding
        // the pushed text with the marked-up-manuscript transcript (the one owner
        // `crate::history::diff_preview`, cached per id). The BUFFER (its
        // content, version, undo history) is NEVER touched, so Esc just closes
        // the overlay and the next sync pushes the buffer's own text again —
        // "back to now exactly". `None` whenever the picker isn't open / the row
        // is the empty-state one. (The old plain-content preview and the separate
        // Compare takeover both retired into this one surface.)
        let preview = self.history_preview_text();
        // DIFF-AS-PREVIEW scroll: while the diff preview is up, the page shows the
        // OVERLAY's own `diff_scroll` (PgUp/PgDn / panel-focus ↑/↓ / the wheel over
        // the page all mutate it) — and `self.active.extra.scroll`, the DOCUMENT's
        // viewport, is deliberately never touched, so "Esc = back to now exactly"
        // includes the scroll by construction. Clamped against the shaped
        // transcript below (with the clamp written back, so the sidecar reports
        // the honest value).
        let diff_scroll = if preview.is_some() {
            self.workspace_state.overlay().map(|o| o.diff_scroll)
        } else {
            None
        };
        // ROPE-CLONE SHORT-CIRCUIT: reuse the last materialised rope clone while the
        // buffer version is unchanged (see [`Self::view_text`]). A PREVIEW bypasses
        // `view_text` entirely — the version-keyed `sync_text_cache` must never hold
        // a previewed version's bytes (the cache-key discipline).
        let text = match &preview {
            Some(p) => p.clone(),
            None => self.view_text(),
        };
        // The follow branch chases the BUFFER cursor; a preview clamps that cursor
        // into a DIFFERENT text, so arrowing the rows must never scroll-chase it.
        let follow = follow && preview.is_none();

        let version = self.active.buffer.version();
        let streak_override = std::mem::take(&mut self.caret_edit_streaks);
        let is_edit_move = version != self.active.extra.caret_synced_version && !streak_override;
        self.active.extra.caret_synced_version = version;
        // Was the keypress driving this sync an OS auto-repeat (a HELD arrow)?
        // One-shot, like `caret_edit_streaks`: consumed here so a following
        // non-keyboard sync (IME/wheel) doesn't inherit a stale held flag.
        let held = std::mem::take(&mut self.caret_held);

        let popover = if self.workspace_state.popover_holds_attention()
            && crate::popover::popover_on()
            && self.active.buffer.has_selection()
        {
            crate::actions::popover::plan(
                &self.active.buffer.text(),
                self.active.buffer.anchor_char(),
                self.active.buffer.cursor_char(),
                self.active.buffer.is_markdown(),
            )
        } else {
            None
        };

        let (
            search_matches,
            search_current,
            search_query,
            search_active,
            search_case_sensitive,
            search_replace_active,
            search_replacement,
            search_editing_replacement,
            search_query_caret,
            search_replacement_caret,
        ) = self.search_view_fields();

        // KEYED (the completed-word-lag fix's second half): filter the cache down
        // to the verdicts still valid against the text about to be pushed
        // (`spell::visible`, THE ONE reader) — a verdict whose exact word has
        // since changed underneath its span (an edit this same sync just made,
        // or — belt and suspenders — any future path that mutates `spell_cache`
        // without an immediate rescan) can never paint. A DIFF-AS-PREVIEW
        // substitutes a different transcript into `text` above; every verdict
        // stays keyed to the REAL buffer text, so key against that (a cached
        // `view_text()` clone, not a fresh rope walk) rather than the preview
        // transcript it would never match anyway.
        let misspelled = if preview.is_some() {
            let buffer_text = self.view_text();
            crate::spell::visible(&self.active.extra.spell_cache, &buffer_text)
        } else {
            crate::spell::visible(&self.active.extra.spell_cache, &text)
        };

        // The summoned picker, bound ONCE for the whole projection below (item
        // 172): fourteen `overlay_*` fields read it, and asking its owner
        // fourteen times made rustfmt break every one across five lines.
        let ov = self.workspace_state.overlay();
        let mut view = ViewState {
            text,
            cursor_line,
            cursor_col,
            // The caret's wrap affinity (Upstream only right after a visual line-END
            // motion) — the pipeline reads it to render the caret on the row it
            // visually belongs to at a shared soft-wrap boundary.
            caret_affinity: self.active.buffer.affinity(),
            scroll: scroll::resolved_scroll(diff_scroll, self.active.extra.scroll),
            zoom: self.zoom,
            selection: self.active.buffer.selection_line_col(),
            preedit: self.preedit.clone(),
            misspelled,
            is_edit_move,
            held,
            selecting_drag: self.dragging,
            search_matches,
            search_current,
            search_query,
            search_active,
            search_case_sensitive,
            search_replace_active,
            search_replacement,
            search_editing_replacement,
            search_query_caret,
            search_replacement_caret,
            overlay_active: self.workspace_state.overlay_open(),
            // ITEM 45: carry the alignment FROZEN at summon (`OverlayState::align`)
            // straight through — read verbatim every frame, so a live theme-preview
            // crossing never recomputes it and the open card holds its placement.
            overlay_align: ov.map(|o| o.align),
            overlay_crisp: ov
                .map(|o| {
                    matches!(
                        o.kind,
                        crate::overlay::OverlayKind::Theme
                            | crate::overlay::OverlayKind::Caret
                            | crate::overlay::OverlayKind::History
                    )
                })
                .unwrap_or(false),
            overlay_query: ov.map(|o| o.query.text().to_string()).unwrap_or_default(),
            overlay_query_caret: ov.map(|o| o.query.caret()).unwrap_or(0),
            overlay_title: ov
                .filter(|o| o.kind.draws_title_prefix())
                .map(|o| o.kind.title())
                .unwrap_or(""),
            overlay_row_path_splits: ov.map(|o| o.kind.row_path_splits()).unwrap_or(false),
            overlay_items: ov.map(|o| o.item_strings()).unwrap_or_default(),
            // EMPTY STATE: the shared calm message when the overlay has no rows (empty
            // corpus / query matched nothing); `None` when there are rows or no overlay.
            overlay_empty: ov.and_then(|o| o.empty_notice()),
            overlay_bindings: ov.map(|o| o.item_bindings()).unwrap_or_default(),
            overlay_ranges: ov.map(|o| o.item_range_fracs()).unwrap_or_default(),
            overlay_times: ov.map(|o| o.item_times()).unwrap_or_default(),
            overlay_git: ov.map(|o| o.item_git_tags()).unwrap_or_default(),
            overlay_selected: ov.map(|o| o.selected).unwrap_or(0),
            overlay_scroll: ov.map(|o| o.scroll).unwrap_or(0),
            // The per-kind visible-row cap (item 64's MAX_SUGGESTIONS + 1 for spell /
            // 12 flat+faceted / more for theme), the ONE owner the pipeline windows
            // against so the drawn rows match the hover/keyboard item-window exactly.
            overlay_window_rows: ov.map(|o| o.window_rows()).unwrap_or(12),
            overlay_hint: ov.map(|o| o.foot_hint()).unwrap_or_default(),
            overlay_lens: ov.map(|o| o.lens_strip()).unwrap_or_default(),
            overlay_sections: ov.map(|o| o.item_sections()).unwrap_or_default(),
            caret_preview: ov
                .filter(|o| o.kind == crate::overlay::OverlayKind::Caret)
                .and_then(|o| o.selected_caret_mode()),
            gutter_name: self.active.buffer.display_name(),
            gutter_project: self.project.name.clone(),
            is_markdown: self.active.buffer.is_markdown(),
            doc_dir: self
                .active
                .buffer
                .path()
                .and_then(|p| p.parent())
                .map(|d| d.to_path_buf()),
            syn_lang: self.active.buffer.syntax_lang(),
            overlay_spell: ov
                .filter(|o| o.kind == crate::overlay::OverlayKind::Spell)
                .and_then(|o| o.spell_target),
            notice: self.notice.clone().unwrap_or_default(),
            cjk_priority: self.config.cjk_priority_or_default(),
            eol: self.active.buffer.eol(),
            popover,
            diff_panel: preview.is_some(),
            diff_panel_focus: ov.map(|o| o.detail_focus).unwrap_or(false),
            folds: Vec::new(),
            fold_tails: Vec::new(),
        };
        // HISTORY PREVIEW geometry safety: the pushed text is a DIFFERENT (possibly
        // shorter) version than the buffer, so every field whose line/col spans
        // index the BUFFER text must be re-bounded or cleared — the cursor clamps
        // into the previewed text (the shared `clamp_line_col`); selection /
        // preedit / squiggles / search highlights are dropped for the preview's
        // duration (they'd misalign, or panic in the glyph-span layer). All
        // restored automatically on close: the next sync rebuilds them from the
        // untouched buffer.
        if preview.is_some() {
            // DIFF-AS-PREVIEW: park the caret on the transcript's blank line 1
            // (between the `# title` and the first diff block) so NO line's WYSIWYG
            // conceal reveals — the reveal is caret-line-scoped and line 1 carries no
            // markup, so the title's `#` and every `==`/`>`/strike marker stay
            // concealed: the clean marked-up manuscript, never a revealed-raw line.
            // The ONE reveal-suppression rule, shared with the `AWL_DIFF_*` capture
            // harness (`main/run.rs` parks the same way), so live == capture.
            let (dl, dc) = crate::history::clamp_line_col(&view.text, 1, 0);
            view.cursor_line = dl;
            view.cursor_col = dc;
            view.selection = None;
            view.preedit = String::new();
            view.misspelled = Vec::new();
            view.search_matches = Vec::new();
            view.search_current = None;
            view.search_query = String::new();
            view.search_active = false;
            view.search_case_sensitive = false;
            view.search_replace_active = false;
            view.search_replacement = String::new();
            view.search_editing_replacement = false;
            view.search_query_caret = 0;
            view.search_replacement_caret = 0;
            // A history preview shows a DIFFERENT version's text; the popover's
            // spans would index the wrong bytes, so it never rides a preview frame.
            view.popover = None;
        }
        // FOLDS: collapse the folded sections out of the shaped text. `view.text`
        // is the full document; drop the hidden lines and remap the caret /
        // selection / search / spell coordinates into the filtered space the
        // pipeline shapes — a hidden line is never laid out, so it contributes ZERO
        // height. Recorded (unfiltered) for the sidecar. A no-op when nothing is
        // folded (byte-identical) and skipped during a history preview (its
        // substitute transcript owns the text). The action-seam auto-expand keeps
        // the caret + any selection on visible lines.
        view.folds = self.active.buffer.folds().iter().copied().collect();
        if preview.is_none() && self.active.buffer.has_folds() {
            crate::fold::apply_to_view(
                &mut view,
                &self.active.buffer.hidden_lines(),
                &self.active.buffer.fold_tails(),
            );
        }
        {
            let gpu = self.gpu.as_mut().unwrap();
            gpu.pipeline.set_view(&view);
        }

        let prev_scroll = self.active.extra.scroll;
        if let Some(anchor) = self.zoom_anchor.take() {
            // ZOOM ANCHOR wins this sync: this `set_view` just reshaped to the newly
            // changed zoom, so re-solve the scroll that keeps the anchored document
            // point at its captured screen y (the ONE owner does the variable-row
            // math + clamp). Overrides cursor-follow — the anchored caret is on
            // screen by construction, and the off-screen fallback deliberately holds
            // the viewport centre rather than yanking to the caret.
            let pipeline = &self.gpu.as_ref().unwrap().pipeline;
            self.active.extra.scroll =
                pipeline.zoom_anchor_scroll_pos(anchor.line, anchor.col, anchor.screen_y, height);
        } else if follow {
            let pipeline = &self.gpu.as_ref().unwrap().pipeline;
            // Affinity resolves shared boundaries to the caret's visual row.
            let cursor_row =
                pipeline.visual_row_of_aff(cursor_line, cursor_col, self.active.buffer.affinity());
            self.active.extra.scroll = match crate::view_policy::follow_scroll_strategy(
                crate::typewriter::typewriter_on(),
                self.dragging,
            ) {
                crate::view_policy::FollowScroll::ShowRow => {
                    pipeline.scroll_to_show_row_pos(cursor_row, self.active.extra.scroll, height)
                }
                crate::view_policy::FollowScroll::CenterRow => {
                    pipeline.scroll_to_center_row_pos(cursor_row, height)
                }
                crate::view_policy::FollowScroll::Deferred => self.active.extra.scroll,
            };
        }
        let max = self.gpu.as_ref().unwrap().pipeline.max_scroll_rows(height);
        match diff_scroll {
            Some(ds) => {
                let clamped = ds.min(max);
                if let Some(ov) = self.workspace_state.overlay_mut() {
                    ov.diff_scroll = clamped;
                }
                if view.scroll != crate::render::ScrollPos::at_row(clamped) {
                    view.scroll = crate::render::ScrollPos::at_row(clamped);
                    self.gpu.as_mut().unwrap().pipeline.set_view(&view);
                }
            }
            None => {
                self.normalize_and_repush_scroll(&mut view, prev_scroll, height);
                debug_assert!(self.active.extra.scroll.px_q >= 0);
            }
        }
        self.update_ime_cursor_area();

        self.apply_caret_impulses();

        // LIFETIME STATS: accumulate the caret's DOCUMENT-space travel now that the
        // pipeline's caret target reflects this sync's cursor. `sync_view` is the
        // one live bridge every caret move passes through; the hook adds distance
        // only when the logical cursor actually moved (never on a pure scroll /
        // re-layout), and no-ops when the odometer is off (config-gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_track_caret();

        // Push the odometer snapshot to the pipeline so a held HUD this frame reads
        // the current lifetime figures (live-only; a capture never calls `sync_view`,
        // so its odometer rows stay the "—" placeholder).
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_sync_hud();

        // WRITING STREAKS: push the live year-view so a summoned card this frame
        // reads the real heatmap (live-only; a capture shows the placeholder).
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_sync_card();

        #[cfg(not(target_arch = "wasm32"))]
        self.sync_hud_saved();

        #[cfg(not(target_arch = "wasm32"))]
        self.sync_update_checked();

        // DISCOVERABILITY (phase 2): push the hold-⌘ peek's personalized rows + the
        // Keybindings footer's tips from the ledger (live-only; a capture never calls
        // `sync_view`, so the peek falls back to the starter six and the footer hides).
        #[cfg(not(target_arch = "wasm32"))]
        self.sync_discoverability();
    }

    /// Re-run spell detection immediately when the text version changes. The cache
    /// trigger is shared with capture, while this live-only effect owns cache mutation.
    fn sync_spell_cache(&mut self) {
        if self.spell.is_some()
            && crate::view_policy::spell_recompute_needed(
                self.active.extra.spell_checked_version,
                self.active.buffer.version(),
            )
        {
            self.recompute_spell_cache();
        }
    }

    /// The document text for this sync — the ROPE-CLONE SHORT-CIRCUIT. `sync_view`
    /// runs on every cursor move / scroll / selection change, none of which bump the
    /// buffer version — yet each would otherwise walk the whole rope into a fresh
    /// `String`. Reuse the last clone (a memcpy) while the version is unchanged;
    /// re-materialise the rope only after a real edit. The resulting bytes are
    /// identical either way. The cache is keyed by the buffer VERSION alone, so a
    /// BUFFER SWAP (open / new note — a fresh buffer restarting at version 0) must
    /// drop it at the swap site (`load_path` / `new_document`): an un-edited previous
    /// buffer also sits at version 0, and its stale entry would otherwise be served
    /// as the NEW document's text (the live "open a file and nothing appears" bug).
    pub(super) fn view_text(&mut self) -> String {
        let text_version = self.active.buffer.version();
        match &self.active.extra.sync_text_cache {
            Some((v, t)) if *v == text_version => t.clone(),
            _ => {
                let t = self.active.buffer.text();
                self.active.extra.sync_text_cache = Some((text_version, t.clone()));
                t
            }
        }
    }

    /// DIFF-AS-PREVIEW: the History picker's live-preview TRANSCRIPT (the writer's
    /// diff of the current buffer vs the highlighted version — see the one owner
    /// [`crate::history::diff_preview`]), or `None` when no preview applies (other
    /// overlays / no overlay / the empty-state row / an unresolvable id — the
    /// document then just shows the buffer, a calm degrade). Rendered ONCE per id
    /// into the `history_preview` cache, so an arrow/hover/wheel burst re-diffs
    /// nothing. Reads only; the buffer is never touched.
    ///
    /// SYNCHRONOUS (no per-arrow debounce): the round's release perf probe measured
    /// ~1-2 ms per diff at contract-document scale — the diff FOLDS unchanged regions, so the
    /// transcript stays tiny and the reshape stays cheap even against a large draft
    /// (~15 ms of compute at 6k lines, still well inside a single stepped selection).
    /// So no measured demand for the theme-font-style debounce; the cost is paid
    /// straight, and live == the deterministic headless capture (`main/run.rs`).
    pub(super) fn history_preview_text(&mut self) -> Option<String> {
        let ov = self
            .workspace_state
            .overlay()
            .filter(|o| o.kind == crate::overlay::OverlayKind::History)?;
        let id = ov.selected_history_id()?.to_string();
        if let Some((cached_id, transcript)) = &self.active.extra.history_preview
            && *cached_id == id
        {
            return Some(transcript.clone());
        }
        let current = self.view_text();
        let ov = self
            .workspace_state
            .overlay()
            .filter(|o| o.kind == crate::overlay::OverlayKind::History)?;
        let (id, transcript, _counts) = crate::history::diff_preview(
            ov,
            self.active.buffer.path(),
            self.active.buffer.is_unnamed_fresh(),
            &current,
        )?;
        self.active.extra.history_preview = Some((id, transcript.clone()));
        Some(transcript)
    }

    #[allow(clippy::type_complexity)]
    fn search_view_fields(
        &self,
    ) -> (
        Vec<((usize, usize), (usize, usize))>,
        Option<usize>,
        String,
        bool,
        bool,
        bool,
        String,
        bool,
        usize,
        usize,
    ) {
        if let Some(st) = self.workspace_state.search() {
            let matches = st
                .matches()
                .iter()
                .map(|m| {
                    (
                        self.active.buffer.char_to_line_col(m.start),
                        self.active.buffer.char_to_line_col(m.end),
                    )
                })
                .collect();
            (
                matches,
                st.current_index(),
                st.query().to_string(),
                true,
                st.is_case_sensitive(),
                st.is_replace_active(),
                st.replacement().to_string(),
                st.is_editing_replacement(),
                st.query_caret(),
                st.replacement_caret(),
            )
        } else {
            (
                Vec::new(),
                None,
                String::new(),
                false,
                false,
                false,
                String::new(),
                false,
                0,
                0,
            )
        }
    }

    fn apply_caret_impulses(&mut self) {
        if let Some(imp) = self.caret_impact.take()
            && let Some(gpu) = self.gpu.as_mut()
        {
            match imp {
                CaretImpact::Type => gpu.pipeline.caret_type_impact(),
                CaretImpact::Delete => gpu.pipeline.caret_delete_squash(),
                CaretImpact::Gulp => gpu.pipeline.caret_gulp(),
                CaretImpact::Land => gpu.pipeline.caret_line_land(),
                CaretImpact::Copy => gpu.pipeline.copy_pulse(),
            }
        }
        // BLOCKED-ACTION RECOIL: a motion/scroll/undo/delete that couldn't proceed
        // bumps the visual caret away from the wall (every caret look).
        if let Some(dir) = self.caret_recoil.take()
            && let Some(gpu) = self.gpu.as_mut()
        {
            gpu.pipeline.caret_recoil(dir);
        }
    }

    pub(super) fn update_ime_cursor_area(&self) {
        let Some(gpu) = self.gpu.as_ref() else {
            return;
        };
        let (x, y, w, h) = gpu.pipeline.caret_pixel_rect();
        gpu.window.set_ime_cursor_area(
            winit::dpi::PhysicalPosition::new(x as f64, y as f64),
            winit::dpi::PhysicalSize::new(w.max(1.0) as f64, h.max(1.0) as f64),
        );
    }
}
