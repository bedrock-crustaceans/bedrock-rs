//! The JSON UI family's codes: the game's lines for UI files, byte for byte, each a definition
//! beside the check that raises it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! The control-creation checks ([`Class::Lint`]) are logged by the game only when a screen builds
//! the control, so the stack reports them as informations; the stack's own lines are errors, or
//! kept errors where the game goes on with the file.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the stack ----

/// A listed file whose lowest copy is not JSON: the file's path and the reader's message, which
/// the finding carries again as its `DiagnosticDetail::Syntax`.
pub static PARSE_ERROR: CodeDef<2> = CodeDef::new(
    "json_ui/parse_error",
    Kind::Syntax,
    Class::Error,
    "{} is not JSON: {}",
)
.game("JSON Parse error in file: {}\n\n{}\n")
.was("MalformedInput");

/// The lowest copy of the global variables is not JSON, of the file's path; the reader's message
/// is the finding's `DiagnosticDetail::Syntax`.
pub static GLOBALS_NOT_PARSED: CodeDef<1> = CodeDef::new(
    "json_ui/globals_not_parsed",
    Kind::Syntax,
    Class::Error,
    "the global variables {} are not JSON",
)
.game("Could not parse file at the top of the stack: {}.")
.was("MalformedInput");

/// A listed file no pack has: the game's line goes to the developer log only, so the finding is an
/// information that carries its text.
pub static NOT_LOADED: CodeDef<1> = CodeDef::new(
    "json_ui/not_loaded",
    Kind::UnresolvedReference,
    Class::Lint,
    "no pack has the listed file {}",
)
.game("The following file could not be loaded: {}. Make sure the file exists or remove the reference in '_ui_defs.json'")
.was("UnresolvedReference");

/// Empty or repeated child names in the base copy, one per line, then the file's path: the merge
/// goes on with the file.
pub static INVALID_NAMES: CodeDef<2> = CodeDef::new(
    "json_ui/invalid_names",
    Kind::InvalidValue,
    Class::KeptError,
    "empty or repeated child names {} in {}",
)
.game("We have invalid names in the resource files. Make sure all your control children have unique names that aren't empty!\n\nControls:\n{}\nFile name:{}")
.was("InvalidValue");

// ---- the pack report (`uiPackError`) ----

/// `uiPackError.invalidChildNames`: an overlay introduced empty or repeated child names.
pub static PACK_INVALID_CHILD_NAMES: CodeDef<2> = CodeDef::new(
    "json_ui/pack_invalid_child_names",
    Kind::InvalidValue,
    Class::Error,
    "{}: an overlay introduced child names that are not unique: {}",
)
.game("{}: Child controls must have unique names: {}In the future this will be an error and this file won't load.")
.was("InvalidValue");

/// `uiPackError.parseError`: an overlay copy is not JSON; the reader's message is the finding's
/// `DiagnosticDetail::Syntax`.
pub static PACK_PARSE_ERROR: CodeDef<2> = CodeDef::new(
    "json_ui/pack_parse_error",
    Kind::Syntax,
    Class::Error,
    "{}: an overlay copy is not JSON: {}",
)
.game("{}: Unable to parse UI JSON file with stack: {}")
.was("MalformedInput");

/// `uiPackError.missingControl`: an overlay key or path names no control.
pub static PACK_MISSING_CONTROL: CodeDef<2> = CodeDef::new(
    "json_ui/pack_missing_control",
    Kind::UnresolvedReference,
    Class::Error,
    "{}: no control {}",
)
.game("{}: Can't find the control: {}")
.was("UnresolvedReference");

/// `uiPackError.missingControlTarget`: a name condition matched no element.
pub static PACK_MISSING_CONTROL_TARGET: CodeDef<4> = CodeDef::new(
    "json_ui/pack_missing_control_target",
    Kind::UnresolvedReference,
    Class::Error,
    "{} ({}): no control {} for the operation {}",
)
.game("{} ({}): Can't find the control '{}' during operation '{}'")
.was("UnresolvedReference");

/// `uiPackError.missingArrayName`: neither `array_name` nor `control_name`. The game's line ends
/// with a space.
pub static PACK_MISSING_ARRAY_NAME: CodeDef<2> = CodeDef::new(
    "json_ui/pack_missing_array_name",
    Kind::MissingRequired,
    Class::Error,
    "{} ({}): neither array_name nor control_name",
)
.game("{} ({}): Missing either the field 'array_name' or 'control_name'. ")
.was("MissingRequiredField");

