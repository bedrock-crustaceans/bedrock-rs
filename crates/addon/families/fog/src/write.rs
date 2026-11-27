//! Writing fog settings back: the members the value holds, in the
//! schema's order, then the members it kept in `extra`; nothing the author did not write.

use facet_value::{VArray, VObject};

use super::read::read_settings;
use super::{
    ColorNorm, DistanceType, FogColor, FogDistanceCollection, FogDistanceSetting, FogSettings,
    FogTransitionSetting, FogVolumetricCollection, FogVolumetricDensitySetting,
    FogVolumetricMediaCoeffsSetting, FogWaterDistanceSetting, Keyframed, LATEST,
};
use crate::history::v1_16_100::{CoeffColorJu, FogColorJu};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Unrepresentable, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};
use bedrock_addon_shared_shapes::Keyframe;

fn refused(what: Unrepresentable) -> EncodeError {
    EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    }
}

/// An `f32` in the shortest decimal form that reads back as the same `f32`; an integral value
/// is written as an integer.
pub(crate) fn float(x: f32) -> Result<Value, EncodeError> {
    if x.is_nan() {
        return Err(refused(Unrepresentable::NaN));
    }
    if x.is_infinite() {
        return Err(refused(if x > 0.0 {
            Unrepresentable::Infinity
        } else {
            Unrepresentable::NegativeInfinity
        }));
    }
    if x == 0.0 && x.is_sign_negative() {
        // The zero of the file keeps its sign: an integer has none.
        return Ok(Value::from(-0.0_f64));
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "checked: integral and within the range an i64 holds exactly"
    )]
    if x.fract() == 0.0 && x.abs() < 1e15 {
        return Ok(Value::from(x as i64));
    }
    let shortest = x.to_string().parse::<f64>().unwrap_or(f64::from(x));
    Ok(Value::from(shortest))
}

/// An object from the members that are present, in the order given.
#[allow(
    clippy::unnecessary_wraps,
    reason = "read with `?` beside the fallible member writers"
)]
pub(crate) fn object_with<const N: usize>(
    members: [(&str, Option<Value>); N],
) -> Result<Value, EncodeError> {
    Ok(object_of(members, &UnknownMembers::default()))
}

/// [`object_with`], then the unknown members the value kept.
fn object_of<const N: usize>(members: [(&str, Option<Value>); N], extra: &UnknownMembers) -> Value {
    let mut object = VObject::new();
    for (key, value) in members {
        if let Some(value) = value {
            object.insert(key, value);
        }
    }
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
    object.into()
}

fn triple(values: [Value; 3]) -> Value {
    values.into_iter().collect::<VArray>().into()
}

/// A `fog_color` of the old schema.
pub(crate) fn color_ju(color: &FogColorJu) -> Value {
    match color {
        FogColorJu::Hex(text) => text.as_str().into(),
        FogColorJu::Array(rgb) => triple(rgb.map(Value::from)),
    }
}

/// A coefficient colour of the old schema.
pub(crate) fn norm_ju(color: &CoeffColorJu) -> Result<Value, EncodeError> {
    match color {
        CoeffColorJu::Hex(text) => Ok(text.as_str().into()),
        CoeffColorJu::Array([r, g, b]) => Ok(triple([float(*r)?, float(*g)?, float(*b)?])),
    }
}

fn fog_color(color: &FogColor) -> Value {
    match color {
        FogColor::Hex(text) => text.as_str().into(),
        FogColor::Array(rgb) => triple(rgb.map(|c| Value::from(i64::from(c)))),
    }
}

fn color_norm(color: &ColorNorm) -> Result<Value, EncodeError> {
    match color {
        ColorNorm::Hex(text) => Ok(text.as_str().into()),
        ColorNorm::Array([r, g, b]) => Ok(triple([float(*r)?, float(*g)?, float(*b)?])),
    }
}

