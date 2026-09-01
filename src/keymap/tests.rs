use std::collections::HashMap;

use winit::event::Modifiers;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

use crate::convention::Convention;

use super::state::insert_default_entry;
use super::*;

/// The live catalog rendered as one `slug|native@mac|native@linux|emacs`
/// line per command, newline-terminated — the value feeding the frozen
/// `catalog_chord_snapshot_is_frozen` guard. `native@mac`/`native@linux` go
/// through `commands::resolved_native` (the one owner of the Cmd->Ctrl
/// translation + override table); the emacs slot is convention-agnostic text.
fn catalog_chord_snapshot() -> String {
    let mut out = String::new();
    for c in crate::commands::COMMANDS.iter() {
        out.push_str(&format!(
            "{}|{}|{}|{}\n",
            crate::commands::slug(c.name),
            crate::commands::resolved_native(c, Convention::Mac),
            crate::commands::resolved_native(c, Convention::Linux),
            c.emacs,
        ));
    }
    out
}

const CATALOG_CHORD_SNAPSHOT: &str = "\
command_palette|Cmd-P|C-p|
go_to|Cmd-O|C-o|
open_file|||
open_folder|||
spell_suggestions|Cmd-;|C-;|
version_history|Cmd-S-h|C-S-h|
compare_with_version|||
clean_unused_assets|||
keep_version|||
last_file|C-Tab|C-Tab|
new_document|Cmd-N|C-n|
keep_tutorial|||
move|||
rename_note|||
duplicate_note|||
save_a_copy|||
move_file_to_trash|||
reveal_in_file_manager|||
copy_file_path|||
finish_file|Cmd-W|C-w|
follow_link|||C-c C-o
copy_link_destination|||
switch_theme|Cmd-T|C-t|
caret_style|||
dictionary|||
keymap|||
toggle_spellcheck|||
toggle_caret_style|||
toggle_page_mode|||
toggle_writing_nits|||
widen_page|||
narrow_page|||
reset_page_width|||
toggle_debug|||
toggle_outline|Cmd-S-o|C-S-o|
fold_section|Cmd-S-e|C-S-e|C-c C-f
collapse_other_sections|Cmd-S-m|C-S-m|C-c C-t
toggle_typewriter_scroll|||
toggle_menu_bar|||
about|||
credits|||
lifetime_stats|||
writing_streaks|||
line_endings|||
align_table|||
tag_document_language|||
insert_date|Cmd-S-d|C-S-d|C-c .
report_a_problem|||
download_file|||
check_for_updates|||
open_scratch|||
blockquote|||
bullet_list|||
numbered_list|||
task_list|Cmd-S-l|C-S-l|
heading|||
cycle_heading|||
code_block|||
bold|Cmd-B|C-b|
italic|Cmd-I|C-i|
inline_code|Cmd-E|C-e|
highlight|||
strikethrough|||
insert_footnote|||
export_as_word|||
export_as_html|||
export_as_pdf|||
insert_link|Cmd-K|C-k|
insert_table|||
save|Cmd-S|C-s|
review_the_change|||
save_your_version|||
use_disk_version|||
quit|Cmd-Q|C-q|
search_forward|Cmd-F|C-f|C-s
search_backward|Cmd-S-f|C-S-f|C-r
find_and_replace|Cmd-R|C-r|
undo|Cmd-Z|C-z|C-/
redo|Cmd-S-z|C-S-z|
copy|Cmd-C|C-c|
cut|Cmd-X|C-x|C-w
paste|Cmd-V|C-v|C-y
select_all|Cmd-A|C-a|
zoom_in|Cmd-=|C-=|
zoom_out|Cmd--|C--|
reset_zoom|Cmd-0|C-0|
forward_word|M-Right|M-Right|
backward_word|M-Left|M-Left|
line_start|Cmd-Left|Home|C-a
line_end|Cmd-Right|End|C-e
document_start|Cmd-Up|C-Home|
document_end|Cmd-Down|C-End|
forward_char|||C-f
backward_char|||C-b
next_line|||C-n
previous_line|||C-p
move_line_up|Option-Up|M-Up|
move_line_down|Option-Down|M-Down|
delete_word_forward|||
delete_word_backward|||
settings|Cmd-,|C-,|
keybindings|||
";

struct KeymapState(super::KeymapState);
impl KeymapState {
    fn new() -> Self {
        Self(super::KeymapState::new_with_convention(Convention::Mac))
    }
    fn with_overrides(keys: &[(String, Vec<String>)]) -> Self {
        Self(super::KeymapState::with_overrides_and_convention(
            keys,
            Convention::Mac,
        ))
    }
    fn new_with_convention(convention: Convention) -> Self {
        Self(super::KeymapState::new_with_convention(convention))
    }
    fn with_overrides_and_convention(
        keys: &[(String, Vec<String>)],
        convention: Convention,
    ) -> Self {
        Self(super::KeymapState::with_overrides_and_convention(
            keys, convention,
        ))
    }
}
impl std::ops::Deref for KeymapState {
    type Target = super::KeymapState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for KeymapState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn ch(s: &str) -> Key {
    Key::Character(SmolStr::new(s))
}

fn mods(state: ModifiersState) -> Modifiers {
    Modifiers::from(state)
}

fn ctrl() -> Modifiers {
    mods(ModifiersState::CONTROL)
}

fn alt() -> Modifiers {
    mods(ModifiersState::ALT)
}

fn none() -> Modifiers {
    mods(ModifiersState::empty())
}

fn sup() -> Modifiers {
    mods(ModifiersState::SUPER)
}

#[test]
fn ctrl_motions() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("f"), &ctrl()), Action::ForwardChar);
    assert_eq!(km.resolve(&ch("b"), &ctrl()), Action::BackwardChar);
    assert_eq!(km.resolve(&ch("n"), &ctrl()), Action::NextLine);
    assert_eq!(km.resolve(&ch("p"), &ctrl()), Action::PreviousLine);
    assert_eq!(km.resolve(&ch("a"), &ctrl()), Action::LineStart);
    assert_eq!(km.resolve(&ch("e"), &ctrl()), Action::LineEnd);
}

#[test]
fn ctrl_editing() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("d"), &ctrl()), Action::DeleteForward);
    assert_eq!(km.resolve(&ch("k"), &ctrl()), Action::KillLine);
    assert_eq!(km.resolve(&ch("y"), &ctrl()), Action::Yank);
    assert_eq!(km.resolve(&ch("g"), &ctrl()), Action::Cancel);
}

#[test]
fn ctrl_search() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("s"), &ctrl()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("r"), &ctrl()), Action::SearchBackward);
}

#[test]
fn both_convention_modifiers_keep_the_data_backed_emacs_fallback() {
    let both = mods(ModifiersState::CONTROL | ModifiersState::SUPER);
    for convention in [Convention::Mac, Convention::Linux] {
        let mut km = KeymapState::new_with_convention(convention);
        assert_eq!(km.resolve(&ch("f"), &both), Action::ForwardChar);
        assert_eq!(km.resolve(&ch("s"), &both), Action::SearchForward);
        assert_eq!(km.resolve(&ch("c"), &both), Action::BeginPrefix);
        assert_eq!(km.resolve(&ch("o"), &both), Action::FollowLink);
    }
}

#[test]
fn cmd_f_find_and_replace_bindings() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("f"), &sup()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("F"), &sup_shift()), Action::SearchBackward);
    assert_eq!(km.resolve(&ch("f"), &sup_alt()), Action::OpenReplace);
    assert_eq!(km.resolve(&ch("r"), &sup()), Action::OpenReplace);
    assert_eq!(km.resolve(&ch("R"), &sup_shift()), Action::OpenReplace);
    // The C-s / C-r isearch chords MUST keep working (additive, not replaced).
    assert_eq!(km.resolve(&ch("s"), &ctrl()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("r"), &ctrl()), Action::SearchBackward);
    assert_eq!(km.resolve(&ch("f"), &none()), Action::InsertChar('f'));
    assert_eq!(km.resolve(&ch("f"), &ctrl()), Action::ForwardChar);
    assert!(!Action::OpenReplace.is_motion() && !Action::OpenReplace.is_edit());
}

#[test]
fn option_letter_layer_is_retired_word_and_buffer_moved_to_native() {
    let mut km = KeymapState::new();
    assert_eq!(
        km.resolve(&ch("f"), &alt()),
        Action::InsertChar('f'),
        "M-f retired"
    );
    assert_eq!(
        km.resolve(&ch("b"), &alt()),
        Action::InsertChar('b'),
        "M-b retired"
    );
    assert_eq!(
        km.resolve(&ch("w"), &alt()),
        Action::InsertChar('w'),
        "M-w retired"
    );
    assert_eq!(
        km.resolve(&ch("v"), &alt()),
        Action::InsertChar('v'),
        "M-v retired"
    );
    assert_eq!(
        km.resolve(&ch("<"), &alt()),
        Action::InsertChar('<'),
        "M-< retired"
    );
    assert_eq!(
        km.resolve(&ch(">"), &alt()),
        Action::InsertChar('>'),
        "M-> retired"
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowRight), &alt()),
        Action::ForwardWord
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowLeft), &alt()),
        Action::BackwardWord
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowUp), &sup()),
        Action::BufferStart
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowDown), &sup()),
        Action::BufferEnd
    );
}

#[test]
fn option_forward_delete_deletes_word_forward() {
    let mut km = KeymapState::new();
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Delete), &alt()),
        Action::DeleteWordForward
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Delete), &ctrl()),
        Action::DeleteWordForward
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Delete), &none()),
        Action::DeleteForward
    );
    assert_eq!(km.resolve(&ch("d"), &alt()), Action::InsertChar('d'));
}

#[test]
fn self_insert() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("h"), &none()), Action::InsertChar('h'));
    assert_eq!(km.resolve(&ch("Z"), &none()), Action::InsertChar('Z'));
}

