//! Find/Replace is a compact field, a navigation row, and a quiet footer.
//! Text is measured before composition: spaces reserve pixel distances rather
//! than guessing where proportional labels or minimum-width targets will end.

use super::*;

pub(in crate::render) const PANEL_PAD: Logical = Logical(20.0);
pub(in crate::render) const PANEL_MARGIN: Logical = Logical(16.0);
pub(in crate::render) const PANEL_MIN_W: Logical = Logical(480.0);

/// A bounded rich-text composer. Each gap is a measured monospace space with
/// its own advance; all visible text retains the world's face and size role.
struct PanelText<'a> {
    fonts: &'a mut FontSystem,
    probe: glyphon::Buffer,
    spans: Vec<(String, Attrs<'static>)>,
    metrics: GlyphMetrics,
    space: f32,
    row: f32,
    byte: usize,
    x: f32,
}

impl<'a> PanelText<'a> {
    fn new(fonts: &'a mut FontSystem, metrics: GlyphMetrics) -> Self {
        let probe = glyphon::Buffer::new(fonts, metrics);
        let mut out = Self {
            fonts,
            probe,
            spans: Vec::new(),
            metrics,
            space: 1.0,
            row: 0.0,
            byte: 0,
            x: 0.0,
        };
        out.space = out.measure(" ", &Attrs::new().family(Family::Monospace));
        out
    }

    fn measure(&mut self, text: &str, attrs: &Attrs<'static>) -> f32 {
        self.probe
            .set_text(self.fonts, text, attrs, Shaping::Advanced, None);
        self.probe.shape_until_scroll(self.fonts, false);
        self.probe
            .layout_runs()
            .map(|r| r.line_w)
            .fold(0.0, f32::max)
    }

    fn push(&mut self, text: &str, attrs: Attrs<'static>) -> ControlSpan {
        let span = ControlSpan {
            row: self.row,
            byte_start: self.byte,
            byte_end: self.byte + text.len(),
        };
        self.x += self.measure(text, &attrs);
        self.byte = span.byte_end;
        self.spans.push((text.to_owned(), attrs));
        span
    }

    fn gap(&mut self, width: f32) {
        if width <= 0.0 {
            return;
        }
        let attrs = Attrs::new()
            .family(Family::Monospace)
            .letter_spacing((width - self.space) / self.metrics.font_size);
        self.spans.push((" ".into(), attrs));
        self.byte += 1;
        self.x += width;
    }

    fn newline(&mut self) {
        self.spans.push(("\n".into(), Attrs::new()));
        self.row += 1.0;
        self.byte = 0;
        self.x = 0.0;
    }

    /// Stretch the shaped field's trailing whitespace to one shared edge.
    /// The published span, border, and hit target all read this same advance.
    fn field(&mut self, text: &str, attrs: Attrs<'static>, right: f32, pad: f32) -> ControlSpan {
        let span = self.push(text, attrs);
        self.gap((right - pad - self.x).max(0.0));
        ControlSpan {
            byte_end: self.byte,
            ..span
        }
    }

    /// Center a label in an honest minimum-width target. The gaps belong to
    /// layout, while the published control continues to name its visible ink.
    fn button(&mut self, text: &str, attrs: Attrs<'static>, min: f32, pad: f32) -> ControlSpan {
        let ink = self.measure(text, &attrs);
        let side = ((min - ink) * 0.5).max(pad);
        self.gap(side);
        let span = self.push(text, attrs);
        self.gap(side);
        span
    }
}

impl TextPipeline {
    pub(in crate::render) fn prepare_panel(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        crate::render::unpin_picker_chrome();
        let shape = self.panel_shape_text(width);
        let (card, left, top, caret_x) = self.panel_layout(
            width,
            shape.caret_byte,
            shape.caret_fallback_chars,
            shape.caret_row,
        );
        self.panel_upload_text(device, queue, width, height, &shape, card, left, top)?;
        self.panel_place_selection(device, queue, (width, height), &shape, left, top);
        self.panel_place_caret(queue, width, height, caret_x, top, shape.caret_row);
        Ok(())
    }

