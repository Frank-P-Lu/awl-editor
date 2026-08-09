use super::*;

impl TextPipeline {
    /// Refresh the height reservation and forcing span for an image on one
    /// concealable line. Image forcing has its own fast-path bookkeeping on
    /// cursor-only updates, separate from the attribute refresh.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn refresh_conceal_image_force(
        &mut self,
        li: usize,
        start: usize,
        tlen: usize,
        cursor_line: usize,
        selection_touch: Option<&std::ops::Range<usize>>,
        attrs: &Attrs<'static>,
        base_font_size: f32,
        wrap: f32,
        md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
        image_heights: &mut [Option<f32>],
        image_force: &mut [Option<(f32, f32)>],
    ) {
        let Some(dh) = self
            .image_report
            .borrow()
            .iter()
            .find(|im| im.line == li)
            .map(|im| im.display_h)
        else {
            return;
        };
        let line_text = self.buffer.lines[li].text().to_string();
        let Some((img_start, img_end)) = md_spans.iter().find_map(|(range, kind)| {
            matches!(
                kind,
                crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::Image)
            )
            .then(|| (range.start.max(start), range.end.min(start + tlen)))
            .filter(|(span_start, span_end)| span_start < span_end)
        }) else {
            return;
        };
        let local_range = (img_start - start)..(img_end - start);
        let mixed =
            crate::render::spans::image_line_has_other_content(&line_text, local_range.clone());
        if mixed {
            if let Some(slot) = image_heights.get_mut(li) {
                *slot = None;
            }
            // This cursor-only rescan must use the same reveal predicate as layout,
            // otherwise a selection-driven park would immediately re-force the row.
            let revealed_now =
                li == cursor_line || selection_touches(selection_touch, &(img_start..img_end));
            let want = if revealed_now {
                None
            } else {
                let prefix = &line_text[..local_range.start];
                let last_row_w = Self::measure_last_row_width(
                    &mut self.font_system,
                    prefix,
                    attrs,
                    base_font_size,
                    wrap,
                );
                let remaining = (wrap - last_row_w).max(0.0);
                Some((dh, remaining + Self::IMAGE_FORCE_MARGIN_PX))
            };
            if let Some(slot) = image_force.get_mut(li) {
                *slot = want;
            }
        } else {
            if let Some(slot) = image_heights.get_mut(li) {
                *slot = Some(dh);
            }
            if let Some(slot) = image_force.get_mut(li) {
                *slot = None;
            }
        }
    }
}
