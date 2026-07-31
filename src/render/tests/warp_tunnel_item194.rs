//! ITEM 194 — ONE CYLINDER AT ONE CONSTANT SCALE, SEEN THROUGH TWO WINDOWS.
//!
//! Item 132's own laws (`backgrounds_item132`) prove the ground exists, stays
//! out of the page, keeps its hierarchy and never aliases. Round 1's laws lived
//! here and proved something else again: that the two margins are two crops of
//! ONE projection rather than two independently composed fields.
//!
//! **THE MODEL THEY WERE WRITTEN FOR WAS SUPERSEDED, so the laws were re-aimed
//! rather than re-tuned.** Round 1's contract was "one cylinder continuing
//! behind the page", with the section SIZED from the page column
//! (`anchor = 3 * page_half`) so the overlap was implied by occlusion. The live
//! review approved that at the narrowest page and failed it everywhere else:
//! widening the page rescaled and flattened the world, because the authored
//! scale was a function of the margin the picture had to land in. Round 2's
//! contract is "two overlapping windows onto one cylinder": the cylinder never
//! rescales, and each margin is a window onto that one fixed projection, sliding
//! inward and OVERLAPPING as the margins narrow so the centre appears in both.
//!
//! The audit that mattered is recorded in THEMES.md: every round-1 law here was
//! asked whether it could still pass under a model it was never written for, and
//! the one that could — a straight-pose composition target measured at a SINGLE
//! page width — was replaced by a sweep, because a single width is exactly where
//! this defect hides. Round 1's own geometry is kept as `theme::Tunnel::
//! PageScaled` so the replacement can be watched failing on the composition it
//! names, at every width but the one round 1 got right.
//!
//! THREE CLAIMS, ALL ARITHMETIC OVER REAL GPU PIXELS — the differential `field`
//! oracle item 86 authored, so the flat ground cancels and what is measured is
//! the lattice alone.
//!
//! 1. **The scale.** Across the whole adaptive-column range, every major
//!    cross-ring arc a margin offers is TRACED and solved — with a FREE centre —
//!    for the ellipse it belongs to, and the best-corroborated survivor is the
//!    margin's answer. Its ASPECT RATIO must be the constant 1.00 and its radius
//!    must sit on the room's own fixed ring ladder. Recovering the opening from
//!    an ARC rather than a whole ring is the point: no margin holds the
//!    section's apex, so the eye infers the opening from the flank, and so does
//!    this law.
//! 2. **The windows.** The same fitted centres say where each window sits: the
//!    two must TILE (both centres behind the page) while the margins are wide,
//!    and OVERLAP (each centre out in its own margin) once they are not — with
//!    the slide monotone in the margin's width between.
//! 3. **The coherence, at every route pose.** Each margin's ink is fitted, ON
//!    ITS OWN, for the camera that would explain it. The two margins must
//!    recover the SAME steering — which is one statement of all three things the
//!    review found disagreeing, since horizon, curvature and vanishing direction
//!    are each a function of that one pose.
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

    /// `warp_window_hide`, mirrored: how far INSIDE the page edge one margin's
    /// copy of the axis falls. Positive hides it behind the page (the windows
    /// tile); negative brings it out into the margin (they overlap).
    fn hide(&self, span: f32, page_half: f32) -> f32 {
        let full = smoothstep(WINDOW_TIGHT, WINDOW_FULL, span / self.anchor.max(1.0));
        let tight = -WINDOW_STRADDLE * span;
        tight + (page_half - tight) * full
    }

    /// Where that margin's window puts the axis on the glass.
    fn axis_x(&self, w: u32, col_left: f32, col_w: f32, on_right: bool) -> f32 {
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
        .max_by_key(|f| (f.votes, f.n))
}

