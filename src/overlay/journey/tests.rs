//! THE LIFECYCLE LAWS. The transition table is swept whole — every state, every
//! event, no wildcard — and then the properties that make it a lifecycle rather
//! than a lookup: a cancel always has an outcome, only a descend suspends, a
//! back off the detail stage never leaves the workspace, a resumed parent lands
//! on the exact row it left, and a cancel reverts what the card was
//! auditioning.

use super::*;
use crate::caret::CaretMode;

// ── FIXTURES ──────────────────────────────────────────────────────────────

fn card(kind: OverlayKind, rows: &[&str]) -> OverlayState {
    OverlayState::new(
        kind,
        rows.iter().map(|r| r.to_string()).collect(),
        vec![],
        vec![],
    )
}

const SETTINGS_ROWS: &[&str] = &[
    "Caret style",
    "Page mode",
    "Theme",
    "Typewriter",
    "Default folder",
];

/// A rebuild hook: hands back a fresh surface of the requested kind, exactly as
/// `ActionCtx::make_overlay` does live and headlessly.
fn rebuilder() -> impl FnMut(OverlayKind) -> Option<OverlayState> {
    |kind| match kind {
        OverlayKind::Settings => Some(card(OverlayKind::Settings, SETTINGS_ROWS)),
        OverlayKind::Command => Some(card(OverlayKind::Command, &["Switch theme…", "Save"])),
        OverlayKind::History => Some(card(OverlayKind::History, &["yesterday", "an hour ago"])),
        OverlayKind::Theme => Some(card(OverlayKind::Theme, &["Tawny", "Gumtree"])),
        _ => None,
    }
}

/// A journey standing on a brief child over a parked `parent`, reached the way
/// production reaches it: open the parent, then descend.
fn suspended_under(parent: OverlayKind, child: OverlayKind) -> Journey {
    let mut journey = Journey::seeded(Some(card(parent, SETTINGS_ROWS)));
    journey.descend(card(child, &["Tawny", "Gumtree"]), Bind::Value);
    journey
}

// ── THE TABLE ─────────────────────────────────────────────────────────────

/// Render the whole table as text, so the law's failure message shows what
/// changed and the table can be read by a human in one place.
fn rendered_table() -> String {
    let mut out = String::new();
    let mut row = |name: &str, state: State| {
        out.push_str(&format!("{name:<26}"));
        for &event in Event::ALL {
            out.push_str(&format!("{:<9}", format!("{:?}", landing_of(state, event))));
        }
        while out.ends_with(' ') {
            out.pop();
        }
        out.push('\n');
    };
    row("Editor", State::Editor);
    for &surface in Surface::ALL {
        for &beneath in Beneath::ALL {
            let name = format!("{surface:?}/{beneath:?}");
            row(&name, State::Summoned { surface, beneath });
        }
    }
    out
}

/// THE TRANSITION TABLE, pinned whole. Ten states × eight events; a changed
/// cell shows up as a diff of the rendered matrix rather than as a mystery
/// failure three layers away.
///
/// Column order: Cancel · AcceptNavigate · AcceptValue · AcceptStayOpen ·
/// Toggle · Descend · ToggleDetail · Dismiss.
#[test]
fn the_transition_table_is_exactly_this() {
    let expect = "\
Editor                    Stay     Stay     Stay     Stay     Stay     Stay     Stay     Editor
Contextual/Editor         Editor   Editor   Editor   Stay     Editor   Suspend  Stay     Editor
Contextual/Workspace      Resume   Editor   Resume   Stay     Resume   Suspend  Stay     Editor
Contextual/Launcher       Resume   Editor   Editor   Stay     Editor   Suspend  Stay     Editor
Workspace/Editor          Editor   Editor   Primary  Stay     Stay     Suspend  Detail   Editor
Workspace/Workspace       Resume   Editor   Resume   Stay     Stay     Suspend  Detail   Editor
Workspace/Launcher        Resume   Editor   Editor   Stay     Stay     Suspend  Detail   Editor
WorkspaceDetail/Editor    Editor   Editor   Primary  Stay     Stay     Suspend  Primary  Editor
WorkspaceDetail/Workspace Resume   Editor   Resume   Stay     Stay     Suspend  Primary  Editor
WorkspaceDetail/Launcher  Resume   Editor   Editor   Stay     Stay     Suspend  Primary  Editor
";
    assert_eq!(rendered_table(), expect, "the transition table changed");
}

