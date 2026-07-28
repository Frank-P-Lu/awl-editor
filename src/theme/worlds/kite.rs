use super::*;

/// Loud light technical room: a stable mineral page travelling through one
/// cool warped-grid field, with the caret as the bird's single vermilion eye.
pub const KITE: Theme = Theme {
    name: "Kite",
    dark: false,
    base_100: Srgb::rgb(0xF8, 0xF7, 0xFC),
    base_200: Srgb::rgb(0xEE, 0xEA, 0xF7),
    base_300: Srgb::rgb(0xDF, 0xD8, 0xED),
    base_content: Srgb::rgb(0x24, 0x24, 0x3A),
    muted: Srgb::rgb(0x68, 0x64, 0x7D),
    faint: Srgb::rgb(0xA1, 0x9B, 0xAF),
    primary: Srgb::rgb(0xE4, 0x47, 0x2F),
    primary_content: Srgb::rgb(0xFF, 0xF2, 0xEE),
    error: Srgb::rgb(0xC8, 0x2F, 0x38),
    selection: Srgb::rgba(0x65, 0x5F, 0x9B, 0x52),
    background: Background::WarpedGrid {
        tones: [
            Srgb::rgb(0xEE, 0xEA, 0xF7),
            Srgb::rgb(0x9E, 0x97, 0xBB),
            Srgb::rgb(0x40, 0x3D, 0x64),
        ],
        spacing_px: 54.0,
        density: 0.78,
        curvature: 0.90,
    },
    font: "Fira Sans",
    mono: "Iosevka",
    icon_cursor: IconCursor::Narrow,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments {
        dash: '◆',
        star: '✦',
        underscore: '◈',
    },
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Day"),
        register: Some("Refined"),
        voice: Some("Modern"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        ..RenderCaps::DEFAULT
    },
};
