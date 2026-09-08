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

impl super::TextPipeline {
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