/// `uiPackError.missingCondition`: an operation without the condition it needs.
pub static PACK_MISSING_CONDITION: CodeDef<3> = CodeDef::new(
    "json_ui/pack_missing_condition",
    Kind::MissingRequired,
    Class::Error,
    "{} ({}): the operation {} has no condition",
)
.game("{} ({}): Missing condition on operation '{}'.")
.was("MissingRequiredField");

/// `uiPackError.missingValue`: an insert or replace without `value`.
pub static PACK_MISSING_VALUE: CodeDef<3> = CodeDef::new(
    "json_ui/pack_missing_value",
    Kind::MissingRequired,
    Class::Error,
    "{} ({}): the operation {} has no value",
)
.game("{} ({}): Missing the value for the operation '{}'.")
.was("MissingRequiredField");

/// `uiPackError.missingOperation`: no `operation`.
pub static PACK_MISSING_OPERATION: CodeDef<2> = CodeDef::new(
    "json_ui/pack_missing_operation",
    Kind::MissingRequired,
    Class::Error,
    "{} ({}): no operation",
)
.game("{} ({}): Missing operation.")
.was("MissingRequiredField");

/// `uiPackError.invalidOperationName`: an unknown `operation`.
pub static PACK_INVALID_OPERATION_NAME: CodeDef<3> = CodeDef::new(
    "json_ui/pack_invalid_operation_name",
    Kind::InvalidEnumValue,
    Class::Error,
    "{} ({}): unknown operation {}",
)
.game("{} ({}): Invalid operation '{}'.")
.was("InvalidEnumValue");

// ---- the typed property getters ----

/// A member read as a string that is not one.
pub static EXPECTED_STRING: CodeDef<0> = CodeDef::new(
    "json_ui/expected_string",
    Kind::WrongType,
    Class::Lint,
    "expected a string",
)
.game("JSON UI parse failure - expected a string.")
.was("InvalidType");

/// A member read as a bool that is neither a bool nor a number.
pub static EXPECTED_BOOL: CodeDef<0> = CodeDef::new(
    "json_ui/expected_bool",
    Kind::WrongType,
    Class::Lint,
    "expected a bool",
)
.game("JSON UI parse failure - expected a bool.")
.was("InvalidType");

/// A member read as a number that is not one.
pub static EXPECTED_NUMBER: CodeDef<0> = CodeDef::new(
    "json_ui/expected_number",
    Kind::WrongType,
    Class::Lint,
    "expected a number",
)
.game("JSON UI parse failure - expected a number.")
.was("InvalidType");

/// A member read as an array that is not one.
pub static EXPECTED_ARRAY: CodeDef<0> = CodeDef::new(
    "json_ui/expected_array",
    Kind::WrongType,
    Class::Lint,
    "expected an array",
)
.game("JSON UI parse failure - expected an array.")
.was("InvalidType");

/// A member read as an array or a string that is neither.
pub static EXPECTED_ARRAY_OR_STRING: CodeDef<0> = CodeDef::new(
    "json_ui/expected_array_or_string",
    Kind::WrongType,
    Class::Lint,
    "expected an array or a string",
)
.game("JSON UI parse failure - expected an array or a string.")
.was("InvalidType");

/// A member read as an array or a number that is neither.
pub static EXPECTED_ARRAY_OR_NUMBER: CodeDef<0> = CodeDef::new(
    "json_ui/expected_array_or_number",
    Kind::WrongType,
    Class::Lint,
    "expected an array or a number",
)
.game("JSON UI parse failure - expected an array or a number.")
.was("InvalidType");

/// A member read as an object that is not one.
pub static EXPECTED_OBJECT: CodeDef<0> = CodeDef::new(
    "json_ui/expected_object",
    Kind::WrongType,
    Class::Lint,
    "expected an object",
)
.game("JSON UI parse failure - expected an object.")
.was("InvalidType");

// ---- control creation ----

/// A control with no `type` whose base did not resolve, of the control's name: often a missing
/// `@base`.
pub static TYPE_NOT_SPECIFIED: CodeDef<1> = CodeDef::new(
    "json_ui/type_not_specified",
    Kind::MissingRequired,
    Class::Lint,
    "no type for the control {}",
)
.game("Type not specified (or @-base not found) for control: {}")
.was("MissingRequiredField");

