//! ITEM 194 — ONE CAMERA, ONE PROJECTED CYLINDER, CROPPED AT THE PAGE.
//!
//! Item 132's own laws (`backgrounds_item132`) prove the ground exists, stays
//! out of the page, keeps its hierarchy and never aliases. They did NOT prove
//! the thing the live motion review failed the world on: that the two margins
//! are two crops of ONE projection rather than two independently composed
//! fields. This module is that proof, and it is entirely arithmetic over real
//! GPU pixels — the differential `field` oracle item 86 authored, so the flat
//! ground cancels and what is measured is the lattice alone.
//!
//! TWO CLAIMS, TWO METHODS.
//!
//! 1. **The composition.** At the settled straight pose, the outermost strong
//!    cross-ring is TRACED across each margin and its ellipse solved for. The
//!    page hides `col_w / 2u` of that section's own width; the target is a
//!    third, so each margin shows a third directly and reads two thirds — its
//!    own third plus the third behind the page that both margins imply.
//!    Recovering `u` from an ARC rather than from a whole ring is the point:
//!    the section is three page widths across, so no margin narrower than the
//!    page can hold its apex — the eye infers the opening from the flank, and
//!    so does this law.
//!
//! 2. **The coherence, at every route pose.** Each margin's ink is fitted, ON
//!    ITS OWN, for the camera that would explain it, by the phase score below:
//!    a candidate steering that is right puts the marks ON its own integer
//!    level sets and a candidate that is wrong scatters them. The two margins
//!    must recover the SAME camera — which is one statement of all three things
//!    the review found disagreeing, since horizon, curvature and vanishing
//!    direction are each a function of that one pose.
//!
//! The fit needs a host model of the projection, which item 132 deliberately
//! avoided for the ROUTE (the route's consumer is Rust, so a WGSL mirror could
//! only drift). This one is the opposite case and follows the `lava`/`dither`
//! precedent instead: the GPU is the only consumer of the projection, the model
//! here exists for the laws alone, and `the_host_model_matches_the_shaders_own
//! _constants` holds the two in lockstep by name.

use super::backgrounds_item69::headless_dq;
use super::backgrounds_item132::{COL_LEFT, COL_W, H, INK_FLOOR, W, field, kite, with_tunnel};
use crate::theme;
use crate::warpgrid;

// --- The host model of the projection (see the module doc) ------------------

const SECTION_PAGE_RATIO: f32 = 3.0;
const ASPECT_FIT: f32 = 0.42;
const ASPECT_MIN: f32 = 1.0;
const ASPECT_MAX: f32 = 4.0;
const RAILS_PER_HALF_TURN: f32 = 10.0;
const RPO_MIN: f32 = 3.0;
const RPO_MAX: f32 = 14.0;
const BEND_GAIN: f32 = 0.075;
const CORE_FRAC: f32 = 0.055;
const SOLVE_STEPS: usize = 20;
/// A candidate that misses by more than this many pixels has missed; capping it
/// keeps one wildly wrong prediction from outvoting a whole margin of evidence.
const MISS_CEILING_PX: f64 = 8.0;

/// The geometry every margin of one frame shares — derived from the page column
/// and the room, exactly as the shader derives it.
#[derive(Clone, Copy)]
struct Camera {
    axis_x: f32,
    axis_y: f32,
    anchor: f32,
    aspect: f32,
    rpo: f32,
}

impl Camera {
    fn new(w: u32, h: u32, col_left: f32, col_w: f32, spacing_px: f32) -> Self {
        let page_half = (col_w * 0.5).max(1.0);
        let anchor = SECTION_PAGE_RATIO * page_half;
        let flank = (anchor * anchor - page_half * page_half).max(1.0).sqrt();
        let _ = w;
        Camera {
            axis_x: col_left + col_w * 0.5,
            axis_y: h as f32 * 0.5,
            anchor,
            aspect: (flank / (ASPECT_FIT * h as f32).max(1.0)).clamp(ASPECT_MIN, ASPECT_MAX),
            rpo: (page_half * std::f32::consts::LN_2 / spacing_px).clamp(RPO_MIN, RPO_MAX),
        }
    }

