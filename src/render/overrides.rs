//! src/render/overrides.rs — the ten `AWL_*_FORCE` dev-only render/theme
//! override knobs, consolidated into ONE struct with ONE env-reading owner.
//!
//! [`RenderOverrides::from_env`] is the ONLY place these env vars are read
//! ([`render_overrides_env_read_law`] enforces it by source scan). Every
//! reader resolves through [`current`]; production sources one value at
//! startup, `#[cfg(test)]` code can override it via [`set_test_override`] or
//! the ten legacy `set_*_test_override` wrappers (kept so existing test call
//! sites need no edits). One `#[cfg(test)]` `Mutex` remains as the test
//! bypass — not zero — because these readers are called from ~200 sites with
//! no shared owner object to thread a parameter through (see the commit
//! message for the full tradeoff). It is NOT testlock-guarded — see
//! [`TEST_OVERRIDE`]'s doc for why a uniform guard was tried and reverted.

use crate::theme;

// ---------------------------------------------------------------------------
// PLACARD (title style)
// ---------------------------------------------------------------------------

pub(crate) fn parse_overlay_style_force(s: &str) -> Option<theme::TitleStyle> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("inline") {
        return Some(theme::TitleStyle::InlinePrefix);
    }
    let mut parts = s.split(':');
    if !parts.next()?.eq_ignore_ascii_case("placard") {
        return None;
    }
    let corner = match parts.next()?.to_ascii_uppercase().as_str() {
        "TL" => theme::PlacardCorner::TL,
        "TR" => theme::PlacardCorner::TR,
        "BL" => theme::PlacardCorner::BL,
        "BR" => theme::PlacardCorner::BR,
        _ => return None,
    };
    let scale: f32 = parts.next()?.parse().ok()?;
    let ink = match parts.next()?.to_ascii_lowercase().as_str() {
        "faint" => theme::PlacardInk::Faint,
        "ghost" => theme::PlacardInk::Ghost,
        "stipple" => theme::PlacardInk::Stipple,
        "muted" => theme::PlacardInk::Muted,
        "bold" => theme::PlacardInk::Bold,
        _ => return None,
    };
    if parts.next().is_some() {
        return None; // trailing garbage — reject rather than silently ignore
    }
    Some(theme::TitleStyle::Placard { corner, scale, ink })
}

// ---------------------------------------------------------------------------
// CARD ANCHOR (two env vars, one resolved field — `AWL_OVERLAY_ALIGN` wins)
// ---------------------------------------------------------------------------

pub(crate) fn parse_overlay_anchor_force(s: &str) -> Option<theme::CardAnchor> {
    let s = s.trim();
    if let Some(rest) = s
        .strip_prefix("inset:")
        .or_else(|| s.strip_prefix("Inset:"))
        .or_else(|| s.strip_prefix("INSET:"))
    {
        let frac: f32 = rest.trim().parse().ok()?;
        if (0.0..=1.0).contains(&frac) {
            return Some(theme::CardAnchor::Inset { x_frac: frac });
        }
        return None;
    }
    match s.to_ascii_lowercase().as_str() {
        "tl" | "topleft" | "left" => Some(theme::CardAnchor::TopLeft),
        "tc" | "topcenter" | "center" | "centre" => Some(theme::CardAnchor::TopCenter),
        "tr" | "topright" | "right" | "mirror" => Some(theme::CardAnchor::TopRight),
        _ => None,
    }
}

pub(crate) fn parse_overlay_align(s: &str) -> Option<theme::CardAnchor> {
    match s.trim().to_ascii_lowercase().as_str() {
        "left" | "l" => Some(theme::CardAnchor::TopLeft),
        "center" | "centre" | "c" => Some(theme::CardAnchor::TopCenter),
        "right" | "r" => Some(theme::CardAnchor::TopRight),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// CHROME FACE
// ---------------------------------------------------------------------------

fn parse_chrome_face_force(s: &str) -> Option<theme::ChromeFace> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    Some(theme::ChromeFace::Named(Box::leak(
        s.to_string().into_boxed_str(),
    )))
}

// ---------------------------------------------------------------------------
// MOTION JUICE
// ---------------------------------------------------------------------------

pub(crate) fn parse_motion_force(s: &str) -> Option<theme::MotionJuice> {
    let (mut entrance, mut band) = (theme::OverlayEntrance::Instant, theme::BandResponse::Snap);
    match s.trim().to_ascii_lowercase().as_str() {
        "off" | "calm" => {}
        "spring" => entrance = theme::OverlayEntrance::SpringIn,
        "slide" => band = theme::BandResponse::Slide,
        "spring:slide" | "slide:spring" | "full" | "on" => {
            entrance = theme::OverlayEntrance::SpringIn;
            band = theme::BandResponse::Slide;
        }
        _ => return None,
    }
    Some(theme::MotionJuice { entrance, band })
}

// ---------------------------------------------------------------------------
// WILD-MENU SLANT PROBE
// ---------------------------------------------------------------------------

/// Each successive row's draw origin steps `px_per_row` further right;
/// `italic` also requests an italic row style. No `RenderCaps` field — an
/// env-gated layout variant, not a shipped world option.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SlantProbe {
    pub px_per_row: f32,
    pub italic: bool,
}