#[test]
fn c_x_defaults_are_retired_but_the_prefix_machinery_survives() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix(), "C-x still arms the prefix");
    assert_eq!(
        km.resolve(&ch("s"), &ctrl()),
        Action::Cancel,
        "C-x C-s retired"
    );
    assert!(!km.in_prefix(), "the second key clears the prefix");
    for (k, m) in [
        (ch("c"), ctrl()),
        (ch("t"), none()),
        (ch("w"), none()),
        (ch("c"), none()),
        (ch("r"), none()),
        (ch("}"), none()),
        (ch("{"), none()),
        (ch("#"), none()),
        (ch("b"), none()),
        (ch("j"), none()),
        (ch("f"), ctrl()),
    ] {
        assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
        assert_eq!(km.resolve(&k, &m), Action::Cancel, "C-x second key retired");
        assert!(!km.in_prefix());
    }
    assert_eq!(km.resolve(&ch("s"), &sup()), Action::Save);
    assert_eq!(km.resolve(&ch("q"), &sup()), Action::Quit);
}

#[test]
fn native_doors_resolve() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("o"), &sup()), Action::OpenGoto);
    assert_eq!(km.resolve(&ch("O"), &sup()), Action::OpenGoto);
    assert_eq!(km.resolve(&ch("n"), &sup()), Action::NewDocument);
    assert_eq!(km.resolve(&ch("t"), &sup()), Action::OpenThemeMenu);
    assert_eq!(km.resolve(&ch("q"), &sup()), Action::Quit);
    assert_eq!(km.resolve(&ch("P"), &sup_shift()), Action::OpenProject);
    assert_eq!(km.resolve(&ch("p"), &sup_shift()), Action::OpenProject);
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Tab), &ctrl()),
        Action::LastBuffer
    );
    for c in ["o", "n", "t", "q"] {
        assert_eq!(
            km.resolve(&ch(c), &none()),
            Action::InsertChar(c.chars().next().unwrap())
        );
    }
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Tab), &none()),
        Action::InsertTab
    );
    // None is a motion or an edit (palette-eligible, undo-neutral).
    for a in [
        Action::OpenGoto,
        Action::NewDocument,
        Action::OpenThemeMenu,
        Action::OpenProject,
        Action::LastBuffer,
    ] {
        assert!(!a.is_motion());
        assert!(!a.is_edit());
    }
}

#[test]
fn cmd_w_finishes_file_and_cmd_comma_opens_settings() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("w"), &sup()), Action::FinishBuffer);
    assert_eq!(km.resolve(&ch("W"), &sup()), Action::FinishBuffer);
    assert_eq!(km.resolve(&ch(","), &sup()), Action::OpenSettingsMenu);
    assert_eq!(km.resolve(&ch("w"), &none()), Action::InsertChar('w'));
    assert_eq!(km.resolve(&ch(","), &none()), Action::InsertChar(','));
    for a in [Action::FinishBuffer, Action::OpenSettingsMenu] {
        assert!(!a.is_motion());
        assert!(!a.is_edit());
    }
}

#[test]
fn cmd_period_cancels_quietly() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("."), &sup()), Action::Cancel);
    assert_eq!(
        km.resolve(&ch("."), &sup_shift()),
        Action::Ignore,
        "Cmd-Shift-. is unbound"
    );
    assert_eq!(
        km.resolve(&ch(">"), &sup_shift()),
        Action::Ignore,
        "Cmd-Shift-. is unbound"
    );
    assert_eq!(km.resolve(&ch("."), &none()), Action::InsertChar('.'));
}

#[test]
fn cmd_shift_l_toggles_task_list() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("L"), &sup_shift()), Action::ToggleTaskList);
    assert_eq!(km.resolve(&ch("l"), &sup_shift()), Action::ToggleTaskList);
    assert_eq!(
        km.resolve(&ch("l"), &sup()),
        Action::Ignore,
        "plain Cmd-L stays unbound"
    );
    assert_eq!(km.resolve(&ch("l"), &none()), Action::InsertChar('l'));
    assert!(!Action::ToggleTaskList.is_motion());
    assert!(Action::ToggleTaskList.is_edit());
}

#[test]
fn insert_date_default_chords_both_conventions() {
    let mut km_mac = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(km_mac.resolve(&ch("d"), &sup_shift()), Action::InsertDate);
    assert_eq!(
        km_mac.resolve(&ch("D"), &sup_shift()),
        Action::InsertDate,
        "case-folded"
    );
    // Plain 'd' (no Super) still self-inserts — the chord doesn't shadow
    // ordinary typing — and Cmd-D alone (no Shift) stays unbound (no command
    // has ever claimed it; the unbound-super guard swallows it, never types).
    assert_eq!(km_mac.resolve(&ch("d"), &none()), Action::InsertChar('d'));
    assert_eq!(
        km_mac.resolve(&ch("d"), &sup()),
        Action::Ignore,
        "plain Cmd-D stays unbound"
    );

    let mut km_linux = KeymapState::new_with_convention(Convention::Linux);
    let ctrl_shift = mods(ModifiersState::CONTROL | ModifiersState::SHIFT);
    assert_eq!(km_linux.resolve(&ch("d"), &ctrl_shift), Action::InsertDate);
    assert_eq!(
        km_linux.resolve(&ch("d"), &ctrl()),
        Action::DeleteForward,
        "bare Ctrl-D keeps its own meaning"
    );

    let mut km = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(km.resolve(&ch("c"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix(), "C-c arms the prefix");
    assert_eq!(km.resolve(&ch("."), &none()), Action::InsertDate);
    assert!(!km.in_prefix(), "the second key clears the prefix");

    // On Linux, `C-c .` is quietly displaced by native Copy — Ctrl-C now
    // resolves straight to `CopyRegion` instead of arming the prefix, so
    // `.` never reaches the `C-c` map (exactly Follow link's own `C-c C-o`
    // situation). Insert Date is never fully unbound there, unlike Follow
    // link, because its native Ctrl-Shift-D slot above still fires.
    let mut km_linux2 = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(km_linux2.resolve(&ch("c"), &ctrl()), Action::CopyRegion);
    assert!(
        !km_linux2.in_prefix(),
        "native Copy wins outright, the prefix never arms"
    );

    assert!(!Action::InsertDate.is_motion());
    assert!(
        !Action::InsertDate.is_edit(),
        "InsertDate only signals an Effect; the live insert isn't dispatched here"
    );
}

#[test]
fn cmd_k_opens_insert_link() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("k"), &sup()), Action::InsertLink);
    assert_eq!(km.resolve(&ch("K"), &sup()), Action::InsertLink);
    assert_eq!(km.resolve(&ch("k"), &none()), Action::InsertChar('k'));
    assert!(!Action::InsertLink.is_motion());
    assert!(!Action::InsertLink.is_edit());
}

#[test]
fn cmd_g_aliases_search_forward_and_backward() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("g"), &sup()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("G"), &sup()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("G"), &sup_shift()), Action::SearchBackward);
    assert_eq!(km.resolve(&ch("g"), &sup_shift()), Action::SearchBackward);
    assert_eq!(km.resolve(&ch("g"), &none()), Action::InsertChar('g'));
    assert_eq!(km.resolve(&ch("g"), &ctrl()), Action::Cancel);
    assert_eq!(km.resolve(&ch("g"), &sup_alt()), Action::Ignore);
}

#[test]
fn c_c_prefix_follows_link() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("c"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix(), "C-c arms the prefix");
    assert_eq!(km.resolve(&ch("o"), &ctrl()), Action::FollowLink);
    assert!(!km.in_prefix(), "the second key clears the prefix");

    assert_eq!(km.resolve(&ch("c"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("z"), &ctrl()), Action::Cancel);
    assert!(!km.in_prefix());
}

fn shift() -> Modifiers {
    mods(ModifiersState::SHIFT)
}

#[test]
fn cmd_p_opens_command_palette() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("p"), &sup()), Action::OpenCommandPalette);
    assert_eq!(km.resolve(&ch("P"), &sup_shift()), Action::OpenProject);
    assert!(!Action::OpenCommandPalette.is_motion());
    assert!(!Action::OpenCommandPalette.is_edit());
    assert_eq!(km.resolve(&ch("p"), &ctrl()), Action::PreviousLine);
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("p"), &none()), Action::Cancel);
}

#[test]
fn cmd_shift_o_toggles_outline_and_plain_cmd_o_goes_to_file() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("O"), &sup_shift()), Action::ToggleOutline);
    assert_eq!(km.resolve(&ch("o"), &sup_shift()), Action::ToggleOutline);
    assert_eq!(km.resolve(&ch("o"), &sup()), Action::OpenGoto);
    assert_eq!(km.resolve(&ch("o"), &none()), Action::InsertChar('o'));
    assert!(!Action::ToggleOutline.is_motion() && !Action::ToggleOutline.is_edit());
}

#[test]
fn cmd_shift_h_opens_history() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("H"), &sup_shift()), Action::OpenHistory);
    assert_eq!(km.resolve(&ch("h"), &sup_shift()), Action::OpenHistory);
    // Plain Cmd-H (no Shift) is NOT the timeline — Shift is required, and it is
    // NOT self-insert either: an unbound Super chord is a calm no-op (the
    // unbound-super swallow guard), never a typed 'h'.
    assert_eq!(km.resolve(&ch("h"), &sup()), Action::Ignore);
    assert_eq!(km.resolve(&ch("h"), &none()), Action::InsertChar('h'));
    assert!(!Action::OpenHistory.is_motion());
    assert!(!Action::OpenHistory.is_edit());
}

