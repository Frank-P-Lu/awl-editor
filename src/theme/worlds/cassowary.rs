use super::*;

pub const CASSOWARY: Theme = Theme {
    name: "Cassowary",
    toast_anchor: ToastAnchor::TopRight,
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
    ornaments: Ornaments::of("󿁎", "󿁏", "󿁍"),
    ornament_face: ORNAMENT_NISHIKI,
    bullet_face: ORNAMENT_MARKS,
    ornament_scale: 2.162, // roster ink-height equalization: was the shared ORNAMENT_SCALE_GEOMETRIC tier (1.5); this glyph's own ink-to-em ratio needs this much MORE to match the roster's tallest normalized ornament (see theme::tests::ornament)
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
        placard_placement: PlacardPlacement::Bleed {
            x_em: 0.0,
            y_em: 0.34,
        },
        card_anchor: CardAnchor::TopRight,
        chrome_face: ChromeFace::Named("Archivo Black"),
        elevation: Elevation::Bordered,
        list_style: ListStyle::Pane,
        pane_split: PaneSplit::Unified,
        facet_style: FacetStyle::DockedTab,
        location_style: LocationStyle::RotatedRail(LocationLabelStyle {
            face: LocationFace::Mono,
            scale: 0.28,
            ink: LocationInk::Flat(PaletteRole::Muted),
            tracking_em: 0.06,
            locator: LocationLocator::IndexOnly { digits: 2 },
        }),
        summoned_material: SummonedMaterial::Scanlines {
            pitch_px: 4.0,
            line_px: 1.0,
            strength: 0.12,
        },
        // The docked seam edge (where the facet strip lives, right above the
        // card's top) stays square; the free bottom edge keeps the shared
        // chamfer — CardShape data, not a Cassowary-only render path.
        card_shape: CardShape::Chamfered {
            top_cut_px: 0.0,
            bottom_cut_px: 11.0,
        },
        ..RenderCaps::DEFAULT
    },
};
