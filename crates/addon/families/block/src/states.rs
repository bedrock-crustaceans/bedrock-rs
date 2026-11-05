//! `description.states` and `description.traits`.

use facet::Facet;

use bedrock_addon_core::json::{StrictFloat, StrictInt, StrictString, Value};
use bedrock_addon_core::schema;
use bedrock_addon_shared_shapes::{BlockDescriptor, StateValue};

/// The values one block state can take (JSON `description.states.<name>`).
///
/// JSON: a list of booleans (exactly `[true, false]` in either order), of integers or of strings
/// (1 to 16 values each, no duplicates), `{"values": [..]}` or `{"values": {"min", "max"}}`
/// (an integer range of at most 16 values). Any other shape is kept as written
/// ([`StateValues::Invalid`]) and reported when the block loads.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum StateValues {
    /// `[true, false]`.
    Bools(Vec<bool>),
    /// A list of integers.
    Ints(Vec<i32>),
    /// A list of strings.
    Strings(Vec<String>),
    /// `{"values": [..]}`: a list in the object form.
    Values(Vec<StateValue>),
    /// `{"values": {"min", "max"}}`: every integer from `min` to `max`.
    Range {
        /// JSON `min`.
        min: i32,
        /// JSON `max`.
        max: i32,
    },
    /// A shape that is none of the above, as written.
    Invalid(Value),
}

impl StateValues {
    /// A boolean state: `[false, true]`.
    pub fn bools() -> StateValues {
        StateValues::Bools(vec![false, true])
    }

    /// An integer state from `min` to `max` inclusive.
    pub fn range(min: i32, max: i32) -> StateValues {
        StateValues::Range { min, max }
    }

    /// The values, in order; nothing for [`StateValues::Invalid`].
    pub fn values(&self) -> Vec<StateValue> {
        match self {
            StateValues::Bools(v) => v.iter().copied().map(StateValue::Bool).collect(),
            StateValues::Ints(v) => v.iter().copied().map(StateValue::Int).collect(),
            StateValues::Strings(v) => v.iter().cloned().map(StateValue::String).collect(),
            StateValues::Values(v) => v.clone(),
            StateValues::Range { min, max } => (*min..=*max).map(StateValue::Int).collect(),
            StateValues::Invalid(_) => Vec::new(),
        }
    }

    /// How many values the state has (a range counts its integers).
    pub fn len(&self) -> usize {
        match self {
            StateValues::Range { min, max } => {
                usize::try_from(i64::from(*max) - i64::from(*min) + 1).unwrap_or(0)
            }
            other => other.values().len(),
        }
    }

    /// The state has no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<Value> for StateValues {
    fn from(value: Value) -> Self {
        read_state_values(&value).unwrap_or(StateValues::Invalid(value))
    }
}

/// One of the list forms, or `None`.
fn read_list(array: &facet_value::VArray) -> Option<StateValues> {
    if array.iter().all(Value::is_bool) && !array.is_empty() {
        return Some(StateValues::Bools(
            array.iter().filter_map(Value::as_bool).collect(),
        ));
    }
    if array
        .iter()
        .all(|v| v.as_number().is_some_and(facet_value::VNumber::is_integer))
        && !array.is_empty()
    {
        let ints: Option<Vec<i32>> = array
            .iter()
            .map(|v| v.as_number().and_then(facet_value::VNumber::to_i32))
            .collect();
        return ints.map(StateValues::Ints);
    }
    if array.iter().all(Value::is_string) && !array.is_empty() {
        return Some(StateValues::Strings(
            array
                .iter()
                .filter_map(|v| v.as_string().map(|s| s.as_str().to_owned()))
                .collect(),
        ));
    }
    None
}

/// The typed form of a state's JSON.
fn read_state_values(value: &Value) -> Option<StateValues> {
    if let Some(array) = value.as_array() {
        return read_list(array);
    }
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    let values = object.get("values")?;
    if let Some(array) = values.as_array() {
        let list: Option<Vec<StateValue>> = array
            .iter()
            .map(|v| StateValue::try_from(v.clone()).ok())
            .collect();
        return list.map(StateValues::Values);
    }
    let range = values.as_object()?;
    let bound = |key| {
        range
            .get(key)
            .and_then(Value::as_number)
            .filter(|n| n.is_integer())
            .and_then(facet_value::VNumber::to_i32)
    };
    (range.len() == 2).then_some(())?;
    Some(StateValues::Range {
        min: bound("min")?,
        max: bound("max")?,
    })
}

impl From<&StateValues> for Value {
    fn from(values: &StateValues) -> Value {
        match values {
            StateValues::Bools(v) => v.iter().copied().collect::<facet_value::VArray>().into(),
            StateValues::Ints(v) => v.iter().copied().collect::<facet_value::VArray>().into(),
            StateValues::Strings(v) => v
                .iter()
                .map(String::as_str)
                .collect::<facet_value::VArray>()
                .into(),
            StateValues::Values(v) => {
                let array: facet_value::VArray = v.iter().map(Value::from).collect();
                let mut object = facet_value::VObject::new();
                object.insert("values", Value::from(array));
                object.into()
            }
            StateValues::Range { min, max } => {
                let mut range = facet_value::VObject::new();
                range.insert("min", Value::from(*min));
                range.insert("max", Value::from(*max));
                let mut object = facet_value::VObject::new();
                object.insert("values", Value::from(range));
                object.into()
            }
            StateValues::Invalid(value) => value.clone(),
        }
    }
}

