//! `minecraft:cubemap_settings`: how a cubemap (skybox) is lit.

use super::value::{Description, Keyframe, Keyframes};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::id::{CubemapSettingsId, kind};

/// The lighting of a cubemap (`minecraft:cubemap_settings`, in `cubemaps/`), at its only
/// format version (1.21.130).
///
/// JSON: `{"format_version": "1.21.130", "minecraft:cubemap_settings": {"description"?:
/// {"identifier": "ns:name"}, "lighting"?: {…}}}`. Client biomes name the settings by their
/// identifier (`minecraft:cubemap_identifier`); a biome without one uses the settings named
/// `minecraft:default_cubemap`, or the game's own values when no pack defines them. A file
/// without `description` loads but can be named by no biome.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:cubemap_settings")]
#[doc(alias = "cubemaps")]
#[doc(alias = "skybox")]
pub struct CubemapSettings {
    /// The settings' name (JSON `description`).
    pub description: Option<Description<kind::CubemapSettings>>,
    /// How the cubemap is lit (JSON `lighting`).
    pub lighting: Option<CubemapLighting>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// How a cubemap is lit (JSON `lighting`); every member is optional and not written back when
/// left out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CubemapLighting {
    /// The ambient light the cubemap gives off over the day, each value 0–100000 (JSON
    /// `ambient_light_illuminance`).
    pub ambient_light_illuminance: Option<Keyframes<f32>>,
    /// How much the sky light lights the cubemap, 0–1 (JSON `sky_light_contribution`).
    pub sky_light_contribution: Option<f32>,
    /// How much the sun and moon light the cubemap, 0–1 (JSON
    /// `directional_light_contribution`).
    pub directional_light_contribution: Option<f32>,
    /// Whether atmospheric scattering is drawn over the cubemap (JSON
    /// `affected_by_atmospheric_scattering`).
    pub affected_by_atmospheric_scattering: Option<bool>,
    /// Whether volumetric fog is drawn over the cubemap (JSON
    /// `affected_by_volumetric_scattering`).
    pub affected_by_volumetric_scattering: Option<bool>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

super::defaults! {
    CubemapLighting {
        ambient_light_illuminance: Keyframes<f32> = Keyframes::Keys(vec![
            Keyframe { time: 0.0, value: 5.625 },
            Keyframe { time: 1.0, value: 5.625 },
        ]);
        sky_light_contribution: f32 = 1.0;
        directional_light_contribution: f32 = 0.0;
        affected_by_atmospheric_scattering: bool = false;
        affected_by_volumetric_scattering: bool = true;
    }
}

mod codec {
    //! Reading and writing `minecraft:cubemap_settings`.

    use super::{CubemapLighting, CubemapSettings};
    use crate::read::{self, Node, Range, Read, members};
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The bounds of each `ambient_light_illuminance` value.
    const ILLUMINANCE: Range = Range::new(0.0, 100_000.0);
    /// The bounds of the two contributions.
    const CONTRIBUTION: Range = Range::new(0.0, 1.0);
    /// The members of `lighting`.
    const LIGHTING: &[&str] = &[
        "ambient_light_illuminance",
        "sky_light_contribution",
        "directional_light_contribution",
        "affected_by_atmospheric_scattering",
        "affected_by_volumetric_scattering",
    ];

    fn lighting(node: &Node<'_>, d: &mut Diagnostics) -> Read<CubemapLighting> {
        let mut m = members(node, d)?;
        let ambient_light_illuminance = m.opt("ambient_light_illuminance", d, |n, d| {
            read::keyframes(n, d, |n, d| read::float_in(ILLUMINANCE, n, d))
        });
        let sky_light_contribution = m.opt("sky_light_contribution", d, |n, d| {
            read::float_in(CONTRIBUTION, n, d)
        });
        let directional_light_contribution = m.opt("directional_light_contribution", d, |n, d| {
            read::float_in(CONTRIBUTION, n, d)
        });
        let atmospheric = m.opt("affected_by_atmospheric_scattering", d, read::boolean);
        let volumetric = m.opt("affected_by_volumetric_scattering", d, read::boolean);
        let extra = m.finish(&[], d);
        Ok(CubemapLighting {
            ambient_light_illuminance: ambient_light_illuminance?,
            sky_light_contribution: sky_light_contribution?,
            directional_light_contribution: directional_light_contribution?,
            affected_by_atmospheric_scattering: atmospheric?,
            affected_by_volumetric_scattering: volumetric?,
            extra,
        })
    }

    fn write_lighting(lighting: &CubemapLighting, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        object.opt(
            "ambient_light_illuminance",
            lighting.ambient_light_illuminance.as_ref(),
            |frames, at| write::keyframes(frames, at, |v, at| write::float_in(ILLUMINANCE, *v, at)),
        )?;
        object.opt(
            "sky_light_contribution",
            lighting.sky_light_contribution.as_ref(),
            |v, at| write::float_in(CONTRIBUTION, *v, at),
        )?;
        object.opt(
            "directional_light_contribution",
            lighting.directional_light_contribution.as_ref(),
            |v, at| write::float_in(CONTRIBUTION, *v, at),
        )?;
        object.opt(
            "affected_by_atmospheric_scattering",
            lighting.affected_by_atmospheric_scattering.as_ref(),
            write::boolean,
        )?;
        object.opt(
            "affected_by_volumetric_scattering",
            lighting.affected_by_volumetric_scattering.as_ref(),
            write::boolean,
        )?;
        object.finish(&lighting.extra, LIGHTING)
    }

    impl VersionStruct for CubemapSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let mut m = members(&read::payload(payload, cx), d).ok()?;
            let description = m.opt("description", d, read::description);
            let lighting = m.opt("lighting", d, lighting);
            let extra = m.finish(&[], d);
            Some(Self {
                description: description.ok()?,
                lighting: lighting.ok()?,
                extra,
            })
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload("minecraft:cubemap_settings"));
            object.opt("description", self.description.as_ref(), write::description)?;
            object.opt("lighting", self.lighting.as_ref(), write_lighting)?;
            object.finish(&self.extra, &["description", "lighting"])
        }
    }
}

impl CubemapSettings {
    /// Cubemap settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: CubemapSettingsId) -> Self {
        Self {
            description: Some(Description::new(identifier)),
            ..Self::default()
        }
    }
}
