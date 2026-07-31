use crate::app::*;

impl App {
    /// WRITE-ON-CHANGE for a STICKY PREFERENCE (theme/zoom/page_mode/caret_mode):
    /// persist the settled value to config.toml format-preservingly (reusing the
    /// rebind menu's surgical [`Config::write_pref`] — comments + `[keys]` + the
    /// other prefs survive) and mirror it into the in-memory [`Self::config`] so a
    /// later live reload / conflict check sees the current value. A no-op when there
    /// is no resolvable config path (e.g. no HOME), and silent on a write error (a
    /// failed remember must never disrupt the edit). `value` is the formatted RHS.
    pub(in crate::app) fn persist_pref(&mut self, key: &str, value: &str) {
        let path = self.config.path.clone();
        if path.as_os_str().is_empty() {
            return; // no config path (no HOME): nothing to remember
        }
        if let Err(e) = Config::write_pref(&path, key, value) {
            eprintln!("could not persist {key} to {}: {e}", path.display());
            return;
        }
        match key {
            "theme" => self.config.theme = Some(value.trim_matches('"').to_string()),
            "caret_mode" => self.config.caret_mode = Some(value.trim_matches('"').to_string()),
            "dictionary" => self.config.dictionary = Some(value.trim_matches('"').to_string()),
            "page_mode" => self.config.page_mode = Some(value == "true"),
            "page_width_prose" => self.config.page_width_prose = value.parse().ok(),
            "page_width_code" => self.config.page_width_code = value.parse().ok(),
            "zoom" => self.config.zoom = value.parse().ok(),
            "scroll_sensitivity" => self.config.scroll_sensitivity = value.parse().ok(),
            "writing_nits" => self.config.writing_nits = Some(value == "true"),
            "spellcheck" => self.config.spellcheck = Some(value == "true"),
            "autosave" => self.config.autosave = Some(value == "true"),
            "history" => self.config.history = Some(value == "true"),
            "session_restore" => self.config.session_restore = Some(value == "true"),
            "wysiwyg" => self.config.wysiwyg = Some(value == "true"),
            "popover" => self.config.popover = Some(value == "true"),
            "inline_images" => self.config.inline_images = Some(value == "true"),
            "code_ligatures" => self.config.code_ligatures = Some(value == "true"),
            "outline" => self.config.outline = Some(value == "true"),
            "menu_bar" => self.config.menu_bar = Some(value == "true"),
            "reduce_motion" => self.config.reduce_motion = Some(value == "true"),
            "file_visibility" => self.config.file_visibility = Some(value == "true"),
            "keymap" => self.config.keymap = Some(value.trim_matches('"').to_string()),
            "date_format" => self.config.date_format = Some(value.trim_matches('"').to_string()),
            "cjk_priority" => self.config.cjk_priority = Some(crate::frontmatter::cjk_priority()),
            _ => {}
        }
    }

    pub(in crate::app) fn persist_theme(&mut self) {
        let name = crate::theme::active().name;
        self.persist_pref("theme", &format!("\"{name}\""));
    }

    pub(in crate::app) fn persist_page_mode(&mut self) {
        let on = crate::page::page_on();
        self.persist_pref("page_mode", if on { "true" } else { "false" });
    }

    pub(in crate::app) fn persist_spellcheck(&mut self) {
        let on = crate::spell::spellcheck_on();
        self.persist_pref("spellcheck", if on { "true" } else { "false" });
    }

