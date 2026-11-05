//! `minecraft:pbr_fallback_settings`: the material values of textures without a texture set.

use bedrock_addon_core::document::UnknownMembers;

/// The fallback material values of the deferred renderer (`minecraft:pbr_fallback_settings`,
/// the one file `pbr/global.json`), at its only format version (1.21.40).
///
/// JSON: `{"format_version": "1.21.40", "minecraft:pbr_fallback_settings": {"blocks"?: {…},
/// "actors"?: {…}, "particles"?: {…}, "items"?: {…}}}`; each kind is a [`PbrFallback`]. A kind
/// the document leaves out is not written back. The highest-priority pack whose file loads
/// decides, whole.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "minecraft:pbr_fallback_settings")]
#[doc(alias = "pbr")]
pub struct PbrFallbackSettings {
    /// The values of blocks (JSON `blocks`; see [`PbrFallbackSettings::blocks`]).
    pub blocks: Option<PbrFallback>,
    /// The values of entities (JSON `actors`; see [`PbrFallbackSettings::actors`]).
    pub actors: Option<PbrFallback>,
    /// The values of particles (JSON `particles`; see [`PbrFallbackSettings::particles`]).
    pub particles: Option<PbrFallback>,
    /// The values of items (JSON `items`; see [`PbrFallbackSettings::items`]).
    pub items: Option<PbrFallback>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The fallback values of one kind of object.
///
/// JSON: `{"global_metalness_emissive_roughness_subsurface"?: [m, e, r, s]}`, or the older
/// form `{"global_metalness_emissive_roughness": [m, e, r]}`. Each value is `0`–`255`; the
/// four-value form may be written `"#SSMMEERR"` (subsurface first), the three-value form
/// `"#MMEERR"`. An object with both members is read by the four-value one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PbrFallback {
    /// The values as written; `None` when the object names neither member (see
    /// [`PbrFallback::mers`]).
    pub value: Option<PbrValue>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

/// The two forms of a kind's fallback values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PbrValue {
    /// `global_metalness_emissive_roughness_subsurface`.
    Mers(Mers),
    /// `global_metalness_emissive_roughness`: the game reads it with subsurface 0.
    Mer(Mer),
}

/// Metalness, emissive, roughness and subsurface, each 0–255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mers {
    /// How metallic the surface is.
    pub metalness: u8,
    /// How much light the surface gives off.
    pub emissive: u8,
    /// How rough the surface is.
    pub roughness: u8,
    /// How much light scatters below the surface.
    pub subsurface: u8,
}

impl Mers {
    /// `[0, 0, 255, 0]`: the game's values for every kind without a file.
    pub const DEFAULT: Self = Self {
        metalness: 0,
        emissive: 0,
        roughness: 255,
        subsurface: 0,
    };
}

/// Metalness, emissive and roughness, each 0–255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mer {
    /// How metallic the surface is.
    pub metalness: u8,
    /// How much light the surface gives off.
    pub emissive: u8,
    /// How rough the surface is.
    pub roughness: u8,
}

impl PbrFallback {
    /// The values the game uses: the four written, the three written with subsurface 0, or
    /// [`Mers::DEFAULT`] when the object names neither.
    #[must_use]
    pub fn mers(&self) -> Mers {
        match self.value {
            Some(PbrValue::Mers(mers)) => mers,
            Some(PbrValue::Mer(mer)) => Mers {
                metalness: mer.metalness,
                emissive: mer.emissive,
                roughness: mer.roughness,
                subsurface: 0,
            },
            None => Mers::DEFAULT,
        }
    }
}

impl PbrFallbackSettings {
    /// The values the game uses for blocks.
    #[must_use]
    pub fn blocks(&self) -> Mers {
        self.blocks
            .as_ref()
            .map_or(Mers::DEFAULT, PbrFallback::mers)
    }

    /// The values the game uses for entities.
    #[must_use]
    pub fn actors(&self) -> Mers {
        self.actors
            .as_ref()
            .map_or(Mers::DEFAULT, PbrFallback::mers)
    }

