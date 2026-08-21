//! Sidecar fields whose honest shape depends on whether a document exists.

use crate::render::{TextPipeline, ViewState};
use anyhow::Result;

pub(super) struct Fields {
    pub summary: String,
    pub text_origin: String,
    pub line_count: usize,
    pub cursor: String,
    pub text: String,
    pub first_lines: String,
    pub layout: String,
}

pub(super) fn fields(view: &ViewState, pipeline: &TextPipeline) -> Result<Fields> {
    let active = pipeline.document_active();
    let actions = pipeline
        .start_actions()
        .iter()
        .map(|label| super::sidecar::json_string(label))
        .collect::<Vec<_>>()
        .join(", ");
    let summary = format!("{{ \"active\": {active}, \"start_actions\": [{actions}] }}");
    if !active {
        return Ok(Fields {
            summary,
            text_origin: "null".to_string(),
            line_count: 0,
            cursor: "null".to_string(),
            text: "null".to_string(),
            first_lines: String::new(),
            layout: "null".to_string(),
        });
    }
    let first_lines = view
        .text
        .lines()
        .take(12)
        .map(super::sidecar::json_string)
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Fields {
        summary,
        text_origin: super::layout_sidecar::text_origin_json(pipeline),
        line_count: pipeline.line_count(),
        cursor: format!(
            "{{ \"line\": {}, \"col\": {} }}",
            view.cursor_line, view.cursor_col
        ),
        text: super::sidecar::json_string(&view.text),
        first_lines,
        layout: super::layout_sidecar::from_pipeline(pipeline)?,
    })
}
