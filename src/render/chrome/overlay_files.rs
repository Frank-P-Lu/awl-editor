//! Files surface pointer regions and geometry probes.
//!
//! Every region is derived from the shaped runs and the shared overlay row plan,
//! so drawing, hit-testing, and evidence all observe the same geometry.

use super::*;

#[derive(Clone, Copy)]
struct FilesChromeLayout {
    query: [f32; 4],
    up: Option<[f32; 4]>,
    change: [f32; 4],
    footer: [f32; 4],
}

impl TextPipeline {
    fn files_surface_chrome_layout(&self) -> Option<FilesChromeLayout> {
        if !self.overlay_active || !self.overlay_files_surface {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let ui = self.metrics.ui();
        let pad_x = ui.px(Logical(7.0));
        let inset_y = ui.px(Logical(3.0));
        let field = self.overlay_query_band(&plan)?;
        let input_x = self.overlay_query_input_x(&geom, &plan);
        let query = [
            (input_x - pad_x).max(geom.text_left),
            field.top + inset_y,
            (geom.text_left + geom.text_w - input_x + pad_x).max(1.0),
            (field.height - 2.0 * inset_y).max(1.0),
        ];

        let split = self.files_query_is_split(&geom);
        let actions_split = self.files_actions_are_split(&geom);
        let action_line = usize::from(split && actions_split);
        let (up, change) = (if split {
            plan.header_lines().get(action_line).copied()
        } else {
            plan.query_band()
        })
        .and_then(|line| {
            let prefix = if actions_split {
                self.files_action_suffix()
            } else if split {
                self.overlay_files_fitted_title_prefix.clone()
            } else {
                self.overlay_title_prefix(&geom)
            };
            let run = self.panel_buffer.layout_runs().nth(action_line)?;
            let origin = self.overlay_head_left(&geom, &plan);
            let glyph_x = |byte: usize| {
                run.glyphs
                    .iter()
                    .find(|glyph| glyph.start >= byte)
                    .map(|glyph| origin + glyph.x)
                    .unwrap_or(origin + run.line_w)
            };
            let region = |needle: &str| {
                let start = prefix.rfind(needle)?;
                let word = needle.trim();
                let word_start = start + needle.find(word)?;
                let word_end = word_start + word.len();
                let x0 = glyph_x(word_start);
                let x1 = glyph_x(word_end);
                Some([x0, line.top, x1 - x0, line.height])
            };
            Some((region("Up"), region("Change folder")?))
        })?;
        let change = [
            change[0] - pad_x,
            change[1] + inset_y,
            change[2] + 2.0 * pad_x,
            (change[3] - 2.0 * inset_y).max(1.0),
        ];
        let up = up.map(|rect| {
            [
                rect[0],
                rect[1] + inset_y,
                rect[2],
                (rect[3] - 2.0 * inset_y).max(1.0),
            ]
        });

        let footer_run = self.panel_buffer.layout_runs().find(|run| {
            (run.text.starts_with("New document") || run.text.starts_with("› New document"))
                && run.glyphs.first().is_some_and(|glyph| glyph.start == 0)
        })?;
        let footer = [
            geom.text_left,
            geom.text_top + footer_run.line_top + inset_y,
            geom.text_w,
            (footer_run.line_height - 2.0 * inset_y).max(1.0),
        ];
        Some(FilesChromeLayout {
            query,
            up,
            change,
            footer,
        })
    }

    pub(super) fn prepare_files_controls(&mut self, surface: OverlayCardSurface<'_>) {
        let OverlayCardSurface {
            device,
            queue,
            width,
            height,
            ..
        } = surface;
        let Some(layout) = self.files_surface_chrome_layout() else {
            self.files_control_fill
                .prepare(device, queue, width, height, &[]);
            self.files_control_rim
                .prepare_multicolor(device, queue, width, height, &[]);
            return;
        };
        let chrome = crate::render::overlay_chrome_theme();
        let radius = self.metrics.ui().px(Logical(4.0));
        self.files_control_fill.set_corner(radius);
        self.files_control_fill
            .set_color(chrome.base_200.rgba_bytes());
        self.files_control_fill.prepare(
            device,
            queue,
            width,
            height,
            &[layout.query, layout.change, layout.footer],
        );

        let focused = chrome.primary.rgba_bytes();
        let quiet = chrome.muted.rgba_bytes();
        let grow = |[x, y, w, h]: [f32; 4]| [x - 1.0, y - 1.0, w + 2.0, h + 2.0];
        let rims = [
            (
                grow(layout.query),
                if self.overlay_query_focused {
                    focused
                } else {
                    quiet
                },
            ),
            (
                grow(layout.change),
                if self.overlay_title.contains("› Change folder") {
                    focused
                } else {
                    quiet
                },
            ),
            (
                grow(layout.footer),
                if self.overlay_hint.starts_with("› New document") {
                    focused
                } else {
                    quiet
                },
            ),
        ];
        self.files_control_rim.set_corner(radius + 1.0);
        self.files_control_rim
            .prepare_multicolor(device, queue, width, height, &rims);
    }

    fn files_surface_action_regions(
        &self,
    ) -> [Option<(crate::render::FilesSurfaceAction, [f32; 4])>; 3] {
        use crate::render::FilesSurfaceAction;
        let Some(layout) = self.files_surface_chrome_layout() else {
            return [None; 3];
        };
        [
            layout.up.map(|rect| (FilesSurfaceAction::Up, rect)),
            Some((FilesSurfaceAction::ChangeFolder, layout.change)),
            Some((FilesSurfaceAction::NewDocument, layout.footer)),
        ]
    }

    /// Hit-test Files controls that are deliberately not candidate rows: Up
    /// and Change folder in the header, and New document in the destination
    /// footer. Their regions come from shaped glyph runs, so the pointer seats
    /// on the words the frame actually drew at every DPI and width.
    pub(crate) fn files_surface_action_at(
        &self,
        px: f32,
        py: f32,
    ) -> Option<crate::render::FilesSurfaceAction> {
        self.files_surface_action_regions()
            .into_iter()
            .flatten()
            .find_map(|(action, [x, y, w, h])| {
                (px >= x && px <= x + w && py >= y && py <= y + h).then_some(action)
            })
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_action_regions_probe(
        &self,
    ) -> [Option<(crate::render::FilesSurfaceAction, [f32; 4])>; 3] {
        self.files_surface_action_regions()
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_control_rects_probe(&self) -> Option<[[f32; 4]; 3]> {
        self.files_surface_chrome_layout()
            .map(|layout| [layout.query, layout.change, layout.footer])
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_ink_bounds_probe(&self) -> Option<[[f32; 4]; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let header_lines = 1
            + usize::from(self.files_query_is_split(&geom))
            + usize::from(self.files_actions_are_split(&geom));
        let mut header_runs = self.panel_buffer.layout_runs().take(header_lines);
        let header = header_runs.next()?;
        let header_w = header_runs.fold(header.line_w, |width, run| width.max(run.line_w));
        let footer = self.panel_buffer.layout_runs().find(|run| {
            (run.text.starts_with("New document") || run.text.starts_with("› New document"))
                && run.glyphs.first().is_some_and(|glyph| glyph.start == 0)
        })?;
        let caret = self.overlay_query_caret_box(&geom, &plan)?;
        Some([
            [geom.text_left, geom.text_top, geom.text_w, geom.card_h],
            [
                self.overlay_head_left(&geom, &plan),
                geom.text_top + header.line_top,
                header_w,
                header.line_height * header_lines as f32,
            ],
            [
                geom.text_left,
                geom.text_top + footer.line_top,
                footer.line_w,
                footer.line_height,
            ],
            caret,
        ])
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_text_probe(&self) -> Option<(String, String)> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let header_lines = 1
            + usize::from(self.files_query_is_split(&geom))
            + usize::from(self.files_actions_are_split(&geom));
        let header = self
            .panel_buffer
            .layout_runs()
            .take(header_lines)
            .map(|run| run.text.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        let footer = self.panel_buffer.layout_runs().find(|run| {
            (run.text.starts_with("New document") || run.text.starts_with("› New document"))
                && run.glyphs.first().is_some_and(|glyph| glyph.start == 0)
        })?;
        Some((header, footer.text.to_string()))
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_fit_attempts_probe(&self) -> (usize, usize, usize) {
        (
            self.overlay_files_split_measure_attempts,
            self.overlay_files_title_fit_attempts,
            self.overlay_files_hint_fit_attempts,
        )
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_line_bounds_probe(
        &self,
        text: &str,
    ) -> Option<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        self.panel_buffer
            .layout_runs()
            .find(|run| run.text == text)
            .map(|run| {
                [
                    geom.text_left,
                    geom.text_top + run.line_top,
                    run.line_w,
                    run.line_height,
                ]
            })
    }

    #[cfg(test)]
    pub(in crate::render) fn files_surface_containing_line_bounds_probe(
        &self,
        text: &str,
    ) -> Option<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        self.panel_buffer
            .layout_runs()
            .find(|run| run.text.contains(text))
            .map(|run| {
                [
                    geom.text_left,
                    geom.text_top + run.line_top,
                    run.line_w,
                    run.line_height,
                ]
            })
    }
}
