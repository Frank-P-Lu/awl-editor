//! Retained spell-squiggle geometry lookup, kept in its own module so its
//! size rides separately from `underlines.rs`. `rects::squiggle_projection`
//! owns the pure bookkeeping this module materializes against real row
//! geometry.

use super::*;

impl TextPipeline {
    /// Whether the retained per-line [`rects::SquiggleProjection`] applies at
    /// all. Reuses [`rects::nit_fast_path_eligible`] verbatim rather than a
    /// second copy of the same envelope: the destination-link exclusion below
    /// can depend on a reference-style link definition ANYWHERE in the
    /// document (not just the line being drawn), a recognized code language
    /// scopes misspellings to lexer-owned prose ranges the same way it scopes
    /// nits, and a table/frontmatter block shifts document-wide context the
    /// same way for both callers — so a document ineligible for one is
    /// ineligible for the other, for the identical reasons.
    fn squiggle_fast_path_eligible(&self) -> bool {
        rects::nit_fast_path_eligible(&self.md_spans, self.syn_lang)
    }

    /// `self.misspelled`, grouped by logical line — one entry per line in
    /// document order, empty for a line with no misspelling. A
    /// [`crate::spell::Misspelling`] carries no index back to a slot beyond
    /// its own `line`, so this is the one O(misspellings + lines) pass every
    /// retained-squiggle caller starts from.
    fn misspelled_by_line(&self) -> Vec<Vec<(usize, usize)>> {
        let mut by_line = vec![Vec::new(); self.buffer.lines.len()];
        for sp in &self.misspelled {
            if let Some(slot) = by_line.get_mut(sp.line) {
                slot.push((sp.start_col, sp.end_col));
            }
        }
        by_line
    }

    /// The shared per-line proto builder every retained-squiggle path funnels
    /// through: `destination_ranges`/`line_starts` are computed ONCE for the
    /// whole call, and [`Self::visual_rows_for_lines`] is called ONCE
    /// (batched) for every line in `lines` that actually carries a span — the
    /// same one-walk discipline the old unretained `ensure_squiggle_protos`
    /// always followed, now amortised across only the lines that changed
    /// instead of the whole document every time. Returns an entry only for a
    /// line in `lines` whose spans (after the destination-link/image-conceal
    /// exclusions) leave at least one visible proto; a caller reading a line
    /// this omits treats it as `Vec::new()`.
    fn build_squiggle_lines(
        &self,
        spans: &[Vec<(usize, usize)>],
        lines: &std::collections::BTreeSet<usize>,
    ) -> std::collections::HashMap<usize, Vec<UnderlineProto>> {
        let destination_ranges = self.destination_ranges();
        let mut line_starts: Vec<usize> = Vec::new();
        if !destination_ranges.is_empty() {
            let mut start = 0usize;
            for line in self.buffer.lines.iter() {
                line_starts.push(start);
                start += line.text().len() + 1; // +1 for the '\n'
            }
        }
        let rows_by_line = self.visual_rows_for_lines(lines);
        let mut out = std::collections::HashMap::with_capacity(lines.len());
        for &li in lines {
            let Some(line_spans) = spans.get(li) else {
                continue;
            };
            let Some(rows) = rows_by_line.get(&li) else {
                continue; // unreachable: every requested line gets rows
            };
            let mut protos = Vec::with_capacity(line_spans.len());
            for &(start_col, end_col) in line_spans {
                if let Some(&ls) = line_starts.get(li) {
                    let text = self.buffer.lines[li].text();
                    if crate::nits::span_in_prose_ranges(
                        text,
                        ls,
                        start_col,
                        end_col,
                        &destination_ranges,
                    ) {
                        continue;
                    }
                }
                // A misspelled span is a single word; cosmic-text wraps at spaces so
                // the word stays on ONE visual run. Find the run owning its start
                // column and keep that run's wrap-aware top + own x boundaries, so the
                // squiggle sits directly under the word's glyphs at any wrap/zoom.
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
            if !protos.is_empty() {
                out.insert(li, protos);
            }
        }
        out
    }

    /// Rebuild the whole retained projection from scratch: a structural edit,
    /// a shape mismatch, a reshape that bypassed `set_text` (zoom/DPI/
    /// restyle), or first activation. Only lines that actually carry a span
    /// pay for a row lookup. Returns the number of lines rebuilt (the
    /// `squiggle_lines_rebuilt` witness).
    fn reseed_squiggle_projection(&self, spans: Vec<Vec<(usize, usize)>>, generation: u64) -> u64 {
        let lines: std::collections::BTreeSet<usize> = spans
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(li, _)| li)
            .collect();
        let rebuilt = lines.len() as u64;
        let built = self.build_squiggle_lines(&spans, &lines);
        let mut p = self.squiggle_projection.borrow_mut();
        p.reseed(spans, built, generation);
        p.mark_reconciled(self.spell_gen);
        rebuilt
    }

