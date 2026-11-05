//! `minecraft:atmosphere_settings`: how the sky scatters light over the day.

use super::value::{Description, Keyframe, Keyframes, Rgb8};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{AtmosphereSettingsId, kind};

/// The atmospheric scattering of the deferred renderer (`minecraft:atmosphere_settings`, in
/// `atmospherics/`), at the newest format version (1.26.20).
///
/// JSON: `{"format_version": "1.26.20", "minecraft:atmosphere_settings": {"description":
/// {"identifier": "ns:name"}, "horizon_blend_stops"?: {…}, "rayleigh_strength"?: keyframes,
/// "sun_mie_strength"?: keyframes, "moon_mie_strength"?: keyframes, "sun_glare_shape"?:
/// keyframes, "sky_zenith_color"?: keyframes, "sky_horizon_color"?: keyframes}}`. Client
/// biomes name the settings by their identifier (`minecraft:atmosphere_identifier`); the
/// identifier of the top pack's `atmospherics/atmospherics.json` is the default of every other
/// biome. A member the document leaves out is not written back; the game then uses its own
/// keyframes for that member alone.
///
/// A 1.26.20 file must have `description`; a 1.21.40 file may leave it out, and then loads
/// without a name no biome can use ([`AtmosphereSettings::description`] is `None`).
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:atmosphere_settings")]
#[doc(alias = "atmospherics")]
pub struct AtmosphereSettings {
    /// The settings' name (JSON `description`); `None` only in a document upgraded from a
    /// 1.21.40 file without one.
    pub description: Option<Description<kind::AtmosphereSettings>>,
    /// Where the horizon colour blends into the zenith colour (JSON `horizon_blend_stops`).
    pub horizon_blend_stops: Option<HorizonBlendStops>,
    /// The strength of Rayleigh scattering, each value 0–50 (JSON `rayleigh_strength`).
    pub rayleigh_strength: Option<Keyframes<f32>>,
    /// The strength of Mie scattering around the sun, each value 0–60 (JSON
    /// `sun_mie_strength`).
    pub sun_mie_strength: Option<Keyframes<f32>>,
    /// The strength of Mie scattering around the moon, each value 0–20 (JSON
    /// `moon_mie_strength`).
    pub moon_mie_strength: Option<Keyframes<f32>>,
    /// The shape of the glare around the sun, each value 0–50 (JSON `sun_glare_shape`).
    pub sun_glare_shape: Option<Keyframes<f32>>,
    /// The colour of the sky overhead (JSON `sky_zenith_color`).
    pub sky_zenith_color: Option<Keyframes<Rgb8>>,
    /// The colour of the sky at the horizon (JSON `sky_horizon_color`).
    pub sky_horizon_color: Option<Keyframes<Rgb8>>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// Where the horizon colour blends into the zenith colour (JSON `horizon_blend_stops`); every
/// member is optional.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HorizonBlendStops {
    /// Where the blend begins below the horizon, each value 0–1 (JSON `min`).
    pub min: Option<Keyframes<f32>>,
    /// Where the horizon colour begins to fade, each value 0–1 (JSON `start`).
    pub start: Option<Keyframes<f32>>,
    /// Where Mie scattering begins, each value 0–1.2 (JSON `mie_start`).
    pub mie_start: Option<Keyframes<f32>>,
    /// Where the zenith colour is reached, each value 0–1 (JSON `max`).
    pub max: Option<Keyframes<f32>>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// Keyframes from `(time, value)` pairs.
fn table<T: Copy>(pairs: &[(f32, T)]) -> Keyframes<T> {
    Keyframes::Keys(
        pairs
            .iter()
            .map(|&(time, value)| Keyframe { time, value })
            .collect(),
    )
}

/// The times of the game's horizon-blend keyframes.
const BLEND_TIMES: [f32; 8] = [0.0, 0.19625, 0.25, 0.34, 0.6, 0.675, 0.74875, 0.8175];
/// The times of the game's scattering-strength keyframes.
const STRENGTH_TIMES: [f32; 8] = [0.0, 0.25, 0.3, 0.30375, 0.325, 0.6, 0.7075, 0.75];

/// Keyframes of `values` at `times`.
fn timed(times: [f32; 8], values: [f32; 8]) -> Keyframes<f32> {
    Keyframes::Keys(
        times
            .into_iter()
            .zip(values)
            .map(|(time, value)| Keyframe { time, value })
            .collect(),
    )
}

// The game's keyframes for a member the document leaves out, member by member.
super::defaults! {
    HorizonBlendStops {
        min: Keyframes<f32> =
            timed(BLEND_TIMES, [0.0, 0.008, 0.0, 0.11, 0.027, 0.01333, 0.0, 0.0]);
        start: Keyframes<f32> =
            timed(BLEND_TIMES, [0.25, 0.41791, 0.213, 0.401, 0.23, 0.21431, 0.19901, 0.618]);
        mie_start: Keyframes<f32> =
            timed(BLEND_TIMES, [0.5, 1.172, 0.5, 0.501, 0.167, 0.15, 0.55, 0.79]);
        max: Keyframes<f32> =
            timed(BLEND_TIMES, [0.25, 0.137, 0.0, 0.467, 0.2, 0.1919, 0.214, 0.0]);
    }
}

super::defaults! {
    AtmosphereSettings {
        horizon_blend_stops: HorizonBlendStops = HorizonBlendStops::default();
        rayleigh_strength: Keyframes<f32> =
            timed(STRENGTH_TIMES, [10.26125, 1.625, 1.625, 1.625, 1.625, 5.0, 1.98254, 1.25]);
        sun_mie_strength: Keyframes<f32> =
            timed(STRENGTH_TIMES, [20.0, 60.0, 60.0, 57.57, 20.0, 20.0, 20.0, 20.0]);
        moon_mie_strength: Keyframes<f32> =
            timed(STRENGTH_TIMES, [0.0, 0.0, 0.0, 2.28, 20.0, 20.0, 0.0, 0.0]);
        sun_glare_shape: Keyframes<f32> =
            timed(STRENGTH_TIMES, [15.899, 15.899, 15.899, 15.8444, 15.0, 4.0, 4.0, 4.0]);
        sky_zenith_color: Keyframes<Rgb8> = table(&[
            (0.0, Rgb8::new(0, 125, 164)),
            (0.25, Rgb8::new(11, 55, 97)),
            (0.5, Rgb8::new(7, 10, 36)),
            (0.75, Rgb8::new(50, 88, 170)),
        ]);
        sky_horizon_color: Keyframes<Rgb8> = table(&[
            (0.0, Rgb8::new(255, 255, 254)),
            (0.171_875, Rgb8::new(196, 133, 102)),
            (0.25, Rgb8::new(255, 85, 85)),
            (0.305, Rgb8::new(173, 60, 67)),
            (0.35, Rgb8::new(168, 76, 140)),
            (0.4175, Rgb8::new(130, 99, 169)),
            (0.47, Rgb8::new(66, 98, 238)),
            (0.603, Rgb8::new(66, 98, 238)),
            (0.6475, Rgb8::new(168, 99, 162)),
            (0.75, Rgb8::new(238, 144, 124)),
            (0.84, Rgb8::new(242, 168, 152)),
        ]);
    }
}

mod codec {
    //! Reading and writing `minecraft:atmosphere_settings` at both format versions, and the
    //! step between them.

