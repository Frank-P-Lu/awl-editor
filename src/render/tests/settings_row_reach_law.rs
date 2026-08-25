//! THE SETTINGS "EVERY SECOND ROW" LAW, the PIXEL/HIT-TEST half.
//! `actions::tests::settings_reach` proves the `apply_transition` seam steps
//! `selected` one row at a time through every facet/direction/parity/filter/
//! scroll; this file proves the RENDERED half stays in lock-step with it — a
//! pointer y-CENTER sweep over every drawn row of the Editor facet (the
//! user's own witness, world Mopoke, 2026-07-26 — "Settings → Editor visibly
//! contains every row, but moving/selecting through it reaches only
//! alternating rows"), asserting the hit-test, a passive HOVER, and the
//! corpus mapping an ACCEPT would use all name the SAME item as the row
//! actually drawn there — across the FULL 18-world roster (not e10b9fa's
//! hand-picked seven — the "full roster, not a hand-picked few"
//! requirement), both `Pane`/`Bars` list styles, and 1×/2× DPI.
//!
//! Also covers the Zoom RANGE row's adjacency at the pixel layer: its own
//! band and its immediate neighbours' bands must each resolve to their own
//! item, never bleeding into one another.

use super::super::*;
use super::{
    SETTINGS_VIEW_PARKED_WINDOW_ROWS, diagonal_worlds, headless_dqp, settings_overlay_view,
};
use crate::overlay::{OverlayKind, OverlayState};

fn values(zoom: f32) -> crate::settings::SettingsValues {
    super::settings_values(zoom, 1.0)
}

/// A REAL Settings overlay, faceted onto Editor exactly as `Right` from the
/// `All` home does (`set_facet_lens`, the one owner a keypress and a
/// lens-strip click both call) — built via the same production wiring
/// `overlay::build`'s Settings arm uses (see `range_rail::settings_state`,
/// this file's sibling for the un-faceted Zoom-row rail tests).
fn editor_overlay() -> OverlayState {
    let vals = values(1.0);
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    let idx = ov
        .facet_scheme()
        .expect("Settings always facets")
        .strip
        .iter()
        .position(|f| f.id == "editor")
        .expect("Settings has an Editor facet");
    ov.set_facet_lens(idx);
    ov
}

/// Fold a Settings overlay into a `ViewState` the way `App::sync_view` does,
/// with the ONE deliberate divergence [`settings_overlay_view`]'s sibling
/// constant [`SETTINGS_VIEW_PARKED_WINDOW_ROWS`] documents — the shared owner
/// both this file and `range_rail.rs` now call, so there is one fixture and
/// one parked note instead of two byte-identical copies of each.
fn settings_view(ov: &OverlayState) -> ViewState {
    settings_overlay_view(ov, SETTINGS_VIEW_PARKED_WINDOW_ROWS)
}

