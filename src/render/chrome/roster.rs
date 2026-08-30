//! ROSTER WIDTHS — "how wide is this whole list", asked of the real shaper.
//!
//! A card that sizes itself from its content has to measure the content, and
//! measuring only what is on screen makes the card a function of the scroll
//! position. Every such question is asked here, over the whole roster, against
//! the same shaper the draw runs — and memoized per [`RosterSlot`], because
//! shaping is linear in the roster while its answer changes only when the rows,
//! the face or the metrics do.

use super::*;

/// The questions [`TextPipeline::measure_panel_roster_px`] answers in one frame,
/// each with its own memo slot so they never evict one another. Named rather
/// than indexed: a fourth roster measurement gets a name and a slot here, not a
/// silent share of somebody else's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum RosterSlot {
    /// A content-hugging card's CANDIDATE rows, unelided.
    Candidates = 0,
    /// The same card's SECONDARY column — key chords, times, git tags.
    Secondary = 1,
    /// The contextual spell popup's suggestions.
    Spell = 2,
}

pub(in crate::render) const ROSTER_SLOTS: usize = 3;

impl TextPipeline {
    fn hug_roster_key(&self, roster: &std::sync::Arc<crate::overlay::HugRoster>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (std::sync::Arc::as_ptr(roster) as usize).hash(&mut h);
        self.metrics.font_size.to_bits().hash(&mut h);
        self.metrics.line_height.to_bits().hash(&mut h);
        (theme::active().font.as_ptr() as usize).hash(&mut h);
        crate::render::code_ligatures_on().hash(&mut h);
        h.finish()
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_hug_measure_count(&self) -> u64 {
        self.overlay_hug_measure_count
    }
    /// THE ONE ROSTER-WIDTH MEASUREMENT: the widest shaped line of `text` at
    /// `metrics`, with no width bound so nothing wraps or elides. Every
    /// "how wide is this whole list" question — the contextual spell popup's
    /// suggestions, a content-hugging card's candidates and its secondary column
    /// — asks it here, against the real shaper rather than a character estimate.
    ///
    /// MEMOIZED per `slot`, because the shaping is linear in the roster while its
    /// answer depends on nothing a scroll or a selection touches. The key carries
    /// exactly what the shaping pass below reads: the text, the metrics, and the
    /// panel face ([`crate::render::panel_attrs`] — the world's display family
    /// plus the ligature toggle).
    ///
    /// It borrows the shared panel buffer as scratch; the frame's own
    /// `overlay_shape_text` re-shapes it before anything is drawn.
    pub(super) fn measure_panel_roster_px(
        &mut self,
        slot: RosterSlot,
        text: &str,
        metrics: GlyphMetrics,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let key = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            text.hash(&mut h);
            metrics.font_size.to_bits().hash(&mut h);
            metrics.line_height.to_bits().hash(&mut h);
            (theme::active().font.as_ptr() as usize).hash(&mut h);
            crate::render::code_ligatures_on().hash(&mut h);
            h.finish()
        };
        if let Some((cached, w)) = self.roster_memo[slot as usize]
            && cached == key
        {
            return w;
        }
        self.panel_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.panel_buffer
            .set_size(&mut self.font_system, None, None);
        let ink = theme::base_content().to_glyphon();
        self.panel_buffer.set_text(
            &mut self.font_system,
            text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut max_w = 0.0_f32;
        for run in self.panel_buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
        }
        self.roster_memo[slot as usize] = Some((key, max_w));
        max_w
    }

