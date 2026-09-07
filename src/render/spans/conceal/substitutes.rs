//! THE ONE OWNER of "how much horizontal room does a PAINTED SUBSTITUTE need".
//!
//! Three conceal families hide real bytes behind a glyph the document layer
//! never shapes — the smart-punctuation dash/ellipsis roster, a tamed bare
//! URL's "…" tail, and a footnote's superscript number. Each one has to force
//! its collapsed source to a reserved advance so the painted mark has a real
//! caret cell and the following prose starts after it.
//!
//! **A RESERVATION DERIVED FROM THE ROW METRIC IS NOT A MEASUREMENT OF THE
//! GLYPH.** Two of the three used to reserve a `line_height` fraction, and both
//! ways that can be wrong were live at once: the bare-URL "…" slot ran 20–105%
//! WIDER than the real glyph across the roster (a hole the mark sat
//! left-aligned in), while the footnote slot ran NARROWER than a two-digit
//! number on six worlds — up to 6.68px of overrun into the following prose, and
//! a `debug_assert!` failure in any debug build that opened a document with ten
//! footnotes. A fraction of the ROW is doubly wrong on a HEADING line, where
//! the row carries the heading's scale but every substitute still paints at
//! BODY size.
//!
//! So every reservation here is the substitute's own shaped advance in the
//! document's settled face at BODY metrics, measured once per face/metric
//! change and read by BOTH the conceal forcing and the ornament that paints
//! into it.

use super::*;

/// The superscript size a painted footnote number is shaped at, as a fraction
/// of body font size. Private: both the reserved-slot measurement and the
/// ornament that paints the ink reach it only through
/// [`shape_footnote_number`], so the slot and the ink cannot be shaped at
/// different sizes.
const FOOTNOTE_NUMBER_SCALE: f32 = 0.68;

/// The calm gap a painted footnote number keeps between itself and the prose
/// that follows, as a fraction of the body row. A TASTE DEFAULT: it sits inside
/// the 0.76–4.63px spread the retired `line_height * 0.34` formula produced
/// across the roster, so no world's spacing moves far, and unlike that formula
/// it is a gap rather than a guess at the digits' width.
const FOOTNOTE_NUMBER_GAP: f32 = 0.10;

/// Every painted substitute's reserved advance, shaped once per face/metric
/// change and threaded into the line-attrs recipe. See the module docs.
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct SubstituteAdvances {
    smart_punct: [f32; 3],
    smart_punct_forcing: [f32; 3],
    /// Per DIGIT `0..=9`, that digit's advance at the footnote superscript
    /// size. A number's slot is the sum over its own digits, which is an upper
    /// bound on the shaped run (digits do not kern apart in any bundled face —
    /// asserted, not assumed, by `footnote_slot_covers_the_shaped_number`).
    digits: [f32; 10],
    footnote_gap: f32,
}

impl SubstituteAdvances {
    pub(in crate::render) fn shape(
        font_system: &mut FontSystem,
        metrics: Metrics,
        family: &'static str,
    ) -> Self {
        let mut smart_punct = [0.0; 3];
        let mut smart_punct_forcing = [0.0; 3];
        for kind in crate::markdown::SmartPunctKind::ALL {
            let (_, width) = shape_smart_punct_glyph(
                font_system,
                metrics,
                family,
                kind,
                theme::base_content().to_glyphon(),
            );
            let index = smart_punct::kind_index(kind);
            smart_punct[index] = width;
            smart_punct_forcing[index] =
                smart_punct::calibrate_forcing_spacing(font_system, metrics, family, kind, width);
        }
        let mut digits = [0.0; 10];
        for (d, slot) in digits.iter_mut().enumerate() {
            let (_, width) =
                shape_footnote_number(font_system, metrics, family, d, theme::muted().to_glyphon());
            *slot = width;
        }
        Self {
            smart_punct,
            smart_punct_forcing,
            digits,
            footnote_gap: metrics.line_height * FOOTNOTE_NUMBER_GAP,
        }
    }

    pub(in crate::render) fn advance(self, kind: crate::markdown::SmartPunctKind) -> f32 {
        self.smart_punct[smart_punct::kind_index(kind)]
    }

