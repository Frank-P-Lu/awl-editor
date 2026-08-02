//! What the renderer KNOWS about a summoned card, as opposed to how it draws
//! one (`hud.rs`).
//!
//! Two kinds of fact meet here and nowhere else. The DOCUMENT figures — word
//! count, frontmatter language, through-doc percent — are derived from the
//! shaped text by `crate::card::figures`, the pure owner the semantic fold
//! derives them through as well. The LIVE-only figures are whatever a running
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

    /// The LIVE-only card figures as they were last pushed in, including the
    /// all-absent reading of a pipeline nobody has fed.
    pub fn card_live(&self) -> crate::card::CardLive {
        crate::card::CardLive {
            stats: self.hud_stats.clone(),
            streaks: self.streaks_view.clone(),
            saved: self.hud_saved,
            peek_rows: self.peek_rows.clone(),
            update_checked: self.hud_update_checked,
            pending_crash: self.hud_pending_crash,
        }
    }

    /// Everything a summoned card can show this frame.
    pub fn card_inputs(&self) -> crate::card::content::CardInputs {
        crate::card::content::CardInputs {
            hud_held: self.hud_showing(),
            peek_shown: self.peek_showing(),
            streaks_page: crate::streaks::card_view(),
            doc: crate::card::figures::DocFigures::of(
                &self.doc_text(),
                self.md_enabled,
                self.cursor_line,
                self.cursor_col,
            ),
            eol: self.eol,
            live: self.card_live(),
        }
    }

    /// The summoned card this frame, as CONTENT. The semantic tree composes the
    /// same value from the same owners, so an assistive technology hears exactly
    /// the card that is drawn rather than a second description of it.
    pub fn card_content(&self) -> Option<crate::card::content::CardContent> {
        crate::card::content::open_card(&self.card_inputs())
    }
}
