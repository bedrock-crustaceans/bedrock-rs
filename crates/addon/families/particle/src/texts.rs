//! Vanilla's content-log texts of the particle loaders, vanilla's spelling kept.

// The legacy particle parser's envelope lines (below 1.20.80).
pub(crate) const NO_PAYLOAD: &str = "no particle_effect field was given";
pub(crate) const NO_DESCRIPTION: &str = "Particle effects file has no description block!";
pub(crate) const NO_IDENTIFIER: &str = "Particle effect description has no identifier field!";
pub(crate) const NO_RENDER_PARAMETERS: &str =
    "basic_render_parameters not found when reading in particle effect";
pub(crate) const NO_MATERIAL: &str = "particle has no material!";
pub(crate) const MATERIAL_NOT_STRING: &str = "particle material field isn't a string";
pub(crate) const NO_TEXTURE: &str = "particle has no texture!";
pub(crate) const TEXTURE_NOT_STRING: &str = "particle texture field isn't a string";
pub(crate) const NO_COMPONENTS: &str = "no components found when reading in particle effect";
pub(crate) const MATERIAL_NOT_FOUND: &str =
    "Unable to find a material when reading in particle effect .json!";
pub(crate) const TEXTURE_NOT_FOUND: &str =
    "Unable to find a texture when reading in particle effect .json!";
pub(crate) const NEEDS_LIFETIME: &str = "Particle effect needs an emitter lifetime component!";
pub(crate) const NEEDS_RATE: &str = "Particle effect needs an emitter rate component!";
// The component list.
pub(crate) const UNRECOGNIZED_COMPONENT: &str =
    "Unrecognized component name '{}' in particle effect";
pub(crate) const COMPONENT_WITHOUT_NAME: &str =
    "Attempting to extract a particle effect component that has no name";
pub(crate) const NO_DATA_FIELDS: &str = "Component has no data fields!";

// A number-or-Molang field, and an array of them.
pub(crate) const NOT_MOLANG: &str = "field is not a number or a Molang expression";
pub(crate) const REQUIRED_MISSING: &str = "required field does not exist";
pub(crate) const ARRAY_ERROR: &str = "error reading array";
pub(crate) const ARRAY_ELEMENT_MISSING: &str = "field does not exist";
pub(crate) const ARRAY_ELEMENT_NOT_MOLANG: &str =
    "array field is not a number or a Molang expression";

// The emitter lifetime events component.
pub(crate) const EVENT_ELEMENT: &str = "Error reading array element";
pub(crate) const EVENT_FIELD: &str = "Error reading field";
pub(crate) const TIMELINE: &str = "Error reading timeline";
pub(crate) const TRAVEL_DISTANCE: &str = "Error reading travel_distance_events";
pub(crate) const LOOPING_ENTRY: &str =
    "Error reading and individual event from looping_travel_distance_events";
pub(crate) const LOOPING: &str = "Error reading looping_travel_distance_events";

// The emitter local space component.
pub(crate) const ROTATION_WITHOUT_POSITION: &str =
    "We do not support rotation being true and position being false";

// emitter shapes
pub(crate) const DIRECTION_INVALID: &str = "direction has invalid value";
// The box shape logs this line when `half_dimensions` is missing (the text says "offset").
pub(crate) const OFFSET_REQUIRED: &str = "offset field required!";
pub(crate) const PLANE_NORMAL_INVALID: &str = "plane_normal has invalid value";

// The block-dependent particle lifetime component.
pub(crate) const BLOCK_NOT_STRING: &str = "Block name is not a string!";
pub(crate) const NO_BLOCK_LIST: &str = "Component has no block list!";

// The particle collision motion component.
pub(crate) const DRAG_NOT_FLOAT: &str = "collision_drag is not a float";
pub(crate) const RESTITUTION_NOT_FLOAT: &str = "coefficient_of_restitution value is not a float";
pub(crate) const RADIUS_NOT_FLOAT: &str = "value is not a float";
pub(crate) const RADIUS_TOO_SMALL: &str = "radius is too small, clamping to 0";
pub(crate) const RADIUS_TOO_LARGE: &str = "radius is too large, clamping to 0.5";
pub(crate) const EXPIRE_NOT_BOOL: &str = "expire_on_contact needs to be true/false";
pub(crate) const EVENTS_NOT_PAIRS: &str = "is not an array of min-speed/event pairs";
pub(crate) const EVENT_NOT_PAIR: &str = "event is not a min-speed/event pair";
pub(crate) const NO_EVENT_FIELD: &str = "no 'event' field!";

