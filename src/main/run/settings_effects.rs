//! ITEM 190 — the settings trio's capability grant, mirroring item 171's shape
//! for `save`/`finish_save`: `SettingToggle`, `SettingValueCommit` and
//! `SettingPathPick` are typed-effect requests a headless replay can only
//! perform for real once its caller hands it [`crate::replay::
//! FilesystemCapability::Isolated`] (`replay::classify_for`, `src/replay.rs`,
//! is the one owner of WHETHER that promotion happens; this file is the
//! INTERPRETATION half — what actually flips and what actually gets written).
//!
//! Each handler below mirrors the LIVE `App` door it has no access to
//! (`app/files/settings.rs::setting_toggle`/`setting_path_pick`,
//! `app/files/range_settings.rs::setting_value_commit`) key for key, calling
//! the SAME process-global setters and the SAME [`crate::config::Config::
//! write_pref`] writer those doors call — so the two doors can never disagree
//! about WHAT a key does, only about which capability is doing it. One key,
//! `"keymap"`, is never dispatched here: `replay::classify_for` keeps it
//! Unsupported even under Isolated (flipping keymap flavor needs a LIVE
//! keymap rebuild so a LATER chord in the same replay resolves against the
//! new flavor, a capability this session does not own — the identical reason
//! `RebindCommit`/`RebindReset` stay Unsupported).

use super::*;
use crate::settings::SettingId;

impl<'a> ReplaySession<'a> {
    /// Write `key = value` into the config path this session launched with,
    /// through the isolated filesystem its caller granted — a no-op when
    /// there is no resolvable config path (no HOME), mirroring `App::
    /// persist_pref`'s own guard. A write error is swallowed the same way: a
    /// failed remember must never abort a replay that is otherwise honestly
    /// Applied.
    fn persist_setting(&self, key: &str, value: &str) {
        if self.config.path.as_os_str().is_empty() {
            return;
        }
        let _ = Config::write_pref(&self.config.path, key, value);
    }

    /// The live `SettingsValues` snapshot IF the Settings overlay is the one
    /// currently open — `None` otherwise, so a caller can skip the refresh
    /// round trip when there is no cached row to correct.
    fn settings_values_if_open(&self) -> Option<crate::settings::SettingsValues> {
        match self.journey.card() {
            Some(ov) if ov.kind == crate::overlay::OverlayKind::Settings => {
                Some(crate::settings::SettingsValues::gather(
                    self.config,
                    &self.root,
                    self.zoom,
                    crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
                ))
            }
            _ => None,
        }
    }

    /// Push `values` into the STILL-OPEN Settings overlay's CACHED cells —
    /// mirrors `App::refresh_settings_overlay`. A row's drawn/captured value
    /// is a cache set at build/refresh time, never re-derived at render time
    /// (`OverlayState::set_secondaries`/`set_range_cells`), so an effect that
    /// changes a setting's value must refresh it explicitly or a later
    /// capture frame keeps showing the OLD cell — exactly the gap this item
    /// closes.
    fn apply_settings_overlay_values(&mut self, values: crate::settings::SettingsValues) {
        let secondaries = crate::settings::visible_value_cells(&values);
        let range_cells = crate::settings::visible_range_cells(&values);
        if let Some(ov) = self.journey.card_mut() {
            ov.set_secondaries(secondaries);
            ov.set_range_cells(range_cells);
        }
    }

