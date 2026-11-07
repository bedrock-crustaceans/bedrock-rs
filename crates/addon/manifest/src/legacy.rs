//! The legacy reader: `format_version` 0, 1 and 2 (vanilla's hand-written manifest parser).
//!
//! Format 1 and 2 read modules first (the last module decides the pack type, which decides
//! the header rules), then the header, dependencies, metadata, settings, subpacks,
//! capabilities and the education flag. Format 0 reads `pack_id` / `packs_version`, takes its
//! modules and metadata from inside the header and skips everything else; it is upgraded to
//! format 1, and its UUID, version and module problems are warnings instead of errors.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use indexmap::IndexMap;
use uuid::Uuid;

use super::capability::{CapabilityCx, CapabilityRegistry, CapabilityValidation};
use super::read::{
    Profile, Sink, below, dependency_place, member, pack_scope, path, pointer, strict_version_text,
    text, version_array,
};
use super::{
    Capability, Dependency, DependencyPlace, Header, Manifest, ManifestFormat, ManifestPlaces,
    MemoryTier, Metadata, Module, ModuleType, NativeModuleVersion, RelaxedSemVersion, Setting,
    Subpack, ids,
};

/// What vanilla shows for a manifest without a name (the `manifestvalidation.noname` key).
pub(super) const NO_NAME: &str = "manifestvalidation.noname";

/// Reads a legacy manifest whose `format_version` did not route to the comprehensive reader.
/// `None` when the manifest has no usable value (every such case has left an error).
pub(super) fn read(
    root: &Value,
    capabilities: &CapabilityRegistry,
    sink: &mut Sink<'_>,
) -> Option<Manifest> {
    sink.set_profile(Profile::Legacy);
    let format = format_version(root, sink)?;
    let mut reader = Legacy {
        format,
        upgraded: format == 0,
        sink,
    };
    if format == 0 {
        // The game logs "Pack Manifest version 0 is deprecated, please include
        // format_version 2 in your manifest.json." for a format-0 manifest at content-log
        // level 2, area 36: a warning.
        reader.sink.push(
            core_codes::DEPRECATED_MEMBER_WORDED.with(["Pack Manifest version 0 is deprecated, please include format_version 2 in your manifest.json.",]),
            bedrock_addon_core::diagnostic::Severity::Warning,
            bedrock_addon_core::diagnostic::Phase::Parse,
            &path(&["format_version"]),
        );
        let manifest = reader.read_v0(root);
        if manifest.is_some() {
            // The game upgrades a format-0 manifest to format 1, and so it is held and
            // written: `pack_id` as `uuid`, `packs_version` as `version`, the header's
            // modules and metadata at the root. The written text is another text, which the
            // load says at the root (the crate's own information; the game upgrades in
            // silence).
            reader.sink.push(
                Code::from(&core_codes::UNUSUAL_FORMAT_VERSION),
                bedrock_addon_core::diagnostic::Severity::Info,
                bedrock_addon_core::diagnostic::Phase::Upgrade,
                &JsonPath::new(),
            );
        }
        manifest
    } else {
        reader.read_v12(root, capabilities)
    }
}

/// `format_version` as vanilla's legacy parser reads it: jsoncpp `asInt` truncated to a `u8`.
/// Absent or `null` is 0; a string, array or object is 0, `true` 1, a real truncated; each
/// coercion and truncation is linted by the crate. 3 and above (after truncation) is an error and
/// nothing is read.
fn format_version(root: &Value, sink: &mut Sink<'_>) -> Option<u8> {
    // The legacy reader takes `asInt` truncated to a `u8`, and other JSON types coerce
    // (a string is 0, `true` is 1, 256 is 0). The coercion is reproduced as parse semantics
    // and the crate adds a lint for it.
    let Some(value) = member(root, "format_version").filter(|v| !v.is_null()) else {
        return Some(0);
    };
    let at = path(&["format_version"]);
    let as_int: i64 = if let Some(number) = value.as_number() {
        if number.is_integer() {
            number.to_i64().unwrap_or(0)
        } else {
            // The coercion is parse semantics; the lint is the crate's own.
            sink.note(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
            #[allow(
                clippy::cast_possible_truncation,
                reason = "jsoncpp `asInt` truncates a real toward zero"
            )]
            let truncated = number.to_f64_lossy().trunc() as i64;
            truncated
        }
    } else if let Some(flag) = value.as_bool() {
        sink.note(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
        i64::from(flag)
    } else {
        sink.note(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
        0
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the `& 0xFF` of vanilla"
    )]
    let format = (as_int & 0xFF) as u8;
    if format >= 3 {
        // The game reports an error at `format_version` before dispatching, so nothing is
        // parsed.
        sink.error(Code::from(&core_codes::INVALID_PROPERTY), &at);
        return None;
    }
    if i64::from(format) != as_int {
        // A lint of the coercion, the crate's own.
        sink.note(Code::from(&core_codes::INVALID_PROPERTY), &at);
    }
    Some(format)
}

