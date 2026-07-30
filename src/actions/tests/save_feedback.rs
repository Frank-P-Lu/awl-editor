//! Persistence boundary laws: the shared transition describes writes and can
//! never perform them. The live interpreter owns the actual save.

use super::super::*;
use super::all_actions;
use crate::overlay::OverlayKind;

fn drive(buffer: &mut Buffer, action: Action) -> Transition {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, &action, false)
}

#[test]
fn save_is_one_typed_persistence_request_for_every_buffer_shape() {
    let _guard = crate::testlock::serial();
    let mut buffers = [
        Buffer::scratch(),
        Buffer::from_str("already pathed"),
        Buffer::scratch(),
    ];
    buffers[1].set_path("/docs/a.md".into());
    buffers[2].start_fresh_doc("/notes".into());

    for buffer in &mut buffers {
        let transition = drive(buffer, Action::Save);
        assert_eq!(
            transition.primary(),
            Effect::Persistence(PersistenceEffect::Save(SaveKind::Manual))
        );
    }
}

#[test]
fn core_save_transition_never_calls_buffer_save_old_direct_path_fails_by_name() {
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let _guard = crate::testlock::serial();
    let path = std::path::PathBuf::from("/docs/a.md");
    let mem = crate::fs::InMemoryFs::new().with_dir("/docs");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buffer = Buffer::from_str("edited");
        buffer.set_path(path.clone());
        let version = buffer.version();

        let transition = drive(&mut buffer, Action::Save);

        assert_eq!(
            transition.primary(),
            Effect::Persistence(PersistenceEffect::Save(SaveKind::Manual))
        );
        assert!(
            !mem.exists(&path),
            "MUTATION TRAP: apply_transition called the retired direct Buffer::save path"
        );
        assert_eq!(buffer.version(), version, "describing save is not a write");
    });
}

#[test]
fn core_finish_transition_never_saves_and_orders_save_notify_switch() {
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let _guard = crate::testlock::serial();
    let path = std::path::PathBuf::from("/docs/a.md");
    let mem = crate::fs::InMemoryFs::new().with_dir("/docs");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buffer = Buffer::from_str("edited");
        buffer.set_path(path.clone());
        let transition = drive(&mut buffer, Action::FinishBuffer);
        assert_eq!(
            &transition.effects()[..3],
            &[
                Effect::Persistence(PersistenceEffect::Save(SaveKind::Finish)),
                Effect::Daemon(DaemonEffect::NotifyFinished),
                Effect::Buffer(BufferEffect::Previous { finished: true }),
            ]
        );
        assert!(
            !mem.exists(&path),
            "MUTATION TRAP: FinishBuffer called the retired direct Buffer::save path"
        );
    });
}

#[test]
fn kill_ring_actions_emit_an_intercepted_clipboard_request_never_an_os_call() {
    let _guard = crate::testlock::serial();
    for action in [
        Action::DeleteWordBackward,
        Action::KillLine,
        Action::CopyRegion,
        Action::KillRegion,
    ] {
        let mut buffer = Buffer::from_str("alpha beta\n");
        buffer.select_range(0, 5);
        let transition = drive(&mut buffer, action.clone());
        let effect = transition
            .effects()
            .iter()
            .find(|effect| matches!(effect, Effect::Clipboard(_)))
            .unwrap_or_else(|| panic!("{action:?}: missing typed clipboard request"));
        assert_eq!(
            crate::replay::classify_for(effect, crate::replay::FilesystemCapability::None).class,
            crate::replay::EffectClass::Intercepted {
                detail: String::new()
            },
            "{action:?}: headless replay observes but never performs OS clipboard work"
        );
    }
}

