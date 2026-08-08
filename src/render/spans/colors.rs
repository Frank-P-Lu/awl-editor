//! Theme-derived ink: syntax role colors, the markdown highlight wash
//! (plain and one-bit-dither), search-match color, and the strike /
//! link-underline line bands.

use super::*;

const HUE_STR: f32 = 140.0;
const HUE_DEF: f32 = 220.0;
const HUE_CONST: f32 = 290.0;
const HUE_COMMENT_WASH: f32 = 50.0;

const S_FG_DARK: f32 = 0.46;
const S_FG_LIGHT: f32 = 0.18;

const T_DARK: [f32; 3] = [0.26, 0.28, 0.44];
const T_LIGHT: [f32; 3] = [0.76, 0.78, 0.80];

const WASH_S_DARK: f32 = 0.62;
const WASH_L_DARK: f32 = 0.66;
const WASH_ALPHA_DARK: u8 = 0x2A;
const WASH_S_LIGHT: f32 = 0.55;
const WASH_L_LIGHT: f32 = 0.50;
const WASH_ALPHA_LIGHT: u8 = 0x2E;

const HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY: f32 = 165.0;
const HIGHLIGHT_S_DARK: f32 = 0.58;
const HIGHLIGHT_L_DARK: f32 = 0.64;
const HIGHLIGHT_ALPHA_DARK: u8 = 0x3A;
const HIGHLIGHT_S_LIGHT: f32 = 0.50;
const HIGHLIGHT_L_LIGHT: f32 = 0.58;
const HIGHLIGHT_ALPHA_LIGHT: u8 = 0x4D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) struct RoleStyle {
    pub fg: theme::Srgb,
    pub wash: Option<theme::Srgb>,
}

pub(in crate::render) fn role_style_for(
    th: &theme::Theme,
    kind: crate::syntax::SynKind,
) -> RoleStyle {
    use crate::syntax::SynKind;
    let ov = th.role_overrides;
    let (_, _, l_full) = th.base_content.to_hsl();
    let (_, _, l_dim) = th.muted.to_hsl();
    let (t, s_fg) = if th.dark {
        (T_DARK, S_FG_DARK)
    } else {
        (T_LIGHT, S_FG_LIGHT)
    };
    let fg_at =
        |anchor: f32, ti: f32| theme::Srgb::from_hsl(anchor, s_fg, l_full + (l_dim - l_full) * ti);
    let derived_wash = |anchor: f32| {
        if th.dark {
            let c = theme::Srgb::from_hsl(anchor, WASH_S_DARK, WASH_L_DARK);
            theme::Srgb::rgba(c.r, c.g, c.b, WASH_ALPHA_DARK)
        } else {
            let c = theme::Srgb::from_hsl(HUE_COMMENT_WASH, WASH_S_LIGHT, WASH_L_LIGHT);
            theme::Srgb::rgba(c.r, c.g, c.b, WASH_ALPHA_LIGHT)
        }
    };
    let with_override = |derived: Option<theme::Srgb>, ov: theme::WashOverride| match ov {
        theme::WashOverride::Default => derived,
        theme::WashOverride::Off => None,
        theme::WashOverride::Pin(c) => Some(c),
    };
    match kind {
        // PROSE comments are PROMINENT (decision: comments are the prose in the
        // code): FULL content ink + the warm wash carrying the comment identity.
        SynKind::Comment => RoleStyle {
            fg: th.base_content,
            wash: with_override(Some(derived_wash(HUE_COMMENT_WASH)), ov.comment_wash),
        },
        SynKind::CommentCode => RoleStyle {
            fg: th.muted,
            wash: None,
        },
        SynKind::Definition => RoleStyle {
            fg: ov.def_fg.unwrap_or_else(|| fg_at(HUE_DEF, t[0])),
            wash: None,
        },
        SynKind::Constant => RoleStyle {
            fg: ov.const_fg.unwrap_or_else(|| fg_at(HUE_CONST, t[1])),
            wash: None,
        },
        // Strings: green fg tint everywhere; the green wash only on DARK worlds
        // (light worlds carry string identity in the fg tint alone).
        SynKind::Str => RoleStyle {
            fg: ov.str_fg.unwrap_or_else(|| fg_at(HUE_STR, t[2])),
            wash: with_override(
                if th.dark {
                    Some(derived_wash(HUE_STR))
                } else {
                    None
                },
                ov.str_wash,
            ),
        },
    }
}

pub(in crate::render) fn wash_rgba_bytes(kind: crate::syntax::SynKind) -> [u8; 4] {
    role_style_for(&theme::active(), kind)
        .wash
        .unwrap_or(theme::Srgb::rgba(0, 0, 0, 0))
        .rgba_bytes()
}

