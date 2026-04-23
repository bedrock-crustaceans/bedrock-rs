use crate::color::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BannerPatternType {
    Base,
    BottomStripe,
    TopStripe,
    LeftStripe,
    RightStripe,
    CenterStripe,
    MiddleStripe,
    DownRightStripe,
    DownLeftStripe,
    SmallStripes,
    DiagonalCross,
    SquareCross,
    LeftOfDiagonal,
    RightOfUpsideDownDiagonal,
    LeftOfUpsideDownDiagonal,
    RightOfDiagonal,
    VerticalHalf,
    VerticalHalfRight,
    HorizontalHalf,
    HorizontalHalfBottom,
    BottomLeftCorner,
    TopLeftCorner,
    TopRightCorner,
    BottomTriangle,
    TopTriangle,
    BottomTriangleSawtooth,
    TopTriangleSawtooth,
    MiddleCircle,
    MiddleRhombus,
    Border,
    CurlyBorder,
    Brick,
    Gradient,
    GradientUpsideDown,
    Creeper,
    Skull,
    Flower,
    Mojang,
    Globe,
    Piglin,
    Flow,
    Guster,
}

mod pattern {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BannerPattern {
    pub color: Color,
    #[serde(with = "pattern")]
    pub pattern: BannerPatternType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Banner {
    /// Base colour of the banner.
    pub base: Color,
    /// The patterns displayed on this banner.
    ///
    /// May not exist.
    pub patterns: Option<Vec<BannerPattern>>,
}
