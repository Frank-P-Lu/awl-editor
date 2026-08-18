//! Reveal-in-file-manager + Copy-file-path: the pure-core half of the two
//! ordinary-file catalog commands. Both actions need a real on-disk path; a
//! path-less scratch buffer is the one axis worth sweeping here — everything
//! live-only (the actual Finder handoff, the OS clipboard write) is proven at
//! its own seam (`app::files::export::tests` for the shared reveal gate,
//! `app::tests::clipboard` for the OS clipboard mirror).

use super::super::*;
use super::drive_act_effect;

const PATH: &str = "/proj/notes/draft.md";

/// MUTATION TARGET: drop the `ctx.buffer.path()` gate in
/// `apply_deferred_action`'s `RevealInFileManager` arm (e.g. force the
/// `Effect::RevealInFileManager` arm unconditionally) and this fails by name —
/// a scratch buffer would then signal a handoff live never performs.
#[test]
fn reveal_on_a_scratch_buffer_signals_no_handoff_at_all() {
    let mut buffer = Buffer::from_str("no path here");
    assert_eq!(buffer.path(), None, "premise: a scratch buffer has no path");
    let effect = drive_act_effect(&mut buffer, &Action::RevealInFileManager);
    assert_eq!(
        effect,
        Effect::None,
        "a path-less buffer has nowhere to reveal; the pure core must not \
         signal a handoff live would never perform"
    );
}

/// A named document's Reveal carries exactly that document's own path — the
/// same shape `Action::FollowLink` already uses for its URL payload, so a
/// headless replay can classify + record the handoff without an `App`.
#[test]
fn reveal_on_a_named_buffer_carries_its_own_path() {
    let mut buffer = Buffer::from_str("# Draft\n");
    buffer.set_path(std::path::PathBuf::from(PATH));
    let effect = drive_act_effect(&mut buffer, &Action::RevealInFileManager);
    assert_eq!(
        effect,
        Effect::RevealInFileManager(std::path::PathBuf::from(PATH))
    );
}

/// MUTATION TARGET: drop the `buffer.path()` gate in `context_menu::copy_file_path`
/// and this fails by name — a scratch buffer's kill ring would pick up SOME
/// text (even if wrong) instead of staying exactly as it started.
#[test]
fn copy_file_path_on_a_scratch_buffer_leaves_the_kill_ring_untouched() {
    let mut buffer = Buffer::from_str("no path here");
    assert_eq!(buffer.kill_buffer(), "", "premise: nothing killed yet");
    drive_act_effect(&mut buffer, &Action::CopyFilePath);
    assert_eq!(
        buffer.kill_buffer(),
        "",
        "a path-less buffer has nothing to copy"
    );
}

/// The kill ring receives the buffer's exact absolute path — swept over a
/// plain path, one with a space, and one with non-ASCII text, since a naive
/// implementation could silently truncate or re-encode any of the three.
#[test]
fn copy_file_path_on_a_named_buffer_kills_the_exact_absolute_path() {
    for path in [
        "/proj/notes/draft.md",
        "/proj/Weekly Notes/plan a.md",
        "/proj/日記/今日.md",
    ] {
        let mut buffer = Buffer::from_str("body");
        buffer.set_path(std::path::PathBuf::from(path));
        drive_act_effect(&mut buffer, &Action::CopyFilePath);
        assert_eq!(
            buffer.kill_buffer(),
            path,
            "fixture {path:?}: the kill ring must hold the exact absolute path"
        );
    }
}

/// `Action::CopyFilePath` rides the same `WriteKillRing` mirror every other
/// kill-ring copy does (`CopyRegion`/`CopyLinkDestination`) — never a second
/// clipboard write path. The live OS-clipboard mirror itself is proven in
/// `app::tests::clipboard`.
#[test]
fn copy_file_path_pushes_the_ordinary_write_kill_ring_effect() {
    let mut buffer = Buffer::from_str("body");
    buffer.set_path(std::path::PathBuf::from(PATH));
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let transition = apply_transition(&mut ctx, &Action::CopyFilePath, false);
    assert!(
        transition
            .contains(|e| e == &Effect::Clipboard(crate::actions::ClipboardEffect::WriteKillRing)),
        "CopyFilePath must mirror to the OS clipboard through the ordinary \
         WriteKillRing effect, not a second write path"
    );
}
