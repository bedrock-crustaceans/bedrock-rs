//! Writing one material file or list back. A member the value does not hold is not written,
//! and a value the loader would refuse or read differently is refused.

use facet_value::{VArray, VObject};

use super::file::{
    Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey, SamplerState, StencilFace,
    StencilRefOverride, Variant, VertexField,
};
use super::list::{ListEntry, MaterialList};
use super::read::{MATERIALS, path_in_file, read_list, read_root};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{EncodeError, MAX_JSON_DEPTH, Unrepresentable, Value};

/// A refusal at `at`.
fn refuse(at: &JsonPath, what: Unrepresentable) -> EncodeError {
    EncodeError::NotRepresentable {
        at: at.clone(),
        what,
    }
}

/// Appends the members of `extra`.
fn put_extra(object: &mut VObject, extra: &UnknownMembers) {
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
        && !version.is_any()
}

/// The file as JSON, checked to read back as `file`.
pub(super) fn file_json(file: &MaterialFile) -> Result<Value, EncodeError> {
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
pub(super) fn list_json(list: &MaterialList) -> Result<Value, EncodeError> {
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
