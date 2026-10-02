use super::super::*;

fn contrast(a: Srgb, b: Srgb) -> f64 {
    let lum = |c: Srgb| {
        0.2126 * srgb_channel_to_linear(c.r)
            + 0.7152 * srgb_channel_to_linear(c.g)
            + 0.0722 * srgb_channel_to_linear(c.b)
    };
    let (a, b) = (lum(a), lum(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn close_hover_keeps_the_worlds_accent_readable_and_distinct_on_both_row_surfaces() {
    let _g = crate::testlock::serial();
    let _pin = WorldPin::snapshot();
    let mut failures = Vec::new();
    let mut adjusted = 0;
    let mut cells = 0;
    for (index, world) in THEMES.iter().enumerate() {
        set_active(index);
        for selected in [false, true] {
            let band = if selected {
                surface_selected()
            } else {
                base_100()
            };
            let rest = if selected {
                selected_row_secondary_ink(band)
            } else {
                faint()
            };
            let ink = accent_ink(band, Some(rest));
            let label = format!("{} selected={selected}", world.name);
            eprintln!(
                "{label}: primary={} band={} rest={} hover={} contrast={:.3}",
                primary().hex(),
                band.hex(),
                rest.hex(),
                ink.hex(),
                contrast(ink, band)
            );
            if world.is_one_bit() {
                assert_eq!(ink, rest, "{label}: retain the readable one-bit no-op");
                assert!(contrast(ink, band) >= 3.0, "{label}: one-bit presence");
            } else {
                let (h, s, _) = ink.to_hsl();
                let (ph, ps, _) = primary().to_hsl();
                let hue_gap = ((h - ph + 180.0).rem_euclid(360.0) - 180.0).abs();
                let separation = ink
                    .rgb_bytes()
                    .into_iter()
                    .zip(rest.rgb_bytes())
                    .map(|(a, b)| (a as f32 - b as f32).powi(2))
                    .sum::<f32>()
                    .sqrt();
                if hue_gap > 2.0
                    || (s - ps).abs() > 0.03
                    || contrast(ink, band) < 3.0
                    || separation < 40.0
                {
                    failures.push(format!(
                        "{label}: hue gap {hue_gap:.1}, saturation {s:.3}/{ps:.3}, \
                         contrast {:.3}, rest distance {separation:.1}",
                        contrast(ink, band)
                    ));
                }
                if primary() != ink {
                    adjusted += 1;
                }
                if contrast(primary(), band) >= 3.0 {
                    let distance = primary()
                        .rgb_bytes()
                        .into_iter()
                        .zip(rest.rgb_bytes())
                        .map(|(a, b)| (a as f32 - b as f32).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    if distance >= 40.0 {
                        assert_eq!(
                            ink,
                            primary(),
                            "{label}: preserve an already suitable authored accent"
                        );
                    }
                }
            }
            cells += 1;
        }
    }
    assert_eq!(cells, THEMES.len() * 2);
    assert!(
        adjusted > 0,
        "the selected-plate correction must be exercised"
    );
    assert!(
        failures.is_empty(),
        "accent outcomes failed:\n{}",
        failures.join("\n")
    );
}
