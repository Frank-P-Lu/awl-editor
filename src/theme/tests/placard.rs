use super::super::*;

/// COMPOSITION-C2 DATA SANITY for shipped placards (the old "every placard is
/// BL" pin is GONE — the poster corner now DERIVES from the card anchor via
/// [`crate::render::derived_placard_corner`], complementary so the wordmark
/// never sits under the command surface, and the no-clip OUTCOME is asserted
/// end-to-end by `render::tests::overlay_personality`'s no-clip law). Here the
/// DATA stays honest: a placard corner is either `Auto` (derive) or a concrete
/// override (Firetail's user-picked `BL`), and every scale sits in a sane band.
/// A placard world MUST NOT centre its card (`TopCenter`) — a centred card with
/// an `Auto` bottom-corner poster would still read fine, but the shipped
/// placard worlds are the statement/asymmetric temperaments that anchor their
/// card away from centre, so this guards the intended composition.
#[test]
fn every_shipped_placard_world_has_sane_corner_and_scale() {
    let placards: Vec<(&str, model::PlacardCorner, f32, model::CardAnchor)> = THEMES
        .iter()
        .filter_map(|t| match t.render_caps.title_style {
            model::TitleStyle::Placard { corner, scale, .. } => {
                Some((t.name, corner, scale, t.render_caps.card_anchor))
            }
            model::TitleStyle::InlinePrefix => None,
        })
        .collect();
    assert!(
        !placards.is_empty(),
        "at least one world ships a Placard (the round that introduced them) — a \
         zero here means the data table lost every placard, not that the guard passed"
    );
    for (name, corner, scale, anchor) in placards {
        // A legal corner: derive (`Auto`) or a concrete override — never junk.
        assert!(
            matches!(
                corner,
                model::PlacardCorner::Auto
                    | model::PlacardCorner::BL
                    | model::PlacardCorner::BR
                    | model::PlacardCorner::TL
                    | model::PlacardCorner::TR
            ),
            "{name}: placard corner {corner:?} must be a legal value"
        );
        // The shipped placard worlds anchor their card away from centre (the
        // statement temperament), so the complementary poster derivation lands
        // it cleanly opposite the card.
        assert_ne!(
            anchor,
            model::CardAnchor::TopCenter,
            "{name}: a shipped placard world anchors its card off-centre (see this test's doc)"
        );
        // The wordmark scale is a loudness dial, not a fit guarantee
        // (`overlay_shape_placard` shrinks a wider-than-canvas mark), but a
        // shipped value staying in a sane band keeps the data honest.
        assert!(
            (0.5..=5.0).contains(&scale),
            "{name}: shipped placard scale {scale} sits outside the sane 0.5..=5.0 band"
        );
    }
}

/// `theme::placard_ink` NEVER invents a free color, and is MODE-AWARE (the
/// personality-assignment round's dark-ground correction): LIGHT worlds keep
/// the gallery-validated originals byte-for-byte (`Faint` = the world's own
/// faint ink verbatim; `Ghost` = a pure `faint`/`base_300` blend); DARK
/// worlds step the SAME two rungs UP the ladder instead (pure
/// `faint`→`base_content` blends — one global lift constant per rung, never
/// a per-world hand value; the legibility floor/ceiling those lifts must
/// clear is the separate law below). `Stipple`'s pixel ink is exactly
/// `base_content` on every world — the density, not the ink, carries its
/// quietness (see `placard_stipple_density`'s own law).
#[test]
fn placard_ink_derives_from_the_ink_ladder_never_a_free_color() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        let faint_rung = derive::placard_ink(model::PlacardInk::Faint);
        let ghost = derive::placard_ink(model::PlacardInk::Ghost);
        if t.dark {
            // A pure blend of two rungs already on the ladder: every channel
            // of the result must sit BETWEEN faint and base_content (a lerp
            // can't leave its endpoints), and the two rungs must be exactly
            // the documented one-formula lifts (re-derived here, so a future
            // per-world special case fails loudly).
            assert_eq!(
                faint_rung,
                t.faint.lerp(t.muted, 0.75),
                "{}: dark-ground PlacardInk::Faint must be the one documented ladder lift",
                t.name
            );
            assert_eq!(
                ghost,
                t.faint.lerp(t.muted, 0.45),
                "{}: dark-ground PlacardInk::Ghost must be the one documented ladder lift",
                t.name
            );
        } else {
            assert_eq!(
                faint_rung, t.faint,
                "{}: light-ground PlacardInk::Faint must be exactly the world's own faint ink \
                 (the gallery-validated original)",
                t.name
            );
            assert_eq!(
                ghost,
                t.faint.lerp(t.base_300, 0.5),
                "{}: light-ground PlacardInk::Ghost must be a pure faint/base_300 blend \
                 (the gallery-validated original)",
                t.name
            );
        }
        assert_eq!(
            derive::placard_ink(model::PlacardInk::Stipple),
            t.base_content,
            "{}: PlacardInk::Stipple pixels draw in exactly the world's own full ink",
            t.name
        );
    }
    set_active(DEFAULT_THEME);
}

