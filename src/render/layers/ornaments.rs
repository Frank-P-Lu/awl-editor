//! Five shaped-buffer families owned through one ornament-frame upload. The
//! fold CHEVRON is not a sixth: it must rotate a quarter turn on fold/unfold,
//! and glyphon 0.11 carries no transform, so it is built from rotated-quad arms
//! and drawn through its own `SelectionPipeline`
//! (`render/layers/fold_chevron.rs`'s `prepare_fold_chevron_marks`, called
//! alongside `prepare_ornaments`, not from within it). The fold TAIL ("… N
//! lines") stays here; only the chevron lives outside this glyphon pipeline.
use super::*;

mod bare_url;
mod footnotes;
use bare_url::BareUrlEllipses;
use footnotes::FootnoteNumbers;

struct RuleOrnaments {
    marks: Vec<(f32, char)>,
    glyphs: Vec<(char, GlyphBuffer)>,
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
            .color(muted);
        let scale = theme::active().ornament_scale;
        let line_h = metrics.line_height * scale;
        let glyph_metrics = GlyphMetrics::new(metrics.font_size * scale, line_h);
        let mut distinct = Vec::new();
        for (_, ch) in &marks {
            if !distinct.contains(ch) {
                distinct.push(*ch);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|ch| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(&mut pipeline.font_system, Some(col_w), Some(line_h));
                buffer.set_text(
                    &mut pipeline.font_system,
                    &ch.to_string(),
                    &attrs,
                    Shaping::Advanced,
                    Some(glyphon::cosmic_text::Align::Center),
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                (ch, buffer)
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
        for (top, ch) in &self.marks {
            let buffer = &self
                .glyphs
                .iter()
                .find(|(candidate, _)| candidate == ch)
                .expect("rule char was deduped in")
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

struct BulletOrnaments {
    marks: Vec<(f32, f32, char)>,
    glyphs: Vec<(char, GlyphBuffer)>,
}

impl BulletOrnaments {
    fn shape(pipeline: &mut TextPipeline, metrics: Metrics, muted: glyphon::Color) -> Self {
        let marks = if pipeline.md_enabled {
            pipeline.bullet_marks()
        } else {
            Vec::new()
        };
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().ornament_face))
            .color(muted);
        let glyph_metrics = GlyphMetrics::new(
            metrics.font_size * theme::active().bullet_scale,
            metrics.line_height,
        );
        let width = (metrics.char_width * 2.0).max(1.0);
        let mut distinct = Vec::new();
        for (_, _, ch) in &marks {
            if !distinct.contains(ch) {
                distinct.push(*ch);
            }
        }
        let glyphs = distinct
            .into_iter()
            .map(|ch| {
                let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
                buffer.set_size(
                    &mut pipeline.font_system,
                    Some(width),
                    Some(metrics.line_height),
                );
                buffer.set_text(
                    &mut pipeline.font_system,
                    &ch.to_string(),
                    &attrs,
                    Shaping::Advanced,
                    None,
                );
                buffer.shape_until_scroll(&mut pipeline.font_system, false);
                (ch, buffer)
            })
            .collect();
        Self { marks, glyphs }
    }

    fn append_areas<'a>(
        &'a self,
        areas: &mut Vec<TextArea<'a>>,
        bounds: TextBounds,
        muted: glyphon::Color,
    ) {
        for (top, left, ch) in &self.marks {
            let buffer = &self
                .glyphs
                .iter()
                .find(|(candidate, _)| candidate == ch)
                .expect("bullet char was deduped in")
                .1;
            areas.push(TextArea {
                buffer,
                left: *left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: muted,
                custom_glyphs: &[],
            });
        }
    }
}

struct QuoteOrnaments {
    tops: Vec<f32>,
    buffer: GlyphBuffer,
    left: f32,
    color: glyphon::Color,
}

impl QuoteOrnaments {
    fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let tops = pipeline.quote_marks();
        let color = theme::faint().to_glyphon();
        let glyph_metrics =
            GlyphMetrics::new(metrics.font_size * QUOTE_MARK_SCALE, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let box_w = (metrics.font_size * QUOTE_MARK_SCALE * 2.0).max(1.0);
        let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
        let mut left = 0.0;
        if !tops.is_empty() {
            buffer.set_size(
                &mut pipeline.font_system,
                Some(box_w),
                Some(metrics.line_height),
            );
            buffer.set_text(
                &mut pipeline.font_system,
                &QUOTE_MARK_GLYPH.to_string(),
                &attrs,
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(&mut pipeline.font_system, false);
            let mark_w = buffer
                .layout_runs()
                .map(|run| run.line_w)
                .fold(0.0f32, f32::max);
            let gap = metrics.char_width * 0.3;
            left = super::geometry::pull_quote_left(
                pipeline.column_left(),
                pipeline.text_left(),
                gap,
                mark_w,
            );
        }
        Self {
            tops,
            buffer,
            left,
            color,
        }
    }

    fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for top in &self.tops {
            areas.push(TextArea {
                buffer: &self.buffer,
                left: self.left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: self.color,
                custom_glyphs: &[],
            });
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
    bullets: BulletOrnaments,
    quotes: QuoteOrnaments,
    fence_labels: FenceLabels,
    fold_tails: FoldTails,
    footnotes: FootnoteNumbers,
    bare_urls: BareUrlEllipses,
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
            bullets: BulletOrnaments::shape(pipeline, metrics, muted),
            quotes: QuoteOrnaments::shape(pipeline, metrics),
            fence_labels: FenceLabels::shape(pipeline, metrics, muted, col_w),
            fold_tails: FoldTails::shape(pipeline, metrics, col_w),
            footnotes: FootnoteNumbers::shape(pipeline, metrics),
            bare_urls: BareUrlEllipses::shape(pipeline, metrics),
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
        let capacity = self.rules.marks.len()
            + self.bullets.marks.len()
            + self.quotes.tops.len()
            + self.fence_labels.marks.len()
            + self.fold_tails.marks.len();
        let capacity = capacity + self.footnotes.len() + self.bare_urls.len();
        let mut areas = Vec::with_capacity(capacity);
        self.rules
            .append_areas(&mut areas, self.text_left, bounds, self.muted);
        self.bullets.append_areas(&mut areas, bounds, self.muted);
        self.quotes.append_areas(&mut areas, bounds);
        self.fence_labels
            .append_areas(&mut areas, self.text_left, bounds, self.muted);
        self.fold_tails
            .append_areas(&mut areas, pipeline, self.text_left + self.col_w, bounds);
        self.footnotes.append_areas(&mut areas, bounds);
        self.bare_urls.append_areas(&mut areas, bounds);
        areas
    }
}
