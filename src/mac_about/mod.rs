//! Native macOS chrome: the ABOUT WINDOW — one deliberate, authored `NSPanel`
//! that replaces `orderFrontStandardAboutPanelWithOptions:` (AppKit's generic
//! default, which every app that ships it looks identical in).
//!
//! Sibling of [`crate::mac_chrome`], which owns every OTHER objc2/AppKit door
//! (the open panel, the trash, the dock icon, the menu-icon rasterizer). This
//! one is split out because a hand-built window is a different KIND of AppKit
//! surface — it constructs and owns an object graph rather than making a call
//! — and because its pure halves ([`facts`], [`layout`]) are unit-testable
//! while nothing in `mac_chrome` is.
//!
//! Routing is unchanged: App menu ▸ "About Awl" and Cmd-P ▸ "About" both reach
//! `Action::About`, which `App::apply` intercepts on macOS and answers with
//! [`show`]. Every other platform keeps the in-app `about.rs` card untouched.
//!
//! # Who owns the window
//!
//! [`PANEL`] — a main-thread-only `thread_local` holding `Retained<AboutPanel>`
//! — is the SOLE owner, for the process's lifetime. Two hazards make that
//! explicit ownership mandatory rather than tidy:
//!
//! 1. **`releasedWhenClosed` is YES by default.** A programmatically created
//!    `NSWindow` releases itself when the user clicks its close button. With a
//!    `Retained` stored anywhere, that is a use-after-free on the next
//!    reference — the same class of bug as the dropped `muda::Menu` recorded in
//!    CLAUDE.md. [`build`] sets it to NO, so closing merely ORDERS OUT a window
//!    this module still holds, and reopening is a re-show rather than a rebuild.
//! 2. **`NSButton`'s target is unretained.** Both link buttons target the panel
//!    itself. That is deliberate: the panel outlives its own subviews, so the
//!    target can never dangle, and because the reference is unretained there is
//!    no cycle to leak — the panel's retain count comes from [`PANEL`] alone.
//!
//! Net effect: exactly one window object is ever built, it is never freed while
//! the app runs, and it is never referenced after a free. The thread-local dies
//! with the main thread, which is process exit.
//!
//! **Main-thread law** (same as `mac_chrome`): every entry point here is a calm
//! no-op off the main thread. `Action::About` is dispatched from `App::apply`,
//! which runs on the winit/main thread.
//!
//! **Zero network:** the panel fetches nothing, ever. The two buttons hand a
//! FIXED, compile-time URL to `NSWorkspace` — only, and always, after a click.
//!
//! **LIVE-ONLY:** the AppKit half cannot be driven by the headless capture
//! harness (there is no window server in a `--screenshot` run). What IS
//! mechanically checked lives in [`facts`] (every word the window says),
//! [`layout`] (every frame it places), [`show_reusing`] (the single-window
//! rule) and [`dismiss_chord`] (the keyboard contract). The remainder —
//! whether the composition reads as authored in both system appearances — is
//! flagged for human confirmation, never claimed here.
#![cfg(target_os = "macos")]

pub mod facts;
/// The rendered-pixel side of the two-ink law. Test-only: it exists to audit
/// captures of the window, never to run inside it.
#[cfg(test)]
pub mod ink;
pub mod layout;

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool};
use objc2::{AnyThread, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSBackingStoreType, NSBezelStyle, NSButton, NSColor, NSEvent, NSEventModifierFlags, NSFont,
    NSFontWeightRegular, NSFontWeightSemibold, NSImage, NSImageView, NSPanel, NSResponder,
    NSTextAlignment, NSTextField, NSView, NSWindow, NSWindowStyleMask, NSWindowTitleVisibility,
    NSWorkspace,
};
use objc2_foundation::{NSBundle, NSData, NSPoint, NSRect, NSSize, NSString, NSURL};

use crate::mac_about::layout::Frame;

