//! FORMAT POPOVER — real shaped label spans and their shared click geometry.

use super::{headless_dqp, view};

/// Every label's shaped span must activate its own button. This drives the real
/// layout pipeline at ordinary, narrow, and Retina/zoomed scales, so changing a
/// label's measured advance cannot make drawing and pointer activation diverge.
#[test]
fn shaped_popover_labels_and_hits_agree_at_narrow_zoom_and_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(880.0, 800.0) else {
        eprintln!("skipping shaped_popover_labels_and_hits_agree: no wgpu adapter");
        return;
    };

    for (zoom, dpi) in [(1.0, 1.0), (1.2, 1.0), (1.2, 2.0)] {
        p.set_dpi(dpi);
        let text = "select this word\n";
        let mut v = view(text, 0, 11);
        v.zoom = zoom;
        v.selection = Some(((0, 7), (0, 11)));
        v.popover = crate::actions::popover::plan(text, Some(7), 11, true);
        p.set_view(&v);
        p.prepare(&device, &queue, 880, 800).unwrap();

        let (card, buttons) = p.popover_report().expect("popover lays out");
        assert_eq!(
            buttons
                .iter()
                .map(|(label, _, _)| label.as_str())
                .collect::<Vec<_>>(),
            vec!["B", "I", "A", "code", "S", "H", "link"],
            "dpi {dpi} zoom {zoom}: exact popover roster"
        );
        let mid_y = card[1] + card[3] * 0.5;
        for ((label, _, [x0, x1]), button) in buttons.iter().zip(crate::popover::PopoverButton::ALL)
        {
            assert!(
                x1 > x0,
                "{label} has a real shaped span at dpi {dpi} zoom {zoom}"
            );
            assert_eq!(
                p.popover_hit((x0 + x1) * 0.5, mid_y),
                Some(button),
                "dpi {dpi} zoom {zoom}: painted {label:?} span hits its own button"
            );
        }
    }
}
