//! `minecraft:lighting_settings`: the sun, the moon, ambient and sky light.

use super::read::{self, Range};
use super::value::{Description, Keyframe, Keyframes, Rgb8};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{LightingSettingsId, kind};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use facet::Facet;

const ILLUMINANCE: Range = read::range(0.0, 110_000.0);
const ORBITAL_OFFSET: Range = read::range(0.0, 359.99);
const DESATURATION: Range = read::range(0.0, 1.0);
pub(crate) const AMBIENT: Range = read::range(0.0, 5.0);
pub(crate) const SKY: Range = read::range(0.1, 1.0);

/// The lighting of the deferred renderer (`minecraft:lighting_settings`, in `lighting/`), at
/// the newest format version (1.26.0).
///
/// JSON: `{"format_version": "1.26.0", "minecraft:lighting_settings": {"description"?:
/// {"identifier": "ns:name"}, "directional_lights"?: {"orbital"?: {…}, "flash"?: {…}},
/// "emissive"?: {"desaturation"?: 0–1}, "ambient"?: {"color": keyframes, "illuminance":
/// keyframes}, "sky"?: {"intensity"?: keyframes}}}`. Client biomes name the settings by their
/// identifier (`minecraft:lighting_identifier`); the identifier of the top pack's
/// `lighting/global.json` is the default of every other biome. A member the document leaves
/// out is not written back. A file without `description` loads but can be named by no biome.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[doc(alias = "minecraft:lighting_settings")]
#[doc(alias = "lighting")]
pub struct LightingSettings {
    /// The settings' name (JSON `description`).
    pub description: Option<Description<kind::LightingSettings>>,
    /// The sun, the moon and the End's flash (JSON `directional_lights`).
    pub directional_lights: Option<DirectionalLights>,
    /// How light-emitting surfaces are drawn (JSON `emissive`).
    pub emissive: Option<Emissive>,
    /// The light that reaches every surface (JSON `ambient`).
    pub ambient: Option<AmbientLight>,
    /// The light of the sky (JSON `sky`, from format 1.21.70).
    pub sky: Option<Sky>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The directional lights (JSON `directional_lights`, from format 1.21.80; older formats hold
/// the members of `orbital` here).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct DirectionalLights {
    /// The sun and the moon (JSON `orbital`).
    pub orbital: Option<OrbitalLights>,
    /// The flashes of the End's sky (JSON `flash`).
    pub flash: Option<Flash>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The sun and the moon (JSON `orbital`). `C` is the colour form: [`Rgb8`] from format
/// 1.21.60, [`Rgba8`](super::Rgba8) in 1.21.40.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct OrbitalLights<C = Rgb8> {
    /// The sun (JSON `sun`).
    pub sun: Option<OrbitalLight<C>>,
    /// The moon (JSON `moon`).
    pub moon: Option<OrbitalLight<C>>,
    /// The tilt of the sun's and moon's path in degrees, each value 0–359.99 (JSON
    /// `orbital_offset_degrees`).
    #[facet(schema::read_with = read::keyframed_in(ORBITAL_OFFSET))]
    pub orbital_offset_degrees: Option<Keyframes<f32>>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The sun or the moon.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct OrbitalLight<C = Rgb8> {
    /// The light's strength in lux over the day, each value 0–110000 (JSON `illuminance`).
    #[facet(schema::read_with = read::keyframed_in(ILLUMINANCE))]
    pub illuminance: Option<Keyframes<f32>>,
    /// The light's colour over the day (JSON `color`).
    #[facet(schema::read_with = read::keyframed(C::from_json))]
    pub color: Option<Keyframes<C>>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The flashes of the End's sky (JSON `flash`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Flash {
    /// The flash's colour (JSON `color`).
    pub color: Option<Rgb8>,
    /// The flash's strength in lux (JSON `illuminance`; the game sets no bound).
    #[facet(schema::finite)]
    pub illuminance: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// How light-emitting surfaces are drawn (JSON `emissive`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Emissive {
    /// How much emissive colours are desaturated, 0–1 (JSON `desaturation`).
    #[facet(schema::read_with = read::float_in(DESATURATION))]
    pub desaturation: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The light that reaches every surface (JSON `ambient`); both members are required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct AmbientLight {
    /// The ambient colour over the day (JSON `color`).
    #[facet(schema::read_with = read::present(read::keyframed(Rgb8::from_json)))]
    pub color: Keyframes<Rgb8>,
    /// The ambient strength over the day, each value 0–5 (JSON `illuminance`).
    #[facet(schema::read_with = read::present(read::keyframed_in(AMBIENT)))]
    pub illuminance: Keyframes<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

impl Default for AmbientLight {
    /// The game's ambient light: white, 0.02.
    fn default() -> Self {
        Self {
            color: Keyframes::at_noon(Rgb8::WHITE),
            illuminance: Keyframes::at_noon(DEFAULT_AMBIENT_ILLUMINANCE),
            extra: UnknownMembers::default(),
        }
    }
}

/// The light of the sky (JSON `sky`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Sky {
    /// The strength of the sky light over the day, each value 0.1–1 (JSON `intensity`).
    #[facet(schema::read_with = read::keyframed_in(SKY))]
    pub intensity: Option<Keyframes<f32>>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The game's ambient strength.
pub(crate) const DEFAULT_AMBIENT_ILLUMINANCE: f32 = 0.02;
/// The game's sky intensity.
pub(crate) const DEFAULT_SKY_INTENSITY: f32 = 1.0;

/// The game's sun strength over the day.
fn default_sun_illuminance() -> Keyframes<f32> {
    Keyframes::Keys(
        [
            (0.0, 109_880.0),
            (0.25, 20000.0),
            (0.35, 400.0),
            (0.5, 1.0),
            (0.65, 400.0),
            (0.75, 20000.0),
            (1.0, 109_880.0),
        ]
        .into_iter()
        .map(|(time, value)| Keyframe { time, value })
        .collect(),
    )
}

// The game's values for a member the document leaves out (the same at every format version).
super::defaults! {
    OrbitalLights<Rgb8> {
        orbital_offset_degrees: Keyframes<f32> = Keyframes::Single(0.0);
    }
}

impl OrbitalLights<Rgb8> {
    /// The sun's strength over the day, or the game's own curve (109 880 lux at noon, 1 lux at
    /// midnight) when the document leaves it out.
    #[must_use]
    pub fn sun_illuminance(&self) -> Keyframes<f32> {
        self.sun
            .as_ref()
            .and_then(|sun| sun.illuminance.clone())
            .unwrap_or_else(default_sun_illuminance)
    }

    /// The moon's strength over the day, or the game's 0.27 lux when the document leaves it
    /// out.
    #[must_use]
    pub fn moon_illuminance(&self) -> Keyframes<f32> {
        self.moon
            .as_ref()
            .and_then(|moon| moon.illuminance.clone())
            .unwrap_or(Keyframes::Single(0.27))
    }

    /// The sun's colour over the day, or white when the document leaves it out.
    #[must_use]
    pub fn sun_color(&self) -> Keyframes<Rgb8> {
        self.sun
            .as_ref()
            .and_then(|sun| sun.color.clone())
            .unwrap_or(Keyframes::Single(Rgb8::WHITE))
    }

    /// The moon's colour over the day, or white when the document leaves it out.
    #[must_use]
    pub fn moon_color(&self) -> Keyframes<Rgb8> {
        self.moon
            .as_ref()
            .and_then(|moon| moon.color.clone())
            .unwrap_or(Keyframes::Single(Rgb8::WHITE))
    }
}

super::defaults! {
    Flash {
        color: Rgb8 = Rgb8::WHITE;
        illuminance: f32 = 1.0;
    }
}

super::defaults! {
    Emissive {
        desaturation: f32 = 0.1;
    }
}

super::defaults! {
    Sky {
        intensity: Keyframes<f32> = Keyframes::Single(DEFAULT_SKY_INTENSITY);
    }
}

super::defaults! {
    DirectionalLights {
        orbital: OrbitalLights = OrbitalLights::default();
        flash: Flash = Flash::default();
    }
}

super::defaults! {
    LightingSettings {
        directional_lights: DirectionalLights = DirectionalLights::default();
        emissive: Emissive = Emissive::default();
        ambient: AmbientLight = AmbientLight::default();
        sky: Sky = Sky::default();
    }
}

mod codec {
    //! Reading and writing `minecraft:lighting_settings` at its five format versions, and the
    //! steps between them.

    use super::{AMBIENT, DESATURATION, ILLUMINANCE, ORBITAL_OFFSET, SKY};
    use super::{
        AmbientLight, DEFAULT_SKY_INTENSITY, DirectionalLights, Emissive, Flash, LightingSettings,
        OrbitalLight, OrbitalLights, Sky,
    };
    use crate::history::lighting::{self as history, v1_21_40, v1_21_60, v1_21_70, v1_21_80};
    use crate::read::{self, Range};
    use crate::value::{Keyframes, Rgb8, Rgba8};
    use crate::write::{self, Object, Written};
    use crate::{V1_21_70, V1_21_80};
    use bedrock_addon_core::diagnostic::codes as core_codes;
    use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};

    const KEY: &str = "minecraft:lighting_settings";

    /// The colour form of the sun and the moon: with alpha in 1.21.40, without from 1.21.60.
    trait Colour: Copy {
        fn write(&self, at: &JsonPath) -> Written;
    }

    impl Colour for Rgb8 {
        fn write(&self, at: &JsonPath) -> Written {
            write::rgb(self, at)
        }
    }

    impl Colour for Rgba8 {
        fn write(&self, at: &JsonPath) -> Written {
            write::rgba(self, at)
        }
    }

    fn write_ranged(range: Range, value: &Keyframes<f32>, at: &JsonPath) -> Written {
        write::keyframes(value, at, |v, at| write::float_in(range, *v, at))
    }

    fn write_orbital_light<C: Colour>(value: &OrbitalLight<C>, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("illuminance", value.illuminance.as_ref(), |v, at| {
            write_ranged(ILLUMINANCE, v, at)
        })?;
        object.opt("color", value.color.as_ref(), |frames, at| {
            write::keyframes(frames, at, C::write)
        })?;
        object.finish(&value.extra, &["illuminance", "color"])
    }

    /// The members of the orbital lights; as `directional_lights` (before 1.21.80) nothing
    /// else is declared at the level.
    const ORBITAL: &[&str] = &["sun", "moon", "orbital_offset_degrees"];

    fn write_orbital<C: Colour>(value: &OrbitalLights<C>, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("sun", value.sun.as_ref(), write_orbital_light)?;
        object.opt("moon", value.moon.as_ref(), write_orbital_light)?;
        object.opt(
            "orbital_offset_degrees",
            value.orbital_offset_degrees.as_ref(),
            |v, at| write_ranged(ORBITAL_OFFSET, v, at),
        )?;
        object.finish(&value.extra, ORBITAL)
    }

    fn write_flash(value: &Flash, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("color", value.color.as_ref(), write::rgb)?;
        object.opt("illuminance", value.illuminance.as_ref(), |v, at| {
            write::float(*v, at)
        })?;
        object.finish(&value.extra, &["color", "illuminance"])
    }

    fn write_directional(value: &DirectionalLights, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("orbital", value.orbital.as_ref(), write_orbital)?;
        object.opt("flash", value.flash.as_ref(), write_flash)?;
        object.finish(&value.extra, &["orbital", "flash"])
    }

    fn write_emissive(value: &Emissive, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("desaturation", value.desaturation.as_ref(), |v, at| {
            write::float_in(DESATURATION, *v, at)
        })?;
        object.finish(&value.extra, &["desaturation"])
    }

    fn write_plain_ambient(value: &history::AmbientLight, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.req("color", &value.color, write::rgb)?;
        object.req("illuminance", &value.illuminance, |v, at| {
            write::float_in(AMBIENT, *v, at)
        })?;
        object.finish(&value.extra, &["color", "illuminance"])
    }

    fn write_ambient(value: &AmbientLight, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.req("color", &value.color, |frames, at| {
            write::keyframes(frames, at, write::rgb)
        })?;
        object.req("illuminance", &value.illuminance, |v, at| {
            write_ranged(AMBIENT, v, at)
        })?;
        object.finish(&value.extra, &["color", "illuminance"])
    }

    fn write_plain_sky(value: &history::Sky, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("intensity", value.intensity.as_ref(), |v, at| {
            write::float_in(SKY, *v, at)
        })?;
        object.finish(&value.extra, &["intensity"])
    }

    fn write_sky(value: &Sky, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("intensity", value.intensity.as_ref(), |v, at| {
            write_ranged(SKY, v, at)
        })?;
        object.finish(&value.extra, &["intensity"])
    }

    /// The members of the payload before 1.21.70.
    const WITHOUT_SKY: &[&str] = &["description", "directional_lights", "emissive", "ambient"];
    /// The members of the payload from 1.21.70.
    const MEMBERS: &[&str] = &[
        "description",
        "directional_lights",
        "emissive",
        "ambient",
        "sky",
    ];
    /// Before 1.21.80 `directional_lights` holds the orbital lights themselves.
    const FLAT_LATER: &[read::Hint] = &[
        read::later("directional_lights/orbital", V1_21_80),
        read::later("directional_lights/flash", V1_21_80),
    ];
    /// Before 1.21.70 there is no `sky` either.
    const LATER_THAN_1_21_60: &[read::Hint] = &[
        read::later("sky", V1_21_70),
        read::later("directional_lights/orbital", V1_21_80),
        read::later("directional_lights/flash", V1_21_80),
    ];

    /// The two structs without `sky`: 1.21.40 (`Rgba8`) and 1.21.60 (`Rgb8`).
    macro_rules! without_sky {
        ($ty:ty, $colour:ty) => {
            impl VersionStruct for $ty {
                fn parse(
                    payload: JsonSlice<'_>,
                    cx: &ParseCx<'_>,
                    d: &mut Diagnostics,
                ) -> Option<Self> {
                    read::payload_hinted(payload, cx, d, LATER_THAN_1_21_60)
                }

                fn write(&self) -> Result<Value, EncodeError> {
                    let mut object = Object::new(&write::payload(KEY));
                    object.opt("description", self.description.as_ref(), write::description)?;
                    object.opt(
                        "directional_lights",
                        self.directional_lights.as_ref(),
                        write_orbital,
                    )?;
                    object.opt("emissive", self.emissive.as_ref(), write_emissive)?;
                    object.opt("ambient", self.ambient.as_ref(), write_plain_ambient)?;
                    object.finish(&self.extra, WITHOUT_SKY)
                }
            }
        };
    }

    without_sky!(v1_21_40::LightingSettings, Rgba8);
    without_sky!(v1_21_60::LightingSettings, Rgb8);

    impl VersionStruct for v1_21_70::LightingSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_hinted(payload, cx, d, FLAT_LATER)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload(KEY));
            object.opt("description", self.description.as_ref(), write::description)?;
            object.opt(
                "directional_lights",
                self.directional_lights.as_ref(),
                write_orbital,
            )?;
            object.opt("emissive", self.emissive.as_ref(), write_emissive)?;
            object.opt("ambient", self.ambient.as_ref(), write_plain_ambient)?;
            object.opt("sky", self.sky.as_ref(), write_plain_sky)?;
            object.finish(&self.extra, MEMBERS)
        }
    }

