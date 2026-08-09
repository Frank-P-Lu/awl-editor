use super::{color::Srgb, model::Theme};

/// A role in the world's authored token palette. Treatments refer to roles,
/// never copied colour literals, so changing a token re-resolves every surface
/// that uses it on the next theme sync.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteRole {
    Base100,
    Base200,
    Base300,
    BaseContent,
    Muted,
    Faint,
    Primary,
    PrimaryContent,
    Error,
    SelectionDocument,
}

impl PaletteRole {
    pub fn resolve(self, theme: &Theme) -> Srgb {
        match self {
            Self::Base100 => theme.base_100,
            Self::Base200 => theme.base_200,
            Self::Base300 => theme.base_300,
            Self::BaseContent => theme.base_content,
            Self::Muted => theme.muted,
            Self::Faint => theme.faint,
            Self::Primary => theme.primary,
            Self::PrimaryContent => theme.primary_content,
            Self::Error => theme.error,
            Self::SelectionDocument => theme.selection_document,
        }
    }
}

/// The two palette roles an inverse treatment swaps. The same resolver serves
/// document selection and a block caret, while each capability chooses its own
/// pair independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TwoColour {
    pub ground: PaletteRole,
    pub ink: PaletteRole,
}

impl TwoColour {
    pub const fn new(ground: PaletteRole, ink: PaletteRole) -> Self {
        Self { ground, ink }
    }

    pub fn resolve(self, theme: &Theme) -> ResolvedTwoColour {
        ResolvedTwoColour {
            ground: self.ground.resolve(theme),
            ink: self.ink.resolve(theme),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedTwoColour {
    pub ground: Srgb,
    pub ink: Srgb,
}
