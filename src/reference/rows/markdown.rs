//! The authored Markdown dialect roster. The compact reference table and the
//! Supported Markdown guide both render this one list; neither is a hand mirror.

use super::super::Block;
use super::super::emit::{Cell, Table, escape_html};
use super::block;

#[derive(Clone, Copy)]
enum Portability {
    CommonMark,
    Extension,
    Awl,
}

impl Portability {
    fn label(self) -> &'static str {
        match self {
            Portability::CommonMark => "Core CommonMark",
            Portability::Extension => "Widely used extension",
            Portability::Awl => "awl-specific extension",
        }
    }
}

/// One syntax a writer can use. `tags` and `conceal` are the enrolment anchors:
/// laws below sweep the renderer's complete vocabularies against this roster.
struct Construct {
    name: &'static str,
    source: &'static str,
    rendered: &'static str,
    reveal: &'static str,
    command: &'static str,
    portability: Portability,
    tags: &'static [&'static str],
    conceal: &'static [crate::markdown::ConcealKind],
}

use crate::markdown::ConcealKind as C;

const CONSTRUCTS: &[Construct] = &[
    Construct {
        name: "ATX headings",
        source: "# Heading\n## Subheading",
        rendered: "A six-level heading ladder with larger type.",
        reveal: "The caret or a selection on the line shows its # markers.",
        command: "Heading; Cycle heading",
        portability: Portability::CommonMark,
        tags: &["h1", "h2", "h3", "h4", "h5", "h6"],
        conceal: &[C::Heading],
    },
    Construct {
        name: "Bold, italic, and bold italic",
        source: "**bold** and *italic* and ***both***",
        rendered: "Weighted or italic text.",
        reveal: "The caret or a selection on the line shows the delimiters.",
        command: "Bold; Italic",
        portability: Portability::CommonMark,
        tags: &["bold", "italic", "bold_italic"],
        conceal: &[C::Emphasis],
    },
    Construct {
        name: "Inline code",
        source: "Use `code` here.",
        rendered: "Monospace text with a quiet pill.",
        reveal: "The caret or a selection on the line shows the backticks.",
        command: "Inline code",
        portability: Portability::CommonMark,
        tags: &["code"],
        conceal: &[C::Code],
    },
    Construct {
        name: "Fenced code blocks",
        source: "```rust\nlet answer = 42;\n```",
        rendered: "A monospace block panel. Recognised language tags add quiet syntax roles.",
        reveal: concat!(
            "The caret anywhere in the block, or a selection touching it, shows its fences ",
            "and language tag.",
        ),
        command: "Code block",
        portability: Portability::CommonMark,
        tags: &[
            "code",
            "code_comment",
            "code_comment_code",
            "code_string",
            "code_constant",
            "code_definition",
        ],
        conceal: &[C::Fence],
    },
    Construct {
        name: "Indented code blocks",
        source: "    plain code",
        rendered: "A plain monospace code block.",
        reveal: "Its source remains visible; it has no fence to conceal.",
        command: "—",
        portability: Portability::CommonMark,
        tags: &["markup"],
        conceal: &[],
    },
    Construct {
        name: "Blockquotes",
        source: "> quoted text",
        rendered: "Quiet quoted text with a hanging quotation mark.",
        reveal: "The caret or a selection on the line shows the > marker.",
        command: "Blockquote",
        portability: Portability::CommonMark,
        tags: &["quote"],
        conceal: &[C::Blockquote],
    },
    Construct {
        name: "Bulleted and numbered lists",
        source: "- first\n- second\n\n1. one\n2. two",
        rendered: "A list with quiet markers.",
        reveal: "The caret or a selection on a line shows its marker.",
        command: "Bullet list; Numbered list",
        portability: Portability::CommonMark,
        tags: &["list_marker"],
        conceal: &[],
    },
    Construct {
        name: "Task lists",
        source: "- [ ] open\n- [x] done",
        rendered: "Open and completed task markers; completed text recedes.",
        reveal: "The source remains editable on its line.",
        command: "Task list",
        portability: Portability::Extension,
        tags: &["task_open", "task_checked", "task_done"],
        conceal: &[],
    },
    Construct {
        name: "Inline links",
        source: "[awl](https://awl-editor.fly.dev)",
        rendered: "Link text in the document ink, with a quiet baseline underline.",
        reveal: "The caret or a selection on the line shows brackets and destination.",
        command: "Insert link…",
        portability: Portability::CommonMark,
        tags: &["link_text"],
        conceal: &[C::Link],
    },
    Construct {
        name: "Images",
        source: "![A description](image.png)",
        rendered: concat!(
            "A local image when inline images are enabled; otherwise its source stays ",
            "visible.",
        ),
        reveal: "The caret or a selection on the line overlays editable source on a dimmed image.",
        command: "Paste can insert an image reference",
        portability: Portability::CommonMark,
        tags: &[],
        conceal: &[C::Image],
    },
    Construct {
        name: "YAML frontmatter",
        source: "---\ntitle: A note\nlang: en\n---",
        rendered: "Metadata is hidden away from the block when WYSIWYG is on.",
        reveal: concat!(
            "The caret anywhere in the block, or a selection touching it, reveals the ",
            "whole block.",
        ),
        command: "Tag document language",
        portability: Portability::Extension,
        tags: &[],
        conceal: &[C::Frontmatter],
    },
    Construct {
        name: "Thematic breaks",
        source: "---",
        rendered: "A centred section-break ornament.",
        reveal: "The caret or a selection on the line shows the typed break.",
        command: "—",
        portability: Portability::CommonMark,
        tags: &["rule"],
        conceal: &[],
    },
    Construct {
        name: "Tables",
        source: "| Name | Value |\n| --- | --- |\n| awl | editor |",
        rendered: "A composed grid; header cells and structure stay legible.",
        reveal: "The caret row, or rows touched by a selection, show their source over the grid.",
        command: "Align table",
        portability: Portability::Extension,
        tags: &["table_pipe", "table_sep", "table_header"],
        conceal: &[C::Table],
    },
    Construct {
        name: "Highlight",
        source: "==highlighted text==",
        rendered: "Full-ink text on a warm highlighter wash.",
        reveal: "The caret or a selection on the line shows the == delimiters.",
        command: "Highlight",
        portability: Portability::Awl,
        tags: &["highlight"],
        conceal: &[C::Highlight],
    },
    Construct {
        name: "Strikethrough",
        source: "~~removed text~~",
        rendered: "Receding text with a strike line.",
        reveal: "The caret or a selection on the line shows the ~~ delimiters.",
        command: "Strikethrough",
        portability: Portability::Extension,
        tags: &["strikethrough"],
        conceal: &[C::Strikethrough],
    },
    Construct {
        name: "Footnotes",
        source: "A note[^source]\n\n[^source]: Its text",
        rendered: "A quiet superscript reference and a composed numbered definition.",
        reveal: concat!(
            "The caret or a selection on the line shows the exact label, marker, ",
            "and indentation."
        ),
        command: "Insert footnote",
        portability: Portability::Extension,
        tags: &["footnote_ref", "footnote_def", "footnote_text"],
        conceal: &[C::Footnote],
    },
    Construct {
        name: "Bare URLs",
        source: "See https://example.com/track?utm_source=x for details.",
        rendered: concat!(
            "The domain, followed by a quiet ellipsis when a path or query is hidden, both ",
            "carrying the same quiet baseline underline as an inline link. A URL with nothing ",
            "past its domain shows in full, with no ellipsis.",
        ),
        reveal: "The caret or a selection on the line shows the full address.",
        command: "—",
        portability: Portability::Awl,
        tags: &["bare_url_text"],
        conceal: &[C::BareUrl],
    },
];

