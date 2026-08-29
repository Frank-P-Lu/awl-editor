//! [`ConcealKind`] -- which WYSIWYG-concealable markup kind a
//! [`super::MdKind::ConcealMarkup`] span carries, and its reveal-scope law
//! (line-scoped vs the fence's block-scoped rule). Split out of the former
//! `markdown.rs` monolith (2026-07 code-organization pass); every item's
//! path is unchanged (`markdown::ConcealKind`) -- only the file it lives in
//! moved.

enum_with_all! {
    /// WHICH markdown construct a [`MdKind::ConcealMarkup`] span belongs to — the
    /// WYSIWYG amendment's dispatch key ("if the caret OR an active selection
    /// touches that line, show the actual markdown; otherwise show the preview" —
    /// widened 2026-07-22 to also reveal on an ACTIVE SELECTION, not just the
    /// caret; the renderer's one shared reveal decision, `wysiwyg_reveals` in
    /// `render/spans.rs`, is what actually implements this). Every kind but
    /// [`Fence`](Self::Fence)
    /// is LINE-scoped: it reveals when the caret sits on the span's OWN line (or
    /// the selection touches that line), exactly mirroring the pre-existing
    /// hr/bullet reveal-on-cursor, itself widened the same way. `Fence` is
    /// BLOCK-scoped: a fenced code block's marker lines reveal only when the caret
    /// is ANYWHERE inside the whole block, or the selection touches ANY line
    /// inside it, because the PANEL (drawn from the same span's byte range, always
    /// present) is the block's affordance — ducking the markers in and out per
    /// LINE inside a multi-line block the caret is actively editing would flicker
    /// distractingly.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ConcealKind {
        /// A heading's leading `#` run (+ a trailing ATX close, if any).
        Heading,
        /// A bold/italic emphasis delimiter run (`**`/`*`/`_`).
        Emphasis,
        /// An inline code span's backtick delimiters (the CONTENT keeps its own
        /// `MdKind::Code { inline: true }` span + wash pill — only the backticks hide).
        Code,
        /// A `==highlight==` delimiter pair (the wash stroke IS the affordance once
        /// the `==` marks hide — see `MdKind::Highlight`).
        Highlight,
        /// A `~~strikethrough~~` delimiter pair (GFM, gated to EXACTLY-two tildes,
        /// mirroring the `==` exactly-two rule) — LINE-scoped exactly like
        /// [`Emphasis`](Self::Emphasis): off the caret's line the `~~` marks hide to
        /// zero-width and the drawn STRIKE LINE (see `render::spans::strike_line_band`,
        /// the one strike-geometry owner) is the affordance; on the caret's own line
        /// the raw markers reveal for editing. The struck CONTENT keeps its own
        /// `MdKind::Strikethrough` span (muted ink + the line), never this kind.
        Strikethrough,
        /// A FENCED code block's ENTIRE range — both fence lines (open + close) and
        /// the info string. The renderer only ever conceals the MARKER lines from
        /// this span (never the body, which carries its own `Code`/`CodeSyntax`
        /// spans) — see `render::spans::add_wysiwyg_conceal_spans`. An INDENTED (no
        /// fence) code block has no marker to hide behind a panel, so it keeps the
        /// plain, non-concealing [`MdKind::Markup`] instead of this kind.
        Fence,
        /// A `---`-delimited FRONTMATTER block's ENTIRE range (see
        /// [`crate::frontmatter::detect`]) — BLOCK-scoped exactly like [`Fence`]
        /// (reveals iff the caret sits anywhere inside the block), reusing the SAME
        /// seam with zero new machinery. Unlike `Fence` there is no body sub-span
        /// to carve out (a frontmatter block is entirely markup, no highlighted
        /// content), so the whole range conceals/reveals as one unit.
        Frontmatter,
        /// A GFM TABLE's ENTIRE byte range — BLOCK-scoped exactly like [`Fence`]
        /// (the span reveals iff the caret sits anywhere inside the table, driving
        /// which ROW gets its raw source floated). Unlike every other conceal kind
        /// this one hides the WHOLE block's SOURCE (all rows — content, pipes,
        /// separator; the renderer replaces it with a drawn pixel GRID,
        /// `render::TextPipeline::prepare_table_grid`) at all times, caret or not.
        /// What the caret's presence changes is per-ROW, not per-block: grid and
        /// source can't share one row's pixels, so ONLY the row the caret currently
        /// sits on drops its drawn cells and shows its raw source floated in that
        /// row's band instead — every OTHER row of the same table keeps drawing its
        /// grid normally (the block is never parked wholesale). The dim
        /// `TablePipe`/`TableSep`/`TableHeader` spans still style the revealed row's
        /// source; this additive span only drives which table is "the caret's table"
        /// for that per-row swap.
        Table,
        /// A markdown IMAGE reference's ENTIRE `![alt](path)` source range —
        /// LINE-scoped exactly like [`Heading`](Self::Heading)/[`Emphasis`](Self::Emphasis)
        /// (an image ref is one line): reveals iff the caret is on the image's own
        /// line. Off-cursor the source conceals (zero-width) and the decoded image
        /// draws in the TALL row the line reserves (image height `h`). On-cursor the
        /// raw `![alt](path)` source reveals at body size CENTRED OVER the still-drawn,
        /// DIMMED image (the caption model, re-decided 2026-07-09) — the row stays
        /// exactly `h`, so the caret landing on / leaving the line causes ZERO reflow;
        /// a soft scrim band lifts the caption's legibility over the image pixels
        /// (`render::spans::build_line_attrs` / `render::layers::prepare_images`). This
        /// differs from the pure "heading model" the [`Table`](Self::Table) kind follows
        /// (grid parks entirely on reveal): an image shows source AND a dimmed preview at
        /// once. Emitted by
        /// [`spans`] ONLY when [`inline_images_on`] is true (native + enabled), so an
        /// images-off / wasm build emits no image span at all and renders the source
        /// byte-identically to the pre-feature editor.
        Image,
        /// A markdown link's MARKUP plumbing — the opening `[`, and the whole
        /// `](url)` tail (closing bracket, parens, destination + any title) — the
        /// LAST markup family that used to keep its brackets/URL visible as dim
        /// [`MdKind::Markup`]. LINE-scoped exactly like [`Heading`](Self::Heading)/
        /// [`Emphasis`](Self::Emphasis): off the caret's line the plumbing conceals
        /// to zero-width and only the link TEXT (its own [`MdKind::LinkText`] span,
        /// full content ink) shows, so `see [the essay](http://…)` reads as `see the
        /// essay`; on the caret's own line the full `[text](url)` source reveals for
        /// editing. Note the link TEXT is NOT part of this span (only the markup
        /// pieces are), so the conceal pass never hides the text. Emitted per
        /// [`push_link_markers`]; a reference-style / malformed link with no `](`
        /// falls back to the plain non-concealing [`MdKind::Markup`]. Calm — plain
        /// content ink, no hyperlink color, no amber (awl has no link accent).
        Link,
        /// A blockquote line's leading `>` marker run (`> `, or a nested `> > `, plus
        /// the trailing space) — LINE-scoped exactly like [`Heading`](Self::Heading)/
        /// [`Emphasis`](Self::Emphasis): off the caret's line the marker(s) conceal to
        /// zero-width, and the block's affordance is the big DIM hanging quotation mark
        /// the renderer hangs in the LEFT MARGIN at the block's first line (page mode
        /// only — see `render::TextPipeline::quote_marks` / `prepare_ornaments`). On
        /// the caret's own line the raw `>` markers reveal for editing. One
        /// [`push_quote_markers`] span per blockquote LINE (nested `>>` markers all
        /// live in one line's run, so they conceal together). The blockquote BODY text
        /// keeps its own [`MdKind::Quote`] styling span (dim or full, a taste flag).
        Blockquote,
        /// A footnote reference's complete `[^label]` source, or one structural
        /// prefix inside a definition (`[^label]: ` / continued-line indent).
        /// LINE-scoped: the quiet display number and composed prose replace the
        /// source off-caret; the caret or a selection touching that exact line
        /// reveals its authored bytes for direct editing.
        Footnote,
        /// A BARE URL typed as plain text (never `[text](url)` or an angle-bracket
        /// autolink, both of which are [`Link`](Self::Link) instead) — LINE-scoped
        /// exactly like every other line-scoped kind. TWO spans per URL flank the
        /// authority (host + optional port) the way [`Link`](Self::Link)'s plumbing
        /// flanks its visible text: a leading SCHEME span (`http://`/`https://`)
        /// always hides off-cursor, and a trailing TAIL span (everything from the
        /// first `/` or `?` after the authority) hides too, IF the URL carries one —
        /// a bare `https://example.com` with nothing past the authority gets no tail
        /// span and so no ellipsis promise. The authority itself carries no span at
        /// all: it is real, always-visible document text, never a painted
        /// substitute (unlike the tail's off-cursor affordance — a single quiet "…"
        /// glyph painted into the tail's reserved zero-width slot, see
        /// `render::spans::conceal::bare_url`). On the caret's own line both
        /// flanking spans reveal, so `https://example.com/track?x=1` shows in full
        /// for editing.
        BareUrl,
        /// A smart-punctuation run typed as prose — `--`, `---`, or `...` —
        /// LINE-scoped exactly like [`Emphasis`](Self::Emphasis). Unlike every
        /// other kind this is not markup at all (there is no delimiter/content
        /// split): the WHOLE literal run is the span, and off-cursor it paints
        /// a substitute glyph ([`crate::markdown::SmartPunctKind::glyph`]) into
        /// a reserved slot rather than collapsing to nothing — the bare-URL
        /// ellipsis's painted-substitute precedent, generalized to three
        /// possible glyphs instead of one fixed "…". On the caret's own line
        /// the literal bytes render exactly as plain prose always has (no
        /// dimming: this is real sentence punctuation, not syntax markup, so
        /// [`crate::render::spans::md_attrs`] carves it out of the "dim the
        /// markup" rule every other `ConcealMarkup` kind follows).
        SmartPunct,
    }
}

impl ConcealKind {
    /// Stable tag string for the capture sidecar's `wysiwyg.concealed` block.
    pub fn tag(self) -> &'static str {
        match self {
            ConcealKind::Heading => "heading",
            ConcealKind::Emphasis => "emphasis",
            ConcealKind::Code => "code",
            ConcealKind::Highlight => "highlight",
            ConcealKind::Strikethrough => "strikethrough",
            ConcealKind::Fence => "fence",
            ConcealKind::Frontmatter => "frontmatter",
            ConcealKind::Table => "table",
            ConcealKind::Image => "image",
            ConcealKind::Link => "link",
            ConcealKind::Blockquote => "blockquote",
            ConcealKind::Footnote => "footnote",
            ConcealKind::BareUrl => "bare_url",
            ConcealKind::SmartPunct => "smart_punct",
        }
    }
}