#[test]
fn unbound_super_chords_are_calm_noops() {
    // THE UNBOUND-SUPER SWALLOW GUARD (keybinding audit, 2026-07-09): on macOS
    // an unhandled Cmd combo is inert (at most a beep) — it never types its
    // letter into the document. Every letter/symbol with no default Cmd
    // binding must resolve to Ignore, never InsertChar. 'k' is NO LONGER on
    // this list — LINKS V2 spent Cmd-K on `Action::InsertLink` (see
    // `cmd_k_opens_insert_link`); it is proven bound elsewhere, not unbound
    // here. 'l' likewise stays unbound PLAIN (only Cmd-Shift-L, task list, is
    // bound — see `cmd_shift_l_toggles_task_list`).
    let mut km = KeymapState::new();
    for c in ['d', 'j', 'l', 'u', 'm', 'h'] {
        assert_eq!(
            km.resolve(&ch(&c.to_string()), &sup()),
            Action::Ignore,
            "Cmd-{c} is unbound and must be a calm no-op, not self-insert"
        );
    }
    assert_eq!(km.resolve(&ch("'"), &sup()), Action::Ignore);
    assert_eq!(km.resolve(&ch("k"), &sup_alt()), Action::Ignore);
    assert_eq!(
        km.resolve(
            &ch("h"),
            &mods(ModifiersState::SUPER | ModifiersState::CONTROL)
        ),
        Action::Ignore
    );
    let keys = vec![("go_to".to_string(), vec!["Cmd-k".to_string()])];
    let mut km_bound = KeymapState::with_overrides(&keys);
    assert_eq!(km_bound.resolve(&ch("k"), &sup()), Action::OpenGoto);
}

#[test]
fn bare_control_unbound_was_already_a_calm_noop_and_still_is() {
    let mut km = KeymapState::new();
    for c in ['h', 'j', 'l', 'm', 'o', 't', 'u', 'z'] {
        assert_eq!(
            km.resolve(&ch(&c.to_string()), &ctrl()),
            Action::Ignore,
            "C-{c} is unbound and must stay a calm no-op"
        );
    }
    // Plain Option-composed letters keep inserting — typing (dead keys,
    // em-dash, bullet) must never be swallowed by either guard.
    assert_eq!(km.resolve(&ch("g"), &alt()), Action::InsertChar('g'));
}

#[test]
fn retired_c_x_actions_stay_undo_neutral_non_motions() {
    // The commands whose C-x default retired (caret/page toggles, page-width
    // nudgers, debug, finish) are still palette-reachable, so they must stay
    // NON-motions and NON-edits (undo-neutral) — the catalog + undo-group logic
    // rely on it even though no chord fires them by default now.
    for a in [
        Action::ToggleCaretMode,
        Action::TogglePageMode,
        Action::PageWider,
        Action::PageNarrower,
        Action::ToggleDebug,
        Action::FinishBuffer,
        Action::OpenBrowse,
        Action::MoveFile,
    ] {
        assert!(!a.is_motion(), "{a:?} must not be a motion");
        assert!(!a.is_edit(), "{a:?} must not be an edit");
    }
}

#[test]
fn cmd_backspace_deletes_to_line_start() {
    // Cmd-⌫ (Super+Backspace) is the macOS-native delete-to-line-start; ⌥⌫ / C-⌫
    // stay word-delete. It is an edit (mutates + records undo), not a motion.
    let mut km = KeymapState::new();
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Backspace), &sup()),
        Action::DeleteToLineStart
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Backspace), &alt()),
        Action::DeleteWordBackward
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Backspace), &ctrl()),
        Action::DeleteWordBackward
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Backspace), &none()),
        Action::DeleteBackward
    );
    assert!(Action::DeleteToLineStart.is_edit());
    assert!(!Action::DeleteToLineStart.is_motion());
}

#[test]
fn cmd_shift_period_is_retired_and_unbound() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("."), &sup_shift()), Action::Ignore);
    assert_eq!(km.resolve(&ch(">"), &sup_shift()), Action::Ignore);
}

#[test]
fn option_cmd_i_summons_stats_hud_plain_cmd_i_is_italic() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("i"), &sup_alt()), Action::ShowStatsHud);
    assert_eq!(km.resolve(&ch("I"), &sup_alt()), Action::ShowStatsHud);
    assert_eq!(km.resolve(&ch("i"), &sup()), Action::Italic);
    assert_eq!(km.resolve(&ch("I"), &sup()), Action::Italic);
    assert_eq!(km.resolve(&ch("i"), &none()), Action::InsertChar('i'));
    // ShowStatsHud is neither a motion nor an edit (hold-only, undo-neutral);
    // Italic is an edit, not a motion.
    assert!(!Action::ShowStatsHud.is_motion());
    assert!(!Action::ShowStatsHud.is_edit());
    assert!(Action::Italic.is_edit());
    assert!(!Action::Italic.is_motion());
}

#[test]
fn cmd_b_i_e_are_the_universal_bold_italic_inline_code_trio() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("b"), &sup()), Action::Bold);
    assert_eq!(km.resolve(&ch("B"), &sup()), Action::Bold);
    assert_eq!(km.resolve(&ch("i"), &sup()), Action::Italic);
    assert_eq!(km.resolve(&ch("I"), &sup()), Action::Italic);
    assert_eq!(km.resolve(&ch("e"), &sup()), Action::InlineCode);
    assert_eq!(km.resolve(&ch("E"), &sup()), Action::InlineCode);
    assert_eq!(km.resolve(&ch("b"), &none()), Action::InsertChar('b'));
    assert_eq!(km.resolve(&ch("i"), &none()), Action::InsertChar('i'));
    assert_eq!(km.resolve(&ch("e"), &none()), Action::InsertChar('e'));
    assert!(Action::Bold.is_edit());
    assert!(Action::Italic.is_edit());
    assert!(Action::InlineCode.is_edit());
    assert!(!Action::Bold.is_motion());
    assert!(!Action::Italic.is_motion());
    assert!(!Action::InlineCode.is_motion());
}

#[test]
fn cmd_a_selects_all() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("a"), &sup()), Action::SelectAll);
    assert_eq!(km.resolve(&ch("A"), &sup()), Action::SelectAll);
    assert_eq!(km.resolve(&ch("A"), &sup_shift()), Action::SelectAll);
    assert_eq!(km.resolve(&ch("a"), &ctrl()), Action::LineStart);
    assert_eq!(km.resolve(&ch("a"), &none()), Action::InsertChar('a'));
    assert!(!Action::SelectAll.is_motion());
    assert!(!Action::SelectAll.is_edit());
}

#[test]
fn c_x_then_unknown_cancels() {
    let mut km = KeymapState::new();
    km.resolve(&ch("x"), &ctrl());
    assert_eq!(km.resolve(&ch("z"), &none()), Action::Cancel);
    assert!(!km.in_prefix());
}

#[test]
fn c_x_then_super_combo_cancels_and_clears_prefix() {
    // A Cmd/Super combo pressed MID-PREFIX is an undefined `C-x <combo>`: it
    // must CANCEL and clear the prefix, NOT fire its global Cmd shortcut while
    // leaving the prefix armed (which would swallow the next key as a C-x
    // second key — a stuck-prefix bug).
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix());
    assert_eq!(km.resolve(&ch("v"), &sup()), Action::Cancel);
    assert!(!km.in_prefix());
    assert_eq!(km.resolve(&ch("a"), &none()), Action::InsertChar('a'));
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("z"), &sup()), Action::Cancel);
    assert!(!km.in_prefix());
    assert_eq!(km.resolve(&ch("v"), &sup()), Action::Yank);
    assert_eq!(km.resolve(&ch("z"), &sup()), Action::Undo);
}

#[test]
fn region_bindings() {
    let mut km = KeymapState::new();
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Space), &ctrl()),
        Action::SetMark
    );
    assert_eq!(km.resolve(&ch("w"), &ctrl()), Action::KillRegion);
    assert_eq!(km.resolve(&ch("w"), &alt()), Action::InsertChar('w'));
    assert_eq!(km.resolve(&ch("c"), &sup()), Action::CopyRegion);
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Space), &none()),
        Action::InsertChar(' ')
    );
}

#[test]
fn page_scroll_bindings() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("v"), &ctrl()), Action::PageScrollDown);
    assert_eq!(km.resolve(&ch("v"), &alt()), Action::InsertChar('v'));
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::PageDown), &none()),
        Action::PageScrollDown
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::PageUp), &none()),
        Action::PageScrollUp
    );
}

#[test]
fn zoom_bindings_super() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("="), &sup()), Action::ZoomIn);
    assert_eq!(km.resolve(&ch("+"), &sup()), Action::ZoomIn);
    assert_eq!(km.resolve(&ch("-"), &sup()), Action::ZoomOut);
    assert_eq!(km.resolve(&ch("_"), &sup_shift()), Action::ZoomOut);
    assert_eq!(km.resolve(&ch("0"), &sup()), Action::ZoomReset);
    assert_eq!(km.resolve(&ch("="), &none()), Action::InsertChar('='));
}

fn sup_shift() -> Modifiers {
    mods(ModifiersState::SUPER | ModifiersState::SHIFT)
}

fn sup_alt() -> Modifiers {
    mods(ModifiersState::SUPER | ModifiersState::ALT)
}

#[test]
fn super_clipboard_aliases() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("c"), &sup()), Action::CopyRegion);
    assert_eq!(km.resolve(&ch("x"), &sup()), Action::KillRegion);
    assert_eq!(km.resolve(&ch("v"), &sup()), Action::Yank);
    assert_eq!(km.resolve(&ch("C"), &sup_shift()), Action::CopyRegion);
    assert_eq!(km.resolve(&ch("X"), &sup_shift()), Action::KillRegion);
    assert_eq!(km.resolve(&ch("V"), &sup_shift()), Action::Yank);
}

#[test]
fn super_clipboard_does_not_disturb_undo_or_zoom() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("z"), &sup()), Action::Undo);
    assert_eq!(km.resolve(&ch("Z"), &sup_shift()), Action::Redo);
    assert_eq!(km.resolve(&ch("0"), &sup()), Action::ZoomReset);
}

