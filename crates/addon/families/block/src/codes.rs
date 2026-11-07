//! The block family's codes: vanilla's content-log texts for block files, byte for byte (the game's
//! typos and missing spaces included), each a definition beside the rules that raise it; findings
//! of the crate's own raise the core's generic definitions. Each id is `block/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the legacy reader ----

/// The legacy reader finds no payload object, or no `description` object.
pub static MEMBER_MISSING: CodeDef<0> = CodeDef::new(
    "block/member_missing",
    Kind::WrongType,
    Class::Error,
    "the block document misses a required member",
)
.game(" -> minecraft:block:  missing required field");

/// The legacy reader's `components` is no object.
pub static COMPONENTS_OBJECT: CodeDef<0> = CodeDef::new(
    "block/components_object",
    Kind::WrongType,
    Class::Error,
    "components is no object",
)
.game(" -> minecraft:block:  the 'components' member should be an object");

/// The legacy reader's `permutations` is no array (the first of two lines).
pub static PERMUTATIONS_ARRAY: CodeDef<0> = CodeDef::new(
    "block/permutations_array",
    Kind::WrongType,
    Class::Error,
    "permutations is no array",
)
.game(" -> minecraft:block:  the 'permutations' member should be an array");

/// The second line of [`PERMUTATIONS_ARRAY`].
pub static PERMUTATIONS_ARRAY_OBJECT: CodeDef<0> = CodeDef::new(
    "block/permutations_array_object",
    Kind::WrongType,
    Class::Error,
    "permutations is no array",
)
.game("Permutations requires an array object");

/// A permutation that is no object (the second of two lines).
pub static PERMUTATIONS_OBJECTS: CodeDef<0> = CodeDef::new(
    "block/permutations_objects",
    Kind::WrongType,
    Class::Error,
    "a permutation is no object",
)
.game("Permutations requires an array of objects");

/// The legacy description's `states` is no object.
pub static STATES_OBJECT: CodeDef<0> = CodeDef::new(
    "block/states_object",
    Kind::WrongType,
    Class::Error,
    "description states is no object",
)
.game("Block description states must be an object");

/// A permutation that is no object (the first of two lines).
pub static PERMUTATION_OBJECT: CodeDef<0> = CodeDef::new(
    "block/permutation_object",
    Kind::WrongType,
    Class::Error,
    "a permutation is no object with components",
)
.game(" -> minecraft:block -> permutations:  every permutation should be an object containing a 'components' member object");

/// The line after [`STATES_OBJECT`]: takes the identifier. The trailing space is vanilla's.
pub static PARSE_BLOCK_FAILED: CodeDef<1> = CodeDef::new(
    "block/parse_block_failed",
    Kind::WrongType,
    Class::Error,
    "the block {} failed to parse",
)
.game("Failed to parse block '{}' ");

/// A component key written twice in one object: takes the key. A kept error.
pub static DUPLICATE_COMPONENT: CodeDef<1> = CodeDef::new(
    "block/duplicate_component",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "the component {} is there twice",
)
.game("Duplicate component {} found. Please remove one of the duplicates.");

/// A component the legacy document's schema has no child for: takes the key. Core's
/// [`CHILD_NOT_VALID`](bedrock_addon_core::diagnostic::codes::CHILD_NOT_VALID) is the same line
/// for a member.
pub static CHILD_NOT_VALID: CodeDef<1> = CodeDef::new(
    "block/child_not_valid",
    Kind::UnknownMember(Disposition::Kept),
    Class::Error,
    "the component {} has no legacy description",
)
.game("child '{}' not valid here.");

// ---- the upgrade ----

/// A component the upgrade cannot carry: takes the component, the first of three lines.
pub static INVALID_COMPONENT: CodeDef<1> = CodeDef::new(
    "block/invalid_component",
    Kind::InvalidValue,
    Class::Error,
    "the component {} is invalid",
)
.game(" -> minecraft:block -> components -> {}:  invalid component");

/// The second line of [`INVALID_COMPONENT`]: takes the target and the document's version.
pub static UPGRADE_FAILED: CodeDef<2> = CodeDef::new(
    "block/upgrade_failed",
    Kind::Closing,
    Class::Error,
    "the upgrade to {} from {} failed",
)
.game("Failed to upgrade to version '{}' from document version '{}'");

/// The third line of [`INVALID_COMPONENT`]: takes the target and the document's version.
pub static FINAL_UPGRADE_FAILED: CodeDef<2> = CodeDef::new(
    "block/final_upgrade_failed",
    Kind::Closing,
    Class::Error,
    "the final version {} failed to upgrade from {}",
)
.game("Final json version '{}' failed to upgrade from document version '{}'");

