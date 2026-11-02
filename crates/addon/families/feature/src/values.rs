//! Value types several feature types share: integers as the schema reads them, block lists,
//! chances and weighted entries.

use facet::Facet;
use facet_value::{VArray, VObject};

use bedrock_addon_core::json::{JsonType, ScalarShape, StrictError, Value};
use bedrock_addon_core::shared::{BlockDescriptor, FloatRange};

/// An integer member as vanilla's feature schema reads it (jsoncpp `isInt`): an integer token,
/// or a real token with an integral value (`2.0`, which vanilla's own geode feature writes).
/// The second field keeps the token, so writing reproduces it.
///
/// JSON: a number. A string, a boolean, `null` or a real with a fraction is the wrong type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct JsonInt(pub i32, pub IntToken);

/// How a [`JsonInt`] was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[repr(u8)]
pub enum IntToken {
    /// An integer token (`2`): the form a new value is written in.
    #[default]
    Integer,
    /// A real token with an integral value (`2.0`).
    Real,
}

impl JsonInt {
    /// `value`, written as an integer token.
    pub fn new(value: i32) -> Self {
        JsonInt(value, IntToken::Integer)
    }
}

impl TryFrom<ScalarShape> for JsonInt {
    type Error = StrictError;

    fn try_from(json: ScalarShape) -> Result<Self, StrictError> {
        match json {
            ScalarShape::UInt(u) => i32::try_from(u)
                .map(JsonInt::new)
                .map_err(|_| StrictError::OutOfRange),
            ScalarShape::Int(i) => i32::try_from(i)
                .map(JsonInt::new)
                .map_err(|_| StrictError::OutOfRange),
            // jsoncpp `isInt` holds for an integral real inside the `int` range.
            ScalarShape::Number(x) if x.fract() == 0.0 => integral(x)
                .map(|value| JsonInt(value, IntToken::Real))
                .ok_or(StrictError::OutOfRange),
            ScalarShape::Number(_) => Err(wrong("integer", JsonType::Real)),
            ScalarShape::String(_) => Err(wrong("integer", JsonType::String)),
            ScalarShape::Bool(_) => Err(wrong("integer", JsonType::Bool)),
            ScalarShape::Null => Err(wrong("integer", JsonType::Null)),
        }
    }
}

impl From<&JsonInt> for ScalarShape {
    fn from(value: &JsonInt) -> Self {
        if value.1 == IntToken::Real {
            return ScalarShape::Number(f64::from(value.0));
        }
        match u64::try_from(value.0) {
            Ok(u) => ScalarShape::UInt(u),
            Err(_) => ScalarShape::Int(i64::from(value.0)),
        }
    }
}

impl From<i32> for JsonInt {
    fn from(value: i32) -> Self {
        JsonInt::new(value)
    }
}

/// `x` as an `i32` when it is integral and in range.
#[allow(clippy::cast_possible_truncation)] // checked against the bounds first
fn integral(x: f64) -> Option<i32> {
    (x >= f64::from(i32::MIN) && x <= f64::from(i32::MAX)).then_some(x as i32)
}

/// The rejection of a value of the wrong JSON type.
fn wrong(expected: &'static str, found: JsonType) -> StrictError {
    StrictError::WrongType { expected, found }
}

/// The JSON type of `value`, as the rejection names it.
fn json_type(value: &Value) -> JsonType {
    if value.is_null() {
        JsonType::Null
    } else if value.as_bool().is_some() {
        JsonType::Bool
    } else if value.as_string().is_some() {
        JsonType::String
    } else if value
        .as_number()
        .is_some_and(facet_value::VNumber::is_integer)
    {
        JsonType::Integer
    } else {
        // A real, or an array or object (which `JsonType` has no word for).
        JsonType::Real
    }
}

/// An integer read from `value` with [`JsonInt`]'s rules.
fn int_of(value: &Value) -> Option<i32> {
    let number = value.as_number()?;
    if let Some(i) = number.to_i64() {
        return i32::try_from(i).ok();
    }
    let x = number.to_f64_lossy();
    (x.fract() == 0.0).then(|| integral(x)).flatten()
}

/// A float read from a number of either tag.
#[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
fn float_of(value: &Value) -> Option<f32> {
    let x = value.as_number()?.to_f64_lossy() as f32;
    x.is_finite().then_some(x)
}

/// The JSON number of a float, written as the crate writes `f32` (shortest form that reads back).
fn float_value(x: f32) -> Value {
    // The text form round-trips the `f32` exactly; `f64::from` would widen it.
    let text = x.to_string();
    text.parse::<f64>()
        .map_or_else(|_| Value::from(f64::from(x)), Value::from)
}

/// One block descriptor or an array of them: vanilla registers both schema options under one
/// member name (`may_attach_to.top`, `base_block`).
///
/// JSON: a descriptor, or an array of descriptors.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum BlockList {
    /// One descriptor.
    One(BlockDescriptor),
    /// An array of descriptors (possibly empty).
    Many(Vec<BlockDescriptor>),
}

impl BlockList {
    /// The descriptors, one or many.
    pub fn descriptors(&self) -> &[BlockDescriptor] {
        match self {
            Self::One(one) => std::slice::from_ref(one),
            Self::Many(many) => many,
        }
    }

    /// The descriptors, mutably.
    pub fn descriptors_mut(&mut self) -> &mut [BlockDescriptor] {
        match self {
            Self::One(one) => std::slice::from_mut(one),
            Self::Many(many) => many,
        }
    }
}

impl From<Value> for BlockList {
    fn from(value: Value) -> Self {
        match value.as_array() {
            Some(array) => Self::Many(array.iter().cloned().map(BlockDescriptor::from).collect()),
            None => Self::One(BlockDescriptor::from(value)),
        }
    }
}

