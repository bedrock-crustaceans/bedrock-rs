//! The particle family's codes: vanilla's content-log lines of the particle loaders, its spelling
//! kept, each a definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy
//! code its sites raised, so a pin or a filter on the code's name keeps matching.
//!
//! Most lines are written by the legacy parser (below 1.20.80) at the error level before its parse
//! returns and the effect loads, and by the 1.20.80 reader where it reads the same way: kept
//! errors. The envelope checks of the legacy parser refuse the document.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the legacy parser's envelope (below 1.20.80) ----

/// No `particle_effect` member: the document is refused.
pub static NO_PAYLOAD: CodeDef<0> = CodeDef::new(
    "particle/no_payload",
    Kind::MissingRequired,
    Class::Error,
    "no particle_effect member",
)
.game("no particle_effect field was given")
.was("MissingRequiredField");

/// No `description` block: the document is refused.
pub static NO_DESCRIPTION: CodeDef<0> = CodeDef::new(
    "particle/no_description",
    Kind::MissingRequired,
    Class::Error,
    "no description block",
)
.game("Particle effects file has no description block!")
.was("MissingRequiredField");

/// No `identifier` in the description: the document is refused (the error is the crate's own
/// there: the game registers an effect named `""`).
pub static NO_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "particle/no_identifier",
    Kind::MissingRequired,
    Class::Error,
    "the description has no identifier",
)
.game("Particle effect description has no identifier field!")
.was("MissingRequiredField");

/// No `basic_render_parameters`: the parser stops and the effect loads.
pub static NO_RENDER_PARAMETERS: CodeDef<0> = CodeDef::new(
    "particle/no_render_parameters",
    Kind::MissingRequired,
    Class::KeptError,
    "no basic_render_parameters",
)
.game("basic_render_parameters not found when reading in particle effect")
.was("MissingRequiredField");

/// No `material` in the render parameters: the parser stops and the effect loads.
pub static NO_MATERIAL: CodeDef<0> = CodeDef::new(
    "particle/no_material",
    Kind::MissingRequired,
    Class::KeptError,
    "no material",
)
.game("particle has no material!")
.was("MissingRequiredField");

/// A `material` that is no string.
pub static MATERIAL_NOT_STRING: CodeDef<0> = CodeDef::new(
    "particle/material_not_string",
    Kind::InvalidValue,
    Class::KeptError,
    "material is not a string",
)
.game("particle material field isn't a string")
.was("InvalidValue");

/// No `texture` in the render parameters.
pub static NO_TEXTURE: CodeDef<0> = CodeDef::new(
    "particle/no_texture",
    Kind::MissingRequired,
    Class::KeptError,
    "no texture",
)
.game("particle has no texture!")
.was("MissingRequiredField");

/// A `texture` that is no string.
pub static TEXTURE_NOT_STRING: CodeDef<0> = CodeDef::new(
    "particle/texture_not_string",
    Kind::InvalidValue,
    Class::KeptError,
    "texture is not a string",
)
.game("particle texture field isn't a string")
.was("InvalidValue");

/// No `components` object: the parser stops and the effect loads.
pub static NO_COMPONENTS: CodeDef<0> = CodeDef::new(
    "particle/no_components",
    Kind::MissingRequired,
    Class::KeptError,
    "no components",
)
.game("no components found when reading in particle effect")
.was("MissingRequiredField");

/// The render parameters' material is not found.
pub static MATERIAL_NOT_FOUND: CodeDef<0> = CodeDef::new(
    "particle/material_not_found",
    Kind::MissingRequired,
    Class::KeptError,
    "the material is not found",
)
.game("Unable to find a material when reading in particle effect .json!")
.was("MissingRequiredField");

/// The render parameters' texture is not found.
pub static TEXTURE_NOT_FOUND: CodeDef<0> = CodeDef::new(
    "particle/texture_not_found",
    Kind::MissingRequired,
    Class::KeptError,
    "the texture is not found",
)
.game("Unable to find a texture when reading in particle effect .json!")
.was("MissingRequiredField");

/// No emitter lifetime component.
pub static NEEDS_LIFETIME: CodeDef<0> = CodeDef::new(
    "particle/needs_lifetime",
    Kind::MissingRequired,
    Class::KeptError,
    "no emitter lifetime component",
)
.game("Particle effect needs an emitter lifetime component!")
.was("MissingRequiredField");