/// What a legacy read needs at every step.
struct Legacy<'s, 'p> {
    /// The format as read: 0, 1 or 2.
    format: u8,
    /// Format 0: findings vanilla reports as "upgraded" twins are warnings.
    upgraded: bool,
    sink: &'s mut Sink<'p>,
}

/// The header as read before the pack type is known.
struct HeaderParts<'v> {
    header: Header,
    object: &'v Value,
}

impl Legacy<'_, '_> {
    fn manifest_format(&self) -> (ManifestFormat, ManifestFormat) {
        match self.format {
            0 => (ManifestFormat::V1, ManifestFormat::V0),
            1 => (ManifestFormat::V1, ManifestFormat::V1),
            _ => (ManifestFormat::V2, ManifestFormat::V2),
        }
    }

    /// Format 1 and 2: modules, header, then the rest, each reading from the root.
    fn read_v12(&mut self, root: &Value, capabilities: &CapabilityRegistry) -> Option<Manifest> {
        let errors = self.sink.errors();
        let modules = self.modules(root, &JsonPath::new());
        // An error from the modules skips the header and the rest.
        if self.sink.errors() > errors {
            return None;
        }
        let pack_type = pack_type(&modules);
        let HeaderParts { mut header, object } = self.header(root, "uuid", "version")?;
        self.header_tail(&mut header, object, pack_type);
        let (dependencies, dependency_places) = self.dependencies(root);
        let metadata = self.metadata(root, &path(&["metadata"]));
        let settings = self.settings(root);
        let subpacks = self.subpacks(root);
        let capability_list =
            capability_list(self.sink, root, header.uuid, pack_type, capabilities);
        let has_education_metadata =
            self.sink
                .boolean(root, "has_education_metadata", &JsonPath::new(), false);
        self.duplicate_uuids(&mut header, &modules);
        let (format, original_format) = self.manifest_format();
        Some(Manifest {
            format,
            original_format,
            header,
            modules,
            dependencies,
            capabilities: capability_list,
            metadata,
            subpacks,
            settings,
            has_education_metadata,
            extra: self.sink.unknown(
                root,
                &JsonPath::new(),
                &[
                    "format_version",
                    "header",
                    "modules",
                    "dependencies",
                    "metadata",
                    "settings",
                    "subpacks",
                    "capabilities",
                    "has_education_metadata",
                ],
            ),
            places: ManifestPlaces {
                dependencies: dependency_places,
            },
        })
    }

    /// Format 0: header (`pack_id`, `packs_version`), then modules and metadata from inside the
    /// header; dependencies, settings, subpacks and capabilities are not read.
    fn read_v0(&mut self, root: &Value) -> Option<Manifest> {
        let errors = self.sink.errors();
        let HeaderParts { mut header, object } = self.header(root, "pack_id", "packs_version")?;
        // An error from the header skips the rest.
        if self.sink.errors() > errors {
            return None;
        }
        let mut modules = self.modules(object, &path(&["header"]));
        if modules.is_empty() && member(object, "modules").is_none_or(Value::is_null) {
            // Lenient path for a manifest without modules: vanilla adds one resources module,
            // and its UUID is a hash.
            modules.push(Module::new(
                ModuleType::Resources,
                ids::hash(&format!("{}modules", header.uuid.hyphenated())),
                SemVersion::new(0, 0, 0),
            ));
        }
        let pack_type = pack_type(&modules);
        header.extra = self.sink.unknown(
            object,
            &path(&["header"]),
            &[
                "name",
                "description",
                "pack_id",
                "packs_version",
                "min_engine_version",
                "base_game_version",
                "pack_scope",
                "platform_locked",
                "allow_random_seed",
                "lock_template_options",
                "pack_optimization_version",
                "modules",
                "metadata",
            ],
        );
        // The format-0/1 `min_engine_version` default and cap apply once the pack type is
        // known.
        self.header_tail(&mut header, object, pack_type);
        let metadata = self.metadata(object, &path(&["header", "metadata"]));
        self.duplicate_uuids(&mut header, &modules);
        let (format, original_format) = self.manifest_format();
        Some(Manifest {
            format,
            original_format,
            header,
            modules,
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            metadata,
            subpacks: Vec::new(),
            settings: Vec::new(),
            has_education_metadata: None,
            extra: self
                .sink
                .unknown(root, &JsonPath::new(), &["format_version", "header"]),
            places: ManifestPlaces::default(),
        })
    }