/// The DEDICATED markdown `==highlight==` wash quad color for a world — its hue
/// DERIVED from the world's OWN accent (`hue(primary) +
/// HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY`, a split-complementary), with the presence
/// (saturation / lightness / alpha) split per light/dark class. Decoupled from
/// the warm comment wash so a highlighter POPS while comments stay a subtle prose
/// whisper, but — unlike the retired fixed violet — now reads as NATIVE to each
/// world (see the `HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY` doc above for the "why" and
/// the sweep that picked 165°). A PURE function of the passed theme (its
/// `primary` hue + `dark` flag), so the law test can sweep every world lock-free.
/// Every world carries it (no override hatch in v1 — unlike the syntax washes, a
/// highlight is never opted out).
///
/// **MONOCHROME WORLDS (`Theme::is_monochrome`, THEMES.md's logged DESIGN.md
/// §3 "no warm thing" amendment):** an achromatic `primary` has NO hue to
/// rotate — `hue(primary)` is a meaningless `0.0` for a plain grey (see
/// `Srgb::to_hsl`'s achromatic case), so deriving a highlight hue from it would
/// silently produce a color the world otherwise renders none of. Forced to
/// saturation `0.0` instead: the highlight becomes a pure VALUE-STEP wash — the
/// same "no hue, only lightness" idiom the WYSIWYG panel/pill already use — at
/// the SAME per-mode `l`/`alpha` every other world's highlight uses, so it still
/// pops exactly as loud, just without a hue to pop WITH.
///
/// **TRUE 1-BIT WORLDS (`Theme::is_monochrome` is the general case;
/// `Theme::is_one_bit` — Wagtail's 2026-07 rework — is the stricter one):**
/// the monochrome branch above still leaves a MID-LIGHTNESS grey wash
/// (`HIGHLIGHT_L_DARK`/`_LIGHT` sit well short of 0.0/1.0), which is exactly
/// the kind of authored grey a 1-bit world forbids outright.
///
/// **THE DITHER ROUND (supersedes the old "fully OFF" answer):** a 1-bit
/// world no longer drops the highlight wash to `alpha = 0` — it routes
/// through **THE ONE WAGTAIL HIGHLIGHT TEXTURE** instead (the user's razor:
/// one kind of emphasis, one texture — see THEMES.md's 1-bit section), a
/// deterministic Bayer-ordered dither stipple (`shaders/selection.wgsl`'s
/// `fs_main` dither branch, density `render::dither::
/// WAGTAIL_HIGHLIGHT_DITHER_DENSITY`) that is EVERY pixel either pure quad
/// color at full opacity or fully transparent — never a fractional alpha, so
/// it never composites a forbidden grey the way the old flat-alpha wash
/// would have. This function's job for a one-bit world simplifies to naming
/// the dither's ONE color: pure opaque white (the token
/// [`highlight_wash_rgba_bytes`] feeds the pipeline; the DENSITY that turns
/// dither mode on is a separate call, [`wagtail_dither_density`], applied at
/// the same construction/re-tint call sites). `==highlight==` still reads
/// structurally either way (the `==` delimiters still conceal/reveal, the
/// marked text still keeps full ink) — now it ALSO carries the dither band,
/// exactly like search matches do on a one-bit world (see
/// `wagtail_dither_density`'s doc for the "one texture, two consumers" wiring).
pub(in crate::render) fn highlight_wash(th: &theme::Theme) -> theme::Srgb {
    if let theme::HighlightTexture::Stipple { color, .. } = th.render_caps.highlight_texture {
        return theme::Srgb::rgba(color.r, color.g, color.b, 0xFF);
    }
    let (s, l, alpha) = if th.dark {
        (HIGHLIGHT_S_DARK, HIGHLIGHT_L_DARK, HIGHLIGHT_ALPHA_DARK)
    } else {
        (HIGHLIGHT_S_LIGHT, HIGHLIGHT_L_LIGHT, HIGHLIGHT_ALPHA_LIGHT)
    };
    let s = if th.is_monochrome() { 0.0 } else { s };
    let (primary_hue, _, _) = th.primary.to_hsl();
    let hue = (primary_hue + HIGHLIGHT_HUE_OFFSET_FROM_PRIMARY).rem_euclid(360.0);
    let c = theme::Srgb::from_hsl(hue, s, l);
    theme::Srgb::rgba(c.r, c.g, c.b, alpha)
}

pub(in crate::render) fn highlight_wash_rgba_bytes() -> [u8; 4] {
    highlight_wash(&theme::active()).rgba_bytes()
}

