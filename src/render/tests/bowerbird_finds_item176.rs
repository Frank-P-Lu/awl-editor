//! ITEM 176 — the ORGANIC ground's crisp COLLECTED-TREASURE arrangement
//! (`theme::Arrangement::Finds`), proved from real rendered pixels.
//!
//! **What the grammar claims.** One cell draws one deliberately arranged
//! collection of THREE crisp objects: a large ANCHOR, a smaller COMPANION
//! offset across its edge, and a tiny CUT-OUT punched back to the open ground.
//! Every proportion — scale, offset, rotation, overlap, kind, tone assignment —
//! is seeded from the cell's own identity, so collections differ while the
//! three roles never do.
//!
//! **Why the reader below does not name roles by tone.** The two pieces
//! deliberately SWAP the world's two object tones cluster to cluster, so a tone
//! is not a role. The reader identifies the pieces by AREA (major/minor) and
//! then states the grammar as four clauses that a two-object or an
//! equal-object field genuinely fails:
//!   * a real second piece exists (`minor >= MIN_MINOR_PX`);
//!   * a real hierarchy separates them (`major >= HIERARCHY_RATIO * minor`) —
//!     two pieces of a similar size are NOT this grammar;
//!   * a real cut-out exists, ENCLOSED by the collection's own ink
//!     (`cutout >= MIN_CUTOUT_PX`);
//!   * the cut-out is the smallest of the three (`minor > 2 * cutout`).
//!
//! `the_collection_reader_rejects_two_object_and_equal_object_fields` feeds the
//! same reader hand-built fields missing exactly one of those properties and
//! asserts it says no — the anti-tautology proof a counting law owes, since a
//! reader that assigns "anchor" to whichever region is bigger would otherwise
//! report `anchor > companion` for any two blobs at all.

use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use crate::theme;

/// A `Background::Organic` at an explicit arrangement and cell scale, on
/// Bowerbird's own authored tones and density — the direct-injection seam, so
/// every claim below is about the MECHANISM rather than one world's literal.
fn organic_bg(arrangement: theme::Arrangement, scale_px: f32) -> theme::Background {
    match theme::BOWERBIRD.background {
        theme::Background::Organic { tones, density, .. } => theme::Background::Organic {
            tones,
            arrangement,
            scale_px,
            density,
        },
        _ => panic!("Bowerbird must ship Background::Organic"),
    }
}

// --- The field reader -------------------------------------------------------

/// Open ground, the two object tones, and everything between them (a boundary
/// pixel inside the shader's sub-pixel feather).
const GROUND: u8 = 0;
const EDGE: u8 = 3;

/// The three plateau colours of a rendered field, DISCOVERED by frequency
/// rather than recomputed here: the shader mixes in linear space and the sRGB
/// target re-encodes on write, so a host-side copy of that arithmetic would be
/// a second implementation of the thing under test. Returned darkest-first, so
/// index 0 is the open ground.
fn plateau_tones(pixels: &[[u8; 4]]) -> [[u8; 4]; 3] {
    let mut counts: std::collections::HashMap<[u8; 4], usize> = std::collections::HashMap::new();
    for p in pixels {
        *counts.entry(*p).or_default() += 1;
    }
    let mut top: Vec<([u8; 4], usize)> = counts.into_iter().collect();
    top.sort_by_key(|(c, n)| (std::cmp::Reverse(*n), *c));
    // A field that draws fewer than three plateaus (a deliberately incomplete
    // one, in the reader's own rejection tests) pads with a colour no pixel can
    // hold, so the missing role reads as an area of ZERO rather than panicking.
    let absent = [1u8, 1, 1, 0];
    let mut three: Vec<[u8; 4]> = top.iter().take(3).map(|(c, _)| *c).collect();
    while three.len() < 3 {
        three.push(absent);
    }
    let ground = three[0];
    three[1..].sort_by_key(|c| c[0] as u32 + c[1] as u32 + c[2] as u32);
    [ground, three[1], three[2]]
}

/// Label every pixel: `GROUND`, `1`/`2` for the two object tones, `EDGE` for
/// anything else (the antialiased skirt).
fn label_field(pixels: &[[u8; 4]], tones: [[u8; 4]; 3]) -> Vec<u8> {
    pixels
        .iter()
        .map(|p| match tones.iter().position(|t| t == p) {
            Some(0) => GROUND,
            Some(i) => i as u8,
            None => EDGE,
        })
        .collect()
}

