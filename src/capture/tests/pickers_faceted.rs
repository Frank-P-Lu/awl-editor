//! Faceted-lens overlay captures (theme/file/command/history pickers, the
//! empty-state row, the grouped scroll window) plus the byte-identical
//! double-capture + preview-id determinism checks -- split out of the former
//! monolithic `capture::tests` (2026-07 code-organization pass).

use super::super::*;
use super::adapter_available;
use crate::actions;
use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::testscratch::ScratchDir;

fn picker_opts(ov: &crate::overlay::OverlayState, empty: Option<String>) -> CaptureOpts {
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    opts.overlay = Some(OverlayInfo {
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title().to_string(),
        query: ov.query.text().to_string(),
        query_caret: ov.query.caret(),
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty,
        show_hidden: false,
    });
    opts
}

fn read_sidecar(png: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap()).unwrap()
}

fn assert_goto_lens_captures(
    dir: &std::path::Path,
    buf: &Buffer,
    goto: &mut crate::overlay::OverlayState,
) {
    let scheme = goto.facet_scheme().expect("Go-to has a facet scheme");
    let roster: Vec<(&str, &str)> = scheme
        .strip
        .iter()
        .map(|facet| (facet.label, facet.id))
        .collect();
    assert_eq!(
        roster,
        vec![
            ("All", "all"),
            ("Files", "files"),
            ("Headings", "headings"),
            ("Folders", "folders"),
            ("Recent", "recent"),
        ],
        "the capture law must enroll the complete typed Go-to roster"
    );

    let cases: [(&str, &str, usize); 5] = [
        ("all", "All", 7),
        ("files", "Files", 3),
        ("headings", "Headings", 2),
        ("folders", "Folders", 3),
        ("recent", "Recent", 2),
    ];
    for (id, label, item_count) in cases {
        goto.focus_facet_id(id);
        assert_eq!(goto.active_facet_id(), Some(id), "enrolled lens {id}");
        let png = dir.join(format!("goto_{id}.png"));
        capture_with(&png, buf, &picker_opts(goto, None)).expect("goto picker capture renders");
        let sidecar = read_sidecar(&png);
        assert_eq!(sidecar["overlay"]["mode"], serde_json::json!("goto"));
        assert_eq!(sidecar["overlay"]["lens"], serde_json::json!(id));
        assert_eq!(
            sidecar["overlay"]["lens_strip"],
            serde_json::json!([
                ["All", id == "all"],
                ["Files", id == "files"],
                ["Headings", id == "headings"],
                ["Folders", id == "folders"],
                ["Recent", id == "recent"]
            ]),
            "sidecar and rendered strip share the five-lens roster at {id}"
        );
        assert_eq!(
            sidecar["overlay"]["items"].as_array().unwrap().len(),
            item_count,
            "{id} enrolls its own typed destination rows"
        );
        let sections = sidecar["overlay"]["sections"].as_array().unwrap();
        if id == "all" {
            assert!(sections.iter().all(|s| s == ""), "All stays ungrouped");
        } else {
            assert!(
                sections.iter().all(|s| s == label),
                "{id} rows report their rendered {label} section: {sections:?}"
            );
        }
        let rows = sidecar["overlay"]["window"]["rows"]
            .as_array()
            .expect("a rendered Go-to lens publishes its row plan");
        assert!(
            rows.iter()
                .filter_map(|row| row["item"].as_u64())
                .all(|item| item < item_count as u64),
            "{id}: every drawn row maps back into the sidecar item roster"
        );
    }
}

fn assert_browse_git_lens(dir: &std::path::Path, buf: &Buffer) {
    use crate::overlay::{OverlayKind, OverlayState};

    let mut browse = OverlayState::new_marked(
        OverlayKind::Browse,
        vec!["repo".into(), "plain".into(), "note.md".into()],
        vec![true, false, false],
        vec![true, true, false],
        vec![],
        vec![],
        None,
    );
    for _ in 0..3 {
        browse.cycle_lens(1);
    }
    assert_eq!(browse.active_facet_id(), Some("git"));
    let png = dir.join("browse.png");
    capture_with(&png, buf, &picker_opts(&browse, None)).expect("browse picker capture renders");
    let sidecar = read_sidecar(&png);
    assert_eq!(sidecar["overlay"]["mode"], serde_json::json!("browse"));
    assert_eq!(sidecar["overlay"]["lens"], serde_json::json!("git"));
    let items = sidecar["overlay"]["items"].as_array().unwrap();
    assert_eq!(
        items.len(),
        1,
        "only the git repo under Git repos: {items:?}"
    );
    assert!(items[0].as_str().unwrap().contains("repo"));
}

fn capture_grouped_top(
    dir: &std::path::Path,
    buf: &Buffer,
    goto: &crate::overlay::OverlayState,
    n: usize,
    menu_bar: bool,
) -> f64 {
    let png = dir.join(format!("goto_top_menubar_{menu_bar}.png"));
    capture_with(&png, buf, &picker_opts(goto, goto.empty_notice()))
        .expect("grouped top capture renders");
    let sidecar = read_sidecar(&png);
    let window = &sidecar["overlay"]["window"];
    assert!(!window.is_null(), "an open faceted picker reports a window");
    let lines = window["lines"].as_u64().unwrap();
    let card_h = window["card_h"].as_f64().unwrap();
    let canvas_h = window["canvas_h"].as_f64().unwrap();
    let sel_row = window["sel_row"].as_u64().unwrap();
    assert!(lines < n as u64, "windowed: {lines} drawn lines < {n} rows");
    assert!(
        lines <= 12 + 1,
        "drawn lines ≤ item cap (12) + section header (1), got {lines}"
    );
    assert!(
        card_h <= canvas_h,
        "card_h {card_h} must fit canvas_h {canvas_h}"
    );
    assert!(
        sel_row < lines,
        "selected row {sel_row} within drawn window {lines}"
    );
    assert_eq!(window["top"].as_u64().unwrap(), 0, "list starts at the top");
    canvas_h
}

