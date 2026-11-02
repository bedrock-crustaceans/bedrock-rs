//! What the generated round-trip tests of the two families share: a fixed-seed generator, the
//! JSON tree the documents are generated as, and the canonical form two JSON texts are
//! compared in (member order aside, the int / real tag of every number kept).

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::collections::BTreeMap;

use bedrock_addon_core::json::Value;

/// `SplitMix64`: a fixed seed gives the same documents on every run and machine.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number below `n`.
    pub fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    /// True `percent` times in a hundred.
    pub fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    /// One of `items`.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// A JSON value as generated. A number is its token, so `1e39`, `2` and `2.0` are written as
/// they are.
#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Bool(bool),
    Null,
    Num(&'static str),
    /// An integer the generator computes.
    Int(i64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn str(text: &str) -> Self {
        Self::Str(text.to_owned())
    }

    pub fn obj(members: &[(&str, J)]) -> Self {
        Self::Obj(
            members
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
        )
    }

    /// The JSON text.
    pub fn text(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    fn write(&self, out: &mut String) {
        match self {
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Null => out.push_str("null"),
            Self::Num(token) => out.push_str(token),
            Self::Int(int) => out.push_str(&int.to_string()),
            Self::Str(text) => quote(text, out),
            Self::Arr(elements) => {
                out.push('[');
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    element.write(out);
                }
                out.push(']');
            }
            Self::Obj(members) => {
                out.push('{');
                for (index, (key, value)) in members.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    quote(key, out);
                    out.push_str(": ");
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

fn quote(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A JSON value in the form two texts are compared in: members by key, every number with its
/// int / real tag (a real by its bits, so `2` and `2.0` differ and `0.1` equals only `0.1`).
#[derive(Clone, Debug, PartialEq)]
pub enum Canon {
    Null,
    Bool(bool),
    Int(i64),
    Real(u64),
    Str(String),
    Arr(Vec<Canon>),
    Obj(BTreeMap<String, Canon>),
}

impl Canon {
    pub fn real(value: f64) -> Self {
        Self::Real(value.to_bits())
    }

    /// The canonical form of a JSON text, read through the door every loader reads through
    /// (`json::from_str_jsonc`), so a number is the number the loaders see: an integer token
    /// beyond 64 bits is the real it spells, not a string of its digits.
    pub fn of_text(text: &str) -> Self {
        let value: Value = bedrock_addon_core::json::from_str_jsonc(text)
            .unwrap_or_else(|e| panic!("no JSON: {e}\n{text}"));
        Self::of(&value)
    }

    pub fn of(value: &Value) -> Self {
        if let Some(object) = value.as_object() {
            Self::Obj(
                object
                    .iter()
                    .map(|(key, member)| (key.as_str().to_owned(), Self::of(member)))
                    .collect(),
            )
        } else if let Some(array) = value.as_array() {
            Self::Arr(array.iter().map(Self::of).collect())
        } else if let Some(text) = value.as_string() {
            Self::Str(text.as_str().to_owned())
        } else if let Some(flag) = value.as_bool() {
            Self::Bool(flag)
        } else if let Some(number) = value.as_number() {
            match number.to_i64() {
                Some(int) if number.is_integer() => Self::Int(int),
                _ => Self::real(number.to_f64_lossy()),
            }
        } else {
            Self::Null
        }
    }

    /// The member at `path`, to change.
    pub fn at_mut(&mut self, path: &[&str]) -> Option<&mut Canon> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        match self {
            Self::Obj(members) => members.get_mut(*first)?.at_mut(rest),
            _ => None,
        }
    }

    /// Removes the member `key` of the object at `path`; whether it was there.
    pub fn remove(&mut self, path: &[&str], key: &str) -> bool {
        match self.at_mut(path) {
            Some(Self::Obj(members)) => members.remove(key).is_some(),
            _ => false,
        }
    }

    /// Removes the member `key` of the object at `path` when it is an empty object or array;
    /// whether it did.
    pub fn remove_if_empty(&mut self, path: &[&str], key: &str) -> bool {
        let empty = match self.at_mut(path) {
            Some(Self::Obj(members)) => match members.get(key) {
                Some(Self::Obj(inner)) => inner.is_empty(),
                Some(Self::Arr(inner)) => inner.is_empty(),
                _ => false,
            },
            _ => false,
        };
        empty && self.remove(path, key)
    }

    /// The number at `path` as an `f64`, whatever its tag.
    pub fn number(&mut self, path: &[&str]) -> Option<f64> {
        match self.at_mut(path)? {
            #[allow(clippy::cast_precision_loss, reason = "the generated ints are small")]
            Self::Int(int) => Some(*int as f64),
            Self::Real(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }
}

/// What a number read as an `f32` and written back reads as: the nearest `f32` (the largest
/// finite one of its sign when the number is beyond `f32`), as the shortest text that reads
/// back to that `f32`, a real. The second value says whether the number was beyond `f32`.
pub fn as_f32_written(number: f64) -> (f64, bool) {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    let nearest = number as f32;
    let (float, clamped) = if nearest.is_infinite() {
        (if number < 0.0 { f32::MIN } else { f32::MAX }, true)
    } else {
        (nearest, false)
    };
    (format!("{float:?}").parse().unwrap(), clamped)
}

/// An integer token beyond 64 bits, generated only where a string is declared: a number
/// there, and the file is refused.
pub const WIDE_FOR_A_STRING: &str = "88888888888888888888888";

/// Slot names of either case, with every character the pattern allows.
const SLOT_KEYS: [&str; 5] = ["default", "Default", "a.b-c_D9", "x1", "walk"];

/// A map of strings: absent, empty, or one to three entries with slot names of either case;
/// rarely a key outside the pattern or a value that is no string.
pub fn string_map(rng: &mut Rng, absent: usize, values: &[&str]) -> Option<J> {
    if rng.chance(absent) {
        return None;
    }
    if rng.chance(10) {
        return Some(J::Obj(Vec::new()));
    }
    let mut members: Vec<(String, J)> = Vec::new();
    for _ in 0..=rng.below(3) {
        let key = if rng.chance(1) {
            "my:slot"
        } else {
            rng.pick(&SLOT_KEYS)
        };
        if members.iter().any(|(k, _)| k == key) {
            continue;
        }
        let value = if rng.chance(2) {
            J::Num(rng.pick(&["3", WIDE_FOR_A_STRING]))
        } else {
            J::str(rng.pick(values))
        };
        members.push((key.to_owned(), value));
    }
    Some(J::Obj(members))
}