#[test]
fn paste_is_a_typed_request_then_one_shared_core_continuation() {
    let _guard = crate::testlock::serial();
    let mut buffer = Buffer::from_str("hello");
    buffer.set_cursor(5);
    buffer.set_kill(" text");

    let request = drive(&mut buffer, Action::Yank);
    assert_eq!(
        request.primary(),
        Effect::Clipboard(ClipboardEffect::PasteImage)
    );
    assert_eq!(
        buffer.text(),
        "hello",
        "the request itself performs no paste"
    );

    let _ = drive(&mut buffer, Action::YankText);
    assert_eq!(buffer.text(), "hello text");
    buffer.undo();
    assert_eq!(buffer.text(), "hello");

    let _ = drive(
        &mut buffer,
        Action::InsertImageReference("assets/pasted-1.png".into()),
    );
    assert_eq!(
        buffer.text(),
        "hello\n![](assets/pasted-1.png)\n",
        "the resolved image reference enters through the shared edit core"
    );
    buffer.undo();
    assert_eq!(buffer.text(), "hello");
}

struct TwoRowsPerLine;

impl LayoutOracle for TwoRowsPerLine {
    fn visual_row_of(&self, line: usize, _col: usize) -> usize {
        line * 2
    }

    fn visual_x_of(&self, _line: usize, col: usize, _affinity: crate::caret::Affinity) -> f32 {
        col as f32
    }

    fn visual_line_up(
        &self,
        line: usize,
        col: usize,
        _goal_x: f32,
        _affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        (line.saturating_sub(1), col)
    }

    fn visual_line_down(
        &self,
        line: usize,
        col: usize,
        _goal_x: f32,
        _affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        (line + 1, col)
    }

    fn visual_line_start(
        &self,
        line: usize,
        _col: usize,
        _affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        (line, 0)
    }

    fn visual_line_end(
        &self,
        line: usize,
        col: usize,
        _affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        (line, col)
    }
}

#[test]
fn measured_page_input_uses_shared_wrapped_geometry_and_effects() {
    let _guard = crate::testlock::serial();
    let mut buffer = Buffer::from_str("0\n1\n2\n3\n4\n");
    let mut shift = true;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k| None;
    let mut browse_to = |_k, _r| None;
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 4,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: Some(&TwoRowsPerLine),
    };
    let transition = apply_transition(&mut ctx, &Action::PageScrollDown, false);
    assert_eq!(ctx.buffer.cursor_line_col(), (2, 0));
    assert!(!*ctx.shift_selecting);
    assert!(transition.contains(|effect| {
        matches!(
            effect,
            Effect::Render(RenderEffect::SyncView { follow: true })
        )
    }));
    assert!(
        transition.contains(|effect| { matches!(effect, Effect::Render(RenderEffect::Redraw)) })
    );
}

#[test]
fn zoom_change_is_a_typed_render_request_never_live_action_inference() {
    let _guard = crate::testlock::serial();
    for action in [Action::ZoomIn, Action::ZoomOut, Action::ZoomReset] {
        let mut buffer = Buffer::scratch();
        let transition = drive(&mut buffer, action.clone());
        assert!(
            transition
                .contains(|effect| { matches!(effect, Effect::Render(RenderEffect::ZoomChanged)) }),
            "{action:?} must describe its live render follow-up"
        );
    }
    let live = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/apply.rs"),
    )
    .unwrap();
    assert!(
        !live.contains("matches!(action, Action::Zoom"),
        "MUTATION TRAP: the live interpreter inferred zoom work from the originating Action"
    );
}

#[test]
fn platform_paging_about_and_image_paste_cannot_bypass_the_transition() {
    let live = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/apply.rs"),
    )
    .unwrap();
    for retired in [
        "page_scroll_intercept",
        "try_paste_image",
        "PreApply::Return",
        "mac_about::intercepts",
    ] {
        assert!(
            !live.contains(retired),
            "MUTATION TRAP: live action bypass `{retired}` returned around apply_transition"
        );
    }
    let paste_start = live.find("fn paste_image_reference").unwrap();
    let paste_end = live[paste_start..]
        .find("\n    pub(super) fn apply")
        .unwrap()
        + paste_start;
    let paste_body = &live[paste_start..paste_end];
    assert!(
        !paste_body.contains("replace_char_range")
            && !paste_body.contains("sync_view(")
            && !paste_body.contains("request_redraw"),
        "MUTATION TRAP: the external image transaction edited or rendered around the core"
    );
}

