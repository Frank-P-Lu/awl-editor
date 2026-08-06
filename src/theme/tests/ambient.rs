use super::super::*;

/// THE TWINKLING-STARS LAWS (2026-07-18 — the "aliveness ≠ loudness" round;
/// RE-SCOPED 2026-07-23 for the LIFECYCLE round). Every world's
/// `render_caps.ambient` is swept with a NO-WILDCARD match (a future
/// `AmbientStyle` variant fails to compile until it's under the law). Every tint
/// is drawn from the world's own star PALETTE ([`crate::stars::star_palette`] —
/// blue-white / white / champagne), the ONE owner the renderer draws from too,
/// and each palette entry is fenced. For a `Stars` world, four fences — the same
/// shapes that fence the lava:
///
/// (a) **VISIBILITY BAND (the RELAXED, user-blessed brightness ceiling).** THE
///     LIFECYCLE round loosens the old `<= muted` whisper cap: a star's shine
///     may now rise ABOVE the muted rung (a real glint, not a whisper), but its
///     PEAK composited pixel — each palette tint alpha-blended in LINEAR light
///     over each margin-ground endpoint, exactly the GPU's SrcAlpha blend —
///     still stays STRICTLY UNDER the `base_content` (text-ink) deviation: the
///     figure stays the text's, a star never outshines the prose. And the
///     relaxation is REAL, not vestigial: the BRIGHTEST palette tint at peak
///     genuinely exceeds the muted whisper cap over the darker ground (else the
///     old cap would still bind and nothing changed). Proven over COMPOSITED
///     values, never authored bytes (the Saltpan/camouflage lesson).
/// (b) **VISIBLE, not the invisible-band trap.** A star lit only to the band
///     FLOOR still composites at least ΔY 0.02 off its local ground — the
///     dimmest LIT star is genuinely seeable (a star that composites to nothing
///     would pass every mechanism test while the sky ships empty — the Wagtail
///     invisible-row lesson). (The DWELL is a separate, deliberate true-zero;
///     the band bounds a star while it is LIT.)
/// (c) **AMBER GUARD.** Every palette tint: a chromatic one (HSL sat > 0.15)
///     sits ≥ 30° of hue from the world's `primary`, and none is literally
///     `primary` — the one-accent law (DESIGN §3): the caret stays the only warm
///     thing. (Champagne holds this by low saturation despite its warm hue.)
/// (d) **ONE-BIT GUARD.** A star's alpha is FRACTIONAL by construction —
///     structurally illegal on a true 1-bit world (any intermediate composite is
///     a forbidden third value), so `Stars` on an `is_one_bit()` world fails
///     here before a render could paint it. (A one-bit sky would need a
///     dither-stipple star mode — banked.)
///
/// Param sanity rides along: band ordered (`0 < floor < peak <= 1`), density in
/// `(0, 1]`, and the dot small enough for its cell's jitter band.
#[test]
fn ambient_stars_laws_hold_for_every_world() {
    fn lin(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    fn rel_lum(c: Srgb) -> f32 {
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }
    fn hue_gap(a: f32, b: f32) -> f32 {
        let d = (a - b).abs() % 360.0;
        d.min(360.0 - d)
    }
    // The GPU blend (linear-space SrcAlpha over) applied to luminance — linear
    // light is additive, so Y composites exactly.
    fn composite_y(tint: Srgb, alpha: f32, ground: Srgb) -> f32 {
        alpha * rel_lum(tint) + (1.0 - alpha) * rel_lum(ground)
    }
    let mut stars_worlds = 0usize;
    for t in THEMES.iter() {
        match t.render_caps.ambient {
            model::AmbientStyle::None => continue,
            model::AmbientStyle::Stars {
                tint,
                cell_px,
                density,
                size_px,
                peak,
                floor,
            } => {
                stars_worlds += 1;
                // Param sanity.
                assert!(
                    0.0 < floor && floor < peak && peak <= 1.0,
                    "{}: the visibility band must be ordered (0 < floor {floor} < peak {peak} <= 1)",
                    t.name
                );
                assert!(
                    (0.0..=1.0).contains(&density) && density > 0.0,
                    "{}: density {density} out of (0, 1]",
                    t.name
                );
                assert!(
                    size_px > 0.0 && size_px < cell_px * 0.3,
                    "{}: dot {size_px}px must stay well inside its {cell_px}px cell's jitter band",
                    t.name
                );
                // SIZE SPREAD: even the WIDEST star the spread ever
                // draws — `size_px * (1 + STAR_SIZE_SPREAD_FRAC)` — must still
                // clear the same cell-jitter safety margin above, not just the
                // authored base size.
                let widest = size_px * (1.0 + crate::stars::STAR_SIZE_SPREAD_FRAC);
                assert!(
                    widest < cell_px * 0.3,
                    "{}: the widest spread star {widest}px must stay well inside its \
                     {cell_px}px cell's jitter band too",
                    t.name
                );
                // (d) ONE-BIT GUARD.
                assert!(
                    !t.is_one_bit(),
                    "{}: a fractional-alpha star is structurally illegal on a true \
                     1-bit world (any intermediate composite is a forbidden third value)",
                    t.name
                );
                // The PALETTE the renderer draws from IS the law's subject — one owner.
                let palette = crate::stars::star_palette(tint);
                let (ph, _ps, _pl) = t.primary.to_hsl();
                for st in palette {
                    // (c) AMBER GUARD, per palette entry.
                    assert_ne!(
                        st, t.primary,
                        "{}: a star tint must never BE the accent",
                        t.name
                    );
                    let (sh, ss, _sl) = st.to_hsl();
                    if ss > 0.15 {
                        let gap = hue_gap(sh, ph);
                        assert!(
                            gap >= 30.0,
                            "{}: star tint hue {sh:.0}° sits only {gap:.0}° from the caret's \
                             {ph:.0}° — a second accent (DESIGN §3)",
                            t.name
                        );
                    }
                }
                // (a)+(b) VISIBILITY BAND, per palette entry, per ground endpoint.
                let muted_dev = (rel_lum(t.muted) - rel_lum(t.base_100)).abs();
                let content_dev = (rel_lum(t.base_content) - rel_lum(t.base_100)).abs();
                // The BRIGHTEST palette tint drives the relaxation-is-real check.
                let brightest = palette
                    .into_iter()
                    .max_by(|a, b| rel_lum(*a).partial_cmp(&rel_lum(*b)).unwrap())
                    .unwrap();
                let mut relaxation_seen = false;
                for st in palette {
                    for (label, ground) in
                        [("from", t.background.from()), ("to", t.background.to())]
                    {
                        let gy = rel_lum(ground);
                        let peak_dev = (composite_y(st, peak, ground) - gy).abs();
                        // CALM CEILING: strictly under the text ink — the figure
                        // stays the prose's, however bright the glint.
                        assert!(
                            peak_dev < content_dev,
                            "{}: a peak star over the {label} ground deviates ΔY {peak_dev:.3} — \
                             not strictly under the text ink's {content_dev:.3}; a glint must \
                             never outshine the prose",
                            t.name
                        );
                        // VISIBLE FLOOR: the dimmest LIT star is still seeable.
                        let floor_dev = (composite_y(st, floor, ground) - gy).abs();
                        assert!(
                            floor_dev >= 0.02,
                            "{}: a floor (dimmest lit) star over the {label} ground deviates only \
                             ΔY {floor_dev:.3} — the invisible-band trap (lit but unseeable)",
                            t.name
                        );
                        assert!(
                            floor_dev < peak_dev,
                            "{}: the band must brighten from floor to peak (floor ΔY {floor_dev:.3} \
                             !< peak ΔY {peak_dev:.3})",
                            t.name
                        );
                        // RELAXATION IS REAL: the brightest tint at peak clears
                        // the old muted whisper cap somewhere (the deliberate,
                        // user-blessed loosening — else nothing actually changed).
                        if st == brightest && peak_dev > muted_dev {
                            relaxation_seen = true;
                        }
                    }
                }
                assert!(
                    relaxation_seen,
                    "{}: the brightest star's peak never exceeds the muted whisper cap \
                     ({muted_dev:.3}) on any ground — the LIFECYCLE round's blessed relaxation \
                     is vestigial (a real glint must rise above the old cap)",
                    t.name
                );
            }
        }
    }
    // The round's assignment: exactly ONE stars world ships (Currawong — the
    // user's pick). A second is a conscious data edit that lands here.
    assert_eq!(
        stars_worlds, 1,
        "exactly one world ships AmbientStyle::Stars today"
    );
}

/// THE SCHEDULING-GATE COMPOSITION law (mirrors `stars.rs`'s
/// `currawong_alone_carries_the_stars_and_the_ambient_gate_composes` — same
/// "one owner" shape): [`Theme::has_ambient_tick`] is EXACTLY
/// `has_ambient_motion() || background.is_waves()` for every world, no
/// per-world name comparison and no re-derived OR at a call site.
///
/// (1) Composition holds for all sixteen worlds.
/// (2) It is a STRICT SUPERSET of `has_ambient_motion` — the only worlds it
///     flips true that `has_ambient_motion` didn't are the `Waves` worlds
///     (today: Bombora alone).
/// (3) NON-VACUOUS: at least one world (Bombora) is flipped by the extra
///     `is_waves()` arm — the widening has a live consumer, not dead data.
#[test]
fn has_ambient_tick_composes_all_ambient_background_capabilities_one_owner() {
    let mut saw_a_flipped_waves_world = false;
    for t in THEMES.iter() {
        let motion = t.has_ambient_motion();
        let tick = t.has_ambient_tick();
        assert_eq!(
            tick,
            motion || t.background.is_waves() || t.background.is_organic(),
            "{}: has_ambient_tick must include every ambient background capability — one owner",
            t.name
        );
        // The superset property: has_ambient_tick can only ADD worlds relative
        // to has_ambient_motion, never remove one.
        assert!(
            !motion || tick,
            "{}: has_ambient_tick must stay true wherever has_ambient_motion is",
            t.name
        );
        if tick && !motion {
            saw_a_flipped_waves_world = true;
            assert!(
                t.background.is_waves() || t.background.is_organic(),
                "{}: tick-only grounds must be Waves or Organic",
                t.name
            );
        }
    }
    assert!(
        saw_a_flipped_waves_world,
        "at least one world (Bombora) must be a Waves world the widening actually reaches"
    );
    assert!(
        BOMBORA.background.is_waves(),
        "Bombora ships Background::Waves (item 69/87)"
    );
    assert!(
        BOMBORA.has_ambient_tick() && !BOMBORA.has_ambient_motion(),
        "Bombora joins the shared tick WITHOUT joining the auto-page-on/move-hold gate \
         (has_ambient_motion) — its ground was already an OPTIONAL margin decoration \
         (item 69), so item 87 must not silently force page mode on at launch"
    );
}

/// THE SCHEDULING law (mirrors the lava/stars precedents in
/// `lava::tests` / `stars.rs`): every freeze condition —
/// `ambient_motion` off, Reduce Motion, a paused (blurred/moving/resizing)
/// window, and a non-Bombora active world — closes `lava_should_tick`'s gate
/// for Bombora's wave drift, scheduling EXACTLY ZERO frames (no `WaitUntil`
/// re-arm, no phase advance, no redraw request — the whole ambient-tick `if`
/// body in `app/schedule.rs` is skipped). NON-VACUITY LIVES IN THE LAST CASE
/// (`ambient_motion` on, not reduced, focused, not paused), which must be
/// TRUE: feed `has_ambient_motion()` here instead of `has_ambient_tick()` and
/// every OTHER assertion still holds, for the trivial reason that Bombora can
/// never arm at all.
#[test]
fn bombora_wave_drift_schedules_zero_frames_under_every_freeze_condition() {
    let active = BOMBORA.has_ambient_tick();
    assert!(active, "Bombora must join the shared tick gate (item 87)");

    // ambient_motion = false.
    assert!(!crate::lava::lava_should_tick(
        active, false, false, true, false
    ));
    // Reduce Motion.
    assert!(!crate::lava::lava_should_tick(
        active, true, true, true, false
    ));
    // Paused: window blurred / mid-move / mid-resize (the shared `paused` OR).
    assert!(!crate::lava::lava_should_tick(
        active, true, false, true, true
    ));
    // Unfocused window.
    assert!(!crate::lava::lava_should_tick(
        active, true, false, false, false
    ));
    // A non-Bombora active world never arms at all (Wagtail: 1-bit, no ambient
    // capability of any kind).
    let other = WAGTAIL.has_ambient_tick();
    assert!(!other, "Wagtail carries no ambient ground of any kind");
    assert!(!crate::lava::lava_should_tick(
        other, true, false, true, false
    ));

    // Deterministic headless capture: `App::new_hermetic`'s / the capture
    // pipeline's `lava_phase` field starts (and, absent a live `about_to_wait`
    // tick, stays) at `LAVA_FROZEN_PHASE` — so the resolved drift is 0.0
    // regardless of the scheduling gate above. See
    // `render::pipeline_draw`'s `lava_phase: crate::lava::LAVA_FROZEN_PHASE`
    // initializer and `crate::background::waves_drift_radians`'s own
    // `drift_is_zero_at_the_settled_phase` law.
    assert_eq!(crate::lava::LAVA_FROZEN_PHASE, 0.0);

    // The genuinely NEW, non-vacuous case: every gate open, Bombora active —
    // the tick MUST arm (this is false for every OTHER world with no ambient
    // capability, and was unreachable for Bombora before this round).
    assert!(crate::lava::lava_should_tick(
        active, true, false, true, false
    ));
}

#[test]
fn bowerbird_organic_schedules_zero_frames_under_every_freeze_condition() {
    let active = BOWERBIRD.has_ambient_tick();
    assert!(active, "Organic must enroll in the shared ambient tick");
    for (ambient, reduced, focused, paused) in [
        (false, false, true, false),
        (true, true, true, false),
        (true, false, false, false),
        (true, false, true, true),
    ] {
        assert!(!crate::lava::lava_should_tick(
            active, ambient, reduced, focused, paused
        ));
    }
    assert!(crate::lava::lava_should_tick(
        active, true, false, true, false
    ));
}
