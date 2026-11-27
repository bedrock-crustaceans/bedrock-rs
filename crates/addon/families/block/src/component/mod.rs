//! The components of a block (`minecraft:block/components` and each permutation's
//! `components`): one type per component, named after the JSON key without its namespace, in
//! `PascalCase`.
//!
//! [`BlockComponent`] is the closed set a [`bedrock_addon_core::document::ComponentMap`] of a
//! block holds; `block.components.get::<Friction>()` reads one. Keys of another namespace are
//! custom components, kept with their parameters as written.

mod collision;
mod deprecated;
mod physics;
mod redstone;
mod types;
mod visual;

pub use collision::{CollisionBoundingBox, CollisionBox};
pub(crate) use collision::{legacy_constraint, read_collision_box};
pub use deprecated::{
    Breathing, DeprecatedBaseTrigger, DeprecatedComponents, DeprecatedDefinitionEvent,
    DeprecatedQueuedTicking, DeprecatedRandomTicking,
};
pub use physics::{
    AcceptsConnections, BlockEntity, ChestObstruction, ConnectionId, ConnectionRule, ContainerData,
    CraftingTable, DestructibleByExplosion, DestructibleByMining, DetectionRule, DisplayName,
    EnchantmentPowerTransmitter, EntityFallOn, ExplosionParams, Flammable, FlammableParams,
    FlowerPottable, Friction, InstrumentSound, ItemSpecificSpeed, LavaFlammable, Leashable,
    LightDampening, LightEmission, LiquidDetection, LiquidReaction, LiquidType, Loot, MiningParams,
    Movable, MovementType, NeighborChange, NeighborChangeDirection, NoteBlockInstrument,
    ObstructionRule, OffsetAxis, PlacementCondition, PlacementFace, PlacementFilter,
    PrecipitationBehavior, PrecipitationInteractions, RandomOffset, Replaceable, SelectionBox,
    Sound, Sticky, Support, SupportShape, Tags, Tick, VibrationProperties,
};
pub use redstone::{RedstoneConductivity, RedstoneConsumer, RedstoneProducer};
pub use types::{
    BoundingBox, Bounds, Color, Direction, Facing, RenderMethod, TintMethod, Vec3, vec3,
};
pub use visual::{
    DestructionParticles, EmbeddedVisual, Geometry, GeometryParams, ItemVisual, MapColor,
    MapColorParams, MaterialInstance, MaterialInstanceEntry, MaterialInstances, NWayVisualRotation,
    Transformation, UvLock,
};

use bedrock_addon_derive::Components;

use super::history::BlockFamily;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPathSegment, Severity, codes as core_codes};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, CustomComponent};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::{Experiment, LoadContext, LoadRequirement};
use bedrock_addon_core::molang::{
    BLOCK_STATE_QUERIES, EmptyRule, MolangFieldSpec, MolangVersion, MolangVersionSource, OpsRule,
    ReaderKind,
};
use bedrock_addon_core::versioning::{ConvertMode, NullHandling, read_component_with};

/// The parser of a typed component's table row: the component from its slice, its unknown
/// members kept ([`crate::typed_read`]).
fn typed<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent>
where
    T: bedrock_addon_core::schema::FromJson + for<'a> facet::Facet<'a> + Into<BlockComponent>,
{
    read_component_with::<T, BlockComponent>(slice, mode, cx, d, crate::typed_read)
}

/// A component whose material instances keep their unknown members: the game says nothing of one
/// in a legacy document, so the crate tells it as an information; a cereal document's is the
/// slice's grade. `NESTED`: the instances are the component's `material_instances` member.
fn read_instances<T, const NESTED: bool>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent>
where
    T: bedrock_addon_core::schema::FromJson + for<'a> facet::Facet<'a> + Into<BlockComponent>,
{
    let before = d.len();
    let component = read_component_with::<T, BlockComponent>(slice, mode, cx, d, crate::typed_read);
    if cx.reading.null != NullHandling::Rejected {
        for finding in d.iter_mut().skip(before) {
            let path = &finding.location.json_path;
            let instance_member = path.len() == if NESTED { 3 } else { 2 }
                && (!NESTED
                    || path.segments().next() == Some(JsonPathSegment::Key("material_instances")));
            if instance_member && finding.code.id() == core_codes::UNRECOGNIZED_FIELD.id() {
                finding.severity = Severity::Info;
            }
        }
    }
    component
}