#[test]
fn undo_redo_bindings() {
    let mut km = KeymapState::new();
    // Cmd+Z = undo, Cmd+Shift+Z = redo (logical key is 'Z' when shifted).
    assert_eq!(km.resolve(&ch("z"), &sup()), Action::Undo);
    assert_eq!(km.resolve(&ch("Z"), &sup_shift()), Action::Redo);
    // C-/ = undo (Emacs-ish alias).
    assert_eq!(km.resolve(&ch("/"), &ctrl()), Action::Undo);
    assert_eq!(km.resolve(&ch("z"), &none()), Action::InsertChar('z'));
}

#[test]
fn edit_classification() {
    assert!(Action::InsertChar('x').is_edit());
    assert!(Action::KillLine.is_edit());
    assert!(!Action::Undo.is_edit());
    assert!(!Action::Redo.is_edit());
    assert!(!Action::ForwardChar.is_edit());
}

#[test]
fn motion_classification() {
    assert!(Action::ForwardChar.is_motion());
    assert!(Action::BufferEnd.is_motion());
    assert!(!Action::InsertChar('x').is_motion());
    assert!(!Action::KillRegion.is_motion());
    assert!(!Action::ZoomIn.is_motion());
}

#[test]
fn config_rebind_single_and_cx() {
    let keys = vec![
        ("switch_theme".to_string(), vec!["C-t".to_string()]),
        ("go_to".to_string(), vec!["C-x g".to_string()]),
    ];
    let mut km = KeymapState::with_overrides(&keys);
    assert_eq!(km.resolve(&ch("t"), &ctrl()), Action::OpenThemeMenu);
    assert_eq!(km.resolve(&ch("t"), &sup()), Action::OpenThemeMenu);
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("t"), &none()), Action::Cancel);
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("g"), &none()), Action::OpenGoto);
}

#[test]
fn config_bad_chord_keeps_default() {
    let keys = vec![("save".to_string(), vec!["C-frobnicate".to_string()])];
    let mut km = KeymapState::with_overrides(&keys);
    assert_eq!(km.resolve(&ch("s"), &sup()), Action::Save);
}

#[test]
fn empty_overrides_behave_like_default() {
    let mut km = KeymapState::with_overrides(&[]);
    assert_eq!(km.resolve(&ch("f"), &ctrl()), Action::ForwardChar);
    assert_eq!(km.resolve(&ch("t"), &ctrl()), Action::Ignore);
}

#[test]
fn native_cmd_motion_and_save_defaults() {
    let mut km = KeymapState::new();
    assert_eq!(km.resolve(&ch("s"), &sup()), Action::Save);
    assert_eq!(km.resolve(&ch("S"), &sup_shift()), Action::Save);
    let cmd_arrow = |km: &mut KeymapState, n| km.resolve(&Key::Named(n), &sup());
    assert_eq!(cmd_arrow(&mut km, NamedKey::ArrowLeft), Action::LineStart);
    assert_eq!(cmd_arrow(&mut km, NamedKey::ArrowRight), Action::LineEnd);
    assert_eq!(cmd_arrow(&mut km, NamedKey::ArrowUp), Action::BufferStart);
    assert_eq!(cmd_arrow(&mut km, NamedKey::ArrowDown), Action::BufferEnd);
    assert_eq!(km.resolve(&ch("a"), &ctrl()), Action::LineStart);
    assert_eq!(km.resolve(&ch("e"), &ctrl()), Action::LineEnd);
    assert_eq!(km.resolve(&ch("<"), &alt()), Action::InsertChar('<'));
    assert_eq!(km.resolve(&ch(">"), &alt()), Action::InsertChar('>'));
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert_eq!(km.resolve(&ch("s"), &ctrl()), Action::Cancel);
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowLeft), &none()),
        Action::BackwardChar
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowUp), &none()),
        Action::PreviousLine
    );
    assert_eq!(km.resolve(&ch("s"), &none()), Action::InsertChar('s'));
}

#[test]
fn in_prefix_tracks_the_c_x_sequence() {
    // The which-key App reads `in_prefix()` right after each resolve: it must be
    // FALSE at rest, TRUE the instant `C-x` is pressed (awaiting the second key),
    // and FALSE again once any second key resolves — the exact pending window the
    // pause timer arms over.
    let mut km = KeymapState::new();
    assert!(!km.in_prefix(), "idle: not mid-prefix");
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert!(
        km.in_prefix(),
        "after C-x: mid-prefix (pending the second key)"
    );
    assert_eq!(km.resolve(&ch("s"), &ctrl()), Action::Cancel);
    assert!(!km.in_prefix(), "after the second key: prefix cleared");
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix());
    assert_eq!(km.resolve(&ch("g"), &ctrl()), Action::Cancel);
    assert!(!km.in_prefix(), "abort clears the prefix");
}

#[test]
fn two_binding_list_resolves_both_slots() {
    let keys = vec![(
        "switch_theme".to_string(),
        vec!["s-t".to_string(), "C-t".to_string()],
    )];
    let mut km = KeymapState::with_overrides(&keys);
    assert_eq!(km.resolve(&ch("t"), &sup()), Action::OpenThemeMenu); // slot 1
    assert_eq!(km.resolve(&ch("t"), &ctrl()), Action::OpenThemeMenu); // slot 2
    // A list is CAPPED at 2: a third chord is ignored — so the M-g slot-3 override
    // is never inserted, and (the Option-letter layer being retired) Option-g just
    // self-inserts 'g'.
    let capped = vec![(
        "go_to".to_string(),
        vec!["C-x g".to_string(), "s-g".to_string(), "M-g".to_string()],
    )];
    let mut km = KeymapState::with_overrides(&capped);
    assert_eq!(km.resolve(&ch("g"), &sup()), Action::OpenGoto); // slot 2 honoured
    assert_eq!(km.resolve(&ch("g"), &alt()), Action::InsertChar('g')); // slot 3 dropped
}

#[test]
fn is_meta_chord_only_true_for_configured_option_rebinds() {
    let km = KeymapState::new();
    for c in ["f", "b", "w", "v", "d", "e", "<", ">"] {
        assert!(
            !km.is_meta_chord(&ch(c)),
            "{c:?} is no longer a built-in Meta chord"
        );
    }
    assert!(!km.is_meta_chord(&Key::Named(NamedKey::ArrowLeft)));
    let km = KeymapState::with_overrides(&[("toggle_debug".to_string(), vec!["M-q".to_string()])]);
    assert!(km.is_meta_chord(&ch("q")));
    assert!(!KeymapState::new().is_meta_chord(&ch("q")));
}

#[test]
fn named_keys() {
    let mut km = KeymapState::new();
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowLeft), &none()),
        Action::BackwardChar
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::ArrowRight), &alt()),
        Action::ForwardWord
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Enter), &none()),
        Action::Newline
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Tab), &none()),
        Action::InsertTab
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Tab), &shift()),
        Action::Outdent
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Backspace), &none()),
        Action::DeleteBackward
    );
}

fn sup_ctrl() -> Modifiers {
    mods(ModifiersState::SUPER | ModifiersState::CONTROL)
}

fn ctrl_shift() -> Modifiers {
    mods(ModifiersState::CONTROL | ModifiersState::SHIFT)
}

fn ctrl_alt() -> Modifiers {
    mods(ModifiersState::CONTROL | ModifiersState::ALT)
}

