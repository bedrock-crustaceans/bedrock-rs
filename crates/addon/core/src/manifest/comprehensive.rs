//! The comprehensive reader: `format_version` 3 (vanilla's cereal manifest document, read
//! strictly, then validated and turned into the pack's manifest).
//!
//! Every member is typed: an unknown member, a missing required one or a value of the wrong
//! JSON type is an error, and versions are strings. The pack-type rules of the header are the
//! legacy format-2 rules.

use std::collections::HashSet;

use crate::diagnostic::{DiagnosticCode as Code, VanillaText};
use crate::document::Extra;
use crate::json::Value;
use crate::pack_identity::PackType;
use crate::version::{ENGINE_VERSION, MinEngineVersion, SemVersion};
use uuid::Uuid;

use super::capability::CapabilityRegistry;
use super::legacy::{base_game_version, check_capability, metadata, modules_consistent, pack_type};
use super::read::{Sink, child, item, member, pack_scope, relaxed_version_text, text};
use super::{
    Capability, Dependency, Header, Manifest, ManifestFormat, MemoryTier, Module, ModuleType,
    NativeModuleVersion, RelaxedSemVersion, Subpack, ids, setting,
};

/// The lowest `min_engine_version` a behaviour or resource pack may declare.
const LOWEST_ENGINE: SemVersion = SemVersion::new(1, 13, 0);

/// Reads a manifest whose `format_version` is 3.0.0 or above. `None` when it has no usable
/// value (every such case has left an error).
pub(super) fn read(
    root: &Value,
    capabilities: &CapabilityRegistry,
    sink: &mut Sink<'_>,
) -> Option<Manifest> {
    if root.as_object().is_none() {
        sink.error(Code::InvalidType { text: None }, "");
        return None;
    }
    unknown_members(
        sink,
        root,
        "",
        &[
            "format_version",
            "header",
            "modules",
            "dependencies",
            "capabilities",
            "settings",
            "subpacks",
            "metadata",
            "has_education_metadata",
        ],
    );
    format_version(sink, root);
    let modules = modules(sink, root);
    let pack_type = pack_type(&modules);
    let header = header(sink, root, pack_type);
    let dependencies = dependencies(sink, root);
    let capability_list = capability_list(sink, root, header.as_ref(), pack_type, capabilities);
    let settings = setting::read(
        sink,
        root,
        header.as_ref().and_then(|h| h.min_engine_version.as_ref()),
    );
    let subpacks = subpacks(sink, root);
    let metadata = metadata(sink, root, "metadata", true);
    let has_education_metadata = boolean(sink, root, "has_education_metadata", "", false);
    if sink.has_errors() {
        return None;
    }
    Some(Manifest {
        format: ManifestFormat::V3,
        original_format: ManifestFormat::V3,
        header: header?,
        modules,
        dependencies,
        capabilities: capability_list,
        metadata,
        subpacks,
        settings,
        has_education_metadata,
        extra: Extra::default(),
    })
}

/// `format_version`, bound as a `u8` and read strictly: a non-negative integer below 256; any
/// other JSON value (a string, an array, a real, `null`) is a type error. The routing has
/// already read it as 3.0.0.
fn format_version(sink: &mut Sink<'_>, root: &Value) {
    let Some(value) = member(root, "format_version") else {
        sink.error(Code::MissingRequiredField { text: None }, "format_version");
        return;
    };
    let fits = value
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_u64)
        .is_some_and(|number| number < 256);
    if !fits {
        sink.error(Code::InvalidType { text: None }, "format_version");
    }
}

/// Reports every member of `object` outside `allowed` as an unrecognised field (an error: the
/// manifest is read with a strict mask).
pub(super) fn unknown_members(sink: &mut Sink<'_>, object: &Value, at: &str, allowed: &[&str]) {
    let Some(object) = object.as_object() else {
        return;
    };
    for (key, _) in object {
        if !allowed.contains(&key.as_str()) {
            sink.error(
                Code::UnrecognizedField { text: None },
                &child(at, key.as_str()),
            );
        }
    }
}

/// The object member `key`, or `None` with an error when it is not an object.
fn object<'v>(
    sink: &mut Sink<'_>,
    container: &'v Value,
    key: &str,
    at: &str,
    required: bool,
) -> Option<&'v Value> {
    let at = child(at, key);
    let Some(value) = member(container, key) else {
        if required {
            sink.error(Code::MissingRequiredField { text: None }, &at);
        }
        return None;
    };
    if value.as_object().is_none() {
        sink.error(Code::InvalidType { text: None }, &at);
        return None;
    }
    Some(value)
}

