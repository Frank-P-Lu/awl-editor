//! Laws for the About window. Carved out of `mod.rs`'s inline `mod tests`
//! when that file crossed the 500-line ceiling; the pure halves keep their
//! own tests beside them in `facts.rs` and `layout.rs`.
//!
//! What is provable here is the part of a live AppKit window that is NOT
//! AppKit: which command reaches it, which artwork it resolves, that a second
//! summon reuses the first window rather than stacking a second, and which
//! chords dismiss it. Whether the composition reads as authored is human
//! confirmation, not a test.

use super::*;
use std::cell::Cell;

/// A window double: knows only that it was built, and counts how often it
/// was presented.
struct FakeWindow {
    presented: Cell<u32>,
}

#[test]
fn a_second_summon_reuses_the_first_window() {
    let builds = Cell::new(0u32);
    let mut slot: Option<FakeWindow> = None;
    for _ in 0..3 {
        show_reusing(
            &mut slot,
            || {
                builds.set(builds.get() + 1);
                Some(FakeWindow {
                    presented: Cell::new(0),
                })
            },
            |w| w.presented.set(w.presented.get() + 1),
        );
    }
    assert_eq!(
        builds.get(),
        1,
        "About must build ONE window and raise it again — a second window \
         object means two About windows stacked on screen"
    );
    assert_eq!(
        slot.as_ref().unwrap().presented.get(),
        3,
        "every summon must raise the window, not silently do nothing"
    );
}

#[test]
fn a_window_that_could_not_be_built_is_not_remembered_or_presented() {
    let presents = Cell::new(0u32);
    let mut slot: Option<FakeWindow> = None;
    show_reusing(&mut slot, || None, |_| presents.set(presents.get() + 1));
    assert!(
        slot.is_none(),
        "a failed build must leave the slot empty, never a phantom this \
         module believes it owns"
    );
    assert_eq!(
        presents.get(),
        0,
        "nothing to present, so nothing presented"
    );
}

/// The keyboard contract, swept across the modifier axis — the one a
/// "check for Cmd-W" implementation gets wrong by also firing on a bare W.
#[test]
fn escape_and_command_w_dismiss_and_nothing_else_does() {
    assert!(dismiss_chord(false, "\u{1b}"), "Escape closes the panel");
    assert!(dismiss_chord(true, "\u{1b}"), "so does Cmd-Escape");
    assert!(dismiss_chord(true, "w"), "Cmd-W closes the panel");
    assert!(dismiss_chord(true, "W"), "Shift-Cmd-W too");
    assert!(
        !dismiss_chord(false, "w"),
        "a bare W must NOT close the window — it is an ordinary keystroke"
    );
    for ch in ["q", "a", "\r", "\t", " ", "", "1"] {
        for command in [false, true] {
            assert!(
                !dismiss_chord(command, ch),
                "{ch:?} (command={command}) must not dismiss the About window"
            );
        }
    }
}

/// MENU ROUTING, swept across the whole command roster: every menu and
/// palette command still reaches the shared `apply_core`, and exactly one
/// — About — is diverted to this window. The axis that matters is the one
/// a future "let's route Credits natively too" edit moves; a second
/// diverted command silently loses its in-app behaviour on macOS and
/// nowhere else.
#[test]
fn exactly_one_command_is_diverted_to_the_native_window() {
    let diverted: Vec<&str> = crate::commands::COMMANDS
        .iter()
        .filter(|c| intercepts(&c.action))
        .map(|c| c.name)
        .collect();
    assert_eq!(
        diverted,
        vec!["About"],
        "macOS diverts exactly the About command to the native window; \
         every other command must still reach apply_core"
    );
    assert!(
        intercepts(&crate::keymap::Action::About),
        "the App menu's 'About Awl' item and Cmd-P 'About' both dispatch \
         Action::About through this one seam"
    );
}

/// The window shows the SHIPPED bundle icon, not whichever world the user
/// happens to be writing in. Pinned to the bytes on disk that
/// `scripts/package-macos.sh` copies into `CFBundleIconFile`, so retinting
/// the About window to the active world — or letting the default world and
/// the bundle icon drift apart — goes red here.
#[test]
fn the_about_window_shows_the_canonical_bundle_icon() {
    let embedded = icon_bytes().expect("the About window resolves an icon on macOS");
    let on_disk = std::fs::read(crate::app_icon::CANONICAL_ICNS)
        .expect("the canonical bundle icon is committed");
    // `assert!`, not `assert_eq!`: these are 100 KB `.icns` blobs, and a
    // failure that dumps both of them byte by byte is unreadable.
    assert!(
        embedded == on_disk.as_slice(),
        "the About window's icon must be the very icon the bundle ships \
         ({}), not the active world's — it resolved {} bytes against the \
         file's {}",
        crate::app_icon::CANONICAL_ICNS,
        embedded.len(),
        on_disk.len()
    );
}
