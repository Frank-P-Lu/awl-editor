use super::super::*;

/// EXACTLY two worlds ship a `Background::Lava` — Firetail (warm, undithered)
/// and Mangrove (cool deepsea, dithered), both with the Glow edge. Pins the
/// roster + each world's edge/dither config, and that every OTHER world stays
/// a STATIC ground (shader id 0..=10, Bands/Waves included) so the lava layer
/// is dormant there and their captures are unaffected.
#[test]
fn exactly_firetail_and_mangrove_ship_lava() {
    let _lock = crate::testlock::serial();
    let lava: Vec<&str> = THEMES
        .iter()
        .filter(|t| t.background.is_lava())
        .map(|t| t.name)
        .collect();
    assert_eq!(
        lava,
        ["Mangrove", "Firetail"],
        "exactly Mangrove + Firetail are lava worlds"
    );
    for t in THEMES.iter().filter(|t| !t.background.is_lava()) {
        assert!(
            t.background.shader_id() <= 10,
            "{}: a non-lava world stays a non-lava ground",
            t.name
        );
    }
    // Firetail: WARM, undithered; ground == its own base_100 (seamless).
    let f = set_active_by_name("Firetail").unwrap();
    let (fg, _flo, _fhi, fd) = f.background.lava_params().unwrap();
    assert_eq!(
        fg, f.base_100,
        "Firetail lava ground == base_100 (seamless margin↔page)"
    );
    assert!(!fd, "Firetail is the SMOOTH warm lamp (undithered)");
    // Mangrove: COOL deepsea, DITHERED; ground == its own base_100.
    let m = set_active_by_name("Mangrove").unwrap();
    let (mg, _mlo, _mhi, md) = m.background.lava_params().unwrap();
    assert_eq!(
        mg, m.base_100,
        "Mangrove lava ground == base_100 (seamless margin↔page)"
    );
    assert!(md, "Mangrove is the DITHERED cool lamp (print-grain)");
    set_active(DEFAULT_THEME);
}

