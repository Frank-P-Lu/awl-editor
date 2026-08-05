use super::super::derive::SELECTED_BAND_STEPS;
use super::super::*;

#[test]
fn surface_selected_is_an_opaque_ramp_step_past_base_300() {
    let _g = crate::testlock::serial();
    for (i, t) in THEMES.iter().enumerate() {
        set_active(i);
        let band = surface_selected();
        // A SOLID band (figure/ground by VALUE), never the translucent selection.
        assert_eq!(band.a, 0xFF, "{} band must be opaque", t.name);
        // TRUE 1-BIT WORLDS (`Theme::is_one_bit`): a DECLARED exemption from
        // "must not be the selection token" — with only two legal values,
        // `surface_selected` (the elevation BORDER, pure white) and
        // `selection` (now also pure OPAQUE white — see the test above) are
        // necessarily the SAME literal color; they're distinguished by SHAPE/
        // CONTEXT (a thin border rim vs. a punched-outline selection band),
        // never by hue or translucency, which no longer exist to distinguish
        // them with. See THEMES.md's "The 1-bit law".
        if t.is_one_bit() {
            assert_eq!(
                band, t.selection_document,
                "{}: one-bit surface_selected and selection are necessarily the same pure white",
                t.name
            );
            continue;
        }
        assert_ne!(
            band, t.selection_document,
            "{} band must not be the selection token",
            t.name
        );
        // Each channel continues the base_200 -> base_300 step SELECTED_BAND_STEPS
        // more increments, or saturates at the gamut edge (never reverses direction).
        let want = SELECTED_BAND_STEPS;
        for (lo, hi, got) in [
            (t.base_200.r, t.base_300.r, band.r),
            (t.base_200.g, t.base_300.g, band.g),
            (t.base_200.b, t.base_300.b, band.b),
        ] {
            let dir = hi as i32 - lo as i32; // ramp direction (toward the ink)
            let step = got as i32 - hi as i32; // band's move past base_300
            if dir > 0 {
                assert!(
                    step >= 0 && (got == 255 || step == dir * want),
                    "{} band channel reversed",
                    t.name
                );
            } else if dir < 0 {
                assert!(
                    step <= 0 && (got == 0 || step == dir * want),
                    "{} band channel reversed",
                    t.name
                );
            }
        }
    }
    set_active(DEFAULT_THEME);
}

#[test]
fn selection_is_the_only_translucent_token() {
    for t in THEMES.iter() {
        assert_eq!(t.base_100.a, 0xFF);
        assert_eq!(t.primary.a, 0xFF);
        assert_eq!(t.error.a, 0xFF);
        // The margin gradient endpoints are opaque (the shader owns the
        // margin opacity), so selection stays the only translucent token.
        assert_eq!(
            t.background.from().a,
            0xFF,
            "{} background from alpha",
            t.name
        );
        assert_eq!(t.background.to().a, 0xFF, "{} background to alpha", t.name);
        // TRUE 1-BIT WORLDS (`Theme::is_one_bit`): a DECLARED exemption from
        // "selection is THE translucent token" — any alpha strictly between 0
        // and 255 composites a forbidden grey over this world's pure ground,
        // so selection is pure OPAQUE white instead (`0xFF`), with legibility
        // over selected text carried by a separate render-side mechanism (the
        // DITHER round's TRUE inverse-video pipeline,
        // `TextPipeline::selection_invert`), not by this token's alpha. See
        // THEMES.md's "The 1-bit law".
        if t.is_one_bit() {
            assert_eq!(
                t.selection_document.a, 0xFF,
                "{}: one-bit selection must be fully OPAQUE",
                t.name
            );
            continue;
        }
        // Selection is the ONE translucent token — a calm highlight, never opaque
        // (a paint fill) nor so sheer it fails the contrast floor. The exact alpha
        // is PER-WORLD now: most sit at 0x52, but a world whose composited
        // selection would be sub-glance over its own ground lifts it (Bombora /
        // Mangrove → 0x60) to clear `ink_ladder_and_selection_laws_*`.
        assert!(
            (0x40..0xA0).contains(&t.selection_document.a),
            "{} selection alpha {:#04x} outside the calm-translucent band [0x40, 0xA0)",
            t.name,
            t.selection_document.a
        );
    }
}

