//! HOW WIDE THE SUMMONED WORKSPACE'S PRIMARY COLUMN IS.
//!
//! Both shapes put a narrow column beside a wide one, and both MEASURE it rather
//! than estimate it — a character-count guess over a proportional display face is
//! not a width, and this same number is the column's clip, its active mark and its
//! pointer hit band. What differs is the corpus and the bound, and that difference
//! is the whole of this file.

use super::*;

/// THE TIMELINE COLUMN'S WIDTH POLICY, and why it is not the rail's.
///
/// A rail of CATEGORY LABELS is measured exactly: it holds a fixed vocabulary of
/// display-face words and must hold the widest of them or a category reads
/// truncated. A TIMELINE is a list of ROWS — `2 hr ago · edited "…"` — that
/// already elide through `rowlayout`, and whose widest member is whatever the
/// longest commit subject in the file's history happens to be. Measuring that
/// exactly would let one verbose commit take the comparison's width away, which
/// is the opposite of DESIGN.md §5's "a narrow timeline beside a large
/// comparison". So the timeline HUGS its content, the way a right-anchored card
/// does, and is then
/// BOUNDED: it can never grow past this fraction of the workspace's interior, and
/// it can never COLLAPSE below a floor that keeps a `yesterday 12:34` label
/// readable — the case a bare hug gets wrong is an empty history, whose corpus is
/// one notice rather than a list.
const TIMELINE_MIN_CHARS: f32 = 12.0;
const TIMELINE_MAX_FRAC: f32 = 0.34;

impl TextPipeline {
    /// MEASURE the workspace's PRIMARY (narrow) column width (device px) from its
    /// own shaped content — the same `&mut FontSystem` measurement a
    /// content-hugging card already makes, and for the same reason: a
    /// character-count estimate over a proportional display face is not a width.
    /// Cached into `workspace_primary_w` at `set_view`, so the geometry stays
    /// `&self` and the drawn column, the clip and the hit band all read one number.
    ///
    /// WHICH content it measures is the shape's one fact: a `RailOverRows`
    /// workspace's primary column carries category LABELS, a
    /// `TimelineOverComparison` one carries the workspace's own ROWS. Both are
    /// shaped through the same buffer with the same attrs; only the corpus and the
    /// bound differ (see [`TIMELINE_MAX_FRAC`]).
    pub(in crate::render) fn measure_workspace_primary_w(&mut self) -> f32 {
        let rows_primary = self.overlay_rows_primary;
        if self.overlay_lens.is_empty() {
            return 0.0;
        }
        let text = match rows_primary {
            // The EMPTY-STATE notice is part of the corpus: an empty history's
            // column has to hold "no history yet" as surely as a full one holds
            // its widest row.
            true => match self.overlay_items.is_empty() {
                true => self.overlay_empty.clone().unwrap_or_default(),
                false => self.overlay_items.join("\n"),
            },
            false => self
                .overlay_lens
                .iter()
                .map(|(label, _)| label.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let widest = self.measure_workspace_column_px(&text);
        let hpad = self.overlay_text_hpad();
        if !rows_primary {
            return widest + 2.0 * hpad;
        }
        // THE FOOTER IS PART OF THE COLUMN'S CONTENT, so it is part of the
        // measurement. On the rail shape the footer rides the wide pane beside the
        // rail, so its width is never the rail's problem; on the timeline shape it
        // rides the timeline itself, and a column measured from the rows alone
        // clips the very line that teaches the keys. It is shaped at the footer's
        // OWN metrics rather than at the UI face scaled by `LABEL`: glyph advances
        // do not scale linearly, and the approximation under-measured a real
        // shipping world's footer by a pixel.
        let hint = match self.overlay_hint.is_empty() {
            true => 0.0,
            false => self.measure_workspace_hint_px(),
        };
        // A timeline is BOUNDED, never merely measured — and it keeps its floor
        // even when the corpus is empty ("no history yet"), so an empty timeline
        // is still a timeline beside a comparison rather than a collapsed column.
        let cw = self.overlay_char_width();
        let interior = (self.window_w - 2.0 * self.workspace_margin() - 2.0 * hpad).max(0.0);
        // THE FOOTER IS A HARD FLOOR, not merely part of the corpus: the width cap
        // is a composition preference (a narrow timeline beside a large
        // comparison) and legibility outranks it (DESIGN.md §8 rule 1 — preserve
        // readable type and honest control sizes before anything else). A world
        // whose display face shapes the footer wider than the cap gets a slightly
        // wider timeline, not a cut footer.
        // CEIL, because the two shapings agree to the pixel but not to the last
        // bit of an `f32`, and a column short by a ten-thousandth of a pixel cuts
        // a glyph exactly as thoroughly as one short by ten.
        let floor = (TIMELINE_MIN_CHARS * cw).max(hint.ceil() + 2.0 * hpad);
        (widest + 2.0 * hpad).clamp(floor, (interior * TIMELINE_MAX_FRAC).max(floor))
    }

    /// Shape `text` into the primary column's own buffer and report its widest
    /// line. The one shaping seam both corpora above go through.
    fn measure_workspace_column_px(&mut self, text: &str) -> f32 {
        self.overlay_remetric();
        let ui_metrics = self.overlay_metrics();
        self.workspace_rail_buffer
            .set_metrics(&mut self.font_system, ui_metrics);
        self.workspace_rail_buffer
            .set_size(&mut self.font_system, None, None);
        self.workspace_rail_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let ink = theme::base_content().to_glyphon();
        self.workspace_rail_buffer.set_text(
            &mut self.font_system,
            text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.workspace_rail_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut widest = 0.0_f32;
        for run in self.workspace_rail_buffer.layout_runs() {
            widest = widest.max(run.line_w);
        }
        widest
    }

    /// Shape the FOOTER at its own metrics and report its width — the size
    /// `push_overlay_hint_spans` really draws it at, so the column is measured
    /// against the line it has to hold rather than a scaled approximation of it.
    fn measure_workspace_hint_px(&mut self) -> f32 {
        self.overlay_remetric();
        let name_fs = self.overlay_metrics().font_size;
        let metrics = GlyphMetrics::new(
            name_fs * crate::markdown::type_scale::LABEL,
            self.overlay_hint_h(),
        );
        let hint = self.overlay_hint.clone();
        self.workspace_rail_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.workspace_rail_buffer
            .set_size(&mut self.font_system, None, None);
        self.workspace_rail_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let ink = theme::base_content().to_glyphon();
        // THE FOOTER IS MEASURED IN THE FACES IT IS DRAWN IN. The drawn line is
        // SYMBOL-SPLIT (`push_overlay_hint_spans` → `push_symbol_split`): the key
        // glyphs `↵ ← → ⌘` come from the symbol family, whose advances are not the
        // body face's. Shaping the raw string in one face UNDER-measures it — by
        // 1.3px on Mopoke's own footer, harmless while the column carried two
        // unspent pads of slack and a clipped footer the moment item 234 spent
        // them. One corpus, the same split, so the floor cannot under-reserve.
        let mut spans: Vec<(&str, Attrs)> = Vec::new();
        push_symbol_split(
            &mut spans,
            &hint,
            || panel_attrs().color(ink).metrics(metrics),
            || {
                Attrs::new()
                    .family(Family::Name(SYMBOL_FAMILY))
                    .color(ink)
                    .metrics(metrics)
            },
        );
        self.workspace_rail_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.workspace_rail_buffer
            .shape_until_scroll(&mut self.font_system, false);
        self.workspace_rail_buffer
            .layout_runs()
            .fold(0.0_f32, |w, run| w.max(run.line_w))
    }
}
