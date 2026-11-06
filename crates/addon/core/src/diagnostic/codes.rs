//! The core's generic definitions: one per legacy [`DiagnosticCode`](super::DiagnosticCode)
//! variant, each [`was`](CodeDef::was) the variant it replaces, so a finding raised by one keeps the variant's
//! name, kind and line while its [`id`](CodeDef::id) is already its own (`core/invalid_value`).
//!
//! A variant with a `text` field has two: the plain one renders the variant's own line (its
//! `text: None` form), the `_WORDED` one takes the game's sentence whole as its one argument
//! (its `text: Some(..)` form, for a site whose sentence has no definition of its own yet). A
//! variant whose line branches on its payload has one definition per branch; a typed payload
//! the line does not print rides as the finding's [`DiagnosticDetail`](super::DiagnosticDetail).
//!
//! The [`Class`] is the default a finding of [`Diagnostic::of`](super::Diagnostic::of) gets: a
//! raise site that moves here from `Diagnostic::error`, `warn` or `info` keeps its severity with
//! [`Diagnostic::severity`](super::Diagnostic::severity) wherever the class grades otherwise.
//! No generic definition is [`Class::KeptError`]: a kept error is kept by its site.
//!
//! [`ALL`] lists every definition; `core/tests/diagnostic_codes.rs` lists every id, and
//! `core/tests/diagnostic_messages.rs` renders each against the legacy variant it replaces.

use super::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// `UnsupportedVersion` without the game's sentence: a `format_version` no struct of the family
/// covers: below the floor, above the ceiling, or between two exact versions.
pub static UNSUPPORTED_VERSION: CodeDef<0> = CodeDef::new(
    "core/unsupported_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.was("UnsupportedVersion");

/// `UnsupportedVersion` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static UNSUPPORTED_VERSION_WORDED: CodeDef<1> = CodeDef::new(
    "core/unsupported_version_worded",
    Kind::UnsupportedVersion,
    Class::Error,
    "{}",
)
.game("{}")
.was("UnsupportedVersion");

/// `BetaFeaturesNotEnabled` without the game's sentence: a beta document (`"format_version":
/// "beta"`) without the Beta APIs toggle.
pub static BETA_FEATURES_NOT_ENABLED: CodeDef<0> = CodeDef::new(
    "core/beta_features_not_enabled",
    Kind::Toggle,
    Class::Error,
    "beta document without the Beta APIs toggle",
)
.was("BetaFeaturesNotEnabled");

/// `BetaFeaturesNotEnabled` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static BETA_FEATURES_NOT_ENABLED_WORDED: CodeDef<1> = CodeDef::new(
    "core/beta_features_not_enabled_worded",
    Kind::Toggle,
    Class::Error,
    "{}",
)
.game("{}")
.was("BetaFeaturesNotEnabled");

/// `PrereleaseFormatVersion`: a prerelease-tagged `format_version` (`1.21.60-beta.3`) that loaded;
/// it selects the struct below its release, as vanilla's semantic-version order does. This
/// definition: the version as written.
pub static PRERELEASE_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "core/prerelease_format_version",
    Kind::FormatVersion,
    Class::Lint,
    "format_version {} has a pre-release tag and selects the schema below its release",
)
.was("PrereleaseFormatVersion");

/// `MissingToggles` without the game's sentence: content that needs a toggle or a requirement the
/// load does not satisfy.
pub static MISSING_TOGGLES: CodeDef<0> = CodeDef::new(
    "core/missing_toggles",
    Kind::Toggle,
    Class::Error,
    "needs a toggle that is not enabled",
)
.was("MissingToggles");

/// `MissingToggles` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static MISSING_TOGGLES_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_toggles_worded",
    Kind::Toggle,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingToggles");

/// `DeprecatedMember` without the game's sentence: a deprecated member: moved to a side-car below
/// its deprecation version, dropped at or above it.
pub static DEPRECATED_MEMBER: CodeDef<0> = CodeDef::new(
    "core/deprecated_member",
    Kind::Deprecated,
    Class::Error,
    "deprecated member",
)
.was("DeprecatedMember");

/// `DeprecatedMember` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static DEPRECATED_MEMBER_WORDED: CodeDef<1> = CodeDef::new(
    "core/deprecated_member_worded",
    Kind::Deprecated,
    Class::Error,
    "{}",
)
.game("{}")
.was("DeprecatedMember");

/// `UnrecognizedField` without the game's sentence: a member the schema does not declare.
pub static UNRECOGNIZED_FIELD: CodeDef<0> = CodeDef::new(
    "core/unrecognized_field",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "unrecognized field",
)
.was("UnrecognizedField");

/// `UnrecognizedField` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static UNRECOGNIZED_FIELD_WORDED: CodeDef<1> = CodeDef::new(
    "core/unrecognized_field_worded",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "{}",
)
.game("{}")
.was("UnrecognizedField");

/// `UnrecognizedComponent` without the game's sentence: a component name the family's table does
/// not know.
pub static UNRECOGNIZED_COMPONENT: CodeDef<0> = CodeDef::new(
    "core/unrecognized_component",
    Kind::UnknownMember(Disposition::Kept),
    Class::Error,
    "unrecognized component",
)
.was("UnrecognizedComponent");

/// `UnrecognizedComponent` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static UNRECOGNIZED_COMPONENT_WORDED: CodeDef<1> = CodeDef::new(
    "core/unrecognized_component_worded",
    Kind::UnknownMember(Disposition::Kept),
    Class::Error,
    "{}",
)
.game("{}")
.was("UnrecognizedComponent");

/// `InvalidValue` without the game's sentence: a value outside what the member accepts (range,
/// length, emptiness, UUID form).
pub static INVALID_VALUE: CodeDef<0> = CodeDef::new(
    "core/invalid_value",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.was("InvalidValue");

/// `InvalidValue` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static INVALID_VALUE_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_value_worded",
    Kind::InvalidValue,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidValue");

/// `MissingRequiredField` without the game's sentence: a required member is absent.
pub static MISSING_REQUIRED_FIELD: CodeDef<0> = CodeDef::new(
    "core/missing_required_field",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.was("MissingRequiredField");

/// `MissingRequiredField` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static MISSING_REQUIRED_FIELD_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_required_field_worded",
    Kind::MissingRequired,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingRequiredField");

/// `ClampedValue` without the game's sentence: a value clamped into its range.
pub static CLAMPED_VALUE: CodeDef<0> = CodeDef::new(
    "core/clamped_value",
    Kind::Clamped,
    Class::Lint,
    "value clamped into its range",
)
.was("ClampedValue");

/// `ClampedValue` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static CLAMPED_VALUE_WORDED: CodeDef<1> = CodeDef::new(
    "core/clamped_value_worded",
    Kind::Clamped,
    Class::Lint,
    "{}",
)
.game("{}")
.was("ClampedValue");

/// `DroppedComponent` without the game's sentence: a component dropped by a rule; the rest of the
/// document loads.
pub static DROPPED_COMPONENT: CodeDef<0> = CodeDef::new(
    "core/dropped_component",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "component dropped",
)
.was("DroppedComponent");

/// `DroppedComponent` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static DROPPED_COMPONENT_WORDED: CodeDef<1> = CodeDef::new(
    "core/dropped_component_worded",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "{}",
)
.game("{}")
.was("DroppedComponent");

/// `DroppedField` without the game's sentence: a member dropped by a rule; the rest of the
/// document loads.
pub static DROPPED_FIELD: CodeDef<0> = CodeDef::new(
    "core/dropped_field",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "field dropped",
)
.was("DroppedField");

/// `DroppedField` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static DROPPED_FIELD_WORDED: CodeDef<1> = CodeDef::new(
    "core/dropped_field_worded",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "{}",
)
.game("{}")
.was("DroppedField");

/// `MemberNotKept` without the game's sentence: a member the load read and does not keep: nothing
/// reads it, the loaded document does not hold it and a written document does not have it. Unlike
/// [`UnrecognizedField`](super::DiagnosticCode::UnrecognizedField), whose member is kept and
/// written back, and unlike [`DroppedField`](super::DiagnosticCode::DroppedField), no rule removes
/// it: it is a member no schema declares, at a place that keeps no unknown members.
pub static MEMBER_NOT_KEPT: CodeDef<0> = CodeDef::new(
    "core/member_not_kept",
    Kind::UnknownMember(Disposition::NotKept),
    Class::Lint,
    "member read and not kept",
)
.was("MemberNotKept");

/// `MemberNotKept` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static MEMBER_NOT_KEPT_WORDED: CodeDef<1> = CodeDef::new(
    "core/member_not_kept_worded",
    Kind::UnknownMember(Disposition::NotKept),
    Class::Lint,
    "{}",
)
.game("{}")
.was("MemberNotKept");

/// `ValueRewritten` without the game's sentence: a value the load holds in another form than the
/// file has it in: the member stays, and a written document has the value as it was read, not as
/// it was written (a name in another case, a boolean for a number). Also a value the written
/// format has no spelling for, written as the nearest one it has. Nothing is dropped.
pub static VALUE_REWRITTEN: CodeDef<0> = CodeDef::new(
    "core/value_rewritten",
    Kind::Rewritten,
    Class::Lint,
    "value held in another form than it is written in",
)
.was("ValueRewritten");

/// `ValueRewritten` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static VALUE_REWRITTEN_WORDED: CodeDef<1> = CodeDef::new(
    "core/value_rewritten_worded",
    Kind::Rewritten,
    Class::Lint,
    "{}",
)
.game("{}")
.was("ValueRewritten");

/// `DuplicateComponentVersion`: two versions of one component in one section (a component and the
/// one it supersedes); the older one is dropped. This definition: the arguments are the superseded
/// component, the new one, the new one again and the superseded one again (the game names each
/// twice).
pub static DUPLICATE_COMPONENT_VERSION: CodeDef<4> = CodeDef::new(
    "core/duplicate_component_version",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "Two versions of the same component found: {} and {}. Please remove {}, or upgrade your format version and remove {}.",
)
.game("Two versions of the same component found: {} and {}. Please remove {}, or upgrade your format version and remove {}.")
.was("DuplicateComponentVersion");

/// `UnknownSound` without the game's sentence: a sound event name that no sound definition
/// provides, or a level sound event name the game does not know.
pub static UNKNOWN_SOUND: CodeDef<0> = CodeDef::new(
    "core/unknown_sound",
    Kind::UnresolvedReference,
    Class::Lint,
    "unknown sound event",
)
.was("UnknownSound");

/// `UnknownSound` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static UNKNOWN_SOUND_WORDED: CodeDef<1> = CodeDef::new(
    "core/unknown_sound_worded",
    Kind::UnresolvedReference,
    Class::Lint,
    "{}",
)
.game("{}")
.was("UnknownSound");

/// `GenericGoalSuffixMissing`: a generic AI goal whose name lacks the required suffix.
pub static GENERIC_GOAL_SUFFIX_MISSING: CodeDef<0> = CodeDef::new(
    "core/generic_goal_suffix_missing",
    Kind::Lint,
    Class::Lint,
    "generic goal name lacks its required suffix",
)
.was("GenericGoalSuffixMissing");

/// `MalformedInput`: a JSON syntax error, or a root or envelope that is not the expected document
/// shape. This definition: without a message.
pub static MALFORMED_INPUT: CodeDef<0> = CodeDef::new(
    "core/malformed_input",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.was("MalformedInput");

/// `MalformedInput`: a JSON syntax error, or a root or envelope that is not the expected document
/// shape. This definition: the reader's message as the line.
pub static SYNTAX: CodeDef<1> =
    CodeDef::new("core/syntax", Kind::Syntax, Class::Error, "{}").was("MalformedInput");

/// `MalformedInput` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet. The reader's message rides as
/// `DiagnosticDetail::Syntax`.
pub static MALFORMED_INPUT_WORDED: CodeDef<1> = CodeDef::new(
    "core/malformed_input_worded",
    Kind::Syntax,
    Class::Error,
    "{}",
)
.game("{}")
.was("MalformedInput");

/// `FormatVersionAboveEngine`: a `format_version` above the engine version, parsed as the newest
/// struct. This definition: the arguments are the version as written and the engine version
/// (`version::ENGINE_VERSION`).
pub static FORMAT_VERSION_ABOVE_ENGINE: CodeDef<2> = CodeDef::new(
    "core/format_version_above_engine",
    Kind::FormatVersion,
    Class::Lint,
    "format_version {} is above the engine version {}; parsed as the newest schema",
)
.was("FormatVersionAboveEngine");

/// `ClientEntityShadowed`: a client entity document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: without a reason.
pub static CLIENT_ENTITY_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/client_entity_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "client entity superseded by another definition of its identifier",
)
.was("ClientEntityShadowed");

/// `ClientEntityShadowed`: a client entity document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: for `ShadowReason::OlderMinEngineVersion`; the reason rides as
/// `DiagnosticDetail::Shadow`.
pub static CLIENT_ENTITY_SHADOWED_OLDER: CodeDef<0> = CodeDef::new(
    "core/client_entity_shadowed_older",
    Kind::Shadowed,
    Class::Lint,
    "client entity ignored: its min_engine_version is below the loaded definition's",
)
.was("ClientEntityShadowed");

/// `ClientEntityShadowed`: a client entity document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: for `ShadowReason::MinEngineVersionAboveStack`; the reason rides as
/// `DiagnosticDetail::Shadow`.
pub static CLIENT_ENTITY_SHADOWED_ABOVE_STACK: CodeDef<0> = CodeDef::new(
    "core/client_entity_shadowed_above_stack",
    Kind::Shadowed,
    Class::Lint,
    "client entity ignored: its min_engine_version is above the stack's",
)
.was("ClientEntityShadowed");

/// `AttachableShadowed`: an attachable document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: without a reason.
pub static ATTACHABLE_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/attachable_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "attachable superseded by another definition of its identifier",
)
.was("AttachableShadowed");

/// `AttachableShadowed`: an attachable document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: for `ShadowReason::OlderMinEngineVersion`; the reason rides as
/// `DiagnosticDetail::Shadow`.
pub static ATTACHABLE_SHADOWED_OLDER: CodeDef<0> = CodeDef::new(
    "core/attachable_shadowed_older",
    Kind::Shadowed,
    Class::Lint,
    "attachable ignored: its min_engine_version is below the loaded definition's",
)
.was("AttachableShadowed");

/// `AttachableShadowed`: an attachable document that defines nothing: superseded by a
/// higher-priority pack, or ignored by the shared identifier map for the `reason` given. This
/// definition: for `ShadowReason::MinEngineVersionAboveStack`; the reason rides as
/// `DiagnosticDetail::Shadow`.
pub static ATTACHABLE_SHADOWED_ABOVE_STACK: CodeDef<0> = CodeDef::new(
    "core/attachable_shadowed_above_stack",
    Kind::Shadowed,
    Class::Lint,
    "attachable ignored: its min_engine_version is above the stack's",
)
.was("AttachableShadowed");

