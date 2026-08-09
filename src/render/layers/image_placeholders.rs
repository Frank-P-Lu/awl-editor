use super::*;

pub(super) struct MissingImagePlaceholder {
    pub(super) dst: [f32; 4],
    pub(super) path: String,
    pub(super) alt: String,
}

impl TextPipeline {
    /// Shape the filename and optional alt labels for missing image placeholders.
    /// The caller retains these buffers until glyphon's preparation has borrowed
    /// them into text areas.
    pub(super) fn build_missing_placeholder_text_buffers(
        &mut self,
        missing: &[MissingImagePlaceholder],
    ) -> Vec<(GlyphBuffer, f32, f32, glyphon::Color)> {
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let line_h = m.line_height * label;
        let muted = theme::muted().to_glyphon();
        let faint = theme::faint().to_glyphon();
        let center = Some(glyphon::cosmic_text::Align::Center);
        let name_attrs = self.doc_attrs().color(muted);
        let alt_attrs = self.doc_attrs().color(faint);
        let mut buffers = Vec::new();
        for placeholder in missing {
            let filename = std::path::Path::new(&placeholder.path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(placeholder.path.as_str());
            let box_w = placeholder.dst[2].max(1.0);
            let box_left = placeholder.dst[0];
            let box_top = placeholder.dst[1];
            let box_h = placeholder.dst[3];
            let two = !placeholder.alt.trim().is_empty();
            let block_h = if two { line_h * 2.0 } else { line_h };
            let start_y = box_top + (box_h - block_h).max(0.0) * 0.5;
            let mut name_buf = GlyphBuffer::new(&mut self.font_system, gm);
            name_buf.set_size(&mut self.font_system, Some(box_w), Some(line_h));
            name_buf.set_text(
                &mut self.font_system,
                filename,
                &name_attrs,
                Shaping::Advanced,
                center,
            );
            name_buf.shape_until_scroll(&mut self.font_system, false);
            buffers.push((name_buf, box_left, start_y, muted));
            if two {
                let mut alt_buf = GlyphBuffer::new(&mut self.font_system, gm);
                alt_buf.set_size(&mut self.font_system, Some(box_w), Some(line_h));
                alt_buf.set_text(
                    &mut self.font_system,
                    placeholder.alt.trim(),
                    &alt_attrs,
                    Shaping::Advanced,
                    center,
                );
                alt_buf.shape_until_scroll(&mut self.font_system, false);
                buffers.push((alt_buf, box_left, start_y + line_h, faint));
            }
        }
        buffers
    }
}
