//! The legacy reader: `format_version` 0, 1 and 2 (vanilla's hand-written manifest parser),
//! declared below in the order the parser reads and read by the derived reader in the legacy
//! dialect (`parse`'s): a member written `null` is absent, unknown members are kept for
//! write-back in silence, a member that fails is an error and absent, and the codes are the pack
//! report's (`WrongTypeProperty`, `MissingProperty`). The read knows the format (0, 1 or 2) and
//! each hook follows it.
//!
//! Format 1 and 2 read modules first (the last module decides the pack type, which decides
//! the header rules), then the header, dependencies, metadata, settings, subpacks,
//! capabilities and the education flag; an error from the modules, or a header without a UUID
//! or a version, ends the read. Format 0 reads `pack_id` / `packs_version`, takes its modules
//! and metadata from inside the header and skips everything else; it is upgraded to format 1,
//! and its UUID, version and module problems are warnings instead of errors.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Members, Node, Read};
use bedrock_addon_core::pack_identity::{PackScope, PackType};
use bedrock_addon_core::schema::{self, ElementFailure, FromJson, Schema, read_list};
use bedrock_addon_core::version::{BaseGameVersion, MinEngineVersion, SemVersion};
use facet::Facet;
use indexmap::IndexMap;
use uuid::Uuid;

use super::read::{
    Dependencies, at, capabilities, dependency_place, engine_bounds, errors, lacking, pointer,
    product_type, strict_version_text,
};
use super::{
    Capability, Dependency, Header, Manifest, ManifestFormat, ManifestPlaces, MemoryTier, Metadata,
    Module, ModuleType, NativeModuleVersion, RelaxedSemVersion, Setting, Subpack, ids,
};

/// What vanilla shows for a manifest without a name (the `manifestvalidation.noname` key).
pub(super) const NO_NAME: &str = "manifestvalidation.noname";

/// A format-1 or format-2 manifest, its members in the parser's order.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent", schema::after_read = duplicate_uuids)]
pub(super) struct Document12 {
    /// Read by the routing.
    #[facet(schema::read_with = schema::read_later)]
    format_version: Option<bool>,
    /// An error here ends the read.
    #[facet(
        opaque,
        schema::scope,
        schema::read_with = modules,
        schema::absent_with = modules_absent
    )]
    modules: Option<Vec<Module>>,
    /// A header that is not read ends the read.
    #[facet(
        schema::scope,
        schema::read_with = header,
        schema::absent_with = header_absent
    )]
    header: Option<Header12>,
    #[facet(opaque, default, schema::read_with = dependencies)]
    dependencies: Dependencies,
    #[facet(schema::read_with = metadata)]
    metadata: Option<Metadata0>,
    #[facet(opaque, default, schema::read_with = settings)]
    settings: Vec<Setting>,
    #[facet(opaque, default, schema::read_with = subpacks)]
    subpacks: Vec<Subpack>,
    #[facet(opaque, default, schema::read_with = capabilities)]
    capabilities: Vec<Capability>,
    has_education_metadata: Option<bool>,
    extra: UnknownMembers,
}

/// A format-0 manifest: the header holds the modules and the metadata.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
pub(super) struct Document0 {
    /// Read by the routing.
    #[facet(schema::read_with = schema::read_later)]
    format_version: Option<bool>,
    #[facet(opaque, schema::read_with = header_v0, schema::absent_with = header_absent)]
    header: Option<Read0>,
    extra: UnknownMembers,
}

/// A format-1 or format-2 header, its members in the parser's order; the members whose rules
/// follow the pack type, and `pack_optimization_version`, are read after the others
/// (`after_read`), once the header has its UUID and version.
#[derive(Clone, Facet, Schema)]
#[facet(schema::unknown = "silent", schema::after_read = header_rest)]
pub(super) struct Header12 {
    #[facet(opaque, schema::required, schema::read_with = header_uuid)]
    uuid: Option<Uuid>,
    #[facet(schema::read_with = name, schema::absent_with = name_absent)]
    name: Option<String>,
    description: Option<String>,
    platform_locked: Option<bool>,
    allow_random_seed: Option<bool>,
    pack_scope: Option<String>,
    #[facet(opaque, schema::required, schema::read_with = header_version)]
    version: Option<SemVersion>,
    // Read after the others, into `rest`.
    #[facet(schema::read_with = schema::read_later)]
    pack_optimization_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    min_engine_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    base_game_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    lock_template_options: Option<Value>,
    #[facet(skip, opaque)]
    rest: Rest,
    extra: UnknownMembers,
}

