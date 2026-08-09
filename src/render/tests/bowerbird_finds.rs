//! ITEM 176 — the ORGANIC ground's crisp COLLECTED-TREASURE arrangement,
//! proved from real rendered pixels. It arrived as one arm of a theme-owned
//! `Arrangement` dial and is now the ground's only behaviour: the dial, its
//! scalar and the rounded cut-paper arm it chose between all went once no
//! world reached for the other one.
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

use super::bands_waves::{bg_desc_for, headless_dq, render_bg};
use crate::theme;

/// A `Background::Organic` at an explicit cell scale, on Bowerbird's own
/// authored tones and density — the direct-injection seam, so every claim
/// below is about the MECHANISM rather than one world's literal.
/// `pub(super)`: item 191's spacing/void laws (`bowerbird_spacing.rs`)
/// reuse this and the field reader below rather than duplicating them —
/// same-behavior-same-code.
pub(super) fn organic_bg(scale_px: f32) -> theme::Background {
    match theme::BOWERBIRD.background {
        theme::Background::Organic { tones, density, .. } => theme::Background::Organic {
            tones,
            scale_px,
            density,
        },
        _ => panic!("Bowerbird must ship Background::Organic"),
    }
}

// --- The field reader -------------------------------------------------------

/// Open ground, the two object tones, and everything between them (a boundary
/// pixel inside the shader's sub-pixel feather).
pub(super) const GROUND: u8 = 0;
const EDGE: u8 = 3;

/// The three plateau colours of a rendered field, DISCOVERED by frequency
/// rather than recomputed here: the shader mixes in linear space and the sRGB
/// target re-encodes on write, so a host-side copy of that arithmetic would be
/// a second implementation of the thing under test. Returned darkest-first, so
/// index 0 is the open ground.
pub(super) fn plateau_tones(pixels: &[[u8; 4]]) -> [[u8; 4]; 3] {
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

/// Sum of absolute per-channel (R/G/B) differences.
fn sad(a: [u8; 4], b: [u8; 4]) -> i32 {
    (0..3).map(|k| (a[k] as i32 - b[k] as i32).abs()).sum()
}

/// A pixel counts as GROUND if it lands within this many [`sad`] levels of
/// the field's single most frequent colour. TIGHT on purpose: the open
/// ground is a flat authored value with no per-pixel variation of its own,
/// so a genuine ground pixel sits at distance 0 and only ITS OWN
/// antialiased fringe (blending toward some piece's ink) sits slightly off
/// it — loosening this would swallow the very first step of a genuine
/// ground-to-ink transition as "still ground".
const GROUND_TOLERANCE: i32 = 1;

/// Within ONE connected ink region, a pixel counts as that region's own
/// major/minor role if it lands within this many [`sad`] levels of
/// whichever of the region's own two most frequent colours it is nearer.
/// TIGHT for the same reason as [`GROUND_TOLERANCE`]: within a single flat
/// SDF-filled shape the fill is perfectly uniform (`organic_finds_rgb`'s
/// `d_b` is a per-CELL scalar, not per-pixel), so only device/quantization
/// jitter needs covering here — a real antialiased transition moves through
/// most of the tone gap in very few pixels (this arrangement's `FINDS_EDGE_
/// AA_PX` is a crisp 0.75px sub-pixel feather) and must still read as EDGE.
const LOCAL_ROLE_TOLERANCE: i32 = 1;

/// Label every pixel `GROUND`, `1`/`2`, or `EDGE` — the general replacement
/// for the former `label_field(pixels, plateau_tones(pixels))` pair, which
/// classified against ONE GLOBAL pair of object tones. The companion's own
/// per-cell value breathe (`organic_finds_rgb`'s `d_b`) means there is no
/// longer one shared pair: each collection draws its OWN anchor/companion
/// ink, cell-seeded. This reads each connected ink region's own two most
/// frequent colours instead — a general fix, not a loosened tolerance: a GLOBAL bound
/// wide enough to absorb the breathe's own visible swing would, at this
/// arrangement's deliberately crisp antialiasing, also be wide enough to
/// swallow a genuine short transition (measured: it did, at a tolerance of
/// 12 — 92% of boundary crossings lost their intermediate pixel). Reading
/// the plateau LOCALLY, per region, needs no such tradeoff: a region's own
/// fill is exact regardless of what any OTHER region's breathe is doing.
fn local_ink_labels(pixels: &[[u8; 4]], w: u32, h: u32) -> Vec<u8> {
    let (wi, hi) = (w as usize, h as usize);
    let ground_tone = plateau_tones(pixels)[0];
    let ink_mask: Vec<bool> = pixels
        .iter()
        .map(|p| sad(*p, ground_tone) > GROUND_TOLERANCE)
        .collect();
    let (id, count) = components(&ink_mask, wi, hi);

    let mut hist: Vec<std::collections::HashMap<[u8; 4], usize>> =
        vec![std::collections::HashMap::new(); count];
    for (i, &c) in id.iter().enumerate() {
        if c != usize::MAX {
            *hist[c].entry(pixels[i]).or_default() += 1;
        }
    }
    let absent1 = [1u8, 1, 1, 0];
    let absent2 = [2u8, 2, 2, 0];
    let plateaus: Vec<[[u8; 4]; 2]> = hist
        .iter()
        .map(|hmap| {
            let mut top: Vec<([u8; 4], usize)> = hmap.iter().map(|(&c, &n)| (c, n)).collect();
            top.sort_by_key(|(c, n)| (std::cmp::Reverse(*n), *c));
            [
                top.first().map_or(absent1, |(c, _)| *c),
                top.get(1).map_or(absent2, |(c, _)| *c),
            ]
        })
        .collect();

    let mut labels = vec![GROUND; pixels.len()];
    for (i, &c) in id.iter().enumerate() {
        let Some(&[p1, p2]) = (c != usize::MAX).then(|| &plateaus[c]) else {
            continue;
        };
        let (d1, d2) = (sad(pixels[i], p1), sad(pixels[i], p2));
        labels[i] = if d1 <= LOCAL_ROLE_TOLERANCE && d1 <= d2 {
            1
        } else if d2 <= LOCAL_ROLE_TOLERANCE {
            2
        } else {
            EDGE
        };
    }
    labels
}

/// One rendered COLLECTION: the connected run of ink one cell drew, with the
/// pixel area of each piece. `major`/`minor` are the two object-tone regions
/// ordered by size (the tones themselves swap cluster to cluster), `cutout` the
/// open-ground region their ink encloses.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Collection {
    pub(super) major_px: usize,
    pub(super) minor_px: usize,
    pub(super) cutout_px: usize,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) cx: f32,
    pub(super) cy: f32,
}

