//! Writing a manifest back in the format it was read in.
//!
//! The value is laid out as a JSON tree in the format's own shape (legacy: `uuid` / `version`
//! arrays, `memory_tier`; format 3: version strings, `memory_performance_tier`), unknown members
//! of a legacy manifest after the known members of their object, and written by core's writer.

use crate::document::UnknownMembers;
use crate::json::{self, EncodeError, Unrepresentable, Value, WriteStyle};
use crate::pack_identity::PackType;
use crate::version::{MinEngineVersion, SemVersion};
use facet_value::{VArray, VNumber, VObject};

use super::read::{json_path, version_array};
use super::{
    Dependency, Header, Manifest, ManifestFormat, Metadata, Module, NativeModuleVersion,
    RelaxedSemVersion, Setting, Subpack,
};

/// Writes `manifest` as JSON text to `out`. On failure nothing is appended.
pub(super) fn write(
    manifest: &Manifest,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    let value = to_value(manifest)?;
    json::write_json(&value, out, style)
}

/// Data this format cannot hold.
fn not_representable(at: &str) -> EncodeError {
    EncodeError::NotRepresentable {
        at: json_path(at),
        what: Unrepresentable::Unsupported,
    }
}

/// A non-finite number.
fn non_finite(at: &str, value: f32) -> EncodeError {
    EncodeError::NotRepresentable {
        at: json_path(at),
        what: if value.is_nan() {
            Unrepresentable::NaN
        } else if value > 0.0 {
            Unrepresentable::Infinity
        } else {
            Unrepresentable::NegativeInfinity
        },
    }
}

/// The manifest's JSON tree.
fn to_value(manifest: &Manifest) -> Result<Value, EncodeError> {
    let v3 = manifest.format == ManifestFormat::V3;
    let mut root = VObject::new();
    // Every format is an integer: the comprehensive document binds `format_version` as a `u8`
    // and the engine's own format-3 manifests write `3`, not `"3.0.0"`. A format-0 value is
    // written as format 1, the format vanilla upgrades it to.
    let number = manifest.format.number().max(1);
    root.insert(
        "format_version",
        Value::from(VNumber::from_u64(u64::from(number))),
    );
    root.insert(
        "header",
        header(
            &manifest.header,
            v3,
            manifest.format == ManifestFormat::V1,
            manifest.pack_type(),
        )?,
    );
    let modules: VArray = manifest
        .modules
        .iter()
        .map(|module| module_value(module, v3))
        .collect();
    root.insert("modules", Value::from(modules));
    if !manifest.dependencies.is_empty() {
        let mut list = VArray::new();
        for (index, dependency) in manifest.dependencies.iter().enumerate() {
            list.push(dependency_value(dependency, v3, index)?);
        }
        root.insert("dependencies", Value::from(list));
    }
    if !manifest.capabilities.is_empty() {
        let list: VArray = manifest
            .capabilities
            .iter()
            .map(|capability| Value::from(capability.name()))
            .collect();
        root.insert("capabilities", Value::from(list));
    }
    if let Some(metadata) = &manifest.metadata {
        root.insert("metadata", metadata_value(metadata, v3));
    }
    if !manifest.subpacks.is_empty() {
        let list: VArray = manifest
            .subpacks
            .iter()
            .map(|subpack| subpack_value(subpack, v3))
            .collect();
        root.insert("subpacks", Value::from(list));
    }
    if !manifest.settings.is_empty() {
        let mut list = VArray::new();
        for (index, setting) in manifest.settings.iter().enumerate() {
            let at = format!("settings/{index}");
            if !v3 && !matches!(setting, Setting::Label { .. }) {
                // A v1/v2 manifest carrying data only v3 can hold (settings beyond labels) is
                // `NotRepresentable`.
                return Err(not_representable(&at));
            }
            list.push(setting_value(setting, &at)?);
        }
        root.insert("settings", Value::from(list));
    }
    if let Some(flag) = manifest.has_education_metadata {
        root.insert("has_education_metadata", Value::from(flag));
    }
    if !v3 {
        append_extra(&mut root, &manifest.extra);
    }
    Ok(Value::from(root))
}

/// The header member `min_engine_version`.
const MIN_ENGINE_VERSION: &str = "min_engine_version";

/// What a header writes for `min_engine_version`.
enum EngineMember {
    /// The value a format-0/1 manifest read, kept in `extra`.
    AsRead(Value),
    /// Nothing: a format-0/1 behaviour or resource pack uses the default it gets without one.
    Default,
    /// The value the pack uses.
    Used,
}

