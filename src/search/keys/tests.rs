use super::*;
use winit::keyboard::SmolStr;

fn ch(s: &str) -> Key {
    Key::Character(SmolStr::new(s))
}

fn named(k: NamedKey) -> Key {
    Key::Named(k)
}

const NONE: ModifiersState = ModifiersState::empty();

/// Open a search over `text` anchored at char 0 and return (search, buffer).
fn open(text: &str) -> (Option<SearchState>, Buffer) {
    let buffer = Buffer::from_str(text);
    let search = Some(SearchState::start(0, Direction::Forward));
    (search, buffer)
}

/// Feed a bare printable string char-by-char through the seam.
fn type_str(search: &mut Option<SearchState>, buffer: &mut Buffer, s: &str) {
    for c in s.chars() {
        let key = if c == ' ' {
            named(NamedKey::Space)
        } else {
            ch(&c.to_string())
        };
        intercept(search, buffer, &key, NONE);
    }
}

/// THE SEARCH-TYPING REGRESSION (the retired "isearch-input gap"): with the
/// panel open, printable keys extend the QUERY — the buffer text is never
/// touched — and the cursor lands on the current match.
#[test]
fn typing_extends_the_query_never_the_buffer() {
    let (mut search, mut buffer) = open("alpha beta alpha");
    type_str(&mut search, &mut buffer, "beta");
    assert_eq!(search.as_ref().unwrap().query(), "beta");
    assert_eq!(
        buffer.text(),
        "alpha beta alpha",
        "the document is untouched"
    );
    assert_eq!(buffer.cursor_char(), 6, "the caret sits on the match");
    // Space through the Named-key arm joins the query too.
    type_str(&mut search, &mut buffer, " a");
    assert_eq!(search.as_ref().unwrap().query(), "beta a");
    assert_eq!(buffer.text(), "alpha beta alpha");
}

#[test]
fn a_search_hit_on_a_hidden_line_reveals_its_fold() {
    // REVEALED PLACEMENT (folds): a match inside a collapsed section must not
    // leave the caret logically inside a hidden row. Fold # A (hiding "needle" on
    // line 1), then search for it — `jump_to_current` places the caret on the
    // match AND routes through the placement owner, which reveals the fold. Shared
    // by the live panel and the headless replay (both call `intercept`).
    let mut buffer = Buffer::from_str("# A\nneedle\n# B\nb");
    buffer.set_cursor(0);
    buffer.toggle_fold_at_cursor(); // fold # A -> hides line 1 ("needle")
    assert!(
        buffer.folds().contains(&0),
        "precondition: # A folded, needle hidden"
    );
    let mut search = Some(SearchState::start(0, Direction::Forward));
    type_str(&mut search, &mut buffer, "needle");
    assert!(
        buffer.folds().is_empty(),
        "landing a search hit on a hidden line revealed the fold"
    );
    assert_eq!(
        buffer.cursor_line_col().0,
        1,
        "caret sits on the found (now visible) line"
    );
}

#[test]
fn backspace_pops_the_focused_field() {
    let (mut search, mut buffer) = open("abc abd");
    type_str(&mut search, &mut buffer, "abc");
    assert_eq!(search.as_ref().unwrap().hit_count(), 1);
    intercept(&mut search, &mut buffer, &named(NamedKey::Backspace), NONE);
    let st = search.as_ref().unwrap();
    assert_eq!(st.query(), "ab");
    assert_eq!(st.hit_count(), 2);
    // With the replace field focused, Backspace edits the REPLACEMENT.
    intercept(&mut search, &mut buffer, &named(NamedKey::Tab), NONE);
    type_str(&mut search, &mut buffer, "xy");
    intercept(&mut search, &mut buffer, &named(NamedKey::Backspace), NONE);
    let st = search.as_ref().unwrap();
    assert_eq!(st.replacement(), "x");
    assert_eq!(
        st.query(),
        "ab",
        "the query is untouched by replace-field edits"
    );
}

