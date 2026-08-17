//! src/app/files/rebind.rs — THE KEYMAP FLAVOR apply (native/emacs, picked
//! from the "Keymap…" sub-overlay) and the REBIND MENU's capture commit/reset
//! + still-open-menu refresh. Peeled out of `files/settings.rs` to keep each
//! file under the ~500-line ceiling.

use crate::app::*;

impl App {
    /// THE KEYMAP FLAVOR APPLY (Enter on a row of the "Keymap…" picker):
    /// set `Config::keymap_flavor` to the CHOSEN flavor (not a toggle — the
    /// picker has no audition, so this is the whole apply), PERSIST it (a
    /// quoted string, not a bool — [`Self::persist_pref`] handles both shapes
    /// identically, like "theme"/"caret_mode"), RE-APPLY the keymap live from
    /// the updated in-memory config — the SAME two calls [`Self::reload_config`]
    /// makes (`apply_overrides` + `apply_linux_keep` against the now-effective,
    /// flavor-widened keep list), so the change takes effect immediately,
    /// exactly like hand-editing `keymap = "emacs"` into the config buffer and
    /// saving it — and NAME the resulting layout with a notice, so accepting
    /// this picker is never a silent flip (the `Action::ConvertLineEndings`
    /// precedent: "which one am I on" is the question a picker-driven change
    /// leaves the user with).
    ///
    /// Deliberately NOT `self.reload_config()` (a re-READ from disk): a config
    /// with a genuinely EMPTY `path` (a bare `Config::empty()`, used by native
    /// test scaffolding — the web build now always resolves a real
    /// `fs::web_config_path()`, so this is no longer the web build's own case)
    /// would silently DISCARD the change, since both `reload_config`'s fresh
    /// `Config::load` and `persist_pref`'s own disk write bail out early on an
    /// empty path. Instead the in-memory mirror is set HERE, unconditionally,
    /// before attempting the disk write, and the keymap is rebuilt straight
    /// from that mirror.
    ///
    /// WEB: the disk write now genuinely persists (`fs::web_config_path` over
    /// `WebFs`/`localStorage` — see the web-config round), so a keymap-flavor
    /// change survives a page reload exactly like on native.
    pub(in crate::app) fn apply_keymap_flavor(&mut self, flavor: crate::keymap::KeymapFlavor) {
        self.config.keymap = Some(flavor.config_name().to_string());
        self.persist_pref("keymap", &format!("\"{}\"", flavor.config_name()));
        let mut keys_with_web_alt = self.config.keys.clone();
        keys_with_web_alt.extend(crate::commands::web_alternate_keys(
            &self.config.keys,
            crate::convention::Convention::current(),
            crate::commands::Platform::current(),
        ));
        let linux_keep = self.config.effective_linux_keep();
        self.input.apply_key_overrides(&keys_with_web_alt);
        self.input.apply_linux_keep(&linux_keep);
        self.refresh_settings_overlay();
        self.emit_notice(crate::actions::NoticeEffect::Toast(format!(
            "keymap: {}",
            flavor.label()
        )));
        // Every sibling settings-mutation door (`setting_toggle`'s generic
        // path, `setting_value_commit`, `setting_path_pick`) ends in a
        // `request_redraw` of its own rather than leaning on whatever
        // generic post-dispatch redraw its caller happens to also issue —
        // match that convention here too (currently masked live by the
        // keyboard/mouse input handlers' own unconditional post-apply
        // redraw, but this door should not silently depend on that).
        self.request_frame();
    }

    /// REBIND MENU commit: persist a captured `binding` to the command `slug`'s
    /// `[keys]` slot, then live-reload + refresh the open menu. A CONFLICT (the binding
    /// already belongs to another command) is GATED unless the user already accepted
    /// it: the menu moves to its `Confirm` phase (showing what's bound) and waits for a
    /// second Enter, so nothing is written behind the user's back. Otherwise the
    /// binding is merged into the command's existing slots (cap 2, newest first),
    /// written to `config.toml`, and the keymap re-applied immediately.
    pub(in crate::app) fn rebind_commit(&mut self, slug: String, binding: String, confirmed: bool) {
        if !confirmed
            && let Some(other) =
                crate::commands::binding_conflict(&binding, &slug, &self.config.keys)
        {
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.capture_into_confirm(other.to_string());
                ov.notice = format!("'{binding}' already bound to {other}");
            }
            return;
        }
        let existing: Vec<String> = self
            .config
            .keys
            .iter()
            .find(|(n, _)| crate::commands::slug(n) == slug)
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let merged = Config::merge_slot(&existing, &binding);
        let path = self.config.path.clone();
        if path.as_os_str().is_empty() {
            self.refresh_rebind_overlay("no config path; not saved".to_string());
            return;
        }
        if let Err(e) = Config::write_binding(&path, &slug, Some(&merged)) {
            eprintln!("rebind: could not write {}: {e}", path.display());
        }
        self.reload_config();
        self.refresh_rebind_overlay(format!("bound {slug} -> {binding}"));
    }

    /// REBIND MENU reset-to-default (Delete on a command): REMOVE the command's
    /// `[keys]` entry, persist, and live-reload so its built-in default applies again.
    pub(in crate::app) fn rebind_reset(&mut self, slug: String) {
        let path = self.config.path.clone();
        if !path.as_os_str().is_empty()
            && let Err(e) = Config::write_binding(&path, &slug, None)
        {
            eprintln!("rebind: could not reset {}: {e}", path.display());
        }
        self.reload_config();
        self.refresh_rebind_overlay(format!("reset {slug} to default"));
    }

    /// After a rebind commit/reset + live-reload, refresh the still-open Keybindings
    /// menu: close any capture, re-pull the EFFECTIVE binding column from the new
    /// config, and set the status `notice`. A no-op if the menu isn't open.
    pub(in crate::app) fn refresh_rebind_overlay(&mut self, notice: String) {
        let keys = self.config.keys.clone();
        let keep = self.config.effective_linux_keep();
        if let Some(ov) = self.workspace_state.overlay_mut()
            && ov.kind == crate::overlay::OverlayKind::Keybindings
        {
            ov.capture = None;
            ov.set_secondaries(crate::commands::effective_bindings(&keys, &keep));
            ov.notice = notice;
        }
    }

    /// True while the rebind menu is RECORDING a capture, so the live key handler
    /// routes the next press into the capture (a chord-level interception) rather than
    /// through the keymap. Enter / Esc are excluded by the caller (they finish / abort).
    pub(in crate::app) fn capture_recording(&self) -> bool {
        self.workspace_state
            .overlay()
            .map(|o| {
                o.kind == crate::overlay::OverlayKind::Keybindings
                    && matches!(
                        o.capture.as_ref().map(|c| c.stage),
                        Some(crate::overlay::CaptureStage::Recording)
                    )
            })
            .unwrap_or(false)
    }
}
