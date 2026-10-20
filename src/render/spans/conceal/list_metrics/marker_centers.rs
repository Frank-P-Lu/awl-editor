//! Optical marker anchors measured with the same face, size and ink scanner as paint.
use super::*;

pub(super) fn shape(fonts: &mut FontSystem, metrics: Metrics, slot: f32) -> [f32; 5] {
    let world = theme::active();
    let mut swash = glyphon::SwashCache::new();
    std::array::from_fn(|index| {
        let (glyph, scale) = if index < 3 {
            let glyph = world.bullet_for_depth(index);
            (glyph, world.bullet_scale_for(glyph))
        } else {
            (theme::task_marker(index == 4), theme::TASK_MARKER_SCALE)
        };
        let mut buffer = GlyphBuffer::new(
            fonts,
            GlyphMetrics::new(metrics.font_size * scale, metrics.line_height),
        );
        buffer.set_size(fonts, Some(slot), Some(metrics.line_height));
        buffer.set_text(
            fonts,
            &glyph.to_string(),
            &Attrs::new().family(Family::Name(world.bullet_face)),
            Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Center),
        );
        buffer.shape_until_scroll(fonts, false);
        let keys = crate::rotated_label::ink::buffer_key(&buffer);
        let width =
            crate::rotated_label::ink::bounds(fonts, &mut swash, &keys).map_or(0.0, |ink| ink[2]);
        slot - metrics.font_size * ListLayoutMetrics::PROSE_GAP_EM - width * 0.5
    })
}
