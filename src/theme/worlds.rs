use super::cjk::{
    CJK_GOTHIC, CJK_JA_KLEE, CJK_JA_SHIPPORI, CJK_JA_ZENMARU, CJK_KO, CJK_KO_SERIF, CJK_MINCHO,
    CJK_ZH_HANS_KLEE, CJK_ZH_HANS_SANS, CJK_ZH_HANS_SERIF, CJK_ZH_HANT,
};
use super::color::Srgb;
use super::ground::{Background, Tunnel, Weave};
use super::model::{
    AmbientStyle, Backdrop, CardAnchor, CardShape, CardTexture, CaretBlockStyle, ChipVariant,
    ChromeFace, DecorativeWash, Elevation, FacetStyle, FoldAfford, Frost, HighlightTexture,
    IconCursor, IconGround, ImageReveal, ListStyle, LocationStyle, MotionJuice, PageFrame,
    PaneSplit, PlacardCorner, PlacardInk, RenderCaps, RoleOverrides, RuleSelection,
    SPELL_UNDERLINE_GAP_DEFAULT, SelectionStyle, Theme, ThemeTags, TitleStyle, WashOverride,
};
use super::ornament::{
    BULLET_SCALE_GARAMOND, BULLET_SCALE_ORNAMENT, BULLET_SCALE_PLAIN, BULLETS_PLAIN,
    LIST_INDENT_SCALE_PLAIN, LIST_INDENT_SCALE_WIDE, ORNAMENT_GARAMOND, ORNAMENT_JUNICODE,
    ORNAMENT_MARKS, ORNAMENT_SCALE_FLEURON, ORNAMENT_SCALE_GEOMETRIC, ORNAMENT_SCALE_ORNATE,
    Ornaments,
};
pub const GUMTREE: Theme = Theme {
    name: "Gumtree",
    dark: false,
    base_100: Srgb::rgb(0xE4, 0xF8, 0xE2),
    base_200: Srgb::rgb(0xCF, 0xF3, 0xCC),
    base_300: Srgb::rgb(0xB7, 0xEF, 0xB4),
    base_content: Srgb::rgb(0x16, 0x24, 0x1A),
    muted: Srgb::rgb(0x5A, 0x6B, 0x57),
    faint: Srgb::rgb(0x91, 0xA3, 0x8F),
    primary: Srgb::rgb(0xDA, 0x52, 0x5D),
    primary_content: Srgb::rgb(0xFB, 0xEC, 0xEC),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0x88, 0x8F, 0x5D, 0x52),
    selection_ui: None,
    background: Background::Zigzag {
        from: Srgb::rgb(0xE4, 0xF8, 0xE2),
        to: Srgb::rgb(0xCF, 0xF3, 0xCC),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0xB7, 0xEF, 0xB4),
        period_px: 170.0,
        amplitude_px: 60.0,
        angle: 0.26,
        density: 0.40,
        banded: false,
    },
    font: "Literata",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_JA_SHIPPORI,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('\u{E67D}', '\u{E270}', '\u{E68A}'),
    ornament_face: ORNAMENT_JUNICODE,
    ornament_scale: ORNAMENT_SCALE_ORNATE,
    bullets: ('❧', '☙', '❦'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Day"),
        register: None,
        voice: Some("Literary"),
        temperature: Some("Cool"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        ..RenderCaps::DEFAULT
    },
};
pub const POTOROO: Theme = Theme {
    name: "Potoroo",
    dark: true,
    base_100: Srgb::rgb(0x1F, 0x04, 0x00),
    base_200: Srgb::rgb(0x31, 0x05, 0x00),
    base_300: Srgb::rgb(0x56, 0x28, 0x00),
    base_content: Srgb::rgb(0xF0, 0xE6, 0xDE),
    muted: Srgb::rgb(0x9C, 0x85, 0x76),
    faint: Srgb::rgb(0x75, 0x5D, 0x51),
    primary: Srgb::rgb(0xFE, 0xAF, 0x69),
    primary_content: Srgb::rgb(0x2A, 0x14, 0x02),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x7E, 0xB4, 0x7C, 0x52),
    selection_ui: None,
    background: Background::Stripes {
        from: Srgb::rgb(0x1F, 0x04, 0x00),
        to: Srgb::rgb(0x56, 0x28, 0x00),
        band: Srgb::rgb(0x6B, 0x3A, 0x12),
        angle: 0.6,
    },
    font: "Monaspace Xenon",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✶', '✦', '◆'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Dusk"),
        register: Some("Humble"),
        voice: Some("Technical"),
        temperature: Some("Warm"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Recessed,
        ..RenderCaps::DEFAULT
    },
};
pub const BILBY: Theme = Theme {
    name: "Bilby",
    dark: false,
    base_100: Srgb::rgb(0xFF, 0xF7, 0xEF),
    base_200: Srgb::rgb(0xFB, 0xED, 0xE6),
    base_300: Srgb::rgb(0xF3, 0xE1, 0xD6),
    base_content: Srgb::rgb(0x26, 0x20, 0x38),
    muted: Srgb::rgb(0x6B, 0x65, 0x7A),
    faint: Srgb::rgb(0xA7, 0x9D, 0xB6),
    primary: Srgb::rgb(0xBC, 0x7E, 0x16),
    primary_content: Srgb::rgb(0xFD, 0xF4, 0xE2),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0x8F, 0x7B, 0xB8, 0x52),
    selection_ui: None,
    background: Background::Gradient {
        from: Srgb::rgb(0xFB, 0xED, 0xE6),
        to: Srgb::rgb(0xF3, 0xE1, 0xD6),
        dir: (0.0, 1.0),
    },
    font: "Newsreader 16pt 16pt",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_JA_SHIPPORI,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('❧', '☙', '❦'),
    ornament_face: ORNAMENT_GARAMOND,
    ornament_scale: ORNAMENT_SCALE_FLEURON,
    bullets: ('❧', '❦', '☙'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Dawn"),
        register: Some("Refined"),
        voice: None,
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        spell_underline_gap: SPELL_UNDERLINE_GAP_DEFAULT - 2.0,
        ..RenderCaps::DEFAULT
    },
};
pub const SALTPAN: Theme = Theme {
    name: "Saltpan",
    dark: false,
    base_100: Srgb::rgb(0xFD, 0xF7, 0xE2),
    base_200: Srgb::rgb(0xFB, 0xF3, 0xDE),
    base_300: Srgb::rgb(0xF2, 0xE6, 0xC7),
    base_content: Srgb::rgb(0x24, 0x1D, 0x12),
    muted: Srgb::rgb(0x7A, 0x6E, 0x55),
    faint: Srgb::rgb(0xAB, 0xA3, 0x8F),
    primary: Srgb::rgb(0x8D, 0x59, 0x25),
    primary_content: Srgb::rgb(0xFB, 0xF1, 0xE6),
    error: Srgb::rgb(0xB5, 0x45, 0x2B),
    selection_document: Srgb::rgba(0xA5, 0x86, 0x50, 0x52),
    selection_ui: None,
    background: Background::Pinstripe {
        from: Srgb::rgb(0xFB, 0xF3, 0xDE),
        to: Srgb::rgb(0xF2, 0xE6, 0xC7),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0xD9, 0xC7, 0x9B),
    },
    font: "Fraunces 9pt",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_MINCHO,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('\u{F01B}', '\u{F01D}', '\u{F01E}'),
    ornament_face: ORNAMENT_JUNICODE,
    ornament_scale: ORNAMENT_SCALE_ORNATE,
    bullets: ('❦', '❧', '☙'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Dawn"),
        register: Some("Refined"),
        voice: None,
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        ..RenderCaps::DEFAULT
    },
};
pub const QUOKKA: Theme = Theme {
    name: "Quokka",
    dark: false,
    base_100: Srgb::rgb(0xFF, 0xEA, 0xDD),
    base_200: Srgb::rgb(0xFF, 0xDF, 0xCF),
    base_300: Srgb::rgb(0xFF, 0xD2, 0xBD),
    base_content: Srgb::rgb(0x2B, 0x18, 0x10),
    muted: Srgb::rgb(0x8A, 0x64, 0x53),
    faint: Srgb::rgb(0xB4, 0x94, 0x85),
    primary: Srgb::rgb(0x07, 0x70, 0x73),
    primary_content: Srgb::rgb(0xE6, 0xF6, 0xF6),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0xBB, 0x80, 0x20, 0x52),
    selection_ui: None,
    background: Background::Zigzag {
        from: Srgb::rgb(0xFF, 0xDF, 0xCF),
        to: Srgb::rgb(0xFF, 0xD2, 0xBD),
        dir: (0.7, 0.7),
        tint: Srgb::rgb(0xE0, 0xAE, 0x92),
        period_px: 100.0,
        amplitude_px: 24.0,
        angle: 0.0,
        density: 0.60,
        banded: true,
    },
    font: "Sour Gummy",
    mono: "IBM Plex Mono",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_JA_KLEE,
    zh_hans: CJK_ZH_HANS_KLEE,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✿', '❀', '✽'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Dawn"),
        register: Some("Everyday"),
        voice: Some("Modern"),
        temperature: Some("Warm"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        card_texture: CardTexture::HalftoneDots {
            angle_deg: 18.0,
            cell_px: 8.0,
            density: 0.30,
        },
        card_shape: CardShape::Chamfered { cut_px: 11.0 },
        ..RenderCaps::DEFAULT
    },
};
pub const BOMBORA: Theme = Theme {
    name: "Bombora",
    dark: true,
    base_100: Srgb::rgb(0x15, 0x0A, 0x2C),
    base_200: Srgb::rgb(0x24, 0x15, 0x40),
    base_300: Srgb::rgb(0x3C, 0x36, 0x54),
    base_content: Srgb::rgb(0xEC, 0xE8, 0xF2),
    muted: Srgb::rgb(0x8A, 0x7F, 0xA8),
    faint: Srgb::rgb(0x53, 0x48, 0x6E),
    primary: Srgb::rgb(0xC5, 0x3C, 0x69),
    primary_content: Srgb::rgb(0x2A, 0x0A, 0x16),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x60, 0x50, 0xA8, 0x60),
    selection_ui: None,
    background: Background::Waves {
        tones: [
            Srgb::rgb(0x15, 0x0A, 0x2C),
            Srgb::rgb(0x24, 0x15, 0x40),
            Srgb::rgb(0x3C, 0x36, 0x54),
        ],
    },
    font: "EB Garamond",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_JA_SHIPPORI,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('☙', '❧', '❦'),
    ornament_face: ORNAMENT_GARAMOND,
    ornament_scale: ORNAMENT_SCALE_FLEURON,
    bullets: ('☞', '❧', '❦'),
    bullet_scale: BULLET_SCALE_GARAMOND,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Night"),
        register: Some("Refined"),
        voice: Some("Literary"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

pub const MULGA: Theme = Theme {
    name: "Mulga",
    dark: true,
    base_100: Srgb::rgb(0x16, 0x1F, 0x0F),
    base_200: Srgb::rgb(0x1E, 0x29, 0x16),
    base_300: Srgb::rgb(0x3E, 0x4A, 0x31),
    base_content: Srgb::rgb(0xEC, 0xEA, 0xE0),
    muted: Srgb::rgb(0x8A, 0x8C, 0x78),
    faint: Srgb::rgb(0x51, 0x56, 0x47),
    primary: Srgb::rgb(0xDE, 0x8E, 0x7F),
    primary_content: Srgb::rgb(0x2A, 0x14, 0x10),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x9B, 0x8B, 0x4B, 0x52),
    selection_ui: None,
    background: Background::Pinstripe {
        from: Srgb::rgb(0x16, 0x1F, 0x0F),
        to: Srgb::rgb(0x1E, 0x29, 0x16),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0x3E, 0x4A, 0x31),
    },
    font: "Zilla Slab",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_MINCHO,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('⁑', '⁂', '❦'),
    ornament_face: ORNAMENT_JUNICODE,
    ornament_scale: ORNAMENT_SCALE_ORNATE,
    bullets: ('☙', '❦', '❧'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: None,
        register: Some("Everyday"),
        voice: None,
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

pub const TAWNY: Theme = Theme {
    name: "Tawny",
    dark: true,
    base_100: Srgb::rgb(0x16, 0x18, 0x1D),
    base_200: Srgb::rgb(0x20, 0x22, 0x28),
    base_300: Srgb::rgb(0x2A, 0x2D, 0x34),
    base_content: Srgb::rgb(0xE6, 0xE6, 0xE6),
    muted: Srgb::rgb(0x8B, 0x91, 0x9D),
    faint: Srgb::rgb(0x4E, 0x52, 0x5A),
    primary: Srgb::rgb(0xFF, 0xC0, 0x5E),
    primary_content: Srgb::rgb(0x26, 0x1A, 0x08),
    error: Srgb::rgb(0xE5, 0x4B, 0x4B),
    selection_document: Srgb::rgba(0x3A, 0x6F, 0xD8, 0x52),
    selection_ui: None,
    background: Background::Dots {
        from: Srgb::rgb(0x16, 0x18, 0x1D),
        to: Srgb::rgb(0x20, 0x22, 0x28),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0x2C, 0x2F, 0x37),
        edge: false,
    },
    font: "IBM Plex Mono",
    mono: "IBM Plex Mono",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✦', '✷', '◈'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: None,
        register: Some("Humble"),
        voice: None,
        temperature: Some("Neutral"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

pub const MOPOKE: Theme = Theme {
    name: "Mopoke",
    dark: true,
    base_100: Srgb::rgb(0x1B, 0x18, 0x14),
    base_200: Srgb::rgb(0x25, 0x21, 0x1B),
    base_300: Srgb::rgb(0x31, 0x2B, 0x22),
    base_content: Srgb::rgb(0xE8, 0xE4, 0xDC),
    muted: Srgb::rgb(0x97, 0x8C, 0x7E),
    faint: Srgb::rgb(0x57, 0x50, 0x47),
    primary: Srgb::rgb(0xF5, 0x6E, 0x3D),
    primary_content: Srgb::rgb(0x26, 0x1A, 0x08),
    error: Srgb::rgb(0xE5, 0x4B, 0x4B),
    selection_document: Srgb::rgba(0x7B, 0x39, 0xC6, 0x52),
    selection_ui: None,
    background: Background::Dots {
        from: Srgb::rgb(0x1B, 0x18, 0x14),
        to: Srgb::rgb(0x25, 0x21, 0x1B),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0x33, 0x2D, 0x24),
        edge: false,
    },
    font: "Bitter",
    mono: "IBM Plex Mono",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_JA_KLEE,
    zh_hans: CJK_ZH_HANS_KLEE,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('\u{E670}', '\u{F011}', '\u{F014}'),
    ornament_face: ORNAMENT_JUNICODE,
    ornament_scale: ORNAMENT_SCALE_ORNATE,
    bullets: ('\u{E670}', '\u{EF92}', '\u{E67D}'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Dusk"),
        register: Some("Humble"),
        voice: None,
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

pub const BOWERBIRD: Theme = Theme {
    name: "Bowerbird",
    dark: true,
    base_100: Srgb::rgb(0x0C, 0x14, 0x26),
    base_200: Srgb::rgb(0x13, 0x1D, 0x33),
    base_300: Srgb::rgb(0x1F, 0x2C, 0x49),
    base_content: Srgb::rgb(0xE7, 0xEA, 0xF2),
    muted: Srgb::rgb(0x80, 0x89, 0xA0),
    faint: Srgb::rgb(0x46, 0x4E, 0x63),
    primary: Srgb::rgb(0xF5, 0xA7, 0x42),
    primary_content: Srgb::rgb(0x2A, 0x1B, 0x06),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x3D, 0x6B, 0xC4, 0x52),
    selection_ui: None,
    background: Background::Organic {
        tones: [
            Srgb::rgb(0x0C, 0x14, 0x26),
            Srgb::rgb(0x13, 0x1D, 0x33),
            Srgb::rgb(0x1F, 0x2C, 0x49),
        ],
        scale_px: 195.0,
        density: 0.46,
    },
    font: "IBM Plex Sans",
    mono: "JetBrains Mono",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_JA_ZENMARU,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('❂', '✴', '◈'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Night"),
        register: Some("Everyday"),
        voice: Some("Modern"),
        temperature: Some("Cool"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps::DEFAULT,
};

pub const CURRAWONG: Theme = Theme {
    name: "Currawong",
    dark: true,
    base_100: Srgb::rgb(0x06, 0x06, 0x07),
    base_200: Srgb::rgb(0x0E, 0x0F, 0x11),
    base_300: Srgb::rgb(0x1C, 0x1E, 0x22),
    base_content: Srgb::rgb(0xED, 0xEE, 0xF0),
    muted: Srgb::rgb(0x88, 0x8C, 0x94),
    faint: Srgb::rgb(0x44, 0x46, 0x4B),
    primary: Srgb::rgb(0xF4, 0xC5, 0x34),
    primary_content: Srgb::rgb(0x1E, 0x1A, 0x06),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x3E, 0x5C, 0x8A, 0x52),
    selection_ui: None,
    background: Background::Gradient {
        from: Srgb::rgb(0x06, 0x06, 0x07),
        to: Srgb::rgb(0x0E, 0x0F, 0x11),
        dir: (0.0, 1.0),
    },
    font: "Iosevka",
    mono: "Iosevka",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✷', '✴', '⬥'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Night"),
        register: None,
        voice: Some("Technical"),
        temperature: Some("Neutral"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        card_anchor: CardAnchor::TopLeft,
        ambient: AmbientStyle::Stars {
            tint: Srgb::rgb(0x9B, 0xB0, 0xD2),
            cell_px: 34.0,
            density: 0.30,
            size_px: 2.6,
            peak: 0.5,
            floor: 0.18,
        },
        ..RenderCaps::DEFAULT
    },
};

pub const MANGROVE: Theme = Theme {
    name: "Mangrove",
    dark: true,
    base_100: Srgb::rgb(0x11, 0x27, 0x23),
    base_200: Srgb::rgb(0x18, 0x34, 0x2E),
    base_300: Srgb::rgb(0x26, 0x43, 0x3B),
    base_content: Srgb::rgb(0xD9, 0xE6, 0xE1),
    muted: Srgb::rgb(0x6F, 0x8A, 0x83),
    faint: Srgb::rgb(0x41, 0x55, 0x51),
    primary: Srgb::rgb(0xF2, 0xA6, 0x5C),
    primary_content: Srgb::rgb(0x2A, 0x18, 0x04),
    error: Srgb::rgb(0xFF, 0x6B, 0x5C),
    selection_document: Srgb::rgba(0x40, 0xA8, 0x9E, 0x60),
    selection_ui: None,
    background: Background::Lava {
        ground: Srgb::rgb(0x11, 0x27, 0x23),
        blob_lo: Srgb::rgb(0x17, 0x23, 0x2B),
        blob_hi: Srgb::rgb(0x22, 0x3C, 0x4F),
        dithered: true,
    },
    font: "JetBrains Mono",
    mono: "JetBrains Mono",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('❖', '◈', '⬥'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: None,
        register: None,
        voice: Some("Technical"),
        temperature: Some("Cool"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink: PlacardInk::Stipple,
        },
        card_anchor: CardAnchor::TopRight,
        elevation: Elevation::Bordered,
        list_style: ListStyle::Diagonal(super::diagonal::DiagonalDirection::Descending),
        facet_style: FacetStyle::Chips(ChipVariant::Bracket),
        fold_afford: FoldAfford {
            chevron_lift: 0.60,
            tail_lift: 0.75,
        },
        ..RenderCaps::DEFAULT
    },
};

pub const GALAH: Theme = Theme {
    name: "Galah",
    dark: false,
    base_100: Srgb::rgb(0xFC, 0xEE, 0xF1),
    base_200: Srgb::rgb(0xF8, 0xE0, 0xE6),
    base_300: Srgb::rgb(0xF1, 0xCF, 0xD9),
    base_content: Srgb::rgb(0x2A, 0x17, 0x1D),
    muted: Srgb::rgb(0x7C, 0x60, 0x68),
    faint: Srgb::rgb(0xA9, 0x92, 0x98),
    primary: Srgb::rgb(0xB2, 0x3A, 0x60),
    primary_content: Srgb::rgb(0xFB, 0xEA, 0xEE),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0x9A, 0x6B, 0x86, 0x52),
    selection_ui: None,
    // Sparse plumage in Galah's mauve ladder.
    background: Background::Deckle {
        ground: Srgb::rgb(0xF8, 0xE0, 0xE6),
        layer: Srgb::rgb(0xF1, 0xCF, 0xD9),
        deckle: Srgb::rgb(0xA9, 0x92, 0x98),
        weave: Weave::Fibres,
        period_px: 64.0,
        wander_px: 8.0,
        density: 0.10,
    },
    font: "Figtree",
    mono: "IBM Plex Mono",
    icon_cursor: IconCursor::Narrow,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_JA_ZENMARU,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('❁', '❂', '✿'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Dawn"),
        register: None,
        voice: Some("Modern"),
        temperature: Some("Warm"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink: PlacardInk::Ghost,
        },
        card_anchor: CardAnchor::TopLeft,
        elevation: Elevation::Bordered,
        list_style: ListStyle::Bars,
        facet_style: FacetStyle::Chips(ChipVariant::Hairline),
        ..RenderCaps::DEFAULT
    },
};

pub const MAGPIE: Theme = Theme {
    name: "Magpie",
    dark: false,
    base_100: Srgb::rgb(0xFB, 0xFB, 0xFA),
    base_200: Srgb::rgb(0xF1, 0xF1, 0xEF),
    base_300: Srgb::rgb(0xE4, 0xE4, 0xE1),
    base_content: Srgb::rgb(0x11, 0x13, 0x17),
    muted: Srgb::rgb(0x6C, 0x70, 0x77),
    faint: Srgb::rgb(0x9F, 0xA2, 0xA6),
    primary: Srgb::rgb(0xDB, 0x5A, 0x2B),
    primary_content: Srgb::rgb(0xFB, 0xEF, 0xE9),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0x46, 0x61, 0x8F, 0x52),
    selection_ui: None,
    background: Background::Bands {
        tones: [
            Srgb::rgb(0xFB, 0xFB, 0xFA),
            Srgb::rgb(0xF1, 0xF1, 0xEF),
            Srgb::rgb(0xE4, 0xE4, 0xE1),
        ],
        angle: 0.62,
    },
    font: "Bitter",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_MINCHO,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    ornaments: Ornaments::of('\u{EF90}', '\u{EF98}', '\u{EF9A}'),
    ornament_face: ORNAMENT_JUNICODE,
    ornament_scale: ORNAMENT_SCALE_ORNATE,
    bullets: ('❦', '☙', '❧'),
    bullet_scale: BULLET_SCALE_ORNAMENT,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Day"),
        register: None,
        voice: Some("Literary"),
        temperature: Some("Neutral"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink: PlacardInk::Ghost,
        },
        card_anchor: CardAnchor::TopLeft,
        elevation: Elevation::Bordered,
        list_style: ListStyle::Diagonal(super::diagonal::DiagonalDirection::Ascending),
        facet_style: FacetStyle::Chips(ChipVariant::Underline),
        location_style: LocationStyle::Raked,
        ..RenderCaps::DEFAULT
    },
};

