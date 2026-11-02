//! The value forms every deferred-settings payload shares: colours, keyframes, a triple of
//! numbers and the `description` object.

use std::fmt;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{Id, Kind};

/// A colour without alpha, each channel 0–255.
///
/// JSON: `[r, g, b]` or `"#RRGGBB"`; written back as the array. An array element outside
/// 0–255 is read as the nearest bound, as the game does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    /// Red.
    pub red: u8,
    /// Green.
    pub green: u8,
    /// Blue.
    pub blue: u8,
}

impl Rgb8 {
    /// White, `[255, 255, 255]`: the game's default for every light colour.
    pub const WHITE: Self = Self::new(255, 255, 255);

    /// The colour of three channels.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// A colour with alpha, each channel a byte, 0–255.
///
/// Named for its bytes: a render controller's `Rgba` holds the four Molang channels of its
/// colour, and the two are not the same kind of value.
///
/// JSON: `[r, g, b, a]` or `"#AARRGGBB"` (alpha is the first byte of the hex form); written
/// back as the array.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgba8 {
    /// Red.
    pub red: u8,
    /// Green.
    pub green: u8,
    /// Blue.
    pub blue: u8,
    /// Alpha.
    pub alpha: u8,
}

impl Rgba8 {
    /// The colour of four channels.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// The colour without its alpha.
    #[must_use]
    pub const fn rgb(self) -> Rgb8 {
        Rgb8::new(self.red, self.green, self.blue)
    }
}

/// One keyframe: a value at a time of day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe<T> {
    /// The time of day in `[0, 1]`: 0 is noon, 0.25 sunset, 0.5 midnight, 0.75 sunrise.
    pub time: f32,
    /// The value at that time.
    pub value: T,
}

/// A value that may change over the day.
///
/// JSON: a single value, or an object `{"<time>": value, …}` with at least one member whose
/// keys are numbers written as strings. The game interpolates linearly between keyframes.
#[derive(Clone, Debug, PartialEq)]
pub enum Keyframes<T> {
    /// A single value, the same all day (JSON: the value itself).
    Constant(T),
    /// Keyframes sorted by time (JSON: the object form). A key outside `[0, 1]` is dropped on
    /// load, as the game drops it.
    Frames(Vec<Keyframe<T>>),
}

impl<T> Keyframes<T> {
    /// One keyframe at time 0: what the game's upgrade steps turn a plain value into.
    #[must_use]
    pub fn at_noon(value: T) -> Self {
        Self::Frames(vec![Keyframe { time: 0.0, value }])
    }

    /// The same keyframes with each value converted by `convert`.
    #[must_use]
    pub fn map<U>(self, mut convert: impl FnMut(T) -> U) -> Keyframes<U> {
        match self {
            Self::Constant(value) => Keyframes::Constant(convert(value)),
            Self::Frames(frames) => Keyframes::Frames(
                frames
                    .into_iter()
                    .map(|frame| Keyframe {
                        time: frame.time,
                        value: convert(frame.value),
                    })
                    .collect(),
            ),
        }
    }
}

impl<T: Clone> Keyframes<T> {
    /// The keyframes as the game holds them: a single value is one keyframe at time 0.
    #[must_use]
    pub fn frames(&self) -> Vec<Keyframe<T>> {
        match self {
            Self::Constant(value) => vec![Keyframe {
                time: 0.0,
                value: value.clone(),
            }],
            Self::Frames(frames) => frames.clone(),
        }
    }
}

/// Three numbers, one per colour channel (the colour-grading controls).
///
/// JSON: `[x, y, z]`, or `{"x": …, "y": …, "z": …}` in which a member left out is 0; written
/// back as the array.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// The first component (red).
    pub x: f32,
    /// The second component (green).
    pub y: f32,
    /// The third component (blue).
    pub z: f32,
}

impl Vec3 {
    /// The triple of three numbers.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// The same number three times.
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }
}

/// The `description` object of a settings file: the identifier client biomes name the settings
/// by (JSON `{"identifier": "ns:name"}`; `identifier` is required inside it).
///
/// The game checks neither a namespace nor the characters of the identifier.
pub struct Description<K: Kind> {
    /// The settings' name (JSON `identifier`).
    pub identifier: Id<K>,
    /// Members the game does not know, kept for write-back.
    pub extra: Extra,
}

impl<K: Kind> Description<K> {
    /// A description naming `identifier`.
    #[must_use]
    pub fn new(identifier: Id<K>) -> Self {
        Self {
            identifier,
            extra: Extra::default(),
        }
    }
}

impl<K: Kind> Clone for Description<K> {
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            extra: self.extra.clone(),
        }
    }
}

impl<K: Kind> fmt::Debug for Description<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Description")
            .field("identifier", &self.identifier)
            .field("extra", &self.extra)
            .finish()
    }
}

impl<K: Kind> PartialEq for Description<K> {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier && self.extra == other.extra
    }
}