/// THE MAC-BYTE-IDENTICAL LAW: a `Convention::Mac` `KeymapState` resolves
/// EVERY chord this file's other tests already pin, identically — pinned here
/// via `new_with_convention` (rather than relying on the ambient compiled
/// target) so this law holds even if these tests ever ran on a non-mac CI
/// runner. Spot-checks the widest possible spread: undo/save/zoom/palette/
/// native-doors/search/select-all/clipboard/formatting chords, PLUS every
/// bare Ctrl+letter this round's collision table is about (which must stay
/// their ORIGINAL emacs meaning on Mac, since `native_down` never claims Ctrl
/// there).
#[test]
fn mac_convention_is_byte_identical_to_the_pre_round_table() {
    let mut km = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(km.resolve(&ch("z"), &sup()), Action::Undo);
    assert_eq!(km.resolve(&ch("s"), &sup()), Action::Save);
    assert_eq!(km.resolve(&ch("p"), &sup()), Action::OpenCommandPalette);
    assert_eq!(km.resolve(&ch("n"), &sup()), Action::NewDocument);
    assert_eq!(km.resolve(&ch("w"), &sup()), Action::FinishBuffer);
    assert_eq!(km.resolve(&ch("f"), &sup()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("e"), &sup()), Action::InlineCode);
    assert_eq!(km.resolve(&ch("a"), &sup()), Action::SelectAll);
    assert_eq!(km.resolve(&ch("g"), &sup()), Action::SearchForward);
    assert_eq!(km.resolve(&ch("r"), &sup()), Action::OpenReplace);
    assert_eq!(km.resolve(&ch("b"), &sup()), Action::Bold);
    assert_eq!(km.resolve(&ch("c"), &sup()), Action::CopyRegion);
    assert_eq!(km.resolve(&ch("x"), &sup()), Action::KillRegion);
    assert_eq!(km.resolve(&ch("v"), &sup()), Action::Yank);
    for (letter, want) in [
        ('s', Action::SearchForward),
        ('p', Action::PreviousLine),
        ('n', Action::NextLine),
        ('w', Action::KillRegion),
        ('f', Action::ForwardChar),
        ('e', Action::LineEnd),
        ('a', Action::LineStart),
        ('g', Action::Cancel),
        ('r', Action::SearchBackward),
        ('b', Action::BackwardChar),
        ('v', Action::PageScrollDown),
    ] {
        let mut km2 = KeymapState::new_with_convention(Convention::Mac);
        assert_eq!(
            km2.resolve(&ch(&letter.to_string()), &ctrl()),
            want,
            "Ctrl-{letter} on Mac"
        );
    }
    let mut km3 = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(km3.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    let mut km4 = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(km4.resolve(&ch("c"), &ctrl()), Action::BeginPrefix);
}

/// THE FROZEN CATALOG-CHORD SNAPSHOT (structural per-command value pinning).
/// Because catalog labels and dispatch now share ONE seed
/// (`assets/keymap-defaults.toml`), the exhaustive agreement sweeps
/// (`commands::tests::catalog_and_keymap_agree_on_every_default_chord`,
/// `every_catalog_default_slot_dispatches_through_real_keymap_under_both_conventions_and_flavors`)
/// can no longer catch a wrong default CHORD — they read the same parse on
/// both sides. This table restores that guard: a checked-in literal of every
/// command's slug -> resolved chord strings across BOTH slots and BOTH
/// conventions (`native@mac | native@linux | emacs`). Adding, retyping, or
/// removing a default chord in the TOML shifts exactly one line here and
/// fails this test, forcing a conscious re-freeze — nothing about a new
/// command's chords can slip past silently.
///
/// REGENERATED DELIBERATELY, never auto-synced: run
/// `cargo test -p awl print_catalog_chord_snapshot -- --ignored --nocapture`,
/// eyeball the diff, and paste the block below (the `print_full_catalog_snapshot`
/// precedent). The manual step IS the point — an accidental chord change must
/// cost a visible, reviewed edit, not a rubber-stamp.
#[test]
fn catalog_chord_snapshot_is_frozen() {
    assert_eq!(catalog_chord_snapshot(), CATALOG_CHORD_SNAPSHOT);
}

#[test]
#[ignore]
fn print_catalog_chord_snapshot() {
    print!("{}", catalog_chord_snapshot());
}

#[test]
fn linux_collision_table_matches_the_documented_displaced_list() {
    let displaced: &[(char, Action)] = &[
        ('s', Action::Save),
        ('p', Action::OpenCommandPalette),
        ('n', Action::NewDocument),
        ('w', Action::FinishBuffer),
        ('f', Action::SearchForward),
        ('e', Action::InlineCode),
        ('a', Action::SelectAll),
        ('g', Action::SearchForward), // "find next" — same action as Cmd-G
        ('r', Action::OpenReplace),
        ('b', Action::Bold),
        ('v', Action::Yank),
    ];
    for (letter, want) in displaced {
        let mut km = KeymapState::new_with_convention(Convention::Linux);
        assert_eq!(
            km.resolve(&ch(&letter.to_string()), &ctrl()),
            *want,
            "Ctrl-{letter} on Linux must resolve to the native meaning"
        );
    }
    let mut kc = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(kc.resolve(&ch("c"), &ctrl()), Action::CopyRegion);
    let mut kx = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(kx.resolve(&ch("x"), &ctrl()), Action::KillRegion);

    let mut kk = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(kk.resolve(&ch("k"), &ctrl()), Action::KillLine);
    let mut kd = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(kd.resolve(&ch("d"), &ctrl()), Action::DeleteForward);

    let displaced_letters: Vec<char> = displaced
        .iter()
        .map(|(l, _)| *l)
        .chain(['c', 'x'])
        .collect();
    let all_bare_ctrl_letters = [
        'f', 'b', 'n', 'p', 'a', 'e', 'd', 'k', 'y', 's', 'r', 'w', 'v', 'g', 'x', 'c',
    ];
    for letter in all_bare_ctrl_letters {
        if displaced_letters.contains(&letter) {
            continue;
        }
        let mut mac = KeymapState::new_with_convention(Convention::Mac);
        let mut linux = KeymapState::new_with_convention(Convention::Linux);
        let key = ch(&letter.to_string());
        assert_eq!(
            mac.resolve(&key, &ctrl()),
            linux.resolve(&key, &ctrl()),
            "Ctrl-{letter} must resolve identically on both conventions (not in the displaced list)"
        );
    }

    // ONE SOURCE OF TRUTH: `LINUX_DISPLACED_LETTERS` (the label-truth owner's
    // data) must be EXACTLY this same set — sorted-and-deduped comparison so a
    // future letter added to one and not the other fails loudly here.
    let mut from_const: Vec<char> = LINUX_DISPLACED_LETTERS.to_vec();
    from_const.sort_unstable();
    let mut from_test = displaced_letters.clone();
    from_test.sort_unstable();
    from_test.dedup();
    assert_eq!(
        from_const, from_test,
        "LINUX_DISPLACED_LETTERS drifted from this test's own displaced list"
    );
}

#[test]
fn linux_displaces_emacs_default_flags_exactly_the_collision_table() {
    for emacs in ["C-s", "C-r", "C-w", "C-a", "C-e"] {
        assert!(
            linux_displaces_emacs_default(emacs, &[]),
            "{emacs:?} should be displaced"
        );
    }
    // A prefix sequence whose FIRST key collides (Follow link's "C-c C-o":
    // Ctrl-C now resolves straight to Copy, so the sequence never arms).
    assert!(linux_displaces_emacs_default("C-c C-o", &[]));
    assert!(!linux_displaces_emacs_default("C-/", &[])); // Undo's emacs slot
    assert!(!linux_displaces_emacs_default("C-y", &[])); // Paste's emacs slot — 'y' is not claimed
    // ...a bare Ctrl letter NOT in the displaced set — Ctrl-D (never claimed)
    // and Ctrl-K (Links v2 spent Cmd-K, but `linux_builtin_keep()` keeps kill-
    // line unconditionally, so it's not on `LINUX_DISPLACED_LETTERS` at all;
    // see the collision-table doc above the keep helpers)...
    assert!(!linux_displaces_emacs_default("C-d", &[]));
    assert!(!linux_displaces_emacs_default("C-k", &[]));
    assert!(!linux_displaces_emacs_default("", &[]));
    assert!(!linux_displaces_emacs_default("   ", &[]));
}

#[test]
fn linux_displaces_emacs_default_respects_the_keep_list() {
    let keep = vec!["C-f".to_string(), "Ctrl-b".to_string()];
    assert!(!linux_displaces_emacs_default("C-f", &keep), "C-f is kept");
    assert!(
        !linux_displaces_emacs_default("C-b", &keep),
        "C-b is kept via an equivalent spelling"
    );
    assert!(
        linux_displaces_emacs_default("C-s", &keep),
        "C-s is not in the keep list"
    );
    assert!(
        linux_displaces_emacs_default("C-n", &keep),
        "C-n is not in the keep list"
    );
}

#[test]
fn linux_keep_emacs_restores_dispatch_for_kept_chords_only() {
    let keep = vec![
        "C-f".to_string(),
        "C-b".to_string(),
        "C-n".to_string(),
        "C-p".to_string(),
        "C-a".to_string(),
        "C-e".to_string(),
    ];
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&keep);
    assert_eq!(
        km.resolve(&ch("f"), &ctrl()),
        Action::ForwardChar,
        "C-f kept"
    );
    assert_eq!(
        km.resolve(&ch("b"), &ctrl()),
        Action::BackwardChar,
        "C-b kept"
    );
    assert_eq!(km.resolve(&ch("n"), &ctrl()), Action::NextLine, "C-n kept");
    assert_eq!(
        km.resolve(&ch("p"), &ctrl()),
        Action::PreviousLine,
        "C-p kept"
    );
    assert_eq!(km.resolve(&ch("a"), &ctrl()), Action::LineStart, "C-a kept");
    assert_eq!(km.resolve(&ch("e"), &ctrl()), Action::LineEnd, "C-e kept");
    assert_eq!(
        km.resolve(&ch("c"), &ctrl()),
        Action::CopyRegion,
        "C-c not kept: native still wins"
    );

    let mut plain = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(plain.resolve(&ch("f"), &ctrl()), Action::SearchForward);
    assert_eq!(plain.resolve(&ch("n"), &ctrl()), Action::NewDocument);

    // MAC IGNORES THE LIST ENTIRELY (the law): a keep-listed chord under
    // `Convention::Mac` resolves exactly as it would with an empty list —
    // Ctrl-F on Mac was never a native chord to begin with (Mac's native
    // layer speaks Cmd, not Ctrl), so it's ForwardChar regardless.
    let mut mac_kept = KeymapState::new_with_convention(Convention::Mac);
    mac_kept.apply_linux_keep(&keep);
    let mut mac_plain = KeymapState::new_with_convention(Convention::Mac);
    for letter in ['f', 'b', 'n', 'p', 'a', 'e'] {
        let key = ch(&letter.to_string());
        assert_eq!(
            mac_kept.resolve(&key, &ctrl()),
            mac_plain.resolve(&key, &ctrl()),
            "Ctrl-{letter} on Mac must be unaffected by a non-empty linux_keep_emacs list"
        );
    }
}

/// A bad/unsupported `linux_keep_emacs` entry (a two-chord `C-x`/`C-c`
/// prefix spec, or outright garbage) is reported + skipped — never a crash,
/// never poisoning the rest of the list.
#[test]
fn linux_keep_emacs_bad_entry_is_skipped_not_a_crash() {
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&[
        "C-x g".to_string(),
        "C-frobnicate".to_string(),
        "C-f".to_string(),
    ]);
    assert_eq!(km.resolve(&ch("f"), &ctrl()), Action::ForwardChar);
    // ...and a fresh C-x still arms the ordinary bare prefix (the bad
    // "C-x g" entry never reached the keep-set).
    assert_eq!(
        km.resolve(&ch("x"), &ctrl()),
        Action::KillRegion,
        "C-x is not itself kept"
    );
}

/// A live config RELOAD re-applies the keep-list exactly like
/// `apply_overrides` re-applies `[keys]` — a later `apply_linux_keep` call
/// clears the prior set first, never accumulating stale entries.
#[test]
fn apply_linux_keep_reload_replaces_not_accumulates() {
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&["C-f".to_string()]);
    assert_eq!(km.resolve(&ch("f"), &ctrl()), Action::ForwardChar);
    assert_eq!(
        km.resolve(&ch("n"), &ctrl()),
        Action::NewDocument,
        "C-n not yet kept"
    );
    km.apply_linux_keep(&["C-n".to_string()]);
    assert_eq!(
        km.resolve(&ch("f"), &ctrl()),
        Action::SearchForward,
        "C-f reverted on reload"
    );
    assert_eq!(
        km.resolve(&ch("n"), &ctrl()),
        Action::NextLine,
        "C-n now kept"
    );
}