/// Vanilla's legacy creator features: the ten allow-listed packs.
const LCF: LoadRequirement = LoadRequirement::LegacyCreatorFeatures;
/// The Upcoming Creator Features toggle.
const UCF: LoadRequirement = LoadRequirement::Experiment(Experiment::UpcomingCreatorFeatures);

// Each component's `released` version is the format version from which the game's component
// table accepts it; `since_engine` marks the members and components added after that.

/// One component of a block: the closed set of `minecraft:` components the 1.26.20 document
/// declares, plus custom (`ns:name`) and unknown keys.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = BlockFamily, custom = Custom, unknown = Unknown, write)]
#[non_exhaustive]
pub enum BlockComponent {
    /// `minecraft:block_entity`. No member of a legacy document; its `container` and
    /// `entity_storage` are behind Upcoming Creator Features by the validator of 1.26.60: the
    /// family gates them itself.
    #[component(name = "minecraft:block_entity", fields = [{ name = "", since_engine = "1.26.30" }], parse = typed::<BlockEntity>)]
    BlockEntity(BlockEntity),
    /// `minecraft:chest_obstruction`.
    #[component(name = "minecraft:chest_obstruction", released = "1.26.20", requires = [UCF], parse = typed::<ChestObstruction>)]
    ChestObstruction(ChestObstruction),
    /// `minecraft:collision_box`.
    #[component(name = "minecraft:collision_box", shorthand = scalar, released = "1.19.50", requires = [LCF], parse = parse_collision_box)]
    CollisionBox(CollisionBox),
    /// `minecraft:connection_rule`.
    #[component(name = "minecraft:connection_rule", released = "1.26.0", requires = [UCF], parse = typed::<ConnectionRule>)]
    ConnectionRule(ConnectionRule),
    /// `minecraft:crafting_table`.
    #[component(name = "minecraft:crafting_table", released = "1.19.50", requires = [LCF], parse = typed::<CraftingTable>)]
    CraftingTable(CraftingTable),
    /// `minecraft:destructible_by_explosion`.
    #[component(name = "minecraft:destructible_by_explosion", shorthand = scalar, parse = typed::<DestructibleByExplosion>)]
    DestructibleByExplosion(DestructibleByExplosion),
    /// `minecraft:destructible_by_mining`.
    #[component(name = "minecraft:destructible_by_mining", shorthand = scalar, molang, parse = typed::<DestructibleByMining>)]
    DestructibleByMining(DestructibleByMining),
    /// `minecraft:destruction_particles` (format 1.21.70, checked by the family).
    #[component(name = "minecraft:destruction_particles", parse = typed::<DestructionParticles>)]
    DestructionParticles(DestructionParticles),
    /// `minecraft:display_name`.
    #[component(name = "minecraft:display_name", released = "1.19.60", requires = [LCF], parse = typed::<DisplayName>)]
    DisplayName(DisplayName),
    /// `minecraft:embedded_visual`.
    #[component(name = "minecraft:embedded_visual", released = "1.21.120", requires = [UCF], molang, parse = read_instances::<EmbeddedVisual, true>)]
    EmbeddedVisual(EmbeddedVisual),
    /// `minecraft:enchantment_power_transmitter` (1.26.60). Behind Upcoming Creator Features
    /// by the validator of 1.26.60, and no member of a legacy document: the family gates it itself.
    #[component(name = "minecraft:enchantment_power_transmitter", parse = typed::<EnchantmentPowerTransmitter>)]
    EnchantmentPowerTransmitter(EnchantmentPowerTransmitter),
    /// `minecraft:entity_fall_on`.
    #[component(name = "minecraft:entity_fall_on", parse = typed::<EntityFallOn>)]
    EntityFallOn(EntityFallOn),
    /// `minecraft:flammable`.
    #[component(name = "minecraft:flammable", shorthand = scalar, fields = [{ name = "lava_flammable", since_engine = "1.26.30" }], parse = typed::<Flammable>)]
    Flammable(Flammable),
    /// `minecraft:flower_pottable`.
    #[component(name = "minecraft:flower_pottable", released = "1.21.120", requires = [UCF], parse = typed::<FlowerPottable>)]
    FlowerPottable(FlowerPottable),
    /// `minecraft:friction`.
    #[component(name = "minecraft:friction", parse = typed::<Friction>)]
    Friction(Friction),
    /// `minecraft:geometry`.
    #[component(name = "minecraft:geometry", shorthand = scalar, released = "1.19.40", requires = [LCF], fields = [{ name = "n_way_visual_rotation", since_engine = "1.26.30" }], molang, parse = typed::<Geometry>)]
    Geometry(Geometry),
    /// `minecraft:instrument_sound`. No member of a legacy document, ungated from 1.26.20.
    #[component(name = "minecraft:instrument_sound", fields = [{ name = "", since_engine = "1.26.30" }], parse = typed::<InstrumentSound>)]
    InstrumentSound(InstrumentSound),
    /// `minecraft:item_visual`.
    #[component(name = "minecraft:item_visual", released = "1.21.50", requires = [UCF], molang, parse = read_instances::<ItemVisual, true>)]
    ItemVisual(ItemVisual),
    /// `minecraft:leashable`.
    #[component(name = "minecraft:leashable", released = "1.26.0", requires = [UCF], parse = typed::<Leashable>)]
    Leashable(Leashable),
    /// `minecraft:light_dampening`.
    #[component(name = "minecraft:light_dampening", parse = typed::<LightDampening>)]
    LightDampening(LightDampening),
    /// `minecraft:light_emission`.
    #[component(name = "minecraft:light_emission", parse = typed::<LightEmission>)]
    LightEmission(LightEmission),
    /// `minecraft:liquid_detection`.
    #[component(name = "minecraft:liquid_detection", released = "1.21.60", requires = [UCF], parse = typed::<LiquidDetection>)]
    LiquidDetection(LiquidDetection),
    /// `minecraft:loot`.
    #[component(name = "minecraft:loot", parse = typed::<Loot>)]
    Loot(Loot),
    /// `minecraft:map_color`.
    #[component(name = "minecraft:map_color", shorthand = scalar, parse = typed::<MapColor>)]
    MapColor(MapColor),
    /// `minecraft:material_instances`.
    #[component(name = "minecraft:material_instances", released = "1.19.40", requires = [LCF], parse = read_instances::<MaterialInstances, false>)]
    MaterialInstances(MaterialInstances),
    /// `minecraft:movable`.
    #[component(name = "minecraft:movable", released = "1.21.100", requires = [UCF], parse = typed::<Movable>)]
    Movable(Movable),
    /// `minecraft:neighbor_change` (1.26.60). Behind Upcoming Creator Features by the
    /// validator of 1.26.60, and no member of a legacy document: the family gates it itself.
    #[component(name = "minecraft:neighbor_change", parse = typed::<NeighborChange>)]
    NeighborChange(NeighborChange),
    /// `minecraft:placement_filter`.
    #[component(name = "minecraft:placement_filter", released = "1.19.60", requires = [LCF], molang, parse = typed::<PlacementFilter>)]
    PlacementFilter(PlacementFilter),
    /// `minecraft:precipitation_interactions`.
    #[component(name = "minecraft:precipitation_interactions", released = "1.21.120", requires = [UCF], parse = typed::<PrecipitationInteractions>)]
    PrecipitationInteractions(PrecipitationInteractions),
    /// `minecraft:random_offset`.
    #[component(name = "minecraft:random_offset", released = "1.21.100", requires = [UCF], parse = typed::<RandomOffset>)]
    RandomOffset(RandomOffset),
    /// `minecraft:redstone_conductivity`.
    #[component(name = "minecraft:redstone_conductivity", released = "1.21.30", requires = [UCF], parse = typed::<RedstoneConductivity>)]
    RedstoneConductivity(RedstoneConductivity),
    /// `minecraft:redstone_consumer`.
    #[component(name = "minecraft:redstone_consumer", released = "1.26.0", requires = [UCF], parse = typed::<RedstoneConsumer>)]
    RedstoneConsumer(RedstoneConsumer),
    /// `minecraft:redstone_producer`.
    #[component(name = "minecraft:redstone_producer", released = "1.21.120", requires = [UCF], parse = typed::<RedstoneProducer>)]
    RedstoneProducer(RedstoneProducer),
    /// `minecraft:replaceable`.
    #[component(name = "minecraft:replaceable", released = "1.21.60", requires = [UCF], parse = typed::<Replaceable>)]
    Replaceable(Replaceable),
    /// `minecraft:selection_box`.
    #[component(name = "minecraft:selection_box", shorthand = scalar, released = "1.19.60", requires = [LCF], parse = typed::<SelectionBox>)]
    SelectionBox(SelectionBox),
    /// `minecraft:sound`. Its legacy row is released at 1.26.40 behind Upcoming Creator
    /// Features (the family's own gate below 1.26.20); ungated from 1.26.20.
    #[component(name = "minecraft:sound", fields = [{ name = "", since_engine = "1.26.50" }], parse = typed::<Sound>)]
    Sound(Sound),
    /// `minecraft:support`.
    #[component(name = "minecraft:support", released = "1.26.0", requires = [UCF], parse = typed::<Support>)]
    Support(Support),
    /// `minecraft:tags`.
    #[component(name = "minecraft:tags", parse = typed::<Tags>)]
    Tags(Tags),
    /// `minecraft:tick`.
    #[component(name = "minecraft:tick", parse = typed::<Tick>)]
    Tick(Tick),
    /// `minecraft:transformation`.
    #[component(name = "minecraft:transformation", released = "1.19.80", requires = [LCF], parse = typed::<Transformation>)]
    Transformation(Transformation),
    /// `minecraft:vibration_properties` (1.26.60). Behind Upcoming Creator Features, by the
    /// validator of 1.26.60 and below 1.26.20 by its legacy row (released 1.26.60): the
    /// family gates it itself.
    #[component(name = "minecraft:vibration_properties", parse = typed::<VibrationProperties>)]
    VibrationProperties(VibrationProperties),
    /// A custom component (`ns:name`) with its parameters.
    Custom(CustomComponent),
    /// A key no table knows, kept for write-back.
    Unknown(Value),
}