/// NON-VACUITY FLOOR for every sweep in this file: the state roster is the
/// editor plus the full `Surface × Beneath` cross-product, and the event roster
/// is the whole vocabulary. A roster that quietly shrank would make every sweep
/// below pass over a fraction of the space — the failure mode item 172's field
/// parser hit when it silently reported 83 of 105 fields.
#[test]
fn the_rosters_are_the_size_the_sweeps_assume() {
    assert_eq!(Surface::ALL.len(), 3, "surfaces");
    assert_eq!(Beneath::ALL.len(), 3, "parent policies");
    assert_eq!(Event::ALL.len(), 8, "events");
    assert_eq!(State::all().len(), 1 + 3 * 3, "states");
    assert_eq!(
        State::all().len() * Event::ALL.len(),
        80,
        "the table has 80 cells"
    );
    assert_eq!(
        rendered_table().lines().count(),
        State::all().len(),
        "the rendered table must have one line per state"
    );
    // Every landing must be REACHABLE, or a variant is decoration.
    let mut seen = std::collections::BTreeSet::new();
    for state in State::all() {
        for &event in Event::ALL {
            seen.insert(format!("{:?}", landing_of(state, event)));
        }
    }
    assert_eq!(
        seen.len(),
        6,
        "every Landing variant must appear somewhere in the table: {seen:?}"
    );
}

/// EVERY ESC/BACK HAS AN OUTCOME. While anything is summoned, a cancel must
/// MOVE the journey — the item's own done-condition. A `Stay` here would be the
/// old exceptional-branch shape sneaking back: a surface where Esc does
/// nothing, and therefore a surface whose Esc some caller has to special-case.
#[test]
fn every_cancel_from_a_summoned_state_moves_the_journey() {
    for state in State::all() {
        let landing = landing_of(state, Event::Cancel);
        if state == State::Editor {
            assert_eq!(
                landing,
                Landing::Stay,
                "the editor's Esc is not the journey's business"
            );
            continue;
        }
        assert_ne!(
            landing,
            Landing::Stay,
            "{state:?}: Esc must have an outcome, not a special case"
        );
        assert_ne!(
            landing,
            Landing::Suspend,
            "{state:?}: a cancel can never open a child"
        );
    }
}

/// ONLY A DESCEND SUSPENDS. `Journey::advance` deliberately performs nothing on
/// `Landing::Suspend`, because only [`Journey::descend`] carries the child — so
/// a `Suspend` in any other column would be a silently-dropped transition.
#[test]
fn the_only_suspending_column_is_descend() {
    for state in State::all() {
        for &event in Event::ALL {
            let suspends = landing_of(state, event) == Landing::Suspend;
            assert_eq!(
                suspends,
                event == Event::Descend && state != State::Editor,
                "{state:?} × {event:?}: Suspend belongs to the Descend column alone"
            );
        }
    }
}

/// A RESUME NEEDS SOMETHING PARKED. `Landing::Resume` may only appear where the
/// state says a parent is beneath — otherwise `perform_resume` would fall to
/// the editor and the transition would be a lie.
#[test]
fn a_resume_never_lands_where_nothing_is_parked() {
    for state in State::all() {
        for &event in Event::ALL {
            if landing_of(state, event) != Landing::Resume {
                continue;
            }
            let State::Summoned { beneath, .. } = state else {
                panic!("{state:?} × {event:?}: the editor cannot resume");
            };
            assert_ne!(
                beneath,
                Beneath::Editor,
                "{state:?} × {event:?}: nothing is parked to resume"
            );
        }
    }
}

