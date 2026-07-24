//! BUFFER + CONFIG management: opening project-relative files, the last-buffer
//! toggle, quick-note creation / auto-save / live-rename / move, the window
//! title, the active project root, the DOCUMENT AUTOSAVE ENGINE (config-gated,
//! default ON: atomic write on idle/blur/switch/quit with a clobber guard, plus
//! the persistent scratch stash), and the sticky-preference + rebind-menu
//! config writes (open Settings, persist theme/zoom/page/caret, live reload,
//! commit/reset a captured binding).
//!
//! DECOMPOSED (item 56, 2026-07) out of the former single-file `app/files.rs`
//! monolith (3017 lines) into this directory, mirroring the `app/input/`
//! precedent — each submodule under the ~500-line ceiling:
//!  - [`active`] — the OWNED ACTIVE BUFFER SLOT (`App::active`,
//!    `BufferExtra`): the SOLE module that constructs/destructures the slot
//!    or touches the park/activate swap.
//!  - [`open`] — opening project-relative files, C-x b, project switching +
//!    the recent MRUs, the i18n write-back-once + fold-reveal jump.
//!  - [`notes`] — the two-desk "Notes" flip's impure apply + its buffer-swap
//!    halves.
//!  - [`autosave`] — the document autosave engine, the note's own debounced
//!    auto-name save, save-feedback dirty/title/HUD sync.
//!  - [`verbs`] — rename/move/duplicate/convert-scratch/manual-save-finish/
//!    trash/the two local-history bridges.
//!  - [`settings`] — the sticky-preference writes + Settings-menu doors +
//!    page-width pair + config reload (dictionary persistence peeled to
//!    [`dictionary`], the rebind-menu capture peeled to [`rebind`]).
//! This file keeps only the PURE, testable-without-an-`App` leftovers
//! (`window_title`, the "Notes" flip's toggle-target decision) plus the
//! module wiring; `#[cfg(test)] mod tests` (the former files.rs's own test
//! module) lives in [`tests`].

mod active;
mod autosave;
mod dictionary;
mod notes;
mod open;
mod rebind;
mod settings;
mod verbs;

pub(in crate::app) use active::BufferExtra;
pub(in crate::app) use active::DeskReturn;

use crate::app::*;
use std::path::Path;

/// THE window title string — a PURE function of "which document, which world,
/// is it dirty", so it is unit-testable without a real window
/// (`Window::set_title`/`with_title` are the only two live call sites:
/// [`App::update_title`] and the initial `Window::default_attributes()` in
/// `resumed()`, which reads this BEFORE a `gpu`/window exists to set a title
/// on). An UNTITLED quick note (a note buffer with no derived filename yet)
/// shows the "scratch" placeholder until its first line names it, so a
/// brand-new C-x n note reads as "scratch" — distinct from the no-path,
/// non-note SCRATCH launch surface's "*scratch*". The active WORLD name is
/// always the trailing `[…]` suffix — this is also the accessibility win
/// noted in `ACCESSIBILITY.md`: a screen reader's window list announces the
/// actual document, not a bare "awl".
///
/// SAVE-FEEDBACK round: `dirty` (`Buffer::is_dirty()`) prepends the
/// conventional macOS/VS Code EDITED marker — a leading `"• "` — the same
/// glyph macOS's own "unsaved changes" affordance uses elsewhere in the OS,
/// so it reads as ambient chrome rather than a new symbol to learn. TASTE
/// FLAGGED (logged, not hidden): the glyph itself (`•` vs a bare `*`, vs
/// nothing at all) is a live-review call — this round picked the bullet for
/// its quieter weight against the amber-caret-only design law (DESIGN §3);
/// see `App::update_title`'s doc for the matching native titlebar dot.
pub(in crate::app) fn window_title(
    file: Option<&Path>,
    is_note: bool,
    theme_name: &str,
    dirty: bool,
) -> String {
    let name = match file {
        Some(p) => p.display().to_string(),
        None if is_note => "scratch".to_string(),
        None => "*scratch*".to_string(),
    };
    let mark = if dirty { "\u{2022} " } else { "" };
    format!("awl - {mark}{name} [{theme_name}]")
}

/// THE "NOTES" FLIP — pure toggle-target resolution for the "Notes" command
/// (`Action::NotesFlip`), unit-testable without a live `App` (mirrors
/// `window_title`'s own pure/impure split just above). Given the ACTIVE
/// project `current`, the (normally always-resolved) `notes_root`, and
/// whatever pre-flip root is currently REMEMBERED (`previous`), decide where
/// the flip lands — the exact same 2-deep-history SHAPE `last_buffer_toggle`
/// uses for buffers, one level up: projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) enum NotesFlipTarget {
    /// No usable `notes_root`: the command is a quiet no-op. `App::notes_root`
    /// always resolves to a real default (`~/notes`, see
    /// `main/args.rs::resolve_notes_root`) — this arm exists so the command
    /// degrades gracefully rather than crashing if that ever changes, and so
    /// the "missing notes_root" case is exercised by a pure unit test.
    Inert,
    /// Already IN `notes_root` with nothing remembered (e.g. a bare launch
    /// landed here directly, having never flipped): nowhere to go BACK to.
    AlreadyHome,
    /// Not currently in `notes_root`: flip there, remembering `remember` (the
    /// root being left) so the NEXT flip returns to it exactly.
    Enter { target: PathBuf, remember: PathBuf },
    /// Already in `notes_root` with a remembered previous root: flip BACK to
    /// it (consuming the memory — a THIRD flip enters fresh).
    Back { target: PathBuf },
}

/// `notes_root` is modeled as a `Path` that may be EMPTY (`Path::new("")`) —
/// the same "no usable folder" sentinel [`App::persist_page_reset`] already
/// uses for an unresolvable config path — rather than an `Option`, so a
/// caller with nothing to flip TO degrades to [`NotesFlipTarget::Inert`]
/// without ever needing to unwrap. Compares `current`/`notes_root` by plain
/// `PathBuf` equality, exactly like `switch_project`'s own root bookkeeping.
pub(in crate::app) fn notes_flip_target(
    current: &Path,
    notes_root: &Path,
    previous: Option<&Path>,
) -> NotesFlipTarget {
    if notes_root.as_os_str().is_empty() {
        return NotesFlipTarget::Inert;
    }
    if current == notes_root {
        match previous {
            Some(p) => NotesFlipTarget::Back { target: p.to_path_buf() },
            None => NotesFlipTarget::AlreadyHome,
        }
    } else {
        NotesFlipTarget::Enter { target: notes_root.to_path_buf(), remember: current.to_path_buf() }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
