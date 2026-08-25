//! src/roster_claim_law.rs — THE SOURCE-COMMENT ROSTER-CLAIM BAN: no comment in
//! this tree types how many worlds the roster holds.
//!
//! `doc_counts_law.rs` keeps roster sizes out of the docs awl SHOWS a reader.
//! This is the same fact one layer in, for the prose only a developer reads,
//! and it exists because that prose rotted first: comments were still naming
//! roster sizes the product had grown past several worlds earlier.
//!
//! WHY A BAN AND NOT A VALUE LAW. A comment has no `{{count:}}` seam to
//! substitute through, so holding the digits to `theme::THEMES.len()` would
//! only relabel today's number as correct and leave it to go stale on the next
//! roster change — which is exactly how the stale claims got written. The fix
//! that cannot rot is ROSTER-RELATIVE phrasing ("every `AmbientStyle::None`
//! world", "every world but the one under test"), so the literal is what gets
//! banned.
//!
//! SCOPE, stated so a reader knows what this does NOT cover.
//!
//! * Only the WHOLE roster. Subset counts ("the two diagonal worlds", "the five
//!   mono-display worlds") are a different claim and stay legal — the closed
//!   [`ROSTER_NEUTRAL_MODIFIERS`] list and the [`RESTRICTIVE_PRONOUNS`] check
//!   are what separate the two, and
//!   [`the_roster_claim_grammar_tells_a_roster_size_from_a_subset`] pins the
//!   separation in BOTH directions against real phrasings from this tree.
//! * Only the `worlds` noun. The tree's other rosters were swept by hand when
//!   this law was written, but none of them has this one's clean shape: `faces`
//!   names several different rosters (display, bold, CJK, symbol) and its
//!   counts almost always carry a narrowing modifier, so a ban on it would
//!   either fire on legitimate prose or miss the claims worth catching. A
//!   second roster earns a second arm only once it has one `len()`-able owner
//!   and a noun that is not shared.
//! * Only claims that NAME the noun. "on all twenty", with `worlds` left
//!   implied by the sentence around it, is invisible here — the noun is what
//!   tells a cardinal apart from a count of rings, rows or lines, and
//!   harvesting bare cardinals would fire on every unrelated number in the
//!   tree. That blind spot is deliberate, and it is swept by hand.
//! * Only comments. A `worlds` inside a string literal — an assertion message
//!   most of all — is never read as prose.
#![cfg(test)]

use super::doc_counts_law::cardinal;

/// Modifiers that do NOT narrow `worlds` to a subset. A cardinal reaching the
/// noun through only these is counting the ROSTER; a cardinal separated from it
/// by any other word ("mono-display worlds", "diagonal worlds", "`Bars` worlds")
/// is counting a subset, which is a different claim and out of scope. Keeping
/// the list CLOSED is what makes the ban sound: an unlisted modifier can only
/// make the harvest find fewer claims, never invent one.
const ROSTER_NEUTRAL_MODIFIERS: &[&str] = &[
    "shipped",
    "current",
    "live",
    "bundled",
    "curated",
    "existing",
    "theme",
    "total",
    "registered",
    "present",
    "other",
    "remaining",
];

/// A relative pronoun straight after the noun restricts it ("the N worlds WHOSE
/// bytes must not move"), which makes the phrase a subset however large N is.
const RESTRICTIVE_PRONOUNS: &[&str] = &["whose", "that", "which", "who", "where"];

/// How far back from `worlds` a cardinal may sit and still be read as
/// counting it. Three words covers every phrasing this tree uses and stops
/// well short of swallowing a previous sentence's number.
const ROSTER_MODIFIER_WINDOW: usize = 3;

/// The largest denominator an "N of M" phrase may carry and still be read as a
/// claim about the roster. Pixel and cell ratios ("3 of 1424", "0 of 960 000")
/// live far above any roster awl will ship; keeping them out is what stops the
/// fraction shape from harvesting measurements.
const MAX_ROSTER_DENOMINATOR: usize = 40;

/// A word from a comment, carrying the line it was written on so a failure can
/// name a `file:line` a reader can jump to.
struct CommentWord {
    line: usize,
    text: String,
}

/// `word` lowercased with its surrounding punctuation and markdown trimmed off.
fn bare_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-')
        .to_ascii_lowercase()
}

