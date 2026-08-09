use super::super::*;
use super::{keyspec, replay_keys};

/// LAW: the caret-MODE preference (an explicit pin, OR auto) must never be
/// mutated by mere THEME movement — a COMMITTED round-trip switch through a
/// one-bit world (Wagtail), or a theme-picker PREVIEW-and-Esc of one — is a
/// true no-op on the caret global. Covers both suspects the caret-style-change
/// bug report named: the 1-bit round's render-time override (`prepare_caret_
/// layer` reads `crate::caret::mode()` but never writes it — this is the
/// sticky round-trip proof of that) and auto-by-design (auto is legitimately
/// theme-dependent, but a journey that ENDS back on the same world must
/// resolve identically to never having left).
#[test]
fn caret_mode_survives_theme_journeys_committed_and_preview_esc() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    let root = PathBuf::from("/tmp");
    let keys =
        keyspec::parse_keys("s-t W a g t a i l RET s-t G u m t r e e RET s-t W a g t a i l Esc")
            .unwrap();

    crate::theme::set_active_by_name("Gumtree").unwrap();
    crate::caret::set_mode(crate::caret::CaretMode::Block);
    let mut buf = Buffer::scratch();
    replay_keys(&mut buf, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        crate::theme::active().name,
        "Gumtree",
        "the journey lands back on Gumtree"
    );
    assert!(
        !crate::caret::is_auto(),
        "an explicit pin is never cleared by a theme journey"
    );
    assert_eq!(crate::caret::mode(), crate::caret::CaretMode::Block);

    crate::caret::clear_override();
    crate::theme::set_active_by_name("Gumtree").unwrap();
    let mut buf2 = Buffer::scratch();
    replay_keys(&mut buf2, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(crate::theme::active().name, "Gumtree");
    assert!(
        crate::caret::is_auto(),
        "a theme-only journey never pins auto"
    );
    assert_eq!(
        crate::caret::mode(),
        crate::caret::CaretMode::Morph,
        "Gumtree (proportional) resolves Morph, exactly as if never visiting Wagtail"
    );

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::caret::clear_override();
}

/// STATELESSNESS LAW: the DRAWN caret for mode `M` in world `W` is a pure
/// function of `(M, W)` — never of the journey that got there. Proves it by
/// rendering the identical settled `(mode, world)` twice — once landed on
/// directly, once after a full COMMITTED Wagtail (one-bit) detour plus a
/// theme-picker preview-and-Esc of Wagtail — and diffing the PNG bytes. This
/// is the capture-level regression guard for suspect #1 (the 1-bit round's
/// `prepare_caret_layer` Morph->Block override must stay a pure per-frame
/// render decision, never leaking into the mode global or any pipeline
/// Globals left set from Wagtail's own frame).
#[test]
fn caret_render_is_a_pure_function_of_mode_and_world_across_a_wagtail_detour() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();
    let text = "hello frame\n";
    let detour_keys =
        keyspec::parse_keys("s-t W a g t a i l RET s-t G u m t r e e RET s-t W a g t a i l Esc")
            .unwrap();

    for mode in [
        crate::caret::CaretMode::Block,
        crate::caret::CaretMode::Morph,
        crate::caret::CaretMode::Ibeam,
    ] {
        crate::theme::set_active_by_name("Gumtree").unwrap();
        crate::caret::set_mode(mode);
        let base_buf = Buffer::from_str(text);
        let Some(_op) = capture::build_oracle(&base_buf, &opts) else {
            eprintln!(
                "skipping caret_render_is_a_pure_function_of_mode_and_world_across_a_wagtail_detour: no wgpu adapter"
            );
            crate::theme::set_active(crate::theme::DEFAULT_THEME);
            crate::caret::clear_override();
            return;
        };
        let dir = std::env::temp_dir();
        let pid = std::process::id();
        let base_png = dir.join(format!("awl_caret_stateless_base_{mode:?}_{pid}.png"));
        capture::capture_with(&base_png, &base_buf, &opts).expect("baseline capture");

        crate::theme::set_active_by_name("Gumtree").unwrap();
        crate::caret::set_mode(mode);
        let mut detour_buf = Buffer::from_str(text);
        replay_keys(
            &mut detour_buf,
            &detour_keys,
            &[],
            &root,
            None,
            &Config::empty(),
            None,
        );
        assert_eq!(
            crate::theme::active().name,
            "Gumtree",
            "the detour lands back on Gumtree"
        );
        assert_eq!(
            crate::caret::mode(),
            mode,
            "the detour never touched the pinned mode"
        );
        let detour_png = dir.join(format!("awl_caret_stateless_detour_{mode:?}_{pid}.png"));
        capture::capture_with(&detour_png, &detour_buf, &opts).expect("detour capture");

        let b1 = std::fs::read(&base_png).expect("read baseline png");
        let b2 = std::fs::read(&detour_png).expect("read detour png");
        assert_eq!(
            b1, b2,
            "mode {mode:?}: caret pixels must be byte-identical whether or not Wagtail was visited in between"
        );
        let _ = std::fs::remove_file(&base_png);
        let _ = std::fs::remove_file(base_png.with_extension("json"));
        let _ = std::fs::remove_file(&detour_png);
        let _ = std::fs::remove_file(detour_png.with_extension("json"));
    }

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::caret::clear_override();
}