/// A fixer's component without its required field: takes the component and the field. A kept
/// error.
pub static COMPONENT_MISSING_FIELD: CodeDef<2> = CodeDef::new(
    "block/component_missing_field",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "the component {} misses the field {} and goes",
)
.game("Component {} is missing required field {}, component will be discarded");

/// An object that has its value member and another: takes the component. A kept error.
pub static MEMBERS_THROWN_AWAY: CodeDef<1> = CodeDef::new(
    "block/members_thrown_away",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "the other members of {} go in the upgrade",
)
.game("Members of component {} will be thrown away as part of the upgrade");

/// The 1.19.30 step's empty `menu_category.category`.
pub static MENU_CATEGORY_EMPTY: CodeDef<0> = CodeDef::new(
    "block/menu_category_empty",
    Kind::InvalidValue,
    Class::Error,
    "the menu category is empty",
)
.game("Block Menu Category should not have an empty value");

/// The 1.19.30 step's unknown `menu_category.category`: takes the value.
pub static MENU_CATEGORY_INVALID: CodeDef<1> = CodeDef::new(
    "block/menu_category_invalid",
    Kind::InvalidValue,
    Class::Error,
    "the menu category {} is invalid",
)
.game("Block Menu Category has an invalid value of {}");

/// The 1.19.40 step drops `minecraft:part_visibility` (the first of two lines). The game writes
/// neither line when it loads a block: an information.
pub static PART_VISIBILITY_DEPRECATED: CodeDef<0> = CodeDef::new(
    "block/part_visibility_deprecated",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "part_visibility is deprecated",
)
.game("The part_visibility component has been deprecated, use the bone_visibility field in the geometry component instead. See Block Documentation for more information.");

/// The second line of [`PART_VISIBILITY_DEPRECATED`].
pub static PART_VISIBILITY_UNSUPPORTED: CodeDef<0> = CodeDef::new(
    "block/part_visibility_unsupported",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "part_visibility is no longer supported",
)
.game("The part_visibility component is no longer supported. Please use the bone_visibility field in the geometry component instead.");

/// A bone's visibility that is no boolean below 1.20.10: takes the bone and the version. A
/// kept error.
pub static BONE_VISIBILITY_BOOLEAN: CodeDef<2> = CodeDef::new(
    "block/bone_visibility_boolean",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Error,
    "the bone visibility of {} is no boolean before {}",
)
.game("bone_visibility | {}: Only boolean values are supported for bone visibilities in format versions before {}.");

/// The 1.20.10 step's `minecraft:unit_cube` beside `minecraft:geometry`.
pub static UNIT_CUBE_AND_GEOMETRY: CodeDef<0> = CodeDef::new(
    "block/unit_cube_and_geometry",
    Kind::InvalidValue,
    Class::Error,
    "both a unit cube and a geometry",
)
.game("A block cannot have both a unit cube and a geometry component.");

/// An event trigger that is no string. The trailing space is vanilla's.
pub static NONSTRING_JSON: CodeDef<0> = CodeDef::new(
    "block/nonstring_json",
    Kind::WrongType,
    Class::Error,
    "an event that is no string",
)
.game("Invalid, nonstring Json ");

/// `description.states` below 1.19.70.
pub static CUSTOM_PROPERTIES_VERSION: CodeDef<0> = CodeDef::new(
    "block/custom_properties_version",
    Kind::Toggle,
    Class::Error,
    "custom block properties need format 1.19.70",
)
.game("To use custom block properties, use format version 1.19.70 or higher");

/// `permutations` below 1.19.70 (vanilla's "verison").
pub static CUSTOM_PERMUTATIONS_VERSION: CodeDef<0> = CodeDef::new(
    "block/custom_permutations_version",
    Kind::Toggle,
    Class::Error,
    "custom block permutations need format 1.19.70",
)
.game("To use custom block permutations, use format verison 1.19.70 or higher.");

/// A component removed at a version: takes the component and the version. A kept error.
pub static COMPONENT_NOT_VALID_FROM: CodeDef<2> = CodeDef::new(
    "block/component_not_valid_from",
    Kind::Deprecated,
    Class::Error,
    "the component {} goes from {}",
)
.game("{} component is not valid from {} onward.");

/// A permutation's `condition` that is no string.
pub static CONDITION_NOT_STRING: CodeDef<0> = CodeDef::new(
    "block/condition_not_string",
    Kind::WrongType,
    Class::Error,
    "a permutation condition that is no string",
)
.game("permutation condition must be a string value");