pub const BROLGA: Theme = Theme {
    name: "Brolga",
    dark: false,
    base_100: Srgb::rgb(0xE9, 0xEF, 0xFB),
    base_200: Srgb::rgb(0xDC, 0xE6, 0xF8),
    base_300: Srgb::rgb(0xC7, 0xD7, 0xF2),
    base_content: Srgb::rgb(0x1B, 0x24, 0x36),
    muted: Srgb::rgb(0x58, 0x63, 0x7A),
    faint: Srgb::rgb(0x99, 0xA3, 0xB6),
    primary: Srgb::rgb(0xD7, 0x5B, 0x41),
    primary_content: Srgb::rgb(0xFC, 0xEE, 0xEA),
    error: Srgb::rgb(0xC0, 0x39, 0x2B),
    selection_document: Srgb::rgba(0x35, 0x57, 0xA0, 0x60),
    selection_ui: None,
    background: Background::Gradient {
        from: Srgb::rgb(0xDC, 0xE6, 0xF8),
        to: Srgb::rgb(0xC7, 0xD7, 0xF2),
        dir: (0.0, 1.0),
    },
    font: "IBM Plex Sans",
    mono: "IBM Plex Mono",
    icon_cursor: IconCursor::Pill,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✧', '✴', '⬥'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Day"),
        register: None,
        voice: None,
        temperature: Some("Cool"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        ..RenderCaps::DEFAULT
    },
};