thread_local! {
    /// The ONE About window, alive from its first summon until the process
    /// exits. See the module doc's ownership section: this `Retained` is the
    /// window's only strong reference, and the window is never released while
    /// the app runs.
    ///
    /// A `thread_local` rather than a `static`: `Retained<AboutPanel>` is
    /// neither `Send` nor `Sync` (AppKit views are main-thread-only), and this
    /// slot is only ever reached through a [`MainThreadMarker`] check.
    static PANEL: RefCell<Option<Retained<AboutPanel>>> = const { RefCell::new(None) };
}

define_class!(
    // SAFETY:
    // - NSPanel has no subclassing requirements beyond being main-thread-only,
    //   which `#[thread_kind = MainThreadOnly]` states.
    // - `AboutPanel` holds no ivars and implements no `Drop`.
    #[unsafe(super(NSPanel, NSWindow, NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[name = "AwlAboutPanel"]
    #[ivars = ()]
    /// awl's About window. An `NSPanel` (not a bare `NSWindow`) for one
    /// behavioural reason: `NSPanel` answers Escape by closing itself, which
    /// is the macOS convention this window must keep. The subclass exists to
    /// add Cmd-W and to be the buttons' action target.
    struct AboutPanel;

    impl AboutPanel {
        /// Close on the dismissal chords ([`dismiss_chord`]) before AppKit's
        /// own key-equivalent dispatch sees them. awl binds no Cmd-W of its
        /// own, so this ADDS the standard close chord to this window rather
        /// than shadowing an editor command.
        #[unsafe(method(performKeyEquivalent:))]
        fn perform_key_equivalent(&self, event: &NSEvent) -> Bool {
            let characters = event.charactersIgnoringModifiers();
            let command = event
                .modifierFlags()
                .contains(NSEventModifierFlags::Command);
            let characters = characters.map(|s| s.to_string()).unwrap_or_default();
            if dismiss_chord(command, &characters) {
                self.performClose(None);
                return Bool::YES;
            }
            // SAFETY: plain super-call of the method being overridden.
            unsafe { msg_send![super(self), performKeyEquivalent: event] }
        }

        /// The "Docs" button's action. Fires ONLY from a real click.
        #[unsafe(method(awlOpenDocs:))]
        fn open_docs(&self, _sender: Option<&AnyObject>) {
            open_url(facts::DOCS_URL);
        }

        /// The "GitHub" button's action. Fires ONLY from a real click.
        #[unsafe(method(awlOpenGitHub:))]
        fn open_github(&self, _sender: Option<&AnyObject>) {
            open_url(facts::GITHUB_URL);
        }
    }
);

/// Whether a key event should dismiss the About window: Escape, or Cmd-W.
///
/// Split out from the `performKeyEquivalent:` override so the keyboard
/// contract is testable without a window server. Escape is listed explicitly
/// even though `NSPanel` already closes on it — the behaviour is part of the
/// contract, so it is stated rather than inherited by luck.
fn dismiss_chord(command: bool, characters: &str) -> bool {
    match characters {
        "\u{1b}" => true,
        "w" | "W" => command,
        _ => false,
    }
}

/// Hand a FIXED destination to the user's browser. The only network-adjacent
/// act this window can perform, and only ever from a click.
fn open_url(url: &str) {
    let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) else {
        return;
    };
    NSWorkspace::sharedWorkspace().openURL(&url);
}

/// Whether this action is the one `App::apply` diverts to the native window on
/// macOS, instead of letting `apply_core` open the in-app `about.rs` card.
///
/// A named predicate rather than an inline `matches!` at the call site so the
/// diversion can be swept against the whole COMMAND roster: exactly one command
/// leaves the shared core, and every other menu and palette entry still reaches
/// it. Deliberately does NOT call [`show`] — a test that touched AppKit would
/// open a real window on whichever thread libtest happened to use.
pub fn intercepts(action: &crate::keymap::Action) -> bool {
    matches!(action, crate::keymap::Action::About)
}