// ---- the description ----

/// The identifier of a block outside the base game.
pub static NAMESPACE: CodeDef<0> = CodeDef::new(
    "block/namespace",
    Kind::Identifier,
    Class::Error,
    "the block identifier needs a namespace other than minecraft",
)
.game("Must have a namespace and can't use the \"minecraft\" one. Example: \"namespace:my_block\"");

/// `internal_vanilla_data.block_archetype` with more than one component.
pub static ARCHETYPE_COUNT: CodeDef<0> = CodeDef::new(
    "block/archetype_count",
    Kind::InvalidValue,
    Class::Error,
    "more than one block_archetype component",
)
.game("\"block_archetype\" can only have a maximum of one component.");

/// A state whose values are no valid list or range.
pub static STATE_DESCRIPTION: CodeDef<0> = CodeDef::new(
    "block/state_description",
    Kind::WrongType,
    Class::Error,
    "a state description that fails",
)
.game("Failed to create Block State Description");

/// A trait the game does not know: takes the trait.
pub static UNKNOWN_TRAIT: CodeDef<1> = CodeDef::new(
    "block/unknown_trait",
    Kind::InvalidEnumValue,
    Class::Error,
    "no trait {}",
)
.game("Failed to find BlockTrait with identifier: '{}'");

/// A trait with an empty `enabled_states`: takes the trait.
pub static TRAIT_NEEDS_STATE: CodeDef<1> = CodeDef::new(
    "block/trait_needs_state",
    Kind::MissingRequired,
    Class::Error,
    "the trait {} enables no state",
)
.game("\"{}\" trait requires at least one 'enabled_state' to be specified.");

/// An enabled state the trait does not have: takes the state and the trait.
pub static INVALID_STATE_OPTION: CodeDef<2> = CodeDef::new(
    "block/invalid_state_option",
    Kind::InvalidValue,
    Class::Error,
    "no state {} for the trait {}",
)
.game("Invalid state option \"{}\" for \"{}\" trait.");

/// `placement_direction`'s `y_rotation_offset` without a state it rotates.
pub static Y_ROTATION_OFFSET_STATES: CodeDef<0> = CodeDef::new(
    "block/y_rotation_offset_states",
    Kind::InvalidValue,
    Class::Error,
    "y_rotation_offset without a rotating state",
)
.game("\"y_rotation_offset\" can only be used when \"minecraft:cardinal_direction\", \"minecraft:facing_direction\", and/or \"minecraft:sixteen_way_rotation\" are enabled in \"enabled_states\".");

/// `placement_direction`'s `y_rotation_offset` off the axes; the game takes the nearest.
pub static Y_ROTATION_OFFSET_FACTOR: CodeDef<0> = CodeDef::new(
    "block/y_rotation_offset_factor",
    Kind::Clamped,
    Class::Warning,
    "y_rotation_offset is no multiple of 90",
)
.game("\"minecraft:placement_direction\" trait \"y_rotation_offset\" must be a factor of 90.0. Defaulting to the nearest axis-aligned angle.");

/// `placement_direction`'s `blocks_to_corner_with` without its state.
pub static CORNER_STATES: CodeDef<0> = CodeDef::new(
    "block/corner_states",
    Kind::InvalidValue,
    Class::Error,
    "blocks_to_corner_with without the corner state",
)
.game("\"blocks_to_corner_with\" can only be used when \"minecraft:corner_and_cardinal_direction\" is enabled in \"enabled_states\".");

// ---- the components ----

/// A single box outside its limit: takes the two corners.
pub static BOX_LIMIT_ONE: CodeDef<2> = CodeDef::new(
    "block/box_limit_one",
    Kind::InvalidValue,
    Class::Error,
    "the box is outside {} to {}",
)
.game("\"origin\" + \"size\", must be inside the following limit: {} and {}");

/// A box of a list outside the limit: takes the two corners.
pub static BOX_LIMIT_EACH: CodeDef<2> = CodeDef::new(
    "block/box_limit_each",
    Kind::InvalidValue,
    Class::Error,
    "a box is outside {} to {}",
)
.game(
    "The total of each \"origin\" + \"size\" pair, must be inside the following limit: {} and {}",
);

/// A collision box with both forms.
pub static BOX_FORMS_MIXED: CodeDef<0> = CodeDef::new(
    "block/box_forms_mixed",
    Kind::InvalidValue,
    Class::Error,
    "origin/size beside min/max",
)
.game("\"origin\"/\"size\" cannot be combined with \"min\"/\"max\"");

