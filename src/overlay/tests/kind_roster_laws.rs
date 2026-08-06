use super::*;

/// The SHARED empty-state owner: a picker with NO matching rows reports a calm
/// message — the universal "no matches" when a QUERY filtered a non-empty corpus
/// out, the per-kind [`OverlayKind::empty_corpus_message`] when the CORPUS itself
/// is empty — and Enter on it is already a no-op (nothing selected). This is the
/// pass-3 unification: every picker shares one empty-state, not a blank card.
#[test]
fn empty_state_message_is_shared_and_accept_is_a_no_op() {
    // A non-empty corpus filtered to nothing by a query → the universal message.
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![],
    );
    ov.push('z'); // matches neither → items empty
    assert!(ov.items.is_empty());
    assert_eq!(ov.empty_notice().as_deref(), Some("no matches"));
    // Enter accept is a no-op: nothing is selected on an empty list.
    assert_eq!(ov.selected_value(), None, "empty list selects nothing");

    // A non-empty list reports NO empty-state (it has rows to show).
    let ov2 = OverlayState::new(OverlayKind::Goto, vec!["alpha.md".into()], vec![], vec![]);
    assert_eq!(
        ov2.empty_notice(),
        None,
        "a picker with rows has no empty-state"
    );

    // An EMPTY corpus reads the per-kind message (query still empty).
    let empty_goto = OverlayState::new(OverlayKind::Goto, vec![], vec![], vec![]);
    assert_eq!(empty_goto.empty_notice().as_deref(), Some("no files here"));
    let empty_hist = OverlayState::new_history(Vec::new(), None, None);
    assert_eq!(empty_hist.empty_notice().as_deref(), Some("no history yet"));
    // Every kind's empty-corpus message is a non-empty calm line (never blank).
    for k in OverlayKind::ALL {
        assert!(
            !k.empty_corpus_message().is_empty(),
            "{k:?} needs an empty line"
        );
    }
}

/// BREADCRUMB LAW: every overlay kind DECLARES a pop-vs-close-all accept class
/// (the no-wildcard match in [`OverlayKind::accept_disposition`] is the real
/// compile-time guard — a future kind won't build until it declares one; this
/// sweep pins the specific classifications so a silent reclassification trips a
/// test). The rule the whole round turns on: Esc/cancel always POPS (uniform, not
/// per-kind); an ACCEPT is Navigate (close the whole stack — you land in the
/// result), ValuePick (pop back to the summoning overlay — you committed a
/// setting), or StayOpen (never closes).
#[test]
fn every_kind_declares_an_accept_disposition() {
    use AcceptDisposition::*;
    for k in OverlayKind::ALL {
        // Exhaustive by construction — this just witnesses each kind resolves.
        let _ = k.accept_disposition();
    }
    // The VALUE-PICKERS pop back to the parent (theme keep / caret apply /
    // dictionary apply commit a setting the summoning overlay was choosing).
    for k in [
        OverlayKind::Theme,
        OverlayKind::Caret,
        OverlayKind::Dictionary,
    ] {
        assert_eq!(
            k.accept_disposition(),
            ValuePick,
            "{k:?} is a value-picker → pop"
        );
    }
    // The NAVIGATORS close the whole stack (open a file, jump, switch project,
    // move a note, restore a version, run a command — you land in the result).
    for k in [
        OverlayKind::Goto,
        OverlayKind::Browse,
        OverlayKind::Project,
        OverlayKind::MoveDest,
        OverlayKind::Spell,
        OverlayKind::History,
        OverlayKind::Command,
    ] {
        assert_eq!(
            k.accept_disposition(),
            Navigate,
            "{k:?} navigates → close-all"
        );
    }
    // The STAY-OPEN kinds never close on accept (trash keeps listing, rebind
    // starts a capture, the settings menu toggles / swaps in place).
    for k in [
        OverlayKind::Assets,
        OverlayKind::Keybindings,
        OverlayKind::Settings,
    ] {
        assert_eq!(
            k.accept_disposition(),
            StayOpen,
            "{k:?} stays open on accept"
        );
    }
}

/// THE OVERLAY-TITLES ROUND: every kind names itself with a nonempty, lowercase
/// title (`OverlayKind::title`) — the no-wildcard law a future kind must satisfy
/// before it compiles. Titles are also pairwise DISTINCT (so a sidecar `overlay.
/// title` read unambiguously identifies which picker is open).
#[test]
fn every_kind_names_itself_with_a_nonempty_distinct_title() {
    use std::collections::HashSet;
    let mut titles: HashSet<&'static str> = HashSet::new();
    for k in OverlayKind::ALL {
        let t = k.title();
        assert!(!t.is_empty(), "{k:?} has no title");
        assert_eq!(t, t.to_lowercase(), "{k:?}'s title {t:?} must be lowercase");
        assert!(
            titles.insert(t),
            "{k:?}'s title {t:?} collides with another kind's"
        );
    }
    // Prompt surfaces and the pointer-anchored context menu orient without a
    // title prefix; every other kind draws one.
    for k in [
        OverlayKind::Rename,
        OverlayKind::InsertLink,
        OverlayKind::KeepName,
        OverlayKind::Context,
    ] {
        assert!(
            !k.draws_title_prefix(),
            "{k:?} should not draw the title prefix"
        );
    }
    for k in OverlayKind::ALL {
        if !matches!(
            k,
            OverlayKind::Rename
                | OverlayKind::InsertLink
                | OverlayKind::KeepName
                | OverlayKind::Context
        ) {
            assert!(k.draws_title_prefix(), "{k:?} should draw the title prefix");
        }
    }
}

