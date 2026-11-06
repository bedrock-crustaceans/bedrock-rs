//! The comprehensive reader: `format_version` 3 (vanilla's cereal manifest document, read
//! strictly, then validated and turned into the pack's manifest).
//!
//! Every member is typed: an unknown member, a missing required one or a value of the wrong
//! JSON type is an error, and versions are strings. The pack-type rules of the header are the
//! legacy format-2 rules.

use std::collections::HashSet;

use crate::diagnostic::{DiagnosticCode as Code, JsonPath, VanillaText};
use crate::document::UnknownMembers;
use crate::json::Value;
use crate::pack_identity::PackType;
use crate::version::MinEngineVersion;
use uuid::Uuid;

use super::capability::CapabilityRegistry;
use super::legacy::{base_game_version, capability_list, metadata, modules_consistent, pack_type};
use super::read::{
    Profile, Sink, below, dependency_place, member, pack_scope, path, relaxed_version_text, text,
};
use super::{
    Dependency, DependencyPlace, Header, Manifest, ManifestFormat, ManifestPlaces, MemoryTier,
    Module, ModuleType, NativeModuleVersion, RelaxedSemVersion, Subpack, setting,
};

/// Reads a manifest whose `format_version` is 3.0.0 or above. `None` when it has no usable
/// value (every such case has left an error).
pub(super) fn read(
    root: &Value,
    capabilities: &CapabilityRegistry,
    sink: &mut Sink<'_>,
) -> Option<Manifest> {
    sink.set_profile(Profile::Comprehensive);
    if root.as_object().is_none() {
        sink.error(Code::InvalidType { text: None }, &JsonPath::new());
        return None;
    }
    sink.unknown(
        root,
        &JsonPath::new(),
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
    let (dependencies, dependency_places) = dependencies(sink, root);
    // Without a header the pack's UUID is the nil one.
    let pack_uuid = header.as_ref().map_or_else(Uuid::nil, |header| header.uuid);
    let capability_list = capability_list(sink, root, pack_uuid, pack_type, capabilities);
    let settings = setting::read(
        sink,
        root,
        header.as_ref().and_then(|h| h.min_engine_version.as_ref()),
    );
    let subpacks = subpacks(sink, root);
    let metadata = metadata(sink, root, &path(&["metadata"]));
    let has_education_metadata =
        sink.boolean(root, "has_education_metadata", &JsonPath::new(), false);
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
        extra: UnknownMembers::default(),
        places: ManifestPlaces {
            dependencies: dependency_places,
        },
    })
}

/// `format_version`, bound as a `u8` and read strictly: a non-negative integer below 256; any
/// other JSON value (a string, an array, a real, `null`) is a type error. The routing has
/// already read it as 3.0.0.
fn format_version(sink: &mut Sink<'_>, root: &Value) {
    let Some(value) = member(root, "format_version") else {
        sink.error(
            Code::MissingRequiredField { text: None },
            &path(&["format_version"]),
        );
        return;
    };
    let fits = value
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_u64)
        .is_some_and(|number| number < 256);
    if !fits {
        sink.error(Code::InvalidType { text: None }, &path(&["format_version"]));
    }
}

