//! Colour and weather components: `minecraft:foliage_appearance`,
//! `minecraft:grass_appearance`, `minecraft:dry_foliage_color`, `minecraft:sky_color`,
//! `minecraft:water_appearance` and `minecraft:precipitation`.

use facet::Facet;
use facet_value::VObject;

use super::{Checked, Report};
use crate::color::{Color255Rgb, clamped_channels, is_rgb};
use crate::texts::{COLOR, WATER_EMPTY, fixed};
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, Value};
use bedrock_addon_core::molang::JsonType;
use bedrock_addon_core::shared::ShapeErrorKind;

/// The member that names a colour map.
const COLOR_MAP: &str = "color_map";

/// A colour, or the name of one of the game's colour maps.
macro_rules! color_or_map {
    ($(#[$doc:meta])* $name:ident, $map:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Facet)]
        #[facet(opaque, proxy = Value)]
        #[repr(u8)]
        pub enum $name {
            /// A fixed colour: `"#RRGGBB"` or `[r, g, b]`.
            Rgb(Color255Rgb),
            /// `{"color_map": "<name>"}`: the colour comes from a colour map by the biome's
            /// climate.
            Map {
                /// JSON `color_map`.
                color_map: $map,
                /// Unknown members of the object.
                extra: Extra,
            },
        }

        impl TryFrom<Value> for $name {
            type Error = ShapeErrorKind;

            fn try_from(value: Value) -> Result<Self, ShapeErrorKind> {
                let Some(object) = value.as_object() else {
                    return Color255Rgb::try_from(value).map(Self::Rgb);
                };
                let name = object
                    .get(COLOR_MAP)
                    .ok_or(ShapeErrorKind::MissingMember { member: COLOR_MAP })?;
                let name = name.as_string().ok_or(ShapeErrorKind::WrongType {
                    expected: "a colour map's name",
                    found: JsonType::Object,
                })?;
                let color_map = $map::from_name(name.as_str()).ok_or(ShapeErrorKind::UnknownName)?;
                let mut extra = Extra::default();
                for (key, member) in object {
                    if key.as_str() != COLOR_MAP {
                        extra.insert(key.as_str().to_owned(), member.clone());
                    }
                }
                Ok(Self::Map { color_map, extra })
            }
        }

        impl From<&$name> for Value {
            fn from(color: &$name) -> Self {
                match color {
                    $name::Rgb(rgb) => Value::from(rgb),
                    $name::Map { color_map, extra } => {
                        let mut object = VObject::new();
                        object.insert(COLOR_MAP, Value::from(color_map.name()));
                        for (key, member) in extra.iter() {
                            object.insert(key, member.clone());
                        }
                        object.into()
                    }
                }
            }
        }
    };
}

color_or_map! {
    /// The colour of leaves: a fixed colour or a foliage colour map.
    FoliageColor, FoliageColorMap
}

color_or_map! {
    /// The colour of grass: a fixed colour or a grass colour map.
    GrassColor, GrassColorMap
}

/// A foliage colour map (JSON `color_map` of `minecraft:foliage_appearance`); the game knows
/// six, numbered 0 to 5 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl FoliageColorMap {
    /// Every map with its JSON name, in vanilla's order.
    const NAMES: [(Self, &'static str); 6] = [
        (Self::Foliage, "foliage"),
        (Self::Birch, "birch"),
        (Self::Evergreen, "evergreen"),
        (Self::MangroveSwampFoliage, "mangrove_swamp_foliage"),
        (Self::SwampFoliage, "swamp_foliage"),
        (Self::DryFoliage, "dry_foliage"),
    ];

    /// The JSON name.
    pub fn name(self) -> &'static str {
        Self::NAMES
            .iter()
            .find(|(map, _)| *map == self)
            .map_or("", |(_, name)| name)
    }

    /// The map named `name`.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(_, n)| *n == name)
            .map(|(map, _)| *map)
    }
}

/// A grass colour map (JSON `color_map` of `minecraft:grass_appearance`); the game knows two,
/// numbered 0 and 1 in the order listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GrassColorMap {
    /// `grass`.
    Grass,
    /// `swamp_grass`.
    SwampGrass,
}

impl GrassColorMap {
    /// The JSON name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Grass => "grass",
            Self::SwampGrass => "swamp_grass",
        }
    }

    /// The map named `name`.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "grass" => Some(Self::Grass),
            "swamp_grass" => Some(Self::SwampGrass),
            _ => None,
        }
    }
}