/// Every CONTIGUOUS run of comment lines in `src`, as its words.
///
/// Joining the run before scanning is load-bearing rather than tidy: a doc
/// comment wraps wherever `rustfmt` put it, and a claim split across the wrap
/// ("(fifteen of / sixteen — byte-identical)") is invisible to a line-at-a-time
/// scan. Only lines whose first non-space is `//` contribute, so a "worlds"
/// inside a string literal — an assertion message, most of all — is never read
/// as prose.
fn comment_blocks(src: &str) -> Vec<Vec<CommentWord>> {
    let mut blocks = Vec::new();
    let mut current: Vec<CommentWord> = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let trimmed = raw.trim_start();
        let body = trimmed
            .strip_prefix("///")
            .or_else(|| trimmed.strip_prefix("//!"))
            .or_else(|| trimmed.strip_prefix("//"));
        match body {
            Some(text) => current.extend(text.split_whitespace().map(|w| CommentWord {
                line: i + 1,
                text: w.to_string(),
            })),
            None => {
                if !current.is_empty() {
                    blocks.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

/// Every claim in `words` that states the size of the WHOLE world roster, as
/// `(line, quoted phrase)`. `floor` is the smallest cardinal read as
/// roster-scale.
///
/// Two shapes, because the tree writes the claim two ways:
///
/// * THE QUANTIFIER — `<N> [neutral modifier]* worlds`, covering "all N shipped
///   worlds", "the N worlds" and a bare "N worlds".
/// * THE FRACTION — `<N> of [the] <M>` next to the word `world`, the
///   exception-to-the-roster shape ("every `AmbientStyle::None` world — N of the
///   M stay byte-identical"). Here it is M, the denominator, that claims the
///   roster's size.
fn roster_size_claims(words: &[CommentWord], floor: usize) -> Vec<(usize, String)> {
    let bare: Vec<String> = words.iter().map(|w| bare_word(&w.text)).collect();
    let quote = |lo: usize, hi: usize| {
        words[lo..=hi.min(words.len() - 1)]
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut found = Vec::new();

    for i in 0..words.len() {
        if bare[i] != "worlds" {
            continue;
        }
        if bare
            .get(i + 1)
            .is_some_and(|w| RESTRICTIVE_PRONOUNS.contains(&w.as_str()))
        {
            continue;
        }
        let mut j = i;
        let mut skipped = 0usize;
        while j > 0 && skipped <= ROSTER_MODIFIER_WINDOW {
            j -= 1;
            if let Some(n) = cardinal(&words[j].text) {
                if n >= floor {
                    found.push((words[j].line, quote(j.saturating_sub(1), i)));
                }
                break;
            }
            if !ROSTER_NEUTRAL_MODIFIERS.contains(&bare[j].as_str()) {
                break;
            }
            skipped += 1;
        }
    }

    for i in 1..words.len() {
        if bare[i] != "of" {
            continue;
        }
        let Some(_numerator) = cardinal(&words[i - 1].text) else {
            continue;
        };
        let mut k = i + 1;
        if bare.get(k).is_some_and(|w| w == "the") {
            k += 1;
        }
        let Some(denominator) = words.get(k).and_then(|w| cardinal(&w.text)) else {
            continue;
        };
        if denominator < floor || denominator > MAX_ROSTER_DENOMINATOR {
            continue;
        }
        // The fraction counts WORLDS only if the noun says so — either the
        // thing being excepted is named just before it ("every `Ambient` world
        // — N of the M stay byte-identical") or `worlds` is the denominator's
        // own head noun. Anything else is a ratio about something that merely
        // sits in a sentence about worlds — a count of CELLS, or a sub-pitch
        // ratio whose denominator runs to the thousands. Reading those as
        // roster claims would make the ban fire on prose it has no business
        // touching, so the head-noun walk crosses only
        // [`ROSTER_NEUTRAL_MODIFIERS`], never an arbitrary word.
        let excepted_before = bare[i.saturating_sub(4)..i]
            .iter()
            .any(|w| w == "world" || w == "worlds");
        let mut head = k + 1;
        while bare
            .get(head)
            .is_some_and(|w| ROSTER_NEUTRAL_MODIFIERS.contains(&w.as_str()))
        {
            head += 1;
        }
        let head_noun_is_worlds = bare.get(head).is_some_and(|w| w == "worlds");
        if excepted_before || head_noun_is_worlds {
            found.push((words[i].line, quote(i.saturating_sub(3), k + 2)));
        }
    }

    found
}

/// Every `.rs` file under `src/`, so the ban is enrolled from the TREE rather
/// than from a list someone has to remember to extend.
fn every_source_file(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_source_file(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// THE SOURCE-COMMENT BAN. No comment under `src/` states how many worlds the
/// roster holds.
///
/// This is the same fact `the_starting_docs_state_no_literal_world_count` keeps
/// out of the reader-facing docs, one layer in. A count typed into a comment is
/// a second copy of `theme::THEMES` with nothing checking it, and comments have
/// no `{{count:}}` seam to substitute through — so the fix is not a fresher
/// number but ROSTER-RELATIVE phrasing ("every `AmbientStyle::None` world"),
/// which cannot rot at all. That is why this bans the literal outright instead
/// of holding it to `THEMES.len()`: a value law leaves today's correct number
/// in place to go stale on the next roster change, which is exactly how the
/// three claims this law first caught were written.
///
/// SCOPE. Only the whole roster. SUBSET counts ("the two diagonal worlds", "the
/// five mono-display worlds") are a different claim and stay legal — the
/// closed [`ROSTER_NEUTRAL_MODIFIERS`] list and the
/// [`RESTRICTIVE_PRONOUNS`] check are what separate the two, and
/// [`the_roster_claim_grammar_tells_a_roster_size_from_a_subset`] pins that
/// separation on real phrasings from this tree in both directions.
#[test]
fn no_source_comment_types_the_world_roster_size() {
    let _g = crate::testlock::serial();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    every_source_file(&root, &mut files);
    files.sort();
    assert!(
        files.len() > 100,
        "the source walk found only {} files under src/ — the ban is scanning \
         almost nothing",
        files.len()
    );

    let floor = crate::theme::THEMES.len() / 2;
    let mut offenders: Vec<String> = Vec::new();
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel = path.strip_prefix(&root).unwrap_or(path).display();
        for block in comment_blocks(&src) {
            for (line, phrase) in roster_size_claims(&block, floor) {
                offenders.push(format!("src/{rel}:{line} — {phrase:?}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "a comment types the size of the world roster. `theme::THEMES` carries \
         {} today, and a number typed beside it goes stale with nothing to \
         catch it — rewrite the sentence so it asks the roster instead (\"every \
         `AmbientStyle::None` world\", \"every world but the one under test\"), \
         rather than refreshing the digits:\n{}",
        crate::theme::THEMES.len(),
        offenders.join("\n")
    );
}

/// THE GRAMMAR'S OWN LAW, in both directions — the ban above is a
/// `assert!(empty)`, which a broken harvester satisfies just as well as a clean
/// tree does. The positives are the three phrasings that were live in this tree
/// when the ban was written; the negatives are real subset phrasings from the
/// same tree that must stay legal.
#[test]
fn the_roster_claim_grammar_tells_a_roster_size_from_a_subset() {
    let floor = 10;
    let claims = |text: &str| {
        let blocks = comment_blocks(text);
        blocks
            .iter()
            .flat_map(|b| roster_size_claims(b, floor))
            .count()
    };

    for roster in [
        "/// so all fifteen shipped worlds stay byte-identical. The animation",
        "/// ZERO instances for every `AmbientStyle::None` world (fifteen of\n\
         /// sixteen — byte-identical), and for page-off (no margins).",
        "//! for how each of the sixteen worlds picks from this data.",
        "// arrowing or sweeping the pointer through twenty worlds re-tints",
        "//! rows for thirteen of the twenty worlds, so seven were undocumented",
    ] {
        assert!(
            claims(roster) > 0,
            "the harvester missed a roster-size claim it must catch: {roster:?}"
        );
    }

    for subset in [
        "/// for the two Klee-derived worlds ([`theme::CJK_JA_KLEE`]: Mopoke)",
        "/// a `**bold**` span in the five mono-display worlds (Tawny = Plex",
        "// The five worlds whose ground-preserving clones land past a window",
        "//! the wider doc web states 18 upright worlds and 11 proportional worlds",
        "/// `ornament_face` only ever holds one of the three registered faces",
        "// Measured across the sweep: Quokka 0.21% (3 of 1424) per world",
        "/// every one of the twelve cells (worlds x pitches) measured 0.00",
    ] {
        assert_eq!(
            claims(subset),
            0,
            "the harvester read a SUBSET count as a roster size: {subset:?}"
        );
    }
}
