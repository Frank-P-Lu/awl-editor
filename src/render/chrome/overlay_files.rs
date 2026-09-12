//! Files surface pointer regions and geometry probes.
//!
//! Every region is derived from the shaped runs and the shared overlay row plan,
//! so drawing, hit-testing, and evidence all observe the same geometry.

use super::*;

impl TextPipeline {
    fn files_surface_action_regions(
        &self,
    ) -> [Option<(crate::render::FilesSurfaceAction, [f32; 4])>; 3] {
        use crate::render::FilesSurfaceAction;
        if !self.overlay_active || !self.overlay_files_surface {
            return [None; 3];
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let split = self.files_query_is_split(&geom);
        let actions_split = self.files_actions_are_split(&geom);
        let action_line = usize::from(split) + usize::from(actions_split);
        let header = (if split {
            plan.header_lines().get(action_line).copied()
        } else {
            plan.query_band()
        })
        .and_then(|field| {
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
            let region = |needle: &str, action| {
                let start = prefix.rfind(needle)?;
                let word = needle.trim();
                let word_start = start + needle.find(word)?;
                let word_end = word_start + word.len();
                let x0 = glyph_x(word_start);
                let x1 = glyph_x(word_end);
                Some((action, [x0, field.top, x1 - x0, field.height]))
            };
            Some((
                region("Up", FilesSurfaceAction::Up),
                region("Change folder", FilesSurfaceAction::ChangeFolder),
            ))
        });
        let footer = self
            .panel_buffer
            .layout_runs()
            .find(|run| {
                (run.text.starts_with("New document") || run.text.starts_with("› New document"))
                    && run.glyphs.first().is_some_and(|glyph| glyph.start == 0)
            })
            .and_then(|run| {
                let start_byte = run.text.find("New document")?;
                let end_byte = start_byte + "New document".len();
                let x0 =
                    geom.text_left + run.glyphs.iter().find(|glyph| glyph.start >= start_byte)?.x;
                let x1 = geom.text_left
                    + run
                        .glyphs
                        .iter()
                        .find(|glyph| glyph.start >= end_byte)
                        .map(|glyph| glyph.x)
                        .unwrap_or(run.line_w);
                Some((
                    FilesSurfaceAction::NewDocument,
                    [x0, geom.text_top + run.line_top, x1 - x0, run.line_height],
                ))
            });
        let (up, change) = header.unwrap_or((None, None));
        [up, change, footer]
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
