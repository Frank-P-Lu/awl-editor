//! `render/overrides` — the ten `AWL_*_FORCE` dev-only render/theme override
//! knobs, consolidated into ONE struct with ONE env-reading owner.
//!
//! [`RenderOverrides::from_env`] is the ONLY place these env vars are read
//! ([`tests::render_overrides_env_read_law`] enforces it by source scan).
//! Every reader resolves through [`current`]; production sources one value at
//! startup, `#[cfg(test)]` code can override it via [`set_test_override`] or
//! the ten legacy `set_*_test_override` wrappers (kept so existing test call
//! sites need no edits). One `#[cfg(test)]` `Mutex` remains as the test
//! bypass — not zero — because these readers are called from ~200 sites with
//! no shared owner object to thread a parameter through (see the commit
//! message for the full tradeoff). Its READ side is NOT testlock-guarded — see
//! [`TEST_OVERRIDE`]'s doc for why a uniform guard was tried and reverted.
//!
//! Its WRITE side is guarded twice over: [`assert_writer_serialized`] rejects a
//! writer that does not hold `crate::testlock::serial()`, and the guard itself
//! snapshots [`pins`] on entry and restores them on exit, so a forced knob
//! cannot survive the window that forced it even when that window unwinds. See
//! [`OverridePins`].
//!
//! The per-knob value parsers live in [`parsers`]; this file owns the
//! consolidated struct, its env resolution, and the test-override plumbing.

use crate::theme;

mod parsers;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use parsers::{ForcedKnob, classify_forced_knob};
pub(crate) use parsers::{
    OverlayMotionProbe, SlantProbe, TypeDensity, parse_facet_style_force, parse_list_style_force,
    parse_motion_force, parse_overlay_align, parse_overlay_anchor_force,
    parse_overlay_density_force, parse_overlay_motion_force, parse_overlay_slant_force,
    parse_overlay_style_force,
};
use parsers::{parse_chrome_face_force, parse_pane_split_force, read_forced_knob};

// ---------------------------------------------------------------------------
// THE CONSOLIDATED STRUCT
// ---------------------------------------------------------------------------

/// The resolved value of every `AWL_*_FORCE` render/theme override knob.
/// `None` means "not forced" — fall through to the world's own `RenderCaps`
/// data (or, for `density`, [`TypeDensity::shipped`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RenderOverrides {
    pub title_style: Option<theme::TitleStyle>,
    pub card_anchor: Option<theme::CardAnchor>,
    pub chrome_face: Option<theme::ChromeFace>,
    pub motion_juice: Option<theme::MotionJuice>,
    pub slant: Option<SlantProbe>,
    pub list_style: Option<theme::ListStyle>,
    pub facet_style: Option<theme::FacetStyle>,
    pub pane_split: Option<theme::PaneSplit>,
    pub density: Option<TypeDensity>,
    pub overlay_motion: Option<OverlayMotionProbe>,
}

