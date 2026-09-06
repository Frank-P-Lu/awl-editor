//! src/spell/personal.rs — the PERSONAL (user-added) dictionary's own
//! near-miss scan: the half of spell-checking that reads `user_words` when a
//! CORRECTION is asked for, not merely when a word is checked.
//!
//! The bundled Hunspell dictionary answers `check` and `suggest` alike; the
//! personal list only ever answered `check`, so a typo one letter off a word
//! the user added on purpose was never offered that word back. This module is
//! the whole repair, and it is deliberately NOT a completion engine: it offers
//! a personal word only when that word sits within a bounded EDIT DISTANCE of
//! what was typed — the same "correction" shape a bundled suggestion has.
//!
//! Kept out of `spell.rs` because the mechanism is self-contained (a distance
//! function, a recasing rule, and one bounded scan) and `spell.rs` is already
//! a grandfathered-large file whose subject is the scope of the check, not the
//! shape of a suggestion.

use std::collections::HashSet;

/// How close a personal word must sit to a typed word (Levenshtein, char-wise)
/// to be offered as a correction — "nearby", not a second dictionary lookup.
/// `2` catches an ordinary one-letter slip AND a two-letter transposition (a
/// swap costs 2 substitutions under plain Levenshtein), while still excluding
/// an unrelated word.
pub(super) const MAX_DISTANCE: usize = 2;

/// The personal words within [`MAX_DISTANCE`] edits of `word` (compared
/// lowercased, since every stored word is lowercase), closest first and
/// alphabetical on a tie, each recased to match `word`'s own capitalization.
///
/// An EXACT match (distance 0) is excluded: a word that is already spelled the
/// way the personal dictionary holds it has nothing to be corrected to.
///
/// Linear over the whole set on purpose. The personal dictionary is a
/// hand-grown vocabulary — the same list a picker is expected to show in one
/// card — so the scan costs nothing worth a prefix index, and `spellbook` is
/// left untouched.
pub(super) fn near_misses(user_words: &HashSet<String>, word: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let mut hits: Vec<(usize, &String)> = user_words
        .iter()
        .filter(|w| w.as_str() != lower)
        .filter_map(|w| {
            let d = edit_distance(&lower, w);
            (d <= MAX_DISTANCE).then_some((d, w))
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    hits.into_iter()
        .map(|(_, w)| recase_like(word, w))
        .collect()
}

/// Merge `personal` AHEAD of `bundled`, de-duplicated case-insensitively.
///
/// The order is the product decision: a personal near-miss is the user's OWN
/// vocabulary, added deliberately, so it must never lose a slot to a bundled
/// guess merely because `spellbook`'s internal ranking put that guess first.
/// The precedence also has to hold here rather than at the picker, because the
/// Spell card truncates its list to `OverlayKind::MAX_SUGGESTIONS` AFTER
/// assembly — a personal word merged in behind the bundled ones could be
/// assembled and then silently cut.
pub(super) fn merge_ahead(personal: Vec<String>, bundled: Vec<String>) -> Vec<String> {
    if personal.is_empty() {
        return bundled;
    }
    let mut seen: HashSet<String> = personal.iter().map(|w| w.to_lowercase()).collect();
    let mut merged = personal;
    merged.extend(
        bundled
            .into_iter()
            .filter(|w| seen.insert(w.to_lowercase())),
    );
    merged
}

/// Plain (Levenshtein) char-wise edit distance — insert/delete/substitute each
/// cost 1. Pure and small-input only.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Recase `lower` (a lowercase-stored personal word) to match `sample`'s own
/// capitalization shape — the convention a bundled Hunspell suggestion already
/// follows against a capitalized query, so the two halves of one list cannot
/// disagree about case. ALL-CAPS sample → ALL-CAPS result; a capitalized
/// sample → capitalized result; anything else → `lower` verbatim.
fn recase_like(sample: &str, lower: &str) -> String {
    let has_alpha = sample.chars().any(char::is_alphabetic);
    let all_upper = has_alpha
        && sample.chars().filter(|c| c.is_alphabetic()).count() > 1
        && sample
            .chars()
            .all(|c| !c.is_alphabetic() || c.is_uppercase());
    if all_upper {
        return lower.to_uppercase();
    }
    if sample.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = lower.chars();
        return match chars.next() {
            Some(f) => f.to_uppercase().chain(chars).collect(),
            None => String::new(),
        };
    }
    lower.to_string()
}