/// The union of the boxes outside the limit: takes the two corners. The missing space is
/// vanilla's.
pub static BOX_BOUNDS: CodeDef<2> = CodeDef::new(
    "block/box_bounds",
    Kind::InvalidValue,
    Class::Error,
    "the boxes are outside {} to {}",
)
.game("Invalid box boundsThe area covered by all boxes must be within {} to {}");

/// A legacy selection box whose origin is outside the cell. A kept error.
pub static SELECTION_ORIGIN: CodeDef<0> = CodeDef::new(
    "block/selection_origin",
    Kind::InvalidValue,
    Class::Error,
    "the selection box origin is outside its range",
)
.game(
    "minecraft:selection_box: origin can't be below (-8, 0, -8) and can't be more than (8, 16, 8)",
);

/// A legacy selection box whose far corner is outside the cell. A kept error.
pub static SELECTION_EXTENT: CodeDef<0> = CodeDef::new(
    "block/selection_extent",
    Kind::InvalidValue,
    Class::Error,
    "the selection box extent is outside its range",
)
.game("minecraft:selection_box: origin with added size can't be below (-8, 0, -8) and can't be more than (8, 16, 8)");

/// A legacy tick's `interval_range` out of order: takes the component. A kept error.
pub static TICK_RANGE: CodeDef<1> = CodeDef::new(
    "block/tick_range",
    Kind::InvalidValue,
    Class::Error,
    "{}: the tick range ends below its start",
)
.game("{}: maximum ticks in interval range must be more than or equal to minimum ticks");

/// A tick's `interval_range` out of order: takes the reason.
pub static INTERVAL_RANGE: CodeDef<1> = CodeDef::new(
    "block/interval_range",
    Kind::InvalidValue,
    Class::Error,
    "the interval range is invalid: {}",
)
.game("Invalid interval_range: {}");

/// `bone_visibility` on a vanilla geometry.
pub static BONE_VISIBILITY_GEOMETRY: CodeDef<0> = CodeDef::new(
    "block/bone_visibility_geometry",
    Kind::InvalidValue,
    Class::Error,
    "bone visibility on a vanilla geometry",
)
.game("Cannot use bone visibility feature with minecraft:geometry.cross or minecraft:geometry.full_block");

/// A legacy `bone_visibility` on a vanilla geometry: takes the geometry.
pub static INVALID_BONES: CodeDef<1> = CodeDef::new(
    "block/invalid_bones",
    Kind::InvalidValue,
    Class::Error,
    "bone visibility on the vanilla geometry {}",
)
.game("invalid bones: cannot use bone visibility feature with: {}");

/// A component before its format version: takes the version and the component.
pub static FORMAT_REQUIRED: CodeDef<2> = CodeDef::new(
    "block/format_required",
    Kind::InvalidValue,
    Class::Error,
    "format {} is needed for {}",
)
.game("Format version {} is at least required to use {}");

/// The text of core's
/// [`NOT_MULTIPLE_OF`](bedrock_addon_core::diagnostic::codes::NOT_MULTIPLE_OF) on a component the
/// rule drops: takes the value and the step.
pub static NOT_MULTIPLE_OF: CodeDef<2> = CodeDef::new(
    "block/not_multiple_of",
    Kind::InvalidValue,
    Class::Error,
    "{} is no multiple of {}",
)
.game("{} is not multiple of {}");

/// `minecraft:geometry`'s `uv_lock` below 1.21.90.
pub static UV_LOCK_VERSION: CodeDef<0> = CodeDef::new(
    "block/uv_lock_version",
    Kind::InvalidValue,
    Class::Error,
    "uv_lock before format 1.21.90",
)
.game("Cannot use field \"uv_lock\" before 'format_version' 1.21.90 when 'minecraft:transformation' component was released.");

/// A material instance's `isotropic` below 1.21.70.
pub static ISOTROPIC_VERSION: CodeDef<0> = CodeDef::new(
    "block/isotropic_version",
    Kind::InvalidValue,
    Class::Error,
    "isotropic before format 1.21.70",
)
.game(
    "'minecraft:material_instances' cannot use 'isotropic' member with 'format_version' < 1.21.70",
);

/// A material instance's `alpha_masked_tint` below 1.21.120.
pub static ALPHA_MASKED_TINT_VERSION: CodeDef<0> = CodeDef::new(
    "block/alpha_masked_tint_version",
    Kind::InvalidValue,
    Class::Error,
    "alpha_masked_tint before format 1.21.120",
)
.game("'alpha_masked_tint' is only supported at format_version 1.21.120 or higher");

