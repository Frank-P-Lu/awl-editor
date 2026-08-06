//! The markdown parser: pulldown-cmark events → `MdKind` spans in
//! document byte coordinates.

use super::detect::{setext_break_range, strike_engaged};
use super::kind::MdKind;
use super::markers::{
    push_delim, push_heading_markers, push_highlight_spans, push_inline_code, push_link_markers,
    push_list_marker, push_quote_markers, push_task_marker,
};
use crate::markdown::ConcealKind;
use crate::markdown::inline_images_on;
use crate::markdown::tables::push_table_markup;
use std::ops::Range;

/// Parse `text` into styling spans in DOCUMENT byte coordinates. Spans may
/// overlap by DESIGN: a link/code-block first pushes a whole-range `Markup`
/// span, then its inner text pushes a `LinkText`/`Code` span; the LATER (inner)
/// span wins its bytes via cosmic-text's "last span wins on overlap" rule.
///
/// A leading FRONTMATTER block ([`crate::frontmatter::detect`]) is carved off
/// first as one `ConcealMarkup(Frontmatter)` span; the rest of the document is
/// what pulldown parses, with every span offset by the block's byte length. No
/// (or malformed) frontmatter parses byte-identically to before.
pub fn spans(text: &str) -> Vec<(Range<usize>, MdKind)> {
    use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    let mut out: Vec<(Range<usize>, MdKind)> = Vec::new();
    let (text, body_offset) = match crate::frontmatter::detect(text) {
        Some(fm) => {
            out.push((
                0..fm.range.end,
                MdKind::ConcealMarkup(ConcealKind::Frontmatter),
            ));
            (&text[fm.range.end..], fm.range.end)
        }
        None => (text, 0),
    };
    let mut body: Vec<(Range<usize>, MdKind)> = Vec::new();
    // Nesting depth / context flags. Headings don't nest, so a single level is
    // enough; the emphasis/quote/link/code contexts use counters so a nested
    // construct restores the outer context on close.
    let mut heading: Option<u8> = None;
    let mut strong = 0u32;
    let mut emph = 0u32;
    let mut quote = 0u32;
    let mut link = 0u32;
    let mut code_block = 0u32;
    // STRIKETHROUGH nesting. `strike` counts only ENGAGED (exactly-two-tilde)
    // spans; pulldown also parses single-tilde `~x~` with the option on, which
    // awl deliberately keeps INERT (the `==` exactly-two precedent — the format
    // command inserts `~~`, so only `~~` means struck). `strike_engaged` is a
    // tiny per-Start stack so a skipped single-tilde span's `TagEnd` never
    // decrements a counter it didn't increment.
    let mut strike = 0u32;
    let mut strike_engaged_stack: Vec<bool> = Vec::new();
    // IMAGE nesting depth. An image's `![alt](path)` source is emitted as ONE
    // `ConcealMarkup(Image)` span over its whole range (see the `Tag::Image`
    // arm); while inside one, the inner alt-text `Event::Text` is SUPPRESSED
    // (the whole ref is concealed off-cursor and reveals as raw source
    // on-cursor, so a per-run styling span on the alt would be dead weight and
    // could mis-highlight an `==`/emphasis run in the alt). Gated on
    // `inline_images_on()` — off/wasm, no image span is pushed and the source
    // stays plain default-ink text (byte-identical to the pre-feature editor).
    let mut image = 0u32;
    // FENCE SYNTAX: `Some((lang, body_start, body_end))` while inside a FENCED code
    // block whose info string named a recognized language. The body byte extent is
    // grown across the block's Text events and lexed as ONE unit at the block's End,
    // so multi-line constructs (block comments, strings) resolve. Left `None` for an
    // indented block, or a fenced block with an unknown / absent language — those
    // keep the plain mono `Code` body and stay byte-identical.
    let mut fence: Option<(crate::syntax::Lang, Option<usize>, usize)> = None;
    // A CHECKED task colours its body text DIM. Set on the checked `TaskListMarker`
    // and cleared at the item's end; flat task lists (the common case) resolve
    // cleanly. A checked PARENT with nested children loses the flag to the child's
    // marker — accepted to keep the walk single-pass.
    let mut task_done = false;
    // TABLE: true while inside a `TableHead` (its cells get the `TableHeader` tag).
    // The pipes + separator row are emitted up-front from the whole table range on
    // `Tag::Table` (pulldown emits no event for either), so no per-row bookkeeping is
    // needed beyond this one header flag.
    let mut in_table_head = false;

    let level_u8 = |l: HeadingLevel| -> u8 {
        match l {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    };

    // ENABLE_TASKLISTS so `- [ ]` / `- [x]` surface as `TaskListMarker` events;
    // ENABLE_STRIKETHROUGH so `~~struck~~` surfaces as `Tag::Strikethrough`
    // (matching the export model's own option set — `export/model.rs` already
    // parsed it; the RENDER now catches up). Every other construct parses
    // exactly as before (the options are additive).
    let opts = Options::ENABLE_TASKLISTS | Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    for (ev, range) in Parser::new_ext(text, opts).into_offset_iter() {
        match ev {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    // ATX vs SETEXT: the Start-tag `range` opens on the line's
                    // first non-indent byte for both, so `#` there means ATX.
                    // awl is ATX-only (`headings.rs`); a `None` heading here lets
                    // `inline_kind` fall through, so a setext title styles plain.
                    if text[range.start..].starts_with('#') {
                        heading = Some(level_u8(level));
                        push_heading_markers(&mut body, text, &range);
                    } else if level == HeadingLevel::H2
                        && let Some(r) = setext_break_range(text, &range)
                    {
                        // DECIDED: `---` always draws as the rule, whatever precedes
                        // it — awl has no setext headings (ATX-only). A dash
                        // underline that independently qualifies as a thematic
                        // break (`is_thematic_break`, a real 3-or-more run) gets a
                        // genuine `Rule` span over its own bytes; the title line
                        // above is untouched (stays plain body via `heading`
                        // staying `None`). `===` (H1) is not a thematic-break
                        // syntax and is left alone. A too-short underline (a bare
                        // `-`, valid CommonMark setext but not a break) stays plain
                        // text too — `setext_break_range` returns `None` for it.
                        body.push((r, MdKind::Rule));
                    }
                }
                Tag::Strong => {
                    strong += 1;
                    push_delim(&mut body, &range, 2, ConcealKind::Emphasis);
                }
                Tag::Emphasis => {
                    emph += 1;
                    push_delim(&mut body, &range, 1, ConcealKind::Emphasis);
                }
                // STRIKETHROUGH: engage ONLY for exactly-two-tilde delimiters
                // (`~~struck~~`). pulldown's GFM option also parses single-tilde
                // `~x~`; awl keeps that inert (no marker span, no content span,
                // no strike line — the bytes render as plain text), mirroring the
                // `==` exactly-two rule. A `~~~` run is a FENCE at block level and
                // never reaches this inline arm.
                Tag::Strikethrough => {
                    // THE shared exactly-two-tilde gate (`strike_engaged`) — the
                    // export's `model::parse` reads the SAME owner, so a single
                    // `~x~` stays inert in both the render and every export.
                    let engaged = strike_engaged(&text[range.clone()]);
                    strike_engaged_stack.push(engaged);
                    if engaged {
                        strike += 1;
                        push_delim(&mut body, &range, 2, ConcealKind::Strikethrough);
                    }
                }
                Tag::BlockQuote(_) => {
                    quote += 1;
                    push_quote_markers(&mut body, text, &range);
                }
                Tag::CodeBlock(kind) => {
                    code_block += 1;
                    // Dim the WHOLE block (fences + info string); the body Text
                    // events below override their bytes to mono `Code`. A FENCED
                    // block's whole-range span is WYSIWYG-concealable (`Fence`) —
                    // its marker lines hide behind the always-present panel unless
                    // the caret sits inside the block; an INDENTED block has no
                    // fence to hide behind a panel, so it keeps the plain,
                    // non-concealing `Markup` (byte-identical to before this round).
                    let fenced = matches!(kind, CodeBlockKind::Fenced(_));
                    body.push((
                        range.clone(),
                        if fenced {
                            MdKind::ConcealMarkup(ConcealKind::Fence)
                        } else {
                            MdKind::Markup
                        },
                    ));
                    // A FENCED block whose info string names a recognized language
                    // arms the body accumulator; its End (below) lexes the body and
                    // emits per-role `CodeSyntax` spans over the mono body. An
                    // indented / unknown-lang / no-lang block leaves `fence` None.
                    if let CodeBlockKind::Fenced(info) = kind
                        && let Some(lang) = crate::syntax::Lang::from_info(&info)
                    {
                        fence = Some((lang, None, 0));
                    }
                }
                Tag::Link { .. } => {
                    link += 1;
                    // Conceal the `[` + `](url)` PLUMBING (WYSIWYG `Link`, line-
                    // scoped) while the inner Text pushes a `LinkText` span over the
                    // visible text (full content ink). Off the caret's line the
                    // plumbing hides to zero-width and only the text shows; on the
                    // line the whole `[text](url)` reveals for editing. A reference /
                    // malformed link with no `](` falls back to a plain dim `Markup`.
                    push_link_markers(&mut body, text, &range);
                }
                // IMAGE: the whole `![alt](path)` reference. Emitted as one
                // WYSIWYG-concealable span (line-scoped) so its source hides off
                // the caret's line while the decoded image draws in the tall row
                // the renderer reserves (the draw + the path/hint payload are
                // read back from this span's byte range — see
                // `render::TextPipeline::rebuild_image_rows`). Only when inline
                // images are ON (native + enabled): off/wasm pushes nothing, so
                // the source renders as plain text exactly as before this round.
                Tag::Image { .. } => {
                    // Only engage (span + alt-text suppression) when images are
                    // ON: off/wasm leaves `image` at 0 so the alt text flows
                    // through the ordinary Text path, byte-identical to before.
                    if inline_images_on() {
                        image += 1;
                        body.push((range.clone(), MdKind::ConcealMarkup(ConcealKind::Image)));
                    }
                }
                Tag::Item => push_list_marker(&mut body, text, &range),
                // TABLE: dim the structural markup (the `|` pipes on every row + the
                // whole `|---|` separator row) up-front from the table's byte range —
                // pulldown emits no event for either. Rendered as styled SOURCE, never
                // a drawn grid (awl is a source editor).
                Tag::Table(_) => push_table_markup(&mut body, text, &range),
                Tag::TableHead => in_table_head = true,
                // A HEADER cell's content (between the header row's pipes) gets the
                // `TableHeader` tag — a no-op full-ink transform (see `md_attrs`), so
                // it's only distinguishable in the sidecar, never in pixels.
                Tag::TableCell if in_table_head => {
                    body.push((range.clone(), MdKind::TableHeader));
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => heading = None,
                TagEnd::Strong => strong = strong.saturating_sub(1),
                TagEnd::Emphasis => emph = emph.saturating_sub(1),
                TagEnd::Strikethrough => {
                    if strike_engaged_stack.pop().unwrap_or(false) {
                        strike = strike.saturating_sub(1);
                    }
                }
                TagEnd::BlockQuote(_) => quote = quote.saturating_sub(1),
                TagEnd::CodeBlock => {
                    code_block = code_block.saturating_sub(1);
                    // The fenced body is complete: lex it as ONE unit and translate
                    // the syntax spans into DOCUMENT byte offsets (body_start + span).
                    // Pushed AFTER the body `Code` spans so a role span WINS its bytes
                    // (mono face from `Code`, role color from `CodeSyntax`); the fence
                    // markers + info string keep the earlier dim `Markup`.
                    if let Some((lang, Some(bs), be)) = fence.take()
                        && bs < be
                    {
                        for (r, role) in crate::syntax::spans(lang, &text[bs..be]) {
                            body.push((
                                bs + r.start..bs + r.end,
                                MdKind::CodeSyntax { role, lang },
                            ));
                        }
                    }
                }
                TagEnd::Link => link = link.saturating_sub(1),
                TagEnd::Image => image = image.saturating_sub(1),
                TagEnd::Item => task_done = false,
                TagEnd::TableHead => in_table_head = false,
                _ => {}
            },
            // A thematic break (`---`/`***`/`___` alone on a line): mark the literal
            // characters as a Rule; the renderer drops a centered fleuron on the row
            // and conceals the dashes unless the caret is editing the line.
            Event::Rule => body.push((range, MdKind::Rule)),
            // The `[ ]`/`[x]` checkbox. Style the marker (+ its trailing space)
            // distinctly; a CHECKED box also dims the item's body text.
            Event::TaskListMarker(checked) => {
                task_done = checked;
                push_task_marker(&mut body, text, &range, checked);
            }
            // Inside an IMAGE, the alt-text `Event::Text` is swallowed: the whole
            // `![alt](path)` is one concealable span already, so no per-run alt
            // styling is wanted (and an `==`/`*` in the alt must not highlight).
            Event::Text(_) if image > 0 => {}
            Event::Text(_) => {
                // FENCE SYNTAX: grow the recognized fenced block's body extent to
                // cover this text run (`range.start`/`.end` are copies, so `range`
                // still moves into the push below). Lexed at the block's End.
                if let Some((_, body_start, body_end)) = fence.as_mut() {
                    body_start.get_or_insert(range.start);
                    *body_end = range.end;
                }
                if let Some(k) =
                    inline_kind(heading, strong, emph, quote, link, code_block, task_done)
                {
                    body.push((range.clone(), k));
                }
                // STRIKETHROUGH: pushed ADDITIVELY over the context span above
                // (the `Highlight` precedent, but receding instead of lifting) —
                // last-wins on overlap means struck text inside a heading / quote
                // / bold / link run still takes the muted strike ink, and the
                // strike-line bucket (`render::rects`) covers exactly these
                // bytes. Never inside a code block (pulldown treats `~~` in code
                // as literal, so `strike` can't be armed there).
                if strike > 0 {
                    body.push((range.clone(), MdKind::Strikethrough));
                }
                // HIGHLIGHT: scan this text run for `==marked==` pairs, pushed
                // AFTER the context span above so a highlighted sub-range always
                // lifts back to the full content ink (mirrors `LinkText` lifting
                // off the whole-range `Markup`). Skipped inside a code block body
                // (fenced or indented) — `==` inside code is never a highlight;
                // inline code never reaches here at all (it arrives via
                // `Event::Code`, not `Event::Text`).
                if code_block == 0 {
                    push_highlight_spans(&mut body, text, &range);
                }
            }
            Event::Code(_) => push_inline_code(&mut body, text, &range),
            _ => {}
        }
    }
    // Shift every body-relative span back into DOCUMENT byte coordinates (a
    // no-op add of 0 when there was no frontmatter block) and append after the
    // frontmatter span pushed above.
    out.extend(
        body.into_iter()
            .map(|(r, k)| (r.start + body_offset..r.end + body_offset, k)),
    );
    out
}

/// Pick the content style for a Text event from the active context, in priority
/// order: a code block wins (mono), then a heading (it owns its whole line), then a
/// CHECKED task (the whole line recedes), then a link's visible text (accent), then
/// a blockquote (dim), then emphasis. Plain body text returns `None` (it rides the
/// default ink — no span needed).
fn inline_kind(
    heading: Option<u8>,
    strong: u32,
    emph: u32,
    quote: u32,
    link: u32,
    code_block: u32,
    task_done: bool,
) -> Option<MdKind> {
    if code_block > 0 {
        Some(MdKind::Code { inline: false })
    } else if let Some(l) = heading {
        Some(MdKind::Heading(l))
    } else if task_done {
        Some(MdKind::TaskDone)
    } else if link > 0 {
        Some(MdKind::LinkText)
    } else if quote > 0 {
        Some(MdKind::Quote)
    } else if strong > 0 && emph > 0 {
        Some(MdKind::BoldItalic)
    } else if strong > 0 {
        Some(MdKind::Bold)
    } else if emph > 0 {
        Some(MdKind::Italic)
    } else {
        None
    }
}
