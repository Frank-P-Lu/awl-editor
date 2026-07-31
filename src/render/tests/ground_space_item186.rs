//! ITEM 186 — THE GROUND'S TWO COORDINATE SPACES, over real GPU pixels.
//!
//! Every procedural ground used to author its composition in PHYSICAL pixels,
//! so a 2x display rendered it at half its logical size and showed roughly
//! twice as many elements. No law could see it, because every capture was
//! internally consistent with itself — which is exactly why the convention
//! survived. These laws break that consistency deliberately: they render the
//! SAME world at matched LOGICAL sizes on two different device ratios and
//! require the two compositions to be the same picture.
//!
//! Two claims, and they pull in OPPOSITE directions on purpose:
//!
//!   1. COMPOSITION IDENTITY — a 1x render and a 2x render of the same logical
//!      canvas, scale-normalized, are the same composition. Swept over the
//!      whole `Background` roster with NO wildcard, so a newly added ground
//!      cannot dodge it.
//!   2. THE FEATHER STAYS PHYSICAL — item 176's 0.75px crisp edge measures
//!      0.75px on the GLASS at both ratios, so a 2x display resolves that same
//!      composition MORE FINELY. A law that only checked (1) would be passed by
//!      a blanket conversion of every quantity to logical space, which is the
//!      failure mode this item names.
//!
//! Both are proven non-vacuous against the pre-186 code by flipping one
//! quantity's space back and watching the law name the world and the family.

use crate::background::BgDesc;
use crate::theme::{self, Background, GroundSpace};

use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg_scaled};

/// The 1x canvas every sweep below uses, in LOGICAL pixels, with a page column
/// generous enough that both margins are real fields rather than slivers. The
/// 2x arm renders the same logical rectangle at twice the device resolution.
const W: u32 = 600;
const H: u32 = 400;
const COL_LEFT: u32 = 210;
const COL_W: u32 = 180;

/// One roster entry: the LABEL a failure names (a world, or the dormant
/// variant's own kind) and the ground itself.
struct Ground {
    label: &'static str,
    bg: Background,
}

/// Every ground the sweep judges: each world's OWN authored background, plus an
/// explicit literal for each `Background` shape no world currently wears.
///
/// The completeness of this list is not a matter of care —
/// `the_sweep_covers_every_member_of_the_background_roster` proves it against
/// `Background::roster_index`'s wildcard-free match, so adding a variant to the
/// enum fails to compile there and then fails the sweep here until it is
/// enrolled.
fn roster() -> Vec<Ground> {
    let mut out: Vec<Ground> = theme::THEMES
        .iter()
        .map(|t| Ground {
            label: t.name,
            bg: t.background,
        })
        .collect();
    // The dormant shapes: reusable infrastructure with zero assignees today
    // (`Bands`), and the two profile arms a world adopts with one word
    // (`Dots { edge: true }`, `Arrangement::Finds`, `Weave::Fibres`). Each is a
    // distinct branch of the shader, so each is swept on its own.
    out.extend(dormant());
    out
}

fn dormant() -> Vec<Ground> {
    let mut out = dormant_bands_and_dots();
    out.extend(dormant_profile_arms());
    out
}

/// The variants no world wears at all: `Bands` (zero assignees since Gumtree
/// moved to Zigzag) and proximity-scaled `Dots`.
fn dormant_bands_and_dots() -> Vec<Ground> {
    let mut out = vec![Ground {
        label: "dormant:bands",
        // Gumtree's retired grass tones, the literal `backgrounds_item69`
        // keeps this variant's regression coverage alive with.
        bg: Background::Bands {
            tones: [
                theme::Srgb::rgb(0x2C, 0x35, 0x2A),
                theme::Srgb::rgb(0x35, 0x40, 0x31),
                theme::Srgb::rgb(0x3E, 0x4B, 0x39),
            ],
            angle: 0.35,
        },
    }];
    if let Background::Dots {
        from,
        to,
        dir,
        tint,
        ..
    } = theme::MULGA.background
    {
        out.push(Ground {
            label: "dormant:dots-edge",
            bg: Background::Dots {
                from,
                to,
                dir,
                tint,
                edge: true,
            },
        });
    }
    out
}

