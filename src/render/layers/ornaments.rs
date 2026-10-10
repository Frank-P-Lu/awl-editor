//! Five shaped-buffer families owned through one ornament-frame upload. The
//! fold CHEVRON is not a sixth: it must rotate a quarter turn on fold/unfold,
//! and glyphon 0.11 carries no transform, so it is built from rotated-quad arms
//! and drawn through its own `SelectionPipeline`
//! (`render/layers/fold_chevron.rs`'s `prepare_fold_chevron_marks`, called
//! alongside `prepare_ornaments`, not from within it). The fold TAIL ("… N
//! lines") stays here; only the chevron lives outside this glyphon pipeline.
use super::*;
use crate::render::rects::QuoteSide;

mod bare_url;
mod footnotes;
#[cfg(test)]
mod probe;
mod quote;
mod smart_punct;
use bare_url::BareUrlEllipses;
use footnotes::FootnoteNumbers;
use quote::QuoteOrnaments;
use smart_punct::SmartPunctGlyphs;

struct RuleOrnaments {
    marks: Vec<(f32, &'static str)>,
    glyphs: Vec<(&'static str, GlyphBuffer)>,
}

impl RuleOrnaments {
    fn shape(
        pipeline: &mut TextPipeline,
        metrics: Metrics,
        muted: glyphon::Color,
        col_w: f32,
    ) -> Self {
        let marks = if pipeline.md_enabled {
            pipeline.rule_marks()
        } else {
            Vec::new()
        };
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().ornament_face))
            .weight(ORNAMENT_WEIGHT)
            .color(muted);
        let scale = theme::active().ornament_scale;
        let line_h = metrics.line_height * scale;
        let glyph_metrics = GlyphMetrics::new(metrics.font_size * scale, line_h);
        let mut distinct = Vec::new();
        for (_, run) in &marks {
            if !distinct.contains(run) {
                distinct.push(*run);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|run| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(&mut pipeline.font_system, Some(col_w), Some(line_h));
                buffer.set_text(
                    &mut pipeline.font_system,
                    run,
                    &attrs,
                    Shaping::Advanced,
                    Some(glyphon::cosmic_text::Align::Center),
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                (run, buffer)
            })
            .collect();
        Self { marks, glyphs }
    }

    fn append_areas<'a>(
        &'a self,
        areas: &mut Vec<TextArea<'a>>,
        left: f32,
        bounds: TextBounds,
        muted: glyphon::Color,
    ) {
        for (top, run) in &self.marks {
            let buffer = &self
                .glyphs
                .iter()
                .find(|(candidate, _)| candidate == run)
                .expect("rule run was deduped in")
                .1;
            areas.push(TextArea {
                buffer,
                left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: muted,
                custom_glyphs: &[],
            });
        }
    }
}

/// One distinct list drawing and its shared tight-ink placement correction.
struct ListMarkerGlyph {
    glyph: char,
    scale_bits: u32,
    slot_width_bits: u32,
    baseline_offset_bits: u32,
    ink: [u8; 4],
    buffer: GlyphBuffer,
    offset: [f32; 2],
}

struct ListMarkers {
    marks: Vec<crate::render::rects::ListMark>,
    glyphs: Vec<ListMarkerGlyph>,
}

impl ListMarkers {
    /// Marker-to-prose clearance in body ems. The prefix spacer reserves the
    /// room; tight raster bounds spend it from the prose edge leftward.
    const PROSE_GAP_EM: f32 = 0.4;

    fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = if pipeline.md_enabled {
            pipeline.list_marks()
        } else {
            Vec::new()
        };
        let attrs = Attrs::new().family(Family::Name(theme::active().bullet_face));
        let mut distinct = Vec::new();
        for mark in &marks {
            let key = (
                mark.glyph,
                mark.scale.to_bits(),
                mark.slot_width.to_bits(),
                (mark.baseline - mark.top).to_bits(),
                mark.ink,
            );
            if !distinct.contains(&key) {
                distinct.push(key);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|(ch, scale_bits, width_bits, baseline_offset_bits, ink)| {
                let scale = f32::from_bits(scale_bits);
                let width = f32::from_bits(width_bits);
                let color = glyphon::Color::rgba(ink[0], ink[1], ink[2], ink[3]);
                let glyph_metrics =
                    GlyphMetrics::new(metrics.font_size * scale, metrics.line_height);
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(
                    &mut pipeline.font_system,
                    Some(width),
                    Some(metrics.line_height),
                );
                buffer.set_text(
                    &mut pipeline.font_system,
                    &ch.to_string(),
                    &attrs.clone().color(color),
                    Shaping::Advanced,
                    Some(glyphon::cosmic_text::Align::Center),
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                let glyphs = crate::rotated_label::ink::buffer_key(&buffer);
                let body_ascent_em =
                    crate::render::facepitch::vertical_em_metrics(pipeline.shaped_font).0;
                let body_typical_height = metrics.font_size
                    * body_ascent_em
                    * crate::render::facepitch::typical_letter_ratio(pipeline.shaped_font);
                let baseline_offset = f32::from_bits(baseline_offset_bits);
                let target_ink_right = width - metrics.font_size * Self::PROSE_GAP_EM;
                let target_ink_center_y = baseline_offset - body_typical_height * 0.5;
                let offset = pipeline
                    .glyph_ink_cache
                    .bounds(
                        &mut pipeline.font_system,
                        &mut pipeline.swash_cache,
                        &glyphs,
                    )
                    .map_or([0.0, 0.0], |ink| {
                        [
                            target_ink_right - (ink[0] + ink[2]),
                            target_ink_center_y - (ink[1] + ink[3] * 0.5),
                        ]
                    });
                ListMarkerGlyph {
                    glyph: ch,
                    scale_bits,
                    slot_width_bits: width_bits,
                    baseline_offset_bits,
                    ink,
                    buffer,
                    offset,
                }
            })
            .collect();
        Self { marks, glyphs }
    }

    fn append_glyph<'a>(
        &'a self,
        areas: &mut Vec<TextArea<'a>>,
        marker: &crate::render::rects::ListMark,
        ch: char,
        bounds: TextBounds,
    ) {
        let glyph = self
            .glyphs
            .iter()
            .find(|glyph| {
                glyph.glyph == ch
                    && glyph.scale_bits == marker.scale.to_bits()
                    && glyph.slot_width_bits == marker.slot_width.to_bits()
                    && glyph.baseline_offset_bits == (marker.baseline - marker.top).to_bits()
                    && glyph.ink == marker.ink
            })
            .expect("list-marker glyph was deduped in");
        areas.push(TextArea {
            buffer: &glyph.buffer,
            left: marker.left + glyph.offset[0],
            top: marker.top + glyph.offset[1],
            scale: 1.0,
            bounds,
            default_color: glyphon::Color::rgba(
                marker.ink[0],
                marker.ink[1],
                marker.ink[2],
                marker.ink[3],
            ),
            custom_glyphs: &[],
        });
    }

    fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for marker in &self.marks {
            self.append_glyph(areas, marker, marker.glyph, bounds);
        }
    }
}

struct FenceLabels {
    marks: Vec<(f32, crate::syntax::Lang)>,
    glyphs: Vec<(crate::syntax::Lang, GlyphBuffer, f32)>,
    right: f32,
    inset: f32,
}

impl FenceLabels {
    fn shape(
        pipeline: &mut TextPipeline,
        metrics: Metrics,
        muted: glyphon::Color,
        col_w: f32,
    ) -> Self {
        let marks = if pipeline.md_enabled {
            pipeline.fence_lang_marks()
        } else {
            Vec::new()
        };
        let glyph_metrics = GlyphMetrics::new(
            metrics.font_size * crate::markdown::type_scale::LABEL,
            metrics.line_height,
        );
        let attrs = panel_attrs().color(muted);
        let mut distinct = Vec::new();
        for (_, lang) in &marks {
            if !distinct.contains(lang) {
                distinct.push(*lang);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|lang| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(
                    &mut pipeline.font_system,
                    Some(col_w),
                    Some(metrics.line_height),
                );
                buffer.set_text(
                    &mut pipeline.font_system,
                    lang.name(),
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                let width = buffer
                    .layout_runs()
                    .map(|run| run.line_w)
                    .fold(0.0f32, f32::max);
                (lang, buffer, width)
            })
            .collect();
        Self {
            marks,
            glyphs,
            right: pipeline.text_left() + pipeline.text_wrap_width(),
            inset: metrics.char_width * 0.5,
        }
    }

    fn append_areas<'a>(
        &'a self,
        areas: &mut Vec<TextArea<'a>>,
        text_left: f32,
        bounds: TextBounds,
        muted: glyphon::Color,
    ) {
        for (top, lang) in &self.marks {
            let (_, buffer, width) = self
                .glyphs
                .iter()
                .find(|(candidate, _, _)| candidate == lang)
                .expect("fence language was deduped in");
            areas.push(TextArea {
                buffer,
                left: (self.right - width - self.inset).max(text_left),
                top: *top,
                scale: 1.0,
                bounds,
                default_color: muted,
                custom_glyphs: &[],
            });
        }
    }
}

struct FoldTails {
    marks: Vec<(f32, f32, usize, usize)>,
    glyphs: Vec<(GlyphBuffer, f32)>,
    color: glyphon::Color,
}