/// THE NO-DRIFT LAW: [`linux_emacs_preset_keep`] is derived FROM
/// [`LINUX_DISPLACED_LETTERS`] itself — exactly one `"C-<letter>"` chord per
/// displaced letter, no more, no less. A future letter added to (or removed
/// from) the displaced table flows into the preset automatically; this test
/// pins that the two can never silently diverge.
#[test]
fn linux_emacs_preset_keep_equals_the_displaced_letters_no_drift() {
    let preset = linux_emacs_preset_keep();
    assert_eq!(preset.len(), LINUX_DISPLACED_LETTERS.len());
    for letter in LINUX_DISPLACED_LETTERS {
        let want = format!("C-{letter}");
        assert!(
            preset.contains(&want),
            "preset missing {want:?} for displaced letter {letter:?}"
        );
    }
    for chord in &preset {
        assert!(
            LINUX_DISPLACED_LETTERS
                .iter()
                .any(|l| *chord == format!("C-{l}")),
            "preset chord {chord:?} has no matching displaced letter"
        );
    }
}

/// THE KEYMAP FLAVOR ROUND — the actual DISPATCH half, RE-PINNED to item
/// 457's real composition: `Config::effective_linux_keep()` under `keymap =
/// "emacs"`, never the raw `linux_emacs_preset_keep()` fed straight into
/// `apply_linux_keep` (that would still exercise the PRE-457 shape, since the
/// native-clipboard carve-out lives only in `effective_linux_keep`'s own
/// filtering). Every letter [`LINUX_DISPLACED_LETTERS`] names reverts to its
/// emacs meaning under `Convention::Linux` — not just a hand-picked few, ALL
/// of them, swept — EXCEPT `c`/`v`: those two are the native-clipboard
/// carve-out, so they resolve to Copy/Paste instead of Mac's untouched emacs
/// reading. Every OTHER letter's resolution matches EXACTLY what the SAME
/// bare Ctrl-letter resolves to under `Convention::Mac` (where Ctrl never
/// carries a native meaning at all), so this test doubles as "the flavor
/// preset makes Linux behave like Mac's Ctrl reading, letter for letter,
/// minus the two carved-out clipboard letters".
#[test]
fn keymap_flavor_emacs_preset_reverts_every_displaced_chord_to_emacs_meaning() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let keep = cfg.effective_linux_keep();
    for letter in LINUX_DISPLACED_LETTERS {
        let key = ch(&letter.to_string());
        let mut linux_kept = KeymapState::new_with_convention(Convention::Linux);
        linux_kept.apply_linux_keep(&keep);
        let got = linux_kept.resolve(&key, &ctrl());
        match letter {
            'c' => assert_eq!(
                got,
                Action::CopyRegion,
                "C-c: the native-clipboard carve-out — Copy, not the emacs prefix"
            ),
            'v' => assert_eq!(
                got,
                Action::Yank,
                "C-v: the native-clipboard carve-out — Paste, not page-down"
            ),
            _ => {
                let mut mac_reference = KeymapState::new_with_convention(Convention::Mac);
                let want = mac_reference.resolve(&key, &ctrl());
                assert_eq!(
                    got, want,
                    "Ctrl-{letter} under the emacs flavor preset should match \
                     Mac's untouched emacs meaning"
                );
            }
        }
    }
    let mut nav = KeymapState::new_with_convention(Convention::Linux);
    nav.apply_linux_keep(&keep);
    assert_eq!(
        nav.resolve(&ch("f"), &ctrl()),
        Action::ForwardChar,
        "C-f nav"
    );
    let mut isearch = KeymapState::new_with_convention(Convention::Linux);
    isearch.apply_linux_keep(&keep);
    assert_eq!(
        isearch.resolve(&ch("s"), &ctrl()),
        Action::SearchForward,
        "C-s isearch"
    );
    let mut cancel = KeymapState::new_with_convention(Convention::Linux);
    cancel.apply_linux_keep(&keep);
    assert_eq!(
        cancel.resolve(&ch("g"), &ctrl()),
        Action::Cancel,
        "C-g cancel"
    );
    let mut xprefix = KeymapState::new_with_convention(Convention::Linux);
    xprefix.apply_linux_keep(&keep);
    assert_eq!(
        xprefix.resolve(&ch("x"), &ctrl()),
        Action::BeginPrefix,
        "C-x: NOT carved out — still the emacs prefix (Save/Open outrank the clipboard trade)"
    );
}

/// A chord OUTSIDE the displaced set is UNCHANGED by the emacs flavor
/// preset — it was never displaced to begin with, so keeping it is a
/// no-op, not a second policy layer. 'd'/'y' are never claimed by any
/// native command at all; 'k' is a DIFFERENT flavor of "outside the
/// preset" — it IS native-claimed (Links v2's Cmd-K), but
/// `linux_builtin_keep()`'s unconditional floor already keeps it before the
/// preset ever gets applied, so applying (or not applying) the preset
/// makes no observable difference to it either.
#[test]
fn keymap_flavor_emacs_preset_is_a_no_op_for_non_displaced_chords() {
    let preset = linux_emacs_preset_keep();
    let mut plain = KeymapState::new_with_convention(Convention::Linux);
    let mut kept = KeymapState::new_with_convention(Convention::Linux);
    kept.apply_linux_keep(&preset);
    for letter in ['k', 'd', 'y'] {
        let key = ch(&letter.to_string());
        assert_eq!(
            plain.resolve(&key, &ctrl()),
            kept.resolve(&key, &ctrl()),
            "Ctrl-{letter} (never displaced) must be unaffected by the emacs preset"
        );
    }
}

#[test]
fn config_keys_override_wins_over_the_emacs_preset() {
    let preset = linux_emacs_preset_keep();
    let mut km = KeymapState::with_overrides_and_convention(
        &[("copy".to_string(), vec!["C-c".to_string()])],
        Convention::Linux,
    );
    km.apply_linux_keep(&preset);
    assert_eq!(
        km.resolve(&ch("c"), &ctrl()),
        Action::CopyRegion,
        "[keys] override wins over the preset"
    );
}

/// LAW: a `[keys]` rebind wins over the native-clipboard carve-out's own
/// default — in BOTH directions, independently. Since `c`/`v` already resolve
/// to Copy/Paste BY DEFAULT under `keymap = "emacs"`, a `[keys] copy = "C-c"`
/// line would prove nothing (the default already agrees) — so each case
/// rebinds the chord to a DIFFERENT action (`fold_section`) and asserts the
/// override, not the default, is what fires, while the OTHER carved-out
/// letter stays on its untouched default.
#[test]
fn config_keys_override_wins_over_the_native_clipboard_carve_out_reclaiming_c() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    cfg.keys
        .push(("fold_section".to_string(), vec!["C-c".to_string()]));
    let mut km = KeymapState::with_overrides_and_convention(&cfg.keys, Convention::Linux);
    km.apply_linux_keep(&cfg.effective_linux_keep());
    assert_eq!(
        km.resolve(&ch("c"), &ctrl()),
        Action::ToggleFold,
        "[keys] reclaiming C-c must win over the carve-out's own native-Copy default"
    );
    assert_eq!(
        km.resolve(&ch("v"), &ctrl()),
        Action::Yank,
        "reclaiming C-c must not disturb C-v's own untouched default"
    );
}

#[test]
fn config_keys_override_wins_over_the_native_clipboard_carve_out_reclaiming_v() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    cfg.keys
        .push(("fold_section".to_string(), vec!["C-v".to_string()]));
    let mut km = KeymapState::with_overrides_and_convention(&cfg.keys, Convention::Linux);
    km.apply_linux_keep(&cfg.effective_linux_keep());
    assert_eq!(
        km.resolve(&ch("v"), &ctrl()),
        Action::ToggleFold,
        "[keys] reclaiming C-v must win over the carve-out's own native-Paste default"
    );
    assert_eq!(
        km.resolve(&ch("c"), &ctrl()),
        Action::CopyRegion,
        "reclaiming C-v must not disturb C-c's own untouched default"
    );
}

/// LAW: under `keymap = "emacs"` on Linux, bare Ctrl-C copies, Ctrl-V pastes,
/// and Ctrl-X still begins the emacs prefix — the three concrete carve-out
/// outcomes, each pinned by its own small law so a regression in any one
/// fails BY NAME rather than only inside the swept roster test above.
#[test]
fn linux_emacs_flavor_c_copies() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&cfg.effective_linux_keep());
    assert_eq!(km.resolve(&ch("c"), &ctrl()), Action::CopyRegion);
    assert!(
        !km.in_prefix(),
        "native Copy wins outright — the C-c prefix never arms"
    );
}

#[test]
fn linux_emacs_flavor_v_pastes() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&cfg.effective_linux_keep());
    assert_eq!(km.resolve(&ch("v"), &ctrl()), Action::Yank);
}

#[test]
fn linux_emacs_flavor_x_still_begins_a_prefix() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.apply_linux_keep(&cfg.effective_linux_keep());
    assert_eq!(km.resolve(&ch("x"), &ctrl()), Action::BeginPrefix);
    assert!(km.in_prefix(), "C-x still arms the emacs prefix");
}

/// LAW: the classic Meta layer — every entry in
/// `platform::LINUX_EMACS_META_SEED` dispatches to its own named `Action`
/// under `Convention::Linux` with the gate set, swept over the WHOLE table
/// rather than a hand-picked chord.
#[test]
fn linux_emacs_meta_layer_dispatches_every_seeded_chord() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let keep = cfg.effective_linux_keep();
    for (spec, want) in LINUX_EMACS_META_SEED {
        let mut km = KeymapState::new_with_convention(Convention::Linux);
        km.apply_linux_keep(&keep);
        km.set_linux_emacs_meta(true);
        let (key, mods) =
            crate::keyspec::parse_chord(spec).unwrap_or_else(|e| panic!("{spec:?}: {e}"));
        assert_eq!(
            km.resolve(&key, &mods),
            *want,
            "seeded Meta chord {spec:?} must dispatch to its own named action"
        );
    }
}

