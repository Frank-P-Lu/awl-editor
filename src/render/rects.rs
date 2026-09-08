//! Rectangle and squiggle builders for document-adjacent GPU layers.

use super::*;

type FootnoteMark = (usize, usize, std::ops::Range<usize>, usize);
mod ranges;
#[cfg(test)]
pub(super) use ranges::intersecting_rows;

/// [`TextPipeline::wash_rects`]'s triple: comment-span, string-span, and
/// syntax-highlight-span wash rects, in that order.
type WashRects = (Vec<[f32; 4]>, Vec<[f32; 4]>, Vec<[f32; 4]>);

mod reveal;
mod underlines;

/// A contiguous run of blockquote lines is ONE block, recorded as `(first, last)`
/// and only ever growing downward — the two ends the hanging pull-quote pair hangs
/// from. `prev` says whether the PREVIOUS line was a quote too, so a run that
/// resumes after a gap opens a NEW block instead of swallowing the gap into the
/// one above it.
fn push_or_grow_quote_block(quotes: &mut Vec<(usize, usize)>, li: usize, prev: bool) {
    match quotes.last_mut() {
        Some(block) if prev => block.1 = li,
        _ => quotes.push((li, li)),
    }
}

/// Which end of a blockquote block a hanging pull-quote mark hangs from. Both
/// ends are shaped from one face, one scale, one [`crate::theme::faint`] value,
/// so the pair can never drift apart in weight — they differ in glyph, and in
/// how their position is derived: Open is a page-geometry constant
/// (`geometry::pull_quote_left`); Close follows its own block's last-row ink
/// (`geometry::pull_quote_close_x`, `TextPipeline::quote_close_row_end_x`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum QuoteSide {
    Open,
    Close,
}

/// CACHED ORNAMENT LINE LISTS — the cursor-INDEPENDENT set of logical lines that
/// carry a markdown thematic-break `Rule` span, and the set of unordered-list
/// (bullet) lines. Both are a pure function of the shaped TEXT, so they are rebuilt
/// only when the document reshapes (keyed by [`TextPipeline::reshape_count`], the
/// pipeline's text version) rather than re-scanned every frame. Each frame the
/// ornament pass just FILTERS these to the visible row range (+ excludes the caret
/// line) — turning the old O(lines × md_spans) per-frame scan into O(visible).
/// Interior-mutable so the read-only `rule_lines` / `bullet_marks` can lazily fill
/// it. Dropped implicitly on the next reshape (the version key no longer matches).
pub(super) struct OrnamentCache {
    version: std::cell::Cell<Option<u64>>,
    rule_lines: std::cell::RefCell<Vec<usize>>,
    bullet_lines: std::cell::RefCell<Vec<usize>>,
    table_blocks: std::cell::RefCell<Vec<(usize, std::ops::Range<usize>)>>,
    /// `(first line, last line)` per contiguous blockquote BLOCK — the two ends the
    /// hanging pull-quote pair hangs from (see [`QuoteSide`]).
    quote_blocks: std::cell::RefCell<Vec<(usize, usize)>>,
    fence_lang_blocks: std::cell::RefCell<Vec<(usize, crate::syntax::Lang)>>,
    /// `(line, char column, source range, display number)` for recognized
    /// references and first-line definition labels. Cursor/selection reveal is
    /// filtered per frame; text-derived membership is reshape-cached.
    footnote_marks: std::cell::RefCell<Vec<FootnoteMark>>,
    /// `(line, source range)` for every bare-URL TAIL span (never a SCHEME span —
    /// see `render::spans::conceal::is_bare_url_tail`), the ellipsis affordance's
    /// text-derived membership. Cursor/selection reveal is filtered per frame in
    /// [`TextPipeline::bare_url_marks`], exactly like `footnote_marks`.
    bare_url_tails: std::cell::RefCell<Vec<(usize, std::ops::Range<usize>)>>,
    /// `(line, source range)` for every smart-punctuation span — the painted
    /// substitute glyph's text-derived membership. Cursor/selection reveal is
    /// filtered per frame in [`TextPipeline::smart_punct_marks`], exactly like
    /// `bare_url_tails`; the WHICH-GLYPH kind is re-derived at read time
    /// ([`smart_punct_kind_for`]), never cached, mirroring `is_bare_url_tail`.
    smart_punct_spans: std::cell::RefCell<Vec<(usize, std::ops::Range<usize>)>>,
}

impl OrnamentCache {
    pub(super) fn new() -> Self {
        Self {
            version: std::cell::Cell::new(None),
            rule_lines: std::cell::RefCell::new(Vec::new()),
            bullet_lines: std::cell::RefCell::new(Vec::new()),
            table_blocks: std::cell::RefCell::new(Vec::new()),
            quote_blocks: std::cell::RefCell::new(Vec::new()),
            fence_lang_blocks: std::cell::RefCell::new(Vec::new()),
            footnote_marks: std::cell::RefCell::new(Vec::new()),
            bare_url_tails: std::cell::RefCell::new(Vec::new()),
            smart_punct_spans: std::cell::RefCell::new(Vec::new()),
        }
    }
}

/// CACHED UNDERLINE GEOMETRY — the scroll-INDEPENDENT part of every spell-squiggle
/// / nit-underline band, precomputed once per shaped-text version instead of
/// rebuilt every frame. Building a band needs the owning visual row's wrap-aware
/// top/height and the span's per-char x boundaries; fetching those per span via
/// `visual_rows(line)` walks EVERY shaped run of the document per call — the exact
/// pre-fix ornament pattern, O(spans × doc) per FRAME (the measured 22 ms of a
/// squiggle-dense doc's 28 ms frame). The protos here hold those row-relative
/// pieces; each frame the builders just add the CURRENT `doc_top` / `text_left`
/// (the only scroll/layout-frame-dependent terms, applied with the identical f32
/// ops) and cull the off-screen bands. Keyed on the [`rowgeom::RowGeom`]
/// GENERATION (bumped at every shaped-geometry seam: reshape / zoom / DPI /
/// restyle / sync-wrap) plus a per-source version (the spell list generation, or
/// the reshape count for the text-derived nits), so anything that could stale the
/// geometry misses and rebuilds — via the ONE-WALK [`TextPipeline::visual_rows_for_lines`],
/// so even the rebuild is O(doc), not O(spans × doc). Interior-mutable so the
/// read-only builders can lazily fill it (mirrors [`OrnamentCache`]).
pub(super) struct UnderlineCache {
    version: std::cell::Cell<Option<(u64, u64)>>,
    protos: std::cell::RefCell<Vec<UnderlineProto>>,
}

/// Release-benchmark instrumentation for the document-wide-typing-work
/// investigation: witnesses for [`TextPipeline::ensure_nit_protos`],
/// [`TextPipeline::ensure_ornament_lists`], [`TextPipeline::destination_ranges`]
/// (called from both nit and spell-squiggle rebuilds, so its counters accumulate
/// rather than overwrite), and [`TextPipeline::ensure_squiggle_protos`]. Every
/// owner here is reached through `&self`, hence `Cell`. Populated only while
/// [`TextPipeline::text_sync_profile`] is set — an ordinary editor run pays one
/// bool check per timed owner and never allocates or reads a clock. Reset once
/// per [`TextPipeline::prepare`] call so a call reached twice in one frame
/// (`destination_ranges`) reports the true total, not the last call's slice.
#[derive(Default)]
pub(super) struct OwnerScanWork {
    /// Written from `TextPipeline::set_text`, once per RESHAPE — BEFORE
    /// `prepare`'s reset() runs — by [`NitProjection::refresh`], and
    /// unconditionally OVERWRITTEN (never accumulated, never reset in between)
    /// by `ensure_nit_protos`'s full path when the fast path is ineligible.
    /// So this always reflects whichever nit work actually ran this frame,
    /// regardless of which of the two owners did it.
    pub(super) nit_scan_ms: std::cell::Cell<f64>,
    /// Lines actually RE-TOKENIZED: on the fast path, only the reshape's own
    /// changed band (0 on a pure geometry-only reshape with no text change);
    /// on the full path, every logical line in the document.
    pub(super) nit_scan_lines: std::cell::Cell<u64>,
    pub(super) ornament_scan_ms: std::cell::Cell<f64>,
    /// Logical lines walked by the last `ensure_ornament_lists` cache MISS.
    pub(super) ornament_scan_lines: std::cell::Cell<u64>,
    /// `md_spans.len()` at the time of that same miss — multiplied by
    /// `ornament_scan_lines`, the floor on span comparisons the per-line loop
    /// performs (two of its four passes over `md_spans` never short-circuit).
    pub(super) ornament_scan_spans: std::cell::Cell<u64>,
    pub(super) destination_join_ms: std::cell::Cell<f64>,
    /// Number of times `destination_ranges` actually joined + parsed the whole
    /// document this `prepare` (0 when `md_spans` is empty; up to 2 when both
    /// the nit and spell-squiggle owners miss their caches on the same frame).
    pub(super) destination_join_calls: std::cell::Cell<u64>,
    /// Bytes of the joined document string on the LAST such call.
    pub(super) destination_join_bytes: std::cell::Cell<u64>,
    pub(super) squiggle_scan_ms: std::cell::Cell<f64>,
    /// `self.misspelled.len()` on the last `ensure_squiggle_protos` cache MISS —
    /// zero either on a hit or because the misspelled list was empty (the caller,
    /// `spell_squiggles`, skips the ensure call entirely in that case).
    pub(super) squiggle_scan_misspellings: std::cell::Cell<u64>,
}

/// A plain-data snapshot of [`OwnerScanWork`], read once per timed sample so a
/// caller outside this module (the `typing_live` benchmark) never touches a
/// `Cell` directly.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OwnerScanSnapshot {
    pub(super) nit_scan_ms: f64,
    pub(super) nit_scan_lines: u64,
    pub(super) ornament_scan_ms: f64,
    pub(super) ornament_scan_lines: u64,
    pub(super) ornament_scan_spans: u64,
    pub(super) destination_join_ms: f64,
    pub(super) destination_join_calls: u64,
    pub(super) destination_join_bytes: u64,
    pub(super) squiggle_scan_ms: f64,
    pub(super) squiggle_scan_misspellings: u64,
}

impl OwnerScanWork {
    pub(super) fn snapshot(&self) -> OwnerScanSnapshot {
        OwnerScanSnapshot {
            nit_scan_ms: self.nit_scan_ms.get(),
            nit_scan_lines: self.nit_scan_lines.get(),
            ornament_scan_ms: self.ornament_scan_ms.get(),
            ornament_scan_lines: self.ornament_scan_lines.get(),
            ornament_scan_spans: self.ornament_scan_spans.get(),
            destination_join_ms: self.destination_join_ms.get(),
            destination_join_calls: self.destination_join_calls.get(),
            destination_join_bytes: self.destination_join_bytes.get(),
            squiggle_scan_ms: self.squiggle_scan_ms.get(),
            squiggle_scan_misspellings: self.squiggle_scan_misspellings.get(),
        }
    }

