//! Disposable Paperbark material study for the item-133 user-choice artifact.
//!
//! This module deliberately does not add a world to `theme::THEMES`. Each capture
//! is a fresh process with `AWL_PAPERBARK_TRIAL=A|B|C|D|E`; without that variable
//! every code path is inert. The selected profile changes only the background
//! shader selector. Palette, type, caret, page, document, and chrome all come
//! from the one provisional [`PAPERBARK_THEME`] value.

use std::sync::OnceLock;

use crate::background::BgDesc;
use crate::theme::{
    Background, CJK_JA_SHIPPORI, CJK_KO_SERIF, CJK_ZH_HANS_SERIF, CJK_ZH_HANT, IconCursor,
    LIST_INDENT_SCALE_WIDE, ORNAMENT_GARAMOND, ORNAMENT_SCALE_FLEURON, Ornaments, RenderCaps,
    RoleOverrides, Srgb, Theme, ThemeTags,
};

pub const ENV: &str = "AWL_PAPERBARK_TRIAL";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    BroadSheets,
    DeckledStrata,
    LooseFibres,
    ReliefPrint,
    PeelingCurls,
}

impl Profile {
    pub const ALL: [Profile; 5] = [
        Profile::BroadSheets,
        Profile::DeckledStrata,
        Profile::LooseFibres,
        Profile::ReliefPrint,
        Profile::PeelingCurls,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Profile::BroadSheets => "A",
            Profile::DeckledStrata => "B",
            Profile::LooseFibres => "C",
            Profile::ReliefPrint => "D",
            Profile::PeelingCurls => "E",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Profile::BroadSheets => "broad-sheets",
            Profile::DeckledStrata => "deckled-strata",
            Profile::LooseFibres => "loose-fibres",
            Profile::ReliefPrint => "relief-print",
            Profile::PeelingCurls => "peeling-curls",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Profile::BroadSheets => "Broad sheets",
            Profile::DeckledStrata => "Deckled strata",
            Profile::LooseFibres => "Loose fibres",
            Profile::ReliefPrint => "Relief print",
            Profile::PeelingCurls => "Peeling curls",
        }
    }

    pub const fn shader_index(self) -> f32 {
        match self {
            Profile::BroadSheets => 0.0,
            Profile::DeckledStrata => 1.0,
            Profile::LooseFibres => 2.0,
            Profile::ReliefPrint => 3.0,
            Profile::PeelingCurls => 4.0,
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::ALL.into_iter().find(|profile| {
            raw.eq_ignore_ascii_case(profile.id()) || raw.eq_ignore_ascii_case(profile.slug())
        })
    }
}

/// Three fixed material-ground tones uploaded to the disposable shader.
pub const GROUND: Srgb = Srgb::rgb(0xF0, 0xDF, 0xBA);
pub const LAYER: Srgb = Srgb::rgb(0xD8, 0xB7, 0x7A);
pub const SHADOW: Srgb = Srgb::rgb(0x9F, 0x69, 0x37);