/// `alpha_masked_tint` without a `tint_method`, from 1.26.20.
pub static ALPHA_MASKED_TINT_METHOD: CodeDef<0> = CodeDef::new(
    "block/alpha_masked_tint_method",
    Kind::InvalidValue,
    Class::Error,
    "alpha_masked_tint without a tint method",
)
.game("\"alpha_masked_tint\" requires a non-default \"tint_method\".");

/// `alpha_masked_tint` without a `tint_method`, below 1.26.20.
pub static ALPHA_MASKED_TINT_METHOD_LEGACY: CodeDef<0> = CodeDef::new(
    "block/alpha_masked_tint_method_legacy",
    Kind::InvalidValue,
    Class::Error,
    "alpha_masked_tint without a tint method",
)
.game("'alpha_masked_tint' requires a non-default 'tint_method'");

/// `alpha_masked_tint` with another `render_method`, from 1.26.20.
pub static ALPHA_MASKED_TINT_OPAQUE: CodeDef<0> = CodeDef::new(
    "block/alpha_masked_tint_opaque",
    Kind::InvalidValue,
    Class::Error,
    "alpha_masked_tint without the opaque render method",
)
.game("\"alpha_masked_tint\" requires \"render_method\" to be \"opaque\".");

/// `alpha_masked_tint` with another `render_method`, below 1.26.20.
pub static ALPHA_MASKED_TINT_OPAQUE_LEGACY: CodeDef<0> = CodeDef::new(
    "block/alpha_masked_tint_opaque_legacy",
    Kind::InvalidValue,
    Class::Error,
    "alpha_masked_tint without the opaque render method",
)
.game("'alpha_masked_tint' requires 'render_method' to be 'opaque'");

/// A field before its format version: takes the version and the field. The game checks no
/// experiment.
pub static FORMAT_REQUIRED_EXPERIMENTAL: CodeDef<2> = CodeDef::new(
    "block/format_required_experimental",
    Kind::InvalidValue,
    Class::Error,
    "format {} is needed for {}",
)
.game("Format version {} is at least required to use {} and experimental creator features");

/// From 1.26.20, a random offset that pushes the collision box out of the cell: takes the axis,
/// the bound and the value.
pub static RANDOM_OFFSET_OUTSIDE: CodeDef<3> = CodeDef::new(
    "block/random_offset_outside",
    Kind::InvalidValue,
    Class::Error,
    "the random offset {} {} of {} pushes the collision box out",
)
.game("\"random_offset pushes \"collision_box\" outside of unit cube for \"{}\" \"range\" \"{}\" of {}");

/// Below 1.26.20, the same: the game restricts the value. Takes the axis, the bound, the value
/// and the value kept.
pub static RANDOM_OFFSET_RESTRICTED: CodeDef<4> = CodeDef::new(
    "block/random_offset_restricted",
    Kind::Clamped,
    Class::Warning,
    "the random offset {}->{} goes from {} to {}",
)
.game("Random offset pushes collision box outside of cube for '{}->{}'. Restricting value from {} to {}");

/// `minecraft:multi_block` with a horizontal direction, without the experiment.
pub static MULTI_BLOCK_HORIZONTAL_TOGGLE: CodeDef<0> = CodeDef::new(
    "block/multi_block_horizontal_toggle",
    Kind::Toggle,
    Class::Error,
    "a horizontal multi_block needs Upcoming Creator Features",
)
.game("Upcoming Creator Features experiment is required for trait 'minecraft:multi_block' for horizontal directions");

/// A member gated by the experiment: takes the member and the component.
pub static MEMBER_NEEDS_CREATOR_FEATURES: CodeDef<2> = CodeDef::new(
    "block/member_needs_creator_features",
    Kind::Toggle,
    Class::Error,
    "{} in {} needs Upcoming Creator Features",
)
.game("To use '{}' in '{}', experimental creator features are required");

/// A component gated by the experiment: takes the component.
pub static NEEDS_CREATOR_FEATURES: CodeDef<1> = CodeDef::new(
    "block/needs_creator_features",
    Kind::Toggle,
    Class::Error,
    "{} needs Upcoming Creator Features",
)
.game("To use '{}', experimental creator features are required");

/// A collision box's `min`/`max` from 1.26.60, without the experiment.
pub static COLLISION_MIN_MAX_TOGGLE: CodeDef<0> = CodeDef::new(
    "block/collision_min_max_toggle",
    Kind::Toggle,
    Class::Error,
    "min/max in a collision box needs Upcoming Creator Features",
)
.game("Upcoming Creator Features experiment is required to use \"min\"/\"max\" in \"minecraft:collision_box\" from version 1.26.60 and after");