/// `ItemBindingShadowed`: a lower-priority item binding was superseded.
pub static ITEM_BINDING_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/item_binding_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "item binding superseded by a higher-priority pack",
)
.was("ItemBindingShadowed");

/// `RenderControllerShadowed`: a lower-priority render controller was superseded.
pub static RENDER_CONTROLLER_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/render_controller_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "render controller superseded by a higher-priority pack",
)
.was("RenderControllerShadowed");

/// `IdentifierCollision` without the game's sentence: two distinct definitions land on one key (a
/// client entity and an attachable sharing an identifier; one item placed twice in the catalog).
pub static IDENTIFIER_COLLISION: CodeDef<0> = CodeDef::new(
    "core/identifier_collision",
    Kind::Duplicate,
    Class::Error,
    "two definitions share one identifier",
)
.was("IdentifierCollision");

/// `IdentifierCollision` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static IDENTIFIER_COLLISION_WORDED: CodeDef<1> = CodeDef::new(
    "core/identifier_collision_worded",
    Kind::Duplicate,
    Class::Error,
    "{}",
)
.game("{}")
.was("IdentifierCollision");

/// `UnresolvedReference` without the game's sentence: a name that must resolve (in another family,
/// the same family or a registry) resolves to nothing. The name is in the diagnostic's
/// [`DiagnosticDetail::Reference`](super::DiagnosticDetail::Reference).
pub static UNRESOLVED_REFERENCE: CodeDef<0> = CodeDef::new(
    "core/unresolved_reference",
    Kind::UnresolvedReference,
    Class::Lint,
    "unresolved reference",
)
.was("UnresolvedReference");

/// `UnresolvedReference` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static UNRESOLVED_REFERENCE_WORDED: CodeDef<1> = CodeDef::new(
    "core/unresolved_reference_worded",
    Kind::UnresolvedReference,
    Class::Lint,
    "{}",
)
.game("{}")
.was("UnresolvedReference");

/// `InvalidReference` without the game's sentence: a reference resolves, but to something the use
/// cannot accept.
pub static INVALID_REFERENCE: CodeDef<0> = CodeDef::new(
    "core/invalid_reference",
    Kind::InvalidReference,
    Class::Error,
    "reference resolves to something this use cannot accept",
)
.was("InvalidReference");

/// `InvalidReference` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static INVALID_REFERENCE_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_reference_worded",
    Kind::InvalidReference,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidReference");

/// `TextureNotFound` without the game's sentence: a referenced texture file is in no pack of the
/// stack.
pub static TEXTURE_NOT_FOUND: CodeDef<0> = CodeDef::new(
    "core/texture_not_found",
    Kind::UnresolvedReference,
    Class::Error,
    "texture not found",
)
.was("TextureNotFound");

/// `TextureNotFound` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static TEXTURE_NOT_FOUND_WORDED: CodeDef<1> = CodeDef::new(
    "core/texture_not_found_worded",
    Kind::UnresolvedReference,
    Class::Error,
    "{}",
)
.game("{}")
.was("TextureNotFound");

/// `SplashesReplaced`: the pack's splashes replace the lower packs' splashes.
pub static SPLASHES_REPLACED: CodeDef<0> = CodeDef::new(
    "core/splashes_replaced",
    Kind::Overridden,
    Class::Lint,
    "splashes replace the lower packs' splashes",
)
.was("SplashesReplaced");

/// `NoSplashes`: the stack has no splash texts.
pub static NO_SPLASHES: CodeDef<0> = CodeDef::new(
    "core/no_splashes",
    Kind::Overridden,
    Class::Lint,
    "no splash texts",
)
.was("NoSplashes");

/// `UnresolvedLangKey`: a localisation key that no `.lang` file defines.
pub static UNRESOLVED_LANG_KEY: CodeDef<0> = CodeDef::new(
    "core/unresolved_lang_key",
    Kind::UnresolvedReference,
    Class::Lint,
    "localisation key not defined",
)
.was("UnresolvedLangKey");

/// `IgnoredFile` without the game's sentence: a file the loader does not read (wrong extension,
/// wrong place).
pub static IGNORED_FILE: CodeDef<0> =
    CodeDef::new("core/ignored_file", Kind::Io, Class::Lint, "file ignored").was("IgnoredFile");

/// `IgnoredFile` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static IGNORED_FILE_WORDED: CodeDef<1> =
    CodeDef::new("core/ignored_file_worded", Kind::Io, Class::Lint, "{}")
        .game("{}")
        .was("IgnoredFile");

/// `PackIconMissing`: the pack has no `pack_icon.png`.
pub static PACK_ICON_MISSING: CodeDef<0> = CodeDef::new(
    "core/pack_icon_missing",
    Kind::Asset,
    Class::Error,
    "pack_icon.png missing",
)
.was("PackIconMissing");

/// `PackIconShape`: `pack_icon.png` is not square or not a power of two.
pub static PACK_ICON_SHAPE: CodeDef<0> = CodeDef::new(
    "core/pack_icon_shape",
    Kind::Asset,
    Class::Lint,
    "pack_icon.png is not a square image",
)
.was("PackIconShape");

/// `GlyphSheetShape`: a glyph sheet whose size is not a 16 × 16 grid.
pub static GLYPH_SHEET_SHAPE: CodeDef<0> = CodeDef::new(
    "core/glyph_sheet_shape",
    Kind::Asset,
    Class::Lint,
    "glyph sheet is not a 16 x 16 grid",
)
.was("GlyphSheetShape");

/// `SkinPackNameMismatch`: a skin pack's name does not match its localisation.
pub static SKIN_PACK_NAME_MISMATCH: CodeDef<0> = CodeDef::new(
    "core/skin_pack_name_mismatch",
    Kind::Pack,
    Class::Lint,
    "skin pack name does not match its localisation",
)
.was("SkinPackNameMismatch");

/// `SkinPackCreatorRule`: a skin pack breaks a creator rule.
pub static SKIN_PACK_CREATOR_RULE: CodeDef<0> = CodeDef::new(
    "core/skin_pack_creator_rule",
    Kind::Pack,
    Class::Lint,
    "skin pack breaks a creator rule",
)
.was("SkinPackCreatorRule");

/// `IgnoredMember` without the game's sentence: a member that is read but dropped, or that has no
/// effect.
pub static IGNORED_MEMBER: CodeDef<0> = CodeDef::new(
    "core/ignored_member",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Lint,
    "member ignored",
)
.was("IgnoredMember");

/// `IgnoredMember` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static IGNORED_MEMBER_WORDED: CodeDef<1> = CodeDef::new(
    "core/ignored_member_worded",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Lint,
    "{}",
)
.game("{}")
.was("IgnoredMember");

/// `UnmatchedCondition`: a UI `where` / `target` condition that matched no element.
pub static UNMATCHED_CONDITION: CodeDef<0> = CodeDef::new(
    "core/unmatched_condition",
    Kind::NeverApplies,
    Class::Lint,
    "condition matched no element",
)
.was("UnmatchedCondition");

/// `WhereArrayNeverMatches`: an array-valued UI `where`, which vanilla never matches.
pub static WHERE_ARRAY_NEVER_MATCHES: CodeDef<0> = CodeDef::new(
    "core/where_array_never_matches",
    Kind::NeverApplies,
    Class::Lint,
    "an array-valued `where` never matches",
)
.was("WhereArrayNeverMatches");

/// `RebasedOverride`: a UI overlay `name@base` key shadows a base short name.
pub static REBASED_OVERRIDE: CodeDef<0> = CodeDef::new(
    "core/rebased_override",
    Kind::Shadowed,
    Class::Lint,
    "overlay `name@base` key shadows a base name",
)
.was("RebasedOverride");

/// `ShadowedDefinition` without the game's sentence: a definition from a lower-priority source was
/// replaced, or a higher one ignored.
pub static SHADOWED_DEFINITION: CodeDef<0> = CodeDef::new(
    "core/shadowed_definition",
    Kind::Shadowed,
    Class::Lint,
    "definition shadowed by a higher-priority source",
)
.was("ShadowedDefinition");

/// `ShadowedDefinition` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static SHADOWED_DEFINITION_WORDED: CodeDef<1> = CodeDef::new(
    "core/shadowed_definition_worded",
    Kind::Shadowed,
    Class::Lint,
    "{}",
)
.game("{}")
.was("ShadowedDefinition");

/// `ReadError`: a file could not be read from its pack. This definition: the argument is the
/// `io::ErrorKind` as `Display` prints it; the kind rides as `DiagnosticDetail::Io`.
pub static READ_ERROR: CodeDef<1> = CodeDef::new(
    "core/read_error",
    Kind::Io,
    Class::Error,
    "cannot read the file ({})",
)
.was("ReadError");

/// `ReadError` with a sentence of the game's (or the family's) given whole as the argument: a site
/// whose template has no definition of its own yet. The `io::ErrorKind` rides as
/// `DiagnosticDetail::Io`.
pub static READ_ERROR_WORDED: CodeDef<1> =
    CodeDef::new("core/read_error_worded", Kind::Io, Class::Error, "{}")
        .game("{}")
        .was("ReadError");

/// `MissingPayload` without the game's sentence: an envelope without the family's payload key.
pub static MISSING_PAYLOAD: CodeDef<0> = CodeDef::new(
    "core/missing_payload",
    Kind::Envelope,
    Class::Error,
    "the document has no payload member",
)
.was("MissingPayload");

/// `MissingPayload` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static MISSING_PAYLOAD_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_payload_worded",
    Kind::Envelope,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingPayload");

/// `NamespaceRule` without the game's sentence: a namespace rule failed, or an identifier that
/// must be namespaced is not.
pub static NAMESPACE_RULE: CodeDef<0> = CodeDef::new(
    "core/namespace_rule",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.was("NamespaceRule");

/// `NamespaceRule` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static NAMESPACE_RULE_WORDED: CodeDef<1> = CodeDef::new(
    "core/namespace_rule_worded",
    Kind::Identifier,
    Class::Error,
    "{}",
)
.game("{}")
.was("NamespaceRule");

/// `InvalidType` without the game's sentence: a member of the wrong JSON type or shape, including
/// a failed typed deserialisation.
pub static INVALID_TYPE: CodeDef<0> = CodeDef::new(
    "core/invalid_type",
    Kind::WrongType,
    Class::Error,
    "wrong JSON type",
)
.was("InvalidType");

/// `InvalidType` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static INVALID_TYPE_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_type_worded",
    Kind::WrongType,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidType");

/// `SkippedCacheDocument`: a binary cache document recognised by its version string; its body was
/// not parsed.
pub static SKIPPED_CACHE_DOCUMENT: CodeDef<0> = CodeDef::new(
    "core/skipped_cache_document",
    Kind::Io,
    Class::Lint,
    "cache document not parsed",
)
.was("SkippedCacheDocument");

/// `MissingFormatVersion` without the game's sentence: `format_version` is absent (or, where the
/// family says so, unreadable).
pub static MISSING_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "core/missing_format_version",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.was("MissingFormatVersion");

/// `MissingFormatVersion` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static MISSING_FORMAT_VERSION_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_format_version_worded",
    Kind::FormatVersion,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingFormatVersion");

/// `MalformedFormatVersion` without the game's sentence: `format_version` is present but neither a
/// semantic-version string nor three integers, or of a shape the family's envelope rejects.
/// Without `text`: vanilla's "Malformed format version".
pub static MALFORMED_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "core/malformed_format_version",
    Kind::FormatVersion,
    Class::Error,
    "Malformed format version",
)
.was("MalformedFormatVersion");

/// `MalformedFormatVersion` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static MALFORMED_FORMAT_VERSION_WORDED: CodeDef<1> = CodeDef::new(
    "core/malformed_format_version_worded",
    Kind::FormatVersion,
    Class::Error,
    "{}",
)
.game("{}")
.was("MalformedFormatVersion");

/// `EmptyDocument` without the game's sentence: the file is empty, or parses to nothing / `null`.
pub static EMPTY_DOCUMENT: CodeDef<0> = CodeDef::new(
    "core/empty_document",
    Kind::Syntax,
    Class::Lint,
    "empty document",
)
.was("EmptyDocument");

/// `EmptyDocument` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static EMPTY_DOCUMENT_WORDED: CodeDef<1> = CodeDef::new(
    "core/empty_document_worded",
    Kind::Syntax,
    Class::Lint,
    "{}",
)
.game("{}")
.was("EmptyDocument");

/// `UnusualFormatVersion`: a version between two known slices, parsed by the lower one.
pub static UNUSUAL_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "core/unusual_format_version",
    Kind::FormatVersion,
    Class::Lint,
    "format_version between two known versions; parsed by the lower one",
)
.was("UnusualFormatVersion");

/// `VanillaRejectsVersion`: a version the crate accepts that vanilla 1.26.20 has no parser for.
pub static VANILLA_REJECTS_VERSION: CodeDef<0> = CodeDef::new(
    "core/vanilla_rejects_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "vanilla has no parser for this format_version",
)
.was("VanillaRejectsVersion");

/// `FormatVersionArray`: `format_version` given as an array of three integers (accepted).
pub static FORMAT_VERSION_ARRAY: CodeDef<0> = CodeDef::new(
    "core/format_version_array",
    Kind::FormatVersion,
    Class::Error,
    "format_version given as an array",
)
.was("FormatVersionArray");

/// `PayloadKeyMismatch`: the payload member is not the family's payload key; loaded anyway.
pub static PAYLOAD_KEY_MISMATCH: CodeDef<0> = CodeDef::new(
    "core/payload_key_mismatch",
    Kind::Envelope,
    Class::Lint,
    "payload member is not the family's payload key",
)
.was("PayloadKeyMismatch");

/// `UnknownPayloadKey`: the payload key names no registered feature type. This definition: the
/// argument is the key.
pub static UNKNOWN_PAYLOAD_KEY: CodeDef<1> = CodeDef::new(
    "core/unknown_payload_key",
    Kind::Envelope,
    Class::Error,
    "Could not find registered feature type that matches name '{}'",
)
.game("Could not find registered feature type that matches name '{}'")
.was("UnknownPayloadKey");

/// `IgnoredPayload`: a second payload in one file is ignored.
pub static IGNORED_PAYLOAD: CodeDef<0> = CodeDef::new(
    "core/ignored_payload",
    Kind::Envelope,
    Class::Error,
    "second payload ignored",
)
.was("IgnoredPayload");

