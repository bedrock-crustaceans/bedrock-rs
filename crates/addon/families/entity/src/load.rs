//! The entity family: version selection, the newest and beta structs, the load of one file and
//! the registry of a stack.

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Placed, Registry,
    RegistryBuilder, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{EntityId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangFailure, MolangFieldSpec, MolangFields, MolangVisitor, OpsRule,
    PROPERTY_DEFAULT_QUERIES, Step,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    BETA_VERSION, ComponentSection, ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity,
    InputKind, Loaded, ParseCx, Parsed, StructIndex, ValidateCx, VersionChain, VersionStruct,
    Versioned, finish, load, member_version, parse_as_written,
};

use super::component::EntityComponent;
use super::{AnimateEntry, Beta, Entity, PropertyDefault, history, read, validate, write};

/// The directory entity files live in.
const DIRECTORY: &str = "entities";

/// The first slice registered with the strict mask.
pub(crate) const STRICT_SLICES: FormatVersion = FormatVersion::new(1, 21, 89);

impl Family for Entity {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "entity";
    const PAYLOAD_KEY: &'static str = "minecraft:entity";
    const DIRECTORIES: &'static [&'static str] = &[DIRECTORY];
    // 1.8.0, the first version with `description.identifier`.
    const FLOOR: FormatVersion = FormatVersion::new(1, 8, 0);
    const SKIP_UPGRADES_KEY: Option<&'static str> = Some("do_not_upgrade");
    // The text is the crate's own; the game reads an absent version as 0.0.0 silently.
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::NoFormatVersionField);

    /// The text is the crate's own and the game is silent: an information.
    fn missing_version_severity() -> Severity {
        Severity::Info
    }

    // A jsoncpp input: the version from the root member, and
    // rule N1, a member written as `null` is absent.
    const INPUT: InputKind = InputKind::JsonCpp;
    // The game takes `format_version` and `minecraft:entity` out of the root and reads nothing
    // else there: it never counts the root's members, so a root member beside the two is
    // passed over and the entity loads.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Uncounted;
    type Latest = Entity;
    type Beta = Beta;
    type Component = EntityComponent;
    type Any = history::Any;

    fn components() -> &'static ComponentTable<Self> {
        EntityComponent::TABLE
    }

    /// The game reads a document below 1.21.89 with the unstrict error codes and every slice
    /// from 1.21.89 on with the strict ones, the beta struct included.
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(i) => {
                let versions = <history::Any as VersionChain<Entity>>::VERSIONS;
                if versions.get(i).is_some_and(|v| *v >= STRICT_SLICES) {
                    ErrorMask::Strict
                } else {
                    ErrorMask::Unstrict
                }
            }
            // The beta slice is read strictly, as every slice after the legacy one.
            StructIndex::Beta => ErrorMask::Strict,
        }
    }

    /// without a root `format_version`, the pre-1.8 in-payload one.
    fn payload_version(payload: JsonSlice<'_>) -> Option<FormatVersionInput> {
        member_version(payload, "format_version")
    }

    fn actor_type(identity: &Identity) -> ActorType {
        ActorType::for_entity(
            identity.identifier.as_deref().unwrap_or_default(),
            identity.runtime_identifier.as_deref(),
        )
    }

    fn component_sections(v: &mut Entity) -> Vec<ComponentSection<'_, EntityComponent>> {
        let mut sections = Vec::with_capacity(1 + v.component_groups.len());
        let mut at = JsonPath::new();
        at.push("components");
        sections.push(ComponentSection::new(at, false, &mut v.components));
        for (name, group) in &mut v.component_groups {
            let mut at = JsonPath::new();
            at.push("component_groups");
            at.push(name);
            sections.push(ComponentSection::new(at, false, &mut group.components));
        }
        sections
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        not_in_the_schema(cx);
        match v {
            Loaded::Stable(entity) => validate::validate(entity, cx),
            Loaded::Beta(beta) => validate::validate(&mut beta.0, cx),
        }
    }
}