/// `description.traits`: built-in behaviours that add states the engine sets on placement.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Traits {
    /// JSON `minecraft:placement_direction`.
    #[facet(rename = "minecraft:placement_direction")]
    pub placement_direction: Option<PlacementDirection>,
    /// JSON `minecraft:placement_position`.
    #[facet(rename = "minecraft:placement_position")]
    pub placement_position: Option<PlacementPosition>,
    /// JSON `minecraft:connection`.
    #[facet(rename = "minecraft:connection")]
    pub connection: Option<Connection>,
    /// JSON `minecraft:multi_block`.
    #[facet(rename = "minecraft:multi_block")]
    pub multi_block: Option<MultiBlock>,
}

impl Traits {
    /// No trait is set.
    pub fn is_empty(&self) -> bool {
        self == &Traits::default()
    }
}

/// `minecraft:placement_direction`: the direction the player faced when placing the block.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct PlacementDirection {
    /// `minecraft:cardinal_direction`, `minecraft:facing_direction`,
    /// `minecraft:corner_and_cardinal_direction`, `minecraft:sixteen_way_rotation` (JSON
    /// `enabled_states`; at least one).
    #[facet(default)]
    pub enabled_states: Vec<StrictString>,
    /// Degrees added to the placement rotation, a multiple of 90 in `0..=360` (JSON
    /// `y_rotation_offset`).
    pub y_rotation_offset: Option<StrictFloat<f32>>,
    /// Blocks a corner placement connects with (JSON `blocks_to_corner_with`).
    pub blocks_to_corner_with: Option<Vec<BlockDescriptor>>,
}

impl PlacementDirection {
    /// The blocks a corner placement connects with: `blocks_to_corner_with`, none when absent.
    pub fn blocks_to_corner_with(&self) -> &[BlockDescriptor] {
        self.blocks_to_corner_with.as_deref().unwrap_or_default()
    }
}

/// `minecraft:placement_position`: where on the face the block was placed.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct PlacementPosition {
    /// `minecraft:block_face`, `minecraft:vertical_half` (JSON `enabled_states`; at least one).
    #[facet(default)]
    pub enabled_states: Vec<StrictString>,
}

/// `minecraft:connection`: connections to neighbouring blocks.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Connection {
    /// `minecraft:cardinal_connections`, `minecraft:vertical_connections` (JSON
    /// `enabled_states`; at least one).
    #[facet(default)]
    pub enabled_states: Vec<StrictString>,
}

/// `minecraft:multi_block`: a block made of several stacked parts (doors, tall plants).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct MultiBlock {
    /// `minecraft:multi_block_part` (JSON `enabled_states`; required).
    #[facet(default)]
    pub enabled_states: Vec<StrictString>,
    /// Where the other parts go: `down`, `up`, or a horizontal direction with Upcoming Creator
    /// Features (JSON `direction`, required).
    pub direction: StrictString,
    /// How many parts, `2..=4` (JSON `parts`); 2 when absent.
    pub parts: Option<StrictInt<u8>>,
}

impl MultiBlock {
    /// How many parts: `parts`, 2 when absent.
    pub fn parts(&self) -> u8 {
        self.parts.map_or(2, |parts| parts.0)
    }
}

/// The states a trait adds, by enabled-state name: `(state, values)`.
pub(crate) fn implied_states(enabled: &str, parts: u8) -> Vec<(&'static str, StateValues)> {
    let strings =
        |values: &[&str]| StateValues::Strings(values.iter().map(|v| (*v).to_owned()).collect());
    let cardinal = || strings(&["north", "south", "east", "west"]);
    match enabled {
        "minecraft:cardinal_direction" => vec![("minecraft:cardinal_direction", cardinal())],
        "minecraft:facing_direction" => vec![(
            "minecraft:facing_direction",
            strings(&["down", "up", "north", "south", "west", "east"]),
        )],
        "minecraft:corner_and_cardinal_direction" => vec![
            ("minecraft:cardinal_direction", cardinal()),
            (
                "minecraft:corner",
                strings(&[
                    "none",
                    "inner_left",
                    "inner_right",
                    "outer_left",
                    "outer_right",
                ]),
            ),
        ],
        // The state name is the crate's own reading of the trait.
        "minecraft:sixteen_way_rotation" => {
            vec![("minecraft:sixteen_way_rotation", StateValues::range(0, 15))]
        }
        "minecraft:block_face" => vec![(
            "minecraft:block_face",
            strings(&["down", "up", "north", "south", "west", "east"]),
        )],
        "minecraft:vertical_half" => {
            vec![("minecraft:vertical_half", strings(&["bottom", "top"]))]
        }
        "minecraft:cardinal_connections" => vec![
            ("minecraft:connection_north", StateValues::bools()),
            ("minecraft:connection_east", StateValues::bools()),
            ("minecraft:connection_south", StateValues::bools()),
            ("minecraft:connection_west", StateValues::bools()),
        ],
        // The state names are the crate's own reading of the trait.
        "minecraft:vertical_connections" => vec![
            ("minecraft:connection_up", StateValues::bools()),
            ("minecraft:connection_down", StateValues::bools()),
        ],
        "minecraft:multi_block_part" => vec![(
            "minecraft:multi_block_part",
            StateValues::range(0, i32::from(parts.max(1)) - 1),
        )],
        _ => Vec::new(),
    }
}