struct Different {
    name: &'static str,
    source: &'static str,
    outcome: &'static str,
}

const DIFFERENT: &[Different] = &[
    Different {
        name: "Setext headings",
        source: "A heading\n---",
        outcome: concat!(
            "Not a heading in awl. A run of three or more dashes renders as a thematic ",
            "break.",
        ),
    },
    Different {
        name: "Reference-style links",
        source: "[text][key]\n\n[key]: https://example.com",
        outcome: "No special link preview. The source remains editable text.",
    },
    Different {
        name: "Raw HTML",
        source: "<mark>text</mark>",
        outcome: "No HTML rendering. The source remains editable text.",
    },
];

pub(crate) fn documented_tags() -> Vec<&'static str> {
    CONSTRUCTS
        .iter()
        .flat_map(|c| c.tags.iter().copied())
        .collect()
}

pub(crate) fn documented_conceal_kinds() -> Vec<crate::markdown::ConcealKind> {
    CONSTRUCTS
        .iter()
        .flat_map(|c| c.conceal.iter().copied())
        .collect()
}

pub(crate) fn markdown() -> Vec<Block> {
    let mut constructs = Table::new(&["Construct", "Written as"]);
    for c in CONSTRUCTS.iter().filter(|c| !c.name.contains("(queued)")) {
        constructs.push(vec![
            Cell::text(c.name),
            Cell::code(c.source.replace('\n', " / ")),
        ]);
    }
    vec![block(
        Some("Constructs"),
        Some(concat!(
            "The file stays plain text. Only the render changes. The Supported Markdown ",
            "guide has full syntax and portability notes.",
        )),
        constructs,
    )]
}

