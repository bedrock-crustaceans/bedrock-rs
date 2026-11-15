//! Colour and weather components: `minecraft:foliage_appearance`,
//! `minecraft:grass_appearance`, `minecraft:dry_foliage_color`, `minecraft:sky_color`,
//! `minecraft:water_appearance` and `minecraft:precipitation`.

use facet::Facet;
use facet_value::VObject;

use crate::codes;
use crate::color::{Color255Rgb, colour};
use bedrock_addon_core::diagnostic::Code;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Disposition, Node, Read};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx, SchemaFields};
use bedrock_addon_core::shared::ShapeErrorKind;
use bedrock_addon_core::version::FormatVersion;

/// The member that names a colour map.
const COLOR_MAP: &str = "color_map";

/// The first format whose foliage colour maps have `dry_foliage`.
const DRY_FOLIAGE_MAP: FormatVersion = FormatVersion::new(1, 21, 70);

/// A colour, or the name of one of the game's colour maps: `M` says which maps.
///
/// JSON: a colour, `"#RRGGBB"` or `[r, g, b]`, or `{"color_map": "<name>"}` (other members are
/// kept as unknown ones).
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum ColorOrMap<M: ColorMap> {
    /// A fixed colour: `"#RRGGBB"` or `[r, g, b]`.
    Rgb(Color255Rgb),
    /// `{"color_map": "<name>"}`: the colour comes from a colour map by the biome's climate.
    Map {
        /// JSON `color_map`.
        color_map: M,
        /// Unknown members of the object.
        extra: UnknownMembers,
    },
}

/// The colour of leaves: a fixed colour or a foliage colour map.
pub type FoliageColor = ColorOrMap<FoliageColorMap>;

/// The colour of grass: a fixed colour or a grass colour map.
pub type GrassColor = ColorOrMap<GrassColorMap>;

/// A colour map the game names: [`FoliageColorMap`] or [`GrassColorMap`].
pub trait ColorMap: for<'a> Facet<'a> + FromJson + Sized {
    /// The JSON name.
    fn name(&self) -> &'static str;

    /// Reads the map named at `node`, in the format the read is of.
    fn read_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        Self::from_json(node, read)
    }
}

impl ColorMap for FoliageColorMap {
    fn name(&self) -> &'static str {
        FoliageColorMap::name(self)
    }

    /// `dry_foliage` is a value of the enum from 1.21.70; below it, an unknown name.
    fn read_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let map = Self::from_json(node, read)?;
        if map == Self::DryFoliage
            && read
                .known_versions()
                .format
                .is_some_and(|v| v < DRY_FOLIAGE_MAP)
        {
            read.refuse(node, Code::from(&core_codes::INVALID_ENUM_VALUE));
            return None;
        }
        Some(map)
    }
}

impl ColorMap for GrassColorMap {
    fn name(&self) -> &'static str {
        GrassColorMap::name(self)
    }
}

impl<M: ColorMap> FromJson for ColorOrMap<M> {
    /// A colour is told as the game words it (`colour`); the object form needs `color_map`,
    /// and keeps and tells every other member.
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if !node.value().is_object() {
            return colour(node, read).map(Self::Rgb);
        }
        let members = read.object(node)?;
        let color_map = read
            .required(&members, COLOR_MAP)
            .and_then(|node| M::read_name(&node, read));
        let mut extra = UnknownMembers::default();
        for (key, value) in read.unknown(&members, &[COLOR_MAP], Disposition::Keep) {
            extra.insert(key.to_owned(), value.clone());
        }
        Some(Self::Map {
            color_map: color_map?,
            extra,
        })
    }
}

/// The facet proxy's read, which the writer's round trip uses: the kit's read of the value.
impl<M: ColorMap> TryFrom<Value> for ColorOrMap<M> {
    type Error = ShapeErrorKind;

    fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
        let mut told = bedrock_addon_core::diagnostic::Diagnostics::default();
        let mut read = Read::new(crate::reading(), &mut told);
        let node = Node::new(&value, bedrock_addon_core::diagnostic::JsonPath::new());
        Self::from_json(&node, &mut read).ok_or(ShapeErrorKind::UnknownName)
    }
}

