//! Numeric field filtering, shared by typed characters and bulk insertion.

use crate::textbox::TextBox;

pub(super) fn filtered(field: &TextBox, text: &str) -> String {
    let selection = field.selection_range();
    let mut seen = [false; 2];
    let mut retained = [false; 2];
    for (i, c) in field.text().chars().enumerate() {
        if let Some(slot) = punctuation(c) {
            seen[slot] = true;
            if !selection.is_some_and(|(start, end)| (start..end).contains(&i)) {
                retained[slot] = true;
            }
        }
    }
    let mut result = String::new();
    for c in text.chars() {
        let slot = punctuation(c);
        if !c.is_ascii_digit() && !slot.is_some_and(|i| !seen[i]) {
            continue;
        }
        // The first admitted character replaces the selection. Rejected
        // punctuation must not delete it or change later admission decisions.
        if result.is_empty() {
            seen = retained;
        }
        if let Some(i) = slot {
            seen[i] = true;
        }
        result.push(c);
    }
    result
}

fn punctuation(c: char) -> Option<usize> {
    match c {
        '.' => Some(0),
        '%' => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_numeric_filter_matches_the_original_scalar_rule_at_every_selection_edge() {
        let _guard = crate::testlock::serial();
        for seed in ["", "1.5%", "日本語.0%"] {
            for prefix in 0..=seed.chars().count() {
                for payload in [".%.%", "1.2%3.4%", "日本/語", "...0%%.9"] {
                    let mut scalar = TextBox::seeded_selecting_prefix(seed, prefix);
                    let mut bulk = scalar.clone();
                    for c in payload.chars() {
                        let text = scalar.text();
                        if c.is_ascii_digit()
                            || (c == '.' && !text.contains('.'))
                            || (c == '%' && !text.contains('%'))
                        {
                            scalar.insert(c);
                        }
                    }
                    let admitted = filtered(&bulk, payload);
                    bulk.insert_text(&admitted);
                    assert_eq!(bulk, scalar, "{seed:?} prefix={prefix} payload={payload:?}");
                }
            }
        }
    }
}