/// One rendered COLLECTION: the connected run of ink one cell drew, with the
/// pixel area of each piece. `major`/`minor` are the two object-tone regions
/// ordered by size (the tones themselves swap cluster to cluster), `cutout` the
/// open-ground region their ink encloses.
#[derive(Debug, Clone, Copy, Default)]
struct Collection {
    major_px: usize,
    minor_px: usize,
    cutout_px: usize,
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
}

/// A real second piece: comfortably under the smallest companion the authored
/// ranges can draw at the smallest defended cell (radius `COMPANION_LO *
/// ANCHOR_LO * MIN_SCALE_PX`), so the floor gates a MISSING piece, not a small
/// one.
const MIN_MINOR_PX: usize = 90;
/// A real cut-out. Derived at the arrangement's own cell FLOOR, where the
/// smallest authored cut-out (`ACCENT_LO * ANCHOR_LO * MIN_SCALE_PX` ~ 2.9px
/// radius) keeps only a ~2px solid core once its antialiased skirt is taken
/// off — so this gates a MISSING hole, not a small one. At Bowerbird's shipped
/// 156px cell the same cut-outs measure five to ten times this.
const MIN_CUTOUT_PX: usize = 8;
/// A real hierarchy. The authored companion is at most `COMPANION_HI` of the
/// anchor radius, so even before the companion's own overlap is subtracted the
/// visible areas differ by `1 / COMPANION_HI^2 - 1` ~ 2.2x; this floor sits
/// well under that and well ABOVE 1.0, so two pieces of a similar size fail.
const HIERARCHY_RATIO: f64 = 1.8;

impl Collection {
    fn has_three_roles(&self) -> bool {
        self.minor_px >= MIN_MINOR_PX
            && self.cutout_px >= MIN_CUTOUT_PX
            && (self.major_px as f64) >= HIERARCHY_RATIO * self.minor_px as f64
            && self.minor_px > 2 * self.cutout_px
    }

    fn describe(&self) -> String {
        format!(
            "major {}px, minor {}px, cut-out {}px, {}x{}px at ({:.0}, {:.0})",
            self.major_px, self.minor_px, self.cutout_px, self.width, self.height, self.cx, self.cy
        )
    }
}

/// Flood-fill component ids over a mask, 8-connected. `usize::MAX` where the
/// mask is false.
fn components(mask: &[bool], w: usize, h: usize) -> (Vec<usize>, usize) {
    let mut id = vec![usize::MAX; mask.len()];
    let mut next = 0usize;
    let mut stack: Vec<usize> = Vec::new();
    for seed in 0..mask.len() {
        if !mask[seed] || id[seed] != usize::MAX {
            continue;
        }
        id[seed] = next;
        stack.push(seed);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let j = ny as usize * w + nx as usize;
                    if mask[j] && id[j] == usize::MAX {
                        id[j] = next;
                        stack.push(j);
                    }
                }
            }
        }
        next += 1;
    }
    (id, next)
}

/// The open-ground pixels reachable from the canvas border, 4-connected — so
/// every ground pixel NOT reached is a cut-out enclosed by some collection.
fn outside_ground(labels: &[u8], w: usize, h: usize) -> Vec<bool> {
    let mut seen = vec![false; labels.len()];
    let mut stack: Vec<usize> = Vec::new();
    let push = |i: usize, seen: &mut Vec<bool>, stack: &mut Vec<usize>| {
        if labels[i] == GROUND && !seen[i] {
            seen[i] = true;
            stack.push(i);
        }
    };
    for x in 0..w {
        push(x, &mut seen, &mut stack);
        push((h - 1) * w + x, &mut seen, &mut stack);
    }
    for y in 0..h {
        push(y * w, &mut seen, &mut stack);
        push(y * w + w - 1, &mut seen, &mut stack);
    }
    while let Some(i) = stack.pop() {
        let (x, y) = (i % w, i / w);
        for (nx, ny) in [
            (x as i32 - 1, y as i32),
            (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1),
            (x as i32, y as i32 + 1),
        ] {
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            push(ny as usize * w + nx as usize, &mut seen, &mut stack);
        }
    }
    seen
}

