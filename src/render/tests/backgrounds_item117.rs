//! Item 117's GPU laws: the organic ground is Bowerbird-only, stays outside
//! the page column, and remains visibly-but-quietly populated at dashboard
//! narrow/wide Room geometries.
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg, render_bg_ambient};
use crate::background::AmbientUpload;
use crate::theme;

#[test]
fn organic_is_bowerbird_alone_no_wildcard() {
    for t in theme::THEMES {
        let organic = match t.background {
            theme::Background::Gradient { .. } => false,
            theme::Background::Dots { .. } => false,
            theme::Background::Pinstripe { .. } => false,
            theme::Background::Stripes { .. } => false,
            theme::Background::Lava { .. } => false,
            theme::Background::Bands { .. } => false,
            theme::Background::Waves { .. } => false,
            theme::Background::Zigzag { .. } => false,
            theme::Background::Organic { .. } => true,
            theme::Background::Deckle { .. } => false,
            theme::Background::WarpedGrid { .. } => false,
        };
        assert_eq!(organic, t.name == "Bowerbird", "{} organic roster", t.name);
    }
}

#[test]
fn organic_occupies_dashboard_margins_but_never_the_page() {
    let _g = crate::testlock::serial();
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

// `organic_phase_moves_and_wraps_without_a_catchup_jump` — DELETED. It was
// the sharpest vacuous law found in this repo, and the reason Bowerbird's
// field-translation pop shipped unguarded. Three compounding reasons it
// could never fail: (a) it called `waves_drift_radians`, a function Organic
// never read (Organic computed its own drift inline, `render/layers.rs`'s
// `is_organic()` arm); (b) it never applied the `0.73` term that actually
// broke; and (c) what remained asserted the 2*pi-periodicity of `sin`/`cos`
// themselves, which cannot fail for any value of any constant in this repo.
// Its replacement — `every_ambient_consumers_shader_term_is_continuous_
// across_the_wrap`, sweeping every ambient ground's own drift-to-shader term
// through the REAL shader, mutation-proven against a reintroduced non-integer
// rate — lives in `render/tests/ambient_wrap_law.rs`. Organic's own
// translation is gone outright: a bower is an arrangement, deliberately
// placed and then left alone; see
// `organic_phase_sweep_stays_cool_at_every_breathe_phase` below for what
// replaced this test's density-mutation half.

#[test]
fn organic_phase_sweep_stays_cool_at_every_breathe_phase() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let bg = theme::BOWERBIRD.background;
    let (w, h, left, col) = (1200, 800, 350.0, 500.0);
    let mut peak_marks = 0usize;
    // The sweep drives `organic_phase` (raw cycles, the companion breathe's
    // own input), not `drift` (radians): the ground no longer reads
    // `g.drift` at all, so sweeping THAT would silently sweep nothing and
    // this law would go vacuous exactly the way its deleted predecessor did.
    for phase in [0.0, 0.37, 1.0, 1.63, 1.9999] {
        let pixels = render_bg_ambient(
            &device,
            &queue,
            bg_desc_for(bg),
            w,
            h,
            left,
            col,
            AmbientUpload {
                organic_phase: phase,
                ..Default::default()
            },
            1.0,
        );
        for (i, p) in pixels.iter().enumerate() {
            let x = (i as u32) % w;
            if (x as f32) >= left && (x as f32) < left + col {
                continue;
            }
            // Bowerbird's Frame remains navy/cool: blue must dominate red and
            // no phase may approach the warm caret's high-red/green hue.
            assert!(
                p[2] >= p[0] && p[0] < 90,
                "phase {phase}: warm/bright frame pixel {p:?}"
            );
            peak_marks += usize::from(p[2] > 35);
        }
    }
    assert!(
        peak_marks > 500,
        "all-phase field must visibly occupy the Frame"
    );
    let mut mutant = bg_desc_for(bg);
    mutant.density = 0.0;
    let muted = render_bg(&device, &queue, mutant, w, h, left, col, 0.0);
    let normal = render_bg(&device, &queue, bg_desc_for(bg), w, h, left, col, 0.0);
    let changed = normal
        .iter()
        .zip(muted.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        changed > 500,
        "mutation witness: density=0 must visibly remove the field"
    );
}

#[test]
fn organic_freeze_conditions_resolve_to_the_settled_phase() {
    let settled = crate::lava::LAVA_FROZEN_PHASE;
    assert_eq!(crate::lava::lava_phase_for(0.63, true, None), settled);
    // `ambient_motion = false` uses the same App freeze door before render.
    assert_eq!(crate::lava::lava_phase_for(settled, false, None), settled);
}
