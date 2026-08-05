use super::super::*;

/// FIRETAIL PALETTE CHARACTER law: the sixteenth world is an ORIGINAL deep
/// oxblood-charcoal + wine-lava + ember-gold system, not Potoroo's rust palette
/// copied under a moving ground. Hue arithmetic pins the authored direction:
/// Firetail's main ground is much nearer red than Bombora's violet, at least
/// 35° away from Potoroo's orange-rust ground, and both its lava and caret stay
/// in their named wine/gold bands.
#[test]
fn firetail_is_oxblood_wine_and_ember_not_potoroo_rust_or_bombora_violet() {
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    fn hue_gap(a: f32, b: f32) -> f32 {
        let d = (a - b).abs() % 360.0;
        d.min(360.0 - d)
    }
    fn red_gap(h: f32) -> f32 {
        hue_gap(h, 0.0)
    }

    let fire_ground = FIRETAIL.base_300.to_hsl().0;
    let potoroo_rust = POTOROO.base_300.to_hsl().0;
    let bombora_violet = BOMBORA.base_300.to_hsl().0;
    assert!(
        red_gap(fire_ground) + 60.0 <= red_gap(bombora_violet),
        "Firetail ground {fire_ground:.1}° must read far redder/warmer than Bombora {bombora_violet:.1}°"
    );
    assert!(
        hue_gap(fire_ground, potoroo_rust) >= 35.0,
        "Firetail ground {fire_ground:.1}° must stay substantially clear of Potoroo's orange-rust {potoroo_rust:.1}°"
    );

    let (base_h, base_s, base_l) = FIRETAIL.base_100.to_hsl();
    assert!(
        red_gap(base_h) <= 25.0 && base_s >= 0.25 && base_l <= 0.08,
        "Firetail base_100 must stay deep oxblood-charcoal, got h={base_h:.1}° s={base_s:.2} l={base_l:.2}"
    );

    let (_ground, lo, hi, dithered) = FIRETAIL.background.lava_params().unwrap();
    for (label, c) in [("blob_lo", lo), ("blob_hi", hi)] {
        let h = c.to_hsl().0;
        assert!(
            h >= 330.0,
            "Firetail {label} hue {h:.1}° must stay in the deep red/wine band"
        );
    }
    let caret_h = FIRETAIL.primary.to_hsl().0;
    assert!(
        (35.0..=50.0).contains(&caret_h),
        "Firetail caret hue {caret_h:.1}° must stay ember-gold"
    );
    assert!(
        hue_gap(caret_h, lo.to_hsl().0) >= 45.0 && hue_gap(caret_h, hi.to_hsl().0) >= 45.0,
        "Firetail's ember caret must stay at least 45° clear of both wine-lava tones"
    );
    assert!(
        redmean(FIRETAIL.base_content, FIRETAIL.base_100) >= 500.0,
        "Firetail blush ink must keep strong contrast over the oxblood ground"
    );
    assert!(
        redmean(FIRETAIL.primary, FIRETAIL.base_100) >= 300.0,
        "Firetail ember caret must remain immediately visible over the ground"
    );
    assert!(
        !dithered,
        "Firetail stays smooth; Mangrove owns lava dither"
    );
}

/// NUMERIC INTER-WORLD DISTINCTNESS law: compare Firetail's WHOLE authored token
/// vector (not merely its animated-background enum) against every other world by
/// RMS redmean distance. A copied palette scores zero; a near-copy cannot
/// hide behind a different ground shader or font. The 70-point RMS floor is a
/// clear multi-token separation while leaving individual quiet rungs coherent.
#[test]
fn firetail_palette_is_numerically_distinct_from_every_other_world() {
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    fn tokens(t: &Theme) -> [Srgb; 10] {
        [
            t.base_100,
            t.base_200,
            t.base_300,
            t.base_content,
            t.muted,
            t.faint,
            t.primary,
            t.primary_content,
            t.error,
            Srgb::rgb(
                t.selection_document.r,
                t.selection_document.g,
                t.selection_document.b,
            ),
        ]
    }

    let fire = tokens(&FIRETAIL);
    for other in THEMES.iter().filter(|t| t.name != FIRETAIL.name) {
        let theirs = tokens(other);
        let rms = (fire
            .iter()
            .zip(theirs)
            .map(|(&a, b)| redmean(a, b).powi(2))
            .sum::<f32>()
            / fire.len() as f32)
            .sqrt();
        assert!(
            rms >= 70.0,
            "Firetail whole-palette distance from {} is only {rms:.1} RMS redmean (floor 70)",
            other.name
        );
    }
}
