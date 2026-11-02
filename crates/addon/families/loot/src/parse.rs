//! The loot table parser: vanilla's hand-written walk over the JSON value (the table, pool,
//! entry, function and condition readers).
//!
//! Vanilla reads each member with jsoncpp's `asInt` / `asFloat` / `asBool` / `asString` and its
//! default, and checks almost nothing: the walk keeps every member it reads as written
//! (`Option`), applies vanilla's drop and keep rules where vanilla logs, and keeps every member
//! it does not read in the level's `extra`.

use facet_value::{VObject, VString};

use super::condition::{ConditionKind, LootCondition, ToolEnchantment};
use super::function::{
    BannerPattern, BannerType, EnchantOption, FunctionKind, ItemColor, ItemData, LootFunction,
    SpecificEnchant, StewEffect,
};
use super::texts;
use super::{Entry, EntryKind, LootTable, Pool, RandomValueBounds, Tiers};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{EntityId, ItemId, LootTablePath};
use bedrock_addon_core::json::{
    Indent, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom, write_json,
};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::shared::{IntRange, RangeForm};
use bedrock_addon_core::version::FormatVersion;

/// Parses one file: the table, or `None` when vanilla does not load it, and the findings.
pub(crate) fn parse_table(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Option<LootTable>, Diagnostics) {
    let mut walk = Walk {
        cx,
        findings: Diagnostics::default(),
    };
    let root = match read_root(bytes) {
        Ok(root) => root,
        Err(message) => {
            // A file the JSON reader rejects, or whose root is not an object, is not loaded
            // and not cached.
            let text = Some(texts::formatted(texts::INVALID_TABLE, [path.as_str()]));
            // A file of no value at all (no bytes, a byte-order mark alone, whitespace and
            // comments, `null`) is the empty document of every family; the line is
            // vanilla's either way.
            let code = if is_blank(bytes) {
                DiagnosticCode::EmptyDocument { text }
            } else {
                DiagnosticCode::MalformedInput {
                    message: message.to_string(),
                    text,
                }
            };
            walk.findings
                .push(finding(code, Severity::Error, &JsonPath::new()));
            return (None, walk.findings);
        }
    };
    let table = walk.table(&root, &JsonPath::new());
    // A loot table has no upgrade to finish its load: the places of the elements the walk
    // kept are settled here, before anything else is told of the file.
    walk.findings.name_as_the_file_has_it();
    (Some(table), walk.findings)
}

/// Whether the file holds no value: nothing but whitespace and comments after its byte-order
/// mark, or `null`.
fn is_blank(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(strip_bom(bytes)) else {
        return false;
    };
    let null = |text: &str| {
        bedrock_addon_core::json::from_str_jsonc::<Value>(text).is_ok_and(|v| v.is_null())
    };
    // A trailing line comment ends at the added newline.
    null(text) || null(&format!("{text}\nnull"))
}

/// Why a file has no root object: the message is the reader's.
#[derive(Debug, thiserror::Error)]
enum RootError {
    /// The nesting passes the reader's limit.
    #[error(transparent)]
    TooDeep(#[from] bedrock_addon_core::json::TooDeep),
    /// The bytes are not UTF-8.
    #[error(transparent)]
    NotUtf8(#[from] std::str::Utf8Error),
    /// The text is not JSON; the parser's message.
    #[error("{0}")]
    Json(String),
    /// The root is JSON but not an object.
    #[error("the root is not an object")]
    NotObject,
}

/// The root object of the file, or the reader's message.
fn read_root(bytes: &[u8]) -> Result<VObject, RootError> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    // of a member written twice the last stands.
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )
    .map_err(|e| RootError::Json(e.to_string()))?;
    match root.as_object() {
        Some(object) => Ok(object.clone()),
        None => Err(RootError::NotObject),
    }
}

/// `value` as vanilla's `asString` prints it in a message: a string as is, `null` as nothing,
/// anything else as one line of JSON.
fn as_written(value: &Value) -> String {
    if let Some(text) = value.as_string() {
        return text.as_str().to_owned();
    }
    if value.is_null() {
        return String::new();
    }
    compact(value)
}

/// `value` as jsoncpp's `toStyledString` prints it in vanilla's banner messages: the JSON text
/// and a newline. Objects and arrays are written on one line, where jsoncpp indents them.
fn styled(value: &Value) -> String {
    let mut text = compact(value);
    text.push('\n');
    text
}

/// `value` as one line of JSON.
fn compact(value: &Value) -> String {
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    // A value read from JSON text always has a JSON form.
    let _ = write_json(value, &mut out, style);
    String::from_utf8(out).unwrap_or_default()
}

/// A finding of the walk at `at`; the loader sets the pack and the file.
fn finding(code: DiagnosticCode, severity: Severity, at: &JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at.clone(), None),
    )
}

/// The function only the Upcoming Creator Features registry holds.
const CARRY_OVER: &str = "carry_over_block_entity_data";

/// The engine that has `carry_over_block_entity_data` (1.26.40).
const CARRY_OVER_SINCE: FormatVersion = FormatVersion::new(1, 26, 40);

/// The engine that has the `biome_has_tag` condition (1.26.30).
const BIOME_HAS_TAG_SINCE: FormatVersion = FormatVersion::new(1, 26, 30);

/// The most patterns a banner holds.
const MAX_PATTERNS: usize = 6;

/// The walk: the load context and the findings so far. The findings also keep, for each
/// list of pools, entries, functions and conditions the walk leaves an element out of, where
/// the file has every element held (`Diagnostics::left_out`, each list once everything is
/// told of it, an inner list before the one that holds it): the link pass counts the held
/// lists and names each element as the file has it.
struct Walk<'a> {
    cx: &'a LoadContext,
    findings: Diagnostics,
}

