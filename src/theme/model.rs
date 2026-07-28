use super::cjk::FontId;
use super::color::Srgb;
use super::ornament::Ornaments;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleOverrides {
    pub def_fg: Option<Srgb>,
    pub const_fg: Option<Srgb>,
    pub str_fg: Option<Srgb>,
    pub comment_wash: WashOverride,
    pub str_wash: WashOverride,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WashOverride {
    Default,
    Off,
    Pin(Srgb),
}

impl RoleOverrides {
    pub const NONE: RoleOverrides = RoleOverrides {
        def_fg: None,
        const_fg: None,
        str_fg: None,
        comment_wash: WashOverride::Default,
        str_wash: WashOverride::Default,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Renderers consume per-theme capabilities as data, never world names.
pub enum SelectionStyle {
    Fill,
    InverseVideo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretBlockStyle {
    Normal,
    InverseVideo,
    Filled,
}

impl CaretBlockStyle {
    pub fn folds_morph_to_block(self) -> bool {
        !matches!(self, CaretBlockStyle::Normal)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backdrop {
    Blur,
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    Flat,
    Recessed,
    Bordered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorativeWash {
    Enabled,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageReveal {
    Translucent,
    Opaque,
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleStyle {
    InlinePrefix,
    Placard {
        corner: PlacardCorner,
        scale: f32,
        ink: PlacardInk,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAnchor {
    TopLeft,
    TopCenter,
    Inset { x_frac: f32 },
    TopRight,
}

impl CardAnchor {
    pub fn mirrors_growth(self) -> bool {
        matches!(self, CardAnchor::TopRight)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ListStyle {
    Pane,
    Bars {
        radius: f32,
        gap: f32,
        grow_px: f32,
        extent: BarExtent,
        coverage: BarCoverage,
    },
}

impl ListStyle {
    pub fn list_backing(self, _spell: bool) -> ListBacking {
        match self {
            ListStyle::Pane => ListBacking::Card,
            ListStyle::Bars { .. } => ListBacking::BarePlates,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ListBacking {
    Card,
    BarePlates,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaneSplit {
    Unified,
    Split,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarExtent {
    FullWidth,
    HugText,
    HugLabel,
}

impl BarExtent {
    pub fn hugs(self) -> bool {
        matches!(self, BarExtent::HugText | BarExtent::HugLabel)
    }

    pub fn inline_shortcut(self) -> bool {
        matches!(self, BarExtent::HugText)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarCoverage {
    All,
    SelectedOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FacetStyle {
    Text,
    Band,
    Chips(ChipVariant),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipVariant {
    Hairline,
    FilledActive,
    Underline,
    Bracket,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageFrame {
    None,
    Line { weight_px: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChromeFace {
    Body,
    Named(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayEntrance {
    Instant,
    SpringIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandResponse {
    Snap,
    Slide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionJuice {
    pub entrance: OverlayEntrance,
    pub band: BandResponse,
}

impl MotionJuice {
    pub const CALM: MotionJuice = MotionJuice {
        entrance: OverlayEntrance::Instant,
        band: BandResponse::Snap,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HighlightTexture {
    Wash,
    Stipple { color: Srgb, density: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AmbientStyle {
    None,
    Stars {
        tint: Srgb,
        cell_px: f32,
        density: f32,
        size_px: f32,
        peak: f32,
        floor: f32,
    },
}

impl AmbientStyle {
    pub fn is_animated(&self) -> bool {
        matches!(self, AmbientStyle::Stars { .. })
    }
    pub fn stars_params(&self) -> Option<(Srgb, f32, f32, f32, f32, f32)> {
        match self {
            AmbientStyle::Stars {
                tint,
                cell_px,
                density,
                size_px,
                peak,
                floor,
            } => Some((*tint, *cell_px, *density, *size_px, *peak, *floor)),
            AmbientStyle::None => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            AmbientStyle::None => "none",
            AmbientStyle::Stars { .. } => "stars",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderCaps {
    pub selection_style: SelectionStyle,
    pub caret_block_style: CaretBlockStyle,
    pub backdrop: Backdrop,
    pub elevation: Elevation,
    pub decorative_wash: DecorativeWash,
    pub image_reveal: ImageReveal,
    pub highlight_texture: HighlightTexture,
    pub title_style: TitleStyle,
    pub page_frame: PageFrame,
    pub card_anchor: CardAnchor,
    pub chrome_face: ChromeFace,
    pub motion: MotionJuice,
    pub list_style: ListStyle,
    pub facet_style: FacetStyle,
    pub pane_split: PaneSplit,
    pub ambient: AmbientStyle,
    pub spell_underline_gap: f32,
    pub frost: Frost,
    pub fold_afford: FoldAfford,
    pub card_texture: CardTexture,
    pub card_shape: CardShape,
}

pub const SPELL_UNDERLINE_GAP_DEFAULT: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frost {
    pub dim: f32,
    pub blur_px: f32,
    pub feather_px: f32,
}

impl Frost {
    pub const DEFAULT: Frost = Frost {
        dim: crate::lava::FROST_DIM,
        blur_px: crate::lava::FROST_BLUR_PX,
        feather_px: crate::lava::FROST_FEATHER_PX,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldAfford {
    pub chevron_lift: f32,
    pub tail_lift: f32,
}

impl FoldAfford {
    pub const DEFAULT: FoldAfford = FoldAfford {
        chevron_lift: 0.0,
        tail_lift: 0.0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardTexture {
    Flat,
    HalftoneDots {
        angle_deg: f32,
        cell_px: f32,
        density: f32,
    },
}

impl CardTexture {
    pub const DEFAULT: CardTexture = CardTexture::Flat;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardShape {
    Rectangular,
    Chamfered { cut_px: f32 },
}

impl CardShape {
    pub const DEFAULT: CardShape = CardShape::Rectangular;
}

impl RenderCaps {
    pub const DEFAULT: RenderCaps = RenderCaps {
        selection_style: SelectionStyle::Fill,
        caret_block_style: CaretBlockStyle::Normal,
        backdrop: Backdrop::Blur,
        elevation: Elevation::Flat,
        decorative_wash: DecorativeWash::Enabled,
        image_reveal: ImageReveal::Translucent,
        highlight_texture: HighlightTexture::Wash,
        title_style: TitleStyle::InlinePrefix,
        page_frame: PageFrame::None,
        card_anchor: CardAnchor::TopCenter,
        chrome_face: ChromeFace::Body,
        motion: MotionJuice::CALM,
        list_style: ListStyle::Pane,
        facet_style: FacetStyle::Text,
        pane_split: PaneSplit::Split,
        ambient: AmbientStyle::None,
        spell_underline_gap: SPELL_UNDERLINE_GAP_DEFAULT,
        frost: Frost::DEFAULT,
        fold_afford: FoldAfford::DEFAULT,
        card_texture: CardTexture::DEFAULT,
        card_shape: CardShape::DEFAULT,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum HighlightTreatment {
    ValueBand(Srgb),
    InverseFill { band: Srgb, ink: Srgb },
}

#[cfg(test)]
pub const ZIGZAG_STROKE_FRAC: f32 = 0.10;
#[cfg(test)]
pub const ZIGZAG_MIN_STROKE_PX: f32 = 1.2;
#[cfg(test)]
pub const ZIGZAG_MAX_ROW_PITCH_PX: f32 = 160.0;

#[derive(Clone, Copy, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Background {
    Gradient { from: Srgb, to: Srgb, dir: (f32, f32) },
    Dots { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb, edge: bool },
    Starfield { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb },
    Pinstripe { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb },
    Stripes { from: Srgb, to: Srgb, band: Srgb, angle: f32 },
    Lava {
        ground: Srgb,
        blob_lo: Srgb,
        blob_hi: Srgb,
        edge: LavaEdge,
        dithered: bool,
    },
    Bands { tones: [Srgb; 3], angle: f32 },
    Waves { tones: [Srgb; 3] },
    Zigzag { from: Srgb, to: Srgb, dir: (f32, f32), tint: Srgb,
        period_px: f32, amplitude_px: f32, angle: f32, density: f32, banded: bool },
    Organic { tones: [Srgb; 3], scale_px: f32, density: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LavaEdge {
    Hard,
    Glow,
}

impl LavaEdge {
    pub fn mask_mode(self) -> f32 {
        match self {
            LavaEdge::Hard => 1.0,
            LavaEdge::Glow => 2.0,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            LavaEdge::Hard => "hard",
            LavaEdge::Glow => "glow",
        }
    }
}

impl Background {
    pub fn shader_id(&self) -> u32 {
        match self {
            Background::Gradient { .. } => 0,
            Background::Dots { .. } => 1,
            Background::Starfield { .. } => 2,
            Background::Pinstripe { .. } => 3,
            Background::Stripes { .. } => 4,
            Background::Lava { .. } => 0,
            Background::Bands { .. } => 5,
            Background::Waves { .. } => 6,
            Background::Zigzag { .. } => 7,
            Background::Organic { .. } => 8,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Background::Gradient { .. } => "gradient",
            Background::Dots { .. } => "dots",
            Background::Starfield { .. } => "starfield",
            Background::Pinstripe { .. } => "pinstripe",
            Background::Stripes { .. } => "stripes",
            Background::Lava { .. } => "lava",
            Background::Bands { .. } => "bands",
            Background::Waves { .. } => "waves",
            Background::Zigzag { .. } => "zigzag",
            Background::Organic { .. } => "organic",
        }
    }
    pub fn from(&self) -> Srgb {
        match self {
            Background::Gradient { from, .. }
            | Background::Dots { from, .. }
            | Background::Starfield { from, .. }
            | Background::Pinstripe { from, .. }
            | Background::Stripes { from, .. }
            | Background::Zigzag { from, .. } => *from,
            Background::Lava { ground, .. } => *ground,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[0],
        }
    }
    pub fn to(&self) -> Srgb {
        match self {
            Background::Gradient { to, .. }
            | Background::Dots { to, .. }
            | Background::Starfield { to, .. }
            | Background::Pinstripe { to, .. }
            | Background::Stripes { to, .. }
            | Background::Zigzag { to, .. } => *to,
            Background::Lava { ground, .. } => *ground,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[2],
        }
    }
    pub fn dir(&self) -> (f32, f32) {
        match self {
            Background::Gradient { dir, .. }
            | Background::Dots { dir, .. }
            | Background::Starfield { dir, .. }
            | Background::Pinstripe { dir, .. }
            | Background::Zigzag { dir, .. } => *dir,
            Background::Stripes { angle, .. } | Background::Bands { angle, .. } => {
                (angle.cos(), angle.sin())
            }
            Background::Lava { .. } | Background::Waves { .. } | Background::Organic { .. } => {
                (0.0, 1.0)
            }
        }
    }
    pub fn tint(&self) -> Srgb {
        match self {
            Background::Dots { tint, .. }
            | Background::Starfield { tint, .. }
            | Background::Pinstripe { tint, .. }
            | Background::Zigzag { tint, .. } => *tint,
            Background::Stripes { band, .. } => *band,
            Background::Gradient { from, .. } => *from,
            Background::Lava { ground, .. } => *ground,
            Background::Bands { tones, .. }
            | Background::Waves { tones }
            | Background::Organic { tones, .. } => tones[1],
        }
    }
    pub fn edge(&self) -> bool {
        matches!(self, Background::Dots { edge: true, .. })
    }
    pub fn angle(&self) -> f32 {
        match self {
            Background::Stripes { angle, .. }
            | Background::Bands { angle, .. }
            | Background::Zigzag { angle, .. } => *angle,
            _ => 0.0,
        }
    }
    pub fn period_px(&self) -> f32 {
        match self {
            Background::Zigzag { period_px, .. } => *period_px,
            Background::Organic { scale_px, .. } => *scale_px,
            _ => 0.0,
        }
    }
    pub fn amplitude_px(&self) -> f32 {
        match self {
            Background::Zigzag { amplitude_px, .. } => *amplitude_px,
            _ => 0.0,
        }
    }
    #[cfg(test)]
    pub fn zigzag_stroke_px(&self) -> f32 {
        (self.amplitude_px() * ZIGZAG_STROKE_FRAC).max(ZIGZAG_MIN_STROKE_PX)
    }
    #[cfg(test)]
    pub fn zigzag_row_pitch_px(&self) -> f32 {
        2.0 * self.amplitude_px() + self.zigzag_stroke_px()
    }
    pub fn density(&self) -> f32 {
        match self {
            Background::Zigzag { density, .. } => *density,
            Background::Organic { density, .. } => *density,
            _ => 0.0,
        }
    }
    pub fn zigzag_banded(&self) -> bool {
        matches!(self, Background::Zigzag { banded: true, .. })
    }
    pub fn is_lava(&self) -> bool {
        matches!(self, Background::Lava { .. })
    }
    pub fn is_waves(&self) -> bool {
        matches!(self, Background::Waves { .. })
    }
    pub fn is_organic(&self) -> bool { matches!(self, Background::Organic { .. }) }
    pub fn lava_params(&self) -> Option<(Srgb, Srgb, Srgb, LavaEdge, bool)> {
        match self {
            Background::Lava {
                ground,
                blob_lo,
                blob_hi,
                edge,
                dithered,
            } => Some((*ground, *blob_lo, *blob_hi, *edge, *dithered)),
            _ => None,
        }
    }
}

enum_with_all! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum IconCursor {
        Block,
        Pill,
        Narrow,
    }
}

impl IconCursor {
    pub fn slug(self) -> &'static str {
        match self {
            IconCursor::Block => "block",
            IconCursor::Pill => "pill",
            IconCursor::Narrow => "narrow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub dark: bool,
    pub base_100: Srgb,
    pub base_200: Srgb,
    pub base_300: Srgb,
    pub base_content: Srgb,
    pub muted: Srgb,
    pub faint: Srgb,
    pub primary: Srgb,
    pub primary_content: Srgb,
    pub error: Srgb,
    pub selection: Srgb,
    pub background: Background,
    pub font: &'static str,
    pub mono: &'static str,
    pub icon_cursor: IconCursor,
    pub heading_bold: bool,
    pub cjk: &'static [&'static str],
    pub zh_hans: &'static [&'static str],
    pub zh_hant: &'static [&'static str],
    pub ko: &'static [&'static str],
    pub ornaments: Ornaments,
    pub ornament_face: &'static str,
    pub ornament_scale: f32,
    pub bullets: (char, char, char),
    pub bullet_scale: f32,
    pub list_indent_scale: f32,
    pub tags: ThemeTags,
    pub role_overrides: RoleOverrides,
    pub render_caps: RenderCaps,
}

impl Theme {
    pub fn highlight_treatment(&self, band: Srgb) -> HighlightTreatment {
        match self.render_caps.selection_style {
            SelectionStyle::Fill => HighlightTreatment::ValueBand(band),
            SelectionStyle::InverseVideo => HighlightTreatment::InverseFill {
                band: self.base_content,
                ink: self.base_300,
            },
        }
    }

    pub fn candidates(&self, id: FontId) -> Vec<&'static str> {
        match id {
            FontId::Latin => vec![self.font],
            FontId::Ja => self.cjk.to_vec(),
            FontId::ZhHans => self.zh_hans.to_vec(),
            FontId::ZhHant => self.zh_hant.to_vec(),
            FontId::Ko => self.ko.to_vec(),
        }
    }

    pub const fn bullet_for_depth(&self, depth: usize) -> char {
        match depth % 3 {
            0 => self.bullets.0,
            1 => self.bullets.1,
            _ => self.bullets.2,
        }
    }

    pub fn is_monochrome(&self) -> bool {
        self.primary.to_hsl().1 <= 0.0
    }

    pub fn is_one_bit(&self) -> bool {
        let pure_bw = |c: Srgb| matches!((c.r, c.g, c.b), (0, 0, 0) | (255, 255, 255));
        self.is_monochrome()
            && pure_bw(self.base_100)
            && pure_bw(self.base_content)
            && pure_bw(self.primary)
    }

    pub fn ink_caret(&self) -> bool {
        self.primary == self.base_content
    }

    pub fn has_ambient_motion(&self) -> bool {
        self.background.is_lava() || self.render_caps.ambient.is_animated()
    }

    pub fn has_ambient_tick(&self) -> bool {
        self.has_ambient_motion() || self.background.is_waves() || self.background.is_organic()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lens {
    Time,
    Register,
    Voice,
    Temperature,
    All,
}

impl Lens {
    pub const STRIP: [Lens; 5] = [
        Lens::All,
        Lens::Time,
        Lens::Register,
        Lens::Voice,
        Lens::Temperature,
    ];

    pub fn sections(self) -> &'static [&'static str] {
        match self {
            Lens::Time => &["Dawn", "Day", "Dusk", "Night"],
            Lens::Register => &["Humble", "Everyday", "Refined"],
            Lens::Voice => &["Literary", "Technical", "Modern"],
            Lens::Temperature => &["Warm", "Cool", "Neutral"],
            Lens::All => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeTags {
    pub time: Option<&'static str>,
    pub register: Option<&'static str>,
    pub voice: Option<&'static str>,
    pub temperature: Option<&'static str>,
}

impl ThemeTags {
    pub fn section(&self, lens: Lens) -> Option<&'static str> {
        match lens {
            Lens::Time => self.time,
            Lens::Register => self.register,
            Lens::Voice => self.voice,
            Lens::Temperature => self.temperature,
            Lens::All => None,
        }
    }
}