    /// Record a reshape's effect on the retained squiggle geometry CHEAPLY —
    /// once per RESHAPE, off THIS reshape's own exact changed-line band —
    /// called from [`Self::set_text`] right after
    /// [`Self::refresh_changed_row_geometry`], so `patched` reflects this
    /// same reshape. Deliberately does NOT look up a single row: see
    /// [`rects::SquiggleProjection`]'s own doc comment for why the actual
    /// materialization is deferred to the next [`Self::ensure_squiggle_protos`]
    /// read instead.
    pub(in crate::render) fn refresh_squiggle_projection(
        &self,
        change: (usize, usize, usize),
        patched: bool,
    ) {
        if !self.squiggle_fast_path_eligible() {
            self.squiggle_projection.borrow_mut().clear();
            return;
        }
        let spans_len = self.buffer.lines.len();
        self.squiggle_projection
            .borrow_mut()
            .note_reshape(spans_len, change, patched);
    }

    /// Materialize whatever [`rects::SquigglePending`] `set_text` left
    /// recorded — a splice, a full reseed, or nothing at all — the ONE row
    /// lookup this reshape (or run of reshapes since the last read) actually
    /// pays for. Returns the `squiggle_lines_rebuilt` witness.
    fn resolve_pending_squiggle_reshape(&self, pending: rects::SquigglePending) -> u64 {
        let generation = self.row_geom.generation();
        match pending {
            rects::SquigglePending::FullReseed => {
                let spans = self.misspelled_by_line();
                self.reseed_squiggle_projection(spans, generation)
            }
            rects::SquigglePending::Splice {
                prefix,
                old_end,
                new_end,
            } => {
                let spans = self.misspelled_by_line();
                let band: std::collections::BTreeSet<usize> = (prefix..new_end)
                    .filter(|&li| !spans[li].is_empty())
                    .collect();
                let rebuilt = band.len() as u64;
                let built = self.build_squiggle_lines(&spans, &band);
                let mut p = self.squiggle_projection.borrow_mut();
                p.splice(&spans, (prefix, old_end, new_end), built, generation);
                p.mark_reconciled(self.spell_gen);
                rebuilt
            }
            rects::SquigglePending::None => 0,
        }
    }

    /// Reconcile the retained projection against the CURRENT `self.misspelled`
    /// with NO reshape at all — a dictionary edit, a personal-dictionary
    /// addition, a spellcheck toggle. Called lazily from
    /// `ensure_squiggle_protos` only when the row-geometry generation is
    /// confirmed unchanged (so every line's pixels are still valid); re-diffs
    /// each line's own span list (cheap: comparing short `Vec`s, no row
    /// lookups) and rebuilds only the lines that actually differ, which a
    /// global dictionary event can scatter anywhere in the document. Returns
    /// the `squiggle_lines_rebuilt` witness.
    fn reconcile_squiggle_projection(&self) -> u64 {
        let spans = self.misspelled_by_line();
        let generation = self.row_geom.generation();
        let diff_lines = self.squiggle_projection.borrow().diff_lines(&spans);
        let Some(diff_lines) = diff_lines else {
            return self.reseed_squiggle_projection(spans, generation);
        };
        let lookup: std::collections::BTreeSet<usize> = diff_lines
            .iter()
            .copied()
            .filter(|&li| !spans[li].is_empty())
            .collect();
        let rebuilt = lookup.len() as u64;
        let built = self.build_squiggle_lines(&spans, &lookup);
        let mut p = self.squiggle_projection.borrow_mut();
        p.apply_diff(spans, &diff_lines, built);
        p.mark_reconciled(self.spell_gen);
        rebuilt
    }

    /// The pre-retention algorithm: rebuild every misspelling's pixel geometry
    /// from a single whole-document `visual_rows_for_lines` walk. The safe
    /// fallback whenever [`Self::squiggle_fast_path_eligible`] does not hold —
    /// exactly today's behavior for a code buffer or a document carrying a
    /// table, frontmatter, or a link/image anywhere.
    fn squiggle_protos_full_scan(&self) -> Vec<UnderlineProto> {
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
        protos
    }

