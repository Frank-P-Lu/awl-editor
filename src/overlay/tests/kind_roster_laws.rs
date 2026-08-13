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

    // Every typed destination lens names the thing that is absent; `All` opts out.
    assert_eq!(
        OverlayKind::Goto.empty_lens_message("recent"),
        Some("no recent destinations"),
    );
    assert_eq!(
        OverlayKind::Goto.empty_lens_message("folders"),
        Some("no folders here")
    );
    assert_eq!(OverlayKind::Goto.empty_lens_message("all"), None);
}

/// A FRESH Go-to Recent lens (both MRUs empty) reads the calm
/// "no recent destinations" line via `empty_message` — the
/// context that matters most this pass. A query still overrides with "no matches".
#[test]
fn goto_recent_empty_lens_reads_the_warm_invitation() {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![], // no recently-opened files → the Recent lens has no members
    );
    ov.focus_facet_id("recent");
    assert_eq!(ov.active_facet_id(), Some("recent"));
    assert!(ov.items.is_empty(), "a fresh Recent lens lists nothing");
    assert_eq!(ov.empty_notice().as_deref(), Some("no recent destinations"),);
    // A query on the empty Recent lens still reads the universal "no matches".
    ov.push('z');
    assert_eq!(ov.empty_notice().as_deref(), Some("no matches"));
}

/// THE CRISP BACKDROP IS EXACTLY THE LIVE-DOCUMENT AUDITION — an IFF, not a
/// subset. A card frosts what it covers; the exemption is earned by one property
/// and one only, that moving the highlight repaints the page BEHIND the card
/// ([`OverlayKind::previews_live_document`]), because frost would then blur the
/// only thing the row is showing.
///
/// ⚠️ **A SUBSET LAW WAS TOO WEAK, AND ITS OWN DOC SAID SO.** The predicate's
/// documentation earned the exemption by previewing live document state, while
/// the law asserted only `crisp ⊆ ValuePick`. `ValuePick` is a much larger set —
/// three of its members preview INSIDE THEIR OWN ROWS (the date formats render
/// today's date; the dictionary and CJK pickers pre-select the live value),
/// change nothing behind the card, and so want the frost. So a new picker could
/// be added to `actions::overlay_nav::preview_overlay`, audition the live
/// document, inherit frost, blur its own preview — and pass every law in the
/// tree, because the audition owner's match ended in a wildcard and no law had
/// the audition as its subject at all.
///
/// The set is asked BOTH ways here, and the two floors below are what keep the
/// IFF from being satisfiable by collapsing its own subject:
/// * neither side may go empty (an empty crisp set asserts nothing; an
///   all-crisp roster means frost has stopped existing);
/// * the value-pickers must STILL SPLIT — if every value-pick were crisp, this
///   predicate would be a synonym for the disposition and the row-previewing
///   pickers would have silently lost their frost.
///
/// Enrolment is [`OverlayKind::ALL`], so a new kind is swept the moment it
/// exists rather than when someone remembers to list it.
#[test]
fn crisp_backdrop_is_exactly_the_live_document_audition() {
    let crisp: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.keeps_backdrop_crisp())
        .collect();
    let frosted: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| !k.keeps_backdrop_crisp())
        .collect();
    let auditions: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.previews_live_document())
        .collect();
    let value_picks: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.accept_disposition() == AcceptDisposition::ValuePick)
        .collect();
    // PRESENCE FLOORS, both ways. A subset law is satisfiable by deleting its own
    // subject — an empty crisp set passes it vacuously, and an all-crisp roster
    // would mean frost had stopped existing. Neither is a state this product is
    // ever in, so both are failures here rather than silent successes.
    assert!(
        !crisp.is_empty(),
        "the crisp-backdrop set went EMPTY — the theme picker cannot preview a \
         world it has frosted, so this law would be asserting nothing"
    );
    assert!(
        !frosted.is_empty(),
        "EVERY kind kept the backdrop crisp — frost has no subject left, and the \
         summoned-surface rule DESIGN.md §5 states is gone: crisp = {crisp:?}"
    );
    // THE DISTINCTION FLOOR. The two predicates are allowed to be equal to each
    // other; neither is allowed to collapse into the accept disposition, because
    // then "previews the live document" would mean "picks a value" and the three
    // pickers that preview inside their own rows would be crisp by accident.
    assert!(
        value_picks.len() > crisp.len(),
        "every value-picker is crisp, so the audition predicate has collapsed into \
         the accept disposition — the pickers that preview INSIDE THEIR OWN ROWS \
         (nothing behind the card moves, so frost costs them nothing) have lost it. \
         value-pickers = {value_picks:?}, crisp = {crisp:?}"
    );

    // ---- THE IFF, both directions, named per kind ---------------------------
    for k in OverlayKind::ALL {
        assert_eq!(
            k.keeps_backdrop_crisp(),
            k.previews_live_document(),
            "{k:?} keeps_backdrop_crisp={} but previews_live_document={} — the frost \
             exemption and the live audition are ONE decision: a kind that repaints \
             the page behind its card must not frost it, and a kind that repaints \
             nothing has no exemption to claim. crisp = {crisp:?}, auditions = \
             {auditions:?}",
            k.keeps_backdrop_crisp(),
            k.previews_live_document()
        );
    }

    // The NECESSARY (never sufficient) condition the subset law used to be: an
    // audition commits a value, so it pops back to its summoning surface.
    for k in &crisp {
        assert_eq!(
            k.accept_disposition(),
            AcceptDisposition::ValuePick,
            "{k:?} keeps its backdrop crisp but is a {:?}, not a ValuePick: only a \
             picker auditioning a value into the live document earns the exemption \
             from frost. Crisp set = {crisp:?}",
            k.accept_disposition()
        );
    }
}