    /// Resets every OWNER except the nit-scan pair: those are written from
    /// `TextPipeline::set_text` (once per RESHAPE, before `prepare` even
    /// starts) and unconditionally overwritten — never accumulated — by
    /// whichever of the fast or full nit path actually ran, so resetting them
    /// here would erase what `set_text` just recorded before `ensure_nit_protos`
    /// ever got a chance to confirm or replace it.
    pub(super) fn reset(&self) {
        self.ornament_scan_ms.set(0.0);
        self.ornament_scan_lines.set(0);
        self.ornament_scan_spans.set(0);
        self.destination_join_ms.set(0.0);
        self.destination_join_calls.set(0);
        self.destination_join_bytes.set(0);
        self.squiggle_scan_ms.set(0.0);
        self.squiggle_scan_misspellings.set(0);
    }
}

/// Whether `ensure_nit_protos`' retained per-line FAST PATH ([`NitProjection`])
/// applies: the whole document must be plain prose, with none of the four
/// inputs that otherwise force `ensure_nit_protos` to read DOCUMENT-WIDE
/// context to score a single line — a table appearing anywhere changes which
/// lines take the table-row nit variant; frontmatter anywhere shifts which
/// lines are skipped; a markdown destination anywhere excludes matching spans
/// wherever they land; and a recognized code language scopes every nit to the
/// LEXER's prose ranges, which can cross logical lines. Any one of these
/// present anywhere routes to the untouched full recompute every time —
/// exactly today's algorithm, byte for byte, for every code buffer, every
/// table, every frontmatter block, every doc with a link or image.
pub(super) fn nit_fast_path_eligible(
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    syn_lang: Option<crate::syntax::Lang>,
) -> bool {
    syn_lang.is_none()
        && crate::markdown::frontmatter_end(md_spans).is_none()
        && !md_spans.iter().any(|(_, k)| {
            k.is_table_markup()
                || matches!(
                    k,
                    crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::Link)
                        | crate::markdown::MdKind::ConcealMarkup(
                            crate::markdown::ConcealKind::Image
                        )
                )
        })
}

/// Retained per-line WRITING-NIT spans:
/// `ensure_nit_protos` used to re-tokenize EVERY logical line's own text on
/// EVERY reshape, even though a raw nit span (before the frontmatter/table/
/// prose/destination filters `ensure_nit_protos` applies afterward) is a pure
/// function of that one line's text alone. This keeps that raw per-line
/// result and reuses it for every line OUTSIDE the exact band a reshape
/// touched — the identical `(prefix, old_end, new_end)` band the row-geometry
/// patch's own `TextChange` already computes, so retention costs no extra
/// document-wide comparison to find: [`TextPipeline::set_text`] hands it over
/// the moment it has it.
///
/// Refreshed exactly once per RESHAPE (from `set_text`, not lazily from
/// `prepare`), so two reshapes that land before the next drawn frame both
/// still patch the cache — nothing is lost to a coalesced redraw the way a
/// prepare-time refresh could lose an intermediate edit's band.
///
/// FAST PATH ONLY: active exactly when [`nit_fast_path_eligible`] holds.
/// Falling out of eligibility (a table/frontmatter/link/code state appears)
/// simply clears the cache; the very next `ensure_nit_protos` call takes the
/// full path and finds `eligible() == false`, so it never consults this at
/// all. Coming BACK into eligibility reseeds it from scratch on that same
/// call (a full scan once, not a correctness risk) rather than trying to
/// reconstruct history it never retained.
#[derive(Default)]
pub(super) struct NitProjection {
    eligible: bool,
    /// Per-line RAW spans (before any filter), in CURRENT line order.
    slots: Vec<Vec<(usize, usize)>>,
}

impl NitProjection {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn eligible(&self) -> bool {
        self.eligible
    }

    pub(super) fn line_count(&self) -> usize {
        self.slots.len()
    }

    pub(super) fn spans_for(&self, line: usize) -> &[(usize, usize)] {
        self.slots.get(line).map_or(&[], Vec::as_slice)
    }

    /// Bring the cache up to date with `lines` (the CURRENT document, one
    /// entry per logical line). `change` is the exact changed-line band the
    /// reshape that produced `lines` reported (`prefix`, old end, new end),
    /// or `None` before this pipeline has ever reshaped. Returns the number
    /// of lines actually re-tokenized (0 on a pure retain).
    pub(super) fn refresh(
        &mut self,
        eligible: bool,
        lines: &[&str],
        change: Option<(usize, usize, usize)>,
    ) -> u64 {
        if !eligible {
            self.eligible = false;
            self.slots.clear();
            return 0;
        }
        if let Some((prefix, old_end, new_end)) = change
            && self.eligible
            && prefix <= old_end
            && new_end <= lines.len()
        {
            let old_len = old_end + (lines.len() - new_end);
            if self.slots.len() == old_len {
                // Splice: the SAME prefix/replace/suffix shape
                // `splice_changed_lines` uses to retain glyphon's unaffected
                // rows, for the identical reason — only the band's own TEXT
                // could have changed; everything outside it is untouched byte
                // for byte (`unchanged_band`'s own guarantee).
                let mut next = Vec::with_capacity(lines.len());
                next.extend_from_slice(&self.slots[..prefix]);
                let mut retokenized = 0u64;
                for &line in &lines[prefix..new_end] {
                    next.push(crate::nits::line_nits(line));
                    retokenized += 1;
                }
                next.extend_from_slice(&self.slots[old_end..]);
                self.slots = next;
                return retokenized;
            }
        }
        // First activation, a state mismatch defensive fallback, or nothing
        // has reshaped through this cache yet: seed every line once.
        self.slots = lines
            .iter()
            .map(|line| crate::nits::line_nits(line))
            .collect();
        self.eligible = true;
        lines.len() as u64
    }
}

/// One cached underline span: the owning visual row's buffer-relative top +
/// height (`VisualRow::line_top` / `line_height`) and the span's x boundaries
/// relative to the text left edge (`row.xs[s]` / `row.xs[e]`, exactly the two
/// values [`row_x_span`] reads). Everything a frame needs to emit the identical
/// [`Squiggle`] once the frame's `doc_top` / `text_left` / metrics are applied.
/// `line`/`start_col`/`end_col` are the span's LOGICAL location (char columns),
/// carried so a per-frame read can apply REVEAL-ON-CURSOR — the caret's own line
/// (nits) or the caret's own word (spell) — without busting the cache on every
/// cursor move (cursor position is not part of the cache KEY; filtering happens
/// at read time in [`TextPipeline::nit_underlines`] / [`TextPipeline::spell_squiggles`],
/// mirroring the existing `rule_lines`/`bullet_marks` reveal-on-cursor pattern).
/// Unused by the wash cache (no caret exclusion there) — harmlessly along for the ride.
struct UnderlineProto {
    line: usize,
    start_col: usize,
    end_col: usize,
    line_top: f32,
    line_height: f32,
    xs_s: f32,
    xs_e: f32,
}

impl UnderlineCache {
    pub(super) fn new() -> Self {
        Self {
            version: std::cell::Cell::new(None),
            protos: std::cell::RefCell::new(Vec::new()),
        }
    }
}

/// CACHED WASH GEOMETRY — the scroll-INDEPENDENT quad protos of the syntax
/// background WASHES: one low-alpha tinted band behind every PROSE-comment span
/// and (on the dark worlds) every string span, per visual row, PLUS (since the
/// WYSIWYG round) a small value-step PILL behind every INLINE code span
/// (`MdKind::Code { inline: true }`). Mirrors [`UnderlineCache`]: keyed on the
/// [`rowgeom::RowGeom`] GENERATION plus `reshape_count` (the `syn_spans` /
/// `md_spans` are re-lexed on every reshape, so the reshape count is the correct
/// source-version half — the same key as the nit cache) PLUS the `wysiwyg_on()`
/// flag (the inline-code PILL bucket is built only when WYSIWYG is on, and that
/// process-global can flip WITHOUT a reshape, so it must be part of the key or a
/// stale bucket would keep drawing a pill after the toggle), rebuilt via the
/// ONE-WALK [`TextPipeline::visual_rows_for_lines`], and per frame just offset by
/// `doc_top` / `text_left` + culled to the visible band (O(visible), never
/// O(doc)). Cursor moves and scrolls never invalidate it. SIX proto buckets so
/// the comment, string, highlight, code-pill, strike-line, and link-underline
/// geometries ride their own fixed-tint pipelines (the markdown `==highlight==`
/// band has its OWN violet tint now, decoupled from the comment wash — see
/// [`super::spans::highlight_wash`]). Interior-mutable like its siblings.
pub(super) struct WashCache {
    version: std::cell::Cell<Option<(u64, u64, bool)>>,
    comment_protos: std::cell::RefCell<Vec<UnderlineProto>>,
    string_protos: std::cell::RefCell<Vec<UnderlineProto>>,
    highlight_protos: std::cell::RefCell<Vec<UnderlineProto>>,
    code_pill_protos: std::cell::RefCell<Vec<UnderlineProto>>,
    /// `MdKind::Strikethrough` span segments — the per-visual-row x-extents the
    /// strike LINE rides ([`TextPipeline::strike_lines`], positioned by the one
    /// owner `super::spans::strike_line_band`). A fifth bucket of the SAME
    /// cache/build walk, not a parallel cache.
    strike_protos: std::cell::RefCell<Vec<UnderlineProto>>,
    /// `MdKind::LinkText` AND `MdKind::BareUrlText` span segments — the ONE
    /// followable-span grammar, per-visual-row x-extents for the quiet
    /// UNDERLINE both ride ([`TextPipeline::link_underlines`], positioned by
    /// `super::spans::link_underline_band` — the SAME line-band primitive
    /// `strike_line_band` rides, just a different vertical fraction). A sixth
    /// bucket of the SAME cache/build walk.
    link_underline_protos: std::cell::RefCell<Vec<UnderlineProto>>,
}

impl WashCache {
    pub(super) fn new() -> Self {
        Self {
            version: std::cell::Cell::new(None),
            comment_protos: std::cell::RefCell::new(Vec::new()),
            string_protos: std::cell::RefCell::new(Vec::new()),
            highlight_protos: std::cell::RefCell::new(Vec::new()),
            code_pill_protos: std::cell::RefCell::new(Vec::new()),
            strike_protos: std::cell::RefCell::new(Vec::new()),
            link_underline_protos: std::cell::RefCell::new(Vec::new()),
        }
    }
}

struct RowBandProto {
    line_top: f32,
    line_height: f32,
}

