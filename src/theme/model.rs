pub use super::icon_ground::IconGround;
use super::{
    cjk::FontId, color::Srgb, diagonal::DiagonalSpine, ground::Background, ornament::Ornaments,
};
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
    Diagonal(DiagonalSpine),
    /// Carries no fields: every `Bars` world has always shipped the identical
    /// [`BarConfig::SHIPPED`], so the layout dials live on the renderer's own
    /// default rather than being re-authored per world. A dev-only override
    /// (`AWL_OVERLAY_LIST_FORCE`'s `bars:` suffix) can still replace that
    /// default for exploration — see [`BarConfig`].
    Bars,
    /// THE QUIET ONE, organised by ABSENCE: leading and hairline rules do the
    /// arranging, and nothing is drawn as an object. Structurally it is neither
    /// a card ([`ListBacking::BarePlates`], so no panel fill, border or shadow)
    /// nor a plate ([`ListStyle::draws_row_plates`] is false, so no per-row
    /// surface and no scrim) — the only ink the style owns is rules, and its
    /// selection mark is a rule too, on the picker rows and on a summoned
    /// workspace's navigation rail alike. The field carries the selection
    /// treatment. Reached by one carrier world and by
    /// `AWL_OVERLAY_LIST_FORCE=rules:<weight|gutter>`; which further worlds
    /// adopt it is a taste call, not a capability gap.
    Rules(RuleSelection),
}

/// Which of the two credible selection treatments a [`ListStyle::Rules`] world
/// draws. Neither may fill the row: a filled band is `Pane`'s answer, and
/// borrowing it would make this style a restyle of that one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleSelection {
    /// The two rules that bound the selected row THICKEN and run out past the
    /// text measure to the card's full width. The row's interior is untouched
    /// ground.
    Weight,
    /// The row and its rules are untouched; a short heavy rule segment — the
    /// same substance the list is built from — hangs in the gutter beside it.
    Gutter,
}

impl ListStyle {
    /// What backs the CARD — a filled panel, or nothing at all. This says
    /// nothing about what backs a ROW: see [`ListStyle::draws_row_plates`].
    pub fn list_backing(self, _spell: bool) -> ListBacking {
        match self {
            ListStyle::Pane => ListBacking::Card,
            ListStyle::Diagonal(_) => ListBacking::BarePlates,
            ListStyle::Bars => ListBacking::BarePlates,
            // Enclosure is the one thing this style refuses: a card is what it
            // is defined against. The blurred backdrop every world already
            // carries is what its chrome reads against, exactly as `Diagonal`'s
            // does.
            ListStyle::Rules(_) => ListBacking::BarePlates,
        }
    }

