//! The code vocabulary: every code the crate lists is a `DiagnosticCode` variant, and no name from
//! the rejected list is one.

use bedrock_addon_core::diagnostic::{Class, CodeDef, DiagnosticCode, Kind, codes};

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
    // a code declared by a `CodeDef`, named by its owner and its own name
    "Specific",
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

/// A code declared by a definition beside its rule.
static DENSITY_OUT_OF_UNIT: CodeDef<1> = CodeDef::new(
    "fog/density_out_of_unit",
    Kind::InvalidValue,
    Class::Error,
    "fog density {} is outside [0, 1]",
);

/// A specific code is the one `Specific` variant of the list, and is known by its stable id,
/// `owner/name`: its name is its own, never a listed or a rejected variant name.
#[test]
fn a_specific_code_is_known_by_owner_and_name() {
    let code = DiagnosticCode::from(DENSITY_OUT_OF_UNIT.with(["2"]));
    assert_eq!(code.variant_name(), "Specific");
    assert_eq!(code.id(), "fog/density_out_of_unit");
    assert_eq!(code.name(), "density_out_of_unit");
    assert!(!LISTED_CODES.contains(&code.name()));
    assert!(!REJECTED_NAMES.contains(&code.name()));
}

/// Every id of the core's definitions (`diagnostic::codes`), in their order. A definition added
/// there is added here before the suite is green; a family's definitions are listed by its own
/// crate.
const LISTED_IDS: &[&str] = &[
    "core/unsupported_version",
    "core/unsupported_version_worded",
    "core/beta_features_not_enabled",
    "core/beta_features_not_enabled_worded",
    "core/prerelease_format_version",
    "core/missing_toggles",
    "core/missing_toggles_worded",
    "core/deprecated_member",
    "core/deprecated_member_worded",
    "core/unrecognized_field",
    "core/unrecognized_field_worded",
    "core/unrecognized_component",
    "core/unrecognized_component_worded",
    "core/invalid_value",
    "core/invalid_value_worded",
    "core/missing_required_field",
    "core/missing_required_field_worded",
    "core/clamped_value",
    "core/clamped_value_worded",
    "core/dropped_component",
    "core/dropped_component_worded",
    "core/dropped_field",
    "core/dropped_field_worded",
    "core/member_not_kept",
    "core/member_not_kept_worded",
    "core/value_rewritten",
    "core/value_rewritten_worded",
    "core/duplicate_component_version",
    "core/unknown_sound",
    "core/unknown_sound_worded",
    "core/generic_goal_suffix_missing",
    "core/malformed_input",
    "core/syntax",
    "core/malformed_input_worded",
    "core/format_version_above_engine",
    "core/client_entity_shadowed",
    "core/client_entity_shadowed_older",
    "core/client_entity_shadowed_above_stack",
    "core/attachable_shadowed",
    "core/attachable_shadowed_older",
    "core/attachable_shadowed_above_stack",
    "core/item_binding_shadowed",
    "core/render_controller_shadowed",
    "core/identifier_collision",
    "core/identifier_collision_worded",
    "core/unresolved_reference",
    "core/unresolved_reference_worded",
    "core/invalid_reference",
    "core/invalid_reference_worded",
    "core/texture_not_found",
    "core/texture_not_found_worded",
    "core/splashes_replaced",
    "core/no_splashes",
    "core/unresolved_lang_key",
    "core/ignored_file",
    "core/ignored_file_worded",
    "core/pack_icon_missing",
    "core/pack_icon_shape",
    "core/glyph_sheet_shape",
    "core/skin_pack_name_mismatch",
    "core/skin_pack_creator_rule",
    "core/ignored_member",
    "core/ignored_member_worded",
    "core/unmatched_condition",
    "core/where_array_never_matches",
    "core/rebased_override",
    "core/shadowed_definition",
    "core/shadowed_definition_worded",
    "core/read_error",
    "core/read_error_worded",
    "core/missing_payload",
    "core/missing_payload_worded",
    "core/namespace_rule",
    "core/namespace_rule_worded",
    "core/invalid_type",
    "core/invalid_type_worded",
    "core/skipped_cache_document",
    "core/missing_format_version",
    "core/missing_format_version_worded",
    "core/malformed_format_version",
    "core/malformed_format_version_worded",
    "core/empty_document",
    "core/empty_document_worded",
    "core/unusual_format_version",
    "core/vanilla_rejects_version",
    "core/format_version_array",
    "core/payload_key_mismatch",
    "core/unknown_payload_key",
    "core/ignored_payload",
    "core/document_rejected",
    "core/document_rejected_worded",
    "core/missing_identifier",
    "core/missing_identifier_worded",
    "core/sound_definitions_wrapper_without_version",
    "core/missing_min_engine_version",
    "core/field_since_engine",
    "core/field_until_engine",
    "core/unsupported_value_form",
    "core/invalid_enum_value",
    "core/invalid_enum_value_worded",
    "core/invalid_molang",
    "core/molang_lint",
    "core/molang_not_checked",
    "core/keyframe_out_of_domain",
    "core/lossy_conversion",
    "core/literal_ignored",
    "core/non_canonical_member_case",
    "core/case_folded_transition",
    "core/unordered_remap_curve",
    "core/implicit_initial_state",
    "core/unrecognized_event_string",
    "core/invalid_node",
    "core/invalid_root",
    "core/unknown_node",
    "core/legacy_item_format",
    "core/persona_piece_pack",
    "core/lang_key_trailing_space",
    "core/scatter_param_value",
    "core/scatter_param_value_worded",
    "core/unsupported_operator",
    "core/invalid_texture",
    "core/invalid_texture_worded",
    "core/layer_image_undecodable",
    "core/layer_image_format",
    "core/capability_hint",
    "core/capability_hint_worded",
    "core/vibrant_visuals_disabled_by_pack",
    "core/font_file_missing",
    "core/font_file_missing_worded",
    "core/font_name_overridden",
    "core/mixed_font_page",
    "core/atomic_filter_lighting_ignored",
    "core/atomic_unsupported_render_controller",
    "core/duplicate_identifier",
    "core/duplicate_identifier_worded",
    "core/duplicate_bone",
    "core/atomic_client_entity_shadowed",
    "core/animation_controller_shadowed",
    "core/shadowed_by_builtin",
    "core/reference_cycle",
    "core/reference_cycle_worded",
    "core/client_biome_shadows_lower_packs",
    "core/client_biome_overridden_by_biomes_client",
    "core/vanilla_biomes_client_skipped",
    "core/legacy_fog_ignored",
    "core/biome_format_hint_legacy",
    "core/biome_format_hint_experimental",
    "core/biome_format_hint_experimental_surface",
    "core/no_effect",
    "core/no_effect_worded",
    "core/synthesized_component",
    "core/identifier_namespaced",
    "core/upgrade_table_missed_bare_identifier",
    "core/item_binding_overlap",
    "core/render_controller_fallback",
    "core/case_folding_disabled_at_line",
    "core/case_folding_disabled_at_line_unresolvable",
    "core/case_folding_disabled",
    "core/case_folding_disabled_unresolvable",
    "core/poi_block_claimed_twice",
    "core/poi_no_blocks",
    "core/sound_definitions_cleared",
    "core/sounds_json_table_cleared",
    "core/sounds_cleared_by_legacy_overlay",
    "core/distance_reset_by_legacy_overlay",
    "core/unused_property",
    "core/no_ui_definitions",
    "core/setting_type_mismatch",
    "core/setting_unknown_option",
    "core/setting_unknown_name",
    "core/setting_clamped",
    "core/legacy_setting_ignored",
    "core/path_rejected",
    "core/file_not_loaded",
    "core/internal_error",
    "core/unsupported_file_format",
    "core/duplicate_entry",
    "core/missing_dependency",
    "core/dependency_auto_added",
    "core/pack_not_found",
    "core/multiple_script_modules",
    "core/script_module_permission",
    "core/script_module_permission_worded",
    "core/script_module_version_unavailable",
    "core/script_module_version_conflict",
    "core/script_module_promoted",
    "core/script_alpha_dependency",
    "core/script_entry_outside_root",
    "core/script_syntax_error",
    "core/script_syntax_error_worded",
    "core/script_import_not_found",
    "core/script_import_crashes_engine",
    "core/script_module_unrecognized",
    "core/script_module_restricted",
    "core/script_import_not_checked",
    "core/script_file_not_loadable",
    "core/script_eval_without_capability",
    "core/script_import_meta_empty",
    "core/scripts_not_checked",
    "core/script_syntax_not_checked",
    "core/pack_config_shadows_default",
    "core/unmatched_config_folder",
    "core/unmatched_config_folder_expected",
    "core/net_config_rejected",
    "core/invalid_property",
    "core/invalid_property_worded",
    "core/malformed_property_version",
    "core/malformed_property_version_worded",
    "core/wrong_type_property",
    "core/wrong_type_property_worded",
    "core/missing_property",
    "core/missing_property_worded",
    "core/empty_property",
    "core/malformed_property_uuid",
    "core/duplicate_uuid",
    "core/missing_modules",
    "core/multiple_modules",
    "core/invalid_capability",
    "core/ignored_property",
    "core/version_too_high",
    "core/version_too_low",
    "core/min_engine_version_cap_format_version1",
];

