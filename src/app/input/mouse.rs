use super::wheel::*;
use crate::app::input::DragGranularity;
use crate::app::*;

impl App {
    pub(in crate::app) fn pointer_over_writing_column(&self) -> bool {
        self.frame.gpu().is_some_and(|gpu| {
            gpu.pipeline
                .over_writing_column(self.input.pointer.cursor_px.0)
        })
    }

    pub(in crate::app) fn hit_test_line_col(&self) -> (usize, usize) {
        let (px, py) = self.input.pointer.cursor_px;
        let gpu = self
            .frame
            .gpu()
            .expect("pointer hit testing requires the live GPU text pipeline");
        gpu.pipeline.hit_test_scroll(px, py, self.document.scroll())
    }

    /// The char index under the pointer — every click, drag endpoint, right-press
    /// and ⌘-click link probe resolves here, and here only. The pixel half is the
    /// hit test above; the document half (fold-space line remap + cluster-boundary
    /// snap) belongs to [`crate::buffer::Buffer::hit_char`], which is where the
    /// rule is stated and unit-tested without a GPU.
    pub(in crate::app) fn hit_test_char(&self) -> usize {
        let (line, col) = self.hit_test_line_col();
        self.document.buffer().hit_char(line, col)
    }

    fn fold_affordance_at_pointer(&self) -> Option<usize> {
        if !self.document.buffer().has_folds() {
            return None;
        }
        let (line, col) = self.hit_test_line_col();
        self.document.buffer().fold_tail_hit(line, col)
    }

    /// THE FOLD CHEVRON's own click/hover target: if the pointer lands on a
    /// currently-REVEALED chevron (any foldable heading, expanded OR collapsed —
    /// [`crate::render::TextPipeline::fold_chevron_hit`]), the FULL-document heading
    /// line to toggle; else `None`. `None` with no GPU pipeline up (headless has no
    /// live pointer to begin with). The ONE hit-test [`Self::sync_cursor_icon`]'s
    /// pointing-hand decision also reads, so hover, cursor, and click can never
    /// disagree on where the target is.
    pub(in crate::app) fn fold_chevron_at_pointer(&self) -> Option<usize> {
        let (px, py) = self.input.pointer.cursor_px;
        let filtered = self.frame.gpu()?.pipeline.fold_chevron_hit(px, py)?;
        Some(self.document.buffer().visible_line_to_full(filtered))
    }

    /// Multi-click detection: same spot, within the time window (`MULTICLICK_MS`) —
    /// bump the running click count (wrapping 1/2/3) and stamp `last_click_time` /
    /// `last_click_px` for the NEXT press, then return the now-current count.
    /// Shared by a normal document press ([`Self::on_press`]) and a press on the
    /// draggable page-column edge ([`Self::begin_page_resize_if_hovering`]) so a
    /// double-click reads the same wherever the pointer lands — one owner, so the
    /// two can't drift apart on what counts as "a double-click".
    pub(in crate::app) fn bump_click_count(&mut self) -> u32 {
        self.input.pointer.bump_click_count(self.frame.now())
    }

    /// THE PHANTOM-SELECTION-CLICK FIX: whether the pointer has traveled far
    /// enough from the press position (`press`) to the current position
    /// (`current`, both PHYSICAL px like `cursor_px`) to treat this `CursorMoved`
    /// as the start of a REAL text-selection drag, rather than pointer jitter or a
    /// WYSIWYG reveal reflow relocating glyphs under an otherwise-stationary
    /// pointer (concealed markup regaining its real advance the instant the caret
    /// lands on that line — which used to look identical to a drag because the old
    /// code re-hit-tested on every move regardless of actual travel, so the
    /// hit-test RESULT drifting was mistaken for pointer motion). Pure
    /// squared-distance compare against [`DRAG_ARM_SLOP_PX`] (no sqrt needed).
    /// Deliberately answers ONLY from pixel geometry — never from a hit-test
    /// result — so it can never be fooled by content reflowing under a still
    /// pointer. See `App::drag_armed`'s doc in `app.rs` for the wiring.
    /// Handle a primary-button press inside the writing column: hit-test, set the
    /// anchor, and (for double / triple clicks) select the word / line under the
    /// cursor. A press in either PAGE MARGIN is swallowed before hit-testing — the
    /// gutter is orientation, not document text, and `hit_test` deliberately clamps
    /// out-of-column x positions to a line endpoint. Without this gate, clicking the
    /// gutter therefore selected text at the page's left edge. A drag that STARTED in
    /// the column may still extend into a margin through [`Self::on_drag`].
    ///
    /// `shift` is
    /// whether Shift was held at press time: a SHIFT-CLICK extends the existing
    /// selection (the standard gesture everywhere — TextEdit/Xcode/browsers/…)
    /// instead of starting a fresh one, so it must never `clear_mark`.
    pub(in crate::app) fn on_press(&mut self, shift: bool, over_writing_column: bool) {
        if !over_writing_column {
            return;
        }
        // FOLD CHEVRON CLICK: a plain click on a REVEALED fold chevron
        // toggles that heading's section EITHER direction (fold an expanded heading,
        // unfold a collapsed one) through the ONE owner
        // (`Buffer::toggle_fold_at_line`) — never starts a text selection or a drag.
        // Checked FIRST: the chevron's narrow left-margin lane never overlaps the
        // tail's own hit region (past the heading text, to the right), so order is
        // only for clarity, not correctness.
        if !shift && let Some(h) = self.fold_chevron_at_pointer() {
            self.document.seal_undo_group();
            self.document.toggle_fold_at_line(h);
            self.document.clear_mark();
            return;
        }
        // CLICK-TO-EXPAND: a plain click on a collapsed heading's "… N lines"
        // tail (past the heading text) OPENS that fold and parks the caret on the
        // heading — it never starts a text selection or a drag (returns before
        // `self.input.pointer.dragging`). The caller's `sync_view(true)` repaints the now-expanded
        // section. A shift-click is left to extend a selection as usual; a click on
        // the heading TEXT (not the affordance) falls through to the normal caret
        // placement below.
        if !shift && let Some(h) = self.fold_affordance_at_pointer() {
            self.document.seal_undo_group();
            self.document.unfold_at(h);
            self.document.clear_mark();
            return;
        }
        let idx = self.hit_test_char();
        self.press_at_char(idx, shift);
    }

