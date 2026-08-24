//! `card` — the ONE summoned float-card mechanism shared by awl's three summoned
//! cards: the About card (`about.rs`), the Lifetime-stats card (`lifetime.rs`),
//! and the hold-⌘ shortcut peek (`peek.rs`). Each is a THIN instance over this
//! one owner (same-behavior-same-code), so the three copies of the open-flag +
//! dismiss intercept collapse to a single source:
//!
//!  * [`CardFlag`] — the process-global OPEN boolean every card wants, with the
//!    identical open/close/read surface. `about`/`lifetime`/`peek` each own one
//!    and expose it under their own verb (`about_open`/`set_open`, …), so the
//!    MECHANISM is shared while each card's public API — and its tests — are
//!    unchanged.
//!  * [`dismiss_summoned_card`] — the any-key / any-click DISMISS intercept for
//!    the two MODAL cards (About + Lifetime stats, which OWN the next key). Both
//!    `actions::apply_transition`'s top-of-function arm and the live App's mouse-press
//!    handler dismiss through this ONE door instead of a per-card check+close.
//!
//! All three render through the SAME float-card pipeline
//! (`render/chrome/hud.rs::prepare_hud`, gated on their open flags). The hold-⌘
//! peek is deliberately NOT part of [`dismiss_summoned_card`]: it is not modal —
//! it closes when the hold breaks (`peek::PeekArm`), never on a key.

//! [`content`] is the sibling half of that same rule for what the cards SAY,
//! and [`figures`] owns the three lines of that content which are derived from
//! the document itself.

pub mod content;
pub mod figures;

use crate::hud::{HudSaved, HudStats};
use crate::peek::PeekRow;
use crate::streaks::StreaksView;

/// The figures a card can show that ONLY a running App can gather — a lifetime
/// odometer, a streak year, a save clock, a learned shortcut ledger, an update
/// marker. They reach a drawn card by being pushed into the render pipeline, so
/// the honest reading wherever no pipeline has been fed is the all-absent
/// [`Default`]: every one of them then composes its documented placeholder,
/// which is exactly what a headless capture draws.
#[derive(Debug, Clone, Default)]
pub struct CardLive {
    /// Lifetime odometer figures; `None` off the live App (every row reads as
    /// the placeholder).
    pub stats: Option<HudStats>,
    /// The streak card's year view, already folded to the placeholder off the
    /// live App.
    pub streaks: Option<StreaksView>,
    /// The SAVED figure; `None` off the live App.
    pub saved: Option<HudSaved>,
    /// The peek card's rows; empty means the ledger has taught nothing and the
    /// starter six show instead.
    pub peek_rows: Vec<PeekRow>,
    /// The About card's "last checked" marker; `None` off the live App.
    pub update_checked: Option<crate::updates::UpdateChecked>,
    /// A previous run left a crash log the About card should mention.
    pub pending_crash: bool,
}

/// The stats HUD DRAWS when its hold is live and no summoned overlay is up: the
/// two are mutually exclusive, so a still-held Option-Cmd-I never lays its card
/// over an open picker. One owner for the renderer's draw gate and the semantic
/// fold's announce gate, so the drawn card and the announced card can never
/// disagree about whether there is one.
pub fn hud_shown(overlay_active: bool) -> bool {
    crate::hud::hud_held() && !overlay_active
}

/// The hold-⌘ shortcut peek's twin of [`hud_shown`], same rule, same reason.
pub fn peek_shown(overlay_active: bool) -> bool {
    crate::peek::peek_open() && !overlay_active
}

use std::sync::atomic::{AtomicBool, Ordering};

/// A summoned-card OPEN flag: the process-global drawn-boolean every card wants,
/// with the identical open/close/read surface. Held as a `static` per card and
/// wrapped by that card's own-verb accessors (`about_open`/`set_open`, …), so the
/// flag boilerplate lives here ONCE.
pub struct CardFlag(AtomicBool);