fn capture_grouped_bottom(
    dir: &std::path::Path,
    buf: &Buffer,
    goto: &crate::overlay::OverlayState,
    canvas_h: f64,
    menu_bar: bool,
) {
    let png = dir.join(format!("goto_bottom_menubar_{menu_bar}.png"));
    capture_with(&png, buf, &picker_opts(goto, goto.empty_notice()))
        .expect("grouped bottom capture renders");
    let sidecar = read_sidecar(&png);
    let window = &sidecar["overlay"]["window"];
    let lines = window["lines"].as_u64().unwrap();
    let top = window["top"].as_u64().unwrap();
    assert!(top > 0, "the window scrolled past the fold (top {top} > 0)");
    assert!(
        window["sel_row"].as_u64().unwrap() < lines,
        "the last row is visible in the scrolled window"
    );
    let card_h = window["card_h"].as_f64().unwrap();
    assert!(
        card_h <= canvas_h,
        "the scrolled card is still bounded ({card_h} ≤ {canvas_h})"
    );
}

fn capture_flat_window(dir: &std::path::Path, buf: &Buffer, menu_bar: bool) {
    use crate::overlay::{OverlayKind, OverlayState};

    let corpus: Vec<String> = (0..40).map(|i| format!("entry{i:02}")).collect();
    let mut flat = OverlayState::new(OverlayKind::MoveDest, corpus, vec![], vec![]);
    flat.move_sel(30);
    let png = dir.join(format!("flat_menubar_{menu_bar}.png"));
    capture_with(&png, buf, &picker_opts(&flat, flat.empty_notice()))
        .expect("flat picker capture renders");
    let sidecar = read_sidecar(&png);
    assert_eq!(
        sidecar["overlay"]["lens"],
        serde_json::json!(null),
        "flat: no lens"
    );
    let window = &sidecar["overlay"]["window"];
    assert_eq!(
        window["lines"].as_u64().unwrap(),
        12,
        "flat list caps at 12 rows"
    );
    assert!(
        window["sel_row"].as_u64().unwrap() < 12,
        "flat selection is on screen"
    );
    assert!(
        window["card_h"].as_f64().unwrap() <= window["canvas_h"].as_f64().unwrap(),
        "flat card fits the canvas"
    );
}

