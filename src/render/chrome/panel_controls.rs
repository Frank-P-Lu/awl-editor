//! THE FIND/REPLACE PANEL'S BORDERED CONTROLS — the field boxes, the nav
//! prev/next buttons, the `Match case` checkbox, and the `Replace`/`Replace
//! all` buttons the reference chrome (`references/find-replace-chrome.png`)
//! calls for. `panel_shape_text` (`panel.rs`) already builds one string per
//! row and knows exactly which BYTE RANGE of that row's own line is each
//! control's text — [`ControlSpan`] is that fact, named, and
//! [`TextPipeline::panel_controls_layout`] is the ONE place a span becomes a
//! physical rect, read alike by the draw prep, `panel_hit`, and the sidecar's
//! `panel_geometry`. Scattering that arithmetic across three call sites is
//! exactly the shape that let the caret and the hit-test disagree before
//! `panel_report.rs` existed; this is the same fix for the newer controls.
//!
//! Every rect comes from the SHAPED glyphs, never a char-count times a
//! hardcoded pitch — labels are the active world's proportional face, so a
//! byte-count assumption would be wrong the instant a label isn't ASCII-mono.

use super::TextPipeline;

/// A control's text, as the BYTE RANGE it occupies within its own row's
/// shaped line (cosmic-text resets `LayoutGlyph::start`/`end` to 0 at every
/// `\n`, so this is never a whole-buffer offset). `row` is the same `f32` row
/// index every other panel geometry seam uses (`PanelRowBands`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct ControlSpan {
    pub row: f32,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Every control [`TextPipeline::panel_shape_text`] may have shaped this
/// frame. `None` for a control this frame's row plan never drew (a plain find
/// panel has no replace field or action buttons at all).
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(in crate::render) struct PanelControlSpans {
    pub find_field: Option<ControlSpan>,
    pub replace_field: Option<ControlSpan>,
    pub nav_prev: Option<ControlSpan>,
    pub nav_next: Option<ControlSpan>,
    pub case_box: Option<ControlSpan>,
    pub replace_button: Option<ControlSpan>,
    pub replace_all_button: Option<ControlSpan>,
}

/// [`PanelControlSpans`], resolved into physical `[x, y, w, h]` rects this
/// frame actually drew (or would draw, for the sidecar) — the outset already
/// applied, so a caller never re-derives the pad.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct ResolvedPanelControls {
    pub find_field: Option<[f32; 4]>,
    pub replace_field: Option<[f32; 4]>,
    pub nav_prev: Option<[f32; 4]>,
    pub nav_next: Option<[f32; 4]>,
    pub case_box: Option<[f32; 4]>,
    pub replace_button: Option<[f32; 4]>,
    pub replace_all_button: Option<[f32; 4]>,
}

impl ResolvedPanelControls {
    /// Every drawn box, as plain rects — what the fill/border pipelines
    /// upload (one `prepare()` call, one shared corner/stroke/color).
    pub(in crate::render) fn boxes(&self) -> Vec<[f32; 4]> {
        [
            self.find_field,
            self.replace_field,
            self.nav_prev,
            self.nav_next,
            self.case_box,
            self.replace_button,
            self.replace_all_button,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Horizontal outset beyond the tight glyph span every control box carries —
/// device px, unscaled by DPI (a crisp, consistently-sized breathing room at
/// any zoom/density, matching [`super::panel::PANEL_PAD`]'s own unscaled
/// contract rather than doubling on Retina).
pub(in crate::render) const CONTROL_BOX_PAD_X: crate::render::Physical =
    crate::render::Physical(6.0);
/// Vertical inset FROM the row band's own top/bottom — leaves the box short
/// of the next row so adjacent controls never visually touch.
pub(in crate::render) const CONTROL_BOX_PAD_Y: crate::render::Physical =
    crate::render::Physical(3.0);

impl TextPipeline {
    /// The one BYTE-SPAN -> X owner for the panel's shaped text: the tight
    /// `[x0, x1]` bound of every glyph on `row` whose start falls in
    /// `[byte_start, byte_end)`, LINE-LOCAL exactly like a raw
    /// `LayoutGlyph::x` (the caller adds `text_left`, mirroring
    /// `panel_glyph_x`'s own contract). `None` when that row shaped no such
    /// glyph (the span's row was never drawn this frame). Generalizes
    /// `panel_glyph_x`'s single-byte lookup (the caret) to a RANGE (a whole
    /// control's text) — both read the same `panel_buffer.layout_runs()`, so
    /// a caret and a box drawn from spans on the same row can never disagree
    /// about where that row's glyphs actually are.
    pub(in crate::render) fn panel_span_x(
        &self,
        row: f32,
        byte_start: usize,
        byte_end: usize,
    ) -> Option<(f32, f32)> {
        if byte_end <= byte_start {
            return None;
        }
        let row = row as usize;
        let mut x0: Option<f32> = None;
        let mut x1: Option<f32> = None;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i != row {
                continue;
            }
            for g in run.glyphs.iter() {
                if g.start >= byte_start && g.start < byte_end {
                    let l = g.x;
                    let r = g.x + g.w;
                    x0 = Some(x0.map_or(l, |v: f32| v.min(l)));
                    x1 = Some(x1.map_or(r, |v: f32| v.max(r)));
                }
            }
        }
        match (x0, x1) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }

    /// One [`ControlSpan`] resolved into a physical rect: the tight glyph
    /// bound from [`Self::panel_span_x`] (LINE-LOCAL, like every raw
    /// `LayoutGlyph::x` — shifted onto the canvas by `text_left` below,
    /// exactly like `panel_glyph_x`'s own caller does for the caret), outset
    /// horizontally by [`CONTROL_BOX_PAD_X`] and inset
    /// vertically within its row band ([`super::panel_report::PanelRowBands`],
    /// via [`Self::panel_rows`]) by [`CONTROL_BOX_PAD_Y`]. `None` when the
    /// span's row shaped no matching glyph this frame.
    fn resolve_one(&self, span: ControlSpan, text_left: f32, text_top: f32) -> Option<[f32; 4]> {
        let (x0, x1) = self.panel_span_x(span.row, span.byte_start, span.byte_end)?;
        let pad_x = self.metrics.px_physical(CONTROL_BOX_PAD_X);
        let pad_y = self.metrics.px_physical(CONTROL_BOX_PAD_Y);
        let (top, h) = self.panel_rows(text_top).band(span.row);
        Some([
            text_left + x0 - pad_x,
            top + pad_y,
            (x1 - x0) + 2.0 * pad_x,
            (h - 2.0 * pad_y).max(0.0),
        ])
    }

    /// **THE ONE OWNER** every control's physical rect comes through: the
    /// draw prep (fills + borders), `panel_hit`'s click test, and the
    /// sidecar's `panel_geometry` all resolve the SAME [`PanelControlSpans`]
    /// through this function, so a box drawn here is a box a press can land
    /// on and a capture can read back — never three separate derivations of
    /// "where is the Replace button".
    pub(in crate::render) fn panel_controls_layout(
        &self,
        spans: &PanelControlSpans,
        text_left: f32,
        text_top: f32,
    ) -> ResolvedPanelControls {
        let one = |s: Option<ControlSpan>| s.and_then(|s| self.resolve_one(s, text_left, text_top));
        ResolvedPanelControls {
            find_field: one(spans.find_field),
            replace_field: one(spans.replace_field),
            nav_prev: one(spans.nav_prev),
            nav_next: one(spans.nav_next),
            case_box: one(spans.case_box),
            replace_button: one(spans.replace_button),
            replace_all_button: one(spans.replace_all_button),
        }
    }
}
