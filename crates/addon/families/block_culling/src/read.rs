//! Reading a `minecraft:block_culling_rules` payload as vanilla's strict cereal slices do: every
//! failure fails the file, an unknown member included, and every failure of the payload is
//! reported, each with vanilla's breadcrumb and text.

// Both slices use the strict error mask 0xfffc.

use facet_json::RawJson;
use facet_value::VObject;
use indexmap::IndexMap;

use super::{Condition, Description, Facing, GeometryPart, Rule};
use bedrock_addon_core::diagnostic::cereal;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Span,
};
use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::json::{JsonSlice, Value};

/// The cereal name of [`Facing`], which vanilla's enum text prints.
const FACING: &str = "Facing";
/// The cereal name of [`Condition`].
const CONDITION: &str = "Rule Condition";

/// Which struct reads the payload.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Slice {
    /// `[1.20.60, 1.21.80)`: a rule is `geometry_part` and `direction`.
    V1_20_60,
    /// 1.21.80: a rule may also hold `condition` and `cull_against_full_and_opaque`.
    V1_21_80,
}

/// A payload as read.
pub(super) struct Payload {
    /// `description`.
    pub(super) description: Description,
    /// `rules`.
    pub(super) rules: Vec<Rule>,
}

/// The reader's state: where the payload is and the findings.
struct Reader<'a> {
    /// The payload's path in the file.
    base: &'a JsonPath,
    /// The findings; any of them fails the file.
    d: &'a mut Diagnostics,
    /// Whether anything failed.
    failed: bool,
}

/// Reads the payload `payload` (the value under `minecraft:block_culling_rules`), reported
/// below `base`. `None` when vanilla's parse fails; `d` then holds every reason, each with
/// the span of the value its path names.
pub(super) fn read(
    payload: JsonSlice<'_>,
    slice: Slice,
    base: &JsonPath,
    d: &mut Diagnostics,
) -> Option<Payload> {
    let before = d.len();
    let read = read_value(payload, slice, base, d);
    if d.len() > before {
        Spans::of(payload).fill(d, before, base);
    }
    read
}

/// The payload's text split once at its root and once at `rules`, so that the span of any
/// finding's path is found below its member, or below its rule, without reading the payload
/// again per finding: linear in the text for any number of findings (pinned in `cost.rs`).
struct Spans<'a> {
    payload: JsonSlice<'a>,
    /// The root's members; a key written twice keeps its last value, the one the reader
    /// takes.
    members: IndexMap<String, RawJson<'a>>,
    /// The elements of `rules`, when it is an array.
    rules: Vec<RawJson<'a>>,
}

impl<'a> Spans<'a> {
    fn of(payload: JsonSlice<'a>) -> Self {
        let members: IndexMap<String, RawJson<'a>> =
            bedrock_addon_core::json::parse_at(payload).unwrap_or_default();
        let rules = members
            .get("rules")
            .and_then(|rules| payload.nested(rules))
            .and_then(|rules| bedrock_addon_core::json::parse_at(rules).ok())
            .unwrap_or_default();
        Self {
            payload,
            members,
            rules,
        }
    }

    /// Gives every finding of `d` from `first` on that has no span the span of the value
    /// its path names below `base`; a path that names nothing in the text (a member that is
    /// absent) stays without one.
    fn fill(&self, d: &mut Diagnostics, first: usize, base: &JsonPath) {
        let below = base.segments().len();
        for (index, mut finding) in std::mem::take(d).into_iter().enumerate() {
            if index >= first && finding.location.span.is_none() {
                let path: Vec<JsonPathSegment<'_>> =
                    finding.location.json_path.segments().skip(below).collect();
                let span = self.at(&path);
                finding.location.span = span;
            }
            d.push(finding);
        }
    }

