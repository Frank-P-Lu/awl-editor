//! Cached spell and writing-nit underline prototypes.

use super::*;

impl TextPipeline {
    fn destination_ranges(&self) -> Vec<std::ops::Range<usize>> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        let join_at = self.text_sync_profile.then(crate::clock::Instant::now);
        let doc_text: String = self
            .buffer
            .lines
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join("\n");
        let out = crate::markdown::destination_ranges(&doc_text, &self.md_spans);
        if let Some(at) = join_at {
            self.owner_scan.destination_join_ms.set(
                self.owner_scan.destination_join_ms.get() + at.elapsed().as_secs_f64() * 1000.0,
            );
            self.owner_scan
                .destination_join_calls
                .set(self.owner_scan.destination_join_calls.get() + 1);
            self.owner_scan
                .destination_join_bytes
                .set(doc_text.len() as u64);
        }
        out
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

    /// True when `li` is a thematic-break (`---`/`***`/`___`) line whose source
    /// is presently CONCEALED under the fleuron — cached membership from the
    /// same set the rule ORNAMENT reads ([`Self::rule_lines`]'s underlying
    /// [`OrnamentCache::rule_lines`]), minus the lines that are REVEALED
    /// through the one owner the ornament's own draw gate uses
    /// ([`Self::line_is_revealed`]). Both halves matter: membership alone
    /// suppressed the nit on a SELECTION-revealed rule line, which draws its
    /// raw `---` source and so has glyphs to tick.
    ///
    /// `selection_touch` comes from the caller so the rope walk behind it
    /// happens once per frame rather than once per proto. The on-screen cull is
    /// still the caller's ([`Self::proto_visible`], after this check).
    fn nit_hidden_by_rule_conceal(
        &self,
        li: usize,
        selection_touch: Option<&std::ops::Range<usize>>,
    ) -> bool {
        if !self.md_enabled || self.md_spans.is_empty() {
            return false;
        }
        self.ensure_ornament_lists();
        self.ornament_cache.rule_lines.borrow().contains(&li)
            && !self.line_is_revealed(li, selection_touch)
    }

