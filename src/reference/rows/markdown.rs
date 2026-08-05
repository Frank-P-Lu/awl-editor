//! The markdown section's rows: which constructs awl renders, and which markup
//! hides while the caret is elsewhere.

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// One markdown construct as a reader meets it: what it is called, how it is
/// written, and which `MdKind` span tags it accounts for.
///
/// `tags` is the drift anchor, not decoration:
/// `super::law::every_markdown_span_tag_is_documented` collects every tag
/// `MdKind::tag` can produce and fails by name for any tag no row here claims,
/// so a new span kind cannot ship undocumented. The `syntax` column is authored
/// — it is an example of the writing, not a fact about the tree.
struct Construct {
    name: &'static str,
    syntax: &'static str,
    tags: &'static [&'static str],
}

const CONSTRUCTS: &[Construct] = &[
    Construct {
        name: "Heading, levels 1–6",
        syntax: "# Heading",
        tags: &["h1", "h2", "h3", "h4", "h5", "h6"],
    },
    Construct {
        name: "Bold",
        syntax: "**bold**",
        tags: &["bold"],
    },
    Construct {
        name: "Italic",
        syntax: "*italic*",
        tags: &["italic"],
    },
    Construct {
        name: "Bold italic",
        syntax: "***both***",
        tags: &["bold_italic"],
    },
    Construct {
        name: "Inline code and code blocks",
        syntax: "`code`",
        tags: &["code"],
    },
    Construct {
        name: "Syntax highlighting in a fenced block",
        syntax: "```rust",
        tags: &[
            "code_comment",
            "code_comment_code",
            "code_string",
            "code_constant",
            "code_definition",
        ],
    },
    Construct {
        name: "Blockquote",
        syntax: "> quoted",
        tags: &["quote"],
    },
    Construct {
        name: "List, bulleted or numbered",
        syntax: "- item",
        tags: &["list_marker"],
    },
    Construct {
        name: "Link",
        syntax: "[text](target)",
        tags: &["link_text"],
    },
    Construct {
        name: "Task list",
        syntax: "- [ ] task",
        tags: &["task_open", "task_checked", "task_done"],
    },
    Construct {
        name: "Highlight",
        syntax: "==highlight==",
        tags: &["highlight"],
    },
    Construct {
        name: "Strikethrough",
        syntax: "~~struck~~",
        tags: &["strikethrough"],
    },
    Construct {
        name: "Thematic break",
        syntax: "---",
        tags: &["rule"],
    },
    Construct {
        name: "Table",
        syntax: "| a | b |",
        tags: &["table_pipe", "table_sep", "table_header"],
    },
    Construct {
        name: "Syntax characters of every construct above",
        syntax: "# * ` > [ ] |",
        tags: &["markup"],
    },
];

pub(crate) fn documented_tags() -> Vec<&'static str> {
    CONSTRUCTS
        .iter()
        .flat_map(|c| c.tags.iter().copied())
        .collect()
}

/// Which markup hides while the caret is elsewhere. Every variant of
/// [`crate::markdown::ConcealKind`] appears, via a no-wildcard match — a new
/// conceal kind fails to compile until it declares its label and span.
pub(crate) fn conceal_facts_for(
    k: crate::markdown::ConcealKind,
) -> (&'static str, &'static str, &'static str) {
    use crate::markdown::ConcealKind;
    match k {
        ConcealKind::Heading => ("Heading", "The leading `#` run", "The line"),
        ConcealKind::Emphasis => ("Bold and italic", "The `*` or `_` delimiters", "The line"),
        ConcealKind::Code => ("Inline code", "The backticks", "The line"),
        ConcealKind::Highlight => ("Highlight", "The `==` delimiters", "The line"),
        ConcealKind::Strikethrough => ("Strikethrough", "The `~~` delimiters", "The line"),
        ConcealKind::Fence => (
            "Fenced code block",
            "Both fence lines and the info string",
            "The whole block",
        ),
        ConcealKind::Frontmatter => ("Frontmatter", "The whole `---` block", "The whole block"),
        ConcealKind::Table => (
            "Table",
            "The whole source, replaced by a drawn grid",
            "The whole block",
        ),
        ConcealKind::Image => ("Image", "The whole `![alt](path)` source", "The line"),
        ConcealKind::Link => ("Link", "The brackets and the target", "The line"),
        ConcealKind::Blockquote => ("Blockquote", "The `>` marker", "The line"),
    }
}

pub(crate) fn markdown() -> Vec<Block> {
    let mut constructs = Table::new(&["Construct", "Written as"]);
    for c in CONSTRUCTS {
        constructs.push(vec![Cell::text(c.name), Cell::code(c.syntax)]);
    }

    let mut conceal = Table::new(&[
        "Construct",
        "Hidden markup",
        "Revealed by",
        "Reveals in place",
    ]);
    for k in crate::markdown::ConcealKind::ALL {
        let (name, hidden, scope) = conceal_facts_for(k);
        conceal.push(vec![
            Cell::text(name),
            Cell::text(hidden),
            Cell::text(scope),
            Cell::text(if reveals_in_place(k) { "Yes" } else { "No" }),
        ]);
    }

    vec![
        block(
            Some("Constructs"),
            Some("The file stays plain text. Only the render changes."),
            constructs,
        ),
        block(
            Some("What hides off the caret"),
            Some(
                "With `wysiwyg = true`, the markup below hides while the caret \
                 and the selection are elsewhere.",
            ),
            conceal,
        ),
    ]
}

/// Asked of the renderer's OWN reveal rule rather than of a doc comment: does
/// putting the caret inside this construct's source un-conceal it where it
/// sits? A table answers no — its rows float over the drawn grid instead.
///
/// Both states are probed, not one. `conceal_off_cursor` is the caller's
/// already-computed "the caret's line is not this span's line", so a single
/// probe answers whichever question that flag was set to and reports it as if
/// it were the construct's property; asking for the caret INSIDE and OUTSIDE
/// and requiring the pair to differ is what actually measures a reveal.
fn reveals_in_place(k: crate::markdown::ConcealKind) -> bool {
    let span = 10..20;
    let inside = crate::render::wysiwyg_reveals(k, false, 12, &span, None);
    let outside = crate::render::wysiwyg_reveals(k, true, 40, &span, None);
    inside && !outside
}
