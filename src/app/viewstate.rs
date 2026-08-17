use super::*;
mod scroll;

impl App {
    pub(super) fn sync_view(&mut self, follow: bool) {
        if self.sync_menu_context_and_gpu_absent() {
            return;
        }
        self.frame.clear_zoom_reflow();
        let height = self.frame.gpu().unwrap().config.height as f32;
        debug_assert!(height.is_finite());
        let (cursor_line, cursor_col) = self.document.buffer().cursor_line_col();
        self.sync_spell_cache();
        // Update the title marker and native edited dot only on a clean↔dirty
        // transition; sync_view already observes every live edit.
        if self.persistence.title_cache_stale(self.is_document_dirty()) {
            self.update_title();
        }
        // Schedule a debounced AUTO-SAVE for the active quick note when its text
        // changed. This lives ONLY here (the live windowed path, gated by the
        // gpu-present check above), so the headless capture/replay never auto-writes
        // — the determinism + no-fixture-mutation guarantee. The write fires in
        // `about_to_wait` after a quiet period.
        if self.document.buffer().is_unnamed_fresh()
            && self
                .persistence
                .note_write_owed(self.document.buffer().version())
        {
            let now = self.frame.now();
            self.persistence.arm_note_debounce(now);
        }
        // Arm the DOCUMENT AUTOSAVE idle timer (config-gated, default ON) when a
        // non-note buffer's text changed since its last write — a pathed document
        // tracks `doc_saved_version`, the no-path scratch its stash version.
        // Same determinism guarantee as the note arming above: this lives ONLY
        // under the gpu-present gate, so headless can never schedule a write.
        if self.config.autosave_on() && !self.document.buffer().is_unnamed_fresh() {
            let unsaved = if self.document.buffer().path().is_some() {
                self.document.doc_saved_version() != Some(self.document.buffer().version())
            } else {
                self.document.scratch_saved_version() != Some(self.document.buffer().version())
            };
            if unsaved {
                self.document.arm_doc_autosave(self.frame.now());
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
        let preview = self.comparison_transcript();
        // DIFF-AS-PREVIEW scroll: while the diff preview is up, the page shows the
        // OVERLAY's own `diff_scroll` (PgUp/PgDn / panel-focus ↑/↓ / the wheel over
        // the page all mutate it) — and `self.document.scroll()`, the DOCUMENT's
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
        // buffer version is unchanged (see [`Self::view_text`]). This is always the
        // BUFFER's own text: a preview substitutes its transcript in below, through
        // the one door that keeps the document behind it, so the version-keyed
        // `sync_text_cache` still never holds a previewed version's bytes (the
        // cache-key discipline).
        let text = self.view_text();
        // The follow branch chases the BUFFER cursor; a preview clamps that cursor
        // into a DIFFERENT text, so arrowing the rows must never scroll-chase it.
        let follow = follow && preview.is_none();

        let version = self.document.buffer().version();
        let (streak_override, held) = self.frame.take_caret_motion_flags();
        let is_edit_move = self.document.caret_was_synced_at(version) && !streak_override;
        // Was the keypress driving this sync an OS auto-repeat (a HELD arrow)?
        // One-shot, like `caret_edit_streaks`: consumed here so a following
        // non-keyboard sync (IME/wheel) doesn't inherit a stale held flag.
        let popover = if self.workspace_state.popover_holds_attention()
            && crate::popover::popover_on()
            && self.document.buffer().has_selection()
        {
            crate::actions::popover::plan(
                &self.document.buffer().text(),
                self.document.buffer().anchor_char(),
                self.document.buffer().cursor_char(),
                self.document.buffer().is_markdown(),
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
        // without an immediate rescan) can never paint. Every verdict is keyed to
        // the REAL buffer text, which `text` above now always is; a preview clears
        // the list outright below.
        let misspelled = crate::spell::visible(self.document.spell_cache(), &text);

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
            caret_affinity: self.document.buffer().affinity(),
            scroll: scroll::resolved_scroll(diff_scroll, self.document.scroll()),
            zoom: self.frame.zoom(),
            selection: self.document.buffer().selection_line_col(),
            preedit: self.input.preedit().to_owned(),
            misspelled,
            is_edit_move,
            held,
            selecting_drag: self.input.selecting_drag(),
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
            // Carry the alignment FROZEN at summon (`OverlayState::align`)
            // straight through — read verbatim every frame, so a live theme-preview
            // crossing never recomputes it and the open card holds its placement.
            overlay_align: ov.map(|o| o.align),
            // THE CRISP-BACKDROP exception, asked of the ONE owner rather than
            // re-decided here: which kinds preview live DOCUMENT state behind
            // their card, and so cannot afford to frost it, is
            // `OverlayKind::keeps_backdrop_crisp`'s question — shared with the
            // capture door so a headless frame cannot disagree with this one.
            overlay_crisp: ov.is_some_and(|o| o.kind.keeps_backdrop_crisp()),
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
            // The per-kind visible-row cap (MAX_SUGGESTIONS + 1 for spell /
            // 12 flat+faceted / more for theme), the ONE owner the pipeline windows
            // against so the drawn rows match the hover/keyboard item-window exactly.
            overlay_window_rows: ov.map(|o| o.window_rows()).unwrap_or(12),
            overlay_hint: self.workspace_state.journey().foot_hint(),
            overlay_lens: ov.map(|o| o.lens_strip()).unwrap_or_default(),
            overlay_workspace: ov.is_some_and(|o| o.workspace_shape().is_some()),
            // The one fact geometry, keyboard handling and the
            // footer hint reduce to: does this workspace's PRIMARY column
            // carry its own rows (a future timeline), rather than category
            // labels? `false` off a workspace and for `RailOverRows`
            // (Settings, today); `true` belongs to `TimelineOverComparison`.
            overlay_rows_primary: ov.is_some_and(|o| {
                o.workspace_shape()
                    .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary)
            }),
            // …and is there anything IN that comparison region? Exactly when the
            // text pushed above is a transcript rather than the user's own
            // document, so an empty timeline leaves the live document where it
            // was instead of standing it up inside the workspace.
            overlay_comparison: preview.is_some(),
            overlay_sections: ov.map(|o| o.item_sections()).unwrap_or_default(),
            overlay_location: ov
                .and_then(|o| o.location())
                .map(std::string::ToString::to_string),
            caret_preview: ov
                .filter(|o| o.kind == crate::overlay::OverlayKind::Caret)
                .and_then(|o| o.selected_caret_mode()),
            gutter_name: self.document.buffer().display_name(),
            gutter_project: self.project_location.project.name.clone(),
            // The stack is scoped to the root the ACTIVE FILE remembers, which is
            // the same root the folder line above names — not the ambient active
            // root, so a buffer activated from another project cannot draw its
            // siblings under this project's heading. Empty (single file, or no
            // remembered root yet) leaves the gutter on its pre-stack path.
            gutter_files: self
                .document
                .working_set()
                .active_root()
                .map(|root| self.document.working_set().stack_rows(root))
                .unwrap_or_default(),
            // THE PERSISTENT AFFORDANCE. Asked of the latch itself, per FRAME —
            // not raised by an event and not re-raised by a poll — so it is true
            // for exactly as long as the conflict is and cannot be cleared by an
            // unrelated toast expiring on top of it.
            gutter_changed: self.change_unresolved(),
            is_markdown: self.document.buffer().is_markdown(),
            doc_dir: self
                .document
                .buffer()
                .path()
                .and_then(|p| p.parent())
                .map(|d| d.to_path_buf()),
            syn_lang: self.document.buffer().syntax_lang(),
            overlay_spell: ov
                .filter(|o| o.kind == crate::overlay::OverlayKind::Spell)
                .and_then(|o| o.spell_target),
            overlay_context_anchor: ov.and_then(|o| o.context_anchor),
            notice: self.frame.notice().owned().unwrap_or_default(),
            // The KIND rides with the text from the one snapshot, so the render
            // layer never has to guess a notice's lifetime from its sentence.
            notice_kind: self.frame.notice().kind(),
            cjk_priority: self.config.cjk_priority_or_default(),
            eol: self.document.buffer().eol(),
            popover,
            overlay_detail_focus: ov.map(|o| o.detail_focus).unwrap_or(false),
            folds: Vec::new(),
            fold_tails: Vec::new(),
            folded_headings: Vec::new(),
            // `text` above IS the document; the two substitutions below (preview
            // transcript, fold filter) each record it via `substitute_text`.
            doc_source: None,
        };
        // Preview text may be shorter than the buffer, so every line/col span
        // index the BUFFER text must be re-bounded or cleared — the cursor clamps
        // into the previewed text (the shared `clamp_line_col`); selection /
        // preedit / squiggles / search highlights are dropped for the preview's
        // duration (they'd misalign, or panic in the glyph-span layer). All
        // restored automatically on close: the next sync rebuilds them from the
        // untouched buffer.
        if let Some(transcript) = preview.clone() {
            // Through the ONE door, which keeps the document behind the transcript
            // so the card figures stay over it: a word count of a writer's diff —
            // markers, both sides of every change — is a fact about nothing, and
            // the announced count never left the buffer.
            view.substitute_text(transcript);
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
        view.folds = self.document.buffer().folds().iter().copied().collect();
        if preview.is_none() && self.document.buffer().has_folds() {
            crate::fold::apply_to_view(
                &mut view,
                &self.document.buffer().hidden_lines(),
                &self.document.buffer().fold_tails(),
                self.document.buffer().folds(),
            );
        }
        {
            let gpu = self.frame.gpu_mut().unwrap();
            gpu.pipeline.set_view(&view);
        }

        let prev_scroll = self.document.scroll();
        if let Some(anchor) = self.frame.take_zoom_anchor() {
            // ZOOM ANCHOR wins this sync: this `set_view` just reshaped to the newly
            // changed zoom, so re-solve the scroll that keeps the anchored document
            // point at its captured screen y (the ONE owner does the variable-row
            // math + clamp). Overrides cursor-follow — the anchored caret is on
            // screen by construction, and the off-screen fallback deliberately holds
            // the viewport centre rather than yanking to the caret.
            let pipeline = &self.frame.gpu().unwrap().pipeline;
            let scroll =
                pipeline.zoom_anchor_scroll_pos(anchor.line, anchor.col, anchor.screen_y, height);
            self.document.set_scroll(scroll);
        } else if follow {
            let pipeline = &self.frame.gpu().unwrap().pipeline;
            // Affinity resolves shared boundaries to the caret's visual row.
            let cursor_row = pipeline.visual_row_of_aff(
                cursor_line,
                cursor_col,
                self.document.buffer().affinity(),
            );
            let scroll = match crate::view_policy::follow_scroll_strategy(
                crate::typewriter::typewriter_on(),
                self.input.selecting_drag(),
            ) {
                crate::view_policy::FollowScroll::ShowRow => {
                    pipeline.scroll_to_show_row_pos(cursor_row, self.document.scroll(), height)
                }
                crate::view_policy::FollowScroll::CenterRow => {
                    pipeline.scroll_to_center_row_pos(cursor_row, height)
                }
                crate::view_policy::FollowScroll::Deferred => self.document.scroll(),
            };
            self.document.set_scroll(scroll);
        }
        let max = self.frame.gpu().unwrap().pipeline.max_scroll_rows(height);
        match diff_scroll {
            Some(ds) => {
                let clamped = ds.min(max);
                if let Some(ov) = self.workspace_state.overlay_mut() {
                    ov.diff_scroll = clamped;
                }
                if view.scroll != crate::render::ScrollPos::at_row(clamped) {
                    view.scroll = crate::render::ScrollPos::at_row(clamped);
                    self.frame.gpu_mut().unwrap().pipeline.set_view(&view);
                }
            }
            None => {
                self.normalize_and_repush_scroll(&mut view, prev_scroll, height);
                debug_assert!(self.document.scroll().px_q >= 0);
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
    pub(super) fn sync_spell_cache(&mut self) {
        if self.document.spell_enabled()
            && crate::view_policy::spell_recompute_needed(
                self.document.spell_checked_version(),
                self.document.buffer().version(),
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
        self.document.sync_text()
    }

    /// **THE ONE RESOLVER OF READ-ONLY COMPARISON PROSE** — what the comparison
    /// region shows this frame, or `None` when nothing does (no overlay, a card
    /// with no comparison, a timeline on its empty-state row, an unresolvable
    /// subject — the document then just shows the buffer, a calm degrade).
    ///
    /// The REQUEST is typed and kind-neutral ([`crate::overlay::ComparisonRequest`],
    /// whose module doc says why); the ANSWER comes from the one dispatch
    /// ([`crate::comparison::prose_for`]), which picks the producer for the
    /// surface that asked. This function owns the CACHE and the live inputs, not
    /// the prose. Rendered ONCE per request into the `history_preview` cache, keyed by
    /// [`crate::overlay::ComparisonRequest::cache_key`] — VIEW and subject, so a
    /// surface offering several read-only views of one subject cannot be served
    /// the wrong one. Reads only; the buffer is never touched.
    ///
    /// SYNCHRONOUS (no per-arrow debounce): the round's release perf probe measured
    /// ~1-2 ms per diff at contract-document scale — the diff FOLDS unchanged regions, so the
    /// transcript stays tiny and the reshape stays cheap even against a large draft
    /// (~15 ms of compute at 6k lines, still well inside a single stepped selection).
    /// So no measured demand for the theme-font-style debounce; the cost is paid
    /// straight, and live == the deterministic headless capture (`main/run.rs`).
    pub(super) fn comparison_transcript(&mut self) -> Option<String> {
        let request = self.workspace_state.overlay()?.comparison_request()?;
        let key = request.cache_key();
        if let Some(transcript) = self.document.history_preview(&key) {
            return Some(transcript.to_string());
        }
        let current = self.view_text();
        let ov = self.workspace_state.overlay()?;
        let (_, transcript, _counts) = crate::comparison::prose_for(
            ov,
            &request,
            self.document.buffer().path(),
            self.document.buffer().is_unnamed_fresh(),
            &current,
        )?;
        self.document.set_history_preview(key, transcript.clone());
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
                        self.document.buffer().char_to_line_col(m.start),
                        self.document.buffer().char_to_line_col(m.end),
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
        if let Some(imp) = self.frame.take_caret_impact()
            && let Some(gpu) = self.frame.gpu_mut()
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
        if let Some(dir) = self.frame.take_caret_recoil()
            && let Some(gpu) = self.frame.gpu_mut()
        {
            gpu.pipeline.caret_recoil(dir);
        }
    }

    pub(super) fn update_ime_cursor_area(&self) {
        let Some(gpu) = self.frame.gpu() else {
            return;
        };
        let (x, y, w, h) = gpu.pipeline.caret_pixel_rect();
        gpu.window.set_ime_cursor_area(
            winit::dpi::PhysicalPosition::new(x as f64, y as f64),
            winit::dpi::PhysicalSize::new(w.max(1.0) as f64, h.max(1.0) as f64),
        );
    }
}
