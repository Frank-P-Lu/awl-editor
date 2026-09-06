//! The `MdKind` vocabulary — every span kind the parser can emit, its
//! sidecar tag, and the thematic-break syntax classification.

use crate::markdown::ConcealKind;

/// One styled span kind. Maps (in `render.rs`) to a concrete `Attrs` transform
/// over the base document attrs. `Markup` is the recede-to-dim role shared by
/// every syntax character; the rest style content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdKind {
    /// Syntax characters that recede to the DIM ink (`#`, `*`/`_`, backticks,
    /// `>`, fences, link brackets + URL). Still present + editable, just quiet.
    /// NOT WYSIWYG-concealable — see [`ConcealMarkup`](MdKind::ConcealMarkup) for
    /// the markup kinds that DO hide off the caret's line/block. `Markup` still
    /// covers a link's brackets + URL (which additionally carry their own
    /// [`ConcealMarkup`](MdKind::ConcealMarkup)`(`[`ConcealKind::Link`]`)` span)
    /// and an INDENTED (no-fence) code block's whole range — the indented block
    /// has no fence to hide behind a panel affordance, so it keeps the plain,
    /// non-concealing `Markup`. (The blockquote `>` marker is NO LONGER plain
    /// `Markup`: it conceals off-caret via [`ConcealKind::Blockquote`] now — the
    /// pull-quote round.)
    Markup,
    /// A WYSIWYG-concealable markup span — same DIM styling as [`MdKind::Markup`]
    /// (see `md_attrs`), but additionally hidden (transparent ink) per the
    /// reveal-on-cursor rule "if the caret is on that line, show the actual
    /// markdown; otherwise show the preview" (the PHILOSOPHY.md WYSIWYG
    /// amendment) — see [`ConcealKind`] for exactly which scope reveals which
    /// kind, and `render::spans::add_wysiwyg_conceal_spans` for the mechanism
    /// (mirrors the pre-existing `Rule`/bullet-marker conceal, generalized).
    /// Gated on `wysiwyg_on()`: OFF, this renders EXACTLY like plain `Markup`
    /// (dim, never concealed) — the sidecar tag is `"markup"` for both, so
    /// `md_spans` stays unchanged; the WYSIWYG state is reported separately.
    ConcealMarkup(ConcealKind),
    /// A heading's CONTENT text. Drives a larger font SIZE per [`heading_scale`]
    /// (applied per-line in `render.rs`) — no bold/color by DESIGN call: size + value
    /// carry the hierarchy on their own. (Inline [`Bold`](Self::Bold) does shape real
    /// bold on every world now; a heading just doesn't spend it.)
    Heading(u8),
    /// `**bold**` / `__bold__` content → Bold weight. Resolves to the world's real
    /// bundled 700 face on EVERY world — proportional and mono alike
    /// (`render::FONT_THEME_BOLD_FACES`).
    Bold,
    /// `*italic*` / `_italic_` content → Italic style.
    Italic,
    /// `***both***` content → Bold + Italic.
    BoldItalic,
    /// Inline `` `code` `` + fenced/indented code-block body → mono family + tint.
    /// `inline` distinguishes the two for the WYSIWYG wash: an INLINE span
    /// (`inline: true`) gets a small background PILL (see
    /// `render::rects::ensure_code_pill_protos`); a BLOCK body (`inline: false`,
    /// fenced or indented) does not — a fenced block instead gets the whole-block
    /// PANEL (from its [`ConcealKind::Fence`] span), and an indented block gets
    /// neither. The sidecar tag is `"code"` for both (unchanged).
    Code { inline: bool },
    /// A FENCED code-block body byte that a recognized info-string language lexed
    /// into an Alabaster syntax ROLE. It rides the SAME mono family as [`MdKind::Code`]
    /// (the fence body is mono) but takes the syntax role's VALUE-based color instead
    /// of the flat Code tint — so a ```` ```rust ```` fence highlights its comments /
    /// strings / constants / definitions in mono, while the fence markers + info
    /// string stay dim [`MdKind::Markup`]. Carries the `role` (which color) and the
    /// `lang` (for the sidecar). Emitted ONLY inside a recognized fence, so an
    /// unknown-lang / no-lang fence and every non-fence buffer stay byte-identical.
    CodeSyntax {
        role: crate::syntax::SynKind,
        lang: crate::syntax::Lang,
    },
    /// Blockquote TEXT → dim (the `>` marker is `Markup`).
    Quote,
    /// A list item's leading marker (`-`/`*`/`+`/`1.`) → dim.
    ListMarker,
    /// A link's visible TEXT → the buffer's full CONTENT ink (the brackets + URL
    /// are `Markup`) — DESIGN §3 keeps the accent for the caret alone. Also the
    /// `render::rects::Bucket::LinkUnderline` enrolment: the followable-span
    /// underline (`render::spans::link_underline_band`) hugs exactly this span's
    /// visual-row extents, whatever the caret's reveal state.
    LinkText,
    /// A bare (non-bracketed) URL's WHOLE source range — scheme through tail,
    /// spanning straight over its unspanned, always-visible authority — pushed
    /// FIRST by [`crate::markdown::spans::markers::push_bare_url_spans`], before
    /// its two flanking [`ConcealMarkup`](Self::ConcealMarkup)`(`[`ConcealKind::BareUrl`]`)`
    /// spans, so those win the scheme/tail bytes on overlap (last-wins, the
    /// `Highlight`-over-context precedent) while the authority gap between them
    /// falls through to this span's own no-op transform — i.e. identical to
    /// having no span there at all. Exists ONLY to enrol the SAME
    /// `render::rects::Bucket::LinkUnderline` bucket [`LinkText`](Self::LinkText)
    /// does: one grammar for every followable span, one shared band/pipeline,
    /// two span sources. Off-caret the underline naturally hugs the tamed
    /// authority + ellipsis slot (the concealed bytes collapse to near-zero
    /// width); on-caret it re-hugs the fully revealed URL — no separate
    /// persist/drop logic needed, the SAME `row.xs`-driven collapse the conceal
    /// mechanism already provides.
    BareUrlText,
    /// A task-list checkbox marker (`[ ]` open / `[x]` checked, plus its trailing
    /// space). The bool is the CHECKED state. Rendered distinctly by value: an open
    /// box stays present (full ink), a checked box recedes to the DIM ink — no accent,
    /// figure/ground by value, amber is the caret's alone (DESIGN §3).
    Task(bool),
    /// The TEXT of a CHECKED task item → DIM, so a completed line recedes the way a
    /// struck-through todo does. An open task's text rides the default ink.
    TaskDone,
    /// `==highlight==` content (the de-facto Obsidian/Typora/iA convention — NOT
    /// CommonMark, which has no `==` construct at all). Rendered as a highlighter
    /// stroke: the warm comment-wash quad BEHIND full content ink (reusing the
    /// existing wash pipeline — see `rects.rs::ensure_wash_protos`), never a color
    /// change on the text itself (no-op transform in `md_attrs`, like `Heading`).
    /// The `==` delimiters are separate `Markup` spans (dim, like every other
    /// syntax character). See [`push_highlight_spans`] for the delimiter rules
    /// (single `=` is deliberately meaningless; only an ISOLATED `==` pair counts).
    Highlight,
    /// `~~struck~~` content (GFM strikethrough, `ENABLE_STRIKETHROUGH`, gated to
    /// EXACTLY-two-tilde delimiters — a single `~x~`, which pulldown also accepts,
    /// stays inert, mirroring the `==` exactly-two rule). Struck text RECEDES: the
    /// content takes the muted strike ink (see `render::spans::strike_ink`, the one
    /// owner the drawn LINE shares) and the renderer draws a thin STRIKE LINE
    /// through the run (`render::rects` strike bucket → `strike_lines`, positioned
    /// by `render::spans::strike_line_band` — the SAME one owner the format
    /// popover's self-demonstrating `S` button rides). Never amber (DESIGN §3).
    /// The `~~` delimiters are separate [`ConcealMarkup`](Self::ConcealMarkup)
    /// spans ([`ConcealKind::Strikethrough`], line-scoped like Emphasis). Pushed
    /// ADDITIVELY over the context span (like [`Highlight`](Self::Highlight)), so
    /// struck text inside a heading/quote/bold run still dims + strikes.
    Strikethrough,
    /// A horizontal rule line (`---`/`***`/`___` alone on a line, INCLUDING a
    /// qualifying SETEXT `-` underline — awl has no setext headings). REVEAL-ON-
    /// CURSOR: the renderer drops a centered ornament (glyph per syntax, see
    /// [`BreakKind`]) and CONCEALS the raw glyphs off the caret's line, revealing
    /// them (dim, editable) on it (`spans::add_rule_conceal_span` +
    /// `TextPipeline::rule_lines`); this span only marks WHERE the rule is.
    Rule,
    /// A GitHub-flavored TABLE's cell-delimiter `|` pipe → dim `Markup` styling.
    /// awl is a SOURCE editor: a table renders as styled SOURCE (the structural
    /// `|` recedes to the muted ink like every other syntax character), NEVER a
    /// drawn grid widget. One span per literal `|` within a table's byte range
    /// (see [`push_table_markup`]); the sidecar tag is `"table_pipe"`.
    TablePipe,
    /// A GFM table's HEADER-SEPARATOR row (`|---|:--:|---|`) — the whole `-`/`:`/`|`
    /// run on that one line → dim `Markup` styling. pulldown emits no event for the
    /// separator row at all, so [`push_table_markup`] identifies it by shape (a line
    /// of only `|-: \t` containing a `-`). Sidecar tag `"table_sep"`.
    TableSep,
    /// A GFM table HEADER cell's CONTENT (the text between the first row's pipes) →
    /// a no-op transform in `md_attrs` (full CONTENT ink, exactly like [`Heading`](Self::Heading)
    /// / [`Highlight`](Self::Highlight) — NO amber, NO new accent; header vs body is
    /// the "safe minimum" value-only treatment). Emitted only so a header cell is
    /// distinguishable in the sidecar (`"table_header"`); it does not change pixels
    /// (body cells ride the same full default ink with no span).
    TableHeader,
    /// A recognized `[^label]` reference. The source range also carries
    /// `ConcealMarkup(Footnote)`; off-caret it collapses behind a separately
    /// shaped first-appearance display number, while reveal shows this range in
    /// quiet source ink. The number is the authored document's first-reference
    /// order, never the label or definition position.
    FootnoteReference(usize),
    /// The first-line `[^label]: ` prefix of a recognized definition. Like a
    /// reference, its raw bytes conceal behind the matching display number.
    FootnoteDefinition(usize),
    /// Definition prose (including continued indented lines), enrolled so the
    /// sidecar and laws distinguish composed footnote text from ordinary prose.
    /// No text transform: the quiet number + concealed prefixes carry hierarchy
    /// without washing out the actual note.
    FootnoteText,
}

