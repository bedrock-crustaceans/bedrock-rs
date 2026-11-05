//! How a block behaves in the world: collision and selection, mining and explosions, light,
//! friction, fire, pistons, liquids, weather, placement and the remaining plain components.

use facet::Facet;
use indexmap::IndexMap;

use super::types::{BoundingBox, Bounds, Direction, Facing, Vec3};
use super::{DESCRIPTOR_TAGS, ITEM_TAGS};
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{BlockDescriptor, ItemDescriptor};

/// `minecraft:selection_box`: what the player's cursor hits.
///
/// JSON: `true` / `false` or one box.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
pub enum SelectionBox {
    /// The full block (`true`) or none (`false`).
    Enabled(bool),
    /// One box.
    Box(BoundingBox),
}

/// `minecraft:destructible_by_mining`: how long the block takes to mine.
///
/// JSON: `false` (cannot be mined), `true`, or `{"seconds_to_destroy", "item_specific_speeds"?}`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
pub enum DestructibleByMining {
    /// Minable (`true`) at the default speed, or not at all (`false`).
    Enabled(bool),
    /// The object form.
    Params(MiningParams),
}

impl DestructibleByMining {
    /// Mined in `seconds` by hand, with no item-specific speeds.
    pub fn seconds(seconds: f32) -> Self {
        DestructibleByMining::Params(MiningParams {
            seconds_to_destroy: seconds,
            item_specific_speeds: Vec::new(),
        })
    }
}

/// The object form of [`DestructibleByMining`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MiningParams {
    /// Seconds to mine by hand (JSON `seconds_to_destroy`, required, `>= 0`).
    #[facet(schema::min = 0.0)]
    pub seconds_to_destroy: f32,
    /// Faster or slower mining with particular items (JSON `item_specific_speeds`; from
    /// 1.21.30).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub item_specific_speeds: Vec<ItemSpecificSpeed>,
}

/// One entry of `item_specific_speeds`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ItemSpecificSpeed {
    /// The item (JSON `item`, required): a name or `{"tags": "<molang>"}`.
    #[facet(schema::molang_with = item_tags)]
    pub item: ItemDescriptor,
    /// Seconds to mine with it (JSON `destroy_speed`, required; negative: cannot be mined).
    pub destroy_speed: f32,
}

/// An item-specific speed's descriptor `tags`, under the block's [`ITEM_TAGS`] rather than the
/// walk's default spec.
fn item_tags(item: &mut ItemDescriptor, visitor: &mut MolangVisitor<'_>) {
    if let ItemDescriptor::Tags { tags, .. } = item {
        visitor.field("tags", &ITEM_TAGS, tags);
    }
}

/// `minecraft:destructible_by_explosion`: how much the block resists explosions.
///
/// JSON: `false` (immune), `true`, or `{"explosion_resistance"}`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
pub enum DestructibleByExplosion {
    /// Destructible (`true`) at the default resistance, or immune (`false`).
    Enabled(bool),
    /// The object form.
    Params(ExplosionParams),
}

/// The object form of [`DestructibleByExplosion`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ExplosionParams {
    /// JSON `explosion_resistance` (required, `>= 0`). Read by the slice's scalar rules: a
    /// number under the strict reader, a number or a bool under the legacy one.
    #[facet(schema::min = 0.0)]
    pub explosion_resistance: f32,
}

/// `minecraft:friction`: `0.0..=0.9`; 0.4 is vanilla's default.
#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct Friction(#[facet(schema::min = 0.0, schema::max = 0.9)] pub f32);

/// `minecraft:light_dampening`: how much light the block absorbs, `0..=15`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct LightDampening(#[facet(schema::max = 15)] pub u8);

/// `minecraft:light_emission`: the light level the block gives off, `0..=15`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct LightEmission(#[facet(schema::max = 15)] pub u8);

/// `minecraft:display_name`: a literal name or a localisation key.
#[derive(Clone, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct DisplayName(pub String);

