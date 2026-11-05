//! Value types several block components share: faces, boxes, colours, tint and render methods.

use facet::Facet;

use bedrock_addon_core::schema::{self, Schema};

/// Three floats `[x, y, z]`.
pub type Vec3 = [f32; 3];

/// One of the six block faces as the game names them.
///
/// JSON: `down`, `up`, `north`, `south`, `west`, `east`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Facing {
    /// `down`.
    Down,
    /// `up`.
    Up,
    /// `north`.
    North,
    /// `south`.
    South,
    /// `west`.
    West,
    /// `east`.
    East,
}

impl Facing {
    /// The six faces in vanilla's order.
    pub const ALL: [Facing; 6] = [
        Facing::Down,
        Facing::Up,
        Facing::North,
        Facing::South,
        Facing::West,
        Facing::East,
    ];
}

/// A horizontal direction as the game names them.
///
/// JSON: `north`, `south`, `east`, `west`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Direction {
    /// `north`.
    North,
    /// `south`.
    South,
    /// `east`.
    East,
    /// `west`.
    West,
}

impl Direction {
    /// The four directions.
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
}

/// An axis-aligned box in sixteenths of a block.
///
/// JSON: `{"origin": [x, y, z], "size": [x, y, z]}`; both optional. The origin is relative to the
/// block's bottom centre, so a full block is `origin [-8, 0, -8]`, `size [16, 16, 16]`: the
/// values of [`origin()`](Self::origin) and [`size()`](Self::size) for an absent member, so the
/// default box (both `None`) is the full block.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct BoundingBox {
    /// The minimum corner (JSON `origin`); each axis in `-8..=8`.
    pub origin: Option<Vec3>,
    /// The extent (JSON `size`); each axis in `-16..=16`.
    pub size: Option<Vec3>,
}

impl BoundingBox {
    /// The minimum corner: `origin`, `[-8, 0, -8]` when absent.
    pub fn origin(&self) -> Vec3 {
        self.origin.unwrap_or(vec3(-8.0, 0.0, -8.0))
    }

    /// The extent: `size`, `[16, 16, 16]` when absent.
    pub fn size(&self) -> Vec3 {
        self.size.unwrap_or(vec3(16.0, 16.0, 16.0))
    }
}

/// How a tinted texture takes its colour from the biome.
///
/// JSON: `none`, `default_foliage`, `birch_foliage`, `evergreen_foliage`, `dry_foliage`,
/// `grass`, `water`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum TintMethod {
    /// `none`: no tint.
    #[default]
    None,
    /// `default_foliage`.
    DefaultFoliage,
    /// `birch_foliage`.
    BirchFoliage,
    /// `evergreen_foliage`.
    EvergreenFoliage,
    /// `dry_foliage`.
    DryFoliage,
    /// `grass`.
    Grass,
    /// `water`.
    Water,
}

/// How a material instance is rendered.
///
/// JSON: `double_sided`, `blend`, `blend_to_opaque`, `opaque`, `alpha_test_to_opaque`,
/// `alpha_test`, `alpha_test_single_sided`, `alpha_test_single_sided_to_opaque`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum RenderMethod {
    /// `double_sided`.
    DoubleSided,
    /// `blend`.
    Blend,
    /// `blend_to_opaque`.
    BlendToOpaque,
    /// `opaque`: the default.
    #[default]
    Opaque,
    /// `alpha_test_to_opaque`.
    AlphaTestToOpaque,
    /// `alpha_test`.
    AlphaTest,
    /// `alpha_test_single_sided`.
    AlphaTestSingleSided,
    /// `alpha_test_single_sided_to_opaque`.
    AlphaTestSingleSidedToOpaque,
}

/// A map colour: `"#rrggbb"` or `[r, g, b]`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum Color {
    /// A hex string.
    Hex(String),
    /// Three channels.
    Rgb([u8; 3]),
}

/// A float range `{min, max}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet, Schema)]
pub struct Bounds {
    /// JSON `min`.
    pub min: f32,
    /// JSON `max`.
    pub max: f32,
}

// The faces and directions keep their own `ALL` arrays (the direction's in the game's order), which
// the string-enum derive would declare again, and a colour is a hex string or three channels:
// leaves of the schema walk, with no bound and no Molang.
bedrock_addon_core::impl_schema_leaf!(Facing, Direction, Color);
bedrock_addon_core::impl_no_molang!(Facing, Direction, Color);

/// `[x, y, z]` as a [`Vec3`].
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    [x, y, z]
}
