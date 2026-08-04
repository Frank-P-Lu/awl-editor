//! Live-only effects layered around shared pure action application.

mod overlay_sync;

use super::apply_context::{CoreBefore, CoreRun};
use super::*;

type SpellTarget = (Vec<String>, (usize, usize, usize), String);

struct OverlayInputs {
    spell_target: Option<SpellTarget>,
    history_entries: Vec<crate::history::TimelineRow>,
    assets: Vec<crate::assets::Orphan>,
    row_gates: crate::commands::RowGates,
}

struct GotoInputs {
    goto_corpus: Vec<String>,
    goto_times: Vec<String>,
    goto_open: Vec<usize>,
    goto_recent: Vec<usize>,
    goto_headings: Vec<(String, usize)>,
}

impl App {
    /// Recompute the text-keyed spell cache without synchronizing the view.
    pub(super) fn recompute_spell_cache(&mut self) {
        self.document.recompute_spell_cache();
    }

    /// FLIGHT RECORDER / PROBE — the open picker's LOGICAL selection
    /// as `(kind, selected, reachable_items, scroll)`, or `None` with no card up.
    /// `reachable_items` is `items.len()` (the refiltered, selectable rows), so a
    /// trace line distinguishes "the selection did not move" from "there was
    /// nowhere left to move to".
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn overlay_selection_probe(
        &self,
    ) -> Option<(crate::overlay::OverlayKind, usize, usize, usize)> {
        self.workspace_state
            .overlay()
            .map(|o| (o.kind, o.selected, o.items.len(), o.scroll))
    }

    pub(super) fn run_spellcheck_now(&mut self) {
        self.recompute_spell_cache();
        self.sync_view(false);
    }

