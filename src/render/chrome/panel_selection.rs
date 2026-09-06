//! THE SEARCH PANEL's FIELD SELECTION BAND — the visible half of the panel's
//! select-all verb, split out of `panel.rs` to keep that file under its
//! production ceiling. Same module, own file, no ownership change. See
//! [`super::panel`] for the rest of the card.

use super::*;

/// Cross the focused field's own `(start, end)` CHAR range into the shaped
/// row's coordinates: scrolled by [`field_view_window_offset`] — the ONE
/// window rule the caret is crossed by — clipped to the `cap` visible cells,
/// then turned into the identical `(byte, char-prefix)` pairs the caret
/// carries, so both band edges and the caret resolve through one glyph scan
/// and one fallback pitch.
///
/// `None` when nothing is selected, and when a band lands entirely outside
/// the visible window: a field scrolled past its fixed width shows only what
/// is on screen, never a band painted past the card's own edge.
pub(in crate::render) fn panel_selection_span(
    selection: Option<(usize, usize)>,
    label: &str,
    view: &str,
    field_caret: usize,
    field_len: usize,
    cap: usize,
) -> Option<((usize, usize), (usize, usize))> {
    let (s, e) = selection?;
    let off = field_view_window_offset(field_len, field_caret.min(field_len), cap);
    let s = s.saturating_sub(off).min(cap);
    let e = e.saturating_sub(off).min(cap);
    (s < e).then(|| {
        let cell = |i: usize| {
            (
                label.len() + field_caret_byte(view, i),
                label.chars().count() + i,
            )
        };
        (cell(s), cell(e))
    })
}

impl TextPipeline {
    /// Paint the focused field's selection band. Without it ⌘A would arm a
    /// mode nothing on screen reports, and the next keystroke would replace
    /// text the writer had no way to know was selected.
    ///
    /// Rides the SAME `panel_query_selection` instance the Rename minibuffer's
    /// seeded stem uses — one selection primitive, and the two surfaces are
    /// mutually exclusive (a card and the panel are never up together; the
    /// frame branch in `pipeline_layers` picks one). Prepared on EVERY panel
    /// frame, empty band included: the panel branch previously never touched
    /// this pipeline, so a Rename band could otherwise survive into the frame
    /// a panel opened on.
    pub(in crate::render) fn panel_place_selection(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        shape: &PanelShape,
        text_left: f32,
        text_top: f32,
    ) {
        let (width, height) = size;
        let Some(((s_byte, s_chars), (e_byte, e_chars))) = shape.selection_span else {
            self.panel_query_selection
                .prepare(device, queue, width, height, &[]);
            return;
        };
        let x0 = self.panel_glyph_x(shape.caret_row, s_byte, s_chars, text_left);
        let x1 = self.panel_glyph_x(shape.caret_row, e_byte, e_chars, text_left);
        // The band matches the caret's own cell height and centre, so the
        // highlight sits on the text rather than on the row's full leading.
        let h = self.metrics.caret_h;
        let cy = self.panel_caret_cy(text_top, shape.caret_row);
        let rects = [[x0, cy - h * 0.5, (x1 - x0).max(0.0), h]];
        self.panel_query_selection
            .prepare(device, queue, width, height, &rects);
    }
}