/// Read every WHOLE collection out of a rendered field. Collections whose ink
/// touches the canvas border are dropped: they are cropped by the viewport, not
/// by the grammar, and a cropped collection has no honest role areas.
fn read_collections(pixels: &[[u8; 4]], w: u32, h: u32) -> Vec<Collection> {
    let (wi, hi) = (w as usize, h as usize);
    let tones = plateau_tones(pixels);
    let labels = label_field(pixels, tones);
    let ink: Vec<bool> = labels.iter().map(|l| *l != GROUND).collect();
    let (id, count) = components(&ink, wi, hi);
    let mut out = vec![Collection::default(); count];
    let mut clipped = vec![false; count];
    let mut x_sum = vec![0f64; count];
    let mut y_sum = vec![0f64; count];
    let mut n = vec![0f64; count];
    let (mut x0, mut x1) = (vec![u32::MAX; count], vec![0u32; count]);
    let (mut y0, mut y1) = (vec![u32::MAX; count], vec![0u32; count]);
    for i in 0..labels.len() {
        let c = id[i];
        if c == usize::MAX {
            continue;
        }
        let (x, y) = ((i % wi) as u32, (i / wi) as u32);
        if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
            clipped[c] = true;
        }
        match labels[i] {
            1 => out[c].major_px += 1,
            2 => out[c].minor_px += 1,
            _ => {}
        }
        x0[c] = x0[c].min(x);
        x1[c] = x1[c].max(x);
        y0[c] = y0[c].min(y);
        y1[c] = y1[c].max(y);
        x_sum[c] += x as f64;
        y_sum[c] += y as f64;
        n[c] += 1.0;
    }
    let seen = outside_ground(&labels, wi, hi);
    for i in 0..labels.len() {
        if labels[i] != GROUND || seen[i] {
            continue;
        }
        let (x, y) = ((i % wi) as i32, (i / wi) as i32);
        'find: for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let c = id[ny as usize * wi + nx as usize];
                if c != usize::MAX {
                    out[c].cutout_px += 1;
                    break 'find;
                }
            }
        }
    }
    for c in 0..count {
        let (hi, lo) = (
            out[c].major_px.max(out[c].minor_px),
            out[c].major_px.min(out[c].minor_px),
        );
        out[c].major_px = hi;
        out[c].minor_px = lo;
        out[c].width = x1[c] + 1 - x0[c];
        out[c].height = y1[c] + 1 - y0[c];
        out[c].cx = (x_sum[c] / n[c]) as f32;
        out[c].cy = (y_sum[c] / n[c]) as f32;
    }
    (0..count)
        .filter(|c| !clipped[*c])
        .map(|c| out[c])
        .collect()
}

// --- The grammar laws -------------------------------------------------------

const FIELD_W: u32 = 1800;
const FIELD_H: u32 = 1200;
/// A `FIELD_W x FIELD_H` canvas at the shipped 156px cell holds ~89 cells, ~10%
/// of which are deliberately empty and ~40% of which touch the border and are
/// dropped as cropped. Anything near or under this floor means the reader found
/// almost nothing and every per-collection assertion below went vacuous.
const MIN_COLLECTIONS: usize = 25;

/// The drift law's own canvas: several cells across in both axes, small enough
/// that an exhaustive shift search stays cheap.
const DRIFT_W: u32 = 900;
const DRIFT_H: u32 = 700;

fn finds_field(device: &wgpu::Device, queue: &wgpu::Queue, scale: f32, drift: f32) -> Vec<[u8; 4]> {
    let bg = organic_bg(theme::Arrangement::Finds, scale);
    render_bg(
        device,
        queue,
        bg_desc_for(bg),
        FIELD_W,
        FIELD_H,
        0.0,
        0.0,
        drift,
    )
}

/// LAW: every whole collection the field draws carries all three roles in the
/// authored hierarchy — a dominant piece, a genuinely smaller second piece on
/// the OTHER tone, and a tiny cut-out enclosed by their own ink.
#[test]
fn finds_every_collection_shows_the_three_role_hierarchy() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_every_collection_shows_the_three_role_hierarchy: no adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0, 0.0);
    let found = read_collections(&pixels, FIELD_W, FIELD_H);
    assert!(
        found.len() >= MIN_COLLECTIONS,
        "only {} whole collections read out of a {FIELD_W}x{FIELD_H} field — under the \
         {MIN_COLLECTIONS} floor every per-collection assertion here is vacuous",
        found.len()
    );
    for c in &found {
        assert!(
            c.has_three_roles(),
            "a collection is not an arrangement of three: {}",
            c.describe()
        );
    }
}

