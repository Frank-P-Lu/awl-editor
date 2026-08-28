use super::{OverlayKind, OverlayState, RangeCell, RowMeta};

pub(super) const HOVER_MOVE_SLOP_PX: f32 = crate::app::DRAG_ARM_SLOP_PX;

impl OverlayState {
    /// The per-row SECTION labels the grouped card draws as faint headers above
    /// each bucket.
    ///
    /// EMPTY for a summoned WORKSPACE. A workspace names the active
    /// category once, in its navigation rail, and repeating it as a header over
    /// the only bucket the rail lets through is the same fact twice. Returning
    /// nothing here (rather than teaching the renderer to skip headers) keeps one
    /// answer to "what sections does this card show", so the drawn rows, the row
    /// plan and the sidecar's `sections` array cannot disagree.
    pub fn item_sections(&self) -> Vec<String> {
        if self.workspace_shape().is_some() {
            return Vec::new();
        }
        self.item_sections.clone()
    }

    pub fn push(&mut self, c: char) {
        self.query.insert(c);
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    pub fn pop(&mut self) {
        self.query.delete_back();
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    pub fn pop_word(&mut self) {
        self.query.delete_word_back();
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    pub fn query_word_right(&mut self) {
        self.query.word_right();
    }

    pub fn query_word_left(&mut self) {
        self.query.word_left();
    }

    /// Whether the query caret sits AT its own resting position — the END of
    /// the typed text, where every ordinary keystroke (`push`/`pop`) leaves
    /// it, and where a fresh/empty field starts. The list-nav overloads on
    /// `ForwardChar`/`BackwardChar`/`LineStart`/`LineEnd` (lens cycle / folder
    /// descend-ascend / row jump — see `actions::overlay_nav::navigate_overlay`)
    /// stay live exactly here; once the caret sits anywhere else — a click, a
    /// drag, or a word-step landing short of the end — those same keys fall
    /// through to ordinary text motion instead, and reaching the end again
    /// (an End, or a char-step that lands there) restores the list-nav
    /// reading on the very next keypress.
    pub fn query_at_rest(&self) -> bool {
        self.query.caret() == self.query.text().chars().count()
    }

    pub fn query_char_left(&mut self) {
        self.query.char_left();
    }

    pub fn query_char_right(&mut self) {
        self.query.char_right();
    }

    /// Text-field Home: the query caret to its own start (char 0), never the
    /// list's first row — the mid-query half of the [`Self::query_at_rest`]
    /// split.
    pub fn query_home(&mut self) {
        self.query.set_caret(0);
    }

    /// Text-field End: the query caret to its own end, never the list's last
    /// row — the mid-query half of the [`Self::query_at_rest`] split.
    pub fn query_end(&mut self) {
        self.query.set_caret(self.query.text().chars().count());
    }

    /// Place the query caret at an arbitrary CHAR index (clamped by
    /// [`TextBox::set_caret`]) — the one door a pointer click or drag uses,
    /// mirroring the click-to-place the rename/link/keep/value sub-editors'
    /// own `TextBox` already supports for a future caller. RENAME is that
    /// caller today: while a rename edit is active, `query` is a MIRROR (see
    /// [`super::rename_edit::RenameEdit`]'s doc via `OverlayState::rename_edit_mirror`),
    /// so the click has to land on `rename_edit.input` first or the next
    /// keystroke's mirror snaps the caret straight back.
    pub fn query_set_caret(&mut self, at: usize) {
        if self.rename_edit.is_some() {
            self.rename_edit_set_caret(at);
            return;
        }
        self.query.set_caret(at);
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
            self.diff_scroll = 0;
            true
        } else {
            false
        }
    }

    /// THE REAL-MOTION GATE: [`Self::hover_select`]'s caller-facing door,
    /// gated on the pointer having GENUINELY moved since the last hover check. A
    /// theme-picker world jump can relocate every row under an otherwise-stationary
    /// pointer — a keyboard/wheel crossing re-anchors the card to the destination
    /// world's own rail (`reanchor`), a Pane↔Bars crossing changes the row pitch, a
    /// deferred font reshape settles into a different line height — and the very
    /// next `CursorMoved`, whether it is a real pixel of travel or a platform-
    /// synthesized duplicate at the IDENTICAL coordinates (a redraw-triggered
    /// spurious event), must not read that RELAYOUT as a pointer gesture.
    ///
    /// Ignore stationary-pointer layout changes until travel exceeds the shared slop.
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
    /// until armed" shape `PointerInput::exceeds_drag_slop`'s own press anchor uses).
    ///
    /// `hit` is the row the CALLER already resolved under `(px, py)` (a plain
    /// injected value, not a pipeline call — keeps this pure/unit-testable).
    /// Returns whether the highlight actually moved, mirroring
    /// [`Self::hover_select`]'s own return. See [`Self::arm_hover_baseline`] for
    /// the OTHER half — re-anchoring this gate at KEYBOARD-action
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

    /// THE KEYBOARD-BASELINE STAMP: re-anchors [`Self::hover_at`]'s
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

    /// RE-STAMP the card's frozen [`Self::align`] to the CURRENTLY-active
    /// world's own anchor. Called on a DELIBERATE selection crossing (keyboard nav,
    /// wheel, page/jump moves) AFTER [`crate::actions::preview_overlay`] has made the
    /// highlighted world active, so an open THEME picker SNAPS its card into the
    /// destination world's own left/center/right rail — choosing a world drops you
    /// inside it (the standing law; it supersedes summon-time freeze for a
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

    pub fn selected_corpus_index(&self) -> Option<usize> {
        self.items.get(self.selected).copied()
    }

    pub fn selected_line(&self) -> Option<usize> {
        let i = self.selected_corpus_index()?;
        match self.rows.get(i)?.meta {
            RowMeta::GotoHeading { line } | RowMeta::GotoLine { line } => Some(line),
            _ => None,
        }
    }

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

    /// Go to Line's own accept-path gate, the numeric sibling of
    /// [`Self::selected_is_heading`]: `true` only when the highlighted row is
    /// the synthesized line-jump row (never a heading, file, or folder row
    /// that merely happens to carry a numeric-looking label).
    pub fn selected_is_line_jump(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| {
                matches!(
                    self.rows.get(i).map(|r| &r.meta),
                    Some(RowMeta::GotoLine { .. })
                )
            })
            .unwrap_or(false)
    }

    pub fn selected_is_goto_folder(&self) -> bool {
        self.selected_corpus_index()
            .and_then(|i| self.rows.get(i))
            .is_some_and(|row| matches!(row.meta, RowMeta::GotoFolder))
    }

    pub fn selected_is_add_to_dictionary(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| matches!(self.rows.get(i).map(|r| &r.meta), Some(RowMeta::SpellAdd)))
            .unwrap_or(false)
    }