impl Versioned for Entity {
    const VERSION: FormatVersion = FormatVersion::new(1, 26, 60);
}

impl VersionStruct for Entity {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::entity(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::payload(self)
    }
}

impl Versioned for Beta {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for Beta {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::entity(payload, cx, d).map(Beta)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::payload(&self.0)
    }
}

impl MolangFields for Beta {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        self.0.visit_molang(visitor);
    }
}

/// The file's text: without a byte-order mark, UTF-8, not nested too deeply. A file that is
/// none of these is rejected with a `MalformedInput` error.
fn decode<'a>(bytes: &'a [u8], origin: &Origin) -> Result<&'a str, Rejected> {
    let bytes = strip_bom(bytes);
    let malformed = |message: &str| {
        let mut found = finding(DiagnosticCode::MalformedInput {
            message: message.to_owned(),
            text: None,
        });
        found.location.pack = Some(origin.pack.clone());
        found.location.path = Some(origin.path.clone());
        Rejected::new(origin.clone(), [found].into_iter().collect())
    };
    let text = std::str::from_utf8(bytes).map_err(|_| malformed("the file is not UTF-8"))?;
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| malformed("the JSON nests too deeply"))?;
    Ok(text)
}

/// The findings of a load, placed in the file of `origin`. A document above the engine gets
/// vanilla's resource-pack text.
fn place(mut found: Diagnostics, version: &VersionInfo, origin: &Origin) -> Diagnostics {
    let above_engine = version.written.filter(|v| *v > ENGINE_VERSION);
    // In place: the list keeps where the members of the entity are in the file.
    for diagnostic in found.iter_mut() {
        if let Some(written) = above_engine
            && let DiagnosticCode::UnsupportedVersion { text } = &mut diagnostic.code
        {
            *text = Some(VanillaText::ActorAboveEngine {
                version: written.to_string(),
                engine: ENGINE_VERSION,
            });
        }
        diagnostic.location.pack = Some(origin.pack.clone());
        diagnostic.location.path = Some(origin.path.clone());
    }
    found
}

/// The line the game writes when the load of an entity carries errors; the definition is not
/// registered. The second `{}` is a log's prefix and then its first entry's message: the parse
/// log whenever it has an entry, else the upgrade log, else the validate log.
const FAILED_TO_LOAD: &str = "ERROR: Entity '{}' failed to load from JSON: {}";

/// A file refused for what its payload holds (an error below
/// `minecraft:entity`) closes with vanilla's line, the identifier the file names and the first
/// such error (`<member below the payload>: <its text>`). A refusal of the envelope, the version
/// or the beta permission is no load of the definition and gets none.
fn close_refusal(found: &mut Diagnostics, bytes: &[u8]) {
    let payload = <Entity as Family>::PAYLOAD_KEY;
    let mut under = JsonPath::new();
    under.push(payload);
    let logged = |finding: &&Diagnostic| {
        finding.severity >= Severity::Warning && finding.location.json_path.starts_with(&under)
    };
    if !found
        .iter()
        .any(|finding| finding.severity == Severity::Error && logged(&finding))
    {
        return;
    }
    // The game closes with the parse log's first entry whenever that log has one, warnings
    // included; only an empty parse log falls through to the upgrade log, then the validate
    // log, each with its prefix.
    let in_phase = |phases: &[Phase]| {
        found
            .iter()
            .filter(logged)
            .find(|finding| phases.contains(&finding.phase))
    };
    let Some((phase, first)) = [
        ("parse errors occurred: ", &[Phase::Read, Phase::Parse][..]),
        ("upgrade errors occurred: ", &[Phase::Upgrade][..]),
        (
            "validate errors occurred: ",
            &[Phase::Validate, Phase::Link][..],
        ),
    ]
    .into_iter()
    .find_map(|(prefix, phases)| in_phase(phases).map(|finding| (prefix, finding))) else {
        return;
    };
    // The member below the payload, its keys joined as the game's logs join them.
    let member: Vec<String> = first
        .location
        .json_path
        .segments()
        .skip(1)
        .map(|segment| match segment {
            bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => key.to_owned(),
            bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => index.to_string(),
        })
        .collect();
    let reason = if member.is_empty() {
        format!("{phase}{}", first.code)
    } else {
        format!("{phase}{}: {}", member.join("/"), first.code)
    };
    let identifier = identifier_of(bytes).map_or_else(String::new, |id| id.as_str().to_owned());
    let mut line = finding(DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Formatted {
            template: FAILED_TO_LOAD,
            args: vec![identifier, reason],
        }),
    });
    line.location.pack.clone_from(&first.location.pack);
    line.location.path.clone_from(&first.location.path);
    found.push(line);
}

