use super::*;
use crate::markdown::MdKind;

/// One document exercising every construct the follow affordance meets: both
/// members of the underline grammar (a named link, an angle autolink, a bare
/// URL with a tail and one without), both destination doors (external, local),
/// the deferred in-document arm, and plenty of plain prose that must follow
/// NOTHING. Shared by the sweep below so the roster and the negative case are
/// read off the same text.
const CORPUS: &str = "\
Plain prose with no destination at all.

A [named link](https://example.com/deep/path?q=1) mid-sentence.

A relative one: [the plan](../notes/plan.md) and a fragment [here](#section).

An autolink <https://autolink.example/a> too.

Bare with a tail https://bare.example/x/y and bare without https://naked.example here.

`https://in-code.example/not-a-link` stays code.
";

fn mid(range: &std::ops::Range<usize>) -> usize {
    range.start + (range.len() / 2)
}

/// THE ENROLMENT IS THE GRAMMAR'S OWN. Every span the underline grammar marks
/// followable — the exact set `render::rects` draws its hairline under — must
/// resolve to a destination. A hairline that promises a place to go and answers
/// nothing is the defect this law names, and it is swept by asking
/// `MdKind::is_followable` rather than by naming `LinkText`/`BareUrlText`, so a
/// future member enrols here the moment it declares itself followable.
#[test]
fn every_span_the_underline_grammar_marks_followable_resolves_a_destination() {
    let _guard = crate::testlock::serial();
    let spans = crate::markdown::spans(CORPUS);
    let followable: Vec<_> = spans
        .iter()
        .filter(|(_, k)| k.is_followable())
        .cloned()
        .collect();
    assert!(
        !followable.is_empty(),
        "the corpus enrolled NOTHING — a vacuous sweep, not a passing law"
    );
    let mut kinds: Vec<MdKind> = Vec::new();
    for (range, kind) in &followable {
        let src = &CORPUS[range.clone()];
        let hit = followable_at(CORPUS, mid(range)).unwrap_or_else(|| {
            panic!(
                "followable span {kind:?} over {range:?} ({src:?}) wears the underline \
                 but resolves to NOTHING"
            )
        });
        assert!(
            !hit.raw.is_empty(),
            "followable span {kind:?} ({src:?}) resolved to an empty destination"
        );
        assert_ne!(
            hit.dest,
            Destination::InDocument(String::new()),
            "followable span {kind:?} ({src:?}) resolved to nowhere"
        );
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    // Name what enrolled, and assert the corpus reached BOTH members rather
    // than passing on one of them twice.
    assert!(
        kinds.contains(&MdKind::LinkText) && kinds.contains(&MdKind::BareUrlText),
        "the corpus must exercise every followable kind; it reached {kinds:?}"
    );
}

/// The destination each followable kind resolves to is the one the grammar says
/// it has: a bare URL follows ITSELF (its own source text, scheme through
/// tail), a named link follows its `(dest)`, never its visible label.
#[test]
fn each_followable_kind_resolves_the_destination_its_own_grammar_names() {
    let _guard = crate::testlock::serial();
    for (range, kind) in crate::markdown::spans(CORPUS)
        .into_iter()
        .filter(|(_, k)| k.is_followable())
    {
        let src = &CORPUS[range.clone()];
        let hit = followable_at(CORPUS, mid(&range)).expect("resolves");
        match kind {
            MdKind::BareUrlText => assert_eq!(
                hit.raw, src,
                "a bare URL's destination is its own source text"
            ),
            // A named link follows its `(dest)`. An ANGLE AUTOLINK's label IS
            // its destination, so "differs from the label" would be false of a
            // correct answer — the claim that holds for both is "it is the
            // enclosing link's own destination, read from the same parse".
            MdKind::LinkText => {
                let link = crate::markdown::link_at_full(CORPUS, range.start)
                    .expect("a LinkText span sits inside a parsed link");
                assert_eq!(
                    hit.raw, link.url,
                    "a named link must follow its destination, not its visible label {src:?}"
                );
            }
            other => panic!("unswept followable kind {other:?} over {range:?}"),
        }
    }
}

/// The bite the sweep above cannot carry on its own: where a named link's label
/// and destination genuinely DIFFER, the follow takes the destination.
#[test]
fn a_named_link_follows_its_destination_not_its_visible_label() {
    let _guard = crate::testlock::serial();
    let text = "see [click here](https://elsewhere.example/real) now";
    let hit = followable_at(text, text.find("click").unwrap() + 1).expect("follows");
    assert_eq!(hit.raw, "https://elsewhere.example/real");
    assert_eq!(hit.kind, MdKind::LinkText);
}

/// A modifier-click / caret on ordinary prose produces NOTHING — the calm no-op
/// that keeps the gesture from stealing a plain press. Swept over every byte
/// outside a followable span rather than one hand-picked word.
#[test]
fn a_byte_outside_every_followable_span_follows_nothing() {
    let _guard = crate::testlock::serial();
    let followable: Vec<_> = crate::markdown::spans(CORPUS)
        .into_iter()
        .filter(|(_, k)| k.is_followable())
        .map(|(r, _)| r)
        .collect();
    // A named link's brackets and `(url)` tail are deliberately followable via
    // the widening probe even though they wear no hairline, so exclude the whole
    // structural link range too — this law is about PROSE.
    let mut probed = 0usize;
    for byte in 0..CORPUS.len() {
        if !CORPUS.is_char_boundary(byte) {
            continue;
        }
        if followable.iter().any(|r| r.contains(&byte))
            || crate::markdown::link_at_full(CORPUS, byte).is_some()
        {
            continue;
        }
        probed += 1;
        assert!(
            followable_at(CORPUS, byte).is_none(),
            "byte {byte} ({:?}) is plain prose and must follow nothing",
            &CORPUS[byte..CORPUS.len().min(byte + 20)]
        );
    }
    assert!(
        probed > 200,
        "the negative sweep probed only {probed} bytes"
    );
}

/// The widening probe: a caret on a named link's bracket or inside its `(url)`
/// tail — bytes that wear no hairline of their own — still follows the link
/// they are part of, and lands on the SAME destination the label does.
#[test]
fn a_named_links_markup_bytes_follow_the_same_destination_its_label_does() {
    let _guard = crate::testlock::serial();
    let text = "see [the plan](../notes/plan.md) now";
    let open_bracket = text.find('[').unwrap();
    let in_url = text.find("../notes").unwrap() + 3;
    let label = text.find("the plan").unwrap() + 2;
    let from_label = followable_at(text, label).expect("label follows");
    for byte in [open_bracket, in_url] {
        let hit = followable_at(text, byte)
            .unwrap_or_else(|| panic!("byte {byte} inside the link followed nothing"));
        assert_eq!(hit.raw, from_label.raw);
        assert_eq!(hit.dest, from_label.dest);
    }
}

#[test]
fn classify_routes_each_destination_to_the_door_that_serves_it() {
    for (raw, want) in [
        (
            "https://example.com/a",
            Destination::External("https://example.com/a".into()),
        ),
        (
            "http://example.com",
            Destination::External("http://example.com".into()),
        ),
        (
            "mailto:a@b.co",
            Destination::External("mailto:a@b.co".into()),
        ),
        (
            "file:///tmp/x.md",
            Destination::External("file:///tmp/x.md".into()),
        ),
        ("notes/x.md", Destination::Local("notes/x.md".into())),
        ("../a/b.md", Destination::Local("../a/b.md".into())),
        ("/abs/x.md", Destination::Local("/abs/x.md".into())),
        // The fragment rides on the path: awl opens the FILE, and the
        // within-file half stays deferred.
        ("x.md#head", Destination::Local("x.md".into())),
        ("#section", Destination::InDocument("#section".into())),
        // A one-letter "scheme" is a Windows drive, not a URI.
        (
            "C:\\notes\\x.md",
            Destination::Local("C:\\notes\\x.md".into()),
        ),
    ] {
        assert_eq!(classify(raw), want, "classify({raw:?})");
    }
}

/// A relative destination is anchored against the DOCUMENT's own directory —
/// markdown's rule — not against the project root. The two differ exactly when
/// the document is not at the root, which is the case this law pins.
#[test]
fn a_relative_destination_anchors_on_the_documents_own_directory() {
    let doc = std::path::Path::new("/proj/notes/deep/today.md");
    assert_eq!(
        resolve_local(Some(doc), "sibling.md").unwrap(),
        std::path::PathBuf::from("/proj/notes/deep/sibling.md"),
    );
    assert_eq!(
        resolve_local(Some(doc), "../plan.md").unwrap(),
        std::path::PathBuf::from("/proj/notes/deep/../plan.md"),
    );
    assert_eq!(
        resolve_local(Some(doc), "/elsewhere/x.md").unwrap(),
        std::path::PathBuf::from("/elsewhere/x.md"),
    );
    // A scratch buffer has no directory for "relative" to mean anything
    // against — the calm no-op, not a guess at the process cwd.
    assert_eq!(resolve_local(None, "sibling.md"), None);
}

/// The go-to row shows the TAMED authority, never the raw URL flood — the same
/// taming the rendered line applies, read through the same owner.
#[test]
fn the_go_to_label_shows_the_tamed_authority_never_the_raw_url() {
    for (raw, want) in [
        ("https://example.com/deep/path?q=1", "Go to example.com…"),
        ("https://example.com", "Go to example.com"),
        ("http://example.com:8080", "Go to example.com:8080"),
        ("mailto:a@b.co", "Go to a@b.co"),
        ("../notes/plan.md", "Go to ../notes/plan.md"),
        ("#section", "Go to #section"),
    ] {
        assert_eq!(go_to_label(raw), want, "label for {raw:?}");
    }
    // The flood case: a long URL is cut to the budget, so the card can never
    // be sized by whatever a document happens to contain.
    let flood = format!("https://{}/x", "a".repeat(200));
    let label = go_to_label(&flood);
    assert!(
        label.chars().count() <= "Go to ".chars().count() + LABEL_BUDGET,
        "the label floods at {} chars: {label:?}",
        label.chars().count()
    );
    assert!(
        label.ends_with('…'),
        "a cut label says it was cut: {label:?}"
    );
}