    /// SETTINGS MENU toggle (Enter on a `SettingKind::Toggle` row): flip the sticky
    /// boolean `key`, apply it LIVE this frame, PERSIST the negated value, then
    /// refresh the STILL-OPEN menu's value cell. Two mechanisms:
    ///   * PROCESS-GLOBAL (page_mode / wysiwyg / inline_images / spellcheck /
    ///     writing_nits) — flip the shared global so the renderer picks it up, then
    ///     reshape / rescan / repaint as that global demands (this is the seam that
    ///     closes the WYSIWYG live-apply gap: `set_wysiwyg_on` fires HERE, and the
    ///     pipeline's per-frame wysiwyg/inline latch — see `render.rs` `set_view` —
    ///     forces the conceal restyle the incremental text diff would otherwise skip);
    ///   * PROCESS-GLOBAL (outline) — flip `outline::OUTLINE_ON` so the renderer
    ///     picks up the margin outline this frame, then repaint (like writing_nits).
    ///   * CONFIG-ONLY (autosave / history / session_restore) — no global;
    ///     persisting the flipped value into `self.config` is enough (they are read
    ///     live from the config on demand).
    ///     Persistence rides the ONE `persist_pref` owner (its mirror-match now covers
    ///     every key here), so there is no bespoke per-toggle writer to drift.
    ///
    /// The read/negate/set core itself (item 193) lives in ONE place,
    /// [`crate::settings::flip_toggle_global`], shared with the replay
    /// interpreter (`main/run/settings_effects.rs::interpret_setting_toggle`)
    /// — this method keeps only what is genuinely App's: the `keymap`/
    /// `date_format` special cases (neither is a boolean flip) and the LIVE
    /// tail below (gpu resize, `sync_view`, `run_spellcheck_now`), none of
    /// which a headless replay has a pipeline for.
    pub(in crate::app) fn setting_toggle(&mut self, key: &str) {
        if key == "keymap" {
            self.toggle_keymap_flavor();
            return;
        }
        if key == "date_format" {
            self.cycle_date_format();
            return;
        }
        let Some(next) = crate::settings::flip_toggle_global(key, &self.config) else {
            return; // unknown key: a calm no-op
        };
        self.persist_pref(key, if next { "true" } else { "false" });
        match key {
            "page_mode" | "wysiwyg" | "inline_images" => {
                if let Some(gpu) = self.gpu.as_mut() {
                    let (w, h) = (gpu.config.width as f32, gpu.config.height as f32);
                    gpu.pipeline.set_size(w, h);
                }
                self.sync_view(true);
            }
            "code_ligatures" => self.sync_view(true),
            "spellcheck" => self.run_spellcheck_now(),
            "writing_nits" => self.sync_view(false),
            "outline" => self.sync_view(false),
            "menu_bar" => self.sync_view(true),
            "typewriter_scroll" => self.sync_view(true),
            _ => {}
        }
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        self.refresh_settings_overlay();
    }

    /// THE DATE-FORMAT CYCLE (Enter on the Settings menu's "Date format" row):
    /// step [`crate::dateformat::active_format`] to the NEXT of the five
    /// formats ([`crate::dateformat::DateFormat::cycle_next`], wrapping
    /// DD/MM/YY -> MM/DD/YY -> ISO -> YYYY/MM/DD -> D Month YYYY -> back),
    /// apply it LIVE (the process-global every reader — Insert Date, the
    /// row's own value cell, a future capture — consults), PERSIST it (a
    /// quoted slug, like `keymap`/`caret_mode`), then refresh the still-open
    /// menu so the secondary column's live TODAY preview updates immediately.
    /// Mirrors [`Self::toggle_keymap_flavor`]'s exact shape (the same "not a
    /// bool, special-cased before the generic match" seam `setting_toggle`
    /// routes both through), minus the keymap rebuild this doesn't need.
    pub(in crate::app) fn cycle_date_format(&mut self) {
        let next = crate::dateformat::active_format().cycle_next();
        crate::dateformat::set_active_format(next);
        self.persist_pref("date_format", &format!("\"{}\"", next.config_name()));
        self.refresh_settings_overlay();
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    /// "Insert Date" (`Effect::InsertDate`): insert TODAY'S date at the caret,
    /// formatted per the active [`crate::dateformat::DateFormat`], as ONE
    /// undoable edit (`Buffer::insert_text` — sealed on both sides, so it
    /// never coalesces with adjacent typing). The real wall clock is read
    /// HERE (the live-only half of the seam — `apply_transition` never touches a
    /// clock); the headless `--keys` replay's own `Effect::InsertDate` arm
    /// performs the identical insert against the FIXED
    /// [`crate::dateformat::CAPTURE_PLACEHOLDER_YMD`] instead, so only the
    /// DATE differs between the two, never the mechanism.
    pub(in crate::app) fn insert_date(&mut self) {
        let fmt = crate::dateformat::active_format();
        let (y, m, d) = crate::dateformat::today_from_system_clock();
        self.active.buffer.insert_text(&fmt.format(y, m, d));
    }

    /// After a settings toggle, rebuild the STILL-OPEN settings menu's value cells in
    /// place (mirrors [`Self::refresh_rebind_overlay`]): re-gather the config/project
    /// values so the flipped row's SECONDARY column reflects the new state (the
    /// process-globals are re-read live inside the readout). A no-op if the settings
    /// menu isn't the open overlay. Reads through [`crate::settings::visible_value_cells`]
    /// — the SAME platform-filtered view `overlay::build`'s own `OverlayKind::Settings`
    /// branch seeds each row's `secondary` from (via `set_secondaries`) — never the raw
    /// unfiltered [`crate::settings::value_cells`]; on native the two coincide (nothing
    /// is filtered), but a refresh must stay index-coherent with `ov.rows`
    /// (`visible_names()`) even on web, where "Edit config as text" is hidden.
    pub(in crate::app) fn refresh_settings_overlay(&mut self) {
        let values = crate::settings::SettingsValues::gather(
            &self.config,
            &self.root,
            self.zoom,
            crate::dateformat::today_from_system_clock(),
        );
        if let Some(ov) = self.workspace_state.overlay_mut()
            && ov.kind == crate::overlay::OverlayKind::Settings
        {
            ov.set_secondaries(crate::settings::visible_value_cells(&values));
            ov.set_range_cells(crate::settings::visible_range_cells(&values));
        }
    }

    /// SETTINGS MENU path pick (the folder navigator opened from a `SettingKind::Path`
    /// row accepted a folder): write the NAMED config key `key` for `path`. For
    /// `project_root` this IS a genuine switch-project (re-index + the session's
    /// one active-folder-context owner + recent-MRU, the ONE `switch_project`
    /// owner — item 76 retired the separate `project_root` config key it used to
    /// write); for `default_folder`/`workspace` we persist the key then
    /// `reload_config`, which re-folds `self.default_folder`/`self.workspace_root`
    /// (flag > config > default) so the NEXT first-run launch / `C-x p` uses the
    /// new folder. Either way the still-open (re-summoned) menu's cell is
    /// refreshed.
    pub(in crate::app) fn setting_path_pick(&mut self, key: &str, path: &str) {
        match key {
            "project_root" => self.switch_project(PathBuf::from(path)),
            "default_folder" | "workspace" => {
                self.persist_pref(key, &format!("\"{path}\""));
                self.reload_config();
            }
            _ => {}
        }
        self.refresh_settings_overlay();
    }

    /// The config key naming the sticky page-width pref for `class` — the ONE
    /// owner every persist/reset/resync call routes the class->key mapping
    /// through, so it can never drift between them.
    fn page_width_key(class: crate::page::PageClass) -> &'static str {
        match class {
            crate::page::PageClass::Prose => "page_width_prose",
            crate::page::PageClass::Code => "page_width_code",
        }
    }

