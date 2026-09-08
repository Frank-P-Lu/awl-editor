/// The line band changed by one text synchronization and whether its
/// document-wide styling context remained local to that band.
#[derive(Clone, Copy, Debug)]
pub(super) struct TextChange {
    pub(super) prefix: usize,
    pub(super) old_end: usize,
    pub(super) new_end: usize,
    pub(super) geometry_safe: bool,
}

impl TextChange {
    /// Whether changed lines can be compared with and substituted into the prior
    /// vertical row partition. Empty and line-structural bands, changed wrap
    /// widths, and document-scope invalidations all require a complete rebuild.
    pub(super) fn can_retain_geometry(self, width_stable: bool) -> bool {
        self.old_end - self.prefix == self.new_end - self.prefix
            && self.new_end > self.prefix
            && self.geometry_safe
            && width_stable
    }
}

pub(super) fn is_markdown_geometry_boundary(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```")
        || line.starts_with("~~~")
        || line == "---"
        || line.contains('|')
        || line.contains("![")
}

/// Fresh line indexing for one text synchronization. It is built before image
/// layout because selection reveal must read the new text rather than the
/// glyph buffer's still-stale pre-splice lines.
pub(super) struct ChangedLines<'a> {
    pub(super) lines: Vec<&'a str>,
    pub(super) starts: Vec<usize>,
    pub(super) cursor_byte: usize,
    pub(super) selection_touch: Option<std::ops::Range<usize>>,
}

impl<'a> ChangedLines<'a> {
    pub(super) fn new(
        text: &'a str,
        cursor_line: usize,
        selection: Option<((usize, usize), (usize, usize))>,
    ) -> Self {
        // Split without line terminators; cosmic-text stores endings separately.
        // `split` retains the trailing empty line needed by an EOF caret.
        let lines: Vec<&str> = text.split('\n').collect();
        let mut starts = Vec::with_capacity(lines.len());
        let mut offset = 0;
        for line in &lines {
            starts.push(offset);
            offset += line.len() + 1;
        }
        let cursor_byte = starts.get(cursor_line).copied().unwrap_or(0);
        let selection_touch = crate::render::spans::selection_touch_bytes(
            selection,
            |line| starts.get(line).copied().unwrap_or(0),
            |line| lines.get(line).map_or(0, |text| text.len()),
        );
        Self {
            lines,
            starts,
            cursor_byte,
            selection_touch,
        }
    }
}

impl super::TextPipeline {
    /// Classify the changed band before it is spliced, while both the old line
    /// text and the new document-wide reservations are available.
    pub(super) fn classify_text_change(
        &self,
        new_lines: &[&str],
        old_image_heights: &[Option<f32>],
        image_heights: &[Option<f32>],
        old_image_force: &[Option<(f32, f32)>],
        image_force: &[Option<(f32, f32)>],
        old_scope: (
            Option<crate::frontmatter::Lang>,
            Option<crate::frontmatter::Lang>,
        ),
    ) -> TextChange {
        let (prefix, old_end, new_end) = self.unchanged_band(new_lines);
        let touches_boundary = self.md_enabled
            && (self.buffer.lines[prefix..old_end]
                .iter()
                .any(|line| is_markdown_geometry_boundary(line.text()))
                || new_lines[prefix..new_end]
                    .iter()
                    .any(|line| is_markdown_geometry_boundary(line)));
        let reservation_changed = old_image_heights.len() != image_heights.len()
            || old_image_force.len() != image_force.len()
            || (0..image_heights.len()).any(|line| {
                !(prefix..new_end).contains(&line)
                    && (old_image_heights[line] != image_heights[line]
                        || old_image_force[line] != image_force[line])
            });
        TextChange {
            prefix,
            old_end,
            new_end,
            geometry_safe: !touches_boundary
                && !reservation_changed
                && old_scope == (self.doc_lang, self.han_evidence),
        }
    }

    /// Build and splice only the changed logical-line band. The unchanged
    /// prefix and suffix retain their glyphon identity and cached shaping.
    pub(super) fn splice_changed_lines(
        &mut self,
        new_lines: &[&str],
        line_starts: &[usize],
        attrs: &glyphon::Attrs<'static>,
        band: (usize, usize, usize),
        mut line_attrs: impl FnMut(&str, usize, usize) -> glyphon::cosmic_text::AttrsList,
    ) {
        let (prefix, old_end, new_end) = band;
        let mut replacement = Vec::with_capacity(new_end - prefix);
        for (offset, &text) in new_lines[prefix..new_end].iter().enumerate() {
            let line_index = prefix + offset;
            if line_index < old_end {
                // `set_text` keeps the reused line's cache when its text and
                // attributes are unchanged, otherwise it resets only this line.
                let mut line = std::mem::replace(
                    &mut self.buffer.lines[line_index],
                    glyphon::cosmic_text::BufferLine::new(
                        "",
                        glyphon::cosmic_text::LineEnding::None,
                        glyphon::cosmic_text::AttrsList::new(attrs),
                        glyphon::Shaping::Advanced,
                    ),
                );
                line.set_text(
                    text,
                    glyphon::cosmic_text::LineEnding::Lf,
                    line_attrs(text, line_starts[line_index], line_index),
                );
                replacement.push(line);
            } else {
                replacement.push(glyphon::cosmic_text::BufferLine::new(
                    text,
                    glyphon::cosmic_text::LineEnding::Lf,
                    line_attrs(text, line_starts[line_index], line_index),
                    glyphon::Shaping::Advanced,
                ));
            }
        }
        self.buffer.lines.splice(prefix..old_end, replacement);
    }

    pub(super) fn refresh_changed_row_geometry(&mut self, change: TextChange, width_stable: bool) {
        let geometry_at = self.text_sync_profile.then(crate::clock::Instant::now);
        // Retain the prior vertical partition only when every changed logical
        // line still owns the same number and heights of visual rows. The common
        // one-line edit becomes a binary row lookup plus replacement of that
        // line's horizontal geometry. Anything that can move later rows, alter
        // document-wide styling context, or change the wrap width invalidates.
        let mut lookup_probes = 0;
        let can_patch = change.can_retain_geometry(width_stable)
            && (change.prefix..change.new_end).all(|line| {
                let expected = self.buffer.lines[line].layout_opt().map(Vec::len);
                expected.is_some()
                    && self
                        .row_geom
                        .cached_line_row_count(line, &mut lookup_probes)
                        == expected
            });
        let mut rows_materialized = 0;
        let mut patches = if can_patch {
            (change.prefix..change.new_end)
                .map(|line| {
                    self.line_rows_local_shaped(line).map(|rows| {
                        rows_materialized += rows.len() as u64;
                        (line, rows)
                    })
                })
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        if can_patch
            && patches.len() == change.new_end - change.prefix
            && let Some(work) = self.row_geom.patch_lines_if_stable(&mut patches)
        {
            self.last_text_sync_phases.geometry_lines_patched = patches.len() as u64;
            self.last_text_sync_phases.geometry_rows_patched = work.rows_patched;
            self.last_text_sync_phases.geometry_index_probes = work.index_probes;
            self.last_text_sync_phases.geometry_patch_hits = 1;
        } else {
            // A structural edit, changed vertical partition, or absent prior
            // frame can move later rows. Rebuild lazily from the shaped buffer.
            self.row_geom.invalidate();
        }
        self.last_text_sync_phases.geometry_rows_materialized = rows_materialized;
        self.last_text_sync_phases.geometry_index_probes += lookup_probes;
        self.last_text_sync_phases.geometry_ms = super::profile_elapsed_ms(geometry_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_geometry_gate_enrolls_only_nonempty_local_equal_line_bands() {
        let ordinary = TextChange {
            prefix: 50,
            old_end: 51,
            new_end: 51,
            geometry_safe: true,
        };
        assert!(ordinary.can_retain_geometry(true));

        let rejected = [
            TextChange {
                prefix: 50,
                old_end: 50,
                new_end: 50,
                geometry_safe: true,
            },
            TextChange {
                prefix: 50,
                old_end: 51,
                new_end: 52,
                geometry_safe: true,
            },
            TextChange {
                geometry_safe: false,
                ..ordinary
            },
        ];
        for change in rejected {
            assert!(!change.can_retain_geometry(true));
        }
        assert!(!ordinary.can_retain_geometry(false));
    }
}