/// No emitter rate component.
pub static NEEDS_RATE: CodeDef<0> = CodeDef::new(
    "particle/needs_rate",
    Kind::MissingRequired,
    Class::KeptError,
    "no emitter rate component",
)
.game("Particle effect needs an emitter rate component!")
.was("MissingRequiredField");

// ---- the component list ----

/// A component name no reader knows, named by the legacy parser's line; not kept (the game
/// stores no unknown key).
pub static UNRECOGNIZED_COMPONENT: CodeDef<1> = CodeDef::new(
    "particle/unrecognized_component",
    Kind::UnknownMember(Disposition::Kept),
    Class::KeptError,
    "unrecognized particle component '{}'",
)
.game("Unrecognized component name '{}' in particle effect")
.was("UnrecognizedComponent");

/// A component under the empty key.
pub static COMPONENT_WITHOUT_NAME: CodeDef<0> = CodeDef::new(
    "particle/component_without_name",
    Kind::InvalidValue,
    Class::KeptError,
    "a component without a name",
)
.game("Attempting to extract a particle effect component that has no name")
.was("InvalidValue");

/// A component whose value holds no data.
pub static NO_DATA_FIELDS: CodeDef<0> = CodeDef::new(
    "particle/no_data_fields",
    Kind::InvalidValue,
    Class::KeptError,
    "a component without data fields",
)
.game("Component has no data fields!")
.was("InvalidValue");

// ---- a number-or-Molang field, and an array of them ----

/// A field that is neither a number nor a Molang expression.
pub static NOT_MOLANG: CodeDef<0> = CodeDef::new(
    "particle/not_molang",
    Kind::InvalidValue,
    Class::KeptError,
    "neither a number nor a Molang expression",
)
.game("field is not a number or a Molang expression")
.was("InvalidValue");

/// A required field is absent.
pub static REQUIRED_MISSING: CodeDef<0> = CodeDef::new(
    "particle/required_missing",
    Kind::InvalidValue,
    Class::KeptError,
    "a required field is absent",
)
.game("required field does not exist")
.was("InvalidValue");

/// An array field that cannot be read.
pub static ARRAY_ERROR: CodeDef<0> = CodeDef::new(
    "particle/array_error",
    Kind::InvalidValue,
    Class::KeptError,
    "the array is unreadable",
)
.game("error reading array")
.was("InvalidValue");

/// An array element that is absent.
pub static ARRAY_ELEMENT_MISSING: CodeDef<0> = CodeDef::new(
    "particle/array_element_missing",
    Kind::InvalidValue,
    Class::KeptError,
    "an array element is absent",
)
.game("field does not exist")
.was("InvalidValue");

/// An array element that is neither a number nor a Molang expression.
pub static ARRAY_ELEMENT_NOT_MOLANG: CodeDef<0> = CodeDef::new(
    "particle/array_element_not_molang",
    Kind::InvalidValue,
    Class::KeptError,
    "an array element is neither a number nor a Molang expression",
)
.game("array field is not a number or a Molang expression")
.was("InvalidValue");

// ---- the emitter lifetime events component ----

/// An element of an event list that cannot be read.
pub static EVENT_ELEMENT: CodeDef<0> = CodeDef::new(
    "particle/event_element",
    Kind::InvalidValue,
    Class::KeptError,
    "an event list element is unreadable",
)
.game("Error reading array element")
.was("InvalidValue");

/// An event field that cannot be read.
pub static EVENT_FIELD: CodeDef<0> = CodeDef::new(
    "particle/event_field",
    Kind::InvalidValue,
    Class::KeptError,
    "an event field is unreadable",
)
.game("Error reading field")
.was("InvalidValue");

/// A `timeline` that cannot be read.
pub static TIMELINE: CodeDef<0> = CodeDef::new(
    "particle/timeline",
    Kind::InvalidValue,
    Class::KeptError,
    "the timeline is unreadable",
)
.game("Error reading timeline")
.was("InvalidValue");

/// A `travel_distance_events` that cannot be read.
pub static TRAVEL_DISTANCE: CodeDef<0> = CodeDef::new(
    "particle/travel_distance",
    Kind::InvalidValue,
    Class::KeptError,
    "travel_distance_events is unreadable",
)
.game("Error reading travel_distance_events")
.was("InvalidValue");

