//! Reading the material files from JSON values: one pack's copy leniently, the wrapper gate,
//! and the typed read of a list and of a material map with the client's rules.

use std::collections::HashMap;

use facet_value::VObject;
use indexmap::{IndexMap, IndexSet};

use super::file::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, SamplerState,
    StencilFace, StencilRefOverride, Variant, VertexField, base_name,
};
use super::list::{ListEntry, MaterialList, MaterialListKind};
use super::values::{
    BLEND_TARGETS, COMPARISON_FUNCS, FACE_KEYS, IGNORED_RENDER_STATE, IS_ANIMATED_TEXTURE,
    MSAA_SUPPORTS, PLAIN_KEYS, PRIMITIVE_MODES, SAMPLER_KEYS, SAMPLER_SLOTS, SHADER_STAGES,
    STENCIL_OPS, TEXTURE_FILTERS, TEXTURE_FORMATS, TEXTURE_WRAPS, VARIANT_MISSPELLINGS,
    VERTEX_FIELDS, render_state_bit,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value, check_depth, last_wins, strip_bom};
use bedrock_addon_core::version::SemVersion;

/// The client's text for two keys of one copy with the same name before the first `:`; the
/// arguments are the name, the file and the two keys.
pub(super) const DEFINED_TWICE: &str = "Material \"{}\" is defined twice in \"{}\" as \"{}\" and \"{}\". You cannot redefine a material.";

/// The client's text for the render state it names and ignores.
pub(super) const STATE_IGNORED: &str = "Render state {} is ignored and will have no effect";
/// The client's text for a render state it does not know.
pub(super) const STATE_UNKNOWN: &str =
    "Trying to use a non-existing render state {} for this material!";

/// The client's text for a list entry's `tag`; the argument is the key.
pub(super) const KEY_REMOVED: &str = "key \"{}\" is no longer supported, please remove";

/// The wrapper's member, and the member of it that holds the version.
pub(super) const MATERIALS: &str = "materials";

/// The path `inner` of a material map as the file has it: below the wrapper's member when the
/// map is `wrapped` in one. A finding names its place in the file that was written.
pub(super) fn path_in_file(inner: &JsonPath, wrapped: bool) -> JsonPath {
    let at = if wrapped {
        JsonPath::new().child(MATERIALS)
    } else {
        JsonPath::new()
    };
    at.join(inner)
}

/// Moves the findings `found` of a material map into `out`, each at its path in the file.
pub(super) fn report_in_file(found: Diagnostics, wrapped: bool, out: &mut Diagnostics) {
    for mut finding in found {
        finding.location.json_path = path_in_file(&finding.location.json_path, wrapped);
        out.push(finding);
    }
}
/// The wrapper's version member.
const VERSION: &str = "version";

/// One pack's copy of a list or a material file, as read.
pub(super) struct Layer {
    /// The root value, every number finite.
    pub(super) root: Value,
    /// Where a number past the range of a double was written, in the order of the file: it
    /// is read as the largest finite double of its sign.
    pub(super) clamped: IndexSet<JsonPath>,
    /// The finding for content after the root value, which is not read.
    pub(super) trailing: Option<Diagnostic>,
}

impl Layer {
    /// Reports each clamped number where the file has it: an information of the crate's own.
    pub(super) fn report_clamped(&self, out: &mut Diagnostics) {
        out.extend(self.trailing.clone());
        for at in &self.clamped {
            let code = DiagnosticCode::InvalidValue { text: None };
            out.push(Diagnostic::info(code).at(at.clone()));
        }
    }
}

/// Replaces every number of `value` that is not finite (`1e999` reads as an infinity) by the
/// largest finite double of its sign, noting where: a value kept as written must be writable.
fn clamp_numbers(value: &mut Value, at: &mut JsonPath, clamped: &mut IndexSet<JsonPath>) {
    if let Some(number) = value.as_number() {
        let wide = number.to_f64_lossy();
        if !wide.is_finite() {
            let finite = if wide < 0.0 { f64::MIN } else { f64::MAX };
            *value = Value::from(finite);
            clamped.insert(at.clone());
        }
    } else if let Some(array) = value.as_array_mut() {
        for (index, element) in array.iter_mut().enumerate() {
            at.push_index(index);
            clamp_numbers(element, at, clamped);
            at.pop();
        }
    } else if let Some(object) = value.as_object_mut() {
        for (key, member) in object.iter_mut() {
            at.push(key.as_str());
            clamp_numbers(member, at, clamped);
            at.pop();
        }
    }
}