#[test]
fn steps_advance_and_recoil_at_the_boundary() {
    let (mut search, mut buffer) = open("x.x.x");
    type_str(&mut search, &mut buffer, "x");
    assert_eq!(buffer.cursor_char(), 0);
    // Every step door advances: C-s, ArrowDown, Cmd-F, Cmd-G.
    assert_eq!(
        intercept(&mut search, &mut buffer, &ch("s"), ModifiersState::CONTROL),
        None
    );
    assert_eq!(buffer.cursor_char(), 2);
    assert_eq!(
        intercept(&mut search, &mut buffer, &named(NamedKey::ArrowDown), NONE),
        None
    );
    assert_eq!(buffer.cursor_char(), 4);
    // First forward press at the last match: recoil UP, cursor stays put.
    assert_eq!(
        intercept(&mut search, &mut buffer, &ch("f"), ModifiersState::SUPER),
        Some(RecoilDir::Up)
    );
    assert_eq!(buffer.cursor_char(), 4);
    // Second press wraps to the first match.
    assert_eq!(
        intercept(&mut search, &mut buffer, &ch("g"), ModifiersState::SUPER),
        None
    );
    assert_eq!(buffer.cursor_char(), 0);
    // Backward from the first match: recoil DOWN, then C-r/ArrowUp step back.
    assert_eq!(
        intercept(&mut search, &mut buffer, &ch("r"), ModifiersState::CONTROL),
        Some(RecoilDir::Down)
    );
    assert_eq!(buffer.cursor_char(), 0);
    // Cmd-Shift-F / Cmd-Shift-G mirror the backward step (post-recoil wrap).
    assert_eq!(
        intercept(
            &mut search,
            &mut buffer,
            &ch("F"),
            ModifiersState::SUPER | ModifiersState::SHIFT
        ),
        None
    );
    assert_eq!(
        buffer.cursor_char(),
        4,
        "armed backward step wrapped to the last match"
    );
}

#[test]
fn alt_c_toggles_case_sensitivity() {
    // The LINUX slot: bare Alt+c (un-composed) toggles case.
    let (mut search, mut buffer) = open("Hello HELLO hello");
    type_str(&mut search, &mut buffer, "hello");
    assert_eq!(search.as_ref().unwrap().hit_count(), 3);
    intercept(&mut search, &mut buffer, &ch("c"), ModifiersState::ALT);
    let st = search.as_ref().unwrap();
    assert!(st.is_case_sensitive());
    assert_eq!(st.hit_count(), 1);
    intercept(&mut search, &mut buffer, &ch("C"), ModifiersState::ALT);
    assert!(!search.as_ref().unwrap().is_case_sensitive());
}

/// THE MAC-REACHABILITY FIX: ⌘⌥C toggles case + re-anchors the caret. Bare
/// Option-c composes to 'ç' on macOS and never reaches the M-c arm, so this
/// Cmd-suppressed chord is the only keyboard door to the case toggle on the
/// advertised keymap — the affordance the user reported as unreachable.
#[test]
fn cmd_option_c_toggles_case_sensitivity_and_reanchors() {
    let (mut search, mut buffer) = open("Hello HELLO hello");
    type_str(&mut search, &mut buffer, "hello");
    assert_eq!(search.as_ref().unwrap().hit_count(), 3);
    let cmd_opt = ModifiersState::SUPER | ModifiersState::ALT;
    // Case ON: only the exact-case "hello" survives; the caret follows it.
    assert_eq!(intercept(&mut search, &mut buffer, &ch("c"), cmd_opt), None);
    let st = search.as_ref().unwrap();
    assert!(st.is_case_sensitive());
    assert_eq!(st.hit_count(), 1);
    assert_eq!(
        buffer.cursor_char(),
        12,
        "the caret re-anchored on the surviving match"
    );
    // Uppercase variant (⌘⌥⇧C emits 'C') toggles back off.
    intercept(
        &mut search,
        &mut buffer,
        &ch("C"),
        cmd_opt | ModifiersState::SHIFT,
    );
    assert!(!search.as_ref().unwrap().is_case_sensitive());
    assert_eq!(
        buffer.text(),
        "Hello HELLO hello",
        "the document is never touched"
    );
}

/// Tab reveals the replace row then flips focus; Cmd-R forces focus into the
/// replacement; Cmd-Option-F rides the same toggle — the affordances the
/// retired `apply_transition` search intercept used to cover at the Action level.
#[test]
fn tab_and_cmd_r_move_between_the_two_fields() {
    let (mut search, mut buffer) = open("alpha beta alpha");
    intercept(&mut search, &mut buffer, &named(NamedKey::Tab), NONE);
    {
        let st = search.as_ref().unwrap();
        assert!(st.is_replace_active());
        assert!(st.is_editing_replacement());
    }
    intercept(&mut search, &mut buffer, &named(NamedKey::Tab), NONE);
    assert!(!search.as_ref().unwrap().is_editing_replacement());
    intercept(&mut search, &mut buffer, &ch("r"), ModifiersState::SUPER);
    assert!(search.as_ref().unwrap().is_editing_replacement());
    // Cmd-Option-F toggles back to the find field.
    intercept(
        &mut search,
        &mut buffer,
        &ch("f"),
        ModifiersState::SUPER | ModifiersState::ALT,
    );
    assert!(!search.as_ref().unwrap().is_editing_replacement());
    // None of the field motion leaked a char anywhere.
    assert_eq!(buffer.text(), "alpha beta alpha");
}