/// WHICH of markdown's three thematic-break syntaxes a `Rule` line was typed with.
/// All three render a `<hr>` in standard markdown, but awl makes each EXPRESSIVE:
/// the syntax picks the ornament (see [`crate::theme::Ornaments`]). Detected from
/// the line's first run character by [`break_kind`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BreakKind {
    /// `---` (three-or-more dashes).
    Dash,
    /// `***` (three-or-more asterisks).
    Star,
    /// `___` (three-or-more underscores).
    Underscore,
}

/// Classify a thematic-break line by its RUN CHARACTER, per CommonMark: a thematic
/// break is a line of three-or-more matching `-`, `*`, or `_`, which MAY be
/// separated and surrounded by spaces/tabs (and indented up to 3 spaces). Since the
/// run char is uniform across a valid break, the FIRST non-space `-`/`*`/`_` decides
/// the kind. Callers only ask about lines pulldown already ruled a thematic break;
/// anything unexpected falls back to [`BreakKind::Dash`] (the plainest ornament).
pub fn break_kind(line: &str) -> BreakKind {
    for ch in line.chars() {
        match ch {
            '-' => return BreakKind::Dash,
            '*' => return BreakKind::Star,
            '_' => return BreakKind::Underscore,
            _ => {}
        }
    }
    BreakKind::Dash
}

