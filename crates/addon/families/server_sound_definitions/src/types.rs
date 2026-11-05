//! The server sound definitions types and their inherent items.

use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{StrictFloat, StrictString};

/// A behaviour pack's server sound definitions file, `sounds/sound_definitions.json`
/// (`#[doc(alias)]` `server_sound_definitions`): the payload is the array itself.
#[doc(alias = "server_sound_definitions")]
#[doc(alias = "sound_definitions.json")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct SoundMetadataFile(pub Vec<SoundMetadata>);

/// The server's metadata of one sound event (`bp.server_sound_definitions()`): how long it lasts, what
/// music it is, the tags scripts filter on, and how far a broadcast reaches. The audio itself
/// is the resource pack's.
#[doc(alias = "server_sound_definitions")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SoundMetadata {
    /// `sound_event_name`: the namespaced sound event this is the metadata of. No whitespace;
    /// a namespace other than `minecraft` is required.
    pub sound_event_name: StrictString,
    /// `duration_info`: how long the sound lasts, so the server knows when it ends.
    pub duration_info: Option<DurationInfo>,
    /// `music_info`.
    pub music_info: Option<MusicInfo>,
    /// `tags`: pack-defined tags, each one value or a list of values; no key or value holds
    /// whitespace.
    pub tags: Option<IndexMap<String, TagValueList>>,
    /// `broadcast_range`: how far from the sound a broadcast play reaches a player (from
    /// engine 1.26.50; an older engine rejects the file).
    pub broadcast_range: Option<StrictFloat<f32>>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SoundMetadata {
    /// The sound event name as an identifier.
    pub fn id(&self) -> SoundEventName {
        SoundEventName::parse(self.sound_event_name.0.as_str())
    }

    /// The duration as the server stores it; `None` without `duration_info`.
    pub fn duration(&self) -> Option<SoundDuration> {
        self.duration_info.as_ref().map(DurationInfo::duration)
    }
}

/// `duration_info`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DurationInfo {
    /// `mode`: what the duration counts in.
    pub mode: DurationMode,
    /// `seconds`.
    pub seconds: StrictFloat<f32>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DurationInfo {
    /// The duration as vanilla converts it at load: game ticks (`seconds` × 20) or
    /// milliseconds (`seconds` × 1000), computed in `f32` and truncated. A product outside
    /// the target's range is the target's minimum, as the x86 conversion gives it.
    // The game multiplies in single precision and converts with a truncating cast.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "vanilla converts with a truncating cast"
    )]
    pub fn duration(&self) -> SoundDuration {
        match self.mode {
            DurationMode::GameTime => {
                let ticks = self.seconds.0 * 20.0_f32;
                // 2^31 is exact in f32; anything at or past it, or NaN, is out of range.
                SoundDuration::Ticks(if (-2_147_483_648.0..2_147_483_648.0).contains(&ticks) {
                    ticks as i32
                } else {
                    i32::MIN
                })
            }
            DurationMode::RealTime => {
                let millis = self.seconds.0 * 1000.0_f32;
                SoundDuration::Millis(
                    if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&millis)
                    {
                        millis as i64
                    } else {
                        i64::MIN
                    },
                )
            }
        }
    }
}

/// What a duration counts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[non_exhaustive]
#[repr(u8)]
pub enum DurationMode {
    /// `game_time`: the duration decrements by game tick.
    #[facet(rename = "game_time")]
    GameTime = 0,
    /// `real_time`: the duration decrements by wall-clock time.
    #[facet(rename = "real_time")]
    RealTime = 1,
}

/// A duration as the server tracks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SoundDuration {
    /// Game ticks.
    Ticks(i32),
    /// Milliseconds.
    Millis(i64),
}

/// `music_info`: free-form music metadata, every member optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MusicInfo {
    /// `genres`.
    pub genres: Option<Vec<StrictString>>,
    /// `moods`.
    pub moods: Option<Vec<StrictString>>,
    /// `artist`: display text or a translation key.
    pub artist: Option<StrictString>,
    /// `title`: display text or a translation key.
    pub title: Option<StrictString>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The value of one tag: one string or a list of strings.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum TagValueList {
    /// A single value.
    One(String),
    /// A list of values; it may be empty and may repeat a value.
    Many(Vec<String>),
}

impl TagValueList {
    /// The values: a single value is a list of one.
    pub fn values(&self) -> &[String] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::Many(values) => values,
        }
    }
}