/// `minecraft:collision_box`: `true` / `false`, an array of boxes, or one box as an object,
/// which only a document below 1.26.60 may write (the 1.26.20 alternative; its reader says
/// whether the document's version has it and keeps it as an array of one).
fn parse_collision_box(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent> {
    if slice.text.trim_start().starts_with('{') {
        typed::<OneCollisionBox>(slice, mode, cx, d)
    } else {
        typed::<CollisionBox>(slice, mode, cx, d)
    }
}

/// One collision box written as an object.
#[derive(facet::Facet, bedrock_addon_core::schema::Schema)]
#[facet(transparent)]
struct OneCollisionBox(CollisionBoundingBox);

impl From<OneCollisionBox> for BlockComponent {
    fn from(one: OneCollisionBox) -> Self {
        BlockComponent::CollisionBox(CollisionBox::Boxes(vec![one.0]))
    }
}

/// A component map as a JSON object, in map order.
pub(crate) fn write_components(map: &ComponentMap<BlockComponent>) -> Result<Value, EncodeError> {
    let mut object = facet_value::VObject::new();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (name.to_owned(), value.write_component()?),
            ComponentRef::Custom(c) => (c.name.clone(), c.parameters.clone()),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                (name.to_owned(), json.clone())
            }
            _ => continue,
        };
        object.insert(name, json);
    }
    Ok(object.into())
}

