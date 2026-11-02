#![allow(clippy::pedantic, missing_docs)]
//! The code vocabulary: every code the crate lists is a `DiagnosticCode` variant, and no name from
//! the rejected list is one.

use bedrock_addon_core::diagnostic::DiagnosticCode;

/// Every code: the document-level list, the RP-side and shared codes, every family's codes and
/// the manifest variants.
const LISTED_CODES: &[&str] = &[
    // document level
    "UnsupportedVersion",
    "BetaFeaturesNotEnabled",
    "PrereleaseFormatVersion",
    "MissingToggles",
    "DeprecatedMember",
    "UnrecognizedField",
    "UnrecognizedComponent",
    "InvalidValue",
    "MissingRequiredField",
    "ClampedValue",
    "DroppedComponent",
    "DroppedField",
    "MemberNotKept",
    "ValueRewritten",
    "DuplicateComponentVersion",
    "UnknownSound",
    "GenericGoalSuffixMissing",
    "MalformedInput",
    // RP side, lints, UI, shared
    "FormatVersionAboveEngine",
    "ClientEntityShadowed",
    "AttachableShadowed",
    "ItemBindingShadowed",
    "RenderControllerShadowed",
    "IdentifierCollision",
    "UnresolvedReference",
    "InvalidReference",
    "TextureNotFound",
    "UnresolvedLangKey",
    "SplashesReplaced",
    "NoSplashes",
    "IgnoredFile",
    "PackIconMissing",
    "PackIconShape",
    "GlyphSheetShape",
    "SkinPackNameMismatch",
    "SkinPackCreatorRule",
    "IgnoredMember",
    "UnmatchedCondition",
    "WhereArrayNeverMatches",
    "RebasedOverride",
    "ShadowedDefinition",
    "ReadError",
    "MissingPayload",
    "NoEffect",
    "NamespaceRule",
    "InvalidType",
    "SkippedCacheDocument",
    // version and envelope
    "MissingFormatVersion",
    "MalformedFormatVersion",
    "EmptyDocument",
    "UnusualFormatVersion",
    "VanillaRejectsVersion",
    "FormatVersionArray",
    "PayloadKeyMismatch",
    "UnknownPayloadKey",
    "IgnoredPayload",
    "DocumentRejected",
    "MissingIdentifier",
    "SoundDefinitionsWrapperWithoutVersion",
    "MissingMinEngineVersion",
    "FieldSinceEngine",
    "FieldUntilEngine",
    "UnsupportedValueForm",
    // values, types and expressions
    "InvalidEnumValue",
    "InvalidMolang",
    "MolangLint",
    "MolangNotChecked",
    "KeyframeOutOfDomain",
    "LossyConversion",
    "LiteralIgnored",
    "NonCanonicalMemberCase",
    "CaseFoldedTransition",
    "UnorderedRemapCurve",
    "ImplicitInitialState",
    "UnrecognizedEventString",
    "InvalidNode",
    "InvalidRoot",
    "UnknownNode",
    "LegacyItemFormat",
    "PersonaPiecePack",
    "LangKeyTrailingSpace",
    "ScatterParamValue",
    "UnsupportedOperator",
    // assets
    "InvalidTexture",
    "LayerImageUndecodable",
    "LayerImageFormat",
    "CapabilityHint",
    "VibrantVisualsDisabledByPack",
    "UnsupportedAudioFormat",
    "FontFileMissing",
    "FontNameOverridden",
    "MixedFontPage",
    "AtomicFilterLightingIgnored",
    "AtomicUnsupportedRenderController",
    // identity, merge and linking
    "DuplicateIdentifier",
    "DuplicateBone",
    "AtomicClientEntityShadowed",
    "AnimationControllerShadowed",
    "ShadowedByBuiltin",
    "ReferenceCycle",
    "ClientBiomeShadowsLowerPacks",
    "ClientBiomeOverriddenByBiomesClient",
    "VanillaBiomesClientSkipped",
    "LegacyFogIgnored",
    "BiomeFormatHint",
    "CaseFoldingDisabled",
    "SynthesizedComponent",
    "IdentifierNamespaced",
    "UpgradeTableMissedBareIdentifier",
    "ItemBindingOverlap",
    "RenderControllerFallback",
    "PoiBlockClaimedTwice",
    "PoiNoBlocks",
    "SoundDefinitionsCleared",
    "SoundsJsonTableCleared",
    "SoundsClearedByLegacyOverlay",
    "DistanceResetByLegacyOverlay",
    "UnusedProperty",
    "NoUiDefinitions",
    // pack settings
    "SettingTypeMismatch",
    "SettingUnknownOption",
    "SettingUnknownName",
    "SettingClamped",
    "LegacySettingIgnored",
    "PackSettingChangeIgnored",
    // loading
    "PathRejected",
    "FileNotLoaded",
    "InternalError",
    // storage and the stack
    "UnsupportedFileFormat",
    "DuplicateEntry",
    "MissingDependency",
    "DependencyAutoAdded",
    "PackNotFound",
    // scripting
    "MultipleScriptModules",
    "ScriptModulePermission",
    "ScriptModuleVersionUnavailable",
    "ScriptModuleVersionConflict",
    "ScriptModulePromoted",
    "ScriptAlphaDependency",
    "ScriptEntryOutsideRoot",
    "ScriptSyntaxError",
    "ScriptImportNotFound",
    "ScriptImportCrashesEngine",
    "ScriptModuleUnrecognized",
    "ScriptModuleRestricted",
    "ScriptImportNotChecked",
    "ScriptFileNotLoadable",
    "ScriptEvalWithoutCapability",
    "ScriptImportMetaEmpty",
    "ScriptsNotChecked",
    "ScriptSyntaxNotChecked",
    "PackConfigShadowsDefault",
    "UnmatchedConfigFolder",
    "NetConfigRejected",
    // the manifest variants
    "InvalidProperty",
    "MalformedPropertyVersion",
    "WrongTypeProperty",
    "MissingProperty",
    // the other manifest variants
    "EmptyProperty",
    "MalformedPropertyUuid",
    "DuplicateUuid",
    "MissingModules",
    "MultipleModules",
    "InvalidCapability",
    "IgnoredProperty",
    "VersionTooHigh",
    "VersionTooLow",
    "MinEngineVersionCapFormatVersion1",
];

