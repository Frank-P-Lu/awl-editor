use super::*;

#[test]
fn spell_picker_lists_suggestions_and_carries_target() {
    // Three corrections for a word flagged at line 2, cols 6..13.
    let sugg = vec![
        "receive".to_string(),
        "relieve".to_string(),
        "reprieve".to_string(),
    ];
    let ov = OverlayState::new_spell(sugg.clone(), (2, 6, 13), "recieve".to_string());
    assert_eq!(ov.kind.as_str(), "spell");
    // Rows are the suggestions in order (best first), then ONE appended "Add
    // '<word>' to dictionary" row; the top suggestion is selected.
    let rows = ov.item_strings();
    assert_eq!(
        &rows[..sugg.len()],
        &sugg[..],
        "the suggestions lead, in order"
    );
    assert_eq!(
        rows.last().map(String::as_str),
        Some(super::state::add_to_dictionary_label("recieve").as_str()),
        "the LAST row is the Add-to-dictionary affordance"
    );
    assert_eq!(ov.selected_value(), Some("receive"));
    // The target span is carried so the accept can replace the word.
    assert_eq!(ov.spell_target, Some((2, 6, 13)));
    // No git / dir markers on the suggestion rows.
    assert!(
        ov.item_strings()
            .iter()
            .all(|s| !s.contains('•') && !s.ends_with('/'))
    );
    // The add row is flagged (only it) and carries the word for the accept effect.
    assert!(
        !ov.selected_is_add_to_dictionary(),
        "a suggestion row is not the add row"
    );
    assert_eq!(ov.add_word.as_deref(), Some("recieve"));
    let last = ov.items.len() - 1;
    assert!(
        matches!(ov.rows[ov.items[last]].meta, RowMeta::SpellAdd),
        "the last corpus row is the add row"
    );
    // The hint names the ↵ action (replace) after the universal jump lead, flat
    // picker (no descend).
    assert_eq!(
        OverlayKind::Spell.hint(),
        "type to filter   \u{21B5} replace"
    );
}

/// THE TOP-FIVE CAP, as a LAW swept over every corpus length from 0 to
/// 20: `new_spell` truncates the dictionary's ordered corrections to the top
/// [`OverlayKind::MAX_SUGGESTIONS`] (ranking/order preserved — a longer list has
/// only its TAIL dropped, never scrolled or reached via a More/ellipsis button),
/// then appends exactly ONE fixed, terminal add row. Also asserts the picker's own
/// `window_rows` never needs to exceed what the (now-capped) corpus holds — "no
/// scrolling" is structural, not merely an untested UI choice.
#[test]
fn spell_picker_caps_corrections_to_the_top_five_across_corpora() {
    let word = "recieve".to_string();
    let target = (0usize, 0usize, 7usize);
    for n in 0..=20usize {
        let all: Vec<String> = (0..n).map(|i| format!("correction{i}")).collect();
        let ov = OverlayState::new_spell(all.clone(), target, word.clone());
        let kept = n.min(OverlayKind::MAX_SUGGESTIONS);
        assert_eq!(
            ov.items.len(),
            kept + 1,
            "n={n}: expected {kept} correction(s) + the one add row"
        );
        assert!(
            ov.items.len() <= OverlayKind::Spell.window_rows(),
            "n={n}: the picker's whole corpus always fits the window — it can never be forced to scroll"
        );
        let rows = ov.item_strings();
        assert_eq!(
            &rows[..kept],
            &all[..kept],
            "n={n}: order preserved, only the TAIL past the cap is dropped"
        );
        assert_eq!(
            rows.last().map(String::as_str),
            Some(super::state::add_to_dictionary_label(&word).as_str()),
            "n={n}: the add row is always the last row"
        );
        let last_ci = *ov.items.last().unwrap();
        assert!(
            matches!(ov.rows[last_ci].meta, RowMeta::SpellAdd),
            "n={n}: the last corpus entry is flagged as the add row"
        );
        assert_eq!(
            ov.rows
                .iter()
                .filter(|r| matches!(r.meta, RowMeta::SpellAdd))
                .count(),
            1,
            "n={n}: exactly one add row, never more"
        );
    }
}

/// The exact boundary corpora (0/1/5/6/20), spelled out one at a time so the
/// literal cases are pinned individually, alongside the
/// general sweep above.
#[test]
fn spell_picker_named_corpora_0_1_5_6_20() {
    let word = "teh".to_string();
    let target = (0usize, 0usize, 3usize);
    let mk = |n: usize| -> Vec<String> { (0..n).map(|i| format!("word{i}")).collect() };

    // 0 corrections -> just the add row (always present, even with nothing to suggest).
    let ov0 = OverlayState::new_spell(mk(0), target, word.clone());
    assert_eq!(ov0.items.len(), 1, "0 corrections -> the add row alone");
    assert!(matches!(ov0.rows[ov0.items[0]].meta, RowMeta::SpellAdd));

    // 1 correction -> 1 + add.
    let ov1 = OverlayState::new_spell(mk(1), target, word.clone());
    assert_eq!(ov1.items.len(), 2, "1 correction -> that correction + add");

    // 5 corrections (exactly the cap) -> all 5 kept + add, nothing dropped.
    let ov5 = OverlayState::new_spell(mk(5), target, word.clone());
    assert_eq!(
        ov5.items.len(),
        6,
        "5 corrections -> all 5 + add, at the cap exactly"
    );
    assert_eq!(&ov5.item_strings()[..5], &mk(5)[..]);

    // 6 corrections (one over the cap) -> still top 5 + add == 6 rows; the 6th
    // never shows, no scrollbar/More button reaches it.
    let ov6 = OverlayState::new_spell(mk(6), target, word.clone());
    assert_eq!(
        ov6.items.len(),
        6,
        "6 corrections still show only 5 + add == 6 rows"
    );
    assert_eq!(
        &ov6.item_strings()[..5],
        &mk(6)[..5],
        "the top 5, not the last 5"
    );
    assert!(
        !ov6.item_strings().iter().any(|s| s == "word5"),
        "the 6th (dropped) correction never shows"
    );

    // 20 corrections -> still top 5 + add == 6 rows.
    let ov20 = OverlayState::new_spell(mk(20), target, word.clone());
    assert_eq!(
        ov20.items.len(),
        6,
        "20 corrections still show only 6 rows total"
    );
    assert_eq!(&ov20.item_strings()[..5], &mk(20)[..5]);
}
