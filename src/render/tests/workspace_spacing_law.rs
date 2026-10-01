//! Sustained workspaces spend more vertical room than transient pickers.
//! The row-pitch owner is shared by shaping, planning, bands, hit testing and
//! sidecar projection; this law reaches it through a real Settings workspace
//! over neighboring staged/wide windows, every world, and both DPI tiers.

use super::super::*;
use super::{headless_dqp, settings_overlay_view, settings_values, view};
use crate::overlay::{OverlayKind, OverlayState};

const WORKSPACE_AIR: f32 = 5.0;

// Shipped density stages the capped Settings pane. The explicit compact
// counterfactual reaches its two-region geometry without changing that policy.
struct DensityPin;

impl DensityPin {
    fn install(compact: bool) -> Self {
        crate::render::overrides::set_overlay_density_test_override(compact.then_some(
            crate::render::overrides::TypeDensity {
                scale: 0.75,
                leading: 0.0,
            },
        ));
        Self
    }
}

impl Drop for DensityPin {
    fn drop(&mut self) {
        crate::render::overrides::set_overlay_density_test_override(None);
    }
}

fn settings_in_content() -> OverlayState {
    let values = settings_values(1.0, 1.0);
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&values));
    ov.set_range_cells(crate::settings::visible_range_cells(&values));
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    journey.toggle_detail();
    journey.card().expect("Settings stays open").clone()
}

fn contextual_view() -> ViewState {
    let mut v = view("", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands".into();
    v.overlay_items = vec!["Open".into(), "Save".into()];
    v
}

fn assert_workspace_arrangements(
    p: &mut TextPipeline,
    roster: &[(OverlayKind, crate::overlay::workspace::WorkspaceShape)],
    world: &str,
    dpi: f32,
    contextual_pitch: f32,
) {
    for (kind, shape) in roster {
        let mut neighbor = contextual_view();
        neighbor.overlay_workspace = true;
        neighbor.overlay_rows_primary = shape.rows_are_primary();
        p.set_view(&neighbor);
        let added = p.overlay_lh() - contextual_pitch;
        let want = if shape.rows_are_primary() {
            0.0
        } else {
            WORKSPACE_AIR * dpi
        };
        assert!(
            (added - want).abs() < 0.02,
            "world={world} dpi={dpi} {kind:?}/{shape:?}: added {added}px, want {want}px"
        );
    }
}

#[test]
fn settings_rows_keep_workspace_air_across_worlds_widths_and_dpi() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(2400.0, 1600.0) else {
        eprintln!("skipping workspace spacing law: no wgpu adapter");
        return;
    };
    let worlds: Vec<_> = crate::theme::THEMES.iter().map(|t| t.name).collect();
    assert!(
        worlds.len() >= 2 && worlds.contains(&"Saltpan"),
        "the roster must include the reported world and neighbors"
    );
    let ov = settings_in_content();
    assert!(
        ov.kind.workspace_shape().is_some(),
        "the enrolled Settings surface must really be a workspace"
    );
    let mut cells = 0usize;
    let mut width_classes = std::collections::BTreeSet::new();
    let roster: Vec<_> = OverlayKind::ALL
        .iter()
        .copied()
        .filter_map(|kind| kind.workspace_shape().map(|shape| (kind, shape)))
        .collect();
    assert!(
        roster.iter().any(|(_, shape)| !shape.rows_are_primary())
            && roster.iter().any(|(_, shape)| shape.rows_are_primary()),
        "both workspace arrangements must enroll themselves: {roster:?}"
    );

    for world in worlds {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0_f32, 2.0] {
            p.set_dpi(dpi);
            for compact in [false, true] {
                let _density = DensityPin::install(compact);
                p.set_view(&contextual_view());
                let contextual_pitch = p.overlay_lh();
                assert_workspace_arrangements(&mut p, &roster, world, dpi, contextual_pitch);
                for (logical_w, logical_h) in [(480_u32, 640_u32), (760, 720), (1200, 800)] {
                    let (w, h) = (
                        (logical_w as f32 * dpi).round() as u32,
                        (logical_h as f32 * dpi).round() as u32,
                    );
                    p.set_size(w as f32, h as f32);
                    let mut v = settings_overlay_view(&ov, ov.window_rows());
                    v.overlay_title = ov.kind.title().to_string();
                    v.overlay_lens = ov.lens_strip();
                    v.overlay_hint = ov.foot_hint();
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();

                    let ctx = format!(
                        "world={world} size={logical_w}x{logical_h} dpi={dpi} compact={compact}"
                    );
                    let workspace_pitch = p.overlay_lh();
                    let added = workspace_pitch - contextual_pitch;
                    assert!(
                        (added - WORKSPACE_AIR * dpi).abs() < 0.02,
                        "{ctx}: workspace adds {added}px, want {}px",
                        WORKSPACE_AIR * dpi
                    );
                    assert!(
                        added >= 4.0 * dpi,
                        "{ctx}: removing the breathing-room term must fail this floor"
                    );

                    let geom = p.overlay_geometry(w);
                    let plan = p.overlay_row_plan(&geom);
                    let rows = plan.rows();
                    assert!(
                        rows.len() >= 2,
                        "{ctx}: Settings must expose neighboring rows"
                    );
                    let planned_pitch = rows[1].top - rows[0].top;
                    assert!(
                        (planned_pitch - workspace_pitch).abs() < 0.02
                            && (rows[0].height - workspace_pitch).abs() < 0.02,
                        "{ctx}: pitch/height ({planned_pitch}/{}) != owner {workspace_pitch}",
                        rows[0].height
                    );
                    let x = geom.row_text_left() + geom.row_text_w() * 0.5;
                    for row in rows.iter().take(2) {
                        let item = row.item.expect("Settings candidate rows name items");
                        assert_eq!(
                            p.overlay_row_at(x, row.top + row.height * 0.5),
                            Some(item),
                            "{ctx}: the roomier drawn row must remain clickable"
                        );
                    }
                    width_classes.insert(p.workspace_is_wide(w));
                    cells += 1;
                }
            }
        }
    }

    assert_eq!(
        width_classes,
        std::collections::BTreeSet::from([false, true]),
        "the neighboring widths must cross staged and two-region workspace geometry"
    );
    assert_eq!(cells, crate::theme::THEMES.len() * 2 * 2 * 3);
}
