//! ONE DRAWN ROW of the margin stack (resting or expanded) — split out of the
//! parent module to keep [`super::WorkingSet`]'s own file under this repo's
//! production line ceiling. `StackRow`/`StackRowKind` stay logically part of
//! the module's public surface (re-exported from `workingset.rs`) rather than
//! becoming a separate namespace a consumer has to know about.

/// ONE DRAWN ROW of the margin's resting stack, already reduced to the two
/// pieces of text a row shows and which one of them is the reader's current
/// file. Deliberately a projection rather than a borrow of [`super::OpenFile`]:
/// the renderer never asks the working set a question mid-frame, so a row
/// cannot answer one thing to the draw and another to the hit-test.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StackRowKind {
    /// A real open file. This is the only row kind that may show the close mark
    /// or carry the active-file plate.
    #[default]
    File,
    /// The collapsed view's single generic overflow affordance.
    More { hidden: usize },
    /// A project heading in the expanded cross-project panel.
    Group { active: bool },
}

/// One projected row in the margin stack.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StackRow {
    /// The file name, in the row's normal ink.
    pub leaf: String,
    /// The root-relative parent with its trailing separator (`"journal/"`), in
    /// quieter ink. Empty when the file sits directly under the root.
    pub parent: String,
    /// Is this the file the reader is currently editing?
    pub active: bool,
    /// Whether this row is a file, the one overflow affordance, or a project
    /// heading. [`super::WorkingSet::stack_rows`] (the resting stack) emits
    /// `File` and, once the active root's group overflows
    /// [`super::RESTING_FILES`], one trailing `More`;
    /// [`super::WorkingSet::expanded_rows`] (the transient scrollable panel)
    /// emits `File` and `Group` heading rows.
    pub kind: StackRowKind,
}