/// The format-0/1 `min_engine_version` member: the value read, kept in `extra` when
/// the used value differs from it or is the default, while the used value is still what that
/// value reads as; nothing when a behaviour or resource pack uses the default.
fn format_1_engine_member(header: &Header, pack_type: PackType) -> EngineMember {
    let used = header.min_engine_version.as_ref();
    let written = header.extra.get(MIN_ENGINE_VERSION).filter(|written| {
        version_array(written)
            .map(|version| MinEngineVersion::format_1(Some(version)))
            .as_ref()
            == used
    });
    if let Some(written) = written {
        EngineMember::AsRead(written.clone())
    } else if matches!(pack_type, PackType::Behavior | PackType::Resource)
        && used == Some(&MinEngineVersion::FORMAT_1_DEFAULT)
    {
        EngineMember::Default
    } else {
        EngineMember::Used
    }
}

fn header(
    header: &Header,
    v3: bool,
    format_1: bool,
    pack_type: PackType,
) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    object.insert("name", Value::from(header.name.as_str()));
    if let Some(description) = &header.description {
        object.insert("description", Value::from(description.as_str()));
    }
    object.insert("uuid", uuid(header.uuid));
    object.insert("version", version(&header.version, v3));
    let member = if format_1 {
        format_1_engine_member(header, pack_type)
    } else {
        EngineMember::Used
    };
    match member {
        EngineMember::AsRead(written) => {
            object.insert(MIN_ENGINE_VERSION, written);
        }
        // Read back, the missing member gives the same default.
        EngineMember::Default => {}
        EngineMember::Used => {
            if let Some(engine) = &header.min_engine_version {
                let at = "header/min_engine_version";
                if v3 {
                    object.insert(MIN_ENGINE_VERSION, Value::from(engine.version.to_string()));
                } else if engine.version.is_any() || engine.version.is_pre_release() {
                    // The legacy reader takes only the array form.
                    return Err(not_representable(at));
                } else {
                    object.insert(MIN_ENGINE_VERSION, array(&engine.version));
                }
            }
        }
    }
    if let Some(base) = &header.base_game_version {
        let value = match &base.version {
            Some(base) if base.is_any() => Value::from("*"),
            Some(base) => version(base, v3),
            None => return Err(not_representable("header/base_game_version")),
        };
        object.insert("base_game_version", value);
    }
    if let Some(scope) = header.pack_scope {
        let name = scope
            .json_name()
            .ok_or_else(|| not_representable("header/pack_scope"))?;
        object.insert("pack_scope", Value::from(name));
    }
    if let Some(flag) = header.platform_locked {
        object.insert("platform_locked", Value::from(flag));
    }
    if let Some(flag) = header.allow_random_seed {
        object.insert("allow_random_seed", Value::from(flag));
    }
    if let Some(flag) = header.lock_template_options {
        object.insert("lock_template_options", Value::from(flag));
    }
    if let Some(version) = &header.pack_optimization_version {
        object.insert("pack_optimization_version", relaxed(version, v3));
    }
    // Format 3 keeps no unknown member, but a `pack_scope` that names no scope sits here. A
    // kept `min_engine_version` is written above or not at all, never as an unknown member.
    for (key, value) in header.extra.iter() {
        if key != MIN_ENGINE_VERSION && !object.contains_key(key) {
            object.insert(key, value.clone());
        }
    }
    Ok(Value::from(object))
}

fn module_value(module: &Module, v3: bool) -> Value {
    let mut object = VObject::new();
    object.insert("type", Value::from(module.module_type.name()));
    object.insert("uuid", uuid(module.uuid));
    object.insert("version", version(&module.version, v3));
    for (key, value) in [
        ("description", &module.description),
        ("language", &module.language),
        ("entry", &module.entry),
    ] {
        if let Some(value) = value {
            object.insert(key, Value::from(value.as_str()));
        }
    }
    if !v3 {
        append_extra(&mut object, &module.extra);
    }
    Value::from(object)
}

fn dependency_value(dependency: &Dependency, v3: bool, index: usize) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    match dependency {
        Dependency::Pack {
            uuid: id,
            version: v,
        } => {
            object.insert("uuid", uuid(*id));
            object.insert("version", version(v, v3));
        }
        Dependency::NativeModule {
            module_name,
            version: v,
            optional,
        } => {
            object.insert("module_name", Value::from(module_name.as_str()));
            let v = match v {
                NativeModuleVersion::Version(v) => version(v, v3),
                NativeModuleVersion::LatestAlpha => Value::from("alpha"),
                NativeModuleVersion::LatestBeta => Value::from("beta"),
                NativeModuleVersion::LatestInternal => Value::from("internal"),
            };
            object.insert("version", v);
            if let Some(flag) = optional {
                if !v3 {
                    // The legacy reader never reads `optional`.
                    return Err(not_representable(&format!("dependencies/{index}/optional")));
                }
                object.insert("optional", Value::from(*flag));
            }
        }
    }
    Ok(Value::from(object))
}