fn keyframed<T>(
    value: &Keyframed<T>,
    write: impl Fn(&T) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    match value {
        Keyframed::Single(value) => write(value),
        Keyframed::Keys(keys) => {
            let mut object = VObject::new();
            for Keyframe { time, value } in keys {
                if !time.is_finite() {
                    return Err(refused(Unrepresentable::NaN));
                }
                object.insert(time.to_string(), write(value)?);
            }
            Ok(object.into())
        }
    }
}

fn scalar(value: Option<&Keyframed<f32>>) -> Result<Option<Value>, EncodeError> {
    value.map(|k| keyframed(k, |x| float(*x))).transpose()
}

fn optional(x: Option<f32>) -> Result<Option<Value>, EncodeError> {
    x.map(float).transpose()
}

fn distance_type(kind: Option<DistanceType>) -> Option<Value> {
    kind.map(|kind| kind.as_str().into())
}

fn distance_setting(setting: &FogDistanceSetting) -> Result<Value, EncodeError> {
    Ok(object_of(
        [
            ("fog_start", optional(setting.fog_start)?),
            ("fog_end", optional(setting.fog_end)?),
            (
                "render_distance_type",
                distance_type(setting.render_distance_type),
            ),
            ("fog_color", setting.fog_color.as_ref().map(fog_color)),
        ],
        &setting.extra,
    ))
}

fn transition(t: &FogTransitionSetting) -> Result<Value, EncodeError> {
    Ok(object_of(
        [
            (
                "init_fog",
                t.init_fog.as_ref().map(distance_setting).transpose()?,
            ),
            ("min_percent", optional(t.min_percent)?),
            ("mid_seconds", optional(t.mid_seconds)?),
            ("mid_percent", optional(t.mid_percent)?),
            ("max_seconds", optional(t.max_seconds)?),
        ],
        &t.extra,
    ))
}

fn water_setting(setting: &FogWaterDistanceSetting) -> Result<Value, EncodeError> {
    Ok(object_of(
        [
            ("fog_start", optional(setting.fog_start)?),
            ("fog_end", optional(setting.fog_end)?),
            (
                "render_distance_type",
                distance_type(setting.render_distance_type),
            ),
            ("fog_color", setting.fog_color.as_ref().map(fog_color)),
            (
                "transition_fog",
                setting
                    .transition_fog
                    .as_ref()
                    .map(transition)
                    .transpose()?,
            ),
        ],
        &setting.extra,
    ))
}

fn distance(d: &FogDistanceCollection) -> Result<Value, EncodeError> {
    let entry = |s: &Option<FogDistanceSetting>| s.as_ref().map(distance_setting).transpose();
    Ok(object_of(
        [
            ("air", entry(&d.air)?),
            ("weather", entry(&d.weather)?),
            ("water", d.water.as_ref().map(water_setting).transpose()?),
            ("lava", entry(&d.lava)?),
            ("lava_resistance", entry(&d.lava_resistance)?),
            ("powder_snow", entry(&d.powder_snow)?),
        ],
        &d.extra,
    ))
}

fn density_setting(s: &FogVolumetricDensitySetting) -> Result<Value, EncodeError> {
    Ok(object_of(
        [
            ("max_density", scalar(s.max_density.as_ref())?),
            ("uniform", s.uniform.map(Value::from)),
            (
                "zero_density_height",
                scalar(s.zero_density_height.as_ref())?,
            ),
            ("max_density_height", scalar(s.max_density_height.as_ref())?),
        ],
        &s.extra,
    ))
}

fn coefficients(s: &FogVolumetricMediaCoeffsSetting) -> Result<Value, EncodeError> {
    let color =
        |c: &Option<Keyframed<ColorNorm>>| c.as_ref().map(|k| keyframed(k, color_norm)).transpose();
    Ok(object_of(
        [
            ("absorption", color(&s.absorption)?),
            ("scattering", color(&s.scattering)?),
        ],
        &s.extra,
    ))
}

