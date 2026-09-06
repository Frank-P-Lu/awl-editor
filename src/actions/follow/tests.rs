use super::*;
use crate::buffer::Buffer;

/// A markdown buffer with a real path, so a relative destination has a
/// directory to anchor on.
fn doc(text: &str, path: &str) -> Buffer {
    let mut b = Buffer::from_str(text);
    b.set_path(std::path::PathBuf::from(path));
    b
}

/// LAW: each followable kind reaches the EFFECT its destination's door needs —
/// an external URL the OS-handoff effect, a local document the open effect
/// awl performs itself, a footnote reference the shared line jump. Swept over
/// the underline grammar's own roster (`MdKind::is_followable`), which names
/// what enrolled if a member ever stops resolving.
#[test]
fn every_followable_kind_reaches_the_effect_its_own_door_needs() {
    let _guard = crate::testlock::serial();
    let text = "\
A [named](https://example.com/a) link.

A [local](sibling.md) one.

Bare https://bare.example/x here.
";
    let buffer = doc(text, "/proj/notes/today.md");
    let spans = crate::markdown::spans(text);
    let followable: Vec<_> = spans
        .iter()
        .filter(|(_, k)| k.is_followable())
        .cloned()
        .collect();
    assert!(
        followable.len() >= 3,
        "non-vacuity: the fixture enrolled only {} followable spans",
        followable.len()
    );
    for (range, kind) in &followable {
        let src = &text[range.clone()];
        let effect = follow_effect(&buffer, range.start + range.len() / 2);
        assert_ne!(
            effect,
            Effect::None,
            "followable span {kind:?} ({src:?}) wears the underline and produces NO effect"
        );
        match src {
            "named" => assert_eq!(
                effect,
                Effect::FollowLink("https://example.com/a".to_string())
            ),
            "local" => assert_eq!(
                effect,
                Effect::OpenPathAtLine {
                    path: "/proj/notes/sibling.md".to_string(),
                    line: 0,
                    col: 0,
                },
                "a relative destination opens IN awl, anchored on the document's own directory"
            ),
            "https://bare.example/x" => assert_eq!(
                effect,
                Effect::FollowLink("https://bare.example/x".to_string()),
                "a tamed bare URL follows ITSELF"
            ),
            other => panic!("unswept followable span {kind:?} ({other:?})"),
        }
    }
}

/// LAW: the gesture on a plain word produces nothing. Swept over every byte of
/// prose outside every followable span rather than one hand-picked column —
/// the whole point being that the gesture must not steal an ordinary press.
#[test]
fn the_follow_gesture_on_plain_prose_produces_nothing() {
    let _guard = crate::testlock::serial();
    let text = "Ordinary prose with a [link](https://x.example/y) inside it.\n";
    let buffer = doc(text, "/proj/notes/today.md");
    let mut probed = 0usize;
    for byte in 0..text.len() {
        if !text.is_char_boundary(byte) {
            continue;
        }
        if crate::markdown::followable_at(text, byte).is_some() {
            continue;
        }
        probed += 1;
        assert_eq!(
            follow_effect(&buffer, byte),
            Effect::None,
            "byte {byte} is plain prose: {:?}",
            &text[byte..text.len().min(byte + 16)]
        );
    }
    assert!(probed > 20, "the negative sweep probed only {probed} bytes");
}

/// LAW: a footnote reference still activates through this door — it is not part
/// of the underline grammar (it wears the painted number, not the hairline) and
/// its destination is a line in this document.
#[test]
fn a_footnote_reference_still_jumps_through_the_same_door() {
    let _guard = crate::testlock::serial();
    let text = "See this[^n].\n\n[^n]: answer\n";
    let buffer = doc(text, "/proj/notes/today.md");
    let byte = text.find("[^n]").unwrap() + 2;
    assert_eq!(follow_effect(&buffer, byte), Effect::JumpToLine(2));
}

/// LAW: the affordance is a promise the RENDER makes. A non-markdown buffer
/// draws no followable underline, so nothing in it follows — the same rule for
/// the hairline and for the gesture, checked against a file whose text would
/// otherwise parse as a link.
#[test]
fn a_non_markdown_buffer_follows_nothing_because_it_underlines_nothing() {
    let _guard = crate::testlock::serial();
    let text = "// see [docs](https://example.com/a) and https://bare.example/x\n";
    let code = doc(text, "/proj/src/lib.rs");
    assert!(!code.is_markdown(), "the fixture must not read as markdown");
    for byte in 0..text.len() {
        if !text.is_char_boundary(byte) {
            continue;
        }
        assert_eq!(
            follow_effect(&code, byte),
            Effect::None,
            "byte {byte} of a non-markdown buffer must follow nothing"
        );
    }
    // And the same bytes in a markdown buffer DO follow — the contrast is what
    // makes the law above a statement about the gate rather than about the text.
    let md = doc(text, "/proj/notes/today.md");
    let byte = text.find("docs").unwrap() + 1;
    assert_ne!(follow_effect(&md, byte), Effect::None);
}

/// LAW: a deferred in-document anchor is a calm no-op, never a guess. Recorded
/// here so the deferral is visible as a decision rather than as an omission.
#[test]
fn a_heading_anchor_is_deferred_to_a_calm_no_op() {
    let _guard = crate::testlock::serial();
    let text = "Jump to [the section](#somewhere) please.\n";
    let buffer = doc(text, "/proj/notes/today.md");
    let byte = text.find("the section").unwrap() + 1;
    assert_eq!(follow_effect(&buffer, byte), Effect::None);
}

/// LAW: a path-less scratch buffer has no directory for a relative destination
/// to anchor against, so it follows nothing rather than guessing the process
/// cwd — while an ABSOLUTE destination in the same buffer still resolves.
#[test]
fn a_relative_destination_in_a_path_less_buffer_is_a_calm_no_op() {
    let _guard = crate::testlock::serial();
    let text = "[rel](sibling.md) and [abs](/tmp/elsewhere.md)\n";
    let buffer = Buffer::from_str(text);
    assert!(buffer.path().is_none() && buffer.is_markdown());
    assert_eq!(
        follow_effect(&buffer, text.find("rel").unwrap() + 1),
        Effect::None
    );
    assert_eq!(
        follow_effect(&buffer, text.find("abs").unwrap() + 1),
        Effect::OpenPathAtLine {
            path: "/tmp/elsewhere.md".to_string(),
            line: 0,
            col: 0,
        }
    );
}