/// MODE-STRING ROUND-TRIP LAW (born from the KeepName drift audit): every kind's
/// sidecar mode string resolves back to the kind via [`OverlayKind::from_mode`] —
/// the lookup the headless capture path uses to consult the REAL per-kind owners
/// (`draws_title_prefix`) instead of hand-listing mode strings (the aligned copy
/// in `capture/modes.rs` that silently kept drawing the title prefix on the
/// KeepName minibuffer until this round caught it in a capture PNG). An unknown
/// string resolves to None (fail-visible: the capture then keeps the title).
#[test]
fn every_mode_string_round_trips_through_from_mode() {
    for k in OverlayKind::ALL {
        assert_eq!(
            OverlayKind::from_mode(k.as_str()),
            Some(k),
            "{k:?}'s mode string must resolve back to itself"
        );
    }
    assert_eq!(OverlayKind::from_mode("not-a-mode"), None);
}

/// BREADCRUMB KINDS ARE VALUE-BASED, never positional. A `return_to` breadcrumb
/// stores an [`OverlayKind`] by VALUE and re-summons it by that value
/// ([`make_overlay`](crate::actions::ActionCtx) is keyed on the kind, not an
/// index), so its identity is its stable `as_str` NAME. This guards against the
/// exact class of bug the lens-fold round could have caused — retiring a sibling
/// variant SHIFTING enum positions and re-aiming a stored breadcrumb at a
/// different picker ("return to palette" decoding as "return to Goto/recents").
/// (a) `as_str` is a bijection over `ALL` — a name maps to exactly one kind, so a
/// stored kind can never be confused with another after a variant is removed.
/// (b) Only the SETTINGS surface re-summons a value-pick child on ACCEPT; every
/// other summoning surface (the Command palette, a direct summon) lands in the
/// buffer — the one gate the ship-blocker fix turns on.
#[test]
fn breadcrumb_kinds_are_value_based_never_positional() {
    use std::collections::HashSet;
    let mut names: HashSet<&'static str> = HashSet::new();
    for k in OverlayKind::ALL {
        assert!(
            names.insert(k.as_str()),
            "{k:?}: overlay names must be a bijection — {:?} is a duplicate",
            k.as_str()
        );
    }
    assert_eq!(
        names.len(),
        OverlayKind::ALL.len(),
        "every kind has a distinct name"
    );
    // Exactly THREE kinds are SUSTAINED workspaces — the shared workspace's whole
    // declared scope: Settings, Version History, and the external-change
    // conflict. Everything else is a brief contextual
    // overlay, and a value-pick child returns to its parent exactly when that
    // parent is one of these three.
    for k in OverlayKind::ALL {
        assert_eq!(
            k.sustained(),
            matches!(
                k,
                OverlayKind::Settings | OverlayKind::History | OverlayKind::Conflict
            ),
            "{k:?}: the sustained-workspace roster is Settings + History + Conflict"
        );
    }
}

/// The calm, per-context empty-state COPY (the "nice text, ready" pass): each
/// context reads a warm, non-error line — the Go-to Recent lens especially
/// invites rather than reports, and a refinement lens with no members reads the
/// calm catch-all "nothing here".
#[test]
fn empty_state_copy_is_calm_and_context_aware() {
    // The refined per-kind corpus lines.
    assert_eq!(
        OverlayKind::Browse.empty_corpus_message(),
        "this folder is empty"
    );
    assert_eq!(
        OverlayKind::History.empty_corpus_message(),
        "no history yet"
    );
    assert_eq!(OverlayKind::Spell.empty_corpus_message(), "no suggestions");
    // Jump-to-heading + recent-projects are LENS empty-states now (the folds):
    assert_eq!(
        OverlayKind::Goto.empty_lens_message("headings"),
        Some("no headings yet")
    );
    assert_eq!(
        OverlayKind::Project.empty_lens_message("recent"),
        Some("no recent projects yet")
    );

    // The lens-scoped lines: Go-to Recent is the warm invitation; every other
    // refinement lens with no members reads the catch-all; `All` opts out (None).
    assert_eq!(
        OverlayKind::Goto.empty_lens_message("recent"),
        Some("no recent files yet"),
    );
    assert_eq!(
        OverlayKind::Goto.empty_lens_message("folder"),
        Some("nothing here")
    );
    assert_eq!(OverlayKind::Goto.empty_lens_message("all"), None);
}

/// A FRESH Go-to Recent lens (the recently-opened MRU is empty, nothing opened
/// yet) reads the calm "no recent files yet" line via `empty_message` — the
/// context that matters most this pass. A query still overrides with "no matches".
#[test]
fn goto_recent_empty_lens_reads_the_warm_invitation() {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![], // no recently-opened files → the Recent lens has no members
    );
    ov.set_facet_lens(1); // strip index 1 == Recent
    assert_eq!(ov.active_facet_id(), Some("recent"));
    assert!(ov.items.is_empty(), "a fresh Recent lens lists nothing");
    assert_eq!(ov.empty_notice().as_deref(), Some("no recent files yet"),);
    // A query on the empty Recent lens still reads the universal "no matches".
    ov.push('z');
    assert_eq!(ov.empty_notice().as_deref(), Some("no matches"));
}
