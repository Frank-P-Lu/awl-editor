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

pub(super) fn classify_persistence(
    effect: &crate::actions::PersistenceEffect,
    filesystem: FilesystemCapability,
) -> Classified {
    use crate::actions::{PersistenceEffect::*, PreferenceEffect::*, SaveKind};
    match effect {
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