impl RenderOverrides {
    /// The only place these env vars are read. `AWL_OVERLAY_ALIGN` wins over
    /// `AWL_OVERLAY_ANCHOR_FORCE` for `card_anchor`; see
    /// [`render_overrides_env_read_law`] below for the exclusivity check.
    pub(crate) fn from_env() -> Self {
        RenderOverrides {
            title_style: std::env::var("AWL_OVERLAY_STYLE_FORCE")
                .ok()
                .and_then(|s| parse_overlay_style_force(&s)),
            card_anchor: std::env::var("AWL_OVERLAY_ALIGN")
                .ok()
                .and_then(|s| parse_overlay_align(&s))
                .or_else(|| {
                    std::env::var("AWL_OVERLAY_ANCHOR_FORCE")
                        .ok()
                        .and_then(|s| parse_overlay_anchor_force(&s))
                }),
            chrome_face: std::env::var("AWL_CHROME_FACE_FORCE")
                .ok()
                .and_then(|s| parse_chrome_face_force(&s)),
            motion_juice: std::env::var("AWL_MOTION_FORCE")
                .ok()
                .and_then(|s| parse_motion_force(&s)),
            slant: std::env::var("AWL_OVERLAY_SLANT_FORCE")
                .ok()
                .and_then(|s| parse_overlay_slant_force(&s)),
            list_style: read_forced_knob(
                "AWL_OVERLAY_LIST_FORCE",
                "pane | bars | bars:<radius>:<gap>:<grow>[:hug|huglabel|full][:selected|all]",
                parse_list_style_force,
            ),
            facet_style: read_forced_knob(
                "AWL_FACET_STYLE_FORCE",
                "text | band | chips[:hairline|bold|filled|underline|tinted|bracket]",
                parse_facet_style_force,
            ),
            pane_split: read_forced_knob(
                "AWL_PANE_SPLIT_FORCE",
                "unified | split",
                parse_pane_split_force,
            ),
            density: read_forced_knob(
                "AWL_OVERLAY_DENSITY_FORCE",
                "<scale> | <scale>:<leading>",
                parse_overlay_density_force,
            ),
            overlay_motion: read_forced_knob(
                "AWL_OVERLAY_MOTION_FORCE",
                "<enter> | <enter>:<band>  (each 0..1)",
                parse_overlay_motion_force,
            ),
        }
    }

    /// `self`'s fields win where `Some`; `base` fills any `None`. Exhaustive
    /// field-by-field so a new field must be added here consciously.
    #[cfg(test)]
    fn or(self, base: &RenderOverrides) -> RenderOverrides {
        RenderOverrides {
            title_style: self.title_style.or(base.title_style),
            card_anchor: self.card_anchor.or(base.card_anchor),
            chrome_face: self.chrome_face.or(base.chrome_face),
            motion_juice: self.motion_juice.or(base.motion_juice),
            slant: self.slant.or(base.slant),
            list_style: self.list_style.or(base.list_style),
            facet_style: self.facet_style.or(base.facet_style),
            pane_split: self.pane_split.or(base.pane_split),
            density: self.density.or(base.density),
            overlay_motion: self.overlay_motion.or(base.overlay_motion),
        }
    }
}

fn env_overrides() -> &'static RenderOverrides {
    static ONCE: std::sync::OnceLock<RenderOverrides> = std::sync::OnceLock::new();
    ONCE.get_or_init(RenderOverrides::from_env)
}

/// The one `#[cfg(test)]` bypass for every knob in [`RenderOverrides`].
///
/// Its READ side is not testlock-guarded, matching nine of the ten predecessor
/// statics this module replaced (only the forced `list_style` asserted
/// `crate::testlock::currently_held()`). Guarding [`current`] uniformly was
/// tried and reverted: `card_anchor` alone is read incidentally by
/// `OverlayState::open` from ~120 test call sites across `overlay::tests`,
/// `actions::tests`, `app::tests`, and `index::tests` that never hold
/// `crate::testlock::serial()` (they don't touch any override, they just
/// build overlay state) — asserting there would demand a testlock refactor
/// across all of them, well outside this round's scope. A test that DOES
/// mutate an override still serializes correctly against a concurrent test,
/// because `crate::testlock::serial()`'s own mutex — not this assert —
/// provides the actual mutual exclusion (see
/// `list_style_override_reader_writer_are_serialized`, which pins that with
/// a real second thread).
#[cfg(test)]
static TEST_OVERRIDE: std::sync::Mutex<RenderOverrides> = std::sync::Mutex::new(RenderOverrides {
    title_style: None,
    card_anchor: None,
    chrome_face: None,
    motion_juice: None,
    slant: None,
    list_style: None,
    facet_style: None,
    pane_split: None,
    density: None,
    overlay_motion: None,
});