/// Show the About window, building it the first time and RE-SHOWING it every
/// time after. The one door; `App::apply`'s macOS [`intercepts`] arm calls
/// this and nothing else.
pub fn show() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    PANEL.with(|slot| {
        // The borrow spans build+present, neither of which re-enters this
        // module (AppKit runs no awl code inside `makeKeyAndOrderFront:`).
        let mut slot = slot.borrow_mut();
        show_reusing(
            &mut slot,
            || build(mtm),
            |panel| {
                panel.makeKeyAndOrderFront(None);
            },
        );
    });
}

/// The single-window rule, as a shape rather than a comment: build into an
/// EMPTY slot only, then present whatever the slot holds.
///
/// Generic over the window type so the rule can be tested with a counting
/// double — the AppKit call it wraps is one line in [`show`]. A failed build
/// leaves the slot empty and presents nothing, so a window that could not be
/// constructed never becomes a phantom this module thinks it owns.
fn show_reusing<W>(slot: &mut Option<W>, build: impl FnOnce() -> Option<W>, present: impl Fn(&W)) {
    if slot.is_none() {
        *slot = build();
    }
    if let Some(window) = slot.as_ref() {
        present(window);
    }
}

/// What this process can learn about its own `.app` bundle. Outside a bundle
/// (a `cargo run`, a bare `target/release/awl`) the info dictionary has no
/// such keys and both answers are `None` — see [`facts::BundleFacts`].
fn bundle_facts() -> facts::BundleFacts {
    let read = |key: &str| -> Option<String> {
        NSBundle::mainBundle()
            .objectForInfoDictionaryKey(&NSString::from_str(key))?
            .downcast::<NSString>()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    };
    facts::BundleFacts {
        short_version: read("CFBundleShortVersionString"),
        build: read("CFBundleVersion"),
    }
}

