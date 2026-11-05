//! [`Keyframes`]: a value that may change over the day, one value or values by time.
//!
//! The type only: what a time key may be (any finite number, or `[0, 1]` with the rest
//! dropped), how keys are spelt on write (`"0"` or `"0.0"`) and the findings of a bad key are
//! each family's reader and writer.

use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};

/// One keyframe: a value at a time of day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe<T> {
    /// The time of day: 0 is noon, 0.25 sunset, 0.5 midnight, 0.75 sunrise.
    pub time: f32,
    /// The value at that time.
    pub value: T,
}

/// A value that may change over the day: one value, or keyframes sorted by time
/// (JSON `{"<time>": value, …}`). The game interpolates linearly between keyframes.
#[derive(Clone, Debug, PartialEq)]
pub enum Keyframes<T> {
    /// A single value, the same all day (JSON: the value itself).
    Single(T),
    /// Keyframes sorted by time (JSON: the object form).
    Keys(Vec<Keyframe<T>>),
}

impl<T> Keyframes<T> {
    /// Keyframes from `(time, value)` pairs, in the order given.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (f32, T)>) -> Self {
        Self::Keys(
            pairs
                .into_iter()
                .map(|(time, value)| Keyframe { time, value })
                .collect(),
        )
    }

    /// One keyframe at time 0: what the game's upgrade steps turn a plain value into.
    #[must_use]
    pub fn at_noon(value: T) -> Self {
        Self::Keys(vec![Keyframe { time: 0.0, value }])
    }

    /// The same keyframes with each value converted by `convert`.
    #[must_use]
    pub fn map<U>(self, mut convert: impl FnMut(T) -> U) -> Keyframes<U> {
        match self {
            Self::Single(value) => Keyframes::Single(convert(value)),
            Self::Keys(frames) => Keyframes::Keys(
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

    /// The one value, when the member is not keyed by time.
    pub fn single(&self) -> Option<&T> {
        match self {
            Self::Single(value) => Some(value),
            Self::Keys(_) => None,
        }
    }

    /// Every value the member holds, with its time key (`None` for a single value).
    pub fn values(&self) -> Vec<(Option<f32>, &T)> {
        match self {
            Self::Single(value) => vec![(None, value)],
            Self::Keys(keys) => keys
                .iter()
                .map(|frame| (Some(frame.time), &frame.value))
                .collect(),
        }
    }
}

impl<T: Clone> Keyframes<T> {
    /// The keyframes as the game holds them: a single value is one keyframe at time 0.
    #[must_use]
    pub fn frames(&self) -> Vec<Keyframe<T>> {
        match self {
            Self::Single(value) => vec![Keyframe {
                time: 0.0,
                value: value.clone(),
            }],
            Self::Keys(frames) => frames.clone(),
        }
    }
}

/// Each value is walked where it is: the single value at the member, a keyframe's at the
/// member too (the time as the family spells it is not known here, so the walk takes no step
/// and a keyframe's Molang is reported at the member, not at its key). No family walks a
/// keyframed value today (fog and the deferred settings hold no Molang); a family that does
/// needs the key step first.
impl<T: MolangFields> MolangFields for Keyframes<T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        match self {
            Self::Single(value) => value.visit_molang(visitor),
            Self::Keys(frames) => {
                for frame in frames {
                    frame.value.visit_molang(visitor);
                }
            }
        }
    }
}

/// A keyframed value holds no bound the derive checks and no trigger.
impl<T> SchemaFields for Keyframes<T> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}
