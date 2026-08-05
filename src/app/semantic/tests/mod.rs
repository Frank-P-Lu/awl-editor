use super::*;
use crate::overlay::{OverlayKind, OverlayState};

/// The drawn-⇔-announced roster law for the passive surfaces, in its own
/// file because it renders real frames and reads their pixels back.
mod passive_roster;

fn hermetic() -> App {
    App::new_hermetic(None, PathBuf::from("/"), Config::empty())
}

fn calm_globals() {
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
    crate::streaks::set_open(false);
    crate::hud::set_held(false);
    crate::peek::set_open(false);
    crate::whichkey::set_force_shown(false);
    crate::menubar::set_menu_bar_on(false);
}

/// [`calm_globals`], but with a restore whose lifetime is the CALLER's: `menu_bar`'s
/// default is platform-dependent (`false` on macOS, `true` elsewhere), so the bare
/// `set_menu_bar_on(false)` above is a silent no-op on macOS and a real mutation on
/// Linux — invisible until `testlock::misc::leaked` audits `menu_bar`, at which point
/// every fixture that calls plain `calm_globals` and never restores it fails on Linux
/// alone. Snapshot BEFORE mutating, and hand the guard to the caller, who binds it
/// after their own `crate::testlock::serial()` guard so it drops first, while the lock
/// is still held (`TogglesRestore`'s restore path asserts that).
fn calm_globals_guarded() -> crate::testlock::misc::TogglesRestore {
    let restore = crate::testlock::misc::TogglesRestore::capture();
    calm_globals();
    restore
}

fn seeded_overlay(kind: OverlayKind) -> OverlayState {
    let corpus = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let mut overlay = OverlayState::new(kind, corpus.clone(), Vec::new(), Vec::new());
    if kind == OverlayKind::Context {
        // A context card carries an action per row, built together by
        // `context_menu::overlay`; a card seeded without them is not a card
        // the product can produce.
        overlay.context_actions = corpus.iter().map(|_| Some(Action::ForwardChar)).collect();
    }
    overlay
}

fn ids(snapshot: &SemanticSnapshot) -> Vec<String> {
    snapshot.nodes.iter().map(|node| node.id.clone()).collect()
}

#[test]
fn raw_markdown_snapshot_has_one_focus_and_grapheme_selection() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let mut app = hermetic();
    app.set_semantic_text_for_test("e\u{301} 👨‍👩‍👧‍👦 🇯🇵");
    let snapshot = app.semantic_snapshot();
    assert_eq!(snapshot.focus_id, DOCUMENT_ID);
    assert_eq!(snapshot.nodes.iter().filter(|node| node.focused).count(), 1);
    let text = snapshot
        .nodes
        .iter()
        .find(|node| crate::semantic::is_run_id(&node.id))
        .unwrap();
    assert_eq!(text.character_lengths.len(), 5);
    let document = snapshot
        .nodes
        .iter()
        .find(|node| node.id == DOCUMENT_ID)
        .unwrap();
    assert_eq!(document.selection.unwrap().focus, 5);
}