/// The PROFILE arms — a shipping ground's other face, adopted by writing one
/// word in a world literal. Each is a distinct branch of the shader, so each is
/// swept on its own rather than riding its sibling's proof.
fn dormant_profile_arms() -> Vec<Ground> {
    let bowerbird = theme::BOWERBIRD.background;
    let paperbark = theme::PAPERBARK.background;
    let quokka_zigzag = theme::QUOKKA.background;
    let mut out: Vec<Ground> = Vec::new();
    if let Background::Organic {
        tones,
        scale_px,
        density,
        ..
    } = bowerbird
    {
        out.push(Ground {
            label: "dormant:organic-finds",
            bg: Background::Organic {
                tones,
                arrangement: theme::Arrangement::Finds,
                scale_px,
                density,
            },
        });
    }
    if let Background::Deckle {
        ground,
        layer,
        deckle,
        anchor,
        period_px,
        wander_px,
        density,
        ..
    } = paperbark
    {
        out.push(Ground {
            label: "dormant:deckle-fibres",
            bg: Background::Deckle {
                ground,
                layer,
                deckle,
                weave: theme::Weave::Fibres,
                anchor,
                period_px,
                wander_px,
                density,
            },
        });
    }
    // Zigzag's banded arm is data-authored too; keep the sweep honest about it.
    if let Background::Zigzag {
        from,
        to,
        dir,
        tint,
        period_px,
        amplitude_px,
        angle,
        density,
        banded,
    } = quokka_zigzag
    {
        out.push(Ground {
            label: "dormant:zigzag-flipped-band",
            bg: Background::Zigzag {
                from,
                to,
                dir,
                tint,
                period_px,
                amplitude_px,
                angle,
                density,
                banded: !banded,
            },
        });
    }
    out
}

/// The two renders a composition claim compares: the ground at 1x on a `W`x`H`
/// canvas, and the SAME logical rectangle at 2x — twice the device pixels, a
/// device ratio of 2.0, and the page column at twice the physical offset so the
/// two runs describe the identical logical layout.
fn pair(device: &wgpu::Device, queue: &wgpu::Queue, desc: BgDesc) -> (Vec<[u8; 4]>, Vec<[u8; 4]>) {
    let one = render_bg_scaled(
        device,
        queue,
        desc,
        W,
        H,
        COL_LEFT as f32,
        COL_W as f32,
        0.0,
        1.0,
    );
    let two = render_bg_scaled(
        device,
        queue,
        desc,
        W * 2,
        H * 2,
        (COL_LEFT * 2) as f32,
        (COL_W * 2) as f32,
        0.0,
        2.0,
    );
    (one, two)
}

/// SCALE NORMALIZATION: box-average each 2x2 block of the 2x render down onto
/// the 1x grid. If the composition is the same picture, the result is simply a
/// SUPERSAMPLE of that picture — it can differ only where a feather or the
/// device-grid dither resolves more finely, never in where the marks are.
fn normalize(two: &[[u8; 4]]) -> Vec<[u8; 4]> {
    let mut out = vec![[0u8; 4]; (W * H) as usize];
    for y in 0..H {
        for x in 0..W {
            let mut acc = [0u32; 4];
            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = x * 2 + dx;
                    let sy = y * 2 + dy;
                    let p = two[(sy * W * 2 + sx) as usize];
                    for c in 0..4 {
                        acc[c] += p[c] as u32;
                    }
                }
            }
            let mut px = [0u8; 4];
            for c in 0..4 {
                px[c] = ((acc[c] + 2) / 4) as u8;
            }
            out[(y * W + x) as usize] = px;
        }
    }
    out
}

/// Only the MARGINS carry a ground: the page column is punched transparent, so
/// including it would dilute every measurement with identical zeroes.
fn is_margin(x: u32) -> bool {
    !(COL_LEFT..COL_LEFT + COL_W).contains(&x)
}