/// THE FOCUS STAGE BELONGS TO A WORKSPACE. `Primary`/`Detail` may only be
/// reached from a sustained surface — a brief overlay has no detail stage, so a
/// focus landing there would write a bit nothing reads.
#[test]
fn only_a_sustained_surface_reaches_a_focus_stage() {
    for state in State::all() {
        for &event in Event::ALL {
            if !matches!(landing_of(state, event), Landing::Primary | Landing::Detail) {
                continue;
            }
            assert!(
                matches!(
                    state,
                    State::Summoned {
                        surface: Surface::Workspace | Surface::WorkspaceDetail,
                        ..
                    }
                ),
                "{state:?} × {event:?}: only a workspace has a focus stage"
            );
        }
    }
}

/// WIDE AND NARROW AGREE, AND SO DO THE TWO REGIONS. A workspace's detail stage
/// sits BESIDE the list when there is room and is PUSHED OVER it when there is
/// not; the lifecycle achieves that by having no width input at all.
///
/// **ONE ESC ALWAYS LEAVES** (user decision 2026-08-02). The consequence stated
/// over the whole `Beneath` axis: a cancel from the DETAIL stage lands EXACTLY
/// where a cancel from the PRIMARY list lands. This is the invariance law, not a
/// restatement of the arms — it compares the two rows of the table against each
/// other, so a future edit that changes one and forgets the other fails here
/// rather than shipping an Esc that means two things depending on where focus
/// sits. `ToggleDetail` is what moves between the regions, and it still does.
#[test]
fn one_esc_leaves_from_either_region_of_a_workspace() {
    for &beneath in Beneath::ALL {
        let primary = State::Summoned {
            surface: Surface::Workspace,
            beneath,
        };
        let detail = State::Summoned {
            surface: Surface::WorkspaceDetail,
            beneath,
        };
        assert_eq!(
            landing_of(detail, Event::Cancel),
            landing_of(primary, Event::Cancel),
            "over {beneath:?}: Esc must land in the same place from the detail stage as from \
             the primary list — one Esc always leaves, and a reader spends their time in the \
             detail stage"
        );
        assert_ne!(
            landing_of(detail, Event::Cancel),
            Landing::Primary,
            "over {beneath:?}: Esc from the detail stage must NOT be a Back — Tab is"
        );
        assert_eq!(
            landing_of(detail, Event::ToggleDetail),
            Landing::Primary,
            "over {beneath:?}: Tab is what returns to the primary list, and the footer says so"
        );
    }
    // NON-VACUITY: the primary list's own cancel really does leave from the
    // plainest case, so the invariance above is "both leave", not "both stay".
    assert_eq!(
        landing_of(
            State::Summoned {
                surface: Surface::Workspace,
                beneath: Beneath::Editor
            },
            Event::Cancel
        ),
        Landing::Editor,
    );
    // …and a CHILD AUDITION over a parked workspace is the one rung that keeps
    // Esc-returns-to-parent, which is what makes "one Esc always leaves" a
    // statement about the workspace rather than about every summoned surface.
    assert_eq!(
        landing_of(
            State::Summoned {
                surface: Surface::Contextual,
                beneath: Beneath::Workspace
            },
            Event::Cancel
        ),
        Landing::Resume,
    );
}

// ── DRIVING A REAL JOURNEY ────────────────────────────────────────────────

