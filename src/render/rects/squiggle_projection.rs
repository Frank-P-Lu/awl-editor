//! Retained per-line spell-squiggle geometry bookkeeping — split out of
//! `rects.rs` (item 633) to keep that file's own frozen size ratchet intact.
//! [`TextPipeline`]'s squiggle methods (in `rects::underlines`) own the actual
//! row-geometry lookup and hand finished [`UnderlineProto`]s in here; this
//! module is pure bookkeeping over that data.

use super::UnderlineProto;

/// A pending, not-yet-materialized change to [`SquiggleProjection`]'s
/// geometry, recorded CHEAPLY (integer bookkeeping only, no row lookups) by
/// [`SquiggleProjection::note_reshape`] once per reshape and consumed exactly
/// once by [`SquiggleProjection::take_pending`] at the next actual read. This
/// is what keeps a run of several reshapes with no read in between — a zoom
/// burst, several `set_view` calls coalesced before one drawn frame — down to
/// exactly ONE row-geometry pass at the eventual read rather than one per
/// reshape: the first reshape without a pending, splice-eligible band records
/// `Splice`; any SECOND reshape landing before that is resolved degrades
/// straight to `FullReseed` rather than attempting to compose two bands'
/// coordinate systems against each other (safe and simple, at the cost of an
/// occasional wider rebuild on an already-rare axis).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::render) enum SquigglePending {
    #[default]
    None,
    Splice {
        prefix: usize,
        old_end: usize,
        new_end: usize,
    },
    FullReseed,
}

/// Retained per-line SPELL-SQUIGGLE geometry (the document-wide-typing-work
/// investigation's third named owner, alongside [`NitProjection`] and
/// [`HanEvidenceProjection`] above, and the biggest: `ensure_squiggle_protos`
/// used to look up the shaped row and rebuild the pixel geometry for EVERY
/// misspelling in the document on every reshape, even though a misspelling's
/// PIXELS are a pure function of that one line's own shaped row plus its own
/// span, and a [`crate::spell::Misspelling`] never crosses a `\n` (a word
/// can't contain one). This keeps each line's finished [`UnderlineProto`]s and
/// reuses them for every line OUTSIDE the exact band a reshape touched,
/// splicing in fresh geometry only for the band `set_text`'s own `TextChange`
/// reports — the identical shape [`NitProjection::refresh`] uses for its own
/// per-line data.
///
/// UNLIKE the two siblings above, a successful splice ALSO requires the row
/// geometry PATCH to have succeeded (`RowGeom::patch_lines_if_stable`): a
/// rejected patch can shift every row below the edited band, so only a
/// PATCHED reshape can trust an out-of-band line's cached pixels — an
/// invalidated (fully rebuilt) `RowGeom` forces a full reseed here even
/// though the misspelling SPANS themselves may not have moved at all. AND
/// UNLIKE the two siblings, the row lookup itself is genuinely expensive (it
/// is the whole point of this cache), so `set_text`'s own eager call
/// ([`Self::note_reshape`]) does NOT perform it — only [`SquigglePending`]
/// bookkeeping. Materializing it eagerly on every reshape regardless of
/// whether anything ever reads the result before the NEXT reshape measurably
/// regressed a zoom burst (five `set_view` calls, one eventual frame): each
/// call was paying for a full per-line rebuild that the very next call would
/// immediately discard unread.
///
/// This struct is pure bookkeeping: it knows nothing about rows, buffers, or
/// destination-link exclusion. [`TextPipeline`]'s own squiggle methods (in
/// `rects::underlines`) own the actual per-line geometry lookup (they alone
/// hold `&self`) and hand the finished [`UnderlineProto`]s in; this only
/// decides WHICH lines need that lookup and splices the results in place —
/// mirroring [`crate::render::rowgeom::patch`]'s own split between "decide the
/// band" and "materialize it".
#[derive(Default)]
pub(in crate::render) struct SquiggleProjection {
    /// Whether `spans`/`slots` hold real per-line data at all (false before
    /// the first successful build, and whenever the document falls out of
    /// [`nit_fast_path_eligible`] — the exact same envelope `NitProjection`
    /// requires, since the destination-link exclusion this cache also applies
    /// can depend on a reference-style link definition ANYWHERE in the
    /// document, not just the line being drawn).
    valid: bool,
    /// The [`rowgeom::RowGeom`] generation this projection's PIXELS were last
    /// built against. A reshape that bypasses `set_text` entirely — a
    /// zoom/DPI/restyle, which invalidate `RowGeom` directly — leaves this
    /// stale; the mismatch against the CURRENT generation is what forces a
    /// full reseed at the next read, mirroring the defensive fallback the two
    /// siblings above take on any state mismatch.
    generation: u64,
    /// The `spell_gen` value `spans` was last made to agree with
    /// `self.misspelled` under. `generation` alone cannot see a misspelling
    /// change with NO reshape at all (a dictionary edit, a spellcheck
    /// toggle) — comparing this catches exactly that axis.
    reconciled_spell_gen: u64,
    /// Per logical line, in document order: the exact `(start_col, end_col)`
    /// misspelling spans this line's cached protos were built from — what
    /// [`Self::diff_lines`] compares the CURRENT grouping against.
    spans: Vec<Vec<(usize, usize)>>,
    /// Per logical line, parallel to `spans`: the finished proto geometry.
    slots: Vec<Vec<UnderlineProto>>,
    /// Recorded by [`Self::note_reshape`], consumed by [`Self::take_pending`].
    pending: SquigglePending,
}

