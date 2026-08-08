//! **THE SEARCH PANEL'S ROW BAND, AND THE WHOLE CARD AS A PUBLISHED FACT** — the
//! summoned top-right find/replace card, planned once and read three ways.
//!
//! The card's exterior rect and inner text origin already had exactly one owner
//! (`panel_layout`), read by the draw and inverted by the pointer. What did NOT
//! have one owner was the card's ROW BAND: the forward `row -> y` step lived
//! inside the caret placer (`text_top + (row + 0.5) * line_height`) and its
//! inverse inside the hit-test (`((py - text_top) / line_height).floor()`), two
//! spellings of one rule that agree only as long as nobody edits one of them.
//! [`PanelRowBands`] is that rule's single owner; the caret placer, the hit-test
//! and the projection below all ask it.
//!
//! **AND NOTHING OUTSIDE THE CRATE COULD SEE ANY OF IT.** The sidecar published
//! the panel's STATE — the query, the hit count, which field is being edited —
//! and no geometry at all, so the one law asking a geometry question about this
//! card ("does a long query widen it?") answered it by walking the PNG inward
//! from the window's right edge with a colour-distance threshold. That is an
//! appearance oracle answering a geometry question: it cannot tell a card that
//! moved from a rim that changed tone, and it silently measures the background
//! whenever the card's fill and the page agree. [`PanelGeometry`] is the same
//! numbers the frame drew, published.
//!
//! **THE COORDINATE SPACE IS PHYSICAL (DEVICE) PIXELS** — the space the pointer
//! arrives in, the space the PNG's pixels are in, and the space the rest of the
//! chrome geometry family already speaks. ⚠️ It does NOT follow that every figure
//! here doubles with `--capture-dpi`: this card's pad and outer margin are
//! unscaled constants, so its `y` is the same physical value at every scale while
//! its row pitch (`metrics.line_height`) doubles. Publishing the pair is what
//! makes that measurable instead of arguable, and the projection must not
//! "correct" either one on the way out.
//!
//! **WHICH ROW IS FOCUSED IS DELIBERATELY ABSENT.** `search.editing_replacement`
//! already reports it, from the state the shaper itself reads; a second answer
//! here could only disagree, and a block whose purpose is making
//! drawn-versus-published agreement assertable must not ship two answers to one
//! question.
//!
//! **WHICH PATH THIS RUNS ON.** [`TextPipeline::panel_geometry`] is reachable only
//! from the sidecar writer (once per capture) and the laws; the frame path calls
//! the band owner, never the projection. One entry per SHAPED ROW of the card —
//! one, or three once the replace row and its hint line are up — so it is bounded
//! by the card, not by the document or the match list.

use crate::render::TextPipeline;

/// **THE PANEL'S ROW BAND** — the one owner of `row <-> y` inside the summoned
/// find/replace card. Uniform rows (the panel does no markdown scaling), so a
/// band is the text origin stepped by whole line heights.
///
/// The three arms are the three spellings that used to be scattered: a band for
/// the projection, its centre for the caret block, and the inverse for the
/// pointer. Each is the arithmetic its old caller used, verbatim, so routing
/// them here moves no pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct PanelRowBands {
    text_top: f32,
    lh: f32,
}

impl PanelRowBands {
    pub(in crate::render) fn new(text_top: f32, lh: f32) -> Self {
        Self { text_top, lh }
    }

    /// Row `row`'s band as `(top, height)`.
    pub(in crate::render) fn band(&self, row: f32) -> (f32, f32) {
        (self.text_top + row * self.lh, self.lh)
    }

    /// That band's vertical centre — where the row's caret block is centred.
    pub(in crate::render) fn center(&self, row: f32) -> f32 {
        self.text_top + (row + 0.5) * self.lh
    }

    /// The inverse: which row a physical pointer `py` falls in. Signed, and
    /// unbounded above, because the caller decides what an out-of-range row
    /// means — the hit-test answers `Elsewhere` for a press in the card's pad.
    pub(in crate::render) fn row_at(&self, py: f32) -> i64 {
        ((py - self.text_top) / self.lh).floor() as i64
    }
}

/// One published panel row: the band that is simultaneously the row the frame
/// DREW its ink on and the band a press in the card resolves to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PanelRowRect {
    pub row: usize,
    pub top: f32,
    pub h: f32,
}

/// The whole summoned find/replace card, published. `card` is the exterior
/// `[x, y, w, h]` the float primitive rims; `text_left`/`text_top` are the ink
/// origin inside it; `rows` is one band per shaped row; `case_toggle` is the `Aa`
/// indicator's own x-span, which is a CLICK TARGET seated on its two shaped
/// glyphs rather than on a hardcoded pitch.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PanelGeometry {
    pub card: [f32; 4],
    pub text_left: f32,
    pub text_top: f32,
    pub rows: Vec<PanelRowRect>,
    pub case_toggle: Option<(f32, f32)>,
}

impl TextPipeline {
    /// The panel's row band for an inner text origin — the seam the caret placer,
    /// the hit-test and the projection share.
    pub(in crate::render) fn panel_rows(&self, text_top: f32) -> PanelRowBands {
        PanelRowBands::new(text_top, self.metrics.line_height)
    }

    /// The card's shaped rows, in draw order — the same population
    /// `panel_layout` sizes the card's height from, asked for its row indices
    /// instead of its count. One entry with the plain find panel up, three once
    /// the replace row and its key-hint line are shaped.
    pub(in crate::render) fn panel_shaped_rows(&self) -> Vec<usize> {
        self.panel_buffer
            .layout_runs()
            .map(|run| run.line_i)
            .collect()
    }

    /// **THE SIDECAR'S PANEL GEOMETRY**, or `None` while the panel is down.
    ///
    /// Gated on `search_active` — the same question `prepare_panel` and
    /// `panel_hit` ask, so a card the frame did not draw reports nothing rather
    /// than reporting where it would have gone. Every figure is read back off
    /// `panel_layout` and the shaped `panel_buffer` this frame really uploaded;
    /// this function performs no arithmetic of its own beyond asking the band
    /// owner, and it retains nothing, so there is no cache key to collide across
    /// a buffer swap.
    pub(crate) fn panel_geometry(&self) -> Option<PanelGeometry> {
        if !self.search_active {
            return None;
        }
        // The caret arguments do not move the card rect or the text origin (the
        // hit-test passes zeros for the same reason); the caret's own x is the
        // shaper's, and is not republished here.
        let (card, text_left, text_top, _caret_x) =
            self.panel_layout(self.window_w as u32, 0, 0, 0.0);
        let bands = self.panel_rows(text_top);
        let rows = self
            .panel_shaped_rows()
            .into_iter()
            .map(|row| {
                let (top, h) = bands.band(row as f32);
                PanelRowRect { row, top, h }
            })
            .collect();
        Some(PanelGeometry {
            card,
            text_left,
            text_top,
            rows,
            case_toggle: self.panel_case_toggle_span(text_left),
        })
    }
}