impl MdKind {
    /// Stable tag string for the capture sidecar's `md_spans` block.
    pub fn tag(self) -> &'static str {
        match self {
            // A WYSIWYG-concealable markup span reports the SAME "markup" tag as
            // plain Markup — `md_spans` is unchanged by this round; the conceal
            // STATE (not the kind) is what's new, reported separately (see
            // `render::TextPipeline::wysiwyg_report`).
            MdKind::Markup | MdKind::ConcealMarkup(_) => "markup",
            MdKind::Heading(1) => "h1",
            MdKind::Heading(2) => "h2",
            MdKind::Heading(3) => "h3",
            MdKind::Heading(4) => "h4",
            MdKind::Heading(5) => "h5",
            MdKind::Heading(_) => "h6",
            MdKind::Bold => "bold",
            MdKind::Italic => "italic",
            MdKind::BoldItalic => "bold_italic",
            MdKind::Code { .. } => "code",
            // Role-only tag; the capture sidecar's `md_report` enriches a fence span
            // with its language (see `render::TextPipeline::md_report`).
            MdKind::CodeSyntax { role, .. } => match role {
                crate::syntax::SynKind::Comment => "code_comment",
                crate::syntax::SynKind::CommentCode => "code_comment_code",
                crate::syntax::SynKind::Str => "code_string",
                crate::syntax::SynKind::Constant => "code_constant",
                crate::syntax::SynKind::Definition => "code_definition",
            },
            MdKind::Quote => "quote",
            MdKind::ListMarker => "list_marker",
            MdKind::LinkText => "link_text",
            MdKind::BareUrlText => "bare_url_text",
            MdKind::Task(false) => "task_open",
            MdKind::Task(true) => "task_checked",
            MdKind::TaskDone => "task_done",
            MdKind::Highlight => "highlight",
            MdKind::Strikethrough => "strikethrough",
            MdKind::Rule => "rule",
            MdKind::TablePipe => "table_pipe",
            MdKind::TableSep => "table_sep",
            MdKind::TableHeader => "table_header",
            MdKind::FootnoteReference(_) => "footnote_ref",
            MdKind::FootnoteDefinition(_) => "footnote_def",
            MdKind::FootnoteText => "footnote_text",
        }
    }

    /// THE FOLLOWABLE-SPAN GRAMMAR'S OWN MEMBERSHIP PREDICATE — is this span
    /// one a person can FOLLOW? The ONE owner of that fact, read by both halves
    /// of the affordance so they cannot drift: `render::rects`'s
    /// `Bucket::LinkUnderline` enrolment (what wears the hairline that PROMISES
    /// a destination) and [`crate::markdown::follow::followable_at`] (what
    /// actually resolves one). A hairline over a span nothing follows, or a
    /// followable span wearing no hairline, is the defect this owner exists to
    /// make impossible.
    ///
    /// NO WILDCARD: a new [`MdKind`] fails to compile here until its author
    /// consciously answers "can this be followed?", which is the same
    /// forcing-function [`crate::render::ViewState`]'s exhaustive `sync_view`
    /// applies to a new view field.
    pub fn is_followable(self) -> bool {
        match self {
            MdKind::LinkText | MdKind::BareUrlText => true,
            MdKind::Markup
            | MdKind::ConcealMarkup(_)
            | MdKind::Heading(_)
            | MdKind::Bold
            | MdKind::Italic
            | MdKind::BoldItalic
            | MdKind::Code { .. }
            | MdKind::CodeSyntax { .. }
            | MdKind::Quote
            | MdKind::ListMarker
            | MdKind::Task(_)
            | MdKind::TaskDone
            | MdKind::Highlight
            | MdKind::Strikethrough
            | MdKind::Rule
            | MdKind::TablePipe
            | MdKind::TableSep
            | MdKind::TableHeader
            // A footnote REFERENCE is activated by the same `Action::FollowLink`
            // door, but it is not part of the underline grammar: it wears the
            // painted number ornament instead of the hairline, and its
            // destination is a line in this document rather than a place
            // outside it. `follow::followable_at` keeps that split.
            | MdKind::FootnoteReference(_)
            | MdKind::FootnoteDefinition(_)
            | MdKind::FootnoteText => false,
        }
    }

    /// True for the three GFM-table structural span kinds ([`MdKind::TablePipe`],
    /// [`MdKind::TableSep`], [`MdKind::TableHeader`]) — used to identify which
    /// document LINES are table rows so the double-space writing-nit is exempted on
    /// them (column alignment like `| Name  | Value |` is intentional, not a slip).
    /// See `render::rects::ensure_nit_protos`.
    pub fn is_table_markup(self) -> bool {
        matches!(
            self,
            MdKind::TablePipe | MdKind::TableSep | MdKind::TableHeader
        )
    }
}