/// `semantic_snapshot()` builds a WHOLE snapshot: every line of the document
/// read out of the rope and segmented under UAX #29. That is the right shape
/// for a one-shot consumer and the wrong shape for a frame, so the call sites
/// are enumerated.
///
/// The live frame path is deliberately absent: `refresh_accessibility` drives
/// the RETAINED `SemanticProjection`, which re-reads only the lines an edit
/// touched, so `app/semantic/mod.rs` naming this function again would mean the
/// per-frame whole-document cost had come back.
#[test]
fn semantic_snapshot_has_no_ungated_frame_side_caller() {
    let mut found: Vec<String> = Vec::new();
    let mut stack = vec![PathBuf::from("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src is readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            // `tests.rs` and everything under a `tests/` directory is test
            // code by this tree's convention, and a test may build a snapshot
            // freely — the cost being rationed is per FRAME, not per test.
            let text = path.to_string_lossy().replace('\\', "/");
            if text.ends_with("/tests.rs") || text.contains("/tests/") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("source is utf-8");
            // Only production lines: a test may build a snapshot freely,
            // and prose naming the function is not a call.
            let production = source
                .split_once("#[cfg(test)]")
                .map_or(source.as_str(), |(before, _)| before);
            if production
                .lines()
                .any(|line| line.contains("semantic_snapshot()") && !line.trim().starts_with("//"))
            {
                found.push(text);
            }
        }
    }
    found.sort();
    let mut sanctioned = vec![
        // The live-App sidecar embeds the same snapshot.
        "src/app/capture_state.rs".to_string(),
        // `--bench-a11y` times the RETIRED whole-document path against the
        // retained one; measuring the old cost is the point of the mode.
        "src/app/semantic/bench.rs".to_string(),
        // `--semantic-json` prints it.
        "src/main/run/live_app.rs".to_string(),
    ];
    sanctioned.sort();
    assert_eq!(
        found, sanctioned,
        "a new caller of semantic_snapshot() must justify its per-frame cost",
    );
}

/// The card captions and figures must exist in exactly one place. If a caption
/// string reappears inside the renderer, the two descriptions have forked and
/// an assistive technology is reading a stale copy of the card.
#[test]
fn the_renderer_composes_no_card_text_of_its_own() {
    let source = std::fs::read_to_string("src/render/chrome/hud.rs").expect("hud.rs is readable");
    for needle in [
        "CURRENT STREAK",
        "WRITTEN TODAY",
        "PAST YEAR",
        "WORD COUNT",
        "THROUGH DOC",
        "LINE ENDINGS",
        "SAVED",
        "GPL-3.0",
        "Credits",
    ] {
        assert!(
            !source.contains(needle),
            "render/chrome/hud.rs still spells card text {needle:?}; \
             card content belongs to crate::card::content alone",
        );
    }
}

/// A screen reader must not be re-announced at 120 Hz because the caret is
/// gliding. The dedup in `frame::accessibility` can only suppress an update if
/// the snapshot really is equal, so the load-bearing claim is that the snapshot
/// carries NO animation phase at all: twenty real scheduling steps with a live
/// caret spring in flight must produce the identical tree.
#[test]
fn animation_only_frames_produce_an_identical_snapshot() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let clock = crate::clock::VirtualClock::new();
    let mut app = hermetic();
    app.set_clock(Box::new(clock.clone()));
    app.set_semantic_text_for_test("some prose with a caret that has somewhere to travel");
    // A real caret jump, so the spring/glide is genuinely mid-flight while the
    // frames below step.
    app.document.set_cursor(0);
    app.sync_view(true);
    let settled = app.semantic_snapshot();
    let sched = schedule::RecordingScheduler::new();
    for step in 0..20 {
        clock.advance_ms(8);
        sched.begin_step();
        app.step_scheduling(&sched);
        assert_eq!(
            app.semantic_snapshot(),
            settled,
            "scheduling step {step} moved the semantic tree with no input",
        );
    }
}

#[test]
fn document_ids_survive_edits() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let mut app = hermetic();
    let before = ids(&app.semantic_snapshot());
    app.document.insert_text("# hello");
    let after = ids(&app.semantic_snapshot());
    assert_eq!(before, after);
}