/// A block entity's `container` beside `minecraft:crafting_table`.
pub static CONTAINER_AND_CRAFTING_TABLE: CodeDef<0> = CodeDef::new(
    "block/container_and_crafting_table",
    Kind::InvalidValue,
    Class::Error,
    "both a container and a crafting table",
)
.game("Block cannot have both a \"container\" in \"minecraft:block_entity\" and a \"minecraft:crafting_table\" component.");

/// `all` beside another neighbour direction.
pub static NEIGHBOR_ALL_ALONE: CodeDef<0> = CodeDef::new(
    "block/neighbor_all_alone",
    Kind::InvalidValue,
    Class::Error,
    "the neighbour direction all beside another",
)
.game("The 'all' neighbor direction must be used alone");

/// A component in a permutation but not in the base: takes the version and the component.
pub static PERMUTATION_ONLY_COMPONENT: CodeDef<2> = CodeDef::new(
    "block/permutation_only_component",
    Kind::InvalidValue,
    Class::Error,
    "from {}, {} in the permutations only",
)
.game("For 'format_version' >= {} the block cannot define '{}' in the 'permutations' array and not define it in the base 'components' object");

// ---- the components together ----

/// From 1.26.20, one of the two without the other.
pub static GEOMETRY_AND_MATERIALS: CodeDef<0> = CodeDef::new(
    "block/geometry_and_materials",
    Kind::InvalidValue,
    Class::Error,
    "geometry without material instances, or the other way",
)
.game("Block needs both a \"geometry\" and \"material_instances\" component.");

/// Below 1.26.20, one of the two without the other.
pub static GEOMETRY_AND_MATERIALS_LEGACY: CodeDef<0> = CodeDef::new(
    "block/geometry_and_materials_legacy",
    Kind::InvalidValue,
    Class::Error,
    "geometry without material instances, or the other way",
)
.game("Block needs both a geometry and material instances component.");

/// `minecraft:flower_pottable` without both.
pub static FLOWER_POTTABLE: CodeDef<0> = CodeDef::new(
    "block/flower_pottable",
    Kind::InvalidValue,
    Class::Error,
    "flower_pottable without geometry and material instances",
)
.game("Block that uses \"flower_pottable\" needs both a \"geometry\" and \"material_instances\" component.");

/// A legacy `minecraft:item_visual` without both: takes the identifier.
pub static ITEM_VISUAL: CodeDef<1> = CodeDef::new(
    "block/item_visual",
    Kind::InvalidValue,
    Class::Error,
    "the block {} has an item visual without geometry and material instances",
)
.game("Block {} needs both a geometry and material instances component when using an Item Visual Component.");

/// Snowlogging with a collision box.
pub static SNOWLOGGING_COLLISION: CodeDef<0> = CodeDef::new(
    "block/snowlogging_collision",
    Kind::InvalidValue,
    Class::Error,
    "snowlogging with a collision box",
)
.game("Blocks with precipitation interactions \"snowlogging\" must have collision box disabled");

/// `minecraft:random_offset` on the full block geometry.
pub static FULL_BLOCK_RANDOM_OFFSET: CodeDef<0> = CodeDef::new(
    "block/full_block_random_offset",
    Kind::InvalidValue,
    Class::Error,
    "a random offset on a full block geometry",
)
.game("Block with \"geometry\" set at \"minecraft:geometry.full_block\" cannot have a \"random_offset\"");

/// From 1.26.20, both redstone components.
pub static REDSTONE_BOTH: CodeDef<0> = CodeDef::new(
    "block/redstone_both",
    Kind::InvalidValue,
    Class::Error,
    "both a redstone producer and consumer",
)
.game("Block cannot have both \"minecraft:redstone_producer\" and \"minecraft:redstone_consumer\" components.");

/// Below 1.26.20, both redstone components.
pub static REDSTONE_BOTH_LEGACY: CodeDef<0> = CodeDef::new(
    "block/redstone_both_legacy",
    Kind::InvalidValue,
    Class::Error,
    "both a redstone producer and consumer",
)
.game("Block cannot have both 'minecraft:redstone_producer' and 'minecraft:redstone_consumer' components");

// ---- multi-block blocks ----