/// A format-0 header, its members in the parser's order: `pack_id` and `packs_version` for
/// `uuid` and `version`, the modules and the metadata inside it (read by `header_v0`).
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct Header0 {
    #[facet(opaque, schema::required, schema::read_with = header_uuid)]
    pack_id: Option<Uuid>,
    #[facet(schema::read_with = name, schema::absent_with = name_absent)]
    name: Option<String>,
    description: Option<String>,
    platform_locked: Option<bool>,
    allow_random_seed: Option<bool>,
    pack_scope: Option<String>,
    #[facet(
        opaque,
        schema::read_with = header_version,
        schema::absent_with = packs_version_absent
    )]
    packs_version: Option<SemVersion>,
    // Read after the others, into `rest`.
    #[facet(schema::read_with = schema::read_later)]
    pack_optimization_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    min_engine_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    base_game_version: Option<Value>,
    #[facet(schema::read_with = schema::read_later)]
    lock_template_options: Option<Value>,
    #[facet(skip, opaque)]
    rest: Rest,
    #[facet(schema::read_with = schema::read_later)]
    modules: Option<bool>,
    #[facet(schema::read_with = schema::read_later)]
    metadata: Option<bool>,
    extra: UnknownMembers,
}

/// The header members read after the others: by the format-1/2 header's `after_read`, by the
/// format-0 header's reader once its modules are read.
#[derive(Clone, Default)]
struct Rest {
    pack_optimization_version: Option<RelaxedSemVersion>,
    min_engine_version: Option<MinEngineVersion>,
    base_game_version: Option<BaseGameVersion>,
    lock_template_options: Option<bool>,
}

/// One `modules[]` element; the UUID and the version are read strictly in every format.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct Module0 {
    #[facet(opaque, schema::required, schema::read_with = strict_uuid)]
    uuid: Option<Uuid>,
    #[facet(opaque, schema::required, schema::read_with = strict_version)]
    version: Option<SemVersion>,
    description: Option<String>,
    language: Option<String>,
    entry: Option<String>,
    /// An absent or invalid type is told as one; format 0 reads `resources` for it.
    #[facet(
        rename = "type",
        opaque,
        schema::read_with = module_type,
        schema::absent_with = module_type_absent
    )]
    module_type: Option<ModuleType>,
    extra: UnknownMembers,
}

/// A pack dependency: an element whose `module_name` is absent or `null`.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct PackDependency0 {
    #[facet(opaque, schema::required, schema::read_with = strict_uuid)]
    uuid: Option<Uuid>,
    #[facet(opaque, schema::required, schema::read_with = strict_version)]
    version: Option<SemVersion>,
    /// The members the parser reads past, which this element keeps none of (dropped).
    dropped: UnknownMembers,
}

/// A native script-module dependency. The legacy parser reads no `optional`.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct NativeDependency0 {
    #[facet(schema::scope, schema::read_with = non_empty)]
    module_name: String,
    #[facet(opaque, schema::required, schema::read_with = native_version)]
    version: Option<NativeModuleVersion>,
    /// The members the parser reads past, which this element keeps none of (dropped).
    dropped: UnknownMembers,
}

/// One `subpacks[]` element's names; the element's hook reads its tier.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct SubpackNames {
    #[facet(schema::required, schema::read_with = non_empty)]
    folder_name: Option<String>,
    #[facet(schema::required, schema::read_with = non_empty)]
    name: Option<String>,
    /// The members the parser reads past, which this element keeps none of (dropped).
    dropped: UnknownMembers,
}

/// The `metadata` object. The legacy parser reads no `authors`, and no other member, which
/// `Metadata` has no place for (`after_read`).
#[derive(Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent", schema::after_read = members_not_kept)]
struct Metadata0 {
    url: Option<String>,
    license: Option<String>,
    #[facet(schema::read_with = product_type)]
    product_type: Option<String>,
    #[facet(schema::read_with = authors)]
    authors: Option<Vec<String>>,
    /// The versions kept, as written (a string or the array form).
    #[facet(schema::read_with = generated_with)]
    generated_with: Option<IndexMap<String, Vec<Value>>>,
    /// The members the parser reads past, which this element keeps none of (dropped).
    dropped: UnknownMembers,
}

/// What a format-0 header gave: the header and the modules and metadata inside it.
struct Read0 {
    header: Header,
    modules: Vec<Module>,
    metadata: Option<Metadata>,
}

bedrock_addon_core::impl_schema_leaf!(Read0, Subpack);
bedrock_addon_core::impl_no_molang!(Read0, Subpack);

// ---- the documents into the manifest ----

/// The manifest of a format-1 or format-2 document read without the read ending.
pub(super) fn manifest(document: Document12, format: u8) -> Option<Manifest> {
    let header = document.header?.into_header()?;
    let format = if format == 1 {
        ManifestFormat::V1
    } else {
        ManifestFormat::V2
    };
    Some(Manifest {
        format,
        original_format: format,
        header,
        modules: document.modules.unwrap_or_default(),
        dependencies: document.dependencies.list,
        capabilities: document.capabilities,
        metadata: document.metadata.map(Metadata0::into_metadata),
        subpacks: document.subpacks,
        settings: document.settings,
        has_education_metadata: document.has_education_metadata,
        extra: document.extra,
        places: ManifestPlaces {
            dependencies: document.dependencies.places,
        },
    })
}

