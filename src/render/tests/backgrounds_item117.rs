//! Item 117's GPU laws: the organic ground is Bowerbird-only, stays outside
//! the page column, and remains visibly-but-quietly populated at dashboard
//! narrow/wide Room geometries.
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use crate::theme;

#[test]
fn organic_is_bowerbird_alone_no_wildcard() {
    for t in theme::THEMES {
        let organic = match t.background {
            theme::Background::Gradient { .. } => false,
            theme::Background::Dots { .. } => false,
            theme::Background::Starfield { .. } => false,
            theme::Background::Pinstripe { .. } => false,
            theme::Background::Stripes { .. } => false,
            theme::Background::Lava { .. } => false,
            theme::Background::Bands { .. } => false,
            theme::Background::Waves { .. } => false,
            theme::Background::Zigzag { .. } => false,
            theme::Background::Organic { .. } => true,
        };
        assert_eq!(organic, t.name == "Bowerbird", "{} organic roster", t.name);
    }
}

#[test]
fn organic_occupies_dashboard_margins_but_never_the_page() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let bg = theme::BOWERBIRD.background;
    for (w, h, left, col) in [(900, 700, 125.0, 650.0), (1800, 1000, 350.0, 1100.0)] {
        let full = render_bg(&device, &queue, bg_desc_for(bg), w, h, left, col, 0.0);
        let mut bare = bg_desc_for(bg);
        bare.density = 0.0;
        let flat = render_bg(&device, &queue, bare, w, h, left, col, 0.0);
        let delta = |x: u32, y: u32| -> i32 {
            (0..3)
                .map(|k| {
                    (full[(y * w + x) as usize][k] as i32 - flat[(y * w + x) as usize][k] as i32)
                        .abs()
                })
                .sum()
        };
        let margin_marks = (0..w)
            .flat_map(|x| (0..h).map(move |y| (x, y)))
            .filter(|&(x, _)| (x as f32) < left || (x as f32) >= left + col)
            .filter(|&(x, y)| delta(x, y) >= 3)
            .count();
        assert!(
            margin_marks > (w * h / 150) as usize,
            "{w}x{h} field is too sparse"
        );
        assert!(
            (left as u32..(left + col) as u32).all(|x| (0..h).all(|y| delta(x, y) == 0)),
            "{w}x{h} organic ink entered page"
        );
    }
}

#[test]
fn organic_phase_moves_and_wraps_without_a_catchup_jump() {
    let p0 = crate::background::waves_drift_radians(0.0);
    let p1 = crate::background::waves_drift_radians(crate::lava::LAVA_LOOP_CYCLES);
    assert!(
        (p0.sin() - p1.sin()).abs() < 1e-5 && (p0.cos() - p1.cos()).abs() < 1e-5,
        "shared settled phase wraps exactly"
    );
}
