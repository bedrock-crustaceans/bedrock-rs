use bedrock_level::color::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct BannerPattern {
    pub color: Color,
    pub pattern: BannerPatternType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Banner {
    /// Base colour of the banner.
    pub base: Color,
    #[serde(rename = "Type")]
    pub ty: i32,
    /// The patterns displayed on this banner.
    ///
    /// May not exist.
    pub patterns: Option<Vec<BannerPattern>>,
}
