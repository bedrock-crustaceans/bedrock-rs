//! `minecraft:water_settings`: how water is drawn.

use super::read::{self, Range};
use super::value::Description;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{WaterSettingsId, kind};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

pub(crate) const CHLOROPHYLL: Range = read::range(0.0, 10.0);
pub(crate) const SUSPENDED_SEDIMENT: Range = read::range(0.0, 300.0);
pub(crate) const CDOM: Range = read::range(0.0, 15.0);
pub(crate) const FREQUENCY: Range = read::range(0.01, 3.0);
pub(crate) const OCTAVES: (u8, u8) = (1, 30);
pub(crate) const DEPTH: Range = read::range(0.0, 3.0);
pub(crate) const SAMPLE_WIDTH: Range = read::range(0.01, 1.0);
pub(crate) const SPEED: Range = read::range(0.01, 10.0);
pub(crate) const PULL: Range = read::range(-1.0, 1.0);
pub(crate) const MIX: Range = read::range(0.0, 1.0);
pub(crate) const SCALING: Range = read::range(0.0, 2.0);
pub(crate) const DIRECTION_INCREMENT: Range = read::range(0.0, 360.0);
pub(crate) const SHAPE: Range = read::range(1.0, 10.0);
pub(crate) const FRAME_LENGTH: Range = read::range(0.01, 5.0);
pub(crate) const SCALE: Range = read::range(0.1, 5.0);
pub(crate) const POWER: Range = read::range(1.0, 6.0);
pub(crate) const CONTRIBUTION: Range = read::range(0.0, 1.0);

