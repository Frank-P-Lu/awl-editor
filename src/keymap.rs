mod action;
pub use action::Action;

mod binding;
pub use binding::parse_binding;

mod platform;
pub use platform::{KeymapFlavor, linux_emacs_preset_keep};
#[cfg(test)]
pub(crate) use platform::{LINUX_DISPLACED_LETTERS, LINUX_EMACS_META_SEED};
pub(crate) use platform::{
    linux_builtin_keep, linux_displaces_emacs_default, linux_is_native_clipboard_chord,
    linux_keeps_chord, seeded_chords_for,
};

mod resolve;

mod state;
pub use state::{Chord, KeymapState};

#[cfg(test)]
mod tests;