/// Construct the window and its whole view tree. Called at most once per
/// process (see [`show_reusing`]).
fn build(mtm: MainThreadMarker) -> Option<Retained<AboutPanel>> {
    let lines = facts::fact_lines(&bundle_facts(), facts::commit());
    let l = layout::layout(lines.len());
    let (width, height) = l.content;
    let content_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height));

    // Titled + Closable: unmistakably a macOS window, with the standard close
    // button and nothing else. FullSizeContentView + a transparent, title-less
    // titlebar lets the composition own the whole card, so the window reads as
    // one authored surface instead of a dialog with a caption bar.
    let style = NSWindowStyleMask::Titled
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::FullSizeContentView;

    // SAFETY: NSPanel's designated initializer, called on a freshly allocated
    // instance of our subclass, on the main thread.
    let panel: Retained<AboutPanel> = unsafe {
        msg_send![
            AboutPanel::alloc(mtm),
            initWithContentRect: content_rect,
            styleMask: style,
            backing: NSBackingStoreType::Buffered,
            defer: false,
        ]
    };

    // THE OWNERSHIP TRIPWIRE (see the module doc): without this, clicking the
    // close button releases the window out from under `PANEL`, and the next
    // "About Awl" is a use-after-free.
    // SAFETY: turning self-release OFF is always sound; it only makes this
    // module's `Retained` the sole owner, which is exactly the intent.
    unsafe { panel.setReleasedWhenClosed(false) };

    // The title is invisible on the window itself but still names it in the
    // Window menu and to VoiceOver.
    panel.setTitle(&NSString::from_str("About Awl"));
    panel.setTitleVisibility(NSWindowTitleVisibility::Hidden);
    panel.setTitlebarAppearsTransparent(true);
    // No titlebar to grab, so the card itself is the drag handle.
    panel.setMovableByWindowBackground(true);
    // An About window is not a floating inspector: it behaves like a document
    // window, staying put when the user switches away and coming back with the
    // app rather than hovering over everything.
    panel.setFloatingPanel(false);
    panel.setHidesOnDeactivate(false);
    panel.setBecomesKeyOnlyIfNeeded(false);

    let content = NSView::initWithFrame(NSView::alloc(mtm), content_rect);

    // THE SHIPPED ICON, at real scale — see [`icon_bytes`].
    if let Some(icns) = icon_bytes()
        && let Some(image) = NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(icns))
    {
        let view = NSImageView::imageViewWithImage(&image, mtm);
        view.setFrame(rect(l.icon));
        content.addSubview(&view);
    }

    // NAME — the one loud element, and loud by SIZE AND WEIGHT ONLY: it is the
    // same body ink as the sentence beneath it.
    let title = NSTextField::labelWithString(&NSString::from_str(facts::NAME), mtm);
    // SAFETY: reading AppKit's `&'static NSFontWeight` constants is a plain
    // immutable static load (same pattern as `mac_chrome`'s weight constant).
    let semibold = unsafe { NSFontWeightSemibold };
    title.setFont(Some(&NSFont::systemFontOfSize_weight(
        layout::TITLE_FONT_SIZE,
        semibold,
    )));
    place(&content, &title, l.title, Ink::Body);

    // PRODUCT LINE — one sentence, body ink, the shared body size.
    let tagline = NSTextField::labelWithString(&NSString::from_str(facts::TAGLINE), mtm);
    tagline.setFont(Some(&NSFont::systemFontOfSize(layout::BODY_FONT_SIZE)));
    place(&content, &tagline, l.tagline, Ink::Body);

    // NOTHING IS DRAWN between the identity block and the provenance block.
    // The separation is `layout::GAP_IDENTITY_FACTS` — air, and only air.

    // PROVENANCE — the window's ONLY secondary ink, monospaced and small so it
    // reads as build metadata. Secondary, not fainter: measured on a packaged
    // light-mode capture, 11pt mono a step below this came out at 1.9:1
    // against the window background — decorative, not readable, and a commit
    // hash exists to be read.
    // SAFETY: as above, an immutable AppKit weight constant.
    let regular = unsafe { NSFontWeightRegular };
    let mono = NSFont::monospacedSystemFontOfSize_weight(layout::FACT_FONT_SIZE, regular);
    for (line, frame) in lines.iter().zip(l.facts.iter()) {
        let label = NSTextField::labelWithString(&NSString::from_str(line), mtm);
        label.setFont(Some(&mono));
        place(&content, &label, *frame, Ink::Secondary);
    }

    // CREDIT — a statement about the product, so body ink at the body size,
    // exactly like the product line above it.
    let attribution = NSTextField::labelWithString(&NSString::from_str(facts::ATTRIBUTION), mtm);
    attribution.setFont(Some(&NSFont::systemFontOfSize(layout::BODY_FONT_SIZE)));
    place(&content, &attribution, l.attribution, Ink::Body);

    // THE TWO ACTIONS. Their target is the panel itself — unretained by
    // AppKit, owned by `PANEL`, so no dangle and no cycle (module doc). The
    // tooltip states the destination, so a user can read where a click goes
    // before making it.
    let target: &AnyObject = &panel;
    for (frame, title, url, action) in [
        (l.buttons[0], "Docs", facts::DOCS_URL, sel!(awlOpenDocs:)),
        (
            l.buttons[1],
            "GitHub",
            facts::GITHUB_URL,
            sel!(awlOpenGitHub:),
        ),
    ] {
        // SAFETY: the action selector is implemented on `AboutPanel` above,
        // with the `(id sender)` signature AppKit invokes it with.
        let button = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str(title),
                Some(target),
                Some(action),
                mtm,
            )
        };
        button.setBezelStyle(NSBezelStyle::Push);
        // Body size; AppKit draws a push button's title in the control text
        // ink, which IS the body label ink — the third role a custom colour
        // here would smuggle in is simply never introduced.
        button.setFont(Some(&NSFont::systemFontOfSize(layout::BODY_FONT_SIZE)));
        button.setFrame(rect(frame));
        button.setToolTip(Some(&NSString::from_str(url)));
        content.addSubview(&button);
    }

    panel.setContentView(Some(&content));
    // NOTHING IS PRESELECTED. Without this, AppKit makes the first button the
    // key window's first responder and rings it blue — which reads as "Docs is
    // the default action" and puts a link one Return away from a user who only
    // meant to dismiss the window. A plain `NSView` refuses first-responder
    // status, so naming the content view here leaves the ring off while Full
    // Keyboard Access can still Tab to either button.
    panel.setInitialFirstResponder(Some(&content));
    panel.center();
    Some(panel)
}