    // ---- header ----

    /// The header members that do not depend on the pack type.
    fn header<'v>(
        &mut self,
        container: &'v Value,
        uuid_key: &str,
        version_key: &str,
    ) -> Option<HeaderParts<'v>> {
        let at = path(&["header"]);
        let object = self
            .sink
            .object(container, "header", &JsonPath::new(), true)?;
        let upgraded = self.upgraded;
        let uuid = self.sink.uuid(object, uuid_key, &at, upgraded);
        let name = self.name(object, uuid);
        let description = self.sink.string(object, "description", &at, false);
        let platform_locked = self.sink.boolean(object, "platform_locked", &at, false);
        let allow_random_seed = self.sink.boolean(object, "allow_random_seed", &at, false);
        let scope_text = self.sink.string(object, "pack_scope", &at, false);
        let version = self.sink.version(object, version_key, &at, upgraded);
        // Read only when present, without findings; kept only when valid.
        let pack_optimization_version =
            member(object, "pack_optimization_version").and_then(|value| {
                text(value)
                    .and_then(RelaxedSemVersion::from_text)
                    .or_else(|| {
                        self.sink
                            .version_array(value, &at.child("pack_optimization_version"))
                            .map(RelaxedSemVersion::new)
                    })
            });
        let (uuid, version) = (uuid?, version?);
        let mut extra = self.sink.unknown(
            object,
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
        let pack_scope = pack_scope(
            self.sink,
            scope_text,
            Code::from(&core_codes::INVALID_PROPERTY),
            &mut extra,
        );
        Some(HeaderParts {
            header: Header {
                name: name.unwrap_or_else(|| uuid.hyphenated().to_string()),
                description,
                uuid,
                version,
                min_engine_version: None,
                base_game_version: None,
                pack_scope,
                platform_locked,
                allow_random_seed,
                lock_template_options: None,
                pack_optimization_version,
                extra,
            },
            object,
        })
    }

    /// `name`: an empty name reads as the UUID in format 0 (with a warning) and as
    /// `manifestvalidation.noname` from format 1. `None` stands for "the UUID".
    fn name(&mut self, object: &Value, uuid: Option<Uuid>) -> Option<String> {
        // A header with no `name` is reported through `manifestvalidation.noname` in formats 1
        // and 2 and with the UUID in format 0, and the pack goes on loading. Format 0 also
        // adds an `empty_property:%s,%s;` warning for the header's name, and the game gives
        // no report entry for the other branch. A pack-report entry carries no content-log
        // level, so it counts as the level-2 class.
        let at = path(&["header"]);
        let read = self.sink.string(
            object,
            "name",
            &at,
            self.format >= 2 && member(object, "name").is_none_or(Value::is_null),
        );
        match read {
            Some(name) if !name.is_empty() => Some(name),
            _ if self.format >= 1 => {
                // The game's noname branch adds no report entry, so the crate keeps format 1's
                // message as an information at most.
                if self.format >= 2 && member(object, "name").is_some_and(|v| text(v) == Some("")) {
                    self.sink
                        .note(Code::from(&core_codes::EMPTY_PROPERTY), &at.child("name"));
                }
                // Kept empty, so it is written back empty; only reports show the placeholder.
                Some(String::new())
            }
            _ => {
                if member(object, "name").is_some_and(|v| text(v) == Some("")) {
                    // The format-0 branch's `empty_property` warning for the name.
                    self.sink
                        .warning(Code::from(&core_codes::EMPTY_PROPERTY), &at.child("name"));
                }
                uuid.map(|uuid| uuid.hyphenated().to_string())
            }
        }
    }

    /// The header members whose rules depend on the pack type: `min_engine_version`,
    /// `base_game_version`, `lock_template_options`.
    fn header_tail(&mut self, header: &mut Header, object: &Value, pack_type: PackType) {
        header.min_engine_version = self.min_engine_version(object, pack_type);
        if self.format < 2 {
            keep_written_min_engine_version(header, object);
        }
        header.base_game_version = self.base_game_version(object, pack_type);
        header.lock_template_options = self.lock_template_options(object, pack_type);
    }

    /// `min_engine_version`: enforced only for behaviour and resource packs, array form only.
    fn min_engine_version(
        &mut self,
        object: &Value,
        pack_type: PackType,
    ) -> Option<MinEngineVersion> {
        let at = path(&["header", "min_engine_version"]);
        let enforced = matches!(pack_type, PackType::Behavior | PackType::Resource);
        let Some(value) = member(object, "min_engine_version").filter(|v| !v.is_null()) else {
            return match (self.format >= 2, enforced) {
                (true, true) => {
                    self.sink
                        .error(Code::from(&core_codes::MISSING_PROPERTY), &at);
                    None
                }
                (false, true) => Some(MinEngineVersion::FORMAT_1_DEFAULT),
                _ => None,
            };
        };
        if self.format >= 2 && !enforced {
            // `ignored_property:%s;` is a pack-report warning with no content-log level, so
            // it counts as the level-2 class.
            self.sink
                .warning(Code::from(&core_codes::IGNORED_PROPERTY), &at);
            return None;
        }
        // The legacy parser accepts only the array form; a string like "1.20.0" is
        // `malformed_property_version`.
        let Some(version) = self.sink.version_array(value, &at) else {
            self.sink
                .error(Code::from(&core_codes::MALFORMED_PROPERTY_VERSION), &at);
            return None;
        };
        if self.format >= 2 {
            return self
                .sink
                .engine_bounds(&version, &at)
                .then_some(MinEngineVersion { version });
        }
        let version = MinEngineVersion { version };
        if version.exceeds_format_1_cap() {
            self.sink.warning(
                Code::from(&core_codes::MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1),
                &at,
            );
            return Some(MinEngineVersion::FORMAT_1_CAP);
        }
        Some(version)
    }

    /// `base_game_version`: read by world templates from format 2; ignored with a warning
    /// elsewhere.
    fn base_game_version(
        &mut self,
        object: &Value,
        pack_type: PackType,
    ) -> Option<BaseGameVersion> {
        base_game_version(
            self.sink,
            object,
            pack_type,
            self.format >= 2,
            &path(&["header", "base_game_version"]),
        )
    }

    /// `lock_template_options`: required by format-2 world templates, ignored with a warning
    /// on other format-2 packs.
    fn lock_template_options(&mut self, object: &Value, pack_type: PackType) -> Option<bool> {
        let at = path(&["header", "lock_template_options"]);
        let template = pack_type == PackType::WorldTemplate;
        let present = member(object, "lock_template_options").is_some_and(|v| !v.is_null());
        if !present {
            if self.format >= 2 && template {
                self.sink
                    .error(Code::from(&core_codes::MISSING_PROPERTY), &at);
            }
            return None;
        }
        let value =
            self.sink
                .boolean(object, "lock_template_options", &path(&["header"]), false)?;
        if template || self.format < 2 {
            Some(value)
        } else {
            // A pack-report warning, which counts as the level-2 class.
            self.sink
                .warning(Code::from(&core_codes::IGNORED_PROPERTY), &at);
            None
        }
    }

    // ---- modules ----

    /// The modules of `container.modules`.
    fn modules(&mut self, container: &Value, at: &JsonPath) -> Vec<Module> {
        let at = below(at, "modules");
        let Some(value) = member(container, "modules").filter(|v| !v.is_null()) else {
            if self.upgraded {
                self.sink
                    .warning(Code::from(&core_codes::MISSING_MODULES), &at);
            } else {
                // The strict path reports the same error as a non-array.
                self.sink
                    .error(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
            }
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink
                .error(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
            return Vec::new();
        };
        let mut modules = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
            let at = at.index(index);
            if element.as_object().is_none() {
                self.sink
                    .error(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
                continue;
            }
            if let Some(module) = self.module(element, &at) {
                modules.push(module);
            }
        }
        if !modules_consistent(&modules) {
            self.sink.error_or_upgraded(
                Code::from(&core_codes::MULTIPLE_MODULES),
                self.upgraded,
                &at,
            );
            if self.upgraded {
                // The lenient path keeps only the last module.
                let last = modules.pop();
                modules = last.into_iter().collect();
            }
        }
        modules
    }

    fn module(&mut self, object: &Value, at: &JsonPath) -> Option<Module> {
        // The module UUID and version are read strictly on every path.
        let uuid = self.sink.uuid(object, "uuid", at, false);
        let version = self.sink.version(object, "version", at, false);
        let description = self.sink.string(object, "description", at, false);
        let language = self.sink.string(object, "language", at, false);
        let entry = self.sink.string(object, "entry", at, false);
        let type_name = self
            .sink
            .string(object, "type", at, false)
            .unwrap_or_default();
        let module_type = match ModuleType::from_name(&type_name) {
            Some(ty) if ty != ModuleType::Invalid => Some(ty),
            _ => {
                let at = below(at, "type");
                if self.upgraded {
                    // `packupgradewarning.invalidpacktype`: substituted by `resources`.
                    self.sink
                        .warning(Code::from(&core_codes::INVALID_PROPERTY), &at);
                    Some(ModuleType::Resources)
                } else {
                    self.sink
                        .error(core_codes::MANIFEST_INVALID_VALUE.with([pointer(&at)]), &at);
                    None
                }
            }
        };
        Some(Module {
            module_type: module_type?,
            uuid: uuid?,
            version: version?,
            description,
            entry,
            language,
            extra: self.sink.unknown(
                object,
                at,
                &[
                    "type",
                    "uuid",
                    "version",
                    "description",
                    "entry",
                    "language",
                ],
            ),
        })
    }

    // ---- the rest of format 1 and 2 ----

    fn dependencies(&mut self, root: &Value) -> (Vec<Dependency>, Vec<DependencyPlace>) {
        let list = path(&["dependencies"]);
        // An absent `dependencies` is no finding.
        let Some(elements) = self
            .sink
            .array(root, "dependencies", &JsonPath::new(), false)
        else {
            return (Vec::new(), Vec::new());
        };
        let mut dependencies = Vec::new();
        let mut places = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            let at = list.index(index);
            if element.as_object().is_none() {
                self.sink
                    .error(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
                continue;
            }
            if let Some(dependency) = self.dependency(element, &at) {
                let native = matches!(dependency, Dependency::NativeModule { .. });
                places.push(dependency_place(element, at, native));
                dependencies.push(dependency);
            }
        }
        (dependencies, places)
    }

    fn dependency(&mut self, object: &Value, at: &JsonPath) -> Option<Dependency> {
        if member(object, "module_name").is_none_or(Value::is_null) {
            let uuid = self.sink.uuid(object, "uuid", at, false);
            let version = self.sink.version(object, "version", at, false);
            return Some(Dependency::Pack {
                uuid: uuid?,
                version: version?,
            });
        }
        if member(object, "uuid").is_some_and(|v| !v.is_null()) {
            // The legacy parser accepts the array and drops an element that has both
            // `module_name` and `uuid`, with an `InvalidProperty` warning.
            self.sink
                .warning(Code::from(&core_codes::INVALID_PROPERTY), at);
            return None;
        }
        let module_name = self.sink.string(object, "module_name", at, true)?;
        let version = match member(object, "version").and_then(text) {
            Some("alpha") => NativeModuleVersion::LatestAlpha,
            Some("beta") => NativeModuleVersion::LatestBeta,
            Some("internal") => NativeModuleVersion::LatestInternal,
            _ => NativeModuleVersion::Version(self.sink.version(object, "version", at, false)?),
        };
        Some(Dependency::NativeModule {
            module_name,
            version,
            optional: None,
        })
    }

    fn metadata(&mut self, container: &Value, at: &JsonPath) -> Option<Metadata> {
        metadata(self.sink, container, at)
    }

    /// Legacy `settings`: only labels are kept; other elements are dropped with a warning.
    fn settings(&mut self, root: &Value) -> Vec<Setting> {
        let list = path(&["settings"]);
        let Some(elements) = self.sink.array(root, "settings", &JsonPath::new(), false) else {
            return Vec::new();
        };
        let mut settings = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            if element.as_object().is_none() {
                continue;
            }
            let label_text = (member(element, "type").and_then(text) == Some("label"))
                .then(|| member(element, "text").and_then(text))
                .flatten();
            match label_text {
                Some(text) => settings.push(Setting::Label {
                    text: text.to_owned(),
                }),
                // A dropped element gets `LegacySettingIgnored`, the crate's own information;
                // vanilla drops them silently.
                None => self.sink.note(
                    Code::from(&core_codes::LEGACY_SETTING_IGNORED),
                    &list.index(index),
                ),
            }
        }
        settings
    }

    fn subpacks(&mut self, root: &Value) -> Vec<Subpack> {
        let list = path(&["subpacks"]);
        let Some(elements) = self.sink.array(root, "subpacks", &JsonPath::new(), false) else {
            return Vec::new();
        };
        let mut subpacks = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            let at = list.index(index);
            if element.as_object().is_none() {
                self.sink
                    .error(Code::from(&core_codes::WRONG_TYPE_PROPERTY), &at);
                continue;
            }
            let folder_name = self
                .sink
                .string(element, "folder_name", &at, true)
                .unwrap_or_default();
            let name = self
                .sink
                .string(element, "name", &at, true)
                .unwrap_or_default();
            let memory_tier = self.memory_tier(element, &at);
            subpacks.push(Subpack {
                name,
                folder_name,
                memory_tier,
            });
        }
        subpacks
    }

    /// `memory_tier` (a memory size mapped to a tier) or `memory_performance_tier` (0..5).
    fn memory_tier(&mut self, object: &Value, at: &JsonPath) -> MemoryTier {
        let integer = |key: &str| {
            member(object, key)
                .and_then(Value::as_number)
                .filter(|n| n.is_integer())
                .and_then(facet_value::VNumber::to_i64)
        };
        // A legacy `memory_tier` is a memory size and picks the highest tier whose threshold it
        // reaches: 32 or more gives tier 5, 24 or more tier 4, 18 or more tier 3, 12 or more
        // tier 2, 11 tier 1, and anything from 0 to 10 tier 0; a negative size is an error.
        // `memory_performance_tier` names the tier directly, an integer from 0 to 5, and a
        // larger one is rejected.
        if let Some(size) = integer("memory_tier") {
            if size < 0 {
                self.sink.error(
                    Code::from(&core_codes::INVALID_PROPERTY),
                    &below(at, "memory_tier"),
                );
            }
            return MemoryTier::from_legacy_size(size);
        }
        if let Some(tier) = integer("memory_performance_tier") {
            return u8::try_from(tier)
                .ok()
                .and_then(MemoryTier::from_number)
                .unwrap_or_else(|| {
                    self.sink.error(
                        Code::from(&core_codes::INVALID_PROPERTY),
                        &below(at, "memory_performance_tier"),
                    );
                    MemoryTier::Tier0
                });
        }
        // The game has no pack-report warning for this, so the crate reports an information.
        self.sink.note(
            Code::from(&core_codes::WRONG_TYPE_PROPERTY),
            &below(at, "memory_tier"),
        );
        MemoryTier::Tier0
    }

    /// A module whose UUID equals the header's. Format 1 and 2: an
    /// error per such module. Format 0: the header UUID is replaced by a hash of itself (once)
    /// with a warning.
    fn duplicate_uuids(&mut self, header: &mut Header, modules: &[Module]) {
        for module in modules {
            if module.uuid != header.uuid {
                continue;
            }
            if self.upgraded {
                header.uuid = ids::hash(&header.uuid.hyphenated().to_string());
                self.sink.warning(
                    Code::from(&core_codes::DUPLICATE_UUID),
                    &path(&["header", "pack_id"]),
                );
            } else {
                self.sink.error(
                    Code::from(&core_codes::DUPLICATE_UUID),
                    &path(&["header", "uuid"]),
                );
            }
        }
    }
}