/// The manifest of a format-0 document: upgraded to format 1, `pack_id` read as `uuid` and
/// `packs_version` as `version`, the header's modules and metadata at the root.
pub(super) fn manifest_v0(document: Document0) -> Option<Manifest> {
    let Read0 {
        header,
        modules,
        metadata,
    } = document.header?;
    Some(Manifest {
        format: ManifestFormat::V1,
        original_format: ManifestFormat::V0,
        header,
        modules,
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        metadata,
        subpacks: Vec::new(),
        settings: Vec::new(),
        has_education_metadata: None,
        extra: document.extra,
        places: ManifestPlaces::default(),
    })
}

impl Header12 {
    /// The pack's UUID, when the header has one.
    pub(super) fn uuid(&self) -> Option<Uuid> {
        self.uuid
    }

    fn into_header(self) -> Option<Header> {
        let uuid = self.uuid?;
        Some(Header {
            // An empty or absent name stays empty from format 1, so it is written back empty;
            // only reports show the placeholder.
            name: self.name.unwrap_or_default(),
            description: self.description,
            uuid,
            version: self.version?,
            min_engine_version: self.rest.min_engine_version,
            base_game_version: self.rest.base_game_version,
            pack_scope: self
                .pack_scope
                .as_deref()
                .and_then(PackScope::from_json_name),
            platform_locked: self.platform_locked,
            allow_random_seed: self.allow_random_seed,
            lock_template_options: self.rest.lock_template_options,
            pack_optimization_version: self.rest.pack_optimization_version,
            extra: self.extra,
        })
    }
}

impl Metadata0 {
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
                        let versions = versions.iter().filter_map(relaxed_version).collect();
                        (tool, versions)
                    })
                    .collect()
            }),
        }
    }
}

// ---- the formats ----

/// The format the read knows: 0, 1 or 2.
fn format(read: &Read<'_>) -> u16 {
    read.known_format().map_or(0, |format| format.major)
}

/// Format 0: findings vanilla reports as "upgraded" twins are warnings.
fn upgraded(read: &Read<'_>) -> bool {
    format(read) == 0
}

/// A pack-report warning, or an error outside format 0 (vanilla's "Upgraded" twin codes are one
/// code here, with the severity picked by the read; a pack-report entry has no content-log
/// level, so it is taken as the level-2 equivalent).
fn upgraded_severity(read: &Read<'_>) -> Severity {
    if upgraded(read) {
        Severity::Warning
    } else {
        Severity::Error
    }
}

// ---- the members ----

/// `modules`, absent or `null`: format 0 has a lenient path for it (the header's reader adds a
/// module); format 1 and 2 report the same error as a non-array, which ends the read.
fn modules_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Module>> {
    read.push(lacking(object, "modules", &core_codes::WRONG_TYPE_PROPERTY));
    read.stop();
    None
}

/// `modules` of format 1 and 2: an error from them ends the read.
fn modules(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Module>> {
    let before = errors(read);
    let modules = module_list(node, read);
    if errors(read) > before {
        read.stop();
    }
    modules
}

/// The modules of a `modules` array: the elements that read; modules of more than one pack type
/// are an error (format 0: a warning, and only the last module is kept).
fn module_list(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Module>> {
    let mut modules = read_list(node, read, ElementFailure::Drop, |element, read| {
        let module = Module0::from_json(element, read)?;
        Some(Module {
            module_type: module.module_type?,
            uuid: module.uuid?,
            version: module.version?,
            description: module.description,
            entry: module.entry,
            language: module.language,
            extra: module.extra,
        })
    })?;
    if !modules_consistent(&modules) {
        read.push(at(
            node,
            &core_codes::MULTIPLE_MODULES,
            upgraded_severity(read),
        ));
        if upgraded(read) {
            // The lenient path keeps only the last module.
            let last = modules.pop();
            modules = last.into_iter().collect();
        }
    }
    Some(modules)
}

/// A module's `type`: a name of the table other than `invalid`; anything else (no string, an
/// absent one) is told, and format 0 reads `resources` for it
/// (`packupgradewarning.invalidpacktype`).
fn module_type(node: &Node<'_>, read: &mut Read<'_>) -> Option<ModuleType> {
    let name = String::from_json(node, read).unwrap_or_default();
    match ModuleType::from_name(&name) {
        Some(ty) if ty != ModuleType::Invalid => Some(ty),
        _ => invalid_module_type(node.path(), node.span(), read),
    }
}

/// A module without a `type` (or with `null`): an invalid one.
fn module_type_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<ModuleType> {
    invalid_module_type(&object.path().child("type"), object.span(), read)
}

/// A module type that is none: `packupgradewarning.invalidpacktype` in format 0, substituted by
/// `resources`; an error naming the member as a pointer otherwise.
fn invalid_module_type(
    path: &bedrock_addon_core::diagnostic::JsonPath,
    span: Option<bedrock_addon_core::diagnostic::Span>,
    read: &mut Read<'_>,
) -> Option<ModuleType> {
    let (code, severity) = if upgraded(read) {
        (Code::from(&core_codes::INVALID_PROPERTY), Severity::Warning)
    } else {
        (
            core_codes::MANIFEST_INVALID_VALUE.with([pointer(path)]),
            Severity::Error,
        )
    };
    let mut finding = Diagnostic::of(code).severity(severity).at(path.clone());
    finding.location.span = span;
    read.push(finding);
    upgraded(read).then_some(ModuleType::Resources)
}

/// The header, absent or `null`: an error, and the read ends.
fn header_absent<T>(object: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    read.push(lacking(object, "header", &core_codes::MISSING_PROPERTY));
    read.stop();
    None
}

/// The format-1/2 header: one without a UUID or a version ends the read.
fn header(node: &Node<'_>, read: &mut Read<'_>) -> Option<Header12> {
    let header = Header12::from_json(node, read);
    if header.is_none() {
        read.stop();
    }
    header
}

/// The header's UUID, upgraded in format 0.
fn header_uuid(node: &Node<'_>, read: &mut Read<'_>) -> Option<Uuid> {
    let upgraded = upgraded(read);
    uuid(node, read, upgraded)
}

/// A UUID read strictly in every format (a module's, a dependency's).
fn strict_uuid(node: &Node<'_>, read: &mut Read<'_>) -> Option<Uuid> {
    uuid(node, read, false)
}

/// A UUID: the empty text is `EmptyProperty` and gives none; a malformed or nil text is replaced
/// by its hash with `MalformedPropertyUuid` (an error; a warning when `upgraded`). Vanilla gives
/// a legacy pack with no UUID a random one; the crate reports `missing_property` and rejects the
/// manifest instead.
fn uuid(node: &Node<'_>, read: &mut Read<'_>, upgraded: bool) -> Option<Uuid> {
    let raw = non_empty(node, read)?;
    if raw.is_empty() {
        return None;
    }
    if let Some(uuid) = ids::parse(&raw) {
        return Some(uuid);
    }
    let severity = if upgraded {
        Severity::Warning
    } else {
        Severity::Error
    };
    read.push(at(node, &core_codes::MALFORMED_PROPERTY_UUID, severity));
    Some(ids::salvage(&raw))
}

/// A required string: an empty one is reported as `empty_property:%s;` and the pack is
/// rejected, so it is an error; the empty text is still read.
fn non_empty(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = String::from_json(node, read)?;
    if text.is_empty() {
        read.error(node, Code::from(&core_codes::EMPTY_PROPERTY));
    }
    Some(text)
}

/// The header's version, upgraded in format 0.
fn header_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
    let upgraded = upgraded(read);
    version(node, read, upgraded)
}

/// A version read strictly in every format (a module's, a dependency's).
fn strict_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
    version(node, read, false)
}