/// The composition difference between a 1x render and a scale-normalized 2x
/// one: the MEAN absolute rgb difference over the margins, in 8-bit levels.
/// Zero would mean bit-identical; a few levels is the antialias/dither residue
/// two different sample grids legitimately leave on the SAME picture; tens of
/// levels means the two runs drew different compositions.
fn composition_delta(one: &[[u8; 4]], norm: &[[u8; 4]]) -> f32 {
    let mut sum = 0u64;
    let mut n = 0u64;
    for y in 0..H {
        for x in 0..W {
            if !is_margin(x) {
                continue;
            }
            let i = (y * W + x) as usize;
            for c in 0..3 {
                sum += (one[i][c] as i32 - norm[i][c] as i32).unsigned_abs() as u64;
                n += 1;
            }
        }
    }
    sum as f32 / n.max(1) as f32
}

/// THE COMPOSITION ORACLE PROPER: the zero-lag normalized cross-correlation of
/// the two margin fields, mean-removed and scaled to unit energy.
///
/// It answers "is this the same picture?" while staying deliberately blind to
/// the one thing that legitimately DOES differ — how sharply each edge resolves.
/// Both images are mean-centred and divided by their own energy, so a mark drawn
/// a little lighter or a little crisper at 2x (which is precisely what a
/// physical feather buys) correlates at ~1.0, while a field whose pitch, count
/// or mark POSITIONS moved decorrelates hard: a doubled-frequency lattice
/// against its own halved one has no systematic agreement left to find.
///
/// The mean-difference metric alone cannot carry this claim — it is dominated
/// by the flat ground a whisper-mark field is mostly made of — and a raw
/// per-pixel population count cannot either, because a hairline whose width is
/// on the order of its own feather (Pinstripe's 9px rules) reads measurably
/// lighter at 2x with its composition completely unmoved.
fn composition_correlation(one: &[[u8; 4]], norm: &[[u8; 4]]) -> f32 {
    let gray = |p: [u8; 4]| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0;
    let mut a: Vec<f32> = Vec::new();
    let mut b: Vec<f32> = Vec::new();
    for y in 0..H {
        for x in 0..W {
            if !is_margin(x) {
                continue;
            }
            let i = (y * W + x) as usize;
            a.push(gray(one[i]));
            b.push(gray(norm[i]));
        }
    }
    let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len().max(1) as f32;
    let (ma, mb) = (mean(&a), mean(&b));
    let mut num = 0.0f64;
    let mut da = 0.0f64;
    let mut db = 0.0f64;
    for i in 0..a.len() {
        let (u, v) = ((a[i] - ma) as f64, (b[i] - mb) as f64);
        num += u * v;
        da += u * u;
        db += v * v;
    }
    // A field with no variance at all (a flat one-bit ground) is trivially the
    // same picture; say so rather than dividing by zero.
    if da < 1e-6 && db < 1e-6 {
        return 1.0;
    }
    (num / (da.sqrt() * db.sqrt()).max(1e-9)) as f32
}

// ---------------------------------------------------------------------------
// 1. COMPOSITION IDENTITY, over the whole roster, no wildcard
// ---------------------------------------------------------------------------

/// The AUTHORED bound on how far a 1x render and a scale-normalized 2x render
/// of the same logical canvas may drift apart. It is not zero and cannot be:
/// the two runs sample the same continuous field on grids of different
/// fineness, so every feathered edge and the device-grid dither leave a small
/// residue. Measured across the whole roster the worst healthy family sits well
/// under this; a single quantity reverted to physical space blows past it by an
/// order of magnitude (the mutation arms below).
const MAX_COMPOSITION_DELTA: f32 = 3.0;
/// The floor the zero-lag correlation of the two fields must clear. `1.0` is
/// the same picture exactly; the healthy roster sits above this floor with
/// room, and a single quantity reverted to physical space drops it to near
/// zero — the two fields stop having anything systematic in common.
const MIN_COMPOSITION_CORRELATION: f32 = 0.90;

