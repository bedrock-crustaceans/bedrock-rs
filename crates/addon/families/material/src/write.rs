//! Reading one material file or list on its own and writing one back: `Content` for
//! [`MaterialFile`] and [`MaterialList`], `Placed` for the list. A member the value does not
//! hold is not written, and a value the loader would refuse or read differently is refused.

use facet_value::{VArray, VObject};

use super::file::{
    KeyRules, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, SamplerState,
    StencilFace, StencilRefOverride, Variant, VertexField,
};
use super::list::{ListEntry, MaterialList, MaterialListKind};
use super::read::{
    MATERIALS, check_duplicates, path_in_file, read_copy, read_file, read_list, report_in_file,
    unwrap_copy,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::{
    Content, Document, Extra, Origin, Placed, Rejected, VersionInfo,
};
use bedrock_addon_core::json::{
    EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value, WriteStyle, write_json,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;

/// The client's text for a copy that is no JSON: the path, then the reader's messages.
pub(super) const PARSE_ERROR: &str = "Error parsing {} :\n {}";

/// The origin of a file read on its own.
fn detached(path: &PackPath) -> Origin {
    Origin::file(PackRef::app_data(), PackSide::Resource, path.clone())
}

/// A document of an unversioned file: no `format_version` is read on any path.
pub(super) fn unversioned<T>(
    value: T,
    origin: Origin,
    mut diagnostics: Diagnostics,
) -> Document<T> {
    diagnostics.locate(&origin);
    let none = FormatVersion::new(0, 0, 0);
    let version = VersionInfo::new(None, false, none, none, false);
    Document::new(value, origin, version, diagnostics)
}

/// A rejection of the file `origin` with its findings.
pub(super) fn rejected(origin: Origin, diagnostics: Diagnostics) -> Rejected {
    Rejected::located(origin, diagnostics)
}

/// The finding for the copy at `path` that is no JSON.
pub(super) fn parse_error(path: &PackPath, message: String) -> Diagnostic {
    let text = VanillaText::Formatted {
        template: PARSE_ERROR,
        args: vec![path.as_str().to_owned(), message.clone()],
    };
    let code = DiagnosticCode::MalformedInput {
        message,
        text: Some(text),
    };
    Diagnostic::error(code)
}

/// The root of a copy that is no object, or of a list that is no array: nothing is read.
pub(super) fn wrong_root() -> Diagnostic {
    let code = DiagnosticCode::InvalidType { text: None };
    Diagnostic::error(code)
}

/// The name of the file at `path` without its directory and its extension.
pub(super) fn stem_of(path: &str) -> &str {
    let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The file in the root object `root` of a file named `stem`, as a stack of one pack reads it:
/// the wrapper gate, then the material map. `rules` are the key rules a merge worked out.
fn read_root(
    root: &VObject,
    stem: &str,
    rules: &dyn Fn(&str) -> Option<KeyRules>,
    out: &mut Diagnostics,
) -> MaterialFile {
    let unwrapped = unwrap_copy(root.clone(), out);
    // A value being written has no number past the doubles.
    let clamped = |_: &JsonPath| false;
    let mut read = read_file(
        &unwrapped.map,
        unwrapped.wrapped,
        stem,
        rules,
        &clamped,
        out,
    );
    if let Some(dropped) = unwrapped.dropped {
        read.file.extra.insert(MATERIALS.to_owned(), dropped);
    }
    read.file
}

impl bedrock_addon_core::__private::Sealed for MaterialFile {}

/// Keyed by the material's key: `Merged::origin` names the highest pack whose copy writes it.
impl bedrock_addon_core::document::Merge for MaterialFile {
    type Key = MaterialKey;
}

impl Content for MaterialFile {
    const NAME: &'static str = "material file";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `.material` file on its own, as a stack of one pack would. Rejected when the
    /// file is no JSON object or a material carries a keyword the client cannot load.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = detached(path);
        let mut diagnostics = Diagnostics::default();
        let copy = match read_copy(bytes) {
            Ok(copy) => copy,
            Err(message) => {
                diagnostics.push(parse_error(path, message));
                return Err(rejected(origin, diagnostics));
            }
        };
        let Some(root) = copy.root.as_object() else {
            diagnostics.push(wrong_root());
            return Err(rejected(origin, diagnostics));
        };
        let stem = stem_of(path.as_str());
        // The gate's finding, the clamped numbers, the duplicates of the map, then the
        // members': each at its path in the file, the wrapper's member included.
        let unwrapped = unwrap_copy(root.clone(), &mut diagnostics);
        copy.report_clamped(&mut diagnostics);
        let wrapped = unwrapped.wrapped.is_some();
        let mut found = Diagnostics::default();
        check_duplicates(&unwrapped.map, path.as_str(), &mut found);
        let no_rules = |_: &str| None;
        let clamped = |inner: &JsonPath| copy.clamped.contains(&path_in_file(inner, wrapped));
        let read = read_file(
            &unwrapped.map,
            unwrapped.wrapped,
            stem,
            &no_rules,
            &clamped,
            &mut found,
        );
        report_in_file(found, wrapped, &mut diagnostics);
        // Each finding gets the span of the value its path names in the text.
        super::span::fill_all(bytes, &mut diagnostics);
        let mut file = read.file;
        if let Some(dropped) = unwrapped.dropped {
            file.extra.insert(MATERIALS.to_owned(), dropped);
        }
        if diagnostics.has_errors() {
            return Err(rejected(origin, diagnostics));
        }
        Ok(unversioned(file, origin, diagnostics))
    }

    /// Writes the file in its form. Refused with `NotRepresentable` and the place when the
    /// text would not read back as this value: a wrapper version other than `1.0.0`, a key
    /// that splits differently, a keyword the loader rejects, a number that is not finite, a
    /// member hidden in `extra` that a reader would take, variants nested deeper than the
    /// loader reads.
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&file_json(self)?, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for MaterialFile {}

impl bedrock_addon_core::__private::Sealed for MaterialList {}

impl Content for MaterialList {
    const NAME: &'static str = "material list";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one list file on its own. The path says which list it is; a path that is none of
    /// the four is read as the common list. Rejected when the file is no JSON array.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = detached(path);
        let mut diagnostics = Diagnostics::default();
        let kind = MaterialListKind::from_path(path.as_str()).unwrap_or(MaterialListKind::Common);
        let copy = match read_copy(bytes) {
            Ok(copy) => copy,
            Err(message) => {
                diagnostics.push(parse_error(path, message));
                return Err(rejected(origin, diagnostics));
            }
        };
        copy.report_clamped(&mut diagnostics);
        let list = read_list(kind, &copy.root, &mut diagnostics);
        super::span::fill_all(bytes, &mut diagnostics);
        if let Some(list) = list {
            return Ok(unversioned(list, origin, diagnostics));
        }
        diagnostics.push(wrong_root());
        Err(rejected(origin, diagnostics))
    }

    /// Writes the list as an array of entries. Refused with `NotRepresentable` when an entry
    /// would read back differently (a member hidden in its `extra` that the reader takes).
    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&list_json(self)?, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for MaterialList {}

impl Placed for MaterialList {
    /// `materials/common.json`, `fancy.json`, `sad.json` or `gameface.json`, by the kind.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(self.kind.path())
    }
}

/// A refusal at `at`.
fn refuse(at: &JsonPath, what: Unrepresentable) -> EncodeError {
    EncodeError::NotRepresentable {
        at: at.clone(),
        what,
    }
}

/// Appends the members of `extra`.
fn put_extra(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

/// Inserts `value` when there is one.
fn put(object: &mut VObject, key: &str, value: Option<impl Into<Value>>) {
    if let Some(value) = value {
        object.insert(key, value);
    }
}

/// An array of strings.
fn strings(list: &[String]) -> Value {
    let mut array = VArray::new();
    for text in list {
        array.push(Value::from(text.as_str()));
    }
    array.into_value()
}

/// The three spellings of a list member, each written when it is there.
fn put_listed<T>(
    object: &mut VObject,
    key: &str,
    listed: &Listed<T>,
    mut write: impl FnMut(&str, &[T]) -> Result<Value, EncodeError>,
) -> Result<(), EncodeError> {
    let spellings = [
        (String::from(key), &listed.replace),
        (format!("+{key}"), &listed.add),
        (format!("-{key}"), &listed.sub),
    ];
    for (key, list) in spellings {
        if let Some(list) = list {
            let value = write(&key, list)?;
            object.insert(key, value);
        }
    }
    Ok(())
}

/// A stencil face.
fn face_json(face: &StencilFace) -> Value {
    let mut object = VObject::new();
    put(&mut object, "stencilFunc", face.stencil_func.as_deref());
    put(
        &mut object,
        "stencilFailOp",
        face.stencil_fail_op.as_deref(),
    );
    put(
        &mut object,
        "stencilDepthFailOp",
        face.stencil_depth_fail_op.as_deref(),
    );
    put(
        &mut object,
        "stencilPassOp",
        face.stencil_pass_op.as_deref(),
    );
    put_extra(&mut object, &face.extra);
    object.into_value()
}

/// A list of sampler entries.
fn samplers_json(samplers: &[SamplerState]) -> Value {
    let mut array = VArray::new();
    for sampler in samplers {
        let mut object = VObject::new();
        put(
            &mut object,
            "samplerIndex",
            sampler.sampler_index.map(i64::from),
        );
        put(
            &mut object,
            "textureFilter",
            sampler.texture_filter.as_deref(),
        );
        put(&mut object, "textureWrap", sampler.texture_wrap.as_deref());
        put(&mut object, "shaderStage", sampler.shader_stage.as_deref());
        put(
            &mut object,
            "comparisonFunc",
            sampler.comparison_func.as_deref(),
        );
        put_extra(&mut object, &sampler.extra);
        array.push(object.into_value());
    }
    array.into_value()
}

/// `vertexFields`.
fn fields_json(fields: &[VertexField]) -> Value {
    let mut array = VArray::new();
    for field in fields {
        let mut object = VObject::new();
        put(&mut object, "field", field.field.as_deref());
        put_extra(&mut object, &field.extra);
        array.push(object.into_value());
    }
    array.into_value()
}

/// A float as JSON: a whole number as an integer, anything else in the shortest decimal that
/// reads back to the same float.
fn float_json(at: &JsonPath, key: &str, value: f32) -> Result<Value, EncodeError> {
    let what = if value.is_nan() {
        Some(Unrepresentable::NaN)
    } else if value == f32::INFINITY {
        Some(Unrepresentable::Infinity)
    } else if value == f32::NEG_INFINITY {
        Some(Unrepresentable::NegativeInfinity)
    } else {
        None
    };
    if let Some(what) = what {
        let at = at.child(key);
        return Err(refuse(&at, what));
    }
    if value == 0.0 && value.is_sign_negative() {
        // The zero of the file keeps its sign: an integer has none.
        return Ok(Value::from(-0.0_f64));
    }
    // 2^53: below it every whole float is an exact integer.
    #[allow(clippy::cast_possible_truncation, reason = "whole and below 2^53")]
    if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
        return Ok(Value::from(value as i64));
    }
    let shortest = value.to_string().parse().unwrap_or(f64::from(value));
    Ok(Value::from(shortest))
}

/// A number as JSON: a whole one as an integer, anything else as the double. One that is not
/// finite has no JSON spelling and is refused.
fn number_json(at: &JsonPath, key: &str, value: f64) -> Result<Value, EncodeError> {
    let what = if value.is_nan() {
        Some(Unrepresentable::NaN)
    } else if value == f64::INFINITY {
        Some(Unrepresentable::Infinity)
    } else if value == f64::NEG_INFINITY {
        Some(Unrepresentable::NegativeInfinity)
    } else {
        None
    };
    if let Some(what) = what {
        let at = at.child(key);
        return Err(refuse(&at, what));
    }
    if value == 0.0 && value.is_sign_negative() {
        return Ok(Value::from(-0.0_f64));
    }
    // 2^53: below it every whole double is an exact integer.
    #[allow(clippy::cast_possible_truncation, reason = "whole and below 2^53")]
    if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
        return Ok(Value::from(value as i64));
    }
    Ok(Value::from(value))
}

/// Writes materials and variant bodies, keeping track of the place.
struct Writer {
    path: JsonPath,
}

impl Writer {
    /// A `variants` array; `depth` is its JSON nesting depth.
    fn variants(
        &mut self,
        key: &str,
        list: &[Variant],
        depth: usize,
    ) -> Result<Value, EncodeError> {
        self.path.push(key);
        let mut array = VArray::new();
        for (index, variant) in list.iter().enumerate() {
            self.path.push_index(index);
            let mut object = VObject::new();
            for (name, body) in &variant.0 {
                self.path.push(name);
                let body = self.def(body, depth + 2)?;
                self.path.pop();
                object.insert(name.as_str(), body.into_value());
            }
            self.path.pop();
            array.push(object.into_value());
        }
        self.path.pop();
        Ok(array.into_value())
    }

    /// One material or variant body, an object at the JSON nesting depth `depth`. Deeper than
    /// the loader reads is refused before anything recurses further.
    fn def(&mut self, def: &MaterialDef, depth: usize) -> Result<VObject, EncodeError> {
        if depth > MAX_JSON_DEPTH {
            return Err(refuse(&self.path, Unrepresentable::Unsupported));
        }
        let mut object = VObject::new();
        put_listed(&mut object, "states", &def.states, |_, list| {
            Ok(strings(list))
        })?;
        put_listed(&mut object, "defines", &def.defines, |_, list| {
            Ok(strings(list))
        })?;
        put(&mut object, "vertexShader", def.vertex_shader.as_deref());
        put(
            &mut object,
            "fragmentShader",
            def.fragment_shader.as_deref(),
        );
        put(
            &mut object,
            "geometryShader",
            def.geometry_shader.as_deref(),
        );
        put(
            &mut object,
            "vertexFields",
            def.vertex_fields.as_deref().map(fields_json),
        );
        put_listed(&mut object, "variants", &def.variants, |key, list| {
            self.variants(key, list, depth + 1)
        })?;
        put(&mut object, "depthFunc", def.depth_func.as_deref());
        put(
            &mut object,
            "frontFace",
            def.front_face.as_ref().map(face_json),
        );
        put(
            &mut object,
            "backFace",
            def.back_face.as_ref().map(face_json),
        );
        match def.stencil_ref_override {
            Some(StencilRefOverride::Flag(on)) => {
                object.insert("stencilRefOverride", on);
            }
            Some(StencilRefOverride::Number(number)) => {
                let key = "stencilRefOverride";
                object.insert(key, number_json(&self.path, key, number)?);
            }
            None => {}
        }
        put(&mut object, "stencilRef", def.stencil_ref.map(i64::from));
        put(
            &mut object,
            "stencilReadMask",
            def.stencil_read_mask.map(i64::from),
        );
        put(
            &mut object,
            "stencilWriteMask",
            def.stencil_write_mask.map(i64::from),
        );
        put(&mut object, "blendSrc", def.blend_src.as_deref());
        put(&mut object, "blendDst", def.blend_dst.as_deref());
        put(&mut object, "alphaSrc", def.alpha_src.as_deref());
        put(&mut object, "alphaDst", def.alpha_dst.as_deref());
        put_listed(
            &mut object,
            "samplerStates",
            &def.sampler_states,
            |_, list| Ok(samplers_json(list)),
        )?;
        if let Some(bias) = def.depth_bias {
            object.insert("depthBias", float_json(&self.path, "depthBias", bias)?);
        }
        if let Some(bias) = def.slope_scaled_depth_bias {
            let key = "slopeScaledDepthBias";
            object.insert(key, float_json(&self.path, key, bias)?);
        }
        put(&mut object, "msaaSupport", def.msaa_support.as_deref());
        put(&mut object, "primitiveMode", def.primitive_mode.as_deref());
        if let Some(formats) = &def.render_target_formats {
            let mut array = VArray::new();
            for list in formats {
                array.push(strings(list));
            }
            object.insert("renderTargetFormats", array.into_value());
        }
        put(
            &mut object,
            "isAnimatedTexture",
            def.is_animated_texture.clone(),
        );
        put_extra(&mut object, &def.extra);
        Ok(object)
    }

    /// The material map of `file` at the JSON nesting depth `depth`.
    fn map(
        &mut self,
        file: &MaterialFile,
        into: &mut VObject,
        depth: usize,
    ) -> Result<(), EncodeError> {
        for (key, def) in &file.materials {
            let text = key.to_string();
            self.path.push(&text);
            // A key that splits at another colon on the way back is another key.
            if MaterialKey::parse(&text) != *key {
                return Err(refuse(&self.path, Unrepresentable::Unsupported));
            }
            let def = self.def(def, depth + 1)?;
            self.path.pop();
            into.insert(text, def.into_value());
        }
        Ok(())
    }
}

/// Whether the client unwraps a wrapper of `version`.
fn passes_gate(version: &bedrock_addon_core::version::SemVersion) -> bool {
    (version.major, version.minor, version.patch) == (1, 0, 0)
        && version.pre_release.is_empty()
        && !version.any
}

/// The file as JSON, checked to read back as `file`.
fn file_json(file: &MaterialFile) -> Result<Value, EncodeError> {
    let mut writer = Writer {
        path: JsonPath::new(),
    };
    let mut root = VObject::new();
    let mut stem = "";
    match &file.form {
        MaterialFileForm::Wrapped { version } => {
            writer.path.push(MATERIALS);
            let at = writer.path.child("version");
            // The wrapper's own member cannot be a material's key.
            let clash = file.materials.contains_key(&MaterialKey::new("version"));
            if !passes_gate(version) || clash {
                return Err(refuse(&at, Unrepresentable::Unsupported));
            }
            let mut inner = VObject::new();
            inner.insert("version", version.to_string());
            writer.map(file, &mut inner, 2)?;
            writer.path.pop();
            // The members of the map that are no material stay in the map.
            put_extra(&mut inner, &file.extra);
            root.insert(MATERIALS, inner.into_value());
        }
        MaterialFileForm::Flat => writer.map(file, &mut root, 1)?,
        MaterialFileForm::Single {
            stem: name,
            wrapper,
        } => {
            stem = name;
            let only = file.materials.get(&MaterialKey::new(name.as_str()));
            let Some(def) = only.filter(|_| file.materials.len() == 1) else {
                return Err(refuse(&writer.path, Unrepresentable::Unsupported));
            };
            if def.vertex_shader.is_none() {
                writer.path.push("vertexShader");
                return Err(refuse(&writer.path, Unrepresentable::Unsupported));
            }
            match wrapper {
                Some(version) => {
                    writer.path.push(MATERIALS);
                    if !passes_gate(version) {
                        writer.path.push("version");
                        return Err(refuse(&writer.path, Unrepresentable::Unsupported));
                    }
                    let mut inner = VObject::new();
                    inner.insert("version", version.to_string());
                    for (key, value) in &writer.def(def, 2)? {
                        inner.insert(key.as_str(), value.clone());
                    }
                    root.insert(MATERIALS, inner.into_value());
                }
                None => root = writer.def(def, 1)?,
            }
        }
    }
    if !matches!(file.form, MaterialFileForm::Wrapped { .. }) {
        put_extra(&mut root, &file.extra);
    }
    verify(file, &root, stem)?;
    Ok(root.into_value())
}

/// Reads `root` back as the loader would and requires it to load as `file`: a keyword the
/// loader rejects, or a member it would read into another place, is refused where it is.
fn verify(file: &MaterialFile, root: &VObject, stem: &str) -> Result<(), EncodeError> {
    let wrapped = matches!(
        file.form,
        MaterialFileForm::Wrapped { .. }
            | MaterialFileForm::Single {
                wrapper: Some(_),
                ..
            }
    );
    let placed = |inner: &JsonPath| path_in_file(inner, wrapped);
    let mut found = Diagnostics::default();
    let mut read = read_root(root, stem, &|_| None, &mut found);
    // The key rules of a merge are not written: a file read on its own has none.
    for (key, def) in &mut read.materials {
        def.key_rules = file.materials.get(key).and_then(|def| def.key_rules);
    }
    // A kept error is a line of the loader's that takes the value: no refusal.
    if let Some(error) = found.iter().find(|finding| finding.rejects()) {
        let at = placed(&error.location.json_path);
        return Err(refuse(&at, Unrepresentable::Unsupported));
    }
    if read == *file {
        return Ok(());
    }
    let mut at = JsonPath::new();
    let differs = file
        .materials
        .iter()
        .find(|(key, def)| read.materials.get(*key) != Some(*def));
    if let Some((key, _)) = differs.filter(|_| read.form == file.form) {
        at.push(&key.to_string());
        at = placed(&at);
    }
    Err(refuse(&at, Unrepresentable::Unsupported))
}

/// One list entry.
fn entry_json(entry: &ListEntry) -> Value {
    let mut object = VObject::new();
    object.insert("path", entry.path.as_str());
    put(
        &mut object,
        "defines",
        entry.defines.as_deref().map(strings),
    );
    put(
        &mut object,
        "+defines",
        entry.add_defines.as_deref().map(strings),
    );
    put(
        &mut object,
        "-defines",
        entry.sub_defines.as_deref().map(strings),
    );
    put_extra(&mut object, &entry.extra);
    object.into_value()
}

/// The list as JSON, checked to read back as `list`.
fn list_json(list: &MaterialList) -> Result<Value, EncodeError> {
    let mut array = VArray::new();
    for entry in &list.entries {
        array.push(entry_json(entry));
    }
    let value = array.into_value();
    let mut found = Diagnostics::default();
    let read = read_list(list.kind, &value, &mut found);
    let read = read.map(|read| read.entries).unwrap_or_default();
    if read == list.entries {
        return Ok(value);
    }
    let mut at = JsonPath::new();
    let differs = list
        .entries
        .iter()
        .zip(&read)
        .position(|(written, read)| written != read);
    if let Some(index) = differs.filter(|_| read.len() == list.entries.len()) {
        at.push_index(index);
    }
    Err(refuse(&at, Unrepresentable::Unsupported))
}
