//! The legacy block document (`[1.16.0, 1.26.20)`): what vanilla's legacy slice sees, JSON
//! members whose shapes its fixers rewrite version by version before the one hand-written parser
//! reads them. One payload serves every legacy version; each `vX_Y_Z::Block` is it at that
//! version (`Legacy<BlockPayload, X, Y, Z>`).

use crate::codes;
use bedrock_addon_core::diagnostic::CodeDef;
use bedrock_addon_core::diagnostic::Severity;
use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::parse::{Options, SpanKind, members_of, parse};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, FromJson, Schema, read_each};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ops;
pub(crate) use bedrock_addon_core::versioning::ops::{number, rename_key, same, string, to_object};
use bedrock_addon_core::versioning::{LegacyPayload, NullHandling, ParseCx, read_payload_with};

/// A block document below 1.26.20: the payload's members as JSON, in the shapes of the version
/// of the struct that holds it (`Legacy<BlockPayload, A, B, C>`).
///
/// `description`, the `components` of the base and of each permutation, and the legacy `events`
/// are held as written; each upgrade step rewrites them as vanilla's JSON fixer of that version
/// does. The 1.26.20 step parses the result into [`Block`](crate::Block). The read refuses the
/// shapes the game's legacy reader refuses, each in its own words, and keeps the members it does
/// not read without a word.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent", schema::molang_with = no_molang)]
pub struct BlockPayload {
    /// `description`: `identifier`, `menu_category`, `states`, `traits`, … as written.
    #[facet(
        schema::read_with = description,
        schema::wrong_type = codes::MEMBER_MISSING,
        schema::missing = codes::MEMBER_MISSING
    )]
    pub description: IndexMap<String, Value>,
    /// The base `components`, in file order; a duplicate key keeps its first value.
    #[facet(
        default,
        schema::read_with = components,
        schema::wrong_type = codes::COMPONENTS_OBJECT
    )]
    pub components: IndexMap<String, Value>,
    /// `permutations`.
    #[facet(default, schema::read_with = permutations)]
    pub permutations: Vec<LegacyPermutation>,
    /// The legacy `events` object, as written.
    pub events: Option<Value>,
    /// The names of the 1.21.60 `minecraft:custom_components` list, in order (from 1.21.90).
    #[facet(default, schema::read_with = strings)]
    pub ordered_v1_custom_components: Vec<String>,
    /// The components a step inserted, as `(section, name)` (`None`: the base); not JSON.
    #[facet(skip)]
    pub synthesized: Vec<(Option<usize>, String)>,
    /// The section whose `minecraft:creative_category` component the 1.19.10 step made
    /// `description.menu_category` of (`Some(None)`: the base; `Some(Some(i))`: permutation
    /// `i`; `None`: the file has the member itself, or none), so that the step that judges
    /// the category tells of it at the member the file has; not JSON.
    #[facet(skip)]
    pub category_from: Option<Option<usize>>,
}

/// One entry of a legacy document's `permutations`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent", schema::molang_with = no_molang)]
pub struct LegacyPermutation {
    /// `condition`, as written.
    pub condition: Option<Value>,
    /// `components`, in file order.
    #[facet(default, schema::read_with = permutation_components)]
    pub components: IndexMap<String, Value>,
    /// The names of the permutation's 1.21.60 `custom_components` list (from 1.21.90).
    #[facet(default, schema::read_with = strings)]
    pub ordered_v1_custom_components: Vec<String>,
}