/// An entry of `looping_travel_distance_events` that cannot be read.
pub static LOOPING_ENTRY: CodeDef<0> = CodeDef::new(
    "particle/looping_entry",
    Kind::InvalidValue,
    Class::KeptError,
    "a looping_travel_distance_events entry is unreadable",
)
.game("Error reading and individual event from looping_travel_distance_events")
.was("InvalidValue");

/// A `looping_travel_distance_events` that cannot be read.
pub static LOOPING: CodeDef<0> = CodeDef::new(
    "particle/looping",
    Kind::InvalidValue,
    Class::KeptError,
    "looping_travel_distance_events is unreadable",
)
.game("Error reading looping_travel_distance_events")
.was("InvalidValue");

// ---- the emitter local space component ----

/// `rotation` true with `position` false.
pub static ROTATION_WITHOUT_POSITION: CodeDef<0> = CodeDef::new(
    "particle/rotation_without_position",
    Kind::InvalidValue,
    Class::KeptError,
    "rotation without position",
)
.game("We do not support rotation being true and position being false")
.was("InvalidValue");

// ---- emitter shapes ----

/// A shape's `direction` that cannot be read.
pub static DIRECTION_INVALID: CodeDef<0> = CodeDef::new(
    "particle/direction_invalid",
    Kind::InvalidValue,
    Class::KeptError,
    "the direction is invalid",
)
.game("direction has invalid value")
.was("InvalidValue");

/// The box shape without `half_dimensions` (the game's line says "offset").
pub static OFFSET_REQUIRED: CodeDef<0> = CodeDef::new(
    "particle/offset_required",
    Kind::InvalidValue,
    Class::KeptError,
    "the box shape has no half_dimensions",
)
.game("offset field required!")
.was("InvalidValue");

/// A disc shape's `plane_normal` that cannot be read.
pub static PLANE_NORMAL_INVALID: CodeDef<0> = CodeDef::new(
    "particle/plane_normal_invalid",
    Kind::InvalidValue,
    Class::KeptError,
    "the plane_normal is invalid",
)
.game("plane_normal has invalid value")
.was("InvalidValue");

// ---- the block-dependent particle lifetime component ----

/// A block name that is no string.
pub static BLOCK_NOT_STRING: CodeDef<0> = CodeDef::new(
    "particle/block_not_string",
    Kind::InvalidValue,
    Class::KeptError,
    "a block name is not a string",
)
.game("Block name is not a string!")
.was("InvalidValue");

/// A component without its block list.
pub static NO_BLOCK_LIST: CodeDef<0> = CodeDef::new(
    "particle/no_block_list",
    Kind::InvalidValue,
    Class::KeptError,
    "no block list",
)
.game("Component has no block list!")
.was("InvalidValue");

// ---- the particle collision motion component ----

/// A `collision_drag` that is no number.
pub static DRAG_NOT_FLOAT: CodeDef<0> = CodeDef::new(
    "particle/drag_not_float",
    Kind::InvalidValue,
    Class::KeptError,
    "collision_drag is not a number",
)
.game("collision_drag is not a float")
.was("InvalidValue");

/// A `coefficient_of_restitution` that is no number.
pub static RESTITUTION_NOT_FLOAT: CodeDef<0> = CodeDef::new(
    "particle/restitution_not_float",
    Kind::InvalidValue,
    Class::KeptError,
    "coefficient_of_restitution is not a number",
)
.game("coefficient_of_restitution value is not a float")
.was("InvalidValue");

/// A `collision_radius` that is no number.
pub static RADIUS_NOT_FLOAT: CodeDef<0> = CodeDef::new(
    "particle/radius_not_float",
    Kind::InvalidValue,
    Class::KeptError,
    "collision_radius is not a number",
)
.game("value is not a float")
.was("InvalidValue");

/// A `collision_radius` below 0: the legacy parser clamps it.
pub static RADIUS_TOO_SMALL: CodeDef<0> = CodeDef::new(
    "particle/radius_too_small",
    Kind::InvalidValue,
    Class::KeptError,
    "collision_radius below 0, clamped to 0",
)
.game("radius is too small, clamping to 0")
.was("InvalidValue");