/// The water of the deferred renderer (`minecraft:water_settings`, in `water/`), at the newest
/// format version (1.26.20).
///
/// JSON: `{"format_version": "1.26.20", "minecraft:water_settings": {"description":
/// {"identifier": "ns:name"}, "particle_concentrations"?: {…}, "caustics"?: {…}, "waves"?:
/// {…}, "biome_water_color_contribution"?: 0–1}}`. Client biomes name the settings by their
/// identifier (`minecraft:water_identifier`); the identifier of the top pack's
/// `water/water.json` is the default of every other biome. A member the document leaves out is
/// not written back; the accessors of each part give the value the game then uses.
///
/// A 1.26.20 file must have `description`; an older file may leave it out, and then loads
/// without a name no biome can use ([`WaterSettings::description`] is `None`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[doc(alias = "minecraft:water_settings")]
#[doc(alias = "water")]
pub struct WaterSettings {
    /// The settings' name (JSON `description`); `None` only in a document upgraded from an
    /// older file without one.
    pub description: Option<Description<kind::WaterSettings>>,
    /// What is dissolved in the water, which colours it (JSON `particle_concentrations`).
    pub particle_concentrations: Option<ParticleConcentrations>,
    /// The light patterns on surfaces under water (JSON `caustics`, from format 1.21.80).
    pub caustics: Option<Caustics>,
    /// The waves of the surface (JSON `waves`).
    pub waves: Option<Waves>,
    /// How much the biome's water colour tints the water, 0–1 (JSON
    /// `biome_water_color_contribution`, from format 1.26.0).
    #[facet(schema::read_with = read::float_in(CONTRIBUTION))]
    pub biome_water_color_contribution: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// What is dissolved in the water (JSON `particle_concentrations`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct ParticleConcentrations {
    /// Chlorophyll, which tints the water green, 0–10 (JSON `chlorophyll`).
    #[facet(schema::read_with = read::float_in(CHLOROPHYLL))]
    pub chlorophyll: Option<f32>,
    /// Suspended sediment, which makes the water murky, 0–300 (JSON `suspended_sediment`).
    #[facet(schema::read_with = read::float_in(SUSPENDED_SEDIMENT))]
    pub suspended_sediment: Option<f32>,
    /// Coloured dissolved organic matter, which tints the water yellow-brown, 0–15 (JSON
    /// `cdom`).
    #[facet(schema::read_with = read::float_in(CDOM))]
    pub cdom: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

super::defaults! {
    ParticleConcentrations {
        chlorophyll: f32 = 0.5;
        suspended_sediment: f32 = 1.0;
        cdom: f32 = 0.5;
    }
}

/// The waves of the water surface (JSON `waves`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Waves {
    /// Whether the surface has waves (JSON `enabled`).
    pub enabled: Option<bool>,
    /// The frequency of the first wave, 0.01–3 (JSON `frequency`).
    #[facet(schema::read_with = read::float_in(FREQUENCY))]
    pub frequency: Option<f32>,
    /// How many waves are summed, 1–30 (JSON `octaves`).
    #[facet(schema::read_with = read::whole_in(OCTAVES))]
    pub octaves: Option<u8>,
    /// The height of the waves, 0–3 (JSON `depth`).
    #[facet(schema::read_with = read::float_in(DEPTH))]
    pub depth: Option<f32>,
    /// How fast the waves move, 0.01–10 (JSON `speed`).
    #[facet(schema::read_with = read::float_in(SPEED))]
    pub speed: Option<f32>,
    /// How strongly each wave drags the next, −1–1 (JSON `pull`).
    #[facet(schema::read_with = read::float_in(PULL))]
    pub pull: Option<f32>,
    /// How much each wave is blended with the next, 0–1 (JSON `mix`).
    #[facet(schema::read_with = read::float_in(MIX))]
    pub mix: Option<f32>,
    /// The frequency factor from one wave to the next, 0–2 (JSON `frequency_scaling`).
    #[facet(schema::read_with = read::float_in(SCALING))]
    pub frequency_scaling: Option<f32>,
    /// The speed factor from one wave to the next, 0–2 (JSON `speed_scaling`).
    #[facet(schema::read_with = read::float_in(SCALING))]
    pub speed_scaling: Option<f32>,
    /// The turn of direction from one wave to the next in degrees, 0–360 (JSON
    /// `direction_increment`, from format 1.21.80).
    #[facet(schema::read_with = read::float_in(DIRECTION_INCREMENT))]
    pub direction_increment: Option<f32>,
    /// How sharp the wave crests are, 1–10 (JSON `shape`).
    #[facet(schema::read_with = read::float_in(SHAPE))]
    pub shape: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

super::defaults! {
    Waves {
        enabled: bool = false;
        frequency: f32 = 1.0;
        octaves: u8 = 28;
        depth: f32 = 1.0;
        speed: f32 = 2.0;
        pull: f32 = 0.38;
        mix: f32 = 0.2;
        frequency_scaling: f32 = 1.2;
        speed_scaling: f32 = 1.03;
        direction_increment: f32 = 80.0;
        shape: f32 = 1.5;
    }
}

/// The light patterns on surfaces under water (JSON `caustics`).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Caustics {
    /// Whether caustics are drawn (JSON `enabled`).
    pub enabled: Option<bool>,
    /// The seconds each frame of the caustics texture is shown, 0.01–5 (JSON `frame_length`).
    #[facet(schema::read_with = read::float_in(FRAME_LENGTH))]
    pub frame_length: Option<f32>,
    /// The caustics texture, a pack path without its extension whose frames are stacked
    /// vertically (JSON `texture`).
    pub texture: Option<String>,
    /// The size of the pattern, 0.1–5 (JSON `scale`).
    #[facet(schema::read_with = read::float_in(SCALE))]
    pub scale: Option<f32>,
    /// The contrast of the pattern, 1–6 (JSON `power`).
    #[facet(schema::read_with = read::float_in(POWER))]
    pub power: Option<f32>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The game's own caustics texture, used when [`Caustics::texture`] is left out or cannot be
/// loaded.
pub const DEFAULT_CAUSTICS_TEXTURE: &str = "renderer/caustics";

super::defaults! {
    Caustics {
        enabled: bool = true;
        frame_length: f32 = 0.09;
        texture: String = DEFAULT_CAUSTICS_TEXTURE.to_owned();
        scale: f32 = 0.5;
        power: f32 = 1.0;
    }
}

super::defaults! {
    WaterSettings {
        particle_concentrations: ParticleConcentrations = ParticleConcentrations::default();
        caustics: Caustics = Caustics::default();
        waves: Waves = Waves::default();
        biome_water_color_contribution: f32 = 0.0;
    }
}

mod codec {
    //! Reading and writing `minecraft:water_settings` at its five format versions, and the
    //! steps between them.