/// FOCUS TRANSFER, driven. Tab moves into the detail stage, Tab comes back, and
/// the workspace is still up with its selection intact — then ONE Esc leaves,
/// from either region.
#[test]
fn focus_transfers_into_the_detail_stage_and_back_without_closing() {
    let mut journey = Journey::seeded(Some(card(OverlayKind::History, &["yesterday", "today"])));
    journey.card_mut().unwrap().move_sel(1);
    assert_eq!(journey.state().rung(), Rung::Sustained);

    assert_eq!(journey.toggle_detail(), Landing::Detail);
    assert!(
        journey.card().unwrap().detail_focus,
        "focus moved to the detail"
    );
    assert_eq!(
        journey.state().rung(),
        Rung::Sustained,
        "still the workspace"
    );

    // TAB is the Back — the footer's advertised affordance, and now the only one.
    assert_eq!(journey.toggle_detail(), Landing::Primary);
    assert!(!journey.card().unwrap().detail_focus, "focus came back");
    assert_eq!(
        journey.card().unwrap().selected,
        1,
        "the primary list kept its row across the round trip"
    );

    // ONE ESC LEAVES from the primary list…
    assert_eq!(journey.cancel(&mut rebuilder()), Landing::Editor);
    assert!(journey.card().is_none());

    // …and ONE Esc leaves from the DETAIL stage too, which is the whole of the
    // 2026-08-02 decision: no second press, no "it depends where focus is".
    let mut journey = Journey::seeded(Some(card(OverlayKind::History, &["yesterday", "today"])));
    assert_eq!(journey.toggle_detail(), Landing::Detail);
    assert_eq!(journey.cancel(&mut rebuilder()), Landing::Editor);
    assert!(
        journey.card().is_none(),
        "Esc from the comparison must leave outright, not unwind one rung"
    );
}

/// POSITION RESTORATION — the defect the breadcrumb could not express. Move
/// down the Settings list, type a filter, descend into a child, come back: the
/// SAME ROW is highlighted, on a surface rebuilt fresh (so its value cells show
/// what the child just committed).
#[test]
fn a_child_returns_the_workspace_to_the_exact_row_it_left() {
    let mut journey = Journey::seeded(Some(card(OverlayKind::Settings, SETTINGS_ROWS)));
    for c in "type".chars() {
        journey.card_mut().unwrap().push(c);
    }
    let left_on = journey
        .card()
        .unwrap()
        .selected_value()
        .unwrap()
        .to_string();
    assert_eq!(
        left_on, "Typewriter",
        "the filter landed on a non-first row"
    );
    let corpus = journey.card().unwrap().selected_corpus_index();

    assert_eq!(
        journey.descend(card(OverlayKind::Theme, &["Tawny"]), Bind::Value),
        Landing::Suspend
    );
    assert_eq!(journey.card().unwrap().kind, OverlayKind::Theme);
    assert_eq!(journey.parked_kind(), Some(OverlayKind::Settings));
    assert_eq!(
        journey.parked_resume().and_then(|r| r.selected_corpus()),
        corpus,
        "the parked parent recorded the CORPUS row, not the filtered position"
    );
    assert_eq!(journey.parked_resume().map(|r| r.query()), Some("type"));

    assert_eq!(
        journey.accept(AcceptDisposition::ValuePick, &mut rebuilder()),
        Landing::Resume
    );
    let back = journey.card().expect("the workspace resumed");
    assert_eq!(back.kind, OverlayKind::Settings);
    assert_eq!(
        back.selected_value(),
        Some(left_on.as_str()),
        "resumed on the row it left, not on row 0"
    );
    assert_eq!(back.query.text(), "type", "and with the filter it left");
    assert_eq!(
        journey.parked_kind(),
        None,
        "single-level: the resumed parent parks nothing itself"
    );
}

/// The SAME restoration on a CANCEL, not just a commit — the two paths share
/// one owner, so a fix to either is a fix to both.
#[test]
fn a_cancelled_child_also_returns_to_the_exact_row() {
    let mut journey = Journey::seeded(Some(card(OverlayKind::Settings, SETTINGS_ROWS)));
    journey.card_mut().unwrap().move_sel(3);
    let left_on = journey
        .card()
        .unwrap()
        .selected_value()
        .unwrap()
        .to_string();
    journey.descend(card(OverlayKind::Theme, &["Tawny"]), Bind::Value);
    assert_eq!(journey.cancel(&mut rebuilder()), Landing::Resume);
    assert_eq!(
        journey.card().unwrap().selected_value(),
        Some(left_on.as_str())
    );
}

