use crate::app::*;

pub(super) fn history_diff_scroll(row: usize) -> crate::render::ScrollPos {
    // The diff transcript is a separate, row-paged surface. Each page action
    // deliberately starts on a raster row boundary; it does not inherit the
    // live document's semantic remainder.
    crate::render::ScrollPos::at_row(row)
}

pub(super) fn resolved_scroll(
    diff_row: Option<usize>,
    document: crate::render::ScrollPos,
) -> crate::render::ScrollPos {
    diff_row.map_or(document, history_diff_scroll)
}

impl App {
    pub(in crate::app) fn normalize_and_repush_scroll(
        &mut self,
        view: &mut ViewState,
        previous: crate::render::ScrollPos,
        height: f32,
    ) {
        let pipeline = &self.frame.gpu().unwrap().pipeline;
        let scroll = pipeline.scroll_by_px(self.document.scroll(), 0.0, height);
        self.document.set_scroll(scroll);
        if self.document.scroll() != previous {
            view.scroll = self.document.scroll();
            self.frame.gpu_mut().unwrap().pipeline.set_view(view);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_diff_scroll_explicitly_resets_the_within_row_remainder() {
        let _g = crate::testlock::serial();
        let pos = super::history_diff_scroll(7);
        assert_eq!(pos.row, 7);
        assert_eq!(
            pos.px_q, 0,
            "history diff paging intentionally lands on a whole row"
        );
        assert_eq!(
            super::resolved_scroll(None, crate::render::ScrollPos { row: 3, px_q: 17 }),
            crate::render::ScrollPos { row: 3, px_q: 17 },
            "ordinary document views retain their semantic remainder"
        );
    }
}
