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

/// Resolve follow against the same line space that the pipeline shapes.
pub(super) fn cursor_follow_row(
    pipeline: &crate::render::TextPipeline,
    view: &ViewState,
    affinity: crate::caret::Affinity,
) -> usize {
    pipeline.visual_row_of_aff(view.cursor_line, view.cursor_col, affinity)
}

#[cfg(test)]
mod fold_follow_tests {
    use super::*;

    #[test]
    fn typing_after_multiple_folds_follows_the_visible_wrapped_caret() {
        let _g = crate::testlock::serial();
        let _page = crate::page::PagePin::snapshot();
        crate::page::set_page_on(true);
        crate::page::set_measure(30);
        let Some(mut pipeline) = crate::test_gpu::with_shared_programs(|device, queue| {
            let cache = glyphon::Cache::new(device);
            crate::render::TextPipeline::new(
                device,
                queue,
                &cache,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            )
        }) else {
            eprintln!("skipping fold follow regression: no GPU adapter");
            return;
        };
        pipeline.set_size(720.0, 460.0);
        let doc = format!(
            "# A\n{}# C\n{}# B\n{}\n{}",
            "hidden line\n".repeat(12),
            "other hidden line\n".repeat(9),
            "visible wrapped sentence with several words ".repeat(6),
            "following paragraph\n".repeat(30)
        );
        let levels = crate::fold::heading_levels(&doc, true);
        for headings in [vec![], vec![0], vec![0, 13]] {
            let folds = headings.into_iter().collect();
            let hidden = crate::fold::hidden_lines(&levels, &folds);
            let tails = crate::fold::fold_tails(&levels, &folds);
            for col in [0, 55, 110] {
                let mut view = ViewState::base();
                let mut buffer = crate::buffer::Buffer::from_str(&doc);
                buffer.set_cursor(buffer.line_col_to_char(24, col));
                buffer.insert_char('x');
                view.text = buffer.text();
                view.is_markdown = true;
                view.cursor_line = 24;
                view.cursor_col = col + 1;
                crate::fold::apply_to_view(&mut view, &hidden, &tails, &folds);
                pipeline.set_view(&view);
                let expected = pipeline.visual_row_of_aff(
                    view.cursor_line,
                    col + 1,
                    crate::caret::Affinity::Downstream,
                );
                let actual =
                    cursor_follow_row(&pipeline, &view, crate::caret::Affinity::Downstream);
                assert_eq!(actual, expected, "folded caret must follow its painted row");
                let scroll = crate::render::ScrollPos::default();
                assert_eq!(
                    pipeline.scroll_to_show_row_pos(actual, scroll, 460.0),
                    pipeline.scroll_to_show_row_pos(expected, scroll, 460.0),
                );
            }
        }
    }
}

#[cfg(test)]
mod follow_routing_tests {
    #[test]
    fn live_follow_routes_through_the_projected_coordinate_owner() {
        let _g = crate::testlock::serial();
        let source = include_str!("../viewstate.rs");
        let follow = source
            .split("} else if follow {")
            .nth(1)
            .expect("live cursor-follow branch")
            .split("let max =")
            .next()
            .expect("follow branch extent");
        assert_eq!(
            follow.matches("scroll::cursor_follow_row(").count(),
            1,
            "live follow must use the fold-projected coordinate owner"
        );
        assert!(
            !follow.contains("pipeline.visual_row_of_aff("),
            "live follow must not bypass projection with raw document coordinates"
        );
    }
}
