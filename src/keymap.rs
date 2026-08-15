mod action;
pub use action::Action;

mod binding;
pub use binding::parse_binding;

mod platform;
#[cfg(test)]
pub(crate) use platform::LINUX_DISPLACED_LETTERS;
pub use platform::{KeymapFlavor, linux_emacs_preset_keep};
pub(crate) use platform::{linux_builtin_keep, linux_displaces_emacs_default, linux_keeps_chord};

mod resolve;

mod state;
pub use state::{Chord, KeymapState};

#[cfg(test)]
mod tests;
