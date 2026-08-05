//! THE SHARED TOGGLE CORE (item 193) — the pure "read the CURRENT value,
//! negate it, and set the process global if one exists" behavior every
//! `SettingKind::Toggle` row's Enter key performs, extracted so the live
//! `App` door (`app/files/settings.rs::setting_toggle`) and the replay
//! interpreter (`main/run/settings_effects.rs::interpret_setting_toggle`)
//! route through the SAME code rather than each restating [`super::
//! toggle_key`]'s roster by hand — the exact duplication item 185 eliminated
//! elsewhere and item 190's replay door re-grew hours later. Carved into its
//! own file (not left inline in `settings.rs`) so the extraction costs that
//! file zero lines rather than raising its file-size mark for growth a
//! submodule avoids — the size-mark rule items 182/192 record.

/// Negate the current value of a core toggle key and set the process global
/// if one exists. `config` supplies the CURRENT value for a CONFIG-ONLY key
/// (`autosave`/`history`/`session_restore` have no process global — negating
/// IS reading `config`; there is nothing to `set` until the caller's own
/// `persist_pref`/`persist_setting` writes the result). `"keymap"` is
/// deliberately NOT part of this core: flipping it needs a LIVE keymap
/// rebuild (`App::toggle_keymap_flavor`), not a boolean negate — `App`
/// special-cases it before ever reaching here, and the replay interpreter
/// never receives it (`replay::classify_for` keeps it Unsupported even under
/// Isolated).
///
/// Returns the NEXT value the caller must persist, or `None` for any key
/// this core does not recognize — the fall-through both doors used to hide
/// their OWN restated copy behind is now hidden behind exactly ONE `_ =>
/// None`, and `the_settings_toggle_core_handles_every_key_toggle_key_names`
/// in `replay::tests` sweeps [`super::toggle_key`]'s whole roster with a
/// no-wildcard match over [`super::SettingId`] to prove nothing in that
/// roster can ever reach this `None` arm.
pub(crate) fn flip_toggle_global(key: &str, config: &crate::config::Config) -> Option<bool> {
    let now = toggle_core_now(key, config)?;
    let next = !now;
    toggle_core_set(key, next);
    Some(next)
}

/// The READ half of [`flip_toggle_global`] — the current value of a core
/// toggle key, from the SAME owner the renderer/readout reads. `pub(crate)`
/// (not just used internally) so the classifier ← interpreter law can ask
/// "does the core recognize this key" without executing a flip.
pub(crate) fn toggle_core_now(key: &str, config: &crate::config::Config) -> Option<bool> {
    Some(match key {
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
        "autosave" => config.autosave_on(),
        "history" => config.history_on(),
        "session_restore" => config.session_restore_on(),
        _ => return None,
    })
}

/// The SET half of [`flip_toggle_global`] — a no-op for a config-only key
/// (`autosave`/`history`/`session_restore`: persisting the flipped value IS
/// the whole effect, per `App::setting_toggle`'s own doc) or an unrecognized
/// one (never reached: callers only ever invoke this after
/// [`toggle_core_now`] already returned `Some`).
fn toggle_core_set(key: &str, next: bool) {
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
}

/// Whether `key` is one the shared toggle core recognizes — queried by
/// [`crate::replay::typed::classify_settings`] so the classifier can never
/// promise `Applied` for a `SettingToggle` key the interpreter would
/// silently drop through its own `_ => return`. Reading with an empty
/// `Config` is safe (and correct): the three config-only keys' CURRENT
/// value never affects whether they are RECOGNIZED, only what they read as.
pub(crate) fn is_core_toggle_key(key: &str) -> bool {
    toggle_core_now(key, &crate::config::Config::empty()).is_some()
}

/// The value a PROCESS-GLOBAL toggle key carries on a fresh install, before any
/// config or settings write — the read-only sibling of
/// [`toggle_core_now`], answering "what is this when nothing has
/// set it" rather than "what is it right now".
///
/// Every arm reads the owning module's OWN default constant, the same one its
/// `Toggle` static is constructed from, so this can never disagree with the
/// running app. `None` for a key with no single process-global default:
/// `autosave`, `history` and `session_restore` are config-only (their fallback
/// is `Config::*_on()`'s `unwrap_or`), `menu_bar` is per-OS and carries two
/// constants rather than one, and `keymap` is not a boolean at all — exactly
/// the split `toggle_core_now`'s own doc records.
///
/// Read by the generated reference (`reference::rows::config_default`) so the
/// documented default is derived from the owner rather than transcribed.
#[allow(dead_code)] // Consumed by the reference generator, which is test-only.
pub fn toggle_default(key: &str) -> Option<bool> {
    Some(match key {
        "page_mode" => crate::page::PAGE_MODE_DEFAULT,
        "typewriter_scroll" => crate::typewriter::TYPEWRITER_SCROLL_DEFAULT,
        "reduce_motion" => crate::motion::REDUCE_MOTION_DEFAULT,
        "wysiwyg" => crate::markdown::WYSIWYG_DEFAULT,
        "popover" => crate::popover::POPOVER_DEFAULT,
        "inline_images" => crate::markdown::INLINE_IMAGES_DEFAULT,
        "code_ligatures" => crate::render::CODE_LIGATURES_DEFAULT,
        "outline" => crate::outline::OUTLINE_DEFAULT,
        "spellcheck" => crate::spell::SPELLCHECK_DEFAULT,
        "writing_nits" => crate::nits::WRITING_NITS_DEFAULT,
        "file_visibility" => crate::file_visibility::FILE_VISIBILITY_DEFAULT,
        _ => return None,
    })
}