pub const WAGTAIL: Theme = Theme {
    name: "Wagtail",
    dark: true,
    base_100: Srgb::rgb(0x00, 0x00, 0x00),
    base_200: Srgb::rgb(0x00, 0x00, 0x00),
    base_300: Srgb::rgb(0x00, 0x00, 0x00),
    base_content: Srgb::rgb(0xFF, 0xFF, 0xFF),
    muted: Srgb::rgb(0xFF, 0xFF, 0xFF),
    faint: Srgb::rgb(0xFF, 0xFF, 0xFF),
    primary: Srgb::rgb(0xFF, 0xFF, 0xFF),
    primary_content: Srgb::rgb(0x00, 0x00, 0x00),
    error: Srgb::rgb(0xFF, 0xFF, 0xFF),
    selection_document: Srgb::rgba(0xFF, 0xFF, 0xFF, 0xFF),
    selection_ui: None,
    background: Background::Gradient {
        from: Srgb::rgb(0x00, 0x00, 0x00),
        to: Srgb::rgb(0x00, 0x00, 0x00),
        dir: (0.0, 1.0),
    },
    font: "JetBrains Mono",
    mono: "JetBrains Mono",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✧', '⭑', '❡'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: Some("Dusk"),
        register: None,
        voice: None,
        temperature: None,
    },
    role_overrides: RoleOverrides {
        def_fg: Some(Srgb::rgb(0xFF, 0xFF, 0xFF)),
        const_fg: Some(Srgb::rgb(0xFF, 0xFF, 0xFF)),
        str_fg: Some(Srgb::rgb(0xFF, 0xFF, 0xFF)),
        comment_wash: WashOverride::Off,
        str_wash: WashOverride::Off,
    },
    render_caps: RenderCaps {
        selection_style: SelectionStyle::InverseVideo,
        caret_block_style: CaretBlockStyle::InverseVideo,
        backdrop: Backdrop::Flat,
        elevation: Elevation::Bordered,
        decorative_wash: DecorativeWash::Off,
        image_reveal: ImageReveal::Opaque,
        highlight_texture: HighlightTexture::Stipple {
            color: Srgb::rgb(0xFF, 0xFF, 0xFF),
            density: crate::render::dither::WAGTAIL_HIGHLIGHT_DITHER_DENSITY,
        },
        title_style: TitleStyle::InlinePrefix,
        page_frame: PageFrame::Line { weight_px: 2.0 },
        card_anchor: CardAnchor::TopLeft,
        chrome_face: ChromeFace::Body,
        motion: MotionJuice::CALM,
        list_style: ListStyle::Pane,
        facet_style: FacetStyle::Text,
        location_style: LocationStyle::Inline,
        pane_split: PaneSplit::Split,
        ambient: AmbientStyle::None,
        spell_underline_gap: SPELL_UNDERLINE_GAP_DEFAULT,
        frost: Frost::DEFAULT,
        fold_afford: FoldAfford::DEFAULT,
        card_texture: CardTexture::DEFAULT,
        card_shape: CardShape::DEFAULT,
    },
};