    /// The `(ring, rail)` lattice coordinates of one pixel under a candidate
    /// steering — the shader's own arithmetic, bisection and all.
    fn lattice(&self, px: f32, py: f32, p: Steer) -> (f32, f32) {
        let (yaw, pitch, curvature, forward) = (p.yaw, p.pitch, p.curvature, p.forward);
        let q = (px - self.axis_x, (py - self.axis_y) * self.aspect);
        let g = BEND_GAIN * curvature * self.anchor * self.anchor;
        let bend = (g * yaw, g * pitch * self.aspect);
        let core = CORE_FRAC * self.anchor;
        let ql = (q.0 * q.0 + q.1 * q.1).sqrt();
        let bl = (bend.0 * bend.0 + bend.1 * bend.1).sqrt();
        let mut lo = core;
        let mut hi = (0.5 * (ql + (ql * ql + 4.0 * bl).sqrt())).max(core);
        let h_of = |d: f32| {
            let s = (q.0 - bend.0 / d, q.1 - bend.1 / d);
            (s.0 * s.0 + s.1 * s.1).sqrt() - d
        };
        for _ in 0..SOLVE_STEPS {
            let mid = 0.5 * (lo + hi);
            if h_of(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let d = (0.5 * (lo + hi)).max(core);
        let w = (q.0 - bend.0 / d, q.1 - bend.1 / d);
        let u = (w.0 * w.0 + w.1 * w.1).sqrt().max(core);
        (
            self.rpo * (self.anchor / u).log2() - forward,
            w.1.atan2(w.0) * (RAILS_PER_HALF_TURN / std::f32::consts::PI),
        )
    }
}

/// One candidate camera: the steering being tested, the world's own bend gain,
/// and the FORWARD travel the route has already made — which is not optional.
/// Forward travel is a pure subtraction from the ring coordinate, so leaving it
/// out shifts every predicted cross-ring by its fractional part and the fit
/// then reads a perfectly coherent field as explained by nothing.
#[derive(Clone, Copy)]
struct Steer {
    yaw: f32,
    pitch: f32,
    curvature: f32,
    forward: f32,
}

/// Kite's own authored dials, read from the world rather than restated.
fn kite_dials() -> (f32, f32) {
    match kite() {
        theme::Background::WarpedGrid {
            spacing_px,
            curvature,
            ..
        } => (spacing_px, curvature),
        _ => unreachable!("Kite's ground is the warped grid"),
    }
}

// --- Pixel measurement -------------------------------------------------------

/// `[x0, x1)` of the left and right margins at the canonical geometry.
fn margins() -> [(u32, u32); 2] {
    let right = (COL_LEFT + COL_W).ceil() as u32;
    [(0, COL_LEFT.floor() as u32), (right, W)]
}

/// The strength above which a mark is one of the MAJOR (every fifth) lines,
/// derived from the frame's OWN ink distribution rather than from a colour
/// constant: the majors are the top of the observed range by construction, and
/// half of the near-maximum separates the two rungs cleanly (item 132's own
/// hierarchy law proves the rungs are genuinely distinct).
fn major_floor(f: &[i32], m: (u32, u32)) -> i32 {
    let mut v: Vec<i32> = (0..H)
        .flat_map(|y| (m.0..m.1).map(move |x| (y, x)))
        .map(|(y, x)| f[(y * W + x) as usize])
        .filter(|&i| i > INK_FLOOR)
        .collect();
    v.sort_unstable();
    let hi = v[v.len() * 995 / 1000];
    (hi / 2).max(INK_FLOOR * 3)
}

fn at(f: &[i32], x: u32, y: u32) -> i32 {
    f[(y * W + x) as usize]
}

/// THE HORIZON, measured and not assumed: the row about which the margin's own
/// ink profile is most nearly its own reflection. A tunnel's cross-rings are
/// symmetric about their centre row, so this is that row.
fn horizon(f: &[i32], m: (u32, u32)) -> u32 {
    let prof: Vec<f64> = (0..H)
        .map(|y| (m.0..m.1).map(|x| at(f, x, y) as f64).sum::<f64>())
        .collect();
    let (mut best, mut best_c) = (-1.0f64, H / 2);
    for c in (H * 3 / 10)..(H * 7 / 10) {
        let k = (c.min(H - 1 - c)).min(H * 28 / 100) as usize;
        let (mut dot, mut na, mut nb) = (0.0, 0.0, 0.0);
        for i in 1..k {
            let a = prof[c as usize - i];
            let b = prof[c as usize + i];
            dot += a * b;
            na += a * a;
            nb += b * b;
        }
        let s = dot / (na.sqrt() * nb.sqrt() + 1e-9);
        if s > best {
            best = s;
            best_c = c;
        }
    }
    best_c
}

/// Trace the OUTERMOST major cross-ring across one margin: seed on the scanline
/// where the page-edge quiet has finished, take the strongest arc furthest from
/// the horizon, and follow it outward one column at a time with a one-step
/// linear prediction, so the walk cannot hop onto a neighbouring ring.
fn trace_outer_ring(f: &[i32], m: (u32, u32), cy: u32, left: bool) -> Vec<(f32, f32)> {
    let maj = major_floor(f, m);
    let quiet = 80u32;
    let seed = if left { m.1 - quiet } else { m.0 + quiet };
    let reach = (H * 44 / 100) as i32;
    let mut pts = Vec::new();
    for up in [true, false] {
        let ys: Vec<u32> = if up {
            ((cy as i32 - reach).max(1) as u32..cy - 24).collect()
        } else {
            (cy + 24..(cy as i32 + reach).min(H as i32 - 2) as u32)
                .rev()
                .collect()
        };
        let Some(&y0) = ys.iter().find(|&&y| {
            at(f, seed, y) >= maj
                && at(f, seed, y) >= at(f, seed, y - 1)
                && at(f, seed, y) >= at(f, seed, y + 1)
        }) else {
            continue;
        };
        pts.push((seed as f32, y0 as f32));
        let (mut y, mut prev) = (y0 as i32, None::<i32>);
        let mut x = seed as i32;
        loop {
            x += if left { -1 } else { 1 };
            if x < m.0 as i32 + 2 || x >= m.1 as i32 - 2 {
                break;
            }
            let pred = prev.map_or(y, |p| 2 * y - p);
            let mut best = None;
            for yy in (pred - 7).max(1)..(pred + 8).min(H as i32 - 2) {
                let (xu, yu) = (x as u32, yy as u32);
                if at(f, xu, yu) >= maj
                    && at(f, xu, yu) >= at(f, xu, yu - 1)
                    && at(f, xu, yu) >= at(f, xu, yu + 1)
                    && best.is_none_or(|b: i32| (yy - pred).abs() < (b - pred).abs())
                {
                    best = Some(yy);
                }
            }
            let Some(b) = best else { break };
            prev = Some(y);
            y = b;
            pts.push((x as f32, y as f32));
        }
    }
    pts
}

/// Solve the traced arc for the section it belongs to, in the page's own frame:
/// `(x - cx)^2 + aspect^2 (y - cy)^2 = u^2`, least squares in `(aspect^2, u^2)`.
/// The centre is the PAGE COLUMN's own centre line and the MEASURED horizon —
/// which is the frame the claim is about ("the page hides a third of it"), not
/// an assumption about the answer: the coherence law below recovers the camera
/// from each margin without being told anything.
fn section_of(pts: &[(f32, f32)], cx: f32, cy: f32) -> Option<(f32, f32)> {
    if pts.len() < 60 {
        return None;
    }
    let (mut saa, mut sab, mut sbb, mut sav, mut sbv) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for &(x, y) in pts {
        let a = -((y - cy) as f64).powi(2);
        let b = 1.0f64;
        let v = ((x - cx) as f64).powi(2);
        saa += a * a;
        sab += a * b;
        sbb += b * b;
        sav += a * v;
        sbv += b * v;
    }
    let det = saa * sbb - sab * sab;
    if det.abs() < 1e-6 {
        return None;
    }
    let a2 = (sav * sbb - sbv * sab) / det;
    let u2 = (saa * sbv - sab * sav) / det;
    (a2 > 0.0 && u2 > 0.0).then(|| (a2.sqrt() as f32, u2.sqrt() as f32))
}

// ---------------------------------------------------------------------------
// 1. THE COMPOSITION — the page hides a third of the section it crops.
// ---------------------------------------------------------------------------

/// THE STRAIGHT-POSE TARGET, measured over pixels. The review's first sentence
/// was that each margin "shows too little of the cross-section and reads as a
/// separately cropped circle": both halves of that are settled here, because a
/// separately cropped circle re-planted at a page edge cannot have its centre
/// behind the page and cannot agree with the other margin's about the section's
/// size.
#[test]
fn the_page_hides_a_third_of_the_cross_section_both_margins_crop() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let f = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    let cx = COL_LEFT + COL_W * 0.5;
    let mut hidden = Vec::new();
    let mut sections = Vec::new();
    for (i, m) in margins().into_iter().enumerate() {
        let cy = horizon(&f, m);
        assert!(
            (cy as i32 - (H / 2) as i32).abs() <= 8,
            "margin {i}: the tunnel's horizon must sit on the room's own middle \
             at a straight pose, measured {cy} against {}",
            H / 2
        );
        let pts = trace_outer_ring(&f, m, cy, i == 0);
        let (aspect, u) = section_of(&pts, cx, cy as f32).unwrap_or_else(|| {
            panic!(
                "margin {i}: the outermost cross-ring did not solve for a section \
                 at all ({} traced points) — the margin is not showing an arc of \
                 one opening",
                pts.len()
            )
        });
        assert!(
            (1.0..=4.0).contains(&aspect),
            "margin {i}: the section's own flattening must be the room's \
             ({aspect} is outside the shader's own clamp)"
        );
        hidden.push(COL_W / (2.0 * u));
        sections.push((u, aspect, pts.len()));
    }
    for (i, &h) in hidden.iter().enumerate() {
        assert!(
            (0.30..=0.37).contains(&h),
            "margin {i}: the page must hide about a THIRD of the tunnel's \
             cross-section — each margin then shows a third directly and reads \
             two thirds, its own plus the third behind the page. Measured \
             {h:.4} (section {:.0}px wide against a {COL_W:.0}px page); \
             sections {sections:?}",
            2.0 * sections[i].0
        );
    }
    let (a, b) = (hidden[0], hidden[1]);
    assert!(
        (a - b).abs() / a.max(b) <= 0.05,
        "the two margins must be cropping ONE section, not two: the left reads \
         the page as hiding {a:.4} of it and the right {b:.4} — a separately \
         cropped circle per margin is exactly what that difference means \
         (sections {sections:?})"
    );
}

// ---------------------------------------------------------------------------
// 2. THE COHERENCE — one camera, recovered independently from each margin.
// ---------------------------------------------------------------------------

/// How badly a candidate camera explains one margin's ink: the marks of a
/// correct camera sit ON its integer level sets, so the ink-weighted distance to
/// the nearest one is near zero, and a wrong camera scatters them toward the
/// 0.25 of a uniform phase. Pure pixels in, one number out.
fn phase_score(f: &[i32], m: (u32, u32), cam: &Camera, st: Steer) -> f64 {
    // The MAJOR rungs only, weighted by their own strength. A bend's fold leaves
    // a broad low-contrast wash where the far wall passes behind the near one,
    // and that wash sits on no level set at all — reading it as evidence is what
    // makes a fit shrug at exactly the pose it most needs to resolve.
    let floor = major_floor(f, m);
    let (mut num, mut den) = (0.0f64, 0.0f64);
    let inset = 80u32;
    let (x0, x1) = if m.0 == 0 {
        (6, m.1 - inset)
    } else {
        (m.0 + inset, m.1 - 6)
    };
    let mut y = 10;
    while y < H - 10 {
        let mut x = x0;
        while x < x1 {
            let v = at(f, x, y);
            if v >= floor {
                let (fx, fy) = (x as f32, y as f32);
                let (r0, l0) = cam.lattice(fx, fy, st);
                let (rx, lx) = cam.lattice(fx + 1.0, fy, st);
                let (ry, ly) = cam.lattice(fx, fy + 1.0, st);
                // The distance to the nearest predicted line, IN PIXELS. Lattice
                // units would hand a free win to any candidate that predicts a
                // dense lattice here — and a wrong steering does exactly that,
                // because it swings the tunnel's own far end into this margin.
                let d = |c: f32, gx: f32, gy: f32| {
                    let g = (gx * gx + gy * gy).sqrt().max(1e-4);
                    (((c + 0.5).rem_euclid(1.0) - 0.5).abs() / g) as f64
                };
                let dr = d(r0, rx - r0, ry - r0);
                let dl = d(l0, lx - l0, ly - l0);
                num += dr.min(dl).min(MISS_CEILING_PX) * v as f64;
                den += v as f64;
            }
            x += 4;
        }
        y += 4;
    }
    if den == 0.0 {
        MISS_CEILING_PX
    } else {
        num / den
    }
}

/// The one shared camera must explain a margin to within this many pixels: a
/// mark of the route's own field lands within about a line's own width of where
/// this model puts it. Measured, not assumed — the worst margin of the worst
/// route pose reads 0.94px.
const ONE_CAMERA_PX: f64 = 1.6;
/// ...and any materially different camera must miss by at least this multiple,
/// so "it fits" is a statement about uniqueness rather than about slack.
const WRONG_CAMERA_RATIO: f64 = 3.0;
/// How far a candidate must differ from the route's own steering before it is
/// held to that ratio. The basin is narrow by construction — the anchor ring
/// moves about a hundred logical pixels per unit of yaw — so this is already
/// several line widths away.
const MATERIALLY_DIFFERENT: f32 = 0.15;

/// Every camera this law holds up against the route's own, for one pose.
fn rivals(yaw: f32, pitch: f32) -> Vec<(f32, f32)> {
    let mut v = vec![(0.0, 0.0), (-yaw, -pitch)];
    for d in [-0.6f32, -0.3, -0.15, 0.15, 0.3, 0.6] {
        v.push((yaw + d, pitch));
        v.push((yaw, pitch + d));
    }
    v.retain(|&(y, p)| {
        (y - yaw).abs().max((p - pitch).abs()) >= MATERIALLY_DIFFERENT
            && (-1.4..=1.4).contains(&y)
            && (-1.2..=1.2).contains(&p)
    });
    v
}

/// THE TURNING-COHERENCE LAW. At EVERY pose the route reaches, the left margin
/// and the right margin — read independently, each on its own ink — must both
/// be explained by the SAME camera, and by no other. Horizon, curvature and
/// vanishing direction are each a function of that one pose, so this is one
/// statement of all three of the disagreements the live review reported: two
/// margins steering independently cannot both sit on one camera's level sets.
///
/// Proven capable of failing: `warp_per_margin_steering_breaks_the_shared_camera`
/// restores the defect through `Tunnel::PerMargin` and watches this exact
/// comparison inverted.
#[test]
fn both_margins_are_one_camera_at_every_route_pose() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let (spacing, curv) = kite_dials();
    let cam = Camera::new(W, H, COL_LEFT, COL_W, spacing);
    let mut report = Vec::new();
    for name in ["straight", "left", "climb", "right", "descent", "wrap"] {
        let phase = warpgrid::named_pose(name).expect("a named route pose");
        let pose = warpgrid::route_pose(phase);
        let f = field(&device, &queue, kite(), W, H, COL_LEFT, COL_W, phase);
        let st = |yaw: f32, pitch: f32| Steer {
            yaw,
            pitch,
            curvature: curv,
            forward: pose.forward_cells,
        };
        for (i, m) in margins().into_iter().enumerate() {
            let side = if i == 0 { "left" } else { "right" };
            let mine = phase_score(&f, m, &cam, st(pose.yaw, pose.pitch));
            report.push((name, side, mine));
            assert!(
                mine < ONE_CAMERA_PX,
                "{name}/{side}: the route's own camera (yaw {:+.2} pitch {:+.2}) \
                 does not explain this margin — its marks miss the one shared \
                 projection by {mine:.2}px on average. The margin is composing \
                 something of its own.\nscores so far: {report:?}",
                pose.yaw,
                pose.pitch
            );
            for (ry, rp) in rivals(pose.yaw, pose.pitch) {
                let theirs = phase_score(&f, m, &cam, st(ry, rp));
                assert!(
                    theirs >= mine * WRONG_CAMERA_RATIO,
                    "{name}/{side}: a DIFFERENT camera (yaw {ry:+.2} pitch {rp:+.2}) \
                     explains this margin nearly as well as the route's own \
                     ({theirs:.2}px against {mine:.2}px) — the margin is not \
                     pinned to the shared pose at all"
                );
            }
        }
        // The two margins are pinned to the SAME camera to the same standard —
        // stated separately so the failure names the comparison the review made.
        let (l, r) = (report[report.len() - 2].2, report[report.len() - 1].2);
        assert!(
            (l - r).abs() < ONE_CAMERA_PX * 0.5,
            "{name}: the two margins are not equally well explained by one camera \
             (left {l:.2}px, right {r:.2}px) — one of them is steering on its own"
        );
    }
}