    /// The values the game uses for particles.
    #[must_use]
    pub fn particles(&self) -> Mers {
        self.particles
            .as_ref()
            .map_or(Mers::DEFAULT, PbrFallback::mers)
    }

    /// The values the game uses for items.
    #[must_use]
    pub fn items(&self) -> Mers {
        self.items.as_ref().map_or(Mers::DEFAULT, PbrFallback::mers)
    }
}

mod codec {
    //! Reading and writing `minecraft:pbr_fallback_settings`.

    use super::{Mer, Mers, PbrFallback, PbrFallbackSettings, PbrValue};
    use crate::read::{self, Node, Read, members};
    use crate::value::{Rgb8, Rgba8};
    use crate::write::{self, Object, Written};
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
    use bedrock_addon_core::versioning::{ParseCx, VersionStruct};

    /// The four-value member of a kind.
    const MERS: &str = "global_metalness_emissive_roughness_subsurface";
    /// The three-value member of a kind.
    const MER: &str = "global_metalness_emissive_roughness";
    /// The kinds, in the order the game reads them.
    const KINDS: &[&str] = &["blocks", "actors", "particles", "items"];

    /// One kind's object. The four-value member is read when present; else the three-value
    /// member; an object with neither is the four-value form with the game's values, since a
    /// three-value form without its member cannot be told from it.
    fn kind(node: &Node<'_>, d: &mut Diagnostics) -> Read<PbrFallback> {
        let mut m = members(node, d)?;
        let mers = m.opt(MERS, d, read::rgba)?;
        let value = match mers {
            // `[m, e, r, s]` is `[r, g, b, a]`; the hex form has subsurface (alpha) first.
            Some(Rgba8 {
                red,
                green,
                blue,
                alpha,
            }) => Some(PbrValue::Mers(Mers {
                metalness: red,
                emissive: green,
                roughness: blue,
                subsurface: alpha,
            })),
            None => m.opt(MER, d, read::rgb)?.map(|Rgb8 { red, green, blue }| {
                PbrValue::Mer(Mer {
                    metalness: red,
                    emissive: green,
                    roughness: blue,
                })
            }),
        };
        Ok(PbrFallback {
            value,
            extra: m.finish(&[], d),
        })
    }

    /// One kind's object, in the form it was read in.
    fn write_kind(kind: &PbrFallback, at: &JsonPath) -> Written {
        let mut object = Object::new(at);
        match kind.value {
            Some(PbrValue::Mers(mers)) => object.req(
                MERS,
                &Rgba8::new(
                    mers.metalness,
                    mers.emissive,
                    mers.roughness,
                    mers.subsurface,
                ),
                write::rgba,
            )?,
            Some(PbrValue::Mer(mer)) => object.req(
                MER,
                &Rgb8::new(mer.metalness, mer.emissive, mer.roughness),
                write::rgb,
            )?,
            None => {}
        }
        // An unknown member named like the three-value member is one only beside the
        // four-value member, which the reader takes first.
        let known: &[&str] = match kind.value {
            Some(PbrValue::Mers(_)) => &[MERS],
            _ => &[MERS, MER],
        };
        object.finish(&kind.extra, known)
    }

    impl VersionStruct for PbrFallbackSettings {
        fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
            let mut m = members(&read::payload(payload, cx), d).ok()?;
            let blocks = m.opt("blocks", d, kind);
            let actors = m.opt("actors", d, kind);
            let particles = m.opt("particles", d, kind);
            let items = m.opt("items", d, kind);
            let extra = m.finish(&[], d);
            Some(Self {
                blocks: blocks.ok()?,
                actors: actors.ok()?,
                particles: particles.ok()?,
                items: items.ok()?,
                extra,
            })
        }

        fn write(&self) -> Result<Value, EncodeError> {
            let mut object = Object::new(&write::payload("minecraft:pbr_fallback_settings"));
            object.opt("blocks", self.blocks.as_ref(), write_kind)?;
            object.opt("actors", self.actors.as_ref(), write_kind)?;
            object.opt("particles", self.particles.as_ref(), write_kind)?;
            object.opt("items", self.items.as_ref(), write_kind)?;
            object.finish(&self.extra, KINDS)
        }
    }
}
