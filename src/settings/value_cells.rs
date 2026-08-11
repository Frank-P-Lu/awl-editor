//! **THE VALUE COLUMN'S WIDTH IS A PROPERTY OF THE SETTINGS ROSTER, NEVER OF
//! THE MACHINE.**
//!
//! A settings row is a name and a value, and the values are one shared COLUMN —
//! `render::rowlayout` grants that column a single width for every row or drops
//! it entirely. Every readout awl authors is a word or two, except one class: a
//! [`SettingKind::Path`](super::SettingKind::Path) row prints wherever the
//! reader happens to keep their work, which is a string awl does not write and
//! cannot bound.
//!
//! Un-elided, that one cell decided the whole picker's geometry. It set the
//! column's width, so it decided whether the accessory column was granted at all
//! (it was not — with a deep project root the ENTIRE value column, Range rails
//! included, vanished from every window narrower than ~1420px), and through the
//! name budget it decided which row names elided. Two readers at the same window
//! size on the same build saw two different layouts, and the difference was the
//! depth of their home directory.
//!
//! So a path cell is elided like any other row text — through
//! [`crate::overlay::elide_path`], the same owner `rowlayout::fit_primary` uses
//! for the NAME lane, which keeps the last path segment and middle-truncates the
//! directory in front of it — to an [`allowance`] the ROSTER sets rather than a
//! written-down number.
//!
//! [`super::value_for`] stays the UNELIDED readout. It answers "what is this set
//! to", which the sub-picker a Path row opens and every test comparing a row
//! against the config it came from both need whole. Presentation is this
//! module's job alone, which is why both corpus doors route through
//! [`for_rows`] rather than mapping `value_for` themselves.

use super::{SETTINGS, SettingKind, SettingRow, SettingsValues, value_for};

/// **A VALUE MAY BE AS WIDE AS A NAME, AND NO WIDER** — the allowance a path
/// cell is elided to, in chars, taken as the widest display NAME in
/// [`SETTINGS`].
///
/// The name column is the other half of the same row, so this is the width at
/// which the two columns balance rather than one dominating; it is also exactly
/// the reading `render::chrome::workspace`'s `MIN_PANE_CHARS` already documents
/// for itself — the widest name, the gap `rowlayout` puts after it, and a value
/// of the same allowance.
///
/// **It takes no arguments, and that is the point.** Deriving it from the
/// authored READOUTS instead would put live state into a geometry decision:
/// switching dictionaries changes the widest authored cell, and how much of
/// their project path a reader can see must not depend on that. Asked of
/// [`SETTINGS`] whole rather than of the caller's subset, so the Settings menu
/// and the command palette's settings union budget alike.
pub(super) fn allowance() -> usize {
    SETTINGS
        .iter()
        .map(|r| r.name.chars().count())
        .max()
        .unwrap_or(0)
}

/// The drawn value cells for `rows`: every authored readout as
/// [`value_for`] gives it, every PATH readout elided to the [`allowance`]. A
/// path already inside the allowance is returned whole.
pub(super) fn for_rows(rows: &[&'static SettingRow], values: &SettingsValues) -> Vec<String> {
    let budget = allowance();
    rows.iter()
        .map(|r| {
            let cell = value_for(r, values);
            match r.kind {
                SettingKind::Path => crate::overlay::elide_path(&cell, budget),
                _ => cell,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