/// Format 0's absent `packs_version`: 0.0.0, with a warning.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the signature `schema::absent_with` calls"
)]
fn packs_version_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
    let mut finding = lacking(object, "packs_version", &core_codes::MISSING_PROPERTY);
    finding.severity = Severity::Warning;
    read.push(finding);
    Some(SemVersion::new(0, 0, 0))
}

/// A version: the strict string form first, then the array form; a format-0 (`upgraded`)
/// manifest substitutes what it cannot read (a bad string: 1.0.0, another value: 0.0.0) with a
/// warning; its errors name the member as a pointer.
fn version(node: &Node<'_>, read: &mut Read<'_>, upgraded: bool) -> Option<SemVersion> {
    if let Some(raw) = node.value().as_string() {
        return match strict_version_text(raw.as_str()) {
            Some(version) => Some(version),
            None if upgraded => {
                read.push(at(
                    node,
                    &core_codes::MALFORMED_PROPERTY_VERSION,
                    Severity::Warning,
                ));
                Some(SemVersion::new(1, 0, 0))
            }
            None => {
                read.refuse(
                    node,
                    core_codes::MANIFEST_NOT_SEMVER.with([pointer(node.path())]),
                );
                None
            }
        };
    }
    if let Some(version) = version_array(node, read) {
        return Some(version);
    }
    // An array of the wrong length or with an unconvertible element, or another type.
    if upgraded {
        read.push(at(
            node,
            &core_codes::WRONG_TYPE_PROPERTY,
            Severity::Warning,
        ));
        return Some(SemVersion::new(0, 0, 0));
    }
    read.refuse(
        node,
        core_codes::MANIFEST_WRONG_TYPE.with([pointer(node.path())]),
    );
    None
}

/// Vanilla's array form of a version, telling what it converts (the crate's own report; vanilla
/// is silent): an element that is `null` or a bool is read as 0 or 1 with an `InvalidType`
/// information at the element, a real with a fraction is truncated with an `InvalidValue` one.
/// `None`, telling nothing, for a value of another form.
pub(super) fn version_array(node: &Node<'_>, read: &mut Read<'_>) -> Option<SemVersion> {
    let version = super::read::version_array(node.value())?;
    let elements = read.array(node)?;
    for element in elements.iter() {
        let value = element.value();
        let fraction = value
            .as_number()
            .is_some_and(|number| number.to_f64_lossy().fract() != 0.0);
        if value.is_null() || value.as_bool().is_some() {
            read.push(at(&element, &core_codes::INVALID_TYPE, Severity::Info));
        } else if fraction {
            read.push(at(&element, &core_codes::INVALID_VALUE, Severity::Info));
        }
    }
    Some(version)
}

