//! Entities: `Entity`, `ComponentGroup`, `Event`, `Property`, `component::*`, `goal::*`,
//! `history::*`.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// The `SchemaFields` of a struct that derives no `Schema` (a goal whose Molang walk reads its
/// triggers' legacy conditions, which the derive's per-field walk cannot, and `tamemount`, which
/// walks its item lists out of declaration order): no check (its checks stay by hand), and the
/// trigger members of the fields listed, each its field's name as the JSON key. Only the
/// trigger-bearing fields are listed; `check_fields` names each, so a field that is not the
/// struct's, or not of the type stated, does not compile.
macro_rules! trigger_fields {
    ($ty:ty { $($field:ident: $field_ty:ty),* $(,)? }) => {
        impl bedrock_addon_core::schema::SchemaFields for $ty {
            fn check_fields(&mut self, _: &mut bedrock_addon_core::schema::SchemaCx<'_>) {
                $(let _: &$field_ty = &self.$field;)*
            }

            #[allow(unused_variables, reason = "a struct with no field that holds a trigger")]
            fn trigger_members(prefix: &str, out: &mut Vec<String>) {
                $(
                    <$field_ty as bedrock_addon_core::schema::SchemaFields>::trigger_members(
                        &bedrock_addon_core::schema::member(prefix, stringify!($field)),
                        out,
                    );
                )*
            }
        }
    };
}
pub(crate) use trigger_fields;

pub mod codes;
pub mod component;
mod dynamic;
mod entity_types;
#[allow(
    dead_code,
    reason = "the schema tables hold rows that only some of the readers use"
)]
mod game_schema;
#[doc(hidden)]
pub use dynamic::DESCRIPTORS;
pub mod goal;
pub mod history;
pub mod links;
mod molang;
#[cfg(feature = "nbt")]
mod network;
mod read;
mod references;
mod typed;
mod types;
mod validate;
mod versions;
mod write;
#[cfg(feature = "nbt")]
#[cfg_attr(docsrs, doc(cfg(feature = "nbt")))]
pub use network::actor_identifiers;

pub use entity_types::EntityTypes as EntityTypesOf;
pub use types::{
    AnimateEntry, Beta, Brain, ComponentGroup, DeprecatedData, Description, Entity, EntityVersion,
    Event, InternalMemoryAccess, Memory, MemoryType, Property, PropertyDefault, PropertyType,
    Scripts, SpawnCategory, UpgradeEntry, Written,
};

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::document::{
    Content, Contribution, Definition, Document, JsonContent, JsonDefinition, JsonDocument,
    JsonPlaced, NoBeta, Origin, Placed, Registry, Rejected, default_path,
};
use bedrock_addon_core::id::{EntityId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    BETA_VERSION, BeyondWidth, ComponentSection, ComponentTable, DocumentRead, Envelope,
    EnvelopeCheck, Family, Identity, InputKind, Loaded, MissingVersion, OpenBeta, ParseCx, Parsed,
    Reading, Scalars, StructIndex, UnknownMask, ValidateCx, VersionChain, VersionStruct, Versioned,
    finish, member_version, parse_as_written, write_envelope,
};

use component::EntityComponent;
use read::{close_refusal, decode, detached_origin, identifier_of, load_document, place};

/// The directory entity files live in.
const DIRECTORY: &str = "entities";

pub(crate) use versions::STRICT_SLICES;

/// What every slice reads by, the unknown mask aside ([`Family::reading`]): a `null` member is
/// absent (rule N1), and a scalar is read strictly (the cereal readers' rule: a number beyond
/// its float's width is refused). The typed reads of the components and goals, which run
/// outside the door, read by the same rules ([`SCALAR_RULES`]).
pub(crate) const READING: Reading = Reading::NULL_ABSENT
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error);

/// The scalar rules of [`READING`]: what the typed read of a component or a goal judges each
/// scalar by.
pub(crate) const SCALAR_RULES: bedrock_addon_core::json::ScalarRules = READING.scalar_rules();

