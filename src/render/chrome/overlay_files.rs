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
    /// A Files surface yields discretionary spacing before its last candidate.
    /// Keep all identity, action, query, and lens rows; the footer action also
    /// survives. Ordinary canvases return their authored geometry unchanged.
    pub(super) fn files_minimum_row_spacing(
        &self,
        spacing: (f32, f32, f32, usize),
        minimum_rows: usize,
        hint_rows: usize,
        margin: f32,
    ) -> (f32, f32, f32, usize) {
        let (mut y, mut pad, mut gap, mut hint_gap) = spacing;
        if !self.overlay_files_surface || self.overlay_items.is_empty() {
            return spacing;
        }
        let slack = self.metrics.ui().px(Logical(0.5));
        let overflow = |y: f32, pad: f32, gap: f32, hint_gap: usize| {
            let rows = minimum_rows - spacing.3 + hint_gap;
            y + self.overlay_card_h(rows, gap, hint_rows, hint_gap, pad) + margin + slack
                - self.window_h
        };
        gap = (gap - overflow(y, pad, gap, hint_gap).max(0.0)).max(0.0);
        let top_floor = margin + self.menubar_reserve();
        y = (y - overflow(y, pad, gap, hint_gap).max(0.0)).max(top_floor);
        if overflow(y, pad, gap, hint_gap) > 0.0 {
            hint_gap = 0;
        }
        pad = (pad - overflow(y, pad, gap, hint_gap).max(0.0) * 0.5)
            .max(self.metrics.ui().px(Logical(4.0)));
        (y, pad, gap, hint_gap)
    }

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

        let action_line = 0;
        let (up, change) = plan
            .header_lines()
            .get(action_line)
            .copied()
            .and_then(|line| {
                let prefix = self
                    .files_action_suffix()
                    .replace("Change folder", "Change…");
                let run = self.panel_bind_buffer.layout_runs().nth(action_line)?;
                let origin = self.overlay_head_left(&geom, &plan);
                let glyph_x = |byte: usize| {
                    run.glyphs
                        .iter()
                        .find(|glyph| glyph.start >= byte)
                        .map(|glyph| origin + glyph.x)
                        .unwrap_or_else(|| {
                            run.glyphs
                                .last()
                                .map_or(origin, |glyph| origin + glyph.x + glyph.w)
                        })
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
                Some((region("Up"), region("Change…")?))
            })?;
        let left_pad = up.map_or(pad_x, |up| {
            pad_x.min(((change[0] - up[0] - up[2]) * 0.5).max(0.0))
        });
        let change = [
            change[0] - left_pad,
            change[1] + inset_y,
            change[2] + left_pad + pad_x,
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
            &[layout.query, layout.footer],
        );

        let focused = chrome.primary.rgba_bytes();
        let quiet = chrome.muted.rgba_bytes();
        let grow = |[x, y, w, h]: [f32; 4]| [x - 1.0, y - 1.0, w + 2.0, h + 2.0];
        let mut rims = vec![
            (
                grow(layout.query),
                if self.overlay_query_focused {
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
        if self.overlay_title.contains("› Change folder") {
            rims.push((grow(layout.change), focused));
        }
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
    pub(in crate::render) fn files_surface_control_quad_counts_probe(&self) -> [usize; 2] {
        [
            self.files_control_fill.instance_count() as usize,
            self.files_control_rim.instance_count() as usize,
        ]
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
        let mut header = self
            .panel_buffer
            .layout_runs()
            .take(header_lines)
            .map(|run| run.text.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        if let Some(actions) = self.panel_bind_buffer.layout_runs().next() {
            header.push_str(" | ");
            header.push_str(actions.text);
        }
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
