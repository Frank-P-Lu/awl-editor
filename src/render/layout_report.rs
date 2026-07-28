//! Machine-readable layout facts borrowed from the exact sealed frame geometry.
//!
//! This module does not shape text, walk glyph runs, or assemble advances. The
//! report seam is deliberately unavailable until `TextPipeline::prepare` seals
//! the partition glyphon prepared for the frame.

use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct LayoutRowReport {
    pub index: usize,
    pub content: String,
    pub logical_line: usize,
    pub start_col: usize,
    pub end_col: usize,
    /// Absolute physical-pixel x boundaries for `start_col..=end_col`.
    pub xs: Vec<f32>,
    /// Absolute physical-pixel row top after scroll.
    pub top: f32,
    pub height: f32,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LayoutCaretReport {
    pub row: usize,
    pub logical_line: usize,
    pub col: usize,
    pub x: f32,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LayoutSelectionReport {
    pub row: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub x0: f32,
    pub x1: f32,
    /// The selection continues through this logical line's newline.
    pub to_next_line: bool,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LayoutReport {
    pub rows: Vec<LayoutRowReport>,
    pub caret: Option<LayoutCaretReport>,
    pub selection: Vec<LayoutSelectionReport>,
}

impl TextPipeline {
    /// Borrow the exact visual rows of the frame most recently prepared for draw.
    ///
    /// `None` means there is no sealed frame. This method never makes layout
    /// reportable by doing work itself: callers must prepare the frame first.
    pub(crate) fn layout_report(&self) -> Option<LayoutReport> {
        let text_left = self.text_left();
        let doc_top = self.doc_top();
        self.row_geom.with_report_rows(|frame_rows| {
            let rows = frame_rows
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let row = &entry.row;
                    let line = self
                        .buffer
                        .lines
                        .get(entry.logical_line)
                        .map(|line| line.text())
                        .unwrap_or("");
                    LayoutRowReport {
                        index,
                        content: chars_between(line, row.start_col, row.end_col),
                        logical_line: entry.logical_line,
                        start_col: row.start_col,
                        end_col: row.end_col,
                        xs: (row.start_col..=row.end_col)
                            .map(|col| text_left + row.xs.get(col).copied().unwrap_or(0.0))
                            .collect(),
                        top: doc_top + row.line_top,
                        height: row.line_height,
                    }
                })
                .collect();
            let caret = caret_report(
                frame_rows,
                self.cursor_line,
                self.cursor_col,
                self.caret_affinity,
                text_left,
            );
            let selection = selection_report(frame_rows, self.selection, text_left);
            LayoutReport {
                rows,
                caret,
                selection,
            }
        })
    }
}

fn chars_between(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn caret_report(
    rows: &[FrameVisualRow],
    line: usize,
    col: usize,
    affinity: crate::caret::Affinity,
    text_left: f32,
) -> Option<LayoutCaretReport> {
    let matching: Vec<(usize, &VisualRow)> = rows
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.logical_line == line)
        .map(|(index, entry)| (index, &entry.row))
        .collect();
    if matching.is_empty() {
        return None;
    }
    let local = if affinity == crate::caret::Affinity::Upstream {
        matching
            .iter()
            .position(|(_, row)| row.end_col == col && row.start_col < col)
    } else {
        None
    }
    .or_else(|| {
        matching
            .iter()
            .position(|(_, row)| col >= row.start_col && col < row.end_col)
    })
    .unwrap_or_else(|| {
        matching
            .iter()
            .rposition(|(_, row)| col >= row.start_col)
            .unwrap_or(matching.len().saturating_sub(1))
    });
    let (row_index, row) = matching[local];
    let col = col.min(row.xs.len().saturating_sub(1));
    Some(LayoutCaretReport {
        row: row_index,
        logical_line: line,
        col,
        x: text_left + row.xs.get(col).copied().unwrap_or(0.0),
    })
}

fn selection_report(
    rows: &[FrameVisualRow],
    selection: Option<((usize, usize), (usize, usize))>,
    text_left: f32,
) -> Vec<LayoutSelectionReport> {
    let Some((mut start, mut end)) = selection else {
        return Vec::new();
    };
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    if start == end {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (index, entry) in rows.iter().enumerate() {
        let line = entry.logical_line;
        if line < start.0 || line > end.0 {
            continue;
        }
        let row = &entry.row;
        let char_count = row.xs.len().saturating_sub(1);
        let line_start = if line == start.0 { start.1 } else { 0 };
        let line_end = if line == end.0 {
            end.1.min(char_count)
        } else {
            char_count
        };
        let start_col = line_start.max(row.start_col).min(row.end_col);
        let end_col = line_end.max(row.start_col).min(row.end_col);
        let is_last_row = rows
            .get(index + 1)
            .is_none_or(|next| next.logical_line != line);
        let to_next_line = line < end.0 && is_last_row && end_col == char_count;
        if start_col >= end_col && !to_next_line {
            continue;
        }
        out.push(LayoutSelectionReport {
            row: index,
            start_col,
            end_col,
            x0: text_left + row.xs.get(start_col).copied().unwrap_or(0.0),
            x1: text_left + row.xs.get(end_col).copied().unwrap_or(0.0),
            to_next_line,
        });
    }
    out
}
