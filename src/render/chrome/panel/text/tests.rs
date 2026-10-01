use super::*;
use crate::render::{FONT_DATA, SYMBOL_FAMILY, register_default_font};

#[test]
fn close_keeps_its_bundled_face_without_host_fonts_or_default_mono() {
    let _guard = crate::testlock::serial();
    let override_bytes: &[u8] = include_bytes!("../../../../../assets/fonts/Iosevka-Regular.ttf");
    for (bytes, expected_mono) in [(FONT_DATA, "IBM Plex Mono"), (override_bytes, "Iosevka")] {
        let mut fonts = FontSystem::new();
        let ids: Vec<_> = fonts.db().faces().map(|face| face.id).collect();
        for id in ids {
            fonts.db_mut().remove_face(id);
        }
        register_default_font(&mut fonts, bytes.to_vec());
        assert_eq!(
            fonts
                .db()
                .family_name(&glyphon::cosmic_text::fontdb::Family::Monospace),
            expected_mono,
            "registering the close face must preserve the selected mono family"
        );
        let metrics = GlyphMetrics::new(24.0, 36.0);
        let mut composer = PanelText::new(&mut fonts, metrics);
        // Deliberately request the subset that has no ×. Production close
        // composition must replace that request before measurement and shaping.
        let color = glyphon::Color::rgb(139, 145, 157);
        let attrs = Attrs::new()
            .family(Family::Name(SYMBOL_FAMILY))
            .metrics(GlyphMetrics::new(19.2, 36.0))
            .color(color);
        composer.close_button(attrs, 32.0, 6.0, 4.0);
        assert!(
            (composer.x - 32.0).abs() < 0.01,
            "keep the approved target width"
        );
        let spans = std::mem::take(&mut composer.spans);
        drop(composer);
        let mut buffer = glyphon::Buffer::new(&mut fonts, metrics);
        buffer.set_rich_text(
            &mut fonts,
            spans
                .iter()
                .map(|(text, attrs)| (text.as_str(), attrs.clone())),
            &Attrs::new(),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let at = buffer.lines[0]
            .text()
            .find('×')
            .expect("close glyph remains text");
        let glyph = buffer
            .layout_runs()
            .flat_map(|run| run.glyphs)
            .find(|glyph| glyph.start <= at && glyph.end > at)
            .expect("close glyph is shaped without a host fallback");
        assert_ne!(glyph.glyph_id, 0, "the close mark must not be tofu");
        assert_eq!(glyph.color_opt, Some(color), "preserve inherited muted ink");
        let face = fonts
            .db()
            .face(glyph.font_id)
            .expect("shaped close face exists");
        assert!(
            face.families
                .iter()
                .any(|(name, _)| name == "IBM Plex Mono")
        );
        assert_eq!(face.weight, glyphon::cosmic_text::fontdb::Weight(300));
        assert_eq!(
            fonts
                .db()
                .with_face_data(glyph.font_id, |data, _| data == FONT_DATA),
            Some(true),
            "the close mark must use bundled data, even with AWL_FONT selected"
        );
    }
}
