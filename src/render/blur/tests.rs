//! `blur.rs`'s own unit laws, carved out to a sibling file so the module stays
//! under its production size ratchet (`scripts/code-health.toml` exempts a
//! `tests.rs`). Every test's NAME is unchanged; only which file it lives in moved.

use super::extent::*;

#[test]
fn doc_capture_cap_is_a_noop_at_or_below_the_cap() {
    // A normal surface, a 2× retina surface, and exactly the cap all pass through
    // UNCHANGED — so the capture (and thus the blurred backdrop) is byte-identical.
    assert_eq!(capped_doc_size(1200, 800, DOWNSAMPLE), (1200, 800));
    assert_eq!(
        capped_doc_size(2400, 1600, downsample_for(2.0)),
        (2400, 1600)
    );
    assert_eq!(
        capped_doc_size(DOC_CAPTURE_MAX, 1000, DOWNSAMPLE),
        (DOC_CAPTURE_MAX, 1000)
    );
    assert_eq!(
        capped_doc_size(1000, DOC_CAPTURE_MAX, DOWNSAMPLE),
        (1000, DOC_CAPTURE_MAX)
    );
}

#[test]
fn doc_capture_cap_scales_a_genuinely_large_surface_and_preserves_aspect() {
    // A 5K surface: the longest side is clamped to the cap, the short side scaled
    // by the same factor (aspect preserved), and the result stays at least the
    // quarter-res blur working size so the downsample is still a downsample.
    let (cw, ch) = capped_doc_size(5120, 2880, DOWNSAMPLE);
    assert_eq!(cw, DOC_CAPTURE_MAX);
    let scale = DOC_CAPTURE_MAX as f32 / 5120.0;
    assert_eq!(ch, (2880.0 * scale).round() as u32);
    assert!(cw >= 5120 / DOWNSAMPLE && ch >= 2880 / DOWNSAMPLE);
    // Portrait orientation clamps on height instead.
    let (pw, ph) = capped_doc_size(2880, 5120, DOWNSAMPLE);
    assert_eq!(ph, DOC_CAPTURE_MAX);
    assert_eq!(pw, (2880.0 * scale).round() as u32);
}

/// THE FROST'S REACH IS AUTHORED IN LOGICAL PX AND MULTIPLIED BY DPI ONCE.
///
/// The Gaussian's reach is a fixed count of quarter-res texels, so the reach in
/// PHYSICAL px is `taps × downsample` and the reach a reader perceives is that
/// over `dpi`. A fixed downsample therefore halves the perceived defocus at 2× —
/// the exact class of defect a capture cannot see, because every capture runs at
/// `--capture-dpi 1`. This law sweeps the DPI axis and requires the LOGICAL reach
/// to be constant, which is what a fixed downsample fails.
#[test]
fn the_frosts_logical_reach_is_constant_across_dpi() {
    // 1× is the historical value exactly — so every capture, and every 1× frame,
    // is byte-identical to before the DPI scaling existed.
    assert_eq!(
        downsample_for(1.0),
        DOWNSAMPLE,
        "1x must return the authored constant untouched (capture byte-identity)"
    );
    for dpi in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
        let ds = downsample_for(dpi);
        // The Gaussian's ±4-tap reach, in LOGICAL px.
        let logical_reach = 4.0 * ds as f32 / dpi;
        let authored = 4.0 * DOWNSAMPLE as f32;
        assert!(
            (logical_reach - authored).abs() <= 1.0,
            "dpi {dpi}: the frost reaches {logical_reach:.2} logical px, \
                 authored {authored:.2} — a reach that changes with DPI is a \
                 device-pixel length"
        );
    }
    // Degenerate DPI can never produce a zero factor (a division by it follows).
    for dpi in [0.0f32, -1.0, f32::NAN] {
        assert!(downsample_for(dpi) >= 1, "dpi {dpi} must floor at 1");
    }
}