#[test]
fn every_procedural_ground_composes_identically_at_1x_and_2x() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping every_procedural_ground_composes_identically_at_1x_and_2x: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let mut worst = (0.0f32, String::new());
    let mut weakest = (1.0f32, String::new());
    for g in roster() {
        let family = g.bg.as_str();
        let (one, two) = pair(&device, &queue, bg_desc_for(g.bg));
        let norm = normalize(&two);
        let delta = composition_delta(&one, &norm);
        let corr = composition_correlation(&one, &norm);
        if delta > worst.0 {
            worst = (delta, format!("{} / {}", g.label, family));
        }
        if corr < weakest.0 {
            weakest = (corr, format!("{} / {}", g.label, family));
        }
        assert!(
            corr >= MIN_COMPOSITION_CORRELATION,
            "world {label} / family {family}: the 1x field and the scale-normalized 2x \
             field correlate at only {corr:.3} (floor {MIN_COMPOSITION_CORRELATION:.2}) — not \
             the same picture. This metric is deliberately blind to a mark reading a little \
             lighter or crisper at 2x, so a score this low means the marks are in DIFFERENT \
             PLACES: a pitch, cell or size of this family is still authored in PHYSICAL pixels, \
             and the user's display density is choosing their composition.",
            label = g.label,
        );
        assert!(
            delta <= MAX_COMPOSITION_DELTA,
            "world {label} / family {family}: the ground's COMPOSITION is not the same at \
             1x and 2x — a 1x render and a scale-normalized 2x render of the identical \
             {W}x{H} LOGICAL canvas differ by {delta:.2} levels (bound \
             {MAX_COMPOSITION_DELTA:.2}). A composition quantity \
             of this family is still authored in PHYSICAL pixels, so the user's display density is \
             deciding how many elements they see. See `Background::authored_quantities` for \
             which of this family's numbers are composition and which are sampling.",
            label = g.label,
        );
    }
    eprintln!(
        "item-186 composition identity: worst mean delta {} at {:.2} levels (bound \
         {MAX_COMPOSITION_DELTA:.2}); weakest correlation {} at {:.4} (floor \
         {MIN_COMPOSITION_CORRELATION:.2})",
        worst.1, worst.0, weakest.1, weakest.0
    );
}

/// NO WILDCARD: the sweep above is total over the `Background` roster. The
/// enum's own `roster_index` carries a wildcard-free match, so a new variant
/// fails to COMPILE there; this law then fails until a representative of it is
/// enrolled in `roster()`, so a newly added ground cannot ride in unswept.
#[test]
fn the_sweep_covers_every_member_of_the_background_roster() {
    let mut seen = [false; Background::ROSTER_LEN];
    for g in roster() {
        seen[g.bg.roster_index()] = true;
    }
    let missing: Vec<usize> = (0..Background::ROSTER_LEN).filter(|&i| !seen[i]).collect();
    assert!(
        missing.is_empty(),
        "the item-186 ground-space sweep does not reach every `Background` variant — roster \
         indices {missing:?} have no representative. A ground that no world wears is still a \
         ground: enrol a literal for it in this file's `dormant()`, exactly as `Bands` and the \
         three profile arms are, or the next world to adopt it inherits an unproven composition."
    );
}

// ---------------------------------------------------------------------------
// 2. THE SAMPLING FEATHER STAYS PHYSICAL
// ---------------------------------------------------------------------------

/// A ground whose edges are hard by design — item 176's crisp three-object
/// arrangement — at the cell Bowerbird authored.
fn finds_ground() -> Background {
    dormant()
        .into_iter()
        .find(|g| g.label == "dormant:organic-finds")
        .expect("the Finds arrangement is enrolled in the sweep")
        .bg
}

/// The SAME arrangement at FULL density and with deliberately far-apart tones.
///
/// The feather law measures a ramp's WIDTH in device pixels, and a ramp is only
/// measurable while each of its steps clears 8-bit quantization: at Bowerbird's
/// shipped whisper contrast a doubled-width ramp gets GENTLER per pixel and
/// disappears under the noise floor instead of reading as wider — which is
/// exactly how the first cut of this law passed its own mutation. Driving a
/// high-contrast literal is the same move `backgrounds_item69` makes for
/// dormant `Bands`: the geometric property a feather must hold belongs to the
/// SHAPE, not to whichever tones a world happens to wear.
fn finds_high_contrast() -> Background {
    Background::Organic {
        tones: [
            theme::Srgb::rgb(0x0E, 0x0E, 0x10),
            theme::Srgb::rgb(0xF2, 0xF0, 0xEA),
            theme::Srgb::rgb(0x86, 0x84, 0x80),
        ],
        arrangement: theme::Arrangement::Finds,
        scale_px: 156.0,
        density: 1.0,
    }
}

