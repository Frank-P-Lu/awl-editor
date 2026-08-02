//! src/save_model_law.rs — THE SAVE-MODEL PROSE LAW.
//!
//! `docs_catalog_law.rs` next door pins the site guide's CHORDS and COMMAND
//! NAMES to the live catalog, and `keytoken.rs` does the same for the markdown
//! docs through their `{{key:}}` / `{{cmd:}}` tokens. Neither can see the thing
//! pinned here: the running prose that states what saving DOES.
//!
//! That prose is where the expensive drift lives. A sentence promising that a
//! manual save force-writes over an outside change, or offering a "reopen"
//! path, cites no chord and no command name — so it survives every existing
//! law, on all three surfaces at once, for as long as nobody rereads it.
//!
//! So the two laws below pin it from BOTH ends. The retired promise may not
//! reappear anywhere; and the vocabulary that replaced it must be the
//! vocabulary the shipping product actually uses — the live notice constant and
//! two real, dispatchable catalog rows, not three strings that happen to be
//! spelled the same in four places.
#![cfg(test)]

use crate::commands;

/// The three docs that state awl's save model to a user. Each spells the same
/// facts its own way — Markdown cites chords through `{{key:}}` tokens, the
/// static site hand-types the glyph — so only the vocabulary is asserted here,
/// never the sentence.
fn save_model_docs() -> [(&'static str, &'static str); 3] {
    [
        ("GUIDE.md", crate::embedded_docs::GUIDE_MD),
        ("samples/welcome.md", crate::embedded_docs::WELCOME_MD),
        ("site/guide.html", crate::embedded_docs::SITE_GUIDE_HTML),
    ]
}

/// **THE RETIRED PROMISE MAY NOT COME BACK.** No user-facing document may claim
/// that a manual Save force-writes over an external change, or advertise the
/// "reopen for theirs" path that never existed.
///
/// Both claims were true-looking and wrong in the same way: each described a
/// door. One of them opened onto data loss and the other opened onto nothing.
#[test]
fn no_document_still_promises_that_a_manual_save_force_writes() {
    let retired = [
        "force-writes",
        "force writes",
        "always overwrites",
        "reopen for theirs",
        "⌘S keeps yours",
        "keeps yours",
    ];
    for (name, text) in save_model_docs() {
        for claim in retired {
            // "does **not** force-write" is the CORRECTION, not the claim, so
            // the ban is on the promise rather than on the word.
            let promised = text
                .match_indices(claim)
                .any(|(at, _)| !preceded_by_negation(text, at));
            assert!(
                !promised,
                "{name} still promises {claim:?} — the shipped path holds the write and \
                 offers two explicit resolutions instead"
            );
        }
    }
}

/// Does a negation sit close enough in front of `at` to be modifying it? A
/// window rather than a parse: these are short declarative sentences, and the
/// only constructions in play are "does not force-write" and "does
/// <strong>not</strong> force-write".
fn preceded_by_negation(text: &str, at: usize) -> bool {
    let start = at.saturating_sub(40);
    let window = &text[text.floor_char_boundary(start)..at];
    window.contains("not")
}

/// **THE LIVE VOCABULARY IS THE DOCUMENTED VOCABULARY.** Every surface must
/// name the state awl actually shows and the two commands that actually settle
/// it — and each of those must be a real, dispatchable catalog row.
///
/// The anti-vacuity half is the middle block: pinning prose against two string
/// literals would go green over a pair of commands that had been renamed or
/// removed, which is exactly how a sentence describing a path that did not
/// exist survived. So the SAME strings are looked up in the live catalog, and
/// the state's name is taken from the live constant rather than retyped.
#[test]
fn every_save_model_document_uses_the_live_conflict_vocabulary() {
    // The state's name, straight from the constant the running app displays.
    let notice = crate::app::CHANGED_ELSEWHERE_NOTICE;
    let state = notice
        .split(" — ")
        .next()
        .expect("the notice leads with the state it names");
    assert_eq!(
        state, "changed elsewhere",
        "the notice's own leading phrase"
    );

    // The two resolutions, verified to EXIST before they are demanded of prose.
    let resolutions = [
        ("Save your version", crate::keymap::Action::ResolveKeepMine),
        ("Use disk version", crate::keymap::Action::ResolveTakeTheirs),
    ];
    for (label, action) in &resolutions {
        assert_eq!(
            commands::action_for_name(label),
            Some(action.clone()),
            "{label:?} must be a real catalog row, or the docs are citing a command \
             that does not exist"
        );
        assert!(
            notice.contains(label),
            "the notice must name {label:?}: a notice that describes a state without \
             naming the way out is the dead end this law retired"
        );
    }

    for (name, text) in save_model_docs() {
        assert!(
            text.contains(state),
            "{name} must name the state awl shows ({state:?})"
        );
        for (label, _) in &resolutions {
            assert!(
                text.contains(label),
                "{name} must name the {label:?} resolution"
            );
        }
    }
}