/// The list knows every definition of `codes::ALL`, and nothing else.
#[test]
fn every_core_definition_is_listed_by_id() {
    let ids: Vec<&str> = codes::ALL.iter().map(|meta| meta.id()).collect();
    let unlisted: Vec<_> = ids.iter().filter(|id| !LISTED_IDS.contains(id)).collect();
    assert!(
        unlisted.is_empty(),
        "definitions missing from the id list: {unlisted:?}"
    );
    let gone: Vec<_> = LISTED_IDS.iter().filter(|id| !ids.contains(id)).collect();
    assert!(gone.is_empty(), "listed ids without a definition: {gone:?}");
}

/// No two definitions share an id, and every one is the core's and names the legacy variant it
/// replaces: a listed one, never a rejected name.
#[test]
fn core_definition_ids_are_unique_and_replace_listed_codes() {
    let mut ids: Vec<&str> = codes::ALL.iter().map(|meta| meta.id()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), codes::ALL.len());
    for meta in codes::ALL {
        assert_eq!(meta.owner(), "core", "{}", meta.id());
        let legacy = meta.legacy().unwrap_or_default();
        assert!(LISTED_CODES.contains(&legacy), "{}", meta.id());
        assert!(!REJECTED_NAMES.contains(&legacy), "{}", meta.id());
        assert_eq!(meta.name(), legacy, "{}", meta.id());
    }
}
