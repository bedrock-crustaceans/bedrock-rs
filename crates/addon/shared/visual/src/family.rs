//! What a family of the shared description is to the code the two families share: [`Visual`]
//! says what differs between client entities and attachables, and the functions here read and
//! write the document of every era from it.

use std::fmt::Debug;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::Profile;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::MolangFields;
use bedrock_addon_core::schema::{FromJson, SchemaFields, derive_support};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::versioning::{
    Envelope, EnvelopeCheck, Family, InputKind, MissingVersion, NullHandling, ParseCx, Reading,
    VersionChain, VersionStruct, write_payload,
};
use facet::Facet;

use crate::history::v1_8_0::ControllerList;
use crate::history::v1_10_0::FoldedLiterals;
use crate::read::{Named, texts};
use crate::rules::ENGINE_1_26_30;
use crate::{Description, Root, SharedDescription};

/// What a family beside the shared members: its own members of the description, flattened
/// into it at its end.
pub trait Own:
    Clone
    + Debug
    + PartialEq
    + Default
    + for<'a> Facet<'a>
    + FromJson
    + derive_support::Fields
    + SchemaFields
    + MolangFields
{
}

impl<T> Own for T where
    T: Clone
        + Debug
        + PartialEq
        + Default
        + for<'a> Facet<'a>
        + FromJson
        + derive_support::Fields
        + SchemaFields
        + MolangFields
{
}

/// The codes of an ignored document: the definition of an identifier a later document of an
/// older `min_engine_version`, or of one above the stack's, did not replace.
#[derive(Clone, Copy, Debug)]
pub struct Shadowed {
    /// A lower `min_engine_version` than the definition's.
    pub older: &'static CodeDef<0>,
    /// A `min_engine_version` above the stack's.
    pub above_stack: &'static CodeDef<0>,
    /// Any other reason.
    pub other: &'static CodeDef<0>,
}

/// A family that reads the description schema client entities and attachables share: where it
/// differs from the other, and the types it is made of. Implemented by the family's marker of
/// the version-chain machinery.
pub trait Visual: Sized + 'static {
    /// The identifier the description names.
    type Id: Named
        + Clone
        + Debug
        + PartialEq
        + for<'a> Facet<'a>
        + FromJson
        + SchemaFields
        + MolangFields;
    /// The members of the description only this family has.
    type Own: Own;
    /// The document's root: the payload of the family and the twin's, which is not read.
    type Root<D: FromJson>: Root<D> + FromJson;
    /// The family of the version-chain machinery, which is this type.
    type Family: Family<Latest = Self::Latest, History = Self::History>;
    /// The newest document.
    type Latest: VersionStruct + SharedDescription;
    /// The document of every era, as written.
    type History: VersionChain<Self::Family>;

    /// The payload key of the family: `minecraft:client_entity` or `minecraft:attachable`.
    const PAYLOAD: &'static str;
    /// The payload key of the twin, which the same root schema declares.
    const TWIN_PAYLOAD: &'static str;
    /// Whether a file with both payloads is told of, with an information at the twin's (the
    /// client entity loader reads its own first and loads it alone).
    const NOTES_TWIN: bool;
    /// D2: whether a number beyond a double of `min_engine_version`'s is read as the largest
    /// finite one, told.
    const CLAMPS_VERSION: bool;
    /// What the 1.8 → 1.10 step does with the list of `animation_controllers`.
    const CONTROLLERS: ControllerList;
    /// D4: whether the 1.10 → 1.26 step tells of a literal the lower-casing changes.
    const FOLDED: FoldedLiterals;
    /// The documents ignored by the shared identifier map, by the reason.
    const SHADOWED: Shadowed;

    /// The wording of a wrong type at a path of a document of the shape `names_only` says
    /// (1.8.x: `render_controllers` holds names only).
    fn wording(names_only: bool) -> fn(&JsonPath) -> Code;

    /// The newest document of `description`, as written at `format_version`.
    fn latest(
        format_version: FormatVersion,
        description: Description<Self::Id, Self::Own>,
    ) -> Self::Latest;

    /// The identifier of the document of whichever era `history` is.
    fn identifier(history: &Self::History) -> &str;

    /// The version `latest` competes with across packs: `description.min_engine_version` read as
    /// a version, or 0.0.0 when it is absent or does not read.
    fn min_engine_version(latest: &Self::Latest) -> SemVersion;
}

