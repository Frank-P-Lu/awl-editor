use super::*;
use std::sync::Arc;

pub struct ViewState {
    /// Whether this frame represents a real active document. `false` is the
    /// explicit post-close start state: the world remains, while page, prose,
    /// caret, outline, and filesystem identity are absent.
    pub document_active: bool,
    pub text: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    /// The caret's wrap AFFINITY (see [`crate::caret::Affinity`]): which visual row
    /// the caret RENDERS on when `cursor_col` lands exactly on a shared soft-wrap
    /// boundary. `Upstream` (set by a visual line-END motion) renders on the UPPER
    /// row's trailing edge; `Downstream` (the default) on the lower row's leading
    /// edge. Read ONLY by the caret's own placement (`caret_affinity`), so every
    /// other overlay is unaffected.
    pub caret_affinity: crate::caret::Affinity,
    pub scroll: ScrollPos,
    pub zoom: f32,
    /// Active selection as ordered ((line0,col0),(line1,col1)) endpoints, or
    /// `None` when there is no selection. line0/col0 is the earlier endpoint.
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub preedit: String,
    pub misspelled: Vec<crate::spell::Misspelling>,
    pub is_edit_move: bool,
    pub held: bool,
    /// True while a live MOUSE text-selection drag is in progress (the left
    /// button is down and the gesture is a text drag, `App::dragging`). Drives
    /// the DRAG-BAR: while dragging, the caret renders as the thin insertion BAR
    /// (the I-beam form) regardless of the configured caret mode, then returns to
    /// the configured look on release. A pure state bit — the mouse POINTER's own
    /// i-beam cursor is a separate OS concern. The headless capture/test paths
    /// leave this `false` (real mouse drag isn't replayable); a render test drives
    /// it directly through this field.
    pub selecting_drag: bool,
    /// Active isearch matches as ordered ((l0,c0),(l1,c1)) CHAR ranges in
    /// document order. Empty when search inactive or zero hits. Same coordinate
    /// convention as `selection`, so highlight rects reuse the selection rect
    /// algorithm.
    pub search_matches: Vec<((usize, usize), (usize, usize))>,
    pub search_current: Option<usize>,
    pub search_query: String,
    pub search_active: bool,
    pub search_case_sensitive: bool,
    pub search_replace_active: bool,
    pub search_replacement: String,
    pub search_editing_replacement: bool,
    pub search_query_caret: usize,
    pub search_replacement_caret: usize,
    pub overlay_active: bool,
    pub overlay_align: Option<theme::CardAnchor>,
    pub overlay_crisp: bool,
    pub overlay_query: String,
    pub overlay_query_caret: usize,
    /// The active selection in `overlay_query`, as CHAR indices `(start, end)`
    /// — armed only while a Rename minibuffer's seeded stem-selection hasn't
    /// yet been collapsed (`OverlayState::query`'s own [`crate::textbox::
    /// TextBox::selection_range`]). `None` for every other card and field,
    /// and for Rename itself once the first keystroke or motion collapses it.
    pub overlay_query_selection: Option<(usize, usize)>,
    pub overlay_title: String,
    pub overlay_row_path_splits: bool,
    pub overlay_items: Vec<String>,
    /// The faceted picker's UNLENSED, UNFILTERED display corpus, augmented with
    /// every empty-state and static chrome label. Right-anchored cards measure
    /// their hug width from this summon-time roster, never the current lens or
    /// query projection, so their left edge stays put while the right rail holds.
    pub overlay_hug_roster: Option<Arc<crate::overlay::HugRoster>>,
    pub overlay_empty: Option<String>,
    pub overlay_bindings: Vec<String>,
    /// Settings range-rail positions parallel to `overlay_items`; `None` marks
    /// an ordinary row with no range control.
    pub overlay_ranges: Vec<Option<f32>>,
    pub overlay_times: Vec<String>,
    pub overlay_git: Vec<String>,
    pub overlay_selected: usize,
    /// The scroll WINDOW's top row: the `overlay_items` index of the FIRST visible row.
    /// Owned by [`crate::overlay::OverlayState::scroll`] (the source of truth for the
    /// list's scroll position); the pipeline reads it straight so the drawn rows + the
    /// hover hit-test share ONE window and can never disagree.
    pub overlay_scroll: usize,
    pub overlay_window_rows: usize,
    pub overlay_hint: String,
    /// THEME PICKER only: the faceting lens STRIP — each lens label plus a flag
    /// marking the ACTIVE one (emphasized by VALUE + a thin underline, never amber).
    /// In strip order with All parked at the far left. EMPTY for every other overlay
    /// kind (so the pipeline draws no strip). Drives the theme picker's branch.
    pub overlay_lens: Vec<(String, bool)>,
    /// Is the summoned card drawn as a SUMMONED WORKSPACE (viewport,
    /// two coordinated regions, document as a quiet backdrop) rather than a
    /// contextual card? Owned by [`crate::overlay::OverlayKind::workspace_shape`]
    /// (`Some` of either shape); the renderer never re-tests the kind.
    pub overlay_workspace: bool,
    /// Within a summoned workspace, does the PRIMARY (narrow)
    /// column carry the workspace's own ROWS (a future timeline), rather than
    /// category labels? The one fact `render::chrome::workspace_geometry`
    /// reduces to for which region is which; owned by
    /// [`crate::overlay::workspace::WorkspaceShape::rows_are_primary`].
    /// `false` off a workspace and for `RailOverRows` (Settings); `true` for the
    /// History timeline.
    pub overlay_rows_primary: bool,
    /// Is the workspace's CONTENT region carrying read-only
    /// COMPARISON PROSE this frame? `true` exactly when the pushed `text` is a
    /// comparison transcript rather than the user's own document
    /// (`App::comparison_transcript` resolved a
    /// [`crate::overlay::ComparisonRequest`]).
    ///
    /// Distinct from [`Self::overlay_rows_primary`], which says the SHAPE has a
    /// comparison region, and necessary because the shape can be up with nothing
    /// to show — an empty history, or a query that filters every version away.
    /// Relocating the document layer there on such a frame would put the user's
    /// LIVE document in the comparison's place: a third readable layer, which is
    /// three competing readable layers, which is what this composition removes.
    pub overlay_comparison: bool,
    pub overlay_sections: Vec<String>,
    /// **THE SUMMONED PICKER'S SECONDARY LOCATION** — the active
    /// category, `None` at the All home and off a faceting picker.
    ///
    /// The projection of [`crate::overlay::OverlayState::location`], which is
    /// its one owner. The renderer reads the hierarchy from here rather than
    /// re-deriving it from the strip, so a world's own expression of the cue —
    /// wherever a world chooses to put it — is an expression of the SAME datum
    /// the strip's active mark is.
    pub overlay_location: Option<String>,
    pub caret_preview: Option<CaretMode>,
    pub gutter_name: String,
    pub gutter_project: String,
    /// THE VISIBLE WORKING SET the bottom identity widens into: one row per file
    /// open under the ACTIVE project root, in stable open order, or EMPTY when
    /// that root holds a single file.
    ///
    /// Empty is not merely "nothing to draw" — it is the one-file contract. The
    /// gutter's single-name path is what shipped before this surface existed, so
    /// an empty vector here routes the frame back through it unchanged rather
    /// than through a stack of one that happens to look the same.
    /// [`crate::workingset::WorkingSet::stack_rows`] is the sole author of that
    /// emptiness.
    pub gutter_files: Vec<crate::workingset::StackRow>,
    /// THE PERSISTENT `changed elsewhere` AFFORDANCE: is the active document's
    /// file holding an unresolved external change right now?
    ///
    /// A notice cannot carry this. There is exactly ONE notice slot and a toast
    /// expiry clears it, so an unrelated "copied" can transiently take the
    /// conflict's line and leave nothing behind it — no work is at risk (every
    /// write door refuses regardless of what is on screen), but the user is left
    /// with no way to see the state they are in. This is chrome instead: it is
    /// true for as long as the conflict is, and it is drawn beside the filename
    /// because the filename is the thing that changed underneath.
    pub gutter_changed: bool,
    /// The user's config `[keys]` overrides — the SAME slice
    /// [`crate::overlay::BuildCtx::config_keys`] hands the palette
    /// (`commands::visible_effective_bindings`). Read by the awl-drawn menu
    /// bar's chord column (`render::chrome::menubar::dropdown`) so a rebind
    /// updates that label through the ONE owner both surfaces share, never a
    /// second config-blind lookup. Empty on every scaffold that has no live
    /// config (bench/perf/capture defaults), which is inert — an empty override
    /// list never matches a command, so the static default chord shows.
    pub config_keys: Vec<(String, Vec<String>)>,
    /// [`crate::config::Config::effective_linux_keep`] — the composed Linux
    /// keep-list (the unconditional builtin floor + the `keymap = "emacs"`
    /// preset + any explicit `linux_keep_emacs` entries), mirrored beside
    /// [`Self::config_keys`] for the same menu-bar reader. Empty by default,
    /// which suppresses nothing (`linux_keeps_chord` is false against an empty
    /// list) — the config-free scaffold default, byte-identical to today's
    /// behavior on every convention other than a configured Linux one.
    pub config_linux_keep: Vec<String>,
    /// The config `keymap` flavor, beside [`Self::config_linux_keep`] for the
    /// SAME menu-bar/palette readers (`commands::join_slots_truthful`/
    /// `commands::menu_native_label`) — a seeded layer chord (Linux
    /// `keymap = "emacs"`'s `M-x`) can only be advertised once the label
    /// layer knows which flavor is active, not just which chords are kept.
    /// `Native` on the config-free scaffold default, matching every other
    /// mirrored config field here.
    pub config_keymap_flavor: crate::keymap::KeymapFlavor,
    pub is_markdown: bool,
    pub doc_dir: Option<std::path::PathBuf>,
    pub syn_lang: Option<crate::syntax::Lang>,
    pub overlay_spell: Option<(usize, usize, usize)>,
    /// Mirror of the live INSERT-TABLE dimension picker's sculpted
    /// `(rows, cols)`, `Some` only while [`crate::overlay::TableDimsEdit`] is
    /// active. Gates the picker's own dedicated geometry + quad-grid draw
    /// exactly the way `overlay_spell` gates the contextual spell popup's —
    /// an `Option` the render layer reads, never a kind string.
    pub overlay_table_dims: Option<(usize, usize)>,
    pub overlay_context_anchor: Option<(f32, f32)>,
    /// THE ASSET CLEANER's live PREVIEW input: the highlighted row's own
    /// image, as an absolute path, `Some` only on an
    /// [`crate::overlay::OverlayKind::Assets`] row
    /// ([`crate::overlay::OverlayState::selected_asset_path`] is the one
    /// owner — gated by the row's own `RowMeta::Asset`, never a kind check
    /// here). Gates the picker's second coordinated region (PHILOSOPHY §1):
    /// a preview panel beside the row list, decoded through the SAME
    /// inline-image cache — never a second decoder — the way
    /// [`Self::overlay_spell`]/[`Self::overlay_table_dims`] each gate their
    /// own dedicated geometry.
    pub overlay_asset_preview: Option<std::path::PathBuf>,
    /// THE CALM NOTICE's text this frame, empty when there is none.
    pub notice: String,
    /// THAT notice's KIND. A sticky notice does not expire, so it is the one the
    /// writer is expected to act on and the one a lifetime can never explain
    /// away; the renderer reads the kind rather than inferring urgency from the
    /// sentence. Meaningless while `notice` is empty (nothing is drawn).
    pub notice_kind: crate::actions::NoticeKind,
    pub cjk_priority: Vec<crate::frontmatter::Lang>,
    pub eol: crate::buffer::Eol,
    /// THE FORMAT POPOVER model for this frame ([`crate::popover::PopoverModel`]),
    /// or `None` when the popover is down. Built by the caller — the live App's
    /// `sync_view` (mouse-summoned + config-gated) or the capture force-summon
    /// probe (`AWL_POPOVER`) — from [`crate::actions::popover::plan`] over the
    /// current selection, so the lit toggles + the `H` button's level stay live
    /// and reflective across format applies. Drives the floating button row + its
    /// hit-test + the sidecar `popover` block. The row is ANCHORED off `selection`
    /// (its earlier endpoint), so a `Some` model always rides a live selection.
    /// `None` parks every popover quad/glyph empty, so a default capture is
    /// byte-identical.
    pub popover: Option<crate::popover::PopoverModel>,
    pub overlay_detail_focus: bool,
    /// COLLAPSED SECTIONS (folds): the FULL-document logical lines of the ATX
    /// headings whose sections are folded, ascending. VIEW state only — the rope is
    /// untouched. `text` above is already the FOLD-FILTERED document (hidden lines
    /// dropped by [`crate::fold::Filter`], so they shape to ZERO height — the row
    /// simply does not exist), and `cursor_line` / `selection` / `search_matches` /
    /// `misspelled` are in that same filtered line space; this field carries the
    /// (unfiltered) folded-heading lines so the sidecar can report the fold state
    /// and a future tail/chevron pass can find each collapsed heading. Empty when
    /// nothing is folded, so a default capture is byte-identical.
    pub folds: Vec<usize>,
    pub fold_tails: Vec<FoldTail>,
    /// The FILTERED-line-space lines of every currently FOLDED heading —
    /// [`Self::folds`]'s own set, remapped through the SAME [`crate::fold::Filter`]
    /// `fold_tails` rides, so both land in the space [`Self::text`] actually shapes
    /// (and the space `outline_headings`' own `Heading::line` is built from). Unlike
    /// `fold_tails`, this is NOT gated on a nonzero hidden count — a heading folded
    /// over an EMPTY section (immediately followed by a sibling-or-shallower
    /// heading) still belongs here, because "is this heading collapsed" is a fact
    /// about the fold set, not about how much it hides. This is the one signal the
    /// fold chevron reads to choose its shape: `›` while collapsed, `⌄` while
    /// expanded — [`crate::fold::chevron_revealed`] answers WHETHER the mark shows,
    /// this answers which way it points. Empty when nothing is folded (or during a
    /// History preview, which substitutes a diff transcript unrelated to any real
    /// fold), so a default capture stays byte-identical.
    pub folded_headings: Vec<usize>,
    /// The user's OWN document, when `text` above is a SUBSTITUTE for it rather
    /// than the document itself — a fold has dropped the hidden lines, or a
    /// History preview has replaced the whole thing with a diff transcript.
    /// `None` on every ordinary frame, where `text` IS the document and the
    /// shaped lines answer for it (so a default capture stays byte-identical).
    ///
    /// Exists because the card's DOCUMENT figures — word count, frontmatter
    /// language, through-doc percent — are facts about the user's document, and
    /// the renderer used to read them off whatever it happened to be shaping.
    /// Fold a section and the drawn WORD COUNT fell to the visible lines while
    /// the announced one (which reads the buffer) stayed whole; open a History
    /// preview and the drawn one counted a diff transcript. Written ONLY by
    /// [`ViewState::substitute_text`], the one door every substitution goes
    /// through.
    pub doc_source: Option<DocSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldTail {
    pub line: usize,
    pub hidden: usize,
}

/// The user's own document and the caret's place IN it — the input the pure
/// figure owner ([`crate::card::figures::DocFigures::of`]) reads when the shaped
/// `text` is a substitute. Carried as ONE value, for the same reason
/// `DocFigures` is gathered: a caller cannot take the text from the document and
/// the caret from the substitute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocSource {
    /// The document text, unfiltered and unsubstituted.
    pub text: String,
    /// The caret's logical line in that text, in CHARACTERS.
    pub cursor_line: usize,
    /// The caret's logical column in that text, in CHARACTERS.
    pub cursor_col: usize,
    /// The active selection in the original document's line/character space.
    /// Fold filtering remaps `ViewState::selection` for painting, but document
    /// statistics must retain the raw buffer region.
    pub selection: Option<((usize, usize), (usize, usize))>,
}

