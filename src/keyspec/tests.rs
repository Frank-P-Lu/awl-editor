use super::*;

// CONVENTION-PROOF SHADOW: every hardcoded literal spec in this module
// documents specifically MAC-native default behavior ("Cmd-S is the save
// door", "s-Down = buffer end", a bare "C-n"/"C-x" resolving to its EMACS
// default) — pinning is the honest fix rather than re-deriving a
// per-convention expectation for each (Linux's own displacement/collision
// behavior is separately, exhaustively law-tested in `keymap.rs`). These
// local definitions SHADOW the module-level `parse_keys`/`parse_keys_with`
// pulled in by `use super::*` (a local item always wins over a glob import
// in Rust name resolution), so no individual call site below needed editing.
fn parse_keys(spec: &str) -> Result<Vec<Action>> {
    super::parse_keys_pinned(spec, crate::convention::Convention::Mac)
}
fn parse_keys_with(spec: &str, cfg: &crate::config::Config) -> Result<Vec<Action>> {
    parse_keys_through(
        spec,
        KeymapState::with_overrides_and_convention(&cfg.keys, crate::convention::Convention::Mac),
    )
}

#[test]
fn ctrl_motions_sequence() {
    assert_eq!(
        parse_keys("C-n C-n").unwrap(),
        vec![Action::NextLine, Action::NextLine]
    );
}

#[test]
fn native_save_and_c_x_retired() {
    assert_eq!(parse_keys("s-s").unwrap(), vec![Action::Save]);
    assert_eq!(parse_keys("C-x C-s").unwrap(), vec![Action::Cancel]);
}

#[test]
fn self_insert_two_chars() {
    assert_eq!(
        parse_keys("a b").unwrap(),
        vec![Action::InsertChar('a'), Action::InsertChar('b')]
    );
}

#[test]
fn native_buffer_end() {
    assert_eq!(parse_keys("s-Down").unwrap(), vec![Action::BufferEnd]);
    assert_eq!(parse_keys("M->").unwrap(), vec![Action::InsertChar('>')]);
}

#[test]
fn native_buffer_start() {
    assert_eq!(parse_keys("s-Up").unwrap(), vec![Action::BufferStart]);
    assert_eq!(parse_keys("M-<").unwrap(), vec![Action::InsertChar('<')]);
}

#[test]
fn parse_keys_with_honours_config_rebind() {
    use crate::config::Config;
    // The config-rebind replay path `main` actually uses: a `[keys]` override
    // resolves a chord to the rebound Action while the spec still splits/prefixes
    // correctly. The default (no-override) path must NOT produce that Action.
    let mut cfg = Config::empty();
    cfg.keys.push(("toggle_debug".into(), vec!["C-j".into()]));
    assert_eq!(
        parse_keys_with("C-j", &cfg).unwrap(),
        vec![Action::ToggleDebug]
    );
    assert_ne!(
        parse_keys("C-j").unwrap(),
        vec![Action::ToggleDebug],
        "default C-j is not ToggleDebug"
    );
    let empty = Config::empty();
    for spec in ["C-n C-n", "C-x C-s", "a b"] {
        assert_eq!(
            parse_keys_with(spec, &empty).unwrap(),
            parse_keys(spec).unwrap(),
            "empty config == default for {spec:?}"
        );
    }
}

#[test]
fn word_form_modifiers_parse_like_terse() {
    // The macOS-native word spellings resolve to the SAME chords as the terse
    // single-letter forms, so a config `Cmd-S` reaches the keymap as Super+S.
    let pairs = [
        ("Cmd-s", "s-s"),
        ("Option-f", "M-f"),
        ("Ctrl-x", "C-x"),
        ("Cmd-S-z", "s-S-z"),
    ];
    for (word, terse) in pairs {
        assert_eq!(
            parse_chord(word).map(|(k, m)| (k, m.state())).unwrap(),
            parse_chord(terse).map(|(k, m)| (k, m.state())).unwrap(),
            "{word:?} should parse like {terse:?}"
        );
    }
    assert_eq!(
        parse_chord("CMD-=").unwrap().1.state(),
        ModifiersState::SUPER
    );
    let (k, m) = parse_chord("Cmd--").unwrap();
    assert_eq!(m.state(), ModifiersState::SUPER);
    assert_eq!(k, Key::Character(SmolStr::new("-")));
}