/// A multi-block block without `minecraft:movable`.
pub static MULTI_BLOCK_MOVABLE: CodeDef<0> = CodeDef::new(
    "block/multi_block_movable",
    Kind::MissingRequired,
    Class::Error,
    "a multi_block without movable",
)
.game("Blocks with trait 'minecraft:multi_block' must define component 'minecraft:movable'");

/// A multi-block block's `movement_type`.
pub static MULTI_BLOCK_MOVEMENT_TYPE: CodeDef<0> = CodeDef::new(
    "block/multi_block_movement_type",
    Kind::InvalidValue,
    Class::Error,
    "a multi_block that is neither popped nor immovable",
)
.game("Blocks with trait 'minecraft:multi_block' for component 'minecraft:movable' the 'movement_type' must either be 'popped' or 'immovable'");

/// A horizontal multi-block block with `minecraft:random_offset`.
pub static MULTI_BLOCK_RANDOM_OFFSET: CodeDef<0> = CodeDef::new(
    "block/multi_block_random_offset",
    Kind::InvalidValue,
    Class::Error,
    "a horizontal multi_block with a random offset",
)
.game("Blocks with trait 'minecraft:multi_block' and a horizontal 'direction' cannot define component 'minecraft:random_offset'");

/// A multi-block block with a trait it excludes: takes the trait.
pub static MULTI_BLOCK_TRAIT: CodeDef<1> = CodeDef::new(
    "block/multi_block_trait",
    Kind::InvalidValue,
    Class::Error,
    "a multi_block with the trait {}",
)
.game("A block with trait 'minecraft:multi_block' cannot also define trait '{}'");

/// A multi-block block's placement direction with another state.
pub static MULTI_BLOCK_PLACEMENT_DIRECTION: CodeDef<0> = CodeDef::new(
    "block/multi_block_placement_direction",
    Kind::InvalidValue,
    Class::Error,
    "a multi_block placement direction with another state",
)
.game("A block with trait 'minecraft:multi_block' and trait 'minecraft:placement_direction' can only enable state 'minecraft:cardinal_direction'");

/// A component a multi-block permutation may not define: takes the component.
pub static MULTI_BLOCK_PERMUTATION_COMPONENT: CodeDef<1> = CodeDef::new(
    "block/multi_block_permutation_component",
    Kind::InvalidValue,
    Class::Error,
    "{} in a multi_block permutation",
)
.game("Component '{}' is not valid in multi block permutations, only in the base BlockType components");

// ---- n_way_visual_rotation ----

/// `n_way_visual_rotation` in a permutation.
pub static N_WAY_PERMUTATIONS: CodeDef<0> = CodeDef::new(
    "block/n_way_permutations",
    Kind::InvalidValue,
    Class::Error,
    "n_way_visual_rotation in a permutation",
)
.game("minecraft:geometry 'n_way_visual_rotation' may only be defined on the base block and is not allowed in permutations.");

/// `n_way_visual_rotation` on a multi-block block.
pub static N_WAY_MULTI_BLOCK: CodeDef<0> = CodeDef::new(
    "block/n_way_multi_block",
    Kind::InvalidValue,
    Class::Error,
    "n_way_visual_rotation on a multi_block",
)
.game("'minecraft:geometry' field 'n_way_visual_rotation' cannot be defined for a block with trait 'minecraft:multi_block'");

/// `n_way_visual_rotation` without an axis.
pub static N_WAY_AXIS: CodeDef<0> = CodeDef::new(
    "block/n_way_axis",
    Kind::MissingRequired,
    Class::Error,
    "n_way_visual_rotation without an axis",
)
.game("'n_way_visual_rotation' must define at least one axis (x, y, or z)");

/// An axis naming a state the block does not have: takes the axis and the state.
pub static N_WAY_UNKNOWN_STATE: CodeDef<2> = CodeDef::new(
    "block/n_way_unknown_state",
    Kind::UnresolvedReference,
    Class::Error,
    "n_way_visual_rotation.{} names no state {}",
)
.game("'minecraft:geometry' field 'n_way_visual_rotation.{}' references unknown state '{}'");

// ---- permutations ----