pub(crate) fn parse_overlay_slant_force(s: &str) -> Option<SlantProbe> {
    let s = s.trim();
    let (px_s, italic) = match s.split_once(':') {
        Some((px, flag)) if flag.trim().eq_ignore_ascii_case("italic") => (px, true),
        Some(_) => return None,
        None => (s, false),
    };
    let px: f32 = px_s.trim().parse().ok()?;
    if px > 0.0 && px.is_finite() {
        Some(SlantProbe {
            px_per_row: px,
            italic,
        })
    } else {
        None
    }
}

const BARS_DEFAULT_RADIUS: f32 = 6.0;
const BARS_DEFAULT_GAP: f32 = 10.0;
const BARS_DEFAULT_GROW: f32 = 24.0;
const BARS_DEFAULT_EXTENT: theme::BarExtent = theme::BarExtent::FullWidth;
const BARS_DEFAULT_COVERAGE: theme::BarCoverage = theme::BarCoverage::All;

#[derive(Debug)]
pub(crate) enum ForcedKnob<T> {
    Unset,
    Parsed(T),
    Retired,
}

pub(crate) fn classify_forced_knob<T>(
    raw: Option<&str>,
    parse: impl Fn(&str) -> Option<T>,
) -> ForcedKnob<T> {
    match raw {
        None => ForcedKnob::Unset,
        Some(s) => match parse(s) {
            Some(v) => ForcedKnob::Parsed(v),
            None => ForcedKnob::Retired,
        },
    }
}

/// Read an `AWL_*_FORCE` dev knob: unset is silent, a recognized value forces
/// the render, an unrecognized one warns to stderr and falls back to default.
fn read_forced_knob<T>(var: &str, grammar: &str, parse: impl Fn(&str) -> Option<T>) -> Option<T> {
    let raw = std::env::var(var).ok();
    match classify_forced_knob(raw.as_deref(), &parse) {
        ForcedKnob::Parsed(v) => Some(v),
        ForcedKnob::Unset => None,
        ForcedKnob::Retired => {
            eprintln!(
                "awl: {var}={:?} is not a recognized value ({grammar}); using the world default",
                raw.unwrap_or_default()
            );
            None
        }
    }
}

// ---------------------------------------------------------------------------
// LIST STYLE
// ---------------------------------------------------------------------------

/// `AWL_OVERLAY_LIST_FORCE` grammar: `"pane"`, bare `"bars"` (default radius/
/// gap/grow/extent/coverage), or `"bars:"` followed by `:`-separated tokens —
/// up to 3 non-negative floats (radius, gap, grow, positional) plus extent
/// keywords (`full` | `hug` | `huglabel`/`hybrid`) and coverage keywords
/// (`all` | `selected`), any order. A 4th float, unknown token, or
/// negative/non-finite float → `None` (falls through to the world default).
pub(crate) fn parse_list_style_force(s: &str) -> Option<theme::ListStyle> {
    let low = s.trim().to_ascii_lowercase();
    if low == "pane" {
        return Some(theme::ListStyle::Pane);
    }
    let rest = if low == "bars" {
        ""
    } else {
        low.strip_prefix("bars:")?
    };
    let mut radius = BARS_DEFAULT_RADIUS;
    let mut gap = BARS_DEFAULT_GAP;
    let mut grow_px = BARS_DEFAULT_GROW;
    let mut extent = BARS_DEFAULT_EXTENT;
    let mut coverage = BARS_DEFAULT_COVERAGE;
    let mut floats_seen = 0usize;
    if !rest.is_empty() {
        for tok in rest.split(':') {
            let tok = tok.trim();
            match tok {
                "full" => extent = theme::BarExtent::FullWidth,
                "hug" => extent = theme::BarExtent::HugText,
                "huglabel" | "hybrid" => extent = theme::BarExtent::HugLabel,
                "all" => coverage = theme::BarCoverage::All,
                "selected" => coverage = theme::BarCoverage::SelectedOnly,
                _ => {
                    let v: f32 = tok.parse().ok()?;
                    if !v.is_finite() || v < 0.0 {
                        return None;
                    }
                    match floats_seen {
                        0 => radius = v,
                        1 => gap = v,
                        2 => grow_px = v,
                        _ => return None, // a fourth float is malformed
                    }
                    floats_seen += 1;
                }
            }
        }
    }
    Some(theme::ListStyle::Bars {
        radius,
        gap,
        grow_px,
        extent,
        coverage,
    })
}