    /// Persist the now-active PAGE WIDTH / measure (write-on-change after a Page wider
    /// / Page narrower command, or a page-column drag release) to the key matching
    /// the ACTIVE buffer's KIND (`page_width_prose` vs `page_width_code` — see
    /// [`crate::page::PageClass`]), so widening a `.rs` file never bleeds into the
    /// prose measure a `.md` file reads. Zoom-independent: remembers the COLUMN
    /// width, not the glyph size (zoom has its own sticky pref).
    pub(in crate::app) fn persist_page_width(&mut self) {
        let w = crate::page::measure();
        let key = Self::page_width_key(self.active.buffer.page_class());
        self.persist_pref(key, &w.to_string());
    }

    /// "Reset page width" WRITE-ON-CHANGE: CLEAR the sticky override MATCHING the
    /// active buffer's KIND entirely (format-preserving removal,
    /// [`Config::remove_pref`]) rather than writing that class's default measure
    /// back — the `Option` already means "built-in default", so a future
    /// [`crate::page::PageClass::default_measure`] change flows through instead of
    /// pinning a stale value. Never touches the OTHER class's override. A no-op
    /// when there is no resolvable config path (e.g. no HOME), and silent on a
    /// write error, mirroring `persist_pref`.
    pub(in crate::app) fn persist_page_reset(&mut self) {
        let path = self.config.path.clone();
        if path.as_os_str().is_empty() {
            return; // no config path (no HOME): nothing to remember
        }
        let class = self.active.buffer.page_class();
        let key = Self::page_width_key(class);
        if let Err(e) = Config::remove_pref(&path, key) {
            eprintln!("could not clear {key} in {}: {e}", path.display());
            return;
        }
        match class {
            crate::page::PageClass::Prose => self.config.page_width_prose = None,
            crate::page::PageClass::Code => self.config.page_width_code = None,
        }
    }

    /// Re-apply the STICKY PAGE-WIDTH MEASURE for the ACTIVE buffer's KIND — the
    /// buffer OPEN/SWITCH half of the prose/code split (see
    /// [`crate::page::PageClass`]): a prose document (markdown / no-path /
    /// unrecognized) reads `page_width_prose`, a recognized code file reads
    /// `page_width_code`, each falling back to its own built-in default when
    /// unconfigured ([`Config::measure_for`]). Called after every buffer swap
    /// (`load_path`, `new_document`) and after a live config reload, so opening a
    /// `.rs` after a `.md` (or back) always shows THAT file's own measure — never
    /// a value carried over from whatever was active before.
    ///
    /// Mirrors the exact re-wrap dance `Action::PageWider`/`TogglePageMode`
    /// already do in `apply.rs`: force `set_size` (which re-derives the wrap
    /// width from the now-updated `page::measure()` and invalidates `row_geom` on
    /// an actual change — see `TextPipeline::set_size`) so the very next
    /// cursor-follow scroll computation reads FRESH row geometry instead of a
    /// stale pre-switch layout for one frame. (The per-frame `sync_wrap_width`
    /// invariant in `prepare` would eventually self-correct on its own, but only
    /// on the NEXT drawn frame — this keeps the switch itself glitch-free. A
    /// no-op pre-GPU-init, since `set_size` only runs when `self.gpu` exists.)
    pub(in crate::app) fn sync_page_measure(&mut self) {
        let target = self.config.measure_for(self.active.buffer.page_class());
        crate::page::set_measure(target);
        if let Some(gpu) = self.gpu.as_mut() {
            let (w, h) = (gpu.config.width as f32, gpu.config.height as f32);
            gpu.pipeline.set_size(w, h);
        }
    }