impl Family for Entity {
    const NAME: &'static str = "entity";
    const PAYLOAD_KEY: &'static str = "minecraft:entity";
    // 1.8.0, the first version with `description.identifier`.
    const FLOOR: FormatVersion = known::V1_8_0;
    const SKIP_UPGRADES_KEY: Option<&'static str> = Some("do_not_upgrade");
    const ENVELOPE: Envelope = Envelope {
        // A jsoncpp input: the version from the root member, and
        // rule N1, a member written as `null` is absent.
        input: InputKind::JsonCpp,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // The game takes `format_version` and `minecraft:entity` out of the root and reads nothing
        // else there: it never counts the root's members, so a root member beside the two is
        // passed over and the entity loads.
        check: EnvelopeCheck::Uncounted,
        // The text is the crate's own and the game is silent (it reads an absent version as
        // 0.0.0): an information.
        missing: MissingVersion::Floor {
            code: &bedrock_addon_core::diagnostic::codes::NO_FORMAT_VERSION_FIELD,
            severity: Some(Severity::Info),
        },
        ..Envelope::DEFAULT
    };

    /// A `null` member is absent at every slice (rule N1). The game reads a document below
    /// 1.21.89 with the unstrict error codes and every slice from 1.21.89 on with the strict
    /// ones, the beta struct included.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        let strict = match index {
            StructIndex::Stable(i) => {
                let versions = <history::EntityHistory as VersionChain<Entity>>::VERSIONS;
                versions.get(i).is_some_and(|v| *v >= STRICT_SLICES)
            }
            // The beta slice is read strictly, as every slice after the legacy one.
            StructIndex::Beta => true,
        };
        READING.with_unknown(if strict {
            UnknownMask::Error
        } else {
            UnknownMask::Warn
        })
    }

    type Latest = Entity;
    type Beta = Beta;
    type Component = EntityComponent;
    type History = history::EntityHistory;

    fn components() -> &'static ComponentTable<Self> {
        EntityComponent::TABLE
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
        let at = JsonPath::new().child("components");
        sections.push(ComponentSection::new(at, false, &mut v.components));
        for (name, group) in &mut v.component_groups {
            let at = JsonPath::new().child("component_groups").child(name);
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
    const VERSION: FormatVersion = versions::VERSION;
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
        if finding.is(&core_codes::DEPRECATED_IN_FORMAT) && finding.phase == Phase::Validate {
            finding.code = bedrock_addon_core::diagnostic::cereal::component_not_in_schema();
            finding.severity = severity;
        }
    }
}