    /// One fixed card width, bounded by the canvas; typing never changes it.
    pub(in crate::render) fn panel_card_width(&self, width: u32) -> f32 {
        let m = self.metrics.panel_ui();
        m.px(PANEL_MIN_W)
            .min((width as f32 - 2.0 * m.px(PANEL_MARGIN)).max(0.0))
    }

    pub(in crate::render) fn panel_shape_text(&mut self, width: u32) -> PanelShape {
        let m = self.metrics.panel_ui();
        let inner = (self.panel_card_width(width) - 2.0 * m.px(PANEL_PAD)).max(m.char_width);
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let red = theme::error().to_glyphon();
        let no_match = self.search_no_matches();
        let label = panel_attrs().metrics(GlyphMetrics::new(
            m.font_size * crate::markdown::type_scale::LABEL,
            m.line_height,
        ));
        let field = Attrs::new().family(Family::Monospace).color(ink);
        let symbol = Attrs::new()
            .family(Family::Name(SYMBOL_FAMILY))
            .metrics(GlyphMetrics::new(
                m.font_size * crate::markdown::type_scale::LABEL,
                m.line_height,
            ));
        let field_mark_range = self.search_field_mark_range();
        let mut t = PanelText::new(&mut self.font_system, m.glyph_metrics());
        let gap = m.px(Logical(12.0));
        let pad = m.px(panel_controls::CONTROL_BOX_PAD_X);
        let target = m.px(panel_controls::CONTROL_MIN_W);
        let stacked = inner < m.px(Logical(300.0));
        let label_w = t.measure("Replace", &label) + gap;
        let field_left = if stacked { 0.0 } else { label_w };
        let field_room = (inner - field_left - target - gap).max(m.char_width);
        let mut cap = (field_room / t.space).floor().max(2.0) as usize - 1;
        let query = self.search_query.clone();
        let replacement = self.search_replacement.clone();
        // Fallback glyphs (including CJK) can be wider than the mono cell.
        // Fit the actual windows, preserving the same scroll rule for both.
        let (query_view, query_caret, replacement_view, replacement_caret) = loop {
            let (q, qc) = field_view_window(&query, self.search_query_caret, cap);
            let (r, rc) = field_view_window(&replacement, self.search_replacement_caret, cap);
            if cap <= 1 || t.measure(&q, &field).max(t.measure(&r, &field)) + t.space <= field_room
            {
                break (q, qc, r, rc);
            }
            cap -= 1;
        };
        let field_right = field_left
            + t.measure(&query_view, &field)
                .max(t.measure(&replacement_view, &field))
            + pad;
        let mut controls = PanelControlSpans {
            stacked_fields: stacked,
            ..Default::default()
        };
        t.push("Find", label.clone().color(muted));
        if stacked {
            t.newline();
        } else {
            t.gap(field_left - t.x);
        }
        let find = t.field(&query_view, field.clone(), field_right, pad);
        controls.find_field = Some(find);
        t.push(" ", field.clone());
        t.gap(inner - target - t.x);
        controls.close = Some(t.button("×", symbol.clone().color(muted), target, pad));
        t.newline();
        let total = self.search_matches.len();
        let counter = if query.is_empty() {
            String::new()
        } else if total == 0 {
            "No matches".into()
        } else {
            format!("{} of {total}", self.search_current.map_or(0, |i| i + 1))
        };
        let nav_ink = if total == 0 {
            theme::faint().to_glyphon()
        } else {
            ink
        };
        let case_label = if self.search_case_sensitive {
            "☑ Match case"
        } else {
            "☐ Match case"
        };
        let case_attrs = label.clone().color(if self.search_case_sensitive {
            ink
        } else {
            muted
        });
        let case_w = t.measure(case_label, &case_attrs);
        t.gap(pad);
        controls.case_box = Some(t.push(case_label, case_attrs));
        let arrow_gap = m.px(Logical(4.0));
        let prev_w = (t.measure("↑", &symbol) + 2.0 * pad).max(target);
        let next_w = (t.measure("↓", &symbol) + 2.0 * pad).max(target);
        let cluster_w = t.measure(&counter, &label) + gap + prev_w + arrow_gap + next_w;
        // Match case belongs at the quiet left edge. The count and navigation
        // belong together below the search field, ending at that field's edge.
        if field_right - cluster_w < case_w + 2.0 * pad + gap {
            t.newline();
        }
        t.gap((field_right - cluster_w - t.x).max(0.0));
        t.push(
            &counter,
            label.clone().color(if no_match { red } else { muted }),
        );
        t.gap(gap);
        controls.nav_prev = Some(t.button("↑", symbol.clone().color(nav_ink), target, pad));
        t.gap(arrow_gap);
        controls.nav_next = Some(t.button("↓", symbol.clone().color(nav_ink), target, pad));

        if self.search_replace_active {
            t.newline();
            t.push("Replace", label.clone().color(muted));
            if stacked {
                t.newline();
            } else {
                t.gap(field_left - t.x);
            }
            controls.replace_field =
                Some(t.field(&replacement_view, field.clone(), field_right, pad));
            t.push(" ", field.clone());
            t.newline();
            let first = (t.measure("Replace", &label) + 2.0 * pad).max(target);
            let all = (t.measure("Replace all", &label) + 2.0 * pad).max(target);
            let actions_fit = first + gap + all <= field_right;
            t.gap(
                (field_right
                    - if actions_fit {
                        first + gap + all
                    } else {
                        first
                    })
                .max(0.0),
            );
            controls.replace_button =
                Some(t.button("Replace", label.clone().color(nav_ink), target, pad));
            if actions_fit {
                t.gap(gap);
            } else {
                t.newline();
                t.gap((field_right - all).max(0.0));
            }
            controls.replace_all_button =
                Some(t.button("Replace all", label.clone().color(nav_ink), target, pad));
        }
        t.newline();
        let disclosure = if self.search_replace_active {
            "Hide replace"
        } else {
            "Replace…"
        };
        t.gap(pad);
        controls.reveal = Some(t.push(disclosure, label.clone().color(muted)));
        t.gap(pad);
        let hint = format!(
            "{} field   {} close",
            crate::keyspec::PANEL_SWITCH_FIELD.label(),
            crate::keyspec::PANEL_CLOSE.label()
        );
        let hint_w = t.measure(&hint, &label);
        if t.x + gap + hint_w > inner {
            t.newline();
        }
        t.gap((inner - hint_w - t.x).max(gap));
        t.push(&hint, label.clone().color(muted));

        let editing = self.search_replace_active && self.search_editing_replacement;
        let (span, view, caret, full_caret, full_len) = if editing {
            (
                controls.replace_field.unwrap(),
                &replacement_view,
                replacement_caret,
                self.search_replacement_caret,
                replacement.chars().count(),
            )
        } else {
            (
                find,
                &query_view,
                query_caret,
                self.search_query_caret,
                query.chars().count(),
            )
        };
        let prefix = " ".repeat(span.byte_start);
        let selection_span =
            panel_selection_span(field_mark_range, &prefix, view, full_caret, full_len, cap);
        let rows = t.row + 1.0;
        let spans = t.spans;
        self.panel_buffer
            .set_metrics(&mut self.font_system, m.glyph_metrics());
        self.panel_buffer.set_size(
            &mut self.font_system,
            Some(width as f32 * 2.0),
            Some(m.line_height * rows),
        );
        self.panel_buffer.set_rich_text(
            &mut self.font_system,
            spans
                .iter()
                .map(|(text, attrs)| (text.as_str(), attrs.clone())),
            &field,
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
        self.panel_control_spans = controls;
        PanelShape {
            no_match,
            ink,
            red,
            caret_byte: span.byte_start + field_caret_byte(view, caret),
            caret_fallback_chars: span.byte_start + caret,
            caret_row: span.row,
            selection_span,
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn panel_field_rows_probe(&self) -> (Option<f32>, Option<f32>) {
        (
            self.panel_control_spans.find_field.map(|s| s.row),
            self.panel_control_spans.replace_field.map(|s| s.row),
        )
    }
}