/// Names that were considered for codes and rejected: they must never become codes.
const REJECTED_NAMES: &[&str] = &[
    "JsonSyntax",
    "InvalidJson",
    "ParseError",
    "InvalidDocument",
    "MalformedEnvelope",
    "EnvelopeShape",
    "MissingField",
    "MissingTextures",
    "WrongType",
    "TypeMismatch",
    "DeserializeError",
    "MissingVersion",
    "InvalidFormatVersion",
    "InvalidVersionShape",
    "InvalidVersion",
    "FutureFormatVersion",
    "ConstraintViolation",
    "OutOfRange",
    "ArrayLength",
    "EmptyArray",
    "ValueNotAllowed",
    "InvalidUuid",
    "UnknownEnumValue",
    "UnknownAnimType",
    "ValueClamped",
    "ColorChannelOutOfRange",
    "EmptyFile",
    "MolangCompile",
    "DuplicateDefinition",
    "DuplicatePackId",
    "Overridden",
    "ShadowedDocument",
    "IoError",
    "UnknownField",
    "Cycle",
    "PoiTagCycle",
    "UnresolvedDefault",
    "UndeclaredArray",
    "UnknownParentGeometry",
    "UnknownParentBone",
    "PoiUnknownBlock",
    "PoiUnknownTag",
    "MissingTexture",
    "LayerImageMissing",
    "ReservedNamespace",
    "IdentifierWithoutNamespace",
    "InvalidIdentifier",
    "ExperimentRequired",
    "TextureSetWithoutPbrCapability",
    "ScriptModuleSkipped",
    "ScriptEntryUnsupported",
    "ScriptEntryMissing",
    "ModuleNotPermitted",
    // a variant that no emitter produces; it stays listed so that dropping it is a deliberate edit
    "PieceIdIgnored",
];

#[test]
fn every_listed_code_is_a_variant() {
    let missing: Vec<_> = LISTED_CODES
        .iter()
        .filter(|name| !DiagnosticCode::NAMES.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "listed codes without a variant: {missing:?}"
    );
}

/// The list knows every variant, so a code added to the enum is added here before the suite is
/// green.
#[test]
fn every_variant_is_a_code_of_the_list() {
    let unlisted: Vec<_> = DiagnosticCode::NAMES
        .iter()
        .filter(|name| !LISTED_CODES.contains(name))
        .collect();
    assert!(
        unlisted.is_empty(),
        "variants missing from the code list: {unlisted:?}"
    );
}

#[test]
fn no_rejected_name_is_a_variant() {
    let revived: Vec<_> = REJECTED_NAMES
        .iter()
        .filter(|name| DiagnosticCode::NAMES.contains(name))
        .collect();
    assert!(
        revived.is_empty(),
        "rejected names used as codes: {revived:?}"
    );
}

#[test]
fn code_names_are_unique() {
    let mut names = DiagnosticCode::NAMES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), DiagnosticCode::NAMES.len());
}

#[test]
fn name_is_the_variant_name() {
    let codes = [
        DiagnosticCode::UnknownSound { text: None },
        DiagnosticCode::UnrecognizedField { text: None },
        DiagnosticCode::MolangNotChecked { fields: 1 },
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::NotFound,
            text: None,
        },
    ];
    for code in codes {
        let debug = format!("{code:?}");
        assert!(debug.starts_with(code.name()), "{debug} vs {}", code.name());
    }
}
