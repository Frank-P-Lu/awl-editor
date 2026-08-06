//! Value parsers for the `AWL_*_FORCE` dev-only render/theme override knobs
//! (see the parent module's doc): one parser per knob, plus the small shared
//! grammar helpers (`ForcedKnob`, `classify_forced_knob`, `read_forced_knob`)
//! that give an unrecognized value a warn-and-fall-through instead of a
//! silent misconfiguration.

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

pub(super) fn parse_chrome_face_force(s: &str) -> Option<theme::ChromeFace> {
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

/// Each successive row's draw origin steps `px_per_row` further in from the
/// content band's own edge; `italic` also requests an italic row style. No
/// `RenderCaps` field — an env-gated layout variant, not a shipped world
/// option.
///
/// SIGNED (item 131a): the sign picks WHICH edge steps. Positive walks the
/// row's LEFT edge in while the right edge stays flush with the band (the
/// shape Mangrove's descending `\` composition needs — the row's own `dx`).
/// Negative walks the RIGHT edge in while the left edge stays flush (Magpie's
/// ascending `/` composition, whose clusters are right-aligned — the row's
/// own `dw`). `plan::overlay_rows::plan_overlay_rows` is the one place that
/// splits the sign into the two-sided `PlannedRow::{dx,dw}` extent; nothing
/// upstream of it needs to know both shapes exist.
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
    // `0.0` stays rejected (an explicit "no stagger" is `AWL_OVERLAY_SLANT_FORCE`
    // simply unset, not a zero step); either sign of a genuinely nonzero, finite
    // step now selects one of the two mirrored compositions (see the struct doc).
    if px != 0.0 && px.is_finite() {
        Some(SlantProbe {
            px_per_row: px,
            italic,
        })
    } else {
        None
    }
}

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
pub(super) fn read_forced_knob<T>(
    var: &str,
    grammar: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    read_forced_knob_from(var, std::env::var(var).ok().as_deref(), grammar, parse)
}

/// Like [`read_forced_knob`] but takes an already-fetched raw value, for a
/// var that resolves more than one field — `AWL_OVERLAY_LIST_FORCE` backs
/// both `list_style` and `bar_config`, since `ListStyle::Bars` carries no
/// fields of its own — so the environment is read exactly once per var, at
/// the one call site `render_overrides_env_read_law` checks.
pub(super) fn read_forced_knob_from<T>(
    var: &str,
    raw: Option<&str>,
    grammar: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    match classify_forced_knob(raw, &parse) {
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

/// `AWL_OVERLAY_LIST_FORCE` grammar: `"pane"`, bare `"bars"`, or `"bars:"`
/// followed by `:`-separated layout tokens (see [`parse_bar_config_force`]).
/// This resolves ONLY which [`theme::ListStyle`] variant is forced — `Bars`
/// carries no fields of its own any more, so the layout tokens (if any) are a
/// second, independent override ([`parse_bar_config_force`]) rather than part
/// of this value. A malformed `"bars:"` suffix rejects the WHOLE force (falls
/// through to the world default), matching the un-split grammar's old
/// behavior: `parse_bar_config_force` is the one parser for that suffix, so
/// asking it to validate here can't drift from what it accepts when read for
/// real.
///
/// `Rules` carries its one field — the selection treatment — in the value
/// itself (`"rules:weight"` / `"rules:gutter"`), because that field is the
/// open taste question the style exists to ask, and forcing it is how the two
/// answers get captured side by side against one world. Bare `"rules"` takes
/// [`RULE_SELECTION_DEFAULT`].
pub(crate) fn parse_list_style_force(s: &str) -> Option<theme::ListStyle> {
    let low = s.trim().to_ascii_lowercase();
    match low.as_str() {
        "pane" => Some(theme::ListStyle::Pane),
        "bars" => Some(theme::ListStyle::Bars),
        "rules" => Some(theme::ListStyle::Rules(RULE_SELECTION_DEFAULT)),
        "rules:weight" => Some(theme::ListStyle::Rules(theme::RuleSelection::Weight)),
        "rules:gutter" => Some(theme::ListStyle::Rules(theme::RuleSelection::Gutter)),
        _ if low.starts_with("bars:") => {
            parse_bar_config_force(&low)?;
            Some(theme::ListStyle::Bars)
        }
        _ => None,
    }
}

/// What bare `"rules"` means. Not a renderer default — the shipped carrier
/// world names its own treatment in its `RenderCaps` — only the force knob's.
const RULE_SELECTION_DEFAULT: theme::RuleSelection = theme::RuleSelection::Weight;

/// The `"bars:"` suffix of `AWL_OVERLAY_LIST_FORCE`: up to 3 non-negative
/// floats (radius, gap, grow, positional) plus extent keywords (`full` |
/// `hug` | `huglabel`/`hybrid`) and coverage keywords (`all` | `selected`),
/// any order, layered over [`theme::BarConfig::SHIPPED`]. `None` for `"pane"`
/// or bare `"bars"` — no override, so [`theme::BarConfig::SHIPPED`] applies —
/// and for a 4th float, an unknown token, or a negative/non-finite float.
pub(crate) fn parse_bar_config_force(s: &str) -> Option<theme::BarConfig> {
    let low = s.trim().to_ascii_lowercase();
    let rest = low.strip_prefix("bars:")?;
    let mut cfg = theme::BarConfig::SHIPPED;
    let mut floats_seen = 0usize;
    for tok in rest.split(':') {
        let tok = tok.trim();
        match tok {
            "full" => cfg.extent = theme::BarExtent::FullWidth,
            "hug" => cfg.extent = theme::BarExtent::HugText,
            "huglabel" | "hybrid" => cfg.extent = theme::BarExtent::HugLabel,
            "all" => cfg.coverage = theme::BarCoverage::All,
            "selected" => cfg.coverage = theme::BarCoverage::SelectedOnly,
            _ => {
                let v: f32 = tok.parse().ok()?;
                if !v.is_finite() || v < 0.0 {
                    return None;
                }
                match floats_seen {
                    0 => cfg.radius = v,
                    1 => cfg.gap = v,
                    2 => cfg.grow_px = v,
                    _ => return None, // a fourth float is malformed
                }
                floats_seen += 1;
            }
        }
    }
    Some(cfg)
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
            scale: crate::render::chrome::OVERLAY_UI_SCALE,
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
