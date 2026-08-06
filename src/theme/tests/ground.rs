use super::super::*;

/// WHICH WORLDS WEAR WHICH GROUND — the assignee roster, pinned as exact lists
/// in `THEMES` order.
///
/// Separate from `every_world_has_a_valid_background`, which asks whether a
/// world's ground is well-formed; this asks who wears what. Exact lists rather
/// than counts, because the failure worth catching is a ground quietly
/// SPREADING across worlds until the roster reads as one idea recoloured — and
/// a count cannot tell "Bands moved to a different world" from "a second world
/// adopted Bands".
#[test]
fn every_ground_has_exactly_the_assignees_it_is_meant_to() {
    let _lock = crate::testlock::serial();
    // Stripes stays Potoroo's alone.
    let stripes: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.background, Background::Stripes { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(stripes, ["Potoroo"], "Stripes is Potoroo's alone");
    // Waves stays Bombora's alone (reusable DATA, but only one world
    // currently picks it).
    let waves: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.background, Background::Waves { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(waves, ["Bombora"], "Waves is Bombora's alone");
    // Zigzag ships on EXACTLY Quokka and Gumtree, in THEMES order.
    let zigzag: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.background, Background::Zigzag { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(
        zigzag,
        ["Gumtree", "Quokka"],
        "Zigzag ships on Gumtree and Quokka alone"
    );
    // Bands is Magpie's alone. It spent a while unworn after its first
    // assignee moved to Zigzag; the roster-completeness assertion above is now
    // what forbids that state, and this pins WHICH world took it.
    let bands: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.background, Background::Bands { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(bands, ["Magpie"], "Bands is Magpie's alone");
    // Pinstripe is the roster's most-worn ground and dropped by one when
    // Magpie took Bands. Pinned as an exact list for the same reason Zigzag's
    // pair is: a ground quietly spreading across worlds is how a roster loses
    // its variety, and Pinstripe is the one with the room to spread.
    let pinstripe: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.background, Background::Pinstripe { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(
        pinstripe,
        ["Saltpan", "Mulga", "Cassowary"],
        "Pinstripe ships on exactly these worlds, in THEMES order"
    );
    // PROXIMITY-SCALED Dots (`edge: true`) rode Mangrove alone, and Mangrove
    // folded into a lava ground (2026-07), so no world carries proximity Dots
    // now — the `edge: bool` machinery is intact but currently unassigned (like
    // `Background::Lava` was before this round). Not a bug: a feature may ship
    // with zero worlds until one wants it.
    let edge_dots: Vec<&str> = THEMES
        .iter()
        .filter(|t| t.background.edge())
        .map(|t| t.name)
        .collect();
    assert!(
        edge_dots.is_empty(),
        "proximity Dots is unassigned since Mangrove became lava, got {edge_dots:?}"
    );
}

/// EVERY MEMBER OF THE `Background` ROSTER IS WORN BY A LIVE WORLD — the ground
/// vocabulary carries no dormant arm.
///
/// Asserted over `roster_index` rather than over a hand-kept list of ground
/// names, which makes it total by construction: `roster_index`'s match has no
/// wildcard, so a new variant fails to COMPILE there, and `ROSTER_LEN` then
/// fails THIS law until some world actually adopts it. A list of names could
/// silently omit one; an array indexed by the enum's own ordinal cannot.
///
/// A ground that loses its last world is therefore a decision, not a drift:
/// adopt it into a world that wants it, or retire it out of `Background` the
/// way the scattered-star ground went. Editing this law to tolerate an unworn
/// arm is the third option, and it wants saying out loud rather than doing
/// quietly — which is the whole reason this is a law and not a grep in a
/// commit message.
#[test]
fn every_ground_in_the_roster_is_worn_by_a_live_world() {
    let _lock = crate::testlock::serial();
    let mut worn = [false; Background::ROSTER_LEN];
    for t in THEMES.iter() {
        worn[t.background.roster_index()] = true;
    }
    let unworn: Vec<usize> = (0..Background::ROSTER_LEN).filter(|&i| !worn[i]).collect();
    assert!(
        unworn.is_empty(),
        "the `Background` roster has unworn arms at index {unworn:?} — every ground must be \
         some world's ground. Adopt it, or retire it out of the enum; do not leave it dormant."
    );
}

/// Every world declares a [`Background`] ground whose gradient endpoints AND
/// mark/band tint are OPAQUE (the shader owns the coverage, so the colors
/// themselves stay fully opaque). The shader id stays within the known range.
#[test]
fn every_world_has_a_valid_background() {
    for t in THEMES.iter() {
        let bg = t.background;
        assert_eq!(
            bg.from().a,
            0xFF,
            "{} background from must be opaque",
            t.name
        );
        assert_eq!(bg.to().a, 0xFF, "{} background to must be opaque", t.name);
        assert_eq!(
            bg.tint().a,
            0xFF,
            "{} background tint must be opaque",
            t.name
        );
        // 0..=4 the static grounds (Lava also degrades to 0 for this
        // base-margin pass), 5=Bands, 6=Waves, 7=Zigzag, 8=Organic,
        // 9=Deckle, 10=WarpedGrid. 2 is vacant and stays that way — see
        // `Background::shader_id`.
        assert!(bg.shader_id() <= 10, "{} bad shader id", t.name);
    }
    // WAVES PALETTE LAW (Bombora's alone — Gumtree's own Zigzag carries its
    // own, separately-checked, palette law below): `tones`
    // is exactly `[base_100, base_200, base_300]`, no separately-tuned tint,
    // and the three rungs are pairwise distinct (a real tone-on-tone field,
    // not a flat repeat).
    match BOMBORA.background {
        Background::Waves { tones } => {
            assert_eq!(
                tones,
                [BOMBORA.base_100, BOMBORA.base_200, BOMBORA.base_300],
                "Bombora's Waves tones must be exactly its own ground ladder"
            );
            assert_ne!(tones[0], tones[1]);
            assert_ne!(tones[1], tones[2]);
            assert_ne!(tones[0], tones[2]);
        }
        _ => panic!("Bombora must ship Background::Waves"),
    }
    // ZIGZAG PALETTE LAW: Gumtree's Zigzag uses ONLY its own ground ladder —
    // `from`/`to`/`tint` are exactly its `base_100`/`base_200`/`base_300`.
    match GUMTREE.background {
        Background::Zigzag { from, to, tint, .. } => {
            assert_eq!(
                from, GUMTREE.base_100,
                "Gumtree's Zigzag `from` must be its own base_100"
            );
            assert_eq!(
                to, GUMTREE.base_200,
                "Gumtree's Zigzag `to` must be its own base_200"
            );
            assert_eq!(
                tint, GUMTREE.base_300,
                "Gumtree's Zigzag `tint` must be its own base_300"
            );
        }
        _ => panic!("Gumtree must ship Background::Zigzag"),
    }
    // ZIGZAG DISTINCTNESS LAW: Quokka and Gumtree's Zigzag fields must NOT
    // read as a recolor of one asset — every one of the four authored dials
    // (scale/spacing, profile, direction, contrast) differs, and Gumtree's is
    // the "broader and quieter" of the pair.
    match (QUOKKA.background, GUMTREE.background) {
        (
            Background::Zigzag {
                period_px: qp,
                amplitude_px: qa,
                angle: qang,
                density: qd,
                ..
            },
            Background::Zigzag {
                period_px: gp,
                amplitude_px: ga,
                angle: gang,
                density: gd,
                ..
            },
        ) => {
            assert_ne!(
                qp, gp,
                "period_px (scale/spacing) must differ between Quokka and Gumtree"
            );
            assert_ne!(
                qa, ga,
                "amplitude_px (profile) must differ between Quokka and Gumtree"
            );
            assert_ne!(
                qang, gang,
                "angle (direction) must differ between Quokka and Gumtree"
            );
            assert_ne!(
                qd, gd,
                "density (contrast) must differ between Quokka and Gumtree"
            );
            assert!(
                gp > qp,
                "Gumtree's period must be BROADER (larger) than Quokka's"
            );
            assert!(
                gd < qd,
                "Gumtree's density must be QUIETER (lower) than Quokka's"
            );
        }
        _ => unreachable!("both Quokka and Gumtree must ship Background::Zigzag"),
    }
}

/// MULGA'S GROUND-LADDER RESTRAINT — the same shape Gumtree's `Zigzag` and
/// Bombora's `Waves` palette laws hold, and the reason Mulga's ground cannot
/// drift loud again. `from`/`to`/`tint` are EXACTLY its own `base_100`/
/// `base_200`/`base_300`, with no separately-authored mark tint, so the
/// brightest ink the margin can reach is the top of this world's own ground
/// ramp and the rules recede by construction.
///
/// The ground this replaced failed precisely here: its mark tint was authored
/// OUTSIDE the ladder and well past `base_300`, which is what let a sparse
/// field of lit points out-shine the page they framed — the user's own verdict
/// on the room. Restoring that tint is what this law is mutation-proved
/// against, so the regression it names is the one that actually happened.
#[test]
fn mulga_ground_stays_on_its_own_ladder() {
    let _lock = crate::testlock::serial();
    match MULGA.background {
        Background::Pinstripe {
            from,
            to,
            dir,
            tint,
        } => {
            assert_eq!(
                from, MULGA.base_100,
                "Mulga's Pinstripe `from` must be its own base_100"
            );
            assert_eq!(
                to, MULGA.base_200,
                "Mulga's Pinstripe `to` must be its own base_200"
            );
            assert_eq!(
                tint, MULGA.base_300,
                "Mulga's Pinstripe `tint` must be its own base_300"
            );
            assert_eq!(dir, (0.0, 1.0), "Mulga's margin gradient runs downward");
        }
        _ => panic!("Mulga must ship Background::Pinstripe"),
    }
}

/// MAGPIE'S GROUND-LADDER RESTRAINT — the same shape Mulga's `Pinstripe`,
/// Gumtree's `Zigzag` and Bombora's `Waves` palette laws hold. `Bands` computes
/// its FINAL rgb from its three tones, with no gradient underneath and no
/// low-coverage mark to dilute them: whatever is authored here is literally
/// what half the margin is painted. That makes it the ground with the least
/// margin for a tone chosen by eye, so its three tones are pinned to EXACTLY
/// this world's own `base_100`/`base_200`/`base_300`.
///
/// Bounding loudness structurally rather than by taste is the point: the
/// brightest and darkest a Bands margin can reach are both rungs of the ramp
/// the page itself is built from, so the ground cannot out-shine or out-weigh
/// the page at any window size or aspect. A tone authored past `base_300` is
/// exactly how a previous ground in this roster got loud enough for the user to
/// object to the room, and it is what this law is mutation-proved against.
///
/// The three rungs must also be pairwise distinct — three bands painted in two
/// tones is a two-band field with a seam in it.
#[test]
fn magpie_ground_stays_on_its_own_ladder() {
    let _lock = crate::testlock::serial();
    match MAGPIE.background {
        Background::Bands { tones, angle } => {
            assert_eq!(
                tones,
                [MAGPIE.base_100, MAGPIE.base_200, MAGPIE.base_300],
                "Magpie's Bands tones must be exactly its own ground ladder"
            );
            assert_ne!(tones[0], tones[1]);
            assert_ne!(tones[1], tones[2]);
            assert_ne!(tones[0], tones[2]);
            // A COMMITTED diagonal. The band boundaries run perpendicular to
            // `angle`, so an angle near 0 or near PI/2 renders bands that are
            // nearly vertical or nearly horizontal — which reads as a level
            // field someone failed to level, not as a rake. Held a clear
            // distance off both axes so the tilt is legible as intent.
            let off_axis = angle.min(std::f32::consts::FRAC_PI_2 - angle);
            assert!(
                off_axis > 0.35,
                "Magpie's band angle {angle} sits {off_axis} rad from an axis — too close to \
                 level to read as a deliberate rake"
            );
        }
        _ => panic!("Magpie must ship Background::Bands"),
    }
}