/// `DocumentRejected` without the game's sentence: the closing error of a document that failed
/// schema validation.
pub static DOCUMENT_REJECTED: CodeDef<0> = CodeDef::new(
    "core/document_rejected",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.was("DocumentRejected");

/// `DocumentRejected` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static DOCUMENT_REJECTED_WORDED: CodeDef<1> = CodeDef::new(
    "core/document_rejected_worded",
    Kind::Closing,
    Class::Error,
    "{}",
)
.game("{}")
.was("DocumentRejected");

/// `MissingIdentifier` without the game's sentence: a document without a usable identifier.
pub static MISSING_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "core/missing_identifier",
    Kind::MissingRequired,
    Class::Error,
    "no usable identifier",
)
.was("MissingIdentifier");

/// `MissingIdentifier` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static MISSING_IDENTIFIER_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_identifier_worded",
    Kind::MissingRequired,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingIdentifier");

/// `SoundDefinitionsWrapperWithoutVersion`: a `{ "sound_definitions": … }` root without a version
/// of at least 1.14.0, which vanilla wraps again.
pub static SOUND_DEFINITIONS_WRAPPER_WITHOUT_VERSION: CodeDef<0> = CodeDef::new(
    "core/sound_definitions_wrapper_without_version",
    Kind::Envelope,
    Class::Lint,
    "`sound_definitions` wrapper without a format_version of at least 1.14.0",
)
.was("SoundDefinitionsWrapperWithoutVersion");

/// `MissingMinEngineVersion`: the pack has no `min_engine_version`, so the command version is
/// unset.
pub static MISSING_MIN_ENGINE_VERSION: CodeDef<0> = CodeDef::new(
    "core/missing_min_engine_version",
    Kind::EngineVersion,
    Class::Error,
    "the pack has no min_engine_version",
)
.was("MissingMinEngineVersion");

/// `FieldSinceEngine`: a field newer than the pack's `min_engine_version` (or the configured
/// target engine, for server configuration). This definition: the argument is the engine version,
/// `major.minor.patch`.
pub static FIELD_SINCE_ENGINE: CodeDef<1> = CodeDef::new(
    "core/field_since_engine",
    Kind::EngineVersion,
    Class::Error,
    "field needs engine version {} or newer",
)
.was("FieldSinceEngine");

/// `FieldUntilEngine`: a field the configured target engine no longer has, although an earlier
/// engine had it. This definition: the argument is the engine version, `major.minor.patch`.
pub static FIELD_UNTIL_ENGINE: CodeDef<1> = CodeDef::new(
    "core/field_until_engine",
    Kind::EngineVersion,
    Class::Error,
    "field is gone from this engine; engine version {} was the last known to have it",
)
.was("FieldUntilEngine");

/// `UnsupportedValueForm`: a value form the configured engine does not support yet.
pub static UNSUPPORTED_VALUE_FORM: CodeDef<0> = CodeDef::new(
    "core/unsupported_value_form",
    Kind::EngineVersion,
    Class::Error,
    "value form not supported by the configured engine",
)
.was("UnsupportedValueForm");

/// `InvalidEnumValue` without the game's sentence: a string or discriminator outside its enum.
pub static INVALID_ENUM_VALUE: CodeDef<0> = CodeDef::new(
    "core/invalid_enum_value",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.was("InvalidEnumValue");

/// `InvalidEnumValue` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static INVALID_ENUM_VALUE_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_enum_value_worded",
    Kind::InvalidEnumValue,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidEnumValue");

/// `InvalidMolang`: one parser message of a Molang source, or a rejecting lint. The compiler's
/// message is the diagnostic's [`DiagnosticDetail::Molang`](super::DiagnosticDetail) (feature
/// `molang`). This definition: the argument is the expression.
pub static INVALID_MOLANG: CodeDef<1> =
    CodeDef::new("core/invalid_molang", Kind::Molang, Class::Error, "{}").was("InvalidMolang");

/// `MolangLint`: a non-rejecting Molang lint, or a behaviour-pack expression the Molang crate
/// cannot evaluate. This definition: the argument is the expression.
pub static MOLANG_LINT: CodeDef<1> =
    CodeDef::new("core/molang_lint", Kind::Molang, Class::Error, "{}").was("MolangLint");

/// `MolangNotChecked`: molang stored unchecked, one per document: the `molang` feature is off, or
/// it is on and no Molang check was given (`LoadOptions::molang` is `None`, the default). The text
/// names the one that holds. The argument is the count of fields; the line names the cause that
/// holds in this build.
#[cfg(not(feature = "molang"))]
pub static MOLANG_NOT_CHECKED: CodeDef<1> = CodeDef::new(
    "core/molang_not_checked",
    Kind::Molang,
    Class::Lint,
    "{} Molang field(s) not checked: the `molang` feature is off",
)
.was("MolangNotChecked");
/// `MolangNotChecked`: molang stored unchecked, one per document: the `molang` feature is off, or
/// it is on and no Molang check was given (`LoadOptions::molang` is `None`, the default). The text
/// names the one that holds. The argument is the count of fields; the line names the cause that
/// holds in this build.
#[cfg(feature = "molang")]
pub static MOLANG_NOT_CHECKED: CodeDef<1> = CodeDef::new(
    "core/molang_not_checked",
    Kind::Molang,
    Class::Lint,
    "{} Molang field(s) not checked: no Molang check was given (`LoadOptions::molang` is `None`)",
)
.was("MolangNotChecked");

/// `KeyframeOutOfDomain`: a keyframe key outside `[0, 1]`; the key is dropped.
pub static KEYFRAME_OUT_OF_DOMAIN: CodeDef<0> = CodeDef::new(
    "core/keyframe_out_of_domain",
    Kind::OutOfRange,
    Class::Lint,
    "keyframe key outside [0, 1]; dropped",
)
.was("KeyframeOutOfDomain");

/// `LossyConversion`: a value written back out loses precision.
pub static LOSSY_CONVERSION: CodeDef<0> = CodeDef::new(
    "core/lossy_conversion",
    Kind::Rewritten,
    Class::Lint,
    "value loses precision when written",
)
.was("LossyConversion");

/// `LiteralIgnored`: a literal ignored because a key form takes precedence, or because a second
/// literal is parsed into the same member.
pub static LITERAL_IGNORED: CodeDef<0> = CodeDef::new(
    "core/literal_ignored",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Lint,
    "literal ignored",
)
.was("LiteralIgnored");

/// `NonCanonicalMemberCase`: a member name matched case-insensitively.
pub static NON_CANONICAL_MEMBER_CASE: CodeDef<0> = CodeDef::new(
    "core/non_canonical_member_case",
    Kind::Rewritten,
    Class::Lint,
    "member name matched ignoring case",
)
.was("NonCanonicalMemberCase");

/// `CaseFoldedTransition`: a transition expression lower-cased below 1.17.30.
pub static CASE_FOLDED_TRANSITION: CodeDef<0> = CodeDef::new(
    "core/case_folded_transition",
    Kind::Rewritten,
    Class::Lint,
    "transition expression lower-cased",
)
.was("CaseFoldedTransition");

/// `UnorderedRemapCurve`: `remap_curve` keys whose string order differs from their numeric order.
pub static UNORDERED_REMAP_CURVE: CodeDef<0> = CodeDef::new(
    "core/unordered_remap_curve",
    Kind::Lint,
    Class::Lint,
    "remap_curve keys out of numeric order",
)
.was("UnorderedRemapCurve");

/// `ImplicitInitialState`: no `initial_state` and no `default` state; the first state is used.
pub static IMPLICIT_INITIAL_STATE: CodeDef<0> = CodeDef::new(
    "core/implicit_initial_state",
    Kind::Lint,
    Class::Lint,
    "no initial_state and no default state; the first state is used",
)
.was("ImplicitInitialState");

/// `UnrecognizedEventString`: an animation event string that is neither a command, an entity event
/// nor Molang.
pub static UNRECOGNIZED_EVENT_STRING: CodeDef<0> = CodeDef::new(
    "core/unrecognized_event_string",
    Kind::Lint,
    Class::Lint,
    "event is neither a command, an entity event nor Molang",
)
.was("UnrecognizedEventString");

/// `InvalidNode`: a behavior-tree child that is not an object with exactly one member.
pub static INVALID_NODE: CodeDef<0> = CodeDef::new(
    "core/invalid_node",
    Kind::Syntax,
    Class::Lint,
    "node is not an object with exactly one member",
)
.was("InvalidNode");

/// `InvalidRoot`: `minecraft:root` is null, not an object, or not exactly one member.
pub static INVALID_ROOT: CodeDef<0> = CodeDef::new(
    "core/invalid_root",
    Kind::Syntax,
    Class::Error,
    "minecraft:root is not an object with exactly one member",
)
.was("InvalidRoot");

/// `UnknownNode`: an unknown behavior-tree node type. This definition: the argument is the node's
/// name.
pub static UNKNOWN_NODE: CodeDef<1> = CodeDef::new(
    "core/unknown_node",
    Kind::InvalidEnumValue,
    Class::Error,
    "Unknown behavior {}",
)
.game("Unknown behavior {}")
.was("UnknownNode");

/// `LegacyItemFormat`: an item parsed with the legacy (non-component) schema. This definition: the
/// argument is the reason.
pub static LEGACY_ITEM_FORMAT: CodeDef<1> = CodeDef::new(
    "core/legacy_item_format",
    Kind::Deprecated,
    Class::Lint,
    "item parsed with the legacy schema: {}",
)
.was("LegacyItemFormat");

/// `PersonaPiecePack`: a persona-piece pack placed in the resource-pack directory.
pub static PERSONA_PIECE_PACK: CodeDef<0> = CodeDef::new(
    "core/persona_piece_pack",
    Kind::Pack,
    Class::Lint,
    "persona-piece pack in the resource-pack directory",
)
.was("PersonaPiecePack");

/// `LangKeyTrailingSpace`: a `.lang` key ends, or its value starts, with a space.
pub static LANG_KEY_TRAILING_SPACE: CodeDef<0> = CodeDef::new(
    "core/lang_key_trailing_space",
    Kind::Lint,
    Class::Lint,
    "key ends, or value starts, with a space",
)
.was("LangKeyTrailingSpace");

/// `ScatterParamValue` without the game's sentence: a scatter parameter vanilla logs and loads
/// anyway; vanilla's replacement applies.
pub static SCATTER_PARAM_VALUE: CodeDef<0> = CodeDef::new(
    "core/scatter_param_value",
    Kind::InvalidValue,
    Class::Error,
    "invalid scatter parameter",
)
.was("ScatterParamValue");

/// `ScatterParamValue` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static SCATTER_PARAM_VALUE_WORDED: CodeDef<1> = CodeDef::new(
    "core/scatter_param_value_worded",
    Kind::InvalidValue,
    Class::Error,
    "{}",
)
.game("{}")
.was("ScatterParamValue");

/// `UnsupportedOperator`: `<=`, `>=` or `%` used as an operator in a UI binding expression.
pub static UNSUPPORTED_OPERATOR: CodeDef<0> = CodeDef::new(
    "core/unsupported_operator",
    Kind::InvalidValue,
    Class::Error,
    "`<=`, `>=` and `%` are not operators",
)
.was("UnsupportedOperator");

/// `InvalidTexture` without the game's sentence: an image that is not decodable or has the wrong
/// proportions.
pub static INVALID_TEXTURE: CodeDef<0> = CodeDef::new(
    "core/invalid_texture",
    Kind::Asset,
    Class::Error,
    "image not decodable or of the wrong proportions",
)
.was("InvalidTexture");

/// `InvalidTexture` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static INVALID_TEXTURE_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_texture_worded",
    Kind::Asset,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidTexture");

/// `LayerImageUndecodable`: a texture-set layer image cannot be decoded.
pub static LAYER_IMAGE_UNDECODABLE: CodeDef<0> = CodeDef::new(
    "core/layer_image_undecodable",
    Kind::Asset,
    Class::Error,
    "layer image cannot be decoded",
)
.was("LayerImageUndecodable");

/// `LayerImageFormat`: a texture-set layer image has an unsupported channel count.
pub static LAYER_IMAGE_FORMAT: CodeDef<0> = CodeDef::new(
    "core/layer_image_format",
    Kind::Asset,
    Class::Error,
    "layer image has an unsupported channel count",
)
.was("LayerImageFormat");

/// `CapabilityHint` without the game's sentence: content that only takes effect with a manifest
/// capability the pack does not declare.
pub static CAPABILITY_HINT: CodeDef<0> = CodeDef::new(
    "core/capability_hint",
    Kind::Toggle,
    Class::Error,
    "takes effect only with a capability the pack does not declare",
)
.was("CapabilityHint");

/// `CapabilityHint` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static CAPABILITY_HINT_WORDED: CodeDef<1> = CodeDef::new(
    "core/capability_hint_worded",
    Kind::Toggle,
    Class::Error,
    "{}",
)
.game("{}")
.was("CapabilityHint");

/// `VibrantVisualsDisabledByPack`: a pack in the stack disables Vibrant Visuals.
pub static VIBRANT_VISUALS_DISABLED_BY_PACK: CodeDef<0> = CodeDef::new(
    "core/vibrant_visuals_disabled_by_pack",
    Kind::Toggle,
    Class::Error,
    "a pack in the stack disables Vibrant Visuals",
)
.was("VibrantVisualsDisabledByPack");

/// `FontFileMissing` without the game's sentence: a font's data file is found under none of the
/// tried paths.
pub static FONT_FILE_MISSING: CodeDef<0> = CodeDef::new(
    "core/font_file_missing",
    Kind::UnresolvedReference,
    Class::Error,
    "font file not found",
)
.was("FontFileMissing");

/// `FontFileMissing` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static FONT_FILE_MISSING_WORDED: CodeDef<1> = CodeDef::new(
    "core/font_file_missing_worded",
    Kind::UnresolvedReference,
    Class::Error,
    "{}",
)
.game("{}")
.was("FontFileMissing");

/// `FontNameOverridden`: a later `fonts[]` entry replaces an earlier one of the same name.
pub static FONT_NAME_OVERRIDDEN: CodeDef<0> = CodeDef::new(
    "core/font_name_overridden",
    Kind::Overridden,
    Class::Lint,
    "font replaced by a later entry",
)
.was("FontNameOverridden");

/// `MixedFontPage`: a font page's files come from different packs.
pub static MIXED_FONT_PAGE: CodeDef<0> = CodeDef::new(
    "core/mixed_font_page",
    Kind::Lint,
    Class::Lint,
    "font page files come from different packs",
)
.was("MixedFontPage");

/// `AtomicFilterLightingIgnored`: `filter_lighting: true` on an atomic client entity's controller
/// is ignored.
pub static ATOMIC_FILTER_LIGHTING_IGNORED: CodeDef<0> = CodeDef::new(
    "core/atomic_filter_lighting_ignored",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Error,
    "Atomic Entity - filter_lighting is not supported.",
)
.game("Atomic Entity - filter_lighting is not supported.")
.was("AtomicFilterLightingIgnored");