/// THE POINTER-REPLAY SEAM, end to end through the REAL
/// headless `--keys` engine (`ReplaySession`, the exact type
/// `--screenshot --keys` constructs) — not a pure `OverlayState`
/// simulation. Opens a real 40-row Goto picker, hovers a row via the oracle's
/// real hit-test, drives a real
/// keyboard scroll past the candidate window, then re-checks the SAME
/// physical pixel: the row now under it (per the SAME real hit-test)
/// must not steal the keyboard's selection. Proves the seam the scout
/// named (`ReplaySession::cursor_px` + `apply_move`, sharing
/// `TextPipeline::resolve_overlay_hover` with the live
/// `App::overlay_hover`) actually reproduces the item's own named live
/// hazard deterministically and headlessly, through the sidecar-adjacent
/// `ReplaySession::overlay()` state oracle.
#[test]
fn item_106_pointer_replay_seam_reproduces_a_keyboard_scroll_stealing_a_stationary_pointer_check() {
    let _g = crate::testlock::serial();
    let mut buffer = Buffer::scratch();
    let corpus: Vec<String> = (0..40).map(|i| format!("row{i}.md")).collect();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();
    let Some(mut oracle) = capture::build_oracle(&buffer, &opts) else {
        eprintln!(
            "skipping item_106_pointer_replay_seam_reproduces_a_keyboard_scroll_stealing_a_stationary_pointer_check: \
                 no wgpu adapter"
        );
        return;
    };
    let config = Config::empty();
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = ReplaySession::new(
        ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        &root,
        None,
        &config,
        Some(&mut oracle),
        &mut km,
    );

    for chord in keyspec::parse_keys("s-o").unwrap() {
        session.apply_chord(&chord).unwrap();
    }
    assert!(session.overlay().is_some(), "Goto must be open");

    session.sync_oracle_overlay();
    let card = session
        .oracle()
        .expect("oracle present")
        .overlay_card_rect()
        .expect("goto card rect");
    let px = card[0] + card[2] * 0.5;
    let (py_top, py_bot) = (card[1], card[1] + card[3]);
    let find_row = |session: &ReplaySession, target: Option<usize>| -> Option<f32> {
        let op = session.oracle().expect("oracle present");
        let mut y = py_top;
        while y < py_bot {
            let hit = op.overlay_row_at(px, y);
            if target.is_none() && hit.is_some() {
                return Some(y);
            }
            if hit == target && target.is_some() {
                return Some(y);
            }
            y += 1.0;
        }
        None
    };
    let py = find_row(&session, Some(3)).expect("row 3 must be found within the card");

    session.apply_move(px, py);
    assert_eq!(
        session.overlay().unwrap().selected,
        3,
        "the real hover selected row 3"
    );

    for chord in keyspec::parse_keys(&"Down ".repeat(22)).unwrap() {
        session.apply_chord(&chord).unwrap();
    }
    assert_eq!(
        session.overlay().unwrap().selected,
        25,
        "keyboard nav landed on row 25"
    );
    assert!(session.overlay().unwrap().scroll > 0, "the window scrolled");

    // The window scrolled, so a different item now sits at the same pixel.
    session.sync_oracle_overlay();
    let hit_now = session
        .oracle()
        .expect("oracle present")
        .overlay_row_at(px, py);
    assert!(
        hit_now.is_some(),
        "the scrolled card still draws SOME row at that pixel"
    );
    assert_ne!(
        hit_now,
        Some(25),
        "a different item now sits under the stationary pixel"
    );

    // THE LAW: a stray re-check with a REAL 1px jitter off the parked
    // pixel — not the exact same coordinate (the exact-equality gate already
    // refused a bare duplicate; this law's
    // own regression needs genuine, if tiny, travel) — through the exact
    // production seam a spurious `CursorMoved` would drive — must not
    // steal the keyboard's selection.
    session.apply_move(px + 1.0, py);
    assert_eq!(
        session.overlay().unwrap().selected,
        25,
        "the keyboard's selection survives a 1px-jittered stationary pointer re-check"
    );

    session.sync_oracle_overlay();
    let py0 = find_row(&session, None).expect("display row 0 must be found");
    let hit0 = session
        .oracle()
        .expect("oracle present")
        .overlay_row_at(px, py0);
    session.apply_move(px, py0);
    assert_eq!(
        session.overlay().unwrap().selected,
        hit0.unwrap(),
        "a genuine pointer move to a different row takes over on the first event"
    );
}