    /// Selection-state half of a document press after the live pipeline has
    /// resolved its shaped pixel position to a document character.
    pub(in crate::app) fn press_at_char(&mut self, idx: usize, shift: bool) {
        let click_count = self.bump_click_count();
        // A click is a non-edit gesture: seal the open undo group so text typed
        // after relocating the cursor is its own undo step.
        self.document.seal_undo_group();
        self.input.pointer.begin_text_drag();
        match click_count {
            1 if shift => {
                // SHIFT-CLICK: keep the mark if one is already active, else drop
                // it at the cursor's CURRENT position (before this click moves
                // it) — then move only the cursor to the hit point. Never
                // `clear_mark`; that's what a plain click is for. Double/triple
                // click arms are unaffected (shift only modifies the single-click
                // arm — a shift+double-click still lands here as click_count 1
                // relative to the NEW spot, since a shift-click is usually a
                // fresh spot rather than a same-spot repeat).
                self.input.pointer.drag_granularity = DragGranularity::Char;
                if self.document.buffer().anchor_char().is_none() {
                    self.document
                        .set_anchor(self.document.buffer().cursor_char());
                }
                self.document.set_cursor(idx);
                self.document.set_shift_selecting(true);
            }
            1 => {
                self.input.pointer.drag_granularity = DragGranularity::Char;
                self.document.set_cursor(idx);
                self.document.clear_mark();
                self.document.set_anchor(idx);
                self.document.set_shift_selecting(false);
            }
            2 => {
                self.input.pointer.drag_granularity = DragGranularity::Word;
                let (s, e) = self.document.buffer().word_bounds(idx);
                self.document.select_range(s, e);
            }
            _ => {
                self.input.pointer.drag_granularity = DragGranularity::Line;
                let (s, e) = self.document.buffer().line_bounds(idx);
                self.document.select_range(s, e);
            }
        }
        // REVEALED PLACEMENT (folds): a shift-click whose new selection spans a
        // collapsed section — or any click/word/line placement that lands on a
        // hidden row — routes through the ONE placement owner so the caret and every
        // selection endpoint stay visible. A cheap no-op unless a section is folded
        // (the fold-affordance click above already returned; it is a deliberate
        // unfold, not a placement). The caller's `sync_view(true)` repaints any
        // now-revealed section.
        self.document.reveal_placement();
    }

    /// Resolve an Outline row's hit-tested line (as
    /// `TextPipeline::outline_hit_line` returns it) to the RAW document line
    /// `jump_to_line` must receive. `outline_hit_line`'s line is FOLD-FILTERED
    /// space — the render shapes the fold-filtered document, and `outline_headings`
    /// is distilled from that filtered text (see its own doc) — so a heading sitting
    /// after an active fold elsewhere in the document reports an index shifted by
    /// every hidden line above it. `jump_to_line` (like Go-to's Headings lens, which
    /// jumps by the RAW `buffer.text()` parse) expects a raw document line, so the
    /// hit-tested line is remapped here through `visible_line_to_full` — the SAME
    /// owner `hit_test_char`/`fold_tail_hit` already route every other click-to-rope
    /// seam through. The identity when nothing is folded, so a no-fold click's target
    /// is byte-identical to before this fix. Split out of `outline_click` as its own
    /// method so this pure line-space remap is unit-testable without a live GPU hit
    /// test (`outline_click`'s pixel half is live-only).
    pub(in crate::app) fn outline_row_target_line(&self, filtered_line: usize) -> usize {
        self.document.buffer().visible_line_to_full(filtered_line)
    }

