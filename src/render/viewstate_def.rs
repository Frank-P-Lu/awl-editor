use super::*;

pub struct ViewState {
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
    pub overlay_title: &'static str,
    pub overlay_row_path_splits: bool,
    pub overlay_items: Vec<String>,
    pub overlay_empty: Option<String>,
    pub overlay_bindings: Vec<String>,
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
    /// ITEM 114 — is the summoned card drawn as a SUMMONED WORKSPACE (viewport,
    /// two coordinated regions, document as a quiet backdrop) rather than a
    /// contextual card? Owned by [`crate::overlay::OverlayKind::workspace_shape`]
    /// (`Some` of either shape); the renderer never re-tests the kind.
    pub overlay_workspace: bool,
    /// ITEM 116a — within a summoned workspace, does the PRIMARY (narrow)
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
    pub caret_preview: Option<CaretMode>,
    pub gutter_name: String,
    pub gutter_project: String,
    pub is_markdown: bool,
    pub doc_dir: Option<std::path::PathBuf>,
    pub syn_lang: Option<crate::syntax::Lang>,
    pub overlay_spell: Option<(usize, usize, usize)>,
    pub overlay_context_anchor: Option<(f32, f32)>,
    pub notice: String,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldTail {
    pub line: usize,
    pub hidden: usize,
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
    /// Non-inert defaults: `zoom = 1.0`, `overlay_window_rows = 12` (the no-overlay
    /// cap the pipeline windows against), `cjk_priority = DEFAULT_CJK_PRIORITY`, and
    /// `eol = Eol::Lf` — matching the value every scaffold previously spelled out.
    pub fn base() -> Self {
        ViewState {
            text: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            caret_affinity: crate::caret::Affinity::Downstream,
            scroll: ScrollPos::default(),
            zoom: 1.0,
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
            overlay_title: "",
            overlay_row_path_splits: false,
            overlay_items: Vec::new(),
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
            caret_preview: None,
            gutter_name: String::new(),
            gutter_project: String::new(),
            is_markdown: false,
            doc_dir: None,
            syn_lang: None,
            overlay_spell: None,
            overlay_context_anchor: None,
            notice: String::new(),
            cjk_priority: crate::frontmatter::DEFAULT_CJK_PRIORITY.to_vec(),
            eol: crate::buffer::Eol::Lf,
            popover: None,
            overlay_detail_focus: false,
            folds: Vec::new(),
            fold_tails: Vec::new(),
        }
    }
}