/// THE ANTI-TAUTOLOGY PROOF. The reader identifies pieces by area, so a naive
/// "the bigger region is the anchor, therefore anchor > companion" law would
/// pass on ANY two blobs. These hand-built fields each break exactly one clause
/// of the grammar and must each be rejected — so the law above is known to
/// count what it thinks it counts.
#[test]
fn the_collection_reader_rejects_two_object_and_equal_object_fields() {
    const S: u32 = 200;
    let ground = [12u8, 20, 38, 255];
    let lo = [15u8, 25, 44, 255];
    let hi = [23u8, 33, 57, 255];
    let paint = |buf: &mut Vec<[u8; 4]>, x0: u32, y0: u32, w: u32, h: u32, c: [u8; 4]| {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                buf[(y * S + x) as usize] = c;
            }
        }
    };
    let three_roles = |anchor: u32, companion: u32, cutout: u32| -> Vec<Collection> {
        let mut buf = vec![ground; (S * S) as usize];
        paint(&mut buf, 50, 50, anchor, anchor, lo);
        if companion > 0 {
            paint(
                &mut buf,
                50 + anchor - 8,
                50 + anchor - 8,
                companion,
                companion,
                hi,
            );
        }
        if cutout > 0 {
            paint(&mut buf, 62, 62, cutout, cutout, ground);
        }
        read_collections(&buf, S, S)
    };

    let ok = three_roles(70, 30, 6);
    assert_eq!(ok.len(), 1, "the control field must read as one collection");
    assert!(
        ok[0].has_three_roles(),
        "the control three-object field must be accepted: {}",
        ok[0].describe()
    );

    let no_cutout = three_roles(70, 30, 0);
    assert_eq!(no_cutout.len(), 1);
    assert_eq!(no_cutout[0].cutout_px, 0);
    assert!(
        !no_cutout[0].has_three_roles(),
        "a TWO-object collection must be rejected, but the reader accepted {}",
        no_cutout[0].describe()
    );

    let no_companion = three_roles(70, 0, 6);
    assert_eq!(no_companion.len(), 1);
    assert_eq!(no_companion[0].minor_px, 0);
    assert!(
        !no_companion[0].has_three_roles(),
        "a ONE-object collection must be rejected, but the reader accepted {}",
        no_companion[0].describe()
    );

    // Two pieces of a similar size are a pair, not an anchor and a companion.
    let equals = three_roles(70, 66, 6);
    assert_eq!(equals.len(), 1);
    assert!(
        !equals[0].has_three_roles(),
        "an EQUAL-object collection must be rejected, but the reader accepted {}",
        equals[0].describe()
    );

    // A speck is not a cut-out.
    let speck = three_roles(70, 30, 2);
    assert_eq!(speck.len(), 1);
    assert!(
        !speck[0].has_three_roles(),
        "a sub-floor cut-out must be rejected, but the reader accepted {}",
        speck[0].describe()
    );
}

/// LAW (the axis a single-scale check would miss): the grammar must survive
/// every cell scale a future Organic world could author, not only Bowerbird's
/// shipped 156px. `32.0` is below the arrangement's own `FINDS_MIN_SCALE_PX`
/// floor and must be CLAMPED up to it rather than aliasing into speckle, so it
/// is swept here deliberately.
#[test]
fn finds_holds_the_three_role_grammar_at_every_reachable_cell_scale() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_holds_the_three_role_grammar_at_every_reachable_cell_scale");
        return;
    };
    let _g = crate::testlock::serial();
    for scale in [
        32.0f32,
        theme::ORGANIC_FINDS_MIN_SCALE_PX,
        156.0,
        312.0,
        400.0,
    ] {
        let pixels = finds_field(&device, &queue, scale, 0.0);
        let found = read_collections(&pixels, FIELD_W, FIELD_H);
        // A 400px cell fits far fewer whole collections than a 96px one, so the
        // floor scales with the cell rather than pinning one number.
        let floor = ((FIELD_W as f32 * FIELD_H as f32) / (scale.max(96.0).powi(2)) * 0.15) as usize;
        assert!(
            found.len() >= floor.max(4),
            "scale {scale}: only {} whole collections (floor {}) — the sweep went vacuous",
            found.len(),
            floor.max(4)
        );
        for c in &found {
            assert!(
                c.has_three_roles(),
                "scale {scale}: a collection is not an arrangement of three: {}",
                c.describe()
            );
        }
    }
}

