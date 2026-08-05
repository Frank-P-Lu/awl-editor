use super::super::*;
use super::{keyspec, replay_keys};

// ── LINKS V2: Cmd-K stays --keys-drivable through the shared core ──

#[test]
fn replay_keys_cmd_k_wraps_a_selection_as_a_markdown_link() {
    // Type "hello", mark it with C-Space + move left across it, Cmd-K opens
    // the URL minibuffer pre-filled empty (WithText mode wrapping "hello"),
    // type a URL, RET commits — one atomic edit, fully sidecar-drivable.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys(
        "h e l l o C-Space Left Left Left Left Left s-k h t t p s : / / x . t e s t RET",
    )
    .unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.journey.card().is_none(), "commit closes the minibuffer");
    assert_eq!(buffer.text(), "[hello](https://x.test)");
}

#[test]
fn replay_keys_cmd_k_prompt_is_sidecar_visible_while_typing() {
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("h e l l o C-Space Left Left Left Left Left s-k h t t p s : / /")
            .unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.journey.card().expect("Cmd-K opens the link minibuffer");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::InsertLink);
    assert_eq!(ov.accepts(), vec!["https://"]);
    assert_eq!(
        ov.foot_hint(),
        "link to: https://   Enter commit   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Rename/Keybindings use"
    );
    assert_eq!(buffer.text(), "hello");
}

#[test]
fn replay_keys_cmd_k_esc_cancels_with_no_buffer_change() {
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("h e l l o C-Space Left Left Left Left Left s-k x x x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "Esc closes the minibuffer outright"
    );
    assert_eq!(buffer.text(), "hello", "cancel never edits the buffer");
}

#[test]
fn replay_keys_cmd_k_no_selection_inserts_empty_markup_caret_between_brackets() {
    // No selection, no existing link under the caret: Cmd-K inserts empty
    // `[](url)` markup; committing an empty URL is still a harmless, one-shot
    // edit (never a silent cancel the user didn't ask for).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-k RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.journey.card().is_none());
    assert_eq!(buffer.text(), "hi[]()");
}

#[test]
fn replay_keys_cmd_k_on_a_non_markdown_buffer_is_a_calm_no_op() {
    use std::path::PathBuf as PB;
    let mut buffer = Buffer::scratch();
    buffer.set_path(PB::from("/proj/main.rs"));
    buffer.insert_char('a');
    assert!(!buffer.is_markdown());
    let keys = keyspec::parse_keys("s-k").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "Cmd-K is a calm no-op on a non-markdown buffer"
    );
    assert_eq!(buffer.text(), "a");
}