/// THE MUTATION PROOF. `Tunnel::PerMargin` is the defect kept as data: each
/// margin re-derives the steering from its own side of the page. The law above
/// must go red on it, and this test is what says so out loud — it asserts the
/// left margin is explained by the FLIPPED camera and not by the route's, so it
/// fails the day the mutation stops being one.
#[test]
fn warp_per_margin_steering_breaks_the_shared_camera() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let (spacing, curv) = kite_dials();
    let cam = Camera::new(W, H, COL_LEFT, COL_W, spacing);
    let bad = with_tunnel(kite(), theme::Tunnel::PerMargin);
    for name in ["left", "right"] {
        let phase = warpgrid::named_pose(name).expect("a named route pose");
        let pose = warpgrid::route_pose(phase);
        let f = field(&device, &queue, bad, W, H, COL_LEFT, COL_W, phase);
        let st = |yaw: f32, pitch: f32| Steer {
            yaw,
            pitch,
            curvature: curv,
            forward: pose.forward_cells,
        };
        let m = margins()[0];
        let shared = phase_score(&f, m, &cam, st(pose.yaw, pose.pitch));
        let flipped = phase_score(&f, m, &cam, st(-pose.yaw, -pose.pitch));
        assert!(
            shared > flipped * WRONG_CAMERA_RATIO && shared > ONE_CAMERA_PX,
            "{name}: the per-margin mutation must be visible to the coherence law \
             — the left margin reads {shared:.2}px against the shared camera and \
             {flipped:.2}px against its own flipped one, which is not the \
             inversion the law exists to catch"
        );
    }
}