/// The pack type: the last module's, or invalid without modules.
pub(super) fn pack_type(modules: &[Module]) -> PackType {
    modules
        .last()
        .map_or(PackType::Invalid, |module| module.module_type.pack_type())
}

/// Modules may not declare different pack types. Types outside vanilla's list of resource
/// types (`skin_pack`, `addon`, …) and `invalid` take part in no comparison.
pub(super) fn modules_consistent(modules: &[Module]) -> bool {
    // Each type falls in a bucket: resources and client_script in 6, data and script in 4,
    // world_template in 8, everything else in 0 (no bucket); a running bucket that reads 0 is
    // reassigned.
    let bucket = |ty: ModuleType| match ty {
        ModuleType::Resources | ModuleType::ClientScript => 6,
        ModuleType::Data | ModuleType::Script => 4,
        ModuleType::WorldTemplate => 8,
        _ => 0,
    };
    let mut running = 0;
    for module in modules {
        let next = bucket(module.module_type);
        if running == 0 {
            running = next;
        } else if next != 0 && next != running {
            return false;
        }
    }
    true
}

/// `capabilities`: each element a capability name, checked against `registry` for the pack
/// `pack_uuid` of `pack_type`; an element that is not a string is the wrong type.
pub(super) fn capability_list(
    sink: &mut Sink<'_>,
    root: &Value,
    pack_uuid: Uuid,
    pack_type: PackType,
    registry: &CapabilityRegistry,
) -> Vec<Capability> {
    let list = path(&["capabilities"]);
    let Some(elements) = sink.array(root, "capabilities", &JsonPath::new(), false) else {
        return Vec::new();
    };
    let mut capabilities = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let at = list.index(index);
        let Some(name) = text(element) else {
            let code = sink.wrong_type();
            sink.error(code, &at);
            continue;
        };
        let capability = Capability::from_name(name);
        check_capability(sink, registry, &capability, pack_uuid, pack_type, &at);
        capabilities.push(capability);
    }
    capabilities
}

