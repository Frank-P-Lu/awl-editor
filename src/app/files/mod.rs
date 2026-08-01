//! BUFFER + CONFIG management: opening folder-relative files, the last-buffer
//! toggle, fresh-document creation / auto-name / move, the window title, the
//! ACTIVE FOLDER, the DOCUMENT AUTOSAVE ENGINE (config-gated, default ON:
//! atomic write on idle/blur/switch/quit with a clobber guard, plus the
//! persistent scratch stash), and the sticky-preference + rebind-menu config
//! writes (open Settings, persist theme/zoom/page/caret, live reload,
//! commit/reset a captured binding).
//!
//! DECOMPOSED (item 56, 2026-07) out of the former single-file `app/files.rs`
//! monolith (3017 lines) into this directory, mirroring the `app/input/`
//! precedent — each submodule under the ~500-line ceiling:
//!  - [`active`] — the OWNED ACTIVE BUFFER SLOT (`App::active`,
//!    `BufferExtra`): the SOLE module that constructs/destructures the slot
//!    or touches the park/activate swap.
//!  - [`open`] — opening folder-relative files, C-x b, folder switching +
//!    the recent MRUs, the i18n write-back-once + fold-reveal jump.
//!  - [`document`] — the fresh-document buffer swap (item 76 retired the
//!    two-desk "Notes" flip that used to live here).
//!  - [`autosave`] — the document autosave engine, the fresh document's own
//!    debounced ONE-SHOT auto-name save, save-feedback dirty/title/HUD sync.
//!  - [`verbs`] — rename/move/duplicate/convert-scratch/manual-save-finish/
//!    trash/the two local-history bridges.
//!  - [`settings`] — the sticky-preference writes + Settings-menu doors +
//!    page-width pair + config reload (dictionary persistence peeled to
//!    [`dictionary`], the rebind-menu capture peeled to [`rebind`]).
//!
//! This file keeps only the PURE, testable-without-an-`App` leftover
//! (`window_title`) plus the module wiring; `#[cfg(test)] mod tests` (the
//! former files.rs's own test module) lives in [`tests`].

mod active;
mod autosave;
mod dictionary;
mod document;
mod open;
mod range_settings;
mod rebind;
mod settings;
mod tutorial;
mod verbs;

pub(in crate::app) use active::BufferExtra;
pub(in crate::app) use tutorial::{TutorialFolderIntent, initial_default_folder};

// Only `tests` (below, via `use super::*`) needs the App-scope glob now that
// this file's own pure leftover (`window_title`) needs nothing from it beyond
// `Path` (imported separately) — cfg-gated identically to `mod tests` so a
// non-test build carries no unused-import warning.
#[cfg(all(test, not(target_arch = "wasm32")))]
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
    is_unnamed_fresh: bool,
    theme_name: &str,
    dirty: bool,
) -> String {
    let name = match file {
        Some(p) => p.display().to_string(),
        None if is_unnamed_fresh => "scratch".to_string(),
        None => "*scratch*".to_string(),
    };
    let mark = if dirty { "\u{2022} " } else { "" };
    format!("awl - {mark}{name} [{theme_name}]")
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
