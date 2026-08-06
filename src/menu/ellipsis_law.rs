//! **THE FILE-MENU ELLIPSIS LAW.** A File-menu label ending in "…" promises
//! that a further surface opens before anything happens (a summoned
//! picker/card, like Browse files… or Version history…); a label with no
//! ellipsis promises the row completes on its own, like Save or Duplicate
//! file. Swept with NO wildcard over `FILE_ITEMS`, so a future row can't dodge
//! it, and driven through a real [`crate::actions::apply_transition`] dispatch
//! rather than a hand-kept table — a row that stops opening its surface, or
//! starts silently opening one it never used to, is caught by what it
//! actually DOES, not by what a second list claims about it.

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
