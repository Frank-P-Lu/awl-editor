//! THE DOCKED FACET STRIP BILLS NO ROW OF ITS OWN.
//!
//! **Defect:** `theme_overlay_geometry` billed a flat two header rows (query
//! line + lens strip) for every grouped card, whether or not the strip's own
//! box ever draws INSIDE the panel. Under `FacetStyle::DockedTab` the strip
//! draws OUTSIDE the card (`docked_facet_band`), so that second box's `lh`
//! was pure overhead — a vacated band the size of one row, sitting between
//! the query line and the first candidate, on every lens.
//!
//! **The law**, over every `DockedTab` world in the roster (derived, never a
//! hardcoded name) × every lens its own command palette carries: the first
//! candidate row's top sits within one query beat of the query line's own
//! bottom — never a beat plus a whole extra row. A presence floor on the
//! query row itself rules out the claim being satisfied by shrinking the
//! query line to nothing rather than by billing the strip correctly.
//!
//! MUTATION-PROVEN, inline: `header_rows_billing_regression_reopens_the_vacated_strip_row`
//! reconstructs the retired flat-billing formula (never reads it back from
//! the fix, `theme_picker.rs`'s own `billed_header_rows`) and shows it fails
//! this exact claim by exactly one row — proving the law is capable of
//! catching the regression it is named for, not merely of passing today.

use super::super::*;
use super::{headless_dqp, view};

/// Every world whose active `FacetStyle` docks its lens strip outside the
/// card — the roster this law's whole claim is about. Derived from the theme
/// roster, never a named world, so a future `DockedTab` world is swept for
/// free.
fn docked_facet_roster() -> Vec<&'static str> {
    theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.facet_style, theme::FacetStyle::DockedTab))
        .map(|t| t.name)
        .collect()
}

/// A COMMAND-palette view at facet lens `lens`, folded the way `App::sync_view`
/// folds one — the same shape `rotated_rail.rs`/`raked_location.rs` use.
fn palette_view(lens: usize) -> ViewState {
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov = crate::overlay::OverlayState::new_command(
        names,
        crate::commands::effective_bindings(&[], &[]),
        hidden,
    );
    ov.set_facet_lens(lens);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Command.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_selected = ov.selected;
    v
}

#[test]
fn the_first_item_row_sits_within_one_beat_of_the_query_row_on_every_docked_facet_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_first_item_row_sits_within_one_beat_of_the_query_row: no wgpu adapter"
        );
        return;
    };

    let roster = docked_facet_roster();
    assert!(
        !roster.is_empty(),
        "no `FacetStyle::DockedTab` world ships — this law would sweep nothing"
    );

    let mut lenses_seen = 0usize;
    let mut graded = 0usize;
    for world in &roster {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let lens_count = palette_view(0).overlay_lens.len();
        assert!(
            lens_count > 1,
            "{world}: the command palette carries no real facet lens to sweep"
        );
        for lens in 0..lens_count {
            lenses_seen += 1;
            let v = palette_view(lens);
            p.set_view(&v);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            let geom = p.overlay_geometry(1200);
            let plan = p.overlay_row_plan(&geom);
            let lh = plan.lh();
            let query = plan
                .query_band()
                .expect("a faceted card draws a query line");

            // PRESENCE FLOOR: the query row is a real, full-pitch box — never
            // degenerate. Without this, the gap claim below could be
            // satisfied by shrinking the query line itself rather than by
            // billing the docked strip correctly. Asserted even for a lens
            // whose bucket turns out empty (below) — the query row is drawn
            // regardless of how many candidates follow it.
            assert!(
                (query.height - lh).abs() < 0.75,
                "{world} lens {lens}: the query row is {}px tall against a {lh}px row pitch \
                 — not a real row, so the gap claim below would be checking nothing",
                query.height
            );

            // A facet bucket can legitimately carry zero matching commands (a
            // niche category with no member in this build's catalog) — the
            // plan then has no candidate row to gap-check, and that is not a
            // failure of the billing this law is about.
            let Some(first_row) = plan.rows().first() else {
                continue;
            };
            let gap = first_row.top - query.bottom();
            let beat = p.overlay_header_gap();
            assert!(
                gap <= beat + 0.75,
                "{world} lens {lens}: the first item row sits {gap:.1}px below the query \
                 row, more than one beat ({beat:.1}px) — the docked strip's own header box \
                 is billing the row budget a full row it never draws"
            );
            graded += 1;
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        lenses_seen >= 8,
        "the sweep must cover every lens of at least one docked-facet world, got {lenses_seen}"
    );
    assert!(
        graded > 0,
        "every lens swept carried an empty facet bucket — the sweep graded nothing real"
    );
}

/// NON-VACUITY: the retired flat-billing formula (`header_rows` charged in
/// full regardless of docking), reconstructed inline rather than read back
/// from `theme_picker.rs`'s fix, fails the law above by exactly the one row
/// the fix reclaims — proving this law is capable of catching the exact
/// regression it is named for.
#[test]
fn header_rows_billing_regression_reopens_the_vacated_strip_row() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping header_rows_billing_regression_reopens_the_vacated_strip_row: no wgpu adapter"
        );
        return;
    };

    let roster = docked_facet_roster();
    let world = *roster.first().expect(
        "no `FacetStyle::DockedTab` world ships — nothing to reconstruct the regression on",
    );
    theme::set_active_by_name(world).unwrap();
    p.sync_theme();

    let v = palette_view(1);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);
    let lh = plan.lh();
    let beat = p.overlay_header_gap();
    let query = plan
        .query_band()
        .expect("a faceted card draws a query line");
    let actual_first_top = plan
        .rows()
        .first()
        .expect("a faceted card plans a first candidate row")
        .top;

    // THE RETIRED FORMULA: both header boxes (query + strip) billed their own
    // full `lh` regardless of `FacetStyle::DockedTab`
    // (`header_band_height(2, lh, header_gap)`, as it read before
    // `theme_overlay_geometry`'s `billed_header_rows`).
    let retired_first_top = query.top + 2.0 * lh + beat;

    assert!(
        (retired_first_top - actual_first_top - lh).abs() < 0.75,
        "{world}: the retired formula and the shipped one must differ by exactly the \
         reclaimed row ({lh:.1}px), or this fixture no longer reconstructs the regression \
         — retired {retired_first_top:.1}, actual {actual_first_top:.1}"
    );
    let retired_gap = retired_first_top - query.bottom();
    assert!(
        retired_gap > beat + 0.75,
        "{world}: the retired flat-billing formula must overrun the law's own floor (beat \
         {beat:.1}px) for this fixture to prove the law can catch it — got gap \
         {retired_gap:.1}px"
    );

    theme::set_active(theme::DEFAULT_THEME);
}
