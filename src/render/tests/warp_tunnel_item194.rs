//! ITEM 194 — ONE CYLINDER, ONE CONSTANT SCALE, PLACED BY THE ROOM AND SEEN
//! THROUGH TWO WINDOWS.
//!
//! Item 132's own laws (`backgrounds_item132`) prove the ground exists, stays
//! out of the page, keeps its hierarchy and never aliases. The laws here prove
//! something else again: that the two margins are two crops of ONE projection
//! rather than two independently composed fields, and that the page column can
//! only MASK that projection.
//!
//! **THE MODEL HAS BEEN SUPERSEDED TWICE, SO THE LAWS WERE RE-AIMED RATHER THAN
//! RE-TUNED — TWICE.**
//!
//! * Round 1's contract was "one cylinder continuing behind the page", with the
//!   section SIZED from the page column (`anchor = 3 * page_half`). The live
//!   review approved it at the narrowest page and failed it everywhere else,
//!   because an authored scale that is a function of the margin is not a
//!   projection. Kept as `theme::Tunnel::PageScaled`.
//! * Round 2 made the scale the ROOM's and moved the page dependence into the
//!   WINDOW PLACEMENT, sliding each margin's axis by that margin's own width.
//!   **Its width law stayed green and the live review failed it again**, because
//!   aspect ratio and radius-on-the-ladder — everything that law measured — are
//!   exactly the quantities a translation preserves, and the remaining defect
//!   was a translation. Kept as `theme::Tunnel::MarginPlaced`.
//! * Round 3's contract is "one scene, framed by the room; the page is a mask".
//!   The placement owner takes no page and no margin argument at all.
//!
//! Two further things round 2's sweep could not see, and this one is built to:
//! it stopped at measure 96 while `page::MAX_MEASURE` is **140**, so the two
//! widths the review named were never rendered; and it swept one room with a
//! bare view, so awl's column was always centred. A real window reserves a
//! gutter for the margin outline, and at the wide end the margins measure 405px
//! and 139px — which is where a margin-derived placement puts the two windows in
//! different regimes at the same instant, i.e. back into round 1's rejected
//! "two separately cropped circles".
//!
//! FOUR CLAIMS, ALL ARITHMETIC OVER REAL GPU PIXELS — the differential `field`
//! oracle item 86 authored, so the flat ground cancels and what is measured is
//! the lattice alone.
//!
//! 1. **The scale.** Across the whole adaptive-column range, in two rooms, every
//!    major cross-ring arc a margin offers is TRACED and solved — with a FREE
//!    centre — for the ellipse it belongs to. Its ASPECT RATIO must be the
//!    constant 1.00 and its radius must sit on the room's own fixed ring ladder.
//!    Recovering the opening from an ARC rather than a whole ring is the point:
//!    no margin holds the section's apex, so the eye infers the opening from the
//!    flank, and so does this law.
//! 2. **The windows, and the headline claim of round 3.** The same fitted centres
//!    say where each window sits, and that x must be a constant of the ROOM: the
//!    same number at every page width the "Narrow page"/"Widen page" commands
//!    reach, the same distance from its own room edge on both sides, including
//!    at the off-centre columns a real window produces.
//! 3. **The coherence, at every route pose.** Each margin's ink is fitted, ON ITS
//!    OWN, for the camera that would explain it. The two margins must recover the
//!    SAME steering — one statement of all three things round 1's review found
//!    disagreeing, since horizon, curvature and vanishing direction are each a
//!    function of that one pose.
//! 4. **The direction.** Forward travel must GROW the projected rings, at the
//!    rate the projection fixes. One ring is tracked across a ladder of route
//!    phases inside a single straight leg.
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

const SECTION_ROOM_FRAC: f32 = 0.432;
const WINDOW_INSET: f32 = 0.4;
const WINDOW_FULL: f32 = 1.0;
const WINDOW_TIGHT: f32 = 0.35;
const WINDOW_STRADDLE: f32 = 0.4;
const RING_PITCH_AT: f32 = 0.3333333;
const RAILS_PER_HALF_TURN: f32 = 10.0;
const RPO_MIN: f32 = 3.0;
const RPO_MAX: f32 = 14.0;
const MAJOR_EVERY: f32 = 5.0;
const BEND_GAIN: f32 = 0.075;
const CORE_FRAC: f32 = 0.055;
const SOLVE_STEPS: usize = 20;
/// A candidate that misses by more than this many pixels has missed; capping it
/// keeps one wildly wrong prediction from outvoting a whole margin of evidence.
const MISS_CEILING_PX: f64 = 8.0;

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The projection every margin of one frame shares. After round 2 it is derived
/// from the ROOM alone — no page column reaches it — which is the whole repair.
#[derive(Clone, Copy)]
struct Camera {
    axis_y: f32,
    anchor: f32,
    rpo: f32,
}

impl Camera {
    fn new(h: u32, spacing_px: f32) -> Self {
        let anchor = SECTION_ROOM_FRAC * (h as f32).max(1.0);
        Camera {
            axis_y: h as f32 * 0.5,
            anchor,
            rpo: (RING_PITCH_AT * anchor * std::f32::consts::LN_2 / spacing_px)
                .clamp(RPO_MIN, RPO_MAX),
        }
    }

    /// ROUND 2's `warp_window_hide`, mirrored, and alive here only to predict
    /// the `MarginPlaced` mutation arm: how far INSIDE the page edge one
    /// margin's copy of the axis fell, as a function of that margin's OWN width.
    fn hide(&self, span: f32, page_half: f32) -> f32 {
        let full = smoothstep(WINDOW_TIGHT, WINDOW_FULL, span / self.anchor.max(1.0));
        let tight = -WINDOW_STRADDLE * span;
        tight + (page_half - tight) * full
    }

    /// Where that margin's window puts the axis on the glass — `warp_window_axis`
    /// mirrored. Note what it does NOT take: no column left, no column width, no
    /// margin span. The placement is the ROOM's, which is the whole of round 3,
    /// and this signature is the cheapest possible statement of it.
    fn axis_x(&self, w: u32, on_right: bool) -> f32 {
        let inset = WINDOW_INSET * self.anchor;
        if on_right { w as f32 - inset } else { inset }
    }

    /// Round 2's placement, for the `MarginPlaced` arm alone.
    fn margin_placed_axis_x(&self, w: u32, col_left: f32, col_w: f32, on_right: bool) -> f32 {
        let col_right = col_left + col_w;
        let span = if on_right {
            w as f32 - col_right
        } else {
            col_left
        }
        .max(1.0);
        let hide = self.hide(span, (col_w * 0.5).max(1.0));
        if on_right {
            col_right - hide
        } else {
            col_left + hide
        }
    }

    /// The `(ring, rail)` lattice coordinates of one pixel under a candidate
    /// steering — the shader's own arithmetic, bisection and all.
    fn lattice(&self, axis_x: f32, px: f32, py: f32, p: Steer) -> (f32, f32) {
        let (yaw, pitch, curvature, forward) = (p.yaw, p.pitch, p.curvature, p.forward);
        let q = (px - axis_x, py - self.axis_y);
        let g = BEND_GAIN * curvature * self.anchor * self.anchor;
        let bend = (g * yaw, g * pitch);
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
            self.rpo * (self.anchor / u).log2() + forward,
            w.1.atan2(w.0) * (RAILS_PER_HALF_TURN / std::f32::consts::PI),
        )
    }
}

/// One candidate camera: the steering being tested, the world's own bend gain,
/// and the FORWARD travel the route has already made — which is not optional.
/// Forward travel is a pure ADDITION to the ring coordinate, so leaving it out
/// shifts every predicted cross-ring by its fractional part and the fit then
/// reads a perfectly coherent field as explained by nothing.
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

/// One rendered differential field, and the canvas it was measured on — so
/// every measurement below works at any swept geometry rather than only at the
/// canonical one.
struct Frame {
    f: Vec<i32>,
    w: u32,
    h: u32,
}

impl Frame {
    fn at(&self, x: u32, y: u32) -> i32 {
        self.f[(y * self.w + x) as usize]
    }

    /// The strength above which a mark is one of the MAJOR (every fifth) lines,
    /// derived from the frame's OWN ink distribution rather than from a colour
    /// constant: the majors are the top of the observed range by construction,
    /// and half of the near-maximum separates the two rungs cleanly (item 132's
    /// own hierarchy law proves the rungs are genuinely distinct).
    fn major_floor(&self, m: (u32, u32)) -> i32 {
        let mut v: Vec<i32> = (0..self.h)
            .flat_map(|y| (m.0..m.1).map(move |x| (y, x)))
            .map(|(y, x)| self.at(x, y))
            .filter(|&i| i > INK_FLOOR)
            .collect();
        if v.is_empty() {
            return i32::MAX;
        }
        v.sort_unstable();
        let hi = v[v.len() * 995 / 1000];
        (hi / 2).max(INK_FLOOR * 3)
    }