/// The resolved overrides for THIS read: the `#[cfg(test)]` override (if any
/// field is set) layered over the env-sourced value; production builds skip
/// straight to the env-sourced value.
pub(super) fn current() -> RenderOverrides {
    #[cfg(test)]
    {
        let test = TEST_OVERRIDE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        test.or(env_overrides())
    }
    #[cfg(not(test))]
    {
        env_overrides().clone()
    }
}

/// Install a whole [`RenderOverrides`] as the test override in one call,
/// instead of the ten named setters below.
#[cfg(test)]
pub(crate) fn set_test_override(overrides: RenderOverrides) {
    assert_writer_serialized();
    *TEST_OVERRIDE.lock().unwrap_or_else(|e| e.into_inner()) = overrides;
}

#[cfg(test)]
fn set_field(f: impl FnOnce(&mut RenderOverrides)) {
    assert_writer_serialized();
    let mut g = TEST_OVERRIDE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut g);
}

/// The WRITE half of the guard, matching `theme::set_active`: mutating a
/// process-global off-guard is a hard error, while reading it is not. Only
/// writers are asserted here — see [`TEST_OVERRIDE`] for why the read side
/// cannot be.
#[cfg(test)]
fn assert_writer_serialized() {
    assert!(
        crate::testlock::currently_held(),
        "a RenderOverrides test override was installed without holding \
         crate::testlock::serial()"
    );
}

// ---------------------------------------------------------------------------
// THE SERIAL GUARD'S RESTORE SURFACE
// ---------------------------------------------------------------------------

/// The living-band choreography probe: an eleventh forced render knob, stored
/// here beside the other ten rather than in `livingband` so that ONE snapshot
/// covers every knob a test can force. Its value does not fit
/// [`RenderOverrides`] — the parsed `AWL_LIVING_BAND` grammar is `livingband`'s
/// own, distinct from `AWL_OVERLAY_MOTION_FORCE`'s — so it rides alongside.
#[cfg(test)]
static LIVING_BAND_OVERRIDE: std::sync::Mutex<Option<crate::render::livingband::MotionForce>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
pub(crate) fn living_band_test_override() -> Option<crate::render::livingband::MotionForce> {
    *LIVING_BAND_OVERRIDE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
pub(crate) fn set_living_band_test_override(m: Option<crate::render::livingband::MotionForce>) {
    assert_writer_serialized();
    *LIVING_BAND_OVERRIDE
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = m;
}

/// Everything a test can force about the render: the ten consolidated knobs
/// plus the living-band probe above.
///
/// `crate::testlock::serial()` snapshots this on entry and restores it on exit,
/// so a forced knob cannot outlive the window that forced it — *including when
/// that window unwinds*, which is the case a reset at the end of a test body
/// cannot cover. Without the restore, a fixture that forces a knob and then dies
/// poisons whatever the harness schedules next, and the victim is a different
/// test in a different file — visible only under a wide `--test-threads`.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OverridePins {
    knobs: RenderOverrides,
    living_band: Option<crate::render::livingband::MotionForce>,
}

#[cfg(test)]
impl OverridePins {
    /// Nothing forced at all — what every test window is entitled to start
    /// from, now that the guard restores.
    pub(crate) fn none() -> OverridePins {
        OverridePins {
            knobs: RenderOverrides::default(),
            living_band: None,
        }
    }
}

/// Read every forced knob, for the guard's entry snapshot.
#[cfg(test)]
pub(crate) fn pins() -> OverridePins {
    OverridePins {
        knobs: TEST_OVERRIDE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
        living_band: living_band_test_override(),
    }
}

/// Put every forced knob back the way [`pins`] found it. Routed through the
/// ordinary asserting writers — the guard still holds the lock when it runs
/// this, so there is exactly one write door, not a private back one.
#[cfg(test)]
pub(crate) fn restore_pins(p: &OverridePins) {
    set_test_override(p.knobs.clone());
    set_living_band_test_override(p.living_band);
}