#[test]
fn format_chord_round_trips_through_parse() {
    // A captured key press → canonical terse spec, in the FIXED modifier order,
    // that parses back to the SAME (key, mods). Covers letter-folding, named
    // keys, and the macOS `s-`/`M-` modifiers.
    for spec in [
        "C-t", "M-f", "s-s", "Left", "Enter", "Esc", "s-S-z", "C-x", "=",
    ] {
        let (k, m) = parse_chord(spec).unwrap();
        let formatted = format_chord(&k, m.state());
        let (k2, m2) = parse_chord(&formatted).unwrap();
        assert_eq!(
            (canon(&k2), m2.state()),
            (canon(&k), m.state()),
            "{spec:?} → {formatted:?}"
        );
    }
    assert_eq!(format_chord(&ch_key("T"), ModifiersState::CONTROL), "C-t");
    assert_eq!(
        format_chord(&ch_key("z"), ModifiersState::SUPER | ModifiersState::SHIFT),
        "S-s-z"
    );
}

#[test]
fn mac_glyph_chord_renders_modifier_glyphs() {
    assert_eq!(mac_glyph_chord("Cmd-S-o"), "⌘⇧O");
    assert_eq!(mac_glyph_chord("Cmd-F"), "⌘F");
    assert_eq!(mac_glyph_chord("Cmd-M-f"), "⌘⌥F"); // Replace: Cmd-Option-F
    assert_eq!(mac_glyph_chord("Cmd-S"), "⌘S"); // the trailing S is the KEY
    assert_eq!(mac_glyph_chord("Cmd-S-z"), "⌘⇧Z"); // Redo
    assert_eq!(mac_glyph_chord("Cmd-;"), "⌘;");
    assert_eq!(mac_glyph_chord("Cmd-="), "⌘=");
    assert_eq!(mac_glyph_chord("C-t"), "⌃T"); // a Ctrl chord → the ⌃ glyph
    assert_eq!(mac_glyph_chord("s-s"), mac_glyph_chord("Cmd-S"));
    // An unparseable token passes through verbatim (never panics).
    assert_eq!(mac_glyph_chord("C-frobnicate"), "C-frobnicate");
}

#[test]
fn translate_native_for_linux_swaps_super_for_control_only() {
    assert_eq!(translate_native_for_linux("Cmd-S"), "C-s");
    assert_eq!(translate_native_for_linux("Cmd-S-p"), "C-S-p");
    assert_eq!(translate_native_for_linux("Cmd-,"), "C-,");
    // ALT/SHIFT-only chords (no Super) pass through unchanged — the naive
    // translator never touches a modifier it didn't ask about.
    assert_eq!(translate_native_for_linux("M-Right"), "M-Right");
    // An unparseable token passes through verbatim (never panics).
    assert_eq!(translate_native_for_linux("C-frobnicate"), "C-frobnicate");
}

#[test]
fn linux_glyph_chord_renders_word_labels() {
    assert_eq!(linux_glyph_chord("C-s"), "Ctrl+S");
    assert_eq!(linux_glyph_chord("S-C-p"), "Ctrl+Shift+P");
    assert_eq!(linux_glyph_chord("C-,"), "Ctrl+,");
    assert_eq!(linux_glyph_chord("M-Right"), "Alt+Right");
    assert_eq!(
        linux_glyph_chord(&format_chord(
            &ch_key("z"),
            ModifiersState::SUPER | ModifiersState::CONTROL
        )),
        "Ctrl+Super+Z"
    );
    // An unparseable token passes through verbatim (never panics).
    assert_eq!(linux_glyph_chord("C-frobnicate"), "C-frobnicate");
}

#[test]
fn canonical_binding_unifies_equivalent_specs() {
    assert_eq!(canonical_binding("Cmd-S"), canonical_binding("s-s"));
    assert_eq!(canonical_binding("C-x C-f").as_deref(), Some("C-x C-f"));
    assert_eq!(canonical_binding("Ctrl-t").as_deref(), Some("C-t"));
    // A garbled token yields None (no panic), so a bad capture can't conflict.
    assert_eq!(canonical_binding("C-frobnicate"), None);
    assert_eq!(canonical_binding("   "), None);
}

fn ch_key(s: &str) -> Key {
    Key::Character(SmolStr::new(s))
}

fn canon(k: &Key) -> Key {
    match k {
        Key::Character(s) => Key::Character(SmolStr::new(s.to_lowercase())),
        other => other.clone(),
    }
}

#[test]
fn unknown_chord_errors() {
    // A multi-char token that is not a named key is an error, not a panic.
    parse_keys("frobnicate").unwrap_err();
}