    fn at(&self, path: &[JsonPathSegment<'_>]) -> Option<Span> {
        let (member, rest) = match path {
            [] => (self.payload, path),
            [
                JsonPathSegment::Key("rules"),
                JsonPathSegment::Index(rule),
                rest @ ..,
            ] => (self.payload.nested(self.rules.get(*rule)?)?, rest),
            [JsonPathSegment::Key(key), rest @ ..] => {
                (self.payload.nested(self.members.get(*key)?)?, rest)
            }
            [JsonPathSegment::Index(_), ..] => return None,
        };
        let mut below = JsonPath::new();
        for segment in rest {
            match segment {
                JsonPathSegment::Key(key) => below.push(key),
                JsonPathSegment::Index(index) => below.push_index(*index),
            }
        }
        bedrock_addon_core::json::span_at(member, &below)
    }
}

fn read_value(
    payload: JsonSlice<'_>,
    slice: Slice,
    base: &JsonPath,
    d: &mut Diagnostics,
) -> Option<Payload> {
    let mut reader = Reader {
        base,
        d,
        failed: false,
    };
    let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(payload.text),
    ) else {
        reader.fail(
            DiagnosticCode::from(bedrock_addon_core::diagnostic::builders::syntax(
                "the payload is not JSON",
            )),
            &JsonPath::new(),
        );
        return None;
    };
    let root = JsonPath::new();
    let Some(object) = json.as_object() else {
        reader.wrong_type(&root);
        return None;
    };
    reader.unknown(object, &["description", "rules"], &root);
    let description = reader
        .required_object(object, "description", &root)
        .and_then(|(description, at)| {
            reader.unknown(description, &["identifier"], &at);
            let identifier = reader.required_string(description, "identifier", &at)?;
            Some(Description {
                identifier: CullingRulesId::parse(identifier),
            })
        });
    let rules = match object.get("rules") {
        None => {
            reader.missing(&root.child("rules"));
            None
        }
        Some(rules) => reader.rules(rules, slice, &root.child("rules")),
    };
    if reader.failed {
        return None;
    }
    Some(Payload {
        description: description?,
        rules: rules?,
    })
}