/// THE WINDOW'S COMPLETE INK VOCABULARY — two roles, and there is no third.
///
/// This is the whole typographic hierarchy: everything that states something
/// about the product is [`Ink::Body`]; the build's provenance, which exists to
/// be findable rather than read first, is [`Ink::Secondary`]. Size and weight
/// separate the name from the sentence under it; ink does not.
///
/// Enforced from BOTH ends, because either alone can go green over the wrong
/// render:
///
/// * From the source: [`ink_color`] is the only place in this module that
///   resolves an `NSColor`, and it matches on `Ink` with no wildcard arm, so a
///   third role cannot be introduced without adding a variant here — which
///   [`tests::the_window_has_exactly_two_ink_roles`] refuses.
/// * From the pixels: [`ink::audit`] counts the distinct text inks in a REAL
///   capture of the rendered window and fails on a third, so a colour applied
///   behind `ink_color`'s back is caught by what actually drew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// The product's own voice: the name, the product line, the credit, and
    /// the button labels. AppKit's primary label ink.
    Body,
    /// Build provenance only — version, build, commit. AppKit's secondary
    /// label ink, the one grey in the window.
    Secondary,
}

impl Ink {
    /// Every role, for the roster law. A no-wildcard match keeps it honest: a
    /// new variant fails to compile until it is listed. Law surface only — the
    /// running window names its roles one at a time.
    #[allow(dead_code)]
    pub const ALL: &'static [Ink] = &[Ink::Body, Ink::Secondary];
}

/// The ONE resolver from a role to an AppKit colour. Module-private on purpose
/// — no other line in this module may name an `NSColor`.
fn ink_color(ink: Ink) -> Retained<NSColor> {
    match ink {
        Ink::Body => NSColor::labelColor(),
        Ink::Secondary => NSColor::secondaryLabelColor(),
    }
}

/// The artwork the About window shows: the CANONICAL bundle icon
/// (`assets/macos/Awl.icns` — the default world's), and deliberately NOT
/// `NSApplication.applicationIconImage`, which [`crate::app_icon::adopt`] swaps
/// to whichever world the user is currently writing in. This window is app
/// identity chrome; it must look the same whatever theme is active.
///
/// A named owner rather than an inline expression in [`build`] so the choice is
/// the thing the law tests, not just the asset — and so a change of mind here
/// has exactly one place to happen.
fn icon_bytes() -> Option<&'static [u8]> {
    crate::app_icon::icns_for(crate::app_icon::canonical_world().name)
}

/// A [`Frame`] as an AppKit rect.
fn rect(f: Frame) -> NSRect {
    NSRect::new(NSPoint::new(f.x, f.y), NSSize::new(f.w, f.h))
}

/// Place a centred label: frame, INK ROLE, alignment, and into the tree. Every
/// text element goes through this, so none can quietly differ from the others
/// — and because it takes an [`Ink`] rather than an `NSColor`, no caller can
/// reach for a colour that is not one of the two roles.
fn place(content: &NSView, label: &NSTextField, frame: Frame, ink: Ink) {
    label.setFrame(rect(frame));
    label.setAlignment(NSTextAlignment::Center);
    label.setTextColor(Some(&ink_color(ink)));
    content.addSubview(label);
}

#[cfg(test)]
mod tests;