    use super::{AtmosphereSettings, HorizonBlendStops};
    use crate::history::atmosphere::v1_21_40;
    use crate::read::{self, Members, Node, Range, Read, members};
    use crate::value::{Keyframes, Rgb8};
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};

    const KEY: &str = "minecraft:atmosphere_settings";

    // The bounds of each keyframe value.
    const RAYLEIGH: Range = Range::new(0.0, 50.0);
    const SUN_MIE: Range = Range::new(0.0, 60.0);
    const MOON_MIE: Range = Range::new(0.0, 20.0);
    const SUN_GLARE: Range = Range::new(0.0, 50.0);
    const STOP: Range = Range::new(0.0, 1.0);
    const MIE_START: Range = Range::new(0.0, 1.2);

    /// A keyframed number whose values lie in `range`.
    fn ranged(
        range: Range,
        key: &str,
        m: &mut Members<'_>,
        d: &mut Diagnostics,
    ) -> Read<Option<Keyframes<f32>>> {
        m.opt(key, d, |n, d| {
            read::keyframes(n, d, |n, d| read::float_in(range, n, d))
        })
    }

    /// A keyframed colour.
    fn colour(
        key: &str,
        m: &mut Members<'_>,
        d: &mut Diagnostics,
    ) -> Read<Option<Keyframes<Rgb8>>> {
        m.opt(key, d, |n, d| read::keyframes(n, d, read::rgb))
    }

    fn stops(node: &Node<'_>, d: &mut Diagnostics) -> Read<HorizonBlendStops> {
        let mut m = members(node, d)?;
        let min = ranged(STOP, "min", &mut m, d);
        let start = ranged(STOP, "start", &mut m, d);
        let mie_start = ranged(MIE_START, "mie_start", &mut m, d);
        let max = ranged(STOP, "max", &mut m, d);
        let extra = m.finish(&[], d);
        Ok(HorizonBlendStops {
            min: min?,
            start: start?,
            mie_start: mie_start?,
            max: max?,
            extra,
        })
    }

