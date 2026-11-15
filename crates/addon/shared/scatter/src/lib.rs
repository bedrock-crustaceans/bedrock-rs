//! The scatter parameters the `feature_rules` and `feature` content families share: a feature
//! rule's `distribution`, which is also the `distribution` of a scatter feature
//! (`minecraft:scatter_feature`): how many positions, with what chance, and where around the
//! input position. A family composes these; the crate depends on `bedrock_addon_core` only.

use facet_value::{VArray, VObject};

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::json::EncodeError;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{JsonType, StrictError, Value};
use bedrock_addon_core::molang::{
    Admission, EmptyRule, Molang, MolangFailure, MolangFieldSpec, MolangFields, MolangSlot,
    MolangVersionSource, MolangVisitor, OpsRule, RawVersion, ReadField, ReaderKind,
    beyond_f32_warning,
};
use bedrock_addon_core::schema::{self, FromJson, Schema, SchemaCx, SchemaFields};
use bedrock_addon_core::versioning::Choices;

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
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct ScatterParams {
    /// How many positions to generate (JSON `iterations`): a number or a Molang expression.
    #[facet(schema::molang = SCATTER)]
    pub iterations: Molang,
    /// The chance that the scatter happens at all, not per iteration (JSON `scatter_chance`);
    /// absent is always.
    #[facet(schema::molang_with = walk_chance)]
    pub scatter_chance: Option<ScatterChance>,
    /// The order the coordinates are evaluated in (JSON `coordinate_eval_order`); absent is
    /// `xzy`.
    pub coordinate_eval_order: Option<CoordinateEvalOrder>,
    /// The x offset (JSON `x`).
    #[facet(schema::molang_with = walk_coordinate)]
    pub x: Option<CoordinateRange>,
    /// The y offset (JSON `y`).
    #[facet(schema::molang_with = walk_coordinate)]
    pub y: Option<CoordinateRange>,
    /// The z offset (JSON `z`).
    #[facet(schema::molang_with = walk_coordinate)]
    pub z: Option<CoordinateRange>,
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

/// The object form of `scatter_chance`: the chance is `numerator / denominator`. Its unknown
/// members are kept where it is read typed under a reading that keeps them (a feature rule's
/// `distribution`).
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct ScatterFraction {
    /// JSON `numerator`, at least 1.
    #[facet(schema::min = 1)]
    pub numerator: i32,
    /// JSON `denominator`, at least 1; vanilla uses 1 when it is not above the numerator.
    #[facet(schema::min = 1)]
    pub denominator: i32,
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

/// The object form of a coordinate: a distribution over an extent. Its unknown members are
/// kept where it is read typed under a reading that keeps them (a feature rule's
/// `distribution`).
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct CoordinateDistribution {
    /// The distribution (JSON `distribution`).
    pub distribution: RandomDistribution,
    /// The lower and upper bound, inclusive, as offsets from the input position (JSON
    /// `extent`).
    #[facet(schema::molang_with = walk_extent)]
    pub extent: [Molang; 2],
    /// The distance between grid steps, at least 1; only for the grid distributions (JSON
    /// `step_size`); absent is 1.
    #[facet(schema::min = 1)]
    pub step_size: Option<i32>,
    /// The grid offset, at least 0; only for the grid distributions (JSON `grid_offset`); absent
    /// is 0.
    #[facet(schema::min = 0)]
    pub grid_offset: Option<i32>,
}

/// `distribution` of a coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
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
    /// `fixed_grid` or `jittered_grid`: the distributions that read `step_size` and
    /// `grid_offset`.
    pub fn is_grid(self) -> bool {
        matches!(self, Self::FixedGrid | Self::JitteredGrid)
    }
}

/// `coordinate_eval_order`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet, Schema)]
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
                if fraction.denominator <= fraction.numerator =>
            {
                ScatterChance::Fraction(ScatterFraction {
                    denominator: 1,
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
            Some(step) if self.distribution.is_grid() => step,
            _ => 1,
        }
    }

    /// The grid offset vanilla uses: `grid_offset` for a grid distribution, else (or when
    /// absent) 0.
    pub fn effective_grid_offset(&self) -> i32 {
        match self.grid_offset {
            Some(offset) if self.distribution.is_grid() => offset,
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

// ---- the reader's form of the two choice members -----------------------------------------------
//
// The struct form first, then the expression, as the typed form below: an object that is not
// `{"expression", "version"}` is the struct form, whose own findings stand.

/// A fraction's parts are checked; a percentage is no bound.
impl SchemaFields for ScatterChance {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        if let Self::Fraction(fraction) = self {
            fraction.check(cx);
        }
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

/// A distribution's grid members are checked; an expression is no bound.
impl SchemaFields for CoordinateRange {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        if let Self::Object(range) = self {
            range.check(cx);
        }
    }

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

impl FromJson for ScatterChance {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if converts(read) {
            return converted(node, read, ScatterChance::try_from);
        }
        if is_struct_form(node.value()) {
            ScatterFraction::from_json(node, read).map(Self::Fraction)
        } else {
            Molang::from_json(node, read).map(Self::Percent)
        }
    }
}

impl FromJson for CoordinateRange {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if converts(read) {
            return converted(node, read, CoordinateRange::try_from);
        }
        if is_struct_form(node.value()) {
            CoordinateDistribution::from_json(node, read).map(Self::Object)
        } else {
            Molang::from_json(node, read).map(Self::Single)
        }
    }
}

/// Whether the reading takes the two choice members as the game's conversion of the whole
/// member does (the typed form below): a struct form with a member it does not take is no form
/// of the member, and a number beyond `f32` is clamped and told, as the scatter feature reads
/// them ([`Choices::Whole`]). A reading that takes them by member (a feature rule's) reads the
/// struct form member by member, a number beyond its width refused where it stands.
fn converts(read: &Read<'_>) -> bool {
    read.reading().choices == Choices::Whole
}

/// The member `node` read by `convert`, the conversion of the whole member (a refusal is a wrong
/// type at the member, whatever its reason), and each number of it beyond `f32` told where it
/// stands: read as the largest finite float of its sign, the crate's own information at the
/// number on its token.
fn converted<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    convert: impl FnOnce(Value) -> Result<T, StrictError>,
) -> Option<T> {
    let value = schema::from_value(node, read, convert)?;
    let beyond = |value: &Value| {
        value
            .as_number()
            .map(facet_value::VNumber::to_f64_lossy)
            .is_some_and(|number| bedrock_addon_core::molang::beyond_f32(number).is_some())
    };
    if beyond(node.value()) {
        read.push(beyond_f32_warning(node.path().clone(), node.span()));
    } else if node.value().is_object()
        && let Some(members) = read.object(node)
        && let Some(extent) = members.get("extent")
        && let Some(bounds) = read.array(&extent)
    {
        for bound in bounds.iter() {
            if beyond(bound.value()) {
                read.push(beyond_f32_warning(bound.path().clone(), bound.span()));
            }
        }
    }
    Some(value)
}

/// Whether `value` is the struct form of a choice member: an object that is no Molang
/// `{"expression", "version"}` (vanilla tries the struct first, as the order of the cereal
/// options shows).
fn is_struct_form(value: &Value) -> bool {
    value.is_object() && !is_expression_object(value)
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
fn cereal_int(value: &Value) -> Result<i32, StrictError> {
    value
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_i64)
        .and_then(|i| i32::try_from(i).ok())
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
        }))
    }
}