/// Reads one pack's copy of a list or a material file: a BOM is skipped, comments are allowed,
/// of a member written twice the last stands, and a number past the doubles is clamped. The
/// error is the reader's message.
pub(super) fn read_copy(bytes: &[u8]) -> Result<Layer, String> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|error| error.to_string())?;
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    // jsoncpp keeps the last of two equal keys, so the text is cleaned before anything reads it.
    let mut root = bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(text))
        .map_err(|error| error.to_string())?;
    let mut clamped = IndexSet::new();
    clamp_numbers(&mut root, &mut JsonPath::new(), &mut clamped);
    let trailing = bedrock_addon_core::json::trailing_content_finding(
        bedrock_addon_core::json::JsonSlice::new(text),
    );
    Ok(Layer {
        root,
        clamped,
        trailing,
    })
}

/// One copy of a material file past the wrapper gate.
#[derive(Debug)]
pub(super) struct Unwrapped {
    /// The material map: the wrapper's content without `version`, or the root itself.
    pub(super) map: VObject,
    /// The wrapper's version, when the client unwraps the copy.
    pub(super) wrapped: Option<SemVersion>,
    /// The `materials` member of a wrapper the client does not unwrap, taken out of the map.
    pub(super) dropped: Option<Value>,
}

/// The version of a wrapper the client unwraps: `materials` is the root's only member, it is
/// not empty, and its `version` is `1.0.0` (build metadata aside).
///
/// The comparison looks at the numbers, the pre-release text and the any flags; a missing or
/// empty `version` is read as 0.0.0.
fn gate(root_members: usize, inner: &VObject) -> Option<SemVersion> {
    // An empty `materials` has no version and fails below like any other.
    if root_members != 1 {
        return None;
    }
    let text = inner.get(VERSION)?.as_string()?.as_str();
    let version = SemVersion::parse(text)?;
    let exact = (version.major, version.minor, version.patch) == (1, 0, 0)
        && version.pre_release.is_empty()
        && !version.any;
    exact.then_some(version)
}

/// `object` without its member `key`, the others in their order.
fn without(object: &VObject, key: &str) -> VObject {
    let mut rest = VObject::with_capacity(object.len());
    for (name, value) in object {
        if name.as_str() != key {
            rest.insert(name.as_str(), value.clone());
        }
    }
    rest
}

/// Applies the wrapper gate to the root of one copy. A `materials` object the client does not
/// unwrap is taken out of the map with an information: the client would silently read the root as
/// the material map and create one material named `materials`.
pub(super) fn unwrap_copy(root: VObject, out: &mut Diagnostics) -> Unwrapped {
    let Some(inner) = root.get(MATERIALS).and_then(Value::as_object) else {
        return Unwrapped {
            map: root,
            wrapped: None,
            dropped: None,
        };
    };
    if let Some(version) = gate(root.len(), inner) {
        return Unwrapped {
            map: without(inner, VERSION),
            wrapped: Some(version),
            dropped: None,
        };
    }
    let at = JsonPath::new().child(MATERIALS).child(VERSION);
    let code = DiagnosticCode::UnsupportedVersion { text: None };
    out.push(Diagnostic::info(code).at(at));
    Unwrapped {
        dropped: root.get(MATERIALS).cloned(),
        map: without(&root, MATERIALS),
        wrapped: None,
    }
}

/// Reports each key of `map` whose name before the first `:` an earlier key has; `file`
/// is the path the client's text names. The keys are taken bytewise, as the client's sorted
/// map holds them. Both keys stay in the document, so the finding is a kept error.
pub(super) fn check_duplicates(map: &VObject, file: &str, out: &mut Diagnostics) {
    let mut keys: Vec<&str> = map.keys().map(facet_value::VString::as_str).collect();
    keys.sort_unstable();
    let mut first: HashMap<&str, &str> = HashMap::with_capacity(keys.len());
    for key in keys {
        let name = base_name(key);
        let Some(earlier) = first.get(name) else {
            first.insert(name, key);
            continue;
        };
        let text = VanillaText::Formatted {
            template: DEFINED_TWICE,
            args: vec![
                name.to_owned(),
                file.to_owned(),
                (*earlier).to_owned(),
                key.to_owned(),
            ],
        };
        let at = JsonPath::new().child(key);
        let code = DiagnosticCode::DuplicateIdentifier { text: Some(text) };
        // The game logs this at the content log's error level and goes on with both keys: a kept
        // error.
        out.push_kept_error(Diagnostic::error(code).at(at));
    }
}

/// A material map read into its typed form.
pub(super) struct ReadFile {
    /// The file.
    pub(super) file: MaterialFile,
    /// Per material, in order: whether it carries a value the client cannot load, so that no
    /// material is created from it.
    pub(super) refused: Vec<bool>,
}