impl FoldTails {
    fn shape(pipeline: &mut TextPipeline, metrics: Metrics, col_w: f32) -> Self {
        let marks = pipeline.fold_tail_marks();
        let color = theme::fold_afford_tail_ink().to_glyphon();
        let mark_h = metrics.line_height * crate::markdown::type_scale::LABEL;
        let glyph_metrics = GlyphMetrics::new(
            metrics.font_size * crate::markdown::type_scale::LABEL,
            mark_h,
        );
        let attrs = panel_attrs().color(color);
        let glyphs = marks
            .iter()
            .map(|&(_, _, count, _)| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(&mut pipeline.font_system, Some(col_w), Some(mark_h));
                buffer.set_text(
                    &mut pipeline.font_system,
                    &fold_tail_text(count),
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                let width = buffer
                    .layout_runs()
                    .map(|run| run.line_w)
                    .fold(0.0f32, f32::max);
                (buffer, width)
            })
            .collect();
        Self {
            marks,
            glyphs,
            color,
        }
    }

    fn append_areas<'a>(
        &'a self,
        areas: &mut Vec<TextArea<'a>>,
        pipeline: &TextPipeline,
        column_right: f32,
        bounds: TextBounds,
    ) {
        for (i, &(baseline, desired_left, _, line)) in self.marks.iter().enumerate() {
            let (buffer, width) = &self.glyphs[i];
            let floor_left = pipeline.fold_affordance_row_end_x(line);
            let draw_left = desired_left.min(column_right - width);
            if draw_left < floor_left {
                continue;
            }
            let line_y =
                buffer.layout_runs().next().map(|run| run.line_y).unwrap_or(
                    pipeline.metrics.line_height * crate::markdown::type_scale::LABEL * 0.8,
                );
            areas.push(TextArea {
                buffer,
                left: draw_left,
                top: baseline - line_y,
                scale: 1.0,
                bounds,
                default_color: self.color,
                custom_glyphs: &[],
            });
        }
    }
}

pub(super) struct OrnamentFrame {
    rules: RuleOrnaments,
    list_markers: ListMarkers,
    quotes: QuoteOrnaments,
    fence_labels: FenceLabels,
    fold_tails: FoldTails,
    footnotes: FootnoteNumbers,
    bare_urls: BareUrlEllipses,
    smart_punct: SmartPunctGlyphs,
    muted: glyphon::Color,
    text_left: f32,
    col_w: f32,
}

impl OrnamentFrame {
    pub(super) fn shape(pipeline: &mut TextPipeline) -> Self {
        let metrics = pipeline.metrics;
        let muted = theme::muted().to_glyphon();
        let text_left = pipeline.text_left();
        let col_w = pipeline.text_wrap_width().max(1.0);
        Self {
            rules: RuleOrnaments::shape(pipeline, metrics, muted, col_w),
            list_markers: ListMarkers::shape(pipeline, metrics),
            quotes: QuoteOrnaments::shape(pipeline, metrics),
            fence_labels: FenceLabels::shape(pipeline, metrics, muted, col_w),
            fold_tails: FoldTails::shape(pipeline, metrics, col_w),
            footnotes: FootnoteNumbers::shape(pipeline, metrics),
            bare_urls: BareUrlEllipses::shape(pipeline, metrics),
            smart_punct: SmartPunctGlyphs::shape(pipeline, metrics),
            muted,
            text_left,
            col_w,
        }
    }

    pub(super) fn text_areas<'a>(
        &'a self,
        pipeline: &TextPipeline,
        bounds: TextBounds,
    ) -> Vec<TextArea<'a>> {
        let list_marker_layers = self.list_markers.marks.len();
        let capacity = self.rules.marks.len()
            + list_marker_layers
            + self.quotes.len()
            + self.fence_labels.marks.len()
            + self.fold_tails.marks.len();
        let capacity =
            capacity + self.footnotes.len() + self.bare_urls.len() + self.smart_punct.len();
        let mut areas = Vec::with_capacity(capacity);
        self.rules
            .append_areas(&mut areas, self.text_left, bounds, self.muted);
        self.list_markers.append_areas(&mut areas, bounds);
        self.quotes.append_areas(&mut areas, bounds);
        self.fence_labels
            .append_areas(&mut areas, self.text_left, bounds, self.muted);
        self.fold_tails
            .append_areas(&mut areas, pipeline, self.text_left + self.col_w, bounds);
        self.footnotes.append_areas(&mut areas, bounds);
        self.bare_urls.append_areas(&mut areas, bounds);
        self.smart_punct.append_areas(&mut areas, bounds);
        areas
    }
}