/// THE DARK-GROUND PLACARD LEGIBILITY LAW (the user's 2026-07-15 taste note,
/// enforced: "the dark worlds — there's not enough contrast for the placard";
/// Bombora's Ghost was near-invisible). On every DARK world both placard
/// rungs must clearly READ against the world's own ground — a relative-
/// luminance floor, the same domain law (h) of the role tints uses, because
/// the eye resolves luminance — while still RECEDING behind the rows: the
/// louder rung (`Faint`) stays at or under the world's own `muted` ink in
/// luminance (a legible ghost, never a competing headline), and presence
/// ordering holds (`Faint` ≥ `Ghost`, mirroring the light-mode ordering).
/// Light worlds are pinned byte-identical by the derivation law above, so
/// this law binds exactly where the taste note pointed. The AMBER GUARD
/// binds BY IDENTITY, the comment-tier way (role-tint law (e)'s own
/// exemption): a placard ink is a pure blend of existing ink-ladder rungs —
/// it IS the world's ink, which on a warm-laddered world (Potoroo) shares
/// the caret's general warmth without being the accent — so the assertable
/// half is that it is never LITERALLY `primary` (monochrome worlds exempt:
/// their caret IS their ink by design, and none ships a placard anyway —
/// the assignment table pins that).
#[test]
fn placard_inks_read_on_dark_grounds_and_stay_below_muted() {
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
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        let faint_rung = derive::placard_ink(model::PlacardInk::Faint);
        let ghost = derive::placard_ink(model::PlacardInk::Ghost);
        // AMBER GUARD by identity: never literally the accent.
        if !t.is_monochrome() {
            for (label, ink) in [("Faint", faint_rung), ("Ghost", ghost)] {
                assert_ne!(
                    ink, t.primary,
                    "{}: placard {label} ink must never be literally the accent",
                    t.name
                );
            }
        }
        if !t.dark {
            continue;
        }
        let ground = rel_lum(t.base_100);
        let dy_ghost = rel_lum(ghost) - ground;
        let dy_faint = rel_lum(faint_rung) - ground;
        // FLOOR: the same ΔY ≥ 0.05 luminance floor the role tints carry —
        // the quieter rung must clear it, so the louder one does a fortiori.
        assert!(
            dy_ghost >= 0.05,
            "{}: dark-ground Ghost placard ink {} sits only ΔY {dy_ghost:.3} above the ground \
             (near-invisible — the Bombora gallery bug)",
            t.name,
            ghost.hex()
        );
        // ORDERING: Faint is the more-present rung, on dark exactly as on light.
        assert!(
            dy_faint >= dy_ghost - 1e-4,
            "{}: placard presence ordering inverted (Faint ΔY {dy_faint:.3} < Ghost ΔY {dy_ghost:.3})",
            t.name
        );
        // CEILING: a legible GHOST, not a competing headline — the louder rung
        // stays at or under the world's own muted ink (the non-selected row
        // ink on the card it bleeds behind). Equality is legal (Wagtail's
        // collapsed ladder makes every ink rung the same white — moot anyway,
        // since Wagtail ships no placard).
        let dy_muted = rel_lum(t.muted) - ground;
        assert!(
            dy_faint <= dy_muted + 1e-4,
            "{}: dark-ground Faint placard ink {} (ΔY {dy_faint:.3}) outshines the world's own \
             muted ink (ΔY {dy_muted:.3}) — a competing headline, not a ghost",
            t.name,
            faint_rung.hex()
        );
    }
    set_active(DEFAULT_THEME);
}

