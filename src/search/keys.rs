//! The ONE search/replace KEY-INTERCEPTION seam. While the isearch panel is
//! open, EVERY key belongs to the search surface — printable chars extend the
//! focused field, Backspace shortens it, C-s/C-r/arrows step matches, ⌘⌥c (mac)
//! / M-c (Linux) toggles case, Tab/Cmd-R move between the find and replace
//! fields, Enter accepts / replaces, Cmd-Enter replaces all, Esc/C-g aborts —
//! and nothing
//! ever reaches the keymap. BOTH drivers route through [`intercept`]: the live
//! window's `App::handle_search_key` (a thin delegate) and the headless
//! `--keys` replay's search guard (`main/run.rs::replay_keys_mode`), so live
//! editing and captured replay cannot drift (merge, don't align — this seam
//! retired the documented "isearch-input gap" where a replayed char landed in
//! the BUFFER instead of the query). Renderer-independent by construction: it
//! touches only the pure [`SearchState`] model and the [`Buffer`], and returns
//! the one live-only consequence (a caret recoil) for the windowed caller to
//! animate. The step/jump/abort/replace helpers are module-private — the only
//! door in is `intercept`.

use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::{Direction, SearchState, StepOutcome};
use crate::buffer::Buffer;
use crate::caret::RecoilDir;