/// THE FOOTPRINT SCISSOR: outward rounding, a clamp, and an empty answer for a
/// footprint that lands off the target.
///
/// Outward rounding is the load-bearing half. A card box lands on fractional
/// physical px at any non-integer scale (`Metrics::px` multiplies a logical pad by
/// the scale), and rounding IN would leave a sliver of sharp document along the
/// card's own edge — a one-pixel version of the defect the frost exists to remove.
#[test]
fn the_footprint_scissor_rounds_outward_clamps_and_rejects_the_off_canvas() {
    // A fractional box grows to cover every pixel it touches.
    assert_eq!(
        scissor_px([10.4, 20.6, 100.3, 50.1], 1200, 800),
        Some((10, 20, 101, 51)),
        "near edges floor, far edges ceil — the frost covers the whole box"
    );
    // An integral box is exact — no free growth.
    assert_eq!(
        scissor_px([10.0, 20.0, 100.0, 50.0], 1200, 800),
        Some((10, 20, 100, 50))
    );
    // A 2x card box (the same logical box at scale 2) stays a doubled rect: the
    // rect arrives PHYSICAL, so this fn applies no scale of its own.
    assert_eq!(
        scissor_px([20.0, 40.0, 200.0, 100.0], 2400, 1600),
        Some((20, 40, 200, 100))
    );
    // Clamped to the target on both ends, never past it (wgpu validates this).
    assert_eq!(
        scissor_px([-30.0, -10.0, 200.0, 100.0], 1200, 800),
        Some((0, 0, 170, 90))
    );
    let (x, y, w, h) = scissor_px([1100.0, 700.0, 400.0, 400.0], 1200, 800).unwrap();
    assert!(
        x + w <= 1200 && y + h <= 800,
        "the scissor stays inside the target: {x},{y} {w}x{h}"
    );
    // Entirely off the target, degenerate, or non-finite: no scissor, and the
    // caller must draw NOTHING rather than fall back to the fullscreen triangle.
    assert_eq!(scissor_px([1300.0, 20.0, 100.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([10.0, 20.0, 0.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([f32::NAN, 20.0, 100.0, 50.0], 1200, 800), None);
    assert_eq!(scissor_px([10.0, 20.0, 100.0, 50.0], 0, 0), None);
}

/// The FOOTPRINT arm dims by nothing and the FULL arm keeps its authored recede —
/// the two extents carry their own dim, so a hue claim inside a footprint is a
/// claim about the blur alone.
#[test]
fn the_footprint_arm_carries_no_dim_and_the_full_arm_keeps_its_own() {
    assert_eq!(upright([0.0, 0.0, 10.0, 10.0]).dim(), 0.0);
    assert_eq!(Frost::Full.dim(), DIM);
    assert!(
        Frost::Full.dim() > 0.0,
        "the full takeover still recedes a value"
    );
}

/// A footprint over an upright composition — the shape a `Rules` world takes.
fn upright(rect: [f32; 4]) -> Frost {
    Frost::Footprint(Footprint { rect, shear: 0.0 })
}

/// THE FEATHER MUST CLEAR THE BLUR IT EDGES.
///
/// This is the floor under [`FOOTPRINT_FEATHER_PX`], and the reason the number is not
/// simply the smallest value that stops a knife edge. The interior the boundary bounds
/// is soft over the Gaussian's own reach; a boundary narrower than that reads HARD
/// beside it, because the eye grades the edge against the blur it terminates. So the
/// authored width may be retuned by taste UPWARD freely and downward only to here.
///
/// `lava::FROST_FEATHER_PX` (7.0) is the tree's other frost-skirt quantity and the
/// closer kind; this law is what says it cannot serve, rather than a comment claiming so.
#[test]
fn the_footprint_feather_is_at_least_the_blur_it_edges() {
    // The Gaussian's ±4-tap reach in LOGICAL px, at the authored 1× downsample.
    let reach = 4.0 * DOWNSAMPLE as f32;
    assert!(
        FOOTPRINT_FEATHER_PX >= reach,
        "the footprint feather is {FOOTPRINT_FEATHER_PX} logical px against a blur that \
         reaches {reach} — an edge narrower than the softness it bounds reads hard"
    );
    assert!(
        crate::lava::FROST_FEATHER_PX < reach,
        "lava's frost skirt ({}) has risen above the blur's reach ({reach}) — it is now \
         a candidate for this quantity and the choice to author separately should be \
         re-argued rather than left standing on a stale figure",
        crate::lava::FROST_FEATHER_PX
    );
    // And it is a LOGICAL length: multiplied by DPI exactly once, so the edge a reader
    // sees is the same width at 1× and 2×. The class every `--capture-dpi 1` is blind to.
    assert_eq!(footprint_feather_px(1.0), FOOTPRINT_FEATHER_PX);
    for dpi in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
        let logical = footprint_feather_px(dpi) / dpi;
        assert!(
            (logical - FOOTPRINT_FEATHER_PX).abs() <= 1e-3,
            "dpi {dpi}: the feather is {logical} logical px, authored {FOOTPRINT_FEATHER_PX}"
        );
    }
    for dpi in [0.0f32, -1.0, f32::NAN] {
        assert!(
            footprint_feather_px(dpi) > 0.0,
            "dpi {dpi}: a degenerate scale must not produce a zero feather — that is the \
             hard edge back again"
        );
    }
}

/// THE MASK IS 1 ON AND INSIDE THE SHAPE, AND RAMPS TO 0 OUTSIDE IT — NOT A STEP.
///
/// The defect this replaced was hard BY CONSTRUCTION (a scissor answers yes or no), so
/// the property that matters is the ramp's existence, its DIRECTION (entirely outside
/// the faces, so nothing the card covers stops being frosted) and its monotonicity.
///
/// Both halves of the roster are graded: an upright footprint and a sheared one.
#[test]
fn the_footprint_mask_is_full_inside_and_ramps_outward_over_the_feather() {
    let f = FOOTPRINT_FEATHER_PX;
    for shear in [0.0f32, 0.35, -0.35] {
        let foot = Footprint {
            rect: [200.0, 100.0, 400.0, 300.0],
            shear,
        };
        let [x, y, w, h] = foot.rect;
        let (cx, cy) = (x + w * 0.5, y + h * 0.5);
        let m = |px: f32, py: f32| footprint_mask(foot, f, px, py);

        // THE CENTRE IS FULL STRENGTH, exactly — the frost's own presence floor at the
        // arithmetic seam. A mask that faded its whole subject satisfies "no hard edge"
        // perfectly, and this is what refuses it.
        assert_eq!(
            m(cx, cy),
            1.0,
            "shear {shear}: the footprint's centre is not fully frosted"
        );
        // Every face's INNER side is full strength (the ramp is entirely outside), and
        // the ramp is spent by the feather's width beyond it. Asked on the horizontal
        // faces, which the shear leaves alone, and on the vertical ones along the
        // centre row, where the shear's displacement is zero.
        for (px, py) in [(cx, y), (cx, y + h), (x, cy), (x + w, cy)] {
            assert_eq!(
                m(px, py),
                1.0,
                "shear {shear}: the face at ({px}, {py}) is not fully frosted — a ramp \
                 that reaches INWARD leaves partly-sharp document under the card's own \
                 outermost rows"
            );
        }
        for (px, py) in [
            (cx, y - f - 1.0),
            (cx, y + h + f + 1.0),
            (x - f - 1.0, cy),
            (x + w + f + 1.0, cy),
        ] {
            assert_eq!(
                m(px, py),
                0.0,
                "shear {shear}: the mask still reaches ({px}, {py}), a feather and a \
                 pixel beyond its own face"
            );
        }
        // MONOTONE across the boundary, and genuinely GRADED: the half-strength
        // crossing sits inside the feather rather than at the face.
        let profile: Vec<f32> = (0..=((f * 2.0) as i32))
            .map(|d| m(cx, y - d as f32))
            .collect();
        for pair in profile.windows(2) {
            assert!(
                pair[1] <= pair[0] + 1e-6,
                "shear {shear}: the mask is not monotone outward: {profile:?}"
            );
        }
        let half = profile
            .iter()
            .position(|v| *v <= 0.5)
            .expect("the ramp reaches half strength within two feathers");
        assert!(
            (0.2 * f) as usize <= half && half <= (0.8 * f) as usize,
            "shear {shear}: the mask crosses half strength {half} px out of a {f} px \
             feather — a step crosses at 0 or 1"
        );
    }
}

/// THE SILHOUETTE IS A PARALLELOGRAM — MEASURED AS ONE, NOT ASSERTED AS ONE.
///
/// The user's call is the shape: the frost under a leaning list must READ as a
/// parallelogram. "Does it lean" does NOT distinguish one from a rectangle, and that is
/// the trap this law is written around — the retired shape leaned too. It was the box
/// UNIONED with the sheared box, so it contained the whole rectangle and the shear could
/// only add two overhang ears to it. Two figures separate the two shapes, and this law
/// asserts both:
///
/// 1. **BOTH faces translate, TOGETHER.** At row `py` the frosted span's left edge and
///    its right edge each sit exactly `shear × (py − cy)` from where they sit on the
///    centre row, so the span's WIDTH is constant and its POSITION is affine in `py` with
///    slope `shear`. A rectangle's two edges do not move at all. The union's moved one at
///    a time — its left edge on the half the rake reached left and its right edge on the
///    other half — so its width GREW away from the centre row, and that is the figure
///    below that fails on it.
/// 2. **The frosted area is strictly SMALLER than its own bounding box**, by the two
///    triangular corners the rake leaves behind: `|shear| × h²` between them. For a
///    rectangle the two areas are equal, and for the union the shortfall was zero because
///    the box filled the corners in.
///
/// A **PRESENCE FLOOR** runs beside both, because "the silhouette is not a rectangle" is
/// satisfied perfectly by frosting nothing — the same satisfied-by-deleting-its-subject
/// trap item 312's own feather floor exists for. So the interior is required FULLY
/// frosted, and the frosted area is required to be the box's own `w × h` rather than
/// merely non-zero: a shape that kept its slope while shrinking to a sliver would pass
/// both figures above.
#[test]
fn the_footprints_silhouette_is_a_parallelogram_and_not_a_leaning_rectangle() {
    let f = FOOTPRINT_FEATHER_PX;
    let rect = [200.0, 100.0, 400.0, 300.0];
    let [x, y, w, h] = rect;
    let (cx, cy) = (x + w * 0.5, y + h * 0.5);
    // A real shear, its mirror, and a steep one. `0.0` is deliberately NOT here: an
    // upright composition's parallelogram IS its rectangle, so it cannot grade a claim
    // about the difference. `the_footprint_mask_is_full_inside_and_ramps_outward_over_the_
    // feather` is where the upright arm is graded.
    for shear in [0.35f32, -0.35, 0.6] {
        let foot = Footprint { rect, shear };
        let m = |px: f32, py: f32| footprint_mask(foot, f, px, py);
        let label = format!("shear {shear}");

        // (1) EVERY ROW'S SPAN, read off the mask rather than predicted: the extreme x's
        // at which the mask is still exactly 1.0, found by scanning the whole bounding
        // box. `footprint_bound` is the shape's own enclosure, so the scan cannot miss a
        // face by starting inside it.
        let [bx, _, bw, _] = footprint_bound(foot, f);
        let span = |py: f32| -> Option<(f32, f32)> {
            let n = 4000;
            let xs = (0..=n).map(|i| bx + bw * i as f32 / n as f32);
            let hit: Vec<f32> = xs.filter(|px| m(*px, py) == 1.0).collect();
            Some((*hit.first()?, *hit.last()?))
        };
        let (l0, r0) = span(cy).expect("the centre row is frosted");
        let step = bw / 4000.0;
        for k in 1..=8 {
            let py = y + h * k as f32 / 9.0;
            let (l, r) = span(py).unwrap_or_else(|| panic!("{label}: row {py} is not frosted"));
            let want = shear * (py - cy);
            // BOTH edges, and the SAME displacement for each. A tolerance of two scan
            // steps: the span is read off a sampled grid, not solved.
            assert!(
                (l - (l0 + want)).abs() <= 2.0 * step,
                "{label}: at row {py} the frosted span's LEFT edge is at {l:.2}, but the \
                 rake puts it at {:.2} ({:+.2} from the centre row's {l0:.2}). A left edge \
                 that does not translate is a rectangle's",
                l0 + want,
                want
            );
            assert!(
                (r - (r0 + want)).abs() <= 2.0 * step,
                "{label}: at row {py} the frosted span's RIGHT edge is at {r:.2}, but the \
                 rake puts it at {:.2}. THIS is the figure the retired box-union shape \
                 failed: it moved one face at a time, so its span WIDENED away from the \
                 centre row instead of translating",
                r0 + want
            );
        }

        // (2) AREA vs its own BOUNDING BOX. Counted on a grid over the bound, so the
        // ratio is a property of the shape rather than of an authored formula.
        let (n, mut inside, mut total) = (600, 0u64, 0u64);
        let [bx, by, bw, bh] = footprint_bound(foot, f);
        for iy in 0..n {
            for ix in 0..n {
                let px = bx + bw * (ix as f32 + 0.5) / n as f32;
                let py = by + bh * (iy as f32 + 0.5) / n as f32;
                total += 1;
                if m(px, py) == 1.0 {
                    inside += 1;
                }
            }
        }
        let cell = (bw / n as f32) * (bh / n as f32);
        let area = inside as f32 * cell;
        let bbox = (w + (shear * h).abs()) * h;
        assert!(
            area < bbox - 0.5 * (shear * h * h).abs(),
            "{label}: the frosted area {area:.0} is not strictly smaller than the {bbox:.0} \
             of its own bounding box — the two triangular corners the rake leaves behind \
             are FILLED, which is what a rectangle (and the retired box-union) does"
        );
        // …and the PRESENCE FLOOR on the same count: the shortfall is the two triangles
        // and NOTHING MORE, so a shape that held its slope while fading or shrinking
        // fails here rather than passing the two figures above.
        assert!(
            (area - w * h).abs() <= 0.02 * w * h,
            "{label}: the frosted area is {area:.0} against the {:.0} the card's own box \
             asks for ({} of {total} grid cells) — a silhouette can satisfy every shape \
             claim above by covering almost nothing, and this is the floor that refuses it",
            w * h,
            inside
        );
        // The interior, sampled on its own grid in the shape's OWN frame, is full
        // strength — the frost the card sits on, not merely a correct outline.
        for iy in 1..20 {
            for ix in 1..20 {
                let py = y + h * iy as f32 / 20.0;
                let px = x + w * ix as f32 / 20.0 + shear * (py - cy);
                assert_eq!(
                    m(px, py),
                    1.0,
                    "{label}: ({px}, {py}) is inside the parallelogram and is not fully \
                     frosted"
                );
            }
        }
    }
    // A degenerate shear is inert rather than catastrophic.
    for bad in [f32::NAN, f32::INFINITY] {
        let foot = Footprint { rect, shear: bad };
        assert_eq!(footprint_mask(foot, f, cx, cy), 1.0);
        assert!(footprint_bound(foot, f).iter().all(|v| v.is_finite()));
    }
}

/// THE COVERAGE FLOOR, WHICH DID NOT GO AWAY — IT STOPPED BEING A SECOND SHAPE.
///
/// The retired union frosted the card's whole box because the card's HEAD band is upright
/// and flush to its text edge while the rows rake away from it, and a shape that only
/// followed the rake left that band over sharp document. The duty is the same; its owner
/// is now [`footprint_box`], which WIDENS the rect until the parallelogram contains the
/// band. Widening a parallelogram leaves a parallelogram, so the silhouette pays nothing.
///
/// Three claims. It grows the box enough (the whole chrome box is inside the resulting
/// shape, at every corner); it grows it no more than it must; and — the one that carries
/// every upright world's byte-identity — chrome ALREADY inside grows the box by NOTHING,
/// returned bit for bit.
#[test]
fn the_footprint_box_widens_until_the_parallelogram_contains_the_cards_upright_chrome() {
    let f = FOOTPRINT_FEATHER_PX;
    let card = [200.0f32, 100.0, 400.0, 300.0];
    let [x, y, w, h] = card;
    // The head band as the two rake directions present it: a narrow band at the card's
    // text edge, high in the card. On one sign the rake carries the shape TOWARD it and
    // on the other AWAY, which is the whole asymmetry — so both must be swept or the law
    // grades one world.
    let chrome = [x + 12.0, y + 12.0, x + 40.0, y + 39.0];
    for shear in [0.0f32, 0.35, -0.35, 0.6] {
        let grown = footprint_box(card, shear, Some(chrome));
        let foot = Footprint { rect: grown, shear };
        let label = format!("shear {shear}");
        // It never SHRINKS, on either face, whatever the chrome asks.
        assert!(
            grown[0] <= x + 1e-3 && grown[0] + grown[2] >= x + w - 1e-3,
            "{label}: {grown:?} does not still contain the card's own box {card:?}"
        );
        assert_eq!(
            [grown[1], grown[3]],
            [y, h],
            "{label}: the box grew VERTICALLY"
        );
        // COVERAGE: every corner of the chrome box is fully frosted.
        for (px, py) in [
            (chrome[0], chrome[1]),
            (chrome[2], chrome[1]),
            (chrome[0], chrome[3]),
            (chrome[2], chrome[3]),
        ] {
            assert_eq!(
                footprint_mask(foot, f, px, py),
                1.0,
                "{label}: the card's upright chrome corner ({px}, {py}) is not fully \
                 frosted by the parallelogram {grown:?} — this is the reported defect \
                 moved onto the card's own chrome, which is what the retired union \
                 existed to prevent"
            );
        }
        // TIGHTNESS: one px narrower on the face that grew and the coverage is gone, so
        // the growth is the deficit rather than a pad.
        let grew_left = grown[0] < x - 1e-3;
        let grew_right = grown[0] + grown[2] > x + w + 1e-3;
        if grew_left || grew_right {
            let tight = Footprint {
                rect: if grew_left {
                    [grown[0] + 1.0, y, grown[2] - 1.0, h]
                } else {
                    [grown[0], y, grown[2] - 1.0, h]
                },
                shear,
            };
            let worst = [
                (chrome[0], chrome[1]),
                (chrome[2], chrome[1]),
                (chrome[0], chrome[3]),
                (chrome[2], chrome[3]),
            ]
            .iter()
            .map(|&(px, py)| footprint_mask(tight, f, px, py))
            .fold(1.0f32, f32::min);
            assert!(
                worst < 1.0,
                "{label}: the box is a whole px wider than the chrome needs — {grown:?} \
                 still covers it with 1px shaved off, so this is a pad and not a deficit"
            );
        }
    }
    // BYTE-IDENTITY, and it is the claim every upright world rests on: no chrome at all,
    // and chrome already inside the shape, each return the card's own rect bit for bit.
    let inside = [x + w * 0.4, y + h * 0.45, x + w * 0.6, y + h * 0.55];
    for shear in [0.0f32, 0.35, -0.35] {
        assert_eq!(
            footprint_box(card, shear, None),
            card,
            "shear {shear}: a card with no upright chrome must keep its own rect"
        );
        assert_eq!(
            footprint_box(card, shear, Some(inside)),
            card,
            "shear {shear}: chrome already inside the parallelogram must grow the box by \
             NOTHING — every upright world's byte-identity is this equality"
        );
    }
    // A degenerate shear or a degenerate chrome box is inert rather than catastrophic.
    for bad in [f32::NAN, f32::INFINITY] {
        assert_eq!(footprint_box(card, bad, Some(chrome)), card);
        assert_eq!(footprint_box(card, 0.35, Some([bad, bad, bad, bad])), card);
    }
}

/// THE SCISSOR'S BOUND ENCLOSES EVERY PIXEL THE MASK REACHES.
///
/// The composite is still scissored — that is what keeps the page beyond the frost
/// byte-identical on every backend instead of resting on an sRGB blend round-trip
/// against a zero alpha. Its correctness condition is exactly this: the mask must be
/// ZERO on and outside [`footprint_bound`], or the scissor clips the skirt and the
/// knife edge is back at the bound's own edge.
#[test]
fn the_footprint_bound_encloses_the_whole_feathered_shape() {
    let f = FOOTPRINT_FEATHER_PX;
    for shear in [0.0f32, 0.35, -0.35, 0.9] {
        let foot = Footprint {
            rect: [200.0, 100.0, 400.0, 300.0],
            shear,
        };
        let [bx, by, bw, bh] = footprint_bound(foot, f);
        // Sample a frame ON the bound and a band beyond it: nothing may be frosted.
        for i in 0..=60 {
            let t = i as f32 / 60.0;
            for (px, py) in [
                (bx + bw * t, by),
                (bx + bw * t, by + bh),
                (bx, by + bh * t),
                (bx + bw, by + bh * t),
            ] {
                assert_eq!(
                    footprint_mask(foot, f, px, py),
                    0.0,
                    "shear {shear}: the mask reaches ({px}, {py}), which is ON the \
                     scissor bound {:?} — the scissor would clip the skirt into a hard \
                     edge at its own boundary",
                    [bx, by, bw, bh]
                );
            }
        }
        // …and the bound is not absurdly loose: it is the box plus the shear's own
        // displacement plus the feather, and no more. The displacement compounds with
        // the feather (the skirt is sheared too), which is the arithmetic this law
        // corrected on its first run.
        let g = (shear * (foot.rect[3] * 0.5 + f)).abs();
        assert!(
            (bw - (foot.rect[2] + 2.0 * (g + f))).abs() <= 1e-3,
            "shear {shear}: the bound's width is not the shape's own reach"
        );
    }
}

/// THE COMPOSITE'S UNIFORM CARRIES THE WHOLE EXTENT, AND `Frost::Full` CARRIES NONE.
///
/// The full-takeover arm's mask flag is OFF, which is what makes its alpha exactly 1.0
/// in the shader and its blended composite a replace. The footprint arm carries the
/// box, the shear and the DPI-resolved feather, and its dim is zero — so the two arms
/// cannot arrive at the shader half-configured.
#[test]
fn the_composite_uniform_carries_the_extent_and_the_full_arm_carries_none() {
    let base = [0.1f32, 0.2, 0.3];
    let full = U::comp(base, Frost::Full, 2.0);
    assert_eq!(full.foot, [0.0; 4], "the full arm masks nothing");
    assert_eq!(
        full.mask, [0.0; 4],
        "the full arm's mask flag must be OFF — that is what makes its alpha exactly \
         1.0 and its blended composite a bit-for-bit replace"
    );
    assert_eq!(full.tint, [0.1, 0.2, 0.3, DIM]);

    let foot = Footprint {
        rect: [10.0, 20.0, 300.0, 400.0],
        shear: 0.4,
    };
    let fp = U::comp(base, Frost::Footprint(foot), 2.0);
    assert_eq!(fp.foot, foot.rect);
    assert_eq!(fp.mask[0], 0.4, "the shear reaches the shader");
    assert_eq!(
        fp.mask[1],
        footprint_feather_px(2.0),
        "the feather is resolved at the ONE dpi boundary, not re-derived downstream"
    );
    assert_eq!(fp.mask[2], 1.0, "the footprint arm's mask flag is on");
    assert_eq!(fp.tint[3], FOOTPRINT_DIM);
    // The downsample / Gaussian passes read only their step; nothing else is live for
    // them, so nothing else may be non-zero and quietly become live.
    let p = U::pass([1.0, 2.0, 0.0, 0.0]);
    assert_eq!((p.tint, p.foot, p.mask), ([0.0; 4], [0.0; 4], [0.0; 4]));
    // The uniform's SIZE must stay a multiple of 16 (a WGSL uniform struct's alignment)
    // and must match the four vec4s `shaders/blur.wgsl` declares.
    assert_eq!(core::mem::size_of::<U>(), 64);
}

/// ENROLMENT, DERIVED FROM THE ROSTER. Every world's own list composition decides
/// whether a crisp picker over it frosts its footprint — nothing here names a
/// world, and the answer follows a world that changes its list style.
///
/// Non-vacuity is asserted in both directions: the enrolled set and the excluded
/// set are both non-empty, and both are NAMED in the failure message. A predicate
/// that quietly stopped matching anything (the shape that swept nothing for the
/// life of a law once already) fails here rather than passing green.
#[test]
fn footprint_enrolment_follows_the_rosters_own_backing_owners() {
    use crate::theme::{ListBacking, ListStyle};
    let mut enrolled: Vec<&str> = Vec::new();
    let mut excluded: Vec<&str> = Vec::new();
    for t in crate::theme::THEMES.iter() {
        let style = t.render_caps.list_style;
        if footprint_frost_applies(style) {
            enrolled.push(t.name);
            assert!(
                !matches!(style.list_backing(false), ListBacking::Card),
                "{}: enrolled but its card is a filled panel",
                t.name
            );
            assert!(
                !style.draws_row_plates(),
                "{}: enrolled but it plates its own rows",
                t.name
            );
        } else {
            excluded.push(t.name);
            assert!(
                matches!(style.list_backing(false), ListBacking::Card) || style.draws_row_plates(),
                "{}: excluded while drawing neither a panel nor plates — \
                     the document shows straight through its rows",
                t.name
            );
        }
    }
    assert!(
        !enrolled.is_empty(),
        "no world enrols — the mechanism has no subject (enrolled={enrolled:?})"
    );
    assert!(
        !excluded.is_empty(),
        "every world enrols — the byte-identical arm has no subject \
         (excluded={excluded:?})"
    );
    // The style axis itself, exhaustively: one member per shape of backing, so a
    // new `ListStyle` cannot slip past with an unconsidered answer.
    assert!(footprint_frost_applies(ListStyle::Rules(
        crate::theme::RuleSelection::Weight
    )));
    assert!(footprint_frost_applies(ListStyle::Diagonal(
        crate::theme::DiagonalSpine::descending(crate::theme::DiagonalMark::CRISP)
    )));
    assert!(
        !footprint_frost_applies(ListStyle::Bars),
        "Bars plates its rows: the plate is the frost's job already"
    );
    assert!(
        !footprint_frost_applies(ListStyle::Pane),
        "Pane's panel covers its whole footprint"
    );
}