/// THE STIPPLE PLACARD LAW: `Stipple`'s two derived halves stay on the
/// world's own ladder and stay LEGIBLE. (a) The pixel ink is exactly
/// `base_content` (asserted per-world by the derivation law above) — so a
/// stipple can only ever paint the ladder's full ink, never amber, never a
/// free color; on a MONOCHROME/1-bit world that ink is its legal pure white,
/// which is why `Stipple` is the one placard ink that would be monochrome-
/// legal by construction (banked — Wagtail ships no placard). (b) The
/// density is the documented perceived-tone formula, clamped to its
/// floor/ceiling band. (c) THE LEGIBILITY FLOOR OVER THE WORLD'S OWN GROUND
/// (the 3b taste-note assertion): the stipple's MEAN tone — ground blended
/// toward the ink at `density` — clears the same ΔY ≥ 0.05 luminance floor
/// the flat placard inks carry, against the flat ground AND, on a lava
/// world, against the brightest pixel the animated margin can ever produce
/// (`blob_hi` — captures render t=0, but the law covers every phase since
/// `mix()` is bounded by its endpoints; the lava figure/ground law proves
/// blob_hi is genuinely reached). Swept over EVERY world (the derivation is
/// total), so a future stipple assignment is born covered.
#[test]
fn stipple_placard_density_clears_the_legibility_floor_over_its_own_ground() {
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
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        let density = derive::placard_stipple_density();
        assert!(
            (0.12..=0.55).contains(&density),
            "{}: stipple density {density:.3} escaped the floor/ceiling band",
            t.name
        );
        let ink = derive::placard_ink(model::PlacardInk::Stipple);
        let ground = rel_lum(t.base_100);
        let mean = ground + density * (rel_lum(ink) - ground);
        assert!(
            (mean - ground).abs() >= 0.05,
            "{}: stipple mean tone ΔY {:.3} vs the flat ground fails the legibility floor",
            t.name,
            (mean - ground).abs()
        );
        // The lava arm: the ONLY moving ground a stipple placard can sit
        // over. Its brightest reachable pixel must not swallow the mark.
        if let Some((_, _, blob_hi, _)) = t.background.lava_params() {
            let worst = rel_lum(blob_hi);
            assert!(
                (mean - worst).abs() >= 0.05,
                "{}: stipple mean tone ΔY {:.3} vs the worst-phase lava pixel {} fails the \
                 legibility floor",
                t.name,
                (mean - worst).abs(),
                blob_hi.hex()
            );
        }
    }
    set_active(DEFAULT_THEME);
}

/// EVERY GREY PLACARD INK, closed structurally: a
/// `TitleStyle::Placard` whose ink is `Faint` OR `Ghost` on a TRUE 1-BIT
/// world (`Theme::is_one_bit`) would render an ordinary intermediate-grey
/// wordmark (and antialiased glyph fringes besides), which that world's own
/// law (`render::tests::syntax_roles::every_one_bit_world_renders_only_pure_
/// black_or_white`) forbids outright. `Stipple` is deliberately EXEMPT: its
/// pixels are hard-thresholded pure `base_content` at full alpha or nothing
/// (the same 1-bit-legality argument as the highlight stipple) — though no
/// one-bit world ships ANY placard today (Wagtail is the user-confirmed
/// silent pole; the assignment-table law pins that). Lives in `theme::`,
/// deliberately never `render::`, where a bare `.is_one_bit()` call is
/// banned outright (`render::tests::theme_caps_law`) — the "pin an identity,
/// not a render mechanism" carve-out that grep-law's own doc describes.
#[test]
fn a_placard_grey_ink_would_violate_a_one_bit_worlds_own_law() {
    for t in THEMES.iter() {
        if let model::TitleStyle::Placard {
            // The FIRETAIL-MAXIMALIST-SHOWCASE dial-up rungs (`Muted`/`Bold`)
            // are ordinary greys on every world today, so they join the
            // guarded set alongside `Faint`/`Ghost`; `Stipple` stays the one
            // 1-bit-legal exemption (hard pure-ink pixels).
            ink:
                ink @ (model::PlacardInk::Faint
                | model::PlacardInk::Ghost
                | model::PlacardInk::Muted
                | model::PlacardInk::Bold),
            ..
        } = t.render_caps.title_style
        {
            assert!(
                !t.is_one_bit(),
                "{}: TitleStyle::Placard{{ink: {ink:?}}} on a true 1-bit world renders an \
                 illegal intermediate grey — of the placard inks only Stipple (hard pure-ink \
                 pixels) is 1-bit-legal by construction",
                t.name
            );
        }
    }
}