/// The header's `name`: an empty one is told from format 2 (`manifestvalidation.noname`, an
/// information from the crate as the game adds no report entry) and as format 0's
/// `empty_property:%s,%s;` warning.
fn name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let name = String::from_json(node, read)?;
    if name.is_empty() {
        match format(read) {
            0 => read.push(at(node, &core_codes::EMPTY_PROPERTY, Severity::Warning)),
            1 => {}
            _ => read.push(at(node, &core_codes::EMPTY_PROPERTY, Severity::Info)),
        }
    }
    Some(name)
}

/// A header with no `name` is reported through `manifestvalidation.noname` in formats 1 and 2,
/// an error from format 2, and with the UUID in format 0; the pack goes on loading.
fn name_absent(object: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    if format(read) >= 2 {
        read.push(lacking(object, "name", &core_codes::MISSING_PROPERTY));
    }
    None
}

/// The members of a format-1/2 header read after the others.
fn header_rest(header: &mut Header12, node: &Node<'_>, read: &mut Read<'_>) {
    let Some(members) = read.object(node) else {
        return;
    };
    let pack_optimization_version = pack_optimization_version(&members, read);
    if let Some(text) = unknown_scope(header.pack_scope.as_deref(), &members, read) {
        // Written back as it was.
        header
            .extra
            .insert("pack_scope".to_owned(), Value::from(text));
    }
    let pack_type = read
        .scope::<Vec<Module>>("modules")
        .map_or(PackType::Invalid, |modules| pack_type(modules));
    header.rest = rest(&members, pack_type, read);
    header.rest.pack_optimization_version = pack_optimization_version;
    if format(read) < 2 {
        keep_written_min_engine_version(
            header.rest.min_engine_version.as_ref(),
            &mut header.extra,
            &members,
        );
    }
}

/// `pack_optimization_version`: read only when present, without findings but the array form's
/// notes, kept only when valid.
fn pack_optimization_version(
    members: &Members<'_>,
    read: &mut Read<'_>,
) -> Option<RelaxedSemVersion> {
    let node = members.get("pack_optimization_version")?;
    node.value()
        .as_string()
        .and_then(|text| RelaxedSemVersion::from_text(text.as_str()))
        .or_else(|| version_array(&node, read).map(RelaxedSemVersion::new))
}

/// The `pack_scope` text that names no scope, told: vanilla reads it in silence, the crate
/// reports an information.
fn unknown_scope<'t>(
    text: Option<&'t str>,
    members: &Members<'_>,
    read: &mut Read<'_>,
) -> Option<&'t str> {
    let text = text.filter(|text| PackScope::from_json_name(text).is_none())?;
    let node = members.get("pack_scope")?;
    read.push(at(&node, &core_codes::INVALID_PROPERTY, Severity::Info));
    Some(text)
}

/// The members whose rules follow the pack type, in the parser's order.
fn rest(members: &Members<'_>, pack_type: PackType, read: &mut Read<'_>) -> Rest {
    Rest {
        pack_optimization_version: None,
        min_engine_version: min_engine_version(members, pack_type, read),
        base_game_version: base_game_version(members, pack_type, read),
        lock_template_options: lock_template_options(members, pack_type, read),
    }
}

/// A member that is written and not `null`.
fn written<'v>(members: &Members<'v>, key: &str) -> Option<Node<'v>> {
    members.get(key).filter(|node| !node.value().is_null())
}

