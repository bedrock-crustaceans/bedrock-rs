//! The lines of the core's definitions: each definition with sample arguments renders the line
//! the legacy code it replaced rendered, byte for byte, and gives the game's sentence where the
//! legacy code gave one. The lines were taken from the legacy codes when the enum was deleted.
//! This is the one test in the crate family that matches on diagnostic text.

use bedrock_addon_core::diagnostic::{Code, codes};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

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

/// The text of `MolangNotChecked` says what is true of this build. Without the `molang`
/// feature nothing can check; with it, the finding is raised only where no `MolangCheck` was
/// given (`LoadOptions::molang` is `None`, the default).
#[cfg(not(feature = "molang"))]
const MOLANG_NOT_CHECKED: &str = "3 Molang field(s) not checked: the `molang` feature is off";
/// See the other half.
#[cfg(feature = "molang")]
const MOLANG_NOT_CHECKED: &str =
    "3 Molang field(s) not checked: no Molang check was given (`LoadOptions::molang` is `None`)";

/// One row per definition of `diagnostic::codes`, in its order: the code with sample arguments,
/// its line, and its game's sentence.
#[expect(
    clippy::too_many_lines,
    reason = "a table with one row per definition; there is no seam to split at"
)]
fn core_definitions() -> Vec<(Code, &'static str, Option<&'static str>)> {
    vec![
        (
            Code::from(&codes::UNSUPPORTED_VERSION),
            "unsupported format_version",
            None,
        ),
        (
            codes::UNSUPPORTED_VERSION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::BETA_FEATURES_NOT_ENABLED),
            "beta document without the Beta APIs toggle",
            None,
        ),
        (
            codes::BETA_FEATURES_NOT_ENABLED_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::PRERELEASE_FORMAT_VERSION.with(["1.21.0-beta"]),
            "format_version 1.21.0-beta has a pre-release tag and selects the schema below its release",
            None,
        ),
        (
            Code::from(&codes::MISSING_TOGGLES),
            "needs a toggle that is not enabled",
            None,
        ),
        (
            codes::MISSING_TOGGLES_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::DEPRECATED_MEMBER),
            "deprecated member",
            None,
        ),
        (
            codes::DEPRECATED_MEMBER_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNRECOGNIZED_FIELD),
            "unrecognized field",
            None,
        ),
        (
            codes::UNRECOGNIZED_FIELD_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNRECOGNIZED_COMPONENT),
            "unrecognized component",
            None,
        ),
        (
            codes::UNRECOGNIZED_COMPONENT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (Code::from(&codes::INVALID_VALUE), "invalid value", None),
        (
            codes::INVALID_VALUE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MISSING_REQUIRED_FIELD),
            "missing required field",
            None,
        ),
        (
            codes::MISSING_REQUIRED_FIELD_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::CLAMPED_VALUE),
            "value clamped into its range",
            None,
        ),
        (
            codes::CLAMPED_VALUE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::DROPPED_COMPONENT),
            "component dropped",
            None,
        ),
        (
            codes::DROPPED_COMPONENT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (Code::from(&codes::DROPPED_FIELD), "field dropped", None),
        (
            codes::DROPPED_FIELD_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MEMBER_NOT_KEPT),
            "member read and not kept",
            None,
        ),
        (
            codes::MEMBER_NOT_KEPT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::VALUE_REWRITTEN),
            "value held in another form than it is written in",
            None,
        ),
        (
            codes::VALUE_REWRITTEN_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::DUPLICATE_COMPONENT_VERSION.with([
                "minecraft:a",
                "minecraft:a.v2",
                "minecraft:a.v2",
                "minecraft:a",
            ]),
            "Two versions of the same component found: minecraft:a and minecraft:a.v2. Please remove minecraft:a.v2, or upgrade your format version and remove minecraft:a.",
            Some(
                "Two versions of the same component found: minecraft:a and minecraft:a.v2. Please remove minecraft:a.v2, or upgrade your format version and remove minecraft:a.",
            ),
        ),
        (
            Code::from(&codes::UNKNOWN_SOUND),
            "unknown sound event",
            None,
        ),
        (
            codes::UNKNOWN_SOUND_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::GENERIC_GOAL_SUFFIX_MISSING),
            "generic goal name lacks its required suffix",
            None,
        ),
        (Code::from(&codes::MALFORMED_INPUT), "malformed input", None),
        (
            codes::SYNTAX.with(["unexpected character '}'"]),
            "unexpected character '}'",
            None,
        ),
        (
            codes::MALFORMED_INPUT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::FORMAT_VERSION_ABOVE_ENGINE.with(["1.99.0", "1.26.60"]),
            "format_version 1.99.0 is above the engine version 1.26.60; parsed as the newest schema",
            None,
        ),
        (
            Code::from(&codes::CLIENT_ENTITY_SHADOWED),
            "client entity superseded by another definition of its identifier",
            None,
        ),
        (
            Code::from(&codes::CLIENT_ENTITY_SHADOWED_OLDER),
            "client entity ignored: its min_engine_version is below the loaded definition's",
            None,
        ),
        (
            Code::from(&codes::CLIENT_ENTITY_SHADOWED_ABOVE_STACK),
            "client entity ignored: its min_engine_version is above the stack's",
            None,
        ),
        (
            Code::from(&codes::ATTACHABLE_SHADOWED),
            "attachable superseded by another definition of its identifier",
            None,
        ),
        (
            Code::from(&codes::ATTACHABLE_SHADOWED_OLDER),
            "attachable ignored: its min_engine_version is below the loaded definition's",
            None,
        ),
        (
            Code::from(&codes::ATTACHABLE_SHADOWED_ABOVE_STACK),
            "attachable ignored: its min_engine_version is above the stack's",
            None,
        ),
        (
            Code::from(&codes::ITEM_BINDING_SHADOWED),
            "item binding superseded by a higher-priority pack",
            None,
        ),
        (
            Code::from(&codes::RENDER_CONTROLLER_SHADOWED),
            "render controller superseded by a higher-priority pack",
            None,
        ),
        (
            Code::from(&codes::IDENTIFIER_COLLISION),
            "two definitions share one identifier",
            None,
        ),
        (
            codes::IDENTIFIER_COLLISION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNRESOLVED_REFERENCE),
            "unresolved reference",
            None,
        ),
        (
            codes::UNRESOLVED_REFERENCE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::INVALID_REFERENCE),
            "reference resolves to something this use cannot accept",
            None,
        ),
        (
            codes::INVALID_REFERENCE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::TEXTURE_NOT_FOUND),
            "texture not found",
            None,
        ),
        (
            codes::TEXTURE_NOT_FOUND_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::SPLASHES_REPLACED),
            "splashes replace the lower packs' splashes",
            None,
        ),
        (Code::from(&codes::NO_SPLASHES), "no splash texts", None),
        (
            Code::from(&codes::UNRESOLVED_LANG_KEY),
            "localisation key not defined",
            None,
        ),
        (Code::from(&codes::IGNORED_FILE), "file ignored", None),
        (
            codes::IGNORED_FILE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::PACK_ICON_MISSING),
            "pack_icon.png missing",
            None,
        ),
        (
            Code::from(&codes::PACK_ICON_SHAPE),
            "pack_icon.png is not a square image",
            None,
        ),
        (
            Code::from(&codes::GLYPH_SHEET_SHAPE),
            "glyph sheet is not a 16 x 16 grid",
            None,
        ),
        (
            Code::from(&codes::SKIN_PACK_NAME_MISMATCH),
            "skin pack name does not match its localisation",
            None,
        ),
        (
            Code::from(&codes::SKIN_PACK_CREATOR_RULE),
            "skin pack breaks a creator rule",
            None,
        ),
        (Code::from(&codes::IGNORED_MEMBER), "member ignored", None),
        (
            codes::IGNORED_MEMBER_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNMATCHED_CONDITION),
            "condition matched no element",
            None,
        ),
        (
            Code::from(&codes::WHERE_ARRAY_NEVER_MATCHES),
            "an array-valued `where` never matches",
            None,
        ),
        (
            Code::from(&codes::REBASED_OVERRIDE),
            "overlay `name@base` key shadows a base name",
            None,
        ),
        (
            Code::from(&codes::SHADOWED_DEFINITION),
            "definition shadowed by a higher-priority source",
            None,
        ),
        (
            codes::SHADOWED_DEFINITION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::READ_ERROR.with([std::io::ErrorKind::NotFound.to_string()]),
            "cannot read the file (entity not found)",
            None,
        ),
        (
            codes::READ_ERROR_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MISSING_PAYLOAD),
            "the document has no payload member",
            None,
        ),
        (
            codes::MISSING_PAYLOAD_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::NAMESPACE_RULE),
            "namespace rule failed",
            None,
        ),
        (
            codes::NAMESPACE_RULE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (Code::from(&codes::INVALID_TYPE), "wrong JSON type", None),
        (
            codes::INVALID_TYPE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::SKIPPED_CACHE_DOCUMENT),
            "cache document not parsed",
            None,
        ),
        (
            Code::from(&codes::MISSING_FORMAT_VERSION),
            "no format_version",
            None,
        ),
        (
            codes::MISSING_FORMAT_VERSION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MALFORMED_FORMAT_VERSION),
            "Malformed format version",
            None,
        ),
        (
            codes::MALFORMED_FORMAT_VERSION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (Code::from(&codes::EMPTY_DOCUMENT), "empty document", None),
        (
            codes::EMPTY_DOCUMENT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNUSUAL_FORMAT_VERSION),
            "format_version between two known versions; parsed by the lower one",
            None,
        ),
        (
            Code::from(&codes::VANILLA_REJECTS_VERSION),
            "vanilla has no parser for this format_version",
            None,
        ),
        (
            Code::from(&codes::FORMAT_VERSION_ARRAY),
            "format_version given as an array",
            None,
        ),
        (
            Code::from(&codes::PAYLOAD_KEY_MISMATCH),
            "payload member is not the family's payload key",
            None,
        ),
        (
            codes::UNKNOWN_PAYLOAD_KEY.with(["minecraft:x"]),
            "Could not find registered feature type that matches name 'minecraft:x'",
            Some("Could not find registered feature type that matches name 'minecraft:x'"),
        ),
        (
            Code::from(&codes::IGNORED_PAYLOAD),
            "second payload ignored",
            None,
        ),
        (
            Code::from(&codes::DOCUMENT_REJECTED),
            "document failed schema validation",
            None,
        ),
        (
            codes::DOCUMENT_REJECTED_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MISSING_IDENTIFIER),
            "no usable identifier",
            None,
        ),
        (
            codes::MISSING_IDENTIFIER_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::SOUND_DEFINITIONS_WRAPPER_WITHOUT_VERSION),
            "`sound_definitions` wrapper without a format_version of at least 1.14.0",
            None,
        ),
        (
            Code::from(&codes::MISSING_MIN_ENGINE_VERSION),
            "the pack has no min_engine_version",
            None,
        ),
        (
            codes::FIELD_SINCE_ENGINE.with(["1.20.0"]),
            "field needs engine version 1.20.0 or newer",
            None,
        ),
        (
            codes::FIELD_UNTIL_ENGINE.with(["1.20.0"]),
            "field is gone from this engine; engine version 1.20.0 was the last known to have it",
            None,
        ),
        (
            Code::from(&codes::UNSUPPORTED_VALUE_FORM),
            "value form not supported by the configured engine",
            None,
        ),
        (
            Code::from(&codes::INVALID_ENUM_VALUE),
            "invalid enum value",
            None,
        ),
        (
            codes::INVALID_ENUM_VALUE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (codes::INVALID_MOLANG.with(["q.foo +"]), "q.foo +", None),
        (codes::MOLANG_LINT.with(["q.foo"]), "q.foo", None),
        (
            codes::MOLANG_NOT_CHECKED.with(["3"]),
            MOLANG_NOT_CHECKED,
            None,
        ),
        (
            Code::from(&codes::KEYFRAME_OUT_OF_DOMAIN),
            "keyframe key outside [0, 1]; dropped",
            None,
        ),
        (
            Code::from(&codes::LOSSY_CONVERSION),
            "value loses precision when written",
            None,
        ),
        (Code::from(&codes::LITERAL_IGNORED), "literal ignored", None),
        (
            Code::from(&codes::NON_CANONICAL_MEMBER_CASE),
            "member name matched ignoring case",
            None,
        ),
        (
            Code::from(&codes::CASE_FOLDED_TRANSITION),
            "transition expression lower-cased",
            None,
        ),
        (
            Code::from(&codes::UNORDERED_REMAP_CURVE),
            "remap_curve keys out of numeric order",
            None,
        ),
        (
            Code::from(&codes::IMPLICIT_INITIAL_STATE),
            "no initial_state and no default state; the first state is used",
            None,
        ),
        (
            Code::from(&codes::UNRECOGNIZED_EVENT_STRING),
            "event is neither a command, an entity event nor Molang",
            None,
        ),
        (
            Code::from(&codes::INVALID_NODE),
            "node is not an object with exactly one member",
            None,
        ),
        (
            Code::from(&codes::INVALID_ROOT),
            "minecraft:root is not an object with exactly one member",
            None,
        ),
        (
            codes::UNKNOWN_NODE.with(["bt:x"]),
            "Unknown behavior bt:x",
            Some("Unknown behavior bt:x"),
        ),
        (
            codes::LEGACY_ITEM_FORMAT.with(["no format_version"]),
            "item parsed with the legacy schema: no format_version",
            None,
        ),
        (
            Code::from(&codes::PERSONA_PIECE_PACK),
            "persona-piece pack in the resource-pack directory",
            None,
        ),
        (
            Code::from(&codes::LANG_KEY_TRAILING_SPACE),
            "key ends, or value starts, with a space",
            None,
        ),
        (
            Code::from(&codes::SCATTER_PARAM_VALUE),
            "invalid scatter parameter",
            None,
        ),
        (
            codes::SCATTER_PARAM_VALUE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::UNSUPPORTED_OPERATOR),
            "`<=`, `>=` and `%` are not operators",
            None,
        ),
        (
            Code::from(&codes::INVALID_TEXTURE),
            "image not decodable or of the wrong proportions",
            None,
        ),
        (
            codes::INVALID_TEXTURE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::LAYER_IMAGE_UNDECODABLE),
            "layer image cannot be decoded",
            None,
        ),
        (
            Code::from(&codes::LAYER_IMAGE_FORMAT),
            "layer image has an unsupported channel count",
            None,
        ),
        (
            Code::from(&codes::CAPABILITY_HINT),
            "takes effect only with a capability the pack does not declare",
            None,
        ),
        (
            codes::CAPABILITY_HINT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::VIBRANT_VISUALS_DISABLED_BY_PACK),
            "a pack in the stack disables Vibrant Visuals",
            None,
        ),
        (
            Code::from(&codes::FONT_FILE_MISSING),
            "font file not found",
            None,
        ),
        (
            codes::FONT_FILE_MISSING_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::FONT_NAME_OVERRIDDEN),
            "font replaced by a later entry",
            None,
        ),
        (
            Code::from(&codes::MIXED_FONT_PAGE),
            "font page files come from different packs",
            None,
        ),
        (
            Code::from(&codes::ATOMIC_FILTER_LIGHTING_IGNORED),
            "Atomic Entity - filter_lighting is not supported.",
            Some("Atomic Entity - filter_lighting is not supported."),
        ),
        (
            Code::from(&codes::ATOMIC_UNSUPPORTED_RENDER_CONTROLLER),
            "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).",
            Some(
                "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).",
            ),
        ),
        (
            Code::from(&codes::DUPLICATE_IDENTIFIER),
            "duplicate identifier",
            None,
        ),
        (
            codes::DUPLICATE_IDENTIFIER_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::DUPLICATE_BONE),
            "two bones collide after lower-casing",
            None,
        ),
        (
            Code::from(&codes::ATOMIC_CLIENT_ENTITY_SHADOWED),
            "atomic client entity superseded by a higher-priority pack",
            None,
        ),
        (
            Code::from(&codes::ANIMATION_CONTROLLER_SHADOWED),
            "animation controller superseded by a higher-priority pack",
            None,
        ),
        (
            Code::from(&codes::SHADOWED_BY_BUILTIN),
            "replaced by a built-in definition",
            None,
        ),
        (
            Code::from(&codes::REFERENCE_CYCLE),
            "references form a cycle",
            None,
        ),
        (
            codes::REFERENCE_CYCLE_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS),
            "invalid client biome still shadows lower packs",
            None,
        ),
        (
            Code::from(&codes::CLIENT_BIOME_OVERRIDDEN_BY_BIOMES_CLIENT),
            "biomes_client.json overrides this client biome",
            None,
        ),
        (
            Code::from(&codes::VANILLA_BIOMES_CLIENT_SKIPPED),
            "base-game biomes_client.json not loaded for this world version",
            None,
        ),
        (
            Code::from(&codes::LEGACY_FOG_IGNORED),
            "legacy fog entry not used",
            None,
        ),
        (
            Code::from(&codes::BIOME_FORMAT_HINT_LEGACY),
            "biome files in the 1.12 format are no longer supported; use the minecraft:biome document format with format_version 1.21.110 or higher",
            None,
        ),
        (
            codes::BIOME_FORMAT_HINT_EXPERIMENTAL.with(["1.20.0"]),
            "biome format 1.20.0 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation)",
            None,
        ),
        (
            codes::BIOME_FORMAT_HINT_EXPERIMENTAL_SURFACE.with(["1.20.0"]),
            "biome format 1.20.0 was experimental and is not loaded; set format_version to 1.21.110 (move climate ash/white_ash/red_spores/blue_spores to the client biome's minecraft:precipitation and replace minecraft:surface_parameters / minecraft:*_surface with minecraft:surface_builder)",
            None,
        ),
        (Code::from(&codes::NO_EFFECT), "member has no effect", None),
        (
            codes::NO_EFFECT_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::SYNTHESIZED_COMPONENT),
            "component inserted by an upgrade step",
            None,
        ),
        (
            Code::from(&codes::IDENTIFIER_NAMESPACED),
            "identifier prefixed with minecraft: by an upgrade step",
            None,
        ),
        (
            Code::from(&codes::UPGRADE_TABLE_MISSED_BARE_IDENTIFIER),
            "upgrade table missed an identifier without namespace",
            None,
        ),
        (
            Code::from(&codes::ITEM_BINDING_OVERLAP),
            "two Always item bindings target one item",
            None,
        ),
        (
            Code::from(&codes::RENDER_CONTROLLER_FALLBACK),
            "render controller falls back to the lower-priority definition",
            None,
        ),
        (
            codes::CASE_FOLDING_DISABLED_AT_LINE.with(["3"]),
            "an apostrophe at line 3 disabled case folding",
            None,
        ),
        (
            codes::CASE_FOLDING_DISABLED_AT_LINE_UNRESOLVABLE.with(["3"]),
            "an apostrophe at line 3 disabled case folding; the game resolves no reference to this name",
            None,
        ),
        (
            Code::from(&codes::CASE_FOLDING_DISABLED),
            "a capital letter the game's case folding does not reach",
            None,
        ),
        (
            Code::from(&codes::CASE_FOLDING_DISABLED_UNRESOLVABLE),
            "a capital letter the game's case folding does not reach; the game resolves no reference to this name",
            None,
        ),
        (
            Code::from(&codes::POI_BLOCK_CLAIMED_TWICE),
            "block listed by two POI types",
            None,
        ),
        (
            Code::from(&codes::POI_NO_BLOCKS),
            "POI type with no blocks",
            None,
        ),
        (
            Code::from(&codes::SOUND_DEFINITIONS_CLEARED),
            "sound_definitions.json clears the lower packs' definitions",
            None,
        ),
        (
            Code::from(&codes::SOUNDS_JSON_TABLE_CLEARED),
            "absent sounds.json table clears the lower packs' entries",
            None,
        ),
        (
            Code::from(&codes::SOUNDS_CLEARED_BY_LEGACY_OVERLAY),
            "legacy overlay clears the definition's sounds",
            None,
        ),
        (
            Code::from(&codes::DISTANCE_RESET_BY_LEGACY_OVERLAY),
            "legacy overlay resets max_distance",
            None,
        ),
        (
            Code::from(&codes::UNUSED_PROPERTY),
            "property not read by this control type",
            None,
        ),
        (
            Code::from(&codes::NO_UI_DEFINITIONS),
            "no UI definition files",
            None,
        ),
        (
            Code::from(&codes::SETTING_TYPE_MISMATCH),
            "value does not match the setting's type",
            None,
        ),
        (
            Code::from(&codes::SETTING_UNKNOWN_OPTION),
            "value names no option of the setting",
            None,
        ),
        (
            Code::from(&codes::SETTING_UNKNOWN_NAME),
            "no setting of this name",
            None,
        ),
        (
            Code::from(&codes::SETTING_CLAMPED),
            "slider value clamped to its range",
            None,
        ),
        (
            Code::from(&codes::LEGACY_SETTING_IGNORED),
            "legacy setting element dropped",
            None,
        ),
        (
            codes::PATH_REJECTED.with(["a\u{fffd}.json"]),
            "entry 'a�.json' skipped: path rejected",
            None,
        ),
        (
            Code::from(&codes::FILE_NOT_LOADED),
            "file indexed but not loaded",
            None,
        ),
        (
            Code::from(&codes::INTERNAL_ERROR),
            "internal error: a loader job panicked",
            None,
        ),
        (
            Code::from(&codes::UNSUPPORTED_FILE_FORMAT),
            "unsupported file format",
            None,
        ),
        (
            Code::from(&codes::DUPLICATE_ENTRY),
            "duplicate archive entry; the last one is kept",
            None,
        ),
        (
            codes::MISSING_DEPENDENCY.with([pack().to_string(), dependency().to_string()]),
            "pack 00000000-0000-0000-0000-000000000001 1.0.0 depends on 00000000-0000-0000-0000-000000000002 2.1.0, which is not available; the pack is hidden",
            None,
        ),
        (
            codes::DEPENDENCY_AUTO_ADDED.with([dependency().to_string(), pack().to_string()]),
            "resource pack 00000000-0000-0000-0000-000000000002 2.1.0 added to the resource stack as a dependency of 00000000-0000-0000-0000-000000000001 1.0.0",
            None,
        ),
        (
            codes::PACK_NOT_FOUND.with([pack().to_string()]),
            "pack 00000000-0000-0000-0000-000000000001 1.0.0 not found",
            None,
        ),
        (
            codes::MULTIPLE_SCRIPT_MODULES.with(["p"]),
            "Plugin [p] - skipped, only one script module allowed per pack.",
            Some("Plugin [p] - skipped, only one script module allowed per pack."),
        ),
        (
            Code::from(&codes::SCRIPT_MODULE_PERMISSION),
            "native module not permitted",
            None,
        ),
        (
            codes::SCRIPT_MODULE_PERMISSION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::SCRIPT_MODULE_VERSION_UNAVAILABLE.with([
                "p",
                "m",
                "9.9.9",
                "@minecraft/server",
                "\nCurrent [stable] version is [1.0.0].",
                "\n1.0.0\n1.1.0",
            ]),
            "Plugin [p] - module [m] requested invalid version [9.9.9] of [@minecraft/server].\nCurrent [stable] version is [1.0.0].\nAvailable versions:\n1.0.0\n1.1.0",
            Some(
                "Plugin [p] - module [m] requested invalid version [9.9.9] of [@minecraft/server].\nCurrent [stable] version is [1.0.0].\nAvailable versions:\n1.0.0\n1.1.0",
            ),
        ),
        (
            codes::SCRIPT_MODULE_VERSION_CONFLICT.with([
                "p",
                "m",
                "\n [1.0.0] requested by [a],\n [2.0.0] requested by [b]",
            ]),
            "Plugin [p] - version conflict for module [m].\n [1.0.0] requested by [a],\n [2.0.0] requested by [b]",
            Some(
                "Plugin [p] - version conflict for module [m].\n [1.0.0] requested by [a],\n [2.0.0] requested by [b]",
            ),
        ),
        (
            codes::SCRIPT_MODULE_PROMOTED.with(["p", "m", "1.0.0", "1.1.0", "q"]),
            "Plugin [p] - promoted [m] from [1.0.0] to [1.1.0] requested by [q].",
            Some("Plugin [p] - promoted [m] from [1.0.0] to [1.1.0] requested by [q]."),
        ),
        (
            Code::from(&codes::SCRIPT_ALPHA_DEPENDENCY),
            "alpha script API dependency",
            None,
        ),
        (
            Code::from(&codes::SCRIPT_ENTRY_OUTSIDE_ROOT),
            "entry lacks the scripts/ prefix",
            None,
        ),
        (
            Code::from(&codes::SCRIPT_SYNTAX_ERROR),
            "syntax error in a script file",
            None,
        ),
        (
            codes::SCRIPT_SYNTAX_ERROR_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            codes::SCRIPT_IMPORT_NOT_FOUND.with(["./a.js"]),
            "ReferenceError: Import [./a.js] not found.",
            Some("ReferenceError: Import [./a.js] not found."),
        ),
        (
            Code::from(&codes::SCRIPT_IMPORT_CRASHES_ENGINE),
            "this import specifier terminates the server",
            None,
        ),
        (
            codes::SCRIPT_MODULE_UNRECOGNIZED.with(["@x/y"]),
            "Module [@x/y] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?",
            Some(
                "Module [@x/y] is unrecognized. If this is a Native module, did you remember to add it to your manifest.json?",
            ),
        ),
        (
            codes::SCRIPT_MODULE_RESTRICTED.with(["@x/y", "main.js"]),
            "Import failed, can not load restricted module '@x/y' from 'main.js'.",
            Some("Import failed, can not load restricted module '@x/y' from 'main.js'."),
        ),
        (
            Code::from(&codes::SCRIPT_IMPORT_NOT_CHECKED),
            "dynamic import not checked",
            None,
        ),
        (
            Code::from(&codes::SCRIPT_FILE_NOT_LOADABLE),
            "script file can never be loaded",
            None,
        ),
        (
            Code::from(&codes::SCRIPT_EVAL_WITHOUT_CAPABILITY),
            "eval without the script_eval capability",
            None,
        ),
        (
            Code::from(&codes::SCRIPT_IMPORT_META_EMPTY),
            "import.meta has no members",
            None,
        ),
        (
            Code::from(&codes::SCRIPTS_NOT_CHECKED),
            "script imports not checked",
            None,
        ),
        (
            codes::SCRIPT_SYNTAX_NOT_CHECKED.with(["2"]),
            "syntax of 2 script files not checked: nested too deeply for the parser",
            None,
        ),
        (
            Code::from(&codes::PACK_CONFIG_SHADOWS_DEFAULT),
            "pack config folder hides config/default",
            None,
        ),
        (
            codes::UNMATCHED_CONFIG_FOLDER.with(["abc"]),
            "config folder 'abc' matches no script module of the stack",
            None,
        ),
        (
            codes::UNMATCHED_CONFIG_FOLDER_EXPECTED
                .with([String::from("abc"), Uuid::from_u128(3).to_string()]),
            "config folder 'abc' is never used; the script module's folder is '00000000-0000-0000-0000-000000000003'",
            None,
        ),
        (
            codes::NET_CONFIG_REJECTED.with(["@minecraft/server-net", "bad"]),
            "Failed to load config for @minecraft/server-net due to error(s): bad",
            Some("Failed to load config for @minecraft/server-net due to error(s): bad"),
        ),
        (
            Code::from(&codes::INVALID_PROPERTY),
            "invalid manifest property",
            None,
        ),
        (
            codes::INVALID_PROPERTY_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MALFORMED_PROPERTY_VERSION),
            "malformed manifest version",
            None,
        ),
        (
            codes::MALFORMED_PROPERTY_VERSION_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::WRONG_TYPE_PROPERTY),
            "manifest property of the wrong type",
            None,
        ),
        (
            codes::WRONG_TYPE_PROPERTY_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::MISSING_PROPERTY),
            "missing manifest property",
            None,
        ),
        (
            codes::MISSING_PROPERTY_WORDED.with(["a game line {}"]),
            "a game line {}",
            Some("a game line {}"),
        ),
        (
            Code::from(&codes::EMPTY_PROPERTY),
            "empty manifest property",
            None,
        ),
        (
            Code::from(&codes::MALFORMED_PROPERTY_UUID),
            "malformed manifest UUID",
            None,
        ),
        (
            Code::from(&codes::DUPLICATE_UUID),
            "duplicate UUID in the manifest",
            None,
        ),
        (
            Code::from(&codes::MISSING_MODULES),
            "the manifest has no modules",
            None,
        ),
        (
            Code::from(&codes::MULTIPLE_MODULES),
            "the manifest's modules declare different pack types",
            None,
        ),
        (
            codes::INVALID_CAPABILITY.with(["flying"]),
            "The pack manifest contains an invalid capability: flying",
            Some("The pack manifest contains an invalid capability: flying"),
        ),
        (
            Code::from(&codes::IGNORED_PROPERTY),
            "manifest property ignored",
            None,
        ),
        (
            codes::VERSION_TOO_HIGH.with(["3.0.0", "2.0.0"]),
            "version 3.0.0 is above 2.0.0",
            None,
        ),
        (
            codes::VERSION_TOO_LOW.with(["0.0.1", "1.0.0"]),
            "version 0.0.1 is below 1.0.0",
            None,
        ),
        (
            Code::from(&codes::MIN_ENGINE_VERSION_CAP_FORMAT_VERSION1),
            "min_engine_version of a format 1 manifest is capped to 1.12.0",
            None,
        ),
        (
            codes::DOCUMENT_JSON.with(["Number too big to be stored in double."]),
            "Failed to parse the Minecraft Document JSON: Number too big to be stored in double.",
            Some(
                "Failed to parse the Minecraft Document JSON: Number too big to be stored in double.",
            ),
        ),
        (
            codes::CHILD_NOT_VALID.with(["k"]),
            "child 'k' not valid here.",
            Some("child 'k' not valid here."),
        ),
        (
            codes::REQUIRED_CHILD.with(["k"]),
            "Required child k not found",
            Some("Required child k not found"),
        ),
        (
            codes::VALUE_NOT_ALLOWED.with(["false"]),
            "Value 'false' not allowed",
            Some("Value 'false' not allowed"),
        ),
        (
            codes::NOT_ALLOWED_REGEX.with(["v", "a, b"]),
            "Value 'v' not allowed.  Must match at least one of the following regex patterns: a, b",
            Some(
                "Value 'v' not allowed.  Must match at least one of the following regex patterns: a, b",
            ),
        ),
        (
            codes::NOT_ALLOWED_KEYS.with(["v", "a, b"]),
            "Value 'v' not allowed.  Must match at least one of the following keys: a, b",
            Some("Value 'v' not allowed.  Must match at least one of the following keys: a, b"),
        ),
        (
            codes::OUTSIDE_RANGE.with(["2.000000", "0.000000", "1.000000"]),
            "Value '2.000000' outside valid range [0.000000, 1.000000]",
            Some("Value '2.000000' outside valid range [0.000000, 1.000000]"),
        ),
        (
            codes::ARRAY_TOO_SMALL.with(["1", "3"]),
            "Array too small (1 < 3)",
            Some("Array too small (1 < 3)"),
        ),
        (
            codes::ARRAY_TOO_LARGE.with(["4", "3"]),
            "Array too large (4 > 3)",
            Some("Array too large (4 > 3)"),
        ),
        (
            codes::PARSE_ERROR.with(["m"]),
            "parse error:\nm",
            Some("parse error:\nm"),
        ),
        (
            Code::from(&codes::BEYOND_FLOAT_MEMBER),
            "the number is beyond the float that reads it; the member holds an infinity",
            Some("the number is beyond the float that reads it; the member holds an infinity"),
        ),
        (
            Code::from(&codes::NO_VALID_VERSION),
            "no valid version passed by the caller or found in the input",
            Some("no valid version passed by the caller or found in the input"),
        ),
        (
            Code::from(&codes::NO_FORMAT_VERSION_FIELD),
            "no format_version field was given",
            Some("no format_version field was given"),
        ),
        (
            Code::from(&codes::FORMAT_VERSION_REQUIRED),
            "missing required field",
            Some("missing required field"),
        ),
        (
            Code::from(&codes::VERSION_NOT_PROCESSABLE),
            "The file cannot be processed without a valid format_version",
            Some("The file cannot be processed without a valid format_version"),
        ),
        (
            Code::from(&codes::VERSION_NOT_READABLE),
            "document's version not found or not readable",
            Some("document's version not found or not readable"),
        ),
        (
            Code::from(&codes::VERSION_NOT_GIVEN_LEGACY_FIELDS),
            "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.",
            Some(
                "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.",
            ),
        ),
        (
            codes::NO_PARSER_FOR_VERSION.with(["1.99.0"]),
            "unsupported version: no parser available for version 1.99.0",
            Some("unsupported version: no parser available for version 1.99.0"),
        ),
        (
            codes::USE_FORMAT_VERSION.with(["component", "minecraft:x", "1.21.40"]),
            "To use component 'minecraft:x', use json format version 1.21.40 or higher",
            Some("To use component 'minecraft:x', use json format version 1.21.40 or higher"),
        ),
        (
            codes::TOGGLE_USE_FORMAT_VERSION.with(["component", "minecraft:x", "1.21.40"]),
            "To use component 'minecraft:x', use json format version 1.21.40 or higher",
            Some("To use component 'minecraft:x', use json format version 1.21.40 or higher"),
        ),
        (
            codes::TURN_ON_TOGGLE.with(["field", "minecraft:x.y", "Beta APIs"]),
            "To use field 'minecraft:x.y', turn on the 'Beta APIs' toggle in settings",
            Some("To use field 'minecraft:x.y', turn on the 'Beta APIs' toggle in settings"),
        ),
        (
            codes::TURN_ON_TOGGLE_BEFORE.with(["component", "minecraft:x", "1.21.40", "Beta APIs"]),
            "To use component 'minecraft:x' in json formats earlier than 1.21.40, turn on the 'Beta APIs' toggle in settings",
            Some(
                "To use component 'minecraft:x' in json formats earlier than 1.21.40, turn on the 'Beta APIs' toggle in settings",
            ),
        ),
        (
            Code::from(&codes::BETA_DOCUMENTS),
            "Turn on the Beta APIs toggle in settings to allow beta documents.",
            Some("Turn on the Beta APIs toggle in settings to allow beta documents."),
        ),
        (
            Code::from(&codes::BETA_FORMAT_VERSION),
            "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled.",
            Some(
                "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled.",
            ),
        ),
        (
            Code::from(&codes::DOCUMENT_MEMBER_COUNT),
            "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
            Some(
                "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
            ),
        ),
        (
            codes::DOCUMENT_TOO_LARGE.with(["1000"]),
            "Minecraft Document JSON exceeds the maximum size of 1000 bytes",
            Some("Minecraft Document JSON exceeds the maximum size of 1000 bytes"),
        ),
        (
            codes::DOCUMENT_NEARLY_TOO_LARGE.with(["900", "1000"]),
            "JSON document is approaching the maximum allowed size: 900 bytes loaded, 1000 bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected.",
            Some(
                "JSON document is approaching the maximum allowed size: 900 bytes loaded, 1000 bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected.",
            ),
        ),
        (
            codes::DEPRECATED_IN_FORMAT.with(["minecraft:foot_size", "1.16.100"]),
            "minecraft:foot_size has been deprecated and is not available in json format 1.16.100",
            Some(
                "minecraft:foot_size has been deprecated and is not available in json format 1.16.100",
            ),
        ),
        (
            codes::STACK_TOP_UNPARSABLE.with(["sounds.json"]),
            "Could not parse file at the top of the stack: sounds.json.",
            Some("Could not parse file at the top of the stack: sounds.json."),
        ),
        (
            codes::FILTER_TEST_NOT_FOUND.with(["is_purple"]),
            "Filter test not found -> is_purple",
            Some("Filter test not found -> is_purple"),
        ),
        (
            Code::from(&codes::MEMBER_NOT_IN_SCHEMA),
            "this member was found in the input, but is not present in the Schema",
            Some("this member was found in the input, but is not present in the Schema"),
        ),
        (
            Code::from(&codes::COMPONENT_NOT_IN_SCHEMA),
            "this component was found in the input, but is not present in the Schema",
            Some("this component was found in the input, but is not present in the Schema"),
        ),
        (
            Code::from(&codes::REQUIRED_FIELD_MISSING),
            "missing required field",
            Some("missing required field"),
        ),
        (
            Code::from(&codes::EXPECTED_OBJECT),
            "expected an object",
            Some("expected an object"),
        ),
        (
            codes::TOO_FEW_COMPONENTS.with(["2"]),
            "too few input components, expected at least 2",
            Some("too few input components, expected at least 2"),
        ),
        (
            Code::from(&codes::INVALID_ENUM_LINE),
            "invalid enum value",
            Some("invalid enum value"),
        ),
        (
            codes::ENUM_EXPECTS.with(["Mode", "'a', 'b'", "c"]),
            "This value is bound to 'Mode' and expects one of the following strings: 'a', 'b'. Found: 'c'",
            Some(
                "This value is bound to 'Mode' and expects one of the following strings: 'a', 'b'. Found: 'c'",
            ),
        ),
        (
            codes::ENUM_EXPECTS_STRING.with(["Mode"]),
            "This value is bound to 'Mode' and expects a \"named value\" (i.e. a string)",
            Some("This value is bound to 'Mode' and expects a \"named value\" (i.e. a string)"),
        ),
        (
            Code::from(&codes::INVALID_NUMERIC_VALUE),
            "invalid numeric value",
            Some("invalid numeric value"),
        ),
        (
            codes::OUTSIDE_ACCEPTED_RANGE.with(["[0, 1]"]),
            "the input is outside of the accepted range [0, 1]",
            Some("the input is outside of the accepted range [0, 1]"),
        ),
        (
            codes::NUMBER_BOUND.with([">= 0"]),
            "the input must be a number >= 0",
            Some("the input must be a number >= 0"),
        ),
        (
            codes::NOT_MULTIPLE_OF.with(["3", "2"]),
            "3 is not multiple of 2",
            Some("3 is not multiple of 2"),
        ),
        (
            codes::STRING_MISMATCH.with(["^a$"]),
            "expected string to match the \"^a$\" regex",
            Some("expected string to match the \"^a$\" regex"),
        ),
        (
            codes::ARRAY_AT_LEAST.with(["1"]),
            "expected array with at least 1 elements",
            Some("expected array with at least 1 elements"),
        ),
        (
            codes::OBJECT_AT_LEAST.with(["1"]),
            "expected object with at least 1 elements",
            Some("expected object with at least 1 elements"),
        ),
        (
            Code::from(&codes::NO_DUPLICATES),
            "No duplicates allowed",
            Some("No duplicates allowed"),
        ),
        (
            codes::KEY_NOT_ALLOWED.with(["x"]),
            "key x is not allowed",
            Some("key x is not allowed"),
        ),
        (
            codes::TOO_MANY_ELEMENTS.with(["4"]),
            "too many input elements, expected no more than 4",
            Some("too many input elements, expected no more than 4"),
        ),
        (
            codes::TOO_MANY_CHARACTERS.with(["8"]),
            "too many characters, expected no more than 8",
            Some("too many characters, expected no more than 8"),
        ),
        (
            Code::from(&codes::TOO_MANY_MAP_ELEMENTS),
            "too many input elements for an associative container",
            Some("too many input elements for an associative container"),
        ),
        (
            Code::from(&codes::PAYLOAD_MISSING),
            "The payload of the Minecraft Document is missing",
            Some("The payload of the Minecraft Document is missing"),
        ),
        (
            codes::PAYLOAD_UNDER_OTHER_KEY.with(["minecraft:a", "minecraft:b"]),
            "The payload key does not match the expected one. Expected: 'minecraft:a'. Actual: 'minecraft:b'.",
            Some(
                "The payload key does not match the expected one. Expected: 'minecraft:a'. Actual: 'minecraft:b'.",
            ),
        ),
        (
            codes::TOO_FEW_CHILDREN.with(["1", "2"]),
            "Node has too few children (1 < 2)",
            Some("Node has too few children (1 < 2)"),
        ),
        (
            codes::TOO_MANY_CHILDREN.with(["3", "2"]),
            "Node has too many children (3 > 2)",
            Some("Node has too many children (3 > 2)"),
        ),
        (
            codes::UNKNOWN_CHILD_OPTION.with(["string"]),
            "unknown child schema option type.  Allowed types: string",
            Some("unknown child schema option type.  Allowed types: string"),
        ),
        (
            codes::VERSION_BELOW_MIN.with(["1.0.0", "1.2.0"]),
            "Version 1.0.0 below min required version (1.2.0)",
            Some("Version 1.0.0 below min required version (1.2.0)"),
        ),
        (
            codes::MEMBER_DEPRECATED.with(["__DEP_fnctns"]),
            "member \"__DEP_fnctns\" is deprecated",
            Some("member \"__DEP_fnctns\" is deprecated"),
        ),
        (
            Code::from(&codes::ENUM_VALUE_DEPRECATED),
            "enum value is deprecated",
            Some("enum value is deprecated"),
        ),
        (
            codes::MANIFEST_INVALID_VALUE.with(["/modules/0/type"]),
            "Provided '/modules/0/type' element has an invalid value in pack manifest.",
            Some("Provided '/modules/0/type' element has an invalid value in pack manifest."),
        ),
        (
            codes::MANIFEST_NOT_SEMVER.with(["/dependencies/0/version"]),
            "Provided '/dependencies/0/version' element is not SemVer (semver.org) compliant in pack manifest.",
            Some(
                "Provided '/dependencies/0/version' element is not SemVer (semver.org) compliant in pack manifest.",
            ),
        ),
        (
            codes::MANIFEST_WRONG_TYPE.with(["/dependencies/0/version"]),
            "Required '/dependencies/0/version' element is the wrong type in pack manifest.",
            Some("Required '/dependencies/0/version' element is the wrong type in pack manifest."),
        ),
        (
            codes::DEPENDENCY_INVALID_VERSION.with(["@minecraft/server", "0.0.0"]),
            "dependency @minecraft/server invalid version: 0.0.0",
            Some("dependency @minecraft/server invalid version: 0.0.0"),
        ),
        (
            codes::DUPLICATE_PACK_SETTING_NAME.with(["volume"]),
            "duplicate pack setting name found: volume",
            Some("duplicate pack setting name found: volume"),
        ),
        (
            codes::SLIDER_MAX_NOT_ABOVE_MIN.with(["volume"]),
            "slider volume max must be greater than min",
            Some("slider volume max must be greater than min"),
        ),
        (
            codes::SLIDER_STEP_TOO_LARGE.with(["volume", "2"]),
            "slider volume step 2 must be <= to max - min",
            Some("slider volume step 2 must be <= to max - min"),
        ),
        (
            codes::SLIDER_DEFAULT_OUT_OF_RANGE.with(["volume", "7"]),
            "slider volume default 7 is out of range",
            Some("slider volume default 7 is out of range"),
        ),
        (
            codes::DROPDOWN_DEFAULT_NOT_IN_OPTIONS.with(["mode", "fast"]),
            "dropdown mode default value fast is not in options",
            Some("dropdown mode default value fast is not in options"),
        ),
        (
            codes::MULTISELECT_DEFAULT_NOT_IN_OPTIONS.with(["modes", "fast"]),
            "multiselect modes default value fast is not in options",
            Some("multiselect modes default value fast is not in options"),
        ),
        (
            codes::INVALID_VALUE_FORMATTED.with(["the input must be a number >= 0"]),
            "the input must be a number >= 0",
            Some("the input must be a number >= 0"),
        ),
    ]
}