/// Filtering is the harder half of stable identity, and the one the edit test
/// above cannot see: a row's id must key off its CORPUS position, not its
/// filtered display position. Keyed by display position, typing one character
/// silently renames every surviving row and assistive focus lands on a
/// different command than the one it was on.
#[test]
fn row_ids_survive_filtering() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let mut app = hermetic();
    let mut overlay = OverlayState::new(
        OverlayKind::Command,
        vec![
            "alpha".to_string(),
            "beta".to_string(),
            "gamma".to_string(),
            "beta two".to_string(),
        ],
        Vec::new(),
        Vec::new(),
    );
    overlay.set_query_text("");
    app.workspace_state.install_overlay_for_test(overlay);

    // Keyed by NAME, not by position: an id set that is merely a subset of the
    // unfiltered one proves nothing (display-position ids are exactly that),
    // so the law asks where one particular row went.
    let named = |app: &App| -> Vec<(String, String)> {
        app.semantic_snapshot()
            .nodes
            .into_iter()
            .filter(|node| node.id.contains(".row."))
            .map(|node| (node.name, node.id))
            .collect()
    };
    let unfiltered = named(&app);
    assert_eq!(unfiltered.len(), 4);
    let gamma = unfiltered
        .iter()
        .find(|(name, _)| name == "gamma")
        .expect("gamma is a row")
        .1
        .clone();

    app.workspace_state
        .overlay_mut()
        .unwrap()
        .set_query_text("gamma");
    let filtered = named(&app);
    assert!(
        filtered.len() < unfiltered.len() && !filtered.is_empty(),
        "the query really filtered: {filtered:?}",
    );
    assert_eq!(
        filtered
            .iter()
            .find(|(name, _)| name == "gamma")
            .map(|(_, id)| id.as_str()),
        Some(gamma.as_str()),
        "filtering renamed a surviving row; identity must key on the corpus, \
         not on the filtered display position",
    );
}

/// The card fold is the one passive surface whose content only the render
/// pipeline holds, so it is folded from a value rather than fetched — which is
/// what makes this law possible at all without a GPU. Every card kind must
/// announce every line it draws, in order, and take no focus.
#[test]
fn a_summoned_card_announces_every_drawn_line_and_takes_no_focus() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let app = hermetic();
    let inputs = crate::card::content::CardInputs {
        doc: crate::card::figures::DocFigures {
            words: "9 words · 1 min".to_string(),
            ..crate::card::figures::DocFigures::default()
        },
        ..crate::card::content::CardInputs::default()
    };
    for kind in crate::card::content::CardKind::ALL {
        let content = crate::card::content::card(kind, &inputs);
        let mut nodes = app.semantic_snapshot().nodes;
        let before = nodes.len();
        app.fold_card(&mut nodes, Some(content.clone()));
        assert_eq!(
            nodes.len(),
            before + content.spans.len() + 1,
            "{kind:?}: one node per drawn line, plus the card itself",
        );
        let card = nodes
            .iter()
            .find(|node| node.id == kind.id())
            .unwrap_or_else(|| panic!("{kind:?} announced no card node"));
        assert_eq!(card.name, kind.title());
        assert_eq!(card.children.len(), content.spans.len());
        for (child, line) in card.children.iter().zip(content.lines()) {
            let node = nodes.iter().find(|node| node.id == *child).unwrap();
            assert_eq!(node.name, line, "{kind:?}: a drawn line was not announced");
        }
        assert_eq!(
            nodes.iter().filter(|node| node.focused).count(),
            1,
            "{kind:?}: a passive card moved the focus owner",
        );
    }
}

/// The no-wildcard surface roster: every overlay kind must produce a named,
/// non-empty surface. A kind added to `OverlayKind::ALL` and forgotten here
/// fails by name rather than silently announcing nothing.
#[test]
fn every_overlay_kind_produces_a_named_surface_with_stable_rows() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    for kind in OverlayKind::ALL {
        let mut app = hermetic();
        app.workspace_state
            .install_overlay_for_test(seeded_overlay(kind));
        let snapshot = app.semantic_snapshot();
        let dialog_id = format!("overlay.{}", kind.as_str());
        let dialog = snapshot
            .nodes
            .iter()
            .find(|node| node.id == dialog_id)
            .unwrap_or_else(|| panic!("{kind:?} produced no dialog node"));
        assert!(
            !dialog.name.is_empty(),
            "{kind:?} produced an unnamed surface",
        );
        assert_eq!(snapshot.nodes.iter().filter(|n| n.focused).count(), 1);
        let rows: Vec<&SemanticNode> = snapshot
            .nodes
            .iter()
            .filter(|node| node.id.starts_with(&format!("{dialog_id}.row.")))
            .collect();
        assert_eq!(rows.len(), 3, "{kind:?} lost rows");
        for row in rows {
            assert!(!row.actions.is_empty(), "{kind:?} row advertises nothing");
        }
    }
}