/// A real second piece: comfortably under the smallest companion the authored
/// ranges can draw at the smallest defended cell (radius `COMPANION_LO *
/// ANCHOR_LO * MIN_SCALE_PX`), so the floor gates a MISSING piece, not a small
/// one.
const MIN_MINOR_PX: usize = 90;
/// A real cut-out. Derived at the arrangement's own cell FLOOR, where the
/// smallest authored cut-out (`ACCENT_LO * ANCHOR_LO * MIN_SCALE_PX` ~ 2.9px
/// radius) keeps only a ~2px solid core once its antialiased skirt is taken
/// off — so this gates a MISSING hole, not a small one. At this file's own
/// 156px reference cell (item 176's own scale; item 191 later opened
/// Bowerbird's shipped `scale_px` to 195, read dynamically by
/// `bowerbird_spacing.rs` rather than duplicated here) the same
/// cut-outs measure five to ten times this.
const MIN_CUTOUT_PX: usize = 8;
/// A real hierarchy. The authored companion is at most `COMPANION_HI` of the
/// anchor radius, so even before the companion's own overlap is subtracted the
/// visible areas differ by `1 / COMPANION_HI^2 - 1` ~ 2.2x; this floor sits
/// well under that and well ABOVE 1.0, so two pieces of a similar size fail.
const HIERARCHY_RATIO: f64 = 1.8;

impl Collection {
    pub(super) fn has_three_roles(&self) -> bool {
        self.minor_px >= MIN_MINOR_PX
            && self.cutout_px >= MIN_CUTOUT_PX
            && (self.major_px as f64) >= HIERARCHY_RATIO * self.minor_px as f64
            && self.minor_px > 2 * self.cutout_px
    }