/// The array member `key`, or `None` with an error when it is not an array.
fn array<'v>(
    sink: &mut Sink<'_>,
    container: &'v Value,
    key: &str,
    at: &str,
    required: bool,
) -> Option<&'v [Value]> {
    let at = child(at, key);
    let Some(value) = member(container, key) else {
        if required {
            sink.error(Code::MissingRequiredField { text: None }, &at);
        }
        return None;
    };
    let Some(array) = value.as_array() else {
        sink.error(Code::InvalidType { text: None }, &at);
        return None;
    };
    Some(array.as_slice())
}

/// A string member; wrong type is an error, so is an absent one when `required`.
pub(super) fn string(
    sink: &mut Sink<'_>,
    container: &Value,
    key: &str,
    at: &str,
    required: bool,
) -> Option<String> {
    let at = child(at, key);
    let Some(value) = member(container, key) else {
        if required {
            sink.error(Code::MissingRequiredField { text: None }, &at);
        }
        return None;
    };
    let read = text(value).map(str::to_owned);
    if read.is_none() {
        sink.error(Code::InvalidType { text: None }, &at);
    }
    read
}

/// A boolean member; wrong type is an error, so is an absent one when `required`.
fn boolean(
    sink: &mut Sink<'_>,
    container: &Value,
    key: &str,
    at: &str,
    required: bool,
) -> Option<bool> {
    let at = child(at, key);
    let Some(value) = member(container, key) else {
        if required {
            sink.error(Code::MissingRequiredField { text: None }, &at);
        }
        return None;
    };
    let read = value.as_bool();
    if read.is_none() {
        sink.error(Code::InvalidType { text: None }, &at);
    }
    read
}

/// A required UUID string; a malformed or nil one is an error and is replaced by its hash.
fn uuid(sink: &mut Sink<'_>, container: &Value, at: &str) -> Option<Uuid> {
    let raw = string(sink, container, "uuid", at, true)?;
    if let Some(uuid) = ids::parse(&raw) {
        return Some(uuid);
    }
    sink.error(Code::MalformedPropertyUuid, &child(at, "uuid"));
    Some(ids::salvage(&raw))
}

/// A required version string (partial versions are filled with zeros;
/// no wildcard). An array is the wrong type.
fn version(sink: &mut Sink<'_>, container: &Value, at: &str) -> Option<SemVersion> {
    let raw = string(sink, container, "version", at, true)?;
    let parsed = relaxed_version_text(&raw);
    if parsed.is_none() {
        sink.error(
            Code::MalformedPropertyVersion { text: None },
            &child(at, "version"),
        );
    }
    parsed
}

fn header(sink: &mut Sink<'_>, root: &Value, pack_type: PackType) -> Option<Header> {
    const AT: &str = "header";
    let header = object(sink, root, "header", "", true)?;
    unknown_members(
        sink,
        header,
        AT,
        &[
            "name",
            "description",
            "uuid",
            "version",
            "min_engine_version",
            "base_game_version",
            "pack_scope",
            "platform_locked",
            "allow_random_seed",
            "lock_template_options",
            "pack_optimization_version",
        ],
    );
    let name = string(sink, header, "name", AT, true).inspect(|name| {
        if name.is_empty() {
            // An empty `name` is reported through `manifestvalidation.noname` from format 2; the
            // name stays empty and only reports show the placeholder. The format-3 reader has
            // no report entry for it, so the game says nothing and the crate reports an
            // information.
            sink.note(Code::EmptyProperty, "header/name");
        }
    });
    let description = string(sink, header, "description", AT, false);
    let uuid = uuid(sink, header, AT);
    let version = version(sink, header, AT);
    let min_engine_version = min_engine_version(sink, header, pack_type);
    let base_game_version =
        if member(header, "base_game_version").is_some_and(|v| text(v).is_none()) {
            sink.error(Code::InvalidType { text: None }, "header/base_game_version");
            None
        } else {
            base_game_version(sink, header, pack_type, true, "header/base_game_version")
        };
    let scope_text = string(sink, header, "pack_scope", AT, false);
    let mut extra = Extra::default();
    let pack_scope = pack_scope(
        sink,
        scope_text,
        Code::InvalidValue { text: None },
        &mut extra,
    );
    let platform_locked = boolean(sink, header, "platform_locked", AT, false);
    let allow_random_seed = boolean(sink, header, "allow_random_seed", AT, false);
    let lock_template_options = boolean(sink, header, "lock_template_options", AT, false);
    let lock_template_options = match (pack_type == PackType::WorldTemplate, lock_template_options)
    {
        (true, None) if member(header, "lock_template_options").is_none() => {
            sink.error(
                Code::MissingProperty { text: None },
                "header/lock_template_options",
            );
            None
        }
        (false, Some(_)) => {
            // A `lock_template_options` on a pack that is not a world template is a pack-report
            // warning, which counts as the level-2 class.
            sink.warning(Code::IgnoredProperty, "header/lock_template_options");
            None
        }
        (_, value) => value,
    };
    let pack_optimization_version = string(sink, header, "pack_optimization_version", AT, false)
        .and_then(|raw| RelaxedSemVersion::from_text(&raw));
    Some(Header {
        name: name?,
        description,
        uuid: uuid?,
        version: version?,
        min_engine_version,
        base_game_version,
        pack_scope,
        platform_locked,
        allow_random_seed,
        lock_template_options,
        pack_optimization_version,
        extra,
    })
}