/// `AtomicUnsupportedRenderController`: a render controller uses features atomic entities do not
/// support; skipped for that entity.
pub static ATOMIC_UNSUPPORTED_RENDER_CONTROLLER: CodeDef<0> = CodeDef::new(
    "core/atomic_unsupported_render_controller",
    Kind::InvalidReference,
    Class::Error,
    "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).",
)
.game("Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).")
.was("AtomicUnsupportedRenderController");

/// `DuplicateIdentifier` without the game's sentence: two definitions of one identifier where one
/// is dropped by a rule other than pack priority.
pub static DUPLICATE_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "core/duplicate_identifier",
    Kind::Duplicate,
    Class::Error,
    "duplicate identifier",
)
.was("DuplicateIdentifier");

/// `DuplicateIdentifier` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static DUPLICATE_IDENTIFIER_WORDED: CodeDef<1> = CodeDef::new(
    "core/duplicate_identifier_worded",
    Kind::Duplicate,
    Class::Error,
    "{}",
)
.game("{}")
.was("DuplicateIdentifier");

/// `DuplicateBone`: two bones that collide after lower-casing.
pub static DUPLICATE_BONE: CodeDef<0> = CodeDef::new(
    "core/duplicate_bone",
    Kind::Duplicate,
    Class::Lint,
    "two bones collide after lower-casing",
)
.was("DuplicateBone");

/// `AtomicClientEntityShadowed`: a lower-priority atomic client entity was superseded.
pub static ATOMIC_CLIENT_ENTITY_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/atomic_client_entity_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "atomic client entity superseded by a higher-priority pack",
)
.was("AtomicClientEntityShadowed");

/// `AnimationControllerShadowed`: a lower-priority animation controller was superseded.
pub static ANIMATION_CONTROLLER_SHADOWED: CodeDef<0> = CodeDef::new(
    "core/animation_controller_shadowed",
    Kind::Shadowed,
    Class::Lint,
    "animation controller superseded by a higher-priority pack",
)
.was("AnimationControllerShadowed");

/// `ShadowedByBuiltin`: a pack tree replaced by a built-in tree of the same name.
pub static SHADOWED_BY_BUILTIN: CodeDef<0> = CodeDef::new(
    "core/shadowed_by_builtin",
    Kind::Shadowed,
    Class::Lint,
    "replaced by a built-in definition",
)
.was("ShadowedByBuiltin");

/// `ReferenceCycle` without the game's sentence: references that form a cycle.
pub static REFERENCE_CYCLE: CodeDef<0> = CodeDef::new(
    "core/reference_cycle",
    Kind::Cycle,
    Class::Error,
    "references form a cycle",
)
.was("ReferenceCycle");

/// `ReferenceCycle` with a sentence of the game's (or the family's) given whole as the argument: a
/// site whose template has no definition of its own yet.
pub static REFERENCE_CYCLE_WORDED: CodeDef<1> = CodeDef::new(
    "core/reference_cycle_worded",
    Kind::Cycle,
    Class::Error,
    "{}",
)
.game("{}")
.was("ReferenceCycle");

/// `ClientBiomeShadowsLowerPacks`: a client biome that fails validation still claims its
/// identifier from lower packs.
pub static CLIENT_BIOME_SHADOWS_LOWER_PACKS: CodeDef<0> = CodeDef::new(
    "core/client_biome_shadows_lower_packs",
    Kind::Shadowed,
    Class::Lint,
    "invalid client biome still shadows lower packs",
)
.was("ClientBiomeShadowsLowerPacks");

/// `ClientBiomeOverriddenByBiomesClient`: `biomes_client.json` overrides a client biome's
/// appearance.
pub static CLIENT_BIOME_OVERRIDDEN_BY_BIOMES_CLIENT: CodeDef<0> = CodeDef::new(
    "core/client_biome_overridden_by_biomes_client",
    Kind::Overridden,
    Class::Lint,
    "biomes_client.json overrides this client biome",
)
.was("ClientBiomeOverriddenByBiomesClient");

/// `VanillaBiomesClientSkipped`: the base-game `biomes_client.json` is not loaded for this world
/// version.
pub static VANILLA_BIOMES_CLIENT_SKIPPED: CodeDef<0> = CodeDef::new(
    "core/vanilla_biomes_client_skipped",
    Kind::Overridden,
    Class::Lint,
    "base-game biomes_client.json not loaded for this world version",
)
.was("VanillaBiomesClientSkipped");

/// `LegacyFogIgnored`: a legacy fog entry for a biome no earlier file has is not used.
pub static LEGACY_FOG_IGNORED: CodeDef<0> = CodeDef::new(
    "core/legacy_fog_ignored",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Lint,
    "legacy fog entry not used",
)
.was("LegacyFogIgnored");

/// `BiomeFormatHint`: a biome document in a format vanilla no longer loads (below 1.21.110), with
/// how to move it to a supported one. This definition: for a 1.12 document (no version).
pub static BIOME_FORMAT_HINT_LEGACY: CodeDef<0> = CodeDef::new(
    "core/biome_format_hint_legacy",
    Kind::Lint,
    Class::Lint,
    "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher",
)
.was("BiomeFormatHint");

/// `BiomeFormatHint`: a biome document in a format vanilla no longer loads (below 1.21.110), with
/// how to move it to a supported one. This definition: for an experimental format without surface
/// components; the argument is the version, the version rides as
/// `DiagnosticDetail::FormatVersion`.
pub static BIOME_FORMAT_HINT_EXPERIMENTAL: CodeDef<1> = CodeDef::new(
    "core/biome_format_hint_experimental",
    Kind::Lint,
    Class::Lint,
    "biome format {} was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation)",
)
.was("BiomeFormatHint");

/// `BiomeFormatHint`: a biome document in a format vanilla no longer loads (below 1.21.110), with
/// how to move it to a supported one. This definition: for an experimental format with surface
/// components; the argument is the version.
pub static BIOME_FORMAT_HINT_EXPERIMENTAL_SURFACE: CodeDef<1> = CodeDef::new(
    "core/biome_format_hint_experimental_surface",
    Kind::Lint,
    Class::Lint,
    "biome format {} was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder)",
)
.was("BiomeFormatHint");

/// `NoEffect` without the game's sentence: a member that has no effect where it is written.
pub static NO_EFFECT: CodeDef<0> = CodeDef::new(
    "core/no_effect",
    Kind::NeverApplies,
    Class::Lint,
    "member has no effect",
)
.was("NoEffect");

/// `NoEffect` with a sentence of the game's (or the family's) given whole as the argument: a site
/// whose template has no definition of its own yet.
pub static NO_EFFECT_WORDED: CodeDef<1> = CodeDef::new(
    "core/no_effect_worded",
    Kind::NeverApplies,
    Class::Lint,
    "{}",
)
.game("{}")
.was("NoEffect");

/// `SynthesizedComponent`: an upgrade step inserted a component.
pub static SYNTHESIZED_COMPONENT: CodeDef<0> = CodeDef::new(
    "core/synthesized_component",
    Kind::Rewritten,
    Class::Lint,
    "component inserted by an upgrade step",
)
.was("SynthesizedComponent");

/// `IdentifierNamespaced`: an upgrade step prefixed `minecraft:` to an identifier.
pub static IDENTIFIER_NAMESPACED: CodeDef<0> = CodeDef::new(
    "core/identifier_namespaced",
    Kind::Rewritten,
    Class::Lint,
    "identifier prefixed with minecraft: by an upgrade step",
)
.was("IdentifierNamespaced");

/// `UpgradeTableMissedBareIdentifier`: an upgrade table missed an identifier because it was not
/// namespaced.
pub static UPGRADE_TABLE_MISSED_BARE_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "core/upgrade_table_missed_bare_identifier",
    Kind::Identifier,
    Class::Error,
    "upgrade table missed an identifier without namespace",
)
.was("UpgradeTableMissedBareIdentifier");

/// `ItemBindingOverlap`: two `Always` item bindings target one item.
pub static ITEM_BINDING_OVERLAP: CodeDef<0> = CodeDef::new(
    "core/item_binding_overlap",
    Kind::Duplicate,
    Class::Error,
    "two Always item bindings target one item",
)
.was("ItemBindingOverlap");

/// `RenderControllerFallback`: a failed higher-priority render controller falls back to the lower
/// definition.
pub static RENDER_CONTROLLER_FALLBACK: CodeDef<0> = CodeDef::new(
    "core/render_controller_fallback",
    Kind::Shadowed,
    Class::Lint,
    "render controller falls back to the lower-priority definition",
)
.was("RenderControllerFallback");

/// `CaseFoldingDisabled`: a render controller file's case folding did not reach some text, which
/// kept its capitals: raised beside the error that text caused, and at a controller key or an
/// array name that kept capitals, which the game registers under a name its lower-cased references
/// never reach. This definition: the argument is the line.
pub static CASE_FOLDING_DISABLED_AT_LINE: CodeDef<1> = CodeDef::new(
    "core/case_folding_disabled_at_line",
    Kind::Lint,
    Class::Lint,
    "an apostrophe at line {} disabled case folding",
)
.was("CaseFoldingDisabled");

/// `CaseFoldingDisabled`: a render controller file's case folding did not reach some text, which
/// kept its capitals: raised beside the error that text caused, and at a controller key or an
/// array name that kept capitals, which the game registers under a name its lower-cased references
/// never reach. This definition: the argument is the line; no reference resolves.
pub static CASE_FOLDING_DISABLED_AT_LINE_UNRESOLVABLE: CodeDef<1> = CodeDef::new(
    "core/case_folding_disabled_at_line_unresolvable",
    Kind::Lint,
    Class::Lint,
    "an apostrophe at line {} disabled case folding; the game resolves no reference to this name",
)
.was("CaseFoldingDisabled");

/// `CaseFoldingDisabled`: a render controller file's case folding did not reach some text, which
/// kept its capitals: raised beside the error that text caused, and at a controller key or an
/// array name that kept capitals, which the game registers under a name its lower-cased references
/// never reach. This definition: without a line.
pub static CASE_FOLDING_DISABLED: CodeDef<0> = CodeDef::new(
    "core/case_folding_disabled",
    Kind::Lint,
    Class::Lint,
    "a capital letter the game's case folding does not reach",
)
.was("CaseFoldingDisabled");

/// `CaseFoldingDisabled`: a render controller file's case folding did not reach some text, which
/// kept its capitals: raised beside the error that text caused, and at a controller key or an
/// array name that kept capitals, which the game registers under a name its lower-cased references
/// never reach. This definition: without a line; no reference resolves.
pub static CASE_FOLDING_DISABLED_UNRESOLVABLE: CodeDef<0> = CodeDef::new(
    "core/case_folding_disabled_unresolvable",
    Kind::Lint,
    Class::Lint,
    "a capital letter the game's case folding does not reach; the game resolves no reference to this name",
)
.was("CaseFoldingDisabled");

/// `PoiBlockClaimedTwice`: one block listed by two POI types; the last load wins.
pub static POI_BLOCK_CLAIMED_TWICE: CodeDef<0> = CodeDef::new(
    "core/poi_block_claimed_twice",
    Kind::Duplicate,
    Class::Error,
    "block listed by two POI types",
)
.was("PoiBlockClaimedTwice");

/// `PoiNoBlocks`: a POI type with an empty `blocks` list.
pub static POI_NO_BLOCKS: CodeDef<0> = CodeDef::new(
    "core/poi_no_blocks",
    Kind::NeverApplies,
    Class::Lint,
    "POI type with no blocks",
)
.was("PoiNoBlocks");

/// `SoundDefinitionsCleared`: a pack's `sound_definitions.json` clears the lower packs'
/// definitions.
pub static SOUND_DEFINITIONS_CLEARED: CodeDef<0> = CodeDef::new(
    "core/sound_definitions_cleared",
    Kind::Overridden,
    Class::Lint,
    "sound_definitions.json clears the lower packs' definitions",
)
.was("SoundDefinitionsCleared");

/// `SoundsJsonTableCleared`: a `sounds.json` table absent from a pack clears the lower packs'
/// entries.
pub static SOUNDS_JSON_TABLE_CLEARED: CodeDef<0> = CodeDef::new(
    "core/sounds_json_table_cleared",
    Kind::Overridden,
    Class::Lint,
    "absent sounds.json table clears the lower packs' entries",
)
.was("SoundsJsonTableCleared");

/// `SoundsClearedByLegacyOverlay`: a legacy overlay clears a definition's sounds.
pub static SOUNDS_CLEARED_BY_LEGACY_OVERLAY: CodeDef<0> = CodeDef::new(
    "core/sounds_cleared_by_legacy_overlay",
    Kind::Overridden,
    Class::Lint,
    "legacy overlay clears the definition's sounds",
)
.was("SoundsClearedByLegacyOverlay");

/// `DistanceResetByLegacyOverlay`: a legacy overlay resets a lower pack's `max_distance`.
pub static DISTANCE_RESET_BY_LEGACY_OVERLAY: CodeDef<0> = CodeDef::new(
    "core/distance_reset_by_legacy_overlay",
    Kind::Overridden,
    Class::Lint,
    "legacy overlay resets max_distance",
)
.was("DistanceResetByLegacyOverlay");

/// `UnusedProperty`: a UI property the control's type never reads.
pub static UNUSED_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/unused_property",
    Kind::NeverApplies,
    Class::Lint,
    "property not read by this control type",
)
.was("UnusedProperty");

/// `NoUiDefinitions`: the combined `ui_defs` list is empty.
pub static NO_UI_DEFINITIONS: CodeDef<0> = CodeDef::new(
    "core/no_ui_definitions",
    Kind::NeverApplies,
    Class::Lint,
    "no UI definition files",
)
.was("NoUiDefinitions");

/// `SettingTypeMismatch`: a saved or admin-set value, or a setting's `type`, that does not match
/// the setting's kind.
pub static SETTING_TYPE_MISMATCH: CodeDef<0> = CodeDef::new(
    "core/setting_type_mismatch",
    Kind::WrongType,
    Class::Error,
    "value does not match the setting's type",
)
.was("SettingTypeMismatch");

/// `SettingUnknownOption`: a dropdown or multiselect value that names no option.
pub static SETTING_UNKNOWN_OPTION: CodeDef<0> = CodeDef::new(
    "core/setting_unknown_option",
    Kind::InvalidValue,
    Class::Error,
    "value names no option of the setting",
)
.was("SettingUnknownOption");

