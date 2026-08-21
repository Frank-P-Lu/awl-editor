//! Source-offset model for the GitHub-compatible footnote extension.
//!
//! pulldown-cmark owns syntax recognition; this module owns the product rules the
//! event stream deliberately does not: display numbers follow first REFERENCE
//! appearance (never definition position), source ranges stay exact for WYSIWYG
//! reveal, and an activation resolves to the first recognized definition. The
//! parser's new/GitHub-compatible mode leaves undefined and malformed references
//! as ordinary text. It recognizes repeated definitions, so this model explicitly
//! enrolls only the first and lets later definitions remain legible literal source
//! rather than inventing a second meaning.

use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::BTreeMap;
use std::ops::Range;

/// One recognized reference, in authored order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteReference {
    pub range: Range<usize>,
    pub label: String,
    pub number: usize,
    pub occurrence: usize,
    pub definition: Range<usize>,
}

/// One source line inside a recognized definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionLine {
    /// The editable structural prefix: `[^label]: ` on the first line, or the
    /// indentation on a continued line. Empty for a lazy continuation.
    pub prefix: Range<usize>,
    /// The prose on this line, excluding its structural prefix and newline.
    pub content: Range<usize>,
}

/// One recognized definition. `number` is assigned after every referenced
/// label, so an unreferenced definition remains composed and cannot perturb the
/// first-appearance numbering of real references.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FootnoteDefinition {
    pub range: Range<usize>,
    pub label: String,
    pub number: usize,
    pub lines: Vec<DefinitionLine>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Footnotes {
    pub references: Vec<FootnoteReference>,
    pub definitions: Vec<FootnoteDefinition>,
}

/// The parser options every awl Markdown consumer shares. Kept here so adding
/// footnotes cannot leave the live renderer and exporters on different dialects.
pub(crate) const OPTIONS: Options = Options::ENABLE_TASKLISTS
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_FOOTNOTES);

fn key(label: &str) -> String {
    // pulldown matches GitHub-style labels case-insensitively. Lowercasing here
    // preserves that equivalence in the side table, including Unicode labels,
    // without changing the authored label stored beside each range.
    label.chars().flat_map(char::to_lowercase).collect()
}

fn definition_lines(source: &str, range: &Range<usize>) -> Vec<DefinitionLine> {
    let mut lines = Vec::new();
    let mut start = range.start;
    let mut first = true;
    while start < range.end {
        let rel_end = source[start..range.end]
            .find('\n')
            .map_or(range.end, |n| start + n);
        let line = &source[start..rel_end];
        let prefix_len = if first {
            line.find("]:").map_or(0, |marker| {
                let mut end = marker + 2;
                while end < line.len() && matches!(line.as_bytes()[end], b' ' | b'\t') {
                    end += 1;
                }
                end
            })
        } else {
            line.as_bytes()
                .iter()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count()
        };
        let prefix_end = start + prefix_len;
        lines.push(DefinitionLine {
            prefix: start..prefix_end,
            content: prefix_end..rel_end,
        });
        first = false;
        if rel_end == range.end {
            break;
        }
        start = rel_end + 1;
    }
    lines
}

/// Build the product model from the exact event stream another consumer is
/// already walking. This avoids a second parse in both the renderer and export
/// fold while keeping [`footnotes`] available to action hit-testing.
pub(crate) fn from_events<'a>(source: &str, events: &[(Event<'a>, Range<usize>)]) -> Footnotes {
    let mut definitions = Vec::new();
    let mut seen_definitions = BTreeMap::new();
    for (event, range) in events {
        if let Event::Start(Tag::FootnoteDefinition(label)) = event {
            let normalized = key(label);
            if seen_definitions.insert(normalized, ()).is_none() {
                definitions.push((label.to_string(), range.clone()));
            }
        }
    }
    let definition_by_key: BTreeMap<String, Range<usize>> = definitions
        .iter()
        .map(|(label, range)| (key(label), range.clone()))
        .collect();

    let mut numbers: BTreeMap<String, usize> = BTreeMap::new();
    let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();
    let mut references = Vec::new();
    for (event, range) in events {
        let Event::FootnoteReference(label) = event else {
            continue;
        };
        let normalized = key(label);
        let Some(definition) = definition_by_key.get(&normalized) else {
            // Defensive only: GitHub-compatible pulldown emits undefined refs as
            // Text. Staying literal is still the safe degradation if that changes.
            continue;
        };
        let next = numbers.len() + 1;
        let number = *numbers.entry(normalized.clone()).or_insert(next);
        let occurrence = occurrences.entry(normalized).or_insert(0);
        *occurrence += 1;
        references.push(FootnoteReference {
            range: range.clone(),
            label: label.to_string(),
            number,
            occurrence: *occurrence,
            definition: definition.clone(),
        });
    }

    // Definitions which have no reference still preserve their meaning in rich
    // export and WYSIWYG prose, but join only AFTER reference numbering is fixed.
    let mut next = numbers.len() + 1;
    let definitions = definitions
        .into_iter()
        .map(|(label, range)| {
            let normalized = key(&label);
            let number = match numbers.get(&normalized) {
                Some(number) => *number,
                None => {
                    let number = next;
                    next += 1;
                    numbers.insert(normalized, number);
                    number
                }
            };
            FootnoteDefinition {
                lines: definition_lines(source, &range),
                range,
                label,
                number,
            }
        })
        .collect();

    Footnotes {
        references,
        definitions,
    }
}