#[test]
fn enter_accepts_a_plain_find_and_remembers_the_query() {
    let _g = crate::testlock::serial();
    crate::search::clear_last_query();
    let (mut search, mut buffer) = open("alpha beta alpha");
    type_str(&mut search, &mut buffer, "beta");
    intercept(&mut search, &mut buffer, &named(NamedKey::Enter), NONE);
    assert!(search.is_none(), "plain-find Enter closes the panel");
    assert_eq!(
        buffer.cursor_char(),
        6,
        "the cursor stays on the accepted match"
    );
    assert_eq!(crate::search::last_query(), "beta");
    crate::search::clear_last_query();
}

#[test]
fn enter_replaces_current_and_cmd_enter_replaces_all() {
    let (mut search, mut buffer) = open("x.x.x");
    type_str(&mut search, &mut buffer, "x");
    intercept(&mut search, &mut buffer, &named(NamedKey::Tab), NONE);
    type_str(&mut search, &mut buffer, "Y");
    // Enter in replace mode: swap ONE match, advance, panel stays open.
    intercept(&mut search, &mut buffer, &named(NamedKey::Enter), NONE);
    assert_eq!(buffer.text(), "Y.x.x");
    assert!(search.is_some(), "replace-current keeps the panel open");
    assert_eq!(buffer.cursor_char(), 2, "cursor advanced to the next match");
    // Cmd-Enter: swap EVERY remaining match in one edit.
    intercept(
        &mut search,
        &mut buffer,
        &named(NamedKey::Enter),
        ModifiersState::SUPER,
    );
    assert_eq!(buffer.text(), "Y.Y.Y");
    assert!(search.is_some());
    assert_eq!(search.as_ref().unwrap().hit_count(), 0, "no needle remains");
}

#[test]
fn escape_aborts_restoring_the_origin_cursor() {
    let _g = crate::testlock::serial();
    crate::search::clear_last_query();
    let mut buffer = Buffer::from_str("alpha beta alpha");
    buffer.set_cursor(3);
    let mut search = Some(SearchState::start(3, Direction::Forward));
    type_str(&mut search, &mut buffer, "beta");
    assert_eq!(buffer.cursor_char(), 6, "the search moved the cursor");
    intercept(&mut search, &mut buffer, &named(NamedKey::Escape), NONE);
    assert!(search.is_none());
    assert_eq!(buffer.cursor_char(), 3, "abort restores the origin");
    assert_eq!(
        crate::search::last_query(),
        "beta",
        "an abandoned query is still remembered"
    );
    crate::search::clear_last_query();
}

/// EVERY key is consumed while the panel is open: a C-x never arms the
/// keymap prefix (it isn't even seen by the keymap), an unbound Super combo
/// and a stray named key are quiet no-ops, and none of them leak into the
/// buffer or close the panel.
#[test]
fn unhandled_chords_are_consumed_no_ops() {
    let (mut search, mut buffer) = open("alpha beta alpha");
    type_str(&mut search, &mut buffer, "beta");
    for (key, mods) in [
        (ch("x"), ModifiersState::CONTROL), // the live C-x prefix chord
        (ch("p"), ModifiersState::SUPER),   // Cmd-P: palette stays shut
        (named(NamedKey::Home), NONE),      // stray named key
        (named(NamedKey::Space), ModifiersState::CONTROL), // modified Space
    ] {
        assert_eq!(intercept(&mut search, &mut buffer, &key, mods), None);
        let st = search.as_ref().expect("the panel stays open");
        assert_eq!(st.query(), "beta", "the query is unchanged");
    }
    assert_eq!(buffer.text(), "alpha beta alpha");
}

enum_with_all! {
    #[derive(Clone, Copy, Debug)]
    enum PanelKeyAffordance {
        TypeQuery,
        Backspace,
        NextMatch,
        PrevMatch,
        ToggleCase,
        FieldSwitch,
        SelectAllField,
        ReplaceCurrent,
        ReplaceAll,
        Accept,
        Abort,
    }
}