/// A `type` the game does not know: the type and the control's name.
pub static TYPE_NOT_RECOGNIZED: CodeDef<2> = CodeDef::new(
    "json_ui/type_not_recognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown type {} of the control {}",
)
.game("Type '{}' not recognized for control: {}")
.was("InvalidEnumValue");

/// A child that fails to be created fails its parent: the parent's namespace and name, then the
/// child's.
pub static CHILD_NOT_CREATED: CodeDef<4> = CodeDef::new(
    "json_ui/child_not_created",
    Kind::InvalidValue,
    Class::Lint,
    "{}.{} failed to create its child {}.{}",
)
.game("UIControlFactory parent control \"{}.{}\" failed to create child control \"{}.{}\", did you add a new type? Did you add it to the string-to-type function?")
.was("InvalidValue");

/// An `ignored` that is neither a bool, an integer nor a binding.
pub static IGNORED_INVALID: CodeDef<0> = CodeDef::new(
    "json_ui/ignored_invalid",
    Kind::WrongType,
    Class::Lint,
    "ignored is neither a bool nor an integer",
)
.game("An invalid value was provided for \"ignored\" field.")
.was("InvalidType");

/// A `grid_item_template` that names no definition, of the control's name.
pub static GRID_ITEM_TEMPLATE_NOT_CREATED: CodeDef<1> = CodeDef::new(
    "json_ui/grid_item_template_not_created",
    Kind::UnresolvedReference,
    Class::Lint,
    "the grid item template of {} names no definition",
)
.game("gridItemTemplate control tree could not be created! Control: {}")
.was("UnresolvedReference");

/// An entry of `anims` that names no animation definition.
pub static INVALID_ANIM_REFERENCE: CodeDef<0> = CodeDef::new(
    "json_ui/invalid_anim_reference",
    Kind::UnresolvedReference,
    Class::Lint,
    "an anims entry names no definition",
)
.game("JSON UI parse failure: Invalid anim reference!")
.was("UnresolvedReference");

/// A `renderer` the game does not know.
pub static RENDERER_UNRECOGNIZED: CodeDef<1> = CodeDef::new(
    "json_ui/renderer_unrecognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown renderer {}",
)
.game("Unrecognized renderer: '{}'")
.was("InvalidEnumValue");

/// A `state_controls` that is not an array.
pub static STATE_CONTROLS_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "json_ui/state_controls_not_array",
    Kind::WrongType,
    Class::Lint,
    "state_controls is not an array",
)
.game("JSON UI parse failure: Must define state_controls as an array!")
.was("InvalidType");

/// An `images` that is not an array.
pub static IMAGES_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "json_ui/images_not_array",
    Kind::WrongType,
    Class::Lint,
    "images is not an array",
)
.game("JSON UI parse failure: Must define 'images' as an array!")
.was("InvalidType");

/// A `font_type` that is not a string.
pub static FONT_TYPE_NOT_STRING: CodeDef<0> = CodeDef::new(
    "json_ui/font_type_not_string",
    Kind::WrongType,
    Class::Lint,
    "font_type is not a string",
)
.game("JSON UI parse failure: Expected a string for 'font_type'.")
.was("InvalidType");

/// A `backup_font_type` that is not a string.
pub static BACKUP_FONT_TYPE_NOT_STRING: CodeDef<0> = CodeDef::new(
    "json_ui/backup_font_type_not_string",
    Kind::WrongType,
    Class::Lint,
    "backup_font_type is not a string",
)
.game("JSON UI parse failure: Expected a string for 'backup_font_type'.")
.was("InvalidType");

// ---- enum members ----

/// An anchor or alignment the game does not know.
pub static ANCHOR_UNRECOGNIZED: CodeDef<1> = CodeDef::new(
    "json_ui/anchor_unrecognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown anchor {}",
)
.game("Unrecognized anchor or alignment value: '{}'")
.was("InvalidEnumValue");

/// An `orientation` the game does not know.
pub static ORIENTATION_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/orientation_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown orientation {}",
)
.game("Unknown Orientation value: {}")
.was("InvalidEnumValue");

/// A `text_alignment` the game does not know.
pub static TEXT_ALIGNMENT_INVALID: CodeDef<0> = CodeDef::new(
    "json_ui/text_alignment_invalid",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown text alignment",
)
.game("JSON UI parse failure - Text Alignment value invalid.")
.was("InvalidEnumValue");

