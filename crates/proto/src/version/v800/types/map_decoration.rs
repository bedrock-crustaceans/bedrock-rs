use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MapDecoration {
    pub map_decoration_type: MapDecorationType,
    pub rotation: i8,
    pub x: i8,
    pub y: i8,
    pub label: String,
    #[endianness(var)]
    pub color_argb: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
pub enum MapDecorationType {
    MarkerWhite = 0,
    MarkerGreen = 1,
    MarkerRed = 2,
    MarkerBlue = 3,
    XWhite = 4,
    TriangleRed = 5,
    SquareWhite = 6,
    MarkerSign = 7,
    MarkerPink = 8,
    MarkerOrange = 9,
    MarkerYellow = 10,
    MarkerTeal = 11,
    TriangleGreen = 12,
    SmallSquareWhite = 13,
    Mansion = 14,
    Monument = 15,
    NoDraw = 16,
    VillageDesert = 17,
    VillagePlains = 18,
    VillageSavanna = 19,
    VillageSnowy = 20,
    VillageTaiga = 21,
    JungleTemple = 22,
    WitchHut = 23,
    TrialChambers = 24,
    Count = 25,
}

impl MapDecorationType {
    pub const PLAYER: MapDecorationType = MapDecorationType::MarkerWhite;
    pub const PLAYER_OFF_MAP: MapDecorationType = MapDecorationType::SquareWhite;
    pub const PLAYER_OFF_LIMITS: MapDecorationType = MapDecorationType::SmallSquareWhite;
    pub const PLAYER_HIDDEN: MapDecorationType = MapDecorationType::NoDraw;
    pub const ITEM_FRAME: MapDecorationType = MapDecorationType::MarkerGreen;
}
