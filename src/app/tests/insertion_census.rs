//! **THE INSERTION-DOOR CENSUS IS COMPLETE, AND A NEW DOOR CANNOT JOIN IT
//! SILENTLY.**
//!
//! # The defect this file exists for
//!
//! The wall that keeps text out of a document hidden behind a reading surface
//! was written against three doors, and the lane that wrote it discovered two of
//! them itself. The class grows every time an input capability lands, and a new
//! door ships OPEN by default — nothing forces it through the wall, so the next
//! one repeats the bug on whatever surface exists then.
//!
//! # Where the force actually comes from, and why it is not this file
//!
//! Most of it is the COMPILER. `DocumentSession`'s four raw text mutators are
//! private to the `document` module, so the only text edit reachable from
//! anywhere else in `crate::app` is `App::write_document_text`, whose first
//! argument is a `TextDoor`; and `TextDoor::gate` / `TextDoor::site` are
//! wildcard-free, so a new variant does not compile until its author has said
//! what stops it and where it lives. `app::tests::read_only_surface`'s sweep
//! adds the third: its `drive` is wildcard-free too, so a new door does not
//! compile until someone has said how it is PRESSED.
//!
//! What is left for a test is the part Rust's privacy model cannot express —
//! that the declarations are TRUE of the source tree. Rust can say "private plus
//! every descendant module" but not "private plus this one function", which is
//! why `app/tests/source_audit.rs` exists at all; this file is the same honest
//! fallback for the same reason, over a roster small enough to keep curated.
//!
//! # What this check does NOT cover, stated rather than assumed
//!
//! The scan reads PRODUCTION source only — it skips `tests.rs`, anything under a
//! `tests/` directory, and the census module's own definitions. A door named
//! from inside a `#[cfg(test)] mod tests` block in a production file would
//! therefore be counted as production and fail this law by name; the fix is to
//! move that fixture into the file's test module, which is where it belongs.

use crate::app::TextDoor;
use std::collections::BTreeMap;

/// Per-file occurrence counts of `needle` across PRODUCTION source under
/// `src/`, with all whitespace stripped first so a line-wrapped call matches
/// however rustfmt happened to break it. Keys are `src/`-relative.
///
/// `extra_skip` drops files whose relative path equals it — used for the
/// census module itself, whose whole job is to spell these names out.
fn production_hits(needle: &str, extra_skip: &[&str]) -> BTreeMap<String, usize> {
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if rel.contains("/tests/") || rel.ends_with("tests.rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            out.push((rel, text.chars().filter(|c| !c.is_whitespace()).collect()));
        }
    }

    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&base, &base, &mut files);
    files
        .into_iter()
        .filter(|(rel, _)| !extra_skip.contains(&rel.as_str()))
        .filter_map(|(rel, text)| {
            let n = text.matches(needle).count();
            (n > 0).then_some((rel, n))
        })
        .collect()
}

const CENSUS_MODULE: &str = "app/input/text_door.rs";

/// **EVERY MEMBER DECLARES A GATE, AND AN EXEMPTION CARRIES A REASON.**
///
/// The roster is generated from the variant list (`enum_with_all!`), so there is
/// no second list to fall out of step with it, and `gate` is wildcard-free — a
/// new door cannot compile without an answer. What is checkable here is that the
/// answer is a real one: an exemption whose reason is blank, or a placeholder, is
/// the unnamed exemption this census exists to prevent, and it would read as
/// enrolled on a board six weeks later.
///
/// The companion is that BOTH populations are non-empty. "Every exemption has a
/// reason" is trivially true of a roster with no exemptions, and "every gated
/// door is swept" is trivially true of a roster with no gated doors.
#[test]
fn every_census_member_declares_a_gate_and_every_exemption_names_its_reason() {
    assert_eq!(
        TextDoor::ALL.len(),
        TextDoor::VARIANT_COUNT,
        "the roster and the variant list disagree"
    );
    let mut seen = std::collections::BTreeSet::new();
    for door in TextDoor::ALL {
        assert!(seen.insert(format!("{door:?}")), "{door:?} listed twice");
    }

    let mut gated = Vec::new();
    let mut exempt = Vec::new();
    for door in TextDoor::ALL {
        match door.exemption_reason() {
            None => gated.push(door),
            Some(reason) => {
                let words = reason.split_whitespace().count();
                assert!(
                    words >= 8,
                    "{door:?} is exempt on a {words}-word reason ({reason:?}) — an \
                     exemption has to say WHY a reading surface must not refuse it, \
                     because a reader six weeks from now cannot tell a considered \
                     exemption from a forgotten one"
                );
                assert!(
                    !reason.to_ascii_lowercase().contains("todo"),
                    "{door:?}'s exemption reason is a placeholder: {reason:?}"
                );
                exempt.push(door);
            }
        }
    }
    assert!(
        !gated.is_empty(),
        "no door is gated — `read_only_surface`'s sweep would have no subject"
    );
    assert!(
        !exempt.is_empty(),
        "no door is exempt — this law's reason check would have no subject"
    );

    // THE RATCHET, and the reason it is a literal list where nothing else here
    // is. Every other enrolment in this census is derived, because a derived
    // enrolment cannot be dodged. This one must NOT be: `gated_doors()` in the
    // sweep is derived from `gate()`, so a door moved from `Wall` to `Exempt`
    // un-enrols itself from the sweep and every refusal law goes quietly green
    // over a smaller set. That is the enrolment failure this repo has been
    // bitten by before, and the only thing that catches it is a name.
    //
    // Leaving the wall is therefore a decision someone takes here, on purpose,
    // with the reason in front of them — not a one-word edit in the roster.
    let names: Vec<String> = exempt.iter().map(|d| format!("{d:?}")).collect();
    assert_eq!(
        names,
        vec![
            "HistoryRestore",
            "ConflictTakeTheirs",
            "RelaunchRecoveryAdopt",
            "AccessibilityBench",
            "PersistenceFaultProbe",
            "HeadlessReplay",
        ],
        "the set of doors OUTSIDE the wall changed. Adding one is a product \
         decision — the surface stops refusing that door — and removing one is \
         too. Update this list deliberately, with the new member's reason beside \
         it, rather than letting the sweep quietly shrink."
    );
}