/// The ladder is the focus owner. Every rung must name exactly one focused
/// node, and that node must be the one `focus_id` points at — otherwise an
/// assistive technology is told about a node the tree does not mark.
#[test]
fn every_ladder_rung_names_exactly_one_focus_owner() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    for rung in workspace::Layer::ROSTER {
        let mut app = hermetic();
        app.set_semantic_text_for_test("some prose");
        match rung {
            workspace::Layer::Editor => {}
            workspace::Layer::Popover => app.workspace_state.summon_popover(true),
            workspace::Layer::Search => {
                app.workspace_state
                    .install_search_for_test(crate::search::SearchState::start(
                        0,
                        crate::search::Direction::Forward,
                    ));
            }
            workspace::Layer::Workspace => app
                .workspace_state
                .install_overlay_for_test(seeded_overlay(OverlayKind::Settings)),
            workspace::Layer::Overlay => app
                .workspace_state
                .install_overlay_for_test(seeded_overlay(OverlayKind::Command)),
        }
        assert_eq!(
            app.workspace_state.layer(),
            *rung,
            "the fixture did not reach {rung:?}",
        );
        let snapshot = app.semantic_snapshot();
        let focused: Vec<&str> = snapshot
            .nodes
            .iter()
            .filter(|node| node.focused)
            .map(|node| node.id.as_str())
            .collect();
        assert_eq!(focused.len(), 1, "{rung:?} focused {focused:?}");
        assert_eq!(focused[0], snapshot.focus_id, "{rung:?}");
    }
}

/// Passive surfaces are announced, never focused. All of them up at once, over
/// every rung of the ladder, must leave the focus owner exactly where the
/// active surface put it.
#[test]
fn passive_surfaces_announce_without_stealing_focus() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    for rung in workspace::Layer::ROSTER {
        let mut app = hermetic();
        match rung {
            workspace::Layer::Editor => {}
            workspace::Layer::Popover => app.workspace_state.summon_popover(true),
            workspace::Layer::Search => {
                app.workspace_state
                    .install_search_for_test(crate::search::SearchState::start(
                        0,
                        crate::search::Direction::Forward,
                    ));
            }
            workspace::Layer::Workspace => app
                .workspace_state
                .install_overlay_for_test(seeded_overlay(OverlayKind::Settings)),
            workspace::Layer::Overlay => app
                .workspace_state
                .install_overlay_for_test(seeded_overlay(OverlayKind::Command)),
        }
        let quiet = app.semantic_snapshot();
        crate::whichkey::set_force_shown(true);
        crate::menubar::set_menu_bar_on(true);
        crate::about::set_open(true);
        let loud = app.semantic_snapshot();
        calm_globals();

        assert_eq!(loud.focus_id, quiet.focus_id, "{rung:?} moved focus");
        assert_eq!(
            loud.nodes.iter().filter(|node| node.focused).count(),
            1,
            "{rung:?} produced more than one focus owner",
        );
        assert!(
            loud.nodes.len() > quiet.nodes.len(),
            "{rung:?} announced no passive surface at all",
        );
        for id in [WHICHKEY_ID, MENUBAR_ID] {
            let node = loud
                .nodes
                .iter()
                .find(|node| node.id == id)
                .unwrap_or_else(|| panic!("{rung:?} did not announce {id}"));
            assert!(!node.focused, "{id} took focus on {rung:?}");
        }
    }
}

