/// The authored mirror of the shared diagonal row composition. Geometry and
/// paint remain owned by the composition; a world supplies only orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagonalDirection {
    /// A descending `\` spine whose successive row starts move right.
    Descending,
    /// An ascending `/` spine whose successive row ends move left.
    Ascending,
}

impl DiagonalDirection {
    pub const fn sign(self) -> f32 {
        match self {
            Self::Descending => 1.0,
            Self::Ascending => -1.0,
        }
    }
}

/// THE SELECTED ROW'S MARK, authored per world BESIDE ITS DISPLAY FACE — because
/// one mark cannot serve two registers. A crisp geometric chevron is right on a
/// technical mono face and contradicts an editorial slab serif, where the same
/// stroke reads as a blunt object dropped beside the text. So weight and form are
/// world data, exactly like `Theme::font`, rather than one shared renderer
/// constant tuned until whichever world was last looked at stopped complaining.
///
/// `weight` and `reach` are LOGICAL pixels, resolved through the composition's
/// one `zoom * dpi` boundary. `aperture` is a fraction of the row's own inset
/// height, so a narrow mark stays a fraction of its row at every scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiagonalMark {
    /// The mark's stroke weight.
    pub weight: f32,
    /// Half the mark's horizontal extent: from its vertex to its arm line.
    pub reach: f32,
    /// How much of the row's inset height the arms span, in `(0, 1]`.
    pub aperture: f32,
}

impl DiagonalMark {
    /// A TECHNICAL face's mark: full-height, geometric, unmistakably drawn.
    pub const CRISP: DiagonalMark = DiagonalMark {
        weight: 3.0,
        reach: 5.0,
        aperture: 1.0,
    };

    /// AN EDITORIAL face's mark: a hairline at half the row's height, sized to
    /// read as a typographic reference mark rather than as a fitting.
    pub const HAIRLINE: DiagonalMark = DiagonalMark {
        weight: 1.25,
        reach: 4.5,
        aperture: 0.55,
    };
}

/// A world's whole diagonal authorship: which way its spine rakes, and the mark
/// its selected row carries. Carried as ONE payload on `ListStyle::Diagonal` so
/// a world cannot author an orientation without a mark, and no world off that
/// composition can carry mark data that nothing reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiagonalSpine {
    pub direction: DiagonalDirection,
    pub mark: DiagonalMark,
}

impl DiagonalSpine {
    pub const fn descending(mark: DiagonalMark) -> Self {
        Self {
            direction: DiagonalDirection::Descending,
            mark,
        }
    }

    pub const fn ascending(mark: DiagonalMark) -> Self {
        Self {
            direction: DiagonalDirection::Ascending,
            mark,
        }
    }
}