    /// THE HORIZON, measured and not assumed: the row about which the margin's
    /// own ink profile is most nearly its own reflection. A tunnel's cross-rings
    /// are symmetric about their centre row, so this is that row.
    fn horizon(&self, m: (u32, u32)) -> u32 {
        let prof: Vec<f64> = (0..self.h)
            .map(|y| (m.0..m.1).map(|x| self.at(x, y) as f64).sum::<f64>())
            .collect();
        let (mut best, mut best_c) = (-1.0f64, self.h / 2);
        for c in (self.h * 3 / 10)..(self.h * 7 / 10) {
            let k = (c.min(self.h - 1 - c)).min(self.h * 28 / 100) as usize;
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

    /// Walk one major cross-ring away from a seed point, one column at a time,
    /// with a one-step linear prediction so the walk cannot hop onto a
    /// neighbour. `dx` is which way to leave the seed.
    fn walk(&self, m: (u32, u32), maj: i32, seed: (i32, i32), dx: i32) -> Vec<(f32, f32)> {
        let mut pts = vec![(seed.0 as f32, seed.1 as f32)];
        let (mut x, mut y, mut prev) = (seed.0, seed.1, None::<i32>);
        loop {
            x += dx;
            if x < m.0 as i32 + 2 || x >= m.1 as i32 - 2 {
                break;
            }
            let pred = prev.map_or(y, |p| 2 * y - p);
            let mut best = None;
            for yy in (pred - 7).max(1)..(pred + 8).min(self.h as i32 - 2) {
                let (xu, yu) = (x as u32, yy as u32);
                if self.at(xu, yu) >= maj
                    && self.at(xu, yu) >= self.at(xu, yu - 1)
                    && self.at(xu, yu) >= self.at(xu, yu + 1)
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
        pts
    }

    /// Every major cross-ring arc this margin can offer, as candidate point
    /// sets. Seeded at SEVERAL columns across the margin rather than one: which
    /// rungs of the ladder a window holds — and where each is near its apex,
    /// where a walk immediately runs out of arc — is exactly what the page width
    /// changes, so a single seed column measures some widths and not others.
    /// Each candidate is one ring's arc above and below the horizon, paired by
    /// its own symmetry about that row.
    fn ring_arcs(&self, m: (u32, u32), cy: u32, left: bool) -> Vec<Vec<(f32, f32)>> {
        let maj = self.major_floor(m);
        if maj == i32::MAX {
            return Vec::new();
        }
        let span = (m.1 - m.0) as f32;
        let reach = (self.h * 44 / 100) as i32;
        let inward = if left { -1 } else { 1 };
        let mut out = Vec::new();
        for frac in [0.15f32, 0.35, 0.55, 0.75, 0.9] {
            let off = (span * frac) as u32;
            let seed = if left { m.1 - 1 - off } else { m.0 + off };
            if seed <= m.0 + 2 || seed >= m.1 - 2 {
                continue;
            }
            // Every ring crossing on this column, above the horizon.
            for dy in 24..reach {
                let y = cy as i32 - dy;
                if y < 1 {
                    break;
                }
                let (yu, xu) = (y as u32, seed);
                if !(self.at(xu, yu) >= maj
                    && self.at(xu, yu) >= self.at(xu, yu - 1)
                    && self.at(xu, yu) >= self.at(xu, yu + 1))
                {
                    continue;
                }
                // Its mirror below the horizon — the same ring, so the arc is
                // traced as one object rather than two half-fits.
                let mirror = cy as i32 + dy;
                let mut pts = self.walk(m, maj, (seed as i32, y), inward);
                pts.extend(self.walk(m, maj, (seed as i32, y), -inward));
                if mirror < self.h as i32 - 2 {
                    let my = ((mirror - 2).max(1)..(mirror + 3).min(self.h as i32 - 2))
                        .filter(|&yy| self.at(xu, yy as u32) >= maj)
                        .max_by_key(|&yy| self.at(xu, yy as u32));
                    if let Some(my) = my {
                        pts.extend(self.walk(m, maj, (seed as i32, my), inward));
                        pts.extend(self.walk(m, maj, (seed as i32, my), -inward));
                    }
                }
                out.push(pts);
            }
        }
        out
    }
}

/// A 4x4 solve by Gaussian elimination with partial pivoting.
fn solve4(mut m: [[f64; 4]; 4], mut rhs: [f64; 4]) -> Option<[f64; 4]> {
    for col in 0..4 {
        let piv = (col..4).max_by(|&a, &b| m[a][col].abs().total_cmp(&m[b][col].abs()))?;
        if m[piv][col].abs() < 1e-9 {
            return None;
        }
        m.swap(col, piv);
        rhs.swap(col, piv);
        for r in 0..4 {
            if r == col {
                continue;
            }
            let k = m[r][col] / m[col][col];
            let pivot_row = m[col];
            for (c, cell) in m[r].iter_mut().enumerate().skip(col) {
                *cell -= k * pivot_row[c];
            }
            rhs[r] -= k * rhs[col];
        }
    }
    Some([
        rhs[0] / m[0][0],
        rhs[1] / m[1][1],
        rhs[2] / m[2][2],
        rhs[3] / m[3][3],
    ])
}

/// What one traced arc says about the section it is an arc of.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fit {
    /// The section's centre — the WINDOW's own axis, read off the pixels.
    cx: f32,
    /// Its flattening: 1.00 is the circle round 2's projection draws.
    aspect: f32,
    /// Its projected radius.
    u: f32,
    /// Its centre row — which must be the horizon, and is checked against it.
    cy: f32,
    /// RMS radial residual, in pixels — how much of an ellipse this arc is.
    rms: f32,
    /// How many points survived.
    n: usize,
    /// How much of the section's own circumference this arc covers, in degrees,
    /// about its own fitted centre. It is the CONDITIONING of the fit: a short
    /// cap trades curvature against radius and can be a plausible ellipse of the
    /// wrong size, and no residual notices.
    arc_deg: f32,
    /// How many OTHER candidate arcs of the same margin agree on this centre.
    /// Every ring in a window is concentric, so corroboration is the evidence
    /// that a walk followed a ring rather than a rail it crossed.
    votes: usize,
}

/// The section one traced arc belongs to, solved with a FREE CENTRE:
/// `(x - cx)^2 + aspect^2 (y - cy)^2 = u^2`, expanded to the linear form
/// `x^2 + A y^2 + D x + E y + F = 0` and least-squared in `(A, D, E, F)`.
///
/// Nothing about the answer is assumed — not where the axis is, not how flat the
/// section is, not how big. That matters twice over: the placement claim reads
/// `cx` straight off the pixels, and the scale claim cannot be handed the scale
/// it is checking.
fn section_of(pts: &[(f32, f32)]) -> Option<(f32, f32, f32, f32)> {
    if pts.len() < 60 {
        return None;
    }
    // Condition the fit on the arc's own centroid, then translate back.
    let n = pts.len() as f64;
    let (mx, my) = pts.iter().fold((0.0f64, 0.0f64), |a, &(x, y)| {
        (a.0 + x as f64 / n, a.1 + y as f64 / n)
    });
    let mut m = [[0.0f64; 4]; 4];
    let mut rhs = [0.0f64; 4];
    for &(px, py) in pts {
        let (x, y) = (px as f64 - mx, py as f64 - my);
        let basis = [y * y, x, y, 1.0];
        let target = -x * x;
        for i in 0..4 {
            for j in 0..4 {
                m[i][j] += basis[i] * basis[j];
            }
            rhs[i] += basis[i] * target;
        }
    }
    let sol = solve4(m, rhs)?;
    let (a, d, e, f) = (sol[0], sol[1], sol[2], sol[3]);
    if a <= 0.0 {
        return None;
    }
    let cx = -d * 0.5;
    let cy = -e / (2.0 * a);
    let u2 = cx * cx + a * cy * cy - f;
    if u2 <= 0.0 {
        return None;
    }
    Some((
        (cx + mx) as f32,
        (cy + my) as f32,
        a.sqrt() as f32,
        u2.sqrt() as f32,
    ))
}

/// An arc's own worst enemy is a RAIL: the radial lines are major every fifth
/// too, and a walk that meets one where it crosses a ring can follow it away
/// and hand back an "ellipse" that is half ring and half straight line. So the
/// fit is refitted twice with its own outliers dropped, and the survivor has to
/// be an ellipse to within [`FIT_RMS_PX`] — a real ring arc fits far tighter
/// than that, and a mixed one cannot.
const FIT_RMS_PX: f32 = 1.0;
/// The extent an arc must cover, AS A FRACTION OF THE SECTION IT CLAIMS, before
/// it determines that section at all. Stated relative to the fitted radius
/// rather than in pixels because that is what conditioning depends on: a short
/// or nearly straight fragment fits an ellipse of almost any size, and the
/// arcs that do it here are the far rungs, whose flank crosses a margin almost
/// vertically. `1.2` reaches at least 37 degrees either side of the horizon.
const FIT_MIN_HEIGHT_RADII: f32 = 1.2;
/// A section is only measured when the margin's OTHER arcs corroborate its
/// centre, with this much evidence behind the winner.
const FIT_MIN_VOTES: usize = 3;
const FIT_MIN_POINTS: usize = 150;
/// ...and when the winning arc covers enough of its own circumference to
/// DETERMINE it. This is the one filter that earned itself by measurement: swept
/// over the band, arcs covering 100 degrees or more fit the section to 0.99-1.02
/// of aspect and 0.04 of a ladder rung, while the 75-95 degree caps a mid-band
/// window leaves — all apex, no flank — fit smooth, low-residual ellipses of the
/// wrong size (0.87 and 335px against a true 1.00 and 432px). Residual cannot
/// see it; coverage can. The widths that fall out are recorded as a measurement
/// gap in THEMES.md rather than quietly dropped.
const FIT_MIN_ARC_DEG: f32 = 100.0;
const FIT_MIN_WIDTH_RADII: f32 = 0.2;

/// GEOMETRIC refinement of an algebraic ellipse fit — Gauss-Newton on the
/// RADIAL residual `sqrt((x-cx)^2 + a^2 (y-cy)^2) - u`.
///
/// It earns its place: the algebraic solve minimises the wrong quantity, and on
/// a short arc — which is every arc a NARROW margin can offer — that bias is
/// large enough to be mistaken for the defect this module measures. Fitting the
/// same 144x644px arc both ways at a mid-band page width read aspect 0.82 / 306px
/// algebraically and 0.99 / 431px geometrically, against a true 1.00 / 432px.
fn polish(pts: &[(f32, f32)], seed: (f64, f64, f64, f64)) -> Option<(f64, f64, f64, f64)> {
    let (mut cx, mut cy, mut a, mut u) = seed;
    for _ in 0..24 {
        let mut m = [[0.0f64; 4]; 4];
        let mut rhs = [0.0f64; 4];
        for &(px, py) in pts {
            let (dx, dy) = (px as f64 - cx, py as f64 - cy);
            let r = (dx * dx + a * a * dy * dy).sqrt().max(1e-6);
            let j = [-dx / r, -a * a * dy / r, a * dy * dy / r, -1.0];
            let res = r - u;
            for i in 0..4 {
                for k in 0..4 {
                    m[i][k] += j[i] * j[k];
                }
                rhs[i] -= j[i] * res;
            }
        }
        // A whisper of damping keeps the step honest where the arc is short.
        for (i, row) in m.iter_mut().enumerate() {
            row[i] *= 1.0 + 1e-6;
        }
        let step = solve4(m, rhs)?;
        cx += step[0];
        cy += step[1];
        a += step[2];
        u += step[3];
        if !(cx.is_finite() && cy.is_finite() && a.is_finite() && u.is_finite()) || a <= 0.0 {
            return None;
        }
        if step.iter().all(|s| s.abs() < 1e-4) {
            break;
        }
    }
    Some((cx, cy, a, u))
}

/// The fitted centre must sit on the horizon the margin's own ink already
/// measured. A walk that pairs one ring's upper half with a NEIGHBOUR's lower
/// half fits a plausible-looking ellipse of the wrong size, and this is what
/// says so: a section's centre is on the horizon, always.
const FIT_HORIZON_TOL_PX: f32 = 12.0;

fn fit_arc(pts: &[(f32, f32)], cy0: f32) -> Option<Fit> {
    let mut kept: Vec<(f32, f32)> = pts.to_vec();
    kept.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    kept.dedup();
    let mut fit = None;
    for round in 0..3 {
        let seed = section_of(&kept)?;
        let (cx, cy, aspect, u) = polish(
            &kept,
            (seed.0 as f64, seed.1 as f64, seed.2 as f64, seed.3 as f64),
        )
        .map(|(a, b, c, d)| (a as f32, b as f32, c as f32, d as f32))?;
        let resid = |&(x, y): &(f32, f32)| {
            (((x - cx).powi(2) + (aspect * (y - cy)).powi(2)).sqrt() - u).abs()
        };
        let rms = (kept.iter().map(|p| resid(p).powi(2)).sum::<f32>() / kept.len() as f32).sqrt();
        let mut ang: Vec<f32> = kept
            .iter()
            .map(|&(x, y)| (aspect * (y - cy)).atan2(x - cx))
            .collect();
        ang.sort_by(f32::total_cmp);
        let tau = std::f32::consts::TAU;
        let gap = ang
            .windows(2)
            .map(|w| w[1] - w[0])
            .fold(ang[0] + tau - ang[ang.len() - 1], f32::max);
        fit = Some(Fit {
            cx,
            cy,
            aspect,
            u,
            arc_deg: (tau - gap).to_degrees(),
            rms,
            n: kept.len(),
            votes: 0,
        });
        if round == 2 {
            break;
        }
        let cut = (rms * 2.5).max(0.75);
        let next: Vec<(f32, f32)> = kept.iter().copied().filter(|p| resid(p) <= cut).collect();
        if next.len() < 60 || next.len() == kept.len() {
            break;
        }
        kept = next;
    }
    let f = fit?;
    let (x0, x1) = kept
        .iter()
        .fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
    let (y0, y1) = kept
        .iter()
        .fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
    (f.n >= 120
        && f.arc_deg >= FIT_MIN_ARC_DEG
        && (f.cy - cy0).abs() <= FIT_HORIZON_TOL_PX
        && f.rms <= FIT_RMS_PX
        && y1 - y0 >= FIT_MIN_HEIGHT_RADII * f.u
        && x1 - x0 >= FIT_MIN_WIDTH_RADII * f.u)
        .then_some(f)
}

/// The best-determined section this margin offers.
///
/// Every candidate arc is fitted on its own, and the winner is the one the OTHER
/// arcs CORROBORATE: every ring in a margin is concentric, so the axis is the
/// one point they all agree on, and a walk that was led astray — a rail crossing
/// a ring is a fork with no local evidence about which way is the ring — agrees
/// with nothing. Ties go to the arc with the most points behind it.
fn best_section(fr: &Frame, m: (u32, u32), left: bool) -> Option<Fit> {
    sections(fr, m, left).into_iter().max_by_key(|f| (f.votes, f.n))
}

/// Every section this margin's arcs determine, corroborated and filtered but not
/// yet reduced to one. [`best_section`] takes the best-corroborated; the
/// direction ladder instead TRACKS one ring across phases, which needs the
/// alternatives — its winner legitimately changes as the lattice travels, and a
/// sequence that silently swapped rings would read as a jump rather than motion.
fn sections(fr: &Frame, m: (u32, u32), left: bool) -> Vec<Fit> {
    let cy = fr.horizon(m);
    let fits: Vec<Fit> = fr
        .ring_arcs(m, cy, left)
        .iter()
        .filter_map(|a| fit_arc(a, cy as f32))
        .collect();
    fits.iter()
        .map(|f| Fit {
            votes: fits.iter().filter(|g| (g.cx - f.cx).abs() <= 12.0).count(),
            ..*f
        })
        .filter(|f| f.votes >= FIT_MIN_VOTES && f.n >= FIT_MIN_POINTS)
        .collect()
}

/// One graded cell of the page-width sweep: the geometry, and what each margin's
/// own traced arc says about the section it is an arc of.
#[derive(Debug, Clone, Copy)]
struct Cell {
    /// The ROOM this cell was rendered in. Round 2 swept one room and one axis;
    /// the defect needs both, because how far off-centre awl's own column sits
    /// at a given measure is a function of the room, and an OFF-CENTRE column is
    /// what put the two margins in different regimes at the same instant.
    room: (u32, u32),
    measure: usize,
    /// True for the hand-placed OFF-CENTRE columns appended to the sweep — the
    /// geometry awl's own outline gutter produces in a real window, which the
    /// bare test view never reaches. See [`OFF_CENTRE`].
    off_centre: bool,
    col_left: f32,
    col_w: f32,
    span: [f32; 2],
    /// What each margin's own ink says about the section it is showing an arc of.
    fit: [Option<Fit>; 2],
}

/// The page widths this round is verified across — the whole band awl's own
/// "Narrow page"/"Widen page" commands reach, stepped by the command's own
/// [`crate::page::MEASURE_STEP`] until the margins close. The PRIMARY AXIS of
/// item 194 round 2: the defect it repairs is invisible at any single width,
/// and round 1's brief was written from one.
/// ROUND 3 EXTENDED THIS TO THE WHOLE BAND, AND THAT IS HOW ROUND 2'S LAW WENT
/// GREEN OVER THE DEFECT. Round 2 swept `MIN_MEASURE..=96` while
/// [`crate::page::MAX_MEASURE`] is 140, so measures 100 through 140 were never
/// rendered — and the live review reported the failure at 105 and 140. A sweep
/// that stops before the command does is a single-width law wearing a sweep's
/// clothes.
fn measure_sweep() -> Vec<usize> {
    (crate::page::MIN_MEASURE..=crate::page::MAX_MEASURE)
        .step_by(crate::page::MEASURE_STEP)
        .collect()
}

/// A margin narrower than this cannot hold an arc worth solving — the field's
/// own narrow-margin simplification has already retired the minor lattice and
/// the page-edge quiet band eats most of what is left.
const TRACEABLE_MARGIN_PX: f32 = 120.0;

/// ...and a margin the page has covered the window's AXIS in cannot be solved
/// either, at any width. It holds only a short outer flank, and this module's
/// own [`FIT_MIN_ARC_DEG`] records what a short cap does: it fits a smooth,
/// low-residual ellipse of the wrong size that no residual notices. Measured
/// here rather than assumed — in the 1000px room a 137px margin, whose axis sits
/// at 173px and is therefore behind the page, returned aspect 0.902 and a radius
/// 0.32 rungs off the ladder from a 164-degree arc with 11 corroborating votes,
/// against 1.000 and 0.00 everywhere the axis was visible.
///
/// This is a MEASUREMENT GAP, not a claim about the product: those widths are
/// exactly where `WARP_WINDOW_INSET`'s own named cost lands, and what happens
/// there is a mask covering the opening. It is recorded in THEMES.md.
fn axis_is_visible(span: f32, anchor: f32) -> bool {
    span >= WINDOW_INSET * anchor
}

/// COLUMNS THAT ARE NOT CENTRED, which the test view cannot produce and a real
/// window does: awl reserves a gutter for the margin outline, so a document with
/// headings pushes the text column sideways. Measured off a real 2560x1440
/// capture of the world-gallery specimen at measure 140 — margins 405px and
/// 139px — and appended to the sweep as its own cells, because THIS is the
/// geometry that turned round 2's margin-derived placement into two different
/// compositions on one screen.
const OFF_CENTRE: [((u32, u32), usize, f32, f32); 2] = [
    ((2560, 1440), crate::page::MAX_MEASURE, 405.0, 2016.0),
    ((2560, 1440), 104, 430.0, 1497.6),
];

/// The ROOMS the width sweep is made in. The canonical test canvas, and a wide
/// Retina room — which is not decoration: awl's column is only centred while it
/// has room to be, and at the wide end of the measure band on the second room
/// the margins measure 405px and 139px. That asymmetry is what a placement read
/// from the margin's OWN width turns into two different compositions on one
/// screen, and one room can never show it.
const ROOMS: [(u32, u32); 2] = [(W, H), (2560, 1440)];

/// Render the straight pose at every page width in the sweep, in every room, at
/// the app's OWN adaptive column, and solve each margin's own arcs for the
/// section it shows.
fn sweep_cells(device: &wgpu::Device, queue: &wgpu::Queue, bg: theme::Background) -> Vec<Cell> {
    let mut out = Vec::new();
    let restore = crate::page::measure();
    for room in ROOMS {
        let (rw, rh) = room;
        for measure in measure_sweep() {
            let Some((_d, _q, mut pipe)) = super::headless_dqp(rw as f32, rh as f32) else {
                crate::page::set_measure(restore);
                return out;
            };
            crate::page::set_measure(measure);
            pipe.set_view(&super::view("hello", 0, 0));
            let (col_left, col_w) = (pipe.column_left(), pipe.column_width());
            let f = Frame {
                f: field(
                    device,
                    queue,
                    bg,
                    rw,
                    rh,
                    col_left,
                    col_w,
                    warpgrid::FROZEN_PHASE,
                ),
                w: rw,
                h: rh,
            };
            let ms = super::backgrounds_item89::margins(rw, col_left, col_w);
            let mut cell = Cell {
                room,
                measure,
                off_centre: false,
                col_left,
                col_w,
                span: [col_left, rw as f32 - (col_left + col_w)],
                fit: [None, None],
            };
            let anchor = SECTION_ROOM_FRAC * rh as f32;
            for (i, m) in ms.into_iter().enumerate() {
                if cell.span[i] < TRACEABLE_MARGIN_PX || !axis_is_visible(cell.span[i], anchor) {
                    continue;
                }
                cell.fit[i] = best_section(&f, m, i == 0);
            }
            out.push(cell);
        }
    }
    for (room, measure, col_left, col_w) in OFF_CENTRE {
        let (rw, rh) = room;
        let Some((_d, _q, _pipe)) = super::headless_dqp(rw as f32, rh as f32) else {
            break;
        };
        let f = Frame {
            f: field(
                device,
                queue,
                bg,
                rw,
                rh,
                col_left,
                col_w,
                warpgrid::FROZEN_PHASE,
            ),
            w: rw,
            h: rh,
        };
        let mut cell = Cell {
            room,
            measure,
            off_centre: true,
            col_left,
            col_w,
            span: [col_left, rw as f32 - (col_left + col_w)],
            fit: [None, None],
        };
        for (i, m) in super::backgrounds_item89::margins(rw, col_left, col_w)
            .into_iter()
            .enumerate()
        {
            if cell.span[i] < TRACEABLE_MARGIN_PX
                || !axis_is_visible(cell.span[i], SECTION_ROOM_FRAC * rh as f32)
            {
                continue;
            }
            cell.fit[i] = best_section(&f, m, i == 0);
        }
        out.push(cell);
    }
    crate::page::set_measure(restore);
    out
}

/// The geometries the coherence laws below are made at: the canonical gallery
/// column, and the NARROWEST page — the composition the live review approved,
/// and the only one whose window holds the tunnel's deep end, where a steering
/// error is largest. Two, because how much of the cylinder a window shows is
/// itself a function of the page width, and so therefore is how much evidence
/// about the camera the margin carries.
const GEOMETRIES: [(f32, f32); 2] = [(COL_LEFT, COL_W), (656.0, 288.0)];

/// `[x0, x1)` of the left and right margins at one geometry.
fn margins_at(col_left: f32, col_w: f32) -> [(u32, u32); 2] {
    super::backgrounds_item89::margins(W, col_left, col_w)
}

// ---------------------------------------------------------------------------
// 1. THE SCALE — one projection, and the page width never touches it.
// ---------------------------------------------------------------------------

/// PER-CELL bounds, and where they come from. Solving a section from an ARC is
/// the only measurement a margin narrower than the page allows, so these are set
/// from the fit's own measured spread over the whole band rather than from a
/// wish. The defect they exist to catch is 3.00 of aspect and 1.5 rungs of
/// scale, an order of magnitude outside them. The CENTRAL bounds below are what
/// pin the value.
const ASPECT_TOL: f32 = 0.08;
/// How far a fitted radius may sit off the room's own ring ladder, in RUNGS —
/// the majors are `MAJOR_EVERY / rpo` octaves apart, so a rung is a factor of
/// about 2.8.
const RUNG_TOL: f32 = 0.06;
/// ...and what the band's own MEDIAN must hold to, which is the real statement:
/// a measurement with per-cell scatter still has a sharp centre, and the centre
/// is the constant the projection is claimed to have.
const ASPECT_MEDIAN_TOL: f32 = 0.02;
const RUNG_MEDIAN_TOL: f32 = 0.02;

/// The camera the cell's own ROOM fixes. Nothing about a cell but its room may
/// reach this, which is the claim these laws are made of.
fn cam_for(room: (u32, u32), spacing: f32) -> Camera {
    Camera::new(room.1, spacing)
}

/// **The projected cross-section's shape and size are invariant across the full
/// adaptive column range**, in every room.
///
/// Swept over the whole measure band at the app's own column owner, each
/// margin's major cross-ring arcs are traced and solved for the ellipse they
/// belong to with a free centre. Two numbers come out and neither may move: the
/// ASPECT must be the constant 1.00 (the section is a circle — round 2 removed
/// the affine fit entirely), and the RADIUS must sit on the ladder the ROOM
/// fixes, `anchor * 2^(-k * MAJOR_EVERY / rpo)`, so no page width can rescale
/// the world it lands in.
///
/// THIS LAW WAS TRUE IN ROUND 2 AND THE WORLD STILL RESCALED, which is why it
/// is no longer the headline. Aspect and radius-on-the-ladder are exactly the
/// two quantities a TRANSLATION preserves, and round 2's remaining defect was a
/// translation: it slid each window's axis by the margin's own width. The claim
/// that catches that is
/// [`the_two_windows_are_placed_by_the_room_so_the_page_can_only_mask`] below.
/// Both are kept, because the scale can regress on its own.
///
/// Proven capable of failing: `warp_page_scaled_projection_breaks_the_one_scale`
/// restores round 1's geometry through `Tunnel::PageScaled`.
#[test]
fn the_projection_never_rescales_across_the_adaptive_column_range() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cells = sweep_cells(&device, &queue, kite());
    let mut graded = 0usize;
    let mut aspects: Vec<f32> = Vec::new();
    let mut rungs: Vec<f32> = Vec::new();
    let report: Vec<((u32, u32), usize, [f32; 2], [Option<Fit>; 2])> = cells
        .iter()
        .map(|c| (c.room, c.measure, c.span, c.fit))
        .collect();
    for c in &cells {
        let cam = cam_for(c.room, spacing);
        for i in 0..2 {
            let Some(f) = c.fit[i] else { continue };
            assert!(
                (f.aspect - 1.0).abs() <= ASPECT_TOL,
                "{:?} m{}/margin {i}: the projected cross-section must be a CIRCLE at \
                 every page width — measured aspect {:.3} against 1.000. A section whose \
                 shape depends on the page is the item-194 defect itself.\n\
                 sweep: {report:?}",
                c.room,
                c.measure,
                f.aspect
            );
            let rung = cam.rpo * (cam.anchor / f.u).log2() / MAJOR_EVERY;
            let rungs_off = rung - rung.round();
            assert!(
                rungs_off.abs() <= RUNG_TOL,
                "{:?} m{}/margin {i}: the traced ring's radius {:.1}px is not a rung of \
                 the room's own ring ladder (anchor {:.1}px, {:.2} rungs off an integer) \
                 — the projection has been rescaled by the page.\nsweep: {report:?}",
                c.room,
                c.measure,
                f.u,
                cam.anchor,
                rungs_off
            );
            aspects.push(f.aspect);
            rungs.push(rungs_off);
            graded += 1;
        }
    }
    // The sweep must reach BOTH ends of the band in a room wide enough to hold
    // them, not merely many widths in the middle.
    for (label, want) in [
        (
            "widest margin",
            cells.iter().any(|c| {
                c.fit[0].is_some() && c.span[0] >= cam_for(c.room, spacing).anchor
            }),
        ),
        (
            "narrowest margin that still shows its window's axis",
            cells.iter().any(|c| {
                let anchor = cam_for(c.room, spacing).anchor;
                c.fit[0].is_some() && c.span[0] <= 1.15 * WINDOW_INSET * anchor
            }),
        ),
        (
            "the widest page the command reaches",
            cells
                .iter()
                .any(|c| c.measure == crate::page::MAX_MEASURE && c.fit.iter().any(|f| f.is_some())),
        ),
    ] {
        assert!(
            want,
            "the page-width sweep must grade the {label} — that is the axis this \
             defect lives on.\nsweep: {report:?}"
        );
    }
    assert!(
        graded >= 20,
        "the page-width sweep must actually grade margins, got {graded}\nsweep: {report:?}"
    );
    let lo = aspects.iter().cloned().fold(f32::MAX, f32::min);
    let hi = aspects.iter().cloned().fold(f32::MIN, f32::max);
    assert!(
        hi - lo <= ASPECT_TOL * 1.25,
        "the aspect ratio must be INVARIANT across the column range, not merely \
         near 1 somewhere: measured {lo:.3}..{hi:.3} over {graded} margins.\n\
         sweep: {report:?}"
    );
    let median = |mut v: Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    let mid = median(aspects.clone());
    assert!(
        (mid - 1.0).abs() <= ASPECT_MEDIAN_TOL,
        "the band's MEDIAN aspect ratio is the constant this projection claims, and \
         it measured {mid:.3} against 1.000 over {graded} margins.\nsweep: {report:?}"
    );
    let mid_rung = median(rungs.clone());
    assert!(
        mid_rung.abs() <= RUNG_MEDIAN_TOL,
        "the band's MEDIAN radius must sit ON the room's ring ladder, and it measured \
         {mid_rung:+.3} rungs off it over {graded} margins.\nsweep: {report:?}"
    );
}

/// THE MUTATION PROOF for the law above. `Tunnel::PageScaled` is round 1's own
/// geometry kept as data — the section sized from the page column and flattened
/// to the flank the page edge cuts. The sweep must go red on it, and this test
/// is what says so out loud: it asserts the aspect ratio SPREADS and the radius
/// leaves the room's ladder, so it fails the day the mutation stops being one.
#[test]
fn warp_page_scaled_projection_breaks_the_one_scale() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let bad = with_tunnel(kite(), theme::Tunnel::PageScaled);
    let cells = sweep_cells(&device, &queue, bad);
    let mut aspects: Vec<(usize, f32)> = Vec::new();
    let mut off_ladder = 0usize;
    for c in &cells {
        let cam = cam_for(c.room, spacing);
        for i in 0..2 {
            let Some(f) = c.fit[i] else {
                continue;
            };
            aspects.push((c.measure, f.aspect));
            let rungs = cam.rpo * (cam.anchor / f.u).log2() / MAJOR_EVERY;
            if (rungs - rungs.round()).abs() > RUNG_TOL {
                off_ladder += 1;
            }
        }
    }
    assert!(
        aspects.len() >= 8,
        "the mutation sweep must grade margins, got {}\n{aspects:?}",
        aspects.len()
    );
    let lo = aspects.iter().map(|a| a.1).fold(f32::MAX, f32::min);
    let hi = aspects.iter().map(|a| a.1).fold(f32::MIN, f32::max);
    assert!(
        hi - lo > ASPECT_TOL * 1.25,
        "PageScaled must SPREAD the aspect ratio across the band — that is the \
         defect — and it measured {lo:.3}..{hi:.3}\n{aspects:?}"
    );
    assert!(
        off_ladder > 0,
        "PageScaled must take the section OFF the room's ring ladder somewhere in \
         the band — that is the rescale — and every graded margin sat on it\n{aspects:?}"
    );
}

// ---------------------------------------------------------------------------
// 2. THE WINDOWS — placed by the ROOM, so the page can only mask.
// ---------------------------------------------------------------------------

/// How far a measured axis may sit from where the one placement owner puts it.
/// A traced arc recovers its centre to a pixel or two; this is the slack that
/// leaves, not the claim.
const PLACEMENT_TOL_PX: f32 = 12.0;

/// **THE HEADLINE LAW OF ROUND 3, and the defect stated as arithmetic: each
/// window's axis is a constant of the ROOM, so a page-width change can only
/// reveal or cover this one fixed scene.**
///
/// Round 1 sized the cylinder from the page column. Round 2 fixed that and moved
/// the page dependence into the WINDOW PLACEMENT, sliding each margin's axis by
/// that margin's own width — and its width law stayed green, because aspect and
/// radius-on-the-ladder are precisely what a translation preserves. This is the
/// claim that does not survive a translation: the fitted centre of each margin's
/// own arc, in ROOM coordinates, must be the same number at every measure the
/// "Narrow page"/"Widen page" commands reach, in every room.
///
/// Two things follow from it that the review asked for by name and that round 2
/// could not deliver:
///
/// * the scene is FRAMED once — the opening does not travel across the margin as
///   the page is dragged, so what the page does is mask;
/// * the two margins CANNOT disagree, even where awl's column sits off-centre.
///   That is not hypothetical: in the wide room at the widest page the margins
///   are 405px and 139px, and round 2's rule put one axis behind the page and
///   the other out in its margin at the same instant — round 1's rejected
///   "two separately cropped circles", re-created by the fix for something else.
///
/// Proven capable of failing:
/// `warp_margin_placed_windows_let_the_page_reframe_the_scene`.
#[test]
fn the_two_windows_are_placed_by_the_room_so_the_page_can_only_mask() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cells = sweep_cells(&device, &queue, kite());
    // (room, measure, span, side, measured axis, the room's own prediction).
    let mut placed: Vec<((u32, u32), usize, f32, usize, f32, f32)> = Vec::new();
    for c in &cells {
        let cam = cam_for(c.room, spacing);
        for (i, f) in c.fit.iter().enumerate() {
            let Some(f) = f else { continue };
            placed.push((
                c.room,
                c.measure,
                c.span[i],
                i,
                f.cx,
                cam.axis_x(c.room.0, i == 1),
            ));
        }
    }
    assert!(
        placed.len() >= 20,
        "the placement sweep must grade margins, got {}\n{placed:?}",
        placed.len()
    );
    // 1. Every window sits where the ROOM puts it — measured, not predicted from
    //    anything the page owns.
    for &(room, measure, span, side, cx, want) in &placed {
        assert!(
            (cx - want).abs() <= PLACEMENT_TOL_PX,
            "{room:?} m{measure}/margin {side}: a {span:.0}px margin put its window's \
             axis at x={cx:.0} where the ROOM's one placement owner puts it at \
             x={want:.0}. A placement that moves with the page is the page reframing \
             the scene, not masking it.\n{placed:?}"
        );
    }
    // 2. ...and therefore the axis does not MOVE across the band. Stated
    //    separately and measured from the pixels alone, so it cannot be a
    //    restatement of the prediction above: within one room and one side, the
    //    whole spread of fitted centres over every page width must be a couple of
    //    pixels of tracing noise.
    for room in ROOMS {
        for side in 0..2usize {
            let xs: Vec<f32> = placed
                .iter()
                .filter(|p| p.0 == room && p.3 == side)
                .map(|p| p.4)
                .collect();
            if xs.len() < 3 {
                continue;
            }
            let lo = xs.iter().cloned().fold(f32::MAX, f32::min);
            let hi = xs.iter().cloned().fold(f32::MIN, f32::max);
            assert!(
                hi - lo <= PLACEMENT_TOL_PX,
                "{room:?}/margin {side}: the window's axis swept {lo:.0}..{hi:.0}px \
                 across the measure band ({} widths). One consistently framed scene \
                 means this number does not move.\n{placed:?}",
                xs.len()
            );
        }
    }
    // 3. ...and the two margins are the same distance from their OWN room edge,
    //    at every width, including the widths where the column is off-centre.
    for room in ROOMS {
        for &(r, measure, _, _, _, _) in placed.iter().filter(|p| p.0 == room) {
            let left: Vec<f32> = placed
                .iter()
                .filter(|p| p.0 == r && p.1 == measure && p.3 == 0)
                .map(|p| p.4)
                .collect();
            let right: Vec<f32> = placed
                .iter()
                .filter(|p| p.0 == r && p.1 == measure && p.3 == 1)
                .map(|p| r.0 as f32 - p.4)
                .collect();
            if let (Some(&l), Some(&rr)) = (left.first(), right.first()) {
                assert!(
                    (l - rr).abs() <= PLACEMENT_TOL_PX,
                    "{r:?} m{measure}: the left window sits {l:.0}px from its room edge \
                     and the right {rr:.0}px from its own. The two margins are not one \
                     scene.\n{placed:?}"
                );
            }
        }
    }
    // 4. The sweep must actually contain an OFF-CENTRE column, or claim 3 above
    //    is being asserted only where it is trivially true.
    let asymmetric = cells
        .iter()
        .any(|c| c.off_centre && c.fit.iter().all(|f| f.is_some()) && (c.span[0] - c.span[1]).abs() > 100.0);
    assert!(
        asymmetric,
        "the sweep must include a page width where awl's own column sits badly \
         off-centre — that geometry is where round 2's placement put the two margins \
         in different regimes, and a sweep without it cannot see the defect.\n\
         spans: {:?}",
        cells
            .iter()
            .map(|c| (c.room, c.measure, c.span))
            .collect::<Vec<_>>()
    );
}

