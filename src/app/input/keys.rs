use crate::app::*;

impl App {
    pub(in crate::app) fn on_key_release(&mut self, released: &Key) {
        if self.input.keyboard.hud_key.as_ref() == Some(released) {
            self.clear_hud();
        }
    }

    /// Dismiss the HELD stats HUD when a SUMMONING modifier is released. macOS does not
    /// deliver a key-UP for a character key while Cmd is held (and the user commonly
    /// lifts Cmd before the letter), so `on_key_release` alone leaves the HUD stuck-on;
    /// a `ModifiersChanged` that drops any modifier present at summon time means the
    /// hold chord is broken, so the HUD vanishes. The pure decision is
    /// [`hud_mods_broken`] (unit-tested without a window).
    pub(in crate::app) fn hud_release_on_mods(&mut self, now: ModifiersState) {
        if self.input.keyboard.hud_key.is_some()
            && hud_mods_broken(self.input.keyboard.hud_mods, now)
        {
            self.clear_hud();
        }
    }

    /// Clear the held stats HUD: drop the process-global held flag, forget the trigger
    /// key/modifiers, and re-sync + redraw so the panel and its scrim vanish. Shared by
    /// both dismissal doors (`on_key_release` for the key, `hud_release_on_mods` for the
    /// modifier) so the HUD is a true momentary hold — gone the instant the chord lifts.
    pub(in crate::app) fn clear_hud(&mut self) {
        crate::hud::set_held(false);
        self.input.keyboard.hud_key = None;
        self.input.keyboard.hud_mods = ModifiersState::empty();
        self.sync_view(false);
        self.request_frame();
    }

    /// Feed ONE stimulus to the HOLD-⌘ SHORTCUT PEEK machine and apply its side
    /// effects. The pure [`crate::peek::PeekArm::next`] decides the next state; this owns
    /// the App-side consequences — stamping the single `WaitUntil` deadline on the
    /// `Idle → Pending` edge, flipping the process-global open/closed, and re-syncing +
    /// redrawing when the card appears or vanishes. THE ONE DOOR every peek transition
    /// routes through (a modifier change, a joined key, a mouse press, a blur, the hold
    /// timer), so the arm state, the global, and the redraw can never drift. An inert
    /// stimulus (no state change — the common case: typing without ⌘, a stray timer) is a
    /// cheap early return with no redraw.
    pub(in crate::app) fn feed_peek(&mut self, stim: crate::peek::PeekStimulus) {
        let before = self.input.keyboard.peek_arm;
        let after = before.next(stim);
        if after == before {
            return;
        }
        self.input.keyboard.peek_arm = after;
        use crate::peek::PeekArm::*;
        match after {
            Pending => self.input.keyboard.peek_armed_at = Some(self.frame.now()),
            Open => {
                self.input.keyboard.peek_armed_at = None;
                crate::peek::set_open(true);
                self.sync_view(false);
                self.request_frame();
            }
            // Any cancellation (broken hold / joined key / click / blur): disarm + close.
            // Only re-sync/redraw when the card was actually up, so a pending-cancel
            // (never drawn) costs no repaint.
            Idle => {
                let was_open = crate::peek::peek_open();
                self.input.keyboard.peek_armed_at = None;
                crate::peek::set_open(false);
                if was_open {
                    self.sync_view(false);
                    self.request_frame();
                }
            }
        }
    }

    pub(in crate::app) fn sync_whichkey_prefix(&mut self) {
        let transition = crate::whichkey::on_key(
            self.input.keyboard.keymap.in_prefix(),
            self.input.keyboard.prefix_pending_at.is_some(),
            self.input.keyboard.whichkey_shown,
        );
        match transition {
            crate::whichkey::PrefixTransition::Arm => {
                self.input.keyboard.prefix_pending_at = Some(self.frame.now());
            }
            // The prefix just resolved or aborted: put the panel down at once (summoned
            // + transient — it never lingers past the chord).
            crate::whichkey::PrefixTransition::Dismiss => self.dismiss_whichkey(),
            crate::whichkey::PrefixTransition::Ignore => {}
        }
    }