/// **THE ROPE HAS ONE DOOR, AND IT HAS ONE CALLER.**
///
/// `DocumentSession::apply_text_edit` is the only text edit visible outside the
/// `document` module (its four raw mutators are private there), and the only
/// thing that may call it is `App::write_document_text`, which demands a named
/// `TextDoor`. A second caller would be a door that reaches the buffer without
/// enrolling — exactly the bypass this census closes — and it is the shape a
/// future convenience helper would take.
///
/// Same technique and same spirit as `source_audit`'s fence on the mutable
/// buffer loan, one layer down.
#[test]
fn the_one_text_edit_has_exactly_one_production_caller() {
    let hits = production_hits("apply_text_edit(", &[]);
    let files: Vec<&str> = hits.keys().map(String::as_str).collect();
    assert_eq!(
        files,
        vec!["app/document/edit.rs", CENSUS_MODULE],
        "`apply_text_edit` must be the definition in the document owner's edit module \
         plus the ONE call in the census module; found: {hits:?}"
    );
    assert_eq!(hits.get("app/document/edit.rs"), Some(&1));
    assert_eq!(hits.get(CENSUS_MODULE), Some(&1));
}

/// **EVERY DOOR IS NAMED WHERE IT SAYS IT LIVES, AND NOWHERE ELSE.**
///
/// The enrolment comes from the roster (`TextDoor::ALL`), not from a list
/// written here, and the expected location comes from each member's own
/// `site()` — wildcard-free, so a new door has to answer. The law then reads the
/// source and checks the answer is true: named in that file, named in no other
/// production file.
///
/// The second half is the one that catches a real drift. A door pressed from a
/// SECOND site is a door whose gate holds in one place and not the other, and
/// that is precisely how `write_back_image_width` came to exist un-walled while
/// three neighbours were walled.
#[test]
fn every_door_is_named_at_its_declared_site_and_nowhere_else() {
    for door in TextDoor::ALL {
        let site = door.site();
        let needle = format!("TextDoor::{door:?}");
        let hits = production_hits(&needle, &[CENSUS_MODULE]);
        let files: Vec<&str> = hits.keys().map(String::as_str).collect();
        assert_eq!(
            files,
            vec![site.file],
            "{door:?} declares its home as {} but production source names it in \
             {hits:?}. A door named in two places is a door whose gate holds in one \
             of them.",
            site.file
        );
    }
}

/// **THE FILES THAT WRITE ARE EXACTLY THE FILES THE ROSTER CLAIMS.**
///
/// The previous law reads the roster outward: each door is where it says it is.
/// This one reads the SOURCE inward: every production call of the door function
/// belongs to a file some census member claims. Without it a door could be
/// pressed through a `TextDoor` held in a variable or a constant, naming no
/// variant at the call and enrolling nothing — which is not hypothetical, since
/// the fault probe presses its door through exactly such a constant.
#[test]
fn the_door_function_is_called_only_from_files_the_roster_claims() {
    let claimed: std::collections::BTreeSet<&str> = TextDoor::ALL
        .into_iter()
        .map(TextDoor::site)
        .filter(|s| s.through_the_door)
        .map(|s| s.file)
        .collect();
    assert!(
        !claimed.is_empty(),
        "no door claims to use the door function"
    );

    let hits = production_hits("write_document_text(", &[CENSUS_MODULE]);
    let found: std::collections::BTreeSet<&str> = hits.keys().map(String::as_str).collect();
    assert_eq!(
        found, claimed,
        "the files calling the one text-mutation door must be exactly the files the \
         census claims. Found {found:?}, claimed {claimed:?} — a file in `found` and \
         not in `claimed` is an unenrolled door; the reverse is a roster entry whose \
         site has moved."
    );
}
