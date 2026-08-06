use bedrock_level::color::Color;
use facet::Facet;

/// Stored as a `String` tag holding the pattern's name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(str))]
#[repr(u8)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct BannerPattern {
    pub color: Color,
    pub pattern: BannerPatternType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Banner {
    /// Base colour of the banner.
    pub base: Color,
    #[facet(rename = "Type")]
    pub ty: i32,
    /// The patterns displayed on this banner.
    ///
    /// May not exist.
    pub patterns: Option<Vec<BannerPattern>>,
}
