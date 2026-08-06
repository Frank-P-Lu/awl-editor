use super::super::*;

/// WRITING-STREAKS HEATMAP tint law (`heatmap_colors`, one owner in
/// `derive.rs`): for EVERY world the calendar squares must be figure/ground by
/// value — the filled intensity rungs distinguishable FROM the card's own
/// `base_300` ground AND from each other, climbing monotonically toward ink, and
/// NEVER amber (the caret's alone). One-bit worlds (Wagtail) carry a DECLARED
/// EXEMPTION: no intermediate grey is permitted there, so the heatmap degrades to
/// BINARY (empty = ground, any writing = full ink), which the arm below asserts is
/// pure black/white instead of a 5-step ramp.
#[test]
fn streaks_heatmap_levels_are_distinguishable_every_world() {
    let _g = crate::testlock::serial();
    // Gamma-correct Rec.709 relative luminance (the `rel_lum` recipe used across
    // the theme laws) — perceived brightness, so "distinguishable" is perceptual.
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
    // A filled rung must clear the ground by at least this perceived-luminance
    // step, and each rung must clear the previous by at least this much — small
    // but non-zero, so even the tightest world's ink span reads as 4 steps.
    const MIN_STEP: f32 = 0.012;
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        let colors = heatmap_colors();
        assert_eq!(colors.len(), crate::streaks::LEVELS);
        let ground = base_300();

        if t.is_one_bit() {
            // 1-BIT DEGRADATION: empty = ground token, every filled rung = full ink,
            // and both are pure (no forbidden intermediate grey). A written cell must
            // still read against the ground.
            assert_eq!(
                colors[0],
                base_200(),
                "{}: 1-bit empty rung is the ground token",
                t.name
            );
            for c in &colors[1..] {
                assert_eq!(
                    *c,
                    base_content(),
                    "{}: 1-bit filled rung is full ink",
                    t.name
                );
            }
            assert!(
                (rel_lum(colors[4]) - rel_lum(ground)).abs() > 0.5,
                "{}: 1-bit written cell reads against the ground",
                t.name
            );
            continue;
        }

        // The direction ink climbs away from the ground (light world: down; dark:
        // up). Every filled rung must move in that one direction, monotonically.
        let ink_dir = (rel_lum(base_content()) - rel_lum(ground)).signum();
        let mut prev = rel_lum(colors[0]); // the empty rung
        for (i, c) in colors.iter().enumerate().skip(1) {
            let y = rel_lum(*c);
            // Distinguishable from the ground.
            assert!(
                (y - rel_lum(ground)).abs() >= MIN_STEP,
                "{}: filled level {i} (Y {:.4}) not distinguishable from base_300 ground (Y {:.4})",
                t.name,
                y,
                rel_lum(ground)
            );
            // Distinguishable from — and climbing past — the previous rung.
            assert!(
                (y - prev) * ink_dir >= MIN_STEP,
                "{}: level {i} (Y {:.4}) is not a clear step up from level {} (Y {:.4})",
                t.name,
                y,
                i - 1,
                prev
            );
            prev = y;
            // NEVER THE ACCENT: every square rides the world's own ink ladder (a
            // blend of `base_200`↔`base_content`), so it can never manufacture
            // chroma BEYOND that ladder — the caret's saturated accent is never a
            // decorative fill here. A warm world's `base_content` legitimately
            // shares the accent's HUE (it's the reading ink every glyph uses), so
            // the guard bounds SATURATION to the ladder's own, not the hue: a cell
            // must be no more saturated than the ink endpoints, and never literally
            // `primary`.
            let (_, s, _) = c.to_hsl();
            let (_, s_ink, _) = base_content().to_hsl();
            let (_, s_empty, _) = base_200().to_hsl();
            assert!(
                s <= s_ink.max(s_empty) + 0.02,
                "{}: heatmap level {i} (sat {:.2}) manufactures chroma beyond the ink ladder (max {:.2})",
                t.name,
                s,
                s_ink.max(s_empty)
            );
            // On an INK-CARET world the brightest rung IS the ink IS the accent
            // (`primary == base_content`; Wagtail's white, Cassowary's phosphor),
            // so "never literally primary" is structurally inapplicable — the
            // heatmap climbing to the full ink is the intended top level.
            if !t.ink_caret() {
                assert_ne!(
                    *c,
                    primary(),
                    "{}: heatmap level {i} must never be literally the accent",
                    t.name
                );
            }
        }
    }
    set_active(DEFAULT_THEME);
}

/// A PAIRED-LOAD gate. A world restore that escapes the serialized window
/// corrupts an unrelated, fully compliant law — this one was that victim.
/// Exercising the explicit pin and the runtime choke point together forty
/// times makes such a restore fail at the writer seam itself.
#[test]
fn streaks_heatmap_law_is_stable_across_forty_world_pin_windows() {
    let _g = crate::testlock::serial();
    let _suite_world = WorldPin::snapshot();
    for pair in 0..40 {
        set_active(pair % THEMES.len());
        {
            let _world = WorldPin::snapshot();
            streaks_heatmap_levels_are_distinguishable_every_world();
        }
        assert_eq!(
            active_index(),
            pair % THEMES.len(),
            "pair {pair}: the explicit pin restores inside the held window"
        );
    }
}