#[test]
fn pageup_pagedown_tokens_parse_and_page_the_picker() {
    for tok in ["PageUp", "pageup", "PgUp"] {
        let chords = parse_chords(tok).unwrap();
        assert_eq!(
            chords[0].key,
            Key::Named(NamedKey::PageUp),
            "{tok} -> PageUp"
        );
    }
    for tok in ["PageDown", "pagedown", "PgDn", "pagedn"] {
        let chords = parse_chords(tok).unwrap();
        assert_eq!(
            chords[0].key,
            Key::Named(NamedKey::PageDown),
            "{tok} -> PageDown"
        );
    }
    assert_eq!(super::key_token(&Key::Named(NamedKey::PageUp)), "PageUp");
    assert_eq!(
        super::key_token(&Key::Named(NamedKey::PageDown)),
        "PageDown"
    );
}

#[test]
fn text_chords_spell_each_char_as_its_keys_token() {
    let chords = text_chords("Hi w,\n\t").unwrap();
    let specs: Vec<&str> = chords.iter().map(|c| c.spec.as_str()).collect();
    assert_eq!(specs, vec!["H", "i", "Space", "w", ",", "Enter", "Tab"]);
    assert_eq!(chords[0].key, Key::Character(SmolStr::new("H")));
    assert_eq!(chords[2].key, Key::Named(NamedKey::Space));
    assert_eq!(chords[5].key, Key::Named(NamedKey::Enter));
    assert_eq!(chords[6].key, Key::Named(NamedKey::Tab));
    // No modifiers on any typed char — the char token itself carries its
    // case ('H' self-inserts 'H'), so text chords never need `S-`. (Motion
    // chords DO honor an explicit `S-` as select-intent; see
    // `main/run.rs::ReplaySession::apply_chord`.)
    assert!(chords.iter().all(|c| c.mods.state().is_empty()));
    let mut km = KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut resolver = ChordResolver::new(&mut km, true);
    assert_eq!(
        resolver.resolve(&chords[0]).unwrap(),
        Some(Action::InsertChar('H'))
    );
    assert_eq!(
        resolver.resolve(&chords[2]).unwrap(),
        Some(Action::InsertChar(' '))
    );
}

/// Strict parse pinned to Mac, mirroring the permissive shadow above (these
/// tests document MAC-native defaults; Linux displacement is law-tested in
/// `keymap.rs`).
fn parse_keys_strict(spec: &str) -> Result<Vec<Action>> {
    parse_keys_mode(
        spec,
        KeymapState::new_with_convention(crate::convention::Convention::Mac),
        true,
    )
}

#[test]
fn strict_rejects_an_unbound_chord_naming_it() {
    assert_eq!(parse_keys("s-l").unwrap(), Vec::<Action>::new());
    let err = parse_keys_strict("s-l").unwrap_err().to_string();
    assert!(err.contains("\"s-l\""), "names the exact chord: {err}");
    assert!(err.contains("unbound"), "says why: {err}");
    let err = parse_keys_strict("C-n s-l C-p").unwrap_err().to_string();
    assert!(err.contains("\"s-l\""), "mid-spec offender named: {err}");
}

#[test]
fn strict_rejects_a_dangling_prefix_sequence_naming_both_chords() {
    let err = parse_keys_strict("C-x C-s").unwrap_err().to_string();
    assert!(err.contains("\"C-s\""), "names the dangling chord: {err}");
    assert!(err.contains("\"C-x\""), "names the prefix: {err}");
}

#[test]
fn strict_allows_an_explicit_prefix_cancel() {
    for spec in ["C-x Esc", "C-x C-g"] {
        assert_eq!(
            parse_keys_strict(spec).unwrap(),
            parse_keys(spec).unwrap(),
            "explicit cancel stays legal in strict: {spec:?}"
        );
    }
}

#[test]
fn strict_matches_permissive_on_fully_bound_specs() {
    for spec in [
        "C-n C-n",
        "s-s",
        "a b C-Space Left",
        "Enter Tab Backspace",
        "s-Down M->",
    ] {
        assert_eq!(
            parse_keys_strict(spec).unwrap(),
            parse_keys(spec).unwrap(),
            "strict is a pure gate, never a different resolution: {spec:?}"
        );
    }
}

#[test]
fn strict_still_errors_on_an_unparseable_token() {
    // The unparseable-token error is shared with the permissive door (both
    // name the token); strict adds no second vocabulary for it.
    let err = parse_keys_strict("frobnicate").unwrap_err().to_string();
    assert!(err.contains("\"frobnicate\""), "{err}");
}

