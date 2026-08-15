use winit::event::Modifiers;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::convention::Convention;

use super::Action;
use super::binding::canon_key;
use super::state::KeymapState;

impl KeymapState {
    /// True when this convention's NATIVE modifier alone is held (never together
    /// with the OTHER convention's own physical modifier, so the two never
    /// double-fire): [`Convention::Mac`] wants Super without Control;
    /// [`Convention::Linux`] wants Control without Super. THE ONE GATE every
    /// native policy arm below reads. Catalog default collision precedence is
    /// applied while seeding the maps; this helper remains for uncatalogued
    /// native aliases such as Cmd-P and Cmd-G.
    fn native_down(&self, state: ModifiersState) -> bool {
        match self.convention {
            Convention::Mac => {
                state.contains(ModifiersState::SUPER) && !state.contains(ModifiersState::CONTROL)
            }
            Convention::Linux => {
                state.contains(ModifiersState::CONTROL) && !state.contains(ModifiersState::SUPER)
            }
        }
    }

    fn linux_keeps(&self, key: &Key, state: ModifiersState) -> bool {
        self.convention == Convention::Linux && self.linux_keep.contains(&(canon_key(key), state))
    }

    pub fn in_prefix(&self) -> bool {
        self.in_c_x || self.in_c_c
    }

    /// True when `key` — interpreted as the UN-COMPOSED logical key while Alt/Meta is
    /// held — would resolve to a real Meta (Option) chord rather than self-insert.
    ///
    /// This exists for the LIVE macOS Option dead-key fix (`app.rs`): Option composes
    /// a letter into a glyph (Option-f -> 'ƒ'), so `event.logical_key` is the composed
    /// char and a Meta chord would never match. The app asks this of the key WITHOUT
    /// Option composition (`key_without_modifiers`): if it IS a Meta chord, the app
    /// feeds the un-composed key to [`resolve`]; otherwise it keeps the composed char
    /// so Option-accent text INPUT (Option-e -> é) still types.
    ///
    /// Since the identity round RETIRED the built-in Option-letter layer (macOS owns
    /// those keys for typing), there are NO default Meta chords left — a key is a Meta
    /// chord ONLY when a config `[keys]` rebind reclaims it with Meta (Alt). So an
    /// unbound Option-letter always keeps its composed glyph and self-inserts, while a
    /// user-configured Option chord is still un-composed to match. Keyed by the
    /// canonical key. The headless `--keys` path already sends the un-composed key +
    /// ALT, so this predicate is only consulted live.
    pub fn is_meta_chord(&self, key: &Key) -> bool {
        let k = canon_key(key);
        self.override_single
            .keys()
            .any(|(mk, ms)| *mk == k && ms.contains(ModifiersState::ALT))
    }

    pub fn resolve(&mut self, logical: &Key, mods: &Modifiers) -> Action {
        let state = mods.state();
        if !self.in_c_x && !self.in_c_c {
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_single.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_single.get(&chord) {
                return a.clone();
            }
        }
        let ctrl = state.contains(ModifiersState::CONTROL);
        let alt = state.contains(ModifiersState::ALT);
        let sup = state.contains(ModifiersState::SUPER);
        let shift = state.contains(ModifiersState::SHIFT);
        let native = self.native_down(state) && !self.linux_keeps(logical, state);

        // MID-PREFIX (C-x ...): interpret this key as the SECOND key BEFORE the
        // global Super shortcuts below. Otherwise a Cmd combo pressed mid-prefix
        // (Cmd+C/V/Z/P/zoom) would fire its global shortcut AND leave the prefix
        // armed (the early `return` never clears `in_c_x`), so the NEXT key is
        // wrongly swallowed as a C-x second key — a stuck-prefix bug. With the
        // check here, an undefined `C-x <combo>` cancels and clears the prefix.
        //
        // THE C-x DEFAULTS ARE RETIRED (identity round): the static second-key
        // arms are gone, so C-x is now a bare, defaultless prefix — the MACHINERY
        // (prefix state + the `c_x` config-override map + the which-key panel) is
        // KEPT so a `[keys]` "C-x <key>" line reclaims any chord, but WITHOUT a
        // config binding a C-x sequence just cancels quietly.
        if self.in_c_x {
            self.in_c_x = false;
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_c_x.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_c_x.get(&chord) {
                return a.clone();
            }
            return Action::Cancel;
        }

        if self.in_c_c {
            self.in_c_c = false;
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_c_c.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_c_c.get(&chord) {
                return a.clone();
            }
            return Action::Cancel;
        }

        if native {
            match logical {
                Key::Character(s) if s.as_str() == "+" => return Action::ZoomIn,
                Key::Character(s) if s.as_str() == "_" => return Action::ZoomOut,
                _ => {}
            }
        }

        // Cmd-P (Super+P): summon the COMMAND PALETTE. This is its OWN dedicated
        // key — NOT a C-x chord — so it never disturbs the prefix bindings. 'p' is
        // free under Super (undo=z, zoom ==/+/-/0, clipboard=c/x/v), so no
        // collision. Shift is Go-to's Folders deep link; plain is the palette.
        if native
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('p') | Some('P'))
        {
            return if shift {
                Action::OpenProject
            } else {
                Action::OpenCommandPalette
            };
        }

        if native
            && !shift
            && let Key::Character(s) = logical
            && s.starts_with('.')
        {
            return Action::Cancel;
        }