/// Name every knob whose value differs, `before -> after`. Both sides are
/// destructured exhaustively, so a new [`RenderOverrides`] field must be listed
/// here consciously and cannot dodge the sweep by defaulting to "unchanged".
#[cfg(test)]
pub(crate) fn leaked_knobs(before: &OverridePins, after: &OverridePins) -> Vec<String> {
    let OverridePins {
        knobs:
            RenderOverrides {
                title_style: b_title_style,
                card_anchor: b_card_anchor,
                chrome_face: b_chrome_face,
                motion_juice: b_motion_juice,
                slant: b_slant,
                list_style: b_list_style,
                facet_style: b_facet_style,
                pane_split: b_pane_split,
                density: b_density,
                overlay_motion: b_overlay_motion,
            },
        living_band: b_living_band,
    } = before;
    let OverridePins {
        knobs:
            RenderOverrides {
                title_style: a_title_style,
                card_anchor: a_card_anchor,
                chrome_face: a_chrome_face,
                motion_juice: a_motion_juice,
                slant: a_slant,
                list_style: a_list_style,
                facet_style: a_facet_style,
                pane_split: a_pane_split,
                density: a_density,
                overlay_motion: a_overlay_motion,
            },
        living_band: a_living_band,
    } = after;

    let mut leaked = Vec::new();
    macro_rules! knob {
        ($name:literal, $b:ident, $a:ident) => {
            if $b != $a {
                leaked.push(format!("{}: {:?} -> {:?}", $name, $b, $a));
            }
        };
    }
    knob!("title_style", b_title_style, a_title_style);
    knob!("card_anchor", b_card_anchor, a_card_anchor);
    knob!("chrome_face", b_chrome_face, a_chrome_face);
    knob!("motion_juice", b_motion_juice, a_motion_juice);
    knob!("slant", b_slant, a_slant);
    knob!("list_style", b_list_style, a_list_style);
    knob!("facet_style", b_facet_style, a_facet_style);
    knob!("pane_split", b_pane_split, a_pane_split);
    knob!("density", b_density, a_density);
    knob!("overlay_motion", b_overlay_motion, a_overlay_motion);
    knob!("living_band", b_living_band, a_living_band);
    leaked
}

// The ten legacy per-knob setters, kept so the ~200 existing test call sites
// (`crate::render::set_card_anchor_test_override(..)` etc.) need no edits.

#[cfg(test)]
pub(crate) fn set_title_style_test_override(style: Option<theme::TitleStyle>) {
    set_field(|o| o.title_style = style);
}

#[cfg(test)]
pub(crate) fn set_card_anchor_test_override(anchor: Option<theme::CardAnchor>) {
    set_field(|o| o.card_anchor = anchor);
}

#[cfg(test)]
pub(crate) fn set_chrome_face_test_override(face: Option<theme::ChromeFace>) {
    set_field(|o| o.chrome_face = face);
}

#[cfg(test)]
pub(crate) fn set_motion_test_override(m: Option<theme::MotionJuice>) {
    set_field(|o| o.motion_juice = m);
}

#[cfg(test)]
pub(crate) fn set_slant_test_override(s: Option<SlantProbe>) {
    set_field(|o| o.slant = s);
}

#[cfg(test)]
pub(crate) fn set_list_style_test_override(s: Option<theme::ListStyle>) {
    set_field(|o| o.list_style = s);
}

#[cfg(test)]
pub(crate) fn set_facet_style_test_override(s: Option<theme::FacetStyle>) {
    set_field(|o| o.facet_style = s);
}

#[cfg(test)]
pub(crate) fn set_pane_split_test_override(s: Option<theme::PaneSplit>) {
    set_field(|o| o.pane_split = s);
}

#[cfg(test)]
pub(crate) fn set_overlay_density_test_override(d: Option<TypeDensity>) {
    set_field(|o| o.density = d);
}

#[cfg(test)]
pub(crate) fn set_overlay_motion_test_override(m: Option<OverlayMotionProbe>) {
    set_field(|o| o.overlay_motion = m);
}
