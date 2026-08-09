//! Cached spell and writing-nit underline prototypes.

use super::*;

impl TextPipeline {
    fn destination_ranges(&self) -> Vec<std::ops::Range<usize>> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        let doc_text: String = self
            .buffer
            .lines
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join("\n");
        crate::markdown::destination_ranges(&doc_text, &self.md_spans)
    }

    fn nit_hidden_by_bullet_glyph(&self, li: usize, end_col: usize) -> bool {
        self.md_enabled
            && self
                .buffer
                .lines
                .get(li)
                .and_then(|l| crate::markdown::list_item(l.text()))
                .is_some_and(|it| !it.ordered && end_col <= it.content)
    }

    fn ensure_squiggle_protos(&self) {
        let key = (self.row_geom.generation(), self.spell_gen);
        if self.squiggle_cache.version.get() == Some(key) {
            return;
        }
        let destination_ranges = self.destination_ranges();
        let mut line_starts: Vec<usize> = Vec::new();
        if !destination_ranges.is_empty() {
            let mut start = 0usize;
            for line in self.buffer.lines.iter() {
                line_starts.push(start);
                start += line.text().len() + 1; // +1 for the '\n'
            }
        }
        let lines: std::collections::BTreeSet<usize> =
            self.misspelled.iter().map(|sp| sp.line).collect();
        let rows_by_line = self.visual_rows_for_lines(&lines);
        let mut protos = Vec::with_capacity(self.misspelled.len());
        for sp in &self.misspelled {
            if let Some(&ls) = line_starts.get(sp.line) {
                let text = self.buffer.lines[sp.line].text();
                if crate::nits::span_in_prose_ranges(
                    text,
                    ls,
                    sp.start_col,
                    sp.end_col,
                    &destination_ranges,
                ) {
                    continue;
                }
            }
            // A misspelled span is a single word; cosmic-text wraps at spaces so
            // the word stays on ONE visual run. Find the run owning its start
            // column and keep that run's wrap-aware top + own x boundaries, so the
            // squiggle sits directly under the word's glyphs at any wrap/zoom.
            let Some(rows) = rows_by_line.get(&sp.line) else {
                continue; // unreachable: every requested line gets rows
            };
            let row = pick_row(rows, sp.start_col);
            let char_count = row.xs.len().saturating_sub(1);
            let s = sp.start_col.min(char_count);
            let e = sp.end_col.min(char_count);
            if e <= s {
                continue;
            }
            let xs_s = row.xs.get(s).copied().unwrap_or(0.0);
            let xs_e = row.xs.get(e).copied().unwrap_or(xs_s);
            if self.line_is_inline_image(sp.line)
                && xs_e - xs_s < Self::IMAGE_CONCEAL_UNDERLINE_MIN_ADVANCE.0
            {
                continue;
            }
            protos.push(UnderlineProto {
                line: sp.line,
                start_col: sp.start_col,
                end_col: sp.end_col,
                line_top: row.line_top,
                line_height: row.line_height,
                xs_s,
                xs_e,
            });
        }
        *self.squiggle_cache.protos.borrow_mut() = protos;
        self.squiggle_cache.version.set(Some(key));
    }

    fn word_at_caret(&self, line: usize, start_col: usize, end_col: usize) -> bool {
        line == self.cursor_line && self.cursor_col >= start_col && self.cursor_col <= end_col
    }

    /// Build the wavy-underline geometry for every misspelled span, in pixels,
    /// for the current scroll + zoom. Mirrors [`Self::selection_rects`]: it reads
    /// the line's real per-char x boundaries (advance-aware) so the squiggle's
    /// x-range matches the word's glyphs, and places the band just below the
    /// glyph cell.
    ///
    /// The scroll-independent geometry comes from the cached protos (see
    /// [`UnderlineCache`] — rebuilt only when the shaped text or the spell list
    /// changes), so the per-frame work is just adding the current `doc_top` /
    /// `text_left` with the IDENTICAL f32 ops the uncached builder used (bitwise-
    /// equal pixels) and culling the off-screen bands (which would rasterize
    /// nothing anyway) — O(misspellings) trivial arithmetic instead of
    /// O(misspellings × doc) run walks. REVEAL-ON-CURSOR: the ONE misspelling the
    /// caret is on/adjacent to is skipped ([`Self::word_at_caret`]) — cursor
    /// position folds in at READ time, not the cache key, so a pure cursor move
    /// keeps the proto cache warm (mirrors `rule_lines`/`bullet_marks`).
    pub(crate) fn spell_squiggles(&self) -> Vec<Squiggle> {
        if self.misspelled.is_empty() {
            return Vec::new();
        }
        self.ensure_squiggle_protos();
        let m = &self.metrics;
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let amp = m.px(SPELL_AMP);
        let period = m.px(SPELL_PERIOD);
        let thickness = m.px(SPELL_THICKNESS);
        let gap = m.px(theme::active().render_caps.spell_underline_gap);
        let band_h = amp * 2.0 + thickness + 2.0;
        let protos = self.squiggle_cache.protos.borrow();
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            if self.word_at_caret(p.line, p.start_col, p.end_col) {
                continue; // reveal-on-cursor: the word under active editing yields
            }
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would be clipped to nothing
            }
            let x = text_left + p.xs_s;
            let w = (p.xs_e - p.xs_s).max(1.0);
            let (band_y, row_caret_h) = self.row_band_for(p.line, p.line_height, line_top);
            let cell_bottom = band_y + row_caret_h;
            let y = cell_bottom + gap;
            if !self.band_admits(y, band_h) {
                continue; // DIFF-AS-PREVIEW: the row scrolled past the card edge
            }
            out.push(Squiggle {
                x,
                y,
                w,
                h: band_h,
                amp,
                period,
                thickness,
            });
        }
        out
    }

    /// Rebuild the cached nit-underline protos IF the shaped geometry changed since
    /// they were last built. The nit spans are a pure function of each line's TEXT
    /// ([`crate::nits::line_nits`]) and the row geometry of the shaped runs, both
    /// covered by the row-geometry GENERATION (every text change reshapes, every
    /// reshape bumps it; `reshape_count` rides along as the text-version half of the
    /// shared key). One text scan + ONE `layout_runs()` walk for ALL nit lines,
    /// amortised across every frame of the same shaped text — this was an O(doc
    /// chars) rescan + O(nit-lines × doc) run walks EVERY frame.
    ///
    /// CODE-BUFFER SCOPE (mirrors [`crate::spell::SpellChecker::misspellings_for`]'s
    /// scoping exactly): nits are a PROSE writing aid, not a code linter — a
    /// recognized code buffer (`self.syn_lang.is_some()`) restricts every nit to the
    /// lexer's own PROSE regions (`self.syn_spans`'s `Comment` + `Str` roles, the
    /// SAME prose scope the syntax wash uses), dropping the rest of a span that
    /// isn't FULLY inside one of those ranges wholesale — so alignment whitespace,
    /// trailing spaces after a semicolon, and identifier punctuation never nit
    /// (commented-OUT code — `SynKind::CommentCode` — is excluded too, same as
    /// spell). A non-code buffer (prose / markdown / the no-path scratch buffer,
    /// `syn_lang == None`) is untouched — every span from every line is eligible,
    /// byte-identical to before this scoping existed.
    fn ensure_nit_protos(&self) {
        let key = (self.row_geom.generation(), self.reshape_count);
        if self.nit_cache.version.get() == Some(key) {
            return;
        }
        let prose_ranges: Option<Vec<std::ops::Range<usize>>> = self.syn_lang.map(|_| {
            use crate::syntax::SynKind;
            let mut ranges: Vec<std::ops::Range<usize>> = self
                .syn_spans
                .iter()
                .filter(|(_, k)| matches!(k, SynKind::Comment | SynKind::Str))
                .map(|(r, _)| r.clone())
                .collect();
            ranges.sort_by_key(|r| r.start);
            ranges
        });
        let fm_end = crate::markdown::frontmatter_end(&self.md_spans);
        let table_ranges: Vec<std::ops::Range<usize>> = self
            .md_spans
            .iter()
            .filter(|(_, k)| k.is_table_markup())
            .map(|(r, _)| r.clone())
            .collect();
        let destination_ranges = self.destination_ranges();
        let mut per_line: Vec<(usize, Vec<(usize, usize)>)> = Vec::new();
        let mut line_start = 0usize;
        for li in 0..self.buffer.lines.len() {
            let text = self.buffer.lines[li].text();
            if fm_end.is_some_and(|end| line_start < end) {
                line_start += text.len() + 1;
                continue;
            }
            let line_end = line_start + text.len();
            let in_table = table_ranges
                .iter()
                .any(|r| r.start <= line_end && r.end > line_start);
            let mut spans = if in_table {
                crate::nits::line_nits_table_row(text)
            } else {
                crate::nits::line_nits(text)
            };
            if let Some(ranges) = &prose_ranges {
                spans.retain(|&(s, e)| {
                    crate::nits::span_in_prose_ranges(text, line_start, s, e, ranges)
                });
            }
            if !destination_ranges.is_empty() {
                spans.retain(|&(s, e)| {
                    !crate::nits::span_in_prose_ranges(text, line_start, s, e, &destination_ranges)
                });
            }
            if !spans.is_empty() {
                per_line.push((li, spans));
            }
            line_start += text.len() + 1; // +1 for the '\n'
        }
        let lines: std::collections::BTreeSet<usize> = per_line.iter().map(|(li, _)| *li).collect();
        let rows_by_line = self.visual_rows_for_lines(&lines);
        let mut protos = Vec::new();
        for (li, spans) in per_line {
            let Some(rows) = rows_by_line.get(&li) else {
                continue; // unreachable: every requested line gets rows
            };
            for (start_col, end_col) in spans {
                // Nit spans are single, space-tight runs; cosmic-text keeps each on
                // one visual run. Use the wrap-aware row owning the span's start.
                let row = pick_row(rows, start_col);
                let char_count = row.xs.len().saturating_sub(1);
                let s = start_col.min(char_count);
                let e = end_col.min(char_count);
                if e <= s {
                    continue;
                }
                let xs_s = row.xs.get(s).copied().unwrap_or(0.0);
                let xs_e = row.xs.get(e).copied().unwrap_or(xs_s);
                if self.line_is_inline_image(li)
                    && xs_e - xs_s < Self::IMAGE_CONCEAL_UNDERLINE_MIN_ADVANCE.0
                {
                    continue;
                }
                protos.push(UnderlineProto {
                    line: li,
                    start_col,
                    end_col,
                    line_top: row.line_top,
                    line_height: row.line_height,
                    xs_s,
                    xs_e,
                });
            }
        }
        *self.nit_cache.protos.borrow_mut() = protos;
        self.nit_cache.version.set(Some(key));
    }

    /// Build the STRAIGHT muted WRITING-NIT underline geometry for every nit span
    /// on every line, in pixels for the current scroll + zoom. MIRRORS
    /// [`Self::spell_squiggles`] — same advance-aware per-char x layout, same
    /// row-centred band, same "just below the glyph cell" placement, same cached
    /// scroll-independent protos (see [`UnderlineCache`]) — with two deliberate
    /// differences: the wave AMPLITUDE is ZERO (so the shared shader draws a FLAT
    /// line, not a squiggle) and the pipeline tints it the MUTED neutral ink (not
    /// the error red), so a nit reads as a calm "tidy this" hint, visually
    /// distinct from a spelling error. The spans come straight from the pure
    /// per-line [`crate::nits::line_nits`] (mechanical typos only — NOT grammar),
    /// read off the shaped buffer's own line text. Empty — so nothing is
    /// uploaded/drawn — when the highlighter is toggled off ([`crate::nits::nits_on`]).
    /// REVEAL-ON-CURSOR: the ENTIRE line the caret occupies is excluded — a line
    /// is judged only once you've moved off it (the active line is workspace, not
    /// manuscript; mirrors `rule_lines`/`bullet_marks`'s per-line reveal, but for
    /// EVERY nit kind, not just the markdown ornaments). Cursor position folds in
    /// at READ time, not the proto cache key, so a pure cursor move keeps the
    /// cache warm.
    pub(crate) fn nit_underlines(&self) -> Vec<Squiggle> {
        if !crate::nits::nits_on() {
            return Vec::new();
        }
        self.ensure_nit_protos();
        let m = &self.metrics;
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let thickness = m.px(NIT_THICKNESS);
        let band_h = thickness + 2.0;
        let protos = self.nit_cache.protos.borrow();
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            if p.line == self.cursor_line {
                continue; // reveal-on-cursor: judged only once you've moved off it
            }
            if self.nit_hidden_by_bullet_glyph(p.line, p.end_col) {
                continue; // the marker prefix is masked by the bullet glyph
            }
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would be clipped to nothing
            }
            let x = text_left + p.xs_s;
            let w = (p.xs_e - p.xs_s).max(m.px(DECOR_MIN_W));
            let (band_y, row_caret_h) = self.row_band_for(p.line, p.line_height, line_top);
            let cell_bottom = band_y + row_caret_h;
            let y = cell_bottom + m.px(NIT_UNDERLINE_GAP);
            if !self.band_admits(y, band_h) {
                continue; // DIFF-AS-PREVIEW: the row scrolled past the card edge
            }
            out.push(Squiggle {
                x,
                y,
                w,
                h: band_h,
                amp: 0.0,    // STRAIGHT — no wave (the shared shader flattens at amp 0)
                period: 1.0, // unused when amp == 0 (kept > 0 so the shader div is safe)
                thickness,
            });
        }
        out
    }
}