/// A permutation's `condition` that is no Molang.
pub static CONDITION_PARSE: CodeDef<0> = CodeDef::new(
    "block/condition_parse",
    Kind::InvalidValue,
    Class::Error,
    "a permutation condition that does not parse",
)
.game("permutation condition failed to parse");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    MEMBER_MISSING.meta(),
    COMPONENTS_OBJECT.meta(),
    PERMUTATIONS_ARRAY.meta(),
    PERMUTATIONS_ARRAY_OBJECT.meta(),
    PERMUTATIONS_OBJECTS.meta(),
    STATES_OBJECT.meta(),
    PERMUTATION_OBJECT.meta(),
    PARSE_BLOCK_FAILED.meta(),
    DUPLICATE_COMPONENT.meta(),
    CHILD_NOT_VALID.meta(),
    INVALID_COMPONENT.meta(),
    UPGRADE_FAILED.meta(),
    FINAL_UPGRADE_FAILED.meta(),
    COMPONENT_MISSING_FIELD.meta(),
    MEMBERS_THROWN_AWAY.meta(),
    MENU_CATEGORY_EMPTY.meta(),
    MENU_CATEGORY_INVALID.meta(),
    PART_VISIBILITY_DEPRECATED.meta(),
    PART_VISIBILITY_UNSUPPORTED.meta(),
    BONE_VISIBILITY_BOOLEAN.meta(),
    UNIT_CUBE_AND_GEOMETRY.meta(),
    NONSTRING_JSON.meta(),
    CUSTOM_PROPERTIES_VERSION.meta(),
    CUSTOM_PERMUTATIONS_VERSION.meta(),
    COMPONENT_NOT_VALID_FROM.meta(),
    CONDITION_NOT_STRING.meta(),
    NAMESPACE.meta(),
    ARCHETYPE_COUNT.meta(),
    STATE_DESCRIPTION.meta(),
    UNKNOWN_TRAIT.meta(),
    TRAIT_NEEDS_STATE.meta(),
    INVALID_STATE_OPTION.meta(),
    Y_ROTATION_OFFSET_STATES.meta(),
    Y_ROTATION_OFFSET_FACTOR.meta(),
    CORNER_STATES.meta(),
    BOX_LIMIT_ONE.meta(),
    BOX_LIMIT_EACH.meta(),
    BOX_FORMS_MIXED.meta(),
    BOX_BOUNDS.meta(),
    SELECTION_ORIGIN.meta(),
    SELECTION_EXTENT.meta(),
    TICK_RANGE.meta(),
    INTERVAL_RANGE.meta(),
    BONE_VISIBILITY_GEOMETRY.meta(),
    INVALID_BONES.meta(),
    FORMAT_REQUIRED.meta(),
    NOT_MULTIPLE_OF.meta(),
    UV_LOCK_VERSION.meta(),
    ISOTROPIC_VERSION.meta(),
    ALPHA_MASKED_TINT_VERSION.meta(),
    ALPHA_MASKED_TINT_METHOD.meta(),
    ALPHA_MASKED_TINT_METHOD_LEGACY.meta(),
    ALPHA_MASKED_TINT_OPAQUE.meta(),
    ALPHA_MASKED_TINT_OPAQUE_LEGACY.meta(),
    FORMAT_REQUIRED_EXPERIMENTAL.meta(),
    RANDOM_OFFSET_OUTSIDE.meta(),
    RANDOM_OFFSET_RESTRICTED.meta(),
    MULTI_BLOCK_HORIZONTAL_TOGGLE.meta(),
    MEMBER_NEEDS_CREATOR_FEATURES.meta(),
    NEEDS_CREATOR_FEATURES.meta(),
    COLLISION_MIN_MAX_TOGGLE.meta(),
    CONTAINER_AND_CRAFTING_TABLE.meta(),
    NEIGHBOR_ALL_ALONE.meta(),
    PERMUTATION_ONLY_COMPONENT.meta(),
    GEOMETRY_AND_MATERIALS.meta(),
    GEOMETRY_AND_MATERIALS_LEGACY.meta(),
    FLOWER_POTTABLE.meta(),
    ITEM_VISUAL.meta(),
    SNOWLOGGING_COLLISION.meta(),
    FULL_BLOCK_RANDOM_OFFSET.meta(),
    REDSTONE_BOTH.meta(),
    REDSTONE_BOTH_LEGACY.meta(),
    MULTI_BLOCK_MOVABLE.meta(),
    MULTI_BLOCK_MOVEMENT_TYPE.meta(),
    MULTI_BLOCK_RANDOM_OFFSET.meta(),
    MULTI_BLOCK_TRAIT.meta(),
    MULTI_BLOCK_PLACEMENT_DIRECTION.meta(),
    MULTI_BLOCK_PERMUTATION_COMPONENT.meta(),
    N_WAY_PERMUTATIONS.meta(),
    N_WAY_MULTI_BLOCK.meta(),
    N_WAY_AXIS.meta(),
    N_WAY_UNKNOWN_STATE.meta(),
    CONDITION_PARSE.meta(),
];
