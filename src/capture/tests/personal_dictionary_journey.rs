//! **"PERSONAL DICTIONARY…" UNDER TIER-1 REPLAY — the summon lands, the rows
//! cannot.** Real chords open the Command Palette, filter to the palette-only
//! "Personal dictionary…" row and accept, the exact route a live user takes
//! (`crate::run::ReplaySession::apply_chord`, the seam `--keys` itself runs).
//!
//! The card opens — that half of the wiring is real and this law is the only
//! thing that drives it. Its ROWS are the other half, and they are structurally
//! out of the headless harness's reach: `ReplaySession` builds its own
//! `SpellChecker` (`run.rs`), and `SpellChecker::new`'s own doc says the
//! personal dictionary starts EMPTY and the caller loads it via
//! `set_user_words` — a call only the live `App` makes
//! (`App::load_user_dictionary`). So the replay's gather asks a checker that
//! has never been told anything, and a `--keys` capture photographs an EMPTY
//! word list no matter what `dictionary.txt` holds.
//!
//! This is asserted rather than merely known, for two reasons. It stops a
//! future brief asking for a tier-1 capture of the picker's rows — the class of
//! request `docs/harness-reach.md` exists to refuse. And it is the tripwire on
//! the other decision: a replay that DID read the ambient `dictionary.txt`
//! would photograph the operator's own added words into a public repo's
//! captures, so extending the harness here is a deliberate act that must go red
//! here first.

use crate::buffer::Buffer;
use crate::config::Config;
use crate::overlay::OverlayKind;
use crate::testscratch::ScratchDir;

fn type_chars(session: &mut crate::run::ReplaySession, text: &str) {
    let spec: String = text
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let chords = crate::keyspec::parse_chords(&spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

fn press(session: &mut crate::run::ReplaySession, spec: &str) {
    let chords = crate::keyspec::parse_chords(spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

#[test]
fn the_palette_summons_the_personal_dictionary_but_replay_can_never_fill_it() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl_personal_dictionary_journey_{}",
        std::process::id()
    )));
    // A REAL word list on disk beside the config the session is handed. The
    // point of the law is that this file is present and still reaches nothing.
    // Through `fs::write_atomic` rather than a bare `std::fs::write`, so this
    // fixture needs no entry in the durable-write ledger.
    crate::fs::write_atomic(&dir.join("dictionary.txt"), b"quokka\nzorbling\n")
        .expect("write dictionary");
    assert!(
        dir.join("dictionary.txt").exists(),
        "precondition: the word list really is on disk, so the empty card below \
         is about the harness's reach and not about a missing fixture"
    );

    let mut buffer = Buffer::from_str("a zorbling document\n");
    let corpus: Vec<String> = Vec::new();
    let root = dir.to_path_buf();
    let mut config = Config::empty();
    config.path = dir.join("config.toml");
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = crate::run::ReplaySession::new(
        crate::run::ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        &root,
        Some(root.as_path()),
        &config,
        None,
        &mut km,
    );

    // `personaldict` is a subsequence of "Personal dictionary…" and of no other
    // command name — asserted by the single-row check below rather than assumed.
    press(&mut session, "s-p");
    type_chars(&mut session, "personaldict");
    assert_eq!(
        session
            .journey()
            .card()
            .map(|o| o.item_strings())
            .unwrap_or_default(),
        vec!["Personal dictionary…".to_string()],
        "the palette filter must resolve to the personal-dictionary row alone"
    );
    press(&mut session, "Enter");

    let card = session
        .journey()
        .card()
        .expect("accepting the palette row summons the picker");
    assert_eq!(
        card.kind,
        OverlayKind::UserWords,
        "the palette route reaches the personal-dictionary picker — the summon \
         half of the wiring is real"
    );
    assert!(
        card.item_strings().is_empty(),
        "TIER-1 CEILING: the replay's own SpellChecker is never told the \
         personal dictionary (only `App::load_user_dictionary` calls \
         `set_user_words`), so the picker's rows cannot exist headlessly — \
         they are reachable only through `--screenshot-app`. If this went \
         green with rows, the harness grew a door: update \
         `docs/harness-reach.md` AND the gather's own comment in \
         `main/run/chord.rs`, and think about whose `dictionary.txt` a public \
         capture is now photographing. Got: {:?}",
        card.item_strings()
    );
}