/// ⌘A selects the WHOLE focused field and leaves the parked document
/// selection exactly as it was — the affordance the routing repair added,
/// driven here through the same `intercept` seam as every other one.
fn assert_select_all_field_drivable(cmd: ModifiersState) {
    let (mut s, mut b) = open("x.x.x");
    type_str(&mut s, &mut b, "x");
    let before = b.selection_range();
    intercept(&mut s, &mut b, &ch("a"), cmd);
    assert_eq!(
        s.as_ref().unwrap().focused_selection(),
        Some((0, 1)),
        "SelectAllField selects the whole focused field via \u{2318}A"
    );
    assert_eq!(
        b.selection_range(),
        before,
        "SelectAllField must leave the parked document selection alone"
    );
}

fn assert_affordance_drivable(affordance: PanelKeyAffordance) {
    use PanelKeyAffordance::*;
    let cmd = ModifiersState::SUPER;
    let ctrl = ModifiersState::CONTROL;
    let cmd_opt = ModifiersState::SUPER | ModifiersState::ALT;
    match affordance {
        TypeQuery => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            assert_eq!(
                s.as_ref().unwrap().query(),
                "x",
                "TypeQuery extends the query"
            );
        }
        Backspace => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "xy");
            intercept(&mut s, &mut b, &named(NamedKey::Backspace), NONE);
            assert_eq!(
                s.as_ref().unwrap().query(),
                "x",
                "Backspace shortens the query"
            );
        }
        NextMatch => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &ch("s"), ctrl);
            assert_eq!(b.cursor_char(), 2, "NextMatch advances the caret");
        }
        PrevMatch => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &named(NamedKey::ArrowDown), NONE); // ->2
            intercept(&mut s, &mut b, &ch("r"), ctrl); // ->0
            assert_eq!(b.cursor_char(), 0, "PrevMatch steps the caret back");
        }
        ToggleCase => {
            let (mut s, mut b) = open("Hi HI hi");
            type_str(&mut s, &mut b, "hi");
            let before = s.as_ref().unwrap().hit_count();
            intercept(&mut s, &mut b, &ch("c"), cmd_opt);
            assert!(
                s.as_ref().unwrap().is_case_sensitive(),
                "ToggleCase flips via the mac-reachable ⌘⌥C"
            );
            assert_ne!(
                s.as_ref().unwrap().hit_count(),
                before,
                "the match set recomputed on toggle"
            );
        }
        FieldSwitch => {
            let (mut s, mut b) = open("x.x.x");
            intercept(&mut s, &mut b, &named(NamedKey::Tab), NONE);
            assert!(
                s.as_ref().unwrap().is_editing_replacement(),
                "FieldSwitch reveals + focuses the replace field"
            );
        }
        SelectAllField => assert_select_all_field_drivable(cmd),
        ReplaceCurrent => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &named(NamedKey::Tab), NONE);
            type_str(&mut s, &mut b, "Y");
            intercept(&mut s, &mut b, &named(NamedKey::Enter), NONE);
            assert_eq!(b.text(), "Y.x.x", "ReplaceCurrent swaps one match");
        }
        ReplaceAll => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &named(NamedKey::Tab), NONE);
            type_str(&mut s, &mut b, "Y");
            intercept(&mut s, &mut b, &named(NamedKey::Enter), cmd);
            assert_eq!(b.text(), "Y.Y.Y", "ReplaceAll swaps every match");
        }
        Accept => {
            let (mut s, mut b) = open("x.x.x");
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &named(NamedKey::Enter), NONE);
            assert!(s.is_none(), "Accept closes the panel");
        }
        Abort => {
            let mut b = Buffer::from_str("x.x.x");
            b.set_cursor(1);
            let mut s = Some(SearchState::start(1, Direction::Forward));
            type_str(&mut s, &mut b, "x");
            intercept(&mut s, &mut b, &named(NamedKey::Escape), NONE);
            assert!(s.is_none(), "Abort closes the panel");
            assert_eq!(b.cursor_char(), 1, "Abort restores the origin caret");
        }
    }
}

/// THE KEY-REACHABILITY LAW. Every in-panel KEYBOARD affordance must drive an
/// observable effect through the ONE `intercept` seam. The match in
/// `assert_affordance_drivable` is NO-WILDCARD, so a new affordance fails to
/// compile until it has a driving arm. Every arm uses the advertised mac
/// chord when one exists; ToggleCase uses ⌘⌥C, not Linux-only M-c.
#[test]
fn every_panel_key_affordance_is_drivable() {
    let _g = crate::testlock::serial();
    crate::search::clear_last_query();
    for affordance in PanelKeyAffordance::ALL {
        assert_affordance_drivable(affordance);
    }
    crate::search::clear_last_query();
}

// --- THE ROUTED-ACTION DOOR ---------------------------------------------

