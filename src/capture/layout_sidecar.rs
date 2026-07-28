//! Serialization of the renderer-owned shaped-frame layout report.

use anyhow::Context;

use crate::render::{LayoutReport, TextPipeline};

use super::json_string;

pub(super) fn from_pipeline(pipeline: &TextPipeline) -> anyhow::Result<String> {
    let report = pipeline
        .layout_report()
        .context("capture layout report requested before the drawn frame was sealed")?;
    Ok(json(&report))
}

pub(super) fn json(report: &LayoutReport) -> String {
    let rows = report
        .rows
        .iter()
        .map(|row| {
            let xs = row
                .xs
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{{ \"index\": {}, \"content\": {}, \"line\": {}, \"start_col\": {}, \
                 \"end_col\": {}, \"xs\": [{}], \"top\": {}, \"height\": {} }}",
                row.index,
                json_string(&row.content),
                row.logical_line,
                row.start_col,
                row.end_col,
                xs,
                row.top,
                row.height
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let caret = match &report.caret {
        Some(caret) => format!(
            "{{ \"row\": {}, \"line\": {}, \"col\": {}, \"x\": {} }}",
            caret.row, caret.logical_line, caret.col, caret.x
        ),
        None => "null".to_string(),
    };
    let selection = report
        .selection
        .iter()
        .map(|segment| {
            format!(
                "{{ \"row\": {}, \"start_col\": {}, \"end_col\": {}, \"x0\": {}, \
                 \"x1\": {}, \"to_next_line\": {} }}",
                segment.row,
                segment.start_col,
                segment.end_col,
                segment.x0,
                segment.x1,
                segment.to_next_line
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ \"rows\": [{rows}], \"caret\": {caret}, \"selection\": [{selection}] }}")
}
