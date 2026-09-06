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
mod smart_punct;
use bare_url::BareUrlEllipses;
use footnotes::FootnoteNumbers;
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
            .family(Family::Name(theme::active().bullet_face))
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

/// The hanging pull-quote PAIR. Both ends are shaped from ONE `attrs`/`GlyphMetrics`
/// pair — one face, one scale, one [`theme::faint`] value — so open and close can only
/// ever differ in glyph and in x; the two `left`s are the mirrored
/// [`super::geometry::pull_quote_left`] / [`super::geometry::pull_quote_right`],
/// computed from each mark's OWN shaped advance so an asymmetric face still seats both
/// the same distance from the text.
struct QuoteOrnaments {
    marks: Vec<(f32, QuoteSide)>,
    /// Indexed by [`Self::slot`]: the shaped glyph and its own gutter x.
    ends: [(GlyphBuffer, f32); 2],
    color: glyphon::Color,
}

impl QuoteOrnaments {
    fn slot(side: QuoteSide) -> usize {
        match side {
            QuoteSide::Open => 0,
            QuoteSide::Close => 1,
        }
    }

    fn shape(pipeline: &mut TextPipeline, metrics: Metrics) -> Self {
        let marks = pipeline.quote_marks();
        let color = theme::faint().to_glyphon();
        let glyph_metrics =
            GlyphMetrics::new(metrics.font_size * QUOTE_MARK_SCALE, metrics.line_height);
        let attrs = Attrs::new()
            .family(Family::Name(theme::active().font))
            .color(color);
        let box_w = (metrics.font_size * QUOTE_MARK_SCALE * 2.0).max(1.0);
        let gap = metrics.char_width * 0.3;
        let column_left = pipeline.column_left();
        let column_right = column_left + pipeline.column_width();
        let text_left = pipeline.text_left();
        let text_right = text_left + pipeline.text_wrap_width();
        let end = |pipeline: &mut TextPipeline, glyph: char, side: QuoteSide| {
            let mut buffer = GlyphBuffer::new(&mut pipeline.font_system, glyph_metrics);
            if marks.is_empty() {
                return (buffer, 0.0);
            }
            buffer.set_size(
                &mut pipeline.font_system,
                Some(box_w),
                Some(metrics.line_height),
            );
            buffer.set_text(
                &mut pipeline.font_system,
                &glyph.to_string(),
                &attrs,
                Shaping::Advanced,
                None,
            );
            buffer.shape_until_scroll(&mut pipeline.font_system, false);
            let mark_w = buffer
                .layout_runs()
                .map(|run| run.line_w)
                .fold(0.0f32, f32::max);
            let x = match side {
                QuoteSide::Open => {
                    super::geometry::pull_quote_left(column_left, text_left, gap, mark_w)
                }
                QuoteSide::Close => {
                    super::geometry::pull_quote_right(column_right, text_right, gap, mark_w)
                }
            };
            (buffer, x)
        };
        let open = end(pipeline, QUOTE_MARK_GLYPH, QuoteSide::Open);
        let close = end(pipeline, QUOTE_MARK_CLOSE_GLYPH, QuoteSide::Close);
        Self {
            marks,
            ends: [open, close],
            color,
        }
    }

    fn append_areas<'a>(&'a self, areas: &mut Vec<TextArea<'a>>, bounds: TextBounds) {
        for (top, side) in &self.marks {
            let (buffer, left) = &self.ends[Self::slot(*side)];
            areas.push(TextArea {
                buffer,
                left: *left,
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
            bullets: BulletOrnaments::shape(pipeline, metrics, muted),
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
        let capacity = self.rules.marks.len()
            + self.bullets.marks.len()
            + self.quotes.marks.len()
            + self.fence_labels.marks.len()
            + self.fold_tails.marks.len();
        let capacity =
            capacity + self.footnotes.len() + self.bare_urls.len() + self.smart_punct.len();
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
        self.smart_punct.append_areas(&mut areas, bounds);
        areas
    }
}