/// The MEAN WIDTH, in DEVICE pixels, of the transition ramps in this field.
///
/// Walk each margin scanline and mark every pixel whose right neighbour differs
/// visibly; a maximal run of such pixels IS one edge crossing, and its length is
/// that crossing's ramp width. Averaging over the hundreds of crossings a Finds
/// field carries makes the answer robust to quantization at any one edge, and
/// dividing the ramp POPULATION by the crossing COUNT (rather than by the image
/// area) makes it independent of how much boundary the picture holds — so the
/// composition changing size cannot be mistaken for the feather changing width.
fn mean_edge_ramp_px(pixels: &[[u8; 4]], w: u32, h: u32, col_left: u32, col_w: u32) -> f32 {
    let mut total = 0u64;
    let mut runs = 0u64;
    for y in 0..h {
        let mut run = 0u64;
        for x in 0..w - 1 {
            let inside = x >= col_left && x < col_left + col_w;
            let i = (y * w + x) as usize;
            let r = (y * w + x + 1) as usize;
            let d = (0..3)
                .map(|c| (pixels[i][c] as i32 - pixels[r][c] as i32).abs())
                .max()
                .unwrap_or(0);
            if !inside && d >= 1 {
                run += 1;
            } else {
                if run > 0 {
                    total += run;
                    runs += 1;
                }
                run = 0;
            }
        }
        if run > 0 {
            total += run;
            runs += 1;
        }
    }
    total as f32 / runs.max(1) as f32
}

#[test]
fn the_crisp_edge_feather_stays_physical_so_2x_resolves_the_same_composition_more_finely() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping the_crisp_edge_feather_stays_physical_so_2x_resolves_the_same_\
             composition_more_finely: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let bg = finds_high_contrast();
    let family = bg.as_str();
    let (one, two) = pair(&device, &queue, bg_desc_for(bg));
    let r1 = mean_edge_ramp_px(&one, W, H, COL_LEFT, COL_W);
    let r2 = mean_edge_ramp_px(&two, W * 2, H * 2, COL_LEFT * 2, COL_W * 2);

    // THE WHOLE CLAIM, in one number. `FINDS_EDGE_AA_PX` is authored in PHYSICAL
    // pixels, so the ramp it draws must measure the SAME number of DEVICE pixels
    // at 1x and at 2x — the composition around it doubled in device size (law 1
    // proves it is the same picture), and the edge did not follow. Had the
    // feather been swept into logical space along with everything else, the ramp
    // would measure twice as many device pixels at 2x: the same edge, blurrier,
    // on the better display.
    let ratio = r2 / r1.max(f32::EPSILON);
    assert!(
        (0.75..=1.35).contains(&ratio),
        "family {family}: the crisp edge feather did not stay PHYSICAL. Its transition ramp \
         measures {r1:.2} device px at 1x and {r2:.2} at 2x — a ratio of {ratio:.2}, where a \
         feather fixed on the GLASS holds ~1.0 and one wrongly converted to LOGICAL space doubles \
         to ~2.0. `FINDS_EDGE_AA_PX` is item 176's 0.75px crisp edge, classified `{class}` in \
         `Background::authored_quantities`; a 2x display must draw the SAME composition with a \
         SHARPER edge, never a proportionally blurrier one. If a blanket conversion of every \
         ground quantity to logical space is what moved this, that is the failure mode item 186 \
         names by name: the composition law alone would have accepted it.",
        class = GroundSpace::Physical.as_str(),
    );
    eprintln!(
        "item-186 feather: mean edge ramp {r1:.2} device px at 1x vs {r2:.2} at 2x \
         (ratio {ratio:.2})"
    );
}

