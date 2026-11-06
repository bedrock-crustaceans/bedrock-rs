//! The legacy block document (`[1.16.0, 1.26.20)`): what vanilla's legacy slice sees, JSON
//! members whose shapes its fixers rewrite version by version before the one hand-written parser
//! reads them. One payload serves every legacy version; each `vX_Y_Z::Block` is it at that
//! version (`Legacy<BlockPayload, X, Y, Z>`).

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ops;
pub(crate) use bedrock_addon_core::versioning::ops::{number, rename_key, same, string, to_object};
use bedrock_addon_core::versioning::{LegacyPayload, ParseCx};

/// A block document below 1.26.20: the payload's members as JSON, in the shapes of the version
/// of the struct that holds it (`Legacy<BlockPayload, A, B, C>`).
///
/// `description`, the `components` of the base and of each permutation, and the legacy `events`
/// are held as written; each upgrade step rewrites them as vanilla's JSON fixer of that version
/// does. The 1.26.20 step parses the result into [`Block`](crate::Block).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockPayload {
    /// `description`: `identifier`, `menu_category`, `states`, `traits`, … as written.
    pub description: IndexMap<String, Value>,
    /// The base `components`, in file order; a duplicate key keeps its first value.
    pub components: IndexMap<String, Value>,
    /// `permutations`.
    pub permutations: Vec<LegacyPermutation>,
    /// The legacy `events` object, as written.
    pub events: Option<Value>,
    /// The names of the 1.21.60 `minecraft:custom_components` list, in order (from 1.21.90).
    pub ordered_v1_custom_components: Vec<String>,
    /// The components a step inserted, as `(section, name)` (`None`: the base); not JSON.
    pub synthesized: Vec<(Option<usize>, String)>,
    /// The section whose `minecraft:creative_category` component the 1.19.10 step made
    /// `description.menu_category` of (`Some(None)`: the base; `Some(Some(i))`: permutation
    /// `i`; `None`: the file has the member itself, or none), so that the step that judges
    /// the category tells of it at the member the file has; not JSON.
    pub category_from: Option<Option<usize>>,
}

/// One entry of a legacy document's `permutations`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyPermutation {
    /// `condition`, as written.
    pub condition: Option<Value>,
    /// `components`, in file order.
    pub components: IndexMap<String, Value>,
    /// The names of the permutation's 1.21.60 `custom_components` list (from 1.21.90).
    pub ordered_v1_custom_components: Vec<String>,
}

impl MolangFields for BlockPayload {
    /// The legacy members are JSON; their Molang is read when the 1.26.20 step parses them.
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

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
                Diagnostic::error(DiagnosticCode::DuplicateComponentVersion {
                    old: old.to_owned(),
                    new: new.to_owned(),
                })
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
    ops::note(d, DiagnosticCode::DroppedField { text: None }, at.clone());
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

/// A vanilla text with arguments (`Some`, as the codes' `text` fields take it).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the codes' text fields are options"
)]
pub(crate) fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|a| (*a).to_owned()).collect(),
    })
}

/// A fixed vanilla text (`Some`, as the codes' `text` fields take it).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the codes' text fields are options"
)]
pub(crate) fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
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
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: text(
                " -> minecraft:block -> components -> {}:  invalid component",
                &[name],
            ),
        })
        .at(at.clone())
        .phase(Phase::Upgrade),
    );
    d.push(
        Diagnostic::error(DiagnosticCode::DocumentRejected {
            text: text(
                "Failed to upgrade to version '{}' from document version '{}'",
                &[&to, &original],
            ),
        })
        .at(at.clone())
        .phase(Phase::Upgrade),
    );
    d.push(
        Diagnostic::error(DiagnosticCode::DocumentRejected {
            text: text(
                "Final json version '{}' failed to upgrade from document version '{}'",
                &[&to, &original],
            ),
        })
        .at(at.clone())
        .phase(Phase::Upgrade),
    );
}

/// A JSON object as an ordered map.
pub(crate) fn object(value: &Value) -> Option<IndexMap<String, Value>> {
    value.as_object().map(ops::members)
}