/// LAW: the classic `C-x` continuations — every entry in
/// `platform::LINUX_EMACS_CLASSIC_SEED` — dispatch to their own named
/// `Action` under `Convention::Linux` with the gate set, via the REAL
/// two-key sequence (arm the prefix, then resolve the second key), swept
/// over the whole table rather than a hand-picked chord.
#[test]
fn linux_emacs_classic_seed_dispatches_every_seeded_cx_chord() {
    assert!(
        !LINUX_EMACS_CLASSIC_SEED.is_empty(),
        "non-vacuity: the classic C-x seed table must not be empty, or this sweep checks nothing"
    );
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let keep = cfg.effective_linux_keep();
    for (spec, want) in LINUX_EMACS_CLASSIC_SEED {
        let toks: Vec<&str> = spec.split_whitespace().collect();
        assert_eq!(toks.len(), 2, "{spec:?} must be a two-token C-x sequence");
        let mut km = KeymapState::new_with_convention(Convention::Linux);
        km.apply_linux_keep(&keep);
        km.set_linux_emacs_meta(true);
        let (px, pm) =
            crate::keyspec::parse_chord(toks[0]).unwrap_or_else(|e| panic!("{spec:?}: {e}"));
        assert_eq!(
            km.resolve(&px, &pm),
            Action::BeginPrefix,
            "{spec:?}: the first key must arm the C-x prefix"
        );
        assert!(
            km.in_prefix(),
            "{spec:?}: the prefix must be pending after the first key"
        );
        let (kx, km2) =
            crate::keyspec::parse_chord(toks[1]).unwrap_or_else(|e| panic!("{spec:?}: {e}"));
        assert_eq!(
            km.resolve(&kx, &km2),
            *want,
            "seeded classic chord {spec:?} must dispatch to its own action"
        );
    }
}

/// LAW: a config `[keys]` override for a seeded action's OWN command still
/// outranks the seed — same precedence `KeymapState::resolve` already gives
/// every override over every default, exercised here specifically over a
/// classic `C-x` seed (`Save`'s `C-x C-s`) and the Meta seed (Command
/// palette's `M-x`), so the override door this round threads seeds through
/// is proven, not assumed.
#[test]
fn keys_override_outranks_a_seeded_chord() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let keep = cfg.effective_linux_keep();

    // Rebind "C-x C-s" itself to a different action (Quit) — the override
    // must fire instead of the seeded Save.
    let overrides = vec![("quit".to_string(), vec!["C-x C-s".to_string()])];
    let mut km = KeymapState::with_overrides_and_convention(&overrides, Convention::Linux);
    km.apply_linux_keep(&keep);
    km.set_linux_emacs_meta(true);
    let (px, pm) = crate::keyspec::parse_chord("C-x").unwrap();
    assert_eq!(km.resolve(&px, &pm), Action::BeginPrefix);
    let (kx, kmods) = crate::keyspec::parse_chord("C-s").unwrap();
    assert_eq!(
        km.resolve(&kx, &kmods),
        Action::Quit,
        "a [keys] override on the exact seeded chord must win over the seed"
    );

    // A single-key seed (M-x -> Command palette): reclaim M-x for a
    // different action (Redo) — the override must fire instead of the seed.
    let overrides2 = vec![("redo".to_string(), vec!["M-x".to_string()])];
    let mut km2 = KeymapState::with_overrides_and_convention(&overrides2, Convention::Linux);
    km2.apply_linux_keep(&keep);
    km2.set_linux_emacs_meta(true);
    let (mx, mmods) = crate::keyspec::parse_chord("M-x").unwrap();
    assert_eq!(
        km2.resolve(&mx, &mmods),
        Action::Redo,
        "a [keys] override on the exact seeded Meta chord must win over the seed"
    );
}

/// LAW: the Meta layer is OFF by default (flavor = native) even on Linux —
/// it is seeded only under `keymap = "emacs"`, never unconditionally. A chord
/// already reachable via some OTHER, gate-independent rule (`M-Backspace` —
/// `resolve_named`'s generic Alt+Backspace-deletes-word arm, unrelated to
/// this seed table) is skipped: a baseline `KeymapState` that never sees the
/// gate already resolves it to the SAME action, so the gate made no
/// observable difference there and it is not evidence either way.
/// Non-vacuity: at most ONE entry may be skipped this way, or the sweep is
/// checking nothing.
#[test]
fn linux_native_flavor_never_seeds_the_meta_layer() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("native".to_string());
    let keep = cfg.effective_linux_keep();
    let mut checked = 0usize;
    for (spec, want) in LINUX_EMACS_META_SEED {
        let (key, mods) = crate::keyspec::parse_chord(spec).unwrap();
        let mut baseline = KeymapState::new_with_convention(Convention::Linux);
        if baseline.resolve(&key, &mods) == *want {
            continue;
        }
        checked += 1;
        let mut native = KeymapState::new_with_convention(Convention::Linux);
        native.apply_linux_keep(&keep);
        assert_ne!(
            native.resolve(&key, &mods),
            *want,
            "{spec:?} must not seed under the native flavor"
        );
    }
    assert!(
        checked + 1 >= LINUX_EMACS_META_SEED.len(),
        "at most one entry may be skipped as gate-independent; {checked} of \
         {} were actually exercised — the sweep is checking too little",
        LINUX_EMACS_META_SEED.len()
    );
}

/// LAW: the Meta layer stays structurally INERT on Mac even if a caller
/// mistakenly sets the gate — Option keeps typing accented characters there,
/// so `Convention::Mac` must never seed it regardless of flavor. Same
/// gate-independent-chord skip and non-vacuity floor as the sibling law above.
#[test]
fn linux_emacs_meta_layer_stays_inert_on_mac() {
    let mut checked = 0usize;
    for (spec, want) in LINUX_EMACS_META_SEED {
        let (key, mods) = crate::keyspec::parse_chord(spec).unwrap();
        let mut baseline = KeymapState::new_with_convention(Convention::Mac);
        if baseline.resolve(&key, &mods) == *want {
            continue;
        }
        checked += 1;
        let mut km = KeymapState::new_with_convention(Convention::Mac);
        km.set_linux_emacs_meta(true);
        assert_ne!(
            km.resolve(&key, &mods),
            *want,
            "{spec:?} must not seed on Mac even with the gate forced true"
        );
    }
    assert!(
        checked + 1 >= LINUX_EMACS_META_SEED.len(),
        "at most one entry may be skipped as gate-independent; {checked} of \
         {} were actually exercised — the sweep is checking too little",
        LINUX_EMACS_META_SEED.len()
    );
}

/// HARD LAW (a): with an EMPTY user config, Ctrl-K resolves to Kill line on
/// Linux under BOTH keymap flavors — the user's decided outcome ("kill-line
/// is too load-bearing for emacs hands to lose by default"). Driven through
/// the REAL composition owner, `Config::effective_linux_keep`, exactly like
/// `App::new`/headless replay construct their keymap — not a bare
/// `KeymapState` with a hand-rolled list, so this is honestly "a real Linux
/// keymap with empty config", not just the primitive's own mechanics.
#[test]
fn out_of_the_box_linux_ctrl_k_is_kill_line_under_both_keymap_flavors() {
    for flavor in ["native", "emacs"] {
        let mut cfg = crate::config::Config::empty();
        cfg.keymap = Some(flavor.to_string());
        let keep = cfg.effective_linux_keep();
        let mut km = KeymapState::new_with_convention(Convention::Linux);
        km.apply_linux_keep(&keep);
        assert_eq!(
            km.resolve(&ch("k"), &ctrl()),
            Action::KillLine,
            "Ctrl-K must stay kill-line out of the box under keymap={flavor:?}"
        );
    }
}

#[test]
fn keys_override_reclaims_ctrl_k_for_insert_link_on_linux_over_the_builtin_keep() {
    let keep = crate::config::Config::empty().effective_linux_keep();
    let mut km = KeymapState::with_overrides_and_convention(
        &[("insert_link".to_string(), vec!["C-k".to_string()])],
        Convention::Linux,
    );
    km.apply_linux_keep(&keep);
    assert_eq!(
        km.resolve(&ch("k"), &ctrl()),
        Action::InsertLink,
        "[keys] override wins over the built-in keep"
    );

    let mut plain = KeymapState::new_with_convention(Convention::Linux);
    plain.apply_linux_keep(&keep);
    assert_eq!(
        plain.resolve(&ch("k"), &ctrl()),
        Action::KillLine,
        "control: without the override, kill-line wins"
    );
}

#[test]
fn linux_convention_resolves_untranslated_native_chords() {
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(km.resolve(&ch("z"), &ctrl()), Action::Undo);
    assert_eq!(km.resolve(&ch("Z"), &ctrl_shift()), Action::Redo);
    assert_eq!(km.resolve(&ch("t"), &ctrl()), Action::OpenThemeMenu);
    assert_eq!(km.resolve(&ch("o"), &ctrl()), Action::OpenGoto);
    assert_eq!(km.resolve(&ch("q"), &ctrl()), Action::Quit);
    assert_eq!(km.resolve(&ch(","), &ctrl()), Action::OpenSettingsMenu);
    assert_eq!(km.resolve(&ch(";"), &ctrl()), Action::OpenSpellSuggest);
    assert_eq!(km.resolve(&ch("i"), &ctrl()), Action::Italic);
    assert_eq!(km.resolve(&ch("i"), &ctrl_alt()), Action::ShowStatsHud);
    assert_eq!(km.resolve(&ch("l"), &ctrl_shift()), Action::ToggleTaskList);
    assert_eq!(km.resolve(&ch("h"), &ctrl_shift()), Action::OpenHistory);
    assert_eq!(km.resolve(&ch("o"), &ctrl_shift()), Action::ToggleOutline);
    assert_eq!(km.resolve(&ch("p"), &ctrl_shift()), Action::OpenProject);
    assert_eq!(km.resolve(&ch("="), &ctrl()), Action::ZoomIn);
    assert_eq!(km.resolve(&ch("0"), &ctrl()), Action::ZoomReset);
    // A SUPER-only (Windows-key) press is NOT the Linux native modifier — it
    // falls through to the unhandled-super swallow guard, staying inert
    // (never self-inserting), exactly as on Mac.
    assert_eq!(km.resolve(&ch("s"), &sup()), Action::Ignore);
    assert_eq!(km.resolve(&ch("s"), &sup_ctrl()), Action::SearchForward);
}