/// `minecraft:loot`: the loot table path.
#[derive(Clone, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct Loot(pub String);

/// `minecraft:tags`: tags of the block (`namespace:tag_name`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct Tags(pub Vec<String>);

/// `minecraft:sound`: the block's sound type (engine 1.26.50).
#[derive(Clone, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(transparent)]
pub struct Sound(pub String);

/// `minecraft:flammable`: how fire spreads to and destroys the block.
///
/// JSON: `true` / `false`, or `{"catch_chance_modifier"?, "destroy_chance_modifier"?,
/// "lava_flammable"?}`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
pub enum Flammable {
    /// Flammable at the default odds (`true`) or not (`false`).
    Enabled(bool),
    /// The object form.
    Params(FlammableParams),
}

/// The object form of [`Flammable`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FlammableParams {
    /// JSON `catch_chance_modifier`; 5 when absent.
    #[facet(schema::fallback = 5)]
    pub catch_chance_modifier: Option<u16>,
    /// JSON `destroy_chance_modifier`; 20 when absent.
    #[facet(schema::fallback = 20)]
    pub destroy_chance_modifier: Option<u16>,
    /// Whether lava sets it alight (JSON `lava_flammable`; engine 1.26.30).
    pub lava_flammable: Option<LavaFlammable>,
}

/// `flammable.lava_flammable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LavaFlammable {
    /// `never`.
    Never,
    /// `always`.
    Always,
}

/// `minecraft:movable`: how pistons move the block.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Movable {
    /// JSON `movement_type` (required).
    pub movement_type: MovementType,
    /// JSON `sticky`; `none` when absent.
    pub sticky: Option<Sticky>,
}

/// `movable.movement_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum MovementType {
    /// `push_pull`.
    PushPull,
    /// `push`.
    Push,
    /// `popped`.
    Popped,
    /// `immovable`.
    Immovable,
}

/// `movable.sticky`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum Sticky {
    /// `none`.
    None,
    /// `same`.
    Same,
}

/// `minecraft:random_offset`: a random shift of the model and boxes per position.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RandomOffset {
    /// JSON `x`.
    pub x: Option<OffsetAxis>,
    /// JSON `y`.
    pub y: Option<OffsetAxis>,
    /// JSON `z`.
    pub z: Option<OffsetAxis>,
}

/// One axis of [`RandomOffset`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OffsetAxis {
    /// The offset range in sixteenths (JSON `range`, required).
    pub range: Bounds,
    /// How many steps the range is cut into (JSON `steps`); 0 (continuous) when absent.
    #[facet(schema::fallback = 0)]
    pub steps: Option<u32>,
}

/// `minecraft:support`: what the block's top can hold.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Support {
    /// JSON `shape` (required).
    pub shape: SupportShape,
}

/// `support.shape`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum SupportShape {
    /// `fence`.
    Fence,
    /// `stair`.
    Stair,
}

/// `minecraft:tick`: a scheduled tick that fires the block's custom components.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Tick {
    /// The delay in ticks, `[min, max]` with `min <= max` (JSON `interval_range`, required).
    pub interval_range: [u32; 2],
    /// Tick again after each tick (JSON `looping`); `true` when absent.
    #[facet(schema::fallback = true)]
    pub looping: Option<bool>,
}

/// `minecraft:entity_fall_on`: report falls onto the block to custom components.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EntityFallOn {
    /// JSON `min_fall_distance`; 0 when absent.
    #[facet(schema::fallback = 0.0)]
    pub min_fall_distance: Option<f32>,
}

/// Where a leash knot sits on a block whose `leashable` writes no `offset`.
const KNOT_OFFSET: Vec3 = [0.0, 0.25, 0.0];

/// `minecraft:leashable`: leads can be tied to the block.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Leashable {
    /// Where the knot sits (JSON `offset`); `[0, 0.25, 0]` when absent.
    #[facet(schema::fallback = KNOT_OFFSET)]
    pub offset: Option<Vec3>,
}

