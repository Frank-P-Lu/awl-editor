//! THE BUFFER-IDENTITY LAW — Credits used to swap the editor to a real
//! editable buffer (`App::open_credits` → `open_bundled_doc` → `load_path`),
//! so opening it, looking at it, and dismissing it silently changed what
//! document you were editing. `Action::OpenCredits` now performs exactly one
//! thing — `ctx.journey.enter(...)` plus the same `toggle_detail()` deep
//! link `Action::CompareVersion` already uses — and neither touches
//! `ctx.buffer` at all, so the regression is structurally impossible rather
//! than merely avoided. This file proves it over a real `Buffer` carrying a
//! real path, driven through the exact `apply_transition` seam `--keys`
//! replay shares.
//!
//! MUTATION-PROVED: temporarily reinstating the OLD dispatch — routing
//! `Action::OpenCredits` through a buffer-touching effect and applying it
//! with `ctx.buffer.set_path(...)` (standing in for the live App's
//! `load_path`, which a pure-core test cannot reach) — turns
//! `opening_credits_scrolling_and_dismissing_never_touches_the_active_buffer`
//! red by name.

use super::super::*;
use crate::buffer::Buffer;
use std::path::PathBuf;

#[test]
fn opening_credits_scrolling_and_dismissing_never_touches_the_active_buffer() {
    let mut buffer =
        Buffer::from_str("# My Notes\n\nSome real prose the user is in the middle of writing.\n");
    buffer.set_path(PathBuf::from("/notes/my-notes.md"));
    let before_path = buffer.path().map(|p| p.to_path_buf());
    let before_version = buffer.version();
    let before_text = buffer.text();
    assert_eq!(
        before_path.as_deref(),
        Some(std::path::Path::new("/notes/my-notes.md")),
        "sanity: the fixture buffer really carries the path the law is about"
    );

    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |k: OverlayKind| match k {
        OverlayKind::Credits => Some(OverlayState::new_credits()),
        _ => None,
    };
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

    // OPEN. Cmd-P → "Credits" lands straight on the content stage — there is
    // no row to choose, so the very first keypress must scroll.
    let _ = apply_transition(&mut ctx, &Action::OpenCredits, false);
    let card = ctx.journey.card().expect("Credits summons an overlay");
    assert_eq!(card.kind, OverlayKind::Credits);
    assert!(
        card.detail_focus,
        "Credits opens with the content stage already focused"
    );
    assert_eq!(card.diff_scroll, 0);

    // SCROLL. The same `diff_scroll` field History/Conflict already drive.
    let _ = apply_transition(&mut ctx, &Action::NextLine, false);
    let _ = apply_transition(&mut ctx, &Action::PageScrollDown, false);
    assert!(
        ctx.journey.card().unwrap().diff_scroll > 0,
        "scrolling the viewer must move diff_scroll, or the law below would \
         be vacuously true of a viewer that never actually opened"
    );

    // DISMISS.
    let _ = apply_transition(&mut ctx, &Action::Cancel, false);
    assert!(
        ctx.journey.card().is_none(),
        "Esc dismisses the viewer back to the editor"
    );

    // THE LAW: none of the above touched the active document.
    assert_eq!(
        buffer.path().map(|p| p.to_path_buf()),
        before_path,
        "the active buffer's path must not change across open/scroll/dismiss"
    );
    assert_eq!(
        buffer.version(),
        before_version,
        "the active buffer's version must not change across open/scroll/dismiss"
    );
    assert_eq!(
        buffer.text(),
        before_text,
        "the active buffer's text must not change across open/scroll/dismiss"
    );
}
