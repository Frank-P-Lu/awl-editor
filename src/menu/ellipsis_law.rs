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
//!
//! **THE EXPORT ROWS' ELLIPSIS IS A DELIBERATE DECISION WITH A NAMED COST.** One
//! static `label` feeds BOTH menu bars (`menu::roster` drives the awl-drawn bar on
//! Linux and web), so it can only tell one story. An export now asks WHERE before
//! it writes on every platform that has a folder to offer — the `ExportDest`
//! navigator through the shared core, and `NSSavePanel` from this menu on macOS —
//! so the ellipsis is TRUE on macOS and TRUE on Linux, on every door. It is FALSE
//! on the WEB build alone, where the bytes go to the browser's own download and
//! the browser owns where they land, so awl opens nothing
//! (`actions::export_picks_destination`). Dropping the ellipsis instead would
//! invert that: false on the two desktop platforms — where a surface genuinely
//! opens — and true only on the browser. A cosmetic over-promise on one build is
//! the cheaper of the two lies, and the third law below pins the exception to
//! EXACTLY that one platform, so it cannot quietly widen.

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
        let transition = apply_transition(&mut ctx, &action, false);

        let opened_a_surface = journey.card().is_some()
            || transition.contains(|effect| matches!(effect, crate::actions::Effect::Surface(_)));
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

/// THE PLATFORM AXIS OF THE ONE STATIC LABEL — the third door onto the same
/// promise, and the one neither law above can see: they both run in ONE
/// configuration (this host, compiled native), while the label is shipped to
/// three platforms from a single `&'static str`.
///
/// Asked through the pure, platform-PARAMETERISED predicate the dispatch itself
/// reads (`actions::export_picks_destination`), so the web arm is swept from a
/// native run rather than trusted. The assertion is not "the label is truthful"
/// — it is not, on one platform, deliberately (see the module doc) — but that the
/// set of platforms it over-promises to is EXACTLY `{Web}`. That pins the
/// exception in both directions: giving the web build a destination surface makes
/// this red (delete the exception), and taking the navigator away from native
/// makes it red too (the lie widened).
///
/// MUTATION TARGET: strip the ellipsis from the export rows, or make
/// `export_picks_destination` answer `true` on `Web`, and this fails by name
/// reporting which platforms disagreed.
#[test]
fn the_export_labels_ellipsis_over_promises_to_exactly_one_platform() {
    use crate::commands::Platform;
    use crate::keymap::Action;

    // ENROLMENT FROM THE ROSTER: a row is an export row because its resolved
    // catalog action is one, never because its label reads like one. The three
    // variants are named so a fourth export action does not join silently.
    let export_rows: Vec<&Routed> = FILE_ITEMS
        .iter()
        .filter(|item| {
            matches!(
                resolve(item.id),
                Some(Action::ExportWord) | Some(Action::ExportHtml) | Some(Action::ExportPdf)
            )
        })
        .collect();
    assert_eq!(
        export_rows.len(),
        3,
        "PRESENCE: the File menu has three export rows; this law is vacuous if the \
         enrolment stops matching them",
    );

    let mut over_promising: Vec<Platform> = Vec::new();
    for platform in [Platform::Native, Platform::Web] {
        let opens = crate::actions::export_picks_destination(platform);
        for item in &export_rows {
            let promises = item.label.ends_with('…');
            assert!(
                promises || !opens,
                "{:?} (label {:?}): a surface opens on {platform:?} and the label \
                 does not promise it",
                item.id,
                item.label,
            );
            if promises && !opens && !over_promising.contains(&platform) {
                over_promising.push(platform);
            }
        }
    }
    assert_eq!(
        over_promising,
        vec![Platform::Web],
        "the export ellipsis may over-promise on the WEB build alone (the browser \
         owns where a download lands); it over-promised on {over_promising:?}",
    );
}