/// WYSIWYG VALUE-STEP LAW (`render/rects.rs`'s fenced-code PANEL + inline-code
/// PILL, `fence_panel_pipeline`/`code_pill_pipeline` in `render.rs`): both quads
/// reuse the ALREADY-DECLARED `base_200` token verbatim — no new color
/// derivation, so this is not a new hue/wash formula to law-test. Two minimal
/// properties DO matter now that the token draws as a distinct opaque surface
/// rather than just a margin-gradient stop:
/// (a) it must actually READ as a step off the ground (`base_100`) — an
/// invisible panel/pill defeats its own affordance — and
/// (b) it must never be LITERALLY the accent color (a background step sharing
/// `primary`'s general warmth is fine and common — many worlds tint their whole
/// ground ramp toward their signature hue, already covered by the ground-
/// contrast + background-validity laws above — but it must never be an exact
/// hit, which would make the panel read as a spent accent rather than a ground
/// step).
#[test]
fn wysiwyg_value_step_law_holds_for_every_world() {
    for t in THEMES.iter() {
        // TRUE 1-BIT WORLDS (`Theme::is_one_bit`): a DECLARED exemption — the
        // panel/pill's "OFF" answer (base_200 flush with the ground, so the
        // WYSIWYG affordance is genuinely invisible) is the whole point on a
        // world with only two legal values and no border companion for this
        // specific primitive; see THEMES.md's "The 1-bit law".
        if t.is_one_bit() {
            assert_eq!(
                t.base_200, t.base_100,
                "{}: one-bit base_200 stays flush with the ground (the panel/pill's OFF answer)",
                t.name
            );
            continue;
        }
        assert_ne!(
            t.base_200, t.base_100,
            "{}: base_200 must differ from base_100 or the WYSIWYG panel/pill is invisible",
            t.name
        );
        assert_ne!(
            t.base_200, t.primary,
            "{}: base_200 must never be exactly the accent color",
            t.name
        );
    }
}

/// Every world defines a NON-DEGENERATE margin gradient: the two endpoints
/// differ (so there is a real gradient, not a flat fill) and the direction
/// vector is non-zero (so `dot(uv, dir)` actually varies across the margin).
#[test]
fn every_world_has_a_real_margin_gradient() {
    for t in THEMES.iter() {
        let bg = t.background;
        // TRUE 1-BIT WORLDS (`Theme::is_one_bit`, Wagtail's 2026-07 rework):
        // a DECLARED exemption, not a weakening — a real (non-degenerate)
        // gradient necessarily interpolates through forbidden intermediate
        // greys between its two endpoints, so a one-bit world's margin ground
        // must be the ONE `Background` variant guaranteed not to (a flat
        // `Gradient` with `from == to`, mathematically the same color at
        // every pixel). See THEMES.md's "The 1-bit law".
        if t.is_one_bit() {
            assert_eq!(
                bg.from(),
                bg.to(),
                "{}: a one-bit world's margin gradient must be FLAT (from == to) — \
                 any real gradient interpolates through forbidden greys",
                t.name
            );
            continue;
        }
        // LAVA WORLDS (`Background::Lava`, Firetail/Mangrove): a DECLARED exemption,
        // not a weakening — the base margin ground is DELIBERATELY flat (from == to
        // == the lava `ground`), because the lava OVERLAY (`crate::lava`, a separate
        // pipeline drawn after this margin pass) carries all the marks + motion and
        // OVERDRAWS the margins opaquely; the flat base is only there so the floor is
        // painted before the overlay draws. See `Background::Lava`'s shader_id() doc.
        if t.background.is_lava() {
            assert_eq!(
                bg.from(),
                bg.to(),
                "{}: a lava world's BASE margin ground must be FLAT (the lava overlay \
                 carries the motion)",
                t.name
            );
            continue;
        }
        assert_ne!(
            bg.from(),
            bg.to(),
            "{} margin gradient is degenerate (from == to)",
            t.name
        );
        let (dx, dy) = bg.dir();
        assert!(
            dx.abs() + dy.abs() > 0.0,
            "{} background dir is the zero vector",
            t.name
        );
    }
}

