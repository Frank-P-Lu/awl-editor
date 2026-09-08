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
    // The folder-naming dim line's own state, verifiable from the sidecar
    // alone — `start_folder` is `null` whenever the start surface draws no
    // such line (no root worth naming, or a document is active), and
    // `start_goto_chord` is the SAME convention-truthful label
    // (`crate::render::chrome::start::goto_chord_label`) the line itself
    // drew, so a law can assert the exact chord glyph a capture shows rather
    // than recomputing a second, possibly-diverging expectation.
    let start_folder = match pipeline.start_folder() {
        Some(name) => super::sidecar::json_string(name),
        None => "null".to_string(),
    };
    let start_goto_chord = super::sidecar::json_string(&crate::render::chrome::goto_chord_label());
    let summary = format!(
        "{{ \"active\": {active}, \"start_actions\": [{actions}], \
         \"start_folder\": {start_folder}, \"start_goto_chord\": {start_goto_chord} }}"
    );
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