// ---- reading -----------------------------------------------------------------------------

/// The legacy parser's structural texts, as the game words them.
const MISSING_REQUIRED_FIELD: &str = " -> minecraft:block:  missing required field";
const COMPONENTS_OBJECT: &str = " -> minecraft:block:  the 'components' member should be an object";
const PERMUTATIONS_ARRAY: &str =
    " -> minecraft:block:  the 'permutations' member should be an array";
/// The texts of the legacy permutation reader.
const PERMUTATIONS_ARRAY_OBJECT: &str = "Permutations requires an array object";
const PERMUTATIONS_OBJECTS: &str = "Permutations requires an array of objects";
/// The text of the legacy description reader.
const STATES_OBJECT: &str = "Block description states must be an object";
const PERMUTATION_OBJECT: &str = " -> minecraft:block -> permutations:  every permutation should be an object containing a 'components' member object";

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
        // A document without the payload key is read as the payload itself; its unknown
        // members are kept where it is written, below the payload key.
        let below = cx.at.child("minecraft:block");
        let Some(root) = object_members(text.text) else {
            return Self::parse_payload(text.text, &cx.at, &below, d);
        };
        let Some(payload) = root
            .iter()
            .find(|(key, _)| key == "minecraft:block")
            .map(|(_, raw)| *raw)
        else {
            return Self::parse_payload(text.text, &cx.at, &below, d);
        };
        let at = cx.at.child("minecraft:block");
        let block = Self::parse_payload(payload, &at, &at, d)?;
        for (key, raw) in root {
            if key == "minecraft:block" || key == "format_version" {
                continue;
            }
            let path = cx.at.child(&key);
            // The legacy reader ignores it with no log, so the crate reports it as an information.
            d.push(Diagnostic::info(DiagnosticCode::UnrecognizedField { text: None }).at(path));
            if let Some(value) = read_value(raw) {
                d.unknown_mut().put(&cx.at, &key, value);
            }
        }
        Some(block)
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

impl BlockPayload {
    /// Reads the payload object `text`, its findings below `base` and its unknown members
    /// below `kept` (the payload's place in the written document).
    fn parse_payload(
        text: &str,
        base: &JsonPath,
        kept: &JsonPath,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        let payload = JsonSlice::new(text);
        let at = |key: Option<&str>| {
            let mut path = base.clone();
            if let Some(key) = key {
                path.push(key);
            }
            path
        };
        let structural = |d: &mut Diagnostics, message: &'static str, path: JsonPath| {
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidType {
                    text: fixed(message),
                })
                .at(path),
            );
        };
        let Some(members) = object_members(payload.text) else {
            structural(d, MISSING_REQUIRED_FIELD, at(None));
            return None;
        };
        let mut block = Self::default();
        let mut found_description = false;
        for (key, raw) in members {
            match key.as_str() {
                "description" => {
                    found_description = true;
                    let Some(description) = read_value(raw).as_ref().and_then(object) else {
                        structural(d, MISSING_REQUIRED_FIELD, at(Some("description")));
                        return None;
                    };
                    // The legacy description reader requires `states` to be an object.
                    if description.get("states").is_some_and(|s| !s.is_object()) {
                        let mut path = at(Some("description"));
                        path.push("states");
                        structural(d, STATES_OBJECT, path);
                        let identifier = description
                            .get("identifier")
                            .and_then(string)
                            .unwrap_or_default()
                            .to_owned();
                        d.push(
                            Diagnostic::error(DiagnosticCode::InvalidType {
                                text: self::text("Failed to parse block '{}' ", &[&identifier]),
                            })
                            .at(at(Some("description"))),
                        );
                        return None;
                    }
                    block.description = description;
                }
                "components" => {
                    let Some(components) = read_components(raw, &at(Some("components")), d) else {
                        structural(d, COMPONENTS_OBJECT, at(Some("components")));
                        return None;
                    };
                    block.components = components;
                }
                "permutations" => {
                    let Some(entries) = array_elements(raw) else {
                        structural(d, PERMUTATIONS_ARRAY, at(Some("permutations")));
                        structural(d, PERMUTATIONS_ARRAY_OBJECT, at(Some("permutations")));
                        return None;
                    };
                    for (index, entry) in entries.into_iter().enumerate() {
                        let mut path = at(Some("permutations"));
                        path.push_index(index);
                        let mut kept_at = kept.child("permutations");
                        kept_at.push_index(index);
                        let Some(permutation) = read_permutation(entry, &path, &kept_at, d) else {
                            structural(d, PERMUTATION_OBJECT, path.clone());
                            if object_members(entry).is_none() {
                                structural(d, PERMUTATIONS_OBJECTS, path);
                            }
                            return None;
                        };
                        block.permutations.push(permutation);
                    }
                }
                "events" => block.events = read_value(raw),
                "ordered_v1_custom_components" => {
                    let at = at(Some("ordered_v1_custom_components"));
                    block.ordered_v1_custom_components = read_strings(raw, &at, d);
                }
                _ => {
                    if let Some(value) = read_value(raw) {
                        d.unknown_mut().put(kept, &key, value);
                    }
                }
            }
        }
        if !found_description {
            structural(d, MISSING_REQUIRED_FIELD, at(Some("description")));
            return None;
        }
        Some(block)
    }

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