/// **THE PANEL'S SECOND DOOR CONSUMES EVERY EDIT VERB, AND GIVES SELECT-ALL A
/// FIELD-SCOPED MEANING — ON BOTH FIELDS.**
///
/// `intercept_action` is what a macOS menu-bar key equivalent, a menu or
/// context-menu click, and a palette row's `Effect::RunAction` reach. Swept
/// over the Edit menu's OWN roster (`menu::edit_verbs::edit_menu_actions`), because that
/// is precisely the set AppKit installs real key equivalents for — the door
/// the defect travelled down. The buffer is passed in and read back so a verb
/// that reached the rope is caught here, one tier below the App.
#[test]
fn the_action_door_keeps_every_edit_verb_inside_the_focused_field() {
    let _g = crate::testlock::serial();
    let verbs = crate::menu::edit_verbs::edit_menu_actions();
    assert!(!verbs.is_empty(), "the Edit-menu verb roster is empty");

    for replacement_focused in [false, true] {
        for action in &verbs {
            let (mut s, mut b) = open("alpha beta alpha");
            type_str(&mut s, &mut b, "beta");
            if replacement_focused {
                let st = s.as_mut().unwrap();
                st.focus_replacement();
                for c in "gamma".chars() {
                    st.push_replace_char(c);
                }
            }
            let doc_before = (b.text(), b.version(), b.selection_range());
            let q_before = s.as_ref().unwrap().query().to_string();
            let r_before = s.as_ref().unwrap().replacement().to_string();

            crate::search::keys::intercept_action(&mut s, action);

            let st = s.as_ref().expect("the panel stays open");
            assert_eq!(
                (b.text(), b.version(), b.selection_range()),
                doc_before,
                "{action:?} (replacement focused: {replacement_focused}) reached the \
                 document behind the panel (verbs swept: {verbs:?})"
            );
            assert_eq!(st.query(), q_before, "{action:?} rewrote the query text");
            assert_eq!(
                st.replacement(),
                r_before,
                "{action:?} rewrote the replacement text"
            );
            let expect = if matches!(action, crate::keymap::Action::SelectAll) {
                let focused = if replacement_focused {
                    &r_before
                } else {
                    &q_before
                };
                Some((0, focused.chars().count()))
            } else {
                None
            };
            assert_eq!(
                st.focused_selection(),
                expect,
                "{action:?} (replacement focused: {replacement_focused}): only \
                 Select all may arm the focused field"
            );
            // The field WITHOUT focus is never touched by either outcome.
            let unfocused = if replacement_focused {
                st.query_selection()
            } else {
                st.replacement_selection()
            };
            assert_eq!(
                unfocused, None,
                "{action:?}: the unfocused field must not acquire a selection"
            );
        }
    }
}

/// **SELECT ALL ON AN EMPTY FIELD ARMS NOTHING** — the "never a zero-width
/// selection" invariant, asked at the door rather than only at `TextBox`, so
/// a fresh panel's ⌘A cannot leave a phantom selection the next keystroke
/// silently "replaces".
#[test]
fn select_all_on_an_empty_field_arms_no_selection() {
    let _g = crate::testlock::serial();
    let (mut s, b) = open("alpha");
    crate::search::keys::intercept_action(&mut s, &crate::keymap::Action::SelectAll);
    assert_eq!(s.as_ref().unwrap().focused_selection(), None);
    assert_eq!(b.text(), "alpha");
}

/// **SELECT ALL SPANS THE FIELD IN CHARS, NOT BYTES.** A CJK / combining /
/// emoji query is multibyte, and a byte length used as a char index would
/// either panic on the next splice or select a range that is not the field.
/// The next typed character must replace the WHOLE field regardless.
#[test]
fn select_all_then_typing_replaces_a_multibyte_field_whole() {
    let _g = crate::testlock::serial();
    for needle in ["日本語", "e\u{0301}te\u{0301}", "🇯🇵🇫🇷", "👍"] {
        let (mut s, mut b) = open("nothing matches here");
        type_str(&mut s, &mut b, needle);
        crate::search::keys::intercept_action(&mut s, &crate::keymap::Action::SelectAll);
        assert_eq!(
            s.as_ref().unwrap().focused_selection(),
            Some((0, needle.chars().count())),
            "{needle:?}: the span is CHARS, not bytes"
        );
        type_str(&mut s, &mut b, "z");
        assert_eq!(
            s.as_ref().unwrap().query(),
            "z",
            "{needle:?}: typing must replace the whole multibyte field"
        );
        assert_eq!(b.text(), "nothing matches here");
    }
}
