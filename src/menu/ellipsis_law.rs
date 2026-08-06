//! **THE FILE-MENU ELLIPSIS LAW.** A File-menu label ending in "…" promises
//! that a further surface opens before anything happens (a summoned
//! picker/card, like Browse files… or Version history…); a label with no
//! ellipsis promises the row completes on its own, like Save or Duplicate
//! file. Swept with NO wildcard over `FILE_ITEMS`, so a future row can't dodge
//! it, and driven through a real [`crate::actions::apply_transition`] dispatch
//! rather than a hand-kept table — a row that stops opening its surface, or
//! starts silently opening one it never used to, is caught by what it
//! actually DOES, not by what a second list claims about it.
//!
//! **THE SHARED CORE IS NOT THE ONLY DOOR ONTO THAT PROMISE, AND THE FIRST LAW
//! HERE CANNOT SEE THE OTHER ONE.** A row [`super::opens_native_panel`] claims
//! is answered by an AppKit panel from the macOS menu handler, above
//! `apply_transition` entirely — so `Journey::card()` stays `None` for it no
//! matter what the panel does, and a row that popped a modal open/save panel
//! with no ellipsis on its label would pass the first law green. That is the
//! "a law dies at its SUBJECT" shape, and the second law below closes it: every
//! id the platform claims must sit on a row whose label already promises a
//! surface. Together they read — the label agrees with the shared core on every
//! platform, AND the platform's own panels only ever land on rows that promised
//! one.

use super::*;
use crate::actions::{ActionCtx, apply_transition};
use crate::buffer::Buffer;
use crate::overlay::{Journey, OverlayState};

/// MUTATION TARGET: put the ellipsis back on `FILE_ITEMS`' export rows (or
/// strip it from a real opener like "Browse files…") and this fails by name,
/// reporting the row and which side lied.
#[test]
fn every_file_menu_row_s_ellipsis_matches_whether_it_opens_a_surface() {
    for item in FILE_ITEMS {
        let action = resolve(item.id)
            .unwrap_or_else(|| panic!("{}: no catalog action resolves for this id", item.id));

        let mut buffer = Buffer::from_str("body text");
        buffer.set_path(std::path::PathBuf::from("/notes/ellipsis-law.md"));
        let mut shift_selecting = false;
        let mut zoom = 1.0_f32;
        let mut search = None;
        let mut journey = Journey::default();
        // A generic surface for WHATEVER kind an action asks to open — this
        // law asks only whether a surface opened, never which one, so every
        // kind must be able to open one.
        let mut make_overlay =
            |kind| Some(OverlayState::new(kind, Vec::new(), Vec::new(), Vec::new()));
        let mut browse_to = |kind, _rel: Option<String>| {
            Some(OverlayState::new(kind, Vec::new(), Vec::new(), Vec::new()))
        };
        let mut ctx = ActionCtx {
            buffer: &mut buffer,
            shift_selecting: &mut shift_selecting,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: None,
        };
        let _ = apply_transition(&mut ctx, &action, false);

        let opened_a_surface = journey.card().is_some();
        let promises_a_surface = item.label.ends_with('…');
        assert_eq!(
            opened_a_surface, promises_a_surface,
            "{:?} (label {:?}): promises_a_surface={promises_a_surface} but \
             opened_a_surface={opened_a_surface} — the ellipsis and the real \
             dispatch disagree",
            item.id, item.label,
        );
    }
}

/// THE PLATFORM-PANEL HALF. Every id the native menu answers with an AppKit
/// panel of its own must name a `FILE_ITEMS` row whose label ends in "…" —
/// because that panel IS a surface opening before anything happens, and the
/// first law above is structurally blind to it (see the module doc).
///
/// Host-independent by construction: the enrolment is [`super::NATIVE_PANEL_IDS`]
/// itself, read as data, so this law sweeps the same set on macOS, Linux and web
/// rather than being a property of whichever OS compiled it. Nothing here calls
/// `cfg!`.
///
/// MUTATION TARGET, and the reason this exists rather than a comment: add an
/// export id to `NATIVE_PANEL_IDS` (wiring a save panel onto a row that still
/// says it completes on its own) and this fails by name. It cannot be satisfied
/// by restoring the ellipsis alone either — the first law then demands the
/// shared core open a surface on EVERY platform for that row, so a half-finished
/// platform split cannot land green in either direction.
#[test]
fn every_native_panel_row_promises_a_surface_and_names_a_real_file_row() {
    let mut claimed = 0usize;
    for item in FILE_ITEMS {
        if !opens_native_panel(item.id) {
            continue;
        }
        claimed += 1;
        assert!(
            item.label.ends_with('…'),
            "{:?} (label {:?}): the native menu answers this row with an AppKit \
             panel — a surface opening before anything happens — so its label must \
             promise one",
            item.id,
            item.label,
        );
    }
    // PRESENCE, not decoration: a law over a set is satisfied by emptying the
    // set, and this one would go quietly vacuous the day an id is renamed out
    // from under the roster. Enrol every claimed id, then require the count to
    // account for all of them.
    assert_eq!(
        claimed,
        NATIVE_PANEL_IDS.len(),
        "every claimed id must name a real FILE_ITEMS row; claimed={claimed} of \
         {:?}",
        NATIVE_PANEL_IDS,
    );
    assert!(
        !NATIVE_PANEL_IDS.is_empty(),
        "the platform-panel door still exists; an empty roster would make this \
         law sweep nothing",
    );
}
