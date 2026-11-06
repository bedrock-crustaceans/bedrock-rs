//! [`Sides`]: the bounds of one field as the derived `check` evaluates them, both sides at
//! once, so a field with `min` and `max` (or `above` and `below`, or one of each) is one range
//! and one finding.

use super::walk::Number;
use crate::diagnostic::cereal::NumericRange;

/// Which side of a field's range a value broke.
// hidden: schema support the families' hand-written rows name; API not yet reviewed
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// The lower bound (`min` or `above`).
    Low,
    /// The upper bound (`max` or `below`).
    High,
}

/// A field's lower and upper bound, each absent where the field states none (or where a
/// cross-field limit names an absent member with nothing it stands for), and whether each is
/// exclusive.
// hidden: schema support the families' hand-written rows name; API not yet reviewed
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sides<T> {
    /// `min` or `above`.
    pub low: Option<T>,
    /// The lower bound is `above`.
    pub low_exclusive: bool,
    /// `max` or `below`.
    pub high: Option<T>,
    /// The upper bound is `below`.
    pub high_exclusive: bool,
}

impl<T: Number> Sides<T> {
    /// The side `value` breaks, the lower one first; `None` when it is inside. A value that
    /// compares with nothing (a NaN) breaks every bound there is.
    pub fn broken(&self, value: T) -> Option<Side> {
        if let Some(low) = self.low {
            let holds = if self.low_exclusive {
                value > low
            } else {
                value >= low
            };
            if !holds {
                return Some(Side::Low);
            }
        }
        if let Some(high) = self.high {
            let holds = if self.high_exclusive {
                value < high
            } else {
                value <= high
            };
            if !holds {
                return Some(Side::High);
            }
        }
        None
    }

    /// The range, as the finding states it.
    pub fn range(&self) -> NumericRange {
        NumericRange {
            min: self.low.map(Number::to_f64),
            max: self.high.map(Number::to_f64),
            exclusive_min: self.low_exclusive,
            exclusive_max: self.high_exclusive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITHIN: Sides<f32> = Sides {
        low: Some(0.0),
        low_exclusive: false,
        high: Some(0.9),
        high_exclusive: true,
    };

    #[test]
    fn a_value_breaks_the_side_it_is_past_and_nan_breaks_the_first_there_is() {
        assert_eq!(WITHIN.broken(0.0), None);
        assert_eq!(WITHIN.broken(-0.1), Some(Side::Low));
        assert_eq!(WITHIN.broken(0.9), Some(Side::High));
        assert_eq!(WITHIN.broken(f32::NAN), Some(Side::Low));
        let high_only = Sides {
            low: None,
            ..WITHIN
        };
        assert_eq!(high_only.broken(f32::NAN), Some(Side::High));
        let none = Sides {
            high: None,
            ..high_only
        };
        assert_eq!(none.broken(f32::NAN), None);
    }

    #[test]
    fn the_range_states_both_sides_and_which_are_exclusive() {
        let range = WITHIN.range();
        assert_eq!(range.min, Some(0.0));
        assert_eq!(range.max, Some(f64::from(0.9_f32)));
        assert!(!range.exclusive_min && range.exclusive_max);
    }
}