/// The measured cluster is a geometry contract, not a Settings-only cosmetic
/// alignment. Sweep every typed settings identity through both diagonal
/// worlds and both reachable surfaces: the label/control gap stays one measured
/// rail, the row-side plan accepts the same point as the drawn label, and Range
/// rails remain wholly inside that accessory cluster.
#[test]
#[allow(clippy::too_many_lines)]
fn every_setting_kind_uses_the_measured_diagonal_cluster_rail_on_overlay_and_workspace() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping measured diagonal cluster rail law: no wgpu adapter");
        return;
    };
    let all = crate::settings::visible_rows();
    // "Keymap" is the one row `visible_rows` ever drops from the full
    // authored table on a native build: hidden on `Convention::Mac`, where
    // the flavor is structurally inert (`settings::row_available_on`).
    // `Convention` is process-frozen, so this reads the ambient value rather
    // than forcing one — `native-gate.sh`'s two-convention run exercises
    // both counts across its two passes.
    let keymap_hidden =
        crate::convention::Convention::current() != crate::convention::Convention::Linux;
    assert_eq!(
        all.len(),
        crate::settings::SETTINGS.len() - usize::from(keymap_hidden)
    );

    for world in diagonal_worlds() {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            p.set_dpi(dpi);
            for logical_width in [640u32, 1200] {
                let width = (logical_width as f32 * dpi).round() as u32;
                let height = (800.0 * dpi).round() as u32;
                p.set_size(width as f32, height as f32);
                // The swept set is DERIVED from the owner, not hand-enumerated:
                // `OverlayKind::Settings.workspace_shape()` returns
                // `Some(RailOverRows)` UNCONDITIONALLY, so `overlay_workspace` is
                // always `true` for a real Settings card — the picker can never be
                // in the un-workspaced form. Enumerating `[false, true]` here once
                // graded a state the product cannot reach: the corpus panic it
                // produced (`workspace=false`) came from a cell no keypress can
                // land on. A kind whose workspace answer becomes conditional later
                // re-opens this axis for free; one with no workspace at all sweeps
                // `false` instead of inventing an unreachable `true`.
                let reachable_workspace: &[bool] = match OverlayKind::Settings.workspace_shape() {
                    Some(_) => &[true],
                    None => &[false],
                };
                for &workspace in reachable_workspace {
                    for (want_item, setting) in all.iter().enumerate() {
                        let ctx = format!(
                            "world={world} dpi={dpi} logical_width={logical_width} \
                             workspace={workspace} setting={:?}",
                            setting.id
                        );
                        let mut ov = OverlayState::new(
                            OverlayKind::Settings,
                            crate::settings::visible_names(),
                            Vec::new(),
                            Vec::new(),
                        );
                        let values = values(1.4);
                        ov.set_secondaries(crate::settings::visible_value_cells(&values));
                        ov.set_range_cells(crate::settings::visible_range_cells(&values));
                        ov.selected = want_item;
                        let mut v = settings_view(&ov);
                        v.overlay_workspace = workspace;
                        v.overlay_detail_focus = workspace;
                        p.set_view(&v);
                        p.prepare(&device, &queue, width, height).unwrap();

                        let geom = p.overlay_geometry(width);
                        let plan = p.overlay_row_plan(&geom);
                        let row = plan
                            .rows()
                            .iter()
                            .find(|row| row.item == Some(want_item))
                            .unwrap_or_else(|| panic!("{ctx}: selected setting must be visible"));
                        let cluster = p.diagonal_cluster_probe().unwrap_or_else(|| {
                            panic!("{ctx}: diagonal world must resolve a cluster")
                        });
                        assert!(
                            cluster.cluster_w > cluster.accessory_w,
                            "{ctx}: the rail must leave the label a column beside its accessory"
                        );
                        assert!(
                            cluster.accessory_w >= 0.0,
                            "{ctx}: an accessory reservation is never negative"
                        );
                        let (selected_dx, selected_dw) = if row.item == Some(want_item) {
                            cluster.selected_offset()
                        } else {
                            (0.0, 0.0)
                        };
                        assert!(
                            (row.dx
                                - (cluster.span.dx
                                    + cluster.span.dx_per_row * row.display as f32
                                    + selected_dx))
                                .abs()
                                < 0.01
                                && (row.dw
                                    - (cluster.span.dw
                                        + cluster.span.dw_per_row * row.display as f32
                                        + selected_dw))
                                    .abs()
                                    < 0.01,
                            "{ctx}: planner row bounds must read the measured cluster span"
                        );
                        // A row's territory is the cluster BUDGET — a property
                        // of the card — with the NAME hanging on the spine end
                        // and the accessory on the outer end, each growing back
                        // toward the other, as an upright card right-aligns its
                        // chord to its own text edge. Sized from the widest row
                        // on SCREEN instead, the rail stepped sideways on every
                        // scroll.
                        let (cl, cr) = cluster.cluster_span(row.display);
                        assert!(
                            (cr - cl - cluster.cluster_w).abs() < 0.01,
                            "{ctx}: a row's rail must span exactly the cluster budget"
                        );
                        let (al, ar) = cluster.accessory_span(row.display);
                        assert!(
                            (ar - al - cluster.accessory_w).abs() < 0.01,
                            "{ctx}: the accessory column reserves exactly its measured ink"
                        );
                        let (left, right) = plan.card_x_span();
                        assert!(
                            cl >= left + row.dx - 0.01 && cr <= right + row.dw + 0.01,
                            "{ctx}: measured cluster exceeds its row span \
                             (cluster={cl}..{cr}, span={}..{}, cluster_w={} accessory_w={})",
                            left + row.dx,
                            right + row.dw,
                            cluster.cluster_w,
                            cluster.accessory_w,
                        );
                        let y = row.top + row.height * 0.5;
                        // The pointer is put half a pixel INSIDE the cluster's
                        // spine end — on Magpie that is the right edge of a
                        // right-aligned name, the exact seam the mirror moved.
                        let inside =
                            cluster.label_anchor(row.display) + 0.5 * cluster.label_flow().sign();
                        assert_eq!(
                            p.overlay_row_at(inside, y),
                            Some(want_item),
                            "{ctx}: the name's drawn side and pointer row must agree"
                        );
                        assert_eq!(
                            p.overlay_spine.instance_count(),
                            1,
                            "{ctx}: diagonal base spine must remain one continuous segment"
                        );
                        assert_eq!(
                            p.overlay_spine_selected.instance_count(),
                            2,
                            "{ctx}: selected row must draw the mark's two chevron arms \
                             (item 247; SHAPE is graded by diagonal_composition's own law)"
                        );
                        assert_eq!(
                            p.overlay_rows.instance_count(),
                            0,
                            "{ctx}: diagonal selection must not fall back to a full-width row fill"
                        );

                        match setting.kind {
                            crate::settings::SettingKind::Range => {
                                let (x0, x1) =
                                    p.overlay_range_scale(want_item).unwrap_or_else(|| {
                                        panic!("{ctx}: every Range setting must retain a rail")
                                    });
                                assert!(
                                    x0 >= al - 0.01 && x1 <= ar + 0.01,
                                    "{ctx}: Range rail ({x0}..{x1}) exceeds its accessory \
                                     cluster ({al}..{ar})"
                                );
                                assert_eq!(
                                    p.overlay_range_at((x0 + x1) * 0.5, y).map(|(item, _)| item),
                                    Some(want_item),
                                    "{ctx}: Range rail and hit target name different settings"
                                );
                            }
                            crate::settings::SettingKind::Toggle
                            | crate::settings::SettingKind::Picker
                            | crate::settings::SettingKind::Value
                            | crate::settings::SettingKind::Path
                            | crate::settings::SettingKind::Submenu
                            | crate::settings::SettingKind::Action => {
                                assert!(
                                    p.overlay_range_scale(want_item).is_none(),
                                    "{ctx}: only Range settings may expose a rail"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
}

/// THE POINTER Y-CENTER SWEEP: for every row of the Editor facet, across the
/// full world roster, both list styles, and both DPIs — a pointer at that
/// row's own drawn y-CENTER (a) hit-tests back to exactly that row, (b) a
/// passive hover from a DIFFERENT starting selection moves onto exactly that
/// row (never a neighbour, never a no-op), and (c) the corpus index that
/// selection resolves to names the SAME setting the row was built from —
/// band, hit-test, and corpus agree on one item, every row, every world.
#[test]
fn every_editor_row_is_hoverable_at_its_own_y_center_across_the_world_roster() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping every_editor_row_is_hoverable_at_its_own_y_center_across_the_world_roster: \
             no wgpu adapter"
        );
        return;
    };

    let styles = [("pane", None), ("bars", Some(theme::ListStyle::Bars))];
    // Harmless for the "pane" arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));

    let n = editor_overlay().items.len();
    assert!(
        n >= 3,
        "the Editor facet must carry a handful of rows for this sweep to be meaningful"
    );

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for world in crate::theme::world_names() {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for (sname, style) in styles {
                crate::render::set_list_style_test_override(style);

                for target in 0..n {
                    let ctx = format!("world={world} dpi={dpi} list={sname} target={target}");
                    // Render with `target` as the LIVE keyboard selection — the
                    // exact frame a real "Down" press to that row would present
                    // (Settings facets, so a category HEADER precedes its rows;
                    // the item index and the DISPLAY row it lands on are NOT the
                    // same number — `overlay_selected_display_line`/`pr.sel_disp`
                    // is the one owner that maps between them, read here instead
                    // of assumed, so this law can't repeat that off-by-one itself).
                    let mut ov = editor_overlay();
                    ov.selected = target;
                    let v = settings_view(&ov);
                    p.set_view(&v);
                    p.prepare(&device, &queue, 1200, 800).unwrap();
                    let pr = p.overlay_row_y_probe();
                    let card = p.overlay_card_rect().unwrap_or_else(|| {
                        panic!("{ctx}: an open Settings card must expose a rect")
                    });
                    let px = card[0] + card[2] * 0.5;
                    let k = pr.sel_disp;
                    let top = *pr.primary.get(&k).unwrap_or_else(|| {
                        panic!("{ctx}: display row {k} (item {target}) must be drawn")
                    });
                    let py = top + pr.lh * 0.5;
                    // GROUND TRUTH CROSS-CHECK: the row's shaped text top must
                    // agree with the highlight BAND's own top (the y-agreement
                    // law's own invariant, re-asserted here at the exact row this
                    // law's pointer targets).
                    assert!(
                        (top - pr.band_top).abs() <= 1.0,
                        "{ctx}: row {target}'s shaped text top {top} must sit on \
                         its own highlight band top {}",
                        pr.band_top
                    );

                    // (a) HIT-TEST: the pointer's own y-center resolves to exactly
                    // this row — never the row above or below it.
                    let hit = p.overlay_row_at(px, py);
                    assert_eq!(
                        hit,
                        Some(target),
                        "{ctx}: a pointer at row {target}'s own y-center ({py}) must \
                         hit-test to row {target}, not {hit:?} — this IS the \"every \
                         second row\" failure mode at the pixel layer"
                    );

                    // (b) PASSIVE HOVER from a DIFFERENT row lands on exactly this
                    // one — the live `overlay_hover` seam, driven with the SAME
                    // hit-test result the pipeline just returned.
                    let mut hov = editor_overlay();
                    hov.selected = (target + 1) % n; // guaranteed different
                    let moved = hov.hover_at(px, py, hit);
                    assert!(moved || hov.selected == target, "{ctx}: hover must move");
                    assert_eq!(
                        hov.selected, target,
                        "{ctx}: hovering row {target}'s own y-center must select \
                         exactly row {target}"
                    );

                    // (c) CORPUS/SIDECAR AGREEMENT: the item this hover landed on
                    // resolves (via the corpus index a real Enter/click would use)
                    // to the SAME setting the row was drawn from.
                    let ci = hov.selected_corpus_index().unwrap_or_else(|| {
                        panic!("{ctx}: a hovered row must resolve a corpus index")
                    });
                    let drawn_name = &v.overlay_items[target];
                    let corpus_name = hov.rows[ci].accept.clone();
                    // `item_strings()` / `.accept` share one source (`SettingRow::name`
                    // for a Settings row), so this is a real identity check, not a
                    // tautology against the same field twice.
                    assert_eq!(
                        &corpus_name, drawn_name,
                        "{ctx}: the row drawn as {drawn_name:?} must accept as the \
                         same setting, not {corpus_name:?}"
                    );
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
}

/// RANGE-ROW ADJACENCY, at the pixel layer: the Zoom row's own band and
/// its immediate neighbours' bands each resolve to their own item — a rail
/// row must not visually or hit-test-wise bleed into the row above/below it.
/// Sweeps the same world/DPI/style axes as the row-center law above (a
/// smaller, three-row-focused pass; the full sweep already proves every row
/// including Zoom's neighbours individually).
#[test]
fn the_zoom_rows_band_and_its_neighbours_never_bleed_into_one_another() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_zoom_rows_band_and_its_neighbours_never_bleed_into_one_another: \
             no wgpu adapter"
        );
        return;
    };
    let base = editor_overlay();
    let zoom = base
        .items
        .iter()
        .position(|&i| base.rows[i].accept == "Zoom")
        .expect("Zoom row present");
    assert!(
        zoom > 0 && zoom + 1 < base.items.len(),
        "Zoom needs a neighbour on both sides to test adjacency"
    );

    for world in ["Mopoke", "Saltpan", "Firetail"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for row in [zoom - 1, zoom, zoom + 1] {
                let ctx = format!("world={world} dpi={dpi} row={row}");
                let mut ov = editor_overlay();
                ov.selected = row;
                let v = settings_view(&ov);
                p.set_view(&v);
                p.prepare(&device, &queue, 1200, 800).unwrap();
                let pr = p.overlay_row_y_probe();
                let card = p.overlay_card_rect().unwrap();
                let px = card[0] + card[2] * 0.5;
                let k = pr.sel_disp;
                let top = pr.primary[&k];
                let py = top + pr.lh * 0.5;
                assert_eq!(
                    p.overlay_row_at(px, py),
                    Some(row),
                    "{ctx}: rows adjacent to the Zoom rail must still hit-test to \
                     themselves, not the rail row"
                );
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
}