    pub(in crate::app) fn persist_caret_mode(&mut self) {
        let name = crate::config::caret_mode_name(crate::caret::mode());
        self.persist_pref("caret_mode", &format!("\"{name}\""));
    }

    pub(in crate::app) fn persist_date_format(&mut self) {
        let slug = crate::dateformat::active_format().config_name();
        self.persist_pref("date_format", &format!("\"{slug}\""));
    }

    fn persist_zoom_now(&mut self) {
        let z = self.zoom;
        self.persist_pref("zoom", &format!("{z:.3}"));
    }

    /// THE ONE OWNER of "the zoom gesture ended — write it": persist the settled value,
    /// disarm the debounce stamp, and drop the floating zoom readout the gesture armed
    /// (`mark_zoom_dirty` re-arms it on every step), parking its label off-screen again.
    ///
    /// Exactly two doors call it, one per shape of gesture ending:
    /// - `about_to_wait`'s quiet window — the ⌘± / ⌘0 / ⌘-wheel path, which has no end
    ///   EVENT, so ~500 ms of silence is inferred as the end (gated by
    ///   [`App::zoom_persist_held`], which is what keeps that inference off while a
    ///   gesture that DOES have an end is in flight);
    /// - [`Self::range_persist`] — the Settings rail's button release and its discrete
    ///   keyboard step, which end explicitly and pay their single write right there.
    ///
    /// Folding the bookkeeping in here is the point: when the rail's release cancelled
    /// the debounce by hand it also inherited the duty to clear the readout, and didn't
    /// — a released scrub left a stale percentage floating over the card. One owner, one
    /// settle. The redraw request mirrors the sibling debounces: it lets the
    /// `RedrawRequested` handler re-decide control flow (Wait, now that this is settled)
    /// instead of leaving an elapsed `WaitUntil` to busy-spin the loop (DESIGN §6).
    pub(in crate::app) fn settle_zoom_persist(&mut self) {
        self.zoom_persist_at = None;
        self.persist_zoom_now();
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.pipeline.set_zoom_readout(None);
            gpu.window.request_redraw();
        }
    }

    pub(in crate::app) fn reload_config(&mut self) {
        let cfg = Config::load(self.config.path.clone());
        let mut keys_with_web_alt = cfg.keys.clone();
        keys_with_web_alt.extend(crate::commands::web_alternate_keys(
            &cfg.keys,
            crate::convention::Convention::current(),
            crate::commands::Platform::current(),
        ));
        self.keymap.apply_overrides(&keys_with_web_alt);
        self.keymap.apply_linux_keep(&cfg.effective_linux_keep());
        // CACHE-KEY DISCIPLINE with `Config::apply_sticky_globals`: an ABSENT
        // key must leave the global AS-IS (the built-in default already
        // carries it), never force it back to ON. The old `unwrap_or(true)`
        // broke that — reachable if a prior `persist_spellcheck`/
        // `setting_toggle` write ever failed to land on disk (I/O error, no
        // resolvable config path) while the runtime toggle sat OFF: the very
        // next config-buffer save or Keybindings rebind (both route through
        // this fn) would silently flip a still-intended-OFF toggle back ON.
        // Mirrors `apply_sticky_globals_restores_spellcheck`'s law ("absent
        // pref leaves the global as-is") — see
        // `reload_config_absent_spellcheck_key_leaves_global_untouched`.
        if let Some(on) = cfg.spellcheck {
            crate::spell::set_spellcheck_on(on);
        }
        self.config = cfg;
        self.default_folder = crate::resolve_default_folder(
            &self
                .cli_default_folder
                .clone()
                .or_else(|| self.config.default_folder.clone()),
        );
        self.resync_project_location(); // root's unchanged; config.workspace may not be
        self.sync_page_measure();
        self.run_spellcheck_now();
    }
}