/// A `collision_radius` above 0.5: the legacy parser clamps it.
pub static RADIUS_TOO_LARGE: CodeDef<0> = CodeDef::new(
    "particle/radius_too_large",
    Kind::InvalidValue,
    Class::KeptError,
    "collision_radius above 0.5, clamped to 0.5",
)
.game("radius is too large, clamping to 0.5")
.was("InvalidValue");

/// An `expire_on_contact` that is no boolean.
pub static EXPIRE_NOT_BOOL: CodeDef<0> = CodeDef::new(
    "particle/expire_not_bool",
    Kind::InvalidValue,
    Class::KeptError,
    "expire_on_contact is not a boolean",
)
.game("expire_on_contact needs to be true/false")
.was("InvalidValue");

/// Collision `events` that are no list of min-speed/event pairs.
pub static EVENTS_NOT_PAIRS: CodeDef<0> = CodeDef::new(
    "particle/events_not_pairs",
    Kind::InvalidValue,
    Class::KeptError,
    "events is not a list of min-speed/event pairs",
)
.game("is not an array of min-speed/event pairs")
.was("InvalidValue");

/// A collision event that is no min-speed/event pair.
pub static EVENT_NOT_PAIR: CodeDef<0> = CodeDef::new(
    "particle/event_not_pair",
    Kind::InvalidValue,
    Class::KeptError,
    "a collision event is not a min-speed/event pair",
)
.game("event is not a min-speed/event pair")
.was("InvalidValue");

/// A collision event without its `event`.
pub static NO_EVENT_FIELD: CodeDef<0> = CodeDef::new(
    "particle/no_event_field",
    Kind::InvalidValue,
    Class::KeptError,
    "a collision event without its event",
)
.game("no 'event' field!")
.was("InvalidValue");

// ---- the billboard appearance component: its direction settings and its UVs ----

/// A `direction` that is no object.
pub static DIRECTION_NOT_OBJECT: CodeDef<0> = CodeDef::new(
    "particle/direction_not_object",
    Kind::InvalidValue,
    Class::KeptError,
    "the direction is not an object",
)
.game("The direction subsection is not defined as a proper json object!")
.was("InvalidValue");

/// A `direction` whose mode is none the game knows.
pub static DIRECTION_MODE: CodeDef<0> = CodeDef::new(
    "particle/direction_mode",
    Kind::InvalidValue,
    Class::KeptError,
    "the direction mode is invalid",
)
.game("The 'direction' sub section is not using a valid mode!")
.was("InvalidValue");

/// A custom `direction` that is no object.
pub static DIRECTION_CUSTOM: CodeDef<0> = CodeDef::new(
    "particle/direction_custom",
    Kind::InvalidValue,
    Class::KeptError,
    "the custom direction is not an object",
)
.game("The 'direction' sub section is not defined as a proper json object!")
.was("InvalidValue");

/// The legacy billboard parser's line when its direction section fails to read.
pub static DIRECTION_NOT_PROPERLY_DEFINED: CodeDef<0> = CodeDef::new(
    "particle/direction_not_properly_defined",
    Kind::InvalidValue,
    Class::KeptError,
    "the direction is not properly defined",
)
.game("The direction sub section is not properly defined!")
.was("InvalidValue");

/// UV data that cannot be read.
pub static BAD_UV: CodeDef<0> = CodeDef::new(
    "particle/bad_uv",
    Kind::InvalidValue,
    Class::KeptError,
    "the uv data is unreadable",
)
.game("bad uv data!")
.was("InvalidValue");

/// The old spelling `texturewidth`.
pub static TEXTURE_WIDTH_NAME: CodeDef<0> = CodeDef::new(
    "particle/texture_width_name",
    Kind::InvalidValue,
    Class::KeptError,
    "texturewidth instead of texture_width",
)
.game("texture width field needs to be named 'texture_width'")
.was("InvalidValue");

/// The old spelling `textureheight`.
pub static TEXTURE_HEIGHT_NAME: CodeDef<0> = CodeDef::new(
    "particle/texture_height_name",
    Kind::InvalidValue,
    Class::KeptError,
    "textureheight instead of texture_height",
)
.game("texture height field needs to be named 'texture_height'")
.was("InvalidValue");

/// A `flipbook` that cannot be read.
pub static BAD_FLIPBOOK: CodeDef<0> = CodeDef::new(
    "particle/bad_flipbook",
    Kind::InvalidValue,
    Class::KeptError,
    "the flipbook is unreadable",
)
.game("bad data!")
.was("InvalidValue");