impl CardFlag {
    /// A CLOSED flag — the calm-room default (no card drawn until summoned).
    pub const fn new() -> Self {
        CardFlag(AtomicBool::new(false))
    }
    /// True while the card is summoned / drawn.
    pub fn is_open(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    /// Open or close the card explicitly.
    pub fn set_open(&self, open: bool) {
        self.0.store(open, Ordering::Relaxed);
    }
}

/// Dismiss whichever MODAL summoned card (About, Lifetime stats, or Writing
/// streaks) is open, returning `true` iff one WAS open (and is now closed). THE
/// one owner of the "a modal card OWNS the next key/click" intercept:
/// `actions::apply_transition`'s top-of-function arm and the live App's mouse-press
/// handler both call this instead of duplicating a per-card check+close. They are
/// mutually exclusive (each opens only after the palette that summoned it closed,
/// and each dismisses on the first key), so closing "the open one" is the whole
/// contract. One carve-out lives UPSTREAM of this door: while the streaks card
/// is open, `apply_transition` intercepts ←/→ to flip its heatmap⇄cumulative page
/// (`streaks::toggle_view`) before ever reaching here — every other key still
/// dismisses. The
/// hold-⌘ peek is deliberately absent — it is not modal (it closes when the hold
/// breaks, via `peek::PeekArm`).
pub fn dismiss_summoned_card() -> bool {
    if crate::about::about_open() {
        crate::about::set_open(false);
        return true;
    }
    if crate::lifetime::lifetime_open() {
        crate::lifetime::set_open(false);
        return true;
    }
    if crate::streaks::streaks_open() {
        crate::streaks::set_open(false);
        return true;
    }
    false
}

/// True while any of the three MODAL summoned cards is up — the same three
/// flags [`dismiss_summoned_card`] checks, asked WITHOUT closing one.
/// `on_mouse_wheel`'s swallow-not-scroll guard reads this: a wheel is not the
/// press-owns-the-next-input intent `dismiss_summoned_card` answers, so the
/// guard needs "is one up" rather than "close the one that is."
pub fn modal_card_open() -> bool {
    crate::about::about_open() || crate::lifetime::lifetime_open() || crate::streaks::streaks_open()
}

/// Whether a PHYSICAL pointer position lands inside `rect` (`[x, y, w, h]`).
/// `None` reads as outside — the shared answer for a caller with no drawn
/// geometry to test against (no live renderer, e.g. a headless `App`), so a
/// press can never be mistaken for landing on a surface that was never drawn.
pub fn point_in_card(px: f32, py: f32, rect: Option<[f32; 4]>) -> bool {
    rect.is_some_and(|[x, y, w, h]| px >= x && px < x + w && py >= y && py < y + h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_card_reads_absent_geometry_as_outside_and_the_rect_as_half_open() {
        let rect = Some([10.0, 20.0, 30.0, 40.0]); // x:[10,40) y:[20,60)
        // No geometry at all: every point is outside, by construction.
        assert!(!point_in_card(15.0, 25.0, None));

        // Interior.
        assert!(point_in_card(11.0, 21.0, rect));
        assert!(point_in_card(39.9, 59.9, rect));

        // The near (top-left) edges are INCLUSIVE.
        assert!(point_in_card(10.0, 20.0, rect));

        // The far (bottom-right) edges are EXCLUSIVE — a point exactly at
        // x+w/y+h sits one pixel past the last column/row the rect covers.
        assert!(!point_in_card(40.0, 30.0, rect));
        assert!(!point_in_card(20.0, 60.0, rect));

        // Outside on every side.
        assert!(!point_in_card(9.9, 25.0, rect));
        assert!(!point_in_card(15.0, 19.9, rect));
        assert!(!point_in_card(41.0, 25.0, rect));
        assert!(!point_in_card(15.0, 61.0, rect));

        // A degenerate zero-size rect contains nothing, not even its own corner.
        assert!(!point_in_card(5.0, 5.0, Some([5.0, 5.0, 0.0, 0.0])));
    }

    #[test]
    fn modal_card_open_tracks_each_of_the_three_flags_independently() {
        let _g = crate::testlock::serial();
        crate::about::set_open(false);
        crate::lifetime::set_open(false);
        crate::streaks::set_open(false);
        assert!(!modal_card_open(), "all three closed: nothing modal is up");

        crate::about::set_open(true);
        assert!(modal_card_open());
        crate::about::set_open(false);
        assert!(!modal_card_open());

        crate::lifetime::set_open(true);
        assert!(modal_card_open());
        crate::lifetime::set_open(false);
        assert!(!modal_card_open());

        crate::streaks::set_open(true);
        assert!(modal_card_open());
        crate::streaks::set_open(false);
        assert!(!modal_card_open());
    }
}
