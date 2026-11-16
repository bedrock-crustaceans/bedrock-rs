//! The comprehensive reader: `format_version` 3, vanilla's cereal manifest document, declared
//! below in the order the game's document reads it and read by the derived reader in the
//! comprehensive dialect (`parse::DIALECTS`), then turned into the pack's manifest.
//!
//! Every member is typed: an unknown member, a missing required one or a value of the wrong
//! JSON type is an error, `null` is a value of the wrong type, and versions are strings. The
//! pack-type rules of the header are the legacy format-2 rules: they follow the modules, which
//! the document reads first and holds in scope for the members after them (`schema::scope`), as
//! it holds the header for the checks of the capabilities and the settings.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Profile, Read, UnknownAt};
use bedrock_addon_core::pack_identity::{PackScope, PackType};
use bedrock_addon_core::schema::{self, ElementFailure, FromJson, Schema, read_list};
use bedrock_addon_core::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use facet::Facet;
use indexmap::IndexMap;
use uuid::Uuid;

use super::read::{
    Dependencies, at, dependency_place, engine_bounds, lacking, pack_type, relaxed_version_text,
    strict_version_text,
};
use super::{
    Capability, Dependency, Header, Manifest, ManifestFormat, ManifestPlaces, MemoryTier, Metadata,
    Module, ModuleType, NativeModuleVersion, RelaxedSemVersion, Setting, Subpack, ids, legacy,
};

/// The document, its members in the game's order: the modules decide the pack type the
/// header's rules follow, the header's UUID and engine version the checks of the capabilities
/// and the settings.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "report")]
pub(super) struct Document {
    /// Bound as a `u8` and read strictly: a non-negative integer below 256; the routing has
    /// already read it as 3.0.0.
    #[facet(
        schema::invalid = core_codes::INVALID_TYPE,
        schema::negative = core_codes::INVALID_TYPE
    )]
    format_version: u8,
    #[facet(opaque, schema::scope, schema::read_with = modules)]
    modules: Vec<Module>,
    #[facet(schema::scope)]
    header: Header3,
    #[facet(opaque, default, schema::read_with = dependencies)]
    dependencies: Dependencies,
    #[facet(opaque, default, schema::read_with = super::read::capabilities)]
    capabilities: Vec<Capability>,
    #[facet(opaque, default, schema::read_with = super::setting::read)]
    settings: Vec<Setting>,
    #[facet(default, schema::drop)]
    subpacks: Vec<Subpack3>,
    #[facet(schema::read_with = metadata)]
    metadata: Option<Metadata3>,
    has_education_metadata: Option<bool>,
}

/// The header, its members in the game's order.
#[derive(Clone, Facet, Schema)]
#[facet(schema::unknown = "report")]
pub(super) struct Header3 {
    #[facet(schema::read_with = name)]
    name: String,
    description: Option<String>,
    #[facet(opaque, schema::read_with = uuid)]
    uuid: Uuid,
    #[facet(opaque, schema::read_with = version)]
    version: SemVersion,
    /// The format-2 rules: required by behaviour and resource packs, between 1.13.0 and the
    /// engine version; ignored with a warning elsewhere.
    #[facet(
        opaque,
        schema::read_with = min_engine_version,
        schema::absent_with = min_engine_version_absent
    )]
    min_engine_version: Option<MinEngineVersion>,
    #[facet(
        opaque,
        schema::read_with = base_game_version,
        schema::absent_with = base_game_version_absent
    )]
    base_game_version: Option<BaseGameVersion>,
    /// The text: a name that is no scope is kept in `extra`.
    #[facet(schema::read_with = pack_scope)]
    pack_scope: Option<String>,
    platform_locked: Option<bool>,
    allow_random_seed: Option<bool>,
    #[facet(
        schema::read_with = lock_template_options,
        schema::absent_with = lock_template_options_absent
    )]
    lock_template_options: Option<bool>,
    /// The text: one that is no relaxed version is dropped in silence.
    pack_optimization_version: Option<String>,
}

/// One `modules[]` element, its members in the game's order.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "report")]
struct Module3 {
    #[facet(rename = "type", opaque, schema::read_with = module_type)]
    module_type: ModuleType,
    #[facet(opaque, schema::read_with = uuid)]
    uuid: Uuid,
    #[facet(opaque, schema::read_with = version)]
    version: SemVersion,
    description: Option<String>,
    language: Option<String>,
    entry: Option<String>,
}

/// A pack dependency: an element without `module_name`.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "report")]
struct PackDependency {
    #[facet(opaque, schema::read_with = uuid)]
    uuid: Uuid,
    #[facet(opaque, schema::read_with = version)]
    version: SemVersion,
}