/// `SettingUnknownName`: a saved value for a setting that is not defined.
pub static SETTING_UNKNOWN_NAME: CodeDef<0> = CodeDef::new(
    "core/setting_unknown_name",
    Kind::InvalidValue,
    Class::Error,
    "no setting of this name",
)
.was("SettingUnknownName");

/// `SettingClamped`: a saved slider value clamped to `[min, max]`.
pub static SETTING_CLAMPED: CodeDef<0> = CodeDef::new(
    "core/setting_clamped",
    Kind::Clamped,
    Class::Lint,
    "slider value clamped to its range",
)
.was("SettingClamped");

/// `LegacySettingIgnored`: a legacy manifest setting element other than a label is dropped.
pub static LEGACY_SETTING_IGNORED: CodeDef<0> = CodeDef::new(
    "core/legacy_setting_ignored",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Error,
    "legacy setting element dropped",
)
.was("LegacySettingIgnored");

/// `PathRejected`: a listed entry name that is lossy on the host or breaks the path rules;
/// skipped. This definition: the argument is the entry.
pub static PATH_REJECTED: CodeDef<1> = CodeDef::new(
    "core/path_rejected",
    Kind::Io,
    Class::Error,
    "entry '{}' skipped: path rejected",
)
.was("PathRejected");

/// `FileNotLoaded`: an accessor needs a file that is indexed but not resident.
pub static FILE_NOT_LOADED: CodeDef<0> = CodeDef::new(
    "core/file_not_loaded",
    Kind::Io,
    Class::Error,
    "file indexed but not loaded",
)
.was("FileNotLoaded");

/// `InternalError`: a job panicked; what it was for is rejected, absent or installed empty.
pub static INTERNAL_ERROR: CodeDef<0> = CodeDef::new(
    "core/internal_error",
    Kind::Internal,
    Class::Error,
    "internal error: a loader job panicked",
)
.was("InternalError");

/// `UnsupportedFileFormat`: a file whose format the loader cannot read: an encrypted file without
/// its content key, a binary (`\x7FMCB`) document (vanilla `unsupported_file_format`).
pub static UNSUPPORTED_FILE_FORMAT: CodeDef<0> = CodeDef::new(
    "core/unsupported_file_format",
    Kind::Io,
    Class::Error,
    "unsupported file format",
)
.was("UnsupportedFileFormat");

/// `DuplicateEntry`: a path an archive holds more than once; the last entry is kept
/// (`duplicate_entry`).
pub static DUPLICATE_ENTRY: CodeDef<0> = CodeDef::new(
    "core/duplicate_entry",
    Kind::Duplicate,
    Class::Error,
    "duplicate archive entry; the last one is kept",
)
.was("DuplicateEntry");

/// `MissingDependency`: a pack dependency that is in no registered pack; the dependent pack is
/// hidden (vanilla `missing_dependency`). This definition: the arguments are the pack and the
/// dependency; both ride as `DiagnosticDetail::Packs`.
pub static MISSING_DEPENDENCY: CodeDef<2> = CodeDef::new(
    "core/missing_dependency",
    Kind::UnresolvedReference,
    Class::Error,
    "pack {} depends on {}, which is not available; the pack is hidden",
)
.was("MissingDependency");

/// `DependencyAutoAdded`: a resource pack a behaviour pack depends on was added to the resource
/// stack after the listed ones (`dependency_auto_added`). This definition: the arguments are the
/// dependency, then the pack (the line's order); both ride as `DiagnosticDetail::Packs`.
pub static DEPENDENCY_AUTO_ADDED: CodeDef<2> = CodeDef::new(
    "core/dependency_auto_added",
    Kind::Pack,
    Class::Lint,
    "resource pack {} added to the resource stack as a dependency of {}",
)
.was("DependencyAutoAdded");

/// `PackNotFound`: a pack a world list or the base-game layer names is not registered; the entry
/// is skipped. This definition: the argument is the pack; it rides as `DiagnosticDetail::Packs`.
pub static PACK_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "core/pack_not_found",
    Kind::UnresolvedReference,
    Class::Error,
    "pack {} not found",
)
.was("PackNotFound");

/// `MultipleScriptModules`: a second or later script module in one pack is skipped. This
/// definition: the argument is the plugin.
pub static MULTIPLE_SCRIPT_MODULES: CodeDef<1> = CodeDef::new(
    "core/multiple_script_modules",
    Kind::Duplicate,
    Class::Error,
    "Plugin [{}] - skipped, only one script module allowed per pack.",
)
.game("Plugin [{}] - skipped, only one script module allowed per pack.")
.was("MultipleScriptModules");

/// `ScriptModulePermission` without the game's sentence: a native-module dependency the pack may
/// not use.
pub static SCRIPT_MODULE_PERMISSION: CodeDef<0> = CodeDef::new(
    "core/script_module_permission",
    Kind::ScriptModule,
    Class::Error,
    "native module not permitted",
)
.was("ScriptModulePermission");

/// `ScriptModulePermission` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static SCRIPT_MODULE_PERMISSION_WORDED: CodeDef<1> = CodeDef::new(
    "core/script_module_permission_worded",
    Kind::ScriptModule,
    Class::Error,
    "{}",
)
.game("{}")
.was("ScriptModulePermission");

/// `ScriptModuleVersionUnavailable`: no version in the engine's module catalog satisfies a
/// requested version. This definition: the arguments are the plugin, the module, the requested
/// version, the dependency, the current line pre-rendered (`\nCurrent [tag] version is
/// [version].`, or empty) and the available versions pre-rendered (`\n` before each).
pub static SCRIPT_MODULE_VERSION_UNAVAILABLE: CodeDef<6> = CodeDef::new(
    "core/script_module_version_unavailable",
    Kind::ScriptModule,
    Class::Error,
    "Plugin [{}] - module [{}] requested invalid version [{}] of [{}].{}\nAvailable versions:{}",
)
.game("Plugin [{}] - module [{}] requested invalid version [{}] of [{}].{}\nAvailable versions:{}")
.was("ScriptModuleVersionUnavailable");

/// `ScriptModuleVersionConflict`: the requests for one module are satisfiable separately but not
/// by one version. This definition: the arguments are the plugin, the module and the requests
/// pre-rendered (`\n [version] requested by [by]`, `,` between two).
pub static SCRIPT_MODULE_VERSION_CONFLICT: CodeDef<3> = CodeDef::new(
    "core/script_module_version_conflict",
    Kind::ScriptModule,
    Class::Error,
    "Plugin [{}] - version conflict for module [{}].{}",
)
.game("Plugin [{}] - version conflict for module [{}].{}")
.was("ScriptModuleVersionConflict");

/// `ScriptModulePromoted`: a request served by a higher version because of another request. This
/// definition: the arguments are the plugin, the module, the old and the new version, and who
/// requested it.
pub static SCRIPT_MODULE_PROMOTED: CodeDef<5> = CodeDef::new(
    "core/script_module_promoted",
    Kind::ScriptModule,
    Class::Error,
    "Plugin [{}] - promoted [{}] from [{}] to [{}] requested by [{}].",
)
.game("Plugin [{}] - promoted [{}] from [{}] to [{}] requested by [{}].")
.was("ScriptModulePromoted");

/// `ScriptAlphaDependency`: an `alpha` selector or a `-alpha` version.
pub static SCRIPT_ALPHA_DEPENDENCY: CodeDef<0> = CodeDef::new(
    "core/script_alpha_dependency",
    Kind::ScriptModule,
    Class::Error,
    "alpha script API dependency",
)
.was("ScriptAlphaDependency");

/// `ScriptEntryOutsideRoot`: `entry` lacks the `scripts/` prefix but the main file is found.
pub static SCRIPT_ENTRY_OUTSIDE_ROOT: CodeDef<0> = CodeDef::new(
    "core/script_entry_outside_root",
    Kind::ScriptSource,
    Class::Error,
    "entry lacks the scripts/ prefix",
)
.was("ScriptEntryOutsideRoot");

/// `ScriptSyntaxError` without the game's sentence: invalid UTF-8, import attributes, or a lexer
/// or parser error in a script file.
pub static SCRIPT_SYNTAX_ERROR: CodeDef<0> = CodeDef::new(
    "core/script_syntax_error",
    Kind::ScriptSource,
    Class::Error,
    "syntax error in a script file",
)
.was("ScriptSyntaxError");

/// `ScriptSyntaxError` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static SCRIPT_SYNTAX_ERROR_WORDED: CodeDef<1> = CodeDef::new(
    "core/script_syntax_error_worded",
    Kind::ScriptSource,
    Class::Error,
    "{}",
)
.game("{}")
.was("ScriptSyntaxError");

/// `ScriptImportNotFound`: an import specifier that normalises to no file of the bundle. This
/// definition: the argument is the specifier; the hint rides as `DiagnosticDetail::NotFound`.
pub static SCRIPT_IMPORT_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "core/script_import_not_found",
    Kind::UnresolvedReference,
    Class::Error,
    "ReferenceError: Import [{}] not found.",
)
.game("ReferenceError: Import [{}] not found.")
.was("ScriptImportNotFound");

/// `ScriptImportCrashesEngine`: a specifier whose leading `./` / `../` run reaches its end:
/// vanilla's normaliser terminates the server.
pub static SCRIPT_IMPORT_CRASHES_ENGINE: CodeDef<0> = CodeDef::new(
    "core/script_import_crashes_engine",
    Kind::ScriptSource,
    Class::Error,
    "this import specifier terminates the server",
)
.was("ScriptImportCrashesEngine");

/// `ScriptModuleUnrecognized`: an `@…` import that is not a module known to the context. This
/// definition: the argument is the module.
pub static SCRIPT_MODULE_UNRECOGNIZED: CodeDef<1> = CodeDef::new(
    "core/script_module_unrecognized",
    Kind::ScriptModule,
    Class::Error,
    "Module [{}] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?",
)
.game("Module [{}] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?")
.was("ScriptModuleUnrecognized");

/// `ScriptModuleRestricted`: an import of a restricted native module. This definition: the
/// arguments are the module and the referrer.
pub static SCRIPT_MODULE_RESTRICTED: CodeDef<2> = CodeDef::new(
    "core/script_module_restricted",
    Kind::ScriptModule,
    Class::Error,
    "Import failed, can not load restricted module '{}' from '{}'.",
)
.game("Import failed, can not load restricted module '{}' from '{}'.")
.was("ScriptModuleRestricted");

/// `ScriptImportNotChecked`: `import()` with a non-literal argument.
pub static SCRIPT_IMPORT_NOT_CHECKED: CodeDef<0> = CodeDef::new(
    "core/script_import_not_checked",
    Kind::ScriptSource,
    Class::Error,
    "dynamic import not checked",
)
.was("ScriptImportNotChecked");

/// `ScriptFileNotLoadable`: a `.js` file outside `scripts/`, or `scripts/` in a pack without a
/// script module.
pub static SCRIPT_FILE_NOT_LOADABLE: CodeDef<0> = CodeDef::new(
    "core/script_file_not_loadable",
    Kind::ScriptSource,
    Class::Error,
    "script file can never be loaded",
)
.was("ScriptFileNotLoadable");

/// `ScriptEvalWithoutCapability`: `eval(` / `Function(` tokens in a pack without the `script_eval`
/// capability.
pub static SCRIPT_EVAL_WITHOUT_CAPABILITY: CodeDef<0> = CodeDef::new(
    "core/script_eval_without_capability",
    Kind::ScriptSource,
    Class::Error,
    "eval without the script_eval capability",
)
.was("ScriptEvalWithoutCapability");

/// `ScriptImportMetaEmpty`: an `import.meta.<member>` read; `import.meta` has no members in
/// vanilla.
pub static SCRIPT_IMPORT_META_EMPTY: CodeDef<0> = CodeDef::new(
    "core/script_import_meta_empty",
    Kind::ScriptSource,
    Class::Error,
    "import.meta has no members",
)
.was("ScriptImportMetaEmpty");

/// `ScriptsNotChecked`: import scanning disabled; one per bundle.
pub static SCRIPTS_NOT_CHECKED: CodeDef<0> = CodeDef::new(
    "core/scripts_not_checked",
    Kind::ScriptSource,
    Class::Error,
    "script imports not checked",
)
.was("ScriptsNotChecked");

/// `ScriptSyntaxNotChecked`: script files the entry reaches that the JavaScript parser was kept
/// from, because they could nest more deeply than it has stack for; one per bundle. This
/// definition: the argument is the count of files.
pub static SCRIPT_SYNTAX_NOT_CHECKED: CodeDef<1> = CodeDef::new(
    "core/script_syntax_not_checked",
    Kind::ScriptSource,
    Class::Error,
    "syntax of {} script files not checked: nested too deeply for the parser",
)
.was("ScriptSyntaxNotChecked");

/// `PackConfigShadowsDefault`: a pack's own `config/<uuid>/` folder hides `config/default/` and
/// lacks a file that `default` has.
pub static PACK_CONFIG_SHADOWS_DEFAULT: CodeDef<0> = CodeDef::new(
    "core/pack_config_shadows_default",
    Kind::Shadowed,
    Class::Lint,
    "pack config folder hides config/default",
)
.was("PackConfigShadowsDefault");

/// `UnmatchedConfigFolder`: a folder under `config/` that matches no script-module UUID of the
/// stack. This definition: the argument is the folder.
pub static UNMATCHED_CONFIG_FOLDER: CodeDef<1> = CodeDef::new(
    "core/unmatched_config_folder",
    Kind::NeverApplies,
    Class::Lint,
    "config folder '{}' matches no script module of the stack",
)
.was("UnmatchedConfigFolder");

/// `UnmatchedConfigFolder`: a folder under `config/` that matches no script-module UUID of the
/// stack. This definition: the arguments are the folder and the UUID the module expects; the UUID
/// rides as `DiagnosticDetail::Uuid`.
pub static UNMATCHED_CONFIG_FOLDER_EXPECTED: CodeDef<2> = CodeDef::new(
    "core/unmatched_config_folder_expected",
    Kind::NeverApplies,
    Class::Lint,
    "config folder '{}' is never used; the script module's folder is '{}'",
)
.was("UnmatchedConfigFolder");

/// `NetConfigRejected`: the `@minecraft/server-net` entry of `module_permissions` failed to load.
/// This definition: the arguments are the module and the errors.
pub static NET_CONFIG_REJECTED: CodeDef<2> = CodeDef::new(
    "core/net_config_rejected",
    Kind::Config,
    Class::Error,
    "Failed to load config for {} due to error(s): {}",
)
.game("Failed to load config for {} due to error(s): {}")
.was("NetConfigRejected");

/// `InvalidProperty` without the game's sentence: a manifest property with an invalid value.
pub static INVALID_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/invalid_property",
    Kind::InvalidValue,
    Class::Error,
    "invalid manifest property",
)
.was("InvalidProperty");