/// `min_engine_version` as a string, with the format-2 rules: required by behaviour and
/// resource packs, between 1.13.0 and the engine version; ignored with a warning elsewhere.
fn min_engine_version(
    sink: &mut Sink<'_>,
    header: &Value,
    pack_type: PackType,
) -> Option<MinEngineVersion> {
    const AT: &str = "header/min_engine_version";
    let enforced = matches!(pack_type, PackType::Behavior | PackType::Resource);
    let Some(value) = member(header, "min_engine_version") else {
        if enforced {
            sink.error(Code::MissingProperty { text: None }, AT);
        }
        return None;
    };
    let Some(raw) = text(value) else {
        sink.error(Code::InvalidType { text: None }, AT);
        return None;
    };
    if !enforced {
        // A `min_engine_version` on a pack that does not enforce one (the legacy reader
        // reports it too) is a pack-report warning, which counts as the level-2 class. The
        // report does not identify the member.
        sink.warning(Code::IgnoredProperty, AT);
        return None;
    }
    let Some(version) = relaxed_version_text(raw) else {
        sink.error(Code::MalformedPropertyVersion { text: None }, AT);
        return None;
    };
    let engine = SemVersion::from(ENGINE_VERSION);
    if version > engine {
        sink.error(
            Code::VersionTooHigh {
                version: version.to_string(),
                limit: engine.to_string(),
            },
            AT,
        );
        return None;
    }
    if version < LOWEST_ENGINE {
        sink.error(
            Code::VersionTooLow {
                version: version.to_string(),
                limit: LOWEST_ENGINE.to_string(),
            },
            AT,
        );
        return None;
    }
    Some(MinEngineVersion { version })
}