    use super::{
        CDOM, CHLOROPHYLL, CONTRIBUTION, Caustics, DEPTH, DIRECTION_INCREMENT, FRAME_LENGTH,
        FREQUENCY, MIX, OCTAVES, POWER, PULL, ParticleConcentrations, SAMPLE_WIDTH, SCALE, SCALING,
        SHAPE, SPEED, SUSPENDED_SEDIMENT, WaterSettings, Waves,
    };
    use crate::history::water::{self as history, v1_21_40, v1_21_80, v1_21_120, v1_26_0};
    use crate::read::{self, Range};
    use crate::value::Description;
    use crate::write::{self, Object, Written};
    use crate::{V1_21_80, V1_21_120, V1_26_0};
    use bedrock_addon_core::diagnostic::codes as core_codes;
    use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::id::kind;
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};

    const KEY: &str = "minecraft:water_settings";

    // The members each struct reads as unknown, which another format version declares or has
    // dropped.
    const LATER_THAN_1_21_40: &[read::Hint] = &[
        read::later("caustics", V1_21_80),
        read::later("waves/direction_increment", V1_21_80),
        read::later("biome_water_color_contribution", V1_26_0),
    ];
    const LATER_THAN_1_21_80: &[read::Hint] =
        &[read::later("biome_water_color_contribution", V1_26_0)];
    const LATER_THAN_1_21_120: &[read::Hint] = &[
        read::later("biome_water_color_contribution", V1_26_0),
        read::gone("waves/sampleWidth", V1_21_120),
    ];
    const NEWEST: &[read::Hint] = &[read::gone("waves/sampleWidth", V1_21_120)];