/// The legacy members are JSON; their Molang is read when the 1.26.20 step parses them.
fn no_molang<T>(_: &mut T, _: &mut MolangVisitor<'_>) {}

impl BlockPayload {
    /// Every component section: the base first, then each permutation's, as `(index, map)`.
    pub(crate) fn sections_mut(
        &mut self,
    ) -> impl Iterator<Item = (Option<usize>, &mut IndexMap<String, Value>)> {
        std::iter::once((None, &mut self.components)).chain(
            self.permutations
                .iter_mut()
                .enumerate()
                .map(|(i, p)| (Some(i), &mut p.components)),
        )
    }

    /// Whether any section holds `name`.
    pub(crate) fn has_anywhere(&self, name: &str) -> bool {
        self.components.contains_key(name)
            || self
                .permutations
                .iter()
                .any(|p| p.components.contains_key(name))
    }

    /// The game's component rename: `old` becomes `new` in place in every section that
    /// holds it, mapping the value with `map`. When `new` already exists somewhere and `old`
    /// does too, the converted older-named component wins: `new` is removed from every section
    /// with "Two versions of the same component found…" (`DuplicateComponentVersion`).
    pub(crate) fn rename_component(
        &mut self,
        old: &str,
        new: &str,
        mut map: impl FnMut(Value) -> Value,
        d: &mut Diagnostics,
    ) {
        if !self.has_anywhere(old) {
            return;
        }
        // The newer-named component of the file goes, told at each section that has it
        // (before the move is recorded: it is about the member the file has there).
        let mut duplicates = Vec::new();
        for (index, section) in self.sections_mut() {
            if section.shift_remove(new).is_some() {
                duplicates.push(index);
            }
        }
        // The game logs this at the content log's error level when the rename removes the
        // component and goes on with the rename: a kept error.
        for index in duplicates {
            d.push_kept_error(
                Diagnostic::of(core_codes::DUPLICATE_COMPONENT_VERSION.with([old, new, new, old]))
                    .at(component_path(index, new))
                    .phase(Phase::Upgrade),
            );
        }
        let mut moved = Vec::new();
        for (index, section) in self.sections_mut() {
            if section.contains_key(old) {
                moved.push(index);
            }
            rename_key(section, old, new, &mut map);
        }
        // What is told of the component from here on is told at the name the file has.
        for index in moved {
            d.moved(component_path(index, old), component_path(index, new));
        }
    }

    /// [`BlockPayload::rename_component`] where the value becomes the member `member` of the new
    /// component (`destroy_time: 3` → `destructible_by_mining: {seconds_to_destroy: 3}`).
    pub(crate) fn wrap_component(
        &mut self,
        old: &str,
        new: &str,
        member: &str,
        d: &mut Diagnostics,
    ) {
        let sections: Vec<Option<usize>> = self
            .sections_mut()
            .filter(|(_, section)| section.contains_key(old))
            .map(|(index, _)| index)
            .collect();
        self.rename_component(old, new, |v| super::ops::single(member, v), d);
        for index in sections {
            let mut inner = component_path(index, new);
            inner.push(member);
            d.moved(component_path(index, old), inner);
        }
    }
}

/// A member of the file that a step replaces by what it makes of another, as the game's own
/// fixer does without a word: `DroppedField`, severity INFO: the game says nothing, so the finding is the crate's own.
pub(crate) fn replaced(at: &JsonPath, d: &mut Diagnostics) {
    ops::note(d, Code::from(&core_codes::DROPPED_FIELD), at.clone());
}

/// Renames the member `old` of the component `name` (held in `members`) to `new`: block's
/// rename, core's [`ops::rename_member_in_place`] (H1: a `null` old is absent and goes, the
/// renamed member stands where `old` stood); a `new` the file has with another value is told
/// as replaced, and the move is recorded.
pub(crate) fn rename_member(
    members: &mut IndexMap<String, Value>,
    section: Option<usize>,
    name: &str,
    old: &str,
    new: &str,
    d: &mut Diagnostics,
) {
    let Some(renamed) = ops::rename_member_in_place(members, old, new) else {
        return;
    };
    let mut to = component_path(section, name);
    to.push(new);
    if renamed.replaced {
        replaced(&to, d);
    }
    let mut from = component_path(section, name);
    from.push(old);
    d.moved(from, to);
}

/// The JSON path of component `name` in section `section` (`None`: the base).
pub(crate) fn component_path(section: Option<usize>, name: &str) -> JsonPath {
    let mut path = JsonPath::new().child("minecraft:block");
    if let Some(index) = section {
        path.push("permutations");
        path.push_index(index);
    }
    path.push("components");
    path.push(name);
    path
}

/// A step's input check failed: " -> minecraft:block ->
/// components -> %s:  invalid component", then "Failed to upgrade to version '%s' from document
/// version '%s'" and "Final json version '%s' failed to upgrade from document version '%s'";
/// the block is not loaded.
pub(crate) fn input_check_failed(
    name: &str,
    to: FormatVersion,
    original: FormatVersion,
    d: &mut Diagnostics,
) {
    let at = component_path(None, name);
    let (to, original) = (to.to_string(), original.to_string());
    d.push(
        Diagnostic::of(codes::INVALID_COMPONENT.with([name]))
            .at(at.clone())
            .phase(Phase::Upgrade),
    );
    d.push(
        Diagnostic::of(codes::UPGRADE_FAILED.with([&to, &original]))
            .at(at.clone())
            .phase(Phase::Upgrade),
    );
    d.push(
        Diagnostic::of(codes::FINAL_UPGRADE_FAILED.with([&to, &original]))
            .at(at.clone())
            .phase(Phase::Upgrade),
    );
}

/// A JSON object as an ordered map.
pub(crate) fn object(value: &Value) -> Option<IndexMap<String, Value>> {
    value.as_object().map(ops::members)
}

// ---- reading -----------------------------------------------------------------------------

/// A legacy document with its payload under the key: the payload, `format_version` (the
/// struct's version, read by the envelope) and the root members the legacy reader ignores with
/// no log, which the crate tells as an information and keeps.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep", schema::molang_with = no_molang)]
struct LegacyDocument {
    /// `minecraft:block`.
    #[facet(rename = "minecraft:block", schema::wrong_type = codes::MEMBER_MISSING)]
    block: BlockPayload,
    /// `format_version`, read by the envelope.
    #[facet(rename = "format_version")]
    _version: Option<Value>,
}

/// The read of a legacy document: a `null` is a value like any other, which the member's read
/// takes (kept as written) or refuses in its own words.
fn legacy_read(read: Read<'_>) -> Read<'_> {
    read.null(NullHandling::AsValue)
}

/// Each legacy struct is the payload at its version.
impl LegacyPayload for BlockPayload {
    /// Reads the whole document (the raw-text path below 1.26.20) or the payload alone
    /// (as the step-file harness hands it); every legacy version reads it the same way.
    fn parse(
        text: JsonSlice<'_>,
        _: FormatVersion,
        cx: &ParseCx<'_>,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        let Some(root) = members_of(text) else {
            d.push(Diagnostic::of(&codes::MEMBER_MISSING).at(cx.at.clone()));
            return None;
        };
        if root.contains_key("minecraft:block") {
            let document: LegacyDocument = read_payload_with(text, cx, d, |read| {
                legacy_read(read).unknown_severity(Severity::Info)
            })?;
            return Some(document.block);
        }
        // A document without the payload key is read as the payload itself; its unknown
        // members are kept where it is written, below the payload key.
        let mut found = Diagnostics::default();
        let block = read_payload_with(text, cx, &mut found, legacy_read);
        found
            .unknown_mut()
            .rebase(&cx.at, &cx.at.child("minecraft:block"));
        d.append(found);
        block
    }

    /// The whole document: `format_version` (the struct's version), the payload and the other
    /// root members.
    fn write(&self, version: FormatVersion) -> Result<Value, EncodeError> {
        let mut root = facet_value::VObject::new();
        root.insert("format_version", version.to_string());
        root.insert("minecraft:block", self.payload_json());
        // The unknown members are the document's, written from its table.
        Ok(root.into())
    }
}

/// `description`: an object whose `states`, when written, is one too ("Block description states
/// must be an object", then "Failed to parse block '…'": the block is not loaded).
fn description(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Value>> {
    read.object(node)?;
    let description = object(node.value())?;
    if description.get("states").is_some_and(|s| !s.is_object()) {
        read.push(Diagnostic::of(&codes::STATES_OBJECT).at(node.path().child("states")));
        let identifier = description
            .get("identifier")
            .and_then(string)
            .unwrap_or_default();
        read.refuse_with(
            Diagnostic::of(codes::PARSE_BLOCK_FAILED.with([identifier])).at(node.path().clone()),
        );
        return None;
    }
    Some(description)
}

/// The base's `components`: an object ([`first_of_twice`]).
fn components(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Value>> {
    read.object(node)?;
    first_of_twice(node, read)
}

/// A permutation's `components`: an object ([`first_of_twice`]); of another type it makes no
/// permutation the game takes, told at the permutation.
fn permutation_components(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Value>> {
    if !node.value().is_object() {
        let mut at = node.path().clone();
        at.pop();
        read.refuse_with(Diagnostic::of(&codes::PERMUTATION_OBJECT).at(at));
        return None;
    }
    first_of_twice(node, read)
}

/// The members of the `components` object `node`, keeping the first of duplicate keys with
/// "Duplicate component {} found. Please remove one of the duplicates." (a kept error, before
/// the first step; vanilla rule 11): the envelope leaves every copy of a component in the text.
fn first_of_twice(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Value>> {
    // A node without its text has only the last copy of each.
    let Some(members) = node.text().and_then(|text| object_members(text.text)) else {
        return object(node.value());
    };
    let mut components = IndexMap::new();
    for (key, raw) in members {
        if components.contains_key(&key) {
            // The game logs a duplicate component at the content log's error level and goes
            // on: a kept error.
            read.push(
                Diagnostic::of(codes::DUPLICATE_COMPONENT.with([key.as_str()]))
                    .at(node.path().child(&key))
                    .into_kept(),
            );
            continue;
        }
        if let Ok(value) = bedrock_addon_core::json::from_str_jsonc(raw) {
            components.insert(key, value);
        }
    }
    Some(components)
}

/// `permutations`: an array of objects, each with an object `components`; anything else makes
/// no block the game loads, told in the game's two lines (one, for a permutation without its
/// `components`, after the permutation's members).
fn permutations(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<LegacyPermutation>> {
    let refuse = |read: &mut Read<'_>, lines: [&'static CodeDef<0>; 2], at: &JsonPath| {
        read.push(Diagnostic::of(lines[0]).at(at.clone()));
        read.refuse_with(Diagnostic::of(lines[1]).at(at.clone()));
    };
    if !node.value().is_array() {
        let lines = [
            &codes::PERMUTATIONS_ARRAY,
            &codes::PERMUTATIONS_ARRAY_OBJECT,
        ];
        refuse(read, lines, node.path());
        return None;
    }
    read_each(node, read, |entry, read| {
        if !entry.value().is_object() {
            let lines = [&codes::PERMUTATION_OBJECT, &codes::PERMUTATIONS_OBJECTS];
            refuse(read, lines, entry.path());
            return None;
        }
        let permutation = LegacyPermutation::from_json(entry, read)?;
        if !entry
            .value()
            .as_object()
            .is_some_and(|entry| entry.contains_key("components"))
        {
            read.refuse_with(Diagnostic::of(&codes::PERMUTATION_OBJECT).at(entry.path().clone()));
            return None;
        }
        Some(permutation)
    })
}

/// A list of names (`ordered_v1_custom_components`): a value that is no array is the empty
/// list, an element that is no string is left out; each is told where it stands (`misread`).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `read_with` signature: no list is refused"
)]
fn strings(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let Some(list) = node.value().as_array() else {
        misread(node.path(), read.findings_mut());
        return Some(Vec::new());
    };
    Some(strings_told(list, node.path(), read.findings_mut()))
}

impl BlockPayload {
    /// The payload as JSON.
    pub(crate) fn payload_json(&self) -> Value {
        let mut object = facet_value::VObject::new();
        object.insert("description", to_object(&self.description));
        object.insert("components", to_object(&self.components));
        if !self.permutations.is_empty() {
            let permutations: facet_value::VArray = self
                .permutations
                .iter()
                .map(LegacyPermutation::write)
                .collect();
            object.insert("permutations", permutations);
        }
        if let Some(events) = &self.events {
            object.insert("events", events.clone());
        }
        if !self.ordered_v1_custom_components.is_empty() {
            object.insert(
                "ordered_v1_custom_components",
                strings_value(&self.ordered_v1_custom_components),
            );
        }
        object.into()
    }
}

impl LegacyPermutation {
    /// The permutation as JSON.
    fn write(&self) -> Value {
        let mut object = facet_value::VObject::new();
        if let Some(condition) = &self.condition {
            object.insert("condition", condition.clone());
        }
        object.insert("components", to_object(&self.components));
        if !self.ordered_v1_custom_components.is_empty() {
            object.insert(
                "ordered_v1_custom_components",
                strings_value(&self.ordered_v1_custom_components),
            );
        }
        object.into()
    }
}

/// A list of strings as a JSON array.
pub(crate) fn strings_value(strings: &[String]) -> Value {
    let array: facet_value::VArray = strings.iter().map(String::as_str).collect();
    array.into()
}

/// A member a step or the legacy reader reads as its fallback because it has another JSON
/// type than the one converted (a list that is no array, a name that is no string): the value
/// is what it always was, and the member is told (`InvalidType`, an information of the
/// upgrade and the crate's own: the game converts only the well-formed shapes).
pub(crate) fn misread(at: &JsonPath, d: &mut Diagnostics) {
    d.push(
        Diagnostic::of(&core_codes::INVALID_TYPE)
            .severity(Severity::Info)
            .at(at.clone())
            .phase(Phase::Upgrade),
    );
}

/// The strings of `list`, which stands at `at`; an element that is no string is left out and
/// told at its index (`misread`).
pub(crate) fn strings_told(
    list: &facet_value::VArray,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Vec<String> {
    let mut strings = Vec::new();
    for (index, element) in list.iter().enumerate() {
        if let Some(text) = string(element) {
            strings.push(text.to_owned());
        } else {
            let at = at.index(index);
            misread(&at, d);
        }
    }
    strings
}

// ---- the members of an object in document order, duplicates kept ------------------------

/// The members of the object `text` as `(key, raw value text)`, in document order with
/// duplicates kept (a value holds the last of a repeated key; the span tree lists every copy).
/// `None` when `text` is not an object.
pub(crate) fn object_members(text: &str) -> Option<Vec<(String, &str)>> {
    let parsed = parse(text, Options::default()).ok()?;
    let root = parsed.spans.root();
    (root.kind() == SpanKind::Object).then(|| {
        root.members()
            .map(|member| (member.key.into_owned(), member.value.text().text))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scanner_keeps_duplicate_keys_in_document_order() {
        let members =
            object_members(r#"{ "a": 1, /* c */ "b": {"x": [1, "]"]}, "a": 2, }"#).unwrap();
        let keys: Vec<_> = members.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["a", "b", "a"]);
        assert_eq!(members[1].1, r#"{"x": [1, "]"]}"#);
        assert_eq!(members[2].1, "2");
    }

    #[test]
    fn the_scanner_refuses_a_non_object() {
        assert!(object_members("[1]").is_none());
    }
}