/// A LAUNCHER PARENT: cancel returns to it, commit completes the errand in the
/// document. Both halves in one law, because the pair is the rule.
#[test]
fn a_launcher_parent_returns_on_cancel_and_completes_on_commit() {
    let mut journey = suspended_under(OverlayKind::Command, OverlayKind::Theme);
    assert_eq!(journey.cancel(&mut rebuilder()), Landing::Resume);
    assert_eq!(journey.card().map(|o| o.kind), Some(OverlayKind::Command));

    let mut journey = suspended_under(OverlayKind::Command, OverlayKind::Theme);
    assert_eq!(
        journey.accept(AcceptDisposition::ValuePick, &mut rebuilder()),
        Landing::Editor,
        "committing a palette-launched value lands in the document"
    );
    assert!(journey.card().is_none());
}

/// THE BIND SURVIVES A RELEVEL. A folder navigator opened from a Settings PATH
/// row keeps writing THAT config key — and keeps its parked parent — through
/// any number of descends and ascends, because `relevel` replaces only the card.
#[test]
fn the_config_key_and_the_parked_parent_survive_every_level_change() {
    let mut journey = Journey::seeded(Some(card(OverlayKind::Settings, SETTINGS_ROWS)));
    journey.card_mut().unwrap().move_sel(4);
    journey.descend(
        card(OverlayKind::Project, &[".", "sub"]),
        Bind::Path {
            key: "default_folder".to_string(),
        },
    );
    for level in 0..5 {
        journey.relevel(card(OverlayKind::Project, &[".", "deeper"]));
        assert_eq!(
            journey.path_key(),
            Some("default_folder"),
            "the config key survived level {level}"
        );
        assert_eq!(
            journey.parked_kind(),
            Some(OverlayKind::Settings),
            "the parked parent survived level {level}"
        );
    }
    assert_eq!(
        journey.accept(AcceptDisposition::ValuePick, &mut rebuilder()),
        Landing::Resume
    );
    assert_eq!(
        journey.card().unwrap().selected_value(),
        Some("Default folder"),
        "and the pick lands back on the row that asked for it"
    );
}

/// DEPTH STAYS ONE. Descending from a child replaces what was parked — the
/// pre-existing single-level rule, now a property of the type rather than of
/// every caller overwriting one field.
#[test]
fn descending_from_a_child_replaces_the_parked_parent_rather_than_stacking() {
    let mut journey = suspended_under(OverlayKind::Settings, OverlayKind::Theme);
    assert_eq!(journey.parked_kind(), Some(OverlayKind::Settings));
    journey.descend(card(OverlayKind::Caret, &["Block"]), Bind::Value);
    assert_eq!(
        journey.parked_kind(),
        Some(OverlayKind::Theme),
        "the new parent is the surface you descended FROM"
    );
    // One resume empties the stack: there is no grandparent to find.
    journey.cancel(&mut rebuilder());
    assert_eq!(journey.parked_kind(), None);
}

// ── THE AUDITION ──────────────────────────────────────────────────────────