impl TryFrom<&ScatterChance> for Value {
    type Error = StrictError;

    fn try_from(chance: &ScatterChance) -> Result<Self, StrictError> {
        match chance {
            ScatterChance::Fraction(fraction) => {
                let mut object = VObject::new();
                object.insert("numerator", Value::from(i64::from(fraction.numerator)));
                object.insert("denominator", Value::from(i64::from(fraction.denominator)));
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
                    object.insert("step_size", Value::from(i64::from(step)));
                }
                if let Some(offset) = range.grid_offset {
                    object.insert("grid_offset", Value::from(i64::from(offset)));
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

/// The Molang of a `scatter_chance` that is a percentage, at the member.
fn walk_chance(chance: &mut Option<ScatterChance>, visitor: &mut MolangVisitor<'_>) {
    if let Some(ScatterChance::Percent(chance)) = chance {
        chance.visit_slot(&SCATTER, visitor);
    }
}

/// The Molang of a coordinate: the expression at the member, the extent's bounds below it.
fn walk_coordinate(coordinate: &mut Option<CoordinateRange>, visitor: &mut MolangVisitor<'_>) {
    match coordinate {
        None => {}
        Some(CoordinateRange::Single(value)) => value.visit_slot(&SCATTER, visitor),
        Some(CoordinateRange::Object(range)) => range.visit_molang(visitor),
    }
}

/// The bounds of an extent, each at its index.
fn walk_extent(extent: &mut [Molang; 2], visitor: &mut MolangVisitor<'_>) {
    for (index, bound) in extent.iter_mut().enumerate() {
        visitor.field(index, &SCATTER, bound);
    }
}

impl ScatterParams {
    /// `distribution` as JSON: the members as written, unknown ones last.
    ///
    /// # Errors
    ///
    /// `EncodeError::NotRepresentable` when a member holds a Molang value that cannot be written as
    /// JSON.
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
        // The unknown members are the document's, written from its table.
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