#[test]
fn hex_round_trips_known_values() {
    assert_eq!(POTOROO.base_100.hex(), "#1f0400");
    assert_eq!(POTOROO.primary.hex(), "#feaf69");
    assert_eq!(GUMTREE.base_100.hex(), "#e4f8e2");
    // Tawny — the default world's exact spec hexes.
    assert_eq!(TAWNY.base_100.hex(), "#16181d");
    assert_eq!(TAWNY.base_content.hex(), "#e6e6e6");
    assert_eq!(TAWNY.primary.hex(), "#ffc05e");
    assert_eq!(TAWNY.error.hex(), "#e54b4b");
    assert_eq!(TAWNY.selection_document.hex(), "#3a6fd8");
}

/// THE LAW ROUND's `Theme::highlight_treatment` — a NO-ABSENT-VARIANT
/// enum consumed by `render/chrome/overlay.rs`'s picker-row highlight and
/// `render/chrome/menubar.rs`'s open-title band, replacing the former
/// hand-rolled `if selection_style == InverseVideo { .. } else { .. }` at
/// each of those two sites. This pins the STRUCTURAL half of the contract
/// (every world resolves to EXACTLY the treatment its `selection_style`
/// names, with no third "neither" outcome reachable) across all sixteen
/// worlds; the REAL-PIXEL half — does the renderer actually honor it — lives
/// in `render::tests::distinguishability`.
#[test]
fn highlight_treatment_matches_selection_style_on_every_world_no_absent_case() {
    for t in THEMES.iter() {
        let band = crate::theme::Srgb::rgb(0x11, 0x22, 0x33);
        let treatment = t.highlight_treatment(band);
        match (t.render_caps.selection_style, treatment) {
            (
                crate::theme::SelectionStyle::Fill,
                crate::theme::HighlightTreatment::ValueBand(c),
            ) => {
                assert_eq!(
                    c, band,
                    "{}: ValueBand must carry the caller's own band color",
                    t.name
                );
            }
            (
                crate::theme::SelectionStyle::InverseVideo,
                crate::theme::HighlightTreatment::InverseFill { band: b, ink },
            ) => {
                // A 1-bit world resolves the pair off its OWN ladder, not the
                // caller's `band`: solid `base_content` fill + `base_300` glyphs.
                assert_eq!(
                    b, t.base_content,
                    "{}: InverseFill band must be base_content",
                    t.name
                );
                assert_eq!(
                    ink, t.base_300,
                    "{}: InverseFill ink must be base_300",
                    t.name
                );
            }
            (style, treatment) => panic!(
                "{}: selection_style {style:?} produced the WRONG treatment {treatment:?} — \
                 the enum's whole point is that this pairing is supposed to be unreachable",
                t.name
            ),
        }
    }
}

// --- THE OVERLAY-PERSONALITY-AS-DATA ROUND -----------------------------

/// `Srgb::lerp` — the pure blend primitive `placard_ink` (below) leans on.
#[test]
fn lerp_interpolates_and_clamps() {
    let a = Srgb::rgb(0, 0, 0);
    let b = Srgb::rgb(100, 200, 40);
    assert_eq!(a.lerp(b, 0.0), a, "t=0 is exactly self");
    assert_eq!(a.lerp(b, 1.0), b, "t=1 is exactly other");
    assert_eq!(
        a.lerp(b, 0.5),
        Srgb::rgb(50, 100, 20),
        "t=0.5 is the exact midpoint"
    );
    // Out-of-range t clamps rather than extrapolating past either endpoint.
    assert_eq!(a.lerp(b, -1.0), a, "t<0 clamps to self");
    assert_eq!(a.lerp(b, 2.0), b, "t>1 clamps to other");
}
