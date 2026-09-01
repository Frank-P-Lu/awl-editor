//! Smart-punctuation source concealment: unlike every other `ConcealKind`
//! this hides real prose bytes (never syntax markup) behind a PAINTED
//! substitute glyph rather than nothing — the bare-URL ellipsis's precedent,
//! generalized from one fixed glyph to a small closed roster of three.

use super::*;

/// The three substitute glyphs' real body-text advances in the document face.
/// Shaped once per face/metric change and threaded into the line-attrs recipe,
/// so layout and the separately-painted glyph read one measurement owner.
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct SmartPunctAdvances {
    advances: [f32; 3],
    forcing_spacing: [f32; 3],
}

impl SmartPunctAdvances {
    pub(in crate::render) fn shape(
        font_system: &mut FontSystem,
        metrics: Metrics,
        family: &'static str,
    ) -> Self {
        let mut advances = [0.0; 3];
        let mut forcing_spacing = [0.0; 3];
        for kind in crate::markdown::SmartPunctKind::ALL {
            let (_, width) = shape_smart_punct_glyph(
                font_system,
                metrics,
                family,
                kind,
                theme::base_content().to_glyphon(),
            );
            let index = kind_index(kind);
            advances[index] = width;
            forcing_spacing[index] =
                calibrate_forcing_spacing(font_system, metrics, family, kind, width);
        }
        Self {
            advances,
            forcing_spacing,
        }
    }

    pub(in crate::render) fn advance(self, kind: crate::markdown::SmartPunctKind) -> f32 {
        self.advances[kind_index(kind)]
    }

    fn forcing_spacing(self, kind: crate::markdown::SmartPunctKind) -> f32 {
        self.forcing_spacing[kind_index(kind)]
    }
}

impl TextPipeline {
    pub(in crate::render) fn refresh_smart_punct_advances(&mut self) {
        self.smart_punct_advances =
            SmartPunctAdvances::shape(&mut self.font_system, self.metrics, self.shaped_font);
    }
}

fn kind_index(kind: crate::markdown::SmartPunctKind) -> usize {
    use crate::markdown::SmartPunctKind;
    match kind {
        SmartPunctKind::EnDash => 0,
        SmartPunctKind::EmDash => 1,
        SmartPunctKind::Ellipsis => 2,
    }
}

fn smart_punct_attrs(family: &'static str, color: glyphon::Color) -> Attrs<'static> {
    Attrs::new()
        .family(Family::Name(family))
        .weight(mono_safe_weight(family))
        .font_features(text::font_features(false, family, code_ligatures_on()))
        .color(color)
}

/// Measure the conceal path at two letter-spacing values and solve its affine
/// response for the substitute's shaped advance. Cosmic treats the dot triplet
/// as one cluster but the dash runs as independent clusters, so the active
/// shaper — not a shared guessed divisor — owns each arm's arithmetic.
fn calibrate_forcing_spacing(
    font_system: &mut FontSystem,
    metrics: Metrics,
    family: &'static str,
    kind: crate::markdown::SmartPunctKind,
    target: f32,
) -> f32 {
    let probe_spacing = 1.0 / CONCEAL_ZERO_WIDTH_FONT_SIZE;
    let zero = concealed_literal_width(font_system, metrics, family, kind, 0.0);
    let probe = concealed_literal_width(font_system, metrics, family, kind, probe_spacing);
    let response = probe - zero;
    debug_assert!(
        response > 0.001,
        "smart-punct forcing probe had no response"
    );
    (target - zero).max(0.0) / response * probe_spacing
}

fn concealed_literal_width(
    font_system: &mut FontSystem,
    metrics: Metrics,
    family: &'static str,
    kind: crate::markdown::SmartPunctKind,
    letter_spacing: f32,
) -> f32 {
    let hidden = smart_punct_attrs(family, RULE_CONCEAL_COLOR).metrics(GlyphMetrics::new(
        CONCEAL_ZERO_WIDTH_FONT_SIZE,
        metrics.line_height,
    ));
    let forcing = hidden.clone().letter_spacing(letter_spacing);
    let mut attrs = glyphon::cosmic_text::AttrsList::new(&hidden);
    attrs.add_span(0..1, &forcing);
    let mut buffer = GlyphBuffer::new_empty(GlyphMetrics::new(
        CONCEAL_ZERO_WIDTH_FONT_SIZE,
        metrics.line_height,
    ));
    buffer.lines.push(glyphon::cosmic_text::BufferLine::new(
        kind.literal(),
        glyphon::cosmic_text::LineEnding::None,
        attrs,
        Shaping::Advanced,
    ));
    buffer.shape_until_scroll(font_system, false);
    buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0f32, f32::max)
}