/// THE ONE WAGTAIL HIGHLIGHT TEXTURE's density switch — `0.0` (dither mode
/// OFF, every non-one-bit world) or [`dither::WAGTAIL_HIGHLIGHT_DITHER_DENSITY`]
/// (one-bit worlds). Fed into `SelectionPipeline::set_dither` at the SAME two
/// call sites [`highlight_wash_rgba_bytes`] feeds `set_color` — construction
/// AND every `sync_theme_colors` re-tint (a switch AWAY from a one-bit world
/// must reset this back to `0.0`, never merely leave it stale). The two
/// consumers this drives — `wash_highlight_pipeline` (`==highlight==` spans)
/// and `match_pipeline` (search matches) — deliberately share this ONE
/// function + density: the razor is ONE texture for ONE meaning ("something
/// here is marked"), not a per-consumer ladder.
pub(in crate::render) fn wagtail_dither_density() -> f32 {
    match theme::active().render_caps.highlight_texture {
        theme::HighlightTexture::Stipple { density, .. } => density,
        theme::HighlightTexture::Wash => 0.0,
    }
}

pub(in crate::render) fn wagtail_stipple_cell_px(dpi: f32) -> f32 {
    if wagtail_dither_density() <= 0.0 {
        return 1.0;
    }
    let logical = stipple_cell_logical_override()
        .unwrap_or(crate::render::dither::WAGTAIL_HIGHLIGHT_STIPPLE_CELL_LOGICAL);
    (logical * dpi.max(1.0)).round().max(1.0)
}

fn stipple_cell_logical_override() -> Option<f32> {
    static ONCE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_STIPPLE_CELL")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|c| *c >= 1.0 && c.is_finite())
    })
}

/// The ACTIVE world's SEARCH-MATCH quad rgba — `theme::selection_document()` on every
/// ordinary world (unchanged), but on a one-bit world this NO LONGER shares
/// the (now true-inverse-video) document-selection token: it instead reads
/// pure opaque white, the SAME single color [`highlight_wash_rgba_bytes`]
/// feeds the dither pipeline, since a one-bit search match renders through
/// THE ONE WAGTAIL HIGHLIGHT TEXTURE too (paired with [`wagtail_dither_density`]
/// on `match_pipeline`) rather than the old solid-white/punch-outline
/// mechanism document selection used to share with it.
pub(in crate::render) fn search_match_rgba_bytes() -> [u8; 4] {
    match theme::active().render_caps.highlight_texture {
        theme::HighlightTexture::Stipple { color, .. } => {
            theme::Srgb::rgba(color.r, color.g, color.b, 0xFF).rgba_bytes()
        }
        theme::HighlightTexture::Wash => theme::selection_document().rgba_bytes(),
    }
}

/// The strike line's stroke weight — a LENGTH, so it meets the same display scale
/// as the glyph cell it crosses. It rode `zoom` alone at all three of its read
/// sites, so a struck phrase kept a hairline stroke beside doubled text.
pub(in crate::render) const STRIKE_THICKNESS: crate::render::Logical = crate::render::Logical(1.3);

pub(in crate::render) const STRIKE_V_FRAC: f32 = 0.5;

/// `scale` is `Metrics::scale` — `zoom * dpi`, never the bare zoom. The band's own
/// PADDING stays physical on purpose: it is the rasterizer's feather either side
/// of the stroke, a device-grid quantity like `menubar::EDGE_BLEED_PX`, not a
/// tuned distance that should grow with the panel.
fn line_band(
    top: f32,
    height: f32,
    scale: f32,
    v_frac: f32,
    thickness: crate::render::Logical,
) -> (f32, f32, f32) {
    let stroke = thickness.px(scale);
    let band_h = stroke + 2.0;
    let center = top + height * v_frac;
    (center - band_h * 0.5, band_h, stroke)
}

pub(in crate::render) fn strike_line_band(top: f32, height: f32, scale: f32) -> (f32, f32, f32) {
    line_band(top, height, scale, STRIKE_V_FRAC, STRIKE_THICKNESS)
}

pub(in crate::render) const LINK_UNDERLINE_THICKNESS: crate::render::Logical = STRIKE_THICKNESS;

pub(in crate::render) const LINK_UNDERLINE_V_FRAC: f32 = 0.92;

pub(in crate::render) fn link_underline_band(top: f32, height: f32, scale: f32) -> (f32, f32, f32) {
    line_band(
        top,
        height,
        scale,
        LINK_UNDERLINE_V_FRAC,
        LINK_UNDERLINE_THICKNESS,
    )
}

pub(in crate::render) fn strike_ink(th: &theme::Theme) -> theme::Srgb {
    th.muted
}

pub(in crate::render) fn strike_srgba_bytes() -> [u8; 4] {
    strike_ink(&theme::active()).rgba_bytes()
}

pub(in crate::render) fn link_underline_ink(th: &theme::Theme) -> theme::Srgb {
    strike_ink(th)
}

pub(in crate::render) fn link_underline_srgba_bytes() -> [u8; 4] {
    link_underline_ink(&theme::active()).rgba_bytes()
}