/// LAW: the deterministic variation must not repeat conspicuously, and the
/// scatter must not resolve into a grid. Two measurements, both from pixels:
/// the collections' own size/shape signatures are mostly distinct, and their
/// centres do NOT line up on the cell pitch (an axis-aligned lattice would put
/// every centre at the same offset within its cell; the rotated, row-sheared
/// lattice spreads them across it).
#[test]
fn finds_variation_does_not_repeat_and_the_scatter_is_not_a_grid() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_variation_does_not_repeat_and_the_scatter_is_not_a_grid");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0, 0.0);
    let found = read_collections(&pixels, FIELD_W, FIELD_H);
    assert!(found.len() >= MIN_COLLECTIONS, "vacuous: {}", found.len());

    let mut signatures: std::collections::HashMap<(usize, usize, u32, u32), usize> =
        std::collections::HashMap::new();
    for c in &found {
        *signatures
            .entry((
                c.major_px / 60,
                c.minor_px / 30,
                c.width / 10,
                c.height / 10,
            ))
            .or_default() += 1;
    }
    let distinct = signatures.len();
    assert!(
        distinct * 10 >= found.len() * 6,
        "only {distinct} distinct size/shape signatures across {} collections — the field \
         repeats conspicuously",
        found.len()
    );
    let worst = *signatures.values().max().unwrap();
    assert!(
        worst * 5 <= found.len(),
        "one signature accounts for {worst} of {} collections",
        found.len()
    );

    let phase = |v: f32| -> f32 { v.rem_euclid(156.0) / 156.0 };
    for axis in 0..2 {
        let xs: Vec<f32> = found
            .iter()
            .map(|c| phase(if axis == 0 { c.cx } else { c.cy }))
            .collect();
        let mean = xs.iter().sum::<f32>() / xs.len() as f32;
        let sd = (xs.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / xs.len() as f32).sqrt();
        // A perfect lattice puts every centre at one phase (sd -> 0); a uniform
        // spread over the cell gives 1/sqrt(12) = 0.289.
        assert!(
            sd > 0.20,
            "axis {axis}: collection centres sit at phase sd {sd:.3} of the cell pitch — the \
             scatter has collapsed onto a visible grid"
        );
    }
}

// --- Motion, gates and the ground it sits on --------------------------------

/// The share of pixels that match `b` within `tol` after shifting `b` by
/// `(sx, sy)`, over the overlapping region only.
#[allow(clippy::too_many_arguments)]
fn agreement(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    w: u32,
    h: u32,
    sx: i32,
    sy: i32,
    tol: i32,
    step: i32,
) -> f64 {
    let (mut hit, mut total) = (0usize, 0usize);
    for y in (0..h as i32).step_by(step as usize) {
        for x in (0..w as i32).step_by(step as usize) {
            let (bx, by) = (x + sx, y + sy);
            if bx < 0 || by < 0 || bx >= w as i32 || by >= h as i32 {
                continue;
            }
            let p = a[(y as u32 * w + x as u32) as usize];
            let q = b[(by as u32 * w + bx as u32) as usize];
            total += 1;
            if (0..3).all(|k| (p[k] as i32 - q[k] as i32).abs() <= tol) {
                hit += 1;
            }
        }
    }
    hit as f64 / total.max(1) as f64
}

/// The share of INK-bearing positions (non-ground in either frame) that agree
/// exactly once `b` is shifted by `at`.
fn ink_agreement(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    w: u32,
    h: u32,
    at: (i32, i32),
    ground: [u8; 4],
) -> f64 {
    let (mut hit, mut total) = (0usize, 0usize);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let (bx, by) = (x + at.0, y + at.1);
            if bx < 0 || by < 0 || bx >= w as i32 || by >= h as i32 {
                continue;
            }
            let p = a[(y as u32 * w + x as u32) as usize];
            let q = b[(by as u32 * w + bx as u32) as usize];
            if p == ground && q == ground {
                continue;
            }
            total += 1;
            if (0..3).all(|k| (p[k] as i32 - q[k] as i32).abs() <= 2) {
                hit += 1;
            }
        }
    }
    hit as f64 / total.max(1) as f64
}