#[test]
fn every_action_routes_every_emitted_effect_through_the_closed_headless_policy() {
    let _guard = crate::testlock::serial();
    let caret0 = crate::caret::mode();
    let page0 = crate::page::page_on();
    let measure0 = crate::page::measure();
    let debug0 = crate::debug::debug_on();
    let hud0 = crate::hud::hud_held();
    let spellcheck0 = crate::spell::spellcheck_on();
    let about0 = crate::about::about_open();
    let lifetime0 = crate::lifetime::lifetime_open();
    let outline0 = crate::outline::outline_on();
    let typewriter0 = crate::typewriter::typewriter_on();
    let menubar0 = crate::menubar::menu_bar_on();
    let nits0 = crate::nits::nits_on();
    for action in all_actions() {
        // Summoned cards own the next action; reset them so every roster member
        // reaches its own transition rather than dismissing a predecessor.
        crate::about::set_open(false);
        crate::lifetime::set_open(false);
        crate::streaks::set_open(false);

        let mut buffer = Buffer::from_str("alpha beta\n");
        buffer.select_range(0, 5);
        let transition = drive(&mut buffer, action.clone());
        assert!(
            transition
                .contains(|effect| { matches!(effect, Effect::Render(RenderEffect::Redraw)) }),
            "{action:?}: every action transition must end in the closed render vocabulary"
        );
        for effect in transition.effects() {
            let ordinary =
                crate::replay::classify_for(effect, crate::replay::FilesystemCapability::None);
            let isolated =
                crate::replay::classify_for(effect, crate::replay::FilesystemCapability::Isolated);
            assert!(
                !ordinary.name.is_empty(),
                "{action:?}: unnamed ordinary route"
            );
            assert!(
                !isolated.name.is_empty(),
                "{action:?}: unnamed isolated route"
            );
        }
    }
    crate::caret::set_mode(caret0);
    crate::page::set_page_on(page0);
    crate::page::set_measure(measure0);
    crate::debug::set_debug_on(debug0);
    crate::hud::set_held(hud0);
    crate::spell::set_spellcheck_on(spellcheck0);
    crate::about::set_open(about0);
    crate::lifetime::set_open(lifetime0);
    crate::outline::set_outline_on(outline0);
    crate::typewriter::set_typewriter_on(typewriter0);
    crate::menubar::set_menu_bar_on(menubar0);
    crate::nits::set_nits_on(nits0);
    crate::streaks::set_open(false);
}

fn source_files_below(dir: &std::path::Path, extension: &str, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
    {
        let path = entry.expect("source directory entry").path();
        if path.is_dir() {
            source_files_below(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
}

#[test]
fn retired_single_effect_action_api_has_no_rust_or_contract_bypass() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    source_files_below(&root.join("src"), "rs", &mut sources);
    source_files_below(&root.join("docs"), "md", &mut sources);
    sources.extend(["CLAUDE.md", "ARCHITECTURE.md", "CAPTURE.md"].map(|name| root.join(name)));
    let retired = concat!("apply_", "core");
    for path in sources {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        assert!(
            !source.contains(retired),
            "{} still names the retired single-effect bypass",
            path.display()
        );
    }
}

#[test]
fn ordinary_capture_cannot_mint_isolated_filesystem_authority() {
    let ordinary_source = include_str!("../../main/run.rs");
    let strict_source = include_str!("../../main/replay_effects.rs");
    let isolated = concat!("FilesystemCapability::", "Isolated");
    assert_eq!(
        ordinary_source.match_indices(isolated).count(),
        0,
        "ordinary capture's owner must have no way to mint isolated authority"
    );
    let strict_owner = strict_source
        .split("impl ReplayPolicy {")
        .nth(1)
        .expect("strict capture owner exists")
        .split("/// Strict capture")
        .next()
        .expect("strict capture owner has a boundary");
    assert_eq!(
        strict_owner.match_indices(isolated).count(),
        1,
        "strict capture, and no ordinary capture door, mints isolated authority"
    );
}
