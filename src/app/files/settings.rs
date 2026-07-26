//! src/app/files/settings.rs — the STICKY-PREFERENCE writes (theme/zoom/
//! page/caret/spellcheck/…, all through the ONE `persist_pref` owner), the
//! Settings-menu toggle/value/path-pick doors, the sticky page-width pair,
//! and the live config reload. Split out of the former `app/files.rs`
//! monolith (item 56); dictionary-specific persistence lives in
//! `files/dictionary.rs`, the rebind-menu capture in `files/rebind.rs` (both
//! peeled out to stay under the ~500-line ceiling). Item 76 retired the
//! `project_root` config key — the active folder is now remembered by the
//! session's one owner (`app/session.rs::session_flush`), not written here.

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
        // Keep the in-memory config in step with the file so it stays the source of
        // truth between explicit reloads.
        match key {
            "theme" => self.config.theme = Some(value.trim_matches('"').to_string()),
            "caret_mode" => self.config.caret_mode = Some(value.trim_matches('"').to_string()),
            "dictionary" => self.config.dictionary = Some(value.trim_matches('"').to_string()),
            "page_mode" => self.config.page_mode = Some(value == "true"),
            "page_width_prose" => self.config.page_width_prose = value.parse().ok(),
            "page_width_code" => self.config.page_width_code = value.parse().ok(),
            "zoom" => self.config.zoom = value.parse().ok(),
            "writing_nits" => self.config.writing_nits = Some(value == "true"),
            "spellcheck" => self.config.spellcheck = Some(value == "true"),
            // Settings-menu TOGGLES that were previously write-only (no mirror): keep
            // `self.config` in step with disk so the still-open menu's value cell
            // (read from `self.config` for the mechanism-B keys) and a later
            // conflict/reload check both see the current value.
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
            // FILE VISIBILITY (item 77): a plain bool key, like the toggles above.
            "file_visibility" => self.config.file_visibility = Some(value == "true"),
            // KEYMAP FLAVOR: a quoted string ("native"/"emacs"), not a bool — mirrors
            // "theme"/"caret_mode"/"dictionary" above, not the bool toggles.
            "keymap" => self.config.keymap = Some(value.trim_matches('"').to_string()),
            // DATE FORMAT: a quoted slug ("ddmmyy"/"mmddyy"/"iso"/"yyyymmdd"/
            // "dmonthyyyy"), not a bool — mirrors "keymap"/"caret_mode" above.
            "date_format" => self.config.date_format = Some(value.trim_matches('"').to_string()),
            // The CJK ladder is written as a whole TOML array (see
            // `persist_cjk_priority`); the mirror reads the LIVE process global
            // (already updated by the picker's core-level accept) rather than
            // re-parsing the formatted `value` string back into a `Vec<Lang>`.
            "cjk_priority" => self.config.cjk_priority = Some(crate::frontmatter::cjk_priority()),
            _ => {}
        }
    }

    /// Persist the now-active THEME name (write-on-change after a theme commit/revert).
    pub(in crate::app) fn persist_theme(&mut self) {
        let name = crate::theme::active().name;
        self.persist_pref("theme", &format!("\"{name}\""));
    }

    /// Persist the now-active PAGE MODE (write-on-change after a page-mode toggle).
    pub(in crate::app) fn persist_page_mode(&mut self) {
        let on = crate::page::page_on();
        self.persist_pref("page_mode", if on { "true" } else { "false" });
    }

    /// Persist the now-active SPELLCHECK on/off (write-on-change after "Toggle
    /// Spellcheck"). Mirrors `persist_page_mode` / the writing-nits persist call.
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
    pub(in crate::app) fn setting_toggle(&mut self, key: &str) {
        // KEYMAP is NOT a plain bool config key (its value is "native"/"emacs", not
        // "true"/"false"), so it can't ride the generic bool mechanism below —
        // special-cased here, before the generic `now`/`next` match, and handled
        // by its own dedicated door (`toggle_keymap_flavor`).
        if key == "keymap" {
            self.toggle_keymap_flavor();
            return;
        }
        // DATE FORMAT is a 5-way CYCLE, not a bool, so it can't ride the
        // generic mechanism below either — special-cased here exactly like
        // "keymap", before the generic `now`/`next` match.
        if key == "date_format" {
            self.cycle_date_format();
            return;
        }
        // Read the CURRENT value from the SAME owner the readout reads, then negate.
        let now = match key {
            "page_mode" => crate::page::page_on(),
            "typewriter_scroll" => crate::typewriter::typewriter_on(),
            "wysiwyg" => crate::markdown::wysiwyg_on(),
            "popover" => crate::popover::popover_on(),
            "inline_images" => crate::markdown::inline_images_on(),
            "code_ligatures" => crate::render::code_ligatures_on(),
            "spellcheck" => crate::spell::spellcheck_on(),
            "writing_nits" => crate::nits::nits_on(),
            "autosave" => self.config.autosave_on(),
            "history" => self.config.history_on(),
            "session_restore" => self.config.session_restore_on(),
            "outline" => crate::outline::outline_on(),
            "menu_bar" => crate::menubar::menu_bar_on(),
            "reduce_motion" => crate::motion::reduced(),
            "file_visibility" => crate::file_visibility::all_on(),
            _ => return, // unknown key: a calm no-op
        };
        let next = !now;
        // (a) Apply the mechanism-A process-globals LIVE so the flip renders. wysiwyg
        //     / inline_images are the two that had NO live-apply path before this seam.
        match key {
            "page_mode" => crate::page::set_page_on(next),
            "typewriter_scroll" => crate::typewriter::set_typewriter_on(next),
            "wysiwyg" => crate::markdown::set_wysiwyg_on(next),
            "popover" => crate::popover::set_popover_on(next),
            "inline_images" => crate::markdown::set_inline_images_on(next),
            "code_ligatures" => crate::render::set_code_ligatures_on(next),
            "spellcheck" => crate::spell::set_spellcheck_on(next),
            "writing_nits" => crate::nits::set_nits_on(next),
            "outline" => crate::outline::set_outline_on(next),
            // ACCESSIBILITY TIER 1: an explicit toggle wins over `auto` from
            // here on — this is a deliberate user action, not a live OS-pref
            // poll (see `motion.rs`'s module doc). Any glide/flinch already in
            // flight settles on its very next step (the gate lives in
            // `advance`'s three callees; nothing further to force here).
            "reduce_motion" => crate::motion::set_reduced(next),
            "menu_bar" => crate::menubar::set_menu_bar_on(next),
            "file_visibility" => crate::file_visibility::set_all_on(next),
            _ => {} // mechanism-B: config-only, applied on read
        }
        // (b) Persist the negated value (the mirror-match keeps `self.config` in step).
        self.persist_pref(key, if next { "true" } else { "false" });
        // (c) Reshape / rescan / repaint as the flipped global demands.
        match key {
            // A page-column / conceal / image change: re-wrap (page mode) + let the
            // next frame's wysiwyg/inline latch restyle the conceal, then re-push.
            "page_mode" | "wysiwyg" | "inline_images" => {
                if let Some(gpu) = self.gpu.as_mut() {
                    let (w, h) = (gpu.config.width as f32, gpu.config.height as f32);
                    gpu.pipeline.set_size(w, h);
                }
                self.sync_view(true);
            }
            // A font-feature change (ligatures) alters `doc_attrs` but neither the
            // text nor the wrap column, so the incremental set_view would skip the
            // reshape — force it (set_view's `force` reshapes with the fresh attrs).
            "code_ligatures" => self.sync_view(true),
            // Squiggles vanish/reappear this frame (mirrors `ToggleSpellcheck`).
            "spellcheck" => self.run_spellcheck_now(),
            // Render-only nit highlighter (mirrors `Action::ToggleWritingNits`).
            "writing_nits" => self.sync_view(false),
            // Render-only margin outline (mirrors `writing_nits`): repaint so the
            // outline appears/vanishes this frame (the draw lands next phase).
            "outline" => self.sync_view(false),
            // The menu bar reserves vertical space via `doc_top`, so re-sync WITH
            // follow to re-inset the document below (or reclaim) the bar strip THIS
            // frame — mirrors the ToggleMenuBar apply arm.
            "menu_bar" => self.sync_view(true),
            // Scroll-only typewriter pin: re-sync with follow so the caret's row
            // re-centers (or reverts to cursor-follow) THIS frame — the cursor-follow
            // in `sync_view` now reads the flipped global.
            "typewriter_scroll" => self.sync_view(true),
            _ => {}
        }
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        // (d) Refresh the still-open menu's value cell in place.
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
    /// HERE (the live-only half of the seam — `apply_core` never touches a
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
        if let Some(ov) = self.overlay.as_mut()
            && ov.kind == crate::overlay::OverlayKind::Settings
        {
            ov.set_secondaries(crate::settings::visible_value_cells(&values));
            // ITEM 94: the rail thumbs are refreshed from the SAME gathered
            // values as the number beside them, in the same call.
            ov.set_range_cells(crate::settings::visible_range_cells(&values));
        }
    }

    /// SETTINGS MENU inline VALUE commit (Enter on a `SettingKind::Value` row): parse
    /// the typed `raw` for config `key`, CLAMP it to that setting's sane range, apply
    /// it LIVE, and PERSIST the NAMED key — then refresh the still-open menu's cell.
    /// Unlike the drag / `C-x {` write (`persist_page_width`, which targets the ACTIVE
    /// buffer's class), the row NAMES its class, so we write exactly `key` and re-sync
    /// through the ONE `sync_page_measure` owner (which applies live iff that class is
    /// the active buffer's — editing the code width while a `.md` is open is persisted
    /// but not visibly re-wrapped, correctly). An unparseable value is a calm no-op
    /// (the cell reverts on the next refresh). Zoom rides the SAME `set_zoom` +
    /// `settle_zoom_persist` path the wheel / ⌘± owner settles through.
    pub(in crate::app) fn setting_value_commit(&mut self, key: &str, raw: &str) {
        match key {
            "page_width_prose" | "page_width_code" => {
                if let Ok(n) = raw.trim().parse::<usize>() {
                    let clamped = crate::settings::clamp_page_width(n);
                    // Persist the NAMED key (the mirror-match keeps `self.config` in
                    // step), then re-resolve the measure for the active buffer's class.
                    self.persist_pref(key, &clamped.to_string());
                    self.sync_page_measure();
                    self.sync_view(true);
                }
            }
            "zoom" => {
                if let Some(z) = crate::settings::parse_zoom(raw) {
                    self.set_zoom(z); // clamps + re-metrics next sync (the ⌘± owner)
                    // A typed commit is a DISCRETE end: settle at once through the one
                    // owner, which also disarms the debounce `set_zoom` just armed (it
                    // would otherwise write the same value a second time 500ms later).
                    self.settle_zoom_persist();
                    self.sync_view(true);
                }
            }
            _ => {}
        }
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        self.refresh_settings_overlay();
    }

    /// ITEM 94 — SETTINGS MENU RANGE step (Left/Right on a rail row): the CORE
    /// already stepped the live scalar through the range spec and mirrored the new
    /// readout + thumb into the still-open menu, so this owns only the LIVE TAIL:
    /// re-metric/reflow for the new value, then the DISCRETE sticky persist — one
    /// `persist_pref` write per authored step, the same "write on every discrete
    /// commit" path a Toggle takes (deliberately NOT the debounced wheel/⌘± path;
    /// key repeat already throttles this to human speed).
    ///
    /// The one owner of the range settings' live-side wiring, keyed by config key —
    /// `range_persist` writes it, `range_apply_live` (the POINTER path) applies it.
    pub(in crate::app) fn setting_range_step(&mut self, key: &str) {
        match key {
            // ZOOM: the value is already in `self.zoom` (mirrored back from the core
            // right after `apply_core`); queue the metric reflow the ⌘± path queues.
            "zoom" => self.zoom_reflow.queue(),
            _ => {}
        }
        self.range_persist(key);
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        self.refresh_settings_overlay();
    }

    /// ITEM 94 — THE ONE LIVE-APPLY DOOR for a range setting's POINTER path (a rail
    /// click, and every resolved step of a drag): apply `value` — already quantized
    /// by the spec, never a raw pointer number — through the setting's own live
    /// owner. NEVER persists: a drag writes config exactly once, on release
    /// ([`Self::range_persist`], from `end_range_drag`).
    pub(in crate::app) fn range_apply_live(&mut self, id: crate::settings::SettingId, value: f32) {
        match id {
            // The SAME `set_zoom` owner the ⌘± / ⌘-wheel doors use (it re-clamps
            // through `clamp_zoom` -> the same spec, so this is idempotent).
            crate::settings::SettingId::Zoom => self.set_zoom(value),
            _ => {}
        }
    }

    /// ITEM 94 — THE ONE PERSIST DOOR for a range setting (a keyboard step, or a
    /// drag RELEASE). Settles the gesture through that setting's own settle owner —
    /// for zoom, [`Self::settle_zoom_persist`], the EXACT call the quiet-window
    /// debounce makes, so a rail release and a ⌘± run end identically (one write, the
    /// debounce disarmed, the floating readout dropped). A whole drag therefore costs
    /// exactly one config write, with nothing trailing behind it.
    pub(in crate::app) fn range_persist(&mut self, key: &str) {
        if key == "zoom" {
            self.settle_zoom_persist()
        }
    }

    /// SETTINGS MENU path pick (the folder navigator opened from a `SettingKind::Path`
    /// row accepted a folder): write the NAMED config key `key` for `path`. For
    /// `project_root` this IS a genuine switch-project (re-index + the session's
    /// one active-folder-context owner + recent-MRU, the ONE `switch_project`
    /// owner — item 76 retired the separate `project_root` config key it used to
    /// write); for `default_folder`/`workspace` we persist the key then
    /// `reload_config`, which re-folds `self.default_folder`/`self.workspace`
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

    /// Persist the now-active CARET MODE (write-on-change after a caret-mode change).
    /// Phase 2 relies on this seam to remember the caret style across launches.
    pub(in crate::app) fn persist_caret_mode(&mut self) {
        let name = crate::config::caret_mode_name(crate::caret::mode());
        self.persist_pref("caret_mode", &format!("\"{name}\""));
    }

    /// Persist the now-active DATE FORMAT (write-on-change after the Date-format
    /// picker commits) — mirrors `persist_caret_mode`. The core already set the
    /// process-global (`dateformat::set_active_format`) in the picker accept, so
    /// this only writes the sticky slug (the SAME quoted `date_format` RHS
    /// `cycle_date_format` wrote before the row became a picker).
    pub(in crate::app) fn persist_date_format(&mut self) {
        let slug = crate::dateformat::active_format().config_name();
        self.persist_pref("date_format", &format!("\"{slug}\""));
    }

    /// Write the CURRENT zoom to config. The raw write alone — every caller reaches it
    /// through [`Self::settle_zoom_persist`], which owns the surrounding bookkeeping.
    /// Trims the float to 3 places so the file stays tidy.
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

    /// Live-reload after the config file is SAVED in the editor: re-read it, rebuild
    /// the keymap overrides, and re-fold default_folder/workspace (flag > config >
    /// default, so a CLI flag still wins). A bad chord keeps its default + prints a
    /// note inside `apply_overrides`; nothing here can crash. `default_folder` only
    /// ever matters again on a FUTURE first-run launch (item 76 — it is not the
    /// active folder); the workspace change affects the NEXT C-x p; the keymap
    /// change is immediate.
    ///
    /// SPELLCHECK and the PAGE-WIDTH pair (`page_width_prose`/`page_width_code`)
    /// are ALSO re-applied here (unlike the other sticky prefs — theme / page /
    /// caret / writing_nits / dictionary — which apply ONCE at launch via
    /// `apply_sticky_globals` and otherwise only change via their own live
    /// toggle): a hand-edited `spellcheck = false` saved straight into the config
    /// buffer takes effect immediately, exactly like using the "Toggle
    /// Spellcheck" palette command, and the rescan below clears/restores
    /// squiggles in the SAME frame rather than waiting for the next text
    /// edit to trip the eager rescan. Likewise, hand-editing `page_width_code`
    /// while a `.rs` file is
    /// open re-wraps it immediately (`sync_page_measure`), since the config alone
    /// (not a live toggle) is the only way to change either key's OVERRIDE value.
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
        self.default_folder = crate::resolve_default_folder(
            &self
                .cli_default_folder
                .clone()
                .or_else(|| cfg.default_folder.clone()),
        );
        let workspace_opt = self.cli_workspace.clone().or_else(|| cfg.workspace.clone());
        self.workspace = Some(crate::resolve_workspace(&workspace_opt, &self.root));
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
        // STICKY PAGE WIDTH: an edited `page_width_prose`/`page_width_code` takes
        // effect immediately too, re-resolved against the buffer that is CURRENTLY
        // active (its kind is unchanged by a config reload; only the configured
        // override might be).
        self.sync_page_measure();
        self.run_spellcheck_now();
    }
}