/// Each definition renders its line byte for byte, and its game's sentence where it has one.
#[test]
fn every_core_definition_renders_its_line() {
    for (code, line, vanilla) in core_definitions() {
        let id = code.id();
        assert_eq!(code.to_string(), line, "{id}");
        assert!(!line.is_empty(), "{id}");
        assert_eq!(code.vanilla().as_deref(), vanilla, "{id}");
    }
}

/// The table has one row per definition of `codes::ALL`, in its order.
#[test]
fn every_core_definition_has_one_row() {
    let rows: Vec<&str> = core_definitions()
        .iter()
        .map(|(code, ..)| code.id())
        .collect();
    let all: Vec<&str> = codes::ALL.iter().map(|meta| meta.id()).collect();
    assert_eq!(rows, all);
}

/// Both builds by name: the sentence of the feature that is off is never printed by a
/// build that has it, and the other way round.
#[test]
fn molang_not_checked_names_the_cause_that_holds_in_this_build() {
    let text = codes::MOLANG_NOT_CHECKED.with(["3"]).to_string();
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

/// The two document-size texts, byte for byte against the game's wording.
#[test]
fn the_json_document_size_texts() {
    assert_eq!(
        codes::DOCUMENT_TOO_LARGE.with(["134217728"]).to_string(),
        "Minecraft Document JSON exceeds the maximum size of 134217728 bytes"
    );
    assert_eq!(
        codes::DOCUMENT_NEARLY_TOO_LARGE
            .with(["100663296", "134217728"])
            .to_string(),
        "JSON document is approaching the maximum allowed size: 100663296 bytes loaded, 134217728 bytes maximum. Consider reducing or splitting the document where possible, or raise this issue if further increases in size are expected."
    );
}
