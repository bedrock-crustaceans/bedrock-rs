//! `minecraft:color_grading_settings`: colour grading and tone mapping.

use super::value::Vec3;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::ColorGradingSettingsId;

/// The colour grading of the deferred renderer (`minecraft:color_grading_settings`, in
/// `color_grading/`), at the newest format version (1.21.90).
///
/// JSON: `{"format_version": "1.21.90", "minecraft:color_grading_settings": {"description"?:
/// {"identifier"?: "ns:name"}, "color_grading"?: {"midtones"?: {…}, "highlights"?: {…},
/// "shadows"?: {…}, "temperature"?: {…}}, "tone_mapping"?: {"operator"?: "…"}}}`. No member is
/// required, and one the document leaves out is not written back. Client biomes name the
/// settings by their identifier (`minecraft:color_grading_identifier`); the identifier of the
/// top pack's `color_grading/color_grading.json` is the default of every other biome. A file
/// without an identifier loads but can be named by no biome.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:color_grading_settings")]
#[doc(alias = "color_grading")]
pub struct ColorGradingSettings {
    /// The settings' name (JSON `description`).
    pub description: Option<ColorGradingDescription>,
    /// The grading controls (JSON `color_grading`).
    pub color_grading: Option<ColorGrading>,
    /// The tone mapper (JSON `tone_mapping`).
    pub tone_mapping: Option<ToneMapping>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The `description` of colour-grading settings: unlike the other settings, its `identifier`
/// is optional too.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColorGradingDescription {
    /// The settings' name (JSON `identifier`).
    pub identifier: Option<ColorGradingSettingsId>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The grading controls (JSON `color_grading`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColorGrading {
    /// The controls of the mid tones, which always apply (JSON `midtones`).
    pub midtones: Option<Midtones>,
    /// The controls of the bright tones (JSON `highlights`).
    pub highlights: Option<Highlights>,
    /// The controls of the dark tones (JSON `shadows`).
    pub shadows: Option<Shadows>,
    /// The white balance (JSON `temperature`).
    pub temperature: Option<Temperature>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The five controls of one tonal range, each a value per colour channel.
macro_rules! tones {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$fmeta:meta])* $field:ident: $t:ty,)* }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct $name {
            $($(#[$fmeta])* pub $field: Option<$t>,)*
            /// The contrast per channel, each 0–4 (JSON `contrast`).
            pub contrast: Option<Vec3>,
            /// The gain per channel, each 0–10 (JSON `gain`).
            pub gain: Option<Vec3>,
            /// The gamma per channel, each 0–4 (JSON `gamma`).
            pub gamma: Option<Vec3>,
            /// The offset per channel, each −1–1 (JSON `offset`).
            pub offset: Option<Vec3>,
            /// The saturation per channel, each 0–10 (JSON `saturation`).
            pub saturation: Option<Vec3>,
            /// Members the game does not know, kept for write-back.
            pub extra: UnknownMembers,
        }

        super::defaults! {
            $name {
                contrast: Vec3 = Vec3::splat(1.0);
                gain: Vec3 = Vec3::splat(1.0);
                gamma: Vec3 = Vec3::splat(2.2);
                offset: Vec3 = Vec3::splat(0.0);
                saturation: Vec3 = Vec3::splat(1.0);
            }
        }
    };
}

tones! {
    /// The controls of the mid tones (JSON `midtones`); they always apply.
    Midtones {}
}

tones! {
    /// The controls of the bright tones (JSON `highlights`).
    Highlights {
        /// Whether the highlight controls apply (JSON `enabled`).
        enabled: bool,
        /// The luminance from which a tone is a highlight, 1–20 (JSON `highlightsMin`).
        highlights_min: f32,
    }
}

tones! {
    /// The controls of the dark tones (JSON `shadows`).
    Shadows {
        /// Whether the shadow controls apply (JSON `enabled`).
        enabled: bool,
        /// The luminance up to which a tone is a shadow, 0–1 (JSON `shadowsMax`).
        shadows_max: f32,
    }
}

super::defaults! {
    Highlights {
        enabled: bool = false;
        highlights_min: f32 = 1.0;
    }
}