/// A `draggable` the game does not know.
pub static DRAGGABLE_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/draggable_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown draggable {}",
)
.game("Unknown draggable value: {}")
.was("InvalidEnumValue");

/// A priority rule the game does not know.
pub static PRIORITY_RULE_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/priority_rule_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown priority rule {}",
)
.game("Unknown priority rule value: {}")
.was("InvalidEnumValue");

/// A toggle's, or a slider's, manager behavior the game does not know: the game's slider code logs
/// the text its toggle code uses.
pub static TOGGLE_MANAGER_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/toggle_manager_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown toggle manager behavior {}",
)
.game("Unknown toggle manager behavior type value: {}")
.was("InvalidEnumValue");

/// A focus navigation mode the game does not know.
pub static FOCUS_NAVIGATION_INVALID: CodeDef<0> = CodeDef::new(
    "json_ui/focus_navigation_invalid",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown focus navigation mode",
)
.game("Invalid Input for Focus Navigation")
.was("InvalidEnumValue");

/// A `debug` color the game does not know. The game's line ends with a space and no value.
pub static DEBUG_COLOR_UNRECOGNIZED: CodeDef<0> = CodeDef::new(
    "json_ui/debug_color_unrecognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown debug color",
)
.game("unrecognized debug color specified: ")
.was("InvalidEnumValue");

// ---- expressions ----

/// A quote in an expression that is never closed.
pub static UNCLOSED_QUOTES: CodeDef<0> = CodeDef::new(
    "json_ui/unclosed_quotes",
    Kind::InvalidValue,
    Class::Lint,
    "a quote that is never closed",
)
.game("You didn't close your quotes >:( \n")
.was("InvalidValue");

/// A `#` in an expression outside quotes.
pub static BARE_HASH: CodeDef<0> = CodeDef::new(
    "json_ui/bare_hash",
    Kind::InvalidValue,
    Class::Lint,
    "a # outside quotes",
)
.game("The # sign cannot be used without providing quotes around it!")
.was("InvalidValue");

// ---- bindings ----

/// A `bindings` that is not an array.
pub static BINDINGS_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "json_ui/bindings_not_array",
    Kind::WrongType,
    Class::Lint,
    "bindings is not an array",
)
.game("JSON UI parse failure: Must define bindings as an array.")
.was("InvalidType");

/// A `binding_type` the game does not know; it reads `global`.
pub static BINDING_TYPE_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/binding_type_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown binding type {}",
)
.game("Unknown binding type value: {}")
.was("InvalidEnumValue");

/// A `binding_condition` the game does not know; it reads `none`.
pub static BINDING_CONDITION_UNKNOWN: CodeDef<1> = CodeDef::new(
    "json_ui/binding_condition_unknown",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown binding condition {}",
)
.game("Unknown binding condition value: {}")
.was("InvalidEnumValue");

/// A global or collection binding without `binding_name`.
pub static NO_BINDING_NAME: CodeDef<0> = CodeDef::new(
    "json_ui/no_binding_name",
    Kind::MissingRequired,
    Class::Lint,
    "a binding without a name",
)
.game("JSON UI parse failure: Must define a binding name!")
.was("MissingRequiredField");

/// A collection binding without `binding_collection_name`.
pub static NO_COLLECTION_NAME: CodeDef<0> = CodeDef::new(
    "json_ui/no_collection_name",
    Kind::MissingRequired,
    Class::Lint,
    "a collection binding without a collection name",
)
.game("JSON UI parse failure: Must define a collection name for the binding!")
.was("MissingRequiredField");

/// A view binding without `source_property_name`.
pub static NO_SOURCE_PROPERTY_NAME: CodeDef<0> = CodeDef::new(
    "json_ui/no_source_property_name",
    Kind::MissingRequired,
    Class::Lint,
    "a view binding without a source property",
)
.game("JSON UI parse failure: Must define a source property name in the binding!")
.was("MissingRequiredField");

/// A view binding without `target_property_name`.
pub static NO_TARGET_PROPERTY_NAME: CodeDef<0> = CodeDef::new(
    "json_ui/no_target_property_name",
    Kind::MissingRequired,
    Class::Lint,
    "a view binding without a target property",
)
.game("JSON UI parse failure: Must define a target property name in the binding!")
.was("MissingRequiredField");