    pub(in crate::app) fn summon_whichkey(&mut self) {
        self.input.keyboard.whichkey_shown = true;
        let rows: Vec<(String, String)> = crate::whichkey::continuations_cx(
            &self.config.keys,
            crate::convention::Convention::current(),
            self.config.keymap_flavor(),
        )
        .into_iter()
        .map(|c| (c.key, c.name))
        .collect();
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_whichkey(Some(rows));
            self.request_frame();
        }
    }

    /// Put the which-key panel down + disarm the pause timer. Idempotent — clearing an
    /// already-down panel just redraws nothing new. Redraws only when the panel was
    /// actually shown, so a bare prefix that never paused long enough costs no repaint.
    pub(in crate::app) fn dismiss_whichkey(&mut self) {
        self.input.keyboard.prefix_pending_at = None;
        let was_shown = self.input.keyboard.whichkey_shown;
        self.input.keyboard.whichkey_shown = false;
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_whichkey(None);
            if was_shown {
                self.request_frame();
            }
        }
    }

    /// Route a key to the active search surface (only called while
    /// `workspace_state.search_active()`). A thin delegate to the ONE
    /// renderer-independent interception seam
    /// — [`crate::search::keys::intercept`], shared verbatim with the headless
    /// `--keys` replay's search guard (`main/run.rs`), so the live panel and a
    /// replayed capture cannot drift. The seam consumes EVERY key
    /// (query/replacement typing, Backspace, C-s/C-r/arrow steps, M-c case
    /// toggle, Tab/Cmd-R field moves, Enter accept/replace, Cmd-Enter
    /// replace-all, Esc/C-g abort) and moves the REAL buffer cursor onto the
    /// current match, so the existing amber caret shows it for free. The
    /// returned recoil is the one LIVE-only consequence — a boundary step's
    /// failing-I-search bump — armed here on the visual caret.
    pub(in crate::app) fn handle_search_key(
        &mut self,
        logical: &Key,
        mods: &Modifiers,
        _exit: &dyn schedule::Exit,
    ) {
        let (search, _) = self.workspace_state.core_slots();
        if let Some(dir) = self
            .document
            .intercept_search_key(search, logical, mods.state())
        {
            self.frame.set_caret_recoil(Some(dir));
        }
    }

    pub(in crate::app) fn set_zoom(&mut self, z: f32) {
        let clamped = render::clamp_zoom(z);
        if clamped != self.frame.zoom() {
            self.frame.set_zoom(clamped);
            self.mark_zoom_dirty();
        }
    }

    /// ZOOM ANCHOR — POINTER (wheel ⌘-scroll). Record the document point under the
    /// mouse + its screen y so the deferred reflow keeps it fixed under the cursor.
    /// Reads the OLD (pre-reshape) geometry — call BEFORE the reshape lands, and only
    /// when the zoom actually changed (a no-op `set_zoom` at a clamp must not leave a
    /// stale anchor for an unrelated `sync_view` to apply). No-op headless (no gpu).
    pub(in crate::app) fn arm_zoom_anchor_pointer(&mut self) {
        let Some(gpu) = self.frame.gpu() else { return };
        let (px, py) = self.input.pointer.cursor_px;
        let (line, col) = gpu.pipeline.hit_test_scroll(px, py, self.document.scroll());
        self.frame.set_zoom_anchor(ZoomAnchor {
            line,
            col,
            screen_y: py,
        });
    }

    /// ZOOM ANCHOR — CARET (keyboard ⌘± / ⌘0). Hold the caret's current screen
    /// position; when the caret is OFF-SCREEN, hold the document point at the viewport
    /// centre instead (so the view stays put rather than jumping to the caret). Reads
    /// the OLD (pre-reshape) geometry — call from the zoom-changed arm in `apply`
    /// BEFORE the reflow. No-op headless (no gpu).
    pub(in crate::app) fn arm_zoom_anchor_caret(&mut self) {
        let Some(gpu) = self.frame.gpu() else { return };
        let height = gpu.config.height as f32;
        let top = gpu.pipeline.text_origin_top();
        let (cl, cc) = self.document.buffer().cursor_line_col();
        let caret_y = gpu
            .pipeline
            .char_screen_top_scroll(cl, cc, self.document.scroll());
        self.frame
            .set_zoom_anchor(if caret_y >= top && caret_y < height {
                ZoomAnchor {
                    line: cl,
                    col: cc,
                    screen_y: caret_y,
                }
            } else {
                let cx = (gpu.config.width as f32) * 0.5;
                let cy = (top + height) * 0.5;
                let (line, col) = gpu.pipeline.hit_test_scroll(cx, cy, self.document.scroll());
                ZoomAnchor {
                    line,
                    col,
                    screen_y: cy,
                }
            });
    }

    pub(in crate::app) fn mark_zoom_dirty(&mut self) {
        self.frame.arm_zoom_persist(self.frame.now());
        self.frame.queue_zoom_reflow();
        // ZOOM READOUT: a quiet muted percentage near the pointer while the gesture is
        // in flight (mirrors the page-drag readout). Armed on EVERY zoom step (this is
        // the ONE owner both the keyboard ⌘± and wheel ⌘-scroll paths funnel through),
        // floated at the last pointer position; cleared on settle in `about_to_wait`.
        let (px, py) = self.input.pointer.cursor_px;
        let zoom = self.frame.zoom();
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_zoom_readout(Some((px, py, zoom)));
            self.request_frame();
        }
    }

    /// Is a ZOOM gesture currently IN FLIGHT — a zoom step landed within the last
    /// `ZOOM_PERSIST_DEBOUNCE` and the sticky-zoom write hasn't settled yet? Backed by
    /// the SAME `zoom_persist_at` stamp `mark_zoom_dirty` arms and `about_to_wait`
    /// clears on settle, so "the zoom gesture is live" has ONE owner. Consulted by the
    /// hold-⌘ shortcut peek's zoom-suppression gate ([`crate::peek::peek_allowed`]) so
    /// the frosted card never pops up over the very text the user is zooming to read.
    pub(in crate::app) fn zoom_in_flight(&self) -> bool {
        self.frame.zoom_persist_at().is_some()
    }

    /// The ONE owner of "how many rows is one page". Both the document pager and
    /// the History diff's `PageScrollDown`/`PageScrollUp` step by this, and they
    /// must agree: a reader paging a diff and a writer paging a document are doing
    /// the same gesture and a drift between them is felt, not merely untidy. The
    /// two-row overlap is what makes paging readable — the line you were reading
    /// is still on screen after the jump. Without a GPU there is no viewport to
    /// measure, so one row is the only honest answer.
    pub(in crate::app) fn page_scroll_rows(&self) -> usize {
        let visible = if let Some(gpu) = self.frame.gpu() {
            let scale = self.frame.zoom() * self.frame.dpi();
            render::visible_lines_z(gpu.config.height as f32, render::LINE_HEIGHT * scale, scale)
        } else {
            1
        };
        visible.saturating_sub(2).max(1)
    }

    /// `WindowEvent::ModifiersChanged`: track the live modifier state, and let a
    /// dropped SUMMONING modifier break a held stats-HUD chord (e.g. lifting Cmd
    /// or Option of Option-Cmd-I), covering the macOS case where the character key-UP is never
    /// delivered.
    pub(in crate::app) fn on_modifiers_changed(&mut self, m: Modifiers) {
        self.input.keyboard.mods = m;
        self.hud_release_on_mods(m.state());
        // HOLD-⌘ SHORTCUT PEEK: the ACTIVE CONVENTION's bare arming modifier ALONE
        // arms the hold (`peek::is_bare_arming_modifier` / `peek::arming_modifier` — ⌘
        // on Mac, Ctrl on Linux, the ONE convention→modifier owner); any other modifier
        // state (that modifier plus another, a release, or the OTHER platform's
        // modifier — bare Super is now inert under Linux convention, since the
        // compositor owns it) breaks it — so a pending peek cancels and an open one
        // closes. Feeding `ArmBroken` while Idle is inert, so ordinary typing (no
        // arming modifier) never churns.
        let convention = crate::convention::Convention::current();
        let stim = if crate::peek::is_bare_arming_modifier(m.state(), convention)
            && crate::peek::peek_allowed(self.zoom_in_flight())
        {
            crate::peek::PeekStimulus::ArmAlone
        } else {
            crate::peek::PeekStimulus::ArmBroken
        };
        self.feed_peek(stim);
        self.resync_pointer_derived_state();
    }
    /// `WindowEvent::KeyboardInput`: the full press pipeline — release handling,
    /// the preedit / lone-modifier / search / rebind-capture guards, the macOS
    /// Option dead-key fix, then keymap resolve → `apply`. Preserves every
    /// early-return exactly.
    pub(in crate::app) fn on_keyboard_input(
        &mut self,
        exit: &dyn schedule::Exit,
        event: winit::event::KeyEvent,
    ) {
        // FLIGHT RECORDER / PROBE: the WINIT-RECEIPT link of the
        // event→present chain — logged BEFORE every guard below, so a press the
        // preedit / lone-modifier / release filters swallow is visibly received
        // and visibly dropped, rather than simply absent from the trace.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!(
                "winit KeyboardInput state={:?} key={:?} repeat={}",
                event.state, event.logical_key, event.repeat
            ));
        }
        if event.state != ElementState::Pressed {
            if event.state == ElementState::Released {
                self.on_key_release(&event.logical_key);
            }
            return;
        }
        if !self.input.keyboard.preedit.is_empty() {
            return;
        }
        if let Key::Named(n) = &event.logical_key {
            use winit::keyboard::NamedKey::*;
            if matches!(n, Control | Shift | Alt | Super | Hyper | Meta) {
                return;
            }
        }
        // macOS OPTION DEAD-KEY input: `event.logical_key` is the COMPOSED glyph
        // under Option (Option-f -> 'ƒ'); `key_without_modifiers` recovers the
        // BARE key. Both forms feed the shared dispatch tail, which decides per
        // consumer which one applies (see `dispatch_pressed_key`'s doc). The
        // un-compose is computed here because it needs the raw `KeyEvent`.
        let bare = if self
            .input
            .keyboard
            .mods
            .state()
            .contains(ModifiersState::ALT)
        {
            key_without_modifiers(&event)
        } else {
            event.logical_key.clone()
        };
        self.dispatch_pressed_key(exit, event.logical_key.clone(), bare, event.repeat);
    }

    /// THE SHARED PRESS-DISPATCH TAIL — everything a real (non-modifier,
    /// non-IME) key press does past the raw-`KeyEvent` guards: popover/menubar
    /// dismiss, peek cancel, latency stamp, pointer auto-hide, the search
    /// guard, rebind capture, keymap resolve → `apply` → re-sync + redraw.
    /// ONE owner with two callers: the real `WindowEvent::KeyboardInput` path
    /// above, and the live-probe harness (`--live-script`, `app/probe.rs`),
    /// which feeds parsed chords through this exact tail so a scripted press
    /// and a physical press are the same code by construction (the probe's
    /// `raw` and `bare` coincide — a parsed chord is already un-composed).
    ///
    /// `raw` is the platform's composed `logical_key` (what search/typing see);
    /// `bare` is the Option-un-composed form (what chord recording and a
    /// configured Meta rebind match against); `repeat` is the OS auto-repeat
    /// flag (drives the held-caret trail).
    pub(in crate::app) fn dispatch_pressed_key(
        &mut self,
        exit: &dyn schedule::Exit,
        raw: Key,
        bare: Key,
        repeat: bool,
    ) {
        // FORMAT POPOVER: it is a MOUSE affordance (reveal-on-select) — any real
        // (non-modifier) key press dismisses it, so a keyboard selection / edit
        // never leaves it hanging and can never SUMMON it (the mouse-only rule).
        // Inert when already down. A popover BUTTON click never routes through here
        // (it fires its Action directly via `App::apply`), so the popover stays open
        // across its own applies.
        self.workspace_state.dismiss_popover();
        // WEB/LINUX MENU BAR: a real (non-modifier) key press dismisses an open
        // dropdown — the awl bar's dropdown is mouse-driven (no keyboard nav in v1), so
        // any key closes it (and is otherwise processed normally, exactly like clicking
        // away). Inert unless a dropdown is open, so an ordinary keystroke is a no-op.
        if crate::menubar::open_menu().is_some() {
            crate::menubar::set_open(None);
        }
        // HOLD-⌘ SHORTCUT PEEK: a real (non-modifier) key press means a chord is forming
        // (⌘S, ⌘⇧P's letter, Cmd-I, … on Mac; C-f, C-s, … on Linux, where the SAME
        // arming modifier also carries the emacs nav layer), so cancel a pending peek /
        // close an open one BEFORE it can flicker — THE CRUX of the cancellation
        // contract. Inert unless a peek is actually pending/open, so an ordinary
        // keystroke is a no-op here.
        self.feed_peek(crate::peek::PeekStimulus::KeyJoined);
        // DEBUG key→px: stamp the dispatch receipt of a real key press —
        // every path from here (search keys, rebind capture, the keymap
        // resolve → apply) ends in request_redraw, so this key's pixels
        // are coming. Placed AFTER the lone-modifier/preedit filters: a
        // bare Ctrl tap or an IME-owned key causes no frame and must not
        // linger as a stale stamp inflating the next input's latency.
        self.stamp_input();
        // POINTER AUTO-HIDE: a real keystroke (past the lone-modifier/IME
        // filters above, same gate `stamp_input` uses) hides the OS
        // pointer IMMEDIATELY — the macOS-native convention
        // (`NSCursor.setHiddenUntilMouseMoves`). Any mouse motion
        // instantly reverses it (the `CursorMoved` arm above); so does
        // the window losing focus (the `Focused(false)` arm above).
        let prev_pointer_hide = self.input.pointer.pointer_hide;
        self.input.pointer.pointer_hide = crate::pointer_hide::on_key(prev_pointer_hide);
        if let Some(visible) = crate::pointer_hide::os_visibility_change(
            prev_pointer_hide,
            self.input.pointer.pointer_hide,
        ) && let Some(gpu) = self.frame.gpu()
        {
            gpu.window.set_cursor_visible(visible);
        }
        // SEARCH GUARD: when isearch is active, EVERY key (printable,
        // Backspace, Enter, Esc, C-s, C-r, M-c) is consumed by the search
        // surface and never reaches the keymap, so printable keys extend
        // the query instead of inserting into the rope. Placed AFTER the
        // lone-modifier filter (so a bare Shift/Ctrl tap during search is
        // dropped) and AFTER the preedit guard, but BEFORE keymap.resolve.
        if self.workspace_state.search_active() {
            let mods = self.input.keyboard.mods;
            self.handle_search_key(&raw, &mods, exit);
            self.sync_view(true);
            self.request_frame();
            return;
        }
        // REBIND MENU live CAPTURE: while the menu is RECORDING, the next press
        // IS the binding — intercepted at the CHORD level, BEFORE keymap
        // resolution, so any combo (C-t / M-f / a bare key) is recorded verbatim
        // rather than run. Enter / Esc are EXCLUDED (they finish / abort the
        // capture via the normal resolve → apply_transition path below). Option
        // composition is undone (like the dead-key fix) so Option-f records as
        // M-f, not the composed glyph. The headless replay records PLAIN keys
        // through `apply_transition` instead; both call `OverlayState::capture_record`.
        if self.capture_recording() {
            let is_ctrl_key = matches!(
                &raw,
                Key::Named(winit::keyboard::NamedKey::Enter)
                    | Key::Named(winit::keyboard::NamedKey::Escape)
            );
            if !is_ctrl_key {
                let combo = crate::keyspec::format_chord(&bare, self.input.keyboard.mods.state());
                let finished = self
                    .workspace_state
                    .overlay_mut()
                    .map(|o| o.capture_record(combo))
                    .unwrap_or(false);
                if finished
                    && let Some((slug, binding)) = self
                        .workspace_state
                        .overlay()
                        .and_then(|o| o.capture_target())
                {
                    self.rebind_commit(slug, binding, false);
                }
                self.sync_view(true);
                self.request_frame();
                return;
            }
        }
        self.frame.set_caret_held(repeat);
        // macOS OPTION DEAD-KEY FIX (LIVE path only): Option composes a
        // letter into a glyph (Option-f -> 'ƒ'), so the raw `logical_key` is the
        // composed char. Since the identity round retired the built-in Option-letter
        // layer, `is_meta_chord` is true ONLY for a key a config `[keys]` Meta rebind
        // reclaims — so when ALT is held we resolve the un-composed `bare` form ONLY
        // for such a configured chord; otherwise we keep the composed `raw` key so
        // Option-accent INPUT (Option-e -> é, Option-n -> ñ) types as text. The
        // headless `--keys` replay already sends the un-composed key + ALT, so this
        // branch is exercised only live (its behaviour with a real composing keyboard
        // needs human confirmation).
        let logical = if self
            .input
            .keyboard
            .mods
            .state()
            .contains(ModifiersState::ALT)
            && self.input.keyboard.keymap.is_meta_chord(&bare)
        {
            bare
        } else {
            raw
        };
        let mods = self.input.keyboard.mods;
        let action = self.input.keyboard.keymap.resolve(&logical, &mods);
        // LIFETIME STATS: record this press into the odometer — a keystroke, a
        // printable char iff it resolved to an insert, and the capped active-
        // writing interval since the previous press. On the keyboard-input path
        // past every filter (lone-modifier/IME/preedit/search/capture), so it
        // counts real presses only; config-gated + native-only inside.
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_note_keystroke(matches!(action, Action::InsertChar(_)));
        self.sync_whichkey_prefix();
        // HELD stats HUD: remember the trigger key AND the modifiers held at
        // summon, so its RELEASE dismisses the HUD — either the key lifting
        // (`on_key_release`) or a summoning modifier dropping (`hud_release_on_mods`,
        // the macOS case where the letter's key-UP never arrives while Cmd is down).
        // The press itself summons it via `apply_transition` (sets the process-global); an
        // OS auto-repeat re-affirms the same key/mods.
        if action == Action::ShowStatsHud {
            self.input.keyboard.hud_key = Some(logical.clone());
            self.input.keyboard.hud_mods = self.input.keyboard.mods.state();
        }
        // SHIFT = SELECT-INTENT, keyed on the pressed CHORD (the resolved logical
        // key), not the Action alone. `M-<` / `M->` need Shift just to TYPE the
        // `<` / `>` glyph (a `Key::Character`), so that Shift is INCIDENTAL and
        // must NOT extend — but Shift+Cmd-Up/Down (macOS) and Shift+Ctrl-Home/End
        // (Linux) reach the SAME BufferStart/BufferEnd actions through a named
        // navigation key and DO extend, exactly like every platform text field.
        // The ONE owner (`motion_honors_shift_select`) makes that call from key
        // shape; the headless `--keys` replay derives its flag through the same fn.
        let shift = self
            .input
            .keyboard
            .mods
            .state()
            .contains(ModifiersState::SHIFT)
            && motion_honors_shift_select(&action, &logical);
        // FLIGHT RECORDER / PROBE: the KEYMAP-RESOLVE link — what the
        // press actually became. A chord that resolved to something other than the
        // navigation action is a DROPPED input; the same action twice for one
        // physical press is a REPEATED one.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            crate::probe::trace(format_args!("resolve -> {action:?} shift={shift}"));
        }
        self.apply(action, shift, exit, crate::stats::Door::Chord);
    }
}