impl SquiggleProjection {
    pub(in crate::render) fn new() -> Self {
        Self::default()
    }

    pub(super) fn is_valid_for(&self, generation: u64) -> bool {
        self.valid && self.generation == generation
    }

    pub(super) fn is_reconciled(&self, spell_gen: u64) -> bool {
        self.valid && self.reconciled_spell_gen == spell_gen
    }

    /// Every cached proto, in document order — the flattened view
    /// `ensure_squiggle_protos` publishes to [`UnderlineCache`].
    pub(super) fn iter(&self) -> impl Iterator<Item = &UnderlineProto> {
        self.slots.iter().flatten()
    }

    /// Drop every retained line: the document fell out of the fast-path
    /// envelope (a link/image/table/frontmatter appeared, or a code language
    /// was recognized). The very next read takes the full-scan path and finds
    /// this invalid; coming BACK into eligibility reseeds from scratch rather
    /// than reconstructing history it never retained.
    pub(super) fn clear(&mut self) {
        self.valid = false;
        self.spans.clear();
        self.slots.clear();
        self.pending = SquigglePending::None;
    }

    /// Whether a splice against `change`/`patched` can retain every line
    /// outside the band. `spans_len` is the CURRENT document's line count.
    fn can_splice(&self, spans_len: usize, change: (usize, usize, usize), patched: bool) -> bool {
        let (prefix, old_end, new_end) = change;
        prefix <= old_end
            && new_end <= spans_len
            && patched
            && self.valid
            && self.spans.len() == old_end + (spans_len - new_end)
            && self.slots.len() == old_end + (spans_len - new_end)
    }

    /// Record a reshape CHEAPLY — no row lookups, just the bookkeeping that
    /// decides whether the eventual read can splice or must fully reseed.
    /// Called once per reshape from `set_text`, mirroring the two siblings'
    /// timing discipline exactly; unlike them, this never touches `spans`/
    /// `slots` — see the struct's own doc comment for why.
    pub(super) fn note_reshape(
        &mut self,
        spans_len: usize,
        change: (usize, usize, usize),
        patched: bool,
    ) {
        if self.pending == SquigglePending::FullReseed {
            return; // already the worst case for this unread stretch
        }
        let can_splice = self.pending == SquigglePending::None
            && self.can_splice(spans_len, change, patched);
        self.pending = if can_splice {
            let (prefix, old_end, new_end) = change;
            SquigglePending::Splice {
                prefix,
                old_end,
                new_end,
            }
        } else {
            SquigglePending::FullReseed
        };
    }