// Molang field specs of block files (rule 19, the block rows). Pass 1 only runs on documents
// parsed as 1.26.20 (the game loads them with Molang version 13); the legacy conversion gives
// its strings the pack's version itself, so the version source of the component fields is the
// constant 13. The rejected expressions of these fields are kept errors: the Molang parser
// logs its refusal at the content log's error level and every consumer below drops the parse
// result and goes on, so the block loads.

/// `geometry.bone_visibility.<bone>`: `{query.block_state}`, no side effects. A string entry
/// that fails to parse is logged and dropped from the parse; the entry is still stored and the
/// loop goes on with the next bone.
pub(crate) const BONE_VISIBILITY: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::LenientVersioned)
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .only(BLOCK_STATE_QUERIES)
    .ops(OpsRule::NoAssignmentsOrRandom)
    .empty(EmptyRule::BlockNoExpression);

/// A block descriptor's `tags`: the `tags` query set, version 13. The game assigns the result
/// of a failed tag-expression parse and returns normally, so the descriptor still loads.
pub(crate) const DESCRIPTOR_TAGS: MolangFieldSpec = MolangFieldSpec::KEPT
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .tags()
    .empty(EmptyRule::BlockNoExpression);

/// An item descriptor's `tags` (`item_specific_speeds`): the `tags` query set. A failed parse
/// builds a node anyway and the caller stores it, so the descriptor still loads. The block's
/// `item` member is assumed to read its descriptor through the same path as the item family.
pub(crate) const ITEM_TAGS: MolangFieldSpec = MolangFieldSpec::KEPT
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .tags()
    .empty(EmptyRule::BlockNoExpression);