/// LAW: the ambient drift moves the WHOLE field as one rigid translation. It
/// never morphs, spawns, dissolves, or animates one object of a collection
/// against its neighbours — the property the item asks be preserved, and the
/// one a per-object animation would break while every "some pixel changed"
/// witness stayed green. Proved by finding the single integer offset that best
/// aligns the two phases and showing the field agrees with itself almost
/// everywhere there, and NOT at rest.
#[test]
fn finds_drift_is_one_rigid_whole_field_translation() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_drift_is_one_rigid_whole_field_translation");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = bg_desc_for(organic_bg(theme::Arrangement::Finds, 156.0));
    let field = |drift: f32| render_bg(&device, &queue, bg, DRIFT_W, DRIFT_H, 0.0, 0.0, drift);
    let settled = field(0.0);
    let phase = std::f32::consts::FRAC_PI_2;
    let moved = field(phase);

    // The authored displacement BETWEEN the two phases, straight from the
    // shader's own formula. The settled frame already sits at a nonzero y
    // offset (`cos(0) == 1`), so the prediction is the delta, not the endpoint.
    let (ax, ay) = ((156.0f32 * 0.13).max(12.0), (156.0f32 * 0.10).max(9.0));
    let dx = -(phase.sin() * ax - 0.0f32.sin() * ax);
    let dy = -((phase * 0.73).cos() * ay - 1.0 * ay);
    // Coarse search over every candidate whole-field shift, on a 1-in-9 sample
    // (the field's own features are tens of pixels across, so a subsample
    // cannot hide the peak), then measured at full resolution where it lands.
    let (mut peak, mut best_at) = (0.0f64, (0i32, 0i32));
    for sy in -24..=24 {
        for sx in -32..=32 {
            let a = agreement(&settled, &moved, DRIFT_W, DRIFT_H, sx, sy, 2, 3);
            if a > peak {
                peak = a;
                best_at = (sx, sy);
            }
        }
    }
    assert!(
        (best_at.0 as f32 - dx).abs() <= 1.5 && (best_at.1 as f32 - dy).abs() <= 1.5,
        "the field's best alignment is {best_at:?}, but the authored drift predicts \
         ({dx:.1}, {dy:.1}) — the field is not translating by the amount the shader asks for"
    );
    // Measured over INK, not over the whole canvas: the arrangement leaves most
    // of the ground open, so two unshifted phases already agree on ~90% of ALL
    // pixels and a whole-canvas figure could never tell travel from stillness.
    let ground = plateau_tones(&settled)[0];
    let best = ink_agreement(&settled, &moved, DRIFT_W, DRIFT_H, best_at, ground);
    assert!(
        best >= 0.90,
        "after its best whole-field shift the drifted field agrees with the settled one on \
         only {:.1}% of its ink — objects are changing shape, not travelling",
        best * 100.0
    );
    let at_rest = ink_agreement(&settled, &moved, DRIFT_W, DRIFT_H, (0, 0), ground);
    assert!(
        at_rest < 0.60,
        "the two phases' ink already agrees {:.1}% WITHOUT shifting — the drift is not \
         moving the field, so the rigid-translation claim above is vacuous",
        at_rest * 100.0
    );
}

/// LAW (the worst-phase sweep, re-stated for the new arrangement): the field
/// stays inside Bowerbird's cool navy value band and out of the writing column
/// at EVERY point of the ambient cycle — a crisp field with hard edges cannot
/// be allowed to push a worst-phase pixel warm, bright, or onto the page.
#[test]
fn finds_worst_phase_stays_cool_and_off_the_page() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_worst_phase_stays_cool_and_off_the_page");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = organic_bg(theme::Arrangement::Finds, 156.0);
    let (w, h, left, col) = (900u32, 600u32, 220.0f32, 460.0f32);
    for i in 0..24 {
        let drift = (i as f32 / 24.0) * std::f32::consts::TAU;
        let pixels = render_bg(&device, &queue, bg_desc_for(bg), w, h, left, col, drift);
        for (idx, p) in pixels.iter().enumerate() {
            let x = (idx as u32) % w;
            if (x as f32) >= left && (x as f32) < left + col {
                assert_eq!(
                    [p[0], p[1], p[2]],
                    [0, 0, 0],
                    "drift {drift}: collected ink entered the page column at x={x}"
                );
                continue;
            }
            assert!(
                p[2] >= p[0] && p[0] < 90,
                "drift {drift}: warm/bright margin pixel {p:?} — the ground must stay cool"
            );
        }
    }
}

/// LAW: `density: 0.0` collapses the arrangement to the flat open ground
/// EXACTLY — the differential oracle every pixel law here measures against, and
/// the property that keeps a future world's `density` dial honest.
#[test]
fn finds_density_zero_is_exactly_the_flat_ground() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_density_zero_is_exactly_the_flat_ground");
        return;
    };
    let _g = crate::testlock::serial();
    let mut flat = bg_desc_for(organic_bg(theme::Arrangement::Finds, 156.0));
    flat.density = 0.0;
    let pixels = render_bg(&device, &queue, flat, 600, 400, 0.0, 0.0, 0.0);
    let first = pixels[0];
    assert!(
        pixels.iter().all(|p| *p == first),
        "density 0 must leave one flat tone; found at least two"
    );
    let inked = render_bg(
        &device,
        &queue,
        bg_desc_for(organic_bg(theme::Arrangement::Finds, 156.0)),
        600,
        400,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        inked.iter().any(|p| *p != first),
        "the authored density must actually draw something over that flat ground"
    );
}

/// LAW: the open navy ground stays generous. A crisp field must not creep
/// toward the wall-to-wall coverage the rounded masses draw (which is exactly
/// what reads as camouflage), nor thin out into an isolated constellation.
#[test]
fn finds_leaves_a_generous_open_ground() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_leaves_a_generous_open_ground");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0, 0.0);
    let tones = plateau_tones(&pixels);
    let ground = pixels.iter().filter(|p| **p == tones[0]).count() as f64 / pixels.len() as f64;
    assert!(
        (0.70..0.95).contains(&ground),
        "the collected field leaves {:.1}% open ground — outside the generous-but-populated \
         band this world's Frame is built on",
        ground * 100.0
    );
}

