//! Documentation world counts resolve through the live roster.
//! Starting documents use count tokens; source comments use roster-relative language.
#![cfg(test)]

/// English cardinals a doc might spell out, up to comfortably past any roster
/// awl ships. This is a LEXICON of the harvester's input language, not a roster
/// of the product — a word missing here can only make the harvest find FEWER
/// claims, which is precisely why every law below that relies on it also
/// asserts its harvest is non-empty.
const CARDINAL_WORDS: &[(&str, usize)] = &[
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("thirteen", 13),
    ("fourteen", 14),
    ("fifteen", 15),
    ("sixteen", 16),
    ("seventeen", 17),
    ("eighteen", 18),
    ("nineteen", 19),
    ("twenty", 20),
    ("twenty-one", 21),
    ("twenty-two", 22),
    ("twenty-three", 23),
    ("twenty-four", 24),
    ("twenty-five", 25),
    ("twenty-six", 26),
    ("thirty", 30),
    ("forty", 40),
];

/// A word's cardinal value: an ASCII decimal run, or a spelled-out
/// [`CARDINAL_WORDS`] entry (case-insensitive). `None` for anything else.
pub(crate) fn cardinal(word: &str) -> Option<usize> {
    let w = word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
    if w.is_empty() {
        return None;
    }
    if w.chars().all(|c| c.is_ascii_digit()) {
        return w.parse().ok();
    }
    let lower = w.to_ascii_lowercase();
    CARDINAL_WORDS
        .iter()
        .find(|(name, _)| *name == lower)
        .map(|(_, n)| *n)
}

/// `text` with every `<...>` span replaced by a space, so a word welded to its
/// markup (`<strong>20`) reads as the word it is. An unclosed `<` swallows the
/// rest of the input, which is correct for a scan that must never invent a
/// count out of markup.
fn without_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '<' => {
                depth += 1;
                out.push(' ');
            }
            '>' if depth > 0 => {
                depth -= 1;
                out.push(' ');
            }
            _ if depth > 0 => {}
            _ => out.push(c),
        }
    }
    out
}

/// How far back from the noun a modifier may sit and still be read as counting
/// it: "N worlds", "N curated theme worlds". Three words covers every
/// phrasing the doc web actually uses and stops well short of swallowing a
/// previous sentence's number.
const MODIFIER_WINDOW: usize = 3;

/// Every count claim about the PLURAL noun `worlds` in `text`, as
/// `(quoted phrase, claimed count)`.
///
/// Walks left from each occurrence of the word from at most
/// [`MODIFIER_WINDOW`] words back, taking the FIRST cardinal it meets — so
/// "N curated theme worlds" harvests N and "each world pairs its own"
/// harvests nothing.
///
/// TAGS ARE STRIPPED FIRST, and that is load-bearing rather than tidy: this
/// scan reads a markdown doc and its HTML mirror, and in the mirror the number
/// is welded to its tag (`<strong>20 worlds`). Trimming punctuation off the
/// word's ends leaves `strong>20`, which is not a cardinal — so the first draft
/// of this law read the page as stating NO count and its own presence floor is
/// what caught that. One grammar for both surfaces, with the markup removed
/// before the words are counted.
fn world_counts(text: &str) -> Vec<(String, usize)> {
    let flat = without_tags(text);
    let mut out = Vec::new();
    let words: Vec<&str> = flat.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        let bare = w
            .trim_matches(|c: char| !c.is_ascii_alphanumeric())
            .to_ascii_lowercase();
        if bare != "worlds" {
            continue;
        }
        let lo = i.saturating_sub(MODIFIER_WINDOW);
        for j in (lo..i).rev() {
            if let Some(n) = cardinal(words[j]) {
                out.push((words[lo..=i].join(" "), n));
                break;
            }
        }
    }
    out
}

/// GUIDE.md's ONE generated block is a keys table, never prose about worlds;
/// stripping it keeps the scans below reading only hand-written text, exactly
/// as `keytoken::tests`' own scans do.
fn without_generated_table(doc: &str) -> String {
    const BEGIN: &str = "<!-- GENERATED:keys-reference:BEGIN -->";
    const END: &str = "<!-- GENERATED:keys-reference:END -->";
    match (doc.find(BEGIN), doc.find(END)) {
        (Some(s), Some(e)) => format!("{}{}", &doc[..s], &doc[e + END.len()..]),
        _ => doc.to_string(),
    }
}

/// THE BAN. No doc awl renders may type a world count: the roster answers it
/// through `{{count:worlds}}`. Enrolled from
/// [`crate::embedded_docs::STARTING_DOCS`] — the same owner every other
/// starting-doc law reads — so a doc added to that seam is swept the day it
/// arrives rather than the day someone remembers this file.
#[test]
fn the_starting_docs_state_no_literal_world_count() {
    let _g = crate::testlock::serial();
    let mut literal: Vec<String> = Vec::new();
    for (name, doc) in crate::embedded_docs::STARTING_DOCS {
        for (phrase, n) in world_counts(&without_generated_table(doc)) {
            literal.push(format!("{name} types {phrase:?} (claims {n})"));
        }
    }
    assert!(
        literal.is_empty(),
        "a doc awl renders types a world count instead of asking the roster \
         for it — write `{{{{count:worlds}}}} worlds` and let \
         `keytoken::COUNTS` answer (`theme::THEMES` has {} today):\n{}",
        crate::theme::THEMES.len(),
        literal.join("\n")
    );
}