super::defaults! {
    Shadows {
        enabled: bool = false;
        shadows_max: f32 = 1.0;
    }
}

/// The white balance (JSON `temperature`, from format 1.21.90).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Temperature {
    /// Whether the white balance applies (JSON `enabled`).
    pub enabled: Option<bool>,
    /// The colour temperature in kelvin, 1000–15000 (JSON `temperature`).
    pub temperature: Option<f32>,
    /// How the temperature is applied (JSON `type`).
    pub kind: Option<TemperatureType>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

super::defaults! {
    Temperature {
        enabled: bool = false;
        temperature: f32 = 6500.0;
        kind: TemperatureType = TemperatureType::WhiteBalance;
    }
}

/// How a colour temperature is applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TemperatureType {
    /// `"white_balance"`: the scene is corrected for a light of that temperature; the default.
    #[default]
    WhiteBalance,
    /// `"color_temperature"`: the scene is tinted to that temperature.
    ColorTemperature,
}

impl TemperatureType {
    /// The JSON names, in the order the game lists them.
    pub(crate) const NAMES: &'static [(&'static str, Self)] = &[
        ("white_balance", Self::WhiteBalance),
        ("color_temperature", Self::ColorTemperature),
    ];
}

/// The tone mapper (JSON `tone_mapping`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToneMapping {
    /// The curve that maps scene luminance to the display (JSON `operator`).
    pub operator: Option<ToneMappingOperator>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

super::defaults! {
    ToneMapping {
        operator: ToneMappingOperator = ToneMappingOperator::Reinhard;
    }
}

/// A tone-mapping curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ToneMappingOperator {
    /// `"reinhard"`; the default.
    #[default]
    Reinhard,
    /// `"reinhard_luma"`.
    ReinhardLuma,
    /// `"reinhard_luminance"`.
    ReinhardLuminance,
    /// `"hable"`.
    Hable,
    /// `"aces"`.
    Aces,
    /// `"generic"`: what vanilla's own files use.
    Generic,
}

impl ToneMappingOperator {
    /// The JSON names, in the order the game lists them.
    pub(crate) const NAMES: &'static [(&'static str, Self)] = &[
        ("reinhard", Self::Reinhard),
        ("reinhard_luma", Self::ReinhardLuma),
        ("reinhard_luminance", Self::ReinhardLuminance),
        ("hable", Self::Hable),
        ("aces", Self::Aces),
        ("generic", Self::Generic),
    ];
}

mod codec {
    //! Reading and writing `minecraft:color_grading_settings` at both format versions, and
    //! the step between them.

