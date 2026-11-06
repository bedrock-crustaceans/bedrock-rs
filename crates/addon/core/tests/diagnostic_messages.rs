//! The message table: every vanilla content-log template rendered from a `DiagnosticCode`
//! with sample arguments and compared with the template byte for byte. This is the one test in
//! the crate family that matches on diagnostic text.

use bedrock_addon_core::diagnostic::ShadowReason;
use bedrock_addon_core::diagnostic::{
    Class, CodeDef, DiagnosticCode, Kind, NotFoundHint, VanillaText,
};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

const V1_20_0: FormatVersion = FormatVersion {
    major: 1,
    minor: 20,
    patch: 0,
};

/// One test per template: `$code` must render exactly `$expected`.
macro_rules! message_table {
    ($($name:ident: $code:expr => $expected:expr;)*) => {
        $(
            #[test]
            fn $name() {
                let code: DiagnosticCode = $code;
                assert_eq!(code.to_string(), $expected, "{code:?}");
            }
        )*
    };
}

message_table! {
    message_table_member_not_kept:
        DiagnosticCode::MemberNotKept { text: None }
        => "member read and not kept";
    message_table_value_rewritten:
        DiagnosticCode::ValueRewritten { text: None }
        => "value held in another form than it is written in";
    message_table_field_until_engine:
        DiagnosticCode::FieldUntilEngine { last: V1_20_0 }
        => "field is gone from this engine; engine version 1.20.0 was the last known to have it";
    message_table_case_folding_disabled:
        DiagnosticCode::CaseFoldingDisabled { line: Some(3), unresolvable: false }
        => "an apostrophe at line 3 disabled case folding";
    message_table_case_folding_disabled_name:
        DiagnosticCode::CaseFoldingDisabled { line: Some(3), unresolvable: true }
        => "an apostrophe at line 3 disabled case folding; the game resolves no reference to this name";
    message_table_case_folding_escaped_name:
        DiagnosticCode::CaseFoldingDisabled { line: None, unresolvable: true }
        => "a capital letter the game's case folding does not reach; the game resolves no reference to this name";
    message_table_unknown_format_version:
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::UnknownFormatVersion { version: "1.7.0".into() }),
        } => "Unknown/unsupported version value specified in \"format_version\" field: 1.7.0";
    message_table_no_format_version_field:
        DiagnosticCode::MissingFormatVersion { text: Some(VanillaText::NoFormatVersionField) }
        => "no format_version field was given";
    message_table_format_version_not_given:
        DiagnosticCode::MissingFormatVersion { text: Some(VanillaText::FormatVersionNotGiven) }
        => "warning parsing : the string format_version field was not given;";
    message_table_no_valid_version:
        DiagnosticCode::MissingFormatVersion { text: Some(VanillaText::NoValidVersion) }
        => "no valid version passed by the caller or found in the input";
    message_table_malformed_format_version:
        DiagnosticCode::MalformedFormatVersion { text: None } => "Malformed format version";
    message_table_document_member_count:
        DiagnosticCode::MalformedInput {
            message: String::new(),
            text: Some(VanillaText::DocumentMemberCount),
        } => "A Minecraft Document must have exactly two members: \"format_version\" and a payload member";
    message_table_document_json:
        DiagnosticCode::MalformedInput {
            message: String::new(),
            text: Some(VanillaText::DocumentJson { message: "unexpected end of input".into() }),
        } => "Failed to parse the Minecraft Document JSON: unexpected end of input";
    message_table_use_format_version:
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::UseFormatVersion {
                what: "item",
                name: "demo:ruby".into(),
                version: V1_20_0,
            }),
        } => "To use item 'demo:ruby', use json format version 1.20.0 or higher";
    message_table_deprecated:
        DiagnosticCode::DeprecatedMember {
            text: Some(VanillaText::Deprecated {
                member: "minecraft:armor".into(),
                version: FormatVersion { major: 1, minor: 20, patch: 20 },
            }),
        } => "minecraft:armor has been deprecated and is not available in json format 1.20.20";
    message_table_two_component_versions:
        DiagnosticCode::DuplicateComponentVersion {
            old: "minecraft:foil".into(),
            new: "minecraft:glint".into(),
        } => "Two versions of the same component found: minecraft:foil and minecraft:glint. Please remove minecraft:glint, or upgrade your format version and remove minecraft:foil.";
    message_table_actor_above_engine:
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::ActorAboveEngine {
                version: "99.0.0".into(),
                engine: FormatVersion { major: 1, minor: 26, patch: 50 },
            }),
        } => "Error: actor resource file has format version: 99.0.0 exceeds game version: 1.26.50. Skipping.";
    message_table_manifest_invalid_value:
        DiagnosticCode::InvalidProperty {
            text: Some(VanillaText::ManifestInvalidValue { path: "/modules/0/type".into() }),
        } => "Provided '/modules/0/type' element has an invalid value in pack manifest.";
    message_table_manifest_not_semver:
        DiagnosticCode::MalformedPropertyVersion {
            text: Some(VanillaText::ManifestNotSemVer { path: "/dependencies/0/version".into() }),
        } => "Provided '/dependencies/0/version' element is not SemVer (semver.org) compliant in pack manifest.";
    message_table_manifest_wrong_type:
        DiagnosticCode::WrongTypeProperty {
            text: Some(VanillaText::ManifestWrongType { path: "/dependencies/0/version".into() }),
        } => "Required '/dependencies/0/version' element is the wrong type in pack manifest.";
    message_table_dependency_invalid_version:
        DiagnosticCode::MalformedPropertyVersion {
            text: Some(VanillaText::DependencyInvalidVersion {
                module: "@minecraft/server".into(),
                version: "0.0.0".into(),
            }),
        } => "dependency @minecraft/server invalid version: 0.0.0";
    message_table_script_invalid_utf8:
        DiagnosticCode::ScriptSyntaxError {
            text: Some(VanillaText::ScriptInvalidUtf8 { file: "main.js".into(), line: 1 }),
        } => "SyntaxError: invalid UTF-8 sequence    at main.js:1";
    message_table_script_expecting_semicolon:
        DiagnosticCode::ScriptSyntaxError {
            text: Some(VanillaText::ScriptExpectingSemicolon { file: "main.js".into(), line: 1 }),
        } => "SyntaxError: expecting ';'    at main.js:1";
    message_table_script_syntax_not_checked:
        DiagnosticCode::ScriptSyntaxNotChecked { files: 3 }
        => "syntax of 3 script files not checked: nested too deeply for the parser";
    message_table_module_unrecognized:
        DiagnosticCode::ScriptModuleUnrecognized { module: "@minecraft/server-ui".into() }
        => "Module [@minecraft/server-ui] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?";
    message_table_plugin_one_module_per_pack:
        DiagnosticCode::MultipleScriptModules { plugin: "Demo - 1.0.0".into() }
        => "Plugin [Demo - 1.0.0] - skipped, only one script module allowed per pack.";
    message_table_plugin_module_exists:
        DiagnosticCode::DuplicateIdentifier {
            text: Some(VanillaText::PluginModuleExists {
                plugin: "Demo - 1.0.0".into(),
                uuid: uuid::Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef),
                version: "1.0.0".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - module uuid [01234567-89ab-cdef-0123-456789abcdef] with version [1.0.0] already exists, skipping.";
    message_table_plugin_unsupported_runtime:
        DiagnosticCode::InvalidEnumValue {
            text: Some(VanillaText::PluginUnsupportedRuntime {
                plugin: "Demo - 1.0.0".into(),
                runtime: String::new(),
            }),
        } => "Plugin [Demo - 1.0.0] - using unsupported runtime [].";
    message_table_plugin_entry_not_supported:
        DiagnosticCode::InvalidValue {
            text: Some(VanillaText::PluginEntryNotSupported {
                plugin: "Demo - 1.0.0".into(),
                entry: "scripts/main.ts".into(),
                runtime: "JavaScript".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - entry [scripts/main.ts] not supported for runtime [JavaScript].";
    message_table_plugin_missing_main_file:
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::PluginMissingMainFile {
                plugin: "Demo - 1.0.0".into(),
                main: "Main.js".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - does not contain main file [Main.js].";
    message_table_plugin_beta_apis:
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::PluginBetaApis {
                plugin: "Demo - 1.0.0".into(),
                dependency: "@minecraft/server - 2.1.0-beta".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - requesting dependency on beta APIs [@minecraft/server - 2.1.0-beta], but the Beta APIs experiment is not enabled.";
    message_table_plugin_beta_apis_module:
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::PluginBetaApisModule {
                plugin: "Demo - 1.0.0".into(),
                module: "@minecraft/server-gametest".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - requesting dependency on [@minecraft/server-gametest], but the Beta APIs experiment is not enabled.";
    message_table_plugin_internal_apis:
        DiagnosticCode::ScriptModulePermission {
            text: Some(VanillaText::PluginInternalApis {
                plugin: "Demo - 1.0.0".into(),
                dependency: "@minecraft/server - 2.2.0-internal".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - requesting dependency on internal APIs [@minecraft/server - 2.2.0-internal], but does not have permission to access them.";
    message_table_plugin_module_not_configured:
        DiagnosticCode::ScriptModulePermission {
            text: Some(VanillaText::PluginModuleNotConfigured {
                plugin: "Demo - 1.0.0".into(),
                module: "@minecraft/server-net".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - requesting dependency on module [@minecraft/server-net] but it is not configured to use it.";
    message_table_plugin_not_editor_world:
        DiagnosticCode::DroppedField {
            text: Some(VanillaText::PluginNotEditorWorld {
                plugin: "Demo - 1.0.0".into(),
                module: "@minecraft/server-editor".into(),
            }),
        } => "Plugin [Demo - 1.0.0] - skipping dependency on [@minecraft/server-editor] because this is not an editor world.";
    message_table_plugin_dependency_by_uuid:
        DiagnosticCode::DeprecatedMember {
            text: Some(VanillaText::PluginDependencyByUuid { plugin: "Demo - 1.0.0".into() }),
        } => "Plugin [Demo - 1.0.0] dependency by UUID is deprecated and only supports version 0.1.0, use module_name instead.";
    message_table_plugin_invalid_version:
        DiagnosticCode::ScriptModuleVersionUnavailable {
            plugin: "Demo".into(),
            module: "Demo - 1.0.0".into(),
            requested: "9.0.0".into(),
            dependency: "@minecraft/server".into(),
            current: None,
            available: vec!["1.19.0".into(), "2.4.0".into()],
        } => "Plugin [Demo] - module [Demo - 1.0.0] requested invalid version [9.0.0] of [@minecraft/server].\nAvailable versions:\n1.19.0\n2.4.0";
    message_table_plugin_invalid_version_with_current:
        DiagnosticCode::ScriptModuleVersionUnavailable {
            plugin: "Demo".into(),
            module: "Demo - 1.0.0".into(),
            requested: "2.0.0-beta".into(),
            dependency: "@minecraft/server".into(),
            current: Some(("beta".into(), "2.5.0-beta".into())),
            available: vec!["2.4.0".into()],
        } => "Plugin [Demo] - module [Demo - 1.0.0] requested invalid version [2.0.0-beta] of [@minecraft/server].\nCurrent [beta] version is [2.5.0-beta].\nAvailable versions:\n2.4.0";
    message_table_plugin_version_conflict:
        DiagnosticCode::ScriptModuleVersionConflict {
            plugin: "Demo".into(),
            module: "@minecraft/server".into(),
            requests: vec![
                ("1.0.0".into(), "Demo - 1.0.0".into()),
                ("2.0.0".into(), "@minecraft/server-ui - 2.0.0".into()),
            ],
        } => "Plugin [Demo] - version conflict for module [@minecraft/server].\n [1.0.0] requested by [Demo - 1.0.0],\n [2.0.0] requested by [@minecraft/server-ui - 2.0.0]";
    message_table_plugin_promoted:
        DiagnosticCode::ScriptModulePromoted {
            plugin: "Demo".into(),
            module: "@minecraft/server".into(),
            from: "1.0.0".into(),
            to: "1.1.0".into(),
            requested_by: "@minecraft/server-ui - 1.1.0".into(),
        } => "Plugin [Demo] - promoted [@minecraft/server] from [1.0.0] to [1.1.0] requested by [@minecraft/server-ui - 1.1.0].";
    message_table_plugin_unknown_module:
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::PluginUnknownModule {
                plugin: "Demo".into(),
                module: "Demo - 1.0.0".into(),
                dependency: "@minecraft/nope".into(),
            }),
        } => "Plugin [Demo] - module [Demo - 1.0.0] depends on unknown module [@minecraft/nope].";
    message_table_config_open_failed:
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::PermissionDenied,
            text: Some(VanillaText::ConfigOpenFailed { file: "config/default/permissions.json".into() }),
        } => "Failed to open file 'config/default/permissions.json'";
    message_table_config_parse_failed:
        DiagnosticCode::MalformedInput {
            message: "unexpected end of input".into(),
            text: Some(VanillaText::ConfigParseFailed { file: "config/default/permissions.json".into() }),
        } => "Failed to parse JSON in file 'config/default/permissions.json'";
    message_table_config_load_failed:
        DiagnosticCode::NetConfigRejected {
            module: "@minecraft/server-net".into(),
            errors: "allowed_uris must be an array".into(),
        } => "Failed to load config for @minecraft/server-net due to error(s): allowed_uris must be an array";
    message_table_turn_on_toggle:
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::TurnOnToggle {
                what: "component",
                name: "minecraft:multi_block".into(),
                toggle: "Upcoming Creator Features",
            }),
        } => "To use component 'minecraft:multi_block', turn on the 'Upcoming Creator Features' toggle in settings";
    message_table_turn_on_toggle_before:
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::TurnOnToggleBefore {
                what: "component",
                name: "minecraft:redstone_conductivity".into(),
                version: FormatVersion { major: 1, minor: 21, patch: 30 },
                toggle: "Upcoming Creator Features",
            }),
        } => "To use component 'minecraft:redstone_conductivity' in json formats earlier than 1.21.30, turn on the 'Upcoming Creator Features' toggle in settings";
    message_table_no_parser_for_version:
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::NoParserForVersion { version: "1.26.60".into() }),
        } => "unsupported version: no parser available for version 1.26.60";
    message_table_beta_documents:
        DiagnosticCode::BetaFeaturesNotEnabled { text: Some(VanillaText::BetaDocuments) }
        => "Turn on the Beta APIs toggle in settings to allow beta documents.";
    // The feature scatter conversion reports a negative iteration count with the game's text.
    message_table_scatter_param_value:
        DiagnosticCode::ScatterParamValue {
            text: Some(VanillaText::Fixed("Bad value for iterations - should be >= 0")),
        } => "Bad value for iterations - should be >= 0";
    message_table_unknown_feature_type:
        DiagnosticCode::UnknownPayloadKey { name: "minecraft:nope_feature".into() }
        => "Could not find registered feature type that matches name 'minecraft:nope_feature'";
    message_table_fog_schema:
        DiagnosticCode::DocumentRejected { text: Some(VanillaText::FogSchema) }
        => "Failed to validate fog schema";
    message_table_unknown_behavior:
        DiagnosticCode::UnknownNode { name: "minecraft:nope".into() } => "Unknown behavior minecraft:nope";
    message_table_atomic_light_color_expression:
        DiagnosticCode::NoEffect { text: Some(VanillaText::AtomicLightColorExpression) }
        => "Atomic Entity - light_color_multiplier expressions are not supported.";
    // The atomic client entity checks: each unsupported feature has the game's own line.
    message_table_atomic_unsupported_render_controller:
        DiagnosticCode::AtomicUnsupportedRenderController
        => "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).";
    message_table_atomic_filter_lighting:
        DiagnosticCode::AtomicFilterLightingIgnored
        => "Atomic Entity - filter_lighting is not supported.";
    message_table_restricted_module:
        DiagnosticCode::ScriptModuleRestricted {
            module: "@minecraft/debug-utilities".into(),
            referrer: "main.js".into(),
        } => "Import failed, can not load restricted module '@minecraft/debug-utilities' from 'main.js'.";
    message_table_import_not_found:
        DiagnosticCode::ScriptImportNotFound { specifier: "util.js".into(), hint: None }
        => "ReferenceError: Import [util.js] not found.";
    message_table_import_not_found_with_hint:
        DiagnosticCode::ScriptImportNotFound {
            specifier: "Util.js".into(),
            hint: Some(NotFoundHint::CaseMismatch),
        } => "ReferenceError: Import [Util.js] not found.";
    message_table_fixed_metadata_text:
        DiagnosticCode::DeprecatedMember {
            text: Some(VanillaText::Fixed(
                "minecraft:foil has been deprecated in format versions 1.20.20 and above. Use minecraft:glint instead.",
            )),
        } => "minecraft:foil has been deprecated in format versions 1.20.20 and above. Use minecraft:glint instead.";
    // Entity schema messages: a deprecated member ("member \"{}\" is deprecated") and an invalid
    // `should_bounce` value ("'%s' is not a valid 'should_bounce' value.").
    message_table_member_deprecated:
        DiagnosticCode::DeprecatedMember {
            text: Some(VanillaText::MemberDeprecated { member: "must_see".into() }),
        } => "member \"must_see\" is deprecated";
    message_table_not_a_valid_value:
        DiagnosticCode::InvalidValue {
            text: Some(VanillaText::NotAValidValue { value: "sometimes".into(), member: "should_bounce" }),
        } => "'sometimes' is not a valid 'should_bounce' value.";
    // An entity `scripts.animate` object with more than one item: "Error: animation script
    // object has %d items - size must be 1.  Skipping."
    message_table_animation_script_object_size:
        DiagnosticCode::InvalidValue {
            text: Some(VanillaText::AnimationScriptObjectSize { size: 2 }),
        } => "Error: animation script object has 2 items - size must be 1.  Skipping.";
    // A non-vanilla item in the `minecraft` namespace: "Non-Vanilla item '%s' should not be
    // using the "minecraft" namespace!"
    message_table_item_non_vanilla_namespace:
        DiagnosticCode::NamespaceRule {
            text: Some(VanillaText::ItemNonVanillaNamespace { identifier: "minecraft:ruby".into() }),
        } => "Non-Vanilla item 'minecraft:ruby' should not be using the \"minecraft\" namespace!";
    // A spawn rule file without its payload key (a payload under another key) has the game's
    // text "Missing identifier in spawn rule file"; without text the crate words it itself.
    message_table_missing_payload_with_vanillas_text:
        DiagnosticCode::MissingPayload {
            text: Some(VanillaText::Fixed("Missing identifier in spawn rule file")),
        } => "Missing identifier in spawn rule file";
    message_table_missing_payload_without_text:
        DiagnosticCode::MissingPayload { text: None } => "the document has no payload member";
    // A dimension definition replaced by another pack's: the game's text is "Dimension
    // definition identifier '%s' registered via scripting was replaced by the definition from
    // pack id '%s'"; without text the crate words it itself.
    message_table_shadowed_definition_with_vanillas_text:
        DiagnosticCode::ShadowedDefinition {
            text: Some(VanillaText::Formatted {
                template: "Dimension definition identifier '{}' registered via scripting was replaced by the definition from pack id '{}'",
                args: vec!["test:void".into(), "00000000-0000-0000-0000-000000000001".into()],
            }),
        } => "Dimension definition identifier 'test:void' registered via scripting was replaced by the definition from pack id '00000000-0000-0000-0000-000000000001'";
    // An unknown sound event name has the game's text; without text the crate words it itself.
    message_table_unknown_sound_with_vanillas_text:
        DiagnosticCode::UnknownSound {
            text: Some(VanillaText::Formatted {
                template: "Event name '{}' is not a valid LevelSoundEvent",
                args: vec!["notanevent".into()],
            }),
        } => "Event name 'notanevent' is not a valid LevelSoundEvent";
    // Each of these codes carries the game's text when the family has one, and the crate's own
    // wording without it.
    message_table_invalid_texture_with_vanillas_text:
        DiagnosticCode::InvalidTexture {
            text: Some(VanillaText::Formatted {
                template: "Caustics texture \"{}\" provided in resource pack does not evenly divide into frames. Ensure the texture height is evenly divisible by the width.",
                args: vec!["textures/c".into()],
            }),
        } => "Caustics texture \"textures/c\" provided in resource pack does not evenly divide into frames. Ensure the texture height is evenly divisible by the width.";
    message_table_invalid_texture_without_text:
        DiagnosticCode::InvalidTexture { text: None } => "image not decodable or of the wrong proportions";
    message_table_capability_hint_with_a_text:
        DiagnosticCode::CapabilityHint {
            text: Some(VanillaText::Fixed("render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability")),
        } => "render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability";
    message_table_capability_hint_without_text:
        DiagnosticCode::CapabilityHint { text: None } => "takes effect only with a capability the pack does not declare";
    message_table_font_file_missing_with_vanillas_text:
        DiagnosticCode::FontFileMissing {
            text: Some(VanillaText::Fixed("Couldn't load MSDF Font information")),
        } => "Couldn't load MSDF Font information";
    message_table_font_file_missing_without_text:
        DiagnosticCode::FontFileMissing { text: None } => "font file not found";
    message_table_client_entity_shadowed_by_priority:
        DiagnosticCode::ClientEntityShadowed { reason: None } => "client entity superseded by another definition of its identifier";
    message_table_client_entity_shadowed_older:
        DiagnosticCode::ClientEntityShadowed {
            reason: Some(ShadowReason::OlderMinEngineVersion),
        } => "client entity ignored: its min_engine_version is below the loaded definition's";
    message_table_client_entity_shadowed_above_the_stack:
        DiagnosticCode::ClientEntityShadowed {
            reason: Some(ShadowReason::MinEngineVersionAboveStack),
        } => "client entity ignored: its min_engine_version is above the stack's";
    message_table_attachable_shadowed_by_priority:
        DiagnosticCode::AttachableShadowed { reason: None } => "attachable superseded by another definition of its identifier";
    message_table_attachable_shadowed_older:
        DiagnosticCode::AttachableShadowed {
            reason: Some(ShadowReason::OlderMinEngineVersion),
        } => "attachable ignored: its min_engine_version is below the loaded definition's";
    message_table_attachable_shadowed_above_the_stack:
        DiagnosticCode::AttachableShadowed {
            reason: Some(ShadowReason::MinEngineVersionAboveStack),
        } => "attachable ignored: its min_engine_version is above the stack's";
    message_table_unknown_sound_without_text:
        DiagnosticCode::UnknownSound { text: None } => "unknown sound event";
    message_table_shadowed_definition_without_text:
        DiagnosticCode::ShadowedDefinition { text: None } => "definition shadowed by a higher-priority source";
    // A scatter parameter without the game's text gets the crate's own wording.
    message_table_scatter_param_value_without_text:
        DiagnosticCode::ScatterParamValue { text: None } => "invalid scatter parameter";
    // The hints for biome files below the lowest format version the crate loads.
    message_table_biome_format_hint_1_12_shape:
        DiagnosticCode::BiomeFormatHint { version: None, surface_components: false }
        => "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher";
    message_table_biome_format_hint_1_12_shape_ignores_surface_components:
        DiagnosticCode::BiomeFormatHint { version: None, surface_components: true }
        => "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher";
    message_table_biome_format_hint_experimental_version:
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion { major: 1, minor: 20, patch: 60 }),
            surface_components: false,
        } => "biome format 1.20.60 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation)";
    message_table_biome_format_hint_experimental_version_with_surface_components:
        DiagnosticCode::BiomeFormatHint {
            version: Some(FormatVersion { major: 1, minor: 13, patch: 0 }),
            surface_components: true,
        } => "biome format 1.13.0 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder)";
    // A duplicate crafting catalog item has the game's text "You can only add an item to the
    // crafting item catalog once. Found duplicate %s"; without text the crate words it itself.
    message_table_identifier_collision_with_vanillas_text:
        DiagnosticCode::IdentifierCollision {
            text: Some(VanillaText::Formatted {
                template: "You can only add an item to the crafting item catalog once. Found duplicate {}",
                args: vec!["minecraft:stone".into()],
            }),
        } => "You can only add an item to the crafting item catalog once. Found duplicate minecraft:stone";
    message_table_identifier_collision_without_text:
        DiagnosticCode::IdentifierCollision { text: None } => "two definitions share one identifier";
    // A cycle among point-of-interest tags is worded "TAG_n was a part of a cycle"; without
    // text the crate words it itself.
    message_table_reference_cycle_with_the_poi_text:
        DiagnosticCode::ReferenceCycle {
            text: Some(VanillaText::Formatted {
                template: "{} was a part of a cycle",
                args: vec!["test:t1".into()],
            }),
        } => "test:t1 was a part of a cycle";
    // A circular camera preset reference has the game's text "Circular reference: Preset '%s'
    // inherits from '%s' more than once and will be discarded."; without text the crate words
    // it itself.
    message_table_reference_cycle_with_vanillas_text:
        DiagnosticCode::ReferenceCycle {
            text: Some(VanillaText::Formatted {
                template: "Circular reference: Preset '{}' inherits from '{}' more than once and will be discarded.",
                args: vec!["ex:a".into(), "ex:a".into()],
            }),
        } => "Circular reference: Preset 'ex:a' inherits from 'ex:a' more than once and will be discarded.";
    message_table_reference_cycle_without_text:
        DiagnosticCode::ReferenceCycle { text: None } => "references form a cycle";
}

/// A code without vanilla text renders one line of the crate's own wording, never empty.
#[test]
fn every_code_without_text_renders_one_nonempty_line() {
    let codes = [
        DiagnosticCode::UnrecognizedField { text: None },
        DiagnosticCode::UnknownSound { text: None },
        DiagnosticCode::PathRejected {
            entry: "a\u{fffd}.json".into(),
        },
        DiagnosticCode::MolangNotChecked { fields: 3 },
        DiagnosticCode::ReadError {
            kind: std::io::ErrorKind::NotFound,
            text: None,
        },
        DiagnosticCode::MalformedInput {
            message: "unexpected character '}'".into(),
            text: None,
        },
    ];
    for code in codes {
        let text = code.to_string();
        assert!(
            !text.is_empty() && !text.contains('\n'),
            "{code:?} renders {text:?}"
        );
    }
}

/// The text of `MolangNotChecked` says what is true of this build. Without the `molang`
/// feature nothing can check; with it, the finding is raised only where no `MolangCheck` was
/// given (`LoadOptions::molang` is `None`, the default).
#[cfg(not(feature = "molang"))]
const MOLANG_NOT_CHECKED: &str = "3 Molang field(s) not checked: the `molang` feature is off";
/// See the other half.
#[cfg(feature = "molang")]
const MOLANG_NOT_CHECKED: &str =
    "3 Molang field(s) not checked: no Molang check was given (`LoadOptions::molang` is `None`)";

/// Both builds by name: the sentence of the feature that is off is never printed by a
/// build that has it, and the other way round.
#[test]
fn molang_not_checked_names_the_cause_that_holds_in_this_build() {
    let text = DiagnosticCode::MolangNotChecked { fields: 3 }.to_string();
    assert_eq!(text, MOLANG_NOT_CHECKED);
    assert_eq!(
        text.contains("feature is off"),
        cfg!(not(feature = "molang")),
        "{text}"
    );
    assert_eq!(
        text.contains("`LoadOptions::molang` is `None`"),
        cfg!(feature = "molang"),
        "{text}"
    );
}

/// The pack of the sample rows that name one.
fn pack() -> PackIdVersion {
    PackIdVersion::new(
        Uuid::from_u128(1),
        SemVersion::new(1, 0, 0),
        PackType::Resource,
    )
}

/// The dependency of the sample rows that name one.
fn dependency() -> PackIdVersion {
    PackIdVersion::new(
        Uuid::from_u128(2),
        SemVersion::new(2, 1, 0),
        PackType::Behavior,
    )
}

/// One row per `DiagnosticCode` variant, in declaration order: a sample value without vanilla
/// text and the line the crate's own wording renders. A code added to the enum gets its row here.
#[expect(
    clippy::too_many_lines,
    reason = "a table with one row per code; there is no seam to split at"
)]
fn crate_text_of_every_code() -> Vec<(DiagnosticCode, &'static str)> {
    use DiagnosticCode as C;
    vec![
        (
            C::UnsupportedVersion { text: None },
            "unsupported format_version",
        ),
        (
            C::BetaFeaturesNotEnabled { text: None },
            "beta document without the Beta APIs toggle",
        ),
        (
            C::PrereleaseFormatVersion {
                version: "1.21.0-beta".into(),
            },
            "format_version 1.21.0-beta has a pre-release tag and selects the schema below its release",
        ),
        (
            C::MissingToggles { text: None },
            "needs a toggle that is not enabled",
        ),
        (C::DeprecatedMember { text: None }, "deprecated member"),
        (C::UnrecognizedField { text: None }, "unrecognized field"),
        (
            C::UnrecognizedComponent { text: None },
            "unrecognized component",
        ),
        (C::InvalidValue { text: None }, "invalid value"),
        (
            C::MissingRequiredField { text: None },
            "missing required field",
        ),
        (
            C::ClampedValue { text: None },
            "value clamped into its range",
        ),
        (C::DroppedComponent { text: None }, "component dropped"),
        (C::DroppedField { text: None }, "field dropped"),
        (C::MemberNotKept { text: None }, "member read and not kept"),
        (
            C::ValueRewritten { text: None },
            "value held in another form than it is written in",
        ),
        (
            C::DuplicateComponentVersion {
                old: "minecraft:a".into(),
                new: "minecraft:a.v2".into(),
            },
            "Two versions of the same component found: minecraft:a and minecraft:a.v2. Please remove minecraft:a.v2, or upgrade your format version and remove minecraft:a.",
        ),
        (C::UnknownSound { text: None }, "unknown sound event"),
        (
            C::GenericGoalSuffixMissing,
            "generic goal name lacks its required suffix",
        ),
        (
            C::MalformedInput {
                message: "unexpected character '}'".into(),
                text: None,
            },
            "unexpected character '}'",
        ),
        (
            C::FormatVersionAboveEngine {
                version: "1.99.0".into(),
            },
            "format_version 1.99.0 is above the engine version 1.26.60; parsed as the newest schema",
        ),
        (
            C::ClientEntityShadowed { reason: None },
            "client entity superseded by another definition of its identifier",
        ),
        (
            C::AttachableShadowed { reason: None },
            "attachable superseded by another definition of its identifier",
        ),
        (
            C::ItemBindingShadowed,
            "item binding superseded by a higher-priority pack",
        ),
        (
            C::RenderControllerShadowed,
            "render controller superseded by a higher-priority pack",
        ),
        (
            C::IdentifierCollision { text: None },
            "two definitions share one identifier",
        ),
        (
            C::UnresolvedReference { text: None },
            "unresolved reference",
        ),
        (
            C::InvalidReference { text: None },
            "reference resolves to something this use cannot accept",
        ),
        (C::TextureNotFound { text: None }, "texture not found"),
        (
            C::SplashesReplaced,
            "splashes replace the lower packs' splashes",
        ),
        (C::NoSplashes, "no splash texts"),
        (C::UnresolvedLangKey, "localisation key not defined"),
        (C::IgnoredFile { text: None }, "file ignored"),
        (C::PackIconMissing, "pack_icon.png missing"),
        (C::PackIconShape, "pack_icon.png is not a square image"),
        (C::GlyphSheetShape, "glyph sheet is not a 16 x 16 grid"),
        (
            C::SkinPackNameMismatch,
            "skin pack name does not match its localisation",
        ),
        (C::SkinPackCreatorRule, "skin pack breaks a creator rule"),
        (C::IgnoredMember { text: None }, "member ignored"),
        (C::UnmatchedCondition, "condition matched no element"),
        (
            C::WhereArrayNeverMatches,
            "an array-valued `where` never matches",
        ),
        (
            C::RebasedOverride,
            "overlay `name@base` key shadows a base name",
        ),
        (
            C::ShadowedDefinition { text: None },
            "definition shadowed by a higher-priority source",
        ),
        (
            C::ReadError {
                kind: std::io::ErrorKind::NotFound,
                text: None,
            },
            "cannot read the file (entity not found)",
        ),
        (
            C::MissingPayload { text: None },
            "the document has no payload member",
        ),
        (C::NamespaceRule { text: None }, "namespace rule failed"),
        (C::InvalidType { text: None }, "wrong JSON type"),
        (C::SkippedCacheDocument, "cache document not parsed"),
        (C::MissingFormatVersion { text: None }, "no format_version"),
        (
            C::MalformedFormatVersion { text: None },
            "Malformed format version",
        ),
        (C::EmptyDocument { text: None }, "empty document"),
        (
            C::UnusualFormatVersion,
            "format_version between two known versions; parsed by the lower one",
        ),
        (
            C::VanillaRejectsVersion,
            "vanilla has no parser for this format_version",
        ),
        (C::FormatVersionArray, "format_version given as an array"),
        (
            C::PayloadKeyMismatch,
            "payload member is not the family's payload key",
        ),
        (
            C::UnknownPayloadKey {
                name: "minecraft:thing".into(),
            },
            "Could not find registered feature type that matches name 'minecraft:thing'",
        ),
        (C::IgnoredPayload, "second payload ignored"),
        (
            C::DocumentRejected { text: None },
            "document failed schema validation",
        ),
        (C::MissingIdentifier { text: None }, "no usable identifier"),
        (
            C::SoundDefinitionsWrapperWithoutVersion,
            "`sound_definitions` wrapper without a format_version of at least 1.14.0",
        ),
        (
            C::MissingMinEngineVersion,
            "the pack has no min_engine_version",
        ),
        (
            C::FieldSinceEngine { since: V1_20_0 },
            "field needs engine version 1.20.0 or newer",
        ),
        (
            C::FieldUntilEngine { last: V1_20_0 },
            "field is gone from this engine; engine version 1.20.0 was the last known to have it",
        ),
        (
            C::UnsupportedValueForm,
            "value form not supported by the configured engine",
        ),
        (C::InvalidEnumValue { text: None }, "invalid enum value"),
        (
            C::InvalidMolang {
                expression: "q.foo(".into(),
            },
            "q.foo(",
        ),
        (
            C::MolangLint {
                expression: "q.foo".into(),
            },
            "q.foo",
        ),
        (C::MolangNotChecked { fields: 3 }, MOLANG_NOT_CHECKED),
        (
            C::KeyframeOutOfDomain,
            "keyframe key outside [0, 1]; dropped",
        ),
        (C::LossyConversion, "value loses precision when written"),
        (C::LiteralIgnored, "literal ignored"),
        (
            C::NonCanonicalMemberCase,
            "member name matched ignoring case",
        ),
        (C::CaseFoldedTransition, "transition expression lower-cased"),
        (
            C::UnorderedRemapCurve,
            "remap_curve keys out of numeric order",
        ),
        (
            C::ImplicitInitialState,
            "no initial_state and no default state; the first state is used",
        ),
        (
            C::UnrecognizedEventString,
            "event is neither a command, an entity event nor Molang",
        ),
        (
            C::InvalidNode,
            "node is not an object with exactly one member",
        ),
        (
            C::InvalidRoot,
            "minecraft:root is not an object with exactly one member",
        ),
        (
            C::UnknownNode {
                name: "minecraft:thing".into(),
            },
            "Unknown behavior minecraft:thing",
        ),
        (
            C::LegacyItemFormat {
                reason: "no menu category",
            },
            "item parsed with the legacy schema: no menu category",
        ),
        (
            C::PersonaPiecePack,
            "persona-piece pack in the resource-pack directory",
        ),
        (
            C::LangKeyTrailingSpace,
            "key ends, or value starts, with a space",
        ),
        (
            C::ScatterParamValue { text: None },
            "invalid scatter parameter",
        ),
        (
            C::UnsupportedOperator,
            "`<=`, `>=` and `%` are not operators",
        ),
        (
            C::InvalidTexture { text: None },
            "image not decodable or of the wrong proportions",
        ),
        (C::LayerImageUndecodable, "layer image cannot be decoded"),
        (
            C::LayerImageFormat,
            "layer image has an unsupported channel count",
        ),
        (
            C::CapabilityHint { text: None },
            "takes effect only with a capability the pack does not declare",
        ),
        (
            C::VibrantVisualsDisabledByPack,
            "a pack in the stack disables Vibrant Visuals",
        ),
        (C::UnsupportedAudioFormat, "audio format not supported"),
        (C::FontFileMissing { text: None }, "font file not found"),
        (C::FontNameOverridden, "font replaced by a later entry"),
        (
            C::MixedFontPage,
            "font page files come from different packs",
        ),
        (
            C::AtomicFilterLightingIgnored,
            "Atomic Entity - filter_lighting is not supported.",
        ),
        (
            C::AtomicUnsupportedRenderController,
            "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).",
        ),
        (
            C::DuplicateIdentifier { text: None },
            "duplicate identifier",
        ),
        (C::DuplicateBone, "two bones collide after lower-casing"),
        (
            C::AtomicClientEntityShadowed,
            "atomic client entity superseded by a higher-priority pack",
        ),
        (
            C::AnimationControllerShadowed,
            "animation controller superseded by a higher-priority pack",
        ),
        (C::ShadowedByBuiltin, "replaced by a built-in definition"),
        (C::ReferenceCycle { text: None }, "references form a cycle"),
        (
            C::ClientBiomeShadowsLowerPacks,
            "invalid client biome still shadows lower packs",
        ),
        (
            C::ClientBiomeOverriddenByBiomesClient,
            "biomes_client.json overrides this client biome",
        ),
        (
            C::VanillaBiomesClientSkipped,
            "base-game biomes_client.json not loaded for this world version",
        ),
        (C::LegacyFogIgnored, "legacy fog entry not used"),
        (
            C::BiomeFormatHint {
                version: None,
                surface_components: false,
            },
            "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher",
        ),
        (C::NoEffect { text: None }, "member has no effect"),
        (
            C::SynthesizedComponent,
            "component inserted by an upgrade step",
        ),
        (
            C::IdentifierNamespaced,
            "identifier prefixed with minecraft: by an upgrade step",
        ),
        (
            C::UpgradeTableMissedBareIdentifier,
            "upgrade table missed an identifier without namespace",
        ),
        (
            C::ItemBindingOverlap,
            "two Always item bindings target one item",
        ),
        (
            C::RenderControllerFallback,
            "render controller falls back to the lower-priority definition",
        ),
        (
            C::CaseFoldingDisabled {
                line: Some(3),
                unresolvable: false,
            },
            "an apostrophe at line 3 disabled case folding",
        ),
        (C::PoiBlockClaimedTwice, "block listed by two POI types"),
        (C::PoiNoBlocks, "POI type with no blocks"),
        (
            C::SoundDefinitionsCleared,
            "sound_definitions.json clears the lower packs' definitions",
        ),
        (
            C::SoundsJsonTableCleared,
            "absent sounds.json table clears the lower packs' entries",
        ),
        (
            C::SoundsClearedByLegacyOverlay,
            "legacy overlay clears the definition's sounds",
        ),
        (
            C::DistanceResetByLegacyOverlay,
            "legacy overlay resets max_distance",
        ),
        (C::UnusedProperty, "property not read by this control type"),
        (C::NoUiDefinitions, "no UI definition files"),
        (
            C::SettingTypeMismatch,
            "value does not match the setting's type",
        ),
        (
            C::SettingUnknownOption,
            "value names no option of the setting",
        ),
        (C::SettingUnknownName, "no setting of this name"),
        (C::SettingClamped, "slider value clamped to its range"),
        (C::LegacySettingIgnored, "legacy setting element dropped"),
        (
            C::PackSettingChangeIgnored,
            "settings change ignored on a dedicated server",
        ),
        (
            C::PathRejected {
                entry: "a/../b.json".into(),
            },
            "entry 'a/../b.json' skipped: path rejected",
        ),
        (C::FileNotLoaded, "file indexed but not loaded"),
        (C::InternalError, "internal error: a loader job panicked"),
        (C::UnsupportedFileFormat, "unsupported file format"),
        (
            C::DuplicateEntry,
            "duplicate archive entry; the last one is kept",
        ),
        (
            C::MissingDependency {
                pack: pack(),
                dependency: dependency(),
            },
            "pack 00000000-0000-0000-0000-000000000001 1.0.0 depends on 00000000-0000-0000-0000-000000000002 2.1.0, which is not available; the pack is hidden",
        ),
        (
            C::DependencyAutoAdded {
                pack: pack(),
                dependency: dependency(),
            },
            "resource pack 00000000-0000-0000-0000-000000000002 2.1.0 added to the resource stack as a dependency of 00000000-0000-0000-0000-000000000001 1.0.0",
        ),
        (
            C::PackNotFound { pack: pack() },
            "pack 00000000-0000-0000-0000-000000000001 1.0.0 not found",
        ),
        (
            C::MultipleScriptModules {
                plugin: "demo".into(),
            },
            "Plugin [demo] - skipped, only one script module allowed per pack.",
        ),
        (
            C::ScriptModulePermission { text: None },
            "native module not permitted",
        ),
        (
            C::ScriptModuleVersionUnavailable {
                plugin: "demo".into(),
                module: "@minecraft/server".into(),
                requested: "9.0.0".into(),
                dependency: "demo".into(),
                current: None,
                available: vec!["1.0.0".into(), "2.0.0".into()],
            },
            "Plugin [demo] - module [@minecraft/server] requested invalid version [9.0.0] of [demo].\nAvailable versions:\n1.0.0\n2.0.0",
        ),
        (
            C::ScriptModuleVersionConflict {
                plugin: "demo".into(),
                module: "@minecraft/server".into(),
                requests: vec![("a".into(), "1.0.0".into()), ("b".into(), "2.0.0".into())],
            },
            "Plugin [demo] - version conflict for module [@minecraft/server].\n [a] requested by [1.0.0],\n [b] requested by [2.0.0]",
        ),
        (
            C::ScriptModulePromoted {
                plugin: "demo".into(),
                module: "@minecraft/server".into(),
                from: "1.0.0".into(),
                to: "1.1.0".into(),
                requested_by: "lib".into(),
            },
            "Plugin [demo] - promoted [@minecraft/server] from [1.0.0] to [1.1.0] requested by [lib].",
        ),
        (C::ScriptAlphaDependency, "alpha script API dependency"),
        (C::ScriptEntryOutsideRoot, "entry lacks the scripts/ prefix"),
        (
            C::ScriptSyntaxError { text: None },
            "syntax error in a script file",
        ),
        (
            C::ScriptImportNotFound {
                specifier: "./a.js".into(),
                hint: None,
            },
            "ReferenceError: Import [./a.js] not found.",
        ),
        (
            C::ScriptImportCrashesEngine,
            "this import specifier terminates the server",
        ),
        (
            C::ScriptModuleUnrecognized {
                module: "@minecraft/nope".into(),
            },
            "Module [@minecraft/nope] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?",
        ),
        (
            C::ScriptModuleRestricted {
                module: "@minecraft/server-net".into(),
                referrer: "main.js".into(),
            },
            "Import failed, can not load restricted module '@minecraft/server-net' from 'main.js'.",
        ),
        (C::ScriptImportNotChecked, "dynamic import not checked"),
        (C::ScriptFileNotLoadable, "script file can never be loaded"),
        (
            C::ScriptEvalWithoutCapability,
            "eval without the script_eval capability",
        ),
        (C::ScriptImportMetaEmpty, "import.meta has no members"),
        (C::ScriptsNotChecked, "script imports not checked"),
        (
            C::ScriptSyntaxNotChecked { files: 3 },
            "syntax of 3 script files not checked: nested too deeply for the parser",
        ),
        (
            C::PackConfigShadowsDefault,
            "pack config folder hides config/default",
        ),
        (
            C::UnmatchedConfigFolder {
                folder: "abc".into(),
                expected: None,
            },
            "config folder 'abc' matches no script module of the stack",
        ),
        (
            C::NetConfigRejected {
                module: "demo".into(),
                errors: "bad host".into(),
            },
            "Failed to load config for demo due to error(s): bad host",
        ),
        (
            C::InvalidProperty { text: None },
            "invalid manifest property",
        ),
        (
            C::MalformedPropertyVersion { text: None },
            "malformed manifest version",
        ),
        (
            C::WrongTypeProperty { text: None },
            "manifest property of the wrong type",
        ),
        (
            C::MissingProperty { text: None },
            "missing manifest property",
        ),
        (C::EmptyProperty, "empty manifest property"),
        (C::MalformedPropertyUuid, "malformed manifest UUID"),
        (C::DuplicateUuid, "duplicate UUID in the manifest"),
        (C::MissingModules, "the manifest has no modules"),
        (
            C::MultipleModules,
            "the manifest's modules declare different pack types",
        ),
        (
            C::InvalidCapability {
                name: "flying".into(),
            },
            "The pack manifest contains an invalid capability: flying",
        ),
        (C::IgnoredProperty, "manifest property ignored"),
        (
            C::VersionTooHigh {
                version: "3.0.0".into(),
                limit: "2.0.0".into(),
            },
            "version 3.0.0 is above 2.0.0",
        ),
        (
            C::VersionTooLow {
                version: "0.0.1".into(),
                limit: "1.0.0".into(),
            },
            "version 0.0.1 is below 1.0.0",
        ),
        (
            C::MinEngineVersionCapFormatVersion1,
            "min_engine_version of a format 1 manifest is capped to 1.12.0",
        ),
        (
            C::from(SAMPLE_SPECIFIC.with(["x"])),
            "a sample code of the test about x",
        ),
    ]
}