/// A view binding with both `resolve_ancestor_scope` and `resolve_sibling_scope` set.
pub static BOTH_SCOPES_SET: CodeDef<0> = CodeDef::new(
    "json_ui/both_scopes_set",
    Kind::InvalidValue,
    Class::Lint,
    "a view binding resolves both the ancestor and the sibling scope",
)
.game("JSON UI parse failure: resolve_ancestor_scope and resolve_sibling_scope cannot both be set!")
.was("InvalidValue");

// ---- factories, focus, input and sounds ----

/// A `factory` without a `name`, or an empty one.
pub static EMPTY_FACTORY_NAME: CodeDef<0> = CodeDef::new(
    "json_ui/empty_factory_name",
    Kind::MissingRequired,
    Class::Lint,
    "a factory without a name",
)
.game("JSON UI parse failure: The factory name should not be empty!")
.was("MissingRequiredField");

/// A `factory_variables` that is not an array of `$variable` strings.
pub static FACTORY_VARIABLES_NOT_VARIABLES: CodeDef<0> = CodeDef::new(
    "json_ui/factory_variables_not_variables",
    Kind::InvalidValue,
    Class::Lint,
    "factory_variables holds more than variables",
)
.game("JSON UI parse failure: factory_variables should all be strings of json variables created higher up the tree.")
.was("InvalidValue");

/// A `focus_mapping` that is not an array.
pub static FOCUS_MAPPING_NOT_ARRAY: CodeDef<0> = CodeDef::new(
    "json_ui/focus_mapping_not_array",
    Kind::WrongType,
    Class::Lint,
    "focus_mapping is not an array",
)
.game("JSON UI parse failure: Expected an array for property 'focus_mapping'.")
.was("InvalidType");

/// An entry of `focus_mapping` that is not an object.
pub static FOCUS_MAPPING_ENTRY_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "json_ui/focus_mapping_entry_not_object",
    Kind::WrongType,
    Class::Lint,
    "a focus_mapping entry is not an object",
)
.game("JSON UI parse failure: Expected an object inside array for property 'focus_mapping'.")
.was("InvalidType");

/// A focus mapping without `focus_identifier`.
pub static NO_FOCUS_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "json_ui/no_focus_identifier",
    Kind::MissingRequired,
    Class::Lint,
    "a focus mapping without an identifier",
)
.game("JSON UI parse failure: Expected a focus identifier.")
.was("MissingRequiredField");

/// A focus mapping with none of the four `focus_change_*` members.
pub static NO_FOCUS_DIRECTION: CodeDef<0> = CodeDef::new(
    "json_ui/no_focus_direction",
    Kind::MissingRequired,
    Class::Lint,
    "a focus mapping without a direction",
)
.game("JSON UI parse failure: Expected a focus direction override.")
.was("MissingRequiredField");

/// A blank `focus_change_up`.
pub static UP_OVERRIDE_BLANK: CodeDef<0> = CodeDef::new(
    "json_ui/up_override_blank",
    Kind::InvalidValue,
    Class::Lint,
    "a blank up override",
)
.game("JSON UI parse failure: Up Override set to blank string. This is redundant as blank strings imply no override.")
.was("InvalidValue");

/// A blank `focus_change_down`.
pub static DOWN_OVERRIDE_BLANK: CodeDef<0> = CodeDef::new(
    "json_ui/down_override_blank",
    Kind::InvalidValue,
    Class::Lint,
    "a blank down override",
)
.game("JSON UI parse failure: Down Override set to blank string. This is redundant as blank strings imply no override.")
.was("InvalidValue");

/// A blank `focus_change_left`.
pub static LEFT_OVERRIDE_BLANK: CodeDef<0> = CodeDef::new(
    "json_ui/left_override_blank",
    Kind::InvalidValue,
    Class::Lint,
    "a blank left override",
)
.game("JSON UI parse failure: Left Override set to blank string. This is redundant as blank strings imply no override.")
.was("InvalidValue");

/// A blank `focus_change_right`.
pub static RIGHT_OVERRIDE_BLANK: CodeDef<0> = CodeDef::new(
    "json_ui/right_override_blank",
    Kind::InvalidValue,
    Class::Lint,
    "a blank right override",
)
.game("JSON UI parse failure: Right Override set to blank string. This is redundant as blank strings imply no override.")
.was("InvalidValue");