/// A component at or past the version its row is deprecated in is no member of the game's actor
/// schema at that version: cereal reports "this component was found in the input, but is not
/// present in the Schema", a warning in the legacy slice (below 1.21.89) and an error that
/// rejects the entity in a strict one. Core's generic deprecation line (the item and block registries' "%s has
/// been deprecated and is not available in json format %s") is the wrong text for an entity.
fn not_in_the_schema(cx: &mut ValidateCx<'_>) {
    // The game logs this at the warning level below 1.21.89 and as an error from it.
    let severity = if cx.original_version >= STRICT_SLICES {
        Severity::Error
    } else {
        Severity::Warning
    };
    for finding in cx.diagnostics.iter_mut() {
        if matches!(
            finding.code,
            DiagnosticCode::DeprecatedMember {
                text: Some(VanillaText::Deprecated { .. })
            }
        ) && finding.phase == Phase::Validate
        {
            finding.code = bedrock_addon_core::diagnostic::cereal::component_not_in_schema();
            finding.severity = severity;
        }
    }
}

/// One file through the chain: the document, or its rejection with every finding.
fn load_document(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<(Loaded<Entity>, VersionInfo, Diagnostics, Origin), Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let text = decode(bytes, &origin)?;
    let outcome = load::<Entity>(JsonSlice::new(text), cx);
    let mut diagnostics = place(outcome.diagnostics, &outcome.version, &origin);
    if let Some(value) = outcome.value {
        Ok((value, outcome.version, diagnostics, origin))
    } else {
        close_refusal(&mut diagnostics, bytes);
        Err(Rejected::new(origin, diagnostics))
    }
}

impl Entity {
    /// Parses without upgrading: the struct the file's `format_version` selects, with the
    /// findings of the parse.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::Any>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = detached_origin(path);
        let text = decode(bytes, &origin)?;
        let parsed = parse_as_written::<Entity>(JsonSlice::new(text), cx);
        let mut diagnostics = place(parsed.diagnostics, &parsed.version, &origin);
        if let Some(any) = parsed.value {
            Ok(Document::new(any, origin, parsed.version, diagnostics))
        } else {
            close_refusal(&mut diagnostics, bytes);
            Err(Rejected::new(origin, diagnostics))
        }
    }

    /// Runs the remaining upgrade steps and the checks on a document
    /// [`parse_as_written`](Self::parse_as_written) gave: the same result
    /// [`Content::parse`] gives for the file.
    // This is `history::Any::upgrade`, placed on `Entity` because only the document has the
    // origin.
    pub fn upgrade(
        written: Document<history::Any>,
        cx: &LoadContext,
    ) -> Result<Document<Entity>, Rejected> {
        let origin = written.origin().clone();
        let version = written.version().clone();
        let found = written.diagnostics().clone();
        let any = written.into_value();
        let (identity, skip_upgrades) = history::identity(&any);
        let at = {
            let mut at = JsonPath::new();
            at.push(<Entity as Family>::PAYLOAD_KEY);
            at
        };
        let input = FormatVersionInput {
            version: if version.beta {
                None
            } else {
                Some(version.written.unwrap_or(version.effective))
            },
            beta_literal: version.beta,
            prerelease: false,
        };
        let actor_type = <Entity as Family>::actor_type(&identity);
        let parsed = Parsed::<Entity>::new(
            Some(any),
            version,
            input,
            identity,
            actor_type,
            skip_upgrades,
            at,
            // What the reader moved, without its findings: what the steps and the checks
            // tell is at the member the file has, as in a load of the file.
            {
                let mut moved = found.clone();
                moved.retain(|_| false);
                moved
            },
        );
        let outcome = finish(parsed, cx);
        let mut diagnostics = found;
        diagnostics.append(place(outcome.diagnostics, &outcome.version, &origin));
        match outcome.value {
            Some(Loaded::Stable(entity)) => {
                Ok(Document::new(entity, origin, outcome.version, diagnostics))
            }
            Some(Loaded::Beta(_)) => {
                diagnostics.push(finding(DiagnosticCode::UnsupportedVersion {
                    text: Some(VanillaText::NoParserForVersion {
                        version: "beta".to_owned(),
                    }),
                }));
                Err(Rejected::new(origin, diagnostics))
            }
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// A file-level finding.
fn finding(code: DiagnosticCode) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, JsonPath::new(), None),
    )
}

