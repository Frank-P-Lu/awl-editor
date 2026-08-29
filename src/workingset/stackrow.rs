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
    /// A real open file. This is the only row kind that may carry the
    /// active-file plate; it may also show the close mark, closing itself.
    #[default]
    File,
    /// The collapsed view's single generic overflow affordance — a resting-
    /// stack row, and drawn ONLY there: clicking it EXPANDS the panel
    /// (`app/input/gutter.rs::gutter_stack_click`).
    More { hidden: usize },
    /// A project heading in the expanded cross-project panel. May also show
    /// the close mark — its own, closing every file under its root, never a
    /// switch target of its own.
    Group { active: bool },
    /// The expanded panel's own PASSIVE scroll-position cue — `↑ N
    /// more` (`up: true`) pinned above the window when rows are hidden above
    /// it, `↓ N more` below when rows remain below. Deliberately a DIFFERENT
    /// kind than `More`: that row is an actionable resting-stack EXPAND
    /// affordance, and reusing it here would make a passive position cue
    /// clickable through the exact same door — hit-tested and filtered out
    /// exactly like `Group` (`gutter_hit::stack_hit_from_plan`), never a
    /// second close/switch target.
    Overflow { up: bool, hidden: usize },
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
