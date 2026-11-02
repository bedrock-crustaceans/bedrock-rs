//! The legacy reader: `format_version` 0, 1 and 2 (vanilla's hand-written manifest parser).
//!
//! Format 1 and 2 read modules first (the last module decides the pack type, which decides
//! the header rules), then the header, dependencies, metadata, settings, subpacks,
//! capabilities and the education flag. Format 0 reads `pack_id` / `packs_version`, takes its
//! modules and metadata from inside the header and skips everything else; it is upgraded to
//! format 1, and its UUID, version and module problems are warnings instead of errors.

use crate::diagnostic::{DiagnosticCode as Code, VanillaText};
use crate::document::Extra;
use crate::json::Value;
use crate::pack_identity::PackType;
use crate::version::{BaseGameVersion, ENGINE_VERSION, MinEngineVersion, SemVersion};
use indexmap::IndexMap;
use uuid::Uuid;

use super::capability::{CapabilityCx, CapabilityRegistry, CapabilityValidation};
use super::read::{
    Sink, child, item, member, pack_scope, strict_version_text, text, version_array,
};
use super::{
    Capability, Dependency, Header, Manifest, ManifestFormat, MemoryTier, Metadata, Module,
    ModuleType, NativeModuleVersion, RelaxedSemVersion, Setting, Subpack, ids,
};

/// What vanilla shows for a manifest without a name (the `manifestvalidation.noname` key).
pub(super) const NO_NAME: &str = "manifestvalidation.noname";

/// The lowest `min_engine_version` a format-2 behaviour or resource pack may declare.
const LOWEST_ENGINE: SemVersion = SemVersion::new(1, 13, 0);