/// The findings of one read, each at its place in the document.
struct Reader<'a> {
    out: &'a mut Diagnostics,
    path: JsonPath,
    /// Whether the number at a path of the value being read was written past the range of a
    /// double: it was clamped and reported where it is written ([`Layer::report_clamped`]),
    /// and is not reported again for being out of a member's range.
    clamped: &'a dyn Fn(&JsonPath) -> bool,
    /// Whether the material being read carries a keyword the client cannot load.
    refused: bool,
    /// Whether the members being read are ones the client never looks at (a `-variants` body,
    /// a prefixed list its plain key hides): their content is read for write-back and checked
    /// for its JSON types only.
    unread: bool,
}

/// Which of the three plain list keys a material writes as an array.
#[derive(Clone, Copy)]
struct Plain {
    states: bool,
    defines: bool,
    samplers: bool,
}

/// Whether the client skips the prefixed list `key` of a material that has the `plain` keys.
fn hidden(rules: KeyRules, key: &str, plain: Plain) -> bool {
    match (rules, key) {
        (KeyRules::Versioned, "+states" | "-states") => plain.states,
        (KeyRules::Versioned, "+defines" | "-defines") | (KeyRules::Legacy, "+defines") => {
            plain.defines
        }
        (KeyRules::Versioned, "+samplerStates") => plain.samplers,
        (KeyRules::Legacy, "+states" | "-states" | "-defines" | "+samplerStates") => true,
        _ => false,
    }
}

/// The number as a whole number from 0 up, when it is one: an integer, or a float without a
/// fraction (`3.0`) in the range of a `u32`.
fn whole(number: &facet_value::VNumber) -> Option<u64> {
    number.to_u64().or_else(|| {
        let float = number.to_f64_lossy();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "checked to be whole and in range"
        )]
        (float.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&float))
            .then_some(float as u64)
    })
}

/// `key` without a `+` or `-` in front, when it has one.
fn unprefixed(key: &str) -> Option<&str> {
    key.strip_prefix('+').or_else(|| key.strip_prefix('-'))
}