/// A UV area without `uv`.
pub static MISSING_UV: CodeDef<0> = CodeDef::new(
    "particle/missing_uv",
    Kind::InvalidValue,
    Class::KeptError,
    "no uv",
)
.game("missing uv field")
.was("InvalidValue");

/// A UV area without `uv_size`.
pub static MISSING_UV_SIZE: CodeDef<0> = CodeDef::new(
    "particle/missing_uv_size",
    Kind::InvalidValue,
    Class::KeptError,
    "no uv_size",
)
.game("missing uv_size field")
.was("InvalidValue");

// ---- the tinting appearance component and its colour expressions ----

/// A tinting component without `color`.
pub static NO_COLOR: CodeDef<0> = CodeDef::new(
    "particle/no_color",
    Kind::InvalidValue,
    Class::KeptError,
    "no color",
)
.game("Component has no color field!")
.was("InvalidValue");

/// A tinting component whose `color` cannot be read.
pub static INVALID_COLOR: CodeDef<0> = CodeDef::new(
    "particle/invalid_color",
    Kind::InvalidValue,
    Class::KeptError,
    "the color is invalid",
)
.game("Component has invalid color field!")
.was("InvalidValue");

/// A colour that is neither a string nor an array.
pub static COLOR_SHAPE: CodeDef<0> = CodeDef::new(
    "particle/color_shape",
    Kind::InvalidValue,
    Class::KeptError,
    "a colour is neither a string nor an array",
)
.game("Color value isn't a string or array of values")
.was("InvalidValue");

/// A colour string that cannot be read.
pub static COLOR_STRING: CodeDef<0> = CodeDef::new(
    "particle/color_string",
    Kind::InvalidValue,
    Class::KeptError,
    "a colour string is unreadable",
)
.game("Color value string has problems!")
.was("InvalidValue");

/// A colour array of the wrong number of elements.
pub static COLOR_COUNT: CodeDef<0> = CodeDef::new(
    "particle/color_count",
    Kind::InvalidValue,
    Class::KeptError,
    "a colour of the wrong number of elements",
)
.game("Color value has wrong number of elements")
.was("InvalidValue");

/// A colour gradient that cannot be read.
pub static GRADIENT: CodeDef<0> = CodeDef::new(
    "particle/gradient",
    Kind::InvalidValue,
    Class::KeptError,
    "the gradient is unreadable",
)
.game("Color gradient area not set up correctly")
.was("InvalidValue");

// ---- the curves section: curves, their node lists and bezier chain nodes ----

/// A `curves` section that is no object.
pub static NO_CURVES: CodeDef<0> = CodeDef::new(
    "particle/no_curves",
    Kind::InvalidValue,
    Class::KeptError,
    "curves is not an object",
)
.game("No curves found in curves section!")
.was("InvalidValue");

/// A curve under the empty key.
pub static CURVE_WITHOUT_NAME: CodeDef<0> = CodeDef::new(
    "particle/curve_without_name",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve without a name",
)
.game("Attempting to extract a curve with an empty name!")
.was("InvalidValue");

/// A curve that is no object.
pub static CURVE_NO_DATA: CodeDef<0> = CodeDef::new(
    "particle/curve_no_data",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve without data",
)
.game("Particle effect curve has no valid data!")
.was("InvalidValue");

/// A curve whose `type` is absent or none the game knows.
pub static CURVE_TYPE: CodeDef<0> = CodeDef::new(
    "particle/curve_type",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve of an invalid type",
)
.game("Particle effect curve has invalid type")
.was("InvalidValue");

/// A curve whose `nodes` are no array.
pub static CURVE_NODES: CodeDef<0> = CodeDef::new(
    "particle/curve_nodes",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve's nodes are not an array",
)
.game("Particle curve declared with invalid nodes array")
.was("InvalidValue");

/// A curve of the wrong number of control nodes: it is 0.0.
pub static CURVE_NODE_COUNT: CodeDef<0> = CodeDef::new(
    "particle/curve_node_count",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve of the wrong number of control nodes, read as 0.0",
)
.game("Invalid number of control nodes in curve!  Defaulting curve to 0.0")
.was("InvalidValue");

