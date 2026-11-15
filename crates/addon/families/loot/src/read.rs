//! The loot table reader: the types derive it (`#[derive(Facet, Schema)]`), and this module is
//! the front of a file and the functions their keys name.
//!
//! Vanilla reads each member with jsoncpp's `asInt` / `asFloat` / `asBool` / `asString` and its
//! default, and checks almost nothing: a member of another JSON type is an `InvalidType`
//! information and reads as absent (the reading is [`Reading::OWN`] strict, a failed member
//! defaulting and told), vanilla's drop and keep rules are the kept errors and drops the
//! functions here tell, and every member it does not read is kept in the level's `extra`.

use super::codes;
use super::condition::ConditionKind;
use super::function::{BannerPattern, BannerType, EnchantOption, FunctionKind, ItemColor};
use super::function::{LootFunction, SpecificEnchant, StewEffect};
use super::{Entry, EntryKind, LootTable, Pool};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::json::kit::{Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{
    Indent, MAX_JSON_DEPTH, Value, WriteStyle, blank_bom, check_depth, write_json,
};
use bedrock_addon_core::json::{PrepareError, Prepared};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::schema::derive_support::Fields;
use bedrock_addon_core::schema::{ElementFailure, FromJson, read_document, read_list};
use bedrock_addon_core::shared::{IntRange, RangeForm};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

/// Parses one file: the table, or `None` when vanilla does not load it, and the findings.
pub(crate) fn parse_table(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Option<LootTable>, Diagnostics) {
    let mut findings = Diagnostics::default();
    let root = match read_root(bytes) {
        Ok(root) => root,
        Err(message) => {
            // A file the JSON reader rejects, or whose root is not an object, is not loaded
            // and not cached.
            // A file of no value at all (no bytes, a byte-order mark alone, whitespace and
            // comments, `null`) is the empty document of every family; the line is
            // vanilla's either way.
            let finding = if is_blank(bytes) {
                Diagnostic::of(codes::INVALID_TABLE_EMPTY.with([path.as_str()]))
            } else {
                Diagnostic::of(codes::INVALID_TABLE.with([path.as_str()]))
                    .detail(DiagnosticDetail::Syntax(message.to_string()))
            };
            findings.push(finding);
            return (None, findings);
        }
    };
    // A wrong type is an information and the member reads as absent; the members are asked for
    // in the order vanilla's readers do, and the unknown ones told last.
    let reading = Reading::OWN
        .strict()
        .with_null(NullHandling::Rejected)
        .with_on_member_failure(MemberFailure::DefaultAndTell)
        .with_unknown(Severity::Info);
    let table = {
        let mut read = Read::new(reading, &mut findings)
            .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
            .unknown_severity(Severity::Info)
            .load(cx);
        read_document::<LootTable>(&Node::new(&root, JsonPath::new()), &mut read)
    };
    // A loot table has no upgrade to finish its load: the places of the elements the walk
    // kept are settled here, before anything else is told of the file.
    findings.name_as_the_file_has_it();
    (table, findings)
}

/// Whether the file holds no value: nothing but whitespace and comments after its byte-order
/// mark, or `null`.
fn is_blank(bytes: &[u8]) -> bool {
    let Ok(prepared) = Prepared::new(bytes) else {
        return false;
    };
    let text = prepared.as_str();
    let null =
        |text: &str| bedrock_addon_core::json::from_str_jsonc(text).is_ok_and(|v| v.is_null());
    // A trailing line comment ends at the added newline.
    null(text) || null(&format!("{text}\nnull"))
}

/// Why a file has no root object: the message is the reader's.
#[derive(Debug, thiserror::Error)]
enum RootError {
    /// The nesting passes the reader's limit, or the bytes are not UTF-8.
    #[error(transparent)]
    Text(#[from] PrepareError),
    /// The text is not JSON; the parser's message.
    #[error("{0}")]
    Json(String),
    /// The root is JSON but not an object.
    #[error("the root is not an object")]
    NotObject,
}

/// The root of the file, an object, or the reader's message.
fn read_root(bytes: &[u8]) -> Result<Value, RootError> {
    // The nesting before the encoding: a text that fails both is reported too deep. The mark
    // is blanked first, or the scan stops at it.
    let bytes = &*blank_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(PrepareError::from)?;
    let prepared = Prepared::new(bytes)?;
    let text = prepared.as_str();
    // of a member written twice the last stands.
    let root = bedrock_addon_core::json::from_str_jsonc(&bedrock_addon_core::json::last_wins(text))
        .map_err(|e| RootError::Json(e.kind.to_string()))?;
    if root.is_object() {
        Ok(root)
    } else {
        Err(RootError::NotObject)
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

/// The function only the Upcoming Creator Features registry holds.
const CARRY_OVER: &str = "carry_over_block_entity_data";

/// The engine that has `carry_over_block_entity_data` (1.26.40).
const CARRY_OVER_SINCE: FormatVersion = known::V1_26_40;

/// The engine that has the `biome_has_tag` condition (1.26.30).
const BIOME_HAS_TAG_SINCE: FormatVersion = known::V1_26_30;

/// The most patterns a banner holds.
const MAX_PATTERNS: usize = 6;

/// `text` without everything up to and including its last `:`, and that prefix.
fn split_namespace(text: &str) -> (Option<String>, &str) {
    match text.rfind(':') {
        Some(at) => (Some(text[..=at].to_owned()), &text[at + 1..]),
        None => (None, text),
    }
}

/// The member `key` of the object at `node`, as written.
fn member<'v>(node: &Node<'v>, key: &str) -> Option<&'v Value> {
    node.value().as_object()?.get(key)
}

/// Tells `code` at `at`: a content-log line of the game's after which it keeps the table (a
/// code of the kept class is told as the kept error it is).
fn tell(read: &mut Read<'_>, at: &Node<'_>, code: impl Into<Code>) {
    let mut finding = Diagnostic::of(code).at(at.path().clone());
    finding.location.span = at.span();
    read.push(finding);
}

/// The engine the table is loaded for and whether Upcoming Creator Features is on.
fn gates(read: &Read<'_>) -> (FormatVersion, bool) {
    // A read always has its load context here: `parse_table` gives it.
    read.load_context().map_or((known::V1_26_30, false), |cx| {
        let ucf = cx.upcoming_creator_features
            || cx.experiments.contains(Experiment::UpcomingCreatorFeatures);
        (cx.engine_version, ucf)
    })
}

// ---- the readers of a member's forms ------------------------------------------------------

/// An integer member (`asInt`): an integer, or a real truncated toward zero, in `i32`. A real
/// with a fraction is truncated as vanilla does, in silence, and the crate tells with an
/// `InvalidValue` information at the member, since the table is written with the integer.
pub(crate) fn int(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let value = node.value().as_number().and_then(|number| {
        if !number.is_float() {
            return number.to_i64().and_then(|i| i32::try_from(i).ok());
        }
        let whole_part = number.to_f64_lossy();
        let x = whole_part.trunc();
        if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&x) {
            return None;
        }
        if whole_part.fract() != 0.0 {
            tell_info(read, node, &core_codes::INVALID_VALUE);
        }
        #[allow(clippy::cast_possible_truncation)] // checked to be in range
        Some(x as i32)
    });
    if value.is_none() {
        read.wrong_type(node);
    }
    value
}

/// An information of the crate's own at `node`: the game reads on in silence.
fn tell_info(read: &mut Read<'_>, node: &Node<'_>, code: &'static CodeDef<0>) {
    let mut finding = Diagnostic::info(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// An integer member read only when jsoncpp's `isInt` holds: an integer, or an integral real,
/// in `i32`.
pub(crate) fn whole(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let integral = node
        .value()
        .as_number()
        .is_some_and(|n| !n.is_float() || n.to_f64_lossy().fract() == 0.0);
    if integral {
        int(node, read)
    } else {
        read.wrong_type(node);
        None
    }
}

/// An `IntRange`, with the shared type's findings; the game's reader takes `range_min` /
/// `range_max`, not the `min` / `max` of the shared type's second form, which is an
/// `InvalidType` finding with the value kept.
pub(crate) fn int_range(node: &Node<'_>, read: &mut Read<'_>) -> Option<IntRange> {
    let Ok(range) = IntRange::try_from(node.value().clone()) else {
        read.wrong_type(node);
        return None;
    };
    if range.form == RangeForm::MinMax {
        read.wrong_type(node);
    }
    for finding in &range.check(node.path()) {
        read.push(finding.clone());
    }
    Some(range)
}

/// An array of strings, or a single string; an element that is not a string is skipped.
pub(crate) fn strings(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    if node.value().is_string() {
        return read.string(node).map(|text| vec![text.to_owned()]);
    }
    let elements = read.array(node)?;
    let mut out = Vec::new();
    for element in elements.iter() {
        out.extend(read.string(&element).map(str::to_owned));
    }
    Some(out)
}

/// A list that takes an array or a single value: an element is read at its index, a single
/// value at the member itself; one that fails is left out.
fn one_or_many<T: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    element: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Vec<T> {
    let mut element = element;
    if node.value().is_array() {
        return read_list(node, read, ElementFailure::Drop, element).unwrap_or_default();
    }
    element(node, read).into_iter().collect()
}

/// The values vanilla iterates for an array member: the elements of an array, the member values
/// of an object (named by their position, in file order); a scalar gives nothing and is a wrong
/// type. A value that fails is left out (`code`: what an element that is no object is told as,
/// where the game says a line of its own).
fn elements<T: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    code: Option<&'static CodeDef<0>>,
) -> Option<Vec<T>> {
    let value = node.value();
    let one = |element: &Node<'_>, read: &mut Read<'_>| match code {
        Some(code) if !element.value().is_object() => {
            tell(read, element, code);
            None
        }
        _ => T::from_json(element, read),
    };
    if value.is_array() {
        return read_list(node, read, ElementFailure::Drop, one);
    }
    let Some(object) = value.as_object() else {
        read.wrong_type(node);
        return None;
    };
    let (mut out, mut kept) = (Vec::new(), Vec::new());
    for (index, member) in object.values().enumerate() {
        let at = Node::new(member, node.path().index(index));
        if let Some(read_it) = one(&at, read) {
            out.push(read_it);
            kept.push(index);
        }
    }
    read.findings_mut().left_out(node.path(), &kept);
    Some(out)
}

pub(crate) fn pools(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Pool>> {
    elements(node, read, None)
}

pub(crate) fn conditions(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<super::LootCondition>> {
    elements(node, read, Some(&codes::CONDITION_OBJECT))
}

pub(crate) fn functions(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<LootFunction>> {
    elements(node, read, Some(&codes::FUNCTION_OBJECT))
}

/// A pool's `entries`: an array, whose elements that are not objects are skipped. Anything else
/// is the game's error and the pool is kept with no entries, the table with it (a kept error).
pub(crate) fn entries(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Entry>> {
    if !node.value().is_array() {
        tell(read, node, &codes::ENTRIES_ARRAY);
        return None;
    }
    read_list(node, read, ElementFailure::Drop, |element, read| {
        if element.value().is_object() {
            Entry::from_json(element, read)
        } else {
            tell(read, element, &codes::ENTRY_OBJECT);
            None
        }
    })
}

/// A pool without `entries`: the game logs the text at the content log's error level and keeps
/// the pool with no entries (`schema::after_read` of [`Pool`]).
pub(crate) fn pool_rules(_: &mut Pool, node: &Node<'_>, read: &mut Read<'_>) {
    if member(node, "entries").is_none() {
        tell(read, node, &codes::ENTRIES_MISSING);
    }
}

// ---- the variants of an entry, a function and a condition -----------------------------------

/// The entry's `type`, matched exactly by length and bytes; any other value drops the entry,
/// vanilla's error printing the type as written (`schema::tag_with` of [`EntryKind`]).
pub(crate) fn entry_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    let given = member(node, "type").map(as_written).unwrap_or_default();
    let found = <EntryKind as Fields>::VARIANTS
        .iter()
        .find(|name| **name == given);
    if found.is_none() {
        tell(read, node, codes::ENTRY_TYPE.with([&given]));
    }
    found.copied()
}

/// The function's name: the text of `function` after its last `:`, looked up in the registry
/// (`schema::tag_with` of [`FunctionKind`]). The name as jsoncpp's `asString` gives it, which a
/// value that is not a string prints too.
pub(crate) fn function_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    let written = member(node, "function").map(as_written).unwrap_or_default();
    let (_, name) = split_namespace(&written);
    let (engine, ucf) = gates(read);
    let has_carry_over = engine >= CARRY_OVER_SINCE;
    if name == CARRY_OVER && has_carry_over && !ucf {
        // From 1.26.40 the game looks the name up in its Upcoming Creator Features registry
        // only with the experiment on, and otherwise logs this text and skips the function: a
        // missing toggle.
        tell(read, node, codes::UCF_FUNCTION.with([&written]));
        return None;
    }
    let found = <FunctionKind as Fields>::VARIANTS
        .iter()
        .find(|known| **known == name && (name != CARRY_OVER || has_carry_over));
    if found.is_none() {
        // The function is skipped; vanilla prints the name it was given.
        tell(read, node, codes::UNSUPPORTED_FUNCTION.with([&written]));
    }
    found.copied()
}

/// The text of JSON `function` up to its last `:` is kept for write-back
/// (`schema::after_read` of [`LootFunction`]).
pub(crate) fn function_namespace(function: &mut LootFunction, node: &Node<'_>, _: &mut Read<'_>) {
    let written = member(node, "function").map(as_written).unwrap_or_default();
    function.namespace = split_namespace(&written).0;
}

/// The condition's name, matched exactly (`schema::tag_with` of [`ConditionKind`]). The game
/// reads `operator` (default `==`) for every condition before the name lookup; any other value
/// logs a kept error and drops the condition. An unknown name is skipped with a
/// developer-log line, so what the condition guarded is not gated by it.
pub(crate) fn condition_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<&'static str> {
    if let Some(operator) = member(node, "operator") {
        let shown = operator
            .as_string()
            .map_or_else(|| compact(operator), |text| text.as_str().to_owned());
        if !matches!(shown.as_str(), "==" | "equals" | "!=" | "not") {
            tell(read, node, codes::CONDITION_OPERATOR.with([&shown]));
            return None;
        }
    }
    let name = member(node, "condition")
        .map(as_written)
        .unwrap_or_default();
    // The 20th condition of the game's map; engines before 1.26.30 have only 19, so an older
    // engine does not know the name.
    let found = <ConditionKind as Fields>::VARIANTS.iter().find(|known| {
        **known == name && (name != "biome_has_tag" || gates(read).0 >= BIOME_HAS_TAG_SINCE)
    });
    if found.is_none() {
        let mut finding =
            Diagnostic::info(codes::UNKNOWN_CONDITION.with([&name])).at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    found.copied()
}

// ---- the parameters of a function ------------------------------------------------------------

/// The rules across the members of a function's parameters (`schema::after_read` of
/// [`FunctionKind`]).
pub(crate) fn function_rules(kind: &mut FunctionKind, node: &Node<'_>, read: &mut Read<'_>) {
    match kind {
        FunctionKind::SetData(_) | FunctionKind::RandomAuxValue(_) => {
            if member(node, "data").is_none() && member(node, "values").is_none() {
                // A content-log warning of the game's; the function is kept with range 0..0.
                let mut finding =
                    Diagnostic::warn(Code::from(&codes::SET_DATA_KEY)).at(node.path().clone());
                finding.location.span = node.span();
                read.push(finding);
            }
        }
        FunctionKind::SetBannerDetails {
            banner_type,
            patterns,
            ..
        } => {
            let Some(value) = member(node, "patterns") else {
                return;
            };
            let at = Node::new(value, node.path().child("patterns"));
            if banner_type.unwrap_or_default() != BannerType::Default {
                // A banner that is not a default one takes no patterns: they are ignored.
                tell(read, &at, &codes::BANNER_NON_DEFAULT);
                *patterns = None;
                return;
            }
            *patterns = Some(banner_patterns(&at, read));
        }
        _ => {}
    }
}

/// The patterns of a default banner: an array or a single object, at most [`MAX_PATTERNS`]; the
/// ones past it are dropped after vanilla's error.
fn banner_patterns(node: &Node<'_>, read: &mut Read<'_>) -> Vec<BannerPattern> {
    let count = node
        .value()
        .as_array()
        .map_or(1, |array| array.as_slice().len());
    if count > MAX_PATTERNS {
        tell(
            read,
            node,
            codes::BANNER_MAX.with([&MAX_PATTERNS.to_string(), &count.to_string()]),
        );
    }
    let mut seen = 0;
    one_or_many(node, read, |element, read| {
        seen += 1;
        if seen > MAX_PATTERNS {
            return None;
        }
        banner_pattern(element, read)
    })
}

/// One banner pattern: a pattern without a name is dropped with vanilla's error (absent, `null`
/// and `""` all read as the empty string, which no pattern has); one whose name is another JSON
/// type is a wrong type; every other name is kept as written and not checked.
fn banner_pattern(node: &Node<'_>, read: &mut Read<'_>) -> Option<BannerPattern> {
    match member(node, "pattern") {
        Some(name) if name.as_string().is_some_and(|text| !text.is_empty()) => {
            BannerPattern::from_json(node, read)
        }
        Some(name) if !name.is_string() && !name.is_null() => {
            read.wrong_type(&Node::new(name, node.path().child("pattern")));
            None
        }
        _ => {
            tell(read, node, codes::BANNER_PATTERN.with([""]));
            None
        }
    }
}

/// `specific_enchants.enchants`, one element: an id (level 1) or `{id, level}`. An id that is
/// not an enabled enchantment's is dropped; vanilla says nothing, so the finding is the crate's
/// own.
fn specific_enchant(node: &Node<'_>, read: &mut Read<'_>) -> Option<SpecificEnchant> {
    let enchant = if let Some(id) = node.value().as_string() {
        SpecificEnchant::new(id.as_str())
    } else if let Some(object) = node.value().as_object() {
        if let Some(id) = object.get("id").filter(|id| !id.is_string()) {
            read.wrong_type(&Node::new(id, node.path().child("id")));
            return None;
        }
        SpecificEnchant::from_json(node, read)?
    } else {
        read.wrong_type(node);
        return None;
    };
    if read
        .load_context()
        .is_some_and(|cx| cx.built_ins().is_enchantment(&enchant.id))
    {
        return Some(enchant);
    }
    tell_info(read, node, &core_codes::UNRESOLVED_REFERENCE);
    None
}

#[allow(clippy::unnecessary_wraps)] // a member reader: `None` is "read as absent"
pub(crate) fn specific_enchants(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<Vec<SpecificEnchant>> {
    Some(one_or_many(node, read, specific_enchant))
}

#[allow(clippy::unnecessary_wraps)] // a member reader: `None` is "read as absent"
pub(crate) fn stew_effects(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<StewEffect>> {
    Some(one_or_many(node, read, StewEffect::from_json))
}

/// `enchant_book_for_trading.enchantments`: an array of options; `null` is none. Each finding
/// keeps the function; vanilla logs errors, which are kept errors since the table loads.
pub(crate) fn enchant_options(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<EnchantOption>> {
    if node.value().is_null() {
        return None;
    }
    if !node.value().is_array() {
        // A member that is no array gives no options.
        tell(read, node, &codes::ENCHANT_ARRAY);
        return None;
    }
    read_list(node, read, ElementFailure::Drop, |element, read| {
        let written = member(element, "name");
        let known = written.and_then(Value::as_string).is_some_and(|name| {
            read.load_context()
                .is_some_and(|cx| cx.built_ins().is_enchantment(name.as_str()))
        });
        if !known {
            // No enchantment has that name (by the context's provider: the game's tables or a
            // host): the option is dropped with an error-level line of the game's content log
            // and the table is kept: a kept error.
            let shown = written.map(as_written).unwrap_or_default();
            tell(read, element, codes::ENCHANT_NAME.with([&shown]));
            return None;
        }
        EnchantOption::from_json(element, read)
    })
}

/// `carry_over_block_entity_data.dynamic_properties`: a bool; anything else logs vanilla's
/// error and keeps `false`: a kept error, since the function and the table load.
pub(crate) fn dynamic_properties(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let flag = node.value().as_bool();
    if flag.is_none() {
        tell(read, node, &codes::DYNAMIC_PROPERTIES);
    }
    flag
}

/// A banner type: a name or 0..1; any other value is vanilla's error and keeps the default.
impl FromJson for BannerType {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        let found = match value.as_string() {
            Some(name) => match name.as_str() {
                "default" => Some(Self::Default),
                "illager_captain" => Some(Self::IllagerCaptain),
                _ => None,
            },
            None => match whole_number(value) {
                Some(0) => Some(Self::Default),
                Some(1) => Some(Self::IllagerCaptain),
                _ => None,
            },
        };
        if found.is_none() {
            tell(read, node, codes::BANNER_TYPE.with([&styled(value)]));
        }
        found
    }
}

/// An item colour: a colour name or 0..15; any other value is vanilla's error and reads white.
impl FromJson for ItemColor {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let found = item_color(node.value());
        if found.is_none() {
            tell(
                read,
                node,
                codes::BANNER_BASE_COLOR.with([&styled(node.value())]),
            );
        }
        found
    }
}

/// A banner pattern's `color`: as a banner's `base_color`, under the pattern's own line.
pub(crate) fn pattern_color(node: &Node<'_>, read: &mut Read<'_>) -> Option<ItemColor> {
    let found = item_color(node.value());
    if found.is_none() {
        tell(
            read,
            node,
            codes::BANNER_PATTERN_COLOR.with([&styled(node.value())]),
        );
    }
    found
}

/// The integer a JSON number is, when it is an integer token.
fn whole_number(value: &Value) -> Option<i64> {
    value.as_number().filter(|n| !n.is_float())?.to_i64()
}

/// An item colour: a colour name or 0..15.
fn item_color(value: &Value) -> Option<ItemColor> {
    if let Some(name) = value.as_string() {
        return ItemColor::from_name(name.as_str());
    }
    usize::try_from(whole_number(value)?)
        .ok()
        .and_then(|i| ItemColor::ALL.get(i).copied())
}

#[cfg(test)]
mod root_tests {
    use super::{PrepareError, RootError, read_root};

    #[test]
    fn read_root_reports_bad_input_as_typed_errors() {
        assert!(matches!(
            read_root(&[0xff, b'{']),
            Err(RootError::Text(PrepareError::NotUtf8(_)))
        ));
        assert!(matches!(read_root(b"{\"a\":"), Err(RootError::Json(_))));
        assert!(matches!(read_root(b"[1]"), Err(RootError::NotObject)));
        let deep = "[".repeat(2000);
        assert!(matches!(
            read_root(deep.as_bytes()),
            Err(RootError::Text(PrepareError::TooDeep(_)))
        ));
    }

    #[test]
    fn read_root_reports_too_deep_before_bad_utf8_after_a_bom() {
        let mut bytes = b"\xEF\xBB\xBF".to_vec();
        bytes.extend("[".repeat(2000).bytes());
        bytes.push(0xff);
        assert!(matches!(
            read_root(&bytes),
            Err(RootError::Text(PrepareError::TooDeep(_)))
        ));
    }

    #[test]
    fn read_root_error_text_is_the_readers_message() {
        let error = read_root(b"[1]").unwrap_err();
        assert_eq!(error.to_string(), "the root is not an object");
    }
}