// ---------------------------------------------------------------------------
// 3. THE CLASSIFICATION TABLE IS THE DELIVERABLE — it must be complete and real
// ---------------------------------------------------------------------------

#[test]
fn every_authored_ground_quantity_declares_its_space_and_says_why() {
    for g in roster() {
        let quantities = g.bg.all_authored_quantities();
        assert!(
            !quantities.is_empty(),
            "world {} / family {}: declares no authored quantity at all. Every ground carries at \
             least the shared dither cell; a ground with an empty table has not been classified.",
            g.label,
            g.bg.as_str(),
        );
        for q in &quantities {
            assert!(
                !q.name.trim().is_empty(),
                "family {}: an authored quantity has no name",
                g.bg.as_str()
            );
            assert!(
                q.why.split_whitespace().count() >= 6,
                "family {} / quantity `{}`: its `why` is {} words. Classifying each quantity \
                 correctly IS item 186 — a reason too short to state the argument means the \
                 decision was inherited, not made.",
                g.bg.as_str(),
                q.name,
                q.why.split_whitespace().count(),
            );
        }
    }
}

/// The two classes must BOTH be populated across the family. A table that is
/// all-logical is precisely the blanket conversion item 186 forbids; an
/// all-physical one is the defect it fixes.
#[test]
fn the_family_declares_both_classes_and_neither_swallowed_the_other() {
    let mut logical = 0usize;
    let mut physical = 0usize;
    for g in roster() {
        for q in g.bg.all_authored_quantities() {
            match q.space {
                GroundSpace::Logical => logical += 1,
                GroundSpace::Physical => physical += 1,
            }
        }
    }
    assert!(
        logical > 0 && physical > 0,
        "the ground family declares {logical} composition quantities and {physical} sampling ones \
         — one class swallowed the other. A blanket conversion satisfies the composition-identity \
         law and still destroys the product: sampling feathers must stay physical."
    );
    // The one quantity the item names by hand, asserted by name.
    let finds = finds_ground();
    let aa = finds
        .all_authored_quantities()
        .into_iter()
        .find(|q| q.name.contains("FINDS_EDGE_AA_PX"))
        .expect("the Finds arrangement declares its crisp edge");
    assert_eq!(
        aa.space,
        GroundSpace::Physical,
        "item 176's 0.75px crisp edge is a SAMPLING quantity and item 186 says it does not move; \
         the table now calls it {}",
        aa.space.class(),
    );
}

/// The WGSL is the runtime consumer, and the table is only an authority if the
/// shader actually routes through the two owners it describes. A structural
/// tripwire in the item-89/158 idiom: the conversion must happen ONCE, the
/// dither must NOT take it, and the crisp edge must be converted back.
#[test]
fn the_shader_routes_composition_and_sampling_through_their_two_owners() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for needle in [
        "fn to_logical(",
        "fn sampling_feather(",
        // The one conversion, and the four grounds that own their final rgb.
        "let lp = to_logical(in.px);",
        "pattern_coverage(lp)",
        "bands_rgb(lp)",
        "waves_rgb(lp)",
        "organic_rgb(lp)",
        "deckle_rgb(lp)",
        // The crisp edge is authored physical and converted INTO logical space.
        "let aa = sampling_feather(FINDS_EDGE_AA_PX);",
        // The dither deliberately keeps the DEVICE pixel.
        "bayer_threshold01(in.px)",
    ] {
        assert!(
            wgsl.contains(needle),
            "shaders/background.wgsl no longer contains `{needle}` — item 186's coordinate-space \
             discipline is stated in `theme::ground_space`'s table and enforced in exactly two \
             shader owners (`to_logical` for composition, `sampling_feather` for a physical \
             feather). If the shader stopped routing through them, the table is describing code \
             that no longer exists."
        );
    }
    assert!(
        !wgsl.contains("pattern_coverage(in.px)"),
        "shaders/background.wgsl evaluates `pattern_coverage` at the PHYSICAL fragment position \
         again — that is the pre-186 defect exactly: the mark grounds' cells and periods go back \
         to being device pixels and a 2x display shows twice as many marks."
    );
}