    /// True when `li` sits inside a table block ([`ConcealKind::Table`]) that is
    /// CURRENTLY x-raying its source to near-zero width. Unlike
    /// [`Self::nit_hidden_by_rule_conceal`], this is NOT a pure function of the
    /// reshape: a table's structural membership (which lines belong to which
    /// table, `ornament_cache.table_blocks`) rides the reshape cache exactly like
    /// `rule_lines` does, but WHETHER that membership is presently concealed does
    /// not — `ConcealKind::Table` only conceals while `wysiwyg_on()` holds
    /// ([`crate::markdown::wysiwyg_reveals`] always answers `false` for a table in
    /// place; the toggle is the only gate), and that process-global can flip
    /// WITHOUT a reshape ([`super::WashCache`]'s own doc comment names the same
    /// hazard for its inline-code pill bucket). Folding "concealed" into a cached
    /// SET here — the shape [`Self::nit_hidden_by_rule_conceal`] uses — would
    /// freeze the decision at whatever `wysiwyg_on()` read at the LAST reshape, so
    /// only the structural line-membership half rides the cache; the toggle
    /// itself is read fresh on every call, matching every other table-conceal
    /// consumer in this crate (`ensure_wash_protos`, `prepare_table_grid`,
    /// `compute_table_layout`, `try_table_pan`).
    fn nit_hidden_by_table_conceal(&self, li: usize) -> bool {
        if !self.md_enabled || self.md_spans.is_empty() || !crate::markdown::wysiwyg_on() {
            return false;
        }
        self.ensure_ornament_lists();
        let line_byte = self.line_doc_byte_start(li);
        self.ornament_cache
            .table_blocks
            .borrow()
            .iter()
            .any(|(_, r)| r.start <= line_byte && line_byte < r.end)
    }

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
    /// pay for a row lookup.
    fn reseed_squiggle_projection(&self, spans: Vec<Vec<(usize, usize)>>, generation: u64) {
        let lines: std::collections::BTreeSet<usize> = spans
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(li, _)| li)
            .collect();
        let built = self.build_squiggle_lines(&spans, &lines);
        self.squiggle_projection
            .borrow_mut()
            .reseed(spans, &built, generation, self.spell_gen);
    }

    /// Refresh the retained per-line squiggle geometry HERE, once per
    /// RESHAPE, off THIS reshape's own exact changed-line band — called from
    /// [`Self::set_text`] right after [`Self::refresh_changed_row_geometry`],
    /// so `patched` reflects this same reshape and `self.misspelled` (mirrored
    /// earlier in the same `set_view` call by `sync_view_fields`) is already
    /// current for the post-edit text. Falls back to a full reseed whenever
    /// the document is outside the fast-path envelope, the row-geometry patch
    /// was rejected, or the retained shape no longer matches — the same
    /// defensive fallback [`rects::NitProjection::refresh`] takes.
    pub(in crate::render) fn refresh_squiggle_projection(
        &self,
        change: (usize, usize, usize),
        patched: bool,
    ) {
        if !self.squiggle_fast_path_eligible() {
            self.squiggle_projection.borrow_mut().clear();
            if self.text_sync_profile {
                self.owner_scan.squiggle_lines_rebuilt.set(0);
            }
            return;
        }
        let spans = self.misspelled_by_line();
        let generation = self.row_geom.generation();
        let can_splice = self
            .squiggle_projection
            .borrow()
            .can_splice(spans.len(), change, patched);
        if !can_splice {
            // Full reseed: every line carrying a span pays for a lookup.
            let rebuilt = spans.iter().filter(|s| !s.is_empty()).count() as u64;
            self.reseed_squiggle_projection(spans, generation);
            if self.text_sync_profile {
                self.owner_scan.squiggle_lines_rebuilt.set(rebuilt);
            }
            return;
        }
        let (prefix, _, new_end) = change;
        let band: std::collections::BTreeSet<usize> = (prefix..new_end)
            .filter(|&li| !spans[li].is_empty())
            .collect();
        if self.text_sync_profile {
            self.owner_scan
                .squiggle_lines_rebuilt
                .set(band.len() as u64);
        }
        let built = self.build_squiggle_lines(&spans, &band);
        self.squiggle_projection
            .borrow_mut()
            .splice(&spans, change, &built, generation, self.spell_gen);
    }

    /// Reconcile the retained projection against the CURRENT `self.misspelled`
    /// with NO reshape at all — a dictionary edit, a personal-dictionary
    /// addition, a spellcheck toggle. Called lazily from
    /// `ensure_squiggle_protos` only when the row-geometry generation is
    /// confirmed unchanged (so every line's pixels are still valid); re-diffs
    /// each line's own span list (cheap: comparing short `Vec`s, no row
    /// lookups) and rebuilds only the lines that actually differ, which a
    /// global dictionary event can scatter anywhere in the document.
    fn reconcile_squiggle_projection(&self) {
        let spans = self.misspelled_by_line();
        let generation = self.row_geom.generation();
        let diff_lines = self.squiggle_projection.borrow().diff_lines(&spans);
        let Some(diff_lines) = diff_lines else {
            self.reseed_squiggle_projection(spans, generation);
            return;
        };
        let lookup: std::collections::BTreeSet<usize> = diff_lines
            .iter()
            .copied()
            .filter(|&li| !spans[li].is_empty())
            .collect();
        let built = self.build_squiggle_lines(&spans, &lookup);
        self.squiggle_projection
            .borrow_mut()
            .apply_diff(spans, &diff_lines, &built, self.spell_gen);
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

    fn ensure_squiggle_protos(&self) {
        let key = (self.row_geom.generation(), self.spell_gen);
        if self.squiggle_cache.version.get() == Some(key) {
            return;
        }
        let scan_at = self.text_sync_profile.then(crate::clock::Instant::now);
        if self.squiggle_fast_path_eligible() {
            let generation = self.row_geom.generation();
            let up_to_date = {
                let p = self.squiggle_projection.borrow();
                p.is_valid_for(generation) && p.is_reconciled(self.spell_gen)
            };
            if !up_to_date {
                if self.squiggle_projection.borrow().is_valid_for(generation) {
                    // Geometry is confirmed stable since the projection was
                    // last built (either just now, by `set_text`'s eager
                    // refresh this same reshape, or unchanged since an
                    // earlier one) — only `self.misspelled` itself can have
                    // moved, with no reshape at all.
                    self.reconcile_squiggle_projection();
                } else {
                    // The row geometry moved WITHOUT going through
                    // `set_text`'s eager refresh (zoom/DPI/restyle) — every
                    // line's pixels are suspect.
                    let spans = self.misspelled_by_line();
                    self.reseed_squiggle_projection(spans, generation);
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
        }
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
    /// `syn_lang == None`) is untouched — every span from every line is
    /// eligible.
    /// Whether the retained [`rects::NitProjection`] can answer this call
    /// outright: the document must qualify today (fresh `md_spans`/`syn_lang`
    /// read — nothing stale) AND the projection's own cache must have been
    /// built under that same eligibility AND cover exactly this many lines.
    /// The third check is the defensive one: it costs nothing and turns any
    /// unforeseen desync into a full recompute rather than a wrong answer.
    fn nit_projection_current(&self) -> bool {
        rects::nit_fast_path_eligible(&self.md_spans, self.syn_lang)
            && self.nit_projection.eligible()
            && self.nit_projection.line_count() == self.buffer.lines.len()
    }

    fn ensure_nit_protos(&self) {
        let key = (self.row_geom.generation(), self.reshape_count);
        if self.nit_cache.version.get() == Some(key) {
            return;
        }
        let per_line: Vec<(usize, Vec<(usize, usize)>)> = if self.nit_projection_current() {
            // FAST PATH: every line's raw span list is already retained
            // (refreshed in `set_text`, off that reshape's own changed-line
            // band); no text is re-tokenized here at all. `owner_scan`'s
            // nit-scan pair is left exactly as `set_text` wrote it this frame.
            (0..self.buffer.lines.len())
                .filter_map(|li| {
                    let spans = self.nit_projection.spans_for(li);
                    (!spans.is_empty()).then(|| (li, spans.to_vec()))
                })
                .collect()
        } else {
            // FULL PATH: unchanged from before the retained cache existed —
            // every code buffer, every table, every frontmatter block, every
            // document with a link or image takes this every time.
            let scan_at = self.text_sync_profile.then(crate::clock::Instant::now);
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
                        !crate::nits::span_in_prose_ranges(
                            text,
                            line_start,
                            s,
                            e,
                            &destination_ranges,
                        )
                    });
                }
                if !spans.is_empty() {
                    per_line.push((li, spans));
                }
                line_start += text.len() + 1; // +1 for the '\n'
            }
            if let Some(at) = scan_at {
                self.owner_scan
                    .nit_scan_ms
                    .set(at.elapsed().as_secs_f64() * 1000.0);
                self.owner_scan
                    .nit_scan_lines
                    .set(self.buffer.lines.len() as u64);
            }
            per_line
        };
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
    /// cache warm. RULE-LINE CONCEAL: a line that conceals to the thematic-break
    /// ornament ([`Self::nit_hidden_by_rule_conceal`]) is excluded too, caret line
    /// excepted (that case is already handled by the check above) — reveal-on-
    /// cursor keeps the nit visible once the raw `---` text itself shows, the same
    /// membership the ornament draws from. TABLE CONCEAL: a line inside a table
    /// block that is CURRENTLY x-raying its source to near-zero width
    /// ([`Self::nit_hidden_by_table_conceal`]) is excluded the same way — but
    /// unlike the rule case, whether it applies is read fresh from `wysiwyg_on()`
    /// on every call rather than folded into a cached set, since that toggle can
    /// flip without a reshape.
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
        let selection_touch = self.selection_touch();
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            if p.line == self.cursor_line {
                continue; // reveal-on-cursor: judged only once you've moved off it
            }
            if self.nit_hidden_by_bullet_glyph(p.line, p.end_col) {
                continue; // the marker prefix is masked by the bullet glyph
            }
            if self.nit_hidden_by_rule_conceal(p.line, selection_touch.as_ref()) {
                continue; // the whole line conceals to the rule ornament — no source glyphs to tick
            }
            if self.nit_hidden_by_table_conceal(p.line) {
                continue; // WYSIWYG x-rays the row to near-zero width — no glyphs to tick
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
    /// CLAUDE.md's own standing note that 629/630 both checked against an
    /// independent recompute rather than the thing being cached. Bit-exact
    /// (`f32::to_bits`), like every other float oracle in this crate's tests.
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