/// Sentence boundaries do not grant a literal count an exemption. The welcome
/// once said "opens the worlds. There are nineteen"; `world_counts` correctly
/// refuses to associate a prior sentence's number with a later noun, so this
/// paragraph-level enrolment pins the actual teaching at its source.
#[test]
fn the_welcome_worlds_paragraph_asks_the_roster_for_its_count() {
    let _g = crate::testlock::serial();
    let paragraph = crate::embedded_docs::WELCOME_MD
        .split("\n\n")
        .find(|paragraph| paragraph.contains("opens the worlds"))
        .expect("the welcome still teaches the theme picker");
    assert!(
        paragraph.contains("{{count:worlds}}"),
        "the welcome's theme-picker paragraph must source its count from the roster: {paragraph}"
    );
}

/// THE PRESENCE AND VALUE FLOOR — the ban's companion, and the reason the ban
/// is not satisfiable by deleting its own subject. Renders the starting docs
/// through the real substitution seam on every surface and asserts (a) the
/// count a reader is SHOWN is the live roster size, and (b) at least one such
/// claim exists to be shown at all.
#[test]
fn the_rendered_starting_docs_state_the_live_world_count() {
    use crate::commands::Platform;
    use crate::convention::Convention;
    let _g = crate::testlock::serial();
    let want = crate::theme::THEMES.len();
    let mut seen = 0usize;
    for (name, doc) in crate::embedded_docs::STARTING_DOCS {
        for convention in [Convention::Mac, Convention::Linux] {
            for platform in [Platform::Native, Platform::Web] {
                let rendered = crate::keytoken::render_key_tokens(
                    &without_generated_table(doc),
                    convention,
                    platform,
                );
                for (phrase, n) in world_counts(&rendered) {
                    seen += 1;
                    assert_eq!(
                        n, want,
                        "{name} shows {phrase:?} under {convention:?}/{platform:?}, \
                         but `theme::THEMES` carries {want} worlds"
                    );
                }
            }
        }
    }
    assert!(
        seen > 0,
        "no doc in `embedded_docs::STARTING_DOCS` states a world count at all \
         — `the_starting_docs_state_no_literal_world_count` is then vacuous, \
         which is the shape it exists to rule out. Restore the count (as \
         `{{{{count:worlds}}}} worlds`) or retire both laws together."
    );
}

/// THE SOURCING CROSS-CHECK, against the generated-document hazard: a figure
/// read from the wrong roster is wrong WITH a law behind it. `WORLDS.md`'s
/// at-a-glance table is a different document, parsed a different way (bolded
/// row labels), and is held to `theme::THEMES` by
/// `reference::law::rosters::worlds`. If `keytoken::COUNTS`' `worlds` entry
/// ever pointed at some other roster in the crate — display faces, overlay
/// kinds, catalog commands — the two answers would part company here.
// `WORLDS_MD` is embedded under `not(wasm32)`; the cross-check follows it. The
// laws above carry no such gate — they are about docs the wasm build itself
// renders.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_world_count_token_agrees_with_the_worlds_md_roster() {
    let _g = crate::testlock::serial();
    let token = crate::keytoken::count_token_label("worlds")
        .expect("`{{count:worlds}}` is a known count token");
    let listed = crate::embedded_docs::WORLDS_MD
        .lines()
        .filter(|l| l.trim_start().starts_with("| **"))
        .filter_map(|l| l.split("**").nth(1))
        .filter(|n| crate::theme::THEMES.iter().any(|t| t.name == *n))
        .count();
    assert_eq!(
        token,
        listed.to_string(),
        "`{{{{count:worlds}}}}` renders {token}, but WORLDS.md's at-a-glance \
         table lists {listed} worlds — the count token and the world list are \
         reading different rosters"
    );
}

/// An unknown count name must be LOUD in the rendered doc, not silent — the
/// same contract `{{key:}}`/`{{cmd:}}` carry, so a typo'd token is visible to
/// a reader even outside the laws above.
#[test]
fn an_unknown_count_token_renders_a_visible_marker() {
    use crate::commands::Platform;
    use crate::convention::Convention;
    let _g = crate::testlock::serial();
    assert_eq!(
        crate::keytoken::render_key_tokens("{{count:galaxies}}", Convention::Mac, Platform::Native),
        "[[unknown-count:galaxies]]"
    );
    assert_eq!(
        crate::keytoken::render_key_tokens("{{count:worlds}}", Convention::Mac, Platform::Native),
        crate::theme::THEMES.len().to_string()
    );
}