impl Reader<'_> {
    /// A finding at the current path.
    fn report(&mut self, code: DiagnosticCode, severity: Severity) {
        self.out.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(self.path.clone()),
        );
    }

    /// The game's content-log error at the current path after which the material loads: a
    /// kept error.
    fn report_kept(&mut self, code: DiagnosticCode) {
        self.out
            .push_kept_error(Diagnostic::error(code).at(self.path.clone()));
    }

    /// A finding at member `key` of the current path.
    fn report_at(&mut self, key: &str, code: DiagnosticCode, severity: Severity) {
        self.path.push(key);
        self.report(code, severity);
        self.path.pop();
    }

    /// Whether the number at member `key` was clamped to the doubles, and reported for it.
    fn clamped_at(&mut self, key: &str) -> bool {
        self.path.push(key);
        let clamped = (self.clamped)(&self.path);
        self.path.pop();
        clamped
    }

    /// A value of the wrong JSON type at member `key`: an information of the crate's own.
    fn wrong_type(&mut self, key: &str) {
        self.report_at(
            key,
            DiagnosticCode::InvalidType { text: None },
            Severity::Info,
        );
    }

    /// Reports a key the client does not read under this spelling.
    fn unrecognized(&mut self, key: &str) {
        self.report_at(
            key,
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Info,
        );
    }

    /// Runs `read` one array element down.
    fn within_index<T>(&mut self, index: usize, read: impl FnOnce(&mut Self) -> T) -> T {
        self.path.push_index(index);
        let value = read(self);
        self.path.pop();
        value
    }

    /// Runs `read` one member down.
    fn within<T>(&mut self, key: &str, read: impl FnOnce(&mut Self) -> T) -> T {
        self.path.push(key);
        let value = read(self);
        self.path.pop();
        value
    }

    /// The string at member `key`.
    fn string(&mut self, key: &str, value: &Value) -> Option<String> {
        let text = value.as_string().map(|text| text.as_str().to_owned());
        if text.is_none() {
            self.wrong_type(key);
        }
        text
    }

    /// The keyword at member `key`, kept as written. One that is not in `table` is an error
    /// and refuses the material: the client reads through the end of its map for it.
    fn keyword(&mut self, key: &str, value: &Value, table: &[&str]) -> Option<String> {
        let text = self.string(key, value)?;
        if !self.unread && !table.contains(&text.as_str()) {
            self.refused = true;
            self.report_at(
                key,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Error,
            );
        }
        Some(text)
    }

    /// The whole number from 0 to the largest `u32` at member `key` (`asUInt`). A number
    /// past the doubles is out of range like any other, and has its finding from the clamp.
    fn unsigned(&mut self, key: &str, value: &Value) -> Option<u32> {
        let Some(number) = value.as_number() else {
            self.wrong_type(key);
            return None;
        };
        let read = whole(number).and_then(|whole| u32::try_from(whole).ok());
        let float = number.to_f64_lossy();
        if number.is_float() && float == 0.0 && float.is_sign_negative() {
            // An unsigned member holds no sign: the file's `-0.0` is held, and written, as
            // 0. The crate notes it; the client reads an unsigned 0 either way.
            self.report_at(
                key,
                DiagnosticCode::ValueRewritten { text: None },
                Severity::Info,
            );
        }
        if read.is_none() && !self.clamped_at(key) {
            self.report_at(
                key,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
            );
        }
        read
    }

    /// The float at member `key`. A number past the range of a float is the largest finite
    /// float of its sign, with an information: never an infinity the writer would refuse.
    fn float(&mut self, key: &str, value: &Value) -> Option<f32> {
        let Some(number) = value.as_number() else {
            self.wrong_type(key);
            return None;
        };
        let wide = number.to_f64_lossy();
        #[allow(
            clippy::cast_possible_truncation,
            reason = "rounded on purpose, then checked"
        )]
        let narrow = wide as f32;
        if narrow.is_finite() {
            return Some(narrow);
        }
        // A number past the doubles has the clamp's finding; the largest double as written
        // has none yet.
        if !self.clamped_at(key) {
            self.report_at(
                key,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
            );
        }
        Some(if wide < 0.0 { f32::MIN } else { f32::MAX })
    }

    /// `stencilRefOverride` at member `key`, which the client's `asBool` reads: a boolean, or
    /// a number (vanilla's own `ui.material` writes `0`), kept as it is written.
    fn stencil_override(&mut self, key: &str, value: &Value) -> Option<StencilRefOverride> {
        let number = value
            .as_number()
            .map(|number| StencilRefOverride::Number(number.to_f64_lossy()));
        let read = value.as_bool().map(StencilRefOverride::Flag).or(number);
        if read.is_none() {
            self.wrong_type(key);
        }
        read
    }

    /// The array at member `key`.
    fn array<'v>(&mut self, key: &str, value: &'v Value) -> Option<&'v [Value]> {
        let array = value.as_array().map(facet_value::VArray::as_slice);
        if array.is_none() {
            self.wrong_type(key);
        }
        array
    }

    /// A list of strings at member `key`; an entry that is no string is dropped with a
    /// information. `states` says that the entries are render states: an ignored one is
    /// named and an unknown one reported with the client's text.
    fn strings(&mut self, key: &str, value: &Value, states: bool) -> Option<Vec<String>> {
        let array = self.array(key, value)?;
        Some(self.within(key, |reader| {
            let mut list = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                let Some(text) = entry.as_string() else {
                    let code = DiagnosticCode::InvalidType { text: None };
                    reader.within_index(index, |reader| reader.report(code, Severity::Info));
                    continue;
                };
                let text = text.as_str();
                if states && !reader.unread {
                    reader.within_index(index, |reader| reader.state(text));
                }
                list.push(text.to_owned());
            }
            list
        }))
    }

    /// Checks the render state `name` at the current path: the ignored state is named, an
    /// unknown one is reported with the client's text.
    fn state(&mut self, name: &str) {
        let text = |template| VanillaText::Formatted {
            template,
            args: vec![name.to_owned()],
        };
        if name == IGNORED_RENDER_STATE {
            let code = DiagnosticCode::NoEffect {
                text: Some(text(STATE_IGNORED)),
            };
            self.report(code, Severity::Info);
        } else if render_state_bit(name).is_none() {
            // The game logs this at the content log's error level; the state is ignored and the
            // material loads: a kept error.
            let code = DiagnosticCode::InvalidValue {
                text: Some(text(STATE_UNKNOWN)),
            };
            self.report_kept(code);
        }
    }

    /// A stencil face at member `key`.
    fn face(&mut self, key: &str, value: &Value) -> Option<StencilFace> {
        let Some(object) = value.as_object() else {
            self.wrong_type(key);
            return None;
        };
        Some(self.within(key, |reader| {
            let mut face = StencilFace::default();
            for (name, value) in object {
                let name = name.as_str();
                if value.is_null() {
                    face.extra.insert(name.to_owned(), value.clone());
                    continue;
                }
                let read = match name {
                    "stencilFunc" => reader
                        .keyword(name, value, &COMPARISON_FUNCS)
                        .map(|read| face.stencil_func = Some(read)),
                    "stencilFailOp" => reader
                        .keyword(name, value, &STENCIL_OPS)
                        .map(|read| face.stencil_fail_op = Some(read)),
                    "stencilDepthFailOp" => reader
                        .keyword(name, value, &STENCIL_OPS)
                        .map(|read| face.stencil_depth_fail_op = Some(read)),
                    "stencilPassOp" => reader
                        .keyword(name, value, &STENCIL_OPS)
                        .map(|read| face.stencil_pass_op = Some(read)),
                    _ => {
                        if !reader.unread
                            && unprefixed(name).is_some_and(|k| FACE_KEYS.contains(&k))
                        {
                            reader.unrecognized(name);
                        }
                        None
                    }
                };
                if read.is_none() {
                    face.extra.insert(name.to_owned(), value.clone());
                }
            }
            face
        }))
    }

    /// One sampler entry at the current path. `applied` says that the list is one the client
    /// applies, so that an entry it skips is reported and the keywords of the others are
    /// looked up; a legacy material reads the filter and the wrap only.
    fn sampler(&mut self, object: &VObject, rules: KeyRules, applied: bool) -> SamplerState {
        let mut sampler = SamplerState::default();
        // The client reads the index first and skips the entry without reading the rest when
        // it is absent, null or out of range.
        let index = object
            .get("samplerIndex")
            .and_then(Value::as_number)
            .and_then(whole);
        let skipped = index.is_none_or(|index| index >= u64::from(SAMPLER_SLOTS));
        let outer = self.unread;
        for (name, value) in object {
            let name = name.as_str();
            if value.is_null() {
                sampler.extra.insert(name.to_owned(), value.clone());
                continue;
            }
            self.unread = outer || !applied || skipped;
            let legacy_unread = rules == KeyRules::Legacy;
            let read = match name {
                "samplerIndex" => self.unsigned(name, value).map(|read| {
                    sampler.sampler_index = Some(read);
                }),
                "textureFilter" => self
                    .keyword(name, value, &TEXTURE_FILTERS)
                    .map(|read| sampler.texture_filter = Some(read)),
                "textureWrap" => self
                    .keyword(name, value, &TEXTURE_WRAPS)
                    .map(|read| sampler.texture_wrap = Some(read)),
                "shaderStage" => {
                    self.unread |= legacy_unread;
                    self.keyword(name, value, &SHADER_STAGES)
                        .map(|read| sampler.shader_stage = Some(read))
                }
                "comparisonFunc" => {
                    self.unread |= legacy_unread;
                    self.keyword(name, value, &COMPARISON_FUNCS)
                        .map(|read| sampler.comparison_func = Some(read))
                }
                _ => {
                    self.unread = outer || !applied;
                    if !self.unread && unprefixed(name).is_some_and(|k| SAMPLER_KEYS.contains(&k)) {
                        self.unrecognized(name);
                    }
                    None
                }
            };
            if read.is_none() {
                sampler.extra.insert(name.to_owned(), value.clone());
            }
        }
        self.unread = outer;
        if applied && !outer {
            match sampler.sampler_index {
                // The client skips the entry and says nothing.
                None => self.report(
                    DiagnosticCode::MissingRequiredField { text: None },
                    Severity::Info,
                ),
                // An assertion in a versioned material, silence in a legacy one.
                Some(index) if index >= SAMPLER_SLOTS => self.report_at(
                    "samplerIndex",
                    DiagnosticCode::InvalidValue { text: None },
                    Severity::Info,
                ),
                Some(_) => {}
            }
        }
        sampler
    }

    /// A list of sampler entries at member `key`; an element that is no object is dropped.
    fn samplers(
        &mut self,
        key: &str,
        value: &Value,
        rules: KeyRules,
        applied: bool,
    ) -> Option<Vec<SamplerState>> {
        let array = self.array(key, value)?;
        Some(self.within(key, |reader| {
            let mut list = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                reader.within_index(index, |reader| match entry.as_object() {
                    Some(object) => list.push(reader.sampler(object, rules, applied)),
                    None => {
                        reader.report(DiagnosticCode::InvalidType { text: None }, Severity::Info);
                    }
                });
            }
            list
        }))
    }

    /// `vertexFields` at member `key`; an element that is no object is dropped.
    fn vertex_fields(&mut self, key: &str, value: &Value) -> Option<Vec<VertexField>> {
        let array = self.array(key, value)?;
        Some(self.within(key, |reader| {
            let mut list = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                reader.within_index(index, |reader| {
                    let Some(object) = entry.as_object() else {
                        let code = DiagnosticCode::InvalidType { text: None };
                        reader.report(code, Severity::Info);
                        return;
                    };
                    let mut field = VertexField::default();
                    for (name, value) in object {
                        let name = name.as_str();
                        let read = (name == "field" && !value.is_null())
                            .then(|| reader.keyword(name, value, &VERTEX_FIELDS))
                            .flatten()
                            .map(|read| field.field = Some(read));
                        if read.is_none() {
                            field.extra.insert(name.to_owned(), value.clone());
                        }
                    }
                    list.push(field);
                });
            }
            list
        }))
    }

    /// `renderTargetFormats` at member `key`: an array of arrays of format names. An element
    /// that is no array is skipped; an unknown name is kept with an information (the
    /// client reads it as an unknown texture format).
    fn formats(&mut self, key: &str, value: &Value) -> Option<Vec<Vec<String>>> {
        let array = self.array(key, value)?;
        Some(self.within(key, |reader| {
            let mut lists = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                reader.within_index(index, |reader| {
                    let Some(names) = entry.as_array() else {
                        let code = DiagnosticCode::InvalidType { text: None };
                        reader.report(code, Severity::Info);
                        return;
                    };
                    let mut list = Vec::with_capacity(names.len());
                    for (at, name) in names.as_slice().iter().enumerate() {
                        let Some(name) = name.as_string() else {
                            let code = DiagnosticCode::InvalidType { text: None };
                            reader.within_index(at, |reader| reader.report(code, Severity::Info));
                            continue;
                        };
                        if !TEXTURE_FORMATS.contains(&name.as_str()) {
                            let code = DiagnosticCode::InvalidValue { text: None };
                            reader.within_index(at, |reader| reader.report(code, Severity::Info));
                        }
                        list.push(name.as_str().to_owned());
                    }
                    lists.push(list);
                });
            }
            lists
        }))
    }

    /// A `variants` array at member `key` (anything else is not read). An element that
    /// is no object, and a body that is none, are dropped with an information.
    fn variants(&mut self, key: &str, value: &Value, rules: KeyRules) -> Option<Vec<Variant>> {
        let array = self.array(key, value)?;
        Some(self.within(key, |reader| {
            let mut list = Vec::with_capacity(array.len());
            for (index, entry) in array.iter().enumerate() {
                reader.within_index(index, |reader| {
                    let Some(object) = entry.as_object() else {
                        let code = DiagnosticCode::InvalidType { text: None };
                        reader.report(code, Severity::Info);
                        return;
                    };
                    let mut bodies = IndexMap::with_capacity(object.len());
                    for (name, body) in object {
                        let name = name.as_str();
                        match body.as_object() {
                            Some(body) => {
                                let body = reader.within(name, |reader| reader.def(body, rules));
                                bodies.insert(name.to_owned(), body);
                            }
                            None => reader.wrong_type(name),
                        }
                    }
                    list.push(Variant(bodies));
                });
            }
            list
        }))
    }

    /// One of the three spellings of a list member whose content `read` reads: a prefixed one
    /// the client skips is reported and read as unread.
    fn listed<T>(
        &mut self,
        key: &str,
        skipped: bool,
        read: impl FnOnce(&mut Self) -> Option<Vec<T>>,
    ) -> Option<Vec<T>> {
        let outer = self.unread;
        self.unread = outer || skipped;
        let list = read(self);
        self.unread = outer;
        if list.is_some() && skipped && !outer {
            self.report_at(key, DiagnosticCode::NoEffect { text: None }, Severity::Info);
        }
        list
    }

    /// One material, or one variant body, read with `rules`.
    #[allow(clippy::too_many_lines, reason = "one arm per member of the format")]
    fn def(&mut self, object: &VObject, rules: KeyRules) -> MaterialDef {
        let mut def = MaterialDef::default();
        let is_array = |key: &str| object.get(key).is_some_and(Value::is_array);
        let plain = Plain {
            states: is_array("states"),
            defines: is_array("defines"),
            samplers: is_array("samplerStates"),
        };
        for (key, value) in object {
            let key = key.as_str();
            // A `null` is absent to every reader, and to the merge it means "no change".
            if value.is_null() {
                def.extra.insert(key.to_owned(), value.clone());
                continue;
            }
            let skipped = hidden(rules, key, plain);
            let read = match key {
                "states" | "+states" | "-states" => self
                    .listed(key, skipped, |reader| reader.strings(key, value, true))
                    .map(|read| *spelling(&mut def.states, key) = Some(read)),
                "defines" | "+defines" | "-defines" => self
                    .listed(key, skipped, |reader| reader.strings(key, value, false))
                    .map(|read| *spelling(&mut def.defines, key) = Some(read)),
                "samplerStates" | "+samplerStates" => self
                    .listed(key, skipped, |reader| {
                        reader.samplers(key, value, rules, true)
                    })
                    .map(|read| *spelling(&mut def.sampler_states, key) = Some(read)),
                "-samplerStates" => {
                    // The client asserts on this key and then ignores it, in both modes.
                    if !self.unread {
                        self.unrecognized(key);
                    }
                    self.samplers(key, value, rules, false)
                        .map(|read| def.sampler_states.sub = Some(read))
                }
                "variants" | "+variants" => self
                    .variants(key, value, rules)
                    .map(|read| *spelling(&mut def.variants, key) = Some(read)),
                "-variants" => {
                    // The client never looks this key up.
                    if !self.unread {
                        self.unrecognized(key);
                    }
                    let outer = std::mem::replace(&mut self.unread, true);
                    let read = self.variants(key, value, rules);
                    self.unread = outer;
                    read.map(|read| def.variants.sub = Some(read))
                }
                "vertexShader" => self
                    .string(key, value)
                    .map(|read| def.vertex_shader = Some(read)),
                "fragmentShader" => self
                    .string(key, value)
                    .map(|read| def.fragment_shader = Some(read)),
                "geometryShader" => self
                    .string(key, value)
                    .map(|read| def.geometry_shader = Some(read)),
                "depthFunc" => self
                    .keyword(key, value, &COMPARISON_FUNCS)
                    .map(|read| def.depth_func = Some(read)),
                "frontFace" => self
                    .face(key, value)
                    .map(|read| def.front_face = Some(read)),
                "backFace" => self.face(key, value).map(|read| def.back_face = Some(read)),
                "stencilRefOverride" => self
                    .stencil_override(key, value)
                    .map(|read| def.stencil_ref_override = Some(read)),
                "stencilRef" => self
                    .unsigned(key, value)
                    .map(|read| def.stencil_ref = Some(read)),
                "stencilReadMask" => self
                    .unsigned(key, value)
                    .map(|read| def.stencil_read_mask = Some(read)),
                "stencilWriteMask" => self
                    .unsigned(key, value)
                    .map(|read| def.stencil_write_mask = Some(read)),
                "blendSrc" => self
                    .keyword(key, value, &BLEND_TARGETS)
                    .map(|read| def.blend_src = Some(read)),
                "blendDst" => self
                    .keyword(key, value, &BLEND_TARGETS)
                    .map(|read| def.blend_dst = Some(read)),
                "alphaSrc" => self
                    .keyword(key, value, &BLEND_TARGETS)
                    .map(|read| def.alpha_src = Some(read)),
                "alphaDst" => self
                    .keyword(key, value, &BLEND_TARGETS)
                    .map(|read| def.alpha_dst = Some(read)),
                "vertexFields" => self
                    .vertex_fields(key, value)
                    .map(|read| def.vertex_fields = Some(read)),
                "depthBias" => self
                    .float(key, value)
                    .map(|read| def.depth_bias = Some(read)),
                "slopeScaledDepthBias" => self
                    .float(key, value)
                    .map(|read| def.slope_scaled_depth_bias = Some(read)),
                "msaaSupport" => self
                    .keyword(key, value, &MSAA_SUPPORTS)
                    .map(|read| def.msaa_support = Some(read)),
                "primitiveMode" => self
                    .keyword(key, value, &PRIMITIVE_MODES)
                    .map(|read| def.primitive_mode = Some(read)),
                "renderTargetFormats" => self
                    .formats(key, value)
                    .map(|read| def.render_target_formats = Some(read)),
                IS_ANIMATED_TEXTURE => {
                    def.is_animated_texture = Some(value.clone());
                    Some(())
                }
                _ => {
                    self.unknown(key);
                    None
                }
            };
            if read.is_none() {
                def.extra.insert(key.to_owned(), value.clone());
            }
        }
        def
    }

    /// A key no reader looks up: silent, unless it is a prefixed spelling of a key that has
    /// none, or a misspelling of `variants`.
    fn unknown(&mut self, key: &str) {
        if self.unread {
            return;
        }
        let bare = unprefixed(key);
        let prefixed_plain =
            bare.is_some_and(|bare| PLAIN_KEYS.contains(&bare) || bare == IS_ANIMATED_TEXTURE);
        if prefixed_plain || VARIANT_MISSPELLINGS.contains(&bare.unwrap_or(key)) {
            self.unrecognized(key);
        }
    }
}

