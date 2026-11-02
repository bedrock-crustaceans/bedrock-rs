//! The scatter parameters of a feature rule's `distribution`: how many positions, with what
//! chance, and where around the input position.

use facet_value::{VArray, VObject};

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::EncodeError;
use bedrock_addon_core::json::{JsonType, StrictError, StrictInt, Value};
use bedrock_addon_core::molang::{
    Admission, EmptyRule, Molang, MolangFailure, MolangFieldSpec, MolangFields,
    MolangVersionSource, MolangVisitor, OpsRule, RawVersion, ReadField, ReaderKind, Step,
};

/// `distribution`: the positions a rule scatters its feature to.
///
/// JSON: `{"iterations", "scatter_chance"?, "coordinate_eval_order"?, "x", "y", "z"}`. Each
/// iteration evaluates the three coordinates, in [`eval_order`](Self::eval_order), as offsets
/// from the input position. The same type is the `distribution` of a scatter feature
/// (`minecraft:scatter_feature`).
///
/// The three coordinates are required in a document. One is `None` only in a scatter feature
/// read from a format below 1.21.10, whose root layout may leave a coordinate out (vanilla logs
/// "… must be provided, either as a Molang expression string or as a sub-object").
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct ScatterParams {
    /// How many positions to generate (JSON `iterations`): a number or a Molang expression.
    pub iterations: Molang,
    /// The chance that the scatter happens at all, not per iteration (JSON `scatter_chance`);
    /// absent is always.
    #[facet(default)]
    pub scatter_chance: Option<ScatterChance>,
    /// The order the coordinates are evaluated in (JSON `coordinate_eval_order`); absent is
    /// `xzy`.
    #[facet(default)]
    pub coordinate_eval_order: Option<CoordinateEvalOrder>,
    /// The x offset (JSON `x`).
    #[facet(default)]
    pub x: Option<CoordinateRange>,
    /// The y offset (JSON `y`).
    #[facet(default)]
    pub y: Option<CoordinateRange>,
    /// The z offset (JSON `z`).
    #[facet(default)]
    pub z: Option<CoordinateRange>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `scatter_chance`: a fraction, or a percentage.
///
/// JSON: `{"numerator", "denominator"}`, or a number or Molang expression giving a percentage in
/// `(0, 100]`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum ScatterChance {
    /// `numerator / denominator`.
    Fraction(ScatterFraction),
    /// A percentage.
    Percent(Molang),
}

/// The object form of `scatter_chance`: the chance is `numerator / denominator`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct ScatterFraction {
    /// JSON `numerator`, at least 1.
    pub numerator: StrictInt<i32>,
    /// JSON `denominator`, at least 1; vanilla uses 1 when it is not above the numerator.
    pub denominator: StrictInt<i32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One coordinate of the scatter parameters: a value drawn from a distribution, or an
/// expression.
///
/// JSON: `{"distribution", "extent", "step_size"?, "grid_offset"?}`, or a number or Molang
/// expression (an `{"expression", "version"}` object included).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum CoordinateRange {
    /// A value drawn from a distribution over an extent.
    Object(CoordinateDistribution),
    /// An expression evaluated per position.
    Single(Molang),
}

/// The object form of a coordinate: a distribution over an extent.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct CoordinateDistribution {
    /// The distribution (JSON `distribution`).
    pub distribution: RandomDistribution,
    /// The lower and upper bound, inclusive, as offsets from the input position (JSON
    /// `extent`).
    pub extent: [Molang; 2],
    /// The distance between grid steps, at least 1; only for the grid distributions (JSON
    /// `step_size`); absent is 1.
    #[facet(default)]
    pub step_size: Option<StrictInt<i32>>,
    /// The grid offset, at least 0; only for the grid distributions (JSON `grid_offset`); absent
    /// is 0.
    #[facet(default)]
    pub grid_offset: Option<StrictInt<i32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `distribution` of a coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum RandomDistribution {
    /// `uniform`.
    Uniform = 1,
    /// `gaussian`.
    Gaussian = 2,
    /// `inverse_gaussian`.
    InverseGaussian = 3,
    /// `triangle`.
    Triangle = 6,
    /// `fixed_grid`.
    FixedGrid = 4,
    /// `jittered_grid`.
    JitteredGrid = 5,
}