/// THEME PICKER (FLAT): its runtime lens strip was RETIRED (2026-07-15) — driving the
/// REAL [`OverlayState::new_theme`] through the capture renders its settled frame as a
/// FLAT browsable world list, and the sidecar reports `lens: null` / an empty strip /
/// no section labels, exactly like every other non-faceting picker. Exercises the flat
/// render branch end-to-end without a panic and pins that the theme picker no longer
/// draws (or reports) a lens strip.
#[test]
fn theme_picker_is_flat_and_reports_no_lens() {
    if !adapter_available() {
        eprintln!("skipping theme_picker_is_flat_and_reports_no_lens: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_themepick_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("preview me\n");

    // Build the REAL flat overlay: open on Potoroo, the active world. The swap is
    // held by an explicit `WorldPin`, so the world this test renders in goes home
    // when it ends — it used to end on a swap to TAWNY (not to the world it
    // found), which handed its world to whatever ran next and made
    // `render::tests::range_rail`'s thumb law pass or fail on test ORDER.
    let _pin = crate::theme::WorldPin::world("Potoroo").expect("Potoroo is a world");
    let names: Vec<String> = crate::theme::THEMES
        .iter()
        .map(|t| t.name.to_string())
        .collect();
    let ov = crate::overlay::OverlayState::new_theme(names.clone(), crate::theme::active_index());
    assert!(!ov.is_faceting(), "the theme picker is flat");
    assert_eq!(ov.active_facet_id(), None);

    // Fold it into capture opts exactly as the live replay does (see main/run.rs).
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    // The capture fixture layers optional overlay state for readable scenario setup.
    opts.overlay = Some(OverlayInfo {
        // Reproduce the prior live-resolved anchor for this capture literal.
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title().to_string(),
        query: ov.query.text().to_string(),
        query_caret: ov.query.caret(),
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: None,
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: None,
        show_hidden: false,
    });
    let png = dir.join("theme.png");
    capture_with(&png, &buf, &opts).expect("theme picker capture renders");
    let j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();
    let o = &j["overlay"];
    assert_eq!(o["mode"], serde_json::json!("theme"));
    // FLAT: null lens, empty strip, no section labels — the non-faceting sidecar shape
    // (a flat picker's per-row section labels are all the empty string, like every
    // other non-faceting picker).
    assert_eq!(
        o["lens"],
        serde_json::json!(null),
        "theme picker reports no lens"
    );
    assert_eq!(o["lens_strip"], serde_json::json!([]), "no lens strip");
    assert!(
        o["sections"].as_array().unwrap().iter().all(|s| s == ""),
        "no section grouping: {:?}",
        o["sections"]
    );
    // The flat list carries EVERY world in THEMES order, with Potoroo selected.
    let items: Vec<String> = o["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(items, names, "every world in declaration order, ungrouped");
    assert_eq!(
        items[o["selected_index"].as_u64().unwrap() as usize],
        serde_json::json!("Potoroo")
    );
}

/// EMPTY-STATE (pass 3): a picker whose query filters every row out renders + reports
/// the shared calm message through the sidecar `overlay.empty` field — "no matches"
/// for a query miss — while a picker WITH rows reports `empty: null`. Driven through
/// the REAL [`OverlayState`] into the capture exactly as `main/run.rs` folds it.
#[test]
fn overlay_empty_state_renders_and_reports() {
    if !adapter_available() {
        eprintln!("skipping overlay_empty_state_renders_and_reports: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_emptystate_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("preview me\n");

    let fold = |ov: &crate::overlay::OverlayState| OverlayInfo {
        // Reproduce the prior live-resolved anchor for this capture literal.
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title().to_string(),
        query: ov.query.text().to_string(),
        query_caret: ov.query.caret(),
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: ov.empty_notice(),
        show_hidden: false,
    };

    // A go-to picker with a query that matches NEITHER file → items empty → the
    // shared "no matches" empty-state, drawn as a dim message row + reported.
    let mut ov = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![],
    );
    for c in "zzz".chars() {
        ov.push(c);
    }
    assert!(
        ov.item_strings().is_empty(),
        "query filtered everything out"
    );
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    // The capture fixture layers optional overlay state for readable scenario setup.
    opts.overlay = Some(fold(&ov));
    let miss_png = dir.join("miss.png");
    capture_with(&miss_png, &buf, &opts).expect("empty-state capture renders");
    let miss: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(miss_png.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        miss["schema"],
        serde_json::json!(crate::capture::schema_plain())
    );
    assert_eq!(miss["overlay"]["items"], serde_json::json!([]), "no rows");
    assert_eq!(miss["overlay"]["empty"], serde_json::json!("no matches"));

    // A go-to picker WITH matching rows reports `empty: null` (there is a row list).
    let ov2 = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Goto,
        vec!["alpha.md".into()],
        vec![],
        vec![],
    );
    let mut opts2 = CaptureOpts {
        ..CaptureOpts::default()
    };
    // The capture fixture layers optional overlay state for readable scenario setup.
    opts2.overlay = Some(fold(&ov2));
    let hit_png = dir.join("hit.png");
    capture_with(&hit_png, &buf, &opts2).expect("non-empty capture renders");
    let hit: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(hit_png.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        hit["overlay"]["empty"],
        serde_json::json!(null),
        "rows → no empty-state"
    );
}

/// FILE PICKERS faceted lens strips: the go-to (Headings) + browse (Git repos)
/// pickers, driven through the REAL [`OverlayState`] into the capture, render their
/// settled frame AND the sidecar surfaces the lens / lens strip / per-row sections —
/// the same generic reporting the theme picker uses, proving the file pickers plug
/// into it end-to-end (the `--keys "… <right>"` payload a live replay produces).
#[test]
fn file_pickers_faceted_lens_render_and_report() {
    if !adapter_available() {
        eprintln!("skipping file_pickers_faceted_lens_render_and_report: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_filepick_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("preview me\n");
    use crate::overlay::{OverlayKind, OverlayState};

    // GO-TO's typed destination roster. Pin the five labels + IDs at their owner
    // before selecting by ID: positional RIGHT counts silently retargeted this
    // capture when Files / Folders joined the strip, leaving the law enrolled on
    // Folders while its assertion still said Headings.
    let goto_corpus = vec![
        "README.md".to_string(),
        "src/main.rs".to_string(),
        "notes.txt".to_string(),
    ];
    let mut goto = OverlayState::new(OverlayKind::Goto, goto_corpus, vec![], vec![1]);
    goto.attach_headings(vec![("Intro".to_string(), 0), ("Setup".to_string(), 3)]);
    goto.attach_folders(
        vec![
            ("notes/archive".to_string(), false),
            ("notes/repo".to_string(), true),
        ],
        &["notes/repo".to_string()],
    );
    // Every typed lens reaches the rendered card and sidecar through the same
    // fold. The row-counts are deliberately heterogeneous so a lens that is
    // positionally mis-enrolled cannot pass by showing another two-row type.
    const H: &str = OverlayKind::HEADING_MARKER_PREFIX;
    assert_goto_lens_captures(&dir, &buf, &mut goto);
    // Heading rows carry the `❡ ` kind-hint marker (the rowlayout PRIMARY-cell
    // disambiguator) even under their own dedicated lens.
    goto.focus_facet_id("headings");
    assert_eq!(
        goto.item_strings(),
        vec![format!("{H}Intro"), format!("{H}Setup")]
    );

    // BROWSE, cycled RIGHT×3 to the Git-repos lens: only the git-marked folder shows.
    assert_browse_git_lens(&dir, &buf);
}

/// GROUPED/FACETED WINDOW BOUND: a faceted picker under a SECTIONED lens on a LARGE
/// corpus draws a BOUNDED card (never past the canvas) and keeps the selected row
/// visible — the fix for the grouped path rendering its whole list uncapped off the
/// bottom of the screen. Driven through the REAL [`OverlayState`] into the capture, so
/// the assertion rides the same geometry the card renders from (the sidecar `window`
/// block). Also checks that MOVING the selection SCROLLS the window (the last section is
/// reachable) and that a FLAT picker still reports a bounded window (unchanged path).
#[test]
fn faceted_grouped_window_is_bounded_and_scrolls_to_selection() {
    if !adapter_available() {
        eprintln!(
            "skipping faceted_grouped_window_is_bounded_and_scrolls_to_selection: no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    use crate::overlay::{OverlayKind, OverlayState};
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_gwindow_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("preview me\n");

    // A LARGE Go-to file corpus focused by the roster's stable ID. This is the
    // five-lens typed surface's Files refinement: one section header + 60 rows,
    // far more than the 12-row window can show at once.
    let n = 60;
    let corpus: Vec<String> = (0..n).map(|i| format!("file{i:02}.md")).collect();
    let mut goto = OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![]);
    goto.focus_facet_id("files");
    assert_eq!(goto.active_facet_id(), Some("files"));
    assert_eq!(
        goto.item_strings().len(),
        n,
        "every file row shows under Files"
    );

    // The menu bar is the platform-default geometry split (off on macOS, on on
    // Linux). Sweep both branches in-process so this law does not depend on which
    // host happened to compile it, then restore the value the guard found.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    for menu_bar in [false, true] {
        crate::menubar::set_menu_bar_on(menu_bar);
        goto.move_sel(-(n as isize));

        // TOP of the list: the window is bounded and the selection (row 0) is on screen.
        let canvas_h = capture_grouped_top(&dir, &buf, &goto, n, menu_bar);

        // MOVE the selection to the LAST row (the bottom of the This-folder section) →
        // the window SCROLLS so the selection stays visible, and the top advances past
        // the fold.
        goto.move_sel(n as isize); // clamps to the last row
        assert_eq!(goto.selected, n - 1);
        capture_grouped_bottom(&dir, &buf, &goto, canvas_h, menu_bar);

        // FLAT PATH (a non-faceting picker) still reports a bounded window: a long list caps
        // at 12 rows, card fits the canvas, and the selection is on screen — unchanged.
        capture_flat_window(&dir, &buf, menu_bar);
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}

/// The COMMAND palette + HISTORY timeline gain the same ←/→ lens strip: the picker
/// renders its settled grouped frame through the real capture, and the sidecar
/// surfaces the lens / strip / grouped items — the same generic reporting the theme /
/// file pickers ride, now proven for the two new schemes. History also pins the
/// DETERMINISM gate: with no reference clock (the headless path) Session / Today group
/// nothing.
#[test]
fn command_and_history_pickers_faceted_lens_render_and_report() {
    if !adapter_available() {
        eprintln!(
            "skipping command_and_history_pickers_faceted_lens_render_and_report: no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_cmdhist_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("preview me\n");
    use crate::overlay::OverlayState;

    let fold = |ov: &OverlayState| {
        let mut opts = CaptureOpts {
            ..CaptureOpts::default()
        };
        // The capture fixture layers optional overlay state for readable scenario setup.
        opts.overlay = Some(OverlayInfo {
            // Reproduce the prior live-resolved anchor for this capture literal.
            align: crate::render::effective_card_anchor(),
            active: true,
            mode: ov.kind.as_str(),
            title: ov.kind.title().to_string(),
            query: ov.query.text().to_string(),
            query_caret: ov.query.caret(),
            items: ov.item_strings(),
            bindings: ov.item_bindings(),
            ranges: ov.item_range_fracs(),
            git: ov.item_git_tags(),
            selected_index: ov.selected,
            hint: ov.foot_hint(),
            browse_dir: ov.browse_dir.clone(),
            return_to: None,
            spell_target: None,
            context_anchor: None,
            capture: None,
            notice: String::new(),
            lens: ov.active_facet_id(),
            lens_strip: ov.lens_strip(),
            sections: ov.item_sections(),
            preview_id: None,
            preview_view: None,
            workspace: false,
            detail_focus: false,
            diff_scroll: 0,
            empty: None,
            show_hidden: false,
        });
        opts
    };
    let read = |png: &std::path::Path| -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap()).unwrap()
    };

    // COMMAND palette, cycled RIGHT once to Files: every shown row has that task
    // category (Save among them).
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut cmd =
        OverlayState::new_command(names, crate::commands::effective_bindings(&[], &[]), hidden);
    cmd.cycle_lens(1);
    assert_eq!(cmd.active_facet_id(), Some("files"));
    let cpng = dir.join("cmd.png");
    capture_with(&cpng, &buf, &fold(&cmd)).expect("command palette capture renders");
    let cj = read(&cpng);
    assert_eq!(cj["overlay"]["mode"], serde_json::json!("command"));
    assert_eq!(cj["overlay"]["lens"], serde_json::json!("files"));
    assert_eq!(
        cj["overlay"]["lens_strip"],
        serde_json::json!([
            ["All", false],
            ["Files", true],
            ["Navigate", false],
            ["Format", false],
            ["View", false],
            ["Tools", false],
            ["Settings", false],
            ["Recent", false]
        ])
    );
    let citems: Vec<String> = cj["overlay"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        citems.iter().any(|s| s == "Save"),
        "Save under Files: {citems:?}"
    );
    assert!(
        cj["overlay"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s == "Files"),
        "every Files-category row is headed Files"
    );

    // HISTORY timeline, headless (no reference clock). All lists every version; the
    // Session lens (RIGHT once) groups NOTHING — the determinism gate.
    let mkrow = |id: &str, pinned: bool| crate::history::TimelineRow {
        when: "x".to_string(),
        which: String::new(),
        counts: "+0 −0".to_string(),
        id: id.to_string(),
        timestamp: id.parse().unwrap_or(0),
        pinned,
        name: None,
    };
    let row = |id: &str| mkrow(id, false);
    // THE CONSCIOUS MARK: the newest version is PINNED, so its faint secondary
    // column wears the "pinned" tag — assertable straight from the sidecar's
    // `overlay.bindings`, the history block the picker draws from. NAMED SAVE
    // POINTS: the middle version is NAMED — its NAME is the primary `items`
    // text and its timestamp demotes into the `bindings` column, sidecar-
    // assertable through the same existing fields (no new plumbing).
    let named = crate::history::TimelineRow {
        name: Some("draft A".into()),
        pinned: true,
        ..mkrow("200", true)
    };
    let mut hist =
        OverlayState::new_history(vec![mkrow("300", true), named, row("100")], None, None);
    assert_eq!(hist.active_facet_id(), Some("all"));
    let hpng = dir.join("hist_all.png");
    capture_with(&hpng, &buf, &fold(&hist)).expect("history all capture renders");
    let hj = read(&hpng);
    assert_eq!(hj["overlay"]["mode"], serde_json::json!("history"));
    let hbinds = hj["overlay"]["bindings"].as_array().unwrap();
    assert!(
        hbinds[0]
            .as_str()
            .unwrap()
            .contains(crate::overlay::PIN_TAG),
        "the pinned version's binding carries the mark: {:?}",
        hbinds[0]
    );
    let hitems = hj["overlay"]["items"].as_array().unwrap();
    assert_eq!(
        hitems[1].as_str().unwrap(),
        "draft A",
        "the named point's NAME is its primary sidecar item: {:?}",
        hitems[1]
    );
    assert_eq!(
        hbinds[1].as_str().unwrap(),
        "x · +0 −0",
        "the named point's timestamp demotes into the secondary column: {:?}",
        hbinds[1]
    );
    assert!(
        !hbinds[2]
            .as_str()
            .unwrap()
            .contains(crate::overlay::PIN_TAG),
        "an un-pinned version stays bare: {:?}",
        hbinds[2]
    );
    assert_eq!(hj["overlay"]["lens"], serde_json::json!("all"));
    assert_eq!(
        hj["overlay"]["lens_strip"],
        serde_json::json!([["All", true], ["Session", false], ["Today", false]])
    );
    assert_eq!(
        hj["overlay"]["items"].as_array().unwrap().len(),
        3,
        "All lists every version"
    );
    hist.cycle_lens(1); // Session
    assert_eq!(hist.active_facet_id(), Some("session"));
    let hpng2 = dir.join("hist_session.png");
    capture_with(&hpng2, &buf, &fold(&hist)).expect("history session capture renders");
    let hj2 = read(&hpng2);
    assert_eq!(hj2["overlay"]["lens"], serde_json::json!("session"));
    assert!(
        hj2["overlay"]["items"].as_array().unwrap().is_empty(),
        "Session groups nothing without a clock — the determinism gate"
    );
}

/// THE BYTE-IDENTICAL LAW, as a durable test: the capture harness has NO clock /
/// animation / random, so running the SAME capture twice — a fresh device +
/// pipeline each run — must produce byte-for-byte identical PNGs AND sidecars.
/// The document exercises the layered render paths at once: markdown styling
/// (heading + bold), a fenced-code syntax block, and spell squiggles (the doc's
/// misspellings are re-derived deterministically inside each run). The same law
/// is asserted for a `capture_timeline` (every per-step PNG + sidecar). Any
/// nondeterminism smuggled into the frame (a clock read, an unseeded hash order,
/// an uninitialized texel) fails this loudly.
///
/// Every process-global that FOLDS INTO THE PIXELS or the sidecar is locked for
/// the whole double-run window — theme (colors/fonts), page (column), caret (look),
/// nits (underlines), debug (panel), hud (card), spell, about (card), lifetime,
/// outline, typewriter — in the suite-wide lock order, so a parallel global write
/// can't split the two runs.
#[test]
fn double_capture_is_byte_identical() {
    if !adapter_available() {
        eprintln!("skipping double_capture_is_byte_identical: no wgpu adapter");
        return;
    }
    // The sidecar reads every render-only process-global; hold each one's TEST_LOCK
    // so a parallel WRITER (the `actions::tests` all-actions sweeps flip
    // spell/outline/typewriter/lifetime/debug/hud/page/caret/about) can't mutate one
    // BETWEEN the two captures below and split the sidecars. Lock order matches the
    // sweeps' (spell before about; lifetime/outline/typewriter after) so the shared
    // locks are always acquired in the same order — no ABBA. (This set previously
    // rode `focus::TEST_LOCK` as an incidental barrier against the same sweeps, which
    // held it too; focus mode is gone, so the specific contended locks are named.)
    let _t = crate::testlock::serial();
    let _p = crate::testlock::serial();
    let _c = crate::testlock::serial();
    let _n = crate::testlock::serial();
    let _d = crate::testlock::serial();
    let _h = crate::testlock::serial();
    let _sp = crate::testlock::serial();
    let _ab = crate::testlock::serial();
    let _lf = crate::testlock::serial();
    let _ol = crate::testlock::serial();
    let _tw = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_double_capture_test_{}", std::process::id())),
    );

    // Markdown + a heading + bold (md spans), a rust fence (syntax roles), and
    // misspelled words ("Ttile" / "mispeled" / "strng") for the squiggle layer.
    let doc = "# Ttile\n\nsome mispeled **bold** prose here\n\n```rust\n// note\nlet s = \"strng\";\n```\n";
    let mut buf = Buffer::from_str(doc);
    buf.set_path(dir.join("doc.md"));

    // --- SINGLE FRAME, captured twice to the SAME path (fresh pipeline each) ---
    let png = dir.join("frame.png");
    capture_with(&png, &buf, &CaptureOpts::default()).expect("first capture");
    let png1 = std::fs::read(&png).unwrap();
    let json1 = std::fs::read(png.with_extension("json")).unwrap();
    capture_with(&png, &buf, &CaptureOpts::default()).expect("second capture");
    let png2 = std::fs::read(&png).unwrap();
    let json2 = std::fs::read(png.with_extension("json")).unwrap();
    assert!(
        png1 == png2,
        "two identical captures must write byte-identical PNGs \
         ({} vs {} bytes)",
        png1.len(),
        png2.len()
    );
    assert!(
        json1 == json2,
        "two identical captures must write byte-identical sidecars"
    );

    // --- TIMELINE, captured twice: every per-step PNG + sidecar matches -------
    let tl = dir.join("tl.png");
    let steps: [u32; 2] = [0, 30];
    capture_timeline(&tl, &buf, (0, 0), &steps, &CaptureOpts::default()).expect("first timeline");
    let read_steps = |dir: &std::path::Path| -> Vec<(Vec<u8>, Vec<u8>)> {
        steps
            .iter()
            .map(|ms| {
                (
                    std::fs::read(dir.join(format!("tl.t{ms}.png"))).unwrap(),
                    std::fs::read(dir.join(format!("tl.t{ms}.json"))).unwrap(),
                )
            })
            .collect()
    };
    let first = read_steps(&dir);
    capture_timeline(&tl, &buf, (0, 0), &steps, &CaptureOpts::default()).expect("second timeline");
    let second = read_steps(&dir);
    for (i, ms) in steps.iter().enumerate() {
        assert!(
            first[i].0 == second[i].0,
            "timeline step t{ms} must render a byte-identical PNG across runs"
        );
        assert!(
            first[i].1 == second[i].1,
            "timeline step t{ms} must write a byte-identical sidecar across runs"
        );
    }
}

/// HISTORY TIMELINE preview, sidecar half: a plain default capture reports
/// `overlay.preview_id: null` (the inactive arm), so every existing capture's
/// shape is stable — the schema-string asserts ride the `SCHEMA_*` consts and
/// update mechanically.
#[test]
fn preview_id_null_by_default() {
    if !adapter_available() {
        eprintln!("skipping preview_id_null_by_default: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_previewid_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("now text\n");
    let png = dir.join("plain.png");
    capture_with(&png, &buf, &CaptureOpts::default()).expect("plain capture");
    let j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        j["schema"],
        serde_json::json!(crate::capture::schema_plain())
    );
    assert_eq!(j["overlay"]["active"], serde_json::json!(false));
    assert_eq!(
        j["overlay"]["preview_id"],
        serde_json::Value::Null,
        "no preview in a default capture"
    );
}

/// HISTORY TIMELINE preview, capture half: `preview_text` folds over the render
/// snapshot BEFORE the scroll math (the live `sync_view` fold), so the sidecar
/// `text` reports THAT VERSION — "shows that version in the document itself",
/// assertable — with the cursor clamped into it and `overlay.preview_id` naming
/// the row. Driven via CaptureOpts exactly as `run.rs` folds a replayed
/// still-open History overlay.
#[test]
fn history_preview_folds_text_and_reports_preview_id() {
    if !adapter_available() {
        eprintln!("skipping history_preview_folds_text_and_reports_preview_id: no wgpu adapter");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_histprev_test_{}", std::process::id())),
    );
    // The buffer is the CURRENT text; the preview is a shorter OLDER version.
    let mut buf = Buffer::from_str("now line one\nnow line two\nnow line three\n");
    buf.set_cursor(buf.text().chars().count()); // cursor deep in the buffer
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    // The capture fixture layers optional history state after shared defaults.
    opts.preview_text = Some("old\n".to_string());
    opts.overlay = Some(OverlayInfo {
        // Reproduce the prior live-resolved anchor for this capture literal.
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: "history",
        title: "version history".to_string(),
        query: String::new(),
        query_caret: 0,
        items: vec!["2 hr ago · edited \"Old\"".into()],
        bindings: vec!["+2 −1".into()],
        ranges: Vec::new(),
        git: Vec::new(),
        selected_index: 0,
        hint: crate::overlay::OverlayKind::History.hint(),
        browse_dir: None,
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: None,
        lens_strip: Vec::new(),
        sections: Vec::new(),
        preview_id: Some("1700000000000".into()),
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: None,
        show_hidden: false,
    });
    let png = dir.join("preview.png");
    capture_with(&png, &buf, &opts).expect("preview capture renders");
    let j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();
    // The document IS the previewed version; the buffer's own text is absent.
    assert_eq!(j["text"], serde_json::json!("old\n"));
    assert_eq!(
        j["overlay"]["preview_id"],
        serde_json::json!("1700000000000")
    );
    assert_eq!(j["overlay"]["mode"], serde_json::json!("history"));
    // The cursor was clamped into the (shorter) previewed text.
    let line = j["cursor"]["line"].as_u64().unwrap();
    let col = j["cursor"]["col"].as_u64().unwrap();
    assert!(line <= 1, "cursor clamped into the preview's rows: {line}");
    assert!(col <= 3, "cursor clamped into the preview's cols: {col}");
}

/// A HISTORY PREVIEW REPLACES THE PAGE, NOT THE DOCUMENT. The preview
/// substitutes a writer's-diff TRANSCRIPT for what the renderer shapes, and the
/// card's document figures used to be read straight off it: a WORD COUNT over a
/// diff — its markers, and both sides of every change — is not a fact about
/// anything, and the LANGUAGE row vanished because a transcript carries no
/// frontmatter. Meanwhile the semantic snapshot, which derives the same figures
/// from the buffer, went on reporting the document's.
///
/// Driven through the real capture entry point with the real `preview_text`
/// fold, so the sidecar's `hud` and `readout` blocks are the drawn side
/// end-to-end. The transcript's own reading is asserted to be a DIFFERENT
/// number, so a green result cannot be a coincidence.
#[test]
fn a_history_preview_leaves_the_card_figures_over_the_users_document() {
    if !adapter_available() {
        eprintln!("skipping a_history_preview_leaves_the_card_figures_over_the_users_document");
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_prevfig_test_{}", std::process::id())),
    );
    use crate::card::figures::{DocFigures, fixture};

    let mut buf = Buffer::from_str(fixture::DOC);
    buf.set_cursor(63); // the start of `beta five six` — fixture::CARET
    assert_eq!(buf.cursor_line_col(), fixture::CARET);

    // THE ANNOUNCED SIDE, verbatim from `App::card_inputs`: the buffer's own text
    // and caret, which a preview never touches.
    let (cl, cc) = buf.cursor_line_col();
    let announced = DocFigures::of(&buf.text(), buf.is_markdown(), cl, cc);
    assert_eq!(announced.words, fixture::WORDS);

    // The transcript reads as something else entirely — the bug's answer.
    let transcript_reading = DocFigures::of(fixture::TRANSCRIPT, true, 0, 0);
    assert_ne!(transcript_reading.words, fixture::WORDS);
    assert_eq!(transcript_reading.lang, None);

    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    opts.preview_text = Some(fixture::TRANSCRIPT.to_string());
    opts.overlay = Some(OverlayInfo {
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: "history",
        title: "version history".to_string(),
        query: String::new(),
        query_caret: 0,
        items: vec!["2 hr ago · edited \"Old\"".into()],
        bindings: vec!["+2 −1".into()],
        ranges: Vec::new(),
        git: Vec::new(),
        selected_index: 0,
        hint: crate::overlay::OverlayKind::History.hint(),
        browse_dir: None,
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: None,
        lens_strip: Vec::new(),
        sections: Vec::new(),
        preview_id: Some("1700000000000".into()),
        // A History preview IS a comparison, so the real path emits its view tag
        // (`capture_fold`: `request.map(|r| r.view.tag())`); `None` here would
        // contradict the `preview_text` this fixture sets.
        preview_view: Some("diff"),
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: None,
        show_hidden: false,
    });
    let png = dir.join("preview_figures.png");
    capture_with(&png, &buf, &opts).expect("preview capture renders");
    let j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();

    // The page really is showing the transcript — else the figures below are
    // over the document by default and this proves nothing.
    assert_eq!(j["text"], serde_json::json!(fixture::TRANSCRIPT));

    // …and every document figure is still the DOCUMENT's.
    assert_eq!(
        j["hud"]["words"],
        serde_json::json!(fixture::WORDS_PAIR.0),
        "the drawn WORD COUNT counted the diff transcript, not the document",
    );
    assert_eq!(
        j["hud"]["reading_min"],
        serde_json::json!(fixture::WORDS_PAIR.1),
        "the drawn reading time followed the transcript's count",
    );
    assert_eq!(
        j["hud"]["unit"],
        serde_json::json!(fixture::WORDS_PAIR.2.tag()),
        "the transcript is plain Latin prose, so the unit stays words",
    );
    assert_eq!(
        j["hud"]["percent"],
        serde_json::json!(fixture::PERCENT),
        "the drawn THROUGH DOC measured the transcript, not the document",
    );
    assert_eq!(
        j["hud"]["lang"],
        serde_json::json!(crate::frontmatter::Lang::Ja.code()),
        "the document's frontmatter language, not the transcript's absence of one",
    );
    assert_eq!(
        j["readout"]["words"],
        serde_json::json!(fixture::WORDS_PAIR.0),
        "the quiet readout counted the diff transcript, not the document",
    );
    assert_eq!(
        j["readout"]["unit"],
        serde_json::json!(fixture::WORDS_PAIR.2.tag()),
        "the quiet readout's unit follows the same Latin transcript",
    );
}

/// THE RANGE ROW THROUGH THE REAL REPLAY + SIDECAR SEAM: drive RIGHT on
/// the Settings menu's Zoom row through the SAME `apply_transition` a `--keys` replay
/// runs, fold the still-open overlay through the SAME
/// [`crate::run::overlay_capture_info`] owner the one-shot capture uses, and
/// render it. The sidecar then reports the stepped value TWICE — as the row's
/// value TEXT (`bindings`) and as its RAIL FRACTION (`ranges`) — and the two must
/// agree, because both come from the one range spec. Every non-range row reports
/// `null`, so no other picker gains a rail.
#[test]
fn a_settings_range_row_steps_and_reports_its_rail_through_the_sidecar() {
    assert!(
        adapter_available(),
        "range sidecar law requires a wgpu adapter"
    );
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_rangerow_test_{}", std::process::id())),
    );
    let mut buf = Buffer::from_str("preview me\n");
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();

    let values = crate::settings::SettingsValues {
        zoom: spec.default,
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
        ..Default::default()
    };
    let mut ov = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Settings,
        crate::settings::visible_names(),
        vec![],
        vec![],
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&values));
    ov.set_range_cells(crate::settings::visible_range_cells(&values));
    let zi = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .unwrap();
    ov.selected = zi;

    // The value lands in the core (hence replay-`Applied`), so a
    // headless session observes it exactly as the live app does.
    let mut zoom = spec.default;
    let mut overlay = crate::overlay::Journey::seeded(Some(ov));
    // Settings is a summoned WORKSPACE: a fresh summon stands on its
    // navigation rail, and the rail's `→` enters the content pane. `→` on a
    // range ROW is the rail step this law is about, so put the card where a user
    // pressing this chord would be, through the lifecycle's own transition.
    overlay.toggle_detail();
    let mut shift = false;
    let mut search = None;
    let mut make = |_k: crate::overlay::OverlayKind| None;
    let mut browse = |_k: crate::overlay::OverlayKind, _p: Option<String>| None;
    let eff = {
        let mut ctx = crate::actions::ActionCtx {
            buffer: &mut buf,
            shift_selecting: &mut shift,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 10,
            journey: &mut overlay,
            make_overlay: &mut make,
            browse_to: &mut browse,
            oracle: None,
        };
        actions::apply_transition(&mut ctx, &Action::ForwardChar, false).primary()
    };
    assert_eq!(
        eff,
        crate::actions::Effect::SettingRangeStep {
            key: "zoom".to_string()
        }
    );
    let stepped = spec.stepped(spec.default, 1);
    assert_eq!(
        zoom, stepped,
        "the replay session's own zoom scalar moved one step"
    );

    // Fold + render through the SAME owner the one-shot `--keys` capture uses.
    let (info, _preview, _diff) =
        crate::run::overlay_capture_info(&overlay, &buf).expect("the menu is still open");
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    opts.overlay = Some(info);
    let png = dir.join("range.png");
    capture_with(&png, &buf, &opts).expect("the settings range row captures");
    let j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
            .unwrap();
    let o = &j["overlay"];
    assert_eq!(o["mode"], serde_json::json!("settings"));
    // …and the CARD'S OWN FOOT LINE, through the same `foot_hint` seam the live card
    // draws, advertises what ←/→ just did here — step the value, NOT cycle the lens.
    // The footer is awl's only statement of what a key does and there is no
    // accessibility tree behind it (ACCESSIBILITY.md), so this is agent-verifiable on
    // the sidecar rather than only in the pixels.
    // The workspace's derived BACK cell rides on the end of that line. It is
    // NAMED here rather than read back off the card, so this stays an assertion:
    // the content pane's query is empty, so the erase key is free and `⌫` is
    // what goes back.
    let mut expected = crate::overlay::OverlayKind::Settings.range_row_actions();
    expected.push(crate::overlay::workspace::BackKey::Erase.hint());
    assert_eq!(
        o["hint"],
        serde_json::json!(crate::overlay::format_hint(&expected)),
        "a selected rail row must advertise its own ←/→ meaning: {:?}",
        o["hint"]
    );
    assert_eq!(
        o["lens_strip"][0][1],
        serde_json::json!(true),
        "the lens did NOT move"
    );

    let items: Vec<String> = o["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let ranges = o["ranges"].as_array().unwrap();
    let bindings = o["bindings"].as_array().unwrap();
    assert!(ranges.iter().any(serde_json::Value::is_number));
    assert_eq!(
        ranges.len(),
        items.len(),
        "the rail column is parallel to the rows"
    );
    let row = items.iter().position(|n| n == "Zoom").unwrap();
    // The VALUE TEXT and the RAIL FRACTION are the same stepped value.
    assert_eq!(bindings[row], serde_json::json!(spec.format(stepped)));
    let frac = ranges[row]
        .as_f64()
        .expect("the Zoom row reports a rail fraction");
    assert!(
        (frac - spec.frac_of(stepped) as f64).abs() < 1e-3,
        "the reported thumb ({frac}) must be the spec's fraction for {stepped}"
    );
    // Every range row reports its own rail; every other row is railless.
    for (i, name) in items.iter().enumerate() {
        let is_range = super::settings_name_is_range(name);
        assert_eq!(
            ranges[i].is_number(),
            is_range,
            "{name}: range identity and sidecar cell must agree"
        );
    }
}

/// `overlay.query_caret` (schema `/209`) round-trips from the query field's
/// own `TextBox` caret through the sidecar writer — the caret position a
/// click-to-place / mid-query char-motion law asserts, proven at the actual
/// JSON serializer rather than only at `picker_opts`' construction.
#[test]
fn query_caret_reports_the_fields_own_position_not_just_its_length() {
    if !adapter_available() {
        eprintln!(
            "skipping query_caret_reports_the_fields_own_position_not_just_its_length: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    use crate::overlay::{OverlayKind, OverlayState};
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_querycaret_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("hello\n");
    let mut goto = OverlayState::new(
        OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![],
    );

    // Ordinary typing (the common case): the caret rests at the query's own
    // end, mirroring `query.text()`'s own length.
    goto.push('a');
    goto.push('b');
    let png = dir.join("at_rest.png");
    capture_with(&png, &buf, &picker_opts(&goto, None)).expect("at-rest capture renders");
    let sidecar = read_sidecar(&png);
    assert_eq!(sidecar["overlay"]["query"], serde_json::json!("ab"));
    assert_eq!(
        sidecar["overlay"]["query_caret"],
        serde_json::json!(2),
        "an ordinary typed query reports its caret at the field's own end"
    );

    // A click landing mid-query (`OverlayState::query_set_caret`, the same
    // door a pointer press resolves through) reports the INTERIOR position,
    // not the length again by coincidence.
    goto.query_set_caret(1);
    let png = dir.join("mid_query.png");
    capture_with(&png, &buf, &picker_opts(&goto, None)).expect("mid-query capture renders");
    let sidecar = read_sidecar(&png);
    assert_eq!(sidecar["overlay"]["query"], serde_json::json!("ab"));
    assert_eq!(
        sidecar["overlay"]["query_caret"],
        serde_json::json!(1),
        "a mid-query click's caret position must reach the sidecar, not just the query text"
    );
}