/// A permutation object; `None` when it is not an object with an object `components`.
fn read_permutation(
    text: &str,
    at: &JsonPath,
    kept: &JsonPath,
    d: &mut Diagnostics,
) -> Option<LegacyPermutation> {
    let members = object_members(text)?;
    let mut permutation = LegacyPermutation::default();
    let mut has_components = false;
    for (key, raw) in members {
        match key.as_str() {
            "condition" => permutation.condition = read_value(raw),
            "components" => {
                let path = at.child("components");
                permutation.components = read_components(raw, &path, d)?;
                has_components = true;
            }
            "ordered_v1_custom_components" => {
                let path = at.child("ordered_v1_custom_components");
                permutation.ordered_v1_custom_components = read_strings(raw, &path, d);
            }
            _ => {
                if let Some(value) = read_value(raw) {
                    d.unknown_mut().put(kept, &key, value);
                }
            }
        }
    }
    has_components.then_some(permutation)
}

/// A `components` object, keeping the first of duplicate keys with "Duplicate component {}
/// found. Please remove one of the duplicates." (a warning, before the first step; vanilla
/// rule 11). `None` when it is not an object.
fn read_components(
    text: &str,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<IndexMap<String, Value>> {
    let mut components = IndexMap::new();
    for (key, raw) in object_members(text)? {
        if components.contains_key(&key) {
            let path = at.child(&key);
            // The game logs a duplicate component at the content log's error level and goes
            // on: a kept error.
            d.push_kept_error(
                Diagnostic::error(DiagnosticCode::DroppedComponent {
                    text: text_owned(
                        "Duplicate component {} found. Please remove one of the duplicates.",
                        &key,
                    ),
                })
                .at(path),
            );
            continue;
        }
        if let Some(value) = read_value(raw) {
            components.insert(key, value);
        }
    }
    Some(components)
}

/// A text with one argument.
fn text_owned(template: &'static str, arg: &str) -> Option<VanillaText> {
    text(template, &[arg])
}

/// A JSON value from its text.
fn read_value(text: &str) -> Option<Value> {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text).ok()
}

/// A member a step or the legacy reader reads as its fallback because it has another JSON
/// type than the one converted (a list that is no array, a name that is no string): the value
/// is what it always was, and the member is told (`InvalidType`, an information of the
/// upgrade and the crate's own: the game converts only the well-formed shapes).
pub(crate) fn misread(at: &JsonPath, d: &mut Diagnostics) {
    d.push(
        Diagnostic::info(DiagnosticCode::InvalidType { text: None })
            .at(at.clone())
            .phase(Phase::Upgrade),
    );
}