/// A native script-module dependency: an element with `module_name` (format 3 refuses a `uuid`
/// beside it as an unknown member).
#[derive(Facet, Schema)]
#[facet(schema::unknown = "report")]
struct NativeDependency {
    #[facet(schema::scope)]
    module_name: String,
    optional: Option<bool>,
    #[facet(opaque, schema::read_with = native_version)]
    version: NativeModuleVersion,
}

/// One `subpacks[]` element.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "report")]
struct Subpack3 {
    folder_name: String,
    name: String,
    #[facet(opaque, schema::read_with = memory_performance_tier)]
    memory_performance_tier: MemoryTier,
}

/// The `metadata` object; its unknown members are told after its members.
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "report")]
struct Metadata3 {
    url: Option<String>,
    license: Option<String>,
    #[facet(schema::read_with = super::read::product_type)]
    product_type: Option<String>,
    #[facet(schema::read_with = authors)]
    authors: Option<Vec<String>>,
    /// The texts of the versions kept.
    #[facet(schema::read_with = generated_with)]
    generated_with: Option<IndexMap<String, Vec<String>>>,
}

bedrock_addon_core::impl_schema_leaf!(
    Module,
    Dependencies,
    Capability,
    Setting,
    ModuleType,
    NativeModuleVersion,
    MemoryTier,
    RelaxedSemVersion,
);
bedrock_addon_core::impl_no_molang!(
    Module,
    Dependencies,
    Capability,
    Setting,
    ModuleType,
    NativeModuleVersion,
    MemoryTier,
    RelaxedSemVersion,
);

/// The manifest of a document read without an error.
pub(super) fn manifest(document: Document) -> Manifest {
    let Document {
        modules,
        header,
        dependencies,
        capabilities,
        settings,
        subpacks,
        metadata,
        has_education_metadata,
        format_version: _,
    } = document;
    Manifest {
        format: ManifestFormat::V3,
        original_format: ManifestFormat::V3,
        header: header.into_header(),
        modules,
        dependencies: dependencies.list,
        capabilities,
        metadata: metadata.map(Metadata3::into_metadata),
        subpacks: subpacks.into_iter().map(Subpack3::into_subpack).collect(),
        settings,
        has_education_metadata,
        extra: UnknownMembers::default(),
        places: ManifestPlaces {
            dependencies: dependencies.places,
        },
    }
}

impl Header3 {
    /// The pack's UUID.
    pub(super) fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn into_header(self) -> Header {
        let mut extra = UnknownMembers::default();
        let pack_scope = self.pack_scope.and_then(|text| {
            let scope = PackScope::from_json_name(&text);
            if scope.is_none() {
                // Written back as it was.
                extra.insert("pack_scope".to_owned(), Value::from(text.as_str()));
            }
            scope
        });
        Header {
            name: self.name,
            description: self.description,
            uuid: self.uuid,
            version: self.version,
            min_engine_version: self.min_engine_version,
            base_game_version: self.base_game_version,
            pack_scope,
            platform_locked: self.platform_locked,
            allow_random_seed: self.allow_random_seed,
            lock_template_options: self.lock_template_options,
            pack_optimization_version: self
                .pack_optimization_version
                .as_deref()
                .and_then(RelaxedSemVersion::from_text),
            extra,
        }
    }
}

impl Subpack3 {
    fn into_subpack(self) -> Subpack {
        Subpack {
            name: self.name,
            folder_name: self.folder_name,
            memory_tier: self.memory_performance_tier,
        }
    }
}

impl Metadata3 {
    fn into_metadata(self) -> Metadata {
        Metadata {
            authors: self.authors,
            license: self.license,
            url: self.url,
            product_type: self.product_type,
            generated_with: self.generated_with.map(|tools| {
                tools
                    .into_iter()
                    .map(|(tool, versions)| {
                        let versions = versions
                            .iter()
                            .filter_map(|text| RelaxedSemVersion::from_text(text))
                            .collect();
                        (tool, versions)
                    })
                    .collect()
            }),
        }
    }
}

// ---- what the read reads with ----

/// The header the document read, if it read one.
fn header_read<'r>(read: &'r Read<'_>) -> Option<&'r Header3> {
    read.scope::<Header3>("header")
}

