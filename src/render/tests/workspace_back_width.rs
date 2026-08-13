//! Shared production-shaped fixtures for the workspace teaching-footer laws.
//!
//! The former version of this module kept horizontal overrun, tight-edge and
//! starvation ledgers. The composition law in `workspace_back_height` now asks
//! for the outcome directly: every enrolled footer is shaped, inked, and inside
//! its card at the minimum geometry and ordinary controls. These helpers remain
//! here because the bundled U+232B roster law also needs the real Settings
//! content-stage journey.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::workspace::{BackKey, WorkspaceShape};
use crate::overlay::{OverlayKind, OverlayState};

/// Every workspace whose candidates live in the content pane. Derived from the
/// workspace roster rather than by naming Settings.
pub(super) fn enrolled() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter(|kind| {
            kind.workspace_shape()
                .is_some_and(|shape| !WorkspaceShape::rows_are_primary(shape))
        })
        .collect()
}

/// A real content-stage card, reached through the lifecycle journey.
pub(super) fn card_in_content(kind: OverlayKind) -> OverlayState {
    let mut overlay = OverlayState::new(
        kind,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    overlay.set_facet_lens(0);
    let mut journey = crate::overlay::Journey::seeded(Some(overlay));
    journey.toggle_detail();
    journey.card().expect("the content card is up").clone()
}

/// The same flat projection `App::sync_view` makes for a workspace card.
pub(super) fn content_view(overlay: &OverlayState) -> ViewState {
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = overlay.kind.title();
    v.overlay_items = overlay.item_strings();
    v.overlay_lens = overlay.lens_strip();
    v.overlay_workspace = overlay.workspace_shape().is_some();
    v.overlay_rows_primary = overlay
        .workspace_shape()
        .is_some_and(WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = overlay.detail_focus;
    v.overlay_sections = overlay.item_sections();
    v.overlay_hint = overlay.foot_hint();
    v.overlay_selected = overlay.selected;
    v.overlay_scroll = overlay.scroll;
    v.overlay_window_rows = overlay.window_rows();
    v
}

/// The bundled erase key is not the cost that made the former width ledger
/// fail. At an ordinary control geometry, shape the shipped footer and the
/// focus-key sentence it replaced through every world; the shipped one must be
/// no wider and neither may trigger narrow-card cell yield.
#[test]
fn naming_the_erase_key_shapes_no_wider_than_naming_the_focus_key() {
    let _guard = crate::testlock::serial();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping erase/focus footer comparison: no adapter");
        return;
    };
    let world_pin = crate::theme::WorldPin::snapshot();
    let overlay = card_in_content(
        enrolled()
            .into_iter()
            .next()
            .expect("a content workspace enrolls"),
    );
    let shipped = overlay.foot_hint();
    let previous = shipped.replace(
        &format!("{} back", BackKey::Erase.glyph()),
        &format!("{} back", BackKey::Focus.glyph()),
    );
    assert_ne!(shipped, previous, "the Back-cell substitution must match");

    for (world_index, world) in crate::theme::THEMES.iter().enumerate() {
        crate::theme::set_active(world_index);
        pipeline.sync_theme();
        let mut widths = Vec::new();
        for hint in [&shipped, &previous] {
            let mut v = content_view(&overlay);
            v.overlay_hint = hint.clone();
            v.zoom = 1.0;
            pipeline.set_view(&v);
            pipeline.prepare(&device, &queue, 1200, 800).unwrap();
            let line = pipeline
                .overlay_hint_line()
                .unwrap_or_else(|| panic!("{} omitted an ordinary footer", world.name));
            assert_eq!(
                pipeline.panel_buffer.lines[line].text(),
                hint,
                "{} unexpectedly yielded a footer cell at the ordinary control",
                world.name
            );
            widths.push(
                pipeline
                    .panel_buffer
                    .layout_runs()
                    .find_map(|run| (run.line_i == line).then_some(run.line_w))
                    .expect("the footer line has a shaped run"),
            );
        }
        assert!(
            widths[0] <= widths[1],
            "{}: `{} back` shapes {:.1}px, wider than `{} back` at {:.1}px",
            world.name,
            BackKey::Erase.glyph(),
            widths[0],
            BackKey::Focus.glyph(),
            widths[1]
        );
    }
    drop(world_pin);
}