/// The slot of `listed` that the spelling `key` fills.
fn spelling<'l, T>(listed: &'l mut Listed<T>, key: &str) -> &'l mut Option<Vec<T>> {
    match key.as_bytes().first() {
        Some(b'+') => &mut listed.add,
        Some(b'-') => &mut listed.sub,
        _ => &mut listed.replace,
    }
}

/// Reads the material map `map` of a file whose wrapper version is `wrapped` and whose name is
/// `stem`. `rules` gives the key rules of a key the stack merge worked out; `None` leaves them
/// to the form. `clamped` says of a path of `map` whether the number there was written past
/// the range of a double.
///
/// A root with a string `vertexShader` is one material named `stem`, read with the legacy
/// rules (the client stamps no version on it). A member that is no object is no material: it
/// is kept in the file's `extra`, a `null` silently (the merge reads it as "no change") and
/// anything else with an information.
pub(super) fn read_file(
    map: &VObject,
    wrapped: Option<SemVersion>,
    stem: &str,
    rules: &dyn Fn(&str) -> Option<KeyRules>,
    clamped: &dyn Fn(&JsonPath) -> bool,
    out: &mut Diagnostics,
) -> ReadFile {
    let mut reader = Reader {
        out,
        path: JsonPath::new(),
        clamped,
        refused: false,
        unread: false,
    };
    let mut materials = IndexMap::new();
    let mut extra = Extra::default();
    let mut refused = Vec::new();
    if map.get("vertexShader").is_some_and(Value::is_string) {
        let mut def = reader.def(map, KeyRules::Legacy);
        // Whatever the merge worked out for a key of that name: nothing is stamped on it.
        def.key_rules = rules(stem).map(|_| KeyRules::Legacy);
        materials.insert(MaterialKey::new(stem), def);
        refused.push(reader.refused);
        let form = MaterialFileForm::Single {
            stem: stem.to_owned(),
            wrapper: wrapped,
        };
        let file = MaterialFile {
            form,
            materials,
            extra,
        };
        return ReadFile { file, refused };
    }
    let form = wrapped.map_or(MaterialFileForm::Flat, |version| {
        MaterialFileForm::Wrapped { version }
    });
    for (key, value) in map {
        let key = key.as_str();
        let Some(object) = value.as_object() else {
            if !value.is_null() {
                let code = DiagnosticCode::InvalidType { text: None };
                reader.report_at(key, code, Severity::Info);
            }
            extra.insert(key.to_owned(), value.clone());
            continue;
        };
        let own = rules(key);
        reader.refused = false;
        reader.path.push(key);
        let mut def = reader.def(object, own.unwrap_or(form.key_rules()));
        reader.path.pop();
        def.key_rules = own;
        materials.insert(MaterialKey::parse(key), def);
        refused.push(reader.refused);
    }
    let file = MaterialFile {
        form,
        materials,
        extra,
    };
    ReadFile { file, refused }
}