        if native
            && alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('i') | Some('I'))
        {
            return Action::ShowStatsHud;
        }

        if native
            && alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('f') | Some('F'))
        {
            return Action::OpenReplace;
        }

        if native
            && !alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('g') | Some('G'))
        {
            return if shift {
                Action::SearchBackward
            } else {
                Action::SearchForward
            };
        }

        match logical {
            Key::Named(named) => self.resolve_named(*named, ctrl, alt, state),
            Key::Character(s) => self.resolve_char(s, ctrl, alt, sup),
            _ => Action::Ignore,
        }
    }

    fn resolve_named(
        &mut self,
        named: NamedKey,
        ctrl: bool,
        alt: bool,
        state: ModifiersState,
    ) -> Action {
        if let NamedKey::Space = named
            && ctrl
        {
            return Action::SetMark;
        }
        let sup = state.contains(ModifiersState::SUPER);
        match named {
            NamedKey::ArrowLeft => {
                if state.contains(ModifiersState::CONTROL) {
                    Action::BackwardWord
                } else {
                    Action::BackwardChar
                }
            }
            NamedKey::ArrowRight => {
                if state.contains(ModifiersState::CONTROL) {
                    Action::ForwardWord
                } else {
                    Action::ForwardChar
                }
            }
            NamedKey::ArrowUp => Action::PreviousLine,
            NamedKey::ArrowDown => Action::NextLine,
            // THE LINUX-NATIVE override for "Document start"/"Document end"
            // (`commands::LINUX_NATIVE_OVERRIDE`): Ctrl-Home/Ctrl-End is the
            // gedit/VS Code/GTK convention for buffer start/end — NOT the naive
            // Cmd→Ctrl translation of Cmd-Up/Down (which would land on Ctrl-Up/Down,
            // an unclaimed but non-idiomatic chord). Convention-gated (never fires
            // on Mac, where Cmd-Up/Down already owns this) and CHECKED BEFORE the
            // unconditional Home/End arms below, so plain Home/End keep meaning
            // line start/end on every convention — only the CTRL-held combination
            // differs by convention.
            NamedKey::Home if self.convention == Convention::Linux && ctrl => Action::BufferStart,
            NamedKey::End if self.convention == Convention::Linux && ctrl => Action::BufferEnd,
            NamedKey::Home => Action::LineStart,
            NamedKey::End => Action::LineEnd,
            NamedKey::PageUp => Action::PageScrollUp,
            NamedKey::PageDown => Action::PageScrollDown,
            NamedKey::Enter if state.contains(ModifiersState::SHIFT) => Action::AcceptAlternate,
            NamedKey::Enter => Action::Newline,
            // Ctrl-Tab: switch to the LAST (previously-open) buffer — the native
            // slot-1 door (the emacs `C-x b` default is retired). Checked before the
            // indent arms so it never inserts a tab. Native-only in practice: a
            // browser grabs Ctrl-Tab on the web build, where the palette is the door.
            // Shift-Tab OUTDENTS a list level (Tab indents); on a plain line it strips
            // up to two leading spaces (a no-op with none).
            NamedKey::Tab if state.contains(ModifiersState::SHIFT) => Action::Outdent,
            NamedKey::Tab => Action::InsertTab,
            NamedKey::Backspace if sup => Action::DeleteToLineStart,
            NamedKey::Backspace if alt || state.contains(ModifiersState::CONTROL) => {
                Action::DeleteWordBackward
            }
            NamedKey::Backspace => Action::DeleteBackward,
            NamedKey::Delete if alt || state.contains(ModifiersState::CONTROL) => {
                Action::DeleteWordForward
            }
            NamedKey::Delete => Action::DeleteForward,
            NamedKey::Space if !alt => Action::InsertChar(' '),
            NamedKey::Space => Action::Ignore,
            NamedKey::Escape => Action::Cancel,
            _ => Action::Ignore,
        }
    }

    fn resolve_char(&mut self, s: &str, ctrl: bool, alt: bool, sup: bool) -> Action {
        let Some(c) = s.chars().next() else {
            return Action::Ignore;
        };
        let lower = c.to_ascii_lowercase();

        if ctrl && !alt {
            return match lower {
                'd' => Action::DeleteForward,
                'k' => Action::KillLine,
                'v' => Action::PageScrollDown,
                'g' => Action::Cancel,
                'x' => {
                    self.in_c_x = true;
                    Action::BeginPrefix
                }
                'c' => {
                    self.in_c_c = true;
                    Action::BeginPrefix
                }
                _ => Action::Ignore,
            };
        }

        // THE UNBOUND-SUPER SWALLOW GUARD (keybinding audit, 2026-07): every bound
        // Cmd-<x> chord already returned earlier in `resolve` (Cmd-Z, Cmd-S, zoom,
        // Cmd-P, Cmd-B/I/E, …) or via a `[keys]` override (consulted before dispatch
        // ever reaches here). Reaching here WITH Super held means the chord truly
        // has no meaning — mac convention is that an unhandled Cmd combo is inert
        // (at most a beep), never text, so ⌘H/⌘K/⌘D/… must NOT type their letter
        // into the document. This intentionally also swallows Cmd+Option combos
        // (Option's dead-key composition doesn't apply once Cmd is held — a
        // Cmd-chord reads as a shortcut attempt, not typing) and Cmd+Control
        // combos with no ctrl arm above. A bare Control chord (no Super) is NOT
        // affected — it already fell through the `ctrl && !alt` match above with
        // its own `Ignore` default.
        //
        // ⌘K WAS RESERVED here (unbound, falling into this guard) since the
        // keybinding-idiom audit's W1 — Bear/Craft/Notion/Things/Ulysses/Slack all
        // spend Cmd-K on insert/edit-link, the single strongest writer-cluster
        // chord awl didn't yet claim. LINKS V2 spent it: Cmd-K now resolves to
        // `Action::InsertLink` in the native-doors block above, so it no longer
        // reaches this guard.
        if sup {
            return Action::Ignore;
        }

        if !c.is_control() {
            Action::InsertChar(c)
        } else {
            Action::Ignore
        }
    }
}