    /// What a format version declares beyond 1.21.40's members.
    #[derive(Clone, Copy)]
    #[allow(
        clippy::struct_excessive_bools,
        reason = "one flag per change between the five versions"
    )]
    struct Shape {
        /// `waves.sampleWidth` (1.21.40 and 1.21.80).
        sample_width: bool,
        /// `caustics` and `waves.direction_increment` (from 1.21.80).
        caustics: bool,
        /// `biome_water_color_contribution` (from 1.26.0).
        contribution: bool,
        /// `description` is required (1.26.20).
        description_required: bool,
    }

    const AT_1_21_40: Shape = Shape {
        sample_width: true,
        caustics: false,
        contribution: false,
        description_required: false,
    };
    const AT_1_21_80: Shape = Shape {
        caustics: true,
        ..AT_1_21_40
    };
    const AT_1_21_120: Shape = Shape {
        sample_width: false,
        ..AT_1_21_80
    };
    const AT_1_26_0: Shape = Shape {
        contribution: true,
        ..AT_1_21_120
    };
    const AT_1_26_20: Shape = Shape {
        description_required: true,
        ..AT_1_26_0
    };

    /// The waves of 1.21.120 and later: `sample_width` is not a member there.
    fn newer(waves: history::Waves) -> Waves {
        Waves {
            enabled: waves.enabled,
            frequency: waves.frequency,
            octaves: waves.octaves,
            depth: waves.depth,
            speed: waves.speed,
            pull: waves.pull,
            mix: waves.mix,
            frequency_scaling: waves.frequency_scaling,
            speed_scaling: waves.speed_scaling,
            direction_increment: waves.direction_increment,
            shape: waves.shape,
            extra: waves.extra,
        }
    }

    /// The waves of 1.21.120 and later as the superset the writer takes.
    fn older(waves: &Waves) -> history::Waves {
        history::Waves {
            enabled: waves.enabled,
            frequency: waves.frequency,
            octaves: waves.octaves,
            depth: waves.depth,
            sample_width: None,
            speed: waves.speed,
            pull: waves.pull,
            mix: waves.mix,
            frequency_scaling: waves.frequency_scaling,
            speed_scaling: waves.speed_scaling,
            direction_increment: waves.direction_increment,
            shape: waves.shape,
            extra: waves.extra.clone(),
        }
    }

    fn write_particles(value: &ParticleConcentrations, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        write_ranged(&mut object, CHLOROPHYLL, "chlorophyll", value.chlorophyll)?;
        write_ranged(
            &mut object,
            SUSPENDED_SEDIMENT,
            "suspended_sediment",
            value.suspended_sediment,
        )?;
        write_ranged(&mut object, CDOM, "cdom", value.cdom)?;
        object.finish(&value.extra, &["chlorophyll", "suspended_sediment", "cdom"])
    }

    /// Writes a ranged number member.
    fn write_ranged(
        object: &mut Object,
        range: Range,
        key: &str,
        value: Option<f32>,
    ) -> Written<()> {
        object.opt(key, value.as_ref(), |v, at| write::float_in(range, *v, at))
    }

    fn write_caustics(value: &Caustics, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt("enabled", value.enabled.as_ref(), write::boolean)?;
        write_ranged(
            &mut object,
            FRAME_LENGTH,
            "frame_length",
            value.frame_length,
        )?;
        object.opt("texture", value.texture.as_ref(), write::string)?;
        write_ranged(&mut object, SCALE, "scale", value.scale)?;
        write_ranged(&mut object, POWER, "power", value.power)?;
        object.finish(
            &value.extra,
            &["enabled", "frame_length", "texture", "scale", "power"],
        )
    }

    /// The members of `waves` over every version.
    const WAVES: &[&str] = &[
        "enabled",
        "frequency",
        "octaves",
        "depth",
        "sampleWidth",
        "speed",
        "pull",
        "mix",
        "frequency_scaling",
        "speed_scaling",
        "direction_increment",
        "shape",
    ];

    /// `waves` with the members `shape` declares; a value holding another member has no form
    /// in that version.
    fn write_waves(shape: Shape, value: &history::Waves, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        if value.sample_width.is_some() && !shape.sample_width {
            return Err(write::refuse(&object.path("sampleWidth")));
        }
        if value.direction_increment.is_some() && !shape.caustics {
            return Err(write::refuse(&object.path("direction_increment")));
        }
        object.opt("enabled", value.enabled.as_ref(), write::boolean)?;
        write_ranged(&mut object, FREQUENCY, "frequency", value.frequency)?;
        object.opt("octaves", value.octaves.as_ref(), |octaves, at| {
            write::integer_in(
                i64::from(OCTAVES.0),
                i64::from(OCTAVES.1),
                i64::from(*octaves),
                at,
            )
        })?;
        write_ranged(&mut object, DEPTH, "depth", value.depth)?;
        write_ranged(&mut object, SAMPLE_WIDTH, "sampleWidth", value.sample_width)?;
        write_ranged(&mut object, SPEED, "speed", value.speed)?;
        write_ranged(&mut object, PULL, "pull", value.pull)?;
        write_ranged(&mut object, MIX, "mix", value.mix)?;
        write_ranged(
            &mut object,
            SCALING,
            "frequency_scaling",
            value.frequency_scaling,
        )?;
        write_ranged(&mut object, SCALING, "speed_scaling", value.speed_scaling)?;
        write_ranged(
            &mut object,
            DIRECTION_INCREMENT,
            "direction_increment",
            value.direction_increment,
        )?;
        write_ranged(&mut object, SHAPE, "shape", value.shape)?;
        // A member the version does not declare is an unknown member there, and is written
        // back as one.
        let known: Vec<&str> = WAVES
            .iter()
            .copied()
            .filter(|key| match *key {
                "sampleWidth" => shape.sample_width,
                "direction_increment" => shape.caustics,
                _ => true,
            })
            .collect();
        object.finish(&value.extra, &known)
    }

    /// The payload with the members `shape` declares.
    fn write_settings(
        shape: Shape,
        description: Option<&Description<kind::WaterSettings>>,
        particle_concentrations: Option<&ParticleConcentrations>,
        caustics: Option<&Caustics>,
        waves: Option<&history::Waves>,
        contribution: Option<f32>,
        extra: &UnknownMembers,
    ) -> Written {
        let mut object = Object::new(&write::payload(KEY));
        if shape.description_required && description.is_none() {
            return Err(write::refuse(&object.path("description")));
        }
        object.opt("description", description, write::description)?;
        object.opt(
            "particle_concentrations",
            particle_concentrations,
            write_particles,
        )?;
        object.opt("caustics", caustics, write_caustics)?;
        object.opt("waves", waves, |waves, at| write_waves(shape, waves, at))?;
        write_ranged(
            &mut object,
            CONTRIBUTION,
            "biome_water_color_contribution",
            contribution,
        )?;
        let mut known = vec!["description", "particle_concentrations", "waves"];
        if shape.caustics {
            known.push("caustics");
        }
        if shape.contribution {
            known.push("biome_water_color_contribution");
        }
        object.finish(extra, &known)
    }

    impl VersionStruct for v1_21_40::WaterSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_hinted(payload, cx, d, LATER_THAN_1_21_40)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_settings(
                AT_1_21_40,
                self.description.as_ref(),
                self.particle_concentrations.as_ref(),
                None,
                self.waves.as_ref(),
                None,
                &self.extra,
            )
        }
    }

    impl VersionStruct for v1_21_80::WaterSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_hinted(payload, cx, d, LATER_THAN_1_21_80)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_settings(
                AT_1_21_80,
                self.description.as_ref(),
                self.particle_concentrations.as_ref(),
                self.caustics.as_ref(),
                self.waves.as_ref(),
                None,
                &self.extra,
            )
        }
    }

    impl VersionStruct for v1_21_120::WaterSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_hinted(payload, cx, d, LATER_THAN_1_21_120)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_settings(
                AT_1_21_120,
                self.description.as_ref(),
                self.particle_concentrations.as_ref(),
                self.caustics.as_ref(),
                self.waves.as_ref().map(older).as_ref(),
                None,
                &self.extra,
            )
        }
    }

    fn write_newest(shape: Shape, value: &WaterSettings) -> Written {
        write_settings(
            shape,
            value.description.as_ref(),
            value.particle_concentrations.as_ref(),
            value.caustics.as_ref(),
            value.waves.as_ref().map(older).as_ref(),
            value.biome_water_color_contribution,
            &value.extra,
        )
    }

    impl VersionStruct for v1_26_0::WaterSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_hinted(payload, cx, d, NEWEST)
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_newest(AT_1_26_0, &self.0)
        }
    }

    impl VersionStruct for WaterSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            read::payload_requiring(payload, cx, d, NEWEST, "description")
        }

        fn write(&self) -> Result<Value, EncodeError> {
            write_newest(AT_1_26_20, self)
        }
    }

    /// 1.21.40 → 1.21.80: every member is copied; `caustics` and
    /// `waves.direction_increment` stay absent (the game's own values apply). An unknown
    /// member of the old document that 1.21.80 declares is superseded.
    impl Upgrade for v1_21_40::WaterSettings {
        type Next = v1_21_80::WaterSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_21_80::WaterSettings {
            let mut extra = self.extra;
            extra.remove("caustics");
            let waves = self.waves.map(|mut waves| {
                waves.extra.remove("direction_increment");
                waves
            });
            v1_21_80::WaterSettings {
                description: self.description,
                particle_concentrations: self.particle_concentrations,
                caustics: None,
                waves,
                extra,
            }
        }
    }

    /// 1.21.80 → 1.21.120: every member is copied but `waves.sampleWidth`, which the copy
    /// ignores: dropped, with a `DroppedField` info (the game is silent, so the finding is the
    /// crate's own).
    impl Upgrade for v1_21_80::WaterSettings {
        type Next = v1_21_120::WaterSettings;

        fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_21_120::WaterSettings {
            let waves = self.waves.map(|waves| {
                if waves.sample_width.is_some() {
                    let mut at = write::payload(KEY);
                    at.push("waves");
                    at.push("sampleWidth");
                    cx.diagnostics.push(
                        Diagnostic::of(&core_codes::DROPPED_FIELD)
                            .at(at)
                            .phase(Phase::Upgrade),
                    );
                }
                newer(waves)
            });
            v1_21_120::WaterSettings {
                description: self.description,
                particle_concentrations: self.particle_concentrations,
                caustics: self.caustics,
                waves,
                extra: self.extra,
            }
        }
    }

    /// 1.21.120 → 1.26.0: every member is copied; `biome_water_color_contribution` stays
    /// absent. An unknown member of that name in the old document is superseded.
    impl Upgrade for v1_21_120::WaterSettings {
        type Next = v1_26_0::WaterSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> v1_26_0::WaterSettings {
            let mut extra = self.extra;
            extra.remove("biome_water_color_contribution");
            v1_26_0::WaterSettings(WaterSettings {
                description: self.description,
                particle_concentrations: self.particle_concentrations,
                caustics: self.caustics,
                waves: self.waves,
                biome_water_color_contribution: None,
                extra,
            })
        }
    }

    /// 1.26.0 → 1.26.20: every member is copied; only the required flags differ, and a
    /// document without `description` stays without one.
    impl Upgrade for v1_26_0::WaterSettings {
        type Next = WaterSettings;

        fn upgrade(self, _: &mut UpgradeCx<'_>) -> WaterSettings {
            self.0
        }
    }
}

impl WaterSettings {
    /// Water settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: WaterSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}