impl RandomDistribution {
    /// The JSON names, in vanilla's order.
    const NAMES: [(&'static str, RandomDistribution); 6] = [
        ("uniform", Self::Uniform),
        ("gaussian", Self::Gaussian),
        ("inverse_gaussian", Self::InverseGaussian),
        ("triangle", Self::Triangle),
        ("fixed_grid", Self::FixedGrid),
        ("jittered_grid", Self::JitteredGrid),
    ];

    /// The JSON name.
    pub fn name(self) -> &'static str {
        Self::NAMES
            .iter()
            .find(|(_, d)| *d == self)
            .map_or("uniform", |(name, _)| name)
    }

    /// The distribution of the JSON name `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::NAMES
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, d)| *d)
    }

    /// `fixed_grid` or `jittered_grid`: the distributions that read `step_size` and
    /// `grid_offset`.
    pub fn is_grid(self) -> bool {
        matches!(self, Self::FixedGrid | Self::JitteredGrid)
    }
}

/// `coordinate_eval_order`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum CoordinateEvalOrder {
    /// `xyz`.
    Xyz = 0,
    /// `xzy`, the default.
    #[default]
    Xzy = 1,
    /// `yxz`.
    Yxz = 2,
    /// `yzx`.
    Yzx = 3,
    /// `zxy`.
    Zxy = 4,
    /// `zyx`.
    Zyx = 5,
}

impl ScatterParams {
    /// The coordinate order: `coordinate_eval_order`, `xzy` when absent.
    pub fn eval_order(&self) -> CoordinateEvalOrder {
        self.coordinate_eval_order.unwrap_or_default()
    }

    /// The iterations vanilla uses: a constant below 0 is replaced by 1 (vanilla logs "Bad
    /// value for iterations - should be >= 0").
    pub fn effective_iterations(&self) -> Molang {
        match self.iterations {
            Molang::Const(c) if c < 0.0 => Molang::Const(1.0),
            ref iterations => iterations.clone(),
        }
    }

    /// The chance vanilla uses: 100 % when absent or when a constant percentage is outside
    /// `(0, 100]`; a fraction whose denominator is not above its numerator gets denominator 1.
    pub fn effective_chance(&self) -> ScatterChance {
        let always = ScatterChance::Percent(Molang::Const(100.0));
        match &self.scatter_chance {
            None => always,
            Some(ScatterChance::Percent(Molang::Const(c))) if !percent_in_range(*c) => always,
            Some(ScatterChance::Fraction(fraction))
                if fraction.denominator.0 <= fraction.numerator.0 =>
            {
                ScatterChance::Fraction(ScatterFraction {
                    denominator: StrictInt(1),
                    ..fraction.clone()
                })
            }
            Some(chance) => chance.clone(),
        }
    }
}

/// Vanilla's percentage check: `value <= 0 || value > 100` fails, so 100 itself passes.
pub fn percent_in_range(value: f32) -> bool {
    value > 0.0 && value <= 100.0
}

impl CoordinateDistribution {
    /// The grid step vanilla uses: `step_size` for a grid distribution, else (or when absent) 1.
    pub fn effective_step_size(&self) -> i32 {
        match self.step_size {
            Some(step) if self.distribution.is_grid() => step.0,
            _ => 1,
        }
    }

    /// The grid offset vanilla uses: `grid_offset` for a grid distribution, else (or when
    /// absent) 0.
    pub fn effective_grid_offset(&self) -> i32 {
        match self.grid_offset {
            Some(offset) if self.distribution.is_grid() => offset.0,
            _ => 0,
        }
    }

    /// Whether both bounds are constants and, rounded to integers, the upper one is not above
    /// the lower one (vanilla logs "Bad value for extent - maximum must be greater than
    /// minimum" and keeps them).
    pub fn extent_is_empty(&self) -> bool {
        match &self.extent {
            [Molang::Const(min), Molang::Const(max)] => max.round() <= min.round(),
            _ => false,
        }
    }
}

