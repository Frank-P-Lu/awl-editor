//! Controls workspaces must replace document ink even on bare-list worlds.
//! The hidden document changes while the real Settings card remains identical;
//! a second pair with the card closed proves that the document actually drew.

use super::super::*;
use super::headless_dqp;
use super::pixeldiff::{Region, diff_region, render_frame};
use crate::overlay::OverlayKind;

#[test]
fn controls_workspace_hides_document_ink_on_every_world() {
    let _g = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let dense = format!(
        "# Heading behind Settings\n\n{}",
        "Document ink crosses controls unless the workspace is opaque.\n".repeat(28)
    );
    let kind = OverlayKind::Settings;
    let shape = kind.workspace_shape().expect("Settings is a workspace");
    assert!(!shape.rows_are_primary());
    let mut graded = 0;
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).unwrap();
        for dpi in [1.0f32, 2.0] {
            for (lw, lh) in [(641u32, 800u32), (1180, 812)] {
                for detail in [false, true] {
                    let (w, h) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    // Fresh pipeline per cell: no cross-world geometry cache assumption.
                    let (device, queue, mut p) = headless_dqp(w as f32, h as f32)
                        .expect("workspace opacity law requires a GPU adapter");
                    p.set_dpi(dpi);
                    let ov = super::workspace::workspace_card(0, detail);
                    let mut v = super::workspace::workspace_view(&ov);
                    v.overlay_crisp = kind.keeps_backdrop_crisp();
                    v.overlay_rows_primary = shape.rows_are_primary();
                    v.is_markdown = true;
                    v.text.clone_from(&dense);
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let geom = p.overlay_geometry(w);
                    assert!(geom.workspace && p.overlay_text_glyph_count() > 10);
                    let card = geom.card_probe();
                    let inset = p.overlay_card_opaque_inset_probe(card).max(6.0 * dpi);
                    let region = Region::new(
                        card[0] + inset,
                        card[1] + inset,
                        card[2] - 2.0 * inset,
                        card[3] - 2.0 * inset,
                    );
                    let covered_dense = render_frame(&mut p, &device, &queue, w, h);
                    v.text.clear();
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let covered_blank = render_frame(&mut p, &device, &queue, w, h);
                    v.overlay_active = false;
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let bare_blank = render_frame(&mut p, &device, &queue, w, h);
                    v.text.clone_from(&dense);
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let bare_dense = render_frame(&mut p, &device, &queue, w, h);
                    let context = format!("{world}@{dpi}/{lw}x{lh}/detail={detail}");
                    assert!(
                        diff_region(&bare_dense, &bare_blank, w as i64, h as i64, region).differing
                            > 100,
                        "{context}: the covered document must visibly change without the card"
                    );
                    let leak =
                        diff_region(&covered_dense, &covered_blank, w as i64, h as i64, region);
                    assert_eq!(
                        leak.differing, 0,
                        "{context}: document ink leaked into the controls workspace: {leak:?}"
                    );
                    graded += 1;
                }
            }
        }
    }
    assert_eq!(graded, theme::THEMES.len() * 8);
}
