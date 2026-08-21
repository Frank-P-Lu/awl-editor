//! What the renderer KNOWS about a summoned card, as opposed to how it draws
//! one (`hud.rs`).
//!
//! Two kinds of fact meet here and nowhere else. The DOCUMENT figures — word
//! count, frontmatter language, through-doc percent — are derived from the
//! user's own document by `crate::card::figures`, the pure owner the semantic
//! fold derives them through as well. The LIVE-only figures are whatever a running
//! `App` last pushed into this pipeline; the pipeline is their courier, not
//! their gatherer, so reading them back out is how a consumer with no pipeline
//! of its own learns which placeholders a frame will show.
//!
//! Composition, captions and phrasing belong to `crate::card::content`. Nothing
//! here decides what a card SAYS.

use super::*;

impl TextPipeline {
    /// The SHAPED document, reassembled from the shaped lines — the `&str` the
    /// pure figure owners read.
    ///
    /// cosmic-text stores each line's terminator separately, so the lines are
    /// exactly the pushed text split on `\n` and rejoining them returns those
    /// same bytes. O(doc) and allocating, so it is called only where the work
    /// already was: the sidecar's readout/HUD blocks and the composition of a
    /// summoned card — never on an ordinary frame.
    pub(in crate::render) fn doc_text(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.buffer.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(line.text());
        }
        out
    }

    /// THE text the document figures are derived from, and the caret's place in
    /// it — the user's own document, which is the shaped text only when nothing
    /// has been substituted for it.
    ///
    /// A fold drops the hidden lines from what is shaped and a History preview
    /// replaces it with a diff transcript; neither changes the document. So each
    /// records the real thing in [`crate::render::ViewState::doc_source`] on its
    /// way past, and this is the ONE seam that chooses between them — every
    /// figure below reads it, so none of them can be over a different text than
    /// its neighbour, and none can disagree with the semantic snapshot, which
    /// derives the same figures from the buffer.
    pub(in crate::render) fn figure_source(
        &self,
    ) -> (
        String,
        usize,
        usize,
        Option<((usize, usize), (usize, usize))>,
    ) {
        match &self.doc_source {
            Some(doc) => (
                doc.text.clone(),
                doc.cursor_line,
                doc.cursor_col,
                doc.selection,
            ),
            None => (
                self.doc_text(),
                self.cursor_line,
                self.cursor_col,
                self.selection,
            ),
        }
    }

    /// The three DOCUMENT figures this frame, through their one pure owner.
    pub(in crate::render) fn doc_figures(&self) -> crate::card::figures::DocFigures {
        let (text, cursor_line, cursor_col, _) = self.figure_source();
        crate::card::figures::DocFigures::of(&text, self.md_enabled, cursor_line, cursor_col)
    }

    /// The LIVE-only card figures as they were last pushed in, including the
    /// all-absent reading of a pipeline nobody has fed.
    pub fn card_live(&self) -> crate::card::CardLive {
        crate::card::CardLive {
            stats: self.hud.stats.clone(),
            streaks: self.streaks_view.clone(),
            saved: self.hud.saved,
            peek_rows: self.peek_rows.clone(),
            update_checked: self.hud.update_checked,
            pending_crash: self.hud.pending_crash,
        }
    }

    /// Everything a summoned card can show this frame.
    pub fn card_inputs(&self) -> crate::card::content::CardInputs {
        crate::card::content::CardInputs {
            hud_held: self.hud_showing(),
            peek_shown: self.peek_showing(),
            streaks_page: crate::streaks::card_view(),
            doc: self.doc_figures(),
            selection: self.selection_figures(),
            eol: self.eol,
            live: self.card_live(),
        }
    }

    /// The active selection's raw-buffer figures, over the SAME source chosen
    /// for the document figures above. This is evaluated only while the HUD is
    /// summoned, through `card_inputs`, never on an ordinary frame.
    pub(in crate::render) fn selection_figures(
        &self,
    ) -> Option<crate::card::figures::SelectionFigures> {
        let (text, _, _, selection) = self.figure_source();
        crate::card::figures::SelectionFigures::of(&text, selection)
    }

    /// The summoned card this frame, as CONTENT. The semantic tree composes the
    /// same value from the same owners, so an assistive technology hears exactly
    /// the card that is drawn rather than a second description of it.
    pub fn card_content(&self) -> Option<crate::card::content::CardContent> {
        crate::card::content::open_card(&self.card_inputs())
    }
}