    /// CLICK-TO-JUMP on a persistent MARGIN OUTLINE row: hit-test the pointer against
    /// the outline's OWN row geometry (`TextPipeline::outline_hit_line`, which folds in
    /// the whole shown/hidden gate — off / non-page / non-md / too-narrow all return
    /// `None`) and, on a hit, jump the caret to that heading's line — the same
    /// `jump_to_line` the retired summoned Outline picker used. Returns whether the
    /// press landed on a row (so the caller skips the document press). A benign,
    /// user-approved navigation affordance (DESIGN.md outline amendment: "click-to-jump
    /// only") — NOT a resizable/focusable sidebar. Never fires while an overlay is open
    /// (its scrim owns the click first, handled upstream in `on_mouse_input`).
    pub(in crate::app) fn outline_click(&mut self) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let line = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.outline_hit_line(px, py, g.config.height));
        if let Some(line) = line {
            self.jump_to_line(self.outline_row_target_line(line));
            true
        } else {
            false
        }
    }

    /// CMD-CLICK activation: hit-test the char under the pointer. A footnote
    /// reference uses the shared folded-line jump; a markdown link hands its URL
    /// to the OS browser through the SAME
    /// [`App::follow_link`] owner the `C-c C-o` keyboard path uses (so the two can't
    /// drift). Returns whether a link was followed, so the caller can SWALLOW the
    /// press — never moving the caret / starting a selection. Reads only. The
    /// mouse-affordance half of the identity round's "⌘-click Follow link" (the
    /// keyboard chord stays too).
    pub(in crate::app) fn follow_link_at_pointer(&mut self) -> bool {
        let byte = self.document.buffer().char_to_byte(self.hit_test_char());
        let text = self.document.buffer().text();
        if let Some(line) = crate::markdown::footnote_target_at(&text, byte) {
            self.jump_to_line(line);
            true
        } else if let Some(url) = crate::markdown::link_at(&text, byte) {
            self.follow_link(&url);
            true
        } else {
            false
        }
    }

    pub(in crate::app) fn overlay_hover(&mut self) {
        let (px, py) = self.input.pointer.cursor_px;
        // `TextPipeline::resolve_overlay_hover` hit-tests THEN runs
        // `OverlayState::hover_at`'s REAL-MOTION + MOVEMENT-SLOP GATE: it
        // re-hit-tests + re-highlights ONLY when `(px, py)` travelled PAST the
        // slop since the last hover check (or the last keyboard action —
        // `App::apply`'s `arm_hover_baseline` stamp), so a world jump's own
        // re-layout (a reanchor, a Pane↔Bars row-pitch change, a settling font
        // reshape) OR a list window scrolling under an otherwise-STATIONARY
        // pointer can never synthesize a new selection on its own — every real
        // `CursorMoved`, jitter or a platform-synthesized duplicate at the
        // identical coordinates, funnels through the same gate. `hover_select`
        // itself still owns the visible-band + no-op checks; `hover_at` never
        // moves the scroll window either, so hovering the top/bottom edge can't
        // auto-scroll the list.
        let Some(gpu) = self.frame.gpu() else { return };
        let kind = match self.workspace_state.overlay_mut() {
            Some(ov) => {
                if !gpu.pipeline.resolve_overlay_hover(ov, px, py) {
                    return;
                }
                ov.kind
            }
            None => return,
        };
        // FLIGHT RECORDER / PROBE: a hover that got PAST the
        // movement-slop gate and re-highlighted a row. If a navigation input's
        // selection is being overwritten by a stationary pointer, this line lands
        // between the `apply` line and the `prepare_highlight` line that disagrees
        // with it — the STATIONARY-POINTER-TAKEOVER break, named where it happens.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            let sel = self.overlay_selection_probe();
            crate::probe::trace(format_args!(
                "hover_took_selection at=({px:.1},{py:.1}) {sel:?}"
            ));
        }
        let prev = crate::theme::active();
        if let Some(ov) = self.workspace_state.overlay() {
            // BARE preview — NOT `preview_move`: a passive HOVER re-tints the world but
            // must NOT re-anchor the card (no spatial chase under a wandering
            // pointer; the summon-time freeze holds the card put). Deliberate crossings
            // (keyboard nav, wheel) re-anchor; hover alone does not.
            crate::actions::preview_overlay(ov);
        }
        // A Theme preview mutated the process-global active world: re-tint the baked GPU
        // pipelines + window title so the hover previews it live, mirroring the theme
        // branch of `post_transition_effects` — colors instantly, the font reshape deferred
        // to the settle (`retint_theme_preview`), so sweeping the pointer down the
        // list costs one recolor per row, not one reshape storm per row.
        if kind == crate::overlay::OverlayKind::Theme {
            self.retint_theme_preview(prev);
        }
        self.sync_view(false);
        self.request_frame();
    }

    pub(in crate::app) fn overlay_wheel(&mut self, lines: f32) {
        let delta = -(lines.round() as isize); // wheel DOWN (lines < 0) advances (↓)
        if delta == 0 {
            return;
        }
        let kind = match self.workspace_state.overlay_mut() {
            Some(ov) => {
                ov.move_sel(delta);
                ov.kind
            }
            None => return,
        };
        let prev = crate::theme::active();
        if let Some(ov) = self.workspace_state.overlay_mut() {
            crate::actions::preview_move(ov);
        }
        // The wheel drives `move_sel` exactly like a keyboard press (it
        // is one of the "deliberate crossing" input classes, same as ↑/↓), so it
        // must re-anchor `hover_at`'s movement-slop gate to the pointer's CURRENT
        // resting position too — mirroring `App::apply`'s stamp after every
        // keyboard-driven action. This function bypasses `App::apply` entirely
        // (the wheel is dispatched straight from `on_mouse_wheel`), so without an
        // explicit stamp here a cold-start session (overlay opened by keyboard,
        // pointer never yet hovered a row, `last_hover_px` still `None`) would
        // leave a wheel-scrolled selection exposed to `hover_at`'s cold-start
        // rule: `None` reads ANY next pointer report — even an incidental
        // redraw-duplicate `CursorMoved` at the pointer's unmoved position — as
        // unconditional real motion, silently overriding the wheel's selection
        // with whatever row now sits under the stationary pointer.
        let (px, py) = self.input.resting_pointer().px();
        if let Some(ov) = self.workspace_state.overlay_mut() {
            ov.arm_hover_baseline(px, py);
        }
        if kind == crate::overlay::OverlayKind::Theme {
            self.retint_theme_preview(prev);
        }
        self.sync_view(false);
        self.request_frame();
    }

    /// A LEFT-CLICK while a picker is open, resolved against the overlay card:
    ///   * ON a candidate ROW → move the selection there and ACCEPT it — the exact
    ///     `Action::Newline` the keyboard's Enter runs, so a click opens the file /
    ///     runs the command / commits the theme / descends the folder identically
    ///     (one path, every kind).
    ///   * OUTSIDE the card rect → DISMISS the overlay, routed through the SAME
    ///     `Action::Cancel` Esc / C-g uses (so a Theme / Caret live preview reverts
    ///     too). Click-away-to-dismiss is GENERAL across every summoned overlay
    ///     (palette / pickers / spell / history / …) — the card rect + row hit-test
    ///     both come from the one kind-agnostic `overlay_geometry`.
    ///   * INSIDE the card but off a row (query line / foot hint) → SWALLOWED (the
    ///     picker stays modal; it never falls through to `on_press`, which would place
    ///     the document cursor beneath the card).
    ///     Always consumes the click while an overlay is open.
    ///
    /// Activate the overlay row under the press-time pointer position. Release
    /// position and cached hover state never choose the row.
    pub(in crate::app) fn overlay_click(&mut self, exit: &dyn schedule::Exit) {
        let (px, py) = self.input.pointer.cursor_px;
        let (row_hit, lens_hit, rail_hit, card) = self
            .frame
            .gpu()
            .map(|g| {
                (
                    g.pipeline.overlay_row_at(px, py),
                    g.pipeline.overlay_lens_at(px, py),
                    g.pipeline.workspace_rail_at(px, py),
                    g.pipeline.overlay_card_rect(),
                )
            })
            .unwrap_or((None, None, None, None));

        // Rail clicks use the same lens and focus transitions as keyboard entry.
        if let Some(rail_idx) = rail_hit {
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.set_facet_lens(rail_idx);
            }
            self.workspace_state.focus_workspace_detail();
            self.sync_view(true);
            self.request_frame();
            return;
        }

        // FACETED PICKER: a click on a LENS label switches the facet (keeping the
        // selection), then previews + re-tints — the pointing counterpart to LEFT/RIGHT.
        // Handled before the row hit-test (the strip sits above the rows, never overlaps).
        if let Some(lens_idx) = lens_hit {
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.set_facet_lens(lens_idx);
            }
            let prev = crate::theme::active();
            if let Some(ov) = self.workspace_state.overlay() {
                crate::actions::preview_overlay(ov);
            }
            self.retint_theme_preview(prev);
            self.sync_view(false);
            self.request_frame();
            return;
        }

        if self.begin_range_drag() {
            self.sync_view(true);
            self.request_frame();
            return;
        }

        if let Some(idx) = row_hit {
            // ON a row: ACCEPT through the shared apply path — byte-for-byte the same
            // as Enter on the highlighted row (open / run / commit / descend / replace).
            if let Some(ov) = self.workspace_state.overlay_mut()
                && idx < ov.items.len()
            {
                ov.selected = idx;
            }
            // A RANGE ROW'S LABEL SELECTS WITHOUT CHANGING. Every other
            // kind treats a row click as Enter; a range row must not, because its
            // Enter opens the modal numeric edit — so clicking the row's NAME (to
            // then use the arrows, or just to look) would hijack the keyboard. The
            // rail above is where a pointer changes the value; everywhere else on
            // the row is a plain selection.
            let is_range = self
                .workspace_state
                .overlay()
                .map(|ov| ov.range_of_item(idx).is_some())
                .unwrap_or(false);
            if is_range {
                self.sync_view(true);
                self.request_frame();
                return;
            }
            self.apply(Action::Newline, false, exit, crate::stats::Door::Chord);
        } else {
            let inside = card
                .map(|[x, y, w, h]| px >= x && px <= x + w && py >= y && py <= y + h)
                .unwrap_or(false);
            if inside {
                return;
            }
            self.apply(Action::Cancel, false, exit, crate::stats::Door::Chord);
        }
        self.sync_view(true);
        self.request_frame();
    }

    /// A LEFT-CLICK inside the summoned find/replace panel: CLICK-TO-SWITCH-FIELD
    /// (plus the `Aa` case toggle). A press on the `Aa` cell flips case sensitivity
    /// and re-anchors the caret; a press on the FIND row (off `Aa`) focuses the
    /// query (`editing_replacement = false`); a press on the REPLACE row focuses
    /// the replacement (`editing_replacement = true`) — the amber caret then rides
    /// the clicked field (Batch-1 fixed the replace caret-x, so focusing via click
    /// places it correctly). A press ELSEWHERE inside the card (the key-hint line,
    /// inter-row gaps) is a calm no-op — swallowed, never dismissing the search or
    /// moving the document cursor beneath the panel. Returns `true` when the press landed on/in the panel and
    /// was handled; `false` (off the card / panel down) lets the caller fall
    /// through to the normal document press. The find↔replace decision is the pure
    /// `TextPipeline::panel_hit` (unit-tested); this only wires the field state +
    /// redraw, mirroring the two focus doors `handle_search_key` already uses.
    pub(in crate::app) fn panel_click(&mut self) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let hit = self.frame.gpu().and_then(|g| g.pipeline.panel_hit(px, py));
        match hit {
            Some(crate::render::PanelHit::CaseToggle) => {
                let hay = self.document.buffer().text();
                let target = self.workspace_state.search_mut().map(|st| {
                    st.toggle_case(&hay);
                    st.current_match()
                });
                if let Some(Some(m)) = target {
                    self.document.set_cursor(m.start);
                }
            }
            Some(crate::render::PanelHit::Find) => {
                if let Some(st) = self.workspace_state.search_mut() {
                    st.focus_query();
                }
            }
            Some(crate::render::PanelHit::Replace) => {
                if let Some(st) = self.workspace_state.search_mut() {
                    st.focus_replacement();
                }
            }
            Some(crate::render::PanelHit::Elsewhere) => {}
            None => return false,
        }
        self.sync_view(true);
        self.request_frame();
        true
    }

    /// WEB/LINUX MENU BAR press handling — the click half of the awl-rendered menu bar
    /// (`menubar.rs` + `render/chrome/menubar.rs`). Returns `true` when it CLAIMED the
    /// press (the caller then repaints + swallows it), `false` to fall through to the
    /// normal overlay/search/document chain. The design law (shared with the macOS
    /// NSMenu bar): an item fires its catalog `Action` through the SAME `App::apply`
    /// seam a keypress uses — never new behaviour. Behaviour:
    ///   * a press on a clickable dropdown ITEM resolves + fires its `Action`, closes
    ///     the dropdown;
    ///   * a press on a TITLE toggles that menu's dropdown (re-click closes), and closes
    ///     any conflicting summoned overlay/search (the bar draws over them, and a
    ///     dropdown + overlay must not both own input);
    ///   * a press ANYWHERE else while a dropdown is open closes it (click-away);
    ///   * a press on the bar's dead strip (no dropdown) is swallowed, so it never moves
    ///     the caret in the document beneath the bar.
    pub(in crate::app) fn menubar_press(&mut self, exit: &dyn schedule::Exit) -> bool {
        if !crate::menubar::menu_bar_on() {
            return false;
        }
        let (px, py) = self.input.pointer.cursor_px;
        let (item_hit, title_hit, over_surface) = {
            let Some(gpu) = self.frame.gpu() else {
                return false;
            };
            (
                gpu.pipeline.menubar_item_at(px, py),
                gpu.pipeline.menubar_title_at(px, py),
                gpu.pipeline.over_menu_surface(px, py),
            )
        };
        if let Some((menu, item)) = item_hit {
            crate::menubar::set_open(None);
            let action = {
                let menus = crate::menu::roster();
                menus.get(menu).and_then(|menu| {
                    crate::menu::dropdown_action(menu, item, self.document.active_is_markdown())
                })
            };
            if let Some(action) = action {
                let exited = self.apply(action, false, exit, crate::stats::Door::Menu);
                if exited {
                    return true;
                }
            }
            self.sync_view(true);
            return true;
        }
        if let Some(i) = title_hit {
            crate::menubar::toggle_open(i);
            self.workspace_state.dismiss_pickers();
            self.sync_view(true);
            return true;
        }
        if crate::menubar::open_menu().is_some() {
            crate::menubar::set_open(None);
            self.sync_view(true);
            return true;
        }
        // 4. A press on the bar's own dead strip: swallow (never a caret move beneath it).
        over_surface
    }

    pub(in crate::app) fn on_drag(&mut self) {
        if !self.input.pointer.dragging {
            return;
        }
        let Some(_) = self.frame.gpu() else {
            return;
        };
        let idx = self.hit_test_char();
        self.drag_to_char(idx);
    }

    /// Selection-state half of a text drag after the live pipeline has resolved
    /// the pointer to a document character.
    pub(in crate::app) fn drag_to_char(&mut self, idx: usize) {
        match self.input.pointer.drag_granularity {
            DragGranularity::Char => self.document.set_cursor(idx),
            DragGranularity::Word => {
                let anchor = self.document.buffer().anchor_char().unwrap_or(idx);
                let (ws, we) = self.document.buffer().word_bounds(idx);
                if idx >= anchor {
                    self.document.set_cursor(we);
                } else {
                    self.document.set_cursor(ws);
                }
            }
            DragGranularity::Line => {
                let anchor = self.document.buffer().anchor_char().unwrap_or(idx);
                let (ls, le) = self.document.buffer().line_bounds(idx);
                if idx >= anchor {
                    self.document.set_cursor(le);
                } else {
                    self.document.set_cursor(ls);
                }
            }
        }
        // REVEALED PLACEMENT (folds): a drag that extends the selection ACROSS a
        // collapsed section reveals every intersected fold before the selection is
        // shown, through the ONE placement owner — so a drag can never span hidden
        // lines invisibly. A cheap no-op unless a section is folded.
        self.document.reveal_placement();
    }

    /// LIVE-ONLY: recompute the CONTEXT-AWARE OS cursor shape (`cursor_shape.rs`) for
    /// the current mouse position + interaction state, and flip `Window::set_cursor`
    /// ONLY when it actually changed (`cursor_shape::cursor_icon_change` — no per-move
    /// winit chatter). Every context flag reads an EXISTING hit-test — `page_resizing`
    /// (the live drag flag), `workspace_state.overlay_open()`, `page_resize_hover`
    /// proximity test the page-edge press/hover already uses), and
    /// `over_writing_column` (the same column bounds `page_resize_hover` reads) — so
    /// this never invents parallel geometry, it only arbitrates priority among the
    /// existing regions (`cursor_shape::cursor_icon_for`).
    ///
    /// Called on every `CursorMoved`, and again from the two doors that change this
    /// context WITHOUT any mouse motion: a page-edge drag beginning/ending
    /// (`begin_page_resize_if_hovering` / `end_page_resize`) and a summoned overlay
    /// opening/closing (`App::apply`'s shared-core slot lend).
    ///
    /// COMPOSES with pointer auto-hide: while the OS pointer is `Hidden`
    /// (`pointer_hide::PointerHide`), the `set_cursor` call is skipped outright (there
    /// is nothing visible to update) and the cache is left untouched, so the very next
    /// un-hide — always a `CursorMoved`, which recomputes context before anything else
    /// — compares the fresh icon against the still-accurate cache and lands directly on
    /// the context-correct shape instead of a stale one from before the hide.
    pub(in crate::app) fn sync_cursor_icon(&mut self) {
        let Some(gpu) = self.frame.gpu() else { return };
        let (px, py) = self.input.pointer.cursor_px;
        // The pointing-hand affordance now covers EVERY summoned picker's clickable
        // rows (Command-P / go-to / browse / theme / history / keybindings / spell /
        // …), not just spell — reuses the SAME kind-agnostic `overlay_row_at`
        // hit-test the pickers' own click handling uses (`overlay_click`), so a
        // hovered row can never disagree with a clickable one. `overlay_row_at`
        // already returns `None` off a row (the query line, foot hint, scrim, empty
        // gaps), so this lights up only on a real actionable row.
        let overlay_open = self.workspace_state.overlay_open();
        let over_clickable_overlay_row =
            overlay_open && gpu.pipeline.overlay_row_at(px, py).is_some();
        // A clickable LENS-STRIP facet (Time/Register/… of a FACETING picker) earns
        // the same pointing hand as a clickable row — reuses the SAME `overlay_lens_at`
        // hit-test the strip's click handling uses (`overlay_click`), so a hovered
        // facet can never disagree with a clickable one. `None` for a non-faceting
        // picker (no strip drawn) or off the strip row.
        // A workspace's RAIL entry is clickable, so it earns the same
        // pointing hand through the same owner its click handling uses.
        let over_clickable_lens = overlay_open
            && (gpu.pipeline.overlay_lens_at(px, py).is_some()
                || gpu.pipeline.workspace_rail_at(px, py).is_some());
        let over_query_input = overlay_open && gpu.pipeline.over_overlay_query(px, py);
        // A clickable MARGIN-OUTLINE row reads as click-to-jump (the pointing hand),
        // reusing the outline's OWN row geometry (`outline_hit_line`, which folds in
        // the whole hidden/off gate). Only while no overlay is open — an overlay's
        // scrim covers the outline, so the outline never claims the hand behind it.
        let over_outline_row = !overlay_open
            && gpu
                .pipeline
                .outline_hit_line(px, py, gpu.config.height)
                .is_some();
        // A clickable WORKING-SET STACK ROW (the bottom-left margin identity's
        // click-to-switch/close list) reads as the same clickable affordance,
        // reusing the stack's OWN row geometry (`gutter_stack_hit`, which folds in
        // the whole shown/hidden gate — single-file margin, no page mode, an open
        // overlay). Only while no overlay is open, matching the outline row above.
        let over_stack_row = !overlay_open
            && (gpu
                .pipeline
                .gutter_stack_hit(px, py, gpu.config.height)
                .is_some()
                || gpu.pipeline.start_action_at(px, py).is_some());
        let image_hover = gpu
            .pipeline
            .image_handle_at(px, py)
            .map(|(_, handle, _)| handle);
        let over_menu_hand = gpu.pipeline.menubar_hand_at(px, py);
        let over_menu_bar = gpu.pipeline.over_menu_surface(px, py);
        // The summoned find/replace panel's `Aa` case-toggle cell reads as click-to-
        // toggle (the pointing hand) — reuses the SAME `panel_hit` the press path uses,
        // so a hover can never disagree with where a click would land. Only while no
        // overlay is open (the panel is its own floating card, never behind a scrim).
        let over_case_toggle = !overlay_open
            && matches!(
                gpu.pipeline.panel_hit(px, py),
                Some(crate::render::PanelHit::CaseToggle)
            );
        // The RAW summon bit, deliberately ladder-free — see its own doc.
        let summoned = self.workspace_state.popover_summon_bit();
        let over_popover_button = summoned && gpu.pipeline.popover_hit(px, py).is_some();
        // A REVEALED fold chevron (any foldable heading, expanded OR
        // collapsed) reads as click-to-toggle (the pointing hand) — reuses the SAME
        // `fold_chevron_hit` the press path uses, so a hover can never disagree with
        // where a click would land. Only while no overlay is open (its scrim covers
        // the document).
        let over_fold_chevron = !overlay_open && gpu.pipeline.fold_chevron_hit(px, py).is_some();
        let over_modified_link = self.document.has_active()
            && !overlay_open
            && gpu.pipeline.over_writing_column(px)
            && crate::context_menu::modified_link_hover(
                self.input
                    .keyboard
                    .mods
                    .state()
                    .contains(ModifiersState::SUPER),
                {
                    let text = self.document.buffer().text();
                    let byte = self.document.buffer().char_to_byte(self.hit_test_char());
                    crate::markdown::link_at(&text, byte).is_some()
                        || crate::markdown::footnote_target_at(&text, byte).is_some()
                },
            );
        let ctx = crate::cursor_shape::CursorContext {
            dragging_edge: self.input.pointer.page_resizing,
            dragging_text: self.input.pointer.dragging,
            overlay_open,
            over_edge: self.document.has_active() && gpu.pipeline.page_resize_hover(px),
            over_text: self.document.has_active() && gpu.pipeline.over_writing_column(px),
            over_clickable_overlay_row,
            over_clickable_lens,
            over_query_input,
            over_outline_row: over_outline_row || over_modified_link,
            over_stack_row,
            over_menu_hand,
            over_menu_bar,
            over_case_toggle,
            image_drag: self.input.pointer.image_resizing.map(|d| d.handle),
            image_hover,
            over_popover_button,
            over_fold_chevron,
        };
        let desired = crate::cursor_shape::cursor_icon_for(ctx);
        let hidden = self.input.pointer.pointer_hide == crate::pointer_hide::PointerHide::Hidden;
        if let Some(icon) =
            crate::cursor_shape::cursor_icon_change(self.input.pointer.cursor_icon, desired, hidden)
        {
            gpu.window.set_cursor(icon);
            self.input.pointer.cursor_icon = icon;
        }
    }

    fn wheel_scroll_px(&mut self, pixels: f32) {
        if let Some(gpu) = self.frame.gpu() {
            let scroll =
                gpu.pipeline
                    .scroll_by_px(self.document.scroll(), pixels, gpu.config.height as f32);
            self.document.set_scroll(scroll);
        }
    }

    /// `WindowEvent::CursorMoved`: track the pointer, un-hide the auto-hidden OS
    /// pointer, drive whichever pointer OWNER is active (overlay hover / live
    /// page-resize drag / text-selection drag), then recompute the context-aware
    /// cursor shape once for the move regardless of which branch fired.
    pub(in crate::app) fn on_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.input.pointer.cursor_px = (position.x as f32, position.y as f32);
        let prev_pointer_hide = self.input.pointer.pointer_hide;
        self.input.pointer.pointer_hide = crate::pointer_hide::on_mouse_move(prev_pointer_hide);
        if let Some(visible) = crate::pointer_hide::os_visibility_change(
            prev_pointer_hide,
            self.input.pointer.pointer_hide,
        ) && let Some(gpu) = self.frame.gpu()
        {
            gpu.window.set_cursor_visible(visible);
        }
        if self.input.pointer.range_drag.is_some() {
            // A live SETTINGS RAIL SCRUB owns the pointer outright (it is
            // a grabbed control): the value tracks the pointer through the range
            // spec, and the hover below must NOT also re-select rows under the
            // travelling pointer mid-gesture.
            self.on_range_drag();
        } else if self.workspace_state.overlay_open() {
            self.overlay_hover();
        } else if self.input.pointer.page_resizing {
            self.on_page_resize_drag();
        } else if self.input.pointer.image_resizing.is_some() {
            self.on_image_resize_drag();
        } else if self.input.pointer.dragging {
            // THE PHANTOM-SELECTION-CLICK FIX: only extend the selection once the
            // pointer has genuinely traveled past the drag-arm slop from the press
            // position (`exceeds_drag_slop`, pure pixel-distance geometry) — never
            // merely because the hit-test RESULT changed. A WYSIWYG reveal reflow
            // can relocate glyphs under an otherwise-stationary pointer between
            // press and release; without this gate that reflow alone used to read
            // as a real drag. `drag_armed` is sticky for the rest of the gesture
            // once tripped, so a fast real drag keeps extending normally.
            if self.input.pointer.arm_text_drag_if_moved() {
                self.on_drag();
                self.sync_view(true);
                self.request_frame();
            }
        }
        self.update_fold_hover();
        let (px, py) = self.input.pointer.cursor_px;
        let stack_hover_changed = self.frame.gpu_mut().is_some_and(|gpu| {
            gpu.pipeline
                .resolve_gutter_stack_hover(px, py, gpu.config.height)
        });
        if stack_hover_changed {
            self.request_frame();
        }
        self.sync_cursor_icon();
    }

    /// `WindowEvent::CursorLeft`: a hover-only affordance cannot survive after
    /// the pointer leaves the window. The no-prototype path holds no state, so
    /// this remains a repaint-free no-op in production.
    pub(in crate::app) fn on_cursor_left(&mut self) {
        let changed = self
            .frame
            .gpu_mut()
            .is_some_and(|gpu| gpu.pipeline.clear_gutter_stack_hover());
        if changed {
            self.request_frame();
        }
    }

    /// Mirror the FILTERED document row under the pointer into the pipeline so a
    /// heading's fold CHEVRON reveals on hover — it applies to a
    /// collapsed heading's tail-row-only reveal to EVERY foldable heading, expanded
    /// or collapsed (LIVE only — the headless capture has no pointer, so this never
    /// runs there and only the caret-on-heading reveal fires). Gated on
    /// `is_markdown()` (cheap — an `O(1)` extension check, unlike a fold-set scan):
    /// ANY markdown buffer may have a heading to hover, whether or not anything in
    /// it is currently folded, so this can no longer cheaply short-circuit on
    /// `has_folds()` the way the item-47b original did. Requests a redraw only when
    /// the hovered row changes (so a chevron flip repaints without a per-move redraw
    /// storm). The row is the pointer's hit-test line when over the writing column,
    /// else `None` (so a chevron never lingers when the pointer leaves the text).
    /// See [`crate::fold::chevron_revealed`].
    pub(in crate::app) fn update_fold_hover(&mut self) {
        if !self.document.has_active() {
            if self
                .frame
                .gpu_mut()
                .is_some_and(|gpu| gpu.pipeline.set_hover_line(None))
            {
                self.request_frame();
            }
            return;
        }
        let over_col = self.document.buffer().is_markdown() && self.pointer_over_writing_column();
        let (px, py) = self.input.pointer.cursor_px;
        let scroll = self.document.scroll();
        let Some(gpu) = self.frame.gpu_mut() else {
            return;
        };
        let line = if over_col {
            Some(gpu.pipeline.hit_test_scroll(px, py, scroll).0)
        } else {
            None
        };
        if gpu.pipeline.set_hover_line(line) {
            self.request_frame();
        }
    }

    pub(in crate::app) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        self.stamp_input();
        // A summoned MODAL card (About / Lifetime / Streaks) owns any press
        // (`card::dismiss_summoned_card`), but a wheel is not that same kind of
        // intent — SWALLOW it whole rather than either dismissing the card or
        // scrolling the document sitting invisibly behind it. The card itself
        // does not respond to a bare wheel either (Streaks pages by click/←→).
        if crate::card::modal_card_open() {
            return;
        }
        if !self.document.has_active() && !self.workspace_state.overlay_open() {
            return;
        }
        // Zoom modifier: Cmd/Super only. (Ctrl must NOT zoom on mac.)
        let zoom_mod = scroll_zoom_intent(self.input.keyboard.mods.state());
        if !zoom_mod && !self.workspace_state.overlay_open() {
            let (dx, dy) = match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    (x * WHEEL_PIXELS_PER_LINE, y * WHEEL_PIXELS_PER_LINE)
                }
                MouseScrollDelta::PixelDelta(p) => pixel_wheel_axes(
                    p.x as f32,
                    p.y as f32,
                    self.input.pointer.scroll_sensitivity,
                ),
            };
            if dx.abs() > dy.abs() * 1.2 && dx.abs() > 0.5 {
                let (px, py) = self.input.pointer.cursor_px;
                let scroll = self.document.scroll();
                if let Some(gpu) = self.frame.gpu_mut()
                    && gpu.pipeline.try_table_pan(px, py, scroll, dx)
                {
                    self.request_frame();
                    return;
                }
            }
        }
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * WHEEL_LINES_PER_NOTCH,
            MouseScrollDelta::PixelDelta(p) => {
                accumulate_picker_pixels(&mut self.input.pointer.scroll_px_accum, p.y as f32)
            }
        };
        if self.workspace_state.overlay_open() {
            if lines.abs() >= 1.0 {
                // WHICH REGION IS UNDER THE WHEEL, asked kind-neutrally. The
                // question is "does this surface HAVE a read-only comparison
                // standing beside its rows", and
                // `comparison_request()` is that fact typed — the same one
                // `workspace_nav`'s intercept reads to decide whether the
                // keyboard has a content region to go into. It used to be
                // spelled `kind == History`, which was true of the only such
                // surface at the time and silently false for the next one: a
                // conflict workspace's wheel moved its three rows instead of
                // scrolling the manuscript under the pointer.
                let diff_wheel = self
                    .workspace_state
                    .overlay()
                    .map(|o| o.comparison_request().is_some())
                    .unwrap_or(false)
                    && !self
                        .frame
                        .gpu()
                        .and_then(|g| g.pipeline.overlay_card_rect())
                        .map(|[x, y, w, h]| {
                            let (px, py) = self.input.pointer.cursor_px;
                            px >= x && px < x + w && py >= y && py < y + h
                        })
                        .unwrap_or(false);
                if diff_wheel {
                    let delta = -lines.round() as isize; // wheel up = toward the top
                    if let Some(ov) = self.workspace_state.overlay_mut() {
                        ov.diff_scroll = if delta >= 0 {
                            ov.diff_scroll.saturating_add(delta as usize)
                        } else {
                            ov.diff_scroll.saturating_sub((-delta) as usize)
                        };
                    }
                    self.sync_view(false);
                } else {
                    self.overlay_wheel(lines);
                }
            }
        } else if zoom_mod {
            if lines.abs() >= 1.0 {
                let dir = lines.signum();
                let before = self.frame.zoom();
                // One AUTHORED step per notch, through the range spec (the
                // same owner ⌘± and the Settings rail step through).
                self.set_zoom(crate::range::ZOOM.stepped(self.frame.zoom(), dir as i32));
                // Anchor the wheel zoom on the POINTER (captured against the OLD
                // geometry before the deferred reflow) — the doc point under the mouse
                // holds its screen position. Only when the zoom actually moved, so a
                // step against the min/max clamp leaves no stale anchor behind.
                if self.frame.zoom() != before {
                    self.arm_zoom_anchor_pointer();
                }
                self.feed_peek(crate::peek::PeekStimulus::Interrupt);
            }
        } else if let MouseScrollDelta::PixelDelta(p) = delta {
            self.wheel_scroll_px(pixel_wheel_document_px(
                p.y as f32,
                self.input.pointer.scroll_sensitivity,
            ));
            self.sync_view(false);
        } else if let MouseScrollDelta::LineDelta(_, y) = delta {
            self.wheel_scroll_px(line_wheel_document_px(
                y,
                self.frame.zoom(),
                self.frame.dpi(),
            ));
            self.sync_view(false);
        }
        self.request_frame();
    }
}
