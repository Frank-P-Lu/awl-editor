//! GALLERY-ONLY world explorations — authored `Theme` values that are NOT in
//! [`super::THEMES`] and never reach a user.
//!
//! They live beside the shipped roster rather than inside `worlds.rs` for one
//! reason: that file is the SHIPPED roster, and a reader counting worlds there
//! should count only worlds. Nothing constructs these; `theme::THEMES` is the
//! single enrollment door, so a value here cannot leak into the picker, a
//! capture, the icon roster or a law sweep by accident.
//!
//! `CASSOWARY_LIGHT` is the light-terminal audition kept from Cassowary's own
//! round. Queue item 118 records the decision NOT to graduate it.

use super::cjk::{CJK_GOTHIC, CJK_KO, CJK_ZH_HANS_SANS, CJK_ZH_HANT};
use super::color::Srgb;
use super::ground::Background;
use super::model::{
    CardAnchor, ChipVariant, ChromeFace, Elevation, FacetStyle, IconCursor, IconGround, PaneSplit,
    PlacardCorner, PlacardInk, RenderCaps, RoleOverrides, Theme, ThemeTags, TitleStyle,
};
use super::ornament::{
    BULLET_SCALE_PLAIN, BULLETS_PLAIN, LIST_INDENT_SCALE_PLAIN, ORNAMENT_MARKS,
    ORNAMENT_SCALE_GEOMETRIC, Ornaments,
};
use super::worlds::POSTER_BARS;

#[allow(dead_code)] // never enrolled; see the module doc.
pub const CASSOWARY_LIGHT: Theme = Theme {
    name: "Cassowary Light",
    dark: false,
    base_100: Srgb::rgb(0xEE, 0xF4, 0xF0),
    base_200: Srgb::rgb(0xE2, 0xEC, 0xE6),
    base_300: Srgb::rgb(0xD2, 0xE1, 0xD8),
    base_content: Srgb::rgb(0x16, 0x24, 0x1B),
    muted: Srgb::rgb(0x5A, 0x6E, 0x62),
    faint: Srgb::rgb(0x92, 0xA3, 0x98),
    primary: Srgb::rgb(0xD9, 0x79, 0x22),
    primary_content: Srgb::rgb(0xFB, 0xEF, 0xE2),
    error: Srgb::rgb(0xC2, 0x34, 0x29),
    selection: Srgb::rgba(0xC8, 0x36, 0x5E, 0x5E),
    background: Background::Pinstripe {
        from: Srgb::rgb(0xE2, 0xEC, 0xE6),
        to: Srgb::rgb(0xD2, 0xE1, 0xD8),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0xAF, 0xC6, 0xB8),
    },
    font: "Iosevka",
    mono: "Iosevka",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('◆', '✴', '◈'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Day"),
        register: None,
        voice: Some("Technical"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink: PlacardInk::Bold,
        },
        card_anchor: CardAnchor::TopLeft,
        chrome_face: ChromeFace::Named("Archivo Black"),
        elevation: Elevation::Bordered,
        list_style: POSTER_BARS,
        facet_style: FacetStyle::Chips(ChipVariant::Bracket),
        pane_split: PaneSplit::Unified,
        ..RenderCaps::DEFAULT
    },
};