/// `min_engine_version`: enforced only for behaviour and resource packs, array form only; a
/// format-0/1 pack of those without one gets 1.2.5, and one of 1.13.0 or above is capped to
/// 1.12.0 with a warning.
fn min_engine_version(
    members: &Members<'_>,
    pack_type: PackType,
    read: &mut Read<'_>,
) -> Option<MinEngineVersion> {
    let enforced = matches!(pack_type, PackType::Behavior | PackType::Resource);
    let format_2 = format(read) >= 2;
    let Some(node) = written(members, "min_engine_version") else {
        return match (format_2, enforced) {
            (true, true) => {
                read.push(lacking(
                    members.node(),
                    "min_engine_version",
                    &core_codes::MISSING_PROPERTY,
                ));
                None
            }
            (false, true) => Some(MinEngineVersion::FORMAT_1_DEFAULT),
            _ => None,
        };
    };
    if format_2 && !enforced {
        // `ignored_property:%s;` is a pack-report warning with no content-log level, so it
        // counts as the level-2 class.
        read.push(at(&node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
        return None;
    }
    // The legacy parser accepts only the array form; a string like "1.20.0" is
    // `malformed_property_version`.
    let Some(version) = version_array(&node, read) else {
        read.error(&node, Code::from(&core_codes::MALFORMED_PROPERTY_VERSION));
        return None;
    };
    if format_2 {
        return engine_bounds(&version, &node, read).then_some(MinEngineVersion { version });
    }
    let version = MinEngineVersion { version };
    if version.exceeds_format_1_cap() {
        read.push(at(
            &node,
            &core_codes::MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1,
            Severity::Warning,
        ));
        return Some(MinEngineVersion::FORMAT_1_CAP);
    }
    Some(version)
}

/// Keeps a format-0/1 `min_engine_version` in `extra` when the value the pack uses does not
/// write it back: a capped value (1.13.0 or above, used as 1.12.0) and an explicit 1.2.5 (the
/// writer leaves the default out). The writer puts it back at its place. A manifest is
/// written as it was read, while the typed field keeps the value the pack uses.
fn keep_written_min_engine_version(
    used: Option<&MinEngineVersion>,
    extra: &mut UnknownMembers,
    members: &Members<'_>,
) {
    let Some(written) = written(members, "min_engine_version") else {
        return;
    };
    let Some(version) = super::read::version_array(written.value()) else {
        return;
    };
    let as_written = MinEngineVersion { version };
    if used != Some(&as_written) || used == Some(&MinEngineVersion::FORMAT_1_DEFAULT) {
        extra.insert("min_engine_version".to_owned(), written.value().clone());
    }
}

/// `base_game_version`: read by world templates from format 2 (`*`, a strict version string or
/// the array form, between 1.13.0 and the engine version); ignored with a warning elsewhere.
fn base_game_version(
    members: &Members<'_>,
    pack_type: PackType,
    read: &mut Read<'_>,
) -> Option<BaseGameVersion> {
    let template = pack_type == PackType::WorldTemplate;
    let format_2 = format(read) >= 2;
    let Some(node) = written(members, "base_game_version") else {
        if template && format_2 {
            read.push(lacking(
                members.node(),
                "base_game_version",
                &core_codes::MISSING_PROPERTY,
            ));
        }
        return None;
    };
    let parsed = match node.value().as_string().map(facet_value::VString::as_str) {
        Some("*") => Some(BaseGameVersion::ANY),
        Some(raw) => strict_version_text(raw).map(BaseGameVersion::new),
        None => version_array(&node, read).map(BaseGameVersion::new),
    };
    let Some(version) = parsed else {
        read.error(&node, Code::from(&core_codes::MALFORMED_PROPERTY_VERSION));
        return None;
    };
    if !template || !format_2 {
        // A pack-report warning, which counts as the level-2 class.
        read.push(at(&node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
        return None;
    }
    match version.version.as_ref().filter(|v| !v.is_any()) {
        Some(semver) if !engine_bounds(semver, &node, read) => None,
        _ => Some(version),
    }
}

/// `lock_template_options`: required by format-2 world templates, ignored with a warning on
/// other format-2 packs.
fn lock_template_options(
    members: &Members<'_>,
    pack_type: PackType,
    read: &mut Read<'_>,
) -> Option<bool> {
    let template = pack_type == PackType::WorldTemplate;
    let format_2 = format(read) >= 2;
    let Some(node) = written(members, "lock_template_options") else {
        if format_2 && template {
            read.push(lacking(
                members.node(),
                "lock_template_options",
                &core_codes::MISSING_PROPERTY,
            ));
        }
        return None;
    };
    let value = bool::from_json(&node, read)?;
    if template || !format_2 {
        return Some(value);
    }
    // A pack-report warning, which counts as the level-2 class.
    read.push(at(&node, &core_codes::IGNORED_PROPERTY, Severity::Warning));
    None
}

/// The format-0 header and what is read from inside it: its members, then (an error from them
/// ends the read) its modules (vanilla adds one resources module, its UUID a hash, where there
/// are none), the members whose rules follow the pack type, its metadata and the UUID a module
/// duplicates.
fn header_v0(node: &Node<'_>, read: &mut Read<'_>) -> Option<Read0> {
    let before = errors(read);
    let header = Header0::from_json(node, read);
    let members = read.object(node);
    let (Some(header), Some(members)) = (header, members) else {
        read.stop();
        return None;
    };
    let (Some(uuid), Some(version)) = (header.pack_id, header.packs_version.clone()) else {
        read.stop();
        return None;
    };
    let pack_optimization_version = pack_optimization_version(&members, read);
    // Told, and not kept: the header's unknown members are taken again below.
    let _ = unknown_scope(header.pack_scope.as_deref(), &members, read);
    if errors(read) > before {
        read.stop();
        return None;
    }
    let mut modules = if let Some(list) = written(&members, "modules") {
        module_list(&list, read).unwrap_or_default()
    } else {
        read.push(
            lacking(node, "modules", &core_codes::MISSING_MODULES).severity(Severity::Warning),
        );
        // Lenient path for a manifest without modules.
        vec![Module::new(
            ModuleType::Resources,
            ids::hash(&format!("{}modules", uuid.hyphenated())),
            SemVersion::new(0, 0, 0),
        )]
    };
    let mut extra = header.extra;
    let rest = rest(&members, pack_type(&modules), read);
    keep_written_min_engine_version(rest.min_engine_version.as_ref(), &mut extra, &members);
    let metadata = written(&members, "metadata")
        .and_then(|node| Metadata0::from_json(&node, read))
        .map(Metadata0::into_metadata);
    let mut header = Header {
        // An empty or absent name reads as the UUID in format 0.
        name: header
            .name
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| uuid.hyphenated().to_string()),
        description: header.description,
        uuid,
        version,
        min_engine_version: rest.min_engine_version,
        base_game_version: rest.base_game_version,
        pack_scope: header
            .pack_scope
            .as_deref()
            .and_then(PackScope::from_json_name),
        platform_locked: header.platform_locked,
        allow_random_seed: header.allow_random_seed,
        lock_template_options: rest.lock_template_options,
        pack_optimization_version,
        extra,
    };
    if let Some(pack_id) = members.get("pack_id") {
        for module in &mut modules {
            if module.uuid == header.uuid {
                // The header UUID is replaced by a hash of itself, with a warning.
                header.uuid = ids::hash(&header.uuid.hyphenated().to_string());
                read.push(at(&pack_id, &core_codes::DUPLICATE_UUID, Severity::Warning));
            }
        }
    }
    Some(Read0 {
        header,
        modules,
        metadata,
    })
}

/// A module whose UUID equals the header's: an error per such module (format 1 and 2).
fn duplicate_uuids(document: &mut Document12, node: &Node<'_>, read: &mut Read<'_>) {
    let (Some(header), Some(modules)) = (&document.header, &document.modules) else {
        return;
    };
    let Some(uuid_node) = read
        .object(node)
        .and_then(|members| members.get("header"))
        .and_then(|header| read.object(&header)?.get("uuid"))
    else {
        return;
    };
    for module in modules {
        if Some(module.uuid) == header.uuid {
            read.error(&uuid_node, Code::from(&core_codes::DUPLICATE_UUID));
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

// ---- the rest of format 1 and 2 ----

/// `dependencies`: a pack (`module_name` absent or `null`) or a native module; an element with
/// both `module_name` and `uuid` is dropped with an `InvalidProperty` warning. The elements that
/// read are kept, with where each is in the file.
fn dependencies(node: &Node<'_>, read: &mut Read<'_>) -> Option<Dependencies> {
    let mut places = Vec::new();
    let list = read_list(node, read, ElementFailure::Drop, |element, read| {
        let members = read.object(element)?;
        let dependency = if written(&members, "module_name").is_none() {
            let dependency = PackDependency0::from_json(element, read)?;
            Dependency::Pack {
                uuid: dependency.uuid?,
                version: dependency.version?,
            }
        } else if written(&members, "uuid").is_some() {
            read.push(at(
                element,
                &core_codes::INVALID_PROPERTY,
                Severity::Warning,
            ));
            return None;
        } else {
            let dependency = NativeDependency0::from_json(element, read)?;
            Dependency::NativeModule {
                module_name: dependency.module_name,
                version: dependency.version?,
                optional: None,
            }
        };
        let native = matches!(dependency, Dependency::NativeModule { .. });
        places.push(dependency_place(
            element.value(),
            element.path().clone(),
            native,
        ));
        Some(dependency)
    })?;
    Some(Dependencies { list, places })
}

/// A native module's `version`: `alpha`, `beta`, `internal`, else a version read strictly. Not
/// read once the `module_name` before it failed: the parser stops at the module there.
fn native_version(node: &Node<'_>, read: &mut Read<'_>) -> Option<NativeModuleVersion> {
    if read.scope::<String>("module_name").is_none() {
        // The element is left out by its `module_name`'s failure, told already.
        return Some(NativeModuleVersion::LatestAlpha);
    }
    match node.value().as_string().map(facet_value::VString::as_str) {
        Some("alpha") => Some(NativeModuleVersion::LatestAlpha),
        Some("beta") => Some(NativeModuleVersion::LatestBeta),
        Some("internal") => Some(NativeModuleVersion::LatestInternal),
        _ => strict_version(node, read).map(NativeModuleVersion::Version),
    }
}

/// `metadata`, its members in the parser's order.
fn metadata(node: &Node<'_>, read: &mut Read<'_>) -> Option<Metadata0> {
    Metadata0::from_json(node, read)
}

/// The members of `metadata` the parser reads no value of: not written back, which the load says
/// at each (the crate's own information; the game is silent). The root, the header and a module
/// keep theirs.
fn members_not_kept(_: &mut Metadata0, node: &Node<'_>, read: &mut Read<'_>) {
    const MEMBERS: [&str; 5] = [
        "authors",
        "generated_with",
        "license",
        "product_type",
        "url",
    ];
    let Some(members) = read.object(node) else {
        return;
    };
    for (key, member) in members.iter() {
        if !MEMBERS.contains(&key) {
            read.push(at(&member, &core_codes::MEMBER_NOT_KEPT, Severity::Info));
        }
    }
}

/// `metadata/authors`: vanilla's legacy parser does not read it, so a value that is no list of
/// strings is an information from the crate, and none is kept.
fn authors(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let authors = node.value().as_array().and_then(|list| {
        list.as_slice()
            .iter()
            .map(|author| author.as_string().map(|text| text.as_str().to_owned()))
            .collect::<Option<Vec<_>>>()
    });
    if authors.is_none() {
        read.push(at(node, &core_codes::WRONG_TYPE_PROPERTY, Severity::Info));
    }
    authors
}

/// `metadata/generated_with`: tool name → versions; a tool whose versions are no list is the
/// wrong type and left out, a version that is neither a relaxed version string nor the array
/// form is dropped with an `ignored_property` pack-report warning, which counts as the level-2
/// class.
fn generated_with(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, Vec<Value>>> {
    let members = read.object(node)?;
    let mut tools = IndexMap::new();
    for (tool, versions) in members.iter() {
        let kept = read_list(&versions, read, ElementFailure::Drop, |version, read| {
            let read_as = version
                .value()
                .as_string()
                .and_then(|text| RelaxedSemVersion::from_text(text.as_str()))
                .or_else(|| version_array(version, read).map(RelaxedSemVersion::new));
            if read_as.is_none() {
                read.push(at(
                    version,
                    &core_codes::IGNORED_PROPERTY,
                    Severity::Warning,
                ));
            }
            read_as.map(|_| version.value().clone())
        });
        if let Some(kept) = kept {
            tools.insert(tool.to_owned(), kept);
        }
    }
    Some(tools)
}

/// A version of `generated_with` as kept: the text as spelled, or the array form in full.
fn relaxed_version(version: &Value) -> Option<RelaxedSemVersion> {
    version
        .as_string()
        .and_then(|text| RelaxedSemVersion::from_text(text.as_str()))
        .or_else(|| super::read::version_array(version).map(RelaxedSemVersion::new))
}

/// Legacy `settings`: only labels are kept; an object that is no label is dropped with
/// `LegacySettingIgnored`, the crate's own information (vanilla drops them silently), and an
/// element that is no object is passed over.
fn settings(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Setting>> {
    let elements = read.array(node)?;
    let mut settings = Vec::new();
    for element in elements.iter() {
        let value = element.value();
        if value.as_object().is_none() {
            continue;
        }
        let text = |key| super::read::member(value, key).and_then(super::read::text);
        match (text("type"), text("text")) {
            (Some("label"), Some(text)) => settings.push(Setting::Label {
                text: text.to_owned(),
            }),
            _ => read.push(at(
                &element,
                &core_codes::LEGACY_SETTING_IGNORED,
                Severity::Info,
            )),
        }
    }
    Some(settings)
}

/// `subpacks`: each element's names and its tier; an element whose names fail is kept with what
/// it read.
fn subpacks(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Subpack>> {
    read_list(node, read, ElementFailure::Drop, |element, read| {
        let members = read.object(element)?;
        let names = SubpackNames::from_json(element, read);
        let memory_tier = memory_tier(&members, read);
        let (folder_name, name) =
            names.map_or((None, None), |names| (names.folder_name, names.name));
        Some(Subpack {
            name: name.unwrap_or_default(),
            folder_name: folder_name.unwrap_or_default(),
            memory_tier,
        })
    })
}

/// `memory_tier` (a memory size mapped to a tier) or `memory_performance_tier` (0..5).
fn memory_tier(members: &Members<'_>, read: &mut Read<'_>) -> MemoryTier {
    let integer = |key: &str| {
        let node = members.get(key)?;
        let number = node
            .value()
            .as_number()
            .filter(|number| number.is_integer())
            .and_then(facet_value::VNumber::to_i64)?;
        Some((node, number))
    };
    // A legacy `memory_tier` is a memory size and picks the highest tier whose threshold it
    // reaches; a negative size is an error. `memory_performance_tier` names the tier directly,
    // an integer from 0 to 5, and a larger one is rejected.
    if let Some((node, size)) = integer("memory_tier") {
        if size < 0 {
            read.error(&node, Code::from(&core_codes::INVALID_PROPERTY));
        }
        return MemoryTier::from_legacy_size(size);
    }
    if let Some((node, tier)) = integer("memory_performance_tier") {
        return u8::try_from(tier)
            .ok()
            .and_then(MemoryTier::from_number)
            .unwrap_or_else(|| {
                read.error(&node, Code::from(&core_codes::INVALID_PROPERTY));
                MemoryTier::Tier0
            });
    }
    // The game has no pack-report warning for this, so the crate reports an information.
    let mut finding = lacking(
        members.node(),
        "memory_tier",
        &core_codes::WRONG_TYPE_PROPERTY,
    );
    finding.severity = Severity::Info;
    if let Some(node) = members.get("memory_tier") {
        finding.location.span = node.span();
    }
    read.push(finding);
    MemoryTier::Tier0
}

/// `format_version` as vanilla's legacy parser reads it: jsoncpp `asInt` truncated to a `u8`.
/// Absent or `null` is 0; a string, array or object is 0, `true` 1, a real truncated; each
/// coercion and truncation is linted by the crate. 3 and above (after truncation) is an error and
/// nothing is read.
pub(super) fn format_version(root: &Value, sink: &mut super::read::Sink<'_>) -> Option<u8> {
    let at = bedrock_addon_core::diagnostic::JsonPath::of_keys(["format_version"]);
    let Some(value) = super::read::member(root, "format_version").filter(|v| !v.is_null()) else {
        return Some(0);
    };
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