#[test]
fn named_keys_and_modifiers() {
    assert_eq!(
        parse_keys("Left Right").unwrap(),
        vec![Action::BackwardChar, Action::ForwardChar]
    );
    assert_eq!(parse_keys("M-Right").unwrap(), vec![Action::ForwardWord]);
    assert_eq!(
        parse_keys("Enter Tab Backspace Delete").unwrap(),
        vec![
            Action::Newline,
            Action::InsertTab,
            Action::DeleteBackward,
            Action::DeleteForward,
        ]
    );
}

#[test]
fn c_space_sets_mark_then_motion() {
    assert_eq!(
        parse_keys("C-Space C-f").unwrap(),
        vec![Action::SetMark, Action::ForwardChar]
    );
}

#[test]
fn shifted_literal_self_inserts() {
    assert_eq!(parse_keys("<").unwrap(), vec![Action::InsertChar('<')]);
}

#[test]
fn case_preserved_on_self_insert() {
    assert_eq!(parse_keys("Z").unwrap(), vec![Action::InsertChar('Z')]);
}

#[test]
fn save_and_quit_via_native() {
    assert_eq!(
        parse_keys("s-s s-q").unwrap(),
        vec![Action::Save, Action::Quit]
    );
    assert_eq!(
        parse_keys("C-x C-s C-x C-c").unwrap(),
        vec![Action::Cancel, Action::Cancel]
    );
}

/// THE CHORD THE RESTORE NOTICE TEACHES IS THE ONE THAT FIRES.
///
/// `undo_chord_label` renders a spec per convention, and a sentence that names
/// a key which does nothing is worse than one that names no key at all. The
/// oracle is the REAL keymap, resolved on each convention in turn — not a
/// second copy of the spec, which would agree with itself forever.
#[test]
fn the_taught_undo_chord_is_the_one_that_fires() {
    let _g = crate::testlock::serial();
    for (convention, spec) in [
        (crate::convention::Convention::Mac, UNDO_SPEC_MAC),
        (crate::convention::Convention::Linux, UNDO_SPEC_LINUX),
    ] {
        let mut km = crate::keymap::KeymapState::new_with_convention(convention);
        let (key, mods) = parse_chord(spec).expect("the taught spec parses");
        assert_eq!(
            km.resolve(&key, &mods),
            Action::Undo,
            "{convention:?}: the restore notice teaches {spec:?}, which must actually \
             undo on that convention"
        );
        let label = match convention {
            crate::convention::Convention::Mac => mac_glyph_chord(spec),
            crate::convention::Convention::Linux => linux_glyph_chord(spec),
        };
        assert!(
            !label.is_empty() && label != spec,
            "{convention:?}: the label must be rendered for a reader, not the raw spec \
             ({label:?})"
        );
    }
}

#[test]
fn empty_spec_is_empty() {
    assert_eq!(parse_keys("   ").unwrap(), Vec::<Action>::new());
}

// THE FIND/REPLACE PANEL'S TAUGHT CHORDS ARE THE ONES THAT FIRE — the
// `undo_chord_label` proof, repeated for a surface with no keymap
// underneath it at all: `search::keys::intercept` consumes these
// directly, so `KeymapState::resolve` cannot see them, and the only
// honest proof is feeding the SPEC through the SAME `intercept` door the
// live panel and `--keys` replay both use. Both convention specs are
// checked for every chord (not just the convention this host happens to
// run), matching the panel's own "sweep both, not the host default" law.
// One `#[test]` per chord family (rather than one long function) so a
// failure names exactly which taught chord stopped firing.

#[test]
fn panel_replace_next_chord_fires() {
    use crate::buffer::Buffer;
    use crate::search::keys::intercept;
    use crate::search::{Direction, SearchState};

    // No replace row: ACCEPTS (closes the panel).
    for spec in [PANEL_REPLACE_NEXT.mac, PANEL_REPLACE_NEXT.linux] {
        let mut buffer = Buffer::from_str("alpha");
        let mut search = Some(SearchState::start_with_query(
            0,
            Direction::Forward,
            "alpha",
            &buffer.text(),
        ));
        let (key, mods) = parse_chord(spec).expect("Enter parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        assert!(
            search.is_none(),
            "{spec:?}: Enter with no replace row accepts and closes"
        );
    }

    // Replace row up: replaces the CURRENT match, panel stays open.
    for spec in [PANEL_REPLACE_NEXT.mac, PANEL_REPLACE_NEXT.linux] {
        let mut buffer = Buffer::from_str("cat cat");
        let mut search = Some(SearchState::start_with_query(
            0,
            Direction::Forward,
            "cat",
            &buffer.text(),
        ));
        let st = search.as_mut().unwrap();
        st.toggle_replace();
        st.push_replace_char('d');
        st.push_replace_char('o');
        st.push_replace_char('g');
        let (key, mods) = parse_chord(spec).expect("Enter parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        assert!(
            search.is_some(),
            "{spec:?}: Enter with a replace row stays open"
        );
        assert_eq!(
            buffer.text(),
            "dog cat",
            "{spec:?}: Enter replaced only the current match"
        );
    }
}