/// **THE AUDITION PREDICATE IS GRADED AGAINST THE AUDITION ITSELF**, not against
/// the crisp-backdrop list next to it. Two hand-written membership lists asserted
/// equal is a real drift guard and still not a law about the product: both could
/// be wrong together, and the whole defect this pair replaces was a *third*
/// spelling drifting away from a *fourth*.
///
/// So this arm calls the real [`crate::actions::preview_overlay`] once per kind in
/// [`OverlayKind::ALL`] and asks whether the running editor MOVED — the active
/// world or the caret look, the only two things an audition can touch today.
/// Exactly the kinds that answer [`OverlayKind::previews_live_document`] may move
/// it. A kind added to the predicate without an arm fails here (it declares an
/// audition it never performs); an arm reached without the declaration fails here
/// too (the gate at the top of `preview_overlay` is what makes that impossible).
///
/// The globals are snapshotted and restored through the same doors the test guard
/// uses, so an ambient caret pin or an `auto` entry state comes back as itself
/// rather than as whatever this sweep happened to leave.
#[test]
fn only_the_declared_auditions_move_the_live_editor() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let entry = crate::testlock::misc::pins();

    // A card of `k` as the product builds it, so the audition has something real
    // to apply. The wildcard is the GENERIC card: a future kind that declares an
    // audition and is built here generically moves nothing, which is exactly the
    // failure this law should report rather than skip.
    fn card(k: OverlayKind) -> OverlayState {
        match k {
            OverlayKind::Theme => {
                let names: Vec<String> =
                    crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
                OverlayState::new_theme(names, crate::theme::active_index())
            }
            OverlayKind::Caret => OverlayState::new_caret(crate::caret::mode()),
            _ => OverlayState::new(k, vec!["alpha.md".into(), "beta.md".into()], vec![], vec![]),
        }
    }

    let live = || (crate::theme::active_index(), crate::caret::mode());
    let mut moved: Vec<OverlayKind> = Vec::new();
    for k in OverlayKind::ALL {
        // Enter every arm from the same place, so "did it move" is a question
        // about the audition rather than about iteration order.
        crate::theme::set_active(0);
        crate::caret::set_mode(crate::caret::CaretMode::ALL[0]);
        let before = live();
        let mut ov = card(k);
        // The highlight MOVES: a card that opens on the live value and never
        // leaves it would report "nothing changed" for a working audition.
        ov.move_sel(1);
        crate::actions::preview_overlay(&ov);
        if live() != before {
            moved.push(k);
        }
    }
    crate::theme::set_active(0);
    crate::testlock::misc::restore(&entry);

    let declared: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.previews_live_document())
        .collect();
    // PRESENCE FLOOR: an equality between two empty sets is the vacuous pass this
    // law is most likely to die of — a `move_sel(1)` that stopped moving, or a
    // roster whose auditions were all retired, reads as agreement.
    assert!(
        !moved.is_empty(),
        "no kind moved the live editor at all, so this law compared two empty sets \
         — the audition fixture (a card built per kind, its highlight stepped once) \
         has stopped exercising anything. declared = {declared:?}"
    );
    assert_eq!(
        moved, declared,
        "the kinds whose preview actually moved the live editor are {moved:?}, but \
         the roster declares {declared:?}. A kind in `declared` and not in `moved` \
         claims an audition it never performs — and takes a crisp backdrop for it. \
         A kind in `moved` and not in `declared` auditions the live document from \
         behind a FROSTED card, blurring the only thing its rows are showing."
    );
}