#[test]
fn linux_convention_buffer_start_end_use_ctrl_home_end_not_ctrl_up_down() {
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Home), &ctrl()),
        Action::BufferStart
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::End), &ctrl()),
        Action::BufferEnd
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::Home), &none()),
        Action::LineStart
    );
    assert_eq!(
        km.resolve(&Key::Named(NamedKey::End), &none()),
        Action::LineEnd
    );
    // On Mac, Ctrl-Home/End is NOT buffer start/end (that's Cmd-Up/Down there;
    // the convention gate never fires for Mac).
    let mut mac = KeymapState::new_with_convention(Convention::Mac);
    assert_eq!(
        mac.resolve(&Key::Named(NamedKey::Home), &ctrl()),
        Action::LineStart
    );
    assert_eq!(
        mac.resolve(&Key::Named(NamedKey::End), &ctrl()),
        Action::LineEnd
    );
}

/// `[keys]` overrides are CONVENTION-AGNOSTIC — a configured chord is taken
/// literally on every convention, never translated.
#[test]
fn keys_overrides_are_convention_agnostic() {
    let cfg = vec![("toggle_debug".to_string(), vec!["Cmd-J".to_string()])];
    let mut linux = KeymapState::with_overrides_and_convention(&cfg, Convention::Linux);
    assert_eq!(linux.resolve(&ch("j"), &sup()), Action::ToggleDebug);
    // And the naive Ctrl-translation of that SAME spec is NOT what fires — a
    // bare unbound Ctrl+letter is a calm `Ignore` (the ordinary emacs-branch
    // default), never a self-insert or the overridden action.
    assert_eq!(linux.resolve(&ch("j"), &ctrl()), Action::Ignore);
}

fn resolve_spec(km: &mut KeymapState, spec: &str) -> Vec<Action> {
    spec.split_whitespace()
        .map(|token| {
            let (key, mods) = crate::keyspec::parse_chord(token).unwrap_or_else(|e| {
                panic!("catalog default {spec:?} contains invalid token {token:?}: {e}")
            });
            km.resolve(&key, &mods)
        })
        .collect()
}

#[test]
fn every_catalog_default_slot_dispatches_through_real_keymap_under_both_conventions_and_flavors() {
    for command in crate::commands::COMMANDS.iter() {
        for spec in [command.native, command.emacs] {
            if spec.is_empty() {
                continue;
            }
            let mut mac = KeymapState::new_with_convention(Convention::Mac);
            let trace = resolve_spec(&mut mac, spec);
            assert_eq!(
                trace.last(),
                Some(&command.action),
                "Mac: {} {spec:?}",
                command.name
            );
            if spec.split_whitespace().count() == 2 {
                assert_eq!(
                    trace.first(),
                    Some(&Action::BeginPrefix),
                    "Mac prefix trace: {}",
                    command.name
                );
            }
        }
    }

    let native_keep: Vec<String> = linux_builtin_keep()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let mut emacs_keep = linux_emacs_preset_keep();
    emacs_keep.extend(linux_builtin_keep().iter().map(|s| (*s).to_string()));
    for command in crate::commands::COMMANDS.iter() {
        let native = crate::commands::resolved_native(command, Convention::Linux);
        if !native.is_empty() {
            let mut km = KeymapState::new_with_convention(Convention::Linux);
            km.apply_linux_keep(&native_keep);
            let actual = resolve_spec(&mut km, &native);
            if linux_keeps_chord(&native_keep, &native) {
                assert_ne!(
                    actual.last(),
                    Some(&command.action),
                    "Linux native keep must suppress {} {native:?}",
                    command.name
                );
            } else {
                assert_eq!(
                    actual.last(),
                    Some(&command.action),
                    "Linux native: {} {native:?}",
                    command.name
                );
            }

            let mut emacs_flavor = KeymapState::new_with_convention(Convention::Linux);
            emacs_flavor.apply_linux_keep(&emacs_keep);
            let actual = resolve_spec(&mut emacs_flavor, &native);
            if !linux_keeps_chord(&emacs_keep, &native) {
                assert_eq!(
                    actual.last(),
                    Some(&command.action),
                    "Linux emacs flavor non-collision: {} {native:?}",
                    command.name
                );
            }
        }

        if !command.emacs.is_empty() {
            let mut native_flavor = KeymapState::new_with_convention(Convention::Linux);
            native_flavor.apply_linux_keep(&native_keep);
            let actual = resolve_spec(&mut native_flavor, command.emacs);
            if linux_displaces_emacs_default(command.emacs, &native_keep) {
                assert_ne!(
                    actual.last(),
                    Some(&command.action),
                    "Linux native flavor must displace {} {:?}",
                    command.name,
                    command.emacs
                );
            } else {
                assert_eq!(
                    actual.last(),
                    Some(&command.action),
                    "Linux native flavor: {} {:?}",
                    command.name,
                    command.emacs
                );
            }

            let mut emacs_flavor = KeymapState::new_with_convention(Convention::Linux);
            emacs_flavor.apply_linux_keep(&emacs_keep);
            let trace = resolve_spec(&mut emacs_flavor, command.emacs);
            assert_eq!(
                trace.last(),
                Some(&command.action),
                "Linux emacs flavor: {} {:?}",
                command.name,
                command.emacs
            );
            if command.emacs.split_whitespace().count() == 2 {
                assert_eq!(
                    trace.first(),
                    Some(&Action::BeginPrefix),
                    "Linux emacs prefix trace: {}",
                    command.name
                );
            }
        }
    }
}

#[test]
#[should_panic(expected = "conflicting effective default")]
fn conflicting_embedded_defaults_fail_loudly_at_the_map_seam() {
    let mut map = HashMap::new();
    insert_default_entry(
        &mut map,
        (ch("q"), ModifiersState::SUPER),
        Action::Quit,
        "Quit",
        "Cmd-Q",
    );
    insert_default_entry(
        &mut map,
        (ch("q"), ModifiersState::SUPER),
        Action::Save,
        "Save",
        "Cmd-Q",
    );
}

/// A slot mutation has one owner: the same chord text labels the command and
/// seeds dispatch. This test uses a local catalog-shaped row so the embedded
/// asset itself remains immutable during the test.
#[test]
fn changing_one_valid_default_slot_changes_both_label_and_dispatch() {
    let mutated = crate::commands::Command {
        name: "Save",
        action: Action::Save,
        native: "Cmd-J",
        emacs: "",
        native_only: false,
        web_only: false,
        description: None,
    };
    assert_eq!(
        crate::commands::join_slots(mutated.native, mutated.emacs),
        "⌘J"
    );

    let mut km = KeymapState::new_with_convention(Convention::Mac);
    km.replace_defaults_for_test(mutated.native, mutated.action, mutated.name);
    assert_eq!(resolve_spec(&mut km, "Cmd-J").last(), Some(&Action::Save));
    assert_eq!(resolve_spec(&mut km, "Cmd-S").last(), Some(&Action::Ignore));
}

/// THE ALTERNATE-ACCEPT CHORD (⇧↵), swept over BOTH conventions
/// and BOTH keymap flavors. Shift reads identically on Mac and Linux (unlike
/// Cmd/Ctrl, which the whole rest of this module exists to translate), so
/// `AcceptAlternate` needs no native/emacs catalog slot and no
/// `linux_keep_emacs` entry at all — proven here rather than assumed, over
/// the axis a chord resolved only in `resolve_named` (never through the
/// catalog-seeded maps) could still silently get wrong: a convention or
/// flavor that quietly shadowed it. Bare Enter is swept alongside it so a
/// future change can't make Shift+Enter "leak" its meaning onto the
/// unshifted chord (or vice versa).
#[test]
fn accept_alternate_resolves_identically_on_every_convention_and_keymap_flavor() {
    for convention in [Convention::Mac, Convention::Linux] {
        for (flavor, keep) in [
            ("native", Vec::new()),
            ("emacs", crate::keymap::linux_emacs_preset_keep()),
        ] {
            let mut km = KeymapState::new_with_convention(convention);
            km.apply_linux_keep(&keep);
            assert_eq!(
                km.resolve(&Key::Named(NamedKey::Enter), &shift()),
                Action::AcceptAlternate,
                "{convention:?}/{flavor}: Shift+Enter must resolve to the alternate accept"
            );
            assert_eq!(
                km.resolve(&Key::Named(NamedKey::Enter), &none()),
                Action::Newline,
                "{convention:?}/{flavor}: bare Enter stays a plain newline"
            );
        }
    }
}

/// THE LINUX KEEP-LIST HAS NOTHING TO SAY ABOUT IT: `AcceptAlternate` is not a
/// single Ctrl-letter chord, so it can never appear in `LINUX_DISPLACED_LETTERS`
/// or `linux_builtin_keep()` — named directly here so a reader does not have
/// to infer it from the resolver sweep above.
#[test]
fn accept_alternate_is_not_a_linux_keep_list_member() {
    assert!(
        crate::keymap::linux_builtin_keep()
            .iter()
            .all(|c| !c.to_ascii_lowercase().contains("s-return")
                && !c.to_ascii_lowercase().contains("enter")),
        "the unconditional Linux keep floor names no Enter chord"
    );
    assert!(
        crate::keymap::linux_emacs_preset_keep()
            .iter()
            .all(|c| !c.to_ascii_lowercase().contains("enter")),
        "the emacs-flavor keep preset (every displaced Ctrl-letter) names no Enter chord either"
    );
}
