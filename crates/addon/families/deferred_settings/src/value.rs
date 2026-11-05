//! The value forms every deferred-settings payload shares: colours, keyframes, a triple of
//! numbers and the `description` object.

use std::fmt;

use bedrock_addon_core::document::UnknownMembers;
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

/// A value that may change over the day (shapes'). The settings' reader takes a single value,
/// or an object `{"<time>": value, …}` with at least one member whose keys are numbers written
/// as strings, and drops a key outside `[0, 1]` on load, as the game drops it.
pub use bedrock_addon_shared_shapes::{Keyframe, Keyframes};

/// Three numbers, one per colour channel (the colour-grading controls).
///
/// JSON: `[x, y, z]`, or `{"x": …, "y": …, "z": …}` in which a member left out is 0. This is
/// shapes' type, but the settings read and write it with their own reader and writer, not with
/// its facet proxy: the reader always gives the array form with no `extra` (an unknown member of
/// the object is reported and dropped), and the writer always writes `[x, y, z]`, so a value
/// built with `form: VecForm::Object` or with `extra` is written as the array and its `extra`
/// is dropped without a finding. Entity goals read and write the same type through the proxy
/// and keep both forms.
pub use bedrock_addon_shared_shapes::Vec3;

/// The `description` object of a settings file: the identifier client biomes name the settings
/// by (JSON `{"identifier": "ns:name"}`; `identifier` is required inside it).
///
/// The game checks neither a namespace nor the characters of the identifier.
pub struct Description<K: Kind> {
    /// The settings' name (JSON `identifier`).
    pub identifier: Id<K>,
    /// Members the game does not know, kept for write-back.
    pub extra: UnknownMembers,
}

impl<K: Kind> Description<K> {
    /// A description naming `identifier`.
    #[must_use]
    pub fn new(identifier: Id<K>) -> Self {
        Self {
            identifier,
            extra: UnknownMembers::default(),
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