/// One graded cell of the page-width sweep: the geometry, and what each margin's
/// own traced arc says about the section it is an arc of.
#[derive(Debug, Clone, Copy)]
struct Cell {
    measure: usize,
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
fn measure_sweep() -> Vec<usize> {
    (crate::page::MIN_MEASURE..=96)
        .step_by(crate::page::MEASURE_STEP)
        .collect()
}

/// A margin narrower than this cannot hold an arc worth solving — the field's
/// own narrow-margin simplification has already retired the minor lattice and
/// the page-edge quiet band eats most of what is left.
const TRACEABLE_MARGIN_PX: f32 = 120.0;

/// Render the straight pose at every page width in the sweep, at the app's OWN
/// adaptive column, and solve each margin's own arcs for the section it shows.
fn sweep_cells(device: &wgpu::Device, queue: &wgpu::Queue, bg: theme::Background) -> Vec<Cell> {
    let mut out = Vec::new();
    let restore = crate::page::measure();
    for measure in measure_sweep() {
        let Some((_d, _q, mut pipe)) = super::headless_dqp(W as f32, H as f32) else {
            break;
        };
        crate::page::set_measure(measure);
        pipe.set_view(&super::view("hello", 0, 0));
        let (col_left, col_w) = (pipe.column_left(), pipe.column_width());
        let f = Frame {
            f: field(
                device,
                queue,
                bg,
                W,
                H,
                col_left,
                col_w,
                warpgrid::FROZEN_PHASE,
            ),
            w: W,
            h: H,
        };
        let ms = super::backgrounds_item89::margins(W, col_left, col_w);
        let mut cell = Cell {
            measure,
            col_left,
            col_w,
            span: [col_left, W as f32 - (col_left + col_w)],
            fit: [None, None],
        };
        for (i, m) in ms.into_iter().enumerate() {
            if cell.span[i] < TRACEABLE_MARGIN_PX {
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
/// from the fit's own measured spread over the whole band — aspect 0.943..1.016
/// and radius within 0.040 of a ladder rung — rather than from a wish. The
/// defect they exist to catch is 3.00 of aspect and 1.5 rungs of scale, an order
/// of magnitude outside them. The CENTRAL bounds below are what pin the value.
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

/// THE HEADLINE LAW OF ROUND 2, and the defect stated as arithmetic: **the
/// projected cross-section's aspect ratio is invariant across the full adaptive
/// column range**, and so is its scale.
///
/// Swept over the whole measure band at the app's own column owner, each
/// margin's major cross-ring arcs are traced and solved for the ellipse they
/// belong to with a free centre. Two numbers come out and neither may move: the ASPECT
/// must be the constant 1.00 (the section is a circle — round 2 removed the
/// affine fit entirely), and the RADIUS must sit on the ladder the ROOM fixes,
/// `anchor * 2^(-k * MAJOR_EVERY / rpo)`, so no page width can rescale the
/// world it lands in.
///
/// Round 1 measured 1.00 to 4.00 on the first of those and 432px to 1942px on
/// the second, over this same band — and it is the sweep that says so: at the
/// NARROWEST page the two models agree exactly, which is why the live review
/// approved that pose and failed every wider one, and why a law measuring one
/// width could pass while the world did something else.
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
    let cam = Camera::new(H, spacing);
    let cells = sweep_cells(&device, &queue, kite());
    let mut graded = 0usize;
    let mut aspects: Vec<f32> = Vec::new();
    let mut rungs: Vec<f32> = Vec::new();
    let report: Vec<(usize, [f32; 2], [Option<Fit>; 2])> =
        cells.iter().map(|c| (c.measure, c.span, c.fit)).collect();
    for c in &cells {
        for i in 0..2 {
            let Some(f) = c.fit[i] else { continue };
            assert!(
                (f.aspect - 1.0).abs() <= ASPECT_TOL,
                "m{}/margin {i}: the projected cross-section must be a CIRCLE at every \
                 page width — measured aspect {:.3} against 1.000. A section whose \
                 shape depends on the page is the item-194 defect itself.\n\
                 sweep: {report:?}",
                c.measure,
                f.aspect
            );
            // The radius must be a rung of the ROOM's ladder, not merely stable.
            let rung = cam.rpo * (cam.anchor / f.u).log2() / MAJOR_EVERY;
            let rungs_off = rung - rung.round();
            assert!(
                rungs_off.abs() <= RUNG_TOL,
                "m{}/margin {i}: the traced ring's radius {:.1}px is not a rung of the \
                 room's own ring ladder (anchor {:.1}px, {:.2} rungs off an integer) — \
                 the projection has been rescaled by the page.\nsweep: {report:?}",
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
    // The sweep must reach BOTH ends of the band, not merely many widths in the
    // middle: the two models agree at the narrowest page, so a sweep that stops
    // short of the wide end is the single-width law again in disguise.
    for (label, want) in [
        (
            "widest margin",
            cells
                .iter()
                .any(|c| c.fit[0].is_some() && c.span[0] >= cam.anchor),
        ),
        (
            "narrowest traceable margin",
            cells
                .iter()
                .any(|c| c.fit[0].is_some() && c.span[0] <= cam.anchor * WINDOW_TIGHT),
        ),
    ] {
        assert!(
            want,
            "the page-width sweep must grade the {label} — that is the axis this \
             defect lives on.\nsweep: {report:?}"
        );
    }
    assert!(
        graded >= 14,
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
///
/// It also pins the shape of the defect the live review reported: the two models
/// AGREE at the narrowest page — which is why that pose was approved — and
/// diverge monotonically as the page widens.
#[test]
fn warp_page_scaled_projection_breaks_the_one_scale() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cam = Camera::new(H, spacing);
    let bad = with_tunnel(kite(), theme::Tunnel::PageScaled);
    let cells = sweep_cells(&device, &queue, bad);
    let mut aspects: Vec<(usize, f32)> = Vec::new();
    let mut off_ladder = 0usize;
    for c in &cells {
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
        aspects.len() >= 4,
        "the mutation sweep must grade margins too, got {}",
        aspects.len()
    );
    let lo = aspects.iter().map(|&(_, a)| a).fold(f32::MAX, f32::min);
    let hi = aspects.iter().map(|&(_, a)| a).fold(f32::MIN, f32::max);
    assert!(
        hi - lo > ASPECT_TOL * 3.0,
        "the page-scaled arm must make the section's aspect ratio a FUNCTION of the \
         page width — measured {lo:.3}..{hi:.3}, which the invariance law would not \
         even notice. The mutation has stopped being one.\n{aspects:?}"
    );
    assert!(
        off_ladder >= 2,
        "the page-scaled arm must also move the ring ladder off the room's own \
         ({off_ladder} margins off it) — otherwise the scale half of the law is \
         untested.\n{aspects:?}"
    );
    // The narrowest page is where round 1 was RIGHT, and the live review said so.
    let narrowest = aspects
        .iter()
        .filter(|&&(m, _)| m == crate::page::MIN_MEASURE)
        .map(|&(_, a)| a)
        .fold(f32::MIN, f32::max);
    assert!(
        (narrowest - 1.0).abs() <= ASPECT_TOL,
        "at the NARROWEST page the two models must still agree ({narrowest:.3}) — that \
         is the pose the live review approved, and the reason a single-width law \
         could pass while the world did something else"
    );
}

// ---------------------------------------------------------------------------
// 2. THE WINDOWS — tiling while there is room, overlapping once there is not.
// ---------------------------------------------------------------------------

/// TWO WINDOWS ONTO ONE CYLINDER, and the slide between them. Round 1's contract
/// was one cylinder continuing behind the page with the overlap merely implied;
/// round 2's makes it VISIBLE, so the law has to be about where each window
/// sits, measured from the fitted centre of each margin's own arc.
///
/// Three claims across the swept page widths:
///
/// * WIDE margins TILE — both centres sit behind the opaque page, so each window
///   shows its own side of the cylinder and nothing is duplicated. This is round
///   1's approved reading at the narrowest page, and it is preserved exactly.
/// * NARROW margins OVERLAP — each centre has come out from behind the page into
///   its own margin, so the centre of the cylinder appears in BOTH windows. The
///   duplication is intended.
/// * The slide between them is MONOTONE in the margin's width: no page width
///   jumps the window, which is what makes dragging the page read as one
///   continuous motion rather than a cut.
#[test]
fn the_two_windows_tile_while_the_margins_are_wide_and_overlap_once_they_are_not() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let (spacing, _) = kite_dials();
    let cam = Camera::new(H, spacing);
    let cells = sweep_cells(&device, &queue, kite());
    /// How far a measured axis may sit from where the one placement owner puts
    /// it. A traced arc recovers its centre to a pixel or two; this is the slack
    /// that leaves, not the claim.
    const PLACEMENT_TOL_PX: f32 = 12.0;
    // (measure, span, span/anchor, measured hide, predicted hide, page half) per
    // margin. The PAGE HALF is carried separately on purpose: the tiling claim
    // below is that the axis sits on the page column's own centre line, and
    // reading that off the placement owner's prediction — which equals it in that
    // regime — would make the claim a restatement of the assertion above it
    // rather than an independent measurement of the composition.
    let mut placed: Vec<(usize, f32, f32, f32, f32, f32)> = Vec::new();
    for c in &cells {
        let page_half = (c.col_w * 0.5).max(1.0);
        for (i, f) in c.fit.iter().enumerate() {
            let Some(f) = f else { continue };
            let hide = if i == 0 {
                f.cx - c.col_left
            } else {
                c.col_left + c.col_w - f.cx
            };
            placed.push((
                c.measure,
                c.span[i],
                c.span[i] / cam.anchor,
                hide,
                cam.hide(c.span[i], page_half),
                page_half,
            ));
        }
    }
    // A bare floor, so a sweep that graded almost nothing cannot pass quietly.
    // The claim about COVERAGE is the two-regime assertion below, which is what a
    // pinned window fails on: it does not merely grade fewer cells, it never
    // reaches the regime where the windows overlap at all.
    assert!(
        placed.len() >= 8,
        "the placement sweep must grade margins, got {}\n{placed:?}",
        placed.len()
    );
    let mut wide = 0usize;
    let mut tight = 0usize;
    for &(measure, span, ratio, hide, want, page_half) in &placed {
        assert!(
            (hide - want).abs() <= PLACEMENT_TOL_PX,
            "m{measure}: a {span:.0}px margin puts its window's axis {hide:.0}px inside \
             the page edge where the ONE placement owner puts it {want:.0}px — the two \
             margins are not reading the same rule.\n{placed:?}"
        );
        if ratio >= WINDOW_FULL {
            wide += 1;
            // TILING: the axis is on the page's own centre line, hidden behind
            // it, so each window shows its own side and nothing is duplicated.
            // This is round 1's approved composition, preserved exactly.
            assert!(
                hide > 0.0 && (hide - page_half).abs() <= PLACEMENT_TOL_PX,
                "m{measure}: a margin {ratio:.2} anchors wide must TILE — its axis \
                 belongs on the page's own centre line, {page_half:.0}px in, and was \
                 measured {hide:.0}px in.\n{placed:?}"
            );
        }
        if ratio <= WINDOW_TIGHT {
            tight += 1;
            // OVERLAP: the axis has come out from behind the page into the
            // margin itself, so the cylinder's centre appears in BOTH windows.
            assert!(
                hide < 0.0,
                "m{measure}: a margin only {ratio:.2} anchors wide must have slid its \
                 window INWARD until the axis left the page ({hide:+.0}px) — that \
                 overlap is what keeps a thin margin showing the opening.\n{placed:?}"
            );
            let straddle = -hide / span;
            assert!(
                (straddle - WINDOW_STRADDLE).abs() <= 0.12,
                "m{measure}: a fully slid window must sit {WINDOW_STRADDLE:.2} of its \
                 own width in from the page — ~60% of its own side of the cylinder and \
                 ~40% of the other's — and this one measured {straddle:.2}.\n{placed:?}"
            );
        }
    }
    assert!(
        wide >= 4 && tight >= 2,
        "both regimes must be exercised: {wide} tiling margins, {tight} overlapping \
         ones.\n{placed:?}"
    );
    // The slide is MONOTONE in the margin's own width: no page width jumps the
    // window, so dragging the page reads as one continuous motion.
    // ...over the SLIDE's own band: past `WINDOW_FULL` the window is parked on
    // the page's centre line, and `page_half` growing with the page is not the
    // window moving.
    let mut by_ratio: Vec<_> = placed
        .iter()
        .copied()
        .filter(|p| p.2 <= WINDOW_FULL)
        .collect();
    by_ratio.sort_by(|a, b| a.2.total_cmp(&b.2));
    for pair in by_ratio.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let slide = |x: &(usize, f32, f32, f32, f32, f32)| x.3 / x.1; // hide, per own width
        assert!(
            slide(&b) >= slide(&a) - 0.02,
            "the slide must be monotone in the margin's width: a margin {:.2} anchors \
             wide holds its axis {:.2} of its own width in and a WIDER {:.2} one only \
             {:.2}.\n{by_ratio:?}",
            a.2,
            slide(&a),
            b.2,
            slide(&b)
        );
    }
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
                let axis_x = cam.axis_x(W, col_left, col_w, i == 1);
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
            let axis_x = cam.axis_x(W, col_left, col_w, false);
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