/// Checks one capability against the registry: a capability the registry does not grant is a
/// warning; the name stays in the manifest.
pub(super) fn check_capability(
    sink: &mut Sink<'_>,
    registry: &CapabilityRegistry,
    capability: &Capability,
    pack_uuid: Uuid,
    pack_type: PackType,
    at: &JsonPath,
) {
    let cx = CapabilityCx {
        capability,
        pack_uuid,
        pack_type,
    };
    match registry.validate(&cx) {
        CapabilityValidation::Succeeded { .. } => {}
        CapabilityValidation::Failed | CapabilityValidation::NotFound => sink.push(
            core_codes::INVALID_CAPABILITY.with([capability.name().to_owned()]),
            // The capability registry is the crate's own API and the game has no pack-report
            // warning for it, so the crate reports an information.
            bedrock_addon_core::diagnostic::Severity::Info,
            bedrock_addon_core::diagnostic::Phase::Validate,
            at,
        ),
    }
}

/// The `base_game_version` rules, shared by both readers (`from_format_2`: the manifest is
/// format 2 or 3).
pub(super) fn base_game_version(
    sink: &mut Sink<'_>,
    object: &Value,
    pack_type: PackType,
    from_format_2: bool,
    at: &JsonPath,
) -> Option<BaseGameVersion> {
    let template = pack_type == PackType::WorldTemplate;
    let Some(value) = member(object, "base_game_version").filter(|v| !v.is_null()) else {
        if template && from_format_2 {
            sink.error(Code::from(&core_codes::MISSING_PROPERTY), at);
        }
        return None;
    };
    let parsed = match text(value) {
        Some("*") => Some(BaseGameVersion::ANY),
        Some(raw) => strict_version_text(raw).map(BaseGameVersion::new),
        None => sink.version_array(value, at).map(BaseGameVersion::new),
    };
    let Some(version) = parsed else {
        sink.error(Code::from(&core_codes::MALFORMED_PROPERTY_VERSION), at);
        return None;
    };
    if !template || !from_format_2 {
        // A pack-report warning, which counts as the level-2 class.
        sink.warning(Code::from(&core_codes::IGNORED_PROPERTY), at);
        return None;
    }
    if let Some(semver) = version.version.as_ref().filter(|v| !v.is_any())
        && !sink.engine_bounds(semver, at)
    {
        return None;
    }
    Some(version)
}