/// THE `Background::Lava` FIGURE/GROUND LAW (Firetail + Mangrove): the ANIMATED
/// metaball margins must READ AS GROUND at EVERY phase — never brightening into
/// "figure" territory that would compete with the flat page column the text sits
/// on, and always leaving the ink a strong contrast to sit against. Asserted over
/// composited PIXELS (the pure-Rust shader mirror in `crate::lava` + each world's
/// own blob colors + color arithmetic), NOT over sidecar state — the Wagtail-
/// invisible-picker-row lesson: appearance is proven over the bytes, never inferred.
#[test]
fn lava_worlds_keep_figure_ground_at_the_worst_animation_phase() {
    // Gamma-correct Rec.709 relative luminance (the `render::tests::syntax_roles`
    // `rel_luminance` recipe), so the "ground value band" is PERCEIVED brightness.
    fn rel_lum(c: Srgb) -> f32 {
        fn lin(u: u8) -> f32 {
            let s = u as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }
    // redmean color distance (the `distinguishability`/`syntax_roles` metric).
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    for t in THEMES.iter().filter(|t| t.background.is_lava()) {
        let (ground, blob_lo, blob_hi, _dith) = t.background.lava_params().unwrap();
        assert_eq!(
            ground, t.base_100,
            "{}: lava ground must be base_100",
            t.name
        );

        // (1) VALUE BAND. The shader only ever blends ground → blob_lo → blob_hi
        //     (`rgb = mix(ground, mix(blob_lo, blob_hi, core_t), edge_t)`), and
        //     mix() is bounded by its endpoints, so blob_hi is the BRIGHTEST pixel
        //     the animated margin can ever produce. It must not brighten past the
        //     world's own brightest GROUND rung (base_300) — else the margins would
        //     read as "figure", competing with the page. (In HSL-L the probe noted a
        //     ~1–3 point overshoot; in perceptual luminance it vanishes — the wine/
        //     teal blobs are red/blue-heavy, luminance-light.)
        let band_ceiling = rel_lum(t.base_300) + 0.005; // +float epsilon only
        assert!(
            rel_lum(blob_hi) <= band_ceiling,
            "{}: blob_hi luminance {:.4} exceeds the ground band ceiling base_300 {:.4} \
             (animated margin brightens into figure territory)",
            t.name,
            rel_lum(blob_hi),
            rel_lum(t.base_300)
        );
        assert!(
            rel_lum(blob_lo) <= band_ceiling,
            "{}: blob_lo luminance {:.4} exceeds the ground band ceiling",
            t.name,
            rel_lum(blob_lo)
        );

        // (2) blob_hi is a REAL rendered pixel, not just a theoretical ceiling: drive
        //     the pure mirror over a full phase sweep and confirm the metaball field
        //     SATURATES the core blend somewhere in the margin (the shader saturates
        //     core_t at field ≥ THRESHOLD + CORE_WIDTH = 0.85; the strongest backdrop
        //     blob's weight alone exceeds that at its own animated center) — so the ground
        //     genuinely reaches blob_hi, and (1) is a check on an ACTUAL worst-phase pixel.
        let vp = (1200.0, 800.0);
        let blobs = &crate::lava::BACKDROP_BLOBS;
        let mut peak = 0.0f32;
        for step in 0..128 {
            let phase = step as f32 * crate::lava::LAVA_LOOP_CYCLES / 128.0;
            for (i, b) in blobs.iter().enumerate() {
                let (cx, cy) = crate::lava::animated_center(i, b[0], b[1], b[2], vp, phase);
                let px = (cx * vp.0, cy * vp.1);
                peak = peak.max(crate::lava::metaball_field(px, vp, blobs, phase));
            }
        }
        assert!(
            peak >= 1.0,
            "{}: metaball field peaks at only {peak:.3} over a full phase sweep — the core \
             never saturates, so blob_hi is unreached (the worst-phase check would be vacuous)",
            t.name
        );

        // (3) TEXT CONTRAST PRESERVED at the worst phase: the ink (base_content) clears
        //     a strong legibility floor even against the LOUDEST reachable ground pixel
        //     (blob_hi). The floor (150) is far below the measured ~500 (both worlds), so
        //     text sitting anywhere near the margins stays unmistakably the figure.
        let d = redmean(t.base_content, blob_hi);
        assert!(
            d >= 150.0,
            "{}: base_content vs the brightest lava pixel blob_hi only {d:.1} redmean apart \
             (ground competes with the ink at the worst phase)",
            t.name
        );
    }
}

/// THE `Background::Lava` AMBER-HUE-CLEAR GUARD (mirrors the syntax role tints'
/// amber-guard): the lava blobs are ambient GROUND motion — the sole DESIGN.md §3
/// exception this round grants — but the CARET's amber must remain the one accent,
/// so any blob tone with real chroma (HSL saturation > 0.15) sits ≥30° of hue from
/// `primary`. Firetail's wine blobs clear it at ~59°; Mangrove's cool blues at ~175°.
#[test]
fn lava_blob_hues_stay_clear_of_the_amber_caret() {
    // Minimal circular hue distance in degrees.
    fn hue_gap(a: f32, b: f32) -> f32 {
        let d = (a - b).abs() % 360.0;
        d.min(360.0 - d)
    }
    for t in THEMES.iter().filter(|t| t.background.is_lava()) {
        let (_ground, blob_lo, blob_hi, _dith) = t.background.lava_params().unwrap();
        let (ph, _ps, _pl) = t.primary.to_hsl();
        for (label, blob) in [("blob_lo", blob_lo), ("blob_hi", blob_hi)] {
            let (bh, bs, _bl) = blob.to_hsl();
            if bs <= 0.15 {
                continue; // a near-grey blob reads as a value step, not a second accent.
            }
            let gap = hue_gap(bh, ph);
            assert!(
                gap >= 30.0,
                "{}: lava {label} hue {bh:.0}° sits only {gap:.0}° from the amber caret {ph:.0}° \
                 (a second accent — DESIGN §3 one-accent law)",
                t.name
            );
        }
    }
}

/// The `Background::Lava` DATA accessors, exercised via a literal so the
/// coverage does not depend on which worlds ship it: it degrades to a FLAT
/// margin ground (`from == to == ground`, shader 0) that the lava overlay
/// overdraws, names itself `"lava"`, is the ONLY `is_lava()` variant, and
/// surfaces its `(ground, blob_lo, blob_hi, dithered)` params. There is
/// deliberately NO `LavaEdge` mask-mode / name assertion: the dial has one arm,
/// so asserting the surviving name against itself would be green forever.
#[test]
fn lava_background_accessors_are_a_flat_ground_plus_metaball_params() {
    let ground = Srgb::rgb(0x11, 0x27, 0x23);
    let lo = Srgb::rgb(0x17, 0x23, 0x2b);
    let hi = Srgb::rgb(0x22, 0x3c, 0x4f);
    let bg = Background::Lava {
        ground,
        blob_lo: lo,
        blob_hi: hi,
        dithered: true,
    };
    // Degrades to a FLAT ground of the lava `ground`, shader 0 (no margin marks).
    assert_eq!(bg.shader_id(), 0);
    assert_eq!(bg.from(), ground);
    assert_eq!(bg.to(), ground, "flat: from == to");
    assert_eq!(bg.tint(), ground);
    assert!(
        !bg.edge(),
        "the Dots proximity flag is unrelated to the lava ground"
    );
    assert_eq!(bg.as_str(), "lava");
    // The one is_lava variant + its params.
    assert!(bg.is_lava());
    assert!(
        !Background::Gradient {
            from: ground,
            to: ground,
            dir: (0.0, 1.0)
        }
        .is_lava()
    );
    assert_eq!(bg.lava_params(), Some((ground, lo, hi, true)));
    assert_eq!(
        Background::Gradient {
            from: ground,
            to: ground,
            dir: (0.0, 1.0)
        }
        .lava_params(),
        None
    );
}