#[test]
fn panel_replace_all_chord_fires() {
    use crate::buffer::Buffer;
    use crate::search::keys::intercept;
    use crate::search::{Direction, SearchState};

    for spec in [PANEL_REPLACE_ALL.mac, PANEL_REPLACE_ALL.linux] {
        let mut buffer = Buffer::from_str("cat cat cat");
        let mut search = Some(SearchState::start_with_query(
            0,
            Direction::Forward,
            "cat",
            &buffer.text(),
        ));
        let st = search.as_mut().unwrap();
        st.toggle_replace();
        st.push_replace_char('d');
        st.push_replace_char('o');
        st.push_replace_char('g');
        let (key, mods) = parse_chord(spec).expect("s-Enter parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        assert_eq!(
            buffer.text(),
            "dog dog dog",
            "{spec:?}: replaced every match"
        );
    }
}

#[test]
fn panel_switch_field_chord_fires() {
    use crate::buffer::Buffer;
    use crate::search::keys::intercept;
    use crate::search::{Direction, SearchState};

    // Switches focus find <-> replace (and reveals the replace row the
    // first time).
    for spec in [PANEL_SWITCH_FIELD.mac, PANEL_SWITCH_FIELD.linux] {
        let mut buffer = Buffer::from_str("alpha");
        let mut search = Some(SearchState::start(0, Direction::Forward));
        let (key, mods) = parse_chord(spec).expect("Tab parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        let st = search.as_ref().unwrap();
        assert!(
            st.is_replace_active(),
            "{spec:?}: Tab reveals the replace row"
        );
        assert!(
            st.is_editing_replacement(),
            "{spec:?}: Tab moves focus onto it"
        );
    }
}

#[test]
fn panel_close_chord_fires() {
    use crate::buffer::Buffer;
    use crate::search::keys::intercept;
    use crate::search::{Direction, SearchState};

    // Closes, restoring the origin cursor.
    for spec in [PANEL_CLOSE.mac, PANEL_CLOSE.linux] {
        let mut buffer = Buffer::from_str("alpha");
        buffer.set_cursor(3);
        let mut search = Some(SearchState::start(3, Direction::Forward));
        let (key, mods) = parse_chord(spec).expect("Esc parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        assert!(search.is_none(), "{spec:?}: Esc closes the panel");
    }
}

#[test]
fn panel_match_case_chord_fires_only_on_its_own_convention() {
    use crate::buffer::Buffer;
    use crate::search::keys::intercept;
    use crate::search::{Direction, SearchState};

    // Mac holds Super down too (bare Option-c composes to an accented
    // letter there); Linux is bare Alt-c. Each fires ONLY on its own
    // convention's spec.
    for spec in [PANEL_MATCH_CASE.mac, PANEL_MATCH_CASE.linux] {
        let mut buffer = Buffer::from_str("Cat cat");
        let mut search = Some(SearchState::start_with_query(
            0,
            Direction::Forward,
            "cat",
            &buffer.text(),
        ));
        let (key, mods) = parse_chord(spec).expect("case chord parses");
        intercept(&mut search, &mut buffer, &key, mods.state());
        assert!(
            search.as_ref().unwrap().is_case_sensitive(),
            "{spec:?}: the case-toggle spec fires"
        );
    }

    // The taught LABEL must actually differ per convention for this one
    // chord, whose spec genuinely differs (the others are unmodified
    // named keys and legitimately read the same word on both).
    let mac_label = PANEL_MATCH_CASE.label_for(crate::convention::Convention::Mac);
    let linux_label = PANEL_MATCH_CASE.label_for(crate::convention::Convention::Linux);
    assert_ne!(
        mac_label, linux_label,
        "match-case must not show the same label on both conventions"
    );
    assert!(
        !mac_label.contains("Alt+") && !mac_label.contains("Ctrl+"),
        "mac label must carry glyphs, not GTK words: {mac_label:?}"
    );
}