/// A button mapping's `mapping_type` the game does not know; absent or empty included.
pub static MAPPING_TYPE_UNRECOGNIZED: CodeDef<1> = CodeDef::new(
    "json_ui/mapping_type_unrecognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown mapping type {}",
)
.game("Unrecognized mapping type String: {}")
.was("InvalidEnumValue");

/// A sound's `event_type` other than `button_event`.
pub static SOUND_EVENT_UNRECOGNIZED: CodeDef<0> = CodeDef::new(
    "json_ui/sound_event_unrecognized",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown sound event type",
)
.game("Unrecognized sound event type.\n")
.was("InvalidEnumValue");

// ---- animations ----

/// A flip-book animation whose `frame_count` is 0, of the control's name.
pub static FRAME_COUNT_ZERO: CodeDef<1> = CodeDef::new(
    "json_ui/frame_count_zero",
    Kind::InvalidValue,
    Class::Lint,
    "a flip book of {} with no frames",
)
.game("FrameCount must be non-zero! Problem found in UIControl{}")
.was("InvalidValue");

/// An offset or size animation without `from` or `to`.
pub static NO_AXIS_OFFSET: CodeDef<0> = CodeDef::new(
    "json_ui/no_axis_offset",
    Kind::MissingRequired,
    Class::Lint,
    "an animation without from or to",
)
.game("The animation needs to have a specified axis offset.")
.was("MissingRequiredField");

/// The endpoints of an animation with a different number of x terms.
pub static X_COUNT_DIFFERS: CodeDef<0> = CodeDef::new(
    "json_ui/x_count_differs",
    Kind::InvalidValue,
    Class::Lint,
    "from and to differ in the number of x terms",
)
.game("The animation needs to have the same number x values.")
.was("InvalidValue");

/// The endpoints of an animation with a different number of y terms.
pub static Y_COUNT_DIFFERS: CodeDef<0> = CodeDef::new(
    "json_ui/y_count_differs",
    Kind::InvalidValue,
    Class::Lint,
    "from and to differ in the number of y terms",
)
.game("The animation needs to have the same number y values.")
.was("InvalidValue");

/// The endpoints of an animation whose terms differ in sign.
pub static OPERATIONS_DIFFER: CodeDef<0> = CodeDef::new(
    "json_ui/operations_differ",
    Kind::InvalidValue,
    Class::Lint,
    "from and to differ in a term's sign",
)
.game("All value's operations need to match with each other.")
.was("InvalidValue");

/// The endpoints of an animation whose terms differ in unit.
pub static TYPES_DIFFER: CodeDef<0> = CodeDef::new(
    "json_ui/types_differ",
    Kind::InvalidValue,
    Class::Lint,
    "from and to differ in a term's unit",
)
.game("All value's types need to match with each other.")
.was("InvalidValue");

// ---- sizes and offsets ----

/// An axis term with a number and no unit.
pub static DANGLING_NUMBER: CodeDef<0> = CodeDef::new(
    "json_ui/dangling_number",
    Kind::InvalidValue,
    Class::Lint,
    "a number without % or px",
)
.game("Dangling number (no % or px in Size)")
.was("InvalidValue");

/// An axis term with a modifier letter and no `%`.
pub static MODIFIER_WITHOUT_PERCENT: CodeDef<0> = CodeDef::new(
    "json_ui/modifier_without_percent",
    Kind::InvalidValue,
    Class::Lint,
    "a % modifier without %",
)
.game("% modifier given without '%' symbol!")
.was("InvalidValue");

/// An axis term of a sibling with letters the game does not know.
pub static UNKNOWN_SIBLING_AXIS: CodeDef<1> = CodeDef::new(
    "json_ui/unknown_sibling_axis",
    Kind::InvalidValue,
    Class::Lint,
    "unknown sibling axis {}",
)
.game("An unknown sibling axis type was found: {}")
.was("InvalidValue");

/// An axis term the game cannot read.
pub static UNRECOGNIZED_TOKEN: CodeDef<0> = CodeDef::new(
    "json_ui/unrecognized_token",
    Kind::InvalidValue,
    Class::Lint,
    "an unknown token in an axis",
)
.game("Unrecognized token!")
.was("InvalidValue");

/// An axis that is neither a number nor a string.
pub static NOT_A_NUMBER: CodeDef<0> = CodeDef::new(
    "json_ui/not_a_number",
    Kind::InvalidValue,
    Class::Lint,
    "an axis that is neither a number nor a string",
)
.game("Trying to give JSON value without a number value!")
.was("InvalidValue");