/// Reads a list file from its root value; `None` when the root is no array.
///
/// An element that is no object, and an entry without a string `path`, are dropped with a
/// warning. `tag` is reported with the client's text and the entry loads.
pub(super) fn read_list(
    kind: MaterialListKind,
    root: &Value,
    out: &mut Diagnostics,
) -> Option<MaterialList> {
    read_list_placed(kind, root, out).map(|(list, _)| list)
}

/// [`read_list`], with the index each entry the list holds has in the file: the reader drops
/// an element that is no object and one that names no file.
pub(super) fn read_list_placed(
    kind: MaterialListKind,
    root: &Value,
    out: &mut Diagnostics,
) -> Option<(MaterialList, Vec<usize>)> {
    let array = root.as_array()?;
    // A list has no member with a range of its own.
    let mut reader = Reader {
        out,
        path: JsonPath::new(),
        clamped: &|_| false,
        refused: false,
        unread: false,
    };
    let mut entries = Vec::with_capacity(array.len());
    let mut places = Vec::with_capacity(array.len());
    for (index, element) in array.as_slice().iter().enumerate() {
        reader.within_index(index, |reader| {
            let Some(object) = element.as_object() else {
                let code = DiagnosticCode::InvalidType { text: None };
                reader.report(code, Severity::Info);
                return;
            };
            if let Some(entry) = reader.entry(object) {
                entries.push(entry);
                places.push(index);
            }
        });
    }
    Some((MaterialList { kind, entries }, places))
}