    /// Preview colors immediately and defer reshaping until navigation settles.
    /// Every preview arms the compositor transaction before redraw.
    // `prev` (the outgoing world) is now only named by the native probe trace, so
    // the wasm build — which never runs a probe — reads it as unused.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    pub(super) fn retint_theme_preview(&mut self, prev: crate::theme::Theme) {
        // ITEM 85 — arm the MOVEMENT-LATENCY clock: this is the ONE owner every input
        // kind (keyboard nav, mouse hover, mouse wheel) funnels a theme-picker world
        // change through, so marking HERE — right before the real relayout work below
        // — measures the actual event → first-presented-frame round trip regardless
        // of which input drove it. Closed out in `Gpu::redraw` at the exact point the
        // frame this step produces gets presented. A no-op unless `probe::recording()`.
        #[cfg(not(target_arch = "wasm32"))]
        crate::probe::mark_movement_input();
        // DEBUG settle readout (live-only): stamp the input that triggered this preview
        // step as the switch's start. Re-stamped every arrow, so once the selection
        // rests and the deferred reshape settles, the felt total measures from the LAST
        // input to the settled present. Gated on `debug_on()` — the pane never creates
        // the work it measures. Off the headless path (replay never calls this seam).
        if crate::debug::debug_on() {
            self.frame.stamp_theme_switch(self.frame.now());
        }
        let needs_theme_reshape = if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.sync_theme_colors();
            gpu.pipeline.needs_theme_reshape()
        } else {
            false
        };
        if needs_theme_reshape {
            let now = self.frame.now();
            // An isolated step reshapes immediately; a rapid run coalesces into
            // one trailing reshape.
            let schedule = self.frame.theme_font_schedule();
            match theme_font_reshape_decision(
                schedule.pending,
                schedule.last_reshape_at,
                schedule.last_reshape_cost,
                now,
                theme_font_debounce(),
                theme_font_cheap_reshape(),
            ) {
                ThemeFontReshapeDecision::Immediate => {
                    // The IMMEDIATE arm is MEASURED too. An untimed reshape here
                    // would leave the settle readout able to report only the
                    // COALESCED path — the one that includes a deliberate wait —
                    // so a genuinely isolated arrow run drew no theme lines at
                    // all, which is how a breakdown covering a fraction of its
                    // own headline became the normal reading. The measurement is
                    // also the leading edge's own cost input.
                    let input_at = crate::debug::debug_on()
                        .then(|| self.frame.theme_switch_at())
                        .flatten();
                    let cost = self.sync_theme_font_measured(input_at);
                    self.frame.mark_theme_font_reshaped(now, cost);
                    self.frame.clear_theme_font();
                }
                ThemeFontReshapeDecision::Coalesce => {
                    self.frame.arm_theme_font(now);
                }
            }
        } else {
            self.frame.clear_theme_font();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            // Log the DESTINATION page ground (`base_100`) — what the writing
            // surface SHOULD be this preview step. A vanish is "the page went
            // blank/stale on screen": pairing this intended color with the
            // present outcome (traced below) tells the black box whether awl's
            // correct light-ground frame actually reached the compositor.
            let g = crate::theme::base_100().rgb_bytes();
            crate::probe::trace(format_args!(
                "retint_preview {} -> {} (page_ground #{:02x}{:02x}{:02x}, bracket armed)",
                prev.name,
                crate::theme::active().name,
                g[0],
                g[1],
                g[2],
            ));
        }
        self.frame
            .arm_settle(frame::SettleKind::Crossing, self.frame.now());
        self.sync_present_txn();
        self.update_title();
    }

    /// THEME re-tint, SETTLED form: the full synchronous `sync_theme` (colors +
    /// font reshape) plus the title refresh, cancelling any pending deferred
    /// reshape — the commit (Enter) / revert (Esc, C-g, click-away) path, where
    /// the chosen world must apply completely before the picker's absence.
    pub(super) fn retint_theme_now(&mut self) {
        self.frame.clear_theme_font();
        // DEBUG settle readout (live-only): a direct/commit retint stamps the switch
        // start at the retint (essentially the input time). Colors always apply now
        // (`sync_theme` = colors + font); the font half routes through the shared
        // timed-or-plain door below so it can feed the settle breakdown.
        let input_at = crate::debug::debug_on().then(|| self.frame.now());
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.sync_theme_colors();
        }
        let cost = self.sync_theme_font_measured(input_at);
        // ITEM 202: a commit/revert reshapes for real, so it counts as the most
        // recent reshape for the NEXT summon's leading-edge decision.
        self.frame.mark_theme_font_reshaped(self.frame.now(), cost);
        self.update_title();
    }

    /// Apply one theme-font reshape and return the RESHAPE-SIDE cost it took
    /// (`SwitchPhase::RESHAPE_SIDE`), or `None` when the guard found no work.
    ///
    /// THE LIVE APP ALWAYS MEASURES. The cost is not a diagnostic: it is the
    /// leading-edge test's own work input
    /// (`theme_font_debounce`), so gating the measurement on `debug_on()` would
    /// gate the fix on the debug panel being open. Every live reshape therefore
    /// runs the same `sync_theme_font_timed` door — identical work to the plain
    /// variant plus a forced row-geom walk the next prepare would do anyway (that
    /// method's own doc: the rendered frame stays byte-identical). The plain
    /// `sync_theme_font` remains the headless path's only variant, so a capture
    /// still touches no `Instant`, and a headless `App` has no GPU to reach this
    /// at all.
    ///
    /// `input_at` additionally arms the DEBUG settle transaction, and is where
    /// `SwitchPhase::Wait` is measured: input → the start of the work is the
    /// deliberate interval, zero on the immediate path and the whole debounce
    /// window on a coalesced one.
    fn sync_theme_font_measured(&mut self, input_at: Option<Instant>) -> Option<Duration> {
        use crate::themeswitch::SwitchPhase;
        let started = input_at.map(|_| self.frame.now());
        let mut phases = self.frame.gpu_mut()?.pipeline.sync_theme_font_timed()?;
        let cost = phases
            .reshape_side_ms()
            .map(|ms| Duration::from_secs_f32(ms / 1000.0));
        if let (Some(input_at), Some(started)) = (input_at, started) {
            let work_done_at = self.frame.now();
            phases.record(
                SwitchPhase::Wait,
                started.saturating_duration_since(input_at).as_secs_f32() * 1000.0,
            );
            self.frame.set_theme_settle(Some(ThemeSettleInFlight {
                input_at,
                phases,
                work_done_at,
            }));
        }
        cost
    }

    pub(super) fn apply_deferred_theme_font(&mut self) {
        self.frame.clear_theme_font();
        // ITEM 202: this IS a reshape (the coalesced-burst settle), so the next
        // preview step's leading-edge decision must see it as recent, not stale.
        // Stamped BEFORE the GPU-gated reshape (a hermetic App has no GPU, so the
        // trailing half stays exercised for real in `theme_debounce_item202.rs`);
        // the measured cost lands on the same mark a moment later.
        self.frame.mark_theme_font_reshaped(self.frame.now(), None);
        let input_at = crate::debug::debug_on()
            .then_some(self.frame.theme_switch_at())
            .flatten();
        let cost = self.sync_theme_font_measured(input_at);
        self.frame.mark_theme_font_reshaped(self.frame.now(), cost);
        self.sync_view(false);
        // The reshape is now APPLIED into the view but not yet PRESENTED. This
        // redraw carries it to the screen; because the present-transaction bracket
        // is still armed (the settle in this same `about_to_wait` pass only marks
        // `crossing_teardown_pending`, it never disarms the bracket), that present
        // lands INSIDE the transaction. See `finish_crossing_settle`.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "deferred_reshape applied (bracketed present to follow)"
            ));
        }
        self.request_frame();
    }

    // MIRROR-ON-COPY/KILL. Call AFTER a buffer mutation that may have changed
    // the kill ring top. Writes to the OS clipboard only when the value is
    // non-empty AND differs from what we last wrote (avoids feedback loops and
    // redundant writes; an unchanged kill — e.g. a no-op copy or a selection
    // delete that didn't fill the kill ring — writes nothing).
    //
    // WAYLAND NOTE: on a Wayland compositor (e.g. Hyprland/Omarchy) the write
    // succeeds only if awl holds a clipboard-capable seat; arboard keeps the
    // single App-lifetime Clipboard alive to retain ownership. Errors here are
    // swallowed (graceful degradation) — never panic on a clipboard write.
    pub(super) fn sync_kill_to_clipboard(&mut self) {
        let Some(clip) = self.clipboard.as_mut() else {
            return;
        };
        let killed = self.document.buffer().kill_buffer();
        if killed.is_empty() {
            return; // never clobber the OS clipboard with an empty kill
        }
        if self.clipboard_last_written.as_deref() == Some(killed) {
            return; // we already wrote exactly this; skip redundant write
        }
        let owned = killed.to_string(); // drop the &self.document.buffer() borrow
        if let Ok(()) = clip.set_text(owned.clone()) {
            self.clipboard_last_written = Some(owned);
        }
    }

    pub(super) fn refresh_kill_from_clipboard(&mut self) {
        let Some(clip) = self.clipboard.as_mut() else {
            return;
        };
        let text = match clip.get_text() {
            Ok(t) => t,
            Err(_) => return, // empty / non-text / unsupported: keep internal
        };
        if text.is_empty() {
            return; // empty external clipboard does not override internal kill
        }
        if self.clipboard_last_written.as_deref() == Some(text.as_str()) {
            return; // it's our own value; nothing external changed
        }
        self.document.set_kill(&text);
        self.clipboard_last_written = Some(text);
    }

    /// PASTE-IMAGE (native, LIVE-only): if the OS clipboard holds an IMAGE rather
    /// than text, save it as a PNG into an `assets/` folder beside the doc and
    /// resolve the reference for a typed `InsertImageReference` continuation —
    /// the Typora/Obsidian convention. Returns `None` when the clipboard held no
    /// image or any step failed gracefully, so the interpreter feeds a typed
    /// text-yank continuation through the same core. Mirrors the error discipline of the
    /// text clipboard bridge (`sync_kill_to_clipboard`) — NEVER panics on a bad
    /// image / a failed fs write / a mismatched buffer.
    ///
    /// NO-PATH BUFFER (settled): a path-less buffer — bare scratch or an unnamed
    /// quick note — first runs [`Self::ensure_note_named_before_paste`], which
    /// triggers the notes system's OWN auto-name save so the paste lands beside
    /// a real, notes-root-relative file rather than a scratch-only location. An
    /// EMPTY buffer has no first line to derive a name from and stays path-less
    /// (that save quietly errs); the ABSOLUTE data-root fallback below still
    /// makes THAT paste succeed.
    ///
    /// UNDO NOTE (documented): Cmd-Z removes the inserted REF TEXT only; the
    /// written PNG is left on disk as a harmless orphan (like any editor — we do
    /// not track+delete the file on undo). DETERMINISM: the unique filename comes
    /// from PROBING the assets dir (`pasted-1.png`, `pasted-2.png`, …), never a
    /// clock/random; and the whole path lives only on the live App, so a headless
    /// `--screenshot`/`--keys` capture never reaches a real clipboard image.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn paste_image_reference(&mut self) -> Option<String> {
        use crate::paste_image;
        let clip = self.clipboard.as_mut()?;
        let img = match clip.get_image() {
            Ok(img) => img,
            Err(_) => return None,
        };
        let png = paste_image::encode_rgba_png(img.width, img.height, &img.bytes)?;
        if self.document.buffer().path().is_none() {
            self.ensure_note_named_before_paste();
        }
        let data_root = crate::fs::data_root();
        let doc_path = self.document.buffer().path().map(|p| p.to_path_buf());
        paste_image::persist_png(doc_path.as_deref(), &data_root, &png)
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn paste_image_reference(&mut self) -> Option<String> {
        None
    }

    pub(super) fn apply(
        &mut self,
        action: Action,
        shift: bool,
        exit: &dyn schedule::Exit,
        door: crate::stats::Door,
    ) -> bool {
        let action = self.prepare_tutorial_action(action);
        self.pre_apply(&action, door);

        // FLIGHT RECORDER / PROBE: the STATE link of the event→present
        // chain, sampled either side of the shared core so one trace line answers
        // "did this input advance the selection, advance it twice, or not at all".
        #[cfg(not(target_arch = "wasm32"))]
        let sel_before = self.overlay_selection_probe();

        let CoreRun {
            transition,
            theme_overlay_before,
            theme_before,
            history_overlay_before,
        } = self.run_action_core(&action, shift);
        let quit = transition.contains(|effect| matches!(effect, actions::Effect::Quit));
        let theme_committed = transition.contains(|effect| {
            matches!(
                effect,
                actions::Effect::OverlayAccept(crate::overlay::OverlayKind::Theme, _)
            )
        });
        let history_accepted = transition.contains(|effect| {
            matches!(
                effect,
                actions::Effect::OverlayAccept(crate::overlay::OverlayKind::History, _)
            )
        });
        let mut nested_quit = false;
        actions::visit_transition_effects(transition, |effect| {
            match effect {
                actions::Effect::RunAction(act) => {
                    // Run the nested transition at this exact position, then
                    // continue the outer stream. Returning here used to drop
                    // the outer SyncView + Redraw requests.
                    crate::commands::record_recent(&act);
                    nested_quit |= self.apply(act, shift, exit, crate::stats::Door::Palette);
                    let (_, journey) = self.workspace_state.core_slots();
                    journey.attribute_launch(Some(crate::overlay::OverlayKind::Command));
                }
                actions::Effect::Clipboard(actions::ClipboardEffect::PasteImage) => {
                    let continuation = match self.paste_image_reference() {
                        Some(reference) => Action::InsertImageReference(reference),
                        None => {
                            self.refresh_kill_from_clipboard();
                            Action::YankText
                        }
                    };
                    nested_quit |=
                        self.apply(continuation, false, exit, crate::stats::Door::Palette);
                }
                effect => self.apply_live_effect(effect),
            }
        });
        if !history_overlay_before
            && self
                .workspace_state
                .overlay()
                .is_some_and(|o| o.kind == crate::overlay::OverlayKind::History)
        {
            self.document.remember_history_scroll();
        }
        if history_overlay_before && !self.workspace_state.overlay_open() {
            self.history_overlay_closed(history_accepted);
        }
        self.post_transition_effects(theme_overlay_before, theme_committed, theme_before);

        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            let after = self.overlay_selection_probe();
            crate::probe::trace(format_args!(
                "apply {action:?} sel {sel_before:?} -> {after:?}"
            ));
        }

        // Quit routes back through an unresolved external change once; see
        // `defer_quit_once_for_conflict`.
        let quit = (quit || nested_quit) && !self.defer_quit_once_for_conflict();
        if quit {
            exit.exit();
        }
        quit
    }

    fn pre_apply(&mut self, action: &Action, door: crate::stats::Door) {
        // SILENT USAGE LEDGER: record this dispatch by its door into the persisted
        // per-command counts (`app/stats.rs`) — the discoverability signal phase 2
        // surfaces (never a nudge). Native-only + config-gated inside; a non-catalog
        // action (motion / self-insert / overlay-open) is filtered there. Placed at
        // the very top so it sees EVERY dispatch (incl. the macOS About early-return
        // and the palette `RunAction` re-dispatch); `apply` is the ONE seam all three
        // doors funnel through, so none needs a parallel recording path.
        #[cfg(not(target_arch = "wasm32"))]
        self.ledger_note_dispatch(action, door);
        #[cfg(target_arch = "wasm32")]
        let _ = (action, door);
        // DIFF-AS-PREVIEW note: the old Compare TAKEOVER's read-only gate lived
        // here. The takeover is RETIRED — the writer's diff now lives entirely
        // inside the History picker's live preview, whose read-only law is the
        // overlay's own modality (every key routes through `overlay_intercept`;
        // typing filters the QUERY, never the transcript or the buffer).

        // Buffer/zoom/search transitions are shared with headless `--keys` via
        // `actions::apply_transition`. Work the core cannot own is returned in
        // the closed typed-effect vocabulary and interpreted below.
        //
        // The render-only TOGGLES (caret look / page mode) flip a
        // process-global. That flip lives in the shared core,
        // so BOTH this live path and the headless `--keys` replay flow through one
        // place; GPU re-wrap, sync, persistence, and clipboard work arrive as
        // typed effects rather than being inferred again from the Action.
        //
    }

    fn gather_goto_inputs(&mut self, action: &Action) -> GotoInputs {
        if matches!(action, Action::OpenGoto | Action::OpenAssetClean) {
            self.rescan_file_index();
        }
        let location = &self.project_location;
        let recency_now = if location.root == self.config.default_folder {
            Some(crate::clock::system_now())
        } else {
            None
        };
        let (goto_corpus, goto_times) =
            crate::index::with_recency(&location.root, location.file_index.clone(), recency_now);
        let goto_open: Vec<usize> = {
            let active_rel = self.document.buffer().path().and_then(|p| {
                p.strip_prefix(&location.root)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            });
            goto_corpus
                .iter()
                .enumerate()
                .filter(|(_, c)| Some(*c) == active_rel.as_ref())
                .map(|(i, _)| i)
                .collect()
        };
        let goto_recent: Vec<usize> = location
            .recent_files
            .iter()
            .filter_map(|abs| {
                abs.strip_prefix(&location.root)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            })
            .filter_map(|rel| goto_corpus.iter().position(|c| *c == rel))
            .collect();
        let goto_headings: Vec<(String, usize)> =
            if matches!(action, Action::OpenGoto | Action::OpenOutline)
                && self.document.buffer().is_markdown()
            {
                crate::markdown::headings(&self.document.buffer().text())
                    .into_iter()
                    .map(|h| (h.label(), h.line))
                    .collect()
            } else {
                Vec::new()
            };
        GotoInputs {
            goto_corpus,
            goto_times,
            goto_open,
            goto_recent,
            goto_headings,
        }
    }

    fn gather_overlay_inputs(&mut self, action: &Action) -> OverlayInputs {
        #[allow(clippy::type_complexity)]
        let spell_target: Option<(Vec<String>, (usize, usize, usize), String)> =
            if matches!(action, Action::OpenSpellSuggest) {
                let (line, col) = self.document.buffer().cursor_line_col();
                self.document.spell_suggestion_target(line, col).map(|t| {
                    (
                        t.suggestions,
                        (
                            t.misspelling.line,
                            t.misspelling.start_col,
                            t.misspelling.end_col,
                        ),
                        t.word,
                    )
                })
            } else {
                None
            };
        // HISTORY TIMELINE rows: the current file's versions (newest-first), each
        // answering WHEN + WHICH with a "+N −M" changed-count vs the CURRENT buffer.
        // Gathered HERE (before the &mut self.document.buffer() borrow) and ONLY when the History
        // binding fired — reading + line-diffing the store is pure waste on every
        // other keystroke. The history key derivation lives in ONE place
        // (`history::source_path`): buffer path, else the persistent scratch's own
        // stash path — so the no-path scratch has a timeline too; only an unnamed
        // note has none (the picker then shows "no history yet"). `now` stamps the
        // relative labels; History is an explicitly-summoned, non-default overlay,
        // so this clock read never touches a default capture.
        let history_entries: Vec<crate::history::TimelineRow> =
            if matches!(action, Action::OpenHistory | Action::CompareVersion) {
                match crate::history::source_path(
                    self.document.buffer().path(),
                    self.document.buffer().is_unnamed_fresh(),
                ) {
                    Some(path) => crate::history::timeline_rows(
                        &path,
                        &self.document.buffer().text(),
                        crate::history::now_millis(),
                    ),
                    None => Vec::new(),
                }
            } else {
                Vec::new()
            };
        #[cfg(not(target_arch = "wasm32"))]
        let assets: Vec<crate::assets::Orphan> = if matches!(action, Action::OpenAssetClean) {
            let location = &self.project_location;
            crate::assets::scan(&location.root, &location.file_index)
        } else {
            Vec::new()
        };
        #[cfg(target_arch = "wasm32")]
        let assets: Vec<crate::assets::Orphan> = Vec::new();
        // DAEMON WAITER: is a `--wait` client actively parked on the CURRENT
        // buffer right now (the one "Finish file" would actually save + notify
        // + switch away from)? Gated exactly like `wait_conns` itself (native,
        // non-`mas` — see that field's doc): wasm/`mas` builds have no daemon at
        // all, so the palette row stays hidden there unconditionally. Drives
        // `commands::visible_hidden_mask` below — the ONE live fact behind the
        // "Finish file" row's visibility.
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        let has_waiter = crate::buffers::BufferKey::of(self.document.buffer())
            .is_some_and(|key| self.wait_conns.get(&key).is_some_and(|w| !w.is_empty()));
        #[cfg(any(target_arch = "wasm32", feature = "mas"))]
        let has_waiter = false;
        OverlayInputs {
            spell_target,
            history_entries,
            assets,
            row_gates: crate::commands::RowGates {
                has_waiter,
                change_unresolved: self.change_unresolved(),
            },
        }
    }

    fn run_action_core(&mut self, action: &Action, shift: bool) -> CoreRun {
        let page_scroll_lines = self.page_scroll_rows();
        let mut shift_selecting = self.document.shift_selecting();
        let mut zoom = self.frame.zoom();
        // Borrow the summoned-layer slots in place for the transition run.
        let overlay_was_open = self.workspace_state.overlay_open();
        let CoreBefore {
            theme_overlay_before,
            theme_before,
            history_overlay_before,
        } = CoreBefore::of(self.workspace_state.overlay());
        let config_keys = self.config.keys.clone();
        let config_linux_keep = self.config.effective_linux_keep();
        // Gather live picker inputs before the mutable buffer borrow below. File
        // and asset pickers rescan only when summoned, so their transient corpus
        // reflects disk changes without a watcher or per-keystroke I/O.
        let GotoInputs {
            goto_corpus,
            goto_times,
            goto_open,
            goto_recent,
            goto_headings,
        } = self.gather_goto_inputs(action);
        let OverlayInputs {
            spell_target,
            history_entries,
            assets,
            row_gates,
        } = self.gather_overlay_inputs(action);
        let location = &self.project_location;
        let build_ctx = crate::overlay::BuildCtx {
            goto_corpus,
            goto_open,
            goto_recent,
            goto_times,
            config_keys: &config_keys,
            config_linux_keep: &config_linux_keep,
            goto_headings,
            spell_target,
            history_entries,
            history_now: Some(crate::history::now_millis()),
            history_session_start: crate::history::session_epoch_ms(),
            settings_values: crate::settings::SettingsValues::gather(
                &self.config,
                &location.root,
                self.frame.zoom(),
                crate::dateformat::today_from_system_clock(),
            ),
            assets,
            row_gates,
        };
        let mut make_overlay =
            |kind: crate::overlay::OverlayKind| crate::overlay::build(kind, &build_ctx);
        // Browse rebuild hook: list ONE level via the shared `overlay::browse_level`
        // builder. `Browse` (C-x j) walks the active root and shows files + folders;
        // `MoveDest` (C-x m) walks the SAME active root and shows FOLDERS only (you
        // move a document into a folder within it); `Project` (C-x p) walks the
        // workspace by absolute path. Cloned roots dodge the &mut self.document.buffer()
        // borrow.
        let browse_root = location.root.clone();
        let workspace = location.workspace_root.clone();
        let recent_projects: Vec<String> = location
            .recent_projects
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        let mut browse_to = |kind: crate::overlay::OverlayKind, rel: Option<String>| {
            crate::overlay::browse_level(
                kind,
                rel,
                &browse_root,
                workspace.as_deref(),
                &recent_projects,
            )
        };
        // The visual-line motion LAYOUT ORACLE: the live GPU pipeline, which owns
        // the shaped wrap geometry. A shared borrow of `self.frame.gpu()` (disjoint from the
        // `&mut self.document.buffer()` below), so the same transition seam sees the SAME
        // geometry headless replay sees through its offscreen pipeline. `None` before
        // the window's GPU exists; motion then falls back to LOGICAL lines.
        let oracle = self
            .frame
            .gpu()
            .map(|g| &g.pipeline as &dyn actions::LayoutOracle);
        let (search, journey) = self.workspace_state.core_slots();
        let mut ctx = actions::ActionCtx {
            buffer: self.document.action_buffer_mut(),
            shift_selecting: &mut shift_selecting,
            zoom: &mut zoom,
            search,
            scroll_page_lines: page_scroll_lines,
            journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle,
        };
        let transition = actions::apply_transition(&mut ctx, action, shift);
        self.document.set_shift_selecting(shift_selecting);
        self.frame.set_zoom(zoom);
        let _ = make_overlay;
        let _ = browse_to;
        self.sync_overlay_after_core(overlay_was_open, self.input.resting_pointer());
        CoreRun {
            transition,
            theme_overlay_before,
            theme_before,
            history_overlay_before,
        }
    }

    #[cfg(test)]
    pub(in crate::app) fn transition_for_test(
        &mut self,
        action: &Action,
        shift: bool,
    ) -> actions::Transition {
        self.run_action_core(action, shift).transition
    }

    pub(in crate::app) fn apply_live_effect(&mut self, effect: actions::Effect) {
        match effect {
            actions::Effect::JumpToLine(line) => self.jump_to_line(line),
            actions::Effect::AddToDictionary(word) => self.add_to_dictionary(&word),
            actions::Effect::RebindCommit {
                slug,
                binding,
                confirmed,
            } => self.rebind_commit(slug, binding, confirmed),
            actions::Effect::RebindReset { slug } => self.rebind_reset(slug),
            actions::Effect::Recoil(dir) => self.frame.set_caret_recoil(Some(dir)),
            actions::Effect::TypeImpact => self.frame.set_caret_impact(Some(CaretImpact::Type)),
            actions::Effect::DeleteSquash => self.frame.set_caret_impact(Some(CaretImpact::Delete)),
            actions::Effect::Gulp => self.frame.set_caret_impact(Some(CaretImpact::Gulp)),
            actions::Effect::LineLand => self.frame.set_caret_impact(Some(CaretImpact::Land)),
            actions::Effect::CopyPulse => self.frame.set_caret_impact(Some(CaretImpact::Copy)),
            actions::Effect::SettingToggle { key } => self.setting_toggle(&key),
            actions::Effect::SettingValueCommit { key, value } => {
                self.setting_value_commit(&key, &value)
            }
            actions::Effect::SettingPathPick { key, path } => self.setting_path_pick(&key, &path),
            actions::Effect::SettingRangeStep { key } => self.setting_range_step(&key),
            actions::Effect::TrashAsset { rel } => self.trash_asset(rel),
            actions::Effect::KeepVersion { name } => self.keep_version(name.as_deref()),
            actions::Effect::FollowLink(url) => self.follow_link(&url),
            actions::Effect::ReportProblem => self.report_problem(),
            actions::Effect::DownloadFile => self.download_file(),
            actions::Effect::Export(format) => self.export_document(format),
            // "Check for Updates": record the local "last checked" marker (the
            // app never fetches anything itself) and open the site's own
            // check page through the same OS-handoff seam.
            actions::Effect::CheckForUpdates => self.check_for_updates(),
            actions::Effect::Persistence(effect) => self.apply_persistence_effect(effect),
            actions::Effect::Clipboard(actions::ClipboardEffect::WriteKillRing) => {
                self.sync_kill_to_clipboard()
            }
            actions::Effect::Buffer(effect) => self.apply_buffer_effect(effect),
            actions::Effect::Daemon(actions::DaemonEffect::NotifyFinished) => {
                self.notify_finished_buffer()
            }
            actions::Effect::Surface(actions::SurfaceEffect::ShowAbout) => {
                #[cfg(target_os = "macos")]
                crate::mac_about::show();
                #[cfg(not(target_os = "macos"))]
                crate::about::set_open(true);
            }
            actions::Effect::Notice(effect) => self.apply_notice_effect(effect),
            actions::Effect::Render(effect) => self.apply_render_effect(effect),
            // NOTES VERBS round: the RENAME minibuffer committed — perform the
            // actual disk rename + the one-owner path-keyed bookkeeping (refusing
            // calmly on a git-managed file or a name collision).
            actions::Effect::RenameNoteCommit { new_name } => self.rename_current_file(&new_name),
            actions::Effect::DuplicateNote => self.duplicate_current_file(),
            actions::Effect::Quit | actions::Effect::None => {}
            actions::Effect::RunAction(_)
            | actions::Effect::Clipboard(actions::ClipboardEffect::PasteImage)
            | actions::Effect::InsertDate
            | actions::Effect::OverlayAccept(_, _) => match effect {
                actions::Effect::InsertDate => self.insert_date(),
                actions::Effect::OverlayAccept(kind, value) => {
                    self.apply_overlay_accept(kind, &value)
                }
                _ => unreachable!("continuation effects handled before live interpretation"),
            },
        }
    }

    fn apply_overlay_accept(&mut self, kind: crate::overlay::OverlayKind, value: &str) {
        use crate::overlay::OverlayKind::*;
        match kind {
            Goto => self.open_rel(value),
            Project => {
                self.switch_project(PathBuf::from(value));
                self.complete_tutorial_folder_choice();
            }
            MoveDest => self.move_current_file(value),
            Caret => self.persist_caret_mode(),
            Dictionary => self.set_dictionary(crate::spell::active_variant()),
            CjkLang => self.persist_cjk_priority(),
            Date => {
                self.persist_date_format();
                self.refresh_settings_overlay();
            }
            History => self.restore_history(value),
            // The conflict workspace accepts nothing: stepping its rows already
            // changes what the comparison shows, and neither resolution is
            // reachable by pressing `↵` on a view of a manuscript. Both run from
            // named palette rows, which is what makes destroying a version an
            // act with a name rather than a keypress.
            Theme | Browse | Command | Spell | Keybindings | Settings | Assets | Rename
            | InsertLink | KeepName | Context | Conflict => {}
        }
    }

    pub(super) fn history_overlay_closed(&mut self, accepted: bool) {
        self.document.close_history(accepted);
    }

    /// C-c C-o (follow-link-at-point): hand `url` off to the OS default browser.
    /// This is a USER-INITIATED launch — the app spawns the platform opener
    /// (`open` on macOS, `xdg-open` on Linux) or `window.open` on the web — NOT a
    /// network fetch, so awl's zero-network invariant holds (exactly like the
    /// daemon spawning a process, or a shell's `$EDITOR` handoff). LIVE-APP-ONLY:
    /// this method is never reached from the headless `--keys` replay (its
    /// `Effect::FollowLink` arm is a no-op), so a capture never spawns anything.
    /// A spawn failure is logged, never fatal — following a link is best-effort.
    /// `pub(super)` so the Cmd-click mouse affordance (`app::input`) shares this one
    /// browser-handoff owner with the `Effect::FollowLink` keyboard path.
    pub(super) fn follow_link(&self, url: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let _ = w.open_with_url(url);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            #[cfg(target_os = "macos")]
            let opener = "open";
            #[cfg(all(unix, not(target_os = "macos")))]
            let opener = "xdg-open";
            #[cfg(windows)]
            let opener = "explorer";
            if let Err(e) = std::process::Command::new(opener).arg(url).spawn() {
                eprintln!("follow link: could not open {url:?}: {e}");
            }
        }
    }

    /// "Report a Problem" (Cmd-P, `native_only: false`): compose the mailto:
    /// URL LIVE — the newest crash log's path (native only; the web build has
    /// no crash-log directory) is fs state the pure core can't reach — then
    /// hand it to the SAME OS-handoff seam [`Self::follow_link`] uses. Never
    /// reads document content; the composition is a pure function
    /// (`crashlog::report_problem_mailto`) of static build metadata + a
    /// path string.
    pub(super) fn report_problem(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let crash_log_path: Option<String> = {
            let dir = crate::crashlog::crashes_dir();
            crate::crashlog::newest_log(&dir).map(|name| dir.join(name).display().to_string())
        };
        #[cfg(target_arch = "wasm32")]
        let crash_log_path: Option<String> = None;
        let meta = crate::crashlog::PanicMeta::current(None);
        let url = crate::crashlog::report_problem_mailto(&meta, crash_log_path.as_deref());
        self.follow_link(&url);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(name) = self.pending_crash.take() {
            crate::crashlog::acknowledge(&crate::crashlog::crashes_dir(), &name);
        }
    }

    /// "Download file" (WEB-ONLY, Cmd-P, `web_only: true`): export the active
    /// buffer's text as a browser download — filename from
    /// [`crate::web_export::filename_for`] (reuses `Buffer::display_name()`,
    /// never re-derived), content the buffer's plain `text()`. On native this
    /// is a documented no-op: `Action::DownloadFile` is gated off entirely by
    /// `commands::action_available` before `apply_transition` ever signals the
    /// effect that reaches here, so the `#[cfg(not(wasm32))]` arm below is
    /// structurally unreachable in the shipped native binary — it exists only
    /// so this method compiles on every platform (mirrors `follow_link`'s /
    /// `report_problem`'s own dual-`cfg` shape).
    pub(super) fn download_file(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            let filename = crate::web_export::filename_for(self.document.buffer());
            let text = self.document.buffer().text();
            crate::web_export::trigger_download(&filename, &text);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Unreachable in practice (see doc comment) — never a real disk
            // write; native has its own real save doors for this.
        }
    }

    /// EXPORT (`Effect::Export`): render the active markdown buffer to `.docx`,
    /// standalone `.html`, or native `.pdf` and land it where the user can find it
    /// — a SIBLING file beside a saved document (`doc.md` → `doc.pdf`), or into the
    /// ACTIVE folder (`self.root`) for a path-less scratch/untitled buffer. Images embedded in
    /// the export are read off the doc's own `assets/` directory through the
    /// filesystem seam (`export::FsImages`). A calm toast names the target on
    /// success; a write failure raises a sticky notice (export never crashes).
    /// On the WEB build there is no real filesystem, so DOCX/HTML bytes are handed
    /// to the browser download shim (`web_export::trigger_download_bytes`) instead;
    /// PDF has no web command or format variant.
    pub(super) fn export_document(&mut self, format: crate::export::Format) {
        let markdown = self.document.buffer().text();
        let doc_dir = self
            .document
            .buffer()
            .path()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        let images = crate::export::FsImages { doc_dir };
        let bytes = crate::export::to_bytes(&markdown, format, &images);

        #[cfg(target_arch = "wasm32")]
        {
            let name = crate::web_export::export_name(self.document.buffer(), format);
            crate::web_export::trigger_download_bytes(&name, format.mime(), &bytes);
            self.set_toast_notice(format!("downloaded {name}"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (target, show_full) = match self.document.buffer().path() {
                Some(p) => (p.with_extension(format.ext()), false),
                None => {
                    let stem = crate::web_export::export_stem(self.document.buffer());
                    (
                        self.project_location
                            .root
                            .join(format!("{stem}.{}", format.ext())),
                        true,
                    )
                }
            };
            if let Some(parent) = target.parent() {
                let _ = crate::fs::active().create_dir_all(parent);
            }
            match crate::fs::write_atomic(&target, &bytes) {
                Ok(()) => {
                    let shown = if show_full {
                        target.display().to_string()
                    } else {
                        target
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    };
                    self.set_toast_notice(format!("exported {shown}"));
                }
                Err(e) => self.set_sticky_notice(format!("export failed: {e}")),
            }
        }
    }

    /// "Check for Updates" (Cmd-P, `native_only: true`): the app never
    /// fetches anything itself. Records the LOCAL "last checked" marker
    /// (best-effort — a write failure never blocks the handoff, mirroring
    /// `crashlog::acknowledge`), then composes [`crate::updates::check_url`]
    /// (this build's own `CARGO_PKG_VERSION`, statically known — no fs read
    /// needed for the URL itself) and hands it to the SAME OS-handoff seam
    /// [`Self::follow_link`] / [`Self::report_problem`] use. Never reads
    /// document content. The marker write is native-only (mirrors
    /// `crashlog`'s own gate — the command itself is unreachable on web via
    /// `native_only: true`, so this is belt-and-suspenders, not load-bearing).
    pub(super) fn check_for_updates(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dir = crate::fs::data_root();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            crate::updates::record_checked(&dir, now);
        }
        let url = crate::updates::check_url(env!("CARGO_PKG_VERSION"));
        self.follow_link(&url);
    }

    fn apply_persistence_effect(&mut self, effect: actions::PersistenceEffect) {
        use actions::{PersistenceEffect::*, PreferenceEffect::*};
        match effect {
            Save(actions::SaveKind::Manual) => self.manual_save(),
            Save(actions::SaveKind::Finish) => self.save_finished_buffer(),
            ReviewExternalChange => self.review_external_change(),
            ResolveExternalChange(actions::Resolution::KeepMine) => self.resolve_keep_mine(),
            ResolveExternalChange(actions::Resolution::TakeTheirs) => self.resolve_take_theirs(),
            Preference(CaretMode) => self.persist_caret_mode(),
            Preference(PageMode) => self.persist_page_mode(),
            Preference(PageWidth) => self.persist_page_width(),
            Preference(PageReset) => self.persist_page_reset(),
            Preference(Outline) => {
                let on = crate::outline::outline_on();
                self.persist_pref("outline", if on { "true" } else { "false" });
            }
            Preference(MenuBar) => {
                let on = crate::menubar::menu_bar_on();
                self.persist_pref("menu_bar", if on { "true" } else { "false" });
            }
            Preference(Typewriter) => {
                let on = crate::typewriter::typewriter_on();
                self.persist_pref("typewriter_scroll", if on { "true" } else { "false" });
            }
            Preference(Spellcheck) => {
                self.persist_spellcheck();
                self.run_spellcheck_now();
            }
            Preference(WritingNits) => {
                let on = crate::nits::nits_on();
                self.persist_pref("writing_nits", if on { "true" } else { "false" });
            }
            Preference(WritingStreaks) => {
                #[cfg(not(target_arch = "wasm32"))]
                self.streaks_flush();
            }
        }
    }

    fn apply_buffer_effect(&mut self, effect: actions::BufferEffect) {
        match effect {
            actions::BufferEffect::Previous { .. } => self.last_buffer_toggle(),
            actions::BufferEffect::NewDocument => self.new_document(),
            actions::BufferEffect::OpenSettings => self.open_settings(),
            actions::BufferEffect::OpenCredits => self.open_credits(),
            actions::BufferEffect::OpenGuide => self.open_guide(),
        }
    }

    fn apply_notice_effect(&mut self, effect: actions::NoticeEffect) {
        match effect {
            actions::NoticeEffect::Toast(message) => self.set_toast_notice(message),
            actions::NoticeEffect::Sticky(message) => self.set_sticky_notice(message),
            actions::NoticeEffect::Clear => self.clear_notice(),
        }
    }

    pub(in crate::app) fn emit_notice(&mut self, effect: actions::NoticeEffect) {
        self.apply_live_effect(actions::Effect::Notice(effect));
    }

    fn apply_render_effect(&mut self, effect: actions::RenderEffect) {
        match effect {
            actions::RenderEffect::SyncView { follow } => self.sync_view(follow),
            actions::RenderEffect::Reshape => {
                if let Some(gpu) = self.frame.gpu_mut() {
                    let (w, h) = (gpu.config.width as f32, gpu.config.height as f32);
                    gpu.pipeline.set_size(w, h);
                }
            }
            actions::RenderEffect::ZoomChanged => {
                self.arm_zoom_anchor_caret();
                self.mark_zoom_dirty();
            }
            actions::RenderEffect::Redraw => {
                self.request_frame();
            }
            actions::RenderEffect::EditStreak => self.frame.set_caret_edit_streaks(true),
        }
    }

    pub(super) fn post_transition_effects(
        &mut self,
        theme_overlay_before: bool,
        theme_committed: bool,
        theme_before: crate::theme::Theme,
    ) {
        // Re-tint for the THEME picker: a live preview (overlay still open) OR a
        // commit/revert (overlay just closed) changed the active theme, so reskin
        // the baked GPU pipelines and refresh the title to the now-active world.
        // A PREVIEW re-colors instantly but DEFERS the font reshape until the
        // selection settles (`retint_theme_preview`); a COMMIT/REVERT applies the
        // full switch synchronously and cancels any pending deferral, so Esc can
        // never leave a stray reshape to land after the picker closed.
        if theme_committed || (theme_overlay_before && !self.workspace_state.overlay_open()) {
            self.retint_theme_now();
        } else if theme_overlay_before {
            self.retint_theme_preview(theme_before);
        }
        // STICKY THEME write-on-change: persist ONLY on the picker's COMMIT/revert
        // (`theme_committed`), never on a live PREVIEW (`theme_overlay_before` while
        // the picker is still open) — so scrolling through worlds doesn't hammer the
        // disk; the SETTLED choice is what's remembered for next launch.
        //
        // THE DOCK ICON rides this exact guard, for the same reason and one more:
        // a preview must not churn the Dock. Arrowing or sweeping the pointer
        // through twenty worlds re-tints pipelines through
        // `retint_theme_preview`, which has no route to `app_icon` at all — so the
        // "no churn" property is structural, not a rule anybody has to remember.
        // The two doors that DO adopt are both settled states: this commit/revert,
        // and startup after the sticky theme has been restored (`app.rs`'s
        // `resumed`). See `app_icon`'s module doc.
        if theme_committed {
            self.persist_theme();
            #[cfg(not(target_arch = "wasm32"))]
            crate::app_icon::adopt(&crate::theme::active());
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "apply_tests.rs"]
mod tests;