pub(crate) fn supported_markdown_markdown() -> String {
    let mut out = String::from(concat!(
        "awl keeps documents as ordinary plain text. This page describes the syntax awl ",
        "renders; unsupported syntax remains editable text. It does not promise parity ",
        "with an unnamed Markdown dialect.\n\n## Syntax awl renders\n\n",
    ));
    for c in CONSTRUCTS {
        out.push_str(&format!(
            concat!(
                "### {}\n\n{}| awl renders | Caret and selection | Formatting command | ",
                "Portability |\n|---|---|---|---|\n| {} | {} | {} | {} |\n\n",
            ),
            c.name,
            markdown_example(c.source),
            c.rendered,
            c.reveal,
            c.command,
            c.portability.label()
        ));
    }
    out.push_str("## Not supported / deliberately different\n\n");
    for d in DIFFERENT {
        out.push_str(&format!(
            "### {}\n\n{}{}\n\n",
            d.name,
            markdown_example(d.source),
            d.outcome
        ));
    }
    out
}

/// A source example is itself Markdown and may include a fence. Use a longer
/// wrapper so every generated example stays copyable.
fn markdown_example(source: &str) -> String {
    let mut run = 0usize;
    let mut longest = 0usize;
    for ch in source.chars() {
        run = if ch == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}markdown\n{source}\n{fence}\n\n")
}

pub(crate) fn supported_markdown_html() -> String {
    let mut out = String::from(concat!(
        "<p>awl keeps documents as ordinary plain text. This page describes the syntax awl ",
        "renders; unsupported syntax remains editable text. It does not promise parity ",
        "with an unnamed Markdown dialect.</p>\n<h2>Syntax awl renders</h2>\n",
    ));
    for c in CONSTRUCTS {
        out.push_str(&format!(
            concat!(
                "<section class=\"markdown-construct\">\n<h3>{}</h3>\n",
                "<pre><code class=\"language-markdown\">{}</code></pre>\n",
                "<table><thead><tr><th>awl renders</th><th>Caret and selection</th>",
                "<th>Formatting command</th><th>Portability</th></tr></thead>",
                "<tbody><tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
                "</tr></tbody></table>\n</section>\n",
            ),
            escape_html(c.name),
            escape_html(c.source),
            escape_html(c.rendered),
            escape_html(c.reveal),
            escape_html(c.command),
            c.portability.label()
        ));
    }
    out.push_str("<h2>Not supported / deliberately different</h2>\n");
    for d in DIFFERENT {
        out.push_str(&format!(
            concat!(
                "<section class=\"markdown-construct\">\n<h3>{}</h3>\n",
                "<pre><code class=\"language-markdown\">{}</code></pre>\n",
                "<p>{}</p>\n</section>\n",
            ),
            escape_html(d.name),
            escape_html(d.source),
            escape_html(d.outcome)
        ));
    }
    out
}

pub(crate) fn supported_markdown_names() -> Vec<&'static str> {
    CONSTRUCTS.iter().map(|c| c.name).collect()
}
pub(crate) fn deliberately_different_names() -> Vec<&'static str> {
    DIFFERENT.iter().map(|d| d.name).collect()
}
