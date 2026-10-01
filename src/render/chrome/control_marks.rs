//! Chrome marks reserve a measured slot, then center their visible ink on the
//! caption. Drawing reuses the same mask and optical geometry as folded headings.
use super::*;
use crate::rotated_label::{RotatedLabelPipeline, geometry, mask::LabelMask};

pub(in crate::render) const CONTROL_MARK_SLOT: Logical = Logical(14.0);
pub(in crate::render) const CONTROL_MARK_GAP: Logical = Logical(6.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) enum ControlMarkKind {
    Checkbox(bool),
    Disclosure(bool),
}

impl ControlMarkKind {
    fn drawing(self) -> (&'static str, char, f32) {
        match self {
            Self::Checkbox(checked) => (SYMBOL_FAMILY, theme::task_marker(checked), 0.0),
            Self::Disclosure(expanded) => ("Iosevka", '▸', if expanded { -90.0 } else { 0.0 }),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::render) struct ControlMarkSpan {
    pub kind: ControlMarkKind,
    pub slot: ControlSpan,
    pub caption: ControlSpan,
    pub color: [u8; 4],
}

struct MarkPainter {
    label: RotatedLabelPipeline,
    mask: Option<LabelMask>,
}

#[derive(Default)]
pub(in crate::render) struct ControlMarks {
    pub spans: Vec<ControlMarkSpan>,
    painters: Vec<MarkPainter>,
}

impl ControlMarks {
    pub(in crate::render) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        for painter in &self.painters {
            painter.label.draw(pass);
        }
    }
}

fn shape_mark(fonts: &mut FontSystem, face: &'static str, ch: char, size: f32) -> GlyphBuffer {
    let mut buffer = GlyphBuffer::new(fonts, GlyphMetrics::new(size, size * 1.5));
    buffer.set_size(fonts, Some(size * 4.0), Some(size * 3.0));
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_text(
        fonts,
        &ch.to_string(),
        &Attrs::new().family(Family::Name(face)),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(fonts, false);
    buffer
}

impl TextPipeline {
    /// Measure a caption in its real baseline frame, using the shared cached
    /// nonzero-coverage owner. Called during preparation, never pointer dispatch.
    fn control_caption_ink(&mut self, span: ControlSpan) -> Option<[f32; 4]> {
        let keys = crate::rotated_label::ink::span_key(
            &self.panel_buffer,
            span.row as usize,
            span.byte_start..span.byte_end,
        );
        self.glyph_ink_cache
            .bounds(&mut self.font_system, &mut self.swash_cache, &keys)
    }

    pub(in crate::render) fn prepare_control_marks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas: (u32, u32),
        origin: [f32; 2],
    ) {
        let spans = self.panel_control_marks.spans.clone();
        while self.panel_control_marks.painters.len() < spans.len() {
            self.panel_control_marks.painters.push(MarkPainter {
                label: RotatedLabelPipeline::new(device, self.format),
                mask: None,
            });
        }
        for painter in &mut self.panel_control_marks.painters {
            painter.label.clear();
        }
        for (i, span) in spans.into_iter().enumerate() {
            let Some((x0, x1)) =
                self.panel_span_x(span.slot.row, span.slot.byte_start, span.slot.byte_end)
            else {
                continue;
            };
            let Some(caption) = self.control_caption_ink(span.caption) else {
                continue;
            };
            let (face, ch, angle) = span.kind.drawing();
            let size = self.metrics.panel_ui().font_size;
            let probe = shape_mark(&mut self.font_system, face, ch, size);
            let keys = crate::rotated_label::mask::run_key(&probe);
            let Some(ink) =
                self.glyph_ink_cache
                    .bounds(&mut self.font_system, &mut self.swash_cache, &keys)
            else {
                continue;
            };
            // Every role fits the complete drawing into its authored visible slot.
            // The checked tick's overshoot remains present rather than clipped.
            let fit = (x1 - x0) / ink[2].max(ink[3]);
            let buffer = shape_mark(&mut self.font_system, face, ch, size * fit);
            let painter = &mut self.panel_control_marks.painters[i];
            if painter
                .mask
                .as_ref()
                .is_none_or(|mask| !mask.matches(&buffer))
            {
                painter.mask = LabelMask::compose(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.swash_cache,
                    &buffer,
                );
            }
            let Some(mask) = painter.mask.as_ref() else {
                continue;
            };
            let axis = geometry::label_axis_deg(angle);
            let center = [
                origin[0] + (x0 + x1) * 0.5,
                origin[1] + caption[1] + caption[3] * 0.5,
            ];
            let pen = geometry::pixel_aligned_centered_origin(mask.tight_ink(), center, axis);
            let color = srgb_u8_to_linear3(span.color);
            painter.label.prepare(
                device, queue, canvas.0, canvas.1, mask, pen, axis, color, color, 1.0,
            );
        }
    }
}
