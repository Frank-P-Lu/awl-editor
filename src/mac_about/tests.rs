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

// --- THE TWO-INK LAW, over real rendered pixels -----------------------------
//
// The window is a live `NSPanel`: awl's headless harness cannot render it (no
// window server in a `--screenshot` run, no main thread in a `cargo test`
// worker), so these laws read COMMITTED CAPTURES of the packaged app instead of
// rendering their own. The captures are produced by `CGWindowListCreateImage`
// at native (2x) resolution against `Awl.app`, one per system appearance.
//
// The staleness that a fixture invites is closed from two directions:
//
//   * `fixture_geometry_still_matches_the_layout` pins each capture's pixel
//     size to `layout::content_height`'s own arithmetic, so ANY change to the
//     composition's geometry fails here until the capture is retaken.
//   * `the_window_has_exactly_two_ink_roles` pins the SOURCE side — `Ink` has
//     two variants and `ink_color` is the only resolver — so a third colour
//     cannot be introduced even by an edit that leaves geometry alone.
//
// Neither is claimed to be sufficient alone, and that is stated rather than
// glossed: a pure colour change to an existing role would pass both until the
// fixture is retaken.

/// How many provenance lines the committed captures were taken with.
const FIXTURE_FACT_LINES: usize = 2;
/// The captures are retina self-captures: one layout point is two pixels.
const FIXTURE_SCALE: f64 = 2.0;

fn fixture(name: &str) -> image::RgbaImage {
    let path = format!("tests/fixtures/about/{name}.png");
    image::open(&path)
        .unwrap_or_else(|e| panic!("committed About capture {path} is unreadable: {e}"))
        .to_rgba8()
}

/// Both appearances, so every law below sweeps the axis that actually differs.
const FIXTURES: [&str; 2] = ["light", "dark"];

#[test]
fn fixture_geometry_still_matches_the_layout() {
    let l = layout::layout(FIXTURE_FACT_LINES);
    let want = (
        (l.content.0 * FIXTURE_SCALE) as u32,
        (l.content.1 * FIXTURE_SCALE) as u32,
    );
    for name in FIXTURES {
        let img = fixture(name);
        assert_eq!(
            (img.width(), img.height()),
            want,
            "the committed {name} capture is {}x{} but the current layout wants \
             {}x{} — the composition changed and the capture no longer shows \
             what this code renders. Retake it (packaged Awl.app, \
             CGWindowListCreateImage) before trusting the ink laws below.",
            img.width(),
            img.height(),
            want.0,
            want.1
        );
    }
}

/// THE LAW THE ITEM ASKS FOR, pixel side: every visible piece of text in the
/// rendered window belongs to exactly two ink roles — one body ink and one
/// secondary grey — with the provenance lines carrying the grey and everything
/// else the body ink; and no divider survives anywhere between the artwork and
/// the buttons.
///
/// Read PER ELEMENT off the committed captures, so the assertion is about what
/// each label actually drew rather than about a constant in this crate. Swept
/// across both system appearances, because a hardcoded colour passes in one and
/// fails in the other.
#[test]
fn the_rendered_window_uses_two_text_inks_and_draws_no_divider() {
    let l = layout::layout(FIXTURE_FACT_LINES);
    // A layout frame as a top-down pixel region in the capture.
    let region = |f: &layout::Frame| {
        let px = |p: f64| (p * FIXTURE_SCALE) as u32;
        (
            px(f.x),
            px(l.content.1 - f.top()),
            px(f.x + f.w),
            px(l.content.1 - f.y),
        )
    };

    for name in FIXTURES {
        let img = fixture(name);
        let read = |label: &str, f: &layout::Frame| {
            ink::element_ink(&img, region(f))
                .unwrap_or_else(|| panic!("{name}: the {label} frame rendered no text at all"))
        };

        // BODY ink: everything that states something about the product.
        let mut body = vec![
            read("name", &l.title),
            read("product line", &l.tagline),
            read("credit", &l.attribution),
            read("Docs button", &l.buttons[0]),
            read("GitHub button", &l.buttons[1]),
        ];
        // SECONDARY ink: the provenance block, and only it.
        let facts: Vec<_> = l
            .facts
            .iter()
            .enumerate()
            .map(|(i, f)| read(&format!("provenance line {i}"), f))
            .collect();

        assert_eq!(
            ink::distinct_roles(&body).len(),
            1,
            "{name}: the name, product line, credit and button labels must all \
             be the SAME body ink; the render used {:?}",
            ink::distinct_roles(&body)
        );
        assert_eq!(
            ink::distinct_roles(&facts).len(),
            1,
            "{name}: every provenance line must share one grey; got {facts:?}"
        );

        let mut all = body.clone();
        all.extend(facts.iter().copied());
        let roles = ink::distinct_roles(&all);
        assert_eq!(
            roles.len(),
            2,
            "{name}: the About window's whole visible type must resolve into \
             exactly two ink roles — body and one secondary grey. The render \
             used {}: {roles:?} (body {body:?}, provenance {facts:?})",
            roles.len()
        );

        // Two roles the eye can actually tell apart, with the grey the quieter
        // of the two — otherwise "two inks" is satisfied by a distinction
        // nobody can see, or by the provenance shouting over the product.
        body.dedup();
        let bg = ink::luminance(ink::background(&img, region(&l.tagline)));
        let (body_l, grey_l) = (ink::luminance(body[0]), ink::luminance(facts[0]));
        assert!(
            (grey_l - body_l).abs() >= 24.0,
            "{name}: the two inks are visually the same (body {body_l:.0}, \
             grey {grey_l:.0}) — one role wearing two hats"
        );
        assert!(
            (grey_l - bg).abs() < (body_l - bg).abs(),
            "{name}: the provenance grey must be the ink CLOSER to the \
             background (bg {bg:.0}, body {body_l:.0}, grey {grey_l:.0}); it is \
             the quiet role, not the loud one"
        );

        // NO DIVIDER, anywhere between the artwork and the buttons — the whole
        // band where a rule could plausibly have been drawn.
        let px = |p: f64| (p * FIXTURE_SCALE) as u32;
        let band = (
            0,
            px(l.content.1 - l.icon.y),
            img.width(),
            px(l.content.1 - l.buttons[0].top()),
        );
        let dividers = ink::divider_rows(&img, band, ink::background(&img, band));
        assert!(
            dividers.is_empty(),
            "{name}: the About window draws no rule; found divider-shaped \
             row(s) at y={dividers:?}. Grouping here is whitespace and rhythm \
             alone."
        );
    }
}

/// The SOURCE side of the same law: the vocabulary is two roles, full stop.
/// `ink_color`'s match has no wildcard arm, so a third colour cannot be spent
/// without adding a variant here — which this refuses.
#[test]
fn the_window_has_exactly_two_ink_roles() {
    assert_eq!(
        Ink::ALL,
        &[Ink::Body, Ink::Secondary],
        "the About window's whole typographic hierarchy is body ink and one \
         secondary grey; adding a role is a product decision, not a detail"
    );
    assert_ne!(Ink::Body, Ink::Secondary);
}