// The billboard appearance component: its direction settings and its UVs.
pub(crate) const DIRECTION_NOT_OBJECT: &str =
    "The direction subsection is not defined as a proper json object!";
pub(crate) const DIRECTION_MODE: &str = "The 'direction' sub section is not using a valid mode!";
pub(crate) const DIRECTION_CUSTOM: &str =
    "The 'direction' sub section is not defined as a proper json object!";
// Logged by the legacy billboard parser when its direction section fails to read.
pub(crate) const DIRECTION_NOT_PROPERLY_DEFINED: &str =
    "The direction sub section is not properly defined!";
pub(crate) const BAD_UV: &str = "bad uv data!";
pub(crate) const TEXTURE_WIDTH_NAME: &str = "texture width field needs to be named 'texture_width'";
pub(crate) const TEXTURE_HEIGHT_NAME: &str =
    "texture height field needs to be named 'texture_height'";
pub(crate) const BAD_FLIPBOOK: &str = "bad data!";
pub(crate) const MISSING_UV: &str = "missing uv field";
pub(crate) const MISSING_UV_SIZE: &str = "missing uv_size field";

// The tinting appearance component and its colour expressions.
pub(crate) const NO_COLOR: &str = "Component has no color field!";
pub(crate) const INVALID_COLOR: &str = "Component has invalid color field!";
pub(crate) const COLOR_SHAPE: &str = "Color value isn't a string or array of values";
pub(crate) const COLOR_STRING: &str = "Color value string has problems!";
pub(crate) const COLOR_COUNT: &str = "Color value has wrong number of elements";
pub(crate) const GRADIENT: &str = "Color gradient area not set up correctly";

// The curves section: curves, their node lists and bezier chain nodes.
pub(crate) const NO_CURVES: &str = "No curves found in curves section!";
pub(crate) const CURVE_WITHOUT_NAME: &str = "Attempting to extract a curve with an empty name!";
pub(crate) const CURVE_NO_DATA: &str = "Particle effect curve has no valid data!";
pub(crate) const CURVE_TYPE: &str = "Particle effect curve has invalid type";
pub(crate) const CURVE_NODES: &str = "Particle curve declared with invalid nodes array";
pub(crate) const CURVE_NODE_COUNT: &str =
    "Invalid number of control nodes in curve!  Defaulting curve to 0.0";
pub(crate) const CURVE_NODE_DATA: &str = "Particle curve declared with invalid node data";
pub(crate) const CHAIN_VALUE: &str =
    "Particle curve bezier chain node is missing required value field";
pub(crate) const CHAIN_SLOPE: &str =
    "Particle curve bezier chain node is missing required slope field";

// The events section and its event nodes.
pub(crate) const NO_EVENTS: &str = "No events found in events section!";
pub(crate) const EVENT_WITHOUT_NAME: &str = "Attempting to extract an event with an empty name!";
pub(crate) const EVENT_NO_DATA: &str = "Particle effect event has no valid data!";
pub(crate) const SEQUENCE: &str = "Error reading sequence field!";
pub(crate) const WEIGHT: &str = "Error reading weight field!  Ignoring random option";
pub(crate) const WEIGHT_NEGATIVE: &str =
    "Weight field must be zero or greater.  Ignoring random option";
pub(crate) const RANDOM: &str = "Error reading random field!";
pub(crate) const EFFECT_SECTION: &str = "particle_effect section is invalid";
pub(crate) const EFFECT_MISSING: &str = "Effect field missing!";
pub(crate) const TYPE_MISSING: &str = "Type field missing!";
pub(crate) const SOUND_SECTION: &str = "sound_effect section should be an object";
pub(crate) const SOUND_NAME: &str = "sound_effect.event_name should be a string";