/// The origin of a file read outside a stack.
fn detached_origin(path: &PackPath) -> Origin {
    Origin::new(
        PackRef::app_data(),
        PackSide::Behavior,
        path.clone(),
        None,
        Vec::new(),
    )
}

impl Sealed for Entity {}

impl Content for Entity {
    const NAME: &'static str = "entity";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one entity file, upgraded to the newest struct. A beta document is rejected
    /// here (it is a [`Beta`]).
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let (value, version, mut diagnostics, origin) =
            load_document(detached_origin(path), bytes, cx)?;
        match value {
            Loaded::Stable(entity) => Ok(Document::new(entity, origin, version, diagnostics)),
            Loaded::Beta(_) => {
                diagnostics.push(finding(DiagnosticCode::UnsupportedVersion {
                    text: Some(VanillaText::NoParserForVersion {
                        version: "beta".to_owned(),
                    }),
                }));
                Err(Rejected::new(origin, diagnostics))
            }
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write::document(self, Self::VERSION, out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for Entity {
    type Kind = kind::Entity;
    type File = Entity;
    type Beta = Beta;
}

/// `entities/<namespace>/<name>.json`.
impl Placed for Entity {
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

impl Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "beta entity";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one beta entity file (`"format_version": "beta"`); any other version is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let (value, version, mut diagnostics, origin) =
            load_document(detached_origin(path), bytes, cx)?;
        match value {
            Loaded::Beta(beta) => Ok(Document::new(beta, origin, version, diagnostics)),
            Loaded::Stable(_) => {
                diagnostics.push(finding(DiagnosticCode::UnsupportedVersion { text: None }));
                Err(Rejected::new(origin, diagnostics))
            }
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write::document(&self.0, BETA_VERSION, out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_json(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

impl Definition for Beta {
    type Kind = kind::Entity;
    type File = Beta;
    type Beta = NoBeta<kind::Entity>;
}

/// `entities/<namespace>/<name>.json`.
impl Placed for Beta {
    fn default_path(&self) -> PackPath {
        default_path(DIRECTORY, &self.0.description.identifier)
    }
}

/// Every `.json` file below `entities/`, packs bottom-up so a higher pack's definition of an
/// identifier replaces a lower one's.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "vanilla's directory walk matches `.json` as written"
)]
fn accept(path: &PackPath) -> bool {
    path.as_str().ends_with(".json")
}

/// One file's contribution to the registry.
fn parse_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Entity> {
    let origin = Origin::new(
        file.pack.clone(),
        file.side,
        file.path.clone(),
        None,
        Vec::new(),
    );
    match load_document(origin, &file.bytes, cx) {
        Ok((Loaded::Stable(entity), version, diagnostics, origin)) => {
            let id = entity.description.identifier.clone();
            Contribution::Stable(id, Document::new(entity, origin, version, diagnostics))
        }
        Ok((Loaded::Beta(beta), version, diagnostics, origin)) => {
            let id = beta.0.description.identifier.clone();
            Contribution::Beta(id, Document::new(beta, origin, version, diagnostics))
        }
        Err(rejected) => {
            let id = identifier_of(&file.bytes);
            Contribution::Rejected(id, rejected)
        }
    }
}

/// The identifier a rejected file names, when it names one.
fn identifier_of(bytes: &[u8]) -> Option<EntityId> {
    let text = std::str::from_utf8(strip_bom(bytes)).ok()?;
    let envelope = bedrock_addon_core::json::probe_envelope(JsonSlice::new(text)).ok()?;
    let payload = envelope.payload(<Entity as Family>::PAYLOAD_KEY)?;
    <Entity as Family>::identity(payload)
        .identifier
        .map(EntityId::parse)
}

/// The registry of every pack's entities: a higher pack's definition of an identifier replaces
/// a lower one's.
fn combine(parsed: Vec<Contribution<Entity>>, cx: &StackCx<'_>) -> Registry<Entity> {
    let mut builder =
        RegistryBuilder::new(CombineRule::top_wins(), CombineRule::top_wins(), cx.options);
    for contribution in parsed {
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.entities()`.
pub const fn load_registry() -> CollectionLoader<Registry<Entity>, Contribution<Entity>> {
    CollectionLoader {
        walk: Walk {
            directories: &[DIRECTORY],
            depth: ListDepth::Recursive,
            accept,
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse: parse_file,
        combine,
        needs: &[],
        then: None,
    }
}

/// `description.scripts.animate[].<name>`: a Molang string at the pack's version; a rejected
/// expression is kept and evaluates to 0.
pub(crate) const ANIMATE: MolangFieldSpec = MolangFieldSpec {
    empty: EmptyRule::Compile,
    // The game logs the rejection at the error level, stores the script anyway and keeps the
    // entity: a kept error.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// `description.properties.<name>.default` (a string that is no enum value): a Molang string
/// at the pack's version that may call only `query.had_component_group` and assigns nothing;
/// a rejected expression drops the entity's property group.
pub(crate) const PROPERTY_DEFAULT: MolangFieldSpec = MolangFieldSpec {
    admission: Admission::Only(PROPERTY_DEFAULT_QUERIES),
    ops: OpsRule::NoAssignments,
    empty: EmptyRule::Compile,
    on_failure: MolangFailure::DropEntry,
    // The game logs "Error loading Actor Properties" at the error level and keeps the entity
    // without its properties: a kept error.
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

impl MolangFields for Entity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::from("description"));
        visitor.enter(Step::from("properties"));
        for (name, property) in &mut self.description.properties {
            if let Some(PropertyDefault::Expression(source)) = &mut property.default {
                visitor.enter(Step::from(name.as_str()));
                visitor.field("default", &PROPERTY_DEFAULT, source);
                visitor.leave();
            }
        }
        visitor.leave();
        if let Some(scripts) = &mut self.description.scripts {
            visitor.enter(Step::from("scripts"));
            visitor.enter(Step::from("animate"));
            // Each entry at the index the file has it at.
            let places: Vec<usize> = (0..scripts.animate.len())
                .map(|index| scripts.place(index))
                .collect();
            for (place, entry) in places.into_iter().zip(&mut scripts.animate) {
                if let AnimateEntry::Conditional(members) = entry {
                    visitor.field(place, &ANIMATE, members);
                }
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
        visitor.nested("components", &mut self.components);
        visitor.enter(Step::from("component_groups"));
        for (name, group) in &mut self.component_groups {
            visitor.nested(name.as_str(), &mut group.components);
        }
        visitor.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The closing line of a refusal whose first error below the payload is of `phase`.
    fn closing_line_of(phase: Phase) -> String {
        let mut at = JsonPath::new();
        at.push("minecraft:entity");
        at.push("components");
        let mut found = Diagnostics::default();
        found.push(Diagnostic::new(
            DiagnosticCode::InternalError,
            Severity::Error,
            phase,
            Location::new(None, None, at, None),
        ));
        close_refusal(
            &mut found,
            br#"{"minecraft:entity": {"description": {"identifier": "test:mob"}}}"#,
        );
        found
            .iter()
            .last()
            .expect("the closing line")
            .code
            .to_string()
    }

    /// The prefix follows the parse log, not the error's phase. A file refused in validation
    /// whose parse also logged a warning closes with the parse prefix and that warning's text.
    #[test]
    fn a_parse_warning_takes_the_closing_line_before_a_validation_error() {
        let at = |key: &str| {
            let mut at = JsonPath::new();
            at.push("minecraft:entity");
            at.push(key);
            at
        };
        let mut found = Diagnostics::default();
        found.push(Diagnostic::new(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            Phase::Parse,
            Location::new(None, None, at("made_up"), None),
        ));
        found.push(Diagnostic::new(
            DiagnosticCode::InternalError,
            Severity::Error,
            Phase::Validate,
            Location::new(None, None, at("components"), None),
        ));
        let warning = found.iter().next().unwrap().code.to_string();
        close_refusal(
            &mut found,
            br#"{"minecraft:entity": {"description": {"identifier": "test:mob"}}}"#,
        );
        let line = found
            .iter()
            .last()
            .expect("the closing line")
            .code
            .to_string();
        assert_eq!(
            line,
            format!(
                "ERROR: Entity 'test:mob' failed to load from JSON: parse errors occurred: made_up: {warning}"
            )
        );
    }

    /// The second `{}` of the closing line is the failed phase's prefix (parse, upgrade or
    /// validate) and then the first entry's message.
    #[test]
    fn the_closing_line_names_the_phase_that_failed() {
        for (phase, prefix) in [
            (Phase::Parse, "parse errors occurred: "),
            (Phase::Upgrade, "upgrade errors occurred: "),
            (Phase::Validate, "validate errors occurred: "),
        ] {
            let line = closing_line_of(phase);
            assert!(
                line.starts_with(&format!(
                    "ERROR: Entity 'test:mob' failed to load from JSON: {prefix}components: "
                )),
                "{phase:?}: {line}"
            );
        }
    }

    /// A newest-format entity of `n` component groups, each with a typed and a marker
    /// component, and `n` events that each add one group.
    fn entity_of_groups(n: usize) -> String {
        let groups: Vec<String> = (0..n)
            .map(|i| {
                format!(
                    r#""g_{i}": {{"minecraft:health": {{"value": 5, "max": 5}}, "minecraft:is_baby": {{}}}}"#
                )
            })
            .collect();
        let events: Vec<String> = (0..n)
            .map(|i| format!(r#""e_{i}": {{"add": {{"component_groups": ["g_{i}"]}}}}"#))
            .collect();
        format!(
            r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "demo:groups", "is_spawnable": true, "is_summonable": true}}, "component_groups": {{{}}}, "components": {{}}, "events": {{{}}}}}}}"#,
            groups.join(","),
            events.join(",")
        )
    }

    /// The document of `text` loaded through the whole chain: its groups, events and findings.
    fn loaded(text: &str) -> (usize, usize, usize) {
        let outcome = load::<Entity>(JsonSlice::new(text), &LoadContext::detached());
        let Some(Loaded::Stable(entity)) = outcome.value else {
            panic!("the entity is refused: {:#?}", outcome.diagnostics);
        };
        (
            entity.component_groups.len(),
            entity.events.len(),
            outcome.diagnostics.len(),
        )
    }

    /// Loading time grows linearly with the number of component groups and events: the test
    /// loads documents of several sizes and checks the cost per group stays flat.
    #[test]
    fn one_document_of_n_component_groups_and_n_events_loads_in_n() {
        assert_eq!(loaded(&entity_of_groups(3)), (3, 3, 0));
        bedrock_addon_core::test_cost::assert_linear_at(
            "entity load: one document, n component groups, n events",
            1_000,
            entity_of_groups,
            |text: &String| {
                let found = loaded(text);
                assert_eq!(found.2, 0);
                found
            },
        );
    }
}