    pub fn selected_history_id(&self) -> Option<&str> {
        let i = self.selected_corpus_index()?;
        match &self.rows.get(i)?.meta {
            RowMeta::History { id, .. } if !id.is_empty() => Some(id.as_str()),
            _ => None,
        }
    }

    pub fn selected_caret_mode(&self) -> Option<crate::caret::CaretMode> {
        if self.kind != OverlayKind::Caret {
            return None;
        }
        self.selected_value()
            .and_then(crate::caret::CaretMode::from_label)
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.selected_corpus_index()
            .map(|i| self.rows[i].accept.as_str())
    }

    pub fn selected_is_dir(&self) -> bool {
        self.selected_corpus_index()
            .map(|i| self.rows[i].is_dir)
            .unwrap_or(false)
    }

    fn display_of(&self, i: usize) -> String {
        super::build::row_display(self.kind, &self.rows[i], self.browse_dir.as_deref())
    }

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
        if let Some(lens) = self.active_facet_id()
            && let Some(msg) = self.kind.empty_lens_message(lens)
        {
            return msg.to_string();
        }
        self.kind.empty_corpus_message().to_string()
    }

    pub fn empty_notice(&self) -> Option<String> {
        if self.items.is_empty() {
            Some(self.empty_message())
        } else {
            None
        }
    }

    pub fn item_bindings(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|&i| self.rows[i].secondary.clone())
            .collect()
    }

    /// The per-row RAIL FRACTION (0..1), in the same row order as
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

    pub fn range_of_item(&self, i: usize) -> Option<RangeCell> {
        self.rows.get(*self.items.get(i)?)?.range
    }

    pub fn selected_range(&self) -> Option<RangeCell> {
        self.rows.get(self.selected_corpus_index()?)?.range
    }

    /// Write the highlighted range row's new STEP and value READOUT in
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
    /// HEADING row (see [`RowMeta::GotoHeading`]) carries no mtime because
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
                RowMeta::GotoLine { .. } => "line".to_string(),
                RowMeta::GotoFolder => "folder".to_string(),
                RowMeta::FolderChooser => String::new(),
                RowMeta::GotoFile { time } => time.clone(),
                _ => String::new(),
            })
            .collect()
    }
}