fn header(sink: &mut Sink<'_>, root: &Value, pack_type: PackType) -> Option<Header> {
    let at = path(&["header"]);
    let header = sink.object(root, "header", &JsonPath::new(), true)?;
    sink.unknown(
        header,
        &at,
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
    let name = sink.string(header, "name", &at, true).inspect(|name| {
        if name.is_empty() {
            // An empty `name` is reported through `manifestvalidation.noname` from format 2; the
            // name stays empty and only reports show the placeholder. The format-3 reader has
            // no report entry for it, so the game says nothing and the crate reports an
            // information.
            sink.note(Code::EmptyProperty, &at.child("name"));
        }
    });
    let description = sink.string(header, "description", &at, false);
    let uuid = sink.uuid(header, "uuid", &at, false);
    let version = sink.version(header, "version", &at, false);
    let min_engine_version = min_engine_version(sink, header, pack_type);
    let base_game_version =
        if member(header, "base_game_version").is_some_and(|v| text(v).is_none()) {
            sink.error(
                Code::InvalidType { text: None },
                &at.child("base_game_version"),
            );
            None
        } else {
            base_game_version(
                sink,
                header,
                pack_type,
                true,
                &at.child("base_game_version"),
            )
        };
    let scope_text = sink.string(header, "pack_scope", &at, false);
    let mut extra = UnknownMembers::default();
    let pack_scope = pack_scope(
        sink,
        scope_text,
        Code::InvalidValue { text: None },
        &mut extra,
    );
    let platform_locked = sink.boolean(header, "platform_locked", &at, false);
    let allow_random_seed = sink.boolean(header, "allow_random_seed", &at, false);
    let lock_template_options = sink.boolean(header, "lock_template_options", &at, false);
    let lock_template_options = match (pack_type == PackType::WorldTemplate, lock_template_options)
    {
        (true, None) if member(header, "lock_template_options").is_none() => {
            sink.error(
                Code::MissingProperty { text: None },
                &at.child("lock_template_options"),
            );
            None
        }
        (false, Some(_)) => {
            // A `lock_template_options` on a pack that is not a world template is a pack-report
            // warning, which counts as the level-2 class.
            sink.warning(Code::IgnoredProperty, &at.child("lock_template_options"));
            None
        }
        (_, value) => value,
    };
    let pack_optimization_version = sink
        .string(header, "pack_optimization_version", &at, false)
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
    let at = path(&["header", "min_engine_version"]);
    let enforced = matches!(pack_type, PackType::Behavior | PackType::Resource);
    let Some(value) = member(header, "min_engine_version") else {
        if enforced {
            sink.error(Code::MissingProperty { text: None }, &at);
        }
        return None;
    };
    let Some(raw) = text(value) else {
        sink.error(Code::InvalidType { text: None }, &at);
        return None;
    };
    if !enforced {
        // A `min_engine_version` on a pack that does not enforce one (the legacy reader
        // reports it too) is a pack-report warning, which counts as the level-2 class. The
        // report does not identify the member.
        sink.warning(Code::IgnoredProperty, &at);
        return None;
    }
    let Some(version) = relaxed_version_text(raw) else {
        sink.error(Code::MalformedPropertyVersion { text: None }, &at);
        return None;
    };
    sink.engine_bounds(&version, &at)
        .then_some(MinEngineVersion { version })
}

fn modules(sink: &mut Sink<'_>, root: &Value) -> Vec<Module> {
    const AT: &str = "modules";
    let list = path(&[AT]);
    let Some(elements) = sink.array(root, AT, &JsonPath::new(), true) else {
        return Vec::new();
    };
    if elements.is_empty() {
        // The schema needs at least one module, and the game asserts that one exists.
        sink.error(Code::InvalidValue { text: None }, &list);
    }
    let mut modules = Vec::new();
    let mut seen = HashSet::new();
    for (index, element) in elements.iter().enumerate() {
        let at = list.index(index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        sink.unknown(
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
        let type_name = sink.string(element, "type", &at, true);
        let module_type = type_name.and_then(|name| {
            let ty = ModuleType::from_name(&name).filter(|ty| *ty != ModuleType::Invalid);
            if ty.is_none() {
                // The game names the member without its element's index.
                let text = Some(VanillaText::ManifestInvalidValue {
                    path: "modules/type".to_owned(),
                });
                sink.error(Code::InvalidProperty { text }, &below(&at, "type"));
            }
            ty
        });
        let uuid = sink.uuid(element, "uuid", &at, false);
        let version = sink.version(element, "version", &at, false);
        let description = sink.string(element, "description", &at, false);
        let language = sink.string(element, "language", &at, false);
        let entry = sink.string(element, "entry", &at, false);
        if let Some(uuid) = uuid
            && !seen.insert(uuid)
        {
            // The game's comprehensive-manifest check reports "duplicate module uuid found: …".
            sink.error(Code::DuplicateUuid, &below(&at, "uuid"));
        }
        if let (Some(module_type), Some(uuid), Some(version)) = (module_type, uuid, version) {
            modules.push(Module {
                module_type,
                uuid,
                version,
                description,
                entry,
                language,
                extra: UnknownMembers::default(),
            });
        }
    }
    if !modules_consistent(&modules) {
        sink.error(Code::MultipleModules, &list);
    }
    modules
}

fn dependencies(sink: &mut Sink<'_>, root: &Value) -> (Vec<Dependency>, Vec<DependencyPlace>) {
    const AT: &str = "dependencies";
    let list = path(&[AT]);
    let Some(elements) = sink.array(root, AT, &JsonPath::new(), false) else {
        return (Vec::new(), Vec::new());
    };
    let mut dependencies = Vec::new();
    let mut places = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = list.index(index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        if member(element, "module_name").is_none() {
            sink.unknown(element, &at, &["uuid", "version"]);
            let uuid = sink.uuid(element, "uuid", &at, false);
            let version = sink.version(element, "version", &at, false);
            if let (Some(uuid), Some(version)) = (uuid, version) {
                dependencies.push(Dependency::Pack { uuid, version });
                places.push(dependency_place(element, at, false));
            }
            continue;
        }
        // Format 3 rejects `module_name` together with `uuid` on one element
        // (`UnrecognizedField`).
        sink.unknown(element, &at, &["module_name", "version", "optional"]);
        let module_name = sink.string(element, "module_name", &at, true);
        let optional = sink.boolean(element, "optional", &at, false);
        let version = script_module_version(sink, element, &at, module_name.as_deref());
        if let (Some(module_name), Some(version)) = (module_name, version) {
            dependencies.push(Dependency::NativeModule {
                module_name,
                version,
                optional,
            });
            places.push(dependency_place(element, at, true));
        }
    }
    (dependencies, places)
}

/// A script-module dependency's `version`: `alpha`, `beta`, `internal` or a relaxed version
/// string (`"2.0"` from 1.26.30); an array is the wrong type, `"*"` and other non-versions are
/// malformed.
fn script_module_version(
    sink: &mut Sink<'_>,
    element: &Value,
    at: &JsonPath,
    module_name: Option<&str>,
) -> Option<NativeModuleVersion> {
    let raw = sink.string(element, "version", at, true)?;
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
                    &below(at, "version"),
                );
            }
            parsed
        }
    }
}

fn subpacks(sink: &mut Sink<'_>, root: &Value) -> Vec<Subpack> {
    const AT: &str = "subpacks";
    let list = path(&[AT]);
    let Some(elements) = sink.array(root, AT, &JsonPath::new(), false) else {
        return Vec::new();
    };
    let mut subpacks = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = list.index(index);
        if element.as_object().is_none() {
            sink.error(Code::InvalidType { text: None }, &at);
            continue;
        }
        sink.unknown(
            element,
            &at,
            &["folder_name", "memory_performance_tier", "name"],
        );
        let folder_name = sink.string(element, "folder_name", &at, true);
        let name = sink.string(element, "name", &at, true);
        let tier_at = below(&at, "memory_performance_tier");
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