/// CANCEL REVERTS — at EVERY rung. Directly-summoned and suspended cards revert
/// identically because they share the one owner, which is the whole point of
/// moving the three loose `original_*` fields into a closed payload.
#[test]
fn a_cancel_reverts_the_live_audition_at_every_rung() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();

    for parent in [
        None,
        Some(OverlayKind::Settings),
        Some(OverlayKind::Command),
    ] {
        crate::theme::set_active(0);
        let opening = crate::theme::active_index();
        let names: Vec<String> = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
        let mut journey = Journey::seeded(Some(OverlayState::new_theme(names, opening)));
        if let Some(parent) = parent {
            let child = journey.card().cloned().unwrap();
            journey = Journey::seeded(Some(card(parent, SETTINGS_ROWS)));
            journey.descend(child, Bind::Value);
        }
        // Audition another world the way a selection move does.
        journey.card_mut().unwrap().move_sel(1);
        crate::actions::preview_overlay(journey.card().unwrap());
        assert_ne!(
            crate::theme::active_index(),
            opening,
            "{parent:?}: the audition must actually change the world first"
        );
        journey.cancel(&mut rebuilder());
        assert_eq!(
            crate::theme::active_index(),
            opening,
            "{parent:?}: a cancel restores the world the picker opened on"
        );
    }
    crate::theme::set_active(0);
}

/// The CARET audition's two shapes — an explicit pin and auto's resolution —
/// revert differently, and the payload is what remembers which.
#[test]
fn a_cancelled_caret_audition_restores_a_pin_or_clears_the_override() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();

    // An explicit pin comes back as a pin.
    crate::caret::set_mode(CaretMode::Ibeam);
    let mut journey = Journey::seeded(Some(OverlayState::new_caret(crate::caret::mode())));
    journey.card_mut().unwrap().move_sel(1);
    crate::actions::preview_overlay(journey.card().unwrap());
    journey.cancel(&mut rebuilder());
    assert_eq!(crate::caret::mode(), CaretMode::Ibeam);
    assert!(!crate::caret::is_auto(), "a pin stays a pin");

    // Auto's resolution comes back as AUTO, not as a pin of the same look.
    crate::caret::clear_override();
    let resolved = crate::caret::mode();
    let mut journey = Journey::seeded(Some(OverlayState::new_caret(resolved)));
    journey.card_mut().unwrap().move_sel(1);
    crate::actions::preview_overlay(journey.card().unwrap());
    journey.cancel(&mut rebuilder());
    assert_eq!(crate::caret::mode(), resolved);
    assert!(
        crate::caret::is_auto(),
        "auto must come back as auto, never pinned to the look it happened to resolve"
    );
    crate::caret::clear_override();
}

/// A COMMIT KEEPS the audition. The counterpart to the revert law: if `advance`
/// ever reverted on more than the `Cancel` event, every value pick would
/// silently undo itself.
#[test]
fn a_commit_keeps_the_audition() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::theme::set_active(0);
    let names: Vec<String> = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
    let mut journey = Journey::seeded(Some(OverlayState::new_theme(names, 0)));
    journey.card_mut().unwrap().move_sel(1);
    crate::actions::preview_overlay(journey.card().unwrap());
    let kept = crate::theme::active_index();
    journey.accept(AcceptDisposition::ValuePick, &mut rebuilder());
    assert_eq!(
        crate::theme::active_index(),
        kept,
        "the commit kept the world"
    );
    crate::theme::set_active(0);
}

// ── THE STRUCTURAL LAW ────────────────────────────────────────────────────

/// Count whitespace-stripped `needle` occurrences per file under `src/`.
fn scan(needle: &str) -> std::collections::BTreeMap<String, usize> {
    fn walk(
        base: &std::path::Path,
        dir: &std::path::Path,
        needle: &str,
        out: &mut std::collections::BTreeMap<String, usize>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, needle, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let collapsed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            let n = collapsed.matches(needle).count();
            if n > 0 {
                let rel = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                *out.entry(rel).or_insert(0) += n;
            }
        }
    }
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Default::default();
    walk(&root, &root, needle, &mut out);
    out
}