/// CACHED FENCE-PANEL GEOMETRY — the scroll-independent row bands behind every
/// FENCED code block (`ConcealKind::Fence`), one per visual row spanning the
/// whole block (marker lines AND body) — the quiet value-step background that is
/// always present once WYSIWYG is on, independent of the caret (only the marker
/// TEXT concealment is caret-gated; this panel is not). Mirrors [`WashCache`]:
/// same generation+reshape+`wysiwyg_on()` key (the whole panel is gated on
/// WYSIWYG, and that global can flip without a reshape, so it rides the key too),
/// same one-walk rebuild, same per-frame O(visible) offset+cull. Empty for a
/// non-markdown / fence-less buffer, or with WYSIWYG off.
pub(super) struct FencePanelCache {
    version: std::cell::Cell<Option<(u64, u64, bool)>>,
    protos: std::cell::RefCell<Vec<RowBandProto>>,
}

impl FencePanelCache {
    pub(super) fn new() -> Self {
        Self {
            version: std::cell::Cell::new(None),
            protos: std::cell::RefCell::new(Vec::new()),
        }
    }
}

/// SUB-DEVICE-PIXEL TOLERANCE on a device-grid coincidence test, so `Physical`
/// for the reason `menubar::FLUSH_EPS` is: the question is not "how much
/// breathing room" but "do these two quad edges land on the same pixel". Scaled,
/// two rows sitting 1.4 device px apart on a 3x display would start counting as
/// contiguous and be merged — a different quad, not a better-tuned one.
const ROW_MERGE_EPS: Physical = Physical(0.5);

/// Merge vertically-CONTIGUOUS quads (already built for THIS frame, in any
/// order) into fewer, taller quads — the fix for the WYSIWYG live-review's
/// "seam between rows" report on the fence PANEL and the multi-row prose
/// WASH. `shaders/selection.wgsl` draws every instance as an independently
/// rounded, ~1px-antialiased quad (`fs_main`'s `smoothstep` edge feather,
/// applied on ALL four edges, not just the rounded corners); two quads that
/// merely TOUCH at a shared edge each fade toward that boundary
/// independently, and compositing two half-faded edges (`over` blending)
/// reads as a visible thin band spanning the FULL WIDTH of the shared edge —
/// this is what showed as "horizontal lines between rows" even though
/// `fence_panel_rects`' per-row geometry was already mathematically
/// contiguous (cosmic-text accumulates `line_top += line_height` exactly,
/// see `buffer.rs`'s `LayoutRunIter` — there is no real gap to close). The
/// fix is structural, not a bigger overlap (which would double-blend the
/// shared strip instead): collapse any two quads whose vertical extents are
/// CONTIGUOUS (the next's top sits within [`ROW_MERGE_EPS`] of the current's
/// bottom) into ONE instance spanning their union — rounding + edge
/// antialiasing then only ever happens at the TRUE outer edges of a
/// contiguous run, never at an internal row boundary. For same-x-width quads
/// (the fence panel: every row spans the whole text column) this is EXACT;
/// for a variable-width prose wash (a wrapped comment, or a multi-line
/// docstring where each row's own glyph extent differs) the merged quad
/// takes the UNION x-range — a minor, common editorial looseness (the
/// highlight reads as one continuous band rather than hugging every row's
/// own width) preferred over re-opening the seam by keeping separate
/// abutting quads. `bands` need not be pre-sorted; two quads on the SAME row
/// (equal `y`) never merge into each other (their "bottom" only reaches the
/// next row's top, never their own `y` again).
pub(super) fn merge_row_bands(mut bands: Vec<[f32; 4]>) -> Vec<[f32; 4]> {
    if bands.len() < 2 {
        return bands;
    }
    bands.sort_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal));
    let mut out: Vec<[f32; 4]> = Vec::with_capacity(bands.len());
    for b in bands {
        if let Some(last) = out.last_mut() {
            let last_bottom = last[1] + last[3];
            if (b[1] - last_bottom).abs() <= ROW_MERGE_EPS.0 {
                let new_left = last[0].min(b[0]);
                let new_right = (last[0] + last[2]).max(b[0] + b[2]);
                let new_bottom = (b[1] + b[3]).max(last_bottom);
                last[0] = new_left;
                last[2] = new_right - new_left;
                last[3] = new_bottom - last[1];
                continue;
            }
        }
        out.push(b);
    }
    out
}

impl TextPipeline {
    fn ensure_ornament_lists(&self) {
        if self.ornament_cache.version.get() == Some(self.reshape_count) {
            return;
        }
        let scan_at = self.text_sync_profile.then(crate::clock::Instant::now);
        if self.text_sync_profile {
            self.owner_scan
                .ornament_scan_lines
                .set(self.buffer.lines.len() as u64);
            self.owner_scan
                .ornament_scan_spans
                .set(self.md_spans.len() as u64);
        }
        let mut rules = Vec::new();
        let mut bullets = Vec::new();
        let mut tables: Vec<(usize, std::ops::Range<usize>)> = Vec::new();
        let mut quotes: Vec<(usize, usize)> = Vec::new();
        let mut prev_quote = false;
        let mut fence_langs: Vec<(usize, crate::syntax::Lang)> = Vec::new();
        let mut footnotes = Vec::new();
        let mut bare_url_tails: Vec<(usize, std::ops::Range<usize>)> = Vec::new();
        let mut smart_punct_spans: Vec<(usize, std::ops::Range<usize>)> = Vec::new();
        // SWEEP-LINE span index: `md_spans` sorted by start, walked alongside
        // the lines in byte order via an `active` set (spans whose range can
        // still overlap the CURRENT or a later line). Each span enters
        // `active` once (when its start comes into view) and leaves once
        // (when its end falls behind the sweep), so the per-line work below
        // is bounded by how many spans genuinely overlap nearby lines —
        // O(lines + spans log spans) total — rather than the old O(lines ×
        // spans): every one of the four passes below used to walk the WHOLE
        // `md_spans` list for EVERY line, regardless of how far that span sat
        // from the line in question.
        let mut sorted_spans: Vec<&(std::ops::Range<usize>, crate::markdown::MdKind)> =
            self.md_spans.iter().collect();
        sorted_spans.sort_by_key(|(r, _)| r.start);
        let mut next_span = 0usize;
        let mut active: Vec<&(std::ops::Range<usize>, crate::markdown::MdKind)> = Vec::new();

        let mut start = 0usize;
        for (li, line) in self.buffer.lines.iter().enumerate() {
            let text = line.text();
            let end = start + text.len();
            if !self.md_spans.is_empty() {
                // The loosest look-ahead any check below needs is the RULE
                // check's `end + 1`; admitting spans that far ahead keeps
                // every other (tighter) check correct too, since each still
                // applies its OWN exact predicate against `active` below.
                while next_span < sorted_spans.len() && sorted_spans[next_span].0.start < end + 1 {
                    active.push(sorted_spans[next_span]);
                    next_span += 1;
                }
                active.retain(|(r, _)| r.end > start);

                for (r, k) in &active {
                    if r.start < start || r.start >= end {
                        continue; // overlapping this line, but didn't START here
                    }
                    if *k
                        == crate::markdown::MdKind::ConcealMarkup(
                            crate::markdown::ConcealKind::Fence,
                        )
                        && let Some(lang) = crate::markdown::fence_line_lang(text)
                    {
                        fence_langs.push((li, lang));
                    }
                    if *k
                        == crate::markdown::MdKind::ConcealMarkup(
                            crate::markdown::ConcealKind::BareUrl,
                        )
                        && is_bare_url_tail(text, r.start - start)
                    {
                        bare_url_tails.push((li, (*r).clone()));
                    }
                    if *k
                        == crate::markdown::MdKind::ConcealMarkup(
                            crate::markdown::ConcealKind::SmartPunct,
                        )
                    {
                        smart_punct_spans.push((li, (*r).clone()));
                    }
                    let number = match *k {
                        crate::markdown::MdKind::FootnoteReference(number)
                        | crate::markdown::MdKind::FootnoteDefinition(number) => Some(number),
                        _ => None,
                    };
                    if let Some(number) = number {
                        let byte = r.start - start;
                        let col = text[..byte].chars().count();
                        footnotes.push((li, col, (*r).clone(), number));
                    }
                }
            }
            let is_quote = !self.md_spans.is_empty()
                && active.iter().any(|(r, k)| {
                    *k == crate::markdown::MdKind::ConcealMarkup(
                        crate::markdown::ConcealKind::Blockquote,
                    ) && r.start < end
                        && r.end > start
                });
            if is_quote {
                push_or_grow_quote_block(&mut quotes, li, prev_quote);
            }
            prev_quote = is_quote;
            if !self.md_spans.is_empty() {
                for (r, k) in &active {
                    if *k
                        == crate::markdown::MdKind::ConcealMarkup(
                            crate::markdown::ConcealKind::Table,
                        )
                        && r.start == start
                    {
                        tables.push((li, (*r).clone()));
                    }
                }
            }
            // A thematic-break line (driven by the parsed md_spans, exactly as the old
            // per-frame `rule_lines` scan) — cursor-independent (the caret exclusion is
            // applied at read time so the cache survives a pure cursor move).
            if !self.md_spans.is_empty()
                && active.iter().any(|(r, k)| {
                    *k == crate::markdown::MdKind::Rule && r.start < end + 1 && r.end > start
                })
            {
                rules.push(li);
            }
            if crate::markdown::list_item(text).is_some_and(|it| !it.ordered) {
                bullets.push(li);
            }
            start = end + 1;
        }
        *self.ornament_cache.rule_lines.borrow_mut() = rules;
        *self.ornament_cache.bullet_lines.borrow_mut() = bullets;
        *self.ornament_cache.table_blocks.borrow_mut() = tables;
        *self.ornament_cache.quote_blocks.borrow_mut() = quotes;
        *self.ornament_cache.fence_lang_blocks.borrow_mut() = fence_langs;
        *self.ornament_cache.footnote_marks.borrow_mut() = footnotes;
        *self.ornament_cache.bare_url_tails.borrow_mut() = bare_url_tails;
        *self.ornament_cache.smart_punct_spans.borrow_mut() = smart_punct_spans;
        self.ornament_cache.version.set(Some(self.reshape_count));
        if let Some(at) = scan_at {
            self.owner_scan
                .ornament_scan_ms
                .set(at.elapsed().as_secs_f64() * 1000.0);
        }
    }

    /// Buffer-relative -> absolute: the top y of logical `line`'s ornament (its first
    /// visual row), read O(1) from the cached [`rowgeom::RowGeom`] first-row-top table
    /// (== `doc_top() + visual_rows(line)[0].line_top`, byte-identical). The ornament
    /// CULL + placement both read this instead of the whole-doc `visual_rows(line)`.
    pub(super) fn line_ornament_top(&self, line: usize) -> f32 {
        self.doc_top()
            + self
                .row_geom
                .line_first_top(&self.buffer, &self.metrics, line)
    }

    /// Buffer-relative -> absolute top y of logical `line`'s LAST visual row — the
    /// wrap-aware counterpart of [`Self::line_ornament_top`], read O(1) from the same
    /// sealed row-geometry walk. The blockquote pull-quote's CLOSING mark hangs here.
    pub(super) fn line_ornament_last_top(&self, line: usize) -> f32 {
        self.doc_top()
            + self
                .row_geom
                .line_last_top(&self.buffer, &self.metrics, line)
    }