/// `minecraft:chest_obstruction`: whether the block stops a chest below it opening.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ChestObstruction {
    /// JSON `obstruction_rule`.
    #[facet(schema::fallback = ObstructionRule::Shape)]
    pub obstruction_rule: Option<ObstructionRule>,
}

/// `chest_obstruction.obstruction_rule`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ObstructionRule {
    /// `always`.
    Always,
    /// `never`.
    Never,
    /// `shape`: by the collision shape.
    Shape,
}

/// `minecraft:connection_rule`: which blocks connect to this one (fences, walls, panes).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ConnectionRule {
    /// JSON `accepts_connections_from`.
    #[facet(schema::fallback = AcceptsConnections::All)]
    pub accepts_connections_from: Option<AcceptsConnections>,
    /// JSON `enabled_directions`; all four when absent.
    #[facet(schema::fallback = vec![Direction::South, Direction::North, Direction::East, Direction::West])]
    pub enabled_directions: Option<Vec<Direction>>,
    /// JSON `vanilla_connections_from`: base-game packs only.
    pub vanilla_connections_from: Option<Vec<ConnectionId>>,
}

/// `connection_rule.accepts_connections_from`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum AcceptsConnections {
    /// `none`.
    None,
    /// `only_fences`.
    OnlyFences,
    /// `all`.
    All,
}

/// `connection_rule.vanilla_connections_from`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum ConnectionId {
    /// `fence`.
    Fence,
    /// `nether_brick_fence`.
    NetherBrickFence,
    /// `wall`.
    Wall,
    /// `thin_fence`.
    ThinFence,
}

/// `minecraft:crafting_table`: the block opens a crafting grid.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct CraftingTable {
    /// Recipe tags the table crafts (JSON `crafting_tags`, required; at most 64, each at most
    /// 64 characters).
    pub crafting_tags: Vec<String>,
    /// The screen title, a localisation key (JSON `table_name`); `""` when absent.
    pub table_name: Option<String>,
}

impl CraftingTable {
    /// The screen title: `table_name`, `""` when absent.
    pub fn table_name(&self) -> &str {
        self.table_name.as_ref().map_or("", |name| name.as_str())
    }
}

/// `minecraft:placement_filter`: where the block may be placed or survive.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PlacementFilter {
    /// JSON `conditions` (at most 64); any one passing allows the placement.
    pub conditions: Option<Vec<PlacementCondition>>,
}

impl PlacementFilter {
    /// The conditions: `conditions`, none when absent.
    pub fn conditions(&self) -> &[PlacementCondition] {
        self.conditions.as_deref().unwrap_or_default()
    }
}

/// One entry of `placement_filter.conditions`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PlacementCondition {
    /// The faces it may be placed against (JSON `allowed_faces`).
    pub allowed_faces: Option<Vec<PlacementFace>>,
    /// The blocks it may be placed on (JSON `block_filter`; at most 64).
    #[facet(schema::molang_with = block_filter_tags)]
    pub block_filter: Option<Vec<BlockDescriptor>>,
}

impl PlacementCondition {
    /// The faces the block may be placed against: `allowed_faces`, none listed when absent.
    pub fn allowed_faces(&self) -> &[PlacementFace] {
        self.allowed_faces.as_deref().unwrap_or_default()
    }

    /// The blocks it may be placed on: `block_filter`, none listed when absent.
    pub fn block_filter(&self) -> &[BlockDescriptor] {
        self.block_filter.as_deref().unwrap_or_default()
    }
}

/// `placement_filter.conditions[].allowed_faces`: a face, `side` or `all`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum PlacementFace {
    /// `up`.
    Up,
    /// `down`.
    Down,
    /// `north`.
    North,
    /// `south`.
    South,
    /// `east`.
    East,
    /// `west`.
    West,
    /// `side`: the four horizontal faces.
    Side,
    /// `all`.
    All,
}