    use super::{
        ColorGrading, ColorGradingDescription, ColorGradingSettings, Highlights, Midtones, Shadows,
        Temperature, TemperatureType, ToneMapping, ToneMappingOperator,
    };
    use crate::V1_21_90;
    use crate::history::color_grading::v1_21_40;
    use crate::read::{self, Members, Node, Range, Read, members};
    use crate::value::Vec3;
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};

    const KEY: &str = "minecraft:color_grading_settings";

    // The bounds of each component of the five controls.
    const CONTRAST: Range = Range::new(0.0, 4.0);
    const GAIN: Range = Range::new(0.0, 10.0);
    const GAMMA: Range = Range::new(0.0, 4.0);
    const OFFSET: Range = Range::new(-1.0, 1.0);
    const SATURATION: Range = Range::new(0.0, 10.0);
    /// The game's bounds, wider than the published documentation's 1–4.
    const HIGHLIGHTS_MIN: Range = Range::new(1.0, 20.0);
    /// The game's bounds, wider than the published documentation's 0.1–1.
    const SHADOWS_MAX: Range = Range::new(0.0, 1.0);
    const TEMPERATURE: Range = Range::new(1000.0, 15000.0);
    /// The game's temperature of a file that names none.
    const DEFAULT_TEMPERATURE: f32 = 6500.0;

    /// The five controls, in the order the game reads them, with their bounds.
    const CONTROLS: [(&str, Range); 5] = [
        ("contrast", CONTRAST),
        ("gain", GAIN),
        ("gamma", GAMMA),
        ("offset", OFFSET),
        ("saturation", SATURATION),
    ];

    /// The five controls of one tonal range: every one is read, so every failure is reported.
    fn controls(m: &mut Members<'_>, d: &mut Diagnostics) -> Read<[Option<Vec3>; 5]> {
        let read = CONTROLS.map(|(key, range)| m.opt(key, d, |n, d| read::vec3_in(range, n, d)));
        let [contrast, gain, gamma, offset, saturation] = read;
        Ok([contrast?, gain?, gamma?, offset?, saturation?])
    }

    /// Writes the five controls of one tonal range.
    fn write_controls(object: &mut Object, values: [Option<&Vec3>; 5]) -> Written<()> {
        for ((key, range), value) in CONTROLS.into_iter().zip(values) {
            object.opt(key, value, |v, at| write::vec3_in(range, v, at))?;
        }
        Ok(())
    }

    fn midtones(node: &Node<'_>, d: &mut Diagnostics) -> Read<Midtones> {
        let mut m = members(node, d)?;
        let controls = controls(&mut m, d);
        let extra = m.finish(&[], d);
        let [contrast, gain, gamma, offset, saturation] = controls?;
        Ok(Midtones {
            contrast,
            gain,
            gamma,
            offset,
            saturation,
            extra,
        })
    }

    fn highlights(node: &Node<'_>, d: &mut Diagnostics) -> Read<Highlights> {
        let mut m = members(node, d)?;
        let enabled = m.opt("enabled", d, read::boolean);
        let highlights_min = m.opt("highlightsMin", d, |n, d| {
            read::float_in(HIGHLIGHTS_MIN, n, d)
        });
        let controls = controls(&mut m, d);
        let extra = m.finish(&[], d);
        let [contrast, gain, gamma, offset, saturation] = controls?;
        Ok(Highlights {
            enabled: enabled?,
            highlights_min: highlights_min?,
            contrast,
            gain,
            gamma,
            offset,
            saturation,
            extra,
        })
    }

    fn shadows(node: &Node<'_>, d: &mut Diagnostics) -> Read<Shadows> {
        let mut m = members(node, d)?;
        let enabled = m.opt("enabled", d, read::boolean);
        let shadows_max = m.opt("shadowsMax", d, |n, d| read::float_in(SHADOWS_MAX, n, d));
        let controls = controls(&mut m, d);
        let extra = m.finish(&[], d);
        let [contrast, gain, gamma, offset, saturation] = controls?;
        Ok(Shadows {
            enabled: enabled?,
            shadows_max: shadows_max?,
            contrast,
            gain,
            gamma,
            offset,
            saturation,
            extra,
        })
    }

    fn temperature(node: &Node<'_>, d: &mut Diagnostics) -> Read<Temperature> {
        let mut m = members(node, d)?;
        let enabled = m.opt("enabled", d, read::boolean);
        let temperature = m.opt("temperature", d, |n, d| read::float_in(TEMPERATURE, n, d));
        let kind = m.opt("type", d, |n, d| read::choice(TemperatureType::NAMES, n, d));
        let extra = m.finish(&[], d);
        Ok(Temperature {
            enabled: enabled?,
            temperature: temperature?,
            kind: kind?,
            extra,
        })
    }

    fn tone_mapping(node: &Node<'_>, d: &mut Diagnostics) -> Read<ToneMapping> {
        let mut m = members(node, d)?;
        let operator = m.opt("operator", d, |n, d| {
            read::choice(ToneMappingOperator::NAMES, n, d)
        });
        let extra = m.finish(&[], d);
        Ok(ToneMapping {
            operator: operator?,
            extra,
        })
    }

    /// `description`: unlike the other payloads', its `identifier` is optional.
    fn description(node: &Node<'_>, d: &mut Diagnostics) -> Read<ColorGradingDescription> {
        let mut m = members(node, d)?;
        let identifier = m.opt("identifier", d, read::identifier);
        let extra = m.finish(&[], d);
        Ok(ColorGradingDescription {
            identifier: identifier?,
            extra,
        })
    }

    /// `color_grading` as 1.21.40 declares it; `temperature` is a member of 1.21.90.
    fn grading_1_21_40(node: &Node<'_>, d: &mut Diagnostics) -> Read<v1_21_40::ColorGrading> {
        let mut m = members(node, d)?;
        let midtones = m.opt("midtones", d, midtones);
        let highlights = m.opt("highlights", d, highlights);
        let shadows = m.opt("shadows", d, shadows);
        let extra = m.finish(&[("temperature", V1_21_90)], d);
        Ok(v1_21_40::ColorGrading {
            midtones: midtones?,
            highlights: highlights?,
            shadows: shadows?,
            extra,
        })
    }

    fn grading(node: &Node<'_>, d: &mut Diagnostics) -> Read<ColorGrading> {
        let mut m = members(node, d)?;
        let midtones = m.opt("midtones", d, midtones);
        let highlights = m.opt("highlights", d, highlights);
        let shadows = m.opt("shadows", d, shadows);
        let temperature = m.opt("temperature", d, temperature);
        let extra = m.finish(&[], d);
        Ok(ColorGrading {
            midtones: midtones?,
            highlights: highlights?,
            shadows: shadows?,
            temperature: temperature?,
            extra,
        })
    }

    fn write_midtones(value: &Midtones, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        write_controls(
            &mut object,
            [
                value.contrast.as_ref(),
                value.gain.as_ref(),
                value.gamma.as_ref(),
                value.offset.as_ref(),
                value.saturation.as_ref(),
            ],
        )?;
        object.finish(
            &value.extra,
            &["contrast", "gain", "gamma", "offset", "saturation"],
        )
    }

    fn write_highlights(value: &Highlights, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("enabled", value.enabled.as_ref(), write::boolean)?;
        object.opt("highlightsMin", value.highlights_min.as_ref(), |v, at| {
            write::float_in(HIGHLIGHTS_MIN, *v, at)
        })?;
        write_controls(
            &mut object,
            [
                value.contrast.as_ref(),
                value.gain.as_ref(),
                value.gamma.as_ref(),
                value.offset.as_ref(),
                value.saturation.as_ref(),
            ],
        )?;
        object.finish(
            &value.extra,
            &[
                "enabled",
                "highlightsMin",
                "contrast",
                "gain",
                "gamma",
                "offset",
                "saturation",
            ],
        )
    }

    fn write_shadows(value: &Shadows, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("enabled", value.enabled.as_ref(), write::boolean)?;
        object.opt("shadowsMax", value.shadows_max.as_ref(), |v, at| {
            write::float_in(SHADOWS_MAX, *v, at)
        })?;
        write_controls(
            &mut object,
            [
                value.contrast.as_ref(),
                value.gain.as_ref(),
                value.gamma.as_ref(),
                value.offset.as_ref(),
                value.saturation.as_ref(),
            ],
        )?;
        object.finish(
            &value.extra,
            &[
                "enabled",
                "shadowsMax",
                "contrast",
                "gain",
                "gamma",
                "offset",
                "saturation",
            ],
        )
    }

    fn write_temperature(value: &Temperature, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("enabled", value.enabled.as_ref(), write::boolean)?;
        object.opt("temperature", value.temperature.as_ref(), |v, at| {
            write::float_in(TEMPERATURE, *v, at)
        })?;
        object.opt("type", value.kind.as_ref(), |kind, at| {
            write::choice(TemperatureType::NAMES, *kind, at)
        })?;
        object.finish(&value.extra, &["enabled", "temperature", "type"])
    }

    fn write_tone_mapping(value: &ToneMapping, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("operator", value.operator.as_ref(), |operator, at| {
            write::choice(ToneMappingOperator::NAMES, *operator, at)
        })?;
        object.finish(&value.extra, &["operator"])
    }

    fn write_description(value: &ColorGradingDescription, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("identifier", value.identifier.as_ref(), |id, _| {
            Ok(Value::from(id.as_str()))
        })?;
        object.finish(&value.extra, &["identifier"])
    }

    /// The members of `color_grading` in 1.21.90; 1.21.40 has no `temperature`.
    const GRADING: &[&str] = &["midtones", "highlights", "shadows", "temperature"];

    fn write_grading(value: &ColorGrading, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("midtones", value.midtones.as_ref(), write_midtones)?;
        object.opt("highlights", value.highlights.as_ref(), write_highlights)?;
        object.opt("shadows", value.shadows.as_ref(), write_shadows)?;
        object.opt("temperature", value.temperature.as_ref(), write_temperature)?;
        object.finish(&value.extra, GRADING)
    }

    fn write_grading_1_21_40(value: &v1_21_40::ColorGrading, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("midtones", value.midtones.as_ref(), write_midtones)?;
        object.opt("highlights", value.highlights.as_ref(), write_highlights)?;
        object.opt("shadows", value.shadows.as_ref(), write_shadows)?;
        object.finish(&value.extra, &GRADING[..3])
    }

    /// The members of the payload, the same in both structs.
    const MEMBERS: &[&str] = &["description", "color_grading", "tone_mapping"];

    impl VersionStruct for v1_21_40::ColorGradingSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let mut m = members(&read::payload(payload, cx), d).ok()?;
            let description = m.opt("description", d, description);
            let color_grading = m.opt("color_grading", d, grading_1_21_40);
            let tone_mapping = m.opt("tone_mapping", d, tone_mapping);
            let extra = m.finish(&[], d);
            Some(Self {
                description: description.ok()?,
                color_grading: color_grading.ok()?,
                tone_mapping: tone_mapping.ok()?,
                extra,
            })
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload(KEY));
            object.opt("description", self.description.as_ref(), write_description)?;
            object.opt(
                "color_grading",
                self.color_grading.as_ref(),
                write_grading_1_21_40,
            )?;
            object.opt(
                "tone_mapping",
                self.tone_mapping.as_ref(),
                write_tone_mapping,
            )?;
            object.finish(&self.extra, MEMBERS)
        }
    }

    impl VersionStruct for ColorGradingSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let mut m = members(&read::payload(payload, cx), d).ok()?;
            let description = m.opt("description", d, description);
            let color_grading = m.opt("color_grading", d, grading);
            let tone_mapping = m.opt("tone_mapping", d, tone_mapping);
            let extra = m.finish(&[], d);
            Some(Self {
                description: description.ok()?,
                color_grading: color_grading.ok()?,
                tone_mapping: tone_mapping.ok()?,
                extra,
            })
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload(KEY));
            object.opt("description", self.description.as_ref(), write_description)?;
            object.opt("color_grading", self.color_grading.as_ref(), write_grading)?;
            object.opt(
                "tone_mapping",
                self.tone_mapping.as_ref(),
                write_tone_mapping,
            )?;
            object.finish(&self.extra, MEMBERS)
        }
    }

    /// 1.21.40 → 1.21.90: every member is copied, and `color_grading.temperature` is written
    /// switched off, 6500 K, `white_balance`, whether or not the file has `color_grading`
    /// (the game runs the conversion unconditionally). An unknown `temperature` member of the
    /// old document is superseded by it.
    impl Upgrade for v1_21_40::ColorGradingSettings {
        type Next = ColorGradingSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> ColorGradingSettings {
            let old = self.color_grading.unwrap_or_default();
            let mut extra = old.extra;
            extra.remove("temperature");
            ColorGradingSettings {
                description: self.description,
                color_grading: Some(ColorGrading {
                    midtones: old.midtones,
                    highlights: old.highlights,
                    shadows: old.shadows,
                    temperature: Some(Temperature {
                        enabled: Some(false),
                        temperature: Some(DEFAULT_TEMPERATURE),
                        kind: Some(TemperatureType::WhiteBalance),
                        extra: UnknownMembers::default(),
                    }),
                    extra,
                }),
                tone_mapping: self.tone_mapping,
                extra: self.extra,
            }
        }
    }
}

impl ColorGradingSettings {
    /// Colour-grading settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: ColorGradingSettingsId) -> Self {
        Self {
            description: Some(ColorGradingDescription {
                identifier: Some(identifier),
                ..ColorGradingDescription::default()
            }),
            ..Self::default()
        }
    }
}