pub const FIRETAIL: Theme = Theme {
    name: "Firetail",
    dark: true,
    base_100: Srgb::rgb(0x17, 0x09, 0x0C),
    base_200: Srgb::rgb(0x24, 0x0D, 0x12),
    base_300: Srgb::rgb(0x52, 0x16, 0x29),
    base_content: Srgb::rgb(0xEF, 0xE5, 0xE2),
    muted: Srgb::rgb(0x9F, 0x7E, 0x7C),
    faint: Srgb::rgb(0x69, 0x48, 0x4A),
    primary: Srgb::rgb(0xF2, 0xB1, 0x40),
    primary_content: Srgb::rgb(0x23, 0x14, 0x05),
    error: Srgb::rgb(0xE6, 0x4E, 0x48),
    selection_document: Srgb::rgba(0xB6, 0x5A, 0x6E, 0x60),
    selection_ui: None,
    background: Background::Lava {
        ground: Srgb::rgb(0x17, 0x09, 0x0C),
        blob_lo: Srgb::rgb(0x24, 0x0C, 0x14),
        blob_hi: Srgb::rgb(0x52, 0x18, 0x2C),
        dithered: false,
    },
    font: "Monaspace Xenon",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    // Item 121: the user's A/B/C pick, from one Block-cursor comparison
    // sheet — C reads burgundy at every size down to 24px, where A stays
    // near-black/cream/gold (no wine identity) and B's wine fades out by
    // 32px. Firetail is the one shipped exception to the inert default;
    // see `every_shipped_world_defaults_to_the_inert_base_100_ground_except_firetail`.
    icon_ground: IconGround::Blend40,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('✷', '✶', '✦'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: None,
        register: None,
        voice: None,
        temperature: Some("Warm"),
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::BL,
            scale: 4.5,
            ink: PlacardInk::Bold,
        },
        card_anchor: CardAnchor::TopLeft,
        chrome_face: ChromeFace::Named("Archivo Black"),
        elevation: Elevation::Bordered,
        list_style: ListStyle::Bars,
        facet_style: FacetStyle::Chips(ChipVariant::FilledActive),
        fold_afford: FoldAfford {
            chevron_lift: 0.0,
            tail_lift: 0.40,
        },
        ..RenderCaps::DEFAULT
    },
};