/// Reads a legacy manifest whose `format_version` did not route to the comprehensive reader.
/// `None` when the manifest has no usable value (every such case has left an error).
pub(super) fn read(
    root: &Value,
    capabilities: &CapabilityRegistry,
    sink: &mut Sink<'_>,
) -> Option<Manifest> {
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
            Code::DeprecatedMember {
                text: Some(VanillaText::Fixed(
                    "Pack Manifest version 0 is deprecated, please include format_version 2 in your manifest.json.",
                )),
            },
            crate::diagnostic::Severity::Warning,
            crate::diagnostic::Phase::Parse,
            "format_version",
        );
        let manifest = reader.read_v0(root);
        if manifest.is_some() {
            // The game upgrades a format-0 manifest to format 1, and so it is held and
            // written: `pack_id` as `uuid`, `packs_version` as `version`, the header's
            // modules and metadata at the root. The written text is another text, which the
            // load says at the root (the crate's own information; the game upgrades in
            // silence).
            reader.sink.push(
                Code::UnusualFormatVersion,
                crate::diagnostic::Severity::Info,
                crate::diagnostic::Phase::Upgrade,
                "",
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
    let as_int: i64 = if let Some(number) = value.as_number() {
        if number.is_integer() {
            number.to_i64().unwrap_or(0)
        } else {
            // The coercion is parse semantics; the lint is the crate's own.
            sink.note(Code::WrongTypeProperty { text: None }, "format_version");
            #[allow(
                clippy::cast_possible_truncation,
                reason = "jsoncpp `asInt` truncates a real toward zero"
            )]
            let truncated = number.to_f64_lossy().trunc() as i64;
            truncated
        }
    } else if let Some(flag) = value.as_bool() {
        sink.note(Code::WrongTypeProperty { text: None }, "format_version");
        i64::from(flag)
    } else {
        sink.note(Code::WrongTypeProperty { text: None }, "format_version");
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
        sink.error(Code::InvalidProperty { text: None }, "format_version");
        return None;
    }
    if i64::from(format) != as_int {
        // A lint of the coercion, the crate's own.
        sink.note(Code::InvalidProperty { text: None }, "format_version");
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
        let modules = self.modules(root, "");
        // An error from the modules skips the header and the rest.
        if self.sink.errors() > errors {
            return None;
        }
        let pack_type = pack_type(&modules);
        let HeaderParts { mut header, object } = self.header(root, "uuid", "version")?;
        self.header_tail(&mut header, object, pack_type);
        let dependencies = self.dependencies(root);
        let metadata = self.metadata(root, "metadata");
        let settings = self.settings(root);
        let subpacks = self.subpacks(root);
        let capability_list = self.capabilities(root, &header, pack_type, capabilities);
        let has_education_metadata =
            self.boolean(root, "has_education_metadata", "has_education_metadata");
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
            extra: extra(
                root,
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
        let mut modules = self.modules(object, "header");
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
        header.extra = extra(
            object,
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
        let metadata = self.metadata(object, "header/metadata");
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
            extra: extra(root, &["format_version", "header"]),
        })
    }

    // ---- primitives ----

    /// A string member: wrong type is always an error; absent and empty are errors only when
    /// `required`. `None` unless a string was read.
    fn string(&mut self, object: &Value, key: &str, at: &str, required: bool) -> Option<String> {
        let at = child(at, key);
        match member(object, key).filter(|v| !v.is_null()) {
            None => {
                if required {
                    self.sink.error(Code::MissingProperty { text: None }, &at);
                }
                None
            }
            Some(value) => match text(value) {
                None => {
                    self.sink.error(Code::WrongTypeProperty { text: None }, &at);
                    None
                }
                Some("") => {
                    if required {
                        // An empty required string is reported as `empty_property:%s;`
                        // (skipped for an optional member), and the pack is rejected, so
                        // it is an error.
                        self.sink.error(Code::EmptyProperty, &at);
                    }
                    Some(String::new())
                }
                Some(text) => Some(text.to_owned()),
            },
        }
    }

    /// An optional boolean member; wrong type is an error.
    fn boolean(&mut self, object: &Value, key: &str, at: &str) -> Option<bool> {
        let value = member(object, key).filter(|v| !v.is_null())?;
        if let Some(flag) = value.as_bool() {
            Some(flag)
        } else {
            self.sink.error(Code::WrongTypeProperty { text: None }, at);
            None
        }
    }

    /// The pack UUID: absent, wrong type and empty are errors and give no UUID; a
    /// malformed or nil text is replaced by its hash (an error, a warning when upgrading).
    fn uuid(&mut self, object: &Value, key: &str, at: &str, upgraded: bool) -> Option<Uuid> {
        let at = child(at, key);
        let Some(value) = member(object, key).filter(|v| !v.is_null()) else {
            // Vanilla gives a pack with no UUID a random one; the crate reports
            // `missing_property` and rejects the manifest instead.
            self.sink.error(Code::MissingProperty { text: None }, &at);
            return None;
        };
        let Some(raw) = text(value) else {
            self.sink.error(Code::WrongTypeProperty { text: None }, &at);
            return None;
        };
        if raw.is_empty() {
            // An empty UUID is read through the same string reader as `string` above and
            // reported the same way, as an error.
            self.sink.error(Code::EmptyProperty, &at);
            return None;
        }
        if let Some(uuid) = ids::parse(raw) {
            return Some(uuid);
        }
        self.sink
            .error_or_upgraded(Code::MalformedPropertyUuid, upgraded, &at);
        Some(ids::salvage(raw))
    }

    /// A version member: the string form first, then the array form.
    fn version(
        &mut self,
        object: &Value,
        key: &str,
        at: &str,
        upgraded: bool,
    ) -> Option<SemVersion> {
        let at = child(at, key);
        let Some(value) = member(object, key).filter(|v| !v.is_null()) else {
            if upgraded {
                self.sink.warning(Code::MissingProperty { text: None }, &at);
                return Some(SemVersion::new(0, 0, 0));
            }
            self.sink.error(Code::MissingProperty { text: None }, &at);
            return None;
        };
        if let Some(raw) = text(value) {
            return match strict_version_text(raw) {
                Some(version) => Some(version),
                None if upgraded => {
                    self.sink
                        .warning(Code::MalformedPropertyVersion { text: None }, &at);
                    Some(SemVersion::new(1, 0, 0))
                }
                None => {
                    let text = Some(VanillaText::ManifestNotSemVer { path: pointer(&at) });
                    self.sink
                        .error(Code::MalformedPropertyVersion { text }, &at);
                    None
                }
            };
        }
        if let Some(version) = self.sink.version_array(value, &at) {
            return Some(version);
        }
        // An array of the wrong length or with an unconvertible element, or another type.
        let text = (!upgraded).then(|| VanillaText::ManifestWrongType { path: pointer(&at) });
        self.sink
            .error_or_upgraded(Code::WrongTypeProperty { text }, upgraded, &at);
        upgraded.then(|| SemVersion::new(0, 0, 0))
    }

    // ---- header ----

    /// The header members that do not depend on the pack type.
    fn header<'v>(
        &mut self,
        container: &'v Value,
        uuid_key: &str,
        version_key: &str,
    ) -> Option<HeaderParts<'v>> {
        let Some(object) = member(container, "header").filter(|v| !v.is_null()) else {
            self.sink
                .error(Code::MissingProperty { text: None }, "header");
            return None;
        };
        if object.as_object().is_none() {
            self.sink
                .error(Code::WrongTypeProperty { text: None }, "header");
            return None;
        }
        let upgraded = self.upgraded;
        let uuid = self.uuid(object, uuid_key, "header", upgraded);
        let name = self.name(object, uuid);
        let description = self.string(object, "description", "header", false);
        let platform_locked = self.boolean(object, "platform_locked", "header/platform_locked");
        let allow_random_seed =
            self.boolean(object, "allow_random_seed", "header/allow_random_seed");
        let scope_text = self.string(object, "pack_scope", "header", false);
        let version = self.version(object, version_key, "header", upgraded);
        // Read only when present, without findings; kept only when valid.
        let pack_optimization_version =
            member(object, "pack_optimization_version").and_then(|value| {
                text(value)
                    .and_then(RelaxedSemVersion::from_text)
                    .or_else(|| {
                        self.sink
                            .version_array(value, "header/pack_optimization_version")
                            .map(RelaxedSemVersion::new)
                    })
            });
        let (uuid, version) = (uuid?, version?);
        let mut extra = extra(
            object,
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
            Code::InvalidProperty { text: None },
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
        let read = self.string(
            object,
            "name",
            "header",
            self.format >= 2 && member(object, "name").is_none_or(Value::is_null),
        );
        match read {
            Some(name) if !name.is_empty() => Some(name),
            _ if self.format >= 1 => {
                // The game's noname branch adds no report entry, so the crate keeps format 1's
                // message as an information at most.
                if self.format >= 2 && member(object, "name").is_some_and(|v| text(v) == Some("")) {
                    self.sink.note(Code::EmptyProperty, "header/name");
                }
                // Kept empty, so it is written back empty; only reports show the placeholder.
                Some(String::new())
            }
            _ => {
                if member(object, "name").is_some_and(|v| text(v) == Some("")) {
                    // The format-0 branch's `empty_property` warning for the name.
                    self.sink.warning(Code::EmptyProperty, "header/name");
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
        const AT: &str = "header/min_engine_version";
        let enforced = matches!(pack_type, PackType::Behavior | PackType::Resource);
        let Some(value) = member(object, "min_engine_version").filter(|v| !v.is_null()) else {
            return match (self.format >= 2, enforced) {
                (true, true) => {
                    self.sink.error(Code::MissingProperty { text: None }, AT);
                    None
                }
                (false, true) => Some(MinEngineVersion::FORMAT_1_DEFAULT),
                _ => None,
            };
        };
        if self.format >= 2 && !enforced {
            // `ignored_property:%s;` is a pack-report warning with no content-log level, so
            // it counts as the level-2 class.
            self.sink.warning(Code::IgnoredProperty, AT);
            return None;
        }
        // The legacy parser accepts only the array form; a string like "1.20.0" is
        // `malformed_property_version`.
        let Some(version) = self.sink.version_array(value, AT) else {
            self.sink
                .error(Code::MalformedPropertyVersion { text: None }, AT);
            return None;
        };
        if self.format >= 2 {
            let engine = SemVersion::from(ENGINE_VERSION);
            if version > engine {
                self.sink.error(
                    Code::VersionTooHigh {
                        version: version.to_string(),
                        limit: engine.to_string(),
                    },
                    AT,
                );
                return None;
            }
            if version < LOWEST_ENGINE {
                self.sink.error(
                    Code::VersionTooLow {
                        version: version.to_string(),
                        limit: LOWEST_ENGINE.to_string(),
                    },
                    AT,
                );
                return None;
            }
            return Some(MinEngineVersion { version });
        }
        let version = MinEngineVersion { version };
        if version.exceeds_format_1_cap() {
            self.sink
                .warning(Code::MinEngineVersionCapFormatVersion1, AT);
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
            "header/base_game_version",
        )
    }

    /// `lock_template_options`: required by format-2 world templates, ignored with a warning
    /// on other format-2 packs.
    fn lock_template_options(&mut self, object: &Value, pack_type: PackType) -> Option<bool> {
        const AT: &str = "header/lock_template_options";
        let template = pack_type == PackType::WorldTemplate;
        let present = member(object, "lock_template_options").is_some_and(|v| !v.is_null());
        if !present {
            if self.format >= 2 && template {
                self.sink.error(Code::MissingProperty { text: None }, AT);
            }
            return None;
        }
        let value = self.boolean(object, "lock_template_options", AT)?;
        if template || self.format < 2 {
            Some(value)
        } else {
            // A pack-report warning, which counts as the level-2 class.
            self.sink.warning(Code::IgnoredProperty, AT);
            None
        }
    }

    // ---- modules ----

    /// The modules of `container.modules`.
    fn modules(&mut self, container: &Value, at: &str) -> Vec<Module> {
        let at = child(at, "modules");
        let Some(value) = member(container, "modules").filter(|v| !v.is_null()) else {
            if self.upgraded {
                self.sink.warning(Code::MissingModules, &at);
            } else {
                // The strict path reports the same error as a non-array.
                self.sink.error(Code::WrongTypeProperty { text: None }, &at);
            }
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink.error(Code::WrongTypeProperty { text: None }, &at);
            return Vec::new();
        };
        let mut modules = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
            let at = item(&at, index);
            if element.as_object().is_none() {
                self.sink.error(Code::WrongTypeProperty { text: None }, &at);
                continue;
            }
            if let Some(module) = self.module(element, &at) {
                modules.push(module);
            }
        }
        if !modules_consistent(&modules) {
            self.sink
                .error_or_upgraded(Code::MultipleModules, self.upgraded, &at);
            if self.upgraded {
                // The lenient path keeps only the last module.
                let last = modules.pop();
                modules = last.into_iter().collect();
            }
        }
        modules
    }

    fn module(&mut self, object: &Value, at: &str) -> Option<Module> {
        // The module UUID and version are read strictly on every path.
        let uuid = self.uuid(object, "uuid", at, false);
        let version = self.version(object, "version", at, false);
        let description = self.string(object, "description", at, false);
        let language = self.string(object, "language", at, false);
        let entry = self.string(object, "entry", at, false);
        let type_name = self.string(object, "type", at, false).unwrap_or_default();
        let module_type = match ModuleType::from_name(&type_name) {
            Some(ty) if ty != ModuleType::Invalid => Some(ty),
            _ => {
                let at = child(at, "type");
                if self.upgraded {
                    // `packupgradewarning.invalidpacktype`: substituted by `resources`.
                    self.sink.warning(Code::InvalidProperty { text: None }, &at);
                    Some(ModuleType::Resources)
                } else {
                    let text = Some(VanillaText::ManifestInvalidValue { path: pointer(&at) });
                    self.sink.error(Code::InvalidProperty { text }, &at);
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
            extra: extra(
                object,
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

    fn dependencies(&mut self, root: &Value) -> Vec<Dependency> {
        const AT: &str = "dependencies";
        // An absent `dependencies` is no finding.
        let Some(value) = member(root, AT).filter(|v| !v.is_null()) else {
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink.error(Code::WrongTypeProperty { text: None }, AT);
            return Vec::new();
        };
        let mut dependencies = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
            let at = item(AT, index);
            if element.as_object().is_none() {
                self.sink.error(Code::WrongTypeProperty { text: None }, &at);
                continue;
            }
            if let Some(dependency) = self.dependency(element, &at) {
                dependencies.push(dependency);
            }
        }
        dependencies
    }

    fn dependency(&mut self, object: &Value, at: &str) -> Option<Dependency> {
        if member(object, "module_name").is_none_or(Value::is_null) {
            let uuid = self.uuid(object, "uuid", at, false);
            let version = self.version(object, "version", at, false);
            return Some(Dependency::Pack {
                uuid: uuid?,
                version: version?,
            });
        }
        if member(object, "uuid").is_some_and(|v| !v.is_null()) {
            // The legacy parser accepts the array and drops an element that has both
            // `module_name` and `uuid`, with an `InvalidProperty` warning.
            self.sink.warning(Code::InvalidProperty { text: None }, at);
            return None;
        }
        let module_name = self.string(object, "module_name", at, true)?;
        let version = match member(object, "version").and_then(text) {
            Some("alpha") => NativeModuleVersion::LatestAlpha,
            Some("beta") => NativeModuleVersion::LatestBeta,
            Some("internal") => NativeModuleVersion::LatestInternal,
            _ => NativeModuleVersion::Version(self.version(object, "version", at, false)?),
        };
        Some(Dependency::NativeModule {
            module_name,
            version,
            optional: None,
        })
    }

    fn metadata(&mut self, container: &Value, at: &str) -> Option<Metadata> {
        metadata(self.sink, container, at, false)
    }

    /// Legacy `settings`: only labels are kept; other elements are dropped with a warning.
    fn settings(&mut self, root: &Value) -> Vec<Setting> {
        const AT: &str = "settings";
        let Some(value) = member(root, AT).filter(|v| !v.is_null()) else {
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink.error(Code::WrongTypeProperty { text: None }, AT);
            return Vec::new();
        };
        let mut settings = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
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
                None => self.sink.note(Code::LegacySettingIgnored, &item(AT, index)),
            }
        }
        settings
    }

    fn subpacks(&mut self, root: &Value) -> Vec<Subpack> {
        const AT: &str = "subpacks";
        let Some(value) = member(root, AT).filter(|v| !v.is_null()) else {
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink.error(Code::WrongTypeProperty { text: None }, AT);
            return Vec::new();
        };
        let mut subpacks = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
            let at = item(AT, index);
            if element.as_object().is_none() {
                self.sink.error(Code::WrongTypeProperty { text: None }, &at);
                continue;
            }
            let folder_name = self
                .string(element, "folder_name", &at, true)
                .unwrap_or_default();
            let name = self.string(element, "name", &at, true).unwrap_or_default();
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
    fn memory_tier(&mut self, object: &Value, at: &str) -> MemoryTier {
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
                    Code::InvalidProperty { text: None },
                    &child(at, "memory_tier"),
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
                        Code::InvalidProperty { text: None },
                        &child(at, "memory_performance_tier"),
                    );
                    MemoryTier::Tier0
                });
        }
        // The game has no pack-report warning for this, so the crate reports an information.
        self.sink.note(
            Code::WrongTypeProperty { text: None },
            &child(at, "memory_tier"),
        );
        MemoryTier::Tier0
    }

    fn capabilities(
        &mut self,
        root: &Value,
        header: &Header,
        pack_type: PackType,
        registry: &CapabilityRegistry,
    ) -> Vec<Capability> {
        const AT: &str = "capabilities";
        let Some(value) = member(root, AT).filter(|v| !v.is_null()) else {
            return Vec::new();
        };
        let Some(elements) = value.as_array() else {
            self.sink.error(Code::WrongTypeProperty { text: None }, AT);
            return Vec::new();
        };
        let mut capabilities = Vec::new();
        for (index, element) in elements.as_slice().iter().enumerate() {
            let Some(name) = text(element) else {
                self.sink
                    .error(Code::WrongTypeProperty { text: None }, &item(AT, index));
                continue;
            };
            let capability = Capability::from_name(name);
            check_capability(
                self.sink,
                registry,
                &capability,
                header.uuid,
                pack_type,
                &item(AT, index),
            );
            capabilities.push(capability);
        }
        capabilities
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
                self.sink.warning(Code::DuplicateUuid, "header/pack_id");
            } else {
                self.sink.error(Code::DuplicateUuid, "header/uuid");
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

/// Checks one capability against the registry: a capability the registry does not grant is a
/// warning; the name stays in the manifest.
pub(super) fn check_capability(
    sink: &mut Sink<'_>,
    registry: &CapabilityRegistry,
    capability: &Capability,
    pack_uuid: Uuid,
    pack_type: PackType,
    at: &str,
) {
    let cx = CapabilityCx {
        capability,
        pack_uuid,
        pack_type,
    };
    match registry.validate(&cx) {
        CapabilityValidation::Succeeded { .. } => {}
        CapabilityValidation::Failed | CapabilityValidation::NotFound => sink.push(
            Code::InvalidCapability {
                name: capability.name().to_owned(),
            },
            // The capability registry is the crate's own API and the game has no pack-report
            // warning for it, so the crate reports an information.
            crate::diagnostic::Severity::Info,
            crate::diagnostic::Phase::Validate,
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
    at: &str,
) -> Option<BaseGameVersion> {
    let template = pack_type == PackType::WorldTemplate;
    let Some(value) = member(object, "base_game_version").filter(|v| !v.is_null()) else {
        if template && from_format_2 {
            sink.error(Code::MissingProperty { text: None }, at);
        }
        return None;
    };
    let parsed = match text(value) {
        Some("*") => Some(BaseGameVersion::ANY),
        Some(raw) => strict_version_text(raw).map(BaseGameVersion::new),
        None => sink.version_array(value, at).map(BaseGameVersion::new),
    };
    let Some(version) = parsed else {
        sink.error(Code::MalformedPropertyVersion { text: None }, at);
        return None;
    };
    if !template || !from_format_2 {
        // A pack-report warning, which counts as the level-2 class.
        sink.warning(Code::IgnoredProperty, at);
        return None;
    }
    if let Some(semver) = version.version.as_ref().filter(|v| !v.any) {
        let engine = SemVersion::from(ENGINE_VERSION);
        if *semver > engine {
            sink.error(
                Code::VersionTooHigh {
                    version: semver.to_string(),
                    limit: engine.to_string(),
                },
                at,
            );
            return None;
        }
        if *semver < LOWEST_ENGINE {
            sink.error(
                Code::VersionTooLow {
                    version: semver.to_string(),
                    limit: LOWEST_ENGINE.to_string(),
                },
                at,
            );
            return None;
        }
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
/// (versions that do not parse are dropped with a warning) and `authors`. `strict` (format 3):
/// unknown members are errors, and a member written as `null` is a type error; otherwise
/// `null` is absent.
pub(super) fn metadata(
    sink: &mut Sink<'_>,
    container: &Value,
    at: &str,
    strict: bool,
) -> Option<Metadata> {
    let value = member(container, "metadata").filter(|v| strict || !v.is_null())?;
    if value.as_object().is_none() {
        let code = if strict {
            Code::InvalidType { text: None }
        } else {
            Code::WrongTypeProperty { text: None }
        };
        sink.error(code, at);
        return None;
    }
    let wrong_type = |sink: &mut Sink<'_>, at: &str| {
        let code = if strict {
            Code::InvalidType { text: None }
        } else {
            Code::WrongTypeProperty { text: None }
        };
        sink.error(code, at);
    };
    let string = |sink: &mut Sink<'_>, key: &str| -> Option<String> {
        let value = member(value, key).filter(|v| strict || !v.is_null())?;
        let read = text(value).map(str::to_owned);
        if read.is_none() {
            wrong_type(sink, &child(at, key));
        }
        read
    };
    let url = string(sink, "url");
    let license = string(sink, "license");
    let product_type = string(sink, "product_type");
    if let Some(product) = product_type
        .as_deref()
        .filter(|p| !p.is_empty() && *p != "addon")
    {
        let _ = product;
        sink.error(
            Code::InvalidProperty { text: None },
            &child(at, "product_type"),
        );
    }
    let authors = member(value, "authors")
        .filter(|v| strict || !v.is_null())
        .and_then(|authors| {
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
                    wrong_type(sink, &child(at, "authors"));
                } else {
                    sink.note(
                        Code::WrongTypeProperty { text: None },
                        &child(at, "authors"),
                    );
                }
            }
            list
        });
    let generated_with = generated_with(sink, value, at, strict);
    if strict {
        super::comprehensive::unknown_members(sink, value, at, &METADATA_MEMBERS);
    } else if let Some(object) = value.as_object() {
        // The legacy parser reads no other member, and `Metadata` has no place for one: it
        // is not written back, which the load says at the member (the crate's own
        // information; the game is silent). The root, the header and a module keep theirs.
        for (key, _) in object {
            if !METADATA_MEMBERS.contains(&key.as_str()) {
                sink.push(
                    Code::MemberNotKept { text: None },
                    crate::diagnostic::Severity::Info,
                    crate::diagnostic::Phase::Parse,
                    &child(at, key.as_str()),
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

/// The members of `object` outside `known`, in document order, for write-back.
pub(super) fn extra(object: &Value, known: &[&str]) -> Extra {
    let mut extra = Extra::default();
    if let Some(object) = object.as_object() {
        for (key, value) in object {
            if !known.contains(&key.as_str()) {
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
    }
    extra
}

/// `metadata/generated_with`: tool name → versions; a version that does not parse is dropped
/// with a warning.
fn generated_with(
    sink: &mut Sink<'_>,
    metadata: &Value,
    at: &str,
    strict: bool,
) -> Option<IndexMap<String, Vec<RelaxedSemVersion>>> {
    let tools = member(metadata, "generated_with").filter(|v| strict || !v.is_null())?;
    let at = child(at, "generated_with");
    let wrong_type = |sink: &mut Sink<'_>, at: &str| {
        let code = if strict {
            Code::InvalidType { text: None }
        } else {
            Code::WrongTypeProperty { text: None }
        };
        sink.error(code, at);
    };
    let Some(object) = tools.as_object() else {
        wrong_type(sink, &at);
        return None;
    };
    let mut map = IndexMap::new();
    for (tool, versions) in object {
        let at = child(&at, tool.as_str());
        let Some(list) = versions.as_array() else {
            wrong_type(sink, &at);
            continue;
        };
        let mut kept = Vec::new();
        for (index, version) in list.as_slice().iter().enumerate() {
            let parsed = text(version)
                .and_then(RelaxedSemVersion::from_text)
                .or_else(|| {
                    if strict {
                        None
                    } else {
                        sink.version_array(version, &item(&at, index))
                            .map(RelaxedSemVersion::new)
                    }
                });
            match parsed {
                Some(parsed) => kept.push(parsed),
                // An unparsable version is an `ignored_property` pack-report warning, which
                // counts as the level-2 class.
                None => sink.warning(Code::IgnoredProperty, &item(&at, index)),
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

/// The element at `at` as the legacy reader's messages name it: a JSON pointer
/// (`/dependencies/0/version`).
fn pointer(at: &str) -> String {
    format!("/{at}")
}
