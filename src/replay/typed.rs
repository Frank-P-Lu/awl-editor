use super::*;

pub(super) fn named(name: &'static str, class: EffectClass) -> Classified {
    Classified { name, class }
}

pub(super) fn classify_buffer(effect: &crate::actions::BufferEffect) -> Classified {
    match effect {
        crate::actions::BufferEffect::NewDocument => named("new_document", EffectClass::Applied),
        crate::actions::BufferEffect::OpenSettings => named("open_settings", EffectClass::Applied),
        crate::actions::BufferEffect::OpenCredits => named("open_credits", EffectClass::Applied),
        crate::actions::BufferEffect::OpenGuide => named("open_guide", EffectClass::Applied),
        crate::actions::BufferEffect::OpenReference => {
            named("open_reference", EffectClass::Applied)
        }
        crate::actions::BufferEffect::Previous { finished } => named(
            if *finished {
                "finish_buffer"
            } else {
                "last_buffer"
            },
            EffectClass::Unsupported {
                why: concat!(
                    "the 2-deep buffer history is live-App-only; ",
                    "the buffer switch would not happen"
                ),
            },
        ),
    }
}

pub(super) fn classify_clipboard(effect: &crate::actions::ClipboardEffect) -> Classified {
    match effect {
        crate::actions::ClipboardEffect::WriteKillRing => {
            intercepted("clipboard_write", String::new())
        }
        crate::actions::ClipboardEffect::PasteImage => {
            intercepted("clipboard_paste_image", String::new())
        }
    }
}

/// The settings trio (`SettingToggle`/`SettingValueCommit`/
/// `SettingPathPick`), promoted the same shape [`classify_persistence`]'s
/// `Save` arm already is: under an Isolated filesystem,
/// `main/run/settings_effects.rs::ReplaySession` performs the SAME
/// process-global flip + `Config::write_pref` the live `App` door does, so
/// the replay session ends in the identical state. `why` is the ordinary
/// (`FilesystemCapability::None`) Unsupported reason, specific to the calling
/// effect. `toggle_key` is `Some` only for `SettingToggle`, whose one key
/// `"keymap"` stays Unsupported EVEN under Isolated: flipping it needs a LIVE
/// keymap rebuild (`App::toggle_keymap_flavor`) so a LATER chord in the same
/// replay resolves against the new flavor — a replay session owns no such
/// capability, the identical reason `RebindCommit`/`RebindReset` stay
/// Unsupported.
///
/// This used to promise `Applied` for every `toggle_key`
/// except `"keymap"`, while `ReplaySession::interpret_setting_toggle` handled
/// a hand-copied roster behind its own `_ => return`: a key added to
/// `settings::toggle_key` without a matching interpreter arm would have
/// worked live and silently no-op'd through replay while this classifier
/// still reported `Applied` — the sidecar's own headline tripwire (a state
/// oracle that lies about the state) in a new place. The `toggle_key` arm
/// below now asks [`crate::settings::is_core_toggle_key`] — the SAME
/// recognizer [`crate::settings::flip_toggle_global`] (the interpreter's own
/// core) is built from — so a key the interpreter cannot handle is
/// classified `Unsupported` by construction, never a promise the interpreter
/// can't keep. `the_settings_toggle_core_handles_every_key_toggle_key_names`
/// (`replay::tests`) sweeps the whole roster with a no-wildcard match over
/// `SettingId` to prove the two can never drift apart again.
pub(super) fn classify_settings(
    name: &'static str,
    why: &'static str,
    filesystem: FilesystemCapability,
    toggle_key: Option<&str>,
) -> Classified {
    if let Some(key) = toggle_key {
        if key == "keymap" {
            return named(
                name,
                EffectClass::Unsupported {
                    why: "the live keymap reload is live-App-only; a later chord in the same \
                          replay would keep resolving against the OLD flavor",
                },
            );
        }
        if !crate::settings::is_core_toggle_key(key) {
            return named(
                name,
                EffectClass::Unsupported {
                    why: "the replay's shared toggle core does not recognize this key; \
                          a live App may still handle it, but a replay cannot",
                },
            );
        }
    }
    let class = match filesystem {
        FilesystemCapability::None => EffectClass::Unsupported { why },
        FilesystemCapability::Isolated => EffectClass::Applied,
    };
    named(name, class)
}

pub(super) fn classify_persistence(
    effect: &crate::actions::PersistenceEffect,
    filesystem: FilesystemCapability,
) -> Classified {
    use crate::actions::{PersistenceEffect::*, PreferenceEffect::*, Resolution, SaveKind};
    match effect {
        ResolveExternalChange(resolution) => named(
            match resolution {
                Resolution::KeepMine => "resolve_keep_mine",
                Resolution::TakeTheirs => "resolve_take_theirs",
            },
            EffectClass::Unsupported {
                why: "an external-change conflict is latched on the live App; \
                      replay holds no baseline for a file it never opened",
            },
        ),
        ReviewExternalChange => named(
            "review_external_change",
            EffectClass::Unsupported {
                why: "the conflict this reviews is latched on the live App; \
                      replay holds no baseline for a file it never opened",
            },
        ),
        Save(kind) => {
            let (name, why) = match kind {
                SaveKind::Manual => (
                    "save",
                    "saving is live-only; replay has no filesystem-write capability",
                ),
                SaveKind::Finish => (
                    "finish_save",
                    "finish-save is live-only; replay has no filesystem-write capability",
                ),
            };
            let class = match filesystem {
                FilesystemCapability::None => EffectClass::Unsupported { why },
                FilesystemCapability::Isolated => EffectClass::Applied,
            };
            named(name, class)
        }
        Preference(preference) => named(
            match preference {
                CaretMode => "persist_caret_mode",
                PageMode => "persist_page_mode",
                PageWidth => "persist_page_width",
                PageReset => "persist_page_reset",
                Outline => "persist_outline",
                MenuBar => "persist_menu_bar",
                Typewriter => "persist_typewriter",
                Spellcheck => "persist_spellcheck",
                WritingNits => "persist_writing_nits",
                WritingStreaks => "flush_writing_streaks",
            },
            EffectClass::Applied,
        ),
    }
}