/// The members of `metadata` the readers know.
const METADATA_MEMBERS: [&str; 5] = [
    "authors",
    "generated_with",
    "license",
    "product_type",
    "url",
];

/// The `metadata` object: `url`, `license`, `product_type` (`""` or `addon`), `generated_with`
/// (versions that do not parse are dropped with a warning) and `authors`, by the sink's
/// profile: the comprehensive document reads a member written `null` as one of the wrong type
/// and refuses unknown members; the legacy parser reads `null` as absent, reads no `authors`
/// and drops unknown members.
pub(super) fn metadata(sink: &mut Sink<'_>, container: &Value, at: &JsonPath) -> Option<Metadata> {
    let strict = sink.profile() == Profile::Comprehensive;
    let value = sink.present(container, "metadata")?;
    if value.as_object().is_none() {
        let code = sink.wrong_type();
        sink.error(code, at);
        return None;
    }
    let wrong_type = |sink: &mut Sink<'_>, at: &JsonPath| {
        let code = sink.wrong_type();
        sink.error(code, at);
    };
    let string = |sink: &mut Sink<'_>, key: &str| -> Option<String> {
        let value = sink.present(value, key)?;
        let read = text(value).map(str::to_owned);
        if read.is_none() {
            wrong_type(sink, &below(at, key));
        }
        read
    };
    let url = string(sink, "url");
    let license = string(sink, "license");
    let product_type = string(sink, "product_type");
    if product_type
        .as_deref()
        .is_some_and(|p| !p.is_empty() && p != "addon")
    {
        sink.error(
            Code::from(&core_codes::INVALID_PROPERTY),
            &below(at, "product_type"),
        );
    }
    let authors = sink.present(value, "authors").and_then(|authors| {
        let list: Option<Vec<String>> = authors.as_array().and_then(|a| {
            a.as_slice()
                .iter()
                .map(|v| text(v).map(str::to_owned))
                .collect()
        });
        if list.is_none() {
            // Vanilla's legacy parser does not read `authors`, so a wrong shape is an
            // information from the crate there and an error in the strict format.
            if strict {
                wrong_type(sink, &below(at, "authors"));
            } else {
                sink.note(
                    Code::from(&core_codes::WRONG_TYPE_PROPERTY),
                    &below(at, "authors"),
                );
            }
        }
        list
    });
    let generated_with = generated_with(sink, value, at);
    if strict {
        let _ = sink.unknown(value, at, &METADATA_MEMBERS);
    } else if let Some(object) = value.as_object() {
        // The legacy parser reads no other member, and `Metadata` has no place for one: it
        // is not written back, which the load says at the member (the crate's own
        // information; the game is silent). The root, the header and a module keep theirs.
        for (key, _) in object {
            if !METADATA_MEMBERS.contains(&key.as_str()) {
                sink.push(
                    Code::from(&core_codes::MEMBER_NOT_KEPT),
                    bedrock_addon_core::diagnostic::Severity::Info,
                    bedrock_addon_core::diagnostic::Phase::Parse,
                    &below(at, key.as_str()),
                );
            }
        }
    }
    Some(Metadata {
        authors,
        license,
        url,
        product_type,
        generated_with,
    })
}

