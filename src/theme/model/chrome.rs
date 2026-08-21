#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacardCorner {
    TL,
    TR,
    BL,
    BR,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacardInk {
    Faint,
    Ghost,
    Stipple,
    Muted,
    Bold,
}

/// Placement applied after the placard's ordinary contained corner anchor.
/// `Bleed` is expressed in the placard's own em so the crop scales with the
/// wordmark rather than with an unrelated viewport constant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlacardPlacement {
    Contained,
    Bleed { x_em: f32, y_em: f32 },
}

impl PlacardPlacement {
    pub const DEFAULT: PlacardPlacement = PlacardPlacement::Contained;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleStyle {
    InlinePrefix,
    Placard {
        corner: PlacardCorner,
        scale: f32,
        ink: PlacardInk,
    },
}

/// A static material laid over summoned chrome in absolute canvas space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SummonedMaterial {
    Flat,
    Scanlines {
        pitch_px: f32,
        line_px: f32,
        strength: f32,
    },
}

impl SummonedMaterial {
    pub const DEFAULT: SummonedMaterial = SummonedMaterial::Flat;
}