/// `modules`: at least one; the elements that read, each UUID once, of one pack type.
fn modules(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Module>> {
    let mut seen = HashSet::new();
    let modules = read_list(node, read, ElementFailure::Drop, |element, read| {
        let module = Module3::from_json(element, read)?;
        if !seen.insert(module.uuid)
            && let Some(uuid) = read.object(element).and_then(|members| members.get("uuid"))
        {
            // The game's comprehensive-manifest check reports "duplicate module uuid found: …".
            read.error(&uuid, Code::from(&core_codes::DUPLICATE_UUID));
        }
        Some(Module {
            module_type: module.module_type,
            uuid: module.uuid,
            version: module.version,
            description: module.description,
            entry: module.entry,
            language: module.language,
            extra: UnknownMembers::default(),
        })
    })?;
    if node
        .value()
        .as_array()
        .is_some_and(facet_value::VArray::is_empty)
    {
        // The schema needs at least one module, and the game asserts that one exists.
        read.error(node, Code::from(&core_codes::INVALID_VALUE));
    }
    if !legacy::modules_consistent(&modules) {
        read.error(node, Code::from(&core_codes::MULTIPLE_MODULES));
    }
    Some(modules)
}

/// A module's `type`: a name of the table other than `invalid`. The game names the member
/// without its element's index.
fn module_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<ModuleType> {
    let name = String::from_json(node, read)?;
    let found = ModuleType::from_name(&name).filter(|ty| *ty != ModuleType::Invalid);
    if found.is_none() {
        read.refuse(
            node,
            core_codes::MANIFEST_INVALID_VALUE.with(["modules/type"]),
        );
    }
    found
}

/// A UUID: a malformed or nil text (the empty one too) is replaced by its hash, with
/// `MalformedPropertyUuid`.
fn uuid(node: &Node<'_>, read: &mut Read<'_>) -> Option<Uuid> {
    let text = String::from_json(node, read)?;
    Some(ids::parse(&text).unwrap_or_else(|| {
        read.error(node, Code::from(&core_codes::MALFORMED_PROPERTY_UUID));
        ids::salvage(&text)
    }))
}

/// A version: a relaxed string (`"2.0"`); the any-version `*` and `beta` are malformed.
fn version(node: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
    let text = String::from_json(node, read)?;
    let version = relaxed_version_text(&text);
    if version.is_none() {
        read.refuse(node, Code::from(&core_codes::MALFORMED_PROPERTY_VERSION));
    }
    version
}

/// The header's `name`: an empty one is reported through `manifestvalidation.noname` from format
/// 2; it stays empty and only reports show the placeholder. The format-3 reader has no report
/// entry for it, so the game says nothing and the crate reports an information.
fn name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let name = String::from_json(node, read)?;
    if name.is_empty() {
        read.push(at(node, &core_codes::EMPTY_PROPERTY, Severity::Info));
    }
    Some(name)
}

/// Whether the pack type enforces a `min_engine_version`: behaviour and resource packs.
fn enforces_engine(read: &Read<'_>) -> bool {
    matches!(pack_type(read), PackType::Behavior | PackType::Resource)
}

fn min_engine_version_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<MinEngineVersion> {
    if enforces_engine(read) {
        read.push(lacking(
            object,
            "min_engine_version",
            &core_codes::MISSING_PROPERTY,
        ));
    }
    None
}