/// A curve whose node data cannot be read.
pub static CURVE_NODE_DATA: CodeDef<0> = CodeDef::new(
    "particle/curve_node_data",
    Kind::InvalidValue,
    Class::KeptError,
    "a curve's node data is invalid",
)
.game("Particle curve declared with invalid node data")
.was("InvalidValue");

/// A bezier chain node without its `value`.
pub static CHAIN_VALUE: CodeDef<0> = CodeDef::new(
    "particle/chain_value",
    Kind::InvalidValue,
    Class::KeptError,
    "a bezier chain node without its value",
)
.game("Particle curve bezier chain node is missing required value field")
.was("InvalidValue");

/// A bezier chain node without its `slope`.
pub static CHAIN_SLOPE: CodeDef<0> = CodeDef::new(
    "particle/chain_slope",
    Kind::InvalidValue,
    Class::KeptError,
    "a bezier chain node without its slope",
)
.game("Particle curve bezier chain node is missing required slope field")
.was("InvalidValue");

// ---- the events section and its event nodes ----

/// An `events` section that is no object.
pub static NO_EVENTS: CodeDef<0> = CodeDef::new(
    "particle/no_events",
    Kind::InvalidValue,
    Class::KeptError,
    "events is not an object",
)
.game("No events found in events section!")
.was("InvalidValue");

/// An event under the empty key.
pub static EVENT_WITHOUT_NAME: CodeDef<0> = CodeDef::new(
    "particle/event_without_name",
    Kind::InvalidValue,
    Class::KeptError,
    "an event without a name",
)
.game("Attempting to extract an event with an empty name!")
.was("InvalidValue");

/// An event node that is no object.
pub static EVENT_NO_DATA: CodeDef<0> = CodeDef::new(
    "particle/event_no_data",
    Kind::InvalidValue,
    Class::KeptError,
    "an event without data",
)
.game("Particle effect event has no valid data!")
.was("InvalidValue");

/// A `sequence` that cannot be read.
pub static SEQUENCE: CodeDef<0> = CodeDef::new(
    "particle/sequence",
    Kind::InvalidValue,
    Class::KeptError,
    "the sequence is unreadable",
)
.game("Error reading sequence field!")
.was("InvalidValue");

/// A random option's `weight` that cannot be read: the option is ignored.
pub static WEIGHT: CodeDef<0> = CodeDef::new(
    "particle/weight",
    Kind::InvalidValue,
    Class::KeptError,
    "a random option's weight is unreadable; the option is ignored",
)
.game("Error reading weight field!  Ignoring random option")
.was("InvalidValue");

/// A random option's negative `weight`: the option is ignored.
pub static WEIGHT_NEGATIVE: CodeDef<0> = CodeDef::new(
    "particle/weight_negative",
    Kind::InvalidValue,
    Class::KeptError,
    "a random option's weight is negative; the option is ignored",
)
.game("Weight field must be zero or greater.  Ignoring random option")
.was("InvalidValue");

/// A `randomize` that cannot be read.
pub static RANDOM: CodeDef<0> = CodeDef::new(
    "particle/random",
    Kind::InvalidValue,
    Class::KeptError,
    "the randomize list is unreadable",
)
.game("Error reading random field!")
.was("InvalidValue");

/// A `particle_effect` section that cannot be read.
pub static EFFECT_SECTION: CodeDef<0> = CodeDef::new(
    "particle/effect_section",
    Kind::InvalidValue,
    Class::KeptError,
    "the particle_effect section is invalid",
)
.game("particle_effect section is invalid")
.was("InvalidValue");

/// A `particle_effect` section without its `effect`.
pub static EFFECT_MISSING: CodeDef<0> = CodeDef::new(
    "particle/effect_missing",
    Kind::InvalidValue,
    Class::KeptError,
    "the particle_effect section has no effect",
)
.game("Effect field missing!")
.was("InvalidValue");

/// A `particle_effect` section without its `type`.
pub static TYPE_MISSING: CodeDef<0> = CodeDef::new(
    "particle/type_missing",
    Kind::InvalidValue,
    Class::KeptError,
    "the particle_effect section has no type",
)
.game("Type field missing!")
.was("InvalidValue");

/// A `sound_effect` section that is no object.
pub static SOUND_SECTION: CodeDef<0> = CodeDef::new(
    "particle/sound_section",
    Kind::InvalidValue,
    Class::KeptError,
    "the sound_effect section is not an object",
)
.game("sound_effect section should be an object")
.was("InvalidValue");

