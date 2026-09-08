mod action;
pub use action::Action;

mod binding;
pub use binding::parse_binding;

mod follow;
mod platform;
pub use platform::{KeymapFlavor, linux_emacs_preset_keep};
#[cfg(test)]
pub(crate) use platform::{
    LINUX_DISPLACED_LETTERS, LINUX_EMACS_CLASSIC_SEED, LINUX_EMACS_META_SEED, active_seed_tables,
    linux_emacs_layer,
};
// `FollowGesture` and `active_follow_gestures` are consumed by the roster laws
// and by doc links; the two below are the non-test consumers (keyspec's pointer
// grammar, and mouse dispatch).
#[cfg(test)]
pub(crate) use follow::{FollowGesture, active_follow_gestures};
pub(crate) use follow::{PointerButton, follows_link};
pub(crate) use platform::{
    linux_builtin_keep, linux_displaces_emacs_default, linux_is_native_clipboard_chord,
    linux_keeps_chord, seeded_chords_for,
};

mod resolve;

mod state;
pub use state::{Chord, KeymapState};

#[cfg(test)]
mod tests;
