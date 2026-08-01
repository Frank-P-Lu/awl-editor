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