// ---------------------------------------------------------------------------
// FACET STYLE
// ---------------------------------------------------------------------------

pub(crate) fn parse_facet_style_force(s: &str) -> Option<theme::FacetStyle> {
    let low = s.trim().to_ascii_lowercase();
    match low.as_str() {
        "text" => return Some(theme::FacetStyle::Text),
        "band" => return Some(theme::FacetStyle::Band),
        "chips" => return Some(theme::FacetStyle::Chips(theme::ChipVariant::Hairline)),
        _ => {}
    }
    let variant = low.strip_prefix("chips:")?;
    let v = match variant {
        "hairline" => theme::ChipVariant::Hairline,
        "filled" | "filledactive" => theme::ChipVariant::FilledActive,
        "underline" => theme::ChipVariant::Underline,
        "bracket" => theme::ChipVariant::Bracket,
        _ => return None,
    };
    Some(theme::FacetStyle::Chips(v))
}

// ---------------------------------------------------------------------------
// PANE SPLIT
// ---------------------------------------------------------------------------

pub(crate) fn parse_pane_split_force(s: &str) -> Option<theme::PaneSplit> {
    match s.trim().to_ascii_lowercase().as_str() {
        "unified" => Some(theme::PaneSplit::Unified),
        "split" => Some(theme::PaneSplit::Split),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// OVERLAY TYPE DENSITY + MOTION FRAME-DUMP PROBES
// ---------------------------------------------------------------------------

/// The whole-menu UI `scale` and extra `leading` (device px). No `RenderCaps`
/// field — probe-only until a gallery win.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TypeDensity {
    pub scale: f32,
    pub leading: f32,
}

impl TypeDensity {
    pub(crate) fn shipped() -> Self {
        TypeDensity {
            scale: super::chrome::OVERLAY_UI_SCALE,
            leading: 0.0,
        }
    }
}

pub(crate) fn parse_overlay_density_force(s: &str) -> Option<TypeDensity> {
    let s = s.trim();
    let (scale_s, leading) = match s.split_once(':') {
        Some((sc, ld)) => {
            let ld: f32 = ld.trim().parse().ok()?;
            if !ld.is_finite() || ld < 0.0 {
                return None;
            }
            (sc, ld)
        }
        None => (s, 0.0),
    };
    let scale: f32 = scale_s.trim().parse().ok()?;
    if scale.is_finite() && scale > 0.0 {
        Some(TypeDensity { scale, leading })
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OverlayMotionProbe {
    pub enter: f32,
    pub band: f32,
}

pub(crate) fn parse_overlay_motion_force(s: &str) -> Option<OverlayMotionProbe> {
    let s = s.trim();
    let (enter_s, band_s) = match s.split_once(':') {
        Some((e, b)) => (e, Some(b)),
        None => (s, None),
    };
    let enter: f32 = enter_s.trim().parse().ok()?;
    if !enter.is_finite() {
        return None;
    }
    let band: f32 = match band_s {
        Some(b) => {
            let b: f32 = b.trim().parse().ok()?;
            if !b.is_finite() {
                return None;
            }
            b
        }
        None => enter,
    };
    Some(OverlayMotionProbe {
        enter: enter.clamp(0.0, 1.0),
        band: band.clamp(0.0, 1.0),
    })
}

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
/// NOT testlock-guarded, matching nine of the ten predecessor statics this
/// module replaced (only `LIST_STYLE_TEST_OVERRIDE` asserted
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_test_override_installs_a_whole_struct_directly() {
        let _g = crate::testlock::serial();
        set_test_override(RenderOverrides {
            card_anchor: Some(theme::CardAnchor::TopRight),
            list_style: Some(theme::ListStyle::Pane),
            ..Default::default()
        });
        assert_eq!(current().card_anchor, Some(theme::CardAnchor::TopRight));
        assert_eq!(current().list_style, Some(theme::ListStyle::Pane));
        // A reset override must not leak the previous struct's field into a
        // later reader.
        set_test_override(RenderOverrides::default());
        assert_eq!(current().card_anchor, None);
        assert_eq!(current().list_style, None);
    }

    // LAW: these env vars are read from exactly ONE place — `from_env`.
    const KNOB_ENV_VARS: &[&str] = &[
        "AWL_OVERLAY_STYLE_FORCE",
        "AWL_OVERLAY_ALIGN",
        "AWL_OVERLAY_ANCHOR_FORCE",
        "AWL_CHROME_FACE_FORCE",
        "AWL_MOTION_FORCE",
        "AWL_OVERLAY_SLANT_FORCE",
        "AWL_OVERLAY_LIST_FORCE",
        "AWL_FACET_STYLE_FORCE",
        "AWL_PANE_SPLIT_FORCE",
        "AWL_OVERLAY_DENSITY_FORCE",
        "AWL_OVERLAY_MOTION_FORCE",
    ];

    /// The one file allowed to name these vars.
    const OWNER: &str = "overrides.rs";

    fn scan_dir(dir: &std::path::Path, out: &mut Vec<(String, usize, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            scan_file(&path, out);
        }
    }

    /// Mirrors `println_audit::scan_file`: skips `#[cfg(test)]`-gated bodies,
    /// so this file's own `KNOB_ENV_VARS` checklist doesn't self-match.
    fn scan_file(path: &std::path::Path, out: &mut Vec<(String, usize, String)>) {
        #[derive(Clone, Copy, PartialEq)]
        enum State {
            Normal,
            AfterCfgTest,
            InSkippedBlock(i32),
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let mut state = State::Normal;
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            state = match state {
                State::Normal => {
                    if trimmed.starts_with("#[cfg(test)") || trimmed.starts_with("#[cfg(all(test") {
                        State::AfterCfgTest
                    } else {
                        if !trimmed.starts_with("//") {
                            for var in KNOB_ENV_VARS {
                                let needle = format!("\"{var}\"");
                                if line.contains(&needle) {
                                    out.push((name.clone(), i + 1, (*var).to_string()));
                                }
                            }
                        }
                        State::Normal
                    }
                }
                State::AfterCfgTest => {
                    if trimmed.starts_with("#[") {
                        State::AfterCfgTest // a stacked attribute; keep waiting
                    } else if line.contains('{') {
                        let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                        if d <= 0 {
                            State::Normal
                        } else {
                            State::InSkippedBlock(d)
                        }
                    } else if trimmed.ends_with(';') {
                        State::Normal // a bare `mod tests;` declaration
                    } else {
                        State::AfterCfgTest // a multi-line signature; keep waiting
                    }
                }
                State::InSkippedBlock(depth) => {
                    let d =
                        depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        State::Normal
                    } else {
                        State::InSkippedBlock(d)
                    }
                }
            };
        }
    }

    #[test]
    fn render_overrides_env_read_law() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut hits = Vec::new();
        scan_dir(&root, &mut hits);

        let stray: Vec<_> = hits.iter().filter(|(f, _, _)| f != OWNER).collect();
        assert!(
            stray.is_empty(),
            "these AWL_*_FORCE/AWL_OVERLAY_ALIGN knobs must be read ONLY by \
             `RenderOverrides::from_env` in `{OWNER}` — a second read site is exactly \
             the two-code-paths-to-one-setting bug this module retired. offending lines:\n{}",
            stray
                .iter()
                .map(|(f, l, v)| format!("  {f}:{l}  ({v})"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Non-vacuous: all eleven var names (ten knobs; `card_anchor` reads
        // two) must actually be found in `from_env`, or this count drops.
        let owner_hits = hits.iter().filter(|(f, _, _)| f == OWNER).count();
        assert_eq!(
            owner_hits,
            KNOB_ENV_VARS.len(),
            "expected exactly one `from_env` read site per knob env var in \
             `{OWNER}`; found {owner_hits}"
        );
    }

    #[test]
    fn scan_file_skips_comment_lines() {
        let dir = std::env::temp_dir().join(format!(
            "awl_render_overrides_law_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("fixture.rs");
        std::fs::write(
            &path,
            "// mentions \"AWL_MOTION_FORCE\" in prose, not code\nfn f() {}\n",
        )
        .unwrap();
        let mut out = Vec::new();
        scan_file(&path, &mut out);
        assert!(
            out.is_empty(),
            "a comment line must not count as a code read"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