/// The members of one object, and which of them the walk has read.
struct Members<'v> {
    object: &'v VObject,
    read: Vec<&'static str>,
}

impl<'v> Members<'v> {
    fn new(object: &'v VObject) -> Self {
        Self {
            object,
            read: Vec::new(),
        }
    }

    /// The member `key`, which the walk reads.
    fn get(&mut self, key: &'static str) -> Option<&'v Value> {
        self.read.push(key);
        self.object.get(key)
    }

    /// The object has the member `key`.
    fn has(&self, key: &str) -> bool {
        self.object.contains_key(key)
    }

    /// The members the walk did not read, in document order.
    fn rest(&self) -> impl Iterator<Item = (&'v str, &'v Value)> + '_ {
        self.object
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .filter(|(key, _)| !self.read.contains(key))
    }
}

/// `at/key`.
fn child(at: &JsonPath, key: &str) -> JsonPath {
    let mut path = at.clone();
    path.push(key);
    path
}

/// `at/index`.
fn item_at(at: &JsonPath, index: usize) -> JsonPath {
    let mut path = at.clone();
    path.push_index(index);
    path
}

/// The values vanilla iterates for an array member: the elements of an array, the member values
/// of an object (jsoncpp iterates both); `None` for a scalar, which gives nothing.
fn elements(value: &Value) -> Option<Vec<&Value>> {
    if let Some(array) = value.as_array() {
        Some(array.as_slice().iter().collect())
    } else {
        value.as_object().map(|object| object.values().collect())
    }
}

/// The values of a member that takes an array or a single value, each with its path: an
/// element is at its index, a single value at the member itself.
fn one_or_many<'v>(value: &'v Value, at: &JsonPath) -> Vec<(&'v Value, JsonPath)> {
    match value.as_array() {
        Some(array) => array
            .as_slice()
            .iter()
            .enumerate()
            .map(|(index, element)| (element, item_at(at, index)))
            .collect(),
        None => vec![(value, at.clone())],
    }
}

/// `text` without everything up to and including its last `:`, and that prefix.
fn split_namespace(text: &str) -> (Option<String>, &str) {
    match text.rfind(':') {
        Some(at) => (Some(text[..=at].to_owned()), &text[at + 1..]),
        None => (None, text),
    }
}