/// THE MUTATION PROOF for the law above. `Tunnel::MarginPlaced` is round 2's own
/// placement kept as data — the axis slid inward by a smoothstep on the margin's
/// own width. The sweep must go red on it, and this test says so out loud: it
/// asserts the fitted centre MOVES across the band, so it fails the day the
/// mutation stops being one.
#[test]
fn warp_margin_placed_windows_let_the_page_reframe_the_scene() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let bad = with_tunnel(kite(), theme::Tunnel::MarginPlaced);
    let cells = sweep_cells(&device, &queue, bad);
    let mut moved = 0usize;
    let mut worst: f32 = 0.0;
    // The mutation must be ROUND 2's rule, not merely a different number: every
    // graded margin's measured centre has to land where round 2's own
    // margin-derived placement puts it. Without this the test would still pass
    // if the arm degenerated into noise, and a mutation proof that only asserts
    // "different" cannot tell a restored defect from a broken shader.
    let mut predicted = 0usize;
    for c in &cells {
        let cam = cam_for(c.room, spacing);
        for (i, f) in c.fit.iter().enumerate() {
            let Some(f) = f else { continue };
            // Only where round 2's OWN rule leaves the axis out in the margin.
            // Where it hid the axis behind the page the margin holds a short
            // outer flank and the fit is not a measurement of a centre — the
            // same limit `axis_is_visible` records for the shipping placement,
            // applied to the arm's own geometry rather than to the shipped one.
            let hide = cam.hide(c.span[i], (c.col_w * 0.5).max(1.0));
            if hide >= 0.0 {
                continue;
            }
            let want = cam.margin_placed_axis_x(c.room.0, c.col_left, c.col_w, i == 1);
            assert!(
                (f.cx - want).abs() <= PLACEMENT_TOL_PX,
                "{:?} m{}/margin {i}: the MarginPlaced arm must reproduce round 2's own \
                 placement — measured axis x={:.0}, round 2's rule x={want:.0}",
                c.room,
                c.measure,
                f.cx
            );
            predicted += 1;
        }
    }
    assert!(
        predicted >= 2,
        "the mutation proof must actually check round 2's placement somewhere it can \
         be measured, and it checked {predicted} margins"
    );
    for room in ROOMS {
        for side in 0..2usize {
            let xs: Vec<f32> = cells
                .iter()
                .filter(|c| c.room == room)
                .filter_map(|c| c.fit[side].map(|f| f.cx))
                .collect();
            if xs.len() < 3 {
                continue;
            }
            let lo = xs.iter().cloned().fold(f32::MAX, f32::min);
            let hi = xs.iter().cloned().fold(f32::MIN, f32::max);
            worst = worst.max(hi - lo);
            if hi - lo > PLACEMENT_TOL_PX {
                moved += 1;
            }
        }
    }
    assert!(
        moved > 0,
        "MarginPlaced must let the page MOVE the window's axis across the band — \
         that is round 2's remaining defect — and the widest sweep measured only \
         {worst:.0}px of travel, inside the {PLACEMENT_TOL_PX:.0}px the law allows"
    );
}