/// THE STRUCTURAL LAW — the one a reintroduced parallel flag path must break.
///
/// `OverlayState::detail_focus` is where the workspace's focus stage LIVES, but
/// what it MEANS (Tab moves in, Esc comes back, and neither closes the
/// workspace) belongs to the lifecycle. So the field may be READ anywhere and
/// WRITTEN only inside `overlay/journey/`. A second writer — an exceptional
/// `Esc` arm setting it directly, the shape this item exists to retire — fails
/// here by name, before it can grow its own rules.
///
/// The needle is assembled at runtime so this file's own prose cannot match it,
/// and the count is asserted rather than merely located: a law that only checks
/// "is it in the right file" stays green while a second copy lives beside it.
///
/// It matches a FIELD write (`.detail_focus =`) rather than the bare name.
/// Item 114 gave the render projection of this same fact its own field
/// (`ViewState::overlay_detail_focus`, assigned at the two mirror sites every
/// `ViewState` field is), and a bare-name needle read those projections as
/// second writers of the card's bit. Requiring the receiver dot keeps every
/// shape this law exists to catch — `ov.detail_focus = true` in an exceptional
/// `Esc` arm is still a failure by name — while a distinct field whose name
/// merely ENDS in these characters is not one.
#[test]
fn the_workspace_focus_stage_is_written_only_by_the_lifecycle() {
    let writes = scan(&[".detail_focus", "="].concat());
    // Comparisons (`==`) are reads, not writes — subtract them so a reader is
    // never mistaken for a writer.
    let compares = scan(&[".detail_focus", "=="].concat());
    let mut owned = 0usize;
    let mut leaked: Vec<(String, usize)> = Vec::new();
    for (file, n) in &writes {
        let n = n - compares.get(file).copied().unwrap_or(0);
        if n == 0 {
            continue;
        }
        if file.starts_with("overlay/journey/") {
            owned += n;
        } else {
            leaked.push((file.clone(), n));
        }
    }
    assert!(
        leaked.is_empty(),
        "the workspace focus stage is written outside the lifecycle: {leaked:?}. \
         Route it through `Journey::advance` — Tab/Esc on the detail stage is a \
         transition, not a flag."
    );
    // NON-VACUITY FLOOR: the needle must actually be finding the real writes.
    // Without this the law passes just as happily against a typo.
    assert!(
        owned >= 3,
        "the scanner found only {owned} lifecycle writes of the focus stage — \
         the needle stopped matching, and this law is vacuous"
    );
}

/// The retired fields must STAY RETIRED as `OverlayState` fields. A `return_to`
/// breadcrumb, an `original_theme`/`original_caret` pair or a loose
/// `setting_path_key` growing back on the card would be a second, untyped copy
/// of what [`Parked`], [`Audition`] and [`Bind`] own — and the compiler cannot
/// object, because a new field is always legal.
///
/// Scoped to the card's DECLARATION (`overlay/state.rs`) rather than to every
/// mention of the name: `return_to` is also the sidecar's published field, and
/// the schema is a contract with every agent probe. Its VALUE now comes from
/// `Journey::parked()`, which is exactly the point.
#[test]
fn the_loose_lifecycle_fields_do_not_grow_back_on_the_card() {
    let declaration =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/overlay/state.rs");
    let text: String = std::fs::read_to_string(&declaration)
        .expect("the card's declaration")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    for retired in [
        "return_to",
        "original_theme",
        "original_caret",
        "original_caret_was_auto",
        "setting_path_key",
    ] {
        let needle = ["pub", retired, ":"].concat();
        assert!(
            !text.contains(&needle),
            "`{retired}` came back as an `OverlayState` field — the lifecycle owns \
             the parked parent, the revert payload and the child's write-back"
        );
    }
    // NON-VACUITY FLOOR: the scanner must be reading the real declaration with a
    // needle shape that actually matches, or the loop above is checking nothing.
    for present in ["pubaudition:", "pubdetail_focus:", "pubkind:"] {
        assert!(
            text.contains(present),
            "the scanner did not find `{present}` — it is reading the wrong file, \
             or the needle shape stopped matching, and this law is vacuous"
        );
    }
}