    /// The widest SECONDARY cell in a roster, shaped exactly as
    /// [`Self::shape_overlay_right`] shapes the column it reserves room for —
    /// the monospace chord face with the same symbol-family split. Measuring it
    /// through the panel face instead under-read a chord by a few pixels, and a
    /// reserved extent that is narrower than the ink it reserves for is not a
    /// reservation.
    pub(super) fn measure_bind_roster_px(&mut self, labels: &[String]) -> f32 {
        let text = labels.join("\n");
        if text.is_empty() {
            return 0.0;
        }
        let m = self.metrics;
        let metrics = GlyphMetrics::new(
            m.font_size
                * crate::render::effective_overlay_scale()
                * crate::markdown::type_scale::LABEL,
            self.overlay_lh(),
        );
        let key = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            text.hash(&mut h);
            metrics.font_size.to_bits().hash(&mut h);
            (theme::active().font.as_ptr() as usize).hash(&mut h);
            h.finish()
        };
        if let Some((cached, w)) = self.roster_memo[RosterSlot::Secondary as usize]
            && cached == key
        {
            return w;
        }
        let ink = theme::muted().to_glyphon();
        let mono = |c| Attrs::new().family(Family::Monospace).color(c);
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let mut spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        for label in labels {
            push_symbol_split(&mut spans, label, || mono(ink), || sym(ink));
            spans.push(("\n", mono(ink)));
        }
        self.panel_bind_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.panel_bind_buffer
            .set_size(&mut self.font_system, None, None);
        self.panel_bind_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        self.panel_bind_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.panel_bind_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut max_w = 0.0_f32;
        for run in self.panel_bind_buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
        }
        self.roster_memo[RosterSlot::Secondary as usize] = Some((key, max_w));
        max_w
    }

    pub(in crate::render) fn measure_spell_content_w(&mut self) -> f32 {
        let ui_metrics = self.overlay_metrics();
        let text = self.overlay_items.join("\n");
        self.measure_panel_roster_px(RosterSlot::Spell, &text, ui_metrics)
    }

    /// The widest UNELIDED candidate in the whole roster, in shaped pixels.
    ///
    /// COST GUARD: shaping is linear in the roster, and a file picker can hold
    /// thousands of rows. A roster whose widest row cannot possibly hug — its
    /// character estimate is already twice the card's own text column — is
    /// answered by that column directly, because every such card lands on the cap
    /// regardless. The 2x margin is what keeps a MEAN-glyph-width estimate from
    /// ever short-circuiting a roster that would genuinely have hugged.
    pub(in crate::render) fn measure_roster_primary_px(&mut self, geom: &OverlayGeom) -> f32 {
        if let Some(roster) = self.overlay_hug_roster.clone() {
            return self.measure_roster_primary_strings_px(&roster.primary, geom);
        }
        let items = self.overlay_items.clone();
        self.measure_roster_primary_strings_px(&items, geom)
    }

    fn measure_roster_primary_strings_px(&mut self, items: &[String], geom: &OverlayGeom) -> f32 {
        let Some(widest_chars) = items.iter().map(|s| s.chars().count()).max() else {
            return 0.0;
        };
        if widest_chars as f32 * self.overlay_char_width() >= 2.0 * geom.text_w {
            return geom.text_w;
        }
        let text = items.join("\n");
        let metrics = self.overlay_metrics();
        self.measure_panel_roster_px(RosterSlot::Candidates, &text, metrics)
    }

    /// The widest secondary cell — key chord, time, git tag — in the whole
    /// roster, shaped at the right column's OWN recessive metrics.
    pub(in crate::render) fn measure_roster_secondary_px(&mut self) -> f32 {
        if let Some(roster) = self.overlay_hug_roster.clone() {
            return self.measure_bind_roster_px(&roster.secondary);
        }
        let labels = self.overlay_right_labels().to_vec();
        self.measure_bind_roster_px(&labels)
    }

    /// A right-anchored card's content-hug width. A flat picker measures its
    /// whole candidate roster; a faceted picker measures its summon-time,
    /// unlensed and unfiltered display corpus. Lens and query are projections,
    /// not content, so they cannot translate the card's pinned-left chrome.
    /// Both paths use the memoized joined-shaper pass below.
    pub(in crate::render) fn measure_overlay_content_w(&mut self) -> f32 {
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let geom = self.overlay_geometry(self.window_w as u32);
        self.overlay_remetric();
        let hug_roster = self.overlay_hug_roster.clone();
        let hug_key = hug_roster.as_ref().map(|roster| self.hug_roster_key(roster));
        if let Some(key) = hug_key
            && let Some((cached, width)) = self.overlay_hug_content_memo
            && cached == key
        {
            return width;
        }
        // A pure WIDTH measurement: `selected_ink: None` means no row can flip at
        // all, so this pass wants NO visual selection — and must not touch the
        // band's chase state (measuring may never advance an animation).
        let vis = VisualSelection::default();
        let plan = self.overlay_row_plan(&geom);
        // Flat pickers need their current card chrome shaped before the roster
        // pass reuses this scratch buffer. A faceted picker instead carries that
        // chrome in its stable summon corpus, so the active query cannot affect
        // this measurement.
        let static_corpus = hug_roster.is_some();
        if !static_corpus {
            let _ = self.overlay_shape_text(
                &geom,
                &plan,
                OverlaySpanInks {
                    ink,
                    muted,
                    selected: None,
                },
                &vis,
                false,
            );
        }
        let has_right = if static_corpus {
            !self.overlay_hug_roster.as_ref().unwrap().secondary.is_empty()
        } else {
            !self.overlay_right_labels().is_empty()
        };
        // A flat card's live chrome remains part of its content question. The
        // faceted corpus already contains its static title/strip/empty labels.
        let mut left = 0.0_f32;
        if !static_corpus {
            for run in self.panel_buffer.layout_runs() {
                left = left.max(run.line_w);
            }
        }
        let primary = self.measure_roster_primary_px(&geom) + self.overlay_char_width();
        let secondary = if has_right {
            self.measure_roster_secondary_px()
        } else {
            0.0
        };
        let gap = if secondary > 0.0 {
            rowlayout::GAP_CHARS as f32 * self.overlay_char_width()
        } else {
            0.0
        };
        let content_text = left.max(primary + gap + secondary);
        let reserve_rows = hug_roster
            .as_ref()
            .map(|roster| roster.candidate_rows)
            .unwrap_or_else(|| plan.rows().len());
        let width = content_text
            + 2.0 * self.overlay_text_hpad()
            + self.diagonal_side_reserve_px(reserve_rows);
        if let Some(key) = hug_key {
            self.overlay_hug_content_memo = Some((key, width));
            #[cfg(test)]
            {
                self.overlay_hug_measure_count += 1;
            }
        }
        width
    }
}