/// Route one key press to the active search surface. Only meaningful while
/// `*search` is `Some` (both callers gate on that); a `None` search is a no-op.
/// Consumes EVERY key. Mutates the search state + the buffer (cursor follows
/// the current match; a replace writes the document back; accept/abort close
/// the panel by clearing `*search`). Returns `Some(dir)` when a boundary step
/// RECOILED — the Emacs failing-I-search feedback — so the LIVE caller can bump
/// the visual caret ([`crate::caret::CaretAnim::recoil`]); the headless replay
/// ignores it (no clock, no animation), exactly like `Effect::Recoil`.
pub fn intercept(
    search: &mut Option<SearchState>,
    buffer: &mut Buffer,
    logical: &Key,
    mods: ModifiersState,
) -> Option<RecoilDir> {
    let ctrl = mods.contains(ModifiersState::CONTROL);
    let alt = mods.contains(ModifiersState::ALT);
    let sup = mods.contains(ModifiersState::SUPER);
    let shift = mods.contains(ModifiersState::SHIFT);
    let editing_replacement = search
        .as_ref()
        .map(|s| s.is_editing_replacement())
        .unwrap_or(false);

    match logical {
        Key::Character(s) => intercept_character(
            search,
            buffer,
            s.chars().next()?,
            ctrl,
            alt,
            sup,
            shift,
            editing_replacement,
        ),
        Key::Named(named) => {
            intercept_named(search, buffer, *named, ctrl, alt, sup, editing_replacement)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn intercept_character(
    search: &mut Option<SearchState>,
    buffer: &mut Buffer,
    c: char,
    ctrl: bool,
    alt: bool,
    sup: bool,
    shift: bool,
    editing_replacement: bool,
) -> Option<RecoilDir> {
    // Cmd-based Find/Replace chords WITHIN the panel: Cmd-F skips to the
    // next match, Cmd-Shift-F the previous (so you can pass a match without
    // replacing it), Cmd-Option-F reveals+toggles the replace field, Cmd-R
    // focuses the replace field (the headline door — a fresh Cmd-R opened
    // the panel on the find field), Cmd-Option-C toggles case sensitivity
    // (the MAC-REACHABLE case toggle — see below), and Cmd-G / Cmd-Shift-G
    // MIRROR Cmd-F / Cmd-Shift-F's plain step (P2 — the deeper macOS
    // find-next/previous idiom, alongside Cmd-F's own in-panel step;
    // Cmd-Option-G has no Option-toggle counterpart, so it is simply
    // consumed, no-op). Other Super combos are consumed.
    if sup && !ctrl {
        if c.eq_ignore_ascii_case(&'f') {
            if alt {
                if let Some(st) = search.as_mut() {
                    st.toggle_replace();
                }
            } else if shift {
                return step(search, buffer, Direction::Backward);
            } else {
                return step(search, buffer, Direction::Forward);
            }
        } else if c.eq_ignore_ascii_case(&'c') && alt {
            // Cmd-Option-C (⌘⌥C): toggle case sensitivity. This is the
            // MAC-REACHABLE case toggle — a bare Option-c composes to 'ç'
            // on macOS (the logical key never arrives as 'c'+Alt), so the
            // M-c arm below only fires on Linux. Holding Cmd suppresses the
            // accent composition so ⌘⌥C delivers a plain 'c' — the same
            // reason the ⌘⌥F replace-toggle above works. Mirrors VS Code's
            // ⌥⌘C "match case" idiom.
            toggle_case_and_jump(search, buffer);
        } else if c.eq_ignore_ascii_case(&'g') && !alt {
            return step(
                search,
                buffer,
                if shift {
                    Direction::Backward
                } else {
                    Direction::Forward
                },
            );
        } else if c.eq_ignore_ascii_case(&'r')
            && !alt
            && let Some(st) = search.as_mut()
        {
            st.focus_replacement();
        }
        return None;
    }
    if ctrl && !alt {
        match c.to_ascii_lowercase() {
            's' => return step(search, buffer, Direction::Forward),
            'r' => return step(search, buffer, Direction::Backward),
            'g' => abort(search, buffer),
            _ => {}
        }
        return None;
    }
    if alt && !ctrl {
        if matches!(c, 'c' | 'C') {
            // M-c / Alt+c toggles case sensitivity — the LINUX slot (on
            // macOS Option-c composes to 'ç' and never reaches here; use
            // ⌘⌥C above). Kept as the emacs-flavour door where Alt+letter
            // arrives un-composed.
            toggle_case_and_jump(search, buffer);
        }
    } else if !c.is_control() {
        // Self-insert into the FOCUSED field. The replacement is not
        // searched, so typing it never moves a match; query edits do.
        edit_char(search, buffer, c, editing_replacement);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn intercept_named(
    search: &mut Option<SearchState>,
    buffer: &mut Buffer,
    named: NamedKey,
    ctrl: bool,
    alt: bool,
    sup: bool,
    editing_replacement: bool,
) -> Option<RecoilDir> {
    match named {
        // Tab is the one FIELD-SWITCH key: flip focus find↔replace (revealing the
        // replace row the first time). No longer overloaded — Enter replaces, Tab
        // only moves between the two fields of the one warm panel.
        NamedKey::Tab => {
            if let Some(st) = search.as_mut() {
                st.toggle_replace();
            }
        }
        // Down / Up SKIP to the next / previous match without replacing (alongside
        // Cmd-F / Cmd-Shift-F), so you can pass over a match you don't want changed.
        NamedKey::ArrowDown => return step(search, buffer, Direction::Forward),
        NamedKey::ArrowUp => return step(search, buffer, Direction::Backward),
        // ITEM 10 — Left/Right move the FOCUSED field's own caret (char, or a
        // WORD at a time held with Alt/Option) — previously a no-op. Pure
        // motion: never recomputes/jumps, on EITHER field (the replacement
        // NEVER does regardless; the query's text is unchanged by a move).
        NamedKey::ArrowLeft => move_field(search, editing_replacement, alt, false),
        NamedKey::ArrowRight => move_field(search, editing_replacement, alt, true),
        // ITEM 10 — ⌥⌫ word-delete (the word-DELETE rule, distinct from the
        // word-MOTION arrows above): checked BEFORE the plain-Backspace arm so
        // Alt wins. The replacement's word-delete NEVER recomputes/jumps
        // (mirrors `pop_replace_char`'s own asymmetry); the query's DOES (an
        // edit, like `pop_char`).
        NamedKey::Backspace => delete_back(search, buffer, editing_replacement, alt),
        NamedKey::Enter => {
            // The clarified core loop: once replace is active, Enter ALWAYS
            // replaces the current match + advances to the next (regardless of
            // which field has focus) — Cmd-Enter replaces ALL. In a PLAIN find
            // (no replace row), Enter ACCEPTS (closes, leaving the cursor on the
            // current match). Esc / C-g is the "done" door out of replace.
            let replace_active = search
                .as_ref()
                .map(|s| s.is_replace_active())
                .unwrap_or(false);
            if sup && replace_active {
                replace_all(search, buffer);
            } else if replace_active {
                replace_current(search, buffer);
            } else {
                // ACCEPT: remember the query (P2) before closing, so a
                // LATER bare Cmd-G re-finds it.
                if let Some(st) = search.as_ref() {
                    super::set_last_query(st.query());
                }
                *search = None;
                buffer.seal_undo_group();
            }
        }
        NamedKey::Space if !ctrl && !alt && !sup => {
            // Space arrives as a Named key (not a Character), so without this
            // arm it would fall through to the no-op below and never reach the
            // focused field. Ctrl/Alt/Cmd+Space stay no-ops.
            edit_char(search, buffer, ' ', editing_replacement);
        }
        NamedKey::Escape => abort(search, buffer),
        _ => {} // any other named key: consumed, no-op
    }
    None
}

fn edit_char(search: &mut Option<SearchState>, buffer: &mut Buffer, c: char, replacement: bool) {
    if replacement {
        if let Some(st) = search.as_mut() {
            st.push_replace_char(c);
        }
    } else {
        let hay = buffer.text();
        if let Some(st) = search.as_mut() {
            st.push_char(c, &hay);
        }
        jump_to_current(search, buffer);
    }
}

fn move_field(search: &mut Option<SearchState>, replacement: bool, word: bool, right: bool) {
    if let Some(st) = search.as_mut() {
        match (replacement, word, right) {
            (true, true, true) => st.replacement_word_right(),
            (true, true, false) => st.replacement_word_left(),
            (true, false, true) => st.replacement_char_right(),
            (true, false, false) => st.replacement_char_left(),
            (false, true, true) => st.query_word_right(),
            (false, true, false) => st.query_word_left(),
            (false, false, true) => st.query_char_right(),
            (false, false, false) => st.query_char_left(),
        }
    }
}

fn delete_back(
    search: &mut Option<SearchState>,
    buffer: &mut Buffer,
    replacement: bool,
    word: bool,
) {
    if replacement {
        if let Some(st) = search.as_mut() {
            if word {
                st.replacement_delete_word_back();
            } else {
                st.pop_replace_char();
            }
        }
    } else {
        let hay = buffer.text();
        if let Some(st) = search.as_mut() {
            if word {
                st.query_delete_word_back(&hay);
            } else {
                st.pop_char(&hay);
            }
        }
        jump_to_current(search, buffer);
    }
}

/// C-s / C-r (and arrows / the Cmd-F family) while searching: advance to the
/// next/previous match (the Emacs two-press wrap) and move the real cursor onto
/// it. A step that FAILS at the boundary does NOT advance — it returns the
/// recoil direction (forward travels toward the end → bump UP; backward →
/// DOWN), mirroring the blocked-motion recoil, and arms the two-press wrap.
fn step(
    search: &mut Option<SearchState>,
    buffer: &mut Buffer,
    dir: Direction,
) -> Option<RecoilDir> {
    let outcome = search.as_mut().map(|st| st.step(dir));
    let recoil = match outcome {
        Some(StepOutcome::RecoiledAtBoundary(d)) => Some(match d {
            Direction::Forward => RecoilDir::Up,
            Direction::Backward => RecoilDir::Down,
        }),
        _ => None,
    };
    jump_to_current(search, buffer);
    recoil
}

/// Toggle case sensitivity and re-anchor the caret on the (recomputed) current
/// match. The ONE owner of the toggle-case key path — both the mac ⌘⌥C door and
/// the Linux M-c door route through it (merge, don't align), so they can never
/// disagree on the recompute + caret-follow. Also the effect the panel's "Aa"
/// click drives (`App::panel_click`).
fn toggle_case_and_jump(search: &mut Option<SearchState>, buffer: &mut Buffer) {
    let hay = buffer.text();
    if let Some(st) = search.as_mut() {
        st.toggle_case(&hay);
    }
    jump_to_current(search, buffer);
}

/// Move the real buffer cursor onto the current match (if any) so the amber
/// document caret lands on it. No-op (cursor unchanged) when there is no
/// current match — we don't jump on a no-match query.
fn jump_to_current(search: &Option<SearchState>, buffer: &mut Buffer) {
    if let Some(st) = search.as_ref()
        && let Some(m) = st.current_match()
    {
        buffer.set_cursor(m.start);
        // REVEALED PLACEMENT (folds): a match on a collapsed line must not leave
        // the caret logically inside a hidden row — route through the ONE
        // placement owner so the found line reveals. Shared by the live panel and
        // the headless `--keys` replay (both call `intercept`), so search-next /
        // previous can never drift on reveal. A cheap no-op unless folded.
        buffer.reveal_placement();
    }
}

/// Esc / C-g: restore the cursor to where search began and close the panel.
/// REMEMBERS the query first (P2) — a non-empty abandoned search still
/// survives the close so a later bare Cmd-G re-finds it.
fn abort(search: &mut Option<SearchState>, buffer: &mut Buffer) {
    if let Some(st) = search.as_ref() {
        super::set_last_query(st.query());
        let origin = st.origin();
        buffer.set_cursor(origin);
    }
    buffer.clear_mark();
    *search = None;
}

/// REPLACE-CURRENT (Enter in the replace field): swap the active match for the
/// replacement text, write the new document back as one atomic edit, and ADVANCE
/// the search to the next match (the cursor follows). The panel stays open so a
/// repeated Enter walks forward replacing. A no-op unless replace mode is active
/// and there is a current match.
fn replace_current(search: &mut Option<SearchState>, buffer: &mut Buffer) {
    let hay = buffer.text();
    let new_text = match search.as_mut() {
        Some(st) if st.is_replace_active() => st.replace_current_text(&hay),
        _ => return,
    };
    if let Some(t) = new_text {
        buffer.set_text(&t);
        jump_to_current(search, buffer);
    }
}

/// REPLACE-ALL (Cmd-Enter): swap EVERY current-query match for the replacement
/// in one atomic, undoable edit, then re-anchor the (now usually empty) match
/// set at the search origin. A no-op unless replace mode is active and the text
/// actually changes.
fn replace_all(search: &mut Option<SearchState>, buffer: &mut Buffer) {
    let hay = buffer.text();
    let (new_text, origin) = match search.as_ref() {
        Some(st) if st.is_replace_active() => (st.replace_all_text(&hay), st.origin()),
        _ => return,
    };
    if new_text == hay {
        return;
    }
    buffer.set_text(&new_text);
    let new_hay = buffer.text();
    if let Some(st) = search.as_mut() {
        st.refind(origin, &new_hay);
    }
    jump_to_current(search, buffer);
}

#[cfg(test)]
mod tests;
