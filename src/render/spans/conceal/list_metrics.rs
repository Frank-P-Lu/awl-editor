//! Font-derived dimensions for the concealed list-marker rail.

use super::*;

/// Exact advances that define the shared list rail. Both values are shaped in
/// the same registered faces and body metrics as their former source spans;
/// the hanging-layout inset and marker painter consume this one measurement.
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct ListLayoutMetrics {
    pub(in crate::render) marker_slot: f32,
    indent_space: f32,
}

impl ListLayoutMetrics {
    pub(in crate::render) fn shape(
        font_system: &mut FontSystem,
        metrics: Metrics,
        body_family: &'static str,
        mono_family: &'static str,
    ) -> Self {
        let advance = |font_system: &mut FontSystem, family, tracking| {
            let mut buffer = GlyphBuffer::new(
                font_system,
                GlyphMetrics::new(metrics.font_size, metrics.line_height),
            );
            let attrs = Attrs::new()
                .family(Family::Name(family))
                .letter_spacing(tracking);
            buffer.set_size(font_system, None, Some(metrics.line_height));
            buffer.set_text(font_system, " ", &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(font_system, false);
            buffer.layout_runs().next().map_or(0.0, |run| run.line_w)
        };
        let indent_space = {
            let mut buffer = GlyphBuffer::new(
                font_system,
                GlyphMetrics::new(metrics.font_size, metrics.line_height),
            );
            // Match the actual document body's face selection. In particular,
            // bundled IBM Plex Mono is registered as Light (300); a default-400
            // request falls through to a proportional system face and measures
            // the wrong space even though the family name still looks correct.
            let attrs = Attrs::new()
                .family(Family::Name(body_family))
                .weight(mono_safe_weight(body_family));
            buffer.set_size(font_system, None, Some(metrics.line_height));
            // Bind whitespace to that selected body face with a visible sentinel,
            // then read its two-space boundary. Spaces do not kern with the
            // sentinel, so half the boundary is the body's one-space advance.
            buffer.set_text(font_system, "  x", &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(font_system, false);
            buffer
                .layout_runs()
                .next()
                .and_then(|run| run.glyphs.iter().find(|glyph| glyph.start == 2))
                .map_or(0.0, |sentinel| sentinel.x * 0.5)
                .max(1.0)
        };
        Self {
            marker_slot: advance(
                font_system,
                mono_family,
                super::list::LIST_MARKER_GAP_TRACKING,
            )
            .max(1.0),
            indent_space,
        }
    }

    pub(in crate::render) fn hanging_inset(self, indent: usize, indent_scale: f32) -> f32 {
        self.marker_slot + indent as f32 * self.indent_space * indent_scale
    }
}