// --- Edges: the thing a crisp composition can get wrong ----------------------

/// Boundary statistics along horizontal scanlines: how many plateau-to-plateau
/// crossings the field draws, how many of them jump with NO intermediate pixel
/// (a hard, aliased step), and the mean width of the antialiased ones.
fn edge_stats(pixels: &[[u8; 4]], w: u32, h: u32) -> (usize, usize, f64) {
    let tones = plateau_tones(pixels);
    let labels = label_field(pixels, tones);
    let (mut crossings, mut hard) = (0usize, 0usize);
    let mut runs: Vec<usize> = Vec::new();
    for y in 0..h as usize {
        let row = &labels[y * w as usize..(y + 1) * w as usize];
        let mut last_plateau: Option<u8> = None;
        let mut run = 0usize;
        for l in row {
            if *l == EDGE {
                run += 1;
                continue;
            }
            if let Some(prev) = last_plateau
                && prev != *l
            {
                crossings += 1;
                if run == 0 {
                    hard += 1;
                } else {
                    runs.push(run);
                }
            }
            last_plateau = Some(*l);
            run = 0;
        }
    }
    runs.sort_unstable();
    let median = if runs.is_empty() {
        0.0
    } else {
        runs[runs.len() / 2] as f64
    };
    (crossings, hard, median)
}

/// LAW (1x and 2x): a crisp composition is exactly where aliasing bites, so
/// edge quality is asserted at BOTH sampling densities rather than trusted at
/// one. The shader feathers in PHYSICAL pixels, so doubling the device ratio
/// (twice the canvas, twice the cell) must leave the same sub-pixel skirt:
/// nearly every boundary antialiased, and the skirt still narrow enough to read
/// as an edge rather than a blur.
#[test]
fn finds_edges_stay_antialiased_and_crisp_at_1x_and_2x() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_edges_stay_antialiased_and_crisp_at_1x_and_2x");
        return;
    };
    let _g = crate::testlock::serial();
    for (density, scale, w, h) in [(1u32, 156.0f32, 1200u32, 800u32), (2, 312.0, 2400, 1600)] {
        let bg = organic_bg(theme::Arrangement::Finds, scale);
        let pixels = render_bg(&device, &queue, bg_desc_for(bg), w, h, 0.0, 0.0, 0.0);
        let (crossings, hard, mean) = edge_stats(&pixels, w, h);
        assert!(
            crossings >= 600,
            "{density}x: only {crossings} boundary crossings found — the edge sweep is vacuous"
        );
        assert!(
            hard * 10 <= crossings,
            "{density}x: {hard} of {crossings} boundaries step with no intermediate pixel — the \
             field is aliasing"
        );
        assert!(
            (0.5..2.6).contains(&mean),
            "{density}x: the mean antialiased boundary is {mean:.2}px wide — outside the crisp \
             band (too hard aliases, too soft is the haze this arrangement replaces)"
        );
    }
}

// --- The roster, and everything the arrangement must NOT touch --------------

/// LAW (no-wildcard): the crisp arrangement ships DORMANT. Every world with an
/// Organic ground carries `Masses` today, and every ground without one reports
/// the inert profile scalar, so no other world's upload changes shape.
#[test]
fn organic_arrangement_roster_is_masses_only_and_the_profile_slot_stays_inert() {
    for t in theme::THEMES {
        let arrangement = match t.background {
            theme::Background::Gradient { .. } => None,
            theme::Background::Dots { .. } => None,
            theme::Background::Starfield { .. } => None,
            theme::Background::Pinstripe { .. } => None,
            theme::Background::Stripes { .. } => None,
            theme::Background::Lava { .. } => None,
            theme::Background::Bands { .. } => None,
            theme::Background::Waves { .. } => None,
            theme::Background::Zigzag { .. } => None,
            theme::Background::Organic { arrangement, .. } => Some(arrangement),
            theme::Background::Deckle { .. } => None,
        };
        assert_eq!(
            arrangement,
            (t.name == "Bowerbird").then_some(theme::Arrangement::Masses),
            "{}: the crisp arrangement is not shipped by any world yet",
            t.name
        );
        assert_eq!(
            t.background.arrangement(),
            arrangement,
            "{}: the arrangement accessor disagrees with the literal",
            t.name
        );
    }
    assert_eq!(theme::Arrangement::Masses.mode(), 0.0);
    assert_eq!(theme::Arrangement::Finds.mode(), 1.0);
}