pub const CASSOWARY: Theme = Theme {
    name: "Cassowary",
    dark: true,
    base_100: Srgb::rgb(0x05, 0x05, 0x06),
    base_200: Srgb::rgb(0x0B, 0x0C, 0x0D),
    base_300: Srgb::rgb(0x14, 0x2C, 0x1E),
    base_content: Srgb::rgb(0xA8, 0xEC, 0xBE),
    muted: Srgb::rgb(0x5C, 0x9E, 0x70),
    faint: Srgb::rgb(0x37, 0x63, 0x4A),
    primary: Srgb::rgb(0xA8, 0xEC, 0xBE),
    primary_content: Srgb::rgb(0x05, 0x05, 0x06),
    error: Srgb::rgb(0xFF, 0x44, 0x36),
    selection_document: Srgb::rgba(0xD2, 0x45, 0x5F, 0x70),
    selection_ui: None,
    background: Background::Pinstripe {
        from: Srgb::rgb(0x05, 0x05, 0x06),
        to: Srgb::rgb(0x0B, 0x0C, 0x0D),
        dir: (0.0, 1.0),
        tint: Srgb::rgb(0x1E, 0x4A, 0x32),
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
        time: Some("Night"),
        register: None,
        voice: Some("Technical"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        caret_block_style: CaretBlockStyle::Filled,
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink: PlacardInk::Bold,
        },
        card_anchor: CardAnchor::TopRight,
        chrome_face: ChromeFace::Named("Archivo Black"),
        elevation: Elevation::Bordered,
        list_style: ListStyle::Bars,
        facet_style: FacetStyle::Chips(ChipVariant::Bracket),
        // The active facet (Files, Navigate, …) reads as a small vertical
        // secondary heading flush with the card's own left border,
        // subordinate to the bold "Commands" placard, rather than repeating
        // the inline treatment every other world uses.
        location_style: LocationStyle::RotatedRail,
        pane_split: PaneSplit::Unified,
        ..RenderCaps::DEFAULT
    },
};