/// One provisional world. It is intentionally absent from `theme::THEMES`.
pub const PAPERBARK_THEME: Theme = Theme {
    name: "Paperbark trial",
    dark: false,
    base_100: Srgb::rgb(0xFF, 0xF8, 0xE9),
    base_200: Srgb::rgb(0xF8, 0xEC, 0xD1),
    base_300: Srgb::rgb(0xEB, 0xD8, 0xAE),
    base_content: Srgb::rgb(0x38, 0x25, 0x1A),
    muted: Srgb::rgb(0x80, 0x68, 0x50),
    faint: Srgb::rgb(0xB3, 0x9B, 0x7C),
    primary: Srgb::rgb(0xD8, 0x5A, 0x42),
    primary_content: Srgb::rgb(0xFF, 0xF6, 0xE9),
    error: Srgb::rgb(0xB9, 0x3A, 0x2E),
    selection: Srgb::rgba(0xC7, 0x7A, 0x4B, 0x52),
    background: Background::Gradient {
        from: GROUND,
        to: LAYER,
        dir: (0.0, 1.0),
    },
    font: "Fraunces 9pt",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    heading_bold: false,
    cjk: CJK_JA_SHIPPORI,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments {
        dash: '\u{F01B}',
        star: '\u{F01D}',
        underscore: '\u{F01E}',
    },
    ornament_face: ORNAMENT_GARAMOND,
    ornament_scale: ORNAMENT_SCALE_FLEURON,
    bullets: ('❦', '❧', '☙'),
    bullet_scale: crate::theme::BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Dawn"),
        register: Some("Refined"),
        voice: Some("Literary"),
        temperature: Some("Warm"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

fn selected() -> &'static Option<Profile> {
    static SELECTED: OnceLock<Option<Profile>> = OnceLock::new();
    SELECTED.get_or_init(|| std::env::var(ENV).ok().as_deref().and_then(Profile::parse))
}

pub fn profile() -> Option<Profile> {
    *selected()
}

pub fn theme_override() -> Option<Theme> {
    profile().map(|_| PAPERBARK_THEME)
}

pub fn theme_or(default: Theme) -> Theme {
    theme_override().unwrap_or(default)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSpec {
    pub profile: Profile,
    pub from: Srgb,
    pub to: Srgb,
    pub tint: Srgb,
}

pub fn render_spec() -> Option<RenderSpec> {
    profile().map(|profile| RenderSpec {
        profile,
        from: GROUND,
        to: LAYER,
        tint: SHADOW,
    })
}

pub fn background_desc() -> Option<BgDesc> {
    render_spec().map(|trial| BgDesc {
        from: trial.from.rgba_bytes(),
        to: trial.to.rgba_bytes(),
        dir: (0.0, 1.0),
        shader: 9,
        tint: trial.tint.rgb_bytes(),
        edge: false,
        angle: 0.0,
        // The isolated trial shader reads only params.x as the A–E selector.
        period_px: trial.profile.shader_index(),
        amplitude_px: 0.0,
        density: 0.0,
        banded: false,
    })
}

pub fn background_desc_or(default: BgDesc) -> BgDesc {
    background_desc().unwrap_or(default)
}

pub fn background_json() -> Option<String> {
    render_spec().map(|spec| {
        let hex = |color: Srgb| crate::capture::json_string(&color.hex());
        format!(
            concat!(
                "{{ \"kind\": \"paperbark-trial\", \"profile\": {}, \"slug\": {}, ",
                "\"label\": {}, \"tones\": [{}, {}, {}], \"static\": true }}"
            ),
            crate::capture::json_string(spec.profile.id()),
            crate::capture::json_string(spec.profile.slug()),
            crate::capture::json_string(spec.profile.label()),
            hex(spec.from),
            hex(spec.to),
            hex(spec.tint),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_profiles_have_unique_ids_slugs_and_shader_indices() {
        let _g = crate::testlock::serial();
        let ids: std::collections::HashSet<_> = Profile::ALL.iter().map(|p| p.id()).collect();
        let slugs: std::collections::HashSet<_> = Profile::ALL.iter().map(|p| p.slug()).collect();
        let indices: std::collections::HashSet<_> = Profile::ALL
            .iter()
            .map(|p| p.shader_index() as u8)
            .collect();
        assert_eq!(Profile::ALL.len(), 5);
        assert_eq!(ids.len(), 5, "Paperbark trial profile ids must be unique");
        assert_eq!(slugs.len(), 5, "Paperbark trial slugs must be unique");
        assert_eq!(
            indices.len(),
            5,
            "Paperbark trial shader indices must be unique"
        );
    }

    #[test]
    fn paperbark_is_not_a_shipping_world() {
        let _g = crate::testlock::serial();
        assert!(
            !crate::theme::THEMES
                .iter()
                .any(|world| world.name == PAPERBARK_THEME.name),
            "the disposable Paperbark trial must never enroll in the shipping roster"
        );
        assert_eq!(crate::theme::THEMES.len(), 18);
    }

    #[test]
    fn every_profile_shares_one_fixed_world_and_is_static() {
        let _g = crate::testlock::serial();
        for profile in Profile::ALL {
            let spec = RenderSpec {
                profile,
                from: GROUND,
                to: LAYER,
                tint: SHADOW,
            };
            assert_eq!(spec.from, GROUND);
            assert_eq!(spec.to, LAYER);
            assert_eq!(spec.tint, SHADOW);
            assert_eq!(PAPERBARK_THEME.font, "Fraunces 9pt");
            assert_eq!(PAPERBARK_THEME.primary, Srgb::rgb(0xD8, 0x5A, 0x42));
            assert!(!PAPERBARK_THEME.render_caps.ambient.is_animated());
        }
    }
}
