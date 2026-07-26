//! `OverlayState`'s QUERY/NAVIGATION half: the fuzzy refilter, the
//! type/scroll/move-selection primitives, and the row-display accessors the
//! renderer + sidecar read (`item_strings`, `empty_message`, …). Split out of
//! the former `overlay.rs` monolith (2026-07 code-organization pass); every
//! item's path is unchanged -- only the file it lives in moved.

use super::{OverlayKind, OverlayState, RangeCell, RowMeta};
use crate::fuzzy::{self, Tier};

/// ITEM 106 — Physical-px SLOP a pointer must travel PAST its last known
/// resting position before [`OverlayState::hover_at`] may re-hit-test/
/// re-select. The ONE owner of the hover movement threshold: every
/// hover-selection path — live `CursorMoved` (`App::overlay_hover`,
/// `app/input/mouse.rs`) and the headless pointer-replay step
/// (`ReplaySession::apply_move`, `main/run.rs`) — funnels through
/// `hover_at`, which is the sole place `OverlayState::selected` is ever
/// mutated by pointer input (see the scout roster at the top of this file's
/// history: every `OverlayKind` shares this ONE gate, never a per-picker
/// fudge). IS `app::DRAG_ARM_SLOP_PX` — the identical "content relocating
/// under a stationary pointer must not read as motion" hazard (there, a
/// text-selection drag arming under a WYSIWYG reveal reflow; here, a
/// picker's candidate rows scrolling under a resting hand), so this is a
/// RE-EXPORT of that ONE constant rather than a second declaration of the
/// same 4.0px physical budget — awl does not grow two independently-tuned
/// pointer-jitter constants that could drift apart under a future retune of
/// either. `pub(super)` (not `pub(crate)`) so `overlay::tests` can pin its
/// boundary law to the REAL value instead of a duplicated magic number,
/// while the re-export itself stays invisible past `overlay`'s own module
/// tree — `app::DRAG_ARM_SLOP_PX` remains the one name anything outside
/// `overlay` would ever reach for.
pub(super) const HOVER_MOVE_SLOP_PX: f32 = crate::app::DRAG_ARM_SLOP_PX;

