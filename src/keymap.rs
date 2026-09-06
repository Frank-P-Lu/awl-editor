mod action;
pub use action::Action;

mod binding;
pub use binding::parse_binding;

mod platform;
#[cfg(test)]
pub(crate) use platform::{
    FollowGesture, LINUX_DISPLACED_LETTERS, LINUX_EMACS_CLASSIC_SEED, LINUX_EMACS_META_SEED,
    active_follow_gestures, active_seed_tables, linux_emacs_layer,
};
pub use platform::{KeymapFlavor, linux_emacs_preset_keep};
pub(crate) use platform::{
    PointerButton, follows_link, linux_builtin_keep, linux_displaces_emacs_default,
    linux_is_native_clipboard_chord, linux_keeps_chord, seeded_chords_for,
};

mod resolve;

mod state;
pub use state::{Chord, KeymapState};

#[cfg(test)]
mod tests;