fn min_engine_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<MinEngineVersion> {
    let text = String::from_json(node, read)?;
    if !enforces_engine(read) {
        // A `min_engine_version` on a pack that does not enforce one (the legacy reader
        // reports it too) is a pack-report warning, which counts as the level-2 class. The
        // report does not identify the member.
        read.push(at(node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
        return None;
    }
    let Some(version) = relaxed_version_text(&text) else {
        read.refuse(node, Code::from(&core_codes::MALFORMED_PROPERTY_VERSION));
        return None;
    };
    engine_bounds(&version, node, read).then_some(MinEngineVersion { version })
}

fn base_game_version_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<BaseGameVersion> {
    if pack_type(read) == PackType::WorldTemplate {
        read.push(lacking(
            object,
            "base_game_version",
            &core_codes::MISSING_PROPERTY,
        ));
    }
    None
}

/// `base_game_version`, read by world templates: `*` or a strict version string between 1.13.0
/// and the engine version; ignored with a warning elsewhere.
fn base_game_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<BaseGameVersion> {
    let text = String::from_json(node, read)?;
    let parsed = match text.as_str() {
        "*" => Some(BaseGameVersion::ANY),
        raw => strict_version_text(raw).map(BaseGameVersion::new),
    };
    let Some(version) = parsed else {
        read.refuse(node, Code::from(&core_codes::MALFORMED_PROPERTY_VERSION));
        return None;
    };
    if pack_type(read) != PackType::WorldTemplate {
        // A pack-report warning, which counts as the level-2 class.
        read.push(at(node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
        return None;
    }
    match version.version.as_ref().filter(|v| !v.is_any()) {
        Some(semver) if !engine_bounds(semver, node, read) => None,
        _ => Some(version),
    }
}

/// `pack_scope`: a vanilla spelling (`global`, `world`, `any`) is its scope. Other text reads as
/// no scope (the default, any) and is written back as it was; vanilla reads it in silence, the
/// crate reports an information.
fn pack_scope(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = String::from_json(node, read)?;
    if PackScope::from_json_name(&text).is_none() {
        read.push(at(node, &core_codes::INVALID_VALUE, Severity::Info));
    }
    Some(text)
}

fn lock_template_options_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    if pack_type(read) == PackType::WorldTemplate {
        read.push(lacking(
            object,
            "lock_template_options",
            &core_codes::MISSING_PROPERTY,
        ));
    }
    None
}

/// `lock_template_options`: a world template's; on another pack a pack-report warning, which
/// counts as the level-2 class, and not kept.
fn lock_template_options(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let lock = bool::from_json(node, read)?;
    if pack_type(read) == PackType::WorldTemplate {
        return Some(lock);
    }
    read.push(at(node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
    None
}

/// `dependencies`: each element a pack (no `module_name`) or a native module; the elements that
/// read are kept, with where each is in the file.
fn dependencies(node: &Node<'_>, read: &mut Read<'_>) -> Option<Dependencies> {
    let mut places = Vec::new();
    let list = read_list(node, read, ElementFailure::Drop, |element, read| {
        let native = read.object(element)?.contains("module_name");
        let dependency = if native {
            let NativeDependency {
                module_name,
                optional,
                version,
            } = NativeDependency::from_json(element, read)?;
            Dependency::NativeModule {
                module_name,
                version,
                optional,
            }
        } else {
            let PackDependency { uuid, version } = PackDependency::from_json(element, read)?;
            Dependency::Pack { uuid, version }
        };
        places.push(dependency_place(
            element.value(),
            element.path().clone(),
            native,
        ));
        Some(dependency)
    })?;
    Some(Dependencies { list, places })
}

/// A script-module dependency's `version`: `alpha`, `beta`, `internal` or a relaxed version
/// string (`"2.0"` from 1.26.30); `"*"` and other non-versions are malformed, told as "dependency
/// {} invalid version: {}" with the version the setter left, `0.0.0`.
fn native_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<NativeModuleVersion> {
    let text = String::from_json(node, read)?;
    let version = match text.as_str() {
        "alpha" => Some(NativeModuleVersion::LatestAlpha),
        "beta" => Some(NativeModuleVersion::LatestBeta),
        "internal" => Some(NativeModuleVersion::LatestInternal),
        raw => relaxed_version_text(raw).map(NativeModuleVersion::Version),
    };
    if version.is_none() {
        let code = read.scope::<String>("module_name").map_or_else(
            || Code::from(&core_codes::MALFORMED_PROPERTY_VERSION),
            |module| core_codes::DEPENDENCY_INVALID_VERSION.with([module.as_str(), "0.0.0"]),
        );
        read.refuse(node, code);
    }
    version
}

/// The header's engine version, for the settings' engine gate.
pub(super) fn min_engine(read: &Read<'_>) -> Option<MinEngineVersion> {
    header_read(read).and_then(|header| header.min_engine_version.clone())
}

/// A subpack's `memory_performance_tier`: an integer from 0 to 5 (the schema's).
fn memory_performance_tier(node: &Node<'_>, read: &mut Read<'_>) -> Option<MemoryTier> {
    let Some(number) = node
        .value()
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_i64)
    else {
        read.wrong_type(node);
        return None;
    };
    let tier = u8::try_from(number).ok().and_then(MemoryTier::from_number);
    if tier.is_none() {
        read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
    }
    tier
}

/// `metadata`, its unknown members told after its members.
fn metadata(node: &Node<'_>, read: &mut Read<'_>) -> Option<Metadata3> {
    let mut last = read
        .fork()
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last));
    let metadata = Metadata3::from_json(node, &mut last);
    read.commit(last);
    metadata
}

/// `metadata/authors`: a list of strings; another value, or an element that is no string, is the
/// wrong type of the member.
fn authors(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let authors = node.value().as_array().and_then(|list| {
        list.as_slice()
            .iter()
            .map(|author| author.as_string().map(|text| text.as_str().to_owned()))
            .collect::<Option<Vec<_>>>()
    });
    if authors.is_none() {
        read.wrong_type(node);
    }
    authors
}

/// `metadata/generated_with`: tool name → versions; a tool whose versions are no list is the
/// wrong type and left out, a version that is no relaxed version string is dropped with an
/// `ignored_property` pack-report warning, which counts as the level-2 class.
fn generated_with(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Vec<String>>> {
    let members = read.object(node)?;
    let mut tools = IndexMap::new();
    for (tool, versions) in members.iter() {
        let kept = read_list(&versions, read, ElementFailure::Drop, |version, read| {
            let text = version
                .value()
                .as_string()
                .map(facet_value::VString::as_str)
                .filter(|text| RelaxedSemVersion::from_text(text).is_some());
            if text.is_none() {
                read.push(at(
                    version,
                    &core_codes::IGNORED_PROPERTY,
                    Severity::Warning,
                ));
            }
            text.map(str::to_owned)
        });
        if let Some(kept) = kept {
            tools.insert(tool.to_owned(), kept);
        }
    }
    Some(tools)
}
