//! Footnote side-table projection into live Markdown spans.

use super::MdKind;
use crate::markdown::ConcealKind;
use pulldown_cmark::{Event, Parser};
use std::ops::Range;

pub(super) type OffsetEvent<'a> = (Event<'a>, Range<usize>);

pub(super) fn events_and_spans<'a>(
    body: &mut Vec<(Range<usize>, MdKind)>,
    text: &'a str,
) -> Vec<OffsetEvent<'a>> {
    let events: Vec<_> = Parser::new_ext(text, crate::markdown::PARSE_OPTIONS)
        .into_offset_iter()
        .collect();
    let footnotes = crate::markdown::footnotes_from_events(text, &events);
    for reference in &footnotes.references {
        body.push((
            reference.range.clone(),
            MdKind::ConcealMarkup(ConcealKind::Footnote),
        ));
        body.push((
            reference.range.clone(),
            MdKind::FootnoteReference(reference.number),
        ));
    }
    for definition in &footnotes.definitions {
        push_definition(body, definition);
    }
    events
}

fn push_definition(
    body: &mut Vec<(Range<usize>, MdKind)>,
    definition: &crate::markdown::footnotes::FootnoteDefinition,
) {
    for (index, line) in definition.lines.iter().enumerate() {
        if !line.prefix.is_empty() {
            body.push((
                line.prefix.clone(),
                MdKind::ConcealMarkup(ConcealKind::Footnote),
            ));
            if index == 0 {
                body.push((
                    line.prefix.clone(),
                    MdKind::FootnoteDefinition(definition.number),
                ));
            }
        }
        if !line.content.is_empty() {
            body.push((line.content.clone(), MdKind::FootnoteText));
        }
    }
}