    /// Consume whatever is pending, resetting it to `None` — the caller is
    /// about to materialize it (or has just confirmed there is nothing to
    /// do), so a later read in the same frame must not repeat the work.
    pub(super) fn take_pending(&mut self) -> SquigglePending {
        std::mem::take(&mut self.pending)
    }

    /// Replace the `[prefix, old_end)` band with freshly built lines for
    /// `[prefix, new_end)`, keeping everything outside it untouched. `spans`
    /// is the CURRENT full per-line grouping (used only to read the
    /// replacement band's own spans); `built` holds the finished protos for
    /// every line in that band that carried at least one span.
    pub(super) fn splice(
        &mut self,
        spans: &[Vec<(usize, usize)>],
        change: (usize, usize, usize),
        mut built: std::collections::HashMap<usize, Vec<UnderlineProto>>,
        generation: u64,
    ) {
        let (prefix, old_end, new_end) = change;
        let replacement_spans: Vec<Vec<(usize, usize)>> = spans[prefix..new_end].to_vec();
        let replacement_slots: Vec<Vec<UnderlineProto>> = (prefix..new_end)
            .map(|li| built.remove(&li).unwrap_or_default())
            .collect();
        self.spans.splice(prefix..old_end, replacement_spans);
        self.slots.splice(prefix..old_end, replacement_slots);
        self.generation = generation;
    }

    /// Rebuild every line from scratch — a structural edit, a shape mismatch,
    /// a reshape that bypassed `set_text`, or first activation. `spans` is
    /// the CURRENT full per-line grouping (moved in, becoming the new
    /// baseline); `built` holds the finished protos for every line that
    /// carries at least one span.
    pub(super) fn reseed(
        &mut self,
        spans: Vec<Vec<(usize, usize)>>,
        mut built: std::collections::HashMap<usize, Vec<UnderlineProto>>,
        generation: u64,
    ) {
        self.slots = (0..spans.len())
            .map(|li| built.remove(&li).unwrap_or_default())
            .collect();
        self.spans = spans;
        self.valid = true;
        self.generation = generation;
    }

    /// Lines whose span list differs from what is cached, or `None` when the
    /// shape itself no longer matches (line count changed, or nothing is
    /// cached yet) — the caller's signal to [`Self::reseed`] instead. Used
    /// ONLY on the no-reshape axis (`generation` unchanged, so every OTHER
    /// line's pixels are already known good): a dictionary edit or
    /// spellcheck toggle can move `self.misspelled` on scattered lines with
    /// no accompanying `TextChange` band at all.
    pub(super) fn diff_lines(
        &self,
        spans: &[Vec<(usize, usize)>],
    ) -> Option<std::collections::BTreeSet<usize>> {
        if !self.valid || self.spans.len() != spans.len() {
            return None;
        }
        Some(
            (0..spans.len())
                .filter(|&li| self.spans[li] != spans[li])
                .collect(),
        )
    }

    /// Apply a scattered-line reconciliation: `diff_lines` names exactly the
    /// lines being replaced (their pixels come from `built`, empty for a line
    /// whose spans became empty), every other line is untouched. `spans`
    /// becomes the new baseline for the NEXT [`Self::diff_lines`] call.
    pub(super) fn apply_diff(
        &mut self,
        spans: Vec<Vec<(usize, usize)>>,
        diff_lines: &std::collections::BTreeSet<usize>,
        mut built: std::collections::HashMap<usize, Vec<UnderlineProto>>,
    ) {
        for &li in diff_lines {
            if let Some(slot) = self.slots.get_mut(li) {
                *slot = built.remove(&li).unwrap_or_default();
            }
        }
        self.spans = spans;
    }

    /// Stamp the `spell_gen` the CURRENT `spans`/`slots` now agree with — the
    /// one write every resolution path (splice, reseed, diff-apply, or a
    /// bare confirmation that nothing had drifted) ends on.
    pub(super) fn mark_reconciled(&mut self, spell_gen: u64) {
        self.reconciled_spell_gen = spell_gen;
    }
}