impl OverlayState {
    /// Re-rank `rows` against the current query into `items`, clamping the
    /// selection. Called after every query edit.
    pub fn refilter(&mut self) {
        let accepts = self.accepts();
        let mut scored = fuzzy::rank(self.query.text(), &accepts, |i| {
            if self.open.contains(&i) {
                Tier::Open
            } else if self.recent.contains(&i) {
                Tier::Recent
            } else {
                Tier::Corpus
            }
        });
        // MRU TIEBREAK: `self.recent` is ordered MOST-RECENT-FIRST (the persisted
        // recently-opened MRU for Goto, the recently-run MRU for the Command palette).
        // Among rows with an EQUAL fuzzy+tier score, the more-recently-used one
        // (smaller position in `recent`) sorts first; non-recent rows fall to
        // `usize::MAX` and keep their original corpus order. `fuzzy::rank` already
        // sorted by (score desc, index asc); this stable re-sort inserts the MRU key
        // between them, so the Recent lens reads newest-first without any per-picker
        // code. Inert when `recent` is empty (the headless capture path) — every
        // position is `MAX`, so the order is byte-identical to the plain rank.
        let recent_rank = |ci: usize| {
            self.recent
                .iter()
                .position(|&x| x == ci)
                .unwrap_or(usize::MAX)
        };
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| recent_rank(a.index).cmp(&recent_rank(b.index)))
                .then_with(|| a.index.cmp(&b.index))
        });
        let mut ranked: Vec<usize> = scored.into_iter().map(|r| r.index).collect();
        // SPELL "Add to dictionary" EXEMPTION (ITEM 64 — always the TERMINAL row):
        // the add row acts on the TARGETED word, not the typed query, so it must
        // stay reachable — and stay LAST, right after the trailing correction —
        // for the whole life of the picker, not just while the query drops it.
        // A query can also fuzzy-MATCH the add label itself (e.g. typing "add");
        // left to the plain ranker its `ci == 0` boundary bonus can out-score every
        // correction and float it to the TOP, which would silently break "reachable
        // right after the last correction". So this drops the add row from wherever
        // ranking put it (present or absent) and re-appends it at the END
        // unconditionally — the corrections keep the ranker's order among
        // themselves, the add row simply always trails them. Inert for every other
        // kind (no row ever carries `RowMeta::SpellAdd`).
        if self
            .rows
            .iter()
            .any(|r| matches!(r.meta, RowMeta::SpellAdd))
        {
            let add_rows: Vec<usize> = self
                .rows
                .iter()
                .enumerate()
                .filter(|(_, r)| matches!(r.meta, RowMeta::SpellAdd))
                .map(|(ci, _)| ci)
                .collect();
            ranked.retain(|ci| !add_rows.contains(ci));
            ranked.extend(add_rows);
        }
        // RUNTIME-GATED ROW FILTER (Command palette only, today): drop any row
        // marked `RowMeta::CommandHidden` (e.g. "Finish file" with no daemon
        // `--wait` client actively waiting — see `commands::visible_hidden_mask`).
        // `rows` itself stays untouched — only what's rankable/selectable shrinks —
        // so the row-index math `commands::visible_action_of` relies on stays
        // valid. A no-op (no row ever carries the tag) for every kind but the
        // Command palette.
        ranked.retain(|&i| {
            !matches!(
                self.rows.get(i).map(|r| &r.meta),
                Some(RowMeta::CommandHidden)
            )
        });
        // FILE VISIBILITY (item 77 — file pickers only, gated on the ONE sticky
        // process global `crate::file_visibility::all_on`): Text (the default)
        // drops any corpus entry whose basename / ancestor component starts with
        // `.` (except `.env*`) AND any BROWSE row already marked unsupported
        // (`OverlayRow::secondary` carries the type label — see
        // `overlay::build::browse_level`); All reveals both. The full corpus is
        // untouched — this is purely what's SHOWN — so flipping the setting and
        // re-running `refilter` reveals/re-hides them with no filesystem re-read.
        // A no-op for non-file pickers (theme/command/…).
        // The Project explorer's synthetic "." accept-this-folder row is EXEMPT — it is
        // the "pick THIS folder" affordance, not a dotfile — so it survives the filter.
        // Go-to HEADING rows are likewise exempt (a heading title is prose, not a
        // dotfile path).
        if !crate::file_visibility::all_on() && self.kind.hides_dotfiles() {
            ranked.retain(|&i| {
                self.rows[i].accept == "."
                    || matches!(self.rows[i].meta, RowMeta::GotoHeading { .. })
                    || !crate::index::is_hidden_entry(&self.rows[i].accept)
            });
            // UNSUPPORTED FILES (Browse only — see `browse_level`'s doc for why
            // this classification is scoped to one directory level, not the
            // whole project): a non-empty `secondary` on a FILE row IS the
            // type label `browse_level` stamped, so its presence alone marks
            // the row unsupported — Browse never otherwise sets `secondary`.
            if self.kind == OverlayKind::Browse {
                ranked.retain(|&i| self.rows[i].is_dir || self.rows[i].secondary.is_empty());
            }
        }
        // HEADINGS-LENS GATE (Go-to only): a REFINEMENT lens other than Headings
        // (Recent / This folder) still lists files only — the appended document-
        // heading rows are dropped there. The flat `All` home (`facet_lens == 0`)
        // is the UNIFIED DEFAULT (item 11): it keeps heading rows IN, mixed with
        // file rows and ranked together by the same fuzzy score, so one query can
        // reach either kind. The Headings lens re-admits them exclusively via its
        // own bucket ([`crate::index::goto_bucket`]). A no-op for every other kind
        // (no row ever carries `RowMeta::GotoHeading`) and for a Go-to over a
        // buffer with no headings.
        if self.kind == OverlayKind::Goto
            && self.facet_lens != 0
            && self.active_facet_id() != Some("headings")
        {
            ranked.retain(|&i| !matches!(self.rows[i].meta, RowMeta::GotoHeading { .. }));
        }
        // FACETING picker under a real lens (strip index != 0, the All home): GROUP the
        // (fuzzy-matched) items into the lens's sections, in section order, preserving
        // the fuzzy rank WITHIN each section. `item_sections` records each row's section
        // (the faint header). The flat All home (and every non-faceting kind) keeps the
        // plain ranked list. GENERIC: the picker's own scheme supplies the sections +
        // the per-item bucketing — no picker-specific code here.
        let scheme = self.facet_scheme();
        if let Some(sc) = scheme.filter(|_| self.facet_lens != 0) {
            let mut items = Vec::with_capacity(ranked.len());
            let mut sections = Vec::with_capacity(ranked.len());
            for sect in sc.strip[self.facet_lens].sections {
                for &ci in &ranked {
                    let row = &self.rows[ci];
                    // OPT-OUT faceting: an item with `None` on this lens yields `None`
                    // here, matching no section, so it is omitted from the lens (still
                    // reachable under All). Only `Some(section)` items are placed. The
                    // bucket sees the accept string PLUS the universal dir/git flags
                    // (the file pickers' Folders / Files / Git lenses key off them).
                    let fi = crate::facets::FacetItem {
                        accept: &row.accept,
                        is_dir: row.is_dir,
                        is_git: row.git,
                        // Command palette's Recent lens: reuse the recency tier vec.
                        recent: self.recent.contains(&ci),
                        // Go-to's Headings lens: this row is an appended doc heading.
                        heading: matches!(row.meta, RowMeta::GotoHeading { .. }),
                        // History's Session / Today lenses: the per-row stamp + the
                        // picker-global reference clocks (all `None` headless → inert).
                        ts: match row.meta {
                            RowMeta::History { ts, .. } => Some(ts),
                            _ => None,
                        },
                        now: self.facet_now,
                        session_start: self.facet_session_start,
                    };
                    if (sc.bucket)(fi, self.facet_lens) == Some(*sect) {
                        items.push(ci);
                        sections.push((*sect).to_string());
                    }
                }
            }
            self.items = items;
            self.item_sections = sections;
        } else {
            self.item_sections = vec![String::new(); ranked.len()];
            self.items = ranked;
        }
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        self.scroll_to_selected();
        // DIFF-AS-PREVIEW: the row set changed under the highlight — whatever
        // version is now selected gets a fresh transcript, scrolled to its top.
        self.diff_scroll = 0;
    }

    /// THEME picker: the SECTION label for each filtered row, in the same order as
    /// [`Self::item_strings`] — the faint group header a row sits under (empty under
    /// All / for non-theme kinds). Surfaced to the render pipeline + sidecar so the
    /// grouping is drawable AND agent-verifiable.
    pub fn item_sections(&self) -> Vec<String> {
        self.item_sections.clone()
    }

    /// Insert a char at the query's caret and refilter. A query edit re-ranks the
    /// list, so the selection + scroll reset to the TOP (the best match).
    pub fn push(&mut self, c: char) {
        self.query.insert(c);
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    /// Backspace the query (deletes the char BEFORE its caret) and refilter.
    pub fn pop(&mut self) {
        self.query.delete_back();
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    /// ⌥⌫ (M-Backspace / DeleteWordBackward): remove the trailing WORD from the
    /// query — one whitespace/punct run + one word — then refilter. Routes through
    /// [`TextBox::delete_word_back`], the SAME word-DELETE boundary rule the
    /// document buffer's own word-delete uses (`crate::buffer::
    /// word_delete_backward_boundary`), so ⌥⌫ means the same thing in the palette
    /// as in the text. A NO-OP on an empty query (nothing to remove).
    pub fn pop_word(&mut self) {
        self.query.delete_word_back();
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    /// ITEM 10 — word MOTION right (Ctrl/Opt-Right while typing a filter):
    /// moves the query's caret WITHOUT editing the text or refiltering — plain
    /// L/R stay lens/descend/list (`actions/overlay_nav.rs`'s own gate); only
    /// word-motion reaches the caret at all, since arrows are claimed. Shares
    /// [`crate::buffer::word_forward_boundary`] with the document's own M-f —
    /// the DISTINCT rule from `pop_word`'s word-DELETE boundary above.
    pub fn query_word_right(&mut self) {
        self.query.word_right();
    }

    /// ITEM 10 — word MOTION left, the mirror of [`Self::query_word_right`].
    pub fn query_word_left(&mut self) {
        self.query.word_left();
    }

    /// The per-kind visible ROW CAP (delegates to [`OverlayKind::window_rows`], the ONE
    /// owner). Both the scroll math here AND the pipeline's drawn window (via
    /// [`crate::render::ViewState::overlay_window_rows`]) read the same value, so the
    /// highlighted / hovered / drawn rows can never disagree.
    pub fn window_rows(&self) -> usize {
        self.kind.window_rows()
    }

    /// Scroll the window the MINIMUM needed so `selected` sits within
    /// `[scroll, scroll + window_rows)`, then clamp so the final page never shows a
    /// blank tail. Called after any keyboard move / refilter — NEVER on a hover.
    pub(super) fn scroll_to_selected(&mut self) {
        let window = self.window_rows();
        if window == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + window {
            self.scroll = self.selected + 1 - window;
        }
        let max_top = self.items.len().saturating_sub(window);
        if self.scroll > max_top {
            self.scroll = max_top;
        }
    }

    /// Move the selection by `delta` rows, clamped to the visible item range, then
    /// scroll the window to keep the new selection visible (the keyboard ↑/↓ + PgUp/Dn
    /// path). The WHEEL rides this too, so a wheel notch advances the list exactly like
    /// an arrow press.
    pub fn move_sel(&mut self, delta: isize) {
        if self.items.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        let n = self.items.len() as isize;
        let mut s = self.selected as isize + delta;
        if s < 0 {
            s = 0;
        }
        if s >= n {
            s = n - 1;
        }
        self.selected = s as usize;
        self.scroll_to_selected();
        // DIFF-AS-PREVIEW: a selection move means a NEW transcript — top it out.
        self.diff_scroll = 0;
    }

    /// JUMP the selection to the FIRST visible item (the Home/End-in-picker jump — see
    /// [`crate::actions::overlay_nav::overlay_intercept`]'s LineStart/BufferStart arm),
    /// then scroll the window to it. A saturating counterpart to [`Self::move_sel`] that
    /// can't over/underflow on a huge delta; an empty list floors at 0. The ONE owner of
    /// "go to the top row", so the keyboard jump and any future caller land identically.
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.scroll_to_selected();
        self.diff_scroll = 0;
    }

    /// JUMP the selection to the LAST visible item (the End/Home-in-picker jump — the
    /// LineEnd/BufferEnd arm), then scroll the window to it. The ONE owner of "go to the
    /// bottom row"; an empty list floors at 0 (mirrors [`Self::move_sel`]'s empty guard).
    pub fn select_last(&mut self) {
        self.selected = self.items.len().saturating_sub(1);
        self.scroll_to_selected();
        self.diff_scroll = 0;
    }

    /// A HOVER re-highlights the row `target` ONLY when it is already within the current
    /// visible band `[scroll, scroll + window_rows)` (and is a real item). Returns whether
    /// the highlight moved. Crucially it NEVER touches `scroll`, so hovering the top /
    /// bottom edge — or anywhere off the visible rows — can't make the list auto-scroll:
    /// a hover highlights what's under the pointer, nothing more.
    pub fn hover_select(&mut self, target: usize) -> bool {
        let window = self.window_rows();
        let last = (self.scroll + window).min(self.items.len());
        if target >= self.scroll && target < last && target != self.selected {
            self.selected = target;
            // DIFF-AS-PREVIEW: hovering a different version previews ITS diff,
            // fresh from the top (the same reset every selection move takes).
            self.diff_scroll = 0;
            true
        } else {
            false
        }
    }

    /// ITEM 85 — THE REAL-MOTION GATE: [`Self::hover_select`]'s caller-facing door,
    /// gated on the pointer having GENUINELY moved since the last hover check. A
    /// theme-picker world jump can relocate every row under an otherwise-stationary
    /// pointer — a keyboard/wheel crossing re-anchors the card to the destination
    /// world's own rail (`reanchor`), a Pane↔Bars crossing changes the row pitch, a
    /// deferred font reshape settles into a different line height — and the very
    /// next `CursorMoved`, whether it is a real pixel of travel or a platform-
    /// synthesized duplicate at the IDENTICAL coordinates (a redraw-triggered
    /// spurious event), must not read that RELAYOUT as a pointer gesture.
    ///
    /// ITEM 106 — THE MOVEMENT-SLOP WIDENING: exact-pixel equality closed the
    /// zero-travel case (item 85's own hazard) but left a stationary hand's
    /// ordinary hardware jitter — or the case that actually bites, a list WINDOW
    /// SCROLLING under a genuinely resting pointer, so the ROW under an unmoved
    /// pixel changes on its own — free to steal a keyboard-driven selection the
    /// instant any real `CursorMoved`, even a single-physical-pixel one, arrived.
    /// The gate now compares `(px, py)` against [`Self::last_hover_px`] by
    /// SQUARED DISTANCE against [`HOVER_MOVE_SLOP_PX`] (the hover twin of
    /// `app::DRAG_ARM_SLOP_PX` — the identical "content relocating under a
    /// stationary pointer must not read as motion" hazard, the same physical-px
    /// budget) rather than bare inequality — a pixel of jitter no longer clears
    /// it, while any real travel PAST the slop still fires on that very event
    /// (no added latency, no debounce, no dead zone: the check is pure distance,
    /// never time — the headless replay path stays clock-free).
    ///
    /// The anchor [`Self::last_hover_px`] is deliberately STICKY below the slop:
    /// it is stamped forward only when this call actually reports a move (a cold
    /// start, or a real crossing) — never on a rejected sub-slop check. A run of
    /// tiny below-threshold checks therefore keeps accumulating distance from the
    /// SAME original anchor (so a genuinely slow drag still crosses the slop and
    /// fires the moment its TOTAL travel does, never hiding indefinitely below
    /// the noise floor by re-basing itself every micro-step — the same "fixed
    /// until armed" shape `App::exceeds_drag_slop`'s own `press` anchor uses).
    ///
    /// `hit` is the row the CALLER already resolved under `(px, py)` (a plain
    /// injected value, not a pipeline call — keeps this pure/unit-testable).
    /// Returns whether the highlight actually moved, mirroring
    /// [`Self::hover_select`]'s own return. See [`Self::arm_hover_baseline`] for
    /// the OTHER half of item 106 — re-anchoring this gate at KEYBOARD-action
    /// time, not just at the last hover check.
    pub fn hover_at(&mut self, px: f32, py: f32, hit: Option<usize>) -> bool {
        let moved = match self.last_hover_px {
            None => true,
            Some((lx, ly)) => {
                let dx = px - lx;
                let dy = py - ly;
                dx * dx + dy * dy > HOVER_MOVE_SLOP_PX * HOVER_MOVE_SLOP_PX
            }
        };
        if !moved {
            return false;
        }
        self.last_hover_px = Some((px, py));
        match hit {
            Some(idx) => self.hover_select(idx),
            None => false,
        }
    }

    /// ITEM 106 — THE KEYBOARD-BASELINE STAMP: re-anchors [`Self::hover_at`]'s
    /// movement-slop reference point to the pointer's CURRENT physical position,
    /// WITHOUT hit-testing or touching `selected` — the keyboard-driven twin of a
    /// real hover's own bookkeeping. Called by the shared keyboard-dispatch seam
    /// (`App::apply` live, `ReplaySession::apply_chord` headless) after EVERY
    /// keyboard-driven action while this overlay is open, so a subsequent hover
    /// check measures the pointer's travel since THAT keyboard action, never
    /// since a possibly stale, much-earlier hover position a scrolled/relaid-out
    /// list has since reassigned meaning to (a `None` baseline — nothing hovered
    /// yet at all — would otherwise treat literally the pointer's very first,
    /// resting-hand incidental `CursorMoved` as unconditional real motion, per
    /// [`Self::hover_at`]'s cold-start rule, and hand the keyboard's selection
    /// straight to whatever row a motionless pointer happens to sit over).
    /// `OverlayState` does not own a live pointer position itself (`App` does),
    /// so the caller supplies it — mirroring how `hit` is always caller-resolved
    /// into `hover_at` rather than this state reaching out for it.
    pub fn arm_hover_baseline(&mut self, px: f32, py: f32) {
        self.last_hover_px = Some((px, py));
    }

    /// ITEM 52 — RE-STAMP the card's frozen [`Self::align`] to the CURRENTLY-active
    /// world's own anchor. Called on a DELIBERATE selection crossing (keyboard nav,
    /// wheel, page/jump moves) AFTER [`crate::actions::preview_overlay`] has made the
    /// highlighted world active, so an open THEME picker SNAPS its card into the
    /// destination world's own left/center/right rail — choosing a world drops you
    /// inside it (the standing law; SUPERSEDES item 45's summon-time freeze for a
    /// deliberate move). PASSIVE pointer hover never calls this, so sweeping the
    /// pointer down the rows re-tints every world WITHOUT starting a spatial chase
    /// (the item-45 freeze still holds the card put through a hover). A NO-OP for
    /// every non-Theme picker: the active world can't move under them, so
    /// [`crate::render::effective_card_anchor`] returns the same anchor it froze at
    /// summon. It reads the SAME [`crate::render::effective_card_anchor`] owner the
    /// summon freeze does, so a keyboard crossing and a fresh summon into the same
    /// world resolve to the identical rail.
    pub fn reanchor(&mut self) {
        if self.kind == OverlayKind::Theme {
            self.align = crate::render::effective_card_anchor();
        }
    }

    /// The corpus index currently highlighted (into `rows`), or `None` when no item
    /// matches.
    pub fn selected_corpus_index(&self) -> Option<usize> {
        self.items.get(self.selected).copied()
    }

    /// The document LINE the highlighted HEADING row jumps to (Go-to's Headings lens),
    /// or `None` when no item matches / the row carries no line. Read only when the
    /// highlighted row IS a heading ([`Self::selected_is_heading`]).
    pub fn selected_line(&self) -> Option<usize> {
        let i = self.selected_corpus_index()?;
        match self.rows.get(i)?.meta {
            RowMeta::GotoHeading { line } => Some(line),
            _ => None,
        }
    }

    /// True when the highlighted Go-to row is a document HEADING (the Headings lens),
    /// so the accept JUMPS to [`Self::selected_line`] instead of opening a file. `false`
    /// for an ordinary file row and every non-Go-to picker.
    pub fn selected_is_heading(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| {
                matches!(
                    self.rows.get(i).map(|r| &r.meta),
                    Some(RowMeta::GotoHeading { .. })
                )
            })
            .unwrap_or(false)
    }

    /// True when the highlighted Spell row is the appended "Add '<word>' to
    /// dictionary" affordance (`RowMeta::SpellAdd`), so the accept ADDS the word to
    /// the personal dictionary instead of replacing it with a suggestion. `false`
    /// for a suggestion row and every non-Spell picker. The word to add is
    /// [`Self::add_word`].
    pub fn selected_is_add_to_dictionary(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| matches!(self.rows.get(i).map(|r| &r.meta), Some(RowMeta::SpellAdd)))
            .unwrap_or(false)
    }

    /// The RESTORE id of the highlighted history row (History only), or `None` when
    /// no item matches / this isn't a history picker / an empty history (no rows to
    /// restore). Enter maps this to a restore.
    pub fn selected_history_id(&self) -> Option<&str> {
        let i = self.selected_corpus_index()?;
        match &self.rows.get(i)?.meta {
            RowMeta::History { id, .. } if !id.is_empty() => Some(id.as_str()),
            _ => None,
        }
    }

    /// The caret LOOK the highlighted row selects (Caret picker only), or `None`
    /// when no item matches or this isn't a caret picker. Maps the highlighted row's
    /// label back to its [`crate::caret::CaretMode`] via [`CaretMode::from_label`].
    pub fn selected_caret_mode(&self) -> Option<crate::caret::CaretMode> {
        if self.kind != OverlayKind::Caret {
            return None;
        }
        self.selected_value()
            .and_then(crate::caret::CaretMode::from_label)
    }

    /// The RAW corpus string currently highlighted (the accept value), or `None`
    /// when no item matches.
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_corpus_index()
            .map(|i| self.rows[i].accept.as_str())
    }

    /// True when the highlighted entry is a directory (Browse: Enter descends).
    pub fn selected_is_dir(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| self.rows[i].is_dir)
            .unwrap_or(false)
    }

    /// The DISPLAY string for corpus entry `i`: the raw value plus a trailing
    /// `/` for a directory. A git repo is marked NOT here but by a dim `"git"` tag
    /// in the row's SECONDARY (right) column (see [`Self::item_git_tags`]), so the
    /// primary cell stays the clean folder name; the accept value is always the raw
    /// corpus string.
    fn display_of(&self, i: usize) -> String {
        let row = &self.rows[i];
        // ASSET CLEANER: the corpus holds the root-relative PATH (the accept/trash key
        // + fuzzy corpus, so typing a folder narrows), but the primary cell shows just
        // the leaf FILE NAME — its parent dir rides the secondary column. Every other
        // picker displays its raw corpus value.
        if self.kind == OverlayKind::Assets {
            let rel = &row.accept;
            return rel.rsplit('/').next().unwrap_or(rel).to_string();
        }
        // THE UNION ROUND: a settings row (appended to the Command palette's
        // corpus by `attach_settings_rows`) draws the `§ ` marker glyph before its
        // name — `crate::overlay::row_split` recognizes the SAME prefix constant
        // and mutes it, exactly like a file row's directory prefix.
        if matches!(row.meta, RowMeta::CommandSetting { .. }) {
            return format!("{}{}", OverlayKind::SETTINGS_MARKER_PREFIX, row.accept);
        }
        // ITEM 11's UNIFIED LIST: a Go-to HEADING row (appended after the file rows
        // by `attach_headings`) draws the `❡ ` marker glyph before its (already
        // depth-indented) title — the mirror-image of the settings marker above —
        // so it reads apart from a file row at a glance once the default `All` list
        // mixes both kinds together.
        if matches!(row.meta, RowMeta::GotoHeading { .. }) {
            return format!("{}{}", OverlayKind::HEADING_MARKER_PREFIX, row.accept);
        }
        let mut s = row.accept.clone();
        if row.is_dir {
            s.push('/');
        }
        s
    }

    /// The filtered DISPLAY strings, top-to-bottom (for rendering AND the
    /// sidecar). Directories carry a trailing `/`; a git repo's marker rides the
    /// SECONDARY column ([`Self::item_git_tags`]), not the name.
    pub fn item_strings(&self) -> Vec<String> {
        self.items.iter().map(|&i| self.display_of(i)).collect()
    }

    /// The filtered git-repo TAGS, in the same row order as [`Self::item_strings`]:
    /// a dim `"git"` for a row that is itself a git repo, `""` otherwise. This is
    /// the Project / Browse pickers' SECONDARY (right) column — the same recessive
    /// column the command palette uses for chords and go-to for edit times, so the
    /// tag YIELDS first under width pressure ([`crate::render::rowlayout`]). Returns
    /// an EMPTY vec when NO row is a git repo, so a git-free listing keeps no
    /// secondary column at all (byte-identical to a plain picker). For a picker kind
    /// that never marks git (theme / command / …) every flag is false → empty vec.
    pub fn item_git_tags(&self) -> Vec<String> {
        if !self.items.iter().any(|&i| self.rows[i].git) {
            return Vec::new();
        }
        self.items
            .iter()
            .map(|&i| {
                if self.rows[i].git {
                    "git".to_string()
                } else {
                    String::new()
                }
            })
            .collect()
    }

    /// The calm EMPTY-STATE line to show when NO rows match — a QUERY that filtered
    /// everything out reads the universal "no matches"; an empty CORPUS reads the
    /// per-kind [`OverlayKind::empty_corpus_message`] ("no history yet", "no
    /// suggestions", …). The ONE owner of the empty-state text, shared by the render
    /// message row AND the sidecar `overlay.empty` field so pixels + sidecar agree.
    pub fn empty_message(&self) -> String {
        if !self.query.is_empty() {
            return "no matches".to_string();
        }
        // A REFINEMENT lens (a strip index past the flat `All` home) that filtered
        // the corpus to empty reads its own calm line — e.g. the Go-to Recent lens's
        // "no recent files yet" — distinct from a genuinely empty corpus.
        if let Some(lens) = self.active_facet_id() {
            if let Some(msg) = self.kind.empty_lens_message(lens) {
                return msg.to_string();
            }
        }
        self.kind.empty_corpus_message().to_string()
    }

    /// The empty-state message to DRAW, or `None` when the picker has rows. `Some`
    /// exactly when `items` is empty — the render path then draws one dim,
    /// non-selectable message row (styled like the foot hint), and since `items` is
    /// empty every accept (`selected_value`/`selected_corpus_index`) already returns
    /// `None`, so Enter on the empty state is a calm no-op with no extra guard.
    pub fn empty_notice(&self) -> Option<String> {
        if self.items.is_empty() {
            Some(self.empty_message())
        } else {
            None
        }
    }

    /// The filtered SECONDARY-column labels, in the same row order as
    /// [`item_strings`] (Command palette chords, look/variant descriptions,
    /// setting values, changed-counts, … — empty for a kind that draws no
    /// secondary column). Lets the render + sidecar show each row's secondary
    /// cell without re-deriving it.
    pub fn item_bindings(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|&i| self.rows[i].secondary.clone())
            .collect()
    }

    /// ITEM 94 — the per-row RAIL FRACTION (0..1), in the same row order as
    /// [`item_strings`]: `Some(frac)` for a range row, `None` for every ordinary
    /// row. EMPTY when NO visible row carries a rail, so every other picker feeds
    /// the renderer nothing at all (byte-identical there).
    ///
    /// The fraction is derived — never stored — through the ONE spec owner
    /// (`settings::range_spec` -> `RangeSpec::frac_of_step`), so the drawn thumb
    /// sits exactly where the authored mapping (linear or logarithmic) puts the
    /// row's own quantized step.
    pub fn item_range_fracs(&self) -> Vec<Option<f32>> {
        if !self.items.iter().any(|&i| self.rows[i].range.is_some()) {
            return Vec::new();
        }
        self.items
            .iter()
            .map(|&i| {
                let cell = self.rows[i].range?;
                let spec = crate::settings::range_spec(cell.id)?;
                Some(spec.frac_of_step(cell.step))
            })
            .collect()
    }

    /// ITEM 94 — the RANGE CELL of the row at `items` index `i` (the index a
    /// pointer hit-test returns), or `None` when that row carries no rail. The one
    /// door the pointer path resolves "did this click land on a range row?" through.
    pub fn range_of_item(&self, i: usize) -> Option<RangeCell> {
        self.rows.get(*self.items.get(i)?)?.range
    }

    /// ITEM 94 — the HIGHLIGHTED row's range cell, or `None` when the selection is
    /// not a range row. The keyboard step's gate: Left/Right only claim the key
    /// when this is `Some` (otherwise the faceting lens keeps them, exactly as before).
    pub fn selected_range(&self) -> Option<RangeCell> {
        self.rows.get(self.selected_corpus_index()?)?.range
    }

    /// ITEM 94 — write the highlighted range row's new STEP and value READOUT in
    /// place (keyboard step / pointer scrub). Mirrors the value straight into the
    /// still-open menu's own cell, so the number and the thumb move together in the
    /// same frame — live AND in a headless `--keys` replay, which has no App to
    /// refresh the overlay afterwards. A no-op when the selection carries no rail.
    pub fn set_selected_range(&mut self, step: u16, readout: String) {
        let Some(ci) = self.selected_corpus_index() else {
            return;
        };
        let Some(row) = self.rows.get_mut(ci) else {
            return;
        };
        let Some(cell) = row.range.as_mut() else {
            return;
        };
        cell.step = step;
        row.secondary = readout;
    }

    /// The filtered relative-time LABELS, in the same row order as [`item_strings`]
    /// (go-to picker only; empty for every other kind and in headless capture). A
    /// HEADING row (see [`RowMeta::GotoHeading`]) carries no mtime — since item 11's
    /// unified `All` list mixes heading rows in among file rows, its cell reads the
    /// constant `"heading"` KIND HINT instead, the rowlayout SECONDARY-cell
    /// disambiguator that tells a heading row apart from a file row at a glance (a
    /// file row's cell is its relative edit time live, or blank in headless where
    /// mtime is never read).
    pub fn item_times(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|&i| match &self.rows[i].meta {
                RowMeta::GotoHeading { .. } => "heading".to_string(),
                RowMeta::GotoFile { time } => time.clone(),
                _ => String::new(),
            })
            .collect()
    }
}
