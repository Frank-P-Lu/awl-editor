use super::*;

/// Work performed by one successful retained geometry refresh. Index probes
/// count comparisons in the binary search over the document partition; rows
/// patched counts the changed rows subsequently validated and replaced.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::render) struct RowPatchWork {
    pub(in crate::render) index_probes: u64,
    pub(in crate::render) rows_patched: u64,
}

fn first_row_for_line(rows: &[FrameVisualRow], line: usize, probes: &mut u64) -> usize {
    let (mut left, mut right) = (0, rows.len());
    while left < right {
        *probes += 1;
        let mid = left + (right - left) / 2;
        if rows[mid].logical_line < line {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
}

impl RowGeom {
    /// Reject changed wrapping or partial shaping before materializing glyph
    /// boundaries for a line the retained partition cannot accept.
    pub(in crate::render) fn cached_line_row_count(
        &self,
        line: usize,
        probes: &mut u64,
    ) -> Option<usize> {
        let guard = self.frame_rows.borrow();
        let rows = guard.as_ref()?;
        let first = first_row_for_line(rows, line, probes);
        let end = first_row_for_line(rows, line.checked_add(1)?, probes);
        Some(end - first)
    }

    /// Replace the cached rows for a text-only changed band when its shaped row
    /// partition is vertically identical to the prior frame. A changed row count
    /// or height can move every later row, so either rejects the patch and leaves
    /// the caller to rebuild the whole table. Horizontal glyph positions and
    /// baselines are refreshed from the newly shaped lines.
    pub(in crate::render) fn patch_lines_if_stable(
        &self,
        patches: &mut [(usize, Vec<LocalVisualRow>)],
    ) -> Option<RowPatchWork> {
        if patches.is_empty()
            || self.tops.borrow().is_none()
            || self.heights.borrow().is_none()
            || self.line_tops.borrow().is_none()
            || self.line_baselines.borrow().is_none()
            || self.line_last_tops.borrow().is_none()
            || self.line_last_baselines.borrow().is_none()
        {
            return None;
        }
        let line_baseline_count = self.line_baselines.borrow().as_ref()?.len();
        let line_last_baseline_count = self.line_last_baselines.borrow().as_ref()?.len();

        let (locations, work) = {
            let frame_rows_guard = self.frame_rows.borrow();
            let frame_rows = frame_rows_guard.as_ref()?;
            let mut locations = Vec::with_capacity(patches.len());
            let mut work = RowPatchWork::default();
            for (line, replacement) in patches.iter() {
                if replacement.is_empty()
                    || *line >= line_baseline_count
                    || *line >= line_last_baseline_count
                {
                    return None;
                }
                let first = first_row_for_line(frame_rows, *line, &mut work.index_probes);
                let mut end = first;
                while end < frame_rows.len() && frame_rows[end].logical_line == *line {
                    end += 1;
                }
                work.rows_patched += (end - first) as u64;
                if end - first != replacement.len()
                    || frame_rows[first..end]
                        .iter()
                        .zip(replacement)
                        .any(|(old, new)| old.row.line_height != new.row.line_height)
                {
                    return None;
                }
                locations.push(first..end);
            }
            (locations, work)
        };

        let mut frame_rows = self.frame_rows.borrow_mut();
        let frame_rows = frame_rows.as_mut()?;
        let mut line_baselines = self.line_baselines.borrow_mut();
        let line_baselines = line_baselines.as_mut()?;
        let mut line_last_baselines = self.line_last_baselines.borrow_mut();
        let line_last_baselines = line_last_baselines.as_mut()?;
        for ((line, replacement), indices) in patches.iter_mut().zip(locations) {
            let first = indices.start;
            let last = indices.end - 1;
            line_baselines[*line] = frame_rows[first].row.line_top + replacement[0].baseline_offset;
            line_last_baselines[*line] =
                frame_rows[last].row.line_top + replacement[replacement.len() - 1].baseline_offset;
            for (index, mut local) in indices.zip(replacement.drain(..)) {
                let top = frame_rows[index].row.line_top;
                local.row.line_top = top;
                frame_rows[index].row = local.row;
            }
        }

        self.frame_sealed.set(false);
        self.rows_line.set(None);
        *self.rows.borrow_mut() = None;
        self.generation.set(self.generation.get().wrapping_add(1));
        Some(work)
    }
}