    pub(in crate::render) fn forcing_spacing(self, kind: crate::markdown::SmartPunctKind) -> f32 {
        self.smart_punct_forcing[smart_punct::kind_index(kind)]
    }

    /// Width reserved for the single painted "…" that substitutes a concealed
    /// bare-URL TAIL. The SAME measurement the smart-punctuation ellipsis
    /// reserves, because it is the same codepoint painted for the same reason:
    /// two rosters wearing one glyph, so they get one number rather than two
    /// opinions about it.
    pub(in crate::render) fn ellipsis_slot(self) -> f32 {
        self.advance(crate::markdown::SmartPunctKind::Ellipsis)
    }

    /// Width reserved for one painted footnote `number`: its own digits' real
    /// advances plus one calm gap before the prose that follows.
    pub(in crate::render) fn footnote_slot(self, number: usize) -> f32 {
        let mut n = number;
        let mut sum = self.digits[n % 10];
        n /= 10;
        while n > 0 {
            sum += self.digits[n % 10];
            n /= 10;
        }
        sum + self.footnote_gap
    }
}

/// Shape one footnote number exactly as it will be painted: the document's
/// settled face at the superscript size, over the caller's ink. THE one door —
/// the reserved-slot measurement above and the ornament that paints the ink
/// both come through here, so a later size or family tweak cannot move one
/// without the other.
pub(in crate::render) fn shape_footnote_number(
    font_system: &mut FontSystem,
    metrics: Metrics,
    family: &'static str,
    number: usize,
    color: glyphon::Color,
) -> (GlyphBuffer, f32) {
    let glyph_metrics = GlyphMetrics::new(
        metrics.font_size * FOOTNOTE_NUMBER_SCALE,
        metrics.line_height,
    );
    let attrs = Attrs::new().family(Family::Name(family)).color(color);
    let mut buffer = GlyphBuffer::new(font_system, glyph_metrics);
    buffer.set_size(
        font_system,
        Some(metrics.line_height * 4.0),
        Some(metrics.line_height),
    );
    buffer.set_text(
        font_system,
        &number.to_string(),
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

impl TextPipeline {
    pub(in crate::render) fn refresh_substitute_advances(&mut self) {
        self.substitute_advances =
            SubstituteAdvances::shape(&mut self.font_system, self.metrics, self.shaped_font);
    }
}

/// THE THREE PAINTED-SUBSTITUTE KINDS — a footnote's number, a tamed bare
/// URL's "…", and the smart-punctuation roster — each collapse their source
/// and force its first scalar to the reserved advance the mark is painted at
/// ([`SubstituteAdvances`]). They share one rule about that measurement:
/// `None` means this renderer (the table-grid cell) has no ornament layer to
/// paint a substitute into, so the source stays VISIBLE rather than collapsing
/// into a hole nothing fills. Returns whether the span was handled here; a
/// bare-URL SCHEME half falls back to the caller's uniform collapse.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_substitute_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    range: &std::ops::Range<usize>,
    lo: usize,
    hi: usize,
    hidden: &Attrs<'static>,
    ck: crate::markdown::ConcealKind,
    substitute_advances: Option<SubstituteAdvances>,
) -> bool {
    use crate::markdown::ConcealKind;
    if !matches!(
        ck,
        ConcealKind::Footnote | ConcealKind::BareUrl | ConcealKind::SmartPunct
    ) {
        return false;
    }
    let Some(advances) = substitute_advances else {
        return true;
    };
    match ck {
        ConcealKind::Footnote => super::footnotes::add_footnote_conceal_spans(
            al,
            line_text,
            line_doc_start,
            md_spans,
            range,
            lo,
            hi,
            hidden,
            advances,
        ),
        ConcealKind::BareUrl => super::bare_url::add_bare_url_conceal_spans(
            al,
            line_text,
            line_doc_start,
            lo,
            hi,
            hidden,
            advances,
        ),
        _ => {
            super::smart_punct::add_smart_punct_conceal_spans(
                al,
                line_text,
                line_doc_start,
                lo,
                hi,
                hidden,
                advances,
            );
            true
        }
    }
}