fn volumetric(v: &FogVolumetricCollection) -> Result<Value, EncodeError> {
    let density = match &v.density {
        Some(d) => {
            let entry = |s: &Option<FogVolumetricDensitySetting>| {
                s.as_ref().map(density_setting).transpose()
            };
            Some(object_of(
                [
                    ("air", entry(&d.air)?),
                    ("weather", entry(&d.weather)?),
                    ("water", entry(&d.water)?),
                    ("lava", entry(&d.lava)?),
                    ("lava_resistance", entry(&d.lava_resistance)?),
                ],
                &d.extra,
            ))
        }
        None => None,
    };
    let media = match &v.media_coefficients {
        Some(m) => {
            let entry = |s: &Option<FogVolumetricMediaCoeffsSetting>| {
                s.as_ref().map(coefficients).transpose()
            };
            Some(object_of(
                [
                    ("air", entry(&m.air)?),
                    ("water", entry(&m.water)?),
                    ("cloud", entry(&m.cloud)?),
                ],
                &m.extra,
            ))
        }
        None => None,
    };
    let hg = match &v.henyey_greenstein_g {
        Some(g) => {
            let entry = |s: &Option<super::FogVolumetricHenyeyGreensteinGSetting>| {
                s.as_ref()
                    .map(|s| {
                        Ok(object_of(
                            [(
                                "henyey_greenstein_g",
                                scalar(s.henyey_greenstein_g.as_ref())?,
                            )],
                            &s.extra,
                        ))
                    })
                    .transpose()
            };
            Some(object_of(
                [("air", entry(&g.air)?), ("water", entry(&g.water)?)],
                &g.extra,
            ))
        }
        None => None,
    };
    Ok(object_of(
        [
            ("density", density),
            ("media_coefficients", media),
            ("henyey_greenstein_g", hg),
        ],
        &v.extra,
    ))
}

impl Versioned for FogSettings {
    const VERSION: FormatVersion = LATEST;
}

impl VersionStruct for FogSettings {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_settings(payload, &cx.at, d)
    }

    /// The payload (the value of `minecraft:fog_settings`).
    fn write(&self) -> Result<Value, EncodeError> {
        let mut identifier = VObject::new();
        identifier.insert("identifier", self.description.identifier.as_str());
        for (key, value) in self.description.extra.iter() {
            identifier.insert(key, value.clone());
        }
        Ok(object_of(
            [
                ("description", Some(identifier.into())),
                (
                    "distance",
                    self.distance.as_ref().map(distance).transpose()?,
                ),
                (
                    "volumetric",
                    self.volumetric.as_ref().map(volumetric).transpose()?,
                ),
            ],
            &self.extra,
        ))
    }
}

impl FogSettings {
    /// The refusal before the document is written (`JsonDocument::refuse_write`): the first
    /// rule V15–V20 the value breaks, at its path below the payload key, then a time written
    /// twice in a keyed member, at the second keyframe of that time. The document itself is
    /// `format_version` 1.26.20, the only version the game reads this shape at, and the
    /// payload, through core's `write_envelope`.
    pub(crate) fn refuse_write_rules(&self) -> Result<(), EncodeError> {
        let at = JsonPath::new().child("minecraft:fog_settings");
        let mut found = Diagnostics::default();
        super::rules::cereal_rules(self, &at, &mut found);
        if let Some(broken) = found.first_error() {
            return Err(EncodeError::NotRepresentable {
                at: broken.location.json_path.clone(),
                what: Unrepresentable::Unsupported,
            });
        }
        // One time is one keyframe: a hand-built list holding a time twice has no text that
        // reads back to it.
        if let Some(at) = super::rules::repeated_time(self, &at) {
            return Err(EncodeError::NotRepresentable {
                at,
                what: Unrepresentable::Unsupported,
            });
        }
        Ok(())
    }
}