/// The `tags` of a block filter's descriptors, under the block's [`DESCRIPTOR_TAGS`] rather
/// than the walk's default spec.
fn block_filter_tags(filter: &mut Option<Vec<BlockDescriptor>>, visitor: &mut MolangVisitor<'_>) {
    for (index, descriptor) in filter.iter_mut().flatten().enumerate() {
        if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = descriptor
        {
            visitor.enter(index.into());
            visitor.field("tags", &DESCRIPTOR_TAGS, tags);
            visitor.leave();
        }
    }
}

/// `minecraft:precipitation_interactions`: what rain and snow do on the block.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PrecipitationInteractions {
    /// JSON `precipitation_behavior` (required).
    pub precipitation_behavior: PrecipitationBehavior,
}

/// `precipitation_interactions.precipitation_behavior`.
///
/// `snow_log_no_collision`, the 1.26.20 spelling of `snowlogging`, is read and normalised to
/// [`PrecipitationBehavior::Snowlogging`] when the block loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum PrecipitationBehavior {
    /// `none`.
    None,
    /// `obstruct_rain`.
    ObstructRain,
    /// `obstruct_rain_accumulate_snow`.
    ObstructRainAccumulateSnow,
    /// `snowlogging`: snow layers can share the block's space.
    Snowlogging,
    /// `snow_log_no_collision`: the 1.26.20 spelling of `snowlogging`.
    SnowLogNoCollision,
}

/// `minecraft:liquid_detection`: how the block reacts to liquids.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct LiquidDetection {
    /// JSON `detection_rules`: exactly one rule.
    pub detection_rules: Vec<DetectionRule>,
}

/// One entry of `liquid_detection.detection_rules`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DetectionRule {
    /// JSON `liquid_type`; `water` is the only liquid.
    pub liquid_type: Option<LiquidType>,
    /// JSON `can_contain_liquid`.
    #[facet(schema::fallback = false)]
    pub can_contain_liquid: Option<bool>,
    /// JSON `on_liquid_touches`.
    pub on_liquid_touches: Option<LiquidReaction>,
    /// JSON `stops_liquid_flowing_from_direction`.
    pub stops_liquid_flowing_from_direction: Option<Vec<Facing>>,
    /// JSON `use_liquid_clipping` (from 1.26.0; older documents get `true`).
    #[facet(schema::fallback = false)]
    pub use_liquid_clipping: Option<bool>,
}

impl DetectionRule {
    /// The directions the block stops the liquid flowing from:
    /// `stops_liquid_flowing_from_direction`, none when absent.
    pub fn stops_liquid_flowing_from_direction(&self) -> &[Facing] {
        self.stops_liquid_flowing_from_direction
            .as_deref()
            .unwrap_or_default()
    }
}

/// `detection_rules[].liquid_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LiquidType {
    /// `water`.
    Water,
}

/// `detection_rules[].on_liquid_touches`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum LiquidReaction {
    /// `broken`.
    Broken,
    /// `popped`.
    Popped,
    /// `blocking`.
    Blocking,
    /// `no_reaction`.
    NoReaction,
}

/// `minecraft:flower_pottable`: the block can go in a flower pot.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FlowerPottable {}

/// `minecraft:replaceable`: placing another block replaces this one.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Replaceable {}

/// `minecraft:block_entity`: the block has a block entity (engine 1.26.30, Upcoming Creator
/// Features; base components only).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct BlockEntity {
    /// JSON `dynamic_properties`.
    #[facet(schema::fallback = false)]
    pub dynamic_properties: Option<bool>,
    /// Entities stored in the block through scripting (JSON `entity_storage`, 1.26.60, Upcoming
    /// Creator Features); `false` when absent.
    #[facet(schema::fallback = false)]
    pub entity_storage: Option<bool>,
    /// A container of the block (JSON `container`, Upcoming Creator Features; not with a
    /// `crafting_table`).
    pub container: Option<ContainerData>,
}