/// Every definition of the family, for the registry of ids.
pub static ALL: &[&CodeMeta] = &[
    PARSE_ERROR.meta(),
    GLOBALS_NOT_PARSED.meta(),
    NOT_LOADED.meta(),
    INVALID_NAMES.meta(),
    PACK_INVALID_CHILD_NAMES.meta(),
    PACK_PARSE_ERROR.meta(),
    PACK_MISSING_CONTROL.meta(),
    PACK_MISSING_CONTROL_TARGET.meta(),
    PACK_MISSING_ARRAY_NAME.meta(),
    PACK_MISSING_CONDITION.meta(),
    PACK_MISSING_VALUE.meta(),
    PACK_MISSING_OPERATION.meta(),
    PACK_INVALID_OPERATION_NAME.meta(),
    EXPECTED_STRING.meta(),
    EXPECTED_BOOL.meta(),
    EXPECTED_NUMBER.meta(),
    EXPECTED_ARRAY.meta(),
    EXPECTED_ARRAY_OR_STRING.meta(),
    EXPECTED_ARRAY_OR_NUMBER.meta(),
    EXPECTED_OBJECT.meta(),
    TYPE_NOT_SPECIFIED.meta(),
    TYPE_NOT_RECOGNIZED.meta(),
    CHILD_NOT_CREATED.meta(),
    IGNORED_INVALID.meta(),
    GRID_ITEM_TEMPLATE_NOT_CREATED.meta(),
    INVALID_ANIM_REFERENCE.meta(),
    RENDERER_UNRECOGNIZED.meta(),
    STATE_CONTROLS_NOT_ARRAY.meta(),
    IMAGES_NOT_ARRAY.meta(),
    FONT_TYPE_NOT_STRING.meta(),
    BACKUP_FONT_TYPE_NOT_STRING.meta(),
    ANCHOR_UNRECOGNIZED.meta(),
    ORIENTATION_UNKNOWN.meta(),
    TEXT_ALIGNMENT_INVALID.meta(),
    DRAGGABLE_UNKNOWN.meta(),
    PRIORITY_RULE_UNKNOWN.meta(),
    TOGGLE_MANAGER_UNKNOWN.meta(),
    FOCUS_NAVIGATION_INVALID.meta(),
    DEBUG_COLOR_UNRECOGNIZED.meta(),
    UNCLOSED_QUOTES.meta(),
    BARE_HASH.meta(),
    BINDINGS_NOT_ARRAY.meta(),
    BINDING_TYPE_UNKNOWN.meta(),
    BINDING_CONDITION_UNKNOWN.meta(),
    NO_BINDING_NAME.meta(),
    NO_COLLECTION_NAME.meta(),
    NO_SOURCE_PROPERTY_NAME.meta(),
    NO_TARGET_PROPERTY_NAME.meta(),
    BOTH_SCOPES_SET.meta(),
    EMPTY_FACTORY_NAME.meta(),
    FACTORY_VARIABLES_NOT_VARIABLES.meta(),
    FOCUS_MAPPING_NOT_ARRAY.meta(),
    FOCUS_MAPPING_ENTRY_NOT_OBJECT.meta(),
    NO_FOCUS_IDENTIFIER.meta(),
    NO_FOCUS_DIRECTION.meta(),
    UP_OVERRIDE_BLANK.meta(),
    DOWN_OVERRIDE_BLANK.meta(),
    LEFT_OVERRIDE_BLANK.meta(),
    RIGHT_OVERRIDE_BLANK.meta(),
    MAPPING_TYPE_UNRECOGNIZED.meta(),
    SOUND_EVENT_UNRECOGNIZED.meta(),
    FRAME_COUNT_ZERO.meta(),
    NO_AXIS_OFFSET.meta(),
    X_COUNT_DIFFERS.meta(),
    Y_COUNT_DIFFERS.meta(),
    OPERATIONS_DIFFER.meta(),
    TYPES_DIFFER.meta(),
    DANGLING_NUMBER.meta(),
    MODIFIER_WITHOUT_PERCENT.meta(),
    UNKNOWN_SIBLING_AXIS.meta(),
    UNRECOGNIZED_TOKEN.meta(),
    NOT_A_NUMBER.meta(),
];
