//! The event fold, including the shared footnote roster and duplicate fallback.

use pulldown_cmark::{Event, Parser, Tag};

use super::{
    Block, Document, Frame, Inline, accept_block, close_frame, open_frame, plain_text, push_inline,
    push_text,
};

/// Parse `markdown` into a [`Document`]. Strips a leading frontmatter block
/// (excluded from the export, matching every other awl consumer) and folds the
/// pulldown event stream into the nesting-aware tree.
pub(in crate::export) fn parse(markdown: &str) -> Document {
    let body_start = crate::frontmatter::detect(markdown)
        .map(|f| f.range.end)
        .unwrap_or(0);
    let src = &markdown[body_start..];

    let events: Vec<_> = Parser::new_ext(src, crate::markdown::PARSE_OPTIONS)
        .into_offset_iter()
        .collect();
    let footnotes = crate::markdown::footnotes_from_events(src, &events);
    let duplicate_footnote_ranges: Vec<_> = events
        .iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::FootnoteDefinition(_))
                if !footnotes
                    .definitions
                    .iter()
                    .any(|definition| definition.range == *range) =>
            {
                Some(range.clone())
            }
            _ => None,
        })
        .collect();
    let mut stack: Vec<Frame> = vec![Frame::Root(Vec::new())];
    // A pending task marker: pulldown emits `TaskListMarker` at the START of the
    // item's paragraph, so we stash it and stamp the enclosing Item on close.
    let mut pending_task: Option<bool> = None;

    // The OFFSET iterator: each event carries its byte range into `src`, which the
    // strikethrough gate needs (the exactly-two-tilde decision reads the span's
    // source slice — see `crate::markdown::strike_engaged`).
    for (ev, range) in events {
        if let Some(duplicate) = duplicate_footnote_ranges
            .iter()
            .find(|duplicate| duplicate.start <= range.start && range.end <= duplicate.end)
        {
            if matches!(ev, Event::Start(Tag::FootnoteDefinition(_))) && range == *duplicate {
                accept_block(
                    &mut stack,
                    Block::Paragraph(vec![Inline::Text(src[duplicate.clone()].to_string())]),
                );
            }
            continue;
        }
        match ev {
            // STRIKETHROUGH is gated at its Start on the SHARED exactly-two-tilde
            // owner: an ENGAGED `~~x~~` opens a real `Strikethrough` frame; an
            // INERT single-tilde `~x~` opens a passthrough frame whose children
            // flush UNWRAPPED to the parent on close (so `~x~` is never struck,
            // matching the renderer). Every other Start routes through `open_frame`.
            Event::Start(Tag::Strikethrough) => {
                if crate::markdown::strike_engaged(&src[range.clone()]) {
                    stack.push(Frame::Strikethrough(Vec::new()));
                } else {
                    stack.push(Frame::StrikethroughInert(Vec::new()));
                }
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                let number = footnotes
                    .definitions
                    .iter()
                    .find(|definition| definition.range.start == range.start)
                    .map_or(footnotes.definitions.len() + 1, |definition| {
                        definition.number
                    });
                stack.push(Frame::FootnoteDefinition {
                    label: label.into_string(),
                    number,
                    blocks: Vec::new(),
                    loose: Vec::new(),
                });
            }
            Event::Start(tag) => open_frame(&mut stack, tag),
            Event::End(tag) => close_frame(&mut stack, tag, &mut pending_task),
            Event::Text(t) => push_text(&mut stack, &t),
            Event::Code(c) => push_inline(&mut stack, Inline::Code(c.into_string())),
            Event::SoftBreak => push_inline(&mut stack, Inline::SoftBreak),
            Event::HardBreak => push_inline(&mut stack, Inline::HardBreak),
            Event::Rule => accept_block(&mut stack, Block::Rule),
            Event::TaskListMarker(checked) => pending_task = Some(checked),
            // Raw HTML remains literal text.
            Event::Html(h) | Event::InlineHtml(h) => push_text(&mut stack, &h),
            Event::FootnoteReference(label) => {
                if let Some(reference) = footnotes
                    .references
                    .iter()
                    .find(|reference| reference.range == range)
                {
                    push_inline(
                        &mut stack,
                        Inline::FootnoteReference {
                            label: label.into_string(),
                            number: reference.number,
                            occurrence: reference.occurrence,
                        },
                    );
                }
            }
            Event::InlineMath(_) | Event::DisplayMath(_) => {}
        }
    }

    let blocks = match stack.pop() {
        Some(Frame::Root(b)) => b,
        _ => Vec::new(),
    };
    let title = first_heading_text(&blocks);
    Document { title, blocks }
}

/// The first `# heading`'s plain text — the document title (HTML `<title>`).
fn first_heading_text(blocks: &[Block]) -> Option<String> {
    for b in blocks {
        if let Block::Heading { inlines, .. } = b {
            let text = plain_text(inlines);
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }
    }
    None
}
