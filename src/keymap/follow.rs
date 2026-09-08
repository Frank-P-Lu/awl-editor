//! The POINTER-gesture roster: which mouse chord follows a followable span,
//! per platform, and the `[keys] follow` override that replaces it.
//!
//! Its own module rather than part of [`super::platform`] because it is a
//! different grammar, not a different table. The seed tables next door are KEY
//! chords composed and compared through `keyspec::parse_chord`; a mouse chord
//! has no spelling there at all, which is exactly what keeps the Linux
//! keep-list unable to collide with anything here. Keeping the two rosters in
//! one file invited a reader to assume one grammar.

use winit::keyboard::ModifiersState;

use crate::convention::Convention;

/// A MOUSE chord that follows a followable span. Deliberately its own type
/// rather than a `&str` in the seed tables above: the `[keys]` grammar for
/// COMMAND rebinds and every seed table are KEY chords, parsed by
/// `keyspec::parse_chord` into a `(Key, ModifiersState)` pair, and a mouse
/// button has no spelling there. That separation is also why the Linux
/// keep-list cannot collide with any gesture here — `Config::effective_linux_keep`
/// composes, and `linux_keeps_chord` compares, only strings that parse as key
/// chords. A follow gesture DOES have its own rebinding grammar
/// (`keyspec::parse_pointer_chord`, a deliberate sibling of `parse_chord`,
/// never a branch inside it) — see [`active_follow_gestures`]'s doc for the
/// override this type's `label` is generated for when a config line is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowGesture {
    /// Which physical button. Kept as a plain enum rather than winit's own so
    /// the roster stays a pure core fact the label surfaces can read.
    pub button: PointerButton,
    /// The modifiers that must be held, EXACTLY — a gesture with extra
    /// modifiers down is not this gesture.
    pub mods: ModifiersState,
    /// How the gesture is NAMED to a reader — carried ON the roster rather
    /// than composed at each surface, so the first label surface to want it
    /// reads [`active_follow_gestures`] the way every label surface already
    /// reads [`active_seed_tables`] through [`seeded_chords_for`]. A built-in
    /// default's label is a literal; an override's label is rendered once by
    /// `keyspec::format_pointer_chord` and leaked to `'static` (the same
    /// one-time-per-config-load cost `commands::COMMANDS`'s own splice pays).
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Middle,
    /// The right/secondary button. Spellable in the rebind grammar
    /// (`right-click`, per the decided grammar roster) so a `[keys] follow`
    /// line naming it parses and shows in the roster — but no default
    /// gesture uses it, and dispatch does not route a right press through
    /// [`follows_link`] at all: `app::input::mouse_button::on_mouse_input`
    /// claims every right press for the context-menu card before the follow
    /// predicate is ever asked. A `follow = "right-click"` override is
    /// therefore a named, left gap (documented in docs/config.md), not a
    /// silently broken promise.
    Secondary,
}

/// macOS: ⌘-click, the decided Mac gesture and the platform convention in every
/// Mac editor. Ctrl-click deliberately has NO entry here — macOS itself spends
/// Ctrl-click as the secondary click, so claiming it would fight the OS.
const MAC_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Primary,
    mods: ModifiersState::SUPER,
    label: "Cmd-Click",
}];

/// Linux, BOTH flavors: Ctrl-click. The platform convention in every editor and
/// browser, and a MOUSE chord — so it collides with none of the `C-c`/`C-x`
/// text-chord rules, which govern key chords alone. Super-click is not offered:
/// the compositor usually owns Super.
const LINUX_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Primary,
    mods: ModifiersState::CONTROL,
    label: "Ctrl-Click",
}];

/// Linux, BOTH flavors (widened by user decision off its original
/// `keymap = "emacs"`-only gate: middle-click collided with nothing under
/// `native`, and the flavor gate existed only to keep the platform convention
/// plain) — middle-click (mouse-2). Free to claim on either flavor: awl
/// implements no X11 primary-selection paste, so nothing else wants mouse-2.
const LINUX_MIDDLE_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Middle,
    mods: ModifiersState::empty(),
    label: "Middle-Click",
}];

/// THE FOLLOW GESTURE'S ONE SELECTION POINT — every mouse chord that follows a
/// followable span under `convention`, given any `[keys] follow` override
/// already loaded off `Config` (raw config strings, never pre-parsed — this
/// function owns the ONE parse). Both dispatch (the pointer press,
/// `app::input::mouse_button::press_follow_gesture`) and every label surface
/// read THIS, so an override and the built-in default can never drift apart.
///
/// `overrides` REPLACES the convention's default roster wholesale when it
/// parses to at least one valid gesture — unlike an ordinary `[keys]`
/// command rebind (additive: both the configured chord AND the default still
/// fire), because the follow roster is a small fixed SET, not a per-command
/// native/emacs pair, so "add to it" has no natural meaning. An entry that
/// [`crate::keyspec::parse_pointer_chord`] cannot spell is reported to
/// stderr NAMING the `[keys] follow` line and dropped — the same shape a bad
/// key chord already gets (`KeymapState::apply_overrides`'s own "keeping
/// default" note) — while any other valid entry in the line still takes
/// over; an override with NO valid entry at all leaves the built-in default
/// in force, exactly as if `overrides` were empty.
///
/// No longer flavor-gated (the decided widening above made Linux's roster
/// flavor-INDEPENDENT), so this function does not take one — see
/// `linux_emacs_layer`'s doc for what still does.
pub(crate) fn active_follow_gestures(
    convention: Convention,
    overrides: &[String],
) -> Vec<FollowGesture> {
    parse_follow_overrides(overrides).unwrap_or_else(|| default_follow_gestures(convention))
}

fn default_follow_gestures(convention: Convention) -> Vec<FollowGesture> {
    match convention {
        Convention::Mac => MAC_FOLLOW.to_vec(),
        Convention::Linux => LINUX_FOLLOW
            .iter()
            .chain(LINUX_MIDDLE_FOLLOW)
            .copied()
            .collect(),
    }
}

/// Parse a `[keys] follow` line into the roster it becomes; `None` when
/// `overrides` is empty or every entry fails to parse (the built-in default
/// keeps firing in either case — see [`active_follow_gestures`]'s doc for the
/// full per-entry contract).
fn parse_follow_overrides(overrides: &[String]) -> Option<Vec<FollowGesture>> {
    let mut out = Vec::new();
    for spec in overrides {
        match crate::keyspec::parse_pointer_chord(spec) {
            Ok((button, mods)) => {
                let label: &'static str =
                    Box::leak(crate::keyspec::format_pointer_chord(button, mods).into_boxed_str());
                out.push(FollowGesture {
                    button,
                    mods,
                    label,
                });
            }
            Err(e) => {
                eprintln!("config [keys]: follow = {spec:?}: {e}; keeping default");
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Does pressing `button` with exactly `mods` held FOLLOW, under
/// `convention` and any `[keys] follow` override? The one predicate both the
/// press path and the hover-cursor affordance ask, so the pointing hand and
/// the click can never disagree about which chord follows.
pub(crate) fn follows_link(
    convention: Convention,
    overrides: &[String],
    button: PointerButton,
    mods: ModifiersState,
) -> bool {
    active_follow_gestures(convention, overrides)
        .iter()
        .any(|g| g.button == button && g.mods == mods)
}