    pub(super) fn describe(&self) -> String {
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
pub(super) fn read_collections(pixels: &[[u8; 4]], w: u32, h: u32) -> Vec<Collection> {
    let (wi, hi) = (w as usize, h as usize);
    let labels = local_ink_labels(pixels, w, h);
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
/// A `FIELD_W x FIELD_H` canvas at this file's 156px reference cell holds ~89
/// cells, ~10% of which are deliberately empty and ~40% of which touch the
/// border and are dropped as cropped. Anything near or under this floor means
/// the reader found almost nothing and every per-collection assertion below
/// went vacuous.
const MIN_COLLECTIONS: usize = 25;

// The field no longer translates at all, so this reader has no phase left
// to inject (its every caller already passed a literal `0.0`).
fn finds_field(device: &wgpu::Device, queue: &wgpu::Queue, scale: f32) -> Vec<[u8; 4]> {
    let bg = organic_bg(scale);
    render_bg(
        device,
        queue,
        bg_desc_for(bg),
        FIELD_W,
        FIELD_H,
        0.0,
        0.0,
        0.0,
    )
}

/// LAW: every whole collection the field draws carries all three roles in the
/// authored hierarchy — a dominant piece, a genuinely smaller second piece on
/// the OTHER tone, and a tiny cut-out enclosed by their own ink.
#[test]
fn finds_every_collection_shows_the_three_role_hierarchy() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_every_collection_shows_the_three_role_hierarchy: no adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0);
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
/// every cell scale a future Organic world could author, not only this file's
/// own 156px reference. `32.0` is below the arrangement's own
/// `FINDS_MIN_SCALE_PX` floor and must be CLAMPED up to it rather than
/// aliasing into speckle, so it is swept here deliberately.
#[test]
fn finds_holds_the_three_role_grammar_at_every_reachable_cell_scale() {
    let _g = crate::testlock::serial();
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
        let pixels = finds_field(&device, &queue, scale);
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
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_variation_does_not_repeat_and_the_scatter_is_not_a_grid");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0);
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

// `finds_drift_is_one_rigid_whole_field_translation` is DELETED: the
// field-translation `drift` it proved rigid no longer exists at all —
// `organic_rgb` deletes the `drift` vec2 outright, both terms, so the field
// never translates. Its replacement claim — the field's own silhouette must
// be IDENTICAL at every ambient phase, with only the companion's own VALUE
// free to change — is proved in `bowerbird_breathe.rs`'s
// `bowerbird_organic_field_never_translates_across_the_ambient_clock`.

/// LAW (the worst-phase sweep, re-stated for the new arrangement: now
/// sweeping the companion's own breathe phase rather than the deleted
/// field-translation drift): the field stays inside Bowerbird's cool navy
/// value band and out of the writing column at EVERY point of the ambient
/// cycle — a crisp field with hard edges cannot be allowed to push a
/// worst-phase pixel warm, bright, or onto the page.
#[test]
fn finds_worst_phase_stays_cool_and_off_the_page() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_worst_phase_stays_cool_and_off_the_page");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = organic_bg(156.0);
    let (w, h, left, col) = (900u32, 600u32, 220.0f32, 460.0f32);
    let wrap = crate::lava::LAVA_LOOP_CYCLES;
    for i in 0..24 {
        let phase = wrap * (i as f32) / 24.0;
        let pixels = super::bands_waves::render_bg_ambient(
            &device,
            &queue,
            bg_desc_for(bg),
            w,
            h,
            left,
            col,
            crate::background::AmbientUpload {
                organic_phase: phase,
                ..Default::default()
            },
            1.0,
        );
        for (idx, p) in pixels.iter().enumerate() {
            let x = (idx as u32) % w;
            if (x as f32) >= left && (x as f32) < left + col {
                assert_eq!(
                    [p[0], p[1], p[2]],
                    [0, 0, 0],
                    "phase {phase}: collected ink entered the page column at x={x}"
                );
                continue;
            }
            assert!(
                p[2] >= p[0] && p[0] < 90,
                "phase {phase}: warm/bright margin pixel {p:?} — the ground must stay cool"
            );
        }
    }
}

/// LAW: `density: 0.0` collapses the arrangement to the flat open ground
/// EXACTLY — the differential oracle every pixel law here measures against, and
/// the property that keeps a future world's `density` dial honest.
#[test]
fn finds_density_zero_is_exactly_the_flat_ground() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_density_zero_is_exactly_the_flat_ground");
        return;
    };
    let _g = crate::testlock::serial();
    let mut flat = bg_desc_for(organic_bg(156.0));
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
        bg_desc_for(organic_bg(156.0)),
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
/// toward the wall-to-wall coverage a soft blob field draws (which is exactly
/// what reads as camouflage), nor thin out into an isolated constellation.
#[test]
fn finds_leaves_a_generous_open_ground() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_leaves_a_generous_open_ground");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = finds_field(&device, &queue, 156.0);
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
    let labels = local_ink_labels(pixels, w, h);
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
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_edges_stay_antialiased_and_crisp_at_1x_and_2x");
        return;
    };
    let _g = crate::testlock::serial();
    for (density, scale, w, h) in [(1u32, 156.0f32, 1200u32, 800u32), (2, 312.0, 2400, 1600)] {
        let bg = organic_bg(scale);
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

/// LAW: the ambient gates belong to the GROUND, and the gate is read off the
/// ground's own identity rather than off any dial it carries — which is what
/// keeps the whole freeze truth table already pinned by
/// `theme::tests::bowerbird_organic_schedules_zero_frames_under_every_freeze_
/// condition` (Reduce Motion, `ambient_motion = false`, focus lost, paused) and
/// the real-pixel Reduce Motion proof in `bowerbird_breathe` applicable
/// to the field at any authored cell. This law used to sweep the two
/// arrangements to make that point; with one arrangement left it sweeps the
/// authored SCALE instead, which is the dial the ground still has — a revival
/// that quietly armed a second clock, or that dropped out of the shared one,
/// still shows up right here.
#[test]
fn the_organic_ground_rides_the_one_shared_ambient_gate() {
    for scale_px in [96.0, 156.0, 195.0, 320.0] {
        let mut world = theme::BOWERBIRD;
        world.background = organic_bg(scale_px);
        assert!(
            world.background.is_organic(),
            "{scale_px}: the ground must stay Organic to the renderer"
        );
        assert!(
            world.has_ambient_tick(),
            "{scale_px}: the ground must stay enrolled in the ONE shared ambient tick"
        );
        assert!(
            !world.has_ambient_motion(),
            "{scale_px}: the ground must not claim a lava world's motion budget"
        );
    }
}

/// LAW: every ground's `profile` slot carries exactly its own theme-owned dial
/// (Deckle's `Weave` is the only one left), or the inert `0.0` for a ground
/// with none — checked at the packing seam every ground shares, never at one
/// world's literal. Organic is now in the second class: its arrangement dial
/// collapsed to a single arm and went, so the slot it used to ride must read
/// INERT for every organic world. That is asserted here rather than assumed,
/// because a leftover non-zero would reach the shader as a mode selector on a
/// ground that no longer has modes.
#[test]
fn the_profile_slot_carries_only_a_grounds_own_dial() {
    for t in theme::THEMES {
        let desc = bg_desc_for(t.background);
        assert_eq!(
            desc.profile,
            t.background.profile_mode(),
            "{}: the profile slot must carry this ground's own dial",
            t.name
        );
        if t.background.is_organic() {
            assert_eq!(
                desc.profile, 0.0,
                "{}: Organic has no profile dial any more — its slot must be inert",
                t.name
            );
        }
    }
    for scale_px in [96.0, 156.0, 320.0] {
        let desc = bg_desc_for(organic_bg(scale_px));
        assert_eq!(
            desc.profile, 0.0,
            "{scale_px}: the organic upload must carry no arrangement scalar"
        );
    }
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
        src.contains("let s = max(g.params.x, FINDS_MIN_SCALE_PX);"),
        "the field's own cell floor is no longer enforced by the shader"
    );
    // The arrangement selector is GONE, not merely unused: a revived
    // `params.z` branch inside `organic_rgb` would silently give this ground a
    // second mode again, and the host packs `0.0` into that slot now.
    assert!(
        !src.contains("FINDS_MIN_SCALE_PX, finds"),
        "the retired arrangement selector is back in `organic_rgb`"
    );
}