    /// Whether this style backs its ROWS with plates. THE BARE-PLATE ROSTER IS
    /// NOT THE PLATE-DRAWING ROSTER: `Diagonal` is `BarePlates` and yet draws
    /// no plate anywhere, its selection being a spine segment and a connector.
    /// A law or a probe about plates asks THIS question, not `list_backing`.
    pub fn draws_row_plates(self) -> bool {
        match self {
            ListStyle::Bars => true,
            // A rule is a boundary, not a surface — `Rules` joins `Diagonal` on
            // the bare-plate roster that draws no plate.
            ListStyle::Pane | ListStyle::Diagonal(_) | ListStyle::Rules(_) => false,
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

/// The `Bars` layout dials, off [`ListStyle::Bars`] since no world has ever
/// varied them: [`BarConfig::SHIPPED`] is the one value every `Bars` world
/// (and, by default, the dev probe) renders with. `radius`/`gap`/`grow_px`
/// and the `extent`/`coverage` axes remain independently forceable
/// (`AWL_OVERLAY_LIST_FORCE`'s `bars:` suffix) precisely because they are
/// tested, working renderer behavior that nothing has disproven — only
/// `BarExtent::FullWidth` combined with a plated-chrome world is proven
/// incompatible (`worlds::KITE`'s own doc comment), which is a composition
/// finding about one world's adoption, not a defect in the renderer axis
/// itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarConfig {
    pub radius: f32,
    pub gap: f32,
    pub grow_px: f32,
    pub extent: BarExtent,
    pub coverage: BarCoverage,
}

impl BarConfig {
    pub const SHIPPED: BarConfig = BarConfig {
        radius: 6.0,
        gap: 10.0,
        grow_px: 24.0,
        extent: BarExtent::HugLabel,
        coverage: BarCoverage::All,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FacetStyle {
    Text,
    Band,
    Chips(ChipVariant),
}

/// **THE SECONDARY-LOCATION HEADING'S OWN TREATMENT** — the shared row
/// planner's `PlanLine::Location` (the active facet's name, the second level
/// of a summoned card's title hierarchy) is data every world reads the SAME
/// plan line through; this says how a world DRAWS it, never whether it
/// exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationStyle {
    /// The default: the label shapes inline, in the row-plan slot it already
    /// occupies (`render/chrome/theme_picker.rs`'s `shape_theme_spans`).
    Inline,
    /// The line stays glyph-free; the label instead draws as a run turned 90°
    /// in the ROOM's own outer margin — the one its wordmark placard keeps —
    /// rising from just above that placard at two thirds its type size and in
    /// its ink: a vertical companion at its own scale class, not a caption in
    /// the card's gutter. Still the SAME one planned slot, and no second
    /// rotation path. The size IS the composition, so no wordmark or too tight
    /// a margin PARKS the cue (`chrome::rotated_location`'s own doc).
    RotatedRail,
    /// The line stays glyph-free like `RotatedRail`, but the label keeps
    /// `Inline`'s own POSITION — flush left wherever the row planner's
    /// diagonal stagger already puts it (a `Diagonal` world carries no
    /// attachment inset on its location row, so this is already the card's
    /// own left edge) — and runs along the SAME rake the diagonal spine steps
    /// through its rows instead of upright, in a gradient between the
    /// spine's own two authored tones (`muted` at rest, `base_content`
    /// selected). The angle is DERIVED from the spine's step/row-height
    /// ratio (`render/chrome/diagonal.rs::location_axis_deg`), never pinned,
    /// so the two cannot drift apart. Reuses the same rotated-label
    /// capability and preparation owner as `RotatedRail` — only the flush
    /// edge, the axis, and the two colours differ.
    Raked,
}

impl LocationStyle {
    /// Whether `PlanLine::Location` shapes into the panel's own inline
    /// rich-text run. Every OTHER style paints the location itself, through
    /// the rotated-label capability, and needs the inline slot left
    /// glyph-free so the two attempts don't stack — exhaustive so a future
    /// style is a conscious decision here, not a silent inline draw.
    pub fn draws_inline(self) -> bool {
        match self {
            Self::Inline => true,
            Self::RotatedRail | Self::Raked => false,
        }
    }
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
    /// A LENGTH: the frame's stroke, typed so it meets the display scale.
    /// Untyped it was multiplied by nothing whatever — a 2px frame stayed two
    /// DEVICE px beside text that had doubled.
    Line {
        weight_px: crate::render::Logical,
    },
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
    pub location_style: LocationStyle,
    pub pane_split: PaneSplit,
    pub ambient: AmbientStyle,
    /// How far below the glyph cell the spell squiggle's band hangs. A LENGTH,
    /// typed so it meets the display scale the wave's own amplitude, period and
    /// thickness meet — untyped it rode the reader's zoom alone, hanging a
    /// half-size gap under a correctly-doubled wave on a dense panel.
    pub spell_underline_gap: crate::render::Logical,
    pub frost: Frost,
    pub fold_afford: FoldAfford,
    pub card_texture: CardTexture,
    pub card_shape: CardShape,
}

pub const SPELL_UNDERLINE_GAP_DEFAULT: crate::render::Logical = crate::render::Logical(1.0);

/// Bilby's tighter dial, two logical px in from the shared default — the world
/// whose report was that the squiggle floated too far below the baseline. Named
/// rather than spelled as arithmetic at the world literal, because a `Logical`
/// deliberately carries no arithmetic of its own.
pub const SPELL_UNDERLINE_GAP_TIGHT: crate::render::Logical =
    crate::render::Logical(SPELL_UNDERLINE_GAP_DEFAULT.0 - 2.0);

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
        location_style: LocationStyle::Inline,
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
    /// The wash that covers TEXT: the document's own selection band, shared by
    /// search-match highlights and by the menu-title highlight that reuses the
    /// same `highlight_treatment`. Authored per world, translucent, and held to
    /// a measured legibility floor against the ink it covers.
    pub selection_document: Srgb,
    /// The band under a SELECTED ROW in a summoned surface (picker, palette,
    /// menu list). `None` — the shape every world ships — means DERIVED:
    /// `base_200` climbed a fixed number of steps toward `base_300`, which is
    /// what makes the band a value step and never a new hue (DESIGN §3/§5) BY
    /// CONSTRUCTION rather than by a law someone has to write and enforce. A
    /// world that authors a colour here opts out of that guarantee knowingly.
    pub selection_ui: Option<Srgb>,
    pub background: Background,
    pub font: &'static str,
    pub mono: &'static str,
    pub icon_cursor: IconCursor,
    pub icon_ground: IconGround,
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
        self.background.is_lava()
            || self.background.is_warped_grid()
            || self.render_caps.ambient.is_animated()
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
