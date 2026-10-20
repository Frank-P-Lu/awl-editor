//! Destination geometry for entering a revealed Markdown line.
use super::*;

impl TextPipeline {
    /// Read only the destination's retained attrs before any document/span work.
    /// Transparent source or a preview inset is the only geometry that entering
    /// the line can expand. Plain and selection-revealed lines keep their cache.
    pub(in crate::render) fn needs_reveal_shape(&self, line: usize) -> bool {
        self.md_enabled
            && line != self.cursor_line
            && self.buffer.lines.get(line).is_some_and(|line| {
                line.hanging_inset() > 0.0
                    || line
                        .attrs_list()
                        .spans_iter()
                        .any(|(_, attrs)| attrs.color_opt.is_some_and(|color| color.a() == 0))
            })
    }

    /// Shape just the destination line as it will appear after vertical motion
    /// reveals its source. Hit-testing the concealed preview instead moves the
    /// caret horizontally when markup expands, and can even skip a wrapped row.
    /// This uses the document's font owner and attrs builder without changing
    /// the visible buffer or concealing the origin before goal-x is sampled.
    pub(in crate::render) fn revealed_line_rows(&mut self, line: usize) -> Vec<VisualRow> {
        if !self.needs_reveal_shape(line) {
            return self.line_rows_local(line);
        }
        let Some(text) = self.buffer.lines.get(line).map(|l| l.text().to_owned()) else {
            return self.line_rows_local(line);
        };
        let attrs = self.doc_attrs();
        let fonts = self.cache_script_fonts();
        let priority = crate::script::effective_cjk_priority(self.han_evidence, &self.cjk_priority);
        let start = self.line_doc_byte_start(line);
        let ctx = LineAttrsCtx {
            base: &attrs,
            base_font_size: self.metrics.font_size,
            base_line_height: self.metrics.line_height,
            md: self.md_enabled,
            md_spans: &self.md_spans,
            list_continuations: &self.list_layout.continuations,
            syn_spans: &self.syn_spans,
            doc_lang: self.doc_lang,
            cjk_evidence: self.han_evidence,
            cjk_priority: &priority,
            fonts: &fonts,
            cursor_byte: start,
            selection_touch: None,
            substitute_advances: self.substitute_advances,
        };
        let al = build_line_attrs(
            &ctx,
            &text,
            start,
            false,
            self.image_heights.get(line).copied().flatten(),
            self.image_force.get(line).copied().flatten(),
        );
        let mut buf = GlyphBuffer::new_empty(GlyphMetrics::new(
            self.metrics.font_size,
            self.metrics.line_height,
        ));
        buf.set_wrap(&mut self.font_system, Wrap::WordOrGlyph);
        let width = self.text_wrap_width();
        buf.set_size(&mut self.font_system, Some(width), None);
        buf.lines.push(glyphon::cosmic_text::BufferLine::new(
            &text,
            glyphon::cosmic_text::LineEnding::None,
            al,
            Shaping::Advanced,
        ));
        buf.shape_until_scroll(&mut self.font_system, false);
        let rows: Vec<_> = buf
            .layout_runs()
            .map(|run| super::geometry::visual_row_from_run(&text, &run, self.metrics.char_width))
            .collect();
        if rows.is_empty() {
            self.line_rows_local(line)
        } else {
            rows
        }
    }
}
