//! The narrow borrow the semantic FOLD is allowed to see.
//!
//! Building the accessibility tree is a READ. It needs the document, the
//! summoned-UI ladder, and three values a frame already computed — and nothing
//! else. Taking `&App` gave it the clipboard, the daemon socket, the usage
//! ledger and the GPU as well, so "what may the fold read" was a question only
//! a reader of every fold could answer.
//!
//! `SemanticView` answers it in the type. The whole App is read exactly once,
//! in `App::semantic_view`, and every fold below that line — `surfaces.rs`,
//! `passive.rs`, and the retained `projection.rs` — is handed this value and
//! cannot name `App` at all. `semantic_fold_reads_only_the_narrow_view` in
//! `app/tests/domains.rs` is what keeps that true for the next fold.
//!
//! The three snapshots are values, not borrows, for the same reason
//! `RestingPointer` is: a fold that held a live handle could retain it, and
//! then the narrowing would be a naming convention rather than a boundary.

use super::*;

/// Everything the semantic fold may read.
pub(in crate::app) struct SemanticView<'a> {
    /// The active document. The retained projection re-reads line runs out of
    /// this and nothing else.
    pub(super) document: &'a document::DocumentSession,
    /// The summoned-UI precedence ladder, which names the one active surface.
    pub(super) workspace_state: &'a workspace::WorkspaceState,
    /// The summoned card as CONTENT, composed once by the constructor because
    /// the composition needs the render pipeline's live figures. `None` when no
    /// card is open — and then no card work happened at all.
    pub(super) card: Option<crate::card::content::CardContent>,
    /// The which-key continuation rows when the panel is up.
    pub(super) whichkey: Option<Vec<(String, String)>>,
    /// The center notice's text.
    pub(super) notice: Option<String>,
    /// **WHAT A SIGHTED READER SEES INSTEAD OF THE BUFFER**, when the document
    /// layer is relocated into a comparison (History/Conflict/Credits) —
    /// resolved through the SAME dispatch the pixels use
    /// (`crate::comparison::prose_for`), so the accessibility tree cannot
    /// describe a different document than the screen shows. `None` covers
    /// three cases the retained projection treats alike: no overlay is up, the
    /// open kind shows no comparison at all, and the calm degrade where a
    /// comparison is requested but its subject cannot be resolved (the render
    /// side falls back to the buffer in exactly that case too, so this does
    /// the same) — then the document fold reads the buffer as it always has.
    pub(super) comparison_text: Option<String>,
}

impl SemanticView<'_> {
    pub(super) fn buffer(&self) -> Option<&Buffer> {
        self.document.buffer_opt()
    }

    pub(super) fn layer(&self) -> workspace::Layer {
        self.workspace_state.layer()
    }

    /// **IS THE DOCUMENT PRESENTED READ-ONLY?** The same family predicate the
    /// insertion wall and the caret layer read
    /// (`OverlayState::shows_read_only_prose`), derived from the comparison
    /// roster — so what an assistive technology is TOLD about the document and
    /// what the doors actually do cannot come apart.
    pub(super) fn document_is_read_only(&self) -> bool {
        self.workspace_state
            .overlay()
            .is_some_and(crate::overlay::OverlayState::shows_read_only_prose)
    }

    /// The substituted prose text, borrowed. `None` exactly when the document
    /// fold must read the real buffer instead — see the field's own doc.
    pub(super) fn comparison_text(&self) -> Option<&str> {
        self.comparison_text.as_deref()
    }
}

impl App {
    /// Read the whole live `App` once, and hand the fold the narrow value.
    ///
    /// The card and which-key snapshots are taken behind their own gates, so a
    /// frame with neither open pays two cheap predicates — the same two it paid
    /// when the folds asked for them mid-walk.
    pub(in crate::app) fn semantic_view(&self) -> SemanticView<'_> {
        SemanticView {
            document: &self.document,
            workspace_state: &self.workspace_state,
            card: self.card_content(),
            whichkey: self.whichkey_panel_rows(),
            notice: self.frame.notice().text().map(str::to_string),
            comparison_text: self.comparison_text_for_semantic(),
        }
    }

    /// **THE SAME RESOLUTION `ViewState::comparison_transcript` performs for
    /// the pixels, asked read-only for the accessibility fold.**
    ///
    /// `comparison_transcript` cannot be reused directly — it caches into
    /// `DocumentSession::history_preview`, which needs `&mut self`, and this
    /// fold is handed only `&App`. So this recomputes the same dispatch
    /// uncached: the accessibility fold runs only while an assistive
    /// technology is attached or a `--semantic-json` capture asks for it,
    /// never on an ordinary frame, so it does not compete with the render
    /// cache's reason for existing. Both call the ONE producer dispatch
    /// (`crate::comparison::prose_for`), so a live App and a headless
    /// `--semantic-json` capture cannot disagree about what the tree shows,
    /// the same guarantee `prose_for`'s own doc makes for the pixels.
    fn comparison_text_for_semantic(&self) -> Option<String> {
        let overlay = self.workspace_state.overlay()?;
        let request = overlay.comparison_request()?;
        let buffer = self.document.buffer_opt()?;
        let current = buffer.text();
        let (_, transcript, _counts) = crate::comparison::prose_for(
            overlay,
            &request,
            buffer.path(),
            buffer.is_unnamed_fresh(),
            &current,
        )?;
        Some(transcript)
    }
}