impl From<&BlockList> for Value {
    fn from(list: &BlockList) -> Self {
        match list {
            BlockList::One(one) => Value::from(one),
            BlockList::Many(many) => Value::from(many.iter().map(Value::from).collect::<VArray>()),
        }
    }
}

/// A chance: a ratio `numerator / denominator`, or a probability.
///
/// JSON: `{"numerator": int, "denominator": int}` (`numerator ≥ 0`, `denominator ≥ 1`), or a
/// number, which vanilla's own tree features write (`"decoration_chance": 0.25`).
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum Chance {
    /// `{"numerator", "denominator"}`.
    Ratio {
        /// JSON `numerator` (≥ 0).
        numerator: i32,
        /// JSON `denominator` (≥ 1).
        denominator: i32,
    },
    /// A number: the probability itself.
    Probability(f32),
}

impl TryFrom<Value> for Chance {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        if value.is_number() {
            return float_of(&value)
                .map(Chance::Probability)
                .ok_or(StrictError::OutOfRange);
        }
        let expected = "a chance object or a number";
        let object = value
            .as_object()
            .ok_or_else(|| wrong(expected, json_type(&value)))?;
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "numerator" | "denominator"))
        {
            return Err(wrong(expected, JsonType::Null));
        }
        let member = |key: &str| object.get(key).and_then(int_of);
        match (member("numerator"), member("denominator")) {
            (Some(numerator), Some(denominator)) => Ok(Chance::Ratio {
                numerator,
                denominator,
            }),
            _ => Err(wrong(expected, JsonType::Null)),
        }
    }
}

impl From<&Chance> for Value {
    fn from(chance: &Chance) -> Self {
        match *chance {
            Chance::Ratio {
                numerator,
                denominator,
            } => {
                let mut object = VObject::new();
                object.insert("numerator", Value::from(i64::from(numerator)));
                object.insert("denominator", Value::from(i64::from(denominator)));
                Value::from(object)
            }
            Chance::Probability(p) => float_value(p),
        }
    }
}

/// One chance or an array of them (the simple canopy's `variation_chance`).
///
/// JSON: a [`Chance`], or an array of them.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum ChanceList {
    /// One chance.
    One(Chance),
    /// An array of chances.
    Many(Vec<Chance>),
}

impl ChanceList {
    /// The chances, one or many.
    pub fn chances(&self) -> &[Chance] {
        match self {
            Self::One(one) => std::slice::from_ref(one),
            Self::Many(many) => many,
        }
    }
}

impl TryFrom<Value> for ChanceList {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        match value.as_array() {
            Some(array) => array
                .iter()
                .cloned()
                .map(Chance::try_from)
                .collect::<Result<_, _>>()
                .map(ChanceList::Many),
            None => Chance::try_from(value).map(ChanceList::One),
        }
    }
}

impl From<&ChanceList> for Value {
    fn from(list: &ChanceList) -> Self {
        match list {
            ChanceList::One(one) => Value::from(one),
            ChanceList::Many(many) => Value::from(many.iter().map(Value::from).collect::<VArray>()),
        }
    }
}

/// A block with a weight: `[descriptor, weight]` (vanilla's weighted block lists).
///
/// JSON: a two-element array, a block descriptor and a number.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct WeightedBlock {
    /// Position 0: the block.
    pub block: BlockDescriptor,
    /// Position 1: its weight.
    pub weight: f32,
}

impl TryFrom<Value> for WeightedBlock {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let (first, second) = pair(&value, "a [block, weight] array")?;
        let weight = float_of(second).ok_or_else(|| wrong("a number", json_type(second)))?;
        Ok(WeightedBlock {
            block: BlockDescriptor::from(first.clone()),
            weight,
        })
    }
}

impl From<&WeightedBlock> for Value {
    fn from(entry: &WeightedBlock) -> Self {
        let mut array = VArray::new();
        array.push(Value::from(&entry.block));
        array.push(float_value(entry.weight));
        Value::from(array)
    }
}

/// The two elements of a two-element array.
pub(super) fn pair<'v>(
    value: &'v Value,
    expected: &'static str,
) -> Result<(&'v Value, &'v Value), StrictError> {
    match value.as_array().map(VArray::as_slice) {
        Some([first, second]) => Ok((first, second)),
        _ => Err(wrong(expected, json_type(value))),
    }
}

/// A float read for a weight.
pub(super) fn weight_of(value: &Value) -> Result<f32, StrictError> {
    float_of(value).ok_or_else(|| wrong("a number", json_type(value)))
}

/// The JSON number of a weight.
pub(super) fn weight_value(x: f32) -> Value {
    float_value(x)
}

/// A float range member that vanilla may leave unparsed: an array of
/// another length than two is logged ("Float range array was not parsed. Float range arrays
/// should have 2 values.") and the member keeps its default.
///
/// JSON: a number, `[min, max]` or `{"range_min", "range_max"}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum CarverRange {
    /// A range vanilla reads.
    Range(FloatRange),
    /// An array of another length, kept as written; vanilla leaves the member at its default.
    Unparsed(Value),
}

impl TryFrom<Value> for CarverRange {
    type Error = bedrock_addon_core::shared::ShapeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.as_array().is_some_and(|array| array.len() != 2) {
            return Ok(CarverRange::Unparsed(value));
        }
        FloatRange::try_from(value).map(CarverRange::Range)
    }
}

impl TryFrom<&CarverRange> for Value {
    type Error = bedrock_addon_core::shared::ShapeError;

    fn try_from(range: &CarverRange) -> Result<Self, Self::Error> {
        match range {
            CarverRange::Range(range) => Value::try_from(range),
            CarverRange::Unparsed(value) => Ok(value.clone()),
        }
    }
}