    impl VersionStruct for v1_21_80::LightingSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload(payload, cx, d)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload(KEY));
            object.opt("description", self.description.as_ref(), write::description)?;
            object.opt(
                "directional_lights",
                self.directional_lights.as_ref(),
                write_directional,
            )?;
            object.opt("emissive", self.emissive.as_ref(), write_emissive)?;
            object.opt("ambient", self.ambient.as_ref(), write_plain_ambient)?;
            object.opt("sky", self.sky.as_ref(), write_plain_sky)?;
            object.finish(&self.extra, MEMBERS)
        }
    }

    impl VersionStruct for LightingSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload(payload, cx, d)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload(KEY));
            object.opt("description", self.description.as_ref(), write::description)?;
            object.opt(
                "directional_lights",
                self.directional_lights.as_ref(),
                write_directional,
            )?;
            object.opt("emissive", self.emissive.as_ref(), write_emissive)?;
            object.opt("ambient", self.ambient.as_ref(), write_ambient)?;
            object.opt("sky", self.sky.as_ref(), write_sky)?;
            object.finish(&self.extra, MEMBERS)
        }
    }

    /// The colours of one light without their alpha; an alpha other than 255 is dropped with a
    /// `DroppedField` info at the colour (at its key, for keyframes).
    fn opaque(
        light: Option<OrbitalLight<Rgba8>>,
        name: &str,
        cx: &mut UpgradeCx<'_>,
    ) -> Option<OrbitalLight<Rgb8>> {
        let light = light?;
        let mut at = write::payload(KEY);
        at.push("directional_lights");
        at.push(name);
        at.push("color");
        let mut dropped = |alpha: u8, at: JsonPath| {
            if alpha != u8::MAX {
                cx.diagnostics.push(
                    Diagnostic::of(&core_codes::DROPPED_FIELD)
                        .at(at)
                        .phase(Phase::Upgrade),
                );
            }
        };
        let color = light.color.map(|color| match color {
            Keyframes::Single(value) => {
                dropped(value.alpha, at.clone());
                Keyframes::Single(value.rgb())
            }
            Keyframes::Keys(frames) => Keyframes::Keys(
                frames
                    .into_iter()
                    .map(|frame| {
                        dropped(frame.value.alpha, at.child(&write::time_key(frame.time)));
                        crate::value::Keyframe {
                            time: frame.time,
                            value: frame.value.rgb(),
                        }
                    })
                    .collect(),
            ),
        });
        Some(OrbitalLight {
            illuminance: light.illuminance,
            color,
            extra: light.extra,
        })
    }

    /// 1.21.40 → 1.21.60: every member is copied; the sun's and the moon's colours lose their
    /// alpha (the game forces it opaque). A dropped alpha that was not opaque is a
    /// `DroppedField` info (the game is silent, so the finding is the crate's own).
    impl Upgrade for v1_21_40::LightingSettings {
        type Next = v1_21_60::LightingSettings;

        fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_21_60::LightingSettings {
            let directional_lights = self.directional_lights.map(|lights| OrbitalLights {
                sun: opaque(lights.sun, "sun", cx),
                moon: opaque(lights.moon, "moon", cx),
                orbital_offset_degrees: lights.orbital_offset_degrees,
                extra: lights.extra,
            });
            v1_21_60::LightingSettings {
                description: self.description,
                directional_lights,
                emissive: self.emissive,
                ambient: self.ambient,
                extra: self.extra,
            }
        }
    }

    /// 1.21.60 → 1.21.70: every member is copied and `sky.intensity` is written 1.0, always
    /// (there is no old value). An unknown `sky` member of the old document is superseded.
    impl Upgrade for v1_21_60::LightingSettings {
        type Next = v1_21_70::LightingSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_21_70::LightingSettings {
            let mut extra = self.extra;
            extra.remove("sky");
            v1_21_70::LightingSettings {
                description: self.description,
                directional_lights: self.directional_lights,
                emissive: self.emissive,
                ambient: self.ambient,
                sky: Some(history::Sky {
                    intensity: Some(DEFAULT_SKY_INTENSITY),
                    extra: UnknownMembers::default(),
                }),
                extra,
            }
        }
    }

    /// 1.21.70 → 1.21.80: the sun, the moon and the orbital offset move under
    /// `directional_lights.orbital`; `flash` is not written. The unknown members of the old
    /// `directional_lights` stay at that level, but for `orbital` and `flash`, which the new
    /// struct declares.
    impl Upgrade for v1_21_70::LightingSettings {
        type Next = v1_21_80::LightingSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_21_80::LightingSettings {
            let directional_lights = self.directional_lights.map(|lights| {
                let mut extra = lights.extra;
                extra.remove("orbital");
                extra.remove("flash");
                DirectionalLights {
                    orbital: Some(OrbitalLights {
                        sun: lights.sun,
                        moon: lights.moon,
                        orbital_offset_degrees: lights.orbital_offset_degrees,
                        extra: UnknownMembers::default(),
                    }),
                    flash: None,
                    extra,
                }
            });
            v1_21_80::LightingSettings {
                description: self.description,
                directional_lights,
                emissive: self.emissive,
                ambient: self.ambient,
                sky: self.sky,
                extra: self.extra,
            }
        }
    }

    /// 1.21.80 → 1.26.0: `ambient.color`, `ambient.illuminance` and `sky.intensity` become one
    /// keyframe at 0.0. The game runs this conversion unconditionally, so a document without
    /// `ambient` or `sky` (or without `sky.intensity`) gets the struct defaults, wrapped.
    impl Upgrade for v1_21_80::LightingSettings {
        type Next = LightingSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> LightingSettings {
            let ambient = match self.ambient {
                Some(ambient) => AmbientLight {
                    color: Keyframes::at_noon(ambient.color),
                    illuminance: Keyframes::at_noon(ambient.illuminance),
                    extra: ambient.extra,
                },
                None => AmbientLight::default(),
            };
            let sky = self.sky.unwrap_or_default();
            LightingSettings {
                description: self.description,
                directional_lights: self.directional_lights,
                emissive: self.emissive,
                ambient: Some(ambient),
                sky: Some(Sky {
                    intensity: Some(Keyframes::at_noon(
                        sky.intensity.unwrap_or(DEFAULT_SKY_INTENSITY),
                    )),
                    extra: sky.extra,
                }),
                extra: self.extra,
            }
        }
    }
}

impl LightingSettings {
    /// Lighting settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: LightingSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}