/// `block_entity.container`: the slot count and title of the block's container.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ContainerData {
    /// The number of slots, `1..=54` (JSON `slot_count`, required).
    #[facet(schema::min = 1, schema::max = 54)]
    pub slot_count: u8,
    /// The container's title, a language key or raw text, non-empty, at most 256 characters
    /// (JSON `title`).
    pub title: Option<String>,
}

/// `minecraft:instrument_sound`: the note block instrument above (`up`) and below (`down`) the
/// block (engine 1.26.30, Upcoming Creator Features).
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct InstrumentSound(pub IndexMap<String, NoteBlockInstrument>);

/// A note block instrument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum NoteBlockInstrument {
    #[facet(rename = "note.harp")]
    Harp,
    #[facet(rename = "note.bd")]
    BassDrum,
    #[facet(rename = "note.snare")]
    Snare,
    #[facet(rename = "note.hat")]
    Hat,
    #[facet(rename = "note.bassattack")]
    BassAttack,
    #[facet(rename = "note.flute")]
    Flute,
    #[facet(rename = "note.bell")]
    Bell,
    #[facet(rename = "note.guitar")]
    Guitar,
    #[facet(rename = "note.chime")]
    Chime,
    #[facet(rename = "note.xylophone")]
    Xylophone,
    #[facet(rename = "note.iron_xylophone")]
    IronXylophone,
    #[facet(rename = "note.cow_bell")]
    CowBell,
    #[facet(rename = "note.didgeridoo")]
    Didgeridoo,
    #[facet(rename = "note.bit")]
    Bit,
    #[facet(rename = "note.banjo")]
    Banjo,
    #[facet(rename = "note.pling")]
    Pling,
    #[facet(rename = "note.trumpet")]
    Trumpet,
    #[facet(rename = "note.trumpet_exposed")]
    TrumpetExposed,
    #[facet(rename = "note.trumpet_weathered")]
    TrumpetWeathered,
    #[facet(rename = "note.trumpet_oxidized")]
    TrumpetOxidized,
    #[facet(rename = "note.zombie")]
    Zombie,
    #[facet(rename = "note.skeleton")]
    Skeleton,
    #[facet(rename = "note.creeper")]
    Creeper,
    #[facet(rename = "note.enderdragon")]
    EnderDragon,
    #[facet(rename = "note.witherskeleton")]
    WitherSkeleton,
    #[facet(rename = "note.piglin")]
    Piglin,
    #[facet(rename = "note.none")]
    None,
}

/// `minecraft:enchantment_power_transmitter` (1.26.60, Upcoming Creator Features): placed in
/// the gap between an enchanting table and a bookshelf, the table stays powered (`true`).
///
/// JSON: a bare bool; `false` is the component's absence at run time.
#[derive(Clone, Copy, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct EnchantmentPowerTransmitter(pub bool);

/// `minecraft:vibration_properties` (1.26.60, Upcoming Creator Features): how the block
/// treats vibrations.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct VibrationProperties {
    /// JSON `dampen_vibrations`; `false` when absent.
    #[facet(schema::fallback = false)]
    pub dampen_vibrations: Option<bool>,
    /// JSON `occlude_vibrations`; `false` when absent.
    #[facet(schema::fallback = false)]
    pub occlude_vibrations: Option<bool>,
}

/// `minecraft:neighbor_change` (1.26.60, Upcoming Creator Features): which neighbouring
/// changes reach the block's custom components.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct NeighborChange {
    /// JSON `neighbor_directions`: 1 to 6 directions, `all` alone; `[all]` when absent.
    #[facet(schema::fallback = vec![NeighborChangeDirection::All])]
    pub neighbor_directions: Option<Vec<NeighborChangeDirection>>,
}

/// A direction of `neighbor_change`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
#[allow(missing_docs, reason = "each variant is its JSON spelling")]
pub enum NeighborChangeDirection {
    All,
    Up,
    Down,
    North,
    South,
    West,
    East,
}