/// `InvalidProperty` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static INVALID_PROPERTY_WORDED: CodeDef<1> = CodeDef::new(
    "core/invalid_property_worded",
    Kind::InvalidValue,
    Class::Error,
    "{}",
)
.game("{}")
.was("InvalidProperty");

/// `MalformedPropertyVersion` without the game's sentence: a manifest version property that is
/// malformed.
pub static MALFORMED_PROPERTY_VERSION: CodeDef<0> = CodeDef::new(
    "core/malformed_property_version",
    Kind::InvalidValue,
    Class::Error,
    "malformed manifest version",
)
.was("MalformedPropertyVersion");

/// `MalformedPropertyVersion` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static MALFORMED_PROPERTY_VERSION_WORDED: CodeDef<1> = CodeDef::new(
    "core/malformed_property_version_worded",
    Kind::InvalidValue,
    Class::Error,
    "{}",
)
.game("{}")
.was("MalformedPropertyVersion");

/// `WrongTypeProperty` without the game's sentence: a manifest property of the wrong type.
pub static WRONG_TYPE_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/wrong_type_property",
    Kind::WrongType,
    Class::Error,
    "manifest property of the wrong type",
)
.was("WrongTypeProperty");

/// `WrongTypeProperty` with a sentence of the game's (or the family's) given whole as the
/// argument: a site whose template has no definition of its own yet.
pub static WRONG_TYPE_PROPERTY_WORDED: CodeDef<1> = CodeDef::new(
    "core/wrong_type_property_worded",
    Kind::WrongType,
    Class::Error,
    "{}",
)
.game("{}")
.was("WrongTypeProperty");

/// `MissingProperty` without the game's sentence: a required manifest property is absent.
pub static MISSING_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/missing_property",
    Kind::MissingRequired,
    Class::Error,
    "missing manifest property",
)
.was("MissingProperty");

/// `MissingProperty` with a sentence of the game's (or the family's) given whole as the argument:
/// a site whose template has no definition of its own yet.
pub static MISSING_PROPERTY_WORDED: CodeDef<1> = CodeDef::new(
    "core/missing_property_worded",
    Kind::MissingRequired,
    Class::Error,
    "{}",
)
.game("{}")
.was("MissingProperty");

/// `EmptyProperty`: a manifest property that is empty; the game reports a different kind once the
/// pack is upgraded.
pub static EMPTY_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/empty_property",
    Kind::InvalidValue,
    Class::Error,
    "empty manifest property",
)
.was("EmptyProperty");

/// `MalformedPropertyUuid`: a manifest UUID that is malformed or nil (a different kind once the
/// pack is upgraded); vanilla replaces it by a hash of the text.
pub static MALFORMED_PROPERTY_UUID: CodeDef<0> = CodeDef::new(
    "core/malformed_property_uuid",
    Kind::InvalidValue,
    Class::Error,
    "malformed manifest UUID",
)
.was("MalformedPropertyUuid");

/// `DuplicateUuid`: a module UUID equal to the header UUID, or two modules with one UUID (a
/// different kind once the pack is upgraded).
pub static DUPLICATE_UUID: CodeDef<0> = CodeDef::new(
    "core/duplicate_uuid",
    Kind::Duplicate,
    Class::Error,
    "duplicate UUID in the manifest",
)
.was("DuplicateUuid");

/// `MissingModules`: a deprecated (format 0) manifest without modules.
pub static MISSING_MODULES: CodeDef<0> = CodeDef::new(
    "core/missing_modules",
    Kind::MissingRequired,
    Class::Error,
    "the manifest has no modules",
)
.was("MissingModules");

/// `MultipleModules`: modules that declare different pack types (a different kind once the pack is
/// upgraded).
pub static MULTIPLE_MODULES: CodeDef<0> = CodeDef::new(
    "core/multiple_modules",
    Kind::Duplicate,
    Class::Error,
    "the manifest's modules declare different pack types",
)
.was("MultipleModules");

/// `InvalidCapability`: a manifest capability the host's registry does not grant. This definition:
/// the argument is the capability.
pub static INVALID_CAPABILITY: CodeDef<1> = CodeDef::new(
    "core/invalid_capability",
    Kind::Toggle,
    Class::Error,
    "The pack manifest contains an invalid capability: {}",
)
.game("The pack manifest contains an invalid capability: {}")
.was("InvalidCapability");

/// `IgnoredProperty`: a manifest property read but ignored for this pack type or format.
pub static IGNORED_PROPERTY: CodeDef<0> = CodeDef::new(
    "core/ignored_property",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Error,
    "manifest property ignored",
)
.was("IgnoredProperty");

/// `VersionTooHigh`: a manifest version above the highest accepted one. This definition: the
/// arguments are the version and the limit.
pub static VERSION_TOO_HIGH: CodeDef<2> = CodeDef::new(
    "core/version_too_high",
    Kind::EngineVersion,
    Class::Error,
    "version {} is above {}",
)
.was("VersionTooHigh");

/// `VersionTooLow`: a manifest version below the lowest accepted one. This definition: the
/// arguments are the version and the limit.
pub static VERSION_TOO_LOW: CodeDef<2> = CodeDef::new(
    "core/version_too_low",
    Kind::EngineVersion,
    Class::Error,
    "version {} is below {}",
)
.was("VersionTooLow");

/// `MinEngineVersionCapFormatVersion1`: a format-0/1 manifest's `min_engine_version` of 1.13.0 or
/// above, capped to 1.12.0.
pub static MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1: CodeDef<0> = CodeDef::new(
    "core/min_engine_version_cap_format_version1",
    Kind::Clamped,
    Class::Error,
    "min_engine_version of a format 1 manifest is capped to 1.12.0",
)
.was("MinEngineVersionCapFormatVersion1");

// ---- the game's lines core raises on a family's behalf: the door and the schema tree ----

/// The Minecraft Document reader (`RapidJSON`) refused the text: a number too big for a double,
/// content after the root value, or another of its errors. The argument is the reader's message;
/// the finding carries it again as its `DiagnosticDetail::Syntax`.
pub static DOCUMENT_JSON: CodeDef<1> = CodeDef::new(
    "core/document_json",
    Kind::Syntax,
    Class::Error,
    "the Minecraft Document JSON does not parse: {}",
)
.game("Failed to parse the Minecraft Document JSON: {}")
.was("MalformedInput");

/// The game's older schema tree: a member no name group of its node matches. The argument is the
/// key.
pub static CHILD_NOT_VALID: CodeDef<1> = CodeDef::new(
    "core/child_not_valid",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "member '{}' is not valid here",
)
.game("child '{}' not valid here.")
.was("UnrecognizedField");

/// The game's older schema tree: a required name group without a member. The argument is the name.
pub static REQUIRED_CHILD: CodeDef<1> = CodeDef::new(
    "core/required_child",
    Kind::MissingRequired,
    Class::Error,
    "required member {} is missing",
)
.game("Required child {} not found")
.was("MissingRequiredField");

/// The game's older schema tree: a bool outside its allowed-value set. The argument is the value.
pub static VALUE_NOT_ALLOWED: CodeDef<1> = CodeDef::new(
    "core/value_not_allowed",
    Kind::InvalidValue,
    Class::Error,
    "value '{}' is not allowed",
)
.game("Value '{}' not allowed")
.was("InvalidValue");

/// The game's older schema tree: a string matching none of its node's valid-value regexes. The
/// arguments are the value and the patterns.
pub static NOT_ALLOWED_REGEX: CodeDef<2> = CodeDef::new(
    "core/not_allowed_regex",
    Kind::InvalidValue,
    Class::Error,
    "value '{}' matches none of the patterns {}",
)
.game("Value '{}' not allowed.  Must match at least one of the following regex patterns: {}")
.was("InvalidValue");

/// The game's older schema tree: an enum string outside its node's keys. The arguments are the
/// value and the keys.
pub static NOT_ALLOWED_KEYS: CodeDef<2> = CodeDef::new(
    "core/not_allowed_keys",
    Kind::InvalidValue,
    Class::Error,
    "value '{}' is none of the keys {}",
)
.game("Value '{}' not allowed.  Must match at least one of the following keys: {}")
.was("InvalidValue");

/// The game's older schema tree: a float or an int outside its node's range. The arguments are the
/// value and the bounds, as the game prints them (`%f`).
pub static OUTSIDE_RANGE: CodeDef<3> = CodeDef::new(
    "core/outside_range",
    Kind::InvalidValue,
    Class::Error,
    "value '{}' is outside [{}, {}]",
)
.game("Value '{}' outside valid range [{}, {}]")
.was("InvalidValue");

/// The game's older schema tree: an array with fewer elements than its node's minimum. The
/// arguments are the count and the minimum.
pub static ARRAY_TOO_SMALL: CodeDef<2> = CodeDef::new(
    "core/array_too_small",
    Kind::InvalidValue,
    Class::Error,
    "array of {} elements, fewer than {}",
)
.game("Array too small ({} < {})")
.was("InvalidValue");

/// The game's older schema tree: an array with more elements than its node's maximum. The arguments
/// are the count and the maximum.
pub static ARRAY_TOO_LARGE: CodeDef<2> = CodeDef::new(
    "core/array_too_large",
    Kind::InvalidValue,
    Class::Error,
    "array of {} elements, more than {}",
)
.game("Array too large ({} > {})")
.was("InvalidValue");

/// The game's older schema tree: the JSON reader failed on the file. The argument is the reader's
/// messages; the finding carries them again as its `DiagnosticDetail::Syntax`.
pub static PARSE_ERROR: CodeDef<1> = CodeDef::new(
    "core/parse_error",
    Kind::Syntax,
    Class::Error,
    "parse error: {}",
)
.game("parse error:\n{}")
.was("MalformedInput");

/// A float member read a number beyond its width (`json::Misread`): by it the versioned load knows
/// the numbers to clamp, and its own finding of the clamped number stands in its place. The crate's
/// own sentence, held as the game's as the legacy text field held it, so `vanilla` is unchanged.
pub static BEYOND_FLOAT_MEMBER: CodeDef<0> = CodeDef::new(
    "core/beyond_float_member",
    Kind::InvalidValue,
    Class::Lint,
    "the number is beyond the float that reads it; the member holds an infinity",
)
.game("the number is beyond the float that reads it; the member holds an infinity")
.was("InvalidValue");

// ---- the game's lines core raises on a family's behalf: the versioned load ----

/// An absent `format_version` the versioned loader rejects (`MissingVersion::REJECT`): the version
/// check of the loaders that do not check the version themselves.
pub static NO_VALID_VERSION: CodeDef<0> = CodeDef::new(
    "core/no_valid_version",
    Kind::FormatVersion,
    Class::Error,
    "no valid format_version",
)
.game("no valid version passed by the caller or found in the input")
.was("MissingFormatVersion");

