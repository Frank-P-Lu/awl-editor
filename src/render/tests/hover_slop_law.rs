//! ITEM 106 — THE MOVEMENT-SLOP GATE AGAINST REAL PIPELINE GEOMETRY: the
//! render-level companion to `overlay::tests`' pure `hover_at` laws, proving
//! the item's own named live hazard end to end against a REAL
//! `TextPipeline` — not merely hypothesized. A keyboard-driven `move_sel`
//! that scrolls the candidate WINDOW really does change which item a
//! stationary, physically-unmoved pixel hit-tests to (`overlay_row_at`); the
//! gate really does refuse to read that as a pointer gesture; and a genuine
//! pointer move still takes over immediately. Swept across both list styles
//! (`Pane`/`Bars`) and 1×/2× DPI, mirroring `settings_row_reach_law`'s own
//! sweep shape for item 104.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};

/// A flat (non-faceted) Goto-shaped picker over `n` synthetic rows — no
/// `overlay_lens`, so `overlay_geometry` takes the plain candidate-window
/// path (never `theme_overlay_geometry`), exactly like every non-Theme
/// picker in the affected-surface roster.
fn goto_overlay(n: usize) -> OverlayState {
    let corpus: Vec<String> = (0..n).map(|i| format!("row{i}")).collect();
    OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![])
}

fn goto_view(ov: &OverlayState) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Goto.title();
    v.overlay_items = ov.item_strings();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v
}

#[test]
fn a_keyboard_scroll_moves_what_a_stationary_pixel_hits_and_the_gate_refuses_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping a_keyboard_scroll_moves_what_a_stationary_pixel_hits_and_the_gate_refuses_it: \
             no wgpu adapter"
        );
        return;
    };

    let styles = [
        ("pane", None),
        (
            "bars",
            Some(theme::ListStyle::Bars {
                radius: 6.0,
                gap: 8.0,
                grow_px: 24.0,
                extent: theme::BarExtent::FullWidth,
                coverage: theme::BarCoverage::All,
            }),
        ),
    ];

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for (sname, style) in styles {
            crate::render::set_list_style_test_override(style);
            let ctx = format!("dpi={dpi} list={sname}");

            // 40 rows, window 12 (the flat-picker default). Establish a REAL,
            // earlier hover on row 3's own drawn y-center.
            let mut ov = goto_overlay(40);
            let v0 = goto_view(&ov);
            p.set_view(&v0);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            let pr0 = p.overlay_row_y_probe();
            let card0 = p
                .overlay_card_rect()
                .unwrap_or_else(|| panic!("{ctx}: an open Goto card must expose a rect"));
            let px = card0[0] + card0[2] * 0.5;
            let top3 = *pr0.primary.get(&3).unwrap_or_else(|| {
                panic!("{ctx}: display row 3 must be drawn in a fresh 40-row list")
            });
            let py = top3 + pr0.lh * 0.5;
            let hit0 = p.overlay_row_at(px, py);
            assert_eq!(hit0, Some(3), "{ctx}: the probe pixel must start on row 3");
            assert!(
                ov.hover_at(px, py, hit0),
                "{ctx}: the initial real hover selects row 3"
            );
            assert_eq!(ov.selected, 3, "{ctx}");

            // A REAL keyboard session: Down deep enough that the window must
            // scroll (selected 3 -> 25, well past the 12-row window).
            ov.move_sel(22);
            assert_eq!(ov.selected, 25, "{ctx}");
            assert!(
                ov.scroll > 0,
                "{ctx}: the keyboard session must have actually scrolled the window"
            );
            // `App::apply`'s stamp: the pointer never moved, so this re-anchors
            // to the SAME (px, py) the last hover left.
            ov.arm_hover_baseline(px, py);

            // Re-render at the scrolled state and re-hit-test the SAME
            // physical pixel the pointer never left.
            let v1 = goto_view(&ov);
            p.set_view(&v1);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            let pr1 = p.overlay_row_y_probe();
            let hit1 = p.overlay_row_at(px, py);
            assert!(
                hit1.is_some(),
                "{ctx}: the scrolled window still draws SOME row at that pixel"
            );
            assert_ne!(
                hit1,
                Some(25),
                "{ctx}: the row now under the stationary pixel is a DIFFERENT item than the \
                 keyboard's own selection — exactly the hazard's premise (item 106)"
            );

            // THE LAW: a REAL 1px jitter off the parked pixel — not the exact
            // same coordinate (item 85's own exact-equality gate already
            // refused a bare duplicate; this law's own regression needs
            // genuine, if tiny, travel) — re-checked through the real gate,
            // must not steal the keyboard's selection.
            let (px_j, py_j) = (px + 1.0, py);
            let hit1_j = p.overlay_row_at(px_j, py_j);
            assert!(
                !ov.hover_at(px_j, py_j, hit1_j),
                "{ctx}: a 1px jitter off a stationary pointer, over a window that scrolled under it, \
                 must not steal the keyboard's selection"
            );
            assert_eq!(
                ov.selected, 25,
                "{ctx}: the keyboard selection survives the scroll"
            );

            // AND real motion still works normally: a pointer move to a
            // DIFFERENT display row (well past any reasonable slop — a full
            // row height away, not a jitter) takes over immediately, on the
            // very first such event.
            let top1_0 = *pr1
                .primary
                .get(&0)
                .unwrap_or_else(|| panic!("{ctx}: display row 0 must be drawn"));
            let py2 = top1_0 + pr1.lh * 0.5;
            let hit2 = p.overlay_row_at(px, py2);
            assert!(
                hit2.is_some(),
                "{ctx}: display row 0 must hit-test to a real item"
            );
            assert!(
                ov.hover_at(px, py2, hit2),
                "{ctx}: a genuine pointer move to a different row must still take over immediately"
            );
            assert_eq!(ov.selected, hit2.unwrap(), "{ctx}");
        }
    }
    crate::render::set_list_style_test_override(None);
}