/// Shape one substitute exactly as it will be painted: the document's settled
/// face, full body metrics, and the caller's ink. Width is independent of ink,
/// but keeping both layout measurement and ornament construction on this door
/// prevents a later size/family tweak from splitting them again.
pub(in crate::render) fn shape_smart_punct_glyph(
    font_system: &mut FontSystem,
    metrics: Metrics,
    family: &'static str,
    kind: crate::markdown::SmartPunctKind,
    color: glyphon::Color,
) -> (GlyphBuffer, f32) {
    let glyph_metrics = GlyphMetrics::new(metrics.font_size, metrics.line_height);
    let attrs = smart_punct_attrs(family, color);
    let mut buffer = GlyphBuffer::new(font_system, glyph_metrics);
    buffer.set_size(
        font_system,
        Some(metrics.line_height * 2.0),
        Some(metrics.line_height),
    );
    buffer.set_text(
        font_system,
        &kind.glyph().to_string(),
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
    let width = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0f32, f32::max);
    (buffer, width)
}

/// Which display glyph a concealed smart-punctuation span's own literal bytes
/// map to — re-derived from the raw source rather than carried on the span
/// itself (the `is_bare_url_tail` / `fence_line_lang` "render re-derives from
/// source" precedent), so the concealed byte range and the painted glyph can
/// never disagree. `None` only for a byte range that isn't actually one of
/// the three recognized runs — defensive, not a real case: every span this is
/// called on came from `push_smart_punct_spans`, which only ever emits an
/// exact `--`/`---`/`...` match.
pub(in crate::render) fn smart_punct_kind_for(
    line_text: &str,
    local_range: std::ops::Range<usize>,
) -> Option<crate::markdown::SmartPunctKind> {
    use crate::markdown::SmartPunctKind::*;
    match line_text.get(local_range)? {
        "--" => Some(EnDash),
        "---" => Some(EmDash),
        "..." => Some(Ellipsis),
        _ => None,
    }
}

/// Force a concealed smart-punctuation span's leading scalar to the reserved
/// substitute advance and zero-width the rest — mirrors
/// [`super::footnotes::add_footnote_conceal_spans`] /
/// [`super::bare_url::add_bare_url_conceal_spans`]'s forced-first-scalar
/// shape exactly. Unlike those two this has no
/// "not actually mine" fallthrough: every `ConcealKind::SmartPunct` span
/// always gets this treatment (there is no SCHEME/TAIL-style second case), so
/// this is called unconditionally rather than returning a dispatch bool.
pub(super) fn add_smart_punct_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    lo: usize,
    hi: usize,
    hidden: &Attrs<'static>,
    advances: SmartPunctAdvances,
) {
    let local_range = (lo - line_doc_start)..(hi - line_doc_start);
    let Some(kind) = smart_punct_kind_for(line_text, local_range) else {
        return;
    };
    let first_len = line_text[(lo - line_doc_start)..]
        .chars()
        .next()
        .map_or(0, char::len_utf8);
    let first_end = (lo + first_len).min(hi);
    if first_end > lo {
        let forcing = hidden
            .clone()
            .letter_spacing(advances.forcing_spacing(kind));
        al.add_span(
            (lo - line_doc_start)..(first_end - line_doc_start),
            &forcing,
        );
    }
    if first_end < hi {
        al.add_span((first_end - line_doc_start)..(hi - line_doc_start), hidden);
    }
}