/// LAW: the ambient gates belong to the GROUND, not to the arrangement. Both
/// arrangements report the same `is_organic` / `has_ambient_tick`, so the whole
/// freeze truth table already pinned by
/// `theme::tests::bowerbird_organic_schedules_zero_frames_under_every_freeze_
/// condition` (Reduce Motion, `ambient_motion = false`, focus lost, paused) and
/// the real-pixel Reduce Motion proof in `bowerbird_drift_item163` apply to
/// either one unchanged. A revival that quietly armed a second clock, or that
/// dropped out of the shared one, would show up right here.
#[test]
fn both_arrangements_ride_the_one_shared_ambient_gate() {
    for arrangement in [theme::Arrangement::Masses, theme::Arrangement::Finds] {
        let mut world = theme::BOWERBIRD;
        world.background = organic_bg(arrangement, 156.0);
        assert!(
            world.background.is_organic(),
            "{arrangement:?}: the ground must stay Organic to the renderer"
        );
        assert!(
            world.has_ambient_tick(),
            "{arrangement:?}: the ground must stay enrolled in the ONE shared ambient tick"
        );
        assert!(
            !world.has_ambient_motion(),
            "{arrangement:?}: the ground must not claim a lava world's motion budget"
        );
    }
}

/// LAW: `Masses` uploads the SAME four params it did before the arrangement
/// existed — the inert-default contract, checked at the packing seam every
/// ground shares rather than at Bowerbird's literal.
#[test]
fn the_arrangement_rides_only_organics_own_param_slot() {
    for t in theme::THEMES {
        let desc = bg_desc_for(t.background);
        if desc.shader != 8 {
            assert_eq!(
                desc.profile,
                t.background.profile_mode(),
                "{}: the profile slot must carry this ground's own dial",
                t.name
            );
            continue;
        }
        assert_eq!(
            desc.profile, 0.0,
            "{}: ships the dormant arrangement",
            t.name
        );
    }
    let finds = bg_desc_for(organic_bg(theme::Arrangement::Finds, 156.0));
    let masses = bg_desc_for(organic_bg(theme::Arrangement::Masses, 156.0));
    assert_eq!(finds.profile, 1.0);
    assert_eq!(masses.profile, 0.0);
    assert_eq!(
        (
            finds.period_px,
            finds.density,
            finds.angle,
            finds.amplitude_px
        ),
        (
            masses.period_px,
            masses.density,
            masses.angle,
            masses.amplitude_px
        ),
        "the arrangement must be the ONLY difference between the two organic uploads"
    );
}

/// LAW (grep lockstep): the host-side mirrors this file's arithmetic depends on
/// must still be the numbers the WGSL draws with. A shader retune that silently
/// left the mirrors behind would otherwise leave every bound above measuring a
/// field that no longer exists.
#[test]
fn the_finds_shader_still_reads_its_mirrored_constants() {
    let src = include_str!("../../../shaders/background.wgsl");
    let declared = |name: &str| -> f32 {
        let needle = format!("const {name}: f32 = ");
        let at = src
            .get(..)
            .and_then(|s| s.find(&needle))
            .unwrap_or_else(|| panic!("shaders/background.wgsl no longer declares `{name}`"));
        let rest = &src[at + needle.len()..];
        let end = rest.find(';').expect("unterminated constant");
        rest[..end].trim().parse().expect("constant is not a float")
    };
    for (name, value) in [
        ("FINDS_MIN_SCALE_PX", theme::ORGANIC_FINDS_MIN_SCALE_PX),
        ("FINDS_ANCHOR_LO", theme::ORGANIC_FINDS_ANCHOR_LO),
        ("FINDS_ANCHOR_HI", theme::ORGANIC_FINDS_ANCHOR_HI),
        ("FINDS_COMPANION_LO", theme::ORGANIC_FINDS_COMPANION_LO),
        ("FINDS_COMPANION_HI", theme::ORGANIC_FINDS_COMPANION_HI),
        ("FINDS_ACCENT_HI", theme::ORGANIC_FINDS_ACCENT_HI),
        ("FINDS_DROPOUT", theme::ORGANIC_FINDS_DROPOUT),
    ] {
        assert_eq!(
            declared(name),
            value,
            "shaders/background.wgsl declares a different `{name}` — the host mirror in \
             theme::ground has drifted out of lockstep with the field it describes"
        );
    }
    assert!(
        src.contains("let s = max(g.params.x, select(32.0, FINDS_MIN_SCALE_PX, finds));"),
        "the arrangement's own cell floor is no longer enforced by the shader"
    );
}