    /// The payload; `description` is required from 1.26.20.
    fn settings(node: &Node<'_>, required: bool, d: &mut Diagnostics) -> Read<AtmosphereSettings> {
        let mut m = members(node, d)?;
        let description = if required {
            m.req("description", d, read::description).map(Some)
        } else {
            m.opt("description", d, read::description)
        };
        let horizon_blend_stops = m.opt("horizon_blend_stops", d, stops);
        let rayleigh_strength = ranged(RAYLEIGH, "rayleigh_strength", &mut m, d);
        let sun_mie_strength = ranged(SUN_MIE, "sun_mie_strength", &mut m, d);
        let moon_mie_strength = ranged(MOON_MIE, "moon_mie_strength", &mut m, d);
        let sun_glare_shape = ranged(SUN_GLARE, "sun_glare_shape", &mut m, d);
        let sky_zenith_color = colour("sky_zenith_color", &mut m, d);
        let sky_horizon_color = colour("sky_horizon_color", &mut m, d);
        let extra = m.finish(&[], d);
        Ok(AtmosphereSettings {
            description: description?,
            horizon_blend_stops: horizon_blend_stops?,
            rayleigh_strength: rayleigh_strength?,
            sun_mie_strength: sun_mie_strength?,
            moon_mie_strength: moon_mie_strength?,
            sun_glare_shape: sun_glare_shape?,
            sky_zenith_color: sky_zenith_color?,
            sky_horizon_color: sky_horizon_color?,
            extra,
        })
    }

    /// Writes a keyframed number whose values lie in `range`.
    fn write_ranged(
        object: &mut Object,
        range: Range,
        key: &str,
        value: Option<&Keyframes<f32>>,
    ) -> Written<()> {
        object.opt(key, value, |frames, at| {
            write::keyframes(frames, at, |v, at| write::float_in(range, *v, at))
        })
    }

    fn write_stops(value: &HorizonBlendStops, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        write_ranged(&mut object, STOP, "min", value.min.as_ref())?;
        write_ranged(&mut object, STOP, "start", value.start.as_ref())?;
        write_ranged(
            &mut object,
            MIE_START,
            "mie_start",
            value.mie_start.as_ref(),
        )?;
        write_ranged(&mut object, STOP, "max", value.max.as_ref())?;
        object.finish(&value.extra, &["min", "start", "mie_start", "max"])
    }

    /// The members of the payload, the same in both structs.
    const MEMBERS: &[&str] = &[
        "description",
        "horizon_blend_stops",
        "rayleigh_strength",
        "sun_mie_strength",
        "moon_mie_strength",
        "sun_glare_shape",
        "sky_zenith_color",
        "sky_horizon_color",
    ];

    /// The payload; a value without `description` has no 1.26.20 form.
    fn write_settings(value: &AtmosphereSettings, required: bool) -> Written {
        let mut object = Object::new(&write::payload(KEY));
        if required && value.description.is_none() {
            return Err(write::refuse(&object.path("description")));
        }
        object.opt(
            "description",
            value.description.as_ref(),
            write::description,
        )?;
        object.opt(
            "horizon_blend_stops",
            value.horizon_blend_stops.as_ref(),
            write_stops,
        )?;
        write_ranged(
            &mut object,
            RAYLEIGH,
            "rayleigh_strength",
            value.rayleigh_strength.as_ref(),
        )?;
        write_ranged(
            &mut object,
            SUN_MIE,
            "sun_mie_strength",
            value.sun_mie_strength.as_ref(),
        )?;
        write_ranged(
            &mut object,
            MOON_MIE,
            "moon_mie_strength",
            value.moon_mie_strength.as_ref(),
        )?;
        write_ranged(
            &mut object,
            SUN_GLARE,
            "sun_glare_shape",
            value.sun_glare_shape.as_ref(),
        )?;
        object.opt(
            "sky_zenith_color",
            value.sky_zenith_color.as_ref(),
            |frames, at| write::keyframes(frames, at, write::rgb),
        )?;
        object.opt(
            "sky_horizon_color",
            value.sky_horizon_color.as_ref(),
            |frames, at| write::keyframes(frames, at, write::rgb),
        )?;
        object.finish(&value.extra, MEMBERS)
    }

    impl VersionStruct for v1_21_40::AtmosphereSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            settings(&read::payload(payload, cx), false, d)
                .ok()
                .map(Self)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_settings(&self.0, false)
        }
    }

    impl VersionStruct for AtmosphereSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            settings(&read::payload(payload, cx), true, d).ok()
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_settings(self, true)
        }
    }

    /// 1.21.40 → 1.26.20: every member is copied; only the required flags differ.
    impl Upgrade for v1_21_40::AtmosphereSettings {
        type Next = AtmosphereSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> AtmosphereSettings {
            self.0
        }
    }
}

impl AtmosphereSettings {
    /// Atmosphere settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: AtmosphereSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}
