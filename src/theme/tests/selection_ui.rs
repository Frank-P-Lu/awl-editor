use super::super::derive::OVERLAY_SELROW_EXTRA_STEPS;
use super::super::*;

/// THE PICKER'S SELECTED-ROW BAND
/// ([`selection_ui`]) is the shared [`surface_selected`] climbed
/// [`OVERLAY_SELROW_EXTRA_STEPS`] FURTHER up the SAME surface ramp — a stronger
/// VALUE step, in the ramp's own direction, never a new hue (DESIGN §3/§5; the
/// distinguishability sweep is the law that polices its visibility). The shared
/// band the HUD/menu borders read is untouched.
///
/// This sweeps `selection_ui`'s derivation, which is the only shipped shape.
#[test]
fn selection_ui_is_a_stronger_value_step_never_a_hue() {
    let _g = crate::testlock::serial();
    assert!(
        std::hint::black_box(OVERLAY_SELROW_EXTRA_STEPS) > 0,
        "OVERLAY_SELROW_EXTRA_STEPS must be positive so the picker's selected row is \
         a stronger value step than the shared ramp by default"
    );
    for world in ["Bowerbird", "Saltpan", "Firetail", "Tawny"] {
        let t = set_active_by_name(world).unwrap();
        assert_ne!(
            t.base_200, t.base_300,
            "{world}: ordinary (non-collapsed) ramp"
        );
        let shared = surface_selected();
        let band = selection_ui();
        // Per channel: the overlay band moves in the SAME direction the ramp step
        // does (value-only, no hue reversal) and is at least as far as the shared
        // band (stronger-or-equal, gamut-clamp permitting).
        let chans = [
            (t.base_200.r, t.base_300.r, shared.r, band.r),
            (t.base_200.g, t.base_300.g, shared.g, band.g),
            (t.base_200.b, t.base_300.b, shared.b, band.b),
        ];
        for (lo, hi, sh, bd) in chans {
            let d = (hi as i32 - lo as i32).signum();
            let band_delta = bd as i32 - hi as i32;
            let shared_delta = sh as i32 - hi as i32;
            assert!(
                band_delta * d >= 0,
                "{world}: band stays in the ramp direction"
            );
            assert!(
                band_delta * d >= shared_delta * d,
                "{world}: overlay band is >= the shared band's step (stronger-or-equal)"
            );
        }
    }
    // Non-triviality: on a dark world with ramp headroom the strengthening is
    // STRICT (the extra step actually moves the band).
    set_active_by_name("Bowerbird").unwrap();
    assert_ne!(
        selection_ui(),
        surface_selected(),
        "Bowerbird: the strengthened band differs from the shared band"
    );
    set_active(DEFAULT_THEME);
}

/// THE DERIVATION IS THE GUARANTEE. Every world routes through the one owner;
/// there is no dormant authored override whose unexercised branch can drift.
#[test]
fn every_world_routes_selection_ui_through_the_derivation() {
    let _g = crate::testlock::serial();
    for (i, t) in THEMES.iter().enumerate() {
        set_active(i);
        assert_eq!(
            selection_ui(),
            derive::derived_selection_ui(),
            "{}: with no override, selection_ui must BE the derivation",
            t.name
        );
    }
    set_active(DEFAULT_THEME);
}

/// PER-ITEM LIST SURFACES round (2026-07-16 REFIT) — the OBVIOUS-GLANCE law at
/// the derivation level, covering EVERY world (the pixel test
/// `bars_draw_a_findable_surface_per_row` only exercises the headless default
/// theme). Under [`ListStyle::Bars`] the PANE is dropped — the bars float on the
/// GROUND (`base_100`, the scrim/room), not in a card. So the reference is the
/// GROUND, not the vanished card: the unselected bar ([`overlay_bar_unselected`]
/// == `base_200`) is a WHISPER one gentle step off the ground in the ramp's own
/// direction, and the selected bar's band ([`selection_ui`]) sits
/// further up still — AND the selected↔unselected value step is at least as large
/// as the unselected↔ground step, so the selected bar's pop leads its whisper
/// neighbours at least as strongly as a whisper leads the bare ground. The user's
/// rejected first cut inverted the taste (unselected == a saturated rung under the
/// selected band — "a picket fence where every row shouts"); the whisper gives the
/// selection somewhere to go. Value only, never a hue. One-bit worlds are exempt
/// (a collapsed ramp draws its selected row via `InverseFill`; bars are inert).
#[test]
fn bars_unselected_sits_a_quiet_rung_below_the_selected_band() {
    let _g = crate::testlock::serial();
    // Local redmean (perceptual distance) — the same shape the distinguishability
    // sweeps carry, nested per-test like this file's other color laws.
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        if t.is_one_bit() {
            // Collapsed ramp: `surface_step_band` folds every step to the ink
            // pole, so the ordering degenerates by design — the selected row is
            // drawn by `InverseFill`, not this fill. Declared exemption.
            continue;
        }
        // The GROUND the bars float on, the pane being dropped: base_100.
        let ground = t.base_100;
        let unsel = overlay_bar_unselected();
        let sel = selection_ui();
        // Per channel: `unsel` moves in the ramp direction from the GROUND (a
        // whisper), and `sel` moves at least as far again (whisper strictly between
        // ground and selected in the ramp's own direction, value-only — no hue).
        let chans = [
            (ground.r, unsel.r, sel.r),
            (ground.g, unsel.g, sel.g),
            (ground.b, unsel.b, sel.b),
        ];
        // The overall ramp direction (base_200 -> base_300 carries it onward; the
        // monotone surface ladder makes this the base_100 -> base_200 step's sign too).
        let dir = [
            (t.base_300.r as i32 - t.base_200.r as i32).signum(),
            (t.base_300.g as i32 - t.base_200.g as i32).signum(),
            (t.base_300.b as i32 - t.base_200.b as i32).signum(),
        ];
        for (i, (c, u, s)) in chans.iter().copied().enumerate() {
            let d = dir[i];
            let unsel_step = (u as i32 - c as i32) * d;
            let sel_step = (s as i32 - c as i32) * d;
            assert!(
                unsel_step >= 0,
                "{}: unselected whisper lifts off the ground in the ramp direction",
                t.name
            );
            assert!(
                sel_step >= unsel_step,
                "{}: selected band ({s}) must sit at least as far up the ramp as the unselected whisper ({u}) from the ground ({c})",
                t.name
            );
        }
        // The OBVIOUS-GLANCE law (redmean): the selected↔unselected step reads at
        // least as strong as the unselected↔ground step — selection's pop leads its
        // whisper neighbours at least as much as a whisper leads the bare ground.
        let d_sel = redmean(sel, unsel);
        let d_bar = redmean(unsel, ground);
        assert!(
            d_sel >= d_bar,
            "{}: selected bar {sel:?} must lead the unselected whisper {unsel:?} (redmean {d_sel:.1}) at least as much as the whisper leads the ground {ground:?} (redmean {d_bar:.1})",
            t.name
        );
    }
    set_active(DEFAULT_THEME);
}