impl Walk<'_> {
    /// A warning at `at`: a warning-level line of the game's content log (each caller says which).
    fn warn(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.findings.push(finding(code, Severity::Warning, at));
    }

    /// An information of the crate's own at `at`: the game reads on in silence.
    fn note(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.findings.push(finding(code, Severity::Info, at));
    }

    /// A content-log error of the game's after which it keeps the table (the element is dropped
    /// or defaulted and the loader goes on): a kept error.
    fn kept(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.findings
            .push_kept_error(finding(code, Severity::Error, at));
    }

    /// A member of a JSON type its reader does not take: an `InvalidType` information of the
    /// crate's own, and the member is read as absent.
    fn wrong_type(&mut self, at: &JsonPath) {
        self.note(DiagnosticCode::InvalidType { text: None }, at);
    }

    /// The Upcoming Creator Features experiment is on.
    fn ucf(&self) -> bool {
        self.cx.upcoming_creator_features
            || self
                .cx
                .experiments
                .contains(Experiment::UpcomingCreatorFeatures)
    }

    /// `carry_over_block_entity_data.dynamic_properties`: a bool; anything else logs vanilla's
    /// error and keeps `false`: a kept error, since the function and the table load.
    fn dynamic_properties(&mut self, value: &Value, at: &JsonPath) -> Option<bool> {
        if let Some(flag) = value.as_bool() {
            return Some(flag);
        }
        self.kept(
            DiagnosticCode::InvalidType {
                text: Some(texts::fixed(texts::DYNAMIC_PROPERTIES)),
            },
            at,
        );
        None
    }

    // ---- scalars ---------------------------------------------------------------------------
    //
    // Each reader takes the JSON type vanilla's accessor reads without converting; any other
    // type (`null` included) is an `InvalidType` information of the crate's own and reads as absent.

    /// A float member (`asFloat`): a number.
    fn float(&mut self, value: &Value, at: &JsonPath) -> Option<f32> {
        #[allow(clippy::cast_possible_truncation)] // jsoncpp's `asFloat` narrows the same way
        let read = value
            .as_number()
            .map(|n| n.to_f64_lossy() as f32)
            .filter(|x| x.is_finite());
        if read.is_none() {
            self.wrong_type(at);
        }
        read
    }

    /// An integer member (`asInt`): an integer, or a real truncated toward zero, in `i32`.
    /// A real with a fraction is truncated as vanilla does, in silence, and the crate tells with an `InvalidValue` information at the member, since the table is
    /// written with the integer.
    fn int(&mut self, value: &Value, at: &JsonPath) -> Option<i32> {
        let read = value.as_number().and_then(|number| {
            if number.is_float() {
                let real = number.to_f64_lossy();
                let x = real.trunc();
                if (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&x) && real.fract() != 0.0 {
                    self.note(DiagnosticCode::InvalidValue { text: None }, at);
                }
                #[allow(clippy::cast_possible_truncation)] // checked to be in range
                (x >= f64::from(i32::MIN) && x <= f64::from(i32::MAX)).then_some(x as i32)
            } else {
                number.to_i64().and_then(|i| i32::try_from(i).ok())
            }
        });
        if read.is_none() {
            self.wrong_type(at);
        }
        read
    }

    /// An integer member read only when jsoncpp's `isInt` holds: an integer, or an integral
    /// real, in `i32`.
    fn whole(&mut self, value: &Value, at: &JsonPath) -> Option<i32> {
        let integral = value
            .as_number()
            .is_some_and(|n| !n.is_float() || n.to_f64_lossy().fract() == 0.0);
        if integral {
            self.int(value, at)
        } else {
            self.wrong_type(at);
            None
        }
    }

    /// A bool member (`asBool`): a bool.
    fn boolean(&mut self, value: &Value, at: &JsonPath) -> Option<bool> {
        let read = value.as_bool();
        if read.is_none() {
            self.wrong_type(at);
        }
        read
    }

    /// A string member (`asString`): a string.
    fn string(&mut self, value: &Value, at: &JsonPath) -> Option<String> {
        let read = value.as_string().map(|s| s.as_str().to_owned());
        if read.is_none() {
            self.wrong_type(at);
        }
        read
    }

    /// The member `key` of `m` read with `read`.
    fn member<T>(
        &mut self,
        m: &mut Members<'_>,
        key: &'static str,
        at: &JsonPath,
        read: impl FnOnce(&mut Self, &Value, &JsonPath) -> Option<T>,
    ) -> Option<T> {
        let value = m.get(key)?;
        read(self, value, &child(at, key))
    }

    /// A `RandomValueBounds`: a number sets both bounds; an object gives `min` and `max`.
    /// Other members of the object are reported and not kept.
    fn bounds(&mut self, value: &Value, at: &JsonPath) -> Option<RandomValueBounds> {
        if value.is_number() {
            return self.float(value, at).map(RandomValueBounds::Exact);
        }
        let Some(object) = value.as_object() else {
            self.wrong_type(at);
            return None;
        };
        let mut m = Members::new(object);
        let min = self.member(&mut m, "min", at, Self::float);
        let max = self.member(&mut m, "max", at, Self::float);
        let _ = self.rest(&m, at);
        Some(RandomValueBounds::Range { min, max })
    }

    /// An `IntRange`, with the shared type's findings.
    fn int_range(&mut self, value: &Value, at: &JsonPath) -> Option<IntRange> {
        let Ok(range) = IntRange::try_from(value.clone()) else {
            self.wrong_type(at);
            return None;
        };
        if range.form == RangeForm::MinMax {
            // The game's reader takes `range_min` / `range_max`, not the `min` / `max` of the
            // shared type's second form: an `InvalidType` finding, the value kept.
            self.wrong_type(at);
        }
        for finding in &range.check(at) {
            self.findings.push(finding.clone());
        }
        Some(range)
    }

    /// An array of strings, or a single string; an element that is not a string is skipped.
    fn strings(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<String>> {
        if value.is_string() {
            return self.string(value, at).map(|s| vec![s]);
        }
        let Some(array) = value.as_array() else {
            self.wrong_type(at);
            return None;
        };
        let mut out = Vec::new();
        for (index, element) in array.as_slice().iter().enumerate() {
            if let Some(text) = self.string(element, &item_at(at, index)) {
                out.push(text);
            }
        }
        Some(out)
    }

    /// The members of `m` the walk did not read, kept for write-back.
    fn rest(&mut self, m: &Members<'_>, at: &JsonPath) -> Extra {
        self.rest_except(m, at, &[])
    }

    /// [`rest`](Self::rest), reporting each member as `UnrecognizedField` at info except the
    /// `silent` ones. Vanilla ignores such members without a word; the information is the crate's own, and
    /// stays an information because vanilla's own tables carry them.
    fn rest_except(&mut self, m: &Members<'_>, at: &JsonPath, silent: &[&str]) -> Extra {
        let mut extra = Extra::default();
        for (key, value) in m.rest() {
            if !silent.contains(&key) {
                self.findings.push(finding(
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Info,
                    &child(at, key),
                ));
            }
            extra.insert(key.to_owned(), value.clone());
        }
        extra
    }

    // ---- the table -------------------------------------------------------------------------

    /// The table: reads only `pools`.
    fn table(&mut self, object: &VObject, at: &JsonPath) -> LootTable {
        let mut m = Members::new(object);
        let pools = self.member(&mut m, "pools", at, Self::pools);
        // `format_version` is never read and needs no finding: it is kept as written.
        let extra = self.rest_except(&m, at, &["format_version"]);
        LootTable { pools, extra }
    }

    /// The pools: every element that is an object becomes a pool. Vanilla appends an empty
    /// pool (0 rolls, no entries) for an element that is not one and says nothing; such a pool
    /// gives nothing, so the element is dropped with a wrong-type finding and no `{}` is
    /// invented on write-back.
    fn pools(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<Pool>> {
        let Some(values) = elements(value) else {
            self.wrong_type(at);
            return None;
        };
        let mut pools = Vec::new();
        let mut kept = Vec::new();
        for (index, element) in values.into_iter().enumerate() {
            let at = item_at(at, index);
            match element.as_object() {
                Some(object) => {
                    pools.push(self.pool(object, &at));
                    kept.push(index);
                }
                None => self.wrong_type(&at),
            }
        }
        self.findings.left_out(at, &kept);
        Some(pools)
    }

    /// One pool.
    fn pool(&mut self, object: &VObject, at: &JsonPath) -> Pool {
        let mut m = Members::new(object);
        let rolls = self.member(&mut m, "rolls", at, Self::bounds);
        let bonus_rolls = self.member(&mut m, "bonus_rolls", at, Self::bounds);
        let conditions = self.member(&mut m, "conditions", at, Self::conditions);
        let tiers = self.member(&mut m, "tiers", at, Self::tiers);
        let entries = if let Some(value) = m.get("entries") {
            self.entries(value, &child(at, "entries"))
        } else {
            // The game logs the text at the content log's error level and keeps the pool
            // with no entries; the loot reader keeps the table too: a kept error.
            self.findings.push_kept_error(finding(
                DiagnosticCode::MissingRequiredField {
                    text: Some(texts::fixed(texts::ENTRIES_ARRAY)),
                },
                Severity::Error,
                at,
            ));
            None
        };
        let extra = self.rest(&m, at);
        Pool {
            rolls,
            bonus_rolls,
            conditions,
            tiers,
            entries,
            extra,
        }
    }

    /// A pool's tier selection.
    fn tiers(&mut self, value: &Value, at: &JsonPath) -> Option<Tiers> {
        let Some(object) = value.as_object() else {
            self.wrong_type(at);
            return None;
        };
        let mut m = Members::new(object);
        let initial_range = self.member(&mut m, "initial_range", at, Self::int);
        let bonus_rolls = self.member(&mut m, "bonus_rolls", at, Self::int);
        let bonus_chance = self.member(&mut m, "bonus_chance", at, Self::float);
        let extra = self.rest(&m, at);
        Some(Tiers {
            initial_range,
            bonus_rolls,
            bonus_chance,
            extra,
        })
    }

    /// The entries of a pool: an array, whose elements that are not objects are skipped.
    fn entries(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<Entry>> {
        let Some(array) = value.as_array() else {
            // The same test and line as for a missing array: the game's error, the pool kept
            // with no entries and the table with it (a kept error).
            self.findings.push_kept_error(finding(
                DiagnosticCode::InvalidType {
                    text: Some(texts::fixed(texts::ENTRIES_ARRAY)),
                },
                Severity::Error,
                at,
            ));
            return None;
        };
        let mut entries = Vec::new();
        let mut kept = Vec::new();
        for (index, element) in array.as_slice().iter().enumerate() {
            let at = item_at(at, index);
            let Some(object) = element.as_object() else {
                // The element is skipped, the table kept.
                self.kept(
                    DiagnosticCode::InvalidType {
                        text: Some(texts::fixed(texts::ENTRY_OBJECT)),
                    },
                    &at,
                );
                continue;
            };
            if let Some(entry) = self.entry(object, &at) {
                entries.push(entry);
                kept.push(index);
            }
        }
        self.findings.left_out(at, &kept);
        Some(entries)
    }

    /// One entry: `weight`, `quality`, `conditions`, then `type` and its
    /// members, then an inline `pools`. `None` when the type is not one vanilla knows.
    fn entry(&mut self, object: &VObject, at: &JsonPath) -> Option<Entry> {
        let mut m = Members::new(object);
        let weight = self.member(&mut m, "weight", at, Self::int);
        let quality = self.member(&mut m, "quality", at, Self::int);
        let conditions = self.member(&mut m, "conditions", at, Self::conditions);
        let written = m.get("type");
        // The type is matched exactly by length and bytes.
        let kind = match written.and_then(Value::as_string).map(VString::as_str) {
            Some("item") => EntryKind::Item {
                name: self
                    .member(&mut m, "name", at, Self::string)
                    .map(ItemId::parse),
                functions: self.member(&mut m, "functions", at, Self::functions),
            },
            Some("loot_table") => EntryKind::LootTable {
                name: self
                    .member(&mut m, "name", at, Self::string)
                    .map(LootTablePath::parse),
            },
            Some("empty") => EntryKind::Empty,
            _ => {
                // The entry is dropped; vanilla logs an error and loads
                // the table: a kept error.
                let given = written.map(as_written).unwrap_or_default();
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::formatted(texts::ENTRY_TYPE, [&given])),
                    },
                    at,
                );
                return None;
            }
        };
        let pools = self.member(&mut m, "pools", at, Self::pools);
        let extra = self.rest(&m, at);
        Some(Entry {
            kind,
            weight,
            quality,
            conditions,
            pools,
            extra,
        })
    }

    // ---- functions -------------------------------------------------------------------------

    /// The functions of an entry.
    fn functions(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<LootFunction>> {
        let Some(values) = elements(value) else {
            self.wrong_type(at);
            return None;
        };
        let mut out = Vec::new();
        let mut kept = Vec::new();
        for (index, element) in values.into_iter().enumerate() {
            let at = item_at(at, index);
            let Some(object) = element.as_object() else {
                // The element is skipped, the table kept.
                self.kept(
                    DiagnosticCode::InvalidType {
                        text: Some(texts::fixed(texts::FUNCTION_OBJECT)),
                    },
                    &at,
                );
                continue;
            };
            if let Some(function) = self.function(object, &at) {
                out.push(function);
                kept.push(index);
            }
        }
        self.findings.left_out(at, &kept);
        Some(out)
    }

    /// One function: `conditions` first, then the name (after its last `:`)
    /// looked up in the registry.
    fn function(&mut self, object: &VObject, at: &JsonPath) -> Option<LootFunction> {
        let mut m = Members::new(object);
        // The name as jsoncpp's `asString` gives it: the three `%s` of a name (an entry's `type`, a
        // function's and a condition's) print a value that is not a string the same way.
        let written = m.get("function").map(as_written).unwrap_or_default();
        let conditions = self.member(&mut m, "conditions", at, Self::conditions);
        let (namespace, name) = split_namespace(&written);
        if name == CARRY_OVER && self.cx.engine_version >= CARRY_OVER_SINCE && !self.ucf() {
            // From 1.26.40 the game looks the name up in its Upcoming Creator Features
            // registry only with the experiment on, and otherwise logs this text and skips
            // the function: a missing toggle.
            self.kept(
                DiagnosticCode::MissingToggles {
                    text: Some(texts::formatted(texts::UCF_FUNCTION, [&written])),
                },
                at,
            );
            return None;
        }
        let Some(kind) = self.function_kind(name, &mut m, at) else {
            // The function is skipped; vanilla prints the name it was given.
            self.kept(
                DiagnosticCode::UnrecognizedComponent {
                    text: Some(texts::formatted(texts::UNSUPPORTED_FUNCTION, [&written])),
                },
                at,
            );
            return None;
        };
        let extra = self.rest(&m, at);
        Some(LootFunction {
            namespace,
            kind,
            conditions,
            extra,
        })
    }

    /// The function `name` with its parameters; `None` when the registry has no such name.
    #[allow(clippy::too_many_lines)] // one arm per registered function
    fn function_kind(
        &mut self,
        name: &str,
        m: &mut Members<'_>,
        at: &JsonPath,
    ) -> Option<FunctionKind> {
        let kind = match name {
            "set_count" => FunctionKind::SetCount {
                count: self.member(m, "count", at, Self::bounds),
            },
            "set_data" => FunctionKind::SetData(self.item_data(m, at)),
            "set_damage" => FunctionKind::SetDamage {
                damage: self.member(m, "damage", at, Self::bounds),
            },
            "looting_enchant" => FunctionKind::LootingEnchant {
                count: self.member(m, "count", at, Self::bounds),
            },
            "enchant_with_levels" => FunctionKind::EnchantWithLevels {
                levels: self.member(m, "levels", at, Self::bounds),
                treasure: self.member(m, "treasure", at, Self::boolean),
            },
            "enchant_book_for_trading" => FunctionKind::EnchantBookForTrading {
                base_cost: self.member(m, "base_cost", at, Self::int),
                per_level_cost: self.member(m, "per_level_cost", at, Self::int),
                base_random_cost: self.member(m, "base_random_cost", at, Self::int),
                per_level_random_cost: self.member(m, "per_level_random_cost", at, Self::int),
                enchantments: self.member(m, "enchantments", at, Self::enchant_options),
            },
            "enchant_randomly" => FunctionKind::EnchantRandomly {
                treasure: self.member(m, "treasure", at, Self::boolean),
            },
            "furnace_smelt" => FunctionKind::FurnaceSmelt,
            "set_data_from_color_index" => FunctionKind::SetDataFromColorIndex,
            "enchant_random_gear" => FunctionKind::EnchantRandomGear {
                chance: self.member(m, "chance", at, Self::float),
            },
            "random_aux_value" => FunctionKind::RandomAuxValue(self.item_data(m, at)),
            "random_block_state" => FunctionKind::RandomBlockState {
                block_state: self.member(m, "block_state", at, Self::string),
                values: self.member(m, "values", at, Self::bounds),
            },
            "random_dye" => FunctionKind::RandomDye,
            "exploration_map" => FunctionKind::ExplorationMap {
                destination: self.member(m, "destination", at, Self::string),
            },
            "set_banner_details" => self.banner(m, at),
            "explosion_decay" => FunctionKind::ExplosionDecay,
            "set_name" => FunctionKind::SetName {
                name: self.member(m, "name", at, Self::string),
            },
            "set_lore" => FunctionKind::SetLore {
                lore: self.member(m, "lore", at, Self::strings),
            },
            "specific_enchants" => FunctionKind::SpecificEnchants {
                enchants: self.member(m, "enchants", at, Self::specific_enchants),
            },
            "fill_container" => FunctionKind::FillContainer {
                loot_table: self
                    .member(m, "loot_table", at, Self::string)
                    .map(LootTablePath::parse),
            },
            "set_actor_id" => FunctionKind::SetActorId {
                id: self.member(m, "id", at, Self::string).map(EntityId::parse),
            },
            "set_book_contents" => FunctionKind::SetBookContents {
                author: self.member(m, "author", at, Self::string),
                title: self.member(m, "title", at, Self::string),
                pages: self.member(m, "pages", at, Self::strings),
            },
            "set_stew_effect" => FunctionKind::SetStewEffect {
                effects: self.member(m, "effects", at, Self::stew_effects),
            },
            "set_ominous_bottle_amplifier" => FunctionKind::SetOminousBottleAmplifier {
                amplifier: self.member(m, "amplifier", at, Self::bounds),
            },
            "set_armor_trim" => FunctionKind::SetArmorTrim {
                material: self.member(m, "material", at, Self::string),
                pattern: self.member(m, "pattern", at, Self::string),
            },
            "set_potion" => FunctionKind::SetPotion {
                id: self.member(m, "id", at, Self::string),
            },
            CARRY_OVER if self.cx.engine_version >= CARRY_OVER_SINCE => {
                FunctionKind::CarryOverBlockEntityData {
                    dynamic_properties: self.member(
                        m,
                        "dynamic_properties",
                        at,
                        Self::dynamic_properties,
                    ),
                }
            }
            _ => return None,
        };
        Some(kind)
    }

    /// The parameters of `set_data` and `random_aux_value`: `data`, then `values`.
    fn item_data(&mut self, m: &mut Members<'_>, at: &JsonPath) -> ItemData {
        let data = self.member(m, "data", at, Self::bounds);
        let values = self.member(m, "values", at, Self::bounds);
        if !m.has("data") && !m.has("values") {
            // A content-log warning of the game's; the function is kept with range 0..0.
            self.warn(
                DiagnosticCode::MissingRequiredField {
                    text: Some(texts::fixed(texts::SET_DATA_KEY)),
                },
                at,
            );
        }
        ItemData { data, values }
    }

    /// `enchant_book_for_trading.enchantments`: an array of options; `null` is none. Each finding keeps the function; vanilla logs errors,
    /// which are kept errors since the table loads.
    fn enchant_options(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<EnchantOption>> {
        if value.is_null() {
            return None;
        }
        let Some(array) = value.as_array() else {
            // A member that is no array gives no options.
            self.kept(
                DiagnosticCode::InvalidType {
                    text: Some(texts::fixed(texts::ENCHANT_ARRAY)),
                },
                at,
            );
            return None;
        };
        let mut out = Vec::new();
        for (index, element) in array.as_slice().iter().enumerate() {
            let at = item_at(at, index);
            let empty = VObject::new();
            let object = element.as_object().unwrap_or(&empty);
            let mut m = Members::new(object);
            let written = m.get("name");
            let name = written
                .and_then(Value::as_string)
                .map(VString::as_str)
                .filter(|name| is_enchantment(name));
            let Some(name) = name else {
                // No enchantment has that name: the option is dropped with an error-level
                // line of the game's content log and the table is kept: a kept error.
                let shown = written.map(as_written).unwrap_or_default();
                self.kept(
                    DiagnosticCode::UnresolvedReference {
                        text: Some(texts::formatted(texts::ENCHANT_NAME, [&shown])),
                    },
                    &at,
                );
                continue;
            };
            let name = name.to_owned();
            // `min` and `max` are read only when they are integers.
            let min = self.member(&mut m, "min", &at, Self::whole);
            let max = self.member(&mut m, "max", &at, Self::whole);
            let extra = self.rest(&m, &at);
            out.push(EnchantOption {
                name,
                min,
                max,
                extra,
            });
        }
        Some(out)
    }

    /// `set_banner_details`: `type`, `base_color`, then `patterns`. Every finding keeps the
    /// function; vanilla logs errors, which are kept errors since the table loads.
    fn banner(&mut self, m: &mut Members<'_>, at: &JsonPath) -> FunctionKind {
        let banner_type = m.get("type").and_then(|value| {
            let read = banner_type(value);
            if read.is_none() {
                // A banner type that names neither type keeps the default type.
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::formatted(texts::BANNER_TYPE, [&styled(value)])),
                    },
                    &child(at, "type"),
                );
            }
            read
        });
        let base_color = m.get("base_color").and_then(|value| {
            let read = item_color(value);
            if read.is_none() {
                // A colour that names none of the 16 is white.
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::formatted(texts::BANNER_BASE_COLOR, [&styled(value)])),
                    },
                    &child(at, "base_color"),
                );
            }
            read
        });
        let patterns = m.get("patterns").and_then(|value| {
            let at = child(at, "patterns");
            if banner_type.unwrap_or_default() != BannerType::Default {
                // A banner that is not a default one takes no patterns: they are ignored.
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::fixed(texts::BANNER_NON_DEFAULT)),
                    },
                    &at,
                );
                return None;
            }
            Some(self.banner_patterns(value, &at))
        });
        FunctionKind::SetBannerDetails {
            banner_type,
            base_color,
            patterns,
        }
    }

    /// The patterns of a default banner: an array or a single object, at most
    /// [`MAX_PATTERNS`]; the ones past it are dropped after
    /// vanilla's error.
    fn banner_patterns(&mut self, value: &Value, at: &JsonPath) -> Vec<BannerPattern> {
        let values = one_or_many(value, at);
        if values.len() > MAX_PATTERNS {
            self.kept(
                DiagnosticCode::InvalidValue {
                    text: Some(texts::formatted(
                        texts::BANNER_MAX,
                        [&MAX_PATTERNS.to_string(), &values.len().to_string()],
                    )),
                },
                at,
            );
        }
        let mut out = Vec::new();
        for (element, at) in values.into_iter().take(MAX_PATTERNS) {
            if let Some(pattern) = self.banner_pattern(element, &at) {
                out.push(pattern);
            }
        }
        out
    }

    /// One banner pattern: a pattern without a name is dropped with
    /// vanilla's error; a colour that is not one is white, with vanilla's error.
    fn banner_pattern(&mut self, value: &Value, at: &JsonPath) -> Option<BannerPattern> {
        let empty = VObject::new();
        let object = value.as_object().unwrap_or(&empty);
        let mut m = Members::new(object);
        let name = m.get("pattern");
        let pattern = match name.map(|v| (v, v.as_string())) {
            Some((_, Some(text))) if !text.is_empty() => text.as_str().to_owned(),
            Some((value, None)) if !value.is_null() => {
                self.wrong_type(&child(at, "pattern"));
                return None;
            }
            _ => {
                // No name names a pattern: vanilla's "Invalid banner pattern name" with the
                // empty name, and nothing is appended. Absent, `null` and `""` all read as the
                // empty string, which no pattern has. Every other name is kept as written and
                // not checked.
                self.kept(
                    DiagnosticCode::UnresolvedReference {
                        text: Some(texts::formatted(texts::BANNER_PATTERN, [""])),
                    },
                    at,
                );
                return None;
            }
        };
        let color = m.get("color").and_then(|value| {
            let read = item_color(value);
            if read.is_none() {
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::formatted(
                            texts::BANNER_PATTERN_COLOR,
                            [&styled(value)],
                        )),
                    },
                    &child(at, "color"),
                );
            }
            read
        });
        let extra = self.rest(&m, at);
        Some(BannerPattern {
            pattern,
            color,
            extra,
        })
    }

    /// `specific_enchants.enchants`: an array, or a single value, of ids (level 1) and
    /// `{id, level}` objects. An id that is not an enabled enchantment's is dropped; vanilla
    /// says nothing, so the finding is the crate's own.
    #[allow(clippy::unnecessary_wraps)] // a member reader: `None` is "read as absent"
    fn specific_enchants(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<SpecificEnchant>> {
        let values = one_or_many(value, at);
        let mut out = Vec::new();
        for (element, at) in values {
            let enchant = if let Some(id) = element.as_string() {
                SpecificEnchant::new(id.as_str())
            } else if let Some(object) = element.as_object() {
                let mut m = Members::new(object);
                let written = m.get("id");
                if written.is_some_and(|id| !id.is_string()) {
                    self.wrong_type(&child(&at, "id"));
                    continue;
                }
                // Without an `id` the name is empty, which names no enchantment.
                let id = written.map(as_written).unwrap_or_default();
                let level = self.member(&mut m, "level", &at, Self::int_range);
                let extra = self.rest(&m, &at);
                SpecificEnchant { id, level, extra }
            } else {
                self.wrong_type(&at);
                continue;
            };
            if is_enchantment(&enchant.id) {
                out.push(enchant);
            } else {
                // An `UnresolvedReference` information of the crate's own: vanilla drops it
                // in silence.
                self.note(DiagnosticCode::UnresolvedReference { text: None }, &at);
            }
        }
        Some(out)
    }

    /// `set_stew_effect.effects`: an array of `{id}`, or a single one.
    #[allow(clippy::unnecessary_wraps)] // a member reader: `None` is "read as absent"
    fn stew_effects(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<StewEffect>> {
        let values = one_or_many(value, at);
        let mut out = Vec::new();
        for (element, at) in values {
            let Some(object) = element.as_object() else {
                self.wrong_type(&at);
                continue;
            };
            let mut m = Members::new(object);
            let id = self.member(&mut m, "id", &at, Self::int);
            let extra = self.rest(&m, &at);
            out.push(StewEffect { id, extra });
        }
        Some(out)
    }

    // ---- conditions ------------------------------------------------------------------------

    /// The conditions of a pool, entry or function.
    fn conditions(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<LootCondition>> {
        let Some(values) = elements(value) else {
            self.wrong_type(at);
            return None;
        };
        let mut out = Vec::new();
        let mut kept = Vec::new();
        for (index, element) in values.into_iter().enumerate() {
            let at = item_at(at, index);
            let Some(object) = element.as_object() else {
                // The element is skipped, the table kept.
                self.kept(
                    DiagnosticCode::InvalidType {
                        text: Some(texts::fixed(texts::CONDITION_OBJECT)),
                    },
                    &at,
                );
                continue;
            };
            if let Some(condition) = self.condition(object, &at) {
                out.push(condition);
                kept.push(index);
            }
        }
        self.findings.left_out(at, &kept);
        Some(out)
    }

    /// One condition: the exact name looked up in the factory map.
    fn condition(&mut self, object: &VObject, at: &JsonPath) -> Option<LootCondition> {
        // The game reads `operator` (default `==`) for every condition before the name
        // lookup; any other value logs this text at the content log's error level and drops
        // the condition, the table kept: a kept error. A valid operator stays in the condition's `extra` for write-back.
        if let Some(operator) = object.get("operator") {
            let shown = operator
                .as_string()
                .map_or_else(|| compact(operator), |text| text.as_str().to_owned());
            if !matches!(shown.as_str(), "==" | "equals" | "!=" | "not") {
                self.kept(
                    DiagnosticCode::InvalidValue {
                        text: Some(texts::formatted(texts::CONDITION_OPERATOR, [&shown])),
                    },
                    at,
                );
                return None;
            }
        }
        let mut m = Members::new(object);
        let name = m.get("condition").map(as_written).unwrap_or_default();
        let Some(kind) = self.condition_kind(&name, &mut m, at) else {
            // Vanilla skips it with a developer-log line, so the pool, entry or function is
            // not gated by it; the finding carries that line as an information.
            self.findings.push(finding(
                DiagnosticCode::UnrecognizedComponent {
                    text: Some(texts::formatted(texts::UNKNOWN_CONDITION, [&name])),
                },
                Severity::Info,
                at,
            ));
            return None;
        };
        let extra = self.rest(&m, at);
        Some(LootCondition { kind, extra })
    }

    /// The condition `name` with its parameters; `None` when the map has no such name.
    fn condition_kind(
        &mut self,
        name: &str,
        m: &mut Members<'_>,
        at: &JsonPath,
    ) -> Option<ConditionKind> {
        let entity = |w: &mut Self, m: &mut Members<'_>| {
            w.member(m, "entity_type", at, Self::string)
                .map(EntityId::parse)
        };
        let kind = match name {
            "killed_by_player" => ConditionKind::KilledByPlayer,
            "killed_by_player_or_pets" => ConditionKind::KilledByPlayerOrPets,
            "killed_by_entity" => ConditionKind::KilledByEntity {
                entity_type: entity(self, m),
            },
            "entity_killed" => ConditionKind::EntityKilled {
                entity_type: entity(self, m),
            },
            "random_chance" => ConditionKind::RandomChance {
                chance: self.member(m, "chance", at, Self::float),
            },
            "random_difficulty_chance" => ConditionKind::RandomDifficultyChance {
                default_chance: self.member(m, "default_chance", at, Self::float),
                peaceful: self.member(m, "peaceful", at, Self::float),
                easy: self.member(m, "easy", at, Self::float),
                normal: self.member(m, "normal", at, Self::float),
                hard: self.member(m, "hard", at, Self::float),
            },
            "random_chance_with_looting" => ConditionKind::RandomChanceWithLooting {
                chance: self.member(m, "chance", at, Self::float),
                looting_multiplier: self.member(m, "looting_multiplier", at, Self::float),
            },
            "random_regional_difficulty_chance" => ConditionKind::RandomRegionalDifficultyChance {
                max_chance: self.member(m, "max_chance", at, Self::float),
            },
            "has_mark_variant" => ConditionKind::HasMarkVariant {
                value: self.member(m, "value", at, Self::int),
            },
            "has_variant" => ConditionKind::HasVariant {
                value: self.member(m, "value", at, Self::int),
            },
            "match_tool" => self.match_tool(m, at),
            "damaged_by_entity" => ConditionKind::DamagedByEntity {
                entity_type: entity(self, m),
            },
            "passenger_of_entity" => ConditionKind::PassengerOfEntity {
                entity_type: entity(self, m),
            },
            "is_baby" => ConditionKind::IsBaby,
            "has_property" => ConditionKind::HasProperty {
                domain: self.member(m, "domain", at, Self::string),
            },
            "bool_property" => ConditionKind::BoolProperty {
                domain: self.member(m, "domain", at, Self::string),
                value: self.member(m, "value", at, Self::boolean),
            },
            "int_property" => ConditionKind::IntProperty {
                domain: self.member(m, "domain", at, Self::string),
                value: self.member(m, "value", at, Self::int),
            },
            "float_property" => ConditionKind::FloatProperty {
                domain: self.member(m, "domain", at, Self::string),
                value: self.member(m, "value", at, Self::float),
            },
            "enum_property" => ConditionKind::EnumProperty {
                domain: self.member(m, "domain", at, Self::string),
                value: self.member(m, "value", at, Self::string),
            },
            // The 20th condition of the game's map; engines before 1.26.30 have only 19, so an
            // older engine does not know the name.
            "biome_has_tag" if self.cx.engine_version >= BIOME_HAS_TAG_SINCE => {
                ConditionKind::BiomeHasTag {
                    tag: self.member(m, "tag", at, Self::string),
                }
            }
            _ => return None,
        };
        Some(kind)
    }

    /// `match_tool`.
    fn match_tool(&mut self, m: &mut Members<'_>, at: &JsonPath) -> ConditionKind {
        let tags = |w: &mut Self, m: &mut Members<'_>, key| w.member(m, key, at, Self::strings);
        ConditionKind::MatchTool {
            item: self.member(m, "item", at, Self::string).map(ItemId::parse),
            count: self.member(m, "count", at, Self::int_range),
            durability: self.member(m, "durability", at, Self::int_range),
            enchantments: self.member(m, "enchantments", at, Self::tool_enchantments),
            filter_any: tags(self, m, "minecraft:match_tool_filter_any"),
            filter_all: tags(self, m, "minecraft:match_tool_filter_all"),
            filter_none: tags(self, m, "minecraft:match_tool_filter_none"),
        }
    }

    /// `match_tool.enchantments`.
    fn tool_enchantments(&mut self, value: &Value, at: &JsonPath) -> Option<Vec<ToolEnchantment>> {
        let Some(array) = value.as_array() else {
            self.wrong_type(at);
            return None;
        };
        let mut out = Vec::new();
        for (index, element) in array.as_slice().iter().enumerate() {
            let at = item_at(at, index);
            let Some(object) = element.as_object() else {
                self.wrong_type(&at);
                continue;
            };
            let mut m = Members::new(object);
            let enchantment = self.member(&mut m, "enchantment", &at, Self::string);
            let levels = self.member(&mut m, "levels", &at, Self::int_range);
            let extra = self.rest(&m, &at);
            out.push(ToolEnchantment {
                enchantment,
                levels,
                extra,
            });
        }
        Some(out)
    }
}

/// `name` is the string id of a vanilla enchantment (`"sharpness"`, without a namespace), by the vanilla identifier lists, aliases included.
pub(crate) fn is_enchantment(name: &str) -> bool {
    let ids = bedrock_addon_core::vanilla::ids();
    let bare = |id: &&str| id.strip_prefix("minecraft:") == Some(name);
    ids.enchantments.iter().any(bare)
        || ids
            .enchantment_aliases
            .iter()
            .any(|(alias, target)| bare(alias) || bare(target))
}

/// A banner type: a name or 0..1.
fn banner_type(value: &Value) -> Option<BannerType> {
    if let Some(name) = value.as_string() {
        return match name.as_str() {
            "default" => Some(BannerType::Default),
            "illager_captain" => Some(BannerType::IllagerCaptain),
            _ => None,
        };
    }
    match value.as_number().filter(|n| !n.is_float())?.to_i64()? {
        0 => Some(BannerType::Default),
        1 => Some(BannerType::IllagerCaptain),
        _ => None,
    }
}

/// An item colour: a colour name or 0..15.
fn item_color(value: &Value) -> Option<ItemColor> {
    if let Some(name) = value.as_string() {
        return ItemColor::from_name(name.as_str());
    }
    let index = value.as_number().filter(|n| !n.is_float())?.to_i64()?;
    usize::try_from(index)
        .ok()
        .and_then(|i| ItemColor::ALL.get(i).copied())
}

#[cfg(test)]
mod root_tests {
    use super::{RootError, read_root};

    #[test]
    fn read_root_reports_bad_input_as_typed_errors() {
        assert!(matches!(
            read_root(&[0xff, b'{']),
            Err(RootError::NotUtf8(_))
        ));
        assert!(matches!(read_root(b"{\"a\":"), Err(RootError::Json(_))));
        assert!(matches!(read_root(b"[1]"), Err(RootError::NotObject)));
        let deep = "[".repeat(2000);
        assert!(matches!(
            read_root(deep.as_bytes()),
            Err(RootError::TooDeep(_))
        ));
    }

    #[test]
    fn read_root_error_text_is_the_readers_message() {
        let error = read_root(b"[1]").unwrap_err();
        assert_eq!(error.to_string(), "the root is not an object");
    }
}