/// A `sound_effect` whose `event_name` is no string.
pub static SOUND_NAME: CodeDef<0> = CodeDef::new(
    "particle/sound_name",
    Kind::InvalidValue,
    Class::KeptError,
    "the sound_effect's event_name is not a string",
)
.game("sound_effect.event_name should be a string")
.was("InvalidValue");

/// Every definition of the family, for the registry of ids.
pub static ALL: &[&CodeMeta] = &[
    NO_PAYLOAD.meta(),
    NO_DESCRIPTION.meta(),
    NO_IDENTIFIER.meta(),
    NO_RENDER_PARAMETERS.meta(),
    NO_MATERIAL.meta(),
    MATERIAL_NOT_STRING.meta(),
    NO_TEXTURE.meta(),
    TEXTURE_NOT_STRING.meta(),
    NO_COMPONENTS.meta(),
    MATERIAL_NOT_FOUND.meta(),
    TEXTURE_NOT_FOUND.meta(),
    NEEDS_LIFETIME.meta(),
    NEEDS_RATE.meta(),
    UNRECOGNIZED_COMPONENT.meta(),
    COMPONENT_WITHOUT_NAME.meta(),
    NO_DATA_FIELDS.meta(),
    NOT_MOLANG.meta(),
    REQUIRED_MISSING.meta(),
    ARRAY_ERROR.meta(),
    ARRAY_ELEMENT_MISSING.meta(),
    ARRAY_ELEMENT_NOT_MOLANG.meta(),
    EVENT_ELEMENT.meta(),
    EVENT_FIELD.meta(),
    TIMELINE.meta(),
    TRAVEL_DISTANCE.meta(),
    LOOPING_ENTRY.meta(),
    LOOPING.meta(),
    ROTATION_WITHOUT_POSITION.meta(),
    DIRECTION_INVALID.meta(),
    OFFSET_REQUIRED.meta(),
    PLANE_NORMAL_INVALID.meta(),
    BLOCK_NOT_STRING.meta(),
    NO_BLOCK_LIST.meta(),
    DRAG_NOT_FLOAT.meta(),
    RESTITUTION_NOT_FLOAT.meta(),
    RADIUS_NOT_FLOAT.meta(),
    RADIUS_TOO_SMALL.meta(),
    RADIUS_TOO_LARGE.meta(),
    EXPIRE_NOT_BOOL.meta(),
    EVENTS_NOT_PAIRS.meta(),
    EVENT_NOT_PAIR.meta(),
    NO_EVENT_FIELD.meta(),
    DIRECTION_NOT_OBJECT.meta(),
    DIRECTION_MODE.meta(),
    DIRECTION_CUSTOM.meta(),
    DIRECTION_NOT_PROPERLY_DEFINED.meta(),
    BAD_UV.meta(),
    TEXTURE_WIDTH_NAME.meta(),
    TEXTURE_HEIGHT_NAME.meta(),
    BAD_FLIPBOOK.meta(),
    MISSING_UV.meta(),
    MISSING_UV_SIZE.meta(),
    NO_COLOR.meta(),
    INVALID_COLOR.meta(),
    COLOR_SHAPE.meta(),
    COLOR_STRING.meta(),
    COLOR_COUNT.meta(),
    GRADIENT.meta(),
    NO_CURVES.meta(),
    CURVE_WITHOUT_NAME.meta(),
    CURVE_NO_DATA.meta(),
    CURVE_TYPE.meta(),
    CURVE_NODES.meta(),
    CURVE_NODE_COUNT.meta(),
    CURVE_NODE_DATA.meta(),
    CHAIN_VALUE.meta(),
    CHAIN_SLOPE.meta(),
    NO_EVENTS.meta(),
    EVENT_WITHOUT_NAME.meta(),
    EVENT_NO_DATA.meta(),
    SEQUENCE.meta(),
    WEIGHT.meta(),
    WEIGHT_NEGATIVE.meta(),
    RANDOM.meta(),
    EFFECT_SECTION.meta(),
    EFFECT_MISSING.meta(),
    TYPE_MISSING.meta(),
    SOUND_SECTION.meta(),
    SOUND_NAME.meta(),
];