/// The host model above exists for these laws alone, so nothing but a grep-law
/// keeps it honest. Every constant it carries is asserted to be the literal the
/// shader declares.
#[test]
fn the_host_model_matches_the_shaders_own_constants() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for (name, value) in [
        ("WARP_SECTION_PAGE_RATIO", SECTION_PAGE_RATIO),
        ("WARP_ASPECT_FIT", ASPECT_FIT),
        ("WARP_ASPECT_MIN", ASPECT_MIN),
        ("WARP_ASPECT_MAX", ASPECT_MAX),
        ("WARP_RAILS_PER_HALF_TURN", RAILS_PER_HALF_TURN),
        ("WARP_RPO_MIN", RPO_MIN),
        ("WARP_RPO_MAX", RPO_MAX),
        ("WARP_BEND_GAIN", BEND_GAIN),
        ("WARP_CORE_FRAC", CORE_FRAC),
    ] {
        let want = format!("const {name}: f32 = {value:?};");
        assert!(
            wgsl.contains(&want),
            "shaders/background.wgsl must declare `{want}` — the host model this \
             module fits with has drifted from the field it is fitting"
        );
    }
    let want = format!("const WARP_SOLVE_STEPS: i32 = {SOLVE_STEPS};");
    assert!(
        wgsl.contains(&want),
        "background.wgsl must declare `{want}`"
    );
}