    pub(super) fn ensure_squiggle_protos(&self) {
        let key = (self.row_geom.generation(), self.spell_gen);
        if self.squiggle_cache.version.get() == Some(key) {
            return;
        }
        let scan_at = self.text_sync_profile.then(crate::clock::Instant::now);
        let mut lines_rebuilt = 0u64;
        if self.squiggle_fast_path_eligible() {
            let generation = self.row_geom.generation();
            let pending = self.squiggle_projection.borrow_mut().take_pending();
            match pending {
                rects::SquigglePending::None => {
                    if self.squiggle_projection.borrow().is_valid_for(generation) {
                        // Geometry is confirmed stable since the projection
                        // was last built — only `self.misspelled` itself can
                        // have moved, with no reshape at all.
                        if !self
                            .squiggle_projection
                            .borrow()
                            .is_reconciled(self.spell_gen)
                        {
                            lines_rebuilt = self.reconcile_squiggle_projection();
                        }
                    } else {
                        // The row geometry moved WITHOUT going through
                        // `set_text`'s eager `note_reshape` at all
                        // (defensive: every live reshape does route through
                        // it) — every line's pixels are suspect.
                        let spans = self.misspelled_by_line();
                        lines_rebuilt = self.reseed_squiggle_projection(spans, generation);
                    }
                }
                pending => {
                    lines_rebuilt = self.resolve_pending_squiggle_reshape(pending);
                }
            }
            *self.squiggle_cache.protos.borrow_mut() =
                self.squiggle_projection.borrow().iter().copied().collect();
        } else {
            self.squiggle_projection.borrow_mut().clear();
            *self.squiggle_cache.protos.borrow_mut() = self.squiggle_protos_full_scan();
        }
        self.squiggle_cache.version.set(Some(key));
        if let Some(at) = scan_at {
            self.owner_scan
                .squiggle_scan_ms
                .set(at.elapsed().as_secs_f64() * 1000.0);
            self.owner_scan
                .squiggle_scan_misspellings
                .set(self.misspelled.len() as u64);
            self.owner_scan.squiggle_lines_rebuilt.set(lines_rebuilt);
        }
    }

    /// TEST ONLY: `(line, start_col, end_col, line_top, line_height, xs_s,
    /// xs_e)` for every proto in `protos`, sorted so two independently built
    /// lists compare equal regardless of build order — the plain-tuple shape
    /// keeps [`UnderlineProto`] itself unexported outside this module.
    #[cfg(test)]
    fn squiggle_proto_tuples(
        protos: &[UnderlineProto],
    ) -> Vec<(usize, usize, usize, u32, u32, u32, u32)> {
        let mut out: Vec<_> = protos
            .iter()
            .map(|p| {
                (
                    p.line,
                    p.start_col,
                    p.end_col,
                    p.line_top.to_bits(),
                    p.line_height.to_bits(),
                    p.xs_s.to_bits(),
                    p.xs_e.to_bits(),
                )
            })
            .collect();
        out.sort();
        out
    }

    /// TEST ONLY: the finished squiggle geometry [`Self::ensure_squiggle_protos`]
    /// actually published this call — whichever path filled it (the retained
    /// projection, or the full-scan fallback for an ineligible document). The
    /// oracle every retention law compares this against is
    /// [`Self::squiggle_full_scan_snapshot`], NOT this same value read twice —
    /// grading a retained cache against an independent recompute rather than
    /// against itself, the same discipline `NitProjection`/`HanEvidenceProjection`
    /// are graded under. Bit-exact (`f32::to_bits`), like every other float
    /// oracle in this crate's tests.
    #[cfg(test)]
    pub(in crate::render) fn squiggle_output_snapshot(
        &self,
    ) -> Vec<(usize, usize, usize, u32, u32, u32, u32)> {
        self.ensure_squiggle_protos();
        Self::squiggle_proto_tuples(&self.squiggle_cache.protos.borrow())
    }

    /// TEST ONLY: an INDEPENDENT full recompute of the exact same geometry,
    /// untouched by the retained [`rects::SquiggleProjection`] splice/
    /// reconcile machinery — the pre-retention algorithm, kept verbatim as the
    /// ineligible-document fallback. Any divergence from
    /// [`Self::squiggle_output_snapshot`] is a retention bug, not a shared-
    /// helper bug, because the two do not share a code path beyond the row-
    /// lookup primitives neither this round touched.
    #[cfg(test)]
    pub(in crate::render) fn squiggle_full_scan_snapshot(
        &self,
    ) -> Vec<(usize, usize, usize, u32, u32, u32, u32)> {
        Self::squiggle_proto_tuples(&self.squiggle_protos_full_scan())
    }
}