impl ViewState {
    /// The CANONICAL default `ViewState` — an empty, unscrolled, unzoomed prose
    /// buffer with every search / overlay field inert. This is the ONE owner of
    /// "what a fresh ViewState looks like": the bench / perf / frame / capture /
    /// test scaffolds all build on it (`ViewState { <real fields>, ..base() }`),
    /// so a NEW ViewState field is defaulted in exactly ONE place here and every
    /// scaffold inherits it automatically — retiring the old "update all six
    /// initializers or the build breaks at merge" ritual. The live App's
    /// `sync_view` (`src/app/viewstate.rs`) stays deliberately EXHAUSTIVE (it sets
    /// every field from live state and MUST fail to compile when a field is added,
    /// forcing a conscious render decision) — it is the one authoritative site and
    /// does not route through `base()`.
    ///
    /// Non-inert defaults: `zoom = range::ZOOM.default`, `overlay_window_rows = 12` (the no-overlay
    /// cap the pipeline windows against), `cjk_priority = DEFAULT_CJK_PRIORITY`, and
    /// `eol = Eol::Lf` — matching the value every scaffold previously spelled out.
    pub fn base() -> Self {
        ViewState {
            document_active: true,
            text: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            caret_affinity: crate::caret::Affinity::Downstream,
            scroll: ScrollPos::default(),
            zoom: crate::range::ZOOM.default,
            selection: None,
            preedit: String::new(),
            misspelled: Vec::new(),
            is_edit_move: false,
            held: false,
            selecting_drag: false,
            search_matches: Vec::new(),
            search_current: None,
            search_query: String::new(),
            search_active: false,
            search_case_sensitive: false,
            search_replace_active: false,
            search_replacement: String::new(),
            search_editing_replacement: false,
            search_query_caret: usize::MAX,
            search_replacement_caret: usize::MAX,
            overlay_active: false,
            overlay_align: None,
            overlay_crisp: false,
            overlay_query: String::new(),
            overlay_query_caret: usize::MAX,
            overlay_query_selection: None,
            overlay_title: String::new(),
            overlay_row_path_splits: false,
            overlay_items: Vec::new(),
            overlay_hug_roster: None,
            overlay_empty: None,
            overlay_bindings: Vec::new(),
            overlay_ranges: Vec::new(),
            overlay_times: Vec::new(),
            overlay_git: Vec::new(),
            overlay_selected: 0,
            overlay_scroll: 0,
            overlay_window_rows: 12,
            overlay_hint: String::new(),
            overlay_lens: Vec::new(),
            overlay_workspace: false,
            overlay_rows_primary: false,
            overlay_comparison: false,
            overlay_sections: Vec::new(),
            overlay_location: None,
            caret_preview: None,
            gutter_name: String::new(),
            gutter_project: String::new(),
            gutter_files: Vec::new(),
            gutter_changed: false,
            config_keys: Vec::new(),
            config_linux_keep: Vec::new(),
            config_keymap_flavor: crate::keymap::KeymapFlavor::default(),
            is_markdown: false,
            doc_dir: None,
            syn_lang: None,
            overlay_spell: None,
            overlay_table_dims: None,
            overlay_context_anchor: None,
            overlay_asset_preview: None,
            notice: String::new(),
            notice_kind: crate::actions::NoticeKind::default(),
            cjk_priority: crate::frontmatter::DEFAULT_CJK_PRIORITY.to_vec(),
            eol: crate::buffer::Eol::Lf,
            popover: None,
            overlay_detail_focus: false,
            folds: Vec::new(),
            fold_tails: Vec::new(),
            folded_headings: Vec::new(),
            doc_source: None,
        }
    }

    /// Replace the shaped [`Self::text`] with a SUBSTITUTE — the fold-filtered
    /// document, or a History preview's diff transcript — remembering the
    /// document (and the caret's place in it) in [`Self::doc_source`] first.
    ///
    /// THE one door for every text substitution. A caller that assigns
    /// `view.text` directly leaves the card's document figures derived from the
    /// substitute, which is the bug this exists to make unrepresentable: the
    /// drawn WORD COUNT would be over the visible lines while the announced one
    /// — read from the buffer by `App::card_inputs` — stayed over the whole
    /// document. Must be called BEFORE the caret is remapped or parked into the
    /// substitute's line space; it captures the caret it is given.
    ///
    /// Idempotent in the sense that matters: a second substitution keeps the
    /// FIRST recorded document, since that is still the user's own.
    pub fn substitute_text(&mut self, replacement: String) {
        let previous = std::mem::replace(&mut self.text, replacement);
        if self.doc_source.is_none() {
            self.doc_source = Some(DocSource {
                text: previous,
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
                selection: self.selection,
            });
        }
    }
}