    pub(super) fn line_ornament_baseline(&self, line: usize) -> f32 {
        self.doc_top()
            + self
                .row_geom
                .line_first_baseline(&self.buffer, &self.metrics, line)
    }

    /// Buffer-relative -> absolute BASELINE y of logical `line`'s **LAST** visual
    /// row — the wrap-aware counterpart of [`Self::line_ornament_baseline`]. The
    /// blockquote pull-quote's CLOSING mark anchors here rather than to that row's
    /// top, so a mark shaped taller than one body row still reads as belonging to
    /// its own line's baseline instead of riding above it.
    pub(super) fn line_ornament_last_baseline(&self, line: usize) -> f32 {
        self.doc_top()
            + self
                .row_geom
                .line_last_baseline(&self.buffer, &self.metrics, line)
    }

    /// Buffer-relative -> absolute ink-right x of logical `line`'s **LAST** visual
    /// row — the wrap-aware, LAST-row counterpart of
    /// [`Self::fold_affordance_row_end_x`] (which deliberately reads the FIRST
    /// row, appropriate to a collapsed heading's own affordance). The blockquote
    /// pull-quote's CLOSING mark hangs one gap past THIS edge: the block's real
    /// final row of shaped ink, never a row above it.
    pub(super) fn quote_close_row_end_x(&self, line: usize) -> f32 {
        let end = self
            .visual_rows(line)
            .last()
            .and_then(|r| r.xs.get(r.end_col).copied())
            .filter(|x| x.is_finite())
            .unwrap_or(0.0);
        self.text_left() + end
    }