/// How every slice reads its description: the profile of the envelope's input kind (the
/// default, [`InputKind::FirstRun`]), its scalars read as vanilla's strict schema types them (a
/// member takes its own JSON type alone, and a number beyond its float is a type error; the
/// floats are clamped in the text before the read).
pub const READING: Reading = InputKind::FirstRun.profile().strict();

/// The envelope of both families' files. Vanilla's root schema declares `format_version`,
/// `minecraft:client_entity` and `minecraft:attachable`; a third member is "child '%s' not
/// valid here.", not an envelope error, so each struct reads the whole document.
pub const ENVELOPE: Envelope = Envelope {
    check: EnvelopeCheck::Never,
    // The game logs this at the content log's warning level.
    missing: MissingVersion::Floor {
        code: &core_codes::VERSION_NOT_GIVEN_LEGACY_FIELDS,
        severity: Some(Severity::Warning),
    },
    ..Envelope::DEFAULT
};

/// The description of `V` in the era of `D`, read from the whole document in `document` as the
/// strict schema reads it: every member the schema does not declare, every `null`, every
/// wrong type and every pattern an error, worded as the game's older tree words it. `None` when
/// the read told an error; the reason is in `d`.
pub fn read_description<V: Visual, D: FromJson>(
    document: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
    names_only: bool,
) -> Option<D> {
    let before = d.len();
    let root: Option<V::Root<D>> =
        bedrock_addon_core::versioning::read_payload_with(document, cx, d, |read| {
            let read = read
                .profile(Profile::FIRST_THEN_PASSES)
                .null(NullHandling::Rejected)
                .unknown_severity(Severity::Error)
                .texts(texts(V::wording(names_only)))
                .clamp_molang();
            if V::CLAMPS_VERSION {
                read.clamp_kept()
            } else {
                read
            }
        });
    let (description, twin) = root?.into_parts();
    if twin && V::NOTES_TWIN {
        // A file with both payloads loads the client entity alone and the game says nothing.
        // The crate notes the payload that is not loaded, and not written, with an information
        // at the member, whatever it holds.
        d.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(cx.at.with(&[V::TWIN_PAYLOAD])));
    }
    // A dropped member is a warning; only an error fails the schema.
    let failed = d
        .iter()
        .skip(before)
        .any(|found| found.severity == Severity::Error);
    (!failed).then_some(description)
}

/// The newest document of `V`, read from the whole document in `document`: its description,
/// and the members `engine` (the engine version of the load) does not declare yet.
pub fn parse_newest<V: Visual>(
    document: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<V::Latest>
where
    Description<V::Id, V::Own>: FromJson,
{
    let before = d.len();
    let description: Description<V::Id, V::Own> = read_description::<V, _>(document, cx, d, false)?;
    let at = cx.at.with(&[V::PAYLOAD, "description"]);
    let slots = &description.slots;
    if cx.load.engine_version < ENGINE_1_26_30 {
        if slots
            .scripts
            .as_ref()
            .is_some_and(|scripts| scripts.hide_held_items.is_some())
        {
            // Undeclared before 1.26.30: "child 'hide_held_items' not valid here.".
            let at = at.with(&["scripts", "hide_held_items"]);
            d.push(crate::read::not_valid_here("hide_held_items", at));
        }
        if matches!(slots.held_item_scale, Some(crate::HeldItemScale::Molang(_))) {
            // Before engine 1.26.30 the schema declares `held_item_scale` as a float only.
            let code = bedrock_addon_core::diagnostic::cereal::unknown_child_option("float");
            d.push(Diagnostic::error(code).at(at.with(&["held_item_scale"])));
        }
    }
    let failed = d
        .iter()
        .skip(before)
        .any(|found| found.severity == Severity::Error);
    (!failed).then(|| V::latest(cx.original_version, description))
}

/// The whole document of a family: `format_version` and the payload holding `description`.
///
/// # Errors
///
/// Returns [`EncodeError`] when the description has no JSON form.
pub fn document<'f, V: Visual, D: Facet<'f>>(
    version: FormatVersion,
    description: &D,
) -> Result<Value, EncodeError> {
    let mut payload = facet_value::VObject::new();
    payload.insert("description", write_payload(description)?);
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version.to_string());
    root.insert(V::PAYLOAD, Value::from(payload));
    Ok(root.into())
}
