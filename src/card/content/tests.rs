//! `card::content` unit tests. A sibling of `mod.rs`, named `tests.rs` so
//! it stays exempt from the production line ceiling
//! (`scripts/code-health.py::production`) however large the suite grows.

use super::*;

fn inputs() -> CardInputs {
    CardInputs {
        doc: crate::card::figures::DocFigures {
            words: "12 words · 1 min".to_string(),
            lang: None,
            percent: 42,
        },
        ..CardInputs::default()
    }
}

/// Every card must SAY something. A card that composed to nothing would
/// draw an empty float and announce an empty group — the failure mode the
/// roster sweep exists to catch when a sixth card is added and forgotten.
#[test]
fn every_card_kind_composes_named_nonempty_content() {
    let _guard = crate::testlock::serial();
    let inputs = inputs();
    for kind in CardKind::ALL {
        let content = card(kind, &inputs);
        assert_eq!(content.kind, kind);
        assert!(!content.spans.is_empty(), "{:?} composed no spans", kind);
        assert!(
            !kind.title().is_empty(),
            "{:?} has no accessible name",
            kind
        );
        assert!(
            kind.id().starts_with("card."),
            "{:?} has an off-roster id",
            kind
        );
        for span in &content.spans {
            assert!(
                !span.text.contains('\n'),
                "{:?} put a layout newline inside a span; the flattener owns those",
                kind
            );
        }
    }
}

/// Card ids and names must be distinct, or two cards would collide on one
/// AccessKit node.
#[test]
fn card_ids_and_titles_are_unique_across_the_roster() {
    let mut ids: Vec<&str> = CardKind::ALL.iter().map(|k| k.id()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), CardKind::ALL.len());
    let mut titles: Vec<&str> = CardKind::ALL.iter().map(|k| k.title()).collect();
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), CardKind::ALL.len());
}

/// The flattener owns every line break: a caption is followed by its
/// figure, a gap separates groups, and the card never ends on a blank
/// line.
#[test]
fn flattening_places_the_breaks_and_never_trails_one() {
    let _guard = crate::testlock::serial();
    let content = card(CardKind::Lifetime, &inputs());
    let flat = content.spans();
    assert_eq!(flat.len(), 10, "five caption/figure pairs");
    assert_eq!(flat[0], ("CHARACTERS\n".to_string(), 0));
    assert_eq!(flat[1], ("—\n\n".to_string(), 1));
    assert_eq!(flat[9], ("—".to_string(), 1), "no trailing break");
    let joined: String = flat.iter().map(|(text, _)| text.as_str()).collect();
    assert!(!joined.ends_with('\n'));
}

/// The semantic reading of a card is the same strings without the layout.
#[test]
fn lines_are_the_spans_without_layout_characters() {
    let _guard = crate::testlock::serial();
    let content = card(CardKind::Hud, &inputs());
    assert_eq!(
        content.lines(),
        vec![
            "SAVED",
            "—",
            "WORD COUNT",
            "12 words · 1 min",
            "THROUGH DOC",
            "42%",
            "LINE ENDINGS",
            "LF",
        ]
    );
}

/// The WORD COUNT row — the same row both the drawn HUD card
/// and the semantic snapshot's card fold read (`app/semantic/passive.rs`'s
/// `fold_card`, over this very `CardContent`) — follows the document's
/// dominant script, not a fixed "words" label. A CJK-dominant document
/// (no frontmatter, no spaces to tokenize on) reads in CHARACTERS.
#[test]
fn hud_word_count_row_follows_the_documents_dominant_script() {
    let _guard = crate::testlock::serial();
    let ja = "今日はいい天気ですね。".repeat(3); // 33 chars, decisively CJK
    let doc = crate::card::figures::DocFigures::of(&ja, true, 0, 0);
    assert_eq!(doc.words, "33 characters · 1 min");
    let mut ins = inputs();
    ins.doc = doc;
    let content = card(CardKind::Hud, &ins);
    assert_eq!(
        content.lines(),
        vec![
            "SAVED",
            "—",
            "WORD COUNT",
            "33 characters · 1 min",
            "THROUGH DOC",
            "0%",
            "LINE ENDINGS",
            "LF",
        ],
        "the card row — and so the semantic snapshot's card fold, which \
         reads these same lines — must say CHARACTERS for CJK prose"
    );
}

#[test]
fn hud_keeps_document_figures_and_names_an_active_selection_group() {
    let _guard = crate::testlock::serial();
    let content = card(
        CardKind::Hud,
        &CardInputs {
            selection: Some(crate::card::figures::SelectionFigures {
                words: 2,
                characters: 7,
            }),
            ..inputs()
        },
    );
    assert_eq!(
        content.lines(),
        vec![
            "SAVED",
            "—",
            "WORD COUNT",
            "12 words · 1 min",
            "SELECTION",
            "WORDS",
            "2",
            "CHARACTERS",
            "7",
            "THROUGH DOC",
            "42%",
            "LINE ENDINGS",
            "LF",
        ],
        "selection figures deepen the held card; they never replace document totals"
    );
}

/// A calm room composes no card at all — the gate the renderer's cheap
/// early-out and the semantic fold both depend on.
#[test]
fn a_calm_room_composes_no_card() {
    let _guard = crate::testlock::serial();
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
    crate::streaks::set_open(false);
    assert!(open_card(&CardInputs::default()).is_none());
}

/// The draw order is a rule, not an accident: every gate combination
/// resolves to exactly the card the renderer would have drawn.
#[test]
fn open_card_follows_the_renderers_precedence_for_every_gate_combination() {
    let _guard = crate::testlock::serial();
    for bits in 0..32u8 {
        let (streaks, about, lifetime, peek, hud) = (
            bits & 1 != 0,
            bits & 2 != 0,
            bits & 4 != 0,
            bits & 8 != 0,
            bits & 16 != 0,
        );
        crate::streaks::set_open(streaks);
        crate::about::set_open(about);
        crate::lifetime::set_open(lifetime);
        let inputs = CardInputs {
            peek_shown: peek,
            hud_held: hud,
            ..CardInputs::default()
        };
        let expected = if streaks {
            Some(CardKind::Streaks)
        } else if about {
            Some(CardKind::About)
        } else if lifetime {
            Some(CardKind::Lifetime)
        } else if peek {
            Some(CardKind::Peek)
        } else if hud {
            Some(CardKind::Hud)
        } else {
            None
        };
        assert_eq!(
            open_card(&inputs).map(|content| content.kind),
            expected,
            "gates streaks={streaks} about={about} lifetime={lifetime} peek={peek} hud={hud}"
        );
    }
    crate::streaks::set_open(false);
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
}
