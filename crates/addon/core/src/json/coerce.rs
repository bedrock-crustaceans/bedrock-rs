//! jsoncpp's integer reading of a [`Value`]: `isInt` and `asInt`, behind a [`Coercing`]
//! profile that names the rules the hand-written readers disagree on.
//!
//! The tree holds two `isInt` rules, both quoted as jsoncpp's, and the game has not been asked
//! which a given member follows: [`IsIntRule::Token`] (a real token is a real even when it is
//! integral: `json::kit::fixed_path`, `shared::read`, the strict wrappers, recipe) and
//! [`IsIntRule::Integral`] (an integral real counts: `ui_texture_info`, loot, feature, whose
//! evidence is vanilla's geode feature writing `2.0`). `asInt` agrees everywhere but on an
//! integer token past `i64`: [`WideInt::LowWord`] (texture, persona, entity) or
//! [`WideInt::Zero`] (`json::jsoncpp::fork`). So there is no default profile: a reader names
//! the one its copy followed.

use super::Value;

/// Which numbers `isInt` takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IsIntRule {
    /// An integer token in `i32`; a real token never, even `2.0`.
    Token,
    /// An integer token in `i32`, or a real token whose value is integral and in `i32`.
    Integral,
}

/// What `asInt` gives an integer token past `i64` (a `u64` above `i64::MAX`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WideInt {
    /// Its low 32 bits, as the unchecked cast of a release build loads them.
    LowWord,
    /// 0.
    Zero,
}

/// The rules of one jsoncpp-style reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Coercing {
    /// `isInt`.
    pub is_int: IsIntRule,
    /// `asInt` on an integer token past `i64`.
    pub wide_int: WideInt,
}

impl Coercing {
    /// A profile from its two rules.
    pub const fn new(is_int: IsIntRule, wide_int: WideInt) -> Self {
        Self { is_int, wide_int }
    }
}

/// jsoncpp's integer reading on a [`Value`] (a foreign type, so a sealed extension trait).
pub trait Coerce: seal::Sealed {
    /// `isInt`, with the value when it holds, under `profile`'s [`IsIntRule`].
    fn is_int(&self, profile: Coercing) -> Option<i32>;

    /// `asInt(default)`: `null` is the default; a bool is 0 or 1; a real is cut toward zero
    /// and saturates at the `i32` bounds (NaN is 0); an integer token keeps its low 32 bits,
    /// past `i64` as `profile`'s [`WideInt`] says; a string, an array or an object is 0.
    fn as_int(&self, default: i32, profile: Coercing) -> i32;
}

mod seal {
    /// Only [`Value`](super::Value) coerces.
    pub trait Sealed {}
    impl Sealed for super::Value {}
}

impl Coerce for Value {
    fn is_int(&self, profile: Coercing) -> Option<i32> {
        let number = self.as_number()?;
        if !number.is_float() {
            return number.to_i32();
        }
        match profile.is_int {
            IsIntRule::Token => None,
            IsIntRule::Integral => {
                let real = number.to_f64()?;
                let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&real);
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "checked to be integral and in range"
                )]
                (in_range && real.fract() == 0.0).then_some(real as i32)
            }
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "a float-to-int `as` saturates, which is the game's conversion; an integer \
                  keeps its low 32 bits"
    )]
    fn as_int(&self, default: i32, profile: Coercing) -> i32 {
        if self.is_null() {
            return default;
        }
        if let Some(flag) = self.as_bool() {
            return i32::from(flag);
        }
        let Some(number) = self.as_number() else {
            return 0;
        };
        if number.is_float() {
            return number.to_f64_lossy() as i32;
        }
        match (number.to_i64(), profile.wide_int) {
            (Some(wide), _) => wide as i32,
            (None, WideInt::LowWord) => number.to_u64().map_or(0, |wide| wide as i32),
            (None, WideInt::Zero) => 0,
        }
    }
}