    pub(super) fn table_blocks(&self) -> Vec<(usize, std::ops::Range<usize>)> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        self.ornament_cache.table_blocks.borrow().clone()
    }

    /// True when a row spanning `[top, top + height]` (absolute screen px) could
    /// paint into the canvas — its box INTERSECTS the viewport plus a GENEROUS
    /// margin (many line-heights, far more than any single glyph's vertical
    /// extent), not just its top point. A normal ornament's `height` is ~0 (a
    /// single row is already well inside the margin), but a TALL row — an inline
    /// image's reserved `dh`, which can run to hundreds of px, far past the flat
    /// margin — needs its own BOTTOM edge tested too: a top-only test culls a tall
    /// row the instant its top scrolls `margin` px above the viewport even while
    /// its bottom is still fully on-screen. Byte-identical to the old top-only
    /// test at `height == 0.0`.
    pub(super) fn row_box_visible(&self, top: f32, height: f32) -> bool {
        let margin = self.metrics.line_height * OFFSCREEN_CULL_MARGIN_ROWS.0;
        top + height > -margin && top < self.window_h + margin
    }

    /// True when logical `line`'s ornament could paint into the canvas — its top is
    /// within the viewport plus a GENEROUS margin (many line-heights, far more than
    /// any single glyph's vertical extent). An ornament outside this band is fully
    /// off-screen and would be CLIPPED to nothing by glyphon's `TextBounds` anyway, so
    /// culling it is byte-identical to keeping it; culling merely skips the shaping.
    /// A zero-height point test — see [`Self::row_box_visible`] for a row with real
    /// vertical extent (a tall inline image).
    pub(super) fn line_ornament_visible(&self, line: usize) -> bool {
        self.row_box_visible(self.line_ornament_top(line), 0.0)
    }

    pub(super) fn rule_lines(&self) -> Vec<usize> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        // CACHE + CULL: the rule-line SET is a pure function of the text (cached by
        // reshape version); each frame we just drop the REVEALED lines
        // ([`Self::line_is_revealed`] — caret line or selection touch, the one
        // owner the nit underline's own conceal check reads too) plus the
        // OFF-SCREEN lines (clipped to nothing anyway). Ascending order + the same
        // membership on the visible rows => byte-identical render.
        self.ensure_ornament_lists();
        let selection_touch = self.selection_touch();
        self.ornament_cache
            .rule_lines
            .borrow()
            .iter()
            .copied()
            .filter(|&li| {
                self.line_ornament_visible(li)
                    && !self.line_is_revealed(li, selection_touch.as_ref())
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn rule_line_concealed(&self, li: usize) -> bool {
        let Some(line) = self.buffer.lines.get(li) else {
            return false;
        };
        if line.text().is_empty() {
            return false;
        }
        matches!(line.attrs_list().get_span(0).color_opt, Some(c) if c.a() == 0)
    }

    pub(super) fn rule_marks(&self) -> Vec<(f32, &'static str)> {
        let lines = self.rule_lines();
        if lines.is_empty() {
            return Vec::new();
        }
        let orn = theme::active().ornaments;
        lines
            .into_iter()
            .map(|li| {
                let top = self.line_ornament_top(li);
                let kind = crate::markdown::break_kind(self.buffer.lines[li].text());
                (top, orn.pick(kind))
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn rule_tops(&self) -> Vec<f32> {
        self.rule_marks().into_iter().map(|(t, _)| t).collect()
    }

    pub(super) fn bullet_marks(&self) -> Vec<(f32, f32, char)> {
        if !self.md_enabled {
            return Vec::new();
        }
        // CACHE + CULL (mirrors `rule_lines`): the bullet-line SET is cached by reshape
        // version; each frame we walk only those, skip every REVEALED line (the
        // caret's own, or one a selection touches — `line_is_revealed`, the same
        // owner `rule_lines` reads) and the OFF-SCREEN lines. Ascending order +
        // identical membership on the visible rows => byte-identical to the old
        // whole-document scan.
        self.ensure_ornament_lists();
        let text_left = self.text_left();
        let selection_touch = self.selection_touch();
        // Resolve each visible, non-caret unordered-bullet line to its
        // (line, top, indent, glyph), DEFERRING the marker x: an UNINDENTED bullet's
        // marker sits at column 0 (x == 0), needing no shaped-x lookup at all — the
        // overwhelmingly common case. Only genuinely INDENTED bullets need the shaped
        // x of their marker cell, and those are resolved below in ONE batched
        // `visual_rows_for_lines` walk, NOT a per-line O(li) `line_glyph_xs` (an
        // O(doc) run walk each) — so this pass is O(visible), the same discipline the
        // sibling `rule_marks` honours (cached row-geometry) and the fix `range_rects`
        // already applied for selections.
        let mut items: Vec<(usize, f32, usize, char)> = Vec::new();
        let mut indented: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for &li in self.ornament_cache.bullet_lines.borrow().iter() {
            if self.line_is_revealed(li, selection_touch.as_ref()) {
                continue; // caret's own line, or a selection touching it: raw marker shows
            }
            if !self.line_ornament_visible(li) {
                continue; // off-screen: the glyph would be clipped to nothing
            }
            let Some(it) = crate::markdown::list_item(self.buffer.lines[li].text()) else {
                continue;
            };
            if it.ordered {
                continue; // ordered lists keep their number, no bullet glyph
            }
            let glyph = crate::theme::active().bullet_for_depth(it.depth());
            let top = self.line_ornament_top(li);
            if it.indent > 0 {
                indented.insert(li);
            }
            items.push((li, top, it.indent, glyph));
        }
        let rows_by_line = if indented.is_empty() {
            std::collections::HashMap::new()
        } else {
            self.visual_rows_for_lines(&indented)
        };
        let mut out = Vec::with_capacity(items.len());
        for (li, top, indent, glyph) in items {
            let x = if indent == 0 {
                0.0 // the marker sits at column 0 (text_left), no shaped-x lookup
            } else {
                rows_by_line
                    .get(&li)
                    .and_then(|rows| rows.first())
                    .and_then(|row| row.xs.get(indent).copied())
                    .unwrap_or(0.0)
            };
            out.push((top, text_left + x, glyph));
        }
        out
    }

    /// The visible hanging pull-quote marks: `(row top, side, logical line)`, TWO
    /// per blockquote block — an opening mark on the block's first row and a
    /// closing one on its last, so a quote never reads permanently unclosed. Each
    /// end is culled independently, so a block taller than the viewport still
    /// shows whichever of its two marks is on screen. The closing mark's row is
    /// the last WRAPPED row of the block's last logical line
    /// ([`Self::line_ornament_last_top`]), not that line's first row. A ONE-LINE
    /// block yields both marks on the same row.
    ///
    /// The `line` is the row this mark belongs to (the block's `first` for Open,
    /// `last` for Close) — the caller resolves the OPEN mark's x from
    /// `geometry::pull_quote_left` (a page-geometry constant, independent of which
    /// block) but the CLOSE mark's x from THIS line's own shaped ink
    /// ([`Self::quote_close_row_end_x`]), because unlike the open end the close
    /// end now follows the text rather than hanging in a fixed gutter.
    pub(super) fn quote_marks(&self) -> Vec<(f32, QuoteSide, usize)> {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || !crate::page::page_on() {
            return Vec::new();
        }
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        let mut out = Vec::new();
        for (first, last) in self.ornament_cache.quote_blocks.borrow().iter().copied() {
            if self.line_ornament_visible(first) {
                out.push((self.line_ornament_top(first), QuoteSide::Open, first));
            }
            let close_top = self.line_ornament_last_top(last);
            if self.row_box_visible(close_top, 0.0) {
                out.push((close_top, QuoteSide::Close, last));
            }
        }
        out
    }

    /// `(first line, last line)` of each contiguous blockquote block, in document
    /// order (the reshape-cached [`OrnamentCache::quote_blocks`]) — the blocks a
    /// document produces a pull-quote PAIR for, INDEPENDENT of page mode / scroll
    /// culling. Test accessor for the "one pair per block, nested markers coalesce"
    /// assertion.
    #[cfg(test)]
    pub(super) fn quote_block_lines(&self) -> Vec<(usize, usize)> {
        self.ensure_ornament_lists();
        self.ornament_cache.quote_blocks.borrow().clone()
    }

    pub(super) fn fence_lang_marks(&self) -> Vec<(f32, crate::syntax::Lang)> {
        if !self.md_enabled || self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        self.ornament_cache
            .fence_lang_blocks
            .borrow()
            .iter()
            .copied()
            .filter(|&(li, _)| self.line_ornament_visible(li))
            .map(|(li, lang)| (self.line_ornament_top(li), lang))
            .collect()
    }

    /// Visible WYSIWYG footnote display numbers: `(row top, absolute left,
    /// number, reserved slot width)`. The source marker itself is collapsed by
    /// `add_wysiwyg_conceal_spans`; this uses its char boundary in the SAME
    /// shaped row, so the ornament and the edit/hit-test cell cannot diverge.
    pub(super) fn footnote_marks(&self) -> Vec<(f32, f32, usize, f32)> {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        let selection_touch = selection_touch_bytes(
            self.selection,
            |li| self.line_doc_byte_start(li),
            |li| {
                self.buffer
                    .lines
                    .get(li)
                    .map_or(0, |line| line.text().len())
            },
        );
        let visible: Vec<_> = self
            .ornament_cache
            .footnote_marks
            .borrow()
            .iter()
            .filter(|(line, _, range, _)| {
                *line != self.cursor_line
                    && !selection_touches(selection_touch.as_ref(), range)
                    && self.line_ornament_visible(*line)
            })
            .cloned()
            .collect();
        let lines: std::collections::BTreeSet<_> =
            visible.iter().map(|(line, _, _, _)| *line).collect();
        let rows = self.visual_rows_for_lines(&lines);
        let text_left = self.text_left();
        let doc_top = self.doc_top();
        visible
            .into_iter()
            .filter_map(|(line, col, _, number)| {
                let row = rows
                    .get(&line)?
                    .iter()
                    .find(|row| row.start_col <= col && col <= row.end_col)?;
                let local = col.saturating_sub(row.start_col);
                let x = *row.xs.get(local)?;
                Some((
                    doc_top + row.line_top,
                    text_left + x,
                    number,
                    self.substitute_advances.footnote_slot(number),
                ))
            })
            .collect()
    }

    /// Visible bare-URL ellipsis marks: `(row top, absolute left, reserved slot
    /// width)`. The tail's source is collapsed by `add_wysiwyg_conceal_spans`
    /// (via `is_bare_url_tail`); this reads its char boundary in the SAME shaped
    /// row, so the painted "…" and the reserved zero-width slot can never
    /// diverge — the exact `footnote_marks` precedent, minus the "which number"
    /// payload (an ellipsis carries none, only a position).
    pub(super) fn bare_url_marks(&self) -> Vec<(f32, f32, f32)> {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        let selection_touch = selection_touch_bytes(
            self.selection,
            |li| self.line_doc_byte_start(li),
            |li| {
                self.buffer
                    .lines
                    .get(li)
                    .map_or(0, |line| line.text().len())
            },
        );
        let visible: Vec<_> = self
            .ornament_cache
            .bare_url_tails
            .borrow()
            .iter()
            .filter(|(line, range)| {
                *line != self.cursor_line
                    && !selection_touches(selection_touch.as_ref(), range)
                    && self.line_ornament_visible(*line)
            })
            .cloned()
            .collect();
        let lines: std::collections::BTreeSet<_> = visible.iter().map(|(line, _)| *line).collect();
        let rows = self.visual_rows_for_lines(&lines);
        let text_left = self.text_left();
        let doc_top = self.doc_top();
        visible
            .into_iter()
            .filter_map(|(line, range)| {
                let line_start = self.line_doc_byte_start(line);
                let byte = range.start.checked_sub(line_start)?;
                let col = self.buffer.lines.get(line)?.text()[..byte].chars().count();
                let row = rows
                    .get(&line)?
                    .iter()
                    .find(|row| row.start_col <= col && col <= row.end_col)?;
                let local = col.saturating_sub(row.start_col);
                let x = *row.xs.get(local)?;
                Some((
                    doc_top + row.line_top,
                    text_left + x,
                    self.substitute_advances.ellipsis_slot(),
                ))
            })
            .collect()
    }

    /// Visible smart-punctuation substitute marks: `(row top, absolute left,
    /// which glyph, reserved slot width)`. The span's source is collapsed by
    /// `add_wysiwyg_conceal_spans` (via `add_smart_punct_conceal_spans`); this
    /// reads its char boundary in the SAME shaped row and re-derives WHICH
    /// glyph from the same raw bytes ([`smart_punct_kind_for`]), so the
    /// painted glyph, the reserved zero-width slot, and the concealed source
    /// can never diverge — the `bare_url_marks`/`footnote_marks` precedent,
    /// with a per-mark KIND payload instead of a per-mark NUMBER.
    pub(super) fn smart_punct_marks(
        &self,
    ) -> Vec<(f32, f32, crate::markdown::SmartPunctKind, f32)> {
        if !self.md_enabled || !crate::markdown::wysiwyg_on() || self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_ornament_lists();
        let selection_touch = selection_touch_bytes(
            self.selection,
            |li| self.line_doc_byte_start(li),
            |li| {
                self.buffer
                    .lines
                    .get(li)
                    .map_or(0, |line| line.text().len())
            },
        );
        let visible: Vec<_> = self
            .ornament_cache
            .smart_punct_spans
            .borrow()
            .iter()
            .filter(|(line, range)| {
                *line != self.cursor_line
                    && !selection_touches(selection_touch.as_ref(), range)
                    && self.line_ornament_visible(*line)
            })
            .cloned()
            .collect();
        let lines: std::collections::BTreeSet<_> = visible.iter().map(|(line, _)| *line).collect();
        let rows = self.visual_rows_for_lines(&lines);
        let text_left = self.text_left();
        let doc_top = self.doc_top();
        visible
            .into_iter()
            .filter_map(|(line, range)| {
                let line_start = self.line_doc_byte_start(line);
                let byte = range.start.checked_sub(line_start)?;
                let line_text = self.buffer.lines.get(line)?.text();
                let local_end = range.end.checked_sub(line_start)?;
                let kind = smart_punct_kind_for(line_text, byte..local_end)?;
                let col = line_text[..byte].chars().count();
                let row = rows
                    .get(&line)?
                    .iter()
                    .find(|row| row.start_col <= col && col <= row.end_col)?;
                let local = col.saturating_sub(row.start_col);
                let x = *row.xs.get(local)?;
                Some((
                    doc_top + row.line_top,
                    text_left + x,
                    kind,
                    self.substitute_advances.advance(kind),
                ))
            })
            .collect()
    }

    /// The absolute pixel x of a collapsed heading's own rendered text on its
    /// filtered row `line`, EXACTLY where its last glyph ends — no gap added. The
    /// FLOOR the "… N lines" tail may never cross: it is the true
    /// boundary between the heading's own real ink and the row's trailing
    /// whitespace, so a tail clamped no closer than this can only ever draw over
    /// blank space, never over a real glyph. Reads the FIRST VISUAL ROW's own real
    /// glyph advances — [`Self::visual_rows`], NOT the flattened whole-line
    /// [`Self::line_glyph_xs`] — so a heading that WRAPS to a second visual row
    /// still measures against that FIRST row's own text (it never sees the wrapped
    /// row's glyphs, so it never inherits their cumulative width). This matters
    /// because the tail's baseline ([`Self::fold_tail_marks`]) is ALWAYS the
    /// heading's first-row baseline too (a folded heading's own line is never
    /// hidden, but it can still visually wrap) — the gallery sweep's own
    /// find: `line_glyph_xs` deliberately FLATTENS wrapped rows by offsetting each
    /// one past the previous (documented for callers that "don't care which visual
    /// row a column lands on"), so its whole-line `.last()` on a wrapped heading
    /// returned a cumulative sum FAR past the actual column — the tail rendered off
    /// in the page's right margin, disconnected from the heading it annotates.
    /// `visual_rows`' per-row `xs` are genuinely row-LOCAL (no such offset — see
    /// `visual_row_from_run`), so `rows[0].xs[end_col]` is the first row's own real
    /// end x. Honours the heading SIZE scale + any caret-on-line conceal reveal
    /// (both already baked into the shaped run `visual_rows` reads).
    pub(super) fn fold_affordance_row_end_x(&self, line: usize) -> f32 {
        let end = self
            .visual_rows(line)
            .first()
            .and_then(|r| r.xs.get(r.end_col).copied())
            .filter(|x| x.is_finite())
            .unwrap_or(0.0);
        self.text_left() + end
    }

    fn fold_affordance_base_x(&self, line: usize) -> f32 {
        self.fold_affordance_row_end_x(line) + self.metrics.char_width * 0.6
    }

    pub(super) fn fold_tail_marks(&self) -> Vec<(f32, f32, usize, usize)> {
        if self.fold_tails.is_empty() {
            return Vec::new();
        }
        self.fold_tails
            .iter()
            .filter(|t| self.line_ornament_visible(t.line))
            .map(|t| {
                (
                    self.line_ornament_baseline(t.line),
                    self.fold_affordance_base_x(t.line),
                    t.hidden,
                    t.line,
                )
            })
            .collect()
    }

    /// The bullet GLYPHS the renderer would draw, in document order — the char half of
    /// [`Self::bullet_marks`]. A test accessor for the depth-cycle + reveal-on-cursor
    /// assertions (which care about WHICH glyph, not its pixel placement).
    #[cfg(test)]
    pub(super) fn bullet_glyphs(&self) -> Vec<char> {
        self.bullet_marks().into_iter().map(|(_, _, c)| c).collect()
    }

    #[cfg(test)]
    pub(super) fn bullet_marker_concealed(&self, li: usize) -> bool {
        let Some(line) = self.buffer.lines.get(li) else {
            return false;
        };
        let Some(it) = crate::markdown::list_item(line.text()) else {
            return false;
        };
        if it.ordered {
            return false;
        }
        matches!(line.attrs_list().get_span(it.indent).color_opt, Some(c) if c.a() == 0)
    }

    #[cfg(test)]
    pub(super) fn concealed_at(&self, li: usize, local_byte: usize) -> bool {
        let Some(line) = self.buffer.lines.get(li) else {
            return false;
        };
        if local_byte >= line.text().len() {
            return false;
        }
        matches!(line.attrs_list().get_span(local_byte).color_opt, Some(c) if c.a() == 0)
    }

    /// The row-centred caret-height band `(y, height)` for one visual `row` on
    /// logical line `li`, where `line_top` is the row's ABSOLUTE top (`doc_top +
    /// row.line_top`). The caret height is scaled by [`Self::caret_band_scale`] (the
    /// row's own height on a tall heading; a BODY height on an image line — never
    /// the whole tall image row), then centred vertically in the row. Shared by the
    /// squiggle and selection rect builders so both scale identically to a heading
    /// AND to the caret on an image line (no char-wide × image-height pillar).
    pub(super) fn row_caret_band(&self, li: usize, row: &VisualRow, line_top: f32) -> (f32, f32) {
        self.row_band_for(li, row.line_height, line_top)
    }

    fn row_band_for(&self, li: usize, row_height: f32, line_top: f32) -> (f32, f32) {
        let m = &self.metrics;
        let row_caret_h = m.caret_h * self.caret_band_scale(li, row_height);
        let y = line_top + (row_height - row_caret_h) * 0.5;
        (y, row_caret_h)
    }

    /// True when a cached underline proto's row could paint into the canvas — its
    /// absolute vertical extent is within the viewport plus a GENEROUS margin (the
    /// band sits inside `[line_top, line_top + line_height + a few px]`, and the
    /// margin is many line-heights). A band outside this is fully off-screen: the
    /// quad would rasterize nothing, so culling it is byte-identical to emitting it
    /// (mirrors [`Self::line_ornament_visible`]).
    fn proto_visible(&self, line_top: f32, line_height: f32) -> bool {
        let margin = self.metrics.line_height * 8.0;
        line_top + line_height > -margin && line_top < self.window_h + margin
    }

    /// THE CONTENT CLIP: the ONE resolved region every document-
    /// content, selection-adjacent quad — the selection wash, the search-match
    /// highlight, the IME preedit underline, and the caret — must stay inside.
    /// Horizontally it is ALWAYS the writing column (`column_left()` ..
    /// `+column_width()`) — the RELOCATED column while the
    /// document draws into a workspace's comparison ([`Self::comparison_viewport`]),
    /// so "where the document is" and "what bounds it" stay ONE idea. Vertically it is
    /// the whole canvas on an ordinary frame, narrowed to that region's own
    /// extent while it is up ([`Self::doc_clip_band`]). A drag that clamps its
    /// hit-test to the page's own left edge, or a comparison scrolled so a
    /// selected row leaves the region, both resolve through this ONE rect — it
    /// bounds PAINT only, never what's SELECTABLE (the document range in the
    /// sidecar is untouched either way).
    pub(super) fn content_clip(&self) -> (f32, f32, f32, f32) {
        let x0 = self.column_left();
        let x1 = x0 + self.column_width();
        let (y0, y1) = self
            .doc_clip_band()
            .unwrap_or((f32::NEG_INFINITY, f32::INFINITY));
        (x0, y0, x1, y1)
    }

    /// Clip a list of emitted quads to [`Self::content_clip`] (drop
    /// fully-outside rects, TRIM partial ones at whichever axis's edge they
    /// cross) — the quad counterpart of the text layer's `TextBounds` clip, so
    /// a scrolled diff transcript's washes/pills/panels stop AT the card edge
    /// instead of sliding over the margin above/below it, and so a
    /// selection / search-match / preedit rect dragged past the page's own
    /// left or right edge stops at the writing column instead of bleeding into
    /// the margin. Identity on an ordinary frame with no dragged/scrolled
    /// overflow. **SELECTION-ADJACENT QUADS ONLY** — [`Self::range_rects`]
    /// (feeding `selection_rects` + `search_match_rects`) and
    /// [`Self::preedit_rects`]; the caret gate in `layers.rs` reads
    /// [`Self::content_clip`] directly. The DECORATIVE-OVERHANG emitters
    /// ([`Self::fence_panel_rects`], [`Self::code_pill_rects`]) and the
    /// column-bound washes ([`Self::wash_rects`]) route through
    /// [`Self::clip_decorative_rects_to_band`] instead; its doc comment explains
    /// why this clip is wrong for them.
    fn clip_rects_to_band(&self, mut rects: Vec<[f32; 4]>) -> Vec<[f32; 4]> {
        let (x0, y0, x1, y1) = self.content_clip();
        rects.retain_mut(|r| {
            let (rx0, ry0) = (r[0], r[1]);
            let (rx1, ry1) = (r[0] + r[2], r[1] + r[3]);
            if rx1 <= x0 || rx0 >= x1 || ry1 <= y0 || ry0 >= y1 {
                return false;
            }
            let nx0 = rx0.max(x0);
            let ny0 = ry0.max(y0);
            r[2] = rx1.min(x1) - nx0;
            r[3] = ry1.min(y1) - ny0;
            r[0] = nx0;
            r[1] = ny0;
            true
        });
        rects
    }

    /// DIFF-AS-PREVIEW: clip a list of emitted quads to the panel's content Y
    /// band alone (drop fully-outside rows, TRIM partial ones at the band
    /// edge) — the quad counterpart of the text layer's `TextBounds` clip, so
    /// a scrolled transcript's washes/pills/panels stop AT the card edge
    /// instead of sliding over the margin above/below it. Identity on an
    /// ordinary frame (no band). X is deliberately UNCLIPPED: the owner for
    /// the DECORATIVE-OVERHANG emitters ([`Self::fence_panel_rects`],
    /// [`Self::code_pill_rects`]) plus the column-bound washes
    /// ([`Self::wash_rects`]'s comment/string/highlight buckets), none of
    /// which want [`Self::clip_rects_to_band`]'s strict writing-column X
    /// bound. An audit found these
    /// decorative emitters through `clip_rects_to_band` too, which X-clipped
    /// the fence panel's [`FENCE_PANEL_INSET_X`] and the code pill's
    /// [`CODE_PILL_INSET_X`] overhang flush to the bare glyph column —
    /// invisible with page mode on (the page pad dwarfs both insets) but
    /// UNIVERSAL with page mode off (`text_pad()` is hard-zeroed then, so
    /// there is no margin to absorb the inset): every fenced block's panel
    /// clipped flush on both edges every frame, and an inline-code pill lost
    /// its left cap whenever the span touched column 0 or the wrap edge. This
    /// fn restores their PRE-84 behaviour: only the diff-panel Y band, their
    /// intended overhang untouched. The wash buckets are structurally
    /// column-bound already (glyph-wrapped), so routing them here instead of
    /// `clip_rects_to_band` is a no-op for them either way — kept on this fn
    /// for one consistent "decorative content, Y-clipped only" story rather
    /// than mixing the two owners without a reason.
    fn clip_decorative_rects_to_band(&self, mut rects: Vec<[f32; 4]>) -> Vec<[f32; 4]> {
        let Some((top, bottom)) = self.doc_clip_band() else {
            return rects;
        };
        rects.retain_mut(|r| {
            let y0 = r[1];
            let y1 = r[1] + r[3];
            if y1 <= top || y0 >= bottom {
                return false;
            }
            let ny0 = y0.max(top);
            r[3] = y1.min(bottom) - ny0;
            r[1] = ny0;
            true
        });
        rects
    }

    fn band_admits(&self, y: f32, h: f32) -> bool {
        match self.doc_clip_band() {
            Some((top, bottom)) => y >= top && y + h <= bottom,
            None => true,
        }
    }

    /// OFF-CURSOR IMAGE-CONCEAL underline guard: the shaped
    /// advance (px) a span on an inline-image line must clear for its spell/nit
    /// underline to survive. Off the caret's line an `![alt](path)` source
    /// conceals to a near-ZERO-width run (`CONCEAL_ZERO_WIDTH_FONT_SIZE`), so a
    /// misspelling / nit the raw scan flagged INSIDE that source (an alt or path
    /// word, a double space in the alt) would otherwise collapse to a 1px stray
    /// tick floating inside the placeholder card. Gated on `line_is_inline_image`
    /// so it can NEVER suppress the deliberate faint tick a trailing-whitespace
    /// nit shows on an ordinary line (also a collapsed run); a REVEALED image line
    /// keeps its full-width source (advance well past this) so its behaviour is
    /// unchanged. Sub-pixel by construction at every zoom (the concealed font size
    /// is 0.01), while any real glyph run clears it with room to spare.
    /// `Physical`, and the doc above is its own reason: this discriminates a
    /// CONCEALED run (0.01 font size, sub-pixel at every zoom and density) from a
    /// real glyph run, with orders of magnitude of margin on both sides. It is a
    /// device-grid question about whether anything rasterized at all, not a tuned
    /// distance the reader's eye should scale.
    const IMAGE_CONCEAL_UNDERLINE_MIN_ADVANCE: Physical = Physical(1.0);

    /// Rebuild the cached WASH quad protos IF the shaped geometry / text changed
    /// since they were last built (keyed on the row-geometry GENERATION +
    /// `reshape_count` — `syn_spans` / `md_spans` are re-lexed each reshape, so
    /// that pair covers every source of wash geometry). The wash spans come from
    /// the pipeline-held span lists: a CODE buffer's `syn_spans` (prose
    /// [`crate::syntax::SynKind::Comment`] + [`crate::syntax::SynKind::Str`]), a
    /// MARKDOWN buffer's fenced `MdKind::CodeSyntax` spans of the same two
    /// roles — the fence inherits through the same source (one owner), with zero
    /// extra code — a MARKDOWN buffer's `MdKind::Highlight` spans (the
    /// `==marked==` convention), which ride the SAME comment bucket: the
    /// highlighter stroke reuses the identical warm wash tint + pipeline as the
    /// prose-comment wash (one owner, no third pipeline/shader) — and (since the
    /// WYSIWYG round, gated on [`crate::markdown::wysiwyg_on`]) every INLINE code
    /// span (`MdKind::Code { inline: true }`), riding a THIRD, value-only pill
    /// bucket. `CommentCode` (commented-out code) deliberately gets NO wash. Byte
    /// spans are cut per LINE (one running-offset walk), converted to char cols,
    /// then clipped per VISUAL row (the `range_rects` row logic) via the one-walk
    /// [`TextPipeline::visual_rows_for_lines`]. A buffer with no sources caches
    /// three EMPTY buckets, so prose renders byte-identically.
    fn ensure_wash_protos(&self) {
        // The inline-code PILL bucket is WYSIWYG-gated, and `wysiwyg_on()` can flip
        // WITHOUT a reshape — so it rides the cache key or a stale bucket would keep
        // drawing a pill after the toggle.
        let wysiwyg = crate::markdown::wysiwyg_on();
        let key = (self.row_geom.generation(), self.reshape_count, wysiwyg);
        if self.wash_cache.version.get() == Some(key) {
            return;
        }
        use crate::syntax::SynKind;
        #[derive(Clone, Copy)]
        enum Bucket {
            Comment,
            Str,
            Highlight,
            CodePill,
            Strike,
            LinkUnderline,
        }
        // A GFM table renders as a drawn GRID (`prepare_table_grid`), which styles
        // each cell's inline code/highlight ITSELF; the raw table source is concealed
        // to zero-width and (when a row wraps) reserves a TALL row. A wash built from
        // an inline-code/highlight span INSIDE that concealed source would collapse to
        // a thin, full-row-height sliver at the left margin — so skip any span that
        // overlaps a table's byte range. (In tables-v1 the source row was one line
        // tall so the sliver was invisible; the wrap-not-clip round's tall rows made
        // it show.)
        let table_ranges: Vec<std::ops::Range<usize>> = self
            .md_spans
            .iter()
            .filter(|(_, k)| {
                *k == crate::markdown::MdKind::ConcealMarkup(crate::markdown::ConcealKind::Table)
            })
            .map(|(r, _)| r.clone())
            .collect();
        let in_table = |r: &std::ops::Range<usize>| {
            table_ranges
                .iter()
                .any(|t| t.start < r.end && t.end > r.start)
        };
        let mut spans: Vec<(std::ops::Range<usize>, Bucket)> = Vec::new();
        for (r, k) in &self.syn_spans {
            match k {
                SynKind::Comment => spans.push((r.clone(), Bucket::Comment)),
                SynKind::Str => spans.push((r.clone(), Bucket::Str)),
                SynKind::CommentCode | SynKind::Constant | SynKind::Definition => {}
            }
        }
        for (r, k) in &self.md_spans {
            if in_table(r) {
                continue;
            }
            match k {
                crate::markdown::MdKind::CodeSyntax { role, .. } => match role {
                    SynKind::Comment => spans.push((r.clone(), Bucket::Comment)),
                    SynKind::Str => spans.push((r.clone(), Bucket::Str)),
                    SynKind::CommentCode | SynKind::Constant | SynKind::Definition => {}
                },
                crate::markdown::MdKind::Highlight => spans.push((r.clone(), Bucket::Highlight)),
                crate::markdown::MdKind::Strikethrough => {
                    spans.push((r.clone(), Bucket::Strike));
                }
                crate::markdown::MdKind::Code { inline: true } if wysiwyg => {
                    spans.push((r.clone(), Bucket::CodePill));
                }
                _ if k.is_followable() => {
                    spans.push((r.clone(), Bucket::LinkUnderline));
                }
                _ => {}
            }
        }
        if spans.is_empty() {
            self.wash_cache.comment_protos.borrow_mut().clear();
            self.wash_cache.string_protos.borrow_mut().clear();
            self.wash_cache.highlight_protos.borrow_mut().clear();
            self.wash_cache.code_pill_protos.borrow_mut().clear();
            self.wash_cache.strike_protos.borrow_mut().clear();
            self.wash_cache.link_underline_protos.borrow_mut().clear();
            self.wash_cache.version.set(Some(key));
            return;
        }
        let mut line_starts: Vec<usize> = Vec::with_capacity(self.buffer.lines.len());
        let mut start = 0usize;
        for line in self.buffer.lines.iter() {
            line_starts.push(start);
            start += line.text().len() + 1; // +1 for the '\n'
        }
        let mut segs: Vec<(usize, usize, usize, Bucket)> = Vec::new();
        for (r, bucket) in &spans {
            let mut li = match line_starts.binary_search(&r.start) {
                Ok(i) => i,
                Err(i) => i.saturating_sub(1),
            };
            while li < self.buffer.lines.len() && line_starts[li] < r.end {
                let ls = line_starts[li];
                let text = self.buffer.lines[li].text();
                let le = ls + text.len();
                let lo = r.start.max(ls);
                let hi = r.end.min(le);
                if lo < hi {
                    let char_col =
                        |b: usize| text.char_indices().take_while(|(bi, _)| *bi < b).count();
                    let s_col = char_col(lo - ls);
                    let e_col = char_col(hi - ls);
                    if e_col > s_col {
                        segs.push((li, s_col, e_col, *bucket));
                    }
                }
                li += 1;
            }
        }
        let lines: std::collections::BTreeSet<usize> = segs.iter().map(|(li, ..)| *li).collect();
        let rows_by_line = self.visual_rows_for_lines(&lines);
        let mut comment_protos = Vec::new();
        let mut string_protos = Vec::new();
        let mut highlight_protos = Vec::new();
        let mut code_pill_protos = Vec::new();
        let mut strike_protos = Vec::new();
        let mut link_underline_protos = Vec::new();
        for (li, s_col, e_col, bucket) in segs {
            let Some(rows) = rows_by_line.get(&li) else {
                continue; // unreachable: every requested line gets rows
            };
            for row in rows {
                let rs = s_col.max(row.start_col);
                let re = e_col.min(row.end_col);
                if re <= rs {
                    continue;
                }
                let char_count = row.xs.len().saturating_sub(1);
                let a = rs.min(char_count);
                let b = re.min(char_count);
                if b <= a {
                    continue;
                }
                let xs_s = row.xs.get(a).copied().unwrap_or(0.0);
                let xs_e = row.xs.get(b).copied().unwrap_or(xs_s);
                let proto = UnderlineProto {
                    line: li,
                    start_col: a,
                    end_col: b,
                    line_top: row.line_top,
                    line_height: row.line_height,
                    xs_s,
                    xs_e,
                };
                match bucket {
                    Bucket::Comment => comment_protos.push(proto),
                    Bucket::Str => string_protos.push(proto),
                    Bucket::Highlight => highlight_protos.push(proto),
                    Bucket::CodePill => code_pill_protos.push(proto),
                    Bucket::Strike => strike_protos.push(proto),
                    Bucket::LinkUnderline => link_underline_protos.push(proto),
                }
            }
        }
        *self.wash_cache.comment_protos.borrow_mut() = comment_protos;
        *self.wash_cache.string_protos.borrow_mut() = string_protos;
        *self.wash_cache.highlight_protos.borrow_mut() = highlight_protos;
        *self.wash_cache.code_pill_protos.borrow_mut() = code_pill_protos;
        *self.wash_cache.strike_protos.borrow_mut() = strike_protos;
        *self.wash_cache.link_underline_protos.borrow_mut() = link_underline_protos;
        self.wash_cache.version.set(Some(key));
    }

    /// Build the syntax WASH quads — `(comment_rects, string_rects,
    /// highlight_rects)`, each `[x, y, w, h]` in pixels for the current scroll +
    /// zoom — from the cached protos (see [`WashCache`]). The markdown
    /// `==highlight==` band is its OWN bucket (its own violet
    /// [`super::spans::highlight_wash`] tint/pipeline, decoupled from the comment
    /// wash so it POPS). Per frame this is O(visible): add the current
    /// `doc_top` / `text_left`, size the band to the row's OWN full height (not
    /// the shorter caret-height band `row_band_for` gives the selection/squiggle
    /// builders — a background wash reads as a continuous highlighted region,
    /// not an inset caret-shaped cell), cull the off-screen rows, then
    /// [`merge_row_bands`] collapses any vertically-CONTIGUOUS rows of the same
    /// bucket into one quad — the fix for the live-review's multi-row striping
    /// report (see that fn's doc comment for the shader-level "why"). Which
    /// bucket actually DRAWS is the prepare layer's call (`prepare_wash_layer`
    /// gates each on the active world's effective [`role_style_for`] wash —
    /// geometry is theme-independent, so a theme switch re-tints without
    /// rebuilding). Both empty for a prose / non-fence buffer, keeping those
    /// renders byte-identical.
    // The three parallel quad lists map directly to the three shader wash buckets.
    pub(super) fn wash_rects(&self) -> WashRects {
        if self.syn_spans.is_empty() && self.md_spans.is_empty() {
            return (Vec::new(), Vec::new(), Vec::new());
        }
        self.ensure_wash_protos();
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let build = |protos: &[UnderlineProto]| {
            let mut out = Vec::with_capacity(protos.len());
            for p in protos {
                let line_top = doc_top + p.line_top;
                if !self.proto_visible(line_top, p.line_height) {
                    continue; // off-screen: the quad would rasterize nothing
                }
                let x = text_left + p.xs_s;
                let w = (p.xs_e - p.xs_s).max(1.0);
                out.push([x, line_top, w, p.line_height]);
            }
            merge_row_bands(out)
        };
        let comment =
            self.clip_decorative_rects_to_band(build(&self.wash_cache.comment_protos.borrow()));
        let string =
            self.clip_decorative_rects_to_band(build(&self.wash_cache.string_protos.borrow()));
        let highlight =
            self.clip_decorative_rects_to_band(build(&self.wash_cache.highlight_protos.borrow()));
        (comment, string, highlight)
    }

    /// The wash cache's current version key, or `None` before the first build —
    /// a test accessor for the invalidation contract (cursor moves + scrolls keep
    /// it warm; reshape / zoom / font switches rebuild).
    #[cfg(test)]
    pub(super) fn wash_cache_version(&self) -> Option<(u64, u64, bool)> {
        self.wash_cache.version.get()
    }

    /// Build the WYSIWYG inline-code PILL quads — `[x, y, w, h]` in pixels for the
    /// current scroll + zoom — from the cached [`WashCache::code_pill_protos`]
    /// (the SAME cache/build as [`Self::wash_rects`], a third bucket). A minimal
    /// inset ([`CODE_PILL_INSET_X`]/`_Y`) grows each quad slightly beyond the
    /// span's own glyph box, so the value-step background reads as a small pill
    /// (a caret-height band, unlike the full-row-height wash/panel — a pill is
    /// meant to hug the inline text closely, not read as a block). Almost always
    /// one quad (inline code practically never wraps), but still runs through
    /// [`merge_row_bands`] for the rare case it does — the same seam
    /// `wash_rects`/`fence_panel_rects` use, one owner. Empty when
    /// [`crate::markdown::wysiwyg_on`] is off (`ensure_wash_protos` never
    /// populates the bucket then) or the buffer has no inline code.
    pub(super) fn code_pill_rects(&self) -> Vec<[f32; 4]> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_wash_protos();
        let protos = self.wash_cache.code_pill_protos.borrow();
        if protos.is_empty() {
            return Vec::new();
        }
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let m = &self.metrics;
        let inset_x = m.px(CODE_PILL_INSET_X);
        let inset_y = m.px(CODE_PILL_INSET_Y);
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would rasterize nothing
            }
            let x = text_left + p.xs_s - inset_x;
            let w = (p.xs_e - p.xs_s) + 2.0 * inset_x;
            let (y, h) = self.row_band_for(p.line, p.line_height, line_top);
            out.push([x, y - inset_y, w, h + 2.0 * inset_y]);
        }
        // The DECORATIVE Y-only clip, not the strict writing-
        // column `clip_rects_to_band` — the pill's own `CODE_PILL_INSET_X`
        // overhang is intended to survive at column 0 / the wrap edge.
        self.clip_decorative_rects_to_band(merge_row_bands(out))
    }

    /// Build the `~~strikethrough~~` STRIKE LINES — one flat [`Squiggle`]
    /// (`amp: 0.0`, the nit-underline trick) per visual-row segment of every
    /// [`crate::markdown::MdKind::Strikethrough`] span, in pixels for the current
    /// scroll + zoom, from the cached [`WashCache::strike_protos`] (the SAME
    /// cache/build as [`Self::wash_rects`], a fifth bucket). The band's vertical
    /// placement + stroke come from THE ONE STRIKE-LINE OWNER
    /// (`super::spans::strike_line_band` over the row's caret-height glyph cell,
    /// [`Self::row_band_for`]) — the same fn the format popover's `S` button
    /// rides, so the two strikes can never drift. NOT caret-gated (content
    /// styling, like the highlight wash — only the `~~` MARKER conceal is
    /// reveal-on-cursor) and NOT WYSIWYG-gated (the muted text transform isn't
    /// either). Empty for a strike-less / non-markdown buffer, keeping those
    /// frames byte-identical.
    pub(super) fn strike_lines(&self) -> Vec<Squiggle> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_wash_protos();
        let protos = self.wash_cache.strike_protos.borrow();
        if protos.is_empty() {
            return Vec::new();
        }
        let m = &self.metrics;
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would be clipped to nothing
            }
            let x = text_left + p.xs_s;
            let w = (p.xs_e - p.xs_s).max(m.px(DECOR_MIN_W));
            let (band_y, cell_h) = self.row_band_for(p.line, p.line_height, line_top);
            let (y, band_h, stroke) = super::spans::strike_line_band(band_y, cell_h, m.scale);
            if !self.band_admits(y, band_h) {
                continue; // DIFF-AS-PREVIEW: the row scrolled past the card edge
            }
            out.push(Squiggle {
                x,
                y,
                w,
                h: band_h,
                amp: 0.0,    // STRAIGHT — a strike is a calm flat line
                period: 1.0, // unused when amp == 0 (kept > 0 so the shader div is safe)
                thickness: stroke,
            });
        }
        out
    }

    /// Build the FOLLOWABLE-SPAN UNDERLINE — one flat [`Squiggle`] (`amp: 0.0`)
    /// per visual-row segment of every [`crate::markdown::MdKind::LinkText`] OR
    /// [`crate::markdown::MdKind::BareUrlText`] span (one grammar, two span
    /// sources — a named `[text](url)` link and a tamed bare URL both draw it),
    /// in pixels for the current scroll + zoom, from the cached
    /// [`WashCache::link_underline_protos`] (the SAME cache/build as
    /// [`Self::strike_lines`], a sixth bucket). The band's vertical placement +
    /// stroke come from `super::spans::link_underline_band` — THE SAME line-band
    /// primitive [`Self::strike_lines`] rides (`super::spans::line_band`), just
    /// near the BASELINE instead of mid-run, so it reads as an underline under
    /// the followable text rather than a line through it: the decided quiet
    /// affordance (the text itself stays full content ink — see `md_attrs`'s
    /// `LinkText`/`BareUrlText` arms — only this underline carries the muted
    /// tint, SOLID ink, never a wash). NOT caret-gated (content styling, like
    /// the strike line — only the markup/conceal plumbing's OWN reveal is
    /// caret-gated; for a bare URL the underline's own EXTENT still moves with
    /// that reveal, because it hugs `row.xs`, the same collapsed/revealed glyph
    /// advances the conceal mechanism already produces — no separate persist/
    /// drop branch) and NOT WYSIWYG-gated. Empty for a followable-span-less /
    /// non-markdown buffer, keeping those frames byte-identical.
    pub(super) fn link_underlines(&self) -> Vec<Squiggle> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_wash_protos();
        let protos = self.wash_cache.link_underline_protos.borrow();
        if protos.is_empty() {
            return Vec::new();
        }
        let m = &self.metrics;
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would be clipped to nothing
            }
            let x = text_left + p.xs_s;
            let w = (p.xs_e - p.xs_s).max(m.px(DECOR_MIN_W));
            let (band_y, cell_h) = self.row_band_for(p.line, p.line_height, line_top);
            let (y, band_h, stroke) = super::spans::link_underline_band(band_y, cell_h, m.scale);
            if !self.band_admits(y, band_h) {
                continue; // DIFF-AS-PREVIEW: the row scrolled past the card edge
            }
            out.push(Squiggle {
                x,
                y,
                w,
                h: band_h,
                amp: 0.0,    // STRAIGHT — a calm flat underline
                period: 1.0, // unused when amp == 0 (kept > 0 so the shader div is safe)
                thickness: stroke,
            });
        }
        out
    }

    fn ensure_fence_panel_protos(&self) {
        // The whole panel is WYSIWYG-gated, and `wysiwyg_on()` can flip WITHOUT a
        // reshape — so it rides the cache key or a stale panel would keep drawing
        // after the toggle.
        let wysiwyg = crate::markdown::wysiwyg_on();
        let key = (self.row_geom.generation(), self.reshape_count, wysiwyg);
        if self.fence_panel_cache.version.get() == Some(key) {
            return;
        }
        if !wysiwyg || self.md_spans.is_empty() {
            self.fence_panel_cache.protos.borrow_mut().clear();
            self.fence_panel_cache.version.set(Some(key));
            return;
        }
        use crate::markdown::{ConcealKind, MdKind};
        let fence_ranges: Vec<std::ops::Range<usize>> = self
            .md_spans
            .iter()
            .filter(|&(_r, k)| matches!(k, MdKind::ConcealMarkup(ConcealKind::Fence)))
            .map(|(r, _k)| r.clone())
            .collect();
        if fence_ranges.is_empty() {
            self.fence_panel_cache.protos.borrow_mut().clear();
            self.fence_panel_cache.version.set(Some(key));
            return;
        }
        let mut line_starts: Vec<usize> = Vec::with_capacity(self.buffer.lines.len());
        let mut start = 0usize;
        for line in self.buffer.lines.iter() {
            line_starts.push(start);
            start += line.text().len() + 1;
        }
        let mut lines: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for r in &fence_ranges {
            let mut li = match line_starts.binary_search(&r.start) {
                Ok(i) => i,
                Err(i) => i.saturating_sub(1),
            };
            while li < self.buffer.lines.len() && line_starts[li] < r.end {
                lines.insert(li);
                li += 1;
            }
        }
        let rows_by_line = self.visual_rows_for_lines(&lines);
        let mut protos = Vec::new();
        for li in &lines {
            let Some(rows) = rows_by_line.get(li) else {
                continue; // unreachable: every requested line gets rows
            };
            for row in rows {
                protos.push(RowBandProto {
                    line_top: row.line_top,
                    line_height: row.line_height,
                });
            }
        }
        *self.fence_panel_cache.protos.borrow_mut() = protos;
        self.fence_panel_cache.version.set(Some(key));
    }

    pub(super) fn fence_panel_rects(&self) -> Vec<[f32; 4]> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_fence_panel_protos();
        let protos = self.fence_panel_cache.protos.borrow();
        if protos.is_empty() {
            return Vec::new();
        }
        let doc_top = self.doc_top();
        let m = &self.metrics;
        let inset = m.px(FENCE_PANEL_INSET_X);
        let x = self.text_left() - inset;
        let w = self.text_wrap_width() + 2.0 * inset;
        let mut out = Vec::with_capacity(protos.len());
        for p in protos.iter() {
            let line_top = doc_top + p.line_top;
            if !self.proto_visible(line_top, p.line_height) {
                continue; // off-screen: the quad would rasterize nothing
            }
            out.push([x, line_top, w, p.line_height]);
        }
        // The DECORATIVE Y-only clip, not the strict writing-
        // column `clip_rects_to_band` — the panel's own `FENCE_PANEL_INSET_X`
        // overhang is by design (reads as a distinct surface, not clipped
        // exactly to the glyph edges) and must survive with page mode off.
        self.clip_decorative_rects_to_band(merge_row_bands(out))
    }

    /// The fence-panel cache's current version key, or `None` before the first
    /// build — a test accessor mirroring [`Self::wash_cache_version`].
    #[cfg(test)]
    pub(super) fn fence_panel_cache_version(&self) -> Option<(u64, u64, bool)> {
        self.fence_panel_cache.version.get()
    }

    pub(super) fn panel_layout(
        &self,
        width: u32,
        caret_byte: usize,
        fallback_chars: usize,
        caret_row: f32,
    ) -> ([f32; 4], f32, f32, f32) {
        let m = &self.metrics;
        let pad = m.px_physical(crate::render::chrome::PANEL_PAD);
        let margin = m.px_physical(crate::render::chrome::PANEL_MARGIN);
        let mut text_w = 0.0_f32;
        let mut rows = 0usize;
        for run in self.panel_buffer.layout_runs() {
            text_w = text_w.max(run.line_w);
            rows += 1;
        }
        let rows = rows.max(1) as f32;
        // Preserve the ordinary content-sized card, but on a narrow canvas cap
        // it to the room between the two outer margins. `panel_shape_text`
        // independently fits every shaped row to the corresponding inner width,
        // so this clamp cannot trade a negative left edge for clipped right ink.
        let card_w = (text_w + 2.0 * pad).min((width as f32 - 2.0 * margin).max(0.0));
        let card_h = rows * m.line_height + 2.0 * pad;
        let card_x = (width as f32 - card_w - margin).max(0.0);
        let card_y = margin + self.menubar_reserve();
        let text_left = card_x + pad;
        let text_top = card_y + pad;
        let caret_x = self.panel_glyph_x(caret_row, caret_byte, fallback_chars, text_left);
        (
            [card_x, card_y, card_w, card_h],
            text_left,
            text_top,
            caret_x,
        )
    }

    /// The physical x of the shaped panel glyph that STARTS at `byte` on row
    /// `row`, or the hardcoded-pitch fallback `text_left + char_width *
    /// fallback_chars` when that row has no glyph there.
    ///
    /// THE ONE PANEL X LOOKUP: the amber caret and the field's selection band
    /// both come through here, so a band edge can never be placed by a
    /// different rule than the caret it sits beside — which is exactly the
    /// hardcoded-pitch drift `panel_layout`'s own doc guards the caret from.
    pub(super) fn panel_glyph_x(
        &self,
        row: f32,
        byte: usize,
        fallback_chars: usize,
        text_left: f32,
    ) -> f32 {
        for run in self.panel_buffer.layout_runs() {
            if run.line_i != row as usize {
                continue;
            }
            for g in run.glyphs.iter() {
                if g.start == byte {
                    return text_left + g.x;
                }
            }
        }
        text_left + self.metrics.char_width * fallback_chars as f32
    }

    /// Underline rectangle(s) for an active IME preedit, in the SAME `[x,y,w,h]`
    /// pixel form as selection rects (they share the translucent-quad pipeline).
    /// The preedit occupies `[start_col, cursor_col)` on the cursor line (it was
    /// spliced in there and the caret advanced to its end); the underline is a
    /// thin bar beneath those real shaped glyphs so composing CJK/kana reads as
    /// provisional. Empty when no composition is active.
    pub(super) fn preedit_rects(&self) -> Vec<[f32; 4]> {
        let n = self.preedit.chars().count();
        if n == 0 {
            return Vec::new();
        }
        let line = self.cursor_line;
        let end_col = self.cursor_col;
        let start_col = end_col.saturating_sub(n);
        // Place on the wrap-aware visual row that owns the preedit's start column
        // (using that row's own x boundaries), matching the caret which sits at
        // the preedit's end.
        let rows = self.visual_rows(line);
        let row = pick_row(&rows, start_col);
        let char_count = row.xs.len().saturating_sub(1);
        let s = start_col.min(char_count);
        let e = end_col.min(char_count);
        let (x, w) = row_x_span(row, self.text_left(), s, e, 1.0);
        let m = &self.metrics;
        let line_top = self.doc_top() + row.line_top;
        let cell_top = line_top + (m.line_height - m.caret_h) * 0.5;
        let thickness = m.px(PREEDIT_UNDERLINE_H);
        let y = cell_top + m.caret_h - thickness;
        // The SAME auxiliary-selection-geometry clip `range_rects`
        // routes through — a composing preedit is selection-adjacent (it rides
        // the SAME translucent-quad pipeline), so it stops at the same page /
        // card boundary rather than bleeding into the margin.
        self.clip_rects_to_band(vec![[x, y, w, thickness]])
    }
}