// ---- the typed (facet) form of the two choice members -------------------------------------------
//
// `ScatterParams` as a member of a typed struct (`minecraft:scatter_feature`) reads its choice
// members through these conversions: the struct form first, then the expression, and a struct
// form with a member it does not take is refused. Feature rules
// read the same members from their own slices (`feature_rules/parse.rs`) and keep unknown
// members there.

/// The rejection of a value of none of a member's shapes.
fn mismatch(expected: &'static str) -> StrictError {
    StrictError::WrongType {
        expected,
        found: JsonType::Null,
    }
}

/// A Molang member read leniently; pass 1 checks it against the member's own reader and gives a
/// string its version. A number beyond `f32` is read as the largest finite float of its sign
/// (`molang::read`); the conversion has no diagnostics, so the family that reads through it
/// reports the number from the text (`feature::scatter::beyond_f32_findings`).
fn molang(value: &Value) -> Result<Molang, StrictError> {
    match bedrock_addon_core::molang::read(value, ReaderKind::LenientVersioned, RawVersion(-1)) {
        Ok(ReadField::Value(value)) => Ok(value),
        _ => Err(mismatch("a number, a bool or a Molang expression")),
    }
}

/// A Molang member's JSON.
fn molang_value(value: &Molang) -> Result<Value, StrictError> {
    bedrock_addon_core::molang::write(value).map_err(|_| StrictError::NonFinite)
}

/// An integer member of a cereal object: an integer token.
fn cereal_int(value: &Value) -> Result<StrictInt<i32>, StrictError> {
    value
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_i64)
        .and_then(|i| i32::try_from(i).ok())
        .map(StrictInt)
        .ok_or_else(|| mismatch("an integer"))
}

/// The object form `{"expression", "version"}` of a Molang member.
fn is_expression_object(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.contains_key("expression") && o.contains_key("version"))
}

impl TryFrom<Value> for ScatterChance {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let Some(object) = value.as_object().filter(|_| !is_expression_object(&value)) else {
            return molang(&value).map(ScatterChance::Percent);
        };
        let expected = "{\"numerator\", \"denominator\"}";
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "numerator" | "denominator"))
        {
            return Err(mismatch(expected));
        }
        let member = |key| {
            object
                .get(key)
                .ok_or_else(|| mismatch(expected))
                .and_then(cereal_int)
        };
        Ok(ScatterChance::Fraction(ScatterFraction {
            numerator: member("numerator")?,
            denominator: member("denominator")?,
            extra: Extra::default(),
        }))
    }
}

impl TryFrom<&ScatterChance> for Value {
    type Error = StrictError;

    fn try_from(chance: &ScatterChance) -> Result<Self, StrictError> {
        match chance {
            ScatterChance::Fraction(fraction) => {
                let mut object = VObject::new();
                object.insert("numerator", Value::from(i64::from(fraction.numerator.0)));
                object.insert(
                    "denominator",
                    Value::from(i64::from(fraction.denominator.0)),
                );
                Ok(Value::from(object))
            }
            ScatterChance::Percent(value) => molang_value(value),
        }
    }
}

impl TryFrom<Value> for CoordinateRange {
    type Error = StrictError;

    fn try_from(value: Value) -> Result<Self, StrictError> {
        let Some(object) = value.as_object().filter(|_| !is_expression_object(&value)) else {
            return molang(&value).map(CoordinateRange::Single);
        };
        let expected = "a coordinate object";
        if object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "distribution" | "extent" | "step_size" | "grid_offset"
            )
        }) {
            return Err(mismatch(expected));
        }
        let name = object
            .get("distribution")
            .and_then(Value::as_string)
            .ok_or_else(|| mismatch(expected))?;
        let distribution = RandomDistribution::from_name(name.as_str())
            .ok_or_else(|| mismatch("a distribution name"))?;
        let extent = match object
            .get("extent")
            .and_then(Value::as_array)
            .map(facet_value::VArray::as_slice)
        {
            Some([min, max]) => [molang(min)?, molang(max)?],
            _ => return Err(mismatch("a two-element extent")),
        };
        let optional = |key| object.get(key).map(cereal_int).transpose();
        Ok(CoordinateRange::Object(CoordinateDistribution {
            distribution,
            extent,
            step_size: optional("step_size")?,
            grid_offset: optional("grid_offset")?,
            extra: Extra::default(),
        }))
    }
}