impl Reader<'_> {
    /// One list entry at the current path; `None` when it names no file.
    fn entry(&mut self, object: &VObject) -> Option<ListEntry> {
        let mut entry = ListEntry::default();
        let mut path = None;
        for (key, value) in object {
            let key = key.as_str();
            if key == "tag" {
                // The game logs the removed keyword at the content log's error level and the entry
                // still loads: a kept error.
                let text = VanillaText::Formatted {
                    template: KEY_REMOVED,
                    args: vec![key.to_owned()],
                };
                let code = DiagnosticCode::DeprecatedMember { text: Some(text) };
                self.path.push(key);
                self.report_kept(code);
                self.path.pop();
            }
            let read = match key {
                _ if value.is_null() => None,
                "path" => self.string(key, value).map(|read| path = Some(read)),
                "defines" => self
                    .strings(key, value, false)
                    .map(|read| entry.defines = Some(read)),
                "+defines" => self
                    .strings(key, value, false)
                    .map(|read| entry.add_defines = Some(read)),
                "-defines" => self
                    .strings(key, value, false)
                    .map(|read| entry.sub_defines = Some(read)),
                _ => None,
            };
            if read.is_none() {
                entry.extra.insert(key.to_owned(), value.clone());
            }
        }
        let Some(path) = path else {
            // A path of the wrong type has its own finding.
            if object.get("path").is_none_or(Value::is_null) {
                let code = DiagnosticCode::MissingRequiredField { text: None };
                self.report_at("path", code, Severity::Info);
            }
            return None;
        };
        entry.path = path;
        Some(entry)
    }
}