/// The strings of the list at `at`: a value that is no array is the empty list, an element
/// that is no string is left out; each is told where it stands (`misread`).
fn read_strings(text: &str, at: &JsonPath, d: &mut Diagnostics) -> Vec<String> {
    let value = read_value(text);
    let Some(list) = value.as_ref().and_then(Value::as_array) else {
        misread(at, d);
        return Vec::new();
    };
    strings_told(list, at, d)
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

// ---- a JSONC object scanner: members in document order, duplicates kept ------------------

/// The members of the object `text` as `(key, raw value text)`, in document order with
/// duplicates kept (facet keeps only the last value of a duplicate key). `None` when `text` is
/// not an object.
pub(crate) fn object_members(text: &str) -> Option<Vec<(String, &str)>> {
    let bytes = text.as_bytes();
    let mut i = skip_space(bytes, 0);
    if bytes.get(i) != Some(&b'{') {
        return None;
    }
    i += 1;
    let mut members = Vec::new();
    loop {
        i = skip_space(bytes, i);
        match bytes.get(i)? {
            b'}' => return Some(members),
            b',' => {
                i += 1;
                continue;
            }
            b'"' => {}
            _ => return None,
        }
        let key_end = skip_string(bytes, i)?;
        let key: String = bedrock_addon_core::json::from_str(&text[i..key_end]).ok()?;
        i = skip_space(bytes, key_end);
        if bytes.get(i) != Some(&b':') {
            return None;
        }
        i = skip_space(bytes, i + 1);
        let value_end = skip_value(bytes, i)?;
        members.push((key, &text[i..value_end]));
        i = value_end;
    }
}

/// The elements of the array `text` as raw texts; `None` when it is not an array.
pub(crate) fn array_elements(text: &str) -> Option<Vec<&str>> {
    let bytes = text.as_bytes();
    let mut i = skip_space(bytes, 0);
    if bytes.get(i) != Some(&b'[') {
        return None;
    }
    i += 1;
    let mut elements = Vec::new();
    loop {
        i = skip_space(bytes, i);
        match bytes.get(i)? {
            b']' => return Some(elements),
            b',' => {
                i += 1;
                continue;
            }
            _ => {}
        }
        let end = skip_value(bytes, i)?;
        elements.push(&text[i..end]);
        i = end;
    }
}

/// The index after whitespace and comments from `i`.
fn skip_space(bytes: &[u8], mut i: usize) -> usize {
    loop {
        match bytes.get(i) {
            Some(b' ' | b'\t' | b'\r' | b'\n') => i += 1,
            Some(b'/') if bytes.get(i + 1) == Some(&b'/') => {
                while let Some(&b) = bytes.get(i) {
                    if b == b'\n' {
                        break;
                    }
                    i += 1;
                }
            }
            Some(b'/') if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i < bytes.len() && !(bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/')) {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            _ => return i,
        }
    }
}

/// The index after the string starting at `i` (a `"`).
fn skip_string(bytes: &[u8], mut i: usize) -> Option<usize> {
    i += 1;
    while let Some(&b) = bytes.get(i) {
        match b {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

/// The index after the value starting at `i`.
fn skip_value(bytes: &[u8], i: usize) -> Option<usize> {
    match bytes.get(i)? {
        b'"' => skip_string(bytes, i),
        b'{' | b'[' => {
            let mut depth = 0usize;
            let mut j = i;
            while j < bytes.len() {
                match bytes[j] {
                    b'"' => {
                        j = skip_string(bytes, j)?;
                        continue;
                    }
                    b'/' if matches!(bytes.get(j + 1), Some(b'/' | b'*')) => {
                        j = skip_space(bytes, j);
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(j + 1);
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            None
        }
        _ => {
            let mut j = i;
            while let Some(&b) = bytes.get(j) {
                if matches!(b, b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'/') {
                    break;
                }
                j += 1;
            }
            (j > i).then_some(j)
        }
    }
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
        assert_eq!(
            array_elements("[1, {\"a\": 2}]").unwrap(),
            ["1", "{\"a\": 2}"]
        );
    }
}