/// Parse a document for action/hit-test consumers which do not already own an
/// event stream.
pub fn footnotes(source: &str) -> Footnotes {
    let events: Vec<_> = Parser::new_ext(source, OPTIONS)
        .into_offset_iter()
        .collect();
    from_events(source, &events)
}

/// Raw document line of the definition activated by `byte`, if `byte` touches a
/// recognized reference. This is the payload [`App::jump_to_line`] expects, so
/// activation inherits its one fold-reveal placement path.
pub fn footnote_target_at(source: &str, byte: usize) -> Option<usize> {
    let model = footnotes(source);
    let target = model
        .references
        .iter()
        .find(|reference| reference.range.start <= byte && byte < reference.range.end)?
        .definition
        .start;
    Some(
        source[..target]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_reference_numbers_reordered_repeated_and_unicode_labels() {
        let source = concat!(
            "[^earlier]: defined first\n\n",
            "B[^β] A[^earlier] B again[^β]\n\n",
            "[^β]: unicode definition\n",
        );
        let model = footnotes(source);
        let refs: Vec<_> = model
            .references
            .iter()
            .map(|reference| {
                (
                    reference.label.as_str(),
                    reference.number,
                    reference.occurrence,
                )
            })
            .collect();
        assert_eq!(refs, [("β", 1, 1), ("earlier", 2, 1), ("β", 1, 2)]);
        let defs: Vec<_> = model
            .definitions
            .iter()
            .map(|definition| (definition.label.as_str(), definition.number))
            .collect();
        assert_eq!(defs, [("earlier", 2), ("β", 1)]);
    }

    #[test]
    fn undefined_duplicate_and_malformed_source_is_never_invented_into_the_model() {
        let source = concat!(
            "missing [^none] malformed [^broken\n\n",
            "[^one]: first definition\n",
            "[^one]: duplicate definition\n",
        );
        let model = footnotes(source);
        assert!(model.references.is_empty());
        assert_eq!(
            model.definitions.len(),
            1,
            "only the first recognized definition owns footnote semantics"
        );
        assert_eq!(model.definitions[0].label, "one");
        assert!(source.contains("[^none]"));
        assert!(source.contains("[^broken"));
        assert!(source.contains("duplicate definition"));
    }

    #[test]
    fn multiline_definition_records_exact_editable_prefix_and_content_ranges() {
        let source = "Text[^note].\n\n[^note]: first line\n    continued\n\n    second paragraph\n";
        let definition = &footnotes(source).definitions[0];
        let pieces: Vec<_> = definition
            .lines
            .iter()
            .map(|line| (&source[line.prefix.clone()], &source[line.content.clone()]))
            .collect();
        assert_eq!(
            pieces,
            [
                ("[^note]: ", "first line"),
                ("    ", "continued"),
                ("", ""),
                ("    ", "second paragraph"),
            ]
        );
    }

    #[test]
    fn activation_returns_the_raw_definition_line() {
        let source = "top\nSee this[^x].\n\n[^x]: answer\n";
        let byte = source.find("[^x]").unwrap() + 2;
        assert_eq!(footnote_target_at(source, byte), Some(3));
        assert_eq!(footnote_target_at(source, 0), None);
    }
}