/// `metadata/generated_with`: tool name → versions; a version that does not parse is dropped
/// with a warning. The legacy parser also reads a version's array form.
fn generated_with(
    sink: &mut Sink<'_>,
    metadata: &Value,
    at: &JsonPath,
) -> Option<IndexMap<String, Vec<RelaxedSemVersion>>> {
    let tools = sink.present(metadata, "generated_with")?;
    let at = below(at, "generated_with");
    let wrong_type = |sink: &mut Sink<'_>, at: &JsonPath| {
        let code = sink.wrong_type();
        sink.error(code, at);
    };
    let Some(object) = tools.as_object() else {
        wrong_type(sink, &at);
        return None;
    };
    let mut map = IndexMap::new();
    for (tool, versions) in object {
        let at = below(&at, tool.as_str());
        let Some(list) = versions.as_array() else {
            wrong_type(sink, &at);
            continue;
        };
        let mut kept = Vec::new();
        for (index, version) in list.as_slice().iter().enumerate() {
            let parsed = text(version)
                .and_then(RelaxedSemVersion::from_text)
                .or_else(|| match sink.profile() {
                    Profile::Comprehensive => None,
                    Profile::Legacy => sink
                        .version_array(version, &at.index(index))
                        .map(RelaxedSemVersion::new),
                });
            match parsed {
                Some(parsed) => kept.push(parsed),
                // An unparsable version is an `ignored_property` pack-report warning, which
                // counts as the level-2 class.
                None => sink.warning(Code::from(&core_codes::IGNORED_PROPERTY), &at.index(index)),
            }
        }
        map.insert(tool.as_str().to_owned(), kept);
    }
    Some(map)
}

/// Keeps a format-0/1 `min_engine_version` in `extra` when the value the pack uses does not
/// write it back: a capped value (1.13.0 or above, used as 1.12.0) and an explicit 1.2.5 (the
/// writer leaves the default out). The writer puts it back at its place. A manifest is
/// written as it was read, while the typed field keeps the value the pack uses.
fn keep_written_min_engine_version(header: &mut Header, object: &Value) {
    let Some(written) = member(object, "min_engine_version").filter(|v| !v.is_null()) else {
        return;
    };
    let Some(version) = version_array(written) else {
        return;
    };
    let used = header.min_engine_version.as_ref();
    let as_written = MinEngineVersion { version };
    if used != Some(&as_written) || used == Some(&MinEngineVersion::FORMAT_1_DEFAULT) {
        header
            .extra
            .insert("min_engine_version".to_owned(), written.clone());
    }
}