/// PAPERBARK — handmade paper in a daylit studio. Static deckled contours in
/// [`Weave::Strata`] gather around the flat writing page; deep bark-brown prose
/// and the vermilion caret sit above cream and pale honey. Temperature stays
/// untagged because the Warm band is at its curated cap; WORLDS.md says why.
pub const PAPERBARK: Theme = Theme {
    name: "Paperbark",
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
    selection_document: Srgb::rgba(0xC7, 0x7A, 0x4B, 0x52),
    selection_ui: None,
    background: Background::Deckle {
        ground: Srgb::rgb(0xF0, 0xDF, 0xBA),
        layer: Srgb::rgb(0xD8, 0xB7, 0x7A),
        deckle: Srgb::rgb(0x9F, 0x69, 0x37),
        weave: Weave::Strata,
        period_px: 47.0,
        wander_px: 6.5,
        density: 0.20,
    },
    font: "EB Garamond",
    mono: "Monaspace Xenon",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: false,
    cjk: CJK_JA_SHIPPORI,
    zh_hans: CJK_ZH_HANS_SERIF,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO_SERIF,
    // The third and last EB Garamond fleuron permutation (Bilby ❧ ☙ ❦, Bombora
    // ☙ ❧ ❦); the face ships exactly these three, so the trio is a rotation.
    ornaments: Ornaments::of('❦', '❧', '☙'),
    ornament_face: ORNAMENT_GARAMOND,
    ornament_scale: ORNAMENT_SCALE_FLEURON,
    bullets: ('☙', '❦', '❧'),
    bullet_scale: BULLET_SCALE_GARAMOND,
    list_indent_scale: LIST_INDENT_SCALE_WIDE,
    tags: ThemeTags {
        time: Some("Day"),
        register: Some("Refined"),
        voice: Some("Literary"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        elevation: Elevation::Bordered,
        // ⚠️ THE ONE CARRIER OF THE `Rules` PROTOTYPE, and a prototype is what
        // it is: the surface sweep, the Settings workspace and the pixel-law
        // suite a real row composition owes are all still unwritten. Paperbark
        // holds it because its ground is already a MATERIAL — Deckle's
        // `Weave::Strata` lays contour lanes that gather around the writing
        // column — so a card summoned over it reads as an object dropped on a
        // sheet, while a ruled index reads as the same sheet one register up.
        // `Weight` is the DECIDED treatment, chosen by the user against both
        // rendered side by side: the selected row's own bounding rules thicken
        // and run past the text measure to the card's full band, so the mark is
        // made of the list's own substance and the row's interior stays plain
        // ground. `Gutter` — a short heavy dash hanging in the margin — remains
        // drawable as `AWL_OVERLAY_LIST_FORCE=rules:gutter`, and is the quieter
        // of the two by a wide margin at 1x.
        list_style: ListStyle::Rules(RuleSelection::Weight),
        ..RenderCaps::DEFAULT
    },
};

pub const THEMES: [Theme; 20] = [
    TAWNY, MOPOKE, CURRAWONG, POTOROO, GUMTREE, BILBY, SALTPAN, QUOKKA, BOMBORA, BOWERBIRD, MULGA,
    MANGROVE, GALAH, MAGPIE, BROLGA, WAGTAIL, FIRETAIL, CASSOWARY, PAPERBARK, KITE,
];

/// KITE — a loud light technical room. The stable mineral page glides through
/// ONE cool warped-grid tunnel whose vanishing point sits behind the page, and
/// the caret is the bird's single vermilion eye: the deliberate light
/// counterpart to Firetail's warm, organic, drifting lava (cool / geometric /
/// crisp / directional against warm / organic / liquid / drifting). Temperature
/// stays untagged even though the world is definitionally cool — the picker's
/// Cool band is already at its curated cap of four, the same reason Paperbark
/// leaves Warm untagged.
///
/// THE CHROME IS BUILT FROM THOSE FOUR WORDS, and it answers Firetail dial for
/// dial rather than leaving this world stated only in its margins. GEOMETRIC:
/// Figtree, a geometric grotesque, against Firetail's Archivo Black. CRISP: a
/// hairline page frame drawn round the column, and the active facet on a filled
/// `Band` rather than a chip — drawn edges and right angles, the grid's own
/// grammar carried onto the chrome. DIRECTIONAL: the card takes the TOP-RIGHT
/// corner and the placard the BOTTOM-RIGHT, mirroring Firetail's top-left card
/// and bottom-left placard across the room; and the placard stays SMALL where
/// Firetail's is a 4.5x poster — a label, not a shout, which is the whole
/// difference between crisp and loud.
///
/// `FacetStyle::Band` had never been spent by any world. It is spent here
/// because a filled band under one active category is what a technical panel
/// does, not to use something up.
///
/// ⚠️ TWO FURTHER DIALS WERE BUILT, RENDERED AND REJECTED ON EVIDENCE, and both
/// rejections are findings rather than omissions.
///
/// `CardShape::Chamfered` reads as crisp geometry and it is — but it is
/// QUOKKA'S. `card_texture_shape.rs` holds an exclusivity law by name: the
/// chamfer and the halftone together are that world's printed card, and every
/// other world keeps the rectangular default. Diluting one world's identity is
/// not a thing a different world's round gets to decide.
///
/// A `BarConfig` of `FullWidth` x `SelectedOnly` — "`Pane` without the card"
/// — is worse than unavailable, it is incompatible. `BarExtent::hugs()` is
/// FALSE for `FullWidth`, and five shipped legibility laws gate the whole
/// plated-chrome family on it: the shortcut chord, the lens-strip tabs, the
/// faceted section header and the footer plate all vanish and their glyphs
/// float bare over a blurred document. `SelectedOnly` then removes the row
/// plates as well. `Pane` carries a card precisely so chrome is never read
/// against a blurred page and `Bars` carries per-row plates for the same
/// reason; removing both leaves nothing to read against. Shipping it would
/// need a compensating scrim, which is new mechanism, and this wave's
/// direction is fewer. (This is why `ListStyle::Bars` itself carries no
/// per-world fields any more — see `BarConfig`.)
pub const KITE: Theme = Theme {
    name: "Kite",
    dark: false,
    base_100: Srgb::rgb(0xF6, 0xF4, 0xFA),
    base_200: Srgb::rgb(0xE5, 0xDE, 0xF3),
    base_300: Srgb::rgb(0xCD, 0xC0, 0xE7),
    base_content: Srgb::rgb(0x24, 0x1D, 0x2F),
    muted: Srgb::rgb(0x6B, 0x63, 0x74),
    faint: Srgb::rgb(0xA6, 0x9F, 0xB0),
    primary: Srgb::rgb(0xFF, 0x3B, 0x14),
    primary_content: Srgb::rgb(0xFF, 0xF3, 0xEE),
    error: Srgb::rgb(0xC4, 0x2A, 0x32),
    selection_document: Srgb::rgba(0x5A, 0x4F, 0xB4, 0x55),
    selection_ui: None,
    background: Background::WarpedGrid {
        ground: Srgb::rgb(0xE5, 0xDE, 0xF3),
        minor: Srgb::rgb(0xA9, 0xA2, 0xC8),
        major: Srgb::rgb(0x46, 0x40, 0x6E),
        tunnel: Tunnel::Fixed,
        spacing_px: 30.0,
        density: 0.62,
    },
    font: "Fira Sans",
    mono: "JetBrains Mono",
    icon_cursor: IconCursor::Block,
    icon_ground: IconGround::Base100,
    heading_bold: true,
    cjk: CJK_GOTHIC,
    zh_hans: CJK_ZH_HANS_SANS,
    zh_hant: CJK_ZH_HANT,
    ko: CJK_KO,
    ornaments: Ornaments::of('\u{2B25}', '\u{2736}', '\u{25C6}'),
    ornament_face: ORNAMENT_MARKS,
    ornament_scale: ORNAMENT_SCALE_GEOMETRIC,
    bullets: BULLETS_PLAIN,
    bullet_scale: BULLET_SCALE_PLAIN,
    list_indent_scale: LIST_INDENT_SCALE_PLAIN,
    tags: ThemeTags {
        time: None,
        register: None,
        voice: Some("Modern"),
        temperature: None,
    },
    role_overrides: RoleOverrides::NONE,
    render_caps: RenderCaps {
        title_style: TitleStyle::Placard {
            corner: PlacardCorner::BR,
            scale: 1.4,
            ink: PlacardInk::Muted,
        },
        card_anchor: CardAnchor::TopRight,
        chrome_face: ChromeFace::Named("Figtree"),
        elevation: Elevation::Bordered,
        page_frame: PageFrame::Line { weight_px: 1.0 },
        facet_style: FacetStyle::Band,
        ..RenderCaps::DEFAULT
    },
};

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn world_index(name: &str) -> usize {
    let mut i = 0;
    while i < THEMES.len() {
        if str_eq(THEMES[i].name, name) {
            return i;
        }
        i += 1;
    }
    panic!("world_index: no world by that name")
}

pub const DEFAULT_THEME: usize = world_index("Saltpan");

pub fn world_names() -> Vec<&'static str> {
    THEMES.iter().map(|t| t.name).collect()
}
