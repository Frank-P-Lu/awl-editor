use super::*;

#[test]
fn hint_teaches_descend_only_for_navigable_kinds() {
    // The NON-faceting navigable explorers (Project / MoveDest) teach the
    // select-vs-descend asymmetry — but now with the UNIFIED glyph vocabulary:
    // ↵ selects/accepts FIRST (primary), then → descends, ← ascends (the old
    // ASCII `->/C-f` / `<-/C-b` word-chords are gone). Browse is a FACETED
    // explorer, so its ←/→ teach the LENS, not descend (descend rides Enter).
    for k in [OverlayKind::Project, OverlayKind::MoveDest] {
        let h = k.hint();
        // Unicode arrows, never the old ASCII/word-chord forms.
        assert!(
            h.contains('\u{2192}'),
            "{k:?} hint should teach → descend: {h}"
        );
        assert!(
            h.contains('\u{2190}'),
            "{k:?} hint should teach ← ascend: {h}"
        );
        assert!(
            !h.contains("C-f") && !h.contains("->"),
            "{k:?} no ASCII chord: {h}"
        );
        // The universal type-to-filter cell LEADS the line, then the primary ↵ Return action.
        assert!(
            h.starts_with("type to filter"),
            "{k:?} hint leads with type to filter: {h}"
        );
        assert!(h.contains("\u{21B5}"), "{k:?} hint names ↵ Return: {h}");
    }
    // Project ↵ SELECTS; MoveDest ↵ MOVES.
    assert!(OverlayKind::Project.hint().contains("\u{21B5} select"));
    assert!(OverlayKind::MoveDest.hint().contains("move here"));
    // The ordinary FACETED pickers teach ←/→ lens, not
    // ->/C-f descend, and each starts with the ↵ Return glyph. (The THEME picker
    // retired its lens strip 2026-07-15 — it is checked below as a FLAT picker.)
    for k in [OverlayKind::Goto, OverlayKind::Browse, OverlayKind::History] {
        let h = k.hint();
        assert!(!h.contains("C-f"), "{k:?} facets, no descend hint: {h}");
        assert!(
            h.contains("\u{2190}/\u{2192} lens"),
            "{k:?} hint should teach ←/→ lens: {h}"
        );
        assert!(
            h.starts_with("type to filter"),
            "{k:?} hint leads with type to filter: {h}"
        );
    }
    assert_eq!(
        OverlayKind::Command.hint(),
        "type to filter   ↵ choose   ←/→ category   esc close"
    );
    // The FLAT theme picker teaches ↵ keep + esc revert, and NO lens axis (its strip
    // was retired) — type to filter still leads.
    let th = OverlayKind::Theme.hint();
    assert!(
        th.starts_with("type to filter"),
        "theme hint leads with type to filter: {th}"
    );
    assert!(th.contains("\u{21B5} keep"), "theme ↵ keeps: {th}");
    assert!(
        th.contains("esc") && th.contains("revert"),
        "theme esc reverts: {th}"
    );
    assert!(
        !th.contains("lens"),
        "the flat theme picker teaches no lens: {th}"
    );
    // Browse ↵ still OPENS (a folder descends / a file opens) and ⌫ ascends.
    assert!(OverlayKind::Browse.hint().contains("\u{21B5} open"));
    assert!(OverlayKind::Browse.hint().contains("\u{232B} up"));
}

/// The SHARED hint formatter produces ONE consistent shape for every picker:
/// `glyph SPACE label`, actions joined by the single `HINT_SEP`, the universal
/// `type to filter` FIRST, then the primary (↵), and cancel (esc) — where present —
/// LAST and lowercase. This is the pass-2 unification law: a sample of overlays
/// must all read identically formed.
#[test]
fn hint_formatter_is_consistent_across_pickers() {
    // The formatter itself: `glyph label`, HINT_SEP-joined, in order.
    let sample = [
        HintAction {
            glyph: "\u{21B5}",
            label: "keep",
        },
        HintAction {
            glyph: "\u{2190}/\u{2192}",
            label: "lens",
        },
        HintAction {
            glyph: "esc",
            label: "revert",
        },
    ];
    assert_eq!(
        format_hint(&sample),
        format!("\u{21B5} keep{HINT_SEP}\u{2190}/\u{2192} lens{HINT_SEP}esc revert")
    );
    assert_eq!(
        HINT_SEP, "   ",
        "the one canonical separator is a triple space"
    );

    // Every kind's rendered hint obeys the shape: each action is `glyph SPACE
    // label` (exactly one space), the separator is HINT_SEP, the universal JUMP
    // lead (move → type-to-filter) comes first, the ↵ primary follows it, and any
    // cancel action is the lowercase `esc` (never `Esc`) LAST.
    for k in OverlayKind::ALL {
        let actions = k.hint_actions();
        assert!(
            actions.len() >= 2,
            "{k:?} must teach the filter lead + ↵ primary"
        );
        // The universal jump-affordance lead: type to filter —
        // the discoverability fix for "you can only go one by one".
        assert_eq!(actions[0].glyph, "type", "{k:?} leads with type to filter");
        assert_eq!(
            actions[0].label, "to filter",
            "{k:?} lead cell reads type to filter"
        );
        assert_eq!(
            actions[1].glyph, "\u{21B5}",
            "{k:?} ↵ primary follows the jump lead"
        );
        // Cancel-last + lowercase esc: no action names capital `Esc`; if any
        // action is the esc cancel, it is the LAST one.
        for (i, a) in actions.iter().enumerate() {
            assert_ne!(a.glyph, "Esc", "{k:?} esc must be lowercase");
            if a.glyph == "esc" {
                assert_eq!(i, actions.len() - 1, "{k:?} esc cancel sits last");
            }
        }
        // The rendered line == the formatter over the same actions (one owner).
        let h = k.hint();
        assert_eq!(
            h,
            format_hint(&actions),
            "{k:?} hint routes through format_hint"
        );
        // Separator discipline: the ONLY multi-space runs are the HINT_SEP joins,
        // so splitting on HINT_SEP yields exactly `actions.len()` `glyph label` cells.
        let cells: Vec<&str> = h.split(HINT_SEP).collect();
        assert_eq!(cells.len(), actions.len(), "{k:?} cells == actions: {h}");
        for cell in cells {
            assert!(
                !cell.contains("  "),
                "{k:?} no stray double space in {cell:?}"
            );
        }
    }
}