// ---------------------------------------------------------------------------
// 3. THE COHERENCE — one camera, recovered independently from each margin.
// ---------------------------------------------------------------------------

/// How badly a candidate camera explains one margin's ink: the marks of a
/// correct camera sit ON its integer level sets, so the ink-weighted distance to
/// the nearest one is near zero, and a wrong camera scatters them toward the
/// 0.25 of a uniform phase. Pure pixels in, one number out.
fn phase_score(fr: &Frame, m: (u32, u32), cam: &Camera, axis_x: f32, st: Steer) -> f64 {
    // The MAJOR rungs only, weighted by their own strength. A bend's fold leaves
    // a broad low-contrast wash where the far wall passes behind the near one,
    // and that wash sits on no level set at all — reading it as evidence is what
    // makes a fit shrug at exactly the pose it most needs to resolve.
    let floor = fr.major_floor(m);
    let (mut num, mut den) = (0.0f64, 0.0f64);
    let inset = 80u32;
    let (x0, x1) = if m.0 == 0 {
        (6, m.1 - inset)
    } else {
        (m.0 + inset, m.1 - 6)
    };
    let mut y = 10;
    while y < fr.h - 10 {
        let mut x = x0;
        while x < x1 {
            let v = fr.at(x, y);
            if v >= floor {
                let (fx, fy) = (x as f32, y as f32);
                let (r0, l0) = cam.lattice(axis_x, fx, fy, st);
                let (rx, lx) = cam.lattice(axis_x, fx + 1.0, fy, st);
                let (ry, ly) = cam.lattice(axis_x, fx, fy + 1.0, st);
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
/// this model puts it.
const ONE_CAMERA_PX: f64 = 1.6;
/// ...and any materially different camera must miss by at least this multiple,
/// so "it fits" is a statement about uniqueness rather than about slack. It is
/// asked only of a window that REACHES the tunnel's near-vanishing region (see
/// `identifies_the_camera`), because that is where the evidence about steering
/// lives: measured worst 2.43 over both margins and every pose there.
const WRONG_CAMERA_RATIO: f64 = 2.3;
/// How far a candidate must differ from the route's own steering before it is
/// held to that ratio.
///
/// It WIDENED in round 2, from 0.15, and the reason is the repair itself: round
/// 1 flattened the whole picture by up to 4x to fit the section into the room,
/// which multiplied every PITCH error on the glass by the same factor and made
/// small pitch rivals easy to refute. Round 2's projection is isotropic, so a
/// 0.15 pitch moves the near rings a canonical-column window can see by about
/// seven pixels — genuinely near-degenerate evidence, and claiming to refute it
/// would be claiming precision the picture does not carry. 0.30 is still well
/// under half of any bend the route commits to (yaw 0.70/0.74, pitch 0.48/0.52).
const MATERIALLY_DIFFERENT: f32 = 0.30;

/// Whether a window carries enough of the cylinder to IDENTIFY the steering, as
/// opposed to merely be consistent with it.
///
/// Steering displaces a ring by `|B|/u`, so all the evidence is at small `u` —
/// the deep end. A window whose nearest visible radius is more than half an
/// anchor cannot see that end at all: at the canonical column a committed left
/// bend and a straight camera differ there by 2.4x, and a 0.6 pitch on top of
/// that bend by 1.13x, which is not a refutation and this module will not claim
/// one. So the uniqueness half of the law is asked exactly where the picture can
/// answer it, and the fit half is asked everywhere.
///
/// It is also a product fact worth having in a law's own terms: the wider the
/// page, the less of the cylinder each window holds, and the less a turn can
/// possibly read. The live review is the judge of how much less is too much.
fn identifies_the_camera(cam: &Camera, col_left: f32, col_w: f32, on_right: bool) -> bool {
    let span = if on_right {
        W as f32 - (col_left + col_w)
    } else {
        col_left
    }
    .max(1.0);
    cam.hide(span, (col_w * 0.5).max(1.0)) <= 0.5 * cam.anchor
}

/// Every camera this law holds up against the route's own, for one pose.
fn rivals(yaw: f32, pitch: f32) -> Vec<(f32, f32)> {
    let mut v = vec![(0.0, 0.0), (-yaw, -pitch)];
    for d in [-0.6f32, -0.45, -0.3, 0.3, 0.45, 0.6] {
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

/// THE TURNING-COHERENCE LAW, re-aimed at round 2's model. At EVERY pose the
/// route reaches, the left margin and the right margin — read independently,
/// each on its own ink, each through its OWN window placement — must both be
/// explained by the SAME steering, and by no other.
///
/// The re-aiming is the point. Round 1 fitted both margins against ONE axis at
/// the page's centre line, so the law asserted the round-1 continuation
/// invariant and would have gone red on round 2's honest geometry for the wrong
/// reason. Round 2 gives each margin the window `warp_window_hide` puts there
/// and demands the same CAMERA through both — which is the claim that survived
/// the model change, because a translation is not a camera. Horizon, curvature
/// and vanishing direction are each a function of that one pose, so this remains
/// one statement of all three disagreements the live review reported.
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
    let cam = Camera::new(H, spacing);
    let mut report = Vec::new();
    for (col_left, col_w) in GEOMETRIES {
        for name in ["straight", "left", "climb", "right", "descent", "wrap"] {
            let phase = warpgrid::named_pose(name).expect("a named route pose");
            let pose = warpgrid::route_pose(phase);
            let fr = Frame {
                f: field(&device, &queue, kite(), W, H, col_left, col_w, phase),
                w: W,
                h: H,
            };
            let st = |yaw: f32, pitch: f32| Steer {
                yaw,
                pitch,
                curvature: curv,
                forward: pose.forward_cells,
            };
            for (i, m) in margins_at(col_left, col_w).into_iter().enumerate() {
                let side = if i == 0 { "left" } else { "right" };
                let axis_x = cam.axis_x(W, i == 1);
                let mine = phase_score(&fr, m, &cam, axis_x, st(pose.yaw, pose.pitch));
                report.push((col_w as u32, name, side, mine));
                assert!(
                    mine < ONE_CAMERA_PX,
                    "col {col_w:.0}/{name}/{side}: the route's own camera (yaw {:+.2} \
                     pitch {:+.2}) does not explain this margin — its marks miss the \
                     one shared projection by {mine:.2}px on average. The margin is \
                     composing something of its own.\nscores so far: {report:?}",
                    pose.yaw,
                    pose.pitch
                );
                if !identifies_the_camera(&cam, col_left, col_w, i == 1) {
                    continue;
                }
                for (ry, rp) in rivals(pose.yaw, pose.pitch) {
                    let theirs = phase_score(&fr, m, &cam, axis_x, st(ry, rp));
                    assert!(
                        theirs >= mine * WRONG_CAMERA_RATIO,
                        "col {col_w:.0}/{name}/{side}: a DIFFERENT camera (yaw \
                         {ry:+.2} pitch {rp:+.2}) explains this margin nearly as well \
                         as the route's own ({theirs:.2}px against {mine:.2}px) — the \
                         margin is not pinned to the shared pose at all"
                    );
                }
            }
            // The two margins are pinned to the SAME camera to the same standard —
            // stated separately so the failure names the comparison the review made.
            let (l, r) = (report[report.len() - 2].3, report[report.len() - 1].3);
            assert!(
                (l - r).abs() < ONE_CAMERA_PX * 0.5,
                "col {col_w:.0}/{name}: the two margins are not equally well explained \
                 by one camera (left {l:.2}px, right {r:.2}px) — one of them is \
                 steering on its own"
            );
        }
    }
}

/// THE MUTATION PROOF. `Tunnel::PerMargin` is the defect kept as data: each
/// margin re-derives the steering from its own side of the page. The law above
/// must go red on it, and this test is what says so out loud.
///
/// Two statements, because the evidence a margin carries about the camera is
/// itself a function of the page width. At EVERY geometry the shared camera must
/// be REFUTED — which is exactly the assertion the coherence law makes, so this
/// is that law's own failure, reproduced. At the narrowest page, whose window
/// holds the tunnel's deep end where a steering error is largest, the margin
/// must additionally be explained BETTER by the flipped camera than by the
/// route's, by the same margin the coherence law demands of a rival: the full
/// inversion.
#[test]
fn warp_per_margin_steering_breaks_the_shared_camera() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let (spacing, curv) = kite_dials();
    let cam = Camera::new(H, spacing);
    let bad = with_tunnel(kite(), theme::Tunnel::PerMargin);
    let mut inverted = 0usize;
    let mut report = Vec::new();
    for (col_left, col_w) in GEOMETRIES {
        for name in ["left", "right"] {
            let phase = warpgrid::named_pose(name).expect("a named route pose");
            let pose = warpgrid::route_pose(phase);
            let fr = Frame {
                f: field(&device, &queue, bad, W, H, col_left, col_w, phase),
                w: W,
                h: H,
            };
            let st = |yaw: f32, pitch: f32| Steer {
                yaw,
                pitch,
                curvature: curv,
                forward: pose.forward_cells,
            };
            let m = margins_at(col_left, col_w)[0];
            let axis_x = cam.axis_x(W, false);
            let shared = phase_score(&fr, m, &cam, axis_x, st(pose.yaw, pose.pitch));
            let flipped = phase_score(&fr, m, &cam, axis_x, st(-pose.yaw, -pose.pitch));
            report.push((col_w as u32, name, shared, flipped));
            assert!(
                shared > ONE_CAMERA_PX,
                "col {col_w:.0}/{name}: the per-margin mutation must be visible to the \
                 coherence law — the left margin reads {shared:.2}px against the \
                 shared camera, inside the {ONE_CAMERA_PX}px that law allows, so the \
                 mutation has stopped being one.\n{report:?}"
            );
            if shared > flipped * WRONG_CAMERA_RATIO {
                inverted += 1;
            }
        }
    }
    assert!(
        inverted >= 2,
        "the mutation must fully INVERT the fit somewhere in the band — each \
         margin explained by its OWN flipped camera rather than the route's, by \
         the {WRONG_CAMERA_RATIO}x a rival is held to. Got {inverted}.\n{report:?}"
    );
}

/// The host model above exists for these laws alone, so nothing but a grep-law
/// keeps it honest. Every constant it carries is asserted to be the literal the
/// shader declares.
#[test]
fn the_host_model_matches_the_shaders_own_constants() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for (name, value) in [
        ("WARP_SECTION_ROOM_FRAC", SECTION_ROOM_FRAC),
        ("WARP_WINDOW_INSET", WINDOW_INSET),
        ("WARP_WINDOW_FULL", WINDOW_FULL),
        ("WARP_WINDOW_TIGHT", WINDOW_TIGHT),
        ("WARP_WINDOW_STRADDLE", WINDOW_STRADDLE),
        ("WARP_RING_PITCH_AT", RING_PITCH_AT),
        ("WARP_RAILS_PER_HALF_TURN", RAILS_PER_HALF_TURN),
        ("WARP_MAJOR_EVERY", MAJOR_EVERY),
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

// ---------------------------------------------------------------------------
// 4. THE DIRECTION — forward travel approaches, and one sign says so.
// ---------------------------------------------------------------------------

/// How much forward travel the direction sweep covers, in minor cells. It stays
/// strictly under [`warpgrid::MAJOR_EVERY`] on purpose: one major rung of travel
/// carries every ring onto the NEXT ring's radius, and a sweep that crosses one
/// cannot tell "the rings grew" from "the lattice repeated". Under a rung, the
/// ring a margin is tracing keeps its identity and its radius is a clean
/// monotone function of the travel.
/// Two cells grows a ring by `2^(2/rpo)` — about 50% here — which is far more
/// than tracing noise and comfortably under the 2.83x that would carry it onto
/// its neighbour. Four cells was tried first and is too many for a different
/// reason worth recording: the tracked ring grows out of the window entirely
/// (the anchor ring already reaches the room's half-height) and the ladder then
/// has nothing left to follow.
const DIRECTION_SWEEP_CELLS: f32 = 2.0;
const DIRECTION_SWEEP_STEPS: usize = 5;

/// **FORWARD TRAVEL APPROACHES.** The user-visible defect item 199 opened with
/// was that the world read as travelling BACKWARDS, and it is one character:
/// the ring coordinate is `rpo*log2(anchor/u) + Z`, so a ring is drawn at
/// `u = anchor * 2^((Z - n)/rpo)` and its projected radius GROWS as the route's
/// `forward_cells` grows — the lattice sweeps outward past the reader. Subtract
/// `Z` instead and every radius shrinks toward the axis: the rings converge into
/// the far end, which is what receding looks like.
///
/// This is a deterministic proof over real pixels and not a reading of the
/// shader: the same margin's section is traced at a ladder of phases spanning
/// less than one major rung of travel, and `log2(radius)` must rise, with the
/// slope the projection predicts (`+1/rpo` per cell of travel) rather than
/// merely some positive number. A world that travelled forward at the wrong
/// speed would pass a bare sign test and fails this one.
///
/// Proven capable of failing: `warp_reversed_travel_recedes`.
#[test]
fn forward_travel_grows_the_projected_rings() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cam = Camera::new(H, spacing);
    let (cells, radii) = tracked_ladder(&device, &queue, kite(), &cam);
    // The DIRECTION claim is asserted before the coverage one, so that a world
    // travelling the wrong way fails by its own name rather than as a tracker
    // that ran out of ring to follow.
    assert!(
        radii.len() >= 2,
        "the direction ladder could not track one ring for even two phases\n\
         {cells:?} {radii:?}"
    );
    for w in radii.windows(2) {
        assert!(
            w[1] > w[0],
            "forward travel must GROW the projected rings — the lattice sweeps outward \
             past the reader. Radius went {:.1}px -> {:.1}px as the route travelled \
             forward, which is the world receding.\ncells: {cells:?}\nradii: {radii:?}",
            w[0],
            w[1]
        );
    }
    assert!(
        radii.len() >= DIRECTION_SWEEP_STEPS,
        "the direction ladder must grade every phase, got {} of {DIRECTION_SWEEP_STEPS}\n\
         {cells:?} {radii:?}",
        radii.len()
    );
    // ...at the rate the projection predicts, so a merely-positive drift cannot
    // pass for travel. Slope of log2(radius) against cells travelled is 1/rpo.
    let n = radii.len() as f64;
    let xs: Vec<f64> = cells.iter().map(|c| *c as f64).collect();
    let ys: Vec<f64> = radii.iter().map(|r| (*r as f64).log2()).collect();
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    let slope = num / den;
    let want = 1.0 / cam.rpo as f64;
    assert!(
        (slope - want).abs() <= 0.15 * want,
        "forward travel must grow the rings at the rate the projection fixes: measured \
         {slope:+.4} octaves per cell against the predicted {want:+.4} (rpo {:.2}).\n\
         cells: {cells:?}\nradii: {radii:?}",
        cam.rpo
    );
}

/// THE MUTATION PROOF for the law above. `Tunnel::Reversed` is the old sign kept
/// as data. The ladder must go the other way, and this test says so out loud.
#[test]
fn warp_reversed_travel_recedes() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cam = Camera::new(H, spacing);
    let bad = with_tunnel(kite(), theme::Tunnel::Reversed);
    let (cells, radii) = tracked_ladder(&device, &queue, bad, &cam);
    assert!(
        radii.len() >= DIRECTION_SWEEP_STEPS,
        "the direction ladder must grade every phase under the mutation too, got {}\n\
         {cells:?} {radii:?}",
        radii.len()
    );
    let shrank = radii.windows(2).filter(|w| w[1] < w[0]).count();
    assert_eq!(
        shrank,
        radii.len() - 1,
        "Reversed must make every step of forward travel SHRINK the rings — that is \
         the defect, and the world reading as backwards is what it looks like.\n\
         cells: {cells:?}\nradii: {radii:?}"
    );
}

/// Run the ladder from BOTH ends and keep the longer track. A ring only stays
/// traceable in the direction it has room to move, and which end that is depends
/// on which way the world is travelling — which is the very thing being
/// measured, so it cannot be assumed. Seeding both ways and keeping whichever
/// ring survived means a reversed world fails on the DIRECTION claim, by its own
/// name, instead of on a tracker that ran out of ring.
fn tracked_ladder(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    cam: &Camera,
) -> (Vec<f32>, Vec<f32>) {
    let inner = direction_ladder(device, queue, bg, cam, Seed::Innermost);
    let outer = direction_ladder(device, queue, bg, cam, Seed::Outermost);
    if outer.1.len() > inner.1.len() {
        outer
    } else {
        inner
    }
}

/// Which ring the ladder follows. A ring only stays traceable in the direction
/// it has room to move: the innermost determined rung can GROW for a while
/// before it leaves the window, and the outermost can SHRINK. So the shipped
/// world is tracked from the inside out and the reversed arm from the outside
/// in — the same measurement, seeded where the arm being measured leaves it
/// something to follow.
#[derive(Clone, Copy)]
enum Seed {
    Innermost,
    Outermost,
}

/// Trace ONE margin's section at a ladder of route phases, all inside a single
/// straight leg, and return `(cells travelled, fitted radius)` per phase.
///
/// The margin is the canonical geometry's left one and the phases are taken from
/// the route itself rather than invented, so what is measured is the world the
/// live app runs, one frozen frame at a time.
fn direction_ladder(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    cam: &Camera,
    seed: Seed,
) -> (Vec<f32>, Vec<f32>) {
    let per_cell = warpgrid::ROUTE_LOOP_SECONDS / warpgrid::FORWARD_CELLS_PER_LOOP;
    let (mut cells, mut radii): (Vec<f32>, Vec<f32>) = (Vec::new(), Vec::new());
    for step in 0..DIRECTION_SWEEP_STEPS {
        let travelled =
            DIRECTION_SWEEP_CELLS * step as f32 / (DIRECTION_SWEEP_STEPS - 1).max(1) as f32;
        let phase = warpgrid::FROZEN_PHASE + travelled * per_cell;
        let pose = warpgrid::route_pose(phase);
        // The ladder must stay inside ONE straight leg: a bend moves the section
        // off-centre, and a fit that straddles the ease would be reading the
        // turn, not the travel.
        assert!(
            pose.yaw.abs() < 1e-4 && pose.pitch.abs() < 1e-4,
            "the direction ladder must stay on a straight leg — at {phase:.3}s the route \
             steers yaw {:.4} pitch {:.4}",
            pose.yaw,
            pose.pitch
        );
        let f = Frame {
            f: field(device, queue, bg, W, H, COL_LEFT, COL_W, phase),
            w: W,
            h: H,
        };
        let m = margins_at(COL_LEFT, COL_W)[0];
        let candidates = sections(&f, m, true);
        // TRACK ONE RING. The best-corroborated arc is not necessarily the same
        // ring from frame to frame, and swapping to its neighbour is a factor of
        // ~2.8 in radius — which would drown the few percent the travel itself
        // moves it. So the first frame takes the best-corroborated section and
        // every later frame takes the one NEAREST to it.
        let fit = match radii.last() {
            // SEED ON THE INNERMOST determined section, not the
            // best-corroborated one. The best-corroborated is the anchor ring,
            // which already reaches the room's half-height and leaves the window
            // as soon as it grows; an inner rung has room to travel.
            None => match seed {
                Seed::Innermost => candidates.iter().min_by(|a, b| a.u.total_cmp(&b.u)),
                Seed::Outermost => candidates.iter().max_by(|a, b| a.u.total_cmp(&b.u)),
            }
            .copied(),
            Some(&prev) => candidates
                .iter()
                .min_by(|a, b| {
                    (a.u - prev)
                        .abs()
                        .total_cmp(&(b.u - prev).abs())
                })
                .copied(),
        };
        let Some(fit) = fit else {
            continue;
        };
        // Guard the ring's IDENTITY. The traced ring's radius is SUPPOSED to
        // move — that is the whole measurement — so the guard cannot be on the
        // absolute rung. It is on the STEP: one rung of the room's ladder is a
        // factor of `2^(MAJOR_EVERY/rpo)` (about 2.8 here), and the ladder's own
        // steps are a few percent, so a `best_section` that jumped to the
        // neighbouring ring shows up as a step of more than half a rung.
        let rung_factor = (MAJOR_EVERY / cam.rpo).exp2();
        if let Some(&prev) = radii.last() {
            let step = (fit.u / prev).max(prev / fit.u);
            // The tracked ring is LOST rather than swapped: stop the ladder and
            // let the caller judge what it has. A panic here would mask the
            // claim — under a reversed sign the tracked ring shrinks out of the
            // window, and the reader needs to be told the world is receding, not
            // that a tracker gave up.
            if step >= rung_factor.sqrt() {
                break;
            }
        }
        cells.push(pose.forward_cells);
        radii.push(fit.u);
    }
    (cells, radii)
}