/// The specific code of the table's `Specific` row: a code of the toolkit's own, rendered
/// by its line.
static SAMPLE_SPECIFIC: CodeDef<1> = CodeDef::new(
    "test/sample",
    Kind::Lint,
    Class::Lint,
    "a sample code of the test about {}",
);

/// Every variant has a row in the table of the crate's own texts, in declaration order, once.
#[test]
fn every_variant_has_a_row_of_the_crate_text() {
    let rows: Vec<&str> = crate_text_of_every_code()
        .iter()
        .map(|(code, _)| code.variant_name())
        .collect();
    let missing: Vec<_> = DiagnosticCode::NAMES
        .iter()
        .filter(|name| !rows.contains(name))
        .collect();
    assert!(missing.is_empty(), "variants without a row: {missing:?}");
    assert_eq!(rows, DiagnosticCode::NAMES, "one row per variant, in order");
}

/// Every row renders exactly its line: one line, never empty.
#[test]
fn every_code_without_vanilla_text_renders_the_crate_line() {
    let wrong: Vec<(&str, String, &str)> = crate_text_of_every_code()
        .into_iter()
        .filter(|(code, expected)| code.to_string() != *expected)
        .map(|(code, expected)| (code.name(), code.to_string(), expected))
        .collect();
    assert!(wrong.is_empty(), "(code, rendered, expected): {wrong:#?}");
    for (code, text) in crate_text_of_every_code() {
        assert!(!text.is_empty(), "{code:?} renders nothing");
    }
}

/// The two document-size texts, byte for byte against the game's wording.
#[test]
fn the_json_document_size_texts() {
    use bedrock_addon_core::diagnostic::VanillaText;
    assert_eq!(
        VanillaText::DocumentTooLarge { max: 134_217_728 }.to_string(),
        "Minecraft Document JSON exceeds the maximum size of 134217728 bytes"
    );
    assert_eq!(
        VanillaText::DocumentNearlyTooLarge {
            loaded: 100_663_296,
            max: 134_217_728
        }
        .to_string(),
        "JSON document is approaching the maximum allowed size: 100663296 bytes loaded, 134217728 bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected."
    );
}