/// The card an assistive technology hears is the card that is drawn: the fold
/// reads `crate::card::content`, so every card kind announces the composed
/// lines verbatim.
#[test]
fn every_card_kind_announces_the_composed_lines_verbatim() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let inputs = crate::card::content::CardInputs::default();
    for kind in crate::card::content::CardKind::ALL {
        let content = crate::card::content::card(kind, &inputs);
        let lines = content.lines();
        assert!(!lines.is_empty(), "{kind:?} says nothing");
        // A card node names the card and carries every line as a child, in
        // reading order, with no layout characters.
        let joined = lines.join(", ");
        assert!(!joined.contains('\n'), "{kind:?} leaked layout into speech");
    }
}

/// The menu bar's advertised Expand / Collapse really open and close a
/// dropdown, and its rows really appear when it does.
#[test]
fn menu_bar_expand_and_collapse_drive_the_real_dropdown() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    crate::menubar::set_menu_bar_on(true);
    let mut app = hermetic();
    let title = format!("{MENUBAR_ID}.0");
    assert!(app.apply_semantic_request(SemanticRequest::Expand { id: title.clone() }));
    assert_eq!(crate::menubar::open_menu(), Some(0));
    let open = app.semantic_snapshot();
    assert!(
        open.nodes
            .iter()
            .any(|node| node.id.starts_with(&format!("{title}.item."))),
        "an expanded menu announced no rows",
    );
    assert_eq!(
        open.nodes
            .iter()
            .find(|node| node.id == title)
            .unwrap()
            .expanded,
        Some(true),
    );
    assert!(app.apply_semantic_request(SemanticRequest::Collapse { id: title.clone() }));
    assert_eq!(crate::menubar::open_menu(), None);
    let shut = app.semantic_snapshot();
    assert!(
        !shut
            .nodes
            .iter()
            .any(|node| node.id.starts_with(&format!("{title}.item."))),
    );
    calm_globals();
}

/// A Settings range row advertises Increment / Decrement. They must move the
/// real value, not the caret.
#[test]
fn a_settings_range_row_increments_the_real_setting() {
    let _guard = crate::testlock::serial();
    let _restore = calm_globals_guarded();
    let mem = crate::fs::InMemoryFs::new()
        .with_dir("/ws")
        .with_dir("/cfg");
    let _fs = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let range = crate::settings::visible_rows()
        .iter()
        .position(|row| row.kind == crate::settings::SettingKind::Range)
        .expect("Settings ships at least one range row");
    // The REAL workspace, through its real chord: a hand-built Settings
    // overlay would not carry the category rail the row walk depends on.
    let mut app = App::new_hermetic(
        None,
        PathBuf::from("/ws"),
        Config {
            path: PathBuf::from("/cfg/config.toml"),
            ..Config::empty()
        },
    );
    let chord = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-,",
        crate::convention::Convention::Linux => "C-,",
    };
    app.press_spec_headless(chord).expect("the settings chord");
    app.press_spec_headless("Tab").expect("Tab crosses in");
    let id = format!("overlay.settings.row.{range}");
    let node = app
        .semantic_snapshot()
        .nodes
        .into_iter()
        .find(|node| node.id == id)
        .expect("the range row is in the tree");
    assert_eq!(node.role, SemanticRole::Slider);
    assert!(node.actions.contains(&SemanticAction::Increment));
    let before = node.value.clone();
    assert!(app.apply_semantic_request(SemanticRequest::Increment { id: id.clone() }));
    let after = app
        .semantic_snapshot()
        .nodes
        .into_iter()
        .find(|node| node.id == id)
        .unwrap()
        .value;
    assert_ne!(before, after, "Increment did not move the setting");
    // A range row is a live process-global; put it back so the shared test
    // lock's dirty-globals guard stays honest for everyone else.
    assert!(app.apply_semantic_request(SemanticRequest::Decrement { id: id.clone() }));
    assert_eq!(
        app.semantic_snapshot()
            .nodes
            .into_iter()
            .find(|node| node.id == id)
            .unwrap()
            .value,
        before,
        "Decrement is the exact inverse of Increment",
    );
}