    /// `Effect::SettingToggle` — the Settings picker's Enter-to-flip door.
    /// Mirrors `App::setting_toggle`'s key table: 12 keys flip a live
    /// process-global (the SAME global the renderer and `settings::value_for`
    /// read, so the flip is visible to any later chord in this replay with no
    /// further bookkeeping); 3 (`autosave`/`history`/`session_restore`) are
    /// config-only — no global exists, so persisting the flipped value IS the
    /// whole effect, per `App::setting_toggle`'s own doc. A no-op on any
    /// other key (`"keymap"` included, though `classify_for` never routes it
    /// here), mirroring App's own "unknown key: a calm no-op" catch-all.
    pub(super) fn interpret_setting_toggle(&mut self, key: &str) {
        if self.filesystem != crate::replay::FilesystemCapability::Isolated {
            return;
        }
        let now = match key {
            "page_mode" => crate::page::page_on(),
            "typewriter_scroll" => crate::typewriter::typewriter_on(),
            "wysiwyg" => crate::markdown::wysiwyg_on(),
            "popover" => crate::popover::popover_on(),
            "inline_images" => crate::markdown::inline_images_on(),
            "code_ligatures" => crate::render::code_ligatures_on(),
            "spellcheck" => crate::spell::spellcheck_on(),
            "writing_nits" => crate::nits::nits_on(),
            "outline" => crate::outline::outline_on(),
            "menu_bar" => crate::menubar::menu_bar_on(),
            "reduce_motion" => crate::motion::reduced(),
            "file_visibility" => crate::file_visibility::all_on(),
            "autosave" => self.config.autosave_on(),
            "history" => self.config.history_on(),
            "session_restore" => self.config.session_restore_on(),
            _ => return,
        };
        let next = !now;
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
            "menu_bar" => crate::menubar::set_menu_bar_on(next),
            "reduce_motion" => crate::motion::set_reduced(next),
            "file_visibility" => crate::file_visibility::set_all_on(next),
            // Config-only: autosave/history/session_restore have no global.
            _ => {}
        }
        self.persist_setting(key, if next { "true" } else { "false" });
        if let Some(mut values) = self.settings_values_if_open() {
            match key {
                "autosave" => values.autosave = next,
                "history" => values.history = next,
                "session_restore" => values.session_restore = next,
                _ => {}
            }
            self.apply_settings_overlay_values(values);
        }
    }

    /// `Effect::SettingValueCommit` — the Settings picker's exact-numeric-
    /// entry door (typed digits + Enter on a `Range` row). Mirrors `App::
    /// setting_value_commit` for its three keys; the wrap-width reflow and
    /// zoom debounce bookkeeping App also does are live-only rendering
    /// polish with no headless observer — the same reasoning that already
    /// classifies `SettingRangeStep`'s own live tail as "not a gap".
    pub(super) fn interpret_setting_value_commit(&mut self, key: &str, raw: &str) {
        if self.filesystem != crate::replay::FilesystemCapability::Isolated {
            return;
        }
        match key {
            "page_width_prose" | "page_width_code" => {
                let id = if key == "page_width_prose" {
                    SettingId::PageWidthProse
                } else {
                    SettingId::PageWidthCode
                };
                let Some(spec) = crate::settings::range_spec(id) else {
                    return;
                };
                let Some(value) = spec.parse(raw) else {
                    return;
                };
                let width = spec.quantize(value) as usize;
                let class = if key == "page_width_prose" {
                    crate::page::PageClass::Prose
                } else {
                    crate::page::PageClass::Code
                };
                // Mirrors `App::range_apply_live`'s own guard: the process
                // global only reflows when the ACTIVE buffer's page class
                // matches this row's — the other class's row is background
                // config only, exactly like live.
                if self.buffer.page_class() == class {
                    crate::page::set_measure(width);
                }
                self.persist_setting(key, &spec.persist_value(width as f32));
                if let Some(mut values) = self.settings_values_if_open() {
                    if key == "page_width_prose" {
                        values.page_width_prose = width;
                    } else {
                        values.page_width_code = width;
                    }
                    self.apply_settings_overlay_values(values);
                }
            }
            "zoom" => {
                let Some(z) = crate::settings::parse_zoom(raw) else {
                    return;
                };
                self.zoom = z;
                self.persist_setting("zoom", &format!("{z:.3}"));
                if let Some(mut values) = self.settings_values_if_open() {
                    values.zoom = z;
                    self.apply_settings_overlay_values(values);
                }
            }
            "scroll_sensitivity" => {
                let Some(s) = crate::range::SCROLL_SENSITIVITY.parse(raw) else {
                    return;
                };
                crate::settings::set_scroll_sensitivity(s);
                self.persist_setting(
                    "scroll_sensitivity",
                    &crate::range::SCROLL_SENSITIVITY.persist_value(s),
                );
                if let Some(mut values) = self.settings_values_if_open() {
                    values.scroll_sensitivity = s;
                    self.apply_settings_overlay_values(values);
                }
            }
            _ => {}
        }
    }

    /// `Effect::SettingPathPick` — the Settings picker's folder-navigator
    /// door. `project_root` re-scopes root/workspace/corpus through the SAME
    /// owner (`Self::resync_project_location`) a Project-picker accept
    /// already uses (item 189); `default_folder` persists its key only (no
    /// replay-session field reads it back); `workspace` persists AND
    /// re-derives this session's own `workspace`/`corpus` (root unchanged) so
    /// a chord applied afterward reads the new scope — the one observable
    /// slice of live `App::reload_config`'s work for this key.
    pub(super) fn interpret_setting_path_pick(&mut self, key: &str, path: &str) {
        if self.filesystem != crate::replay::FilesystemCapability::Isolated {
            return;
        }
        match key {
            "project_root" => {
                self.resync_project_location(std::path::PathBuf::from(path));
                if let Some(values) = self.settings_values_if_open() {
                    self.apply_settings_overlay_values(values);
                }
            }
            "default_folder" => {
                self.persist_setting(key, &format!("\"{path}\""));
                if let Some(mut values) = self.settings_values_if_open() {
                    values.default_folder = path.to_string();
                    self.apply_settings_overlay_values(values);
                }
            }
            "workspace" => {
                self.persist_setting(key, &format!("\"{path}\""));
                self.workspace_flag = Some(std::path::PathBuf::from(path));
                let root = self.root.clone();
                self.resync_project_location(root);
                if let Some(mut values) = self.settings_values_if_open() {
                    values.workspace = path.to_string();
                    self.apply_settings_overlay_values(values);
                }
            }
            _ => {}
        }
    }
}