/// An absent `format_version` in the legacy parsers' words (the entity's floor).
pub static NO_FORMAT_VERSION_FIELD: CodeDef<0> = CodeDef::new(
    "core/no_format_version_field",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("no format_version field was given")
.was("MissingFormatVersion");

/// An absent `format_version` a strict bind requires (the camera list): cereal's required-member
/// line.
pub static FORMAT_VERSION_REQUIRED: CodeDef<0> = CodeDef::new(
    "core/format_version_required",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("missing required field")
.was("MissingFormatVersion");

/// An absent `format_version` the crafting catalogue's loader rejects.
pub static VERSION_NOT_PROCESSABLE: CodeDef<0> = CodeDef::new(
    "core/version_not_processable",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("The file cannot be processed without a valid format_version")
.was("MissingFormatVersion");

/// An absent `format_version` the item's document loader rejects.
pub static VERSION_NOT_READABLE: CodeDef<0> = CodeDef::new(
    "core/version_not_readable",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("document's version not found or not readable")
.was("MissingFormatVersion");

/// An absent `format_version` the client entity and attachable loaders read at their floor, with
/// the game's warning (the crate's floor, 1.8.0, in place of the game's 1.0.0).
pub static VERSION_NOT_GIVEN_LEGACY_FIELDS: CodeDef<0> = CodeDef::new(
    "core/version_not_given_legacy_fields",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.")
.was("MissingFormatVersion");

/// A version no struct of the chain covers, in the versioned loader's words. The argument is the
/// version as the input parsed it (a pre-release as written).
pub static NO_PARSER_FOR_VERSION: CodeDef<1> = CodeDef::new(
    "core/no_parser_for_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "no parser for format_version {}",
)
.game("unsupported version: no parser available for version {}")
.was("UnsupportedVersion");

/// A document below the family's floor, or a member below the version that brings it. The arguments
/// are what is gated (`component`, `entity`, …), its name and the version it needs.
pub static USE_FORMAT_VERSION: CodeDef<3> = CodeDef::new(
    "core/use_format_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "{} '{}' needs format_version {} or higher",
)
.game("To use {} '{}', use json format version {} or higher")
.was("UnsupportedVersion");

/// A component behind the legacy creator features, released at a version the document is below. The
/// arguments are what is gated, its name and the version that releases it.
pub static TOGGLE_USE_FORMAT_VERSION: CodeDef<3> = CodeDef::new(
    "core/toggle_use_format_version",
    Kind::Toggle,
    Class::Error,
    "{} '{}' needs format_version {} or higher",
)
.game("To use {} '{}', use json format version {} or higher")
.was("MissingToggles");

/// A member or component behind an experiment the load does not have. The arguments are what is
/// gated, its name, and the toggle as the settings screen shows it.
pub static TURN_ON_TOGGLE: CodeDef<3> = CodeDef::new(
    "core/turn_on_toggle",
    Kind::Toggle,
    Class::Error,
    "{} '{}' needs the '{}' toggle",
)
.game("To use {} '{}', turn on the '{}' toggle in settings")
.was("MissingToggles");

/// A component behind an experiment below the version that releases it. The arguments are what is
/// gated, its name, the releasing version, and the toggle.
pub static TURN_ON_TOGGLE_BEFORE: CodeDef<4> = CodeDef::new(
    "core/turn_on_toggle_before",
    Kind::Toggle,
    Class::Error,
    "{} '{}' needs, before format_version {}, the '{}' toggle",
)
.game("To use {} '{}' in json formats earlier than {}, turn on the '{}' toggle in settings")
.was("MissingToggles");

/// A beta `format_version` without the Beta APIs toggle: the JSON item path's gate.
pub static BETA_DOCUMENTS: CodeDef<0> = CodeDef::new(
    "core/beta_documents",
    Kind::Toggle,
    Class::Error,
    "beta document without the Beta APIs toggle",
)
.game("Turn on the Beta APIs toggle in settings to allow beta documents.")
.was("BetaFeaturesNotEnabled");

/// A beta `format_version` outside a base pack and without the Beta APIs toggle: the check of the
/// versioned cereal loaders.
pub static BETA_FORMAT_VERSION: CodeDef<0> = CodeDef::new(
    "core/beta_format_version",
    Kind::Toggle,
    Class::Error,
    "beta document without the Beta APIs toggle",
)
.game("Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled.")
.was("BetaFeaturesNotEnabled");

/// A counted envelope whose root has other than two members. The member count rides as
/// `DiagnosticDetail::Syntax`.
pub static DOCUMENT_MEMBER_COUNT: CodeDef<0> = CodeDef::new(
    "core/document_member_count",
    Kind::Syntax,
    Class::Error,
    "the root is not \"format_version\" and a payload member",
)
.game("A Minecraft Document must have exactly two members: \"format_version\" and a payload member")
.was("MalformedInput");

/// A document input above the limit, refused. The argument is the limit in bytes; the length rides
/// as `DiagnosticDetail::Syntax`.
pub static DOCUMENT_TOO_LARGE: CodeDef<1> = CodeDef::new(
    "core/document_too_large",
    Kind::Syntax,
    Class::Error,
    "the document is above {} bytes",
)
.game("Minecraft Document JSON exceeds the maximum size of {} bytes")
.was("MalformedInput");

/// A document input near the limit, read. The arguments are the length and the limit in bytes; the
/// length rides as `DiagnosticDetail::Syntax` too.
pub static DOCUMENT_NEARLY_TOO_LARGE: CodeDef<2> = CodeDef::new(
    "core/document_nearly_too_large",
    Kind::Syntax,
    Class::Error,
    "the document is {} bytes long, near its limit of {} bytes",
)
.game("JSON document is approaching the maximum allowed size: {} bytes loaded, {} bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected.")
.was("MalformedInput");

/// A component deprecated at or below the document's version. The arguments are the component and
/// the document's version.
pub static DEPRECATED_IN_FORMAT: CodeDef<2> = CodeDef::new(
    "core/deprecated_in_format",
    Kind::Deprecated,
    Class::Error,
    "{} is deprecated at format_version {}",
)
.game("{} has been deprecated and is not available in json format {}")
.was("DeprecatedMember");

// ---- the game's lines core raises on a family's behalf: the merged sound files ----

/// The top copy of a merged sound file (`sounds.json`, `sound_definitions.json`) that is no JSON:
/// the game reads none of the stack. The argument is the file's path; the reader's message rides as
/// `DiagnosticDetail::Syntax`.
pub static STACK_TOP_UNPARSABLE: CodeDef<1> = CodeDef::new(
    "core/stack_top_unparsable",
    Kind::Syntax,
    Class::Error,
    "cannot parse {}, the top of the stack",
)
.game("Could not parse file at the top of the stack: {}.")
.was("MalformedInput");

// ---- the game's lines core raises on a family's behalf: the shared readers ----

/// A filter test the game does not know: logged at level 3, and the test evaluates false (a kept
/// error). The argument is the test's name as written.
pub static FILTER_TEST_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "core/filter_test_not_found",
    Kind::UnresolvedReference,
    Class::Error,
    "no filter test '{}'",
)
.game("Filter test not found -> {}")
.was("UnresolvedReference");

// ---- the game's generic validation lines: cereal's schemas and constraints, the envelope, the older schema tree ----

/// An unknown member of an object, in cereal's words: an error under the strict log mask only, so
/// graded as an unknown member.
pub static MEMBER_NOT_IN_SCHEMA: CodeDef<0> = CodeDef::new(
    "core/member_not_in_schema",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "unknown member",
)
.game("this member was found in the input, but is not present in the Schema")
.was("UnrecognizedField");

/// An unknown component, in cereal's words.
pub static COMPONENT_NOT_IN_SCHEMA: CodeDef<0> = CodeDef::new(
    "core/component_not_in_schema",
    Kind::UnknownMember(Disposition::Kept),
    Class::Error,
    "unknown component",
)
.game("this component was found in the input, but is not present in the Schema")
.was("UnrecognizedComponent");

/// An object lacks a member its schema requires, in cereal's words.
pub static REQUIRED_FIELD_MISSING: CodeDef<0> = CodeDef::new(
    "core/required_field_missing",
    Kind::MissingRequired,
    Class::Error,
    "a required member is missing",
)
.game("missing required field")
.was("MissingRequiredField");

/// Something else stands where an object is required.
pub static EXPECTED_OBJECT: CodeDef<0> = CodeDef::new(
    "core/expected_object",
    Kind::WrongType,
    Class::Error,
    "not an object",
)
.game("expected an object")
.was("InvalidType");

/// A component storage holds fewer components than its schema demands. The argument is the minimum.
pub static TOO_FEW_COMPONENTS: CodeDef<1> = CodeDef::new(
    "core/too_few_components",
    Kind::InvalidValue,
    Class::Error,
    "fewer than {} components",
)
.game("too few input components, expected at least {}")
.was("InvalidValue");

/// The line the game logs after `ENUM_EXPECTS` or `ENUM_EXPECTS_STRING`.
pub static INVALID_ENUM_LINE: CodeDef<0> = CodeDef::new(
    "core/invalid_enum_line",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.game("invalid enum value")
.was("InvalidEnumValue");

/// An enum given a string outside its set. The arguments are the enum's cereal name, the list of
/// names as the game writes it (`cereal::enum_expects` builds it) and the string as written.
pub static ENUM_EXPECTS: CodeDef<3> = CodeDef::new(
    "core/enum_expects",
    Kind::InvalidEnumValue,
    Class::Error,
    "'{}' takes one of {}, not '{}'",
)
.game("This value is bound to '{}' and expects one of the following strings: {}. Found: '{}'")
.was("InvalidEnumValue");

/// An enum given something other than a string. The argument is the enum's cereal name.
pub static ENUM_EXPECTS_STRING: CodeDef<1> = CodeDef::new(
    "core/enum_expects_string",
    Kind::WrongType,
    Class::Error,
    "'{}' expects a string",
)
.game("This value is bound to '{}' and expects a \"named value\" (i.e. a string)")
.was("InvalidType");

/// No number (or boolean) where one is expected.
pub static INVALID_NUMERIC_VALUE: CodeDef<0> = CodeDef::new(
    "core/invalid_numeric_value",
    Kind::WrongType,
    Class::Error,
    "not a number",
)
.game("invalid numeric value")
.was("InvalidType");

/// A number outside a constraint with both bounds. The argument is the range as the game writes it:
/// `[` or `(`, the bounds as `cereal::number` writes them, `]` or `)`.
pub static OUTSIDE_ACCEPTED_RANGE: CodeDef<1> = CodeDef::new(
    "core/outside_accepted_range",
    Kind::InvalidValue,
    Class::Error,
    "outside the range {}",
)
.game("the input is outside of the accepted range {}")
.was("InvalidValue");

/// A number outside a constraint with one bound. The argument is the comparison and the bound (`>=
/// 0`, `< 16`), the bound as `cereal::number` writes it.
pub static NUMBER_BOUND: CodeDef<1> = CodeDef::new(
    "core/number_bound",
    Kind::InvalidValue,
    Class::Error,
    "the number must be {}",
)
.game("the input must be a number {}")
.was("InvalidValue");

/// A number breaks a multiple-of constraint. The arguments are the number and the step, as
/// `cereal::number` writes them.
pub static NOT_MULTIPLE_OF: CodeDef<2> = CodeDef::new(
    "core/not_multiple_of",
    Kind::InvalidValue,
    Class::Error,
    "{} is not a multiple of {}",
)
.game("{} is not multiple of {}")
.was("InvalidValue");

/// A string breaks a pattern constraint. The argument is the regex.
pub static STRING_MISMATCH: CodeDef<1> = CodeDef::new(
    "core/string_mismatch",
    Kind::InvalidValue,
    Class::Error,
    "the string does not match \"{}\"",
)
.game("expected string to match the \"{}\" regex")
.was("InvalidValue");

/// An array under a minimum-size constraint; the game says `elements` whatever the number. The
/// argument is the minimum.
pub static ARRAY_AT_LEAST: CodeDef<1> = CodeDef::new(
    "core/array_at_least",
    Kind::InvalidValue,
    Class::Error,
    "fewer than {} elements",
)
.game("expected array with at least {} elements")
.was("InvalidValue");

/// A map or object under a minimum-size constraint. The argument is the minimum.
pub static OBJECT_AT_LEAST: CodeDef<1> = CodeDef::new(
    "core/object_at_least",
    Kind::InvalidValue,
    Class::Error,
    "fewer than {} members",
)
.game("expected object with at least {} elements")
.was("InvalidValue");

/// An array with a unique-items constraint holds a repeat.
pub static NO_DUPLICATES: CodeDef<0> = CodeDef::new(
    "core/no_duplicates",
    Kind::InvalidValue,
    Class::Error,
    "duplicate element",
)
.game("No duplicates allowed")
.was("InvalidValue");

/// A component storage holds a key its constraint forbids. The argument is the key.
pub static KEY_NOT_ALLOWED: CodeDef<1> = CodeDef::new(
    "core/key_not_allowed",
    Kind::InvalidValue,
    Class::Error,
    "key {} is not allowed",
)
.game("key {} is not allowed")
.was("InvalidValue");

/// The size cap of a vector or a sequence container. The argument is the maximum.
pub static TOO_MANY_ELEMENTS: CodeDef<1> = CodeDef::new(
    "core/too_many_elements",
    Kind::InvalidValue,
    Class::Error,
    "more than {} elements",
)
.game("too many input elements, expected no more than {}")
.was("InvalidValue");

/// The size cap of a string. The argument is the maximum.
pub static TOO_MANY_CHARACTERS: CodeDef<1> = CodeDef::new(
    "core/too_many_characters",
    Kind::InvalidValue,
    Class::Error,
    "more than {} characters",
)
.game("too many characters, expected no more than {}")
.was("InvalidValue");

/// The size cap of a map.
pub static TOO_MANY_MAP_ELEMENTS: CodeDef<0> = CodeDef::new(
    "core/too_many_map_elements",
    Kind::InvalidValue,
    Class::Error,
    "too many members",
)
.game("too many input elements for an associative container")
.was("InvalidValue");

/// A Minecraft Document without its payload member.
pub static PAYLOAD_MISSING: CodeDef<0> = CodeDef::new(
    "core/payload_missing",
    Kind::Envelope,
    Class::Error,
    "no payload",
)
.game("The payload of the Minecraft Document is missing")
.was("MissingPayload");

/// A payload under another key; the crate files it as the missing payload it is to the family. The
/// arguments are the expected key and the key as written.
pub static PAYLOAD_UNDER_OTHER_KEY: CodeDef<2> = CodeDef::new(
    "core/payload_under_other_key",
    Kind::Envelope,
    Class::Error,
    "payload expected under '{}', found under '{}'",
)
.game("The payload key does not match the expected one. Expected: '{}'. Actual: '{}'.")
.was("MissingPayload");

/// A node of the older schema tree with too few children, logged at level 3. The arguments are the
/// count and the minimum.
pub static TOO_FEW_CHILDREN: CodeDef<2> = CodeDef::new(
    "core/too_few_children",
    Kind::InvalidValue,
    Class::Error,
    "{} children, fewer than {}",
)
.game("Node has too few children ({} < {})")
.was("InvalidValue");

/// A node of the older schema tree with too many children, logged at level 3. The arguments are the
/// count and the maximum.
pub static TOO_MANY_CHILDREN: CodeDef<2> = CodeDef::new(
    "core/too_many_children",
    Kind::InvalidValue,
    Class::Error,
    "{} children, more than {}",
)
.game("Node has too many children ({} > {})")
.was("InvalidValue");

/// A child of the older schema tree of none of the allowed types, logged at level 3; the text has
/// two spaces before `Allowed`. The argument is the allowed types.
pub static UNKNOWN_CHILD_OPTION: CodeDef<1> = CodeDef::new(
    "core/unknown_child_option",
    Kind::WrongType,
    Class::Error,
    "no child type of {}",
)
.game("unknown child schema option type.  Allowed types: {}")
.was("InvalidType");

/// A version below a node's minimum. The arguments are the version and the minimum.
pub static VERSION_BELOW_MIN: CodeDef<2> = CodeDef::new(
    "core/version_below_min",
    Kind::UnsupportedVersion,
    Class::Error,
    "version {} is below {}",
)
.game("Version {} below min required version ({})")
.was("UnsupportedVersion");

// ---- the game's lines core raises on a family's behalf: deprecated members and enum values ----

/// A member its type marks deprecated: cereal skips it without reading the value, and the code is
/// in both error masks, so the document is rejected. The argument is the member's name.
pub static MEMBER_DEPRECATED: CodeDef<1> = CodeDef::new(
    "core/member_deprecated",
    Kind::Deprecated,
    Class::Error,
    "member '{}' is deprecated",
)
.game("member \"{}\" is deprecated")
.was("DeprecatedMember");

/// An enum name its map marks deprecated; in both error masks, so the document is rejected.
pub static ENUM_VALUE_DEPRECATED: CodeDef<0> = CodeDef::new(
    "core/enum_value_deprecated",
    Kind::Deprecated,
    Class::Error,
    "enum value is deprecated",
)
.game("enum value is deprecated")
.was("DeprecatedMember");

/// Every definition of this module, in the order of the legacy variants.
pub static ALL: &[&CodeMeta] = &[
    UNSUPPORTED_VERSION.meta(),
    UNSUPPORTED_VERSION_WORDED.meta(),
    BETA_FEATURES_NOT_ENABLED.meta(),
    BETA_FEATURES_NOT_ENABLED_WORDED.meta(),
    PRERELEASE_FORMAT_VERSION.meta(),
    MISSING_TOGGLES.meta(),
    MISSING_TOGGLES_WORDED.meta(),
    DEPRECATED_MEMBER.meta(),
    DEPRECATED_MEMBER_WORDED.meta(),
    UNRECOGNIZED_FIELD.meta(),
    UNRECOGNIZED_FIELD_WORDED.meta(),
    UNRECOGNIZED_COMPONENT.meta(),
    UNRECOGNIZED_COMPONENT_WORDED.meta(),
    INVALID_VALUE.meta(),
    INVALID_VALUE_WORDED.meta(),
    MISSING_REQUIRED_FIELD.meta(),
    MISSING_REQUIRED_FIELD_WORDED.meta(),
    CLAMPED_VALUE.meta(),
    CLAMPED_VALUE_WORDED.meta(),
    DROPPED_COMPONENT.meta(),
    DROPPED_COMPONENT_WORDED.meta(),
    DROPPED_FIELD.meta(),
    DROPPED_FIELD_WORDED.meta(),
    MEMBER_NOT_KEPT.meta(),
    MEMBER_NOT_KEPT_WORDED.meta(),
    VALUE_REWRITTEN.meta(),
    VALUE_REWRITTEN_WORDED.meta(),
    DUPLICATE_COMPONENT_VERSION.meta(),
    UNKNOWN_SOUND.meta(),
    UNKNOWN_SOUND_WORDED.meta(),
    GENERIC_GOAL_SUFFIX_MISSING.meta(),
    MALFORMED_INPUT.meta(),
    SYNTAX.meta(),
    MALFORMED_INPUT_WORDED.meta(),
    FORMAT_VERSION_ABOVE_ENGINE.meta(),
    CLIENT_ENTITY_SHADOWED.meta(),
    CLIENT_ENTITY_SHADOWED_OLDER.meta(),
    CLIENT_ENTITY_SHADOWED_ABOVE_STACK.meta(),
    ATTACHABLE_SHADOWED.meta(),
    ATTACHABLE_SHADOWED_OLDER.meta(),
    ATTACHABLE_SHADOWED_ABOVE_STACK.meta(),
    ITEM_BINDING_SHADOWED.meta(),
    RENDER_CONTROLLER_SHADOWED.meta(),
    IDENTIFIER_COLLISION.meta(),
    IDENTIFIER_COLLISION_WORDED.meta(),
    UNRESOLVED_REFERENCE.meta(),
    UNRESOLVED_REFERENCE_WORDED.meta(),
    INVALID_REFERENCE.meta(),
    INVALID_REFERENCE_WORDED.meta(),
    TEXTURE_NOT_FOUND.meta(),
    TEXTURE_NOT_FOUND_WORDED.meta(),
    SPLASHES_REPLACED.meta(),
    NO_SPLASHES.meta(),
    UNRESOLVED_LANG_KEY.meta(),
    IGNORED_FILE.meta(),
    IGNORED_FILE_WORDED.meta(),
    PACK_ICON_MISSING.meta(),
    PACK_ICON_SHAPE.meta(),
    GLYPH_SHEET_SHAPE.meta(),
    SKIN_PACK_NAME_MISMATCH.meta(),
    SKIN_PACK_CREATOR_RULE.meta(),
    IGNORED_MEMBER.meta(),
    IGNORED_MEMBER_WORDED.meta(),
    UNMATCHED_CONDITION.meta(),
    WHERE_ARRAY_NEVER_MATCHES.meta(),
    REBASED_OVERRIDE.meta(),
    SHADOWED_DEFINITION.meta(),
    SHADOWED_DEFINITION_WORDED.meta(),
    READ_ERROR.meta(),
    READ_ERROR_WORDED.meta(),
    MISSING_PAYLOAD.meta(),
    MISSING_PAYLOAD_WORDED.meta(),
    NAMESPACE_RULE.meta(),
    NAMESPACE_RULE_WORDED.meta(),
    INVALID_TYPE.meta(),
    INVALID_TYPE_WORDED.meta(),
    SKIPPED_CACHE_DOCUMENT.meta(),
    MISSING_FORMAT_VERSION.meta(),
    MISSING_FORMAT_VERSION_WORDED.meta(),
    MALFORMED_FORMAT_VERSION.meta(),
    MALFORMED_FORMAT_VERSION_WORDED.meta(),
    EMPTY_DOCUMENT.meta(),
    EMPTY_DOCUMENT_WORDED.meta(),
    UNUSUAL_FORMAT_VERSION.meta(),
    VANILLA_REJECTS_VERSION.meta(),
    FORMAT_VERSION_ARRAY.meta(),
    PAYLOAD_KEY_MISMATCH.meta(),
    UNKNOWN_PAYLOAD_KEY.meta(),
    IGNORED_PAYLOAD.meta(),
    DOCUMENT_REJECTED.meta(),
    DOCUMENT_REJECTED_WORDED.meta(),
    MISSING_IDENTIFIER.meta(),
    MISSING_IDENTIFIER_WORDED.meta(),
    SOUND_DEFINITIONS_WRAPPER_WITHOUT_VERSION.meta(),
    MISSING_MIN_ENGINE_VERSION.meta(),
    FIELD_SINCE_ENGINE.meta(),
    FIELD_UNTIL_ENGINE.meta(),
    UNSUPPORTED_VALUE_FORM.meta(),
    INVALID_ENUM_VALUE.meta(),
    INVALID_ENUM_VALUE_WORDED.meta(),
    INVALID_MOLANG.meta(),
    MOLANG_LINT.meta(),
    MOLANG_NOT_CHECKED.meta(),
    KEYFRAME_OUT_OF_DOMAIN.meta(),
    LOSSY_CONVERSION.meta(),
    LITERAL_IGNORED.meta(),
    NON_CANONICAL_MEMBER_CASE.meta(),
    CASE_FOLDED_TRANSITION.meta(),
    UNORDERED_REMAP_CURVE.meta(),
    IMPLICIT_INITIAL_STATE.meta(),
    UNRECOGNIZED_EVENT_STRING.meta(),
    INVALID_NODE.meta(),
    INVALID_ROOT.meta(),
    UNKNOWN_NODE.meta(),
    LEGACY_ITEM_FORMAT.meta(),
    PERSONA_PIECE_PACK.meta(),
    LANG_KEY_TRAILING_SPACE.meta(),
    SCATTER_PARAM_VALUE.meta(),
    SCATTER_PARAM_VALUE_WORDED.meta(),
    UNSUPPORTED_OPERATOR.meta(),
    INVALID_TEXTURE.meta(),
    INVALID_TEXTURE_WORDED.meta(),
    LAYER_IMAGE_UNDECODABLE.meta(),
    LAYER_IMAGE_FORMAT.meta(),
    CAPABILITY_HINT.meta(),
    CAPABILITY_HINT_WORDED.meta(),
    VIBRANT_VISUALS_DISABLED_BY_PACK.meta(),
    FONT_FILE_MISSING.meta(),
    FONT_FILE_MISSING_WORDED.meta(),
    FONT_NAME_OVERRIDDEN.meta(),
    MIXED_FONT_PAGE.meta(),
    ATOMIC_FILTER_LIGHTING_IGNORED.meta(),
    ATOMIC_UNSUPPORTED_RENDER_CONTROLLER.meta(),
    DUPLICATE_IDENTIFIER.meta(),
    DUPLICATE_IDENTIFIER_WORDED.meta(),
    DUPLICATE_BONE.meta(),
    ATOMIC_CLIENT_ENTITY_SHADOWED.meta(),
    ANIMATION_CONTROLLER_SHADOWED.meta(),
    SHADOWED_BY_BUILTIN.meta(),
    REFERENCE_CYCLE.meta(),
    REFERENCE_CYCLE_WORDED.meta(),
    CLIENT_BIOME_SHADOWS_LOWER_PACKS.meta(),
    CLIENT_BIOME_OVERRIDDEN_BY_BIOMES_CLIENT.meta(),
    VANILLA_BIOMES_CLIENT_SKIPPED.meta(),
    LEGACY_FOG_IGNORED.meta(),
    BIOME_FORMAT_HINT_LEGACY.meta(),
    BIOME_FORMAT_HINT_EXPERIMENTAL.meta(),
    BIOME_FORMAT_HINT_EXPERIMENTAL_SURFACE.meta(),
    NO_EFFECT.meta(),
    NO_EFFECT_WORDED.meta(),
    SYNTHESIZED_COMPONENT.meta(),
    IDENTIFIER_NAMESPACED.meta(),
    UPGRADE_TABLE_MISSED_BARE_IDENTIFIER.meta(),
    ITEM_BINDING_OVERLAP.meta(),
    RENDER_CONTROLLER_FALLBACK.meta(),
    CASE_FOLDING_DISABLED_AT_LINE.meta(),
    CASE_FOLDING_DISABLED_AT_LINE_UNRESOLVABLE.meta(),
    CASE_FOLDING_DISABLED.meta(),
    CASE_FOLDING_DISABLED_UNRESOLVABLE.meta(),
    POI_BLOCK_CLAIMED_TWICE.meta(),
    POI_NO_BLOCKS.meta(),
    SOUND_DEFINITIONS_CLEARED.meta(),
    SOUNDS_JSON_TABLE_CLEARED.meta(),
    SOUNDS_CLEARED_BY_LEGACY_OVERLAY.meta(),
    DISTANCE_RESET_BY_LEGACY_OVERLAY.meta(),
    UNUSED_PROPERTY.meta(),
    NO_UI_DEFINITIONS.meta(),
    SETTING_TYPE_MISMATCH.meta(),
    SETTING_UNKNOWN_OPTION.meta(),
    SETTING_UNKNOWN_NAME.meta(),
    SETTING_CLAMPED.meta(),
    LEGACY_SETTING_IGNORED.meta(),
    PATH_REJECTED.meta(),
    FILE_NOT_LOADED.meta(),
    INTERNAL_ERROR.meta(),
    UNSUPPORTED_FILE_FORMAT.meta(),
    DUPLICATE_ENTRY.meta(),
    MISSING_DEPENDENCY.meta(),
    DEPENDENCY_AUTO_ADDED.meta(),
    PACK_NOT_FOUND.meta(),
    MULTIPLE_SCRIPT_MODULES.meta(),
    SCRIPT_MODULE_PERMISSION.meta(),
    SCRIPT_MODULE_PERMISSION_WORDED.meta(),
    SCRIPT_MODULE_VERSION_UNAVAILABLE.meta(),
    SCRIPT_MODULE_VERSION_CONFLICT.meta(),
    SCRIPT_MODULE_PROMOTED.meta(),
    SCRIPT_ALPHA_DEPENDENCY.meta(),
    SCRIPT_ENTRY_OUTSIDE_ROOT.meta(),
    SCRIPT_SYNTAX_ERROR.meta(),
    SCRIPT_SYNTAX_ERROR_WORDED.meta(),
    SCRIPT_IMPORT_NOT_FOUND.meta(),
    SCRIPT_IMPORT_CRASHES_ENGINE.meta(),
    SCRIPT_MODULE_UNRECOGNIZED.meta(),
    SCRIPT_MODULE_RESTRICTED.meta(),
    SCRIPT_IMPORT_NOT_CHECKED.meta(),
    SCRIPT_FILE_NOT_LOADABLE.meta(),
    SCRIPT_EVAL_WITHOUT_CAPABILITY.meta(),
    SCRIPT_IMPORT_META_EMPTY.meta(),
    SCRIPTS_NOT_CHECKED.meta(),
    SCRIPT_SYNTAX_NOT_CHECKED.meta(),
    PACK_CONFIG_SHADOWS_DEFAULT.meta(),
    UNMATCHED_CONFIG_FOLDER.meta(),
    UNMATCHED_CONFIG_FOLDER_EXPECTED.meta(),
    NET_CONFIG_REJECTED.meta(),
    INVALID_PROPERTY.meta(),
    INVALID_PROPERTY_WORDED.meta(),
    MALFORMED_PROPERTY_VERSION.meta(),
    MALFORMED_PROPERTY_VERSION_WORDED.meta(),
    WRONG_TYPE_PROPERTY.meta(),
    WRONG_TYPE_PROPERTY_WORDED.meta(),
    MISSING_PROPERTY.meta(),
    MISSING_PROPERTY_WORDED.meta(),
    EMPTY_PROPERTY.meta(),
    MALFORMED_PROPERTY_UUID.meta(),
    DUPLICATE_UUID.meta(),
    MISSING_MODULES.meta(),
    MULTIPLE_MODULES.meta(),
    INVALID_CAPABILITY.meta(),
    IGNORED_PROPERTY.meta(),
    VERSION_TOO_HIGH.meta(),
    VERSION_TOO_LOW.meta(),
    MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1.meta(),
    DOCUMENT_JSON.meta(),
    CHILD_NOT_VALID.meta(),
    REQUIRED_CHILD.meta(),
    VALUE_NOT_ALLOWED.meta(),
    NOT_ALLOWED_REGEX.meta(),
    NOT_ALLOWED_KEYS.meta(),
    OUTSIDE_RANGE.meta(),
    ARRAY_TOO_SMALL.meta(),
    ARRAY_TOO_LARGE.meta(),
    PARSE_ERROR.meta(),
    BEYOND_FLOAT_MEMBER.meta(),
    NO_VALID_VERSION.meta(),
    NO_FORMAT_VERSION_FIELD.meta(),
    FORMAT_VERSION_REQUIRED.meta(),
    VERSION_NOT_PROCESSABLE.meta(),
    VERSION_NOT_READABLE.meta(),
    VERSION_NOT_GIVEN_LEGACY_FIELDS.meta(),
    NO_PARSER_FOR_VERSION.meta(),
    USE_FORMAT_VERSION.meta(),
    TOGGLE_USE_FORMAT_VERSION.meta(),
    TURN_ON_TOGGLE.meta(),
    TURN_ON_TOGGLE_BEFORE.meta(),
    BETA_DOCUMENTS.meta(),
    BETA_FORMAT_VERSION.meta(),
    DOCUMENT_MEMBER_COUNT.meta(),
    DOCUMENT_TOO_LARGE.meta(),
    DOCUMENT_NEARLY_TOO_LARGE.meta(),
    DEPRECATED_IN_FORMAT.meta(),
    STACK_TOP_UNPARSABLE.meta(),
    FILTER_TEST_NOT_FOUND.meta(),
    MEMBER_NOT_IN_SCHEMA.meta(),
    COMPONENT_NOT_IN_SCHEMA.meta(),
    REQUIRED_FIELD_MISSING.meta(),
    EXPECTED_OBJECT.meta(),
    TOO_FEW_COMPONENTS.meta(),
    INVALID_ENUM_LINE.meta(),
    ENUM_EXPECTS.meta(),
    ENUM_EXPECTS_STRING.meta(),
    INVALID_NUMERIC_VALUE.meta(),
    OUTSIDE_ACCEPTED_RANGE.meta(),
    NUMBER_BOUND.meta(),
    NOT_MULTIPLE_OF.meta(),
    STRING_MISMATCH.meta(),
    ARRAY_AT_LEAST.meta(),
    OBJECT_AT_LEAST.meta(),
    NO_DUPLICATES.meta(),
    KEY_NOT_ALLOWED.meta(),
    TOO_MANY_ELEMENTS.meta(),
    TOO_MANY_CHARACTERS.meta(),
    TOO_MANY_MAP_ELEMENTS.meta(),
    PAYLOAD_MISSING.meta(),
    PAYLOAD_UNDER_OTHER_KEY.meta(),
    TOO_FEW_CHILDREN.meta(),
    TOO_MANY_CHILDREN.meta(),
    UNKNOWN_CHILD_OPTION.meta(),
    VERSION_BELOW_MIN.meta(),
    MEMBER_DEPRECATED.meta(),
    ENUM_VALUE_DEPRECATED.meta(),
];