impl Reader<'_> {
    /// Raises an error at `at` (relative to the payload).
    fn fail(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.failed = true;
        let path = self.base.join(at);
        self.d.push(Diagnostic::error(code).at(path));
    }

    /// The game's generic finding `code` (`diagnostic::cereal`) at `at`, its text after the
    /// breadcrumb: `-> rules[0] -> direction: <text>`.
    fn cereal(&mut self, code: DiagnosticCode, at: &JsonPath) {
        let line = format!("{}: {code}", breadcrumb(at));
        // By legacy name: the variant and the definitions that were it.
        let code = match code.name() {
            "MissingRequiredField" => {
                DiagnosticCode::from(core_codes::MISSING_REQUIRED_FIELD_WORDED.with([line]))
            }
            "UnrecognizedField" => {
                DiagnosticCode::from(core_codes::UNRECOGNIZED_FIELD_WORDED.with([line]))
            }
            "InvalidType" => DiagnosticCode::from(core_codes::INVALID_TYPE_WORDED.with([line])),
            "InvalidEnumValue" => {
                DiagnosticCode::from(core_codes::INVALID_ENUM_VALUE_WORDED.with([line]))
            }
            "InvalidValue" => DiagnosticCode::from(core_codes::INVALID_VALUE_WORDED.with([line])),
            _ => code,
        };
        self.fail(code, at);
    }

    /// A value of a JSON type the member does not take, where vanilla's text is not known.
    fn wrong_type(&mut self, at: &JsonPath) {
        self.fail(DiagnosticCode::from(&core_codes::INVALID_TYPE), at);
    }

    /// "missing required field" at `at`.
    fn missing(&mut self, at: &JsonPath) {
        self.cereal(cereal::missing_required_field(), at);
    }

    /// Every member of `object` that is not one of `known`: an error each.
    fn unknown(&mut self, object: &VObject, known: &[&str], at: &JsonPath) {
        for (name, _) in object {
            if known.contains(&name.as_str()) {
                continue;
            }
            let at = at.child(name.as_str());
            self.cereal(cereal::member_not_in_schema(), &at);
        }
    }

    /// The required object member `name` of `object`, with its path.
    fn required_object<'v>(
        &mut self,
        object: &'v VObject,
        name: &str,
        at: &JsonPath,
    ) -> Option<(&'v VObject, JsonPath)> {
        let at = at.child(name);
        let Some(value) = object.get(name) else {
            self.missing(&at);
            return None;
        };
        let member = value.as_object();
        if member.is_none() {
            self.wrong_type(&at);
        }
        member.map(|member| (member, at))
    }

    /// The required string member `name` of `object`.
    fn required_string(&mut self, object: &VObject, name: &str, at: &JsonPath) -> Option<String> {
        let at = at.child(name);
        let Some(value) = object.get(name) else {
            self.missing(&at);
            return None;
        };
        let text = value.as_string();
        if text.is_none() {
            self.wrong_type(&at);
        }
        text.map(|text| text.as_str().to_owned())
    }

    /// A member bound to an enum: one of `names`, as written.
    fn named<T>(
        &mut self,
        value: &Value,
        bound: &str,
        names: &[&str],
        from_name: fn(&str) -> Option<T>,
        at: &JsonPath,
    ) -> Option<T> {
        let Some(found) = value.as_string() else {
            self.cereal(cereal::enum_expects_string(bound), at);
            return None;
        };
        let read = from_name(found.as_str());
        if read.is_none() {
            self.cereal(cereal::enum_expects(bound, names, found.as_str()), at);
        }
        read
    }

    /// A [`Facing`] member.
    fn facing(&mut self, value: &Value, at: &JsonPath) -> Option<Facing> {
        let names = Facing::ALL.map(Facing::name);
        self.named(value, FACING, &names, Facing::from_name, at)
    }

    /// `rules`: an array of rule objects.
    fn rules(&mut self, value: &Value, slice: Slice, at: &JsonPath) -> Option<Vec<Rule>> {
        let Some(array) = value.as_array() else {
            self.wrong_type(at);
            return None;
        };
        let rules: Vec<Option<Rule>> = array
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let at = at.index(index);
                self.rule(rule, slice, &at)
            })
            .collect();
        rules.into_iter().collect()
    }

    /// One rule.
    fn rule(&mut self, value: &Value, slice: Slice, at: &JsonPath) -> Option<Rule> {
        let Some(object) = value.as_object() else {
            self.wrong_type(at);
            return None;
        };
        let known: &[&str] = match slice {
            Slice::V1_20_60 => &["geometry_part", "direction"],
            Slice::V1_21_80 => &[
                "condition",
                "cull_against_full_and_opaque",
                "geometry_part",
                "direction",
            ],
        };
        self.unknown(object, known, at);
        let latest = slice == Slice::V1_21_80;
        let mut failed = false;
        let condition = match object.get("condition").filter(|_| latest) {
            None => None,
            Some(value) => {
                let names = Condition::ALL.map(Condition::name);
                let read = self.named(
                    value,
                    CONDITION,
                    &names,
                    Condition::from_name,
                    &at.child("condition"),
                );
                failed |= read.is_none();
                read
            }
        };
        let flag = match object
            .get("cull_against_full_and_opaque")
            .filter(|_| latest)
        {
            None => None,
            Some(value) => {
                let read = value.as_bool();
                if read.is_none() {
                    let at = at.child("cull_against_full_and_opaque");
                    self.cereal(cereal::invalid_numeric_value(), &at);
                    failed = true;
                }
                read
            }
        };
        let geometry_part = self
            .required_object(object, "geometry_part", at)
            .and_then(|(part, at)| self.geometry_part(part, &at));
        let direction = match object.get("direction") {
            None => {
                self.missing(&at.child("direction"));
                None
            }
            Some(value) => self.facing(value, &at.child("direction")),
        };
        if failed {
            return None;
        }
        Some(Rule {
            condition,
            cull_against_full_and_opaque: flag,
            geometry_part: geometry_part?,
            direction: direction?,
        })
    }

    /// `geometry_part`.
    fn geometry_part(&mut self, object: &VObject, at: &JsonPath) -> Option<GeometryPart> {
        self.unknown(object, &["bone", "cube", "face"], at);
        let bone = self.required_string(object, "bone", at);
        let mut failed = false;
        let cube = match object.get("cube") {
            None => None,
            Some(value) => {
                let read = self.cube(value, &at.child("cube"));
                failed |= read.is_none();
                read
            }
        };
        let face = match object.get("face") {
            None => None,
            Some(value) => {
                let read = self.facing(value, &at.child("face"));
                failed |= read.is_none();
                read
            }
        };
        if failed {
            return None;
        }
        Some(GeometryPart {
            bone: bone?,
            cube,
            face,
        })
    }

    /// `cube`: an integer of at least 0.
    // The minimum is inclusive.
    fn cube(&mut self, value: &Value, at: &JsonPath) -> Option<i32> {
        let Some(number) = value.as_number() else {
            self.cereal(cereal::invalid_numeric_value(), at);
            return None;
        };
        let wide = number.to_f64_lossy();
        if wide < 0.0 {
            self.cereal(cereal::NumericRange::at_least(0.0).code(), at);
            return None;
        }
        // A fraction or an index beyond `int` is not a cube index; vanilla's text for it is not
        // known.
        #[allow(clippy::cast_possible_truncation)] // whole and in range, checked here
        let index = wide as i32;
        if wide.fract() != 0.0 || wide > f64::from(i32::MAX) {
            self.fail(DiagnosticCode::from(&core_codes::INVALID_VALUE), at);
            return None;
        }
        Some(index)
    }
}

/// Vanilla's breadcrumb of a path below the payload: `-> rules[0] -> direction`.
pub(super) fn breadcrumb(at: &JsonPath) -> String {
    let mut out = String::new();
    for segment in at.segments() {
        match segment {
            JsonPathSegment::Key(key) => {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str("-> ");
                out.push_str(key);
            }
            JsonPathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    out
}
