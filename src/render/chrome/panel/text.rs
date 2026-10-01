//! Measured rich-text composition for the Find/Replace panel.

use super::{Attrs, ControlSpan, Family, FontSystem, GlyphMetrics, Shaping};

/// A bounded rich-text composer. Each gap is a measured monospace space with
/// its own advance; all visible text retains the world's face and size role.
pub(super) struct PanelText<'a> {
    fonts: &'a mut FontSystem,
    probe: glyphon::Buffer,
    pub(super) spans: Vec<(String, Attrs<'static>)>,
    metrics: GlyphMetrics,
    pub(super) space: f32,
    pub(super) row: f32,
    pub(super) byte: usize,
    pub(super) x: f32,
}

impl<'a> PanelText<'a> {
    pub(super) fn new(fonts: &'a mut FontSystem, metrics: GlyphMetrics) -> Self {
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

    pub(super) fn measure(&mut self, text: &str, attrs: &Attrs<'static>) -> f32 {
        self.probe
            .set_text(self.fonts, text, attrs, Shaping::Advanced, None);
        self.probe.shape_until_scroll(self.fonts, false);
        self.probe
            .layout_runs()
            .map(|r| r.line_w)
            .fold(0.0, f32::max)
    }

    pub(super) fn push(&mut self, text: &str, attrs: Attrs<'static>) -> ControlSpan {
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

    pub(super) fn gap(&mut self, width: f32) {
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

    pub(super) fn newline(&mut self) {
        self.spans.push(("\n".into(), Attrs::new()));
        self.row += 1.0;
        self.byte = 0;
        self.x = 0.0;
    }

    /// Stretch the shaped field's trailing whitespace to one shared edge.
    /// The published span, border, and hit target all read this same advance.
    pub(super) fn field(
        &mut self,
        text: &str,
        attrs: Attrs<'static>,
        right: f32,
        pad: f32,
    ) -> ControlSpan {
        let span = self.push(text, attrs.clone());
        // Reserve the caret cell inside the field, including its hit span.
        self.push(" ", attrs);
        self.gap((right - pad - self.x).max(0.0));
        ControlSpan {
            byte_end: self.byte,
            ..span
        }
    }

    /// Center a label in an honest minimum-width target. The gaps belong to
    /// layout, while the published control continues to name its visible ink.
    pub(super) fn button(
        &mut self,
        text: &str,
        attrs: Attrs<'static>,
        min: f32,
        pad: f32,
    ) -> ControlSpan {
        let ink = self.measure(text, &attrs);
        let side = ((min - ink) * 0.5).max(pad);
        self.gap(side);
        let span = self.push(text, attrs);
        self.gap(side);
        span
    }
    /// Caption controls reserve an actual icon slot and gap; visible marks are
    /// prepared separately against the caption's tight ink, not its baseline.
    pub(super) fn marked_caption(
        &mut self,
        text: &str,
        attrs: Attrs<'static>,
        slot_width: f32,
        gap: f32,
    ) -> (ControlSpan, ControlSpan, ControlSpan) {
        let start = self.byte;
        self.gap(slot_width);
        let slot = ControlSpan {
            row: self.row,
            byte_start: start,
            byte_end: self.byte,
        };
        self.gap(gap);
        let caption = self.push(text, attrs);
        let group = ControlSpan {
            row: self.row,
            byte_start: start,
            byte_end: self.byte,
        };
        (group, slot, caption)
    }

    pub(super) fn close_button(
        &mut self,
        attrs: Attrs<'static>,
        min: f32,
        pad: f32,
        shift: f32,
    ) -> ControlSpan {
        // The symbol subset omits ×; request its bundled face and real weight
        // so host fallback cannot consume the air needed by the optical shift.
        let attrs = attrs
            .family(Family::Name("IBM Plex Mono"))
            .weight(crate::render::mono_safe_weight("IBM Plex Mono"));
        let ink = self.measure("×", &attrs);
        let width = min.max(ink + 2.0 * pad);
        let spare = (width - 2.0 * pad - ink).max(0.0);
        let lead = (spare * 0.5 + shift).min(spare);
        self.gap(pad);
        let span = ControlSpan {
            row: self.row,
            byte_start: self.byte,
            byte_end: 0,
        };
        self.gap(lead);
        self.push("×", attrs);
        self.gap(spare - lead);
        let span = ControlSpan {
            byte_end: self.byte,
            ..span
        };
        self.gap(pad);
        span
    }
}

#[cfg(test)]
mod tests;