/// THE FIRETAIL-MAXIMALIST-SHOWCASE round's DIAL-UP ink law: the two new
/// smooth rungs (`Muted`/`Bold`) are pure ladder derivations through the ONE
/// owner (`theme::placard_ink`) — `Muted` IS the world's own `muted` rung
/// verbatim, `Bold` is a pure `muted`→`base_content` blend that stays
/// strictly BELOW full ink (the rows always outshine the wordmark, by
/// construction), presence-ordered above `Faint` (louder is genuinely
/// louder, on every world, both grounds), and — the never-amber guard, in
/// its identity form — never literally the accent on any chromatic world
/// (they're ladder greys; the assertable half is non-identity, the same
/// shape as `page_frame_ink`'s own guard). Every world is swept even though
/// no world SHIPS a dial-up rung yet: the probe (`AWL_OVERLAY_STYLE_FORCE`)
/// makes them reachable on all sixteen today, so the law must already hold
/// everywhere, not just on a future assignee.
#[test]
fn dialup_placard_inks_stay_on_the_ladder_below_full_ink() {
    let _g = crate::testlock::serial();
    // Gamma-correct Rec.709 relative luminance (the same local recipe the
    // other placard-ink laws carry).
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
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        let muted_rung = derive::placard_ink(model::PlacardInk::Muted);
        let bold = derive::placard_ink(model::PlacardInk::Bold);
        let faint_rung = derive::placard_ink(model::PlacardInk::Faint);
        assert_eq!(
            muted_rung, t.muted,
            "{}: PlacardInk::Muted must be exactly the world's own muted rung",
            t.name
        );
        assert_eq!(
            bold,
            t.muted.lerp(t.base_content, 0.5),
            "{}: PlacardInk::Bold must be the one documented muted→base_content blend",
            t.name
        );
        // Presence ordering, in ink-distance-from-ground terms: Faint ≤ Muted ≤
        // Bold < full ink — the dial goes UP, and its ceiling is structural.
        let ground = rel_lum(t.base_100);
        let dy = |c: Srgb| (rel_lum(c) - ground).abs();
        assert!(
            dy(faint_rung) <= dy(muted_rung) + 1e-6,
            "{}: Muted must read at least as present as Faint (ΔY {:.4} < {:.4})",
            t.name,
            dy(muted_rung),
            dy(faint_rung)
        );
        assert!(
            dy(muted_rung) <= dy(bold) + 1e-6,
            "{}: Bold must read at least as present as Muted (ΔY {:.4} < {:.4})",
            t.name,
            dy(bold),
            dy(muted_rung)
        );
        // The strict below-full-ink ceiling exempts a TRUE 1-BIT world (the
        // same declared exemption arm the dark-ground placard law carries):
        // its ladder COLLAPSES (`muted == base_content`, pure white), so the
        // blend is degenerate — and a grey placard rung is already
        // structurally illegal there anyway (`a_placard_grey_ink_would_
        // violate_a_one_bit_worlds_own_law` guards Muted/Bold too).
        if !t.is_one_bit() {
            assert!(
                dy(bold) < dy(t.base_content),
                "{}: Bold (ΔY {:.4}) must stay BELOW full ink (ΔY {:.4}) — the rows always win",
                t.name,
                dy(bold),
                dy(t.base_content)
            );
        }
        // Never-amber, identity form (ladder greys can't carry the accent's
        // hue by construction; the assertable half is non-identity).
        if !t.is_monochrome() {
            for (label, c) in [("Muted", muted_rung), ("Bold", bold)] {
                assert_ne!(
                    c, t.primary,
                    "{}: dial-up placard {label} ink must never be literally the accent",
                    t.name
                );
            }
        }
    }
    set_active(DEFAULT_THEME);
}