/// A deprecated trigger's or event's `condition`: unrestricted, side effects allowed. The
/// same rule serves the `condition` of the trigger components (`on_fall_on`, `on_interact`,
/// `on_placed`, `on_player_destroyed`, `on_player_placing`, `on_step_off`, `on_step_on`), of
/// the definition triggers inside `queued_ticking` and `random_ticking`, and of the block's
/// `events`; in each the game drops a failed parse result and goes on.
pub(crate) const TRIGGER_CONDITION: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::LenientVersioned)
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .empty(EmptyRule::BlockNoExpression);

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::molang::visit_fields;
    use bedrock_addon_core::schema::misshapen_shorthand_rows;
    use bedrock_addon_core::versioning::Reading;

    /// The paths and specs the Molang walk of `value` visits.
    fn walked<T: bedrock_addon_core::molang::MolangFields>(
        value: &mut T,
    ) -> Vec<(String, MolangFieldSpec)> {
        let mut seen = Vec::new();
        visit_fields(value, &JsonPath::new(), |field| {
            seen.push((field.path.to_string(), *field.spec));
        });
        seen
    }

    fn from_json<T: bedrock_addon_core::schema::FromJson>(text: &str) -> T {
        use bedrock_addon_core::json::kit::{Node, Read};
        let value: Value = bedrock_addon_core::json::from_str_jsonc(text).expect("a text");
        let mut found = Diagnostics::default();
        let mut read = Read::new(Reading::OWN, &mut found);
        bedrock_addon_core::schema::read_document(&Node::new(&value, JsonPath::new()), &mut read)
            .expect("the payload reads")
    }

    #[test]
    fn the_faces_and_directions_keep_the_games_order_and_names() {
        assert_eq!(
            Direction::ALL,
            [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West
            ]
        );
        assert_eq!(Direction::NAMES, ["north", "south", "east", "west"]);
        assert_eq!(Facing::ALL[0], Facing::Down);
        assert_eq!(Facing::parse_name("west"), Ok(Facing::West));
    }

    #[test]
    fn an_absent_member_stands_for_the_games_default() {
        let full = BoundingBox::default();
        assert_eq!((full.origin(), full.size()), ([-8.0, 0.0, -8.0], [16.0; 3]));
        let legacy = CollisionBoundingBox::default();
        assert_eq!(
            (legacy.origin(), legacy.size()),
            ([-8.0, 0.0, -8.0], [16.0; 3])
        );
        assert_eq!(Leashable::default().offset(), [0.0, 0.25, 0.0]);
        let tick: Tick = from_json(r#"{"interval_range": [1, 2]}"#);
        assert!(tick.looping());
    }

    #[test]
    fn every_shorthand_row_reads_a_bare_scalar() {
        assert!(misshapen_shorthand_rows::<BlockComponent>().is_empty());
    }

    #[test]
    fn the_untagged_payloads_walk_their_object_forms_molang_under_the_blocks_specs() {
        let mut mining: DestructibleByMining = from_json(
            r#"{"seconds_to_destroy": 1, "item_specific_speeds": [
                {"item": "minecraft:stick", "destroy_speed": 1},
                {"item": {"tags": "q.any_tag('a')"}, "destroy_speed": 2}]}"#,
        );
        assert_eq!(
            walked(&mut mining),
            [("item_specific_speeds/1/item/tags".to_owned(), ITEM_TAGS)]
        );
        assert!(walked(&mut DestructibleByMining::Enabled(true)).is_empty());

        let mut geometry: Geometry = from_json(
            r#"{"identifier": "geometry.a", "bone_visibility": {"b": "q.block_state('s')"}}"#,
        );
        assert_eq!(
            walked(&mut geometry),
            [("bone_visibility/b".to_owned(), BONE_VISIBILITY)]
        );
        assert!(walked(&mut Geometry::Identifier("geometry.a".to_owned())).is_empty());

        let mut filter: PlacementFilter = from_json(
            r#"{"conditions": [{"allowed_faces": ["up"], "block_filter": [
                "minecraft:dirt", {"tags": "q.any_tag('stone')"}]}]}"#,
        );
        assert_eq!(
            walked(&mut filter),
            [(
                "conditions/0/block_filter/1/tags".to_owned(),
                DESCRIPTOR_TAGS
            )]
        );
    }
}