/// `minecraft:foliage_appearance`: the colour of the biome's leaves.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct FoliageAppearance {
    /// JSON `color`; absent leaves the game's colour.
    #[facet(default)]
    pub color: Option<FoliageColor>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:grass_appearance`: the colour of the biome's grass.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct GrassAppearance {
    /// JSON `color`; absent leaves the game's colour.
    #[facet(default)]
    pub color: Option<GrassColor>,
    /// JSON `grass_is_shaded` (format 1.21.100); read it through
    /// [`grass_is_shaded`](Self::grass_is_shaded).
    #[facet(default)]
    pub grass_is_shaded: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl GrassAppearance {
    /// Whether grass is darkened as under a dense canopy; `false` when the member is absent
    /// (the game fills an absent member with zero).
    pub fn grass_is_shaded(&self) -> bool {
        self.grass_is_shaded.is_some_and(|shaded| shaded.0)
    }
}

/// `minecraft:dry_foliage_color`: the colour of leaf litter and other dry foliage.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DryFoliageColor {
    /// JSON `color`.
    pub color: Color255Rgb,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:sky_color`: the colour of the sky.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SkyColor {
    /// JSON `sky_color`.
    pub sky_color: Color255Rgb,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:water_appearance`: the colour and opacity of the water's surface. At least one
/// member is needed.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct WaterAppearance {
    /// JSON `surface_color`.
    #[facet(default)]
    pub surface_color: Option<Color255Rgb>,
    /// JSON `surface_opacity`, from 0 to 1.
    #[facet(default)]
    pub surface_opacity: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:precipitation`: the particles that fall in the biome and how dense they are. At
/// most one member may be present.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Precipitation {
    /// JSON `ash`.
    #[facet(default)]
    pub ash: Option<StrictFloat<f32>>,
    /// JSON `blue_spores`.
    #[facet(default)]
    pub blue_spores: Option<StrictFloat<f32>>,
    /// JSON `red_spores`.
    #[facet(default)]
    pub red_spores: Option<StrictFloat<f32>>,
    /// JSON `white_ash`.
    #[facet(default)]
    pub white_ash: Option<StrictFloat<f32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The member `key` of `json` as an RGB colour: another form is vanilla's error; an array
/// channel outside `0..=255` is clamped, with an information of the crate's own.
fn check_rgb(json: &Value, key: &'static str, report: &mut Report<'_>) {
    let Some(value) = json.as_object().and_then(|object| object.get(key)) else {
        return;
    };
    check_rgb_value(value, &[key], report);
}

/// [`check_rgb`] for a value at `at`.
fn check_rgb_value(value: &Value, at: &[&str], report: &mut Report<'_>) {
    if !is_rgb(value) {
        report.error(DiagnosticCode::InvalidType { text: fixed(COLOR) }, at);
        return;
    }
    for index in clamped_channels(value) {
        let index = index.to_string();
        let mut path = at.to_vec();
        path.push(&index);
        // The game clamps the channel without a log line; the crate tells the user with an
        // information.
        report.info(DiagnosticCode::InvalidValue { text: None }, &path);
    }
}

/// The member `color` of `json`: a colour, or `{"color_map": <one of names>}`.
fn check_color_or_map(json: &Value, known: fn(&str) -> bool, report: &mut Report<'_>) {
    let Some(value) = json.as_object().and_then(|object| object.get("color")) else {
        return;
    };
    let Some(object) = value.as_object() else {
        check_rgb_value(value, &["color"], report);
        return;
    };
    match object.get(COLOR_MAP) {
        None => report.error(
            DiagnosticCode::MissingRequiredField { text: None },
            &["color", COLOR_MAP],
        ),
        Some(name) => match name.as_string() {
            None => report.error(
                DiagnosticCode::InvalidType { text: None },
                &["color", COLOR_MAP],
            ),
            Some(name) if !known(name.as_str()) => report.error(
                DiagnosticCode::InvalidEnumValue { text: None },
                &["color", COLOR_MAP],
            ),
            Some(_) => {}
        },
    }
    for (key, _) in object {
        if key.as_str() != COLOR_MAP {
            report.unknown_member(
                DiagnosticCode::UnrecognizedField { text: None },
                &["color", key.as_str()],
            );
        }
    }
}

impl Checked for FoliageAppearance {
    fn check(json: &Value, report: &mut Report<'_>) {
        check_color_or_map(
            json,
            |name| FoliageColorMap::from_name(name).is_some(),
            report,
        );
    }
}

impl Checked for GrassAppearance {
    fn check(json: &Value, report: &mut Report<'_>) {
        check_color_or_map(
            json,
            |name| GrassColorMap::from_name(name).is_some(),
            report,
        );
    }
}

impl Checked for DryFoliageColor {
    fn check(json: &Value, report: &mut Report<'_>) {
        check_rgb(json, "color", report);
    }
}

impl Checked for SkyColor {
    fn check(json: &Value, report: &mut Report<'_>) {
        check_rgb(json, "sky_color", report);
    }
}

impl Checked for WaterAppearance {
    /// The constraint of every version that at least one member is present, then the colour.
    fn check(json: &Value, report: &mut Report<'_>) {
        if let Some(object) = json.as_object()
            && object.get("surface_color").is_none()
            && object.get("surface_opacity").is_none()
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(WATER_EMPTY),
                },
                &[],
            );
        }
        check_rgb(json, "surface_color", report);
    }
}

impl Checked for Precipitation {}