fn metadata_value(metadata: &Metadata, v3: bool) -> Value {
    let mut object = VObject::new();
    if let Some(authors) = &metadata.authors {
        let list: VArray = authors.iter().map(|a| Value::from(a.as_str())).collect();
        object.insert("authors", Value::from(list));
    }
    for (key, value) in [
        ("license", &metadata.license),
        ("url", &metadata.url),
        ("product_type", &metadata.product_type),
    ] {
        if let Some(value) = value {
            object.insert(key, Value::from(value.as_str()));
        }
    }
    if let Some(tools) = &metadata.generated_with {
        let mut map = VObject::new();
        for (tool, versions) in tools {
            // As spelled; a version without its text in the format's own form.
            let list: VArray = versions.iter().map(|v| relaxed(v, v3)).collect();
            map.insert(tool.as_str(), Value::from(list));
        }
        object.insert("generated_with", Value::from(map));
    }
    Value::from(object)
}

fn subpack_value(subpack: &Subpack, v3: bool) -> Value {
    let mut object = VObject::new();
    object.insert("folder_name", Value::from(subpack.folder_name.as_str()));
    object.insert("name", Value::from(subpack.name.as_str()));
    if v3 {
        object.insert(
            "memory_performance_tier",
            Value::from(VNumber::from_u64(u64::from(subpack.memory_tier.number()))),
        );
    } else {
        let size = u64::from(subpack.memory_tier.legacy_size());
        object.insert("memory_tier", Value::from(VNumber::from_u64(size)));
    }
    Value::from(object)
}

fn setting_value(setting: &Setting, at: &str) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    object.insert("type", Value::from(setting.type_name()));
    if let Some(name) = setting.name() {
        object.insert("name", Value::from(name));
    }
    let number = |key: &str, value: f32| -> Result<Value, EncodeError> {
        if !value.is_finite() {
            return Err(non_finite(&format!("{at}/{key}"), value));
        }
        // The shortest text that reads back to the same `f32`, as an `f64` real.
        let shortest: f64 = value.to_string().parse().unwrap_or(f64::from(value));
        Ok(Value::from(VNumber::from_f64(shortest)))
    };
    let options = |options: &[super::DropdownOption]| -> Value {
        let list: VArray = options
            .iter()
            .map(|option| {
                let mut object = VObject::new();
                object.insert("name", Value::from(option.name.as_str()));
                object.insert("text", Value::from(option.text.as_str()));
                Value::from(object)
            })
            .collect();
        Value::from(list)
    };
    match setting {
        Setting::Label { text } => {
            object.insert("text", Value::from(text.as_str()));
        }
        Setting::Slider {
            text,
            min,
            max,
            step,
            default,
            ..
        } => {
            object.insert("text", Value::from(text.as_str()));
            object.insert("min", number("min", *min)?);
            object.insert("max", number("max", *max)?);
            object.insert("step", number("step", *step)?);
            object.insert("default", number("default", *default)?);
        }
        Setting::Toggle { text, default, .. } => {
            object.insert("text", Value::from(text.as_str()));
            object.insert("default", Value::from(*default));
        }
        Setting::Dropdown {
            text,
            options: list,
            default,
            ..
        } => {
            object.insert("text", Value::from(text.as_str()));
            object.insert("options", options(list));
            object.insert("default", Value::from(default.as_str()));
        }
        Setting::Multiselect {
            text,
            options: list,
            defaults,
            selection_text,
            ..
        } => {
            object.insert("text", Value::from(text.as_str()));
            object.insert("options", options(list));
            let defaults: VArray = defaults.iter().map(|d| Value::from(d.as_str())).collect();
            object.insert("defaults", Value::from(defaults));
            if let Some(selection_text) = selection_text {
                object.insert("selection_text", Value::from(selection_text.as_str()));
            }
        }
    }
    Ok(Value::from(object))
}

/// A UUID in its hyphenated lower-case form.
fn uuid(uuid: uuid::Uuid) -> Value {
    Value::from(uuid.hyphenated().to_string())
}

/// A version in the format's form: a string from format 3; legacy, the `[a, b, c]` array unless
/// the version has a pre-release or build part, which only the string form holds.
fn version(version: &SemVersion, v3: bool) -> Value {
    if v3 || version.is_any() || version.is_pre_release() || !version.build.is_empty() {
        Value::from(version.to_string())
    } else {
        array(version)
    }
}

/// A relaxed version as spelled; one without its text in the format's own form.
fn relaxed(relaxed: &RelaxedSemVersion, v3: bool) -> Value {
    relaxed
        .spelling()
        .map_or_else(|| version(&relaxed.version, v3), Value::from)
}

/// `[major, minor, patch]`.
fn array(version: &SemVersion) -> Value {
    let list: VArray = [version.major, version.minor, version.patch]
        .into_iter()
        .map(|n| Value::from(VNumber::from_u64(u64::from(n))))
        .collect();
    Value::from(list)
}

/// The unknown members after the known ones (a known key is never replaced).
fn append_extra(object: &mut VObject, extra: &UnknownMembers) {
    for (key, value) in extra.iter() {
        if !object.contains_key(key) {
            object.insert(key, value.clone());
        }
    }
}