impl Entity {
    /// Runs the remaining upgrade steps and the checks on a document
    /// [`parse_as_written`](JsonDocument::parse_as_written) gave: the same result
    /// [`Content::parse`] gives for the file.
    ///
    /// # Errors
    ///
    /// [`Rejected`] when the upgrade steps and the checks leave no entity to keep, or the
    /// document is a beta one (which has no stable struct); it carries every finding.
    // This is `history::EntityHistory::upgrade`, placed on `Entity` because only the document has the
    // origin.
    pub fn upgrade(
        written: Document<history::EntityHistory>,
        cx: &LoadContext,
    ) -> Result<Document<Entity>, Rejected> {
        let (any, origin, version, mut found) = written.into_parsed_parts();
        let rows = found.take_unknown();
        let (identity, skip_upgrades) = history::identity(&any);
        let at = { JsonPath::new().child(<Entity as Family>::PAYLOAD_KEY) };
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
        let parsed = Parsed::<Entity>::from_value(
            Some(any),
            version,
            input,
            at,
            // What the reader moved, without its findings: what the steps and the checks
            // tell is at the member the file has, as in a load of the file.
            // The unknown members go with them, for the steps to move and take.
            {
                let mut moved = found.clone();
                moved.retain(|_| false);
                if !rows.is_empty() {
                    *moved.unknown_mut() = rows;
                }
                moved
            },
        )
        .with_identity(identity)
        .with_actor(actor_type, skip_upgrades);
        let outcome = finish(parsed, cx);
        let mut diagnostics = found;
        diagnostics.append(place(outcome.diagnostics, &outcome.version, &origin));
        match outcome.value {
            Some(Loaded::Stable(entity)) => {
                Ok(Document::new(entity, origin, outcome.version, diagnostics))
            }
            Some(Loaded::Beta(_)) => {
                diagnostics.push(Diagnostic::of(
                    core_codes::NO_PARSER_FOR_VERSION.with(["beta"]),
                ));
                Err(Rejected::new(origin, diagnostics))
            }
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

impl JsonDocument for Entity {
    type Family = Entity;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// `read::load_document`: the chain, the above-the-engine text, the closing line of a refused
    /// file; a beta file is handed back open, a failed one names the identifier its raw text
    /// gives (even when its envelope or version is unreadable).
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<Entity> {
        match load_document(origin, bytes, cx) {
            Ok((Loaded::Stable(entity), version, diagnostics, origin)) => {
                DocumentRead::Stable(Document::new(entity, origin, version, diagnostics))
            }
            Ok((Loaded::Beta(beta), version, diagnostics, origin)) => {
                let id = beta.0.description.identifier.as_str().to_owned();
                DocumentRead::Beta(
                    Some(id),
                    OpenBeta::new(Document::new(beta, origin, version, diagnostics)),
                )
            }
            Err(rejected) => DocumentRead::Rejected(
                identifier_of(bytes).map(|id| id.as_str().to_owned()),
                rejected,
            ),
        }
    }

    /// Parses without upgrading: the struct the file's `format_version` selects, with the
    /// findings of the parse.
    ///
    /// # Errors
    ///
    /// [`Rejected`] when the bytes are not UTF-8, the JSON nests deeper than the limit, or the
    /// parse leaves no entity to keep; it carries every finding.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::EntityHistory>, Rejected> {
        let origin = detached_origin(path);
        let text = decode(bytes, &origin)?;
        let parsed = parse_as_written::<Entity>(text.slice(), cx);
        let mut diagnostics = place(parsed.diagnostics, &parsed.version, &origin);
        if let Some(any) = parsed.value {
            Ok(Document::new(any, origin, parsed.version, diagnostics))
        } else {
            close_refusal(&mut diagnostics, bytes);
            Err(Rejected::new(origin, diagnostics))
        }
    }
}

/// `entities/<namespace>/<name>.json`.
impl JsonPlaced for Entity {
    fn place(&self) -> PackPath {
        default_path(DIRECTORY, &self.description.identifier)
    }
}

impl JsonDefinition for Entity {
    type Kind = kind::Entity;

    fn identifier(&self) -> &EntityId {
        &self.description.identifier
    }
}

// The beta struct keeps its hand impls: its family's `Latest` is `Entity`.
impl Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "beta entity";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Parses one beta entity file (`"format_version": "beta"`); any other version is rejected.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        match Entity::read(detached_origin(path), bytes, cx) {
            DocumentRead::Beta(_, beta) => Ok(beta.keep()),
            DocumentRead::Stable(document) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                diagnostics.push(Diagnostic::of(&core_codes::UNSUPPORTED_VERSION));
                Err(Rejected::new(origin, diagnostics))
            }
            DocumentRead::Rejected(_, rejected) => Err(rejected),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let document = write_envelope::<Entity>(
            BETA_VERSION,
            write::payload(&self.0)?,
            &bedrock_addon_core::document::UnknownMembers::default(),
        );
        write_json(&document, out, style)
    }
}

impl JsonContent for Beta {}

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

/// The loader of `bp.entities()`: every `.json` file below `entities/` (vanilla's directory
/// walk matches `.json` as written), packs bottom-up so a higher pack's definition of an
/// identifier replaces a lower one's.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Entity>, Contribution<Entity>> =
    load::registry::<Entity>(Walk::json_tree_in(&[DIRECTORY]));

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn entity_reads_as_pinned() {
        assert_reading_table(
            "Entity",
            &reading_table::<super::Entity>(),
            r"
                text: Root, clamps beyond a double: true, duplicates kept in: []
                envelope: Uncounted
                missing version: at the floor, told at Info: Some(NoFormatVersionField)
                below floor: UseFormatVersion
                empty document: Error
                1.8.0..=1.21.80 (0..=28): json Absent/Strict/Error/Warn/reports/lenient/Refuse, mcb Own/Strict/Error/Warn/reports/lenient/Refuse
                1.21.89..=1.26.60 (29..=41): json Absent/Strict/Error/Error/reports/lenient/Refuse, mcb Own/Strict/Error/Error/reports/lenient/Refuse
                beta: json Absent/Strict/Error/Error/reports/lenient/Refuse, mcb Own/Strict/Error/Error/reports/lenient/Refuse
            ",
        );
    }
}