impl<M: ColorMap> TryFrom<&ColorOrMap<M>> for Value {
    type Error = ShapeErrorKind;

    fn try_from(color: &ColorOrMap<M>) -> Result<Self, ShapeErrorKind> {
        match color {
            ColorOrMap::Rgb(rgb) => Value::try_from(rgb),
            ColorOrMap::Map { color_map, extra } => {
                let mut object = VObject::new();
                object.insert(COLOR_MAP, Value::from(color_map.name()));
                for (key, member) in extra.iter() {
                    object.insert(key, member.clone());
                }
                Ok(object.into())
            }
        }
    }
}

/// A colour reads its own syntax: no Molang, no bound, no trigger.
impl<M: ColorMap> MolangFields for ColorOrMap<M> {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl<M: ColorMap> SchemaFields for ColorOrMap<M> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

/// A foliage colour map (JSON `color_map` of `minecraft:foliage_appearance`); the game knows
/// six, numbered 0 to 5 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case", schema::string_only)]
#[repr(u8)]
#[non_exhaustive]
pub enum FoliageColorMap {
    /// `foliage`.
    Foliage,
    /// `birch`.
    Birch,
    /// `evergreen`.
    Evergreen,
    /// `mangrove_swamp_foliage`.
    MangroveSwampFoliage,
    /// `swamp_foliage`.
    SwampFoliage,
    /// `dry_foliage` (format 1.21.70).
    DryFoliage,
}

/// A grass colour map (JSON `color_map` of `minecraft:grass_appearance`); the game knows two,
/// numbered 0 and 1 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case", schema::string_only)]
#[repr(u8)]
#[non_exhaustive]
pub enum GrassColorMap {
    /// `grass`.
    Grass,
    /// `swamp_grass`.
    SwampGrass,
}

/// `minecraft:foliage_appearance`: the colour of the biome's leaves.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FoliageAppearance {
    /// JSON `color`; absent leaves the game's colour.
    #[facet(schema::wrong_type = codes::COLOR)]
    pub color: Option<FoliageColor>,
}

/// `minecraft:grass_appearance`: the colour of the biome's grass.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GrassAppearance {
    /// JSON `color`; absent leaves the game's colour.
    #[facet(schema::wrong_type = codes::COLOR)]
    pub color: Option<GrassColor>,
    /// JSON `grass_is_shaded` (format 1.21.100); read it through
    /// [`grass_is_shaded`](Self::grass_is_shaded).
    /// Absent, grass is not darkened as under a dense canopy (the game fills an absent member
    /// with zero).
    #[facet(schema::fallback = false)]
    pub grass_is_shaded: Option<bool>,
}

/// `minecraft:dry_foliage_color`: the colour of leaf litter and other dry foliage.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DryFoliageColor {
    /// JSON `color`.
    #[facet(schema::read_with = colour, schema::wrong_type = codes::COLOR)]
    pub color: Color255Rgb,
}

/// `minecraft:sky_color`: the colour of the sky.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SkyColor {
    /// JSON `sky_color`.
    #[facet(schema::read_with = colour, schema::wrong_type = codes::COLOR)]
    pub sky_color: Color255Rgb,
}

/// `minecraft:water_appearance`: the colour and opacity of the water's surface. At least one
/// member is needed.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct WaterAppearance {
    /// JSON `surface_color`.
    #[facet(schema::read_with = colour, schema::wrong_type = codes::COLOR)]
    pub surface_color: Option<Color255Rgb>,
    /// JSON `surface_opacity`, from 0 to 1.
    pub surface_opacity: Option<f32>,
}

/// `minecraft:precipitation`: the particles that fall in the biome and how dense they are. At
/// most one member may be present.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Precipitation {
    /// JSON `ash`.
    pub ash: Option<f32>,
    /// JSON `blue_spores`.
    pub blue_spores: Option<f32>,
    /// JSON `red_spores`.
    pub red_spores: Option<f32>,
    /// JSON `white_ash`.
    pub white_ash: Option<f32>,
}
