//! src/theme/icon_ground.rs — the app-icon tile's GROUND, split
//! out of `model.rs` to stay inside its file-size mark. See
//! `scripts/icons/README.md` for how the exporter reads it.

use super::color::Srgb;
use super::model::Theme;

enum_with_all! {
    /// The icon tile's GROUND, as a theme-owned capability: every world's
    /// shipped default is `Base100` (the tile ground is the world's darkest
    /// plane, same as any other summoned surface). A world may opt into one
    /// of the two bounded blends toward its own `base_300` instead — never a
    /// hand-picked hex, never a world-name branch in the exporter, only this
    /// closed preset roster read back through [`Theme::icon_ground_color`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum IconGround {
        Base100,
        Blend25,
        Blend40,
    }
}

impl IconGround {
    pub fn slug(self) -> &'static str {
        match self {
            IconGround::Base100 => "base_100",
            IconGround::Blend25 => "blend25",
            IconGround::Blend40 => "blend40",
        }
    }

    /// How far this state sits from `base_100` toward `base_300`, in `[0, 1]`.
    fn fraction(self) -> f32 {
        match self {
            IconGround::Base100 => 0.0,
            IconGround::Blend25 => 0.25,
            IconGround::Blend40 => 0.40,
        }
    }
}

impl Theme {
    /// The icon tile's actual ground color: `base_100` unless this world opts
    /// into a blend toward `base_300` via [`IconGround`]. The ONE place that
    /// math happens — the exporter's manifest, the packer's pixel laws, and
    /// any live preview all read this rather than restating the fraction.
    pub fn icon_ground_color(&self) -> Srgb {
        self.base_100
            .lerp(self.base_300, self.icon_ground.fraction())
    }
}