/// THE headline law: an action a node claims but that nothing routes is worse
/// than an absent one, because an assistive technology offers it to the user
/// and it silently does nothing. Every advertised action on every node of
/// every surface must be HANDLED by `apply_semantic_request`.
#[test]
fn every_advertised_action_drives_a_real_transition() {
    let _guard = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    // Every arm below really RUNS: a picker row's Click accepts the row, and
    // an accept can open, rename or trash a file. `new_hermetic` swaps the
    // filesystem only while CONSTRUCTING, so the sweep needs its own guard or
    // it would walk (and write) the developer's real disk from `/`.
    let _fs = crate::fs::FsGuard::install(std::sync::Arc::new(
        crate::fs::InMemoryFs::new().with_dir("/ws"),
    ));
    calm_globals();
    #[derive(Debug)]
    enum Fixture {
        Editor,
        Popover,
        Search,
        MenuBar,
        Overlay(OverlayKind),
    }
    let mut fixtures = vec![
        Fixture::Editor,
        Fixture::Popover,
        Fixture::Search,
        Fixture::MenuBar,
    ];
    fixtures.extend(OverlayKind::ALL.into_iter().map(Fixture::Overlay));

    for fixture in fixtures {
        // A fresh App per (node, action): applying one action changes the
        // tree, and the claim under test is about the tree as advertised.
        let build =
            |fixture: &Fixture| {
                calm_globals();
                let mut app = App::new_hermetic(None, PathBuf::from("/ws"), Config::empty());
                app.set_semantic_text_for_test("some prose to select");
                match fixture {
                    Fixture::Editor => {}
                    Fixture::Popover => {
                        app.document.set_anchor(0);
                        app.document.set_cursor(4);
                        app.workspace_state.summon_popover(true);
                    }
                    Fixture::Search => app.workspace_state.install_search_for_test(
                        crate::search::SearchState::start(0, crate::search::Direction::Forward),
                    ),
                    Fixture::MenuBar => crate::menubar::set_menu_bar_on(true),
                    Fixture::Overlay(kind) => app
                        .workspace_state
                        .install_overlay_for_test(seeded_overlay(*kind)),
                }
                app
            };
        let advertised: Vec<(String, Vec<SemanticAction>)> = build(&fixture)
            .semantic_snapshot()
            .nodes
            .into_iter()
            .filter(|node| !node.actions.is_empty())
            .map(|node| (node.id, node.actions))
            .collect();
        assert!(
            !advertised.is_empty(),
            "{fixture:?} advertised no actions at all",
        );
        for (id, actions) in advertised {
            for action in actions {
                let mut app = build(&fixture);
                let request = match action {
                    SemanticAction::Focus => SemanticRequest::Focus { id: id.clone() },
                    SemanticAction::Click => SemanticRequest::Click { id: id.clone() },
                    SemanticAction::SetTextSelection => SemanticRequest::SetTextSelection {
                        id: id.clone(),
                        anchor: 0,
                        focus: 1,
                    },
                    SemanticAction::ReplaceSelectedText => SemanticRequest::ReplaceSelectedText {
                        id: id.clone(),
                        value: "x".to_string(),
                    },
                    SemanticAction::SetValue => SemanticRequest::SetValue {
                        id: id.clone(),
                        value: "x".to_string(),
                    },
                    SemanticAction::Increment => SemanticRequest::Increment { id: id.clone() },
                    SemanticAction::Decrement => SemanticRequest::Decrement { id: id.clone() },
                    SemanticAction::Expand => SemanticRequest::Expand { id: id.clone() },
                    SemanticAction::Collapse => SemanticRequest::Collapse { id: id.clone() },
                };
                assert!(
                    app.apply_semantic_request(request),
                    "{fixture:?}: node {id} advertises {action:?} but nothing routes it",
                );
            }
        }
    }
    calm_globals();
}