impl TryFrom<&CoordinateRange> for Value {
    type Error = StrictError;

    fn try_from(coordinate: &CoordinateRange) -> Result<Self, StrictError> {
        match coordinate {
            CoordinateRange::Single(value) => molang_value(value),
            CoordinateRange::Object(range) => {
                let mut object = VObject::new();
                object.insert("distribution", Value::from(range.distribution.name()));
                let extent = range
                    .extent
                    .iter()
                    .map(molang_value)
                    .collect::<Result<VArray, _>>()?;
                object.insert("extent", Value::from(extent));
                if let Some(step) = range.step_size {
                    object.insert("step_size", Value::from(i64::from(step.0)));
                }
                if let Some(offset) = range.grid_offset {
                    object.insert("grid_offset", Value::from(i64::from(offset.0)));
                }
                Ok(Value::from(object))
            }
        }
    }
}

/// Every Molang member of a scatter distribution: a number, bool, string or
/// `{expression, version}`, read leniently at the pack's version, with the `world_gen` query set
/// only. A rejected expression is kept as an error and the rule is still inserted.
// The parser's error lines are logged at level 3 and the rule is still inserted, so a
// rejected expression is a kept error: the game converts each member, logs the failure and
// goes on. An empty text is no expression. The object form is compiled at the pack's version
// too; its own version is kept for writing. The read is lenient: members the game does not
// know are not rejected, and an empty expression is cleared.
pub const SCATTER: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::LenientVersioned,
    version: MolangVersionSource::PackIgnoringObject,
    admission: Admission::WorldGen,
    ops: OpsRule::All,
    empty: EmptyRule::NoExpression,
    on_failure: MolangFailure::KeepLogged,
    error_severity: Severity::Error,
    error_kept: true,
};

impl MolangFields for ScatterParams {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("iterations", &SCATTER, &mut self.iterations);
        if let Some(ScatterChance::Percent(chance)) = &mut self.scatter_chance {
            visitor.field("scatter_chance", &SCATTER, chance);
        }
        for (key, coordinate) in [("x", &mut self.x), ("y", &mut self.y), ("z", &mut self.z)] {
            match coordinate {
                None => {}
                Some(CoordinateRange::Single(value)) => visitor.field(key, &SCATTER, value),
                Some(CoordinateRange::Object(range)) => {
                    visitor.enter(Step::Key(key));
                    visitor.enter(Step::Key("extent"));
                    for (index, bound) in range.extent.iter_mut().enumerate() {
                        visitor.field(index, &SCATTER, bound);
                    }
                    visitor.leave();
                    visitor.leave();
                }
            }
        }
    }
}

impl ScatterParams {
    /// `distribution` as JSON: the members as written, unknown ones last.
    pub fn write(&self) -> Result<Value, bedrock_addon_core::json::EncodeError> {
        let mut object = VObject::new();
        object.insert("iterations", scatter_molang(&self.iterations)?);
        match &self.scatter_chance {
            Some(ScatterChance::Fraction(fraction)) => {
                object.insert(
                    "scatter_chance",
                    bedrock_addon_core::versioning::write_payload(fraction)?,
                );
            }
            Some(ScatterChance::Percent(percent)) => {
                object.insert("scatter_chance", scatter_molang(percent)?);
            }
            None => {}
        }
        if let Some(order) = self.coordinate_eval_order {
            object.insert(
                "coordinate_eval_order",
                bedrock_addon_core::versioning::write_payload(&order)?,
            );
        }
        for (key, coordinate) in [("x", &self.x), ("y", &self.y), ("z", &self.z)] {
            // An absent coordinate (a value built by hand, or taken from an old scatter
            // feature) is not written: the document then fails vanilla's required-member check.
            let value = match coordinate {
                Some(CoordinateRange::Object(range)) => {
                    bedrock_addon_core::versioning::write_payload(range)?
                }
                Some(CoordinateRange::Single(value)) => scatter_molang(value)?,
                None => continue,
            };
            object.insert(key, value);
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(object.into())
    }
}

/// A Molang member in the form it was read.
fn scatter_molang(value: &Molang) -> Result<Value, EncodeError> {
    bedrock_addon_core::molang::write(value).map_err(|what| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what,
    })
}