fn modules(sink: &mut Sink<'_>, root: &Value) -> Vec<Module> {
    const AT: &str = "modules";
    let Some(elements) = array(sink, root, AT, "", true) else {
        return Vec::new();
    };
    if elements.is_empty() {
        // The schema needs at least one module, and the game asserts that one exists.
        sink.error(Code::InvalidValue { text: None }, AT);
    }
    let mut modules = Vec::new();
    let mut seen = HashSet::new();
    for (index, element) in elements.iter().enumerate() {
        let at = item(AT, index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        unknown_members(
            sink,
            element,
            &at,
            &[
                "type",
                "uuid",
                "version",
                "description",
                "entry",
                "language",
            ],
        );
        let type_name = string(sink, element, "type", &at, true);
        let module_type = type_name.and_then(|name| {
            let ty = ModuleType::from_name(&name).filter(|ty| *ty != ModuleType::Invalid);
            if ty.is_none() {
                // The game names the member without its element's index.
                let text = Some(VanillaText::ManifestInvalidValue {
                    path: "modules/type".to_owned(),
                });
                sink.error(Code::InvalidProperty { text }, &child(&at, "type"));
            }
            ty
        });
        let uuid = uuid(sink, element, &at);
        let version = version(sink, element, &at);
        let description = string(sink, element, "description", &at, false);
        let language = string(sink, element, "language", &at, false);
        let entry = string(sink, element, "entry", &at, false);
        if let Some(uuid) = uuid
            && !seen.insert(uuid)
        {
            // The game's comprehensive-manifest check reports "duplicate module uuid found: …".
            sink.error(Code::DuplicateUuid, &child(&at, "uuid"));
        }
        if let (Some(module_type), Some(uuid), Some(version)) = (module_type, uuid, version) {
            modules.push(Module {
                module_type,
                uuid,
                version,
                description,
                entry,
                language,
                extra: Extra::default(),
            });
        }
    }
    if !modules_consistent(&modules) {
        sink.error(Code::MultipleModules, AT);
    }
    modules
}

fn dependencies(sink: &mut Sink<'_>, root: &Value) -> Vec<Dependency> {
    const AT: &str = "dependencies";
    let Some(elements) = array(sink, root, AT, "", false) else {
        return Vec::new();
    };
    let mut dependencies = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = item(AT, index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        if member(element, "module_name").is_none() {
            unknown_members(sink, element, &at, &["uuid", "version"]);
            let uuid = uuid(sink, element, &at);
            let version = version(sink, element, &at);
            if let (Some(uuid), Some(version)) = (uuid, version) {
                dependencies.push(Dependency::Pack { uuid, version });
            }
            continue;
        }
        // Format 3 rejects `module_name` together with `uuid` on one element
        // (`UnrecognizedField`).
        unknown_members(sink, element, &at, &["module_name", "version", "optional"]);
        let module_name = string(sink, element, "module_name", &at, true);
        let optional = boolean(sink, element, "optional", &at, false);
        let version = script_module_version(sink, element, &at, module_name.as_deref());
        if let (Some(module_name), Some(version)) = (module_name, version) {
            dependencies.push(Dependency::NativeModule {
                module_name,
                version,
                optional,
            });
        }
    }
    dependencies
}

/// A script-module dependency's `version`: `alpha`, `beta`, `internal` or a relaxed version
/// string (`"2.0"` from 1.26.30); an array is the wrong type, `"*"` and other non-versions are
/// malformed.
fn script_module_version(
    sink: &mut Sink<'_>,
    element: &Value,
    at: &str,
    module_name: Option<&str>,
) -> Option<NativeModuleVersion> {
    let raw = string(sink, element, "version", at, true)?;
    match raw.as_str() {
        "alpha" => Some(NativeModuleVersion::LatestAlpha),
        "beta" => Some(NativeModuleVersion::LatestBeta),
        "internal" => Some(NativeModuleVersion::LatestInternal),
        _ => {
            let parsed = relaxed_version_text(&raw).map(NativeModuleVersion::Version);
            if parsed.is_none() {
                // "dependency {} invalid version: {}": the version printed is the invalid one
                // the setter left, `0.0.0`.
                let text = module_name.map(|module| VanillaText::DependencyInvalidVersion {
                    module: module.to_owned(),
                    version: "0.0.0".to_owned(),
                });
                sink.error(
                    Code::MalformedPropertyVersion { text },
                    &child(at, "version"),
                );
            }
            parsed
        }
    }
}

fn capability_list(
    sink: &mut Sink<'_>,
    root: &Value,
    header: Option<&Header>,
    pack_type: PackType,
    registry: &CapabilityRegistry,
) -> Vec<Capability> {
    const AT: &str = "capabilities";
    let Some(elements) = array(sink, root, AT, "", false) else {
        return Vec::new();
    };
    let mut capabilities = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = item(AT, index);
        let Some(name) = text(element) else {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        };
        let capability = Capability::from_name(name);
        let uuid = header.map_or_else(Uuid::nil, |header| header.uuid);
        check_capability(sink, registry, &capability, uuid, pack_type, &at);
        capabilities.push(capability);
    }
    capabilities
}

fn subpacks(sink: &mut Sink<'_>, root: &Value) -> Vec<Subpack> {
    const AT: &str = "subpacks";
    let Some(elements) = array(sink, root, AT, "", false) else {
        return Vec::new();
    };
    let mut subpacks = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = item(AT, index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        unknown_members(
            sink,
            element,
            &at,
            &["folder_name", "memory_performance_tier", "name"],
        );
        let folder_name = string(sink, element, "folder_name", &at, true);
        let name = string(sink, element, "name", &at, true);
        let tier_at = child(&at, "memory_performance_tier");
        let memory_tier = match member(element, "memory_performance_tier") {
            None => {
                sink.error(Code::MissingRequiredField { text: None }, &tier_at);
                None
            }
            Some(value) => match value
                .as_number()
                .filter(|n| n.is_integer())
                .and_then(facet_value::VNumber::to_i64)
            {
                None => {
                    sink.error(Code::InvalidType { text: None }, &tier_at);
                    None
                }
                Some(tier) => {
                    // Schema: 0 to 5.
                    let tier = u8::try_from(tier).ok().and_then(MemoryTier::from_number);
                    if tier.is_none() {
                        sink.error(Code::InvalidValue { text: None }, &tier_at);
                    }
                    tier
                }
            },
        };
        if let (Some(folder_name), Some(name), Some(memory_tier)) = (folder_name, name, memory_tier)
        {
            subpacks.push(Subpack {
                name,
                folder_name,
                memory_tier,
            });
        }
    }
    subpacks
}
