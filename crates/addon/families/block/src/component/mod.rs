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
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, CustomComponent};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::{Experiment, LoadContext, LoadRequirement};
use bedrock_addon_core::molang::{
    BLOCK_STATE_QUERIES, EmptyRule, MolangFieldSpec, MolangVersion, MolangVersionSource, OpsRule,
    ReaderKind,
};
use bedrock_addon_core::versioning::{ConvertMode, parse_component};

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
    #[component(name = "minecraft:block_entity", fields = [{ name = "", since_engine = "1.26.30" }], parse = parse_component::<BlockEntity, BlockComponent>)]
    BlockEntity(BlockEntity),
    /// `minecraft:chest_obstruction`.
    #[component(name = "minecraft:chest_obstruction", released = "1.26.20", requires = [UCF], parse = parse_component::<ChestObstruction, BlockComponent>)]
    ChestObstruction(ChestObstruction),
    /// `minecraft:collision_box`.
    #[component(name = "minecraft:collision_box", shorthand = scalar, released = "1.19.50", requires = [LCF], parse = parse_collision_box)]
    CollisionBox(CollisionBox),
    /// `minecraft:connection_rule`.
    #[component(name = "minecraft:connection_rule", released = "1.26.0", requires = [UCF], parse = parse_component::<ConnectionRule, BlockComponent>)]
    ConnectionRule(ConnectionRule),
    /// `minecraft:crafting_table`.
    #[component(name = "minecraft:crafting_table", released = "1.19.50", requires = [LCF], parse = parse_component::<CraftingTable, BlockComponent>)]
    CraftingTable(CraftingTable),
    /// `minecraft:destructible_by_explosion`.
    #[component(name = "minecraft:destructible_by_explosion", shorthand = scalar, parse = parse_component::<DestructibleByExplosion, BlockComponent>)]
    DestructibleByExplosion(DestructibleByExplosion),
    /// `minecraft:destructible_by_mining`.
    #[component(name = "minecraft:destructible_by_mining", shorthand = scalar, molang, parse = parse_component::<DestructibleByMining, BlockComponent>)]
    DestructibleByMining(DestructibleByMining),
    /// `minecraft:destruction_particles` (format 1.21.70, checked by the family).
    #[component(name = "minecraft:destruction_particles", parse = parse_component::<DestructionParticles, BlockComponent>)]
    DestructionParticles(DestructionParticles),
    /// `minecraft:display_name`.
    #[component(name = "minecraft:display_name", released = "1.19.60", requires = [LCF], parse = parse_component::<DisplayName, BlockComponent>)]
    DisplayName(DisplayName),
    /// `minecraft:embedded_visual`.
    #[component(name = "minecraft:embedded_visual", released = "1.21.120", requires = [UCF], molang, parse = parse_embedded_visual)]
    EmbeddedVisual(EmbeddedVisual),
    /// `minecraft:enchantment_power_transmitter` (1.26.60). Behind Upcoming Creator Features
    /// by the validator of 1.26.60, and no member of a legacy document: the family gates it itself.
    #[component(name = "minecraft:enchantment_power_transmitter", parse = parse_component::<EnchantmentPowerTransmitter, BlockComponent>)]
    EnchantmentPowerTransmitter(EnchantmentPowerTransmitter),
    /// `minecraft:entity_fall_on`.
    #[component(name = "minecraft:entity_fall_on", parse = parse_component::<EntityFallOn, BlockComponent>)]
    EntityFallOn(EntityFallOn),
    /// `minecraft:flammable`.
    #[component(name = "minecraft:flammable", shorthand = scalar, fields = [{ name = "lava_flammable", since_engine = "1.26.30" }], parse = parse_component::<Flammable, BlockComponent>)]
    Flammable(Flammable),
    /// `minecraft:flower_pottable`.
    #[component(name = "minecraft:flower_pottable", released = "1.21.120", requires = [UCF], parse = parse_component::<FlowerPottable, BlockComponent>)]
    FlowerPottable(FlowerPottable),
    /// `minecraft:friction`.
    #[component(name = "minecraft:friction", parse = parse_component::<Friction, BlockComponent>)]
    Friction(Friction),
    /// `minecraft:geometry`.
    #[component(name = "minecraft:geometry", shorthand = scalar, released = "1.19.40", requires = [LCF], fields = [{ name = "n_way_visual_rotation", since_engine = "1.26.30" }], molang, parse = parse_component::<Geometry, BlockComponent>)]
    Geometry(Geometry),
    /// `minecraft:instrument_sound`. No member of a legacy document, ungated from 1.26.20.
    #[component(name = "minecraft:instrument_sound", fields = [{ name = "", since_engine = "1.26.30" }], parse = parse_component::<InstrumentSound, BlockComponent>)]
    InstrumentSound(InstrumentSound),
    /// `minecraft:item_visual`.
    #[component(name = "minecraft:item_visual", released = "1.21.50", requires = [UCF], molang, parse = parse_item_visual)]
    ItemVisual(ItemVisual),
    /// `minecraft:leashable`.
    #[component(name = "minecraft:leashable", released = "1.26.0", requires = [UCF], parse = parse_component::<Leashable, BlockComponent>)]
    Leashable(Leashable),
    /// `minecraft:light_dampening`.
    #[component(name = "minecraft:light_dampening", parse = parse_component::<LightDampening, BlockComponent>)]
    LightDampening(LightDampening),
    /// `minecraft:light_emission`.
    #[component(name = "minecraft:light_emission", parse = parse_component::<LightEmission, BlockComponent>)]
    LightEmission(LightEmission),
    /// `minecraft:liquid_detection`.
    #[component(name = "minecraft:liquid_detection", released = "1.21.60", requires = [UCF], parse = parse_component::<LiquidDetection, BlockComponent>)]
    LiquidDetection(LiquidDetection),
    /// `minecraft:loot`.
    #[component(name = "minecraft:loot", parse = parse_component::<Loot, BlockComponent>)]
    Loot(Loot),
    /// `minecraft:map_color`.
    #[component(name = "minecraft:map_color", shorthand = scalar, parse = parse_component::<MapColor, BlockComponent>)]
    MapColor(MapColor),
    /// `minecraft:material_instances`.
    #[component(name = "minecraft:material_instances", released = "1.19.40", requires = [LCF], parse = parse_material_instances)]
    MaterialInstances(MaterialInstances),
    /// `minecraft:movable`.
    #[component(name = "minecraft:movable", released = "1.21.100", requires = [UCF], parse = parse_component::<Movable, BlockComponent>)]
    Movable(Movable),
    /// `minecraft:neighbor_change` (1.26.60). Behind Upcoming Creator Features by the
    /// validator of 1.26.60, and no member of a legacy document: the family gates it itself.
    #[component(name = "minecraft:neighbor_change", parse = parse_component::<NeighborChange, BlockComponent>)]
    NeighborChange(NeighborChange),
    /// `minecraft:placement_filter`.
    #[component(name = "minecraft:placement_filter", released = "1.19.60", requires = [LCF], molang, parse = parse_component::<PlacementFilter, BlockComponent>)]
    PlacementFilter(PlacementFilter),
    /// `minecraft:precipitation_interactions`.
    #[component(name = "minecraft:precipitation_interactions", released = "1.21.120", requires = [UCF], parse = parse_component::<PrecipitationInteractions, BlockComponent>)]
    PrecipitationInteractions(PrecipitationInteractions),
    /// `minecraft:random_offset`.
    #[component(name = "minecraft:random_offset", released = "1.21.100", requires = [UCF], parse = parse_component::<RandomOffset, BlockComponent>)]
    RandomOffset(RandomOffset),
    /// `minecraft:redstone_conductivity`.
    #[component(name = "minecraft:redstone_conductivity", released = "1.21.30", requires = [UCF], parse = parse_component::<RedstoneConductivity, BlockComponent>)]
    RedstoneConductivity(RedstoneConductivity),
    /// `minecraft:redstone_consumer`.
    #[component(name = "minecraft:redstone_consumer", released = "1.26.0", requires = [UCF], parse = parse_component::<RedstoneConsumer, BlockComponent>)]
    RedstoneConsumer(RedstoneConsumer),
    /// `minecraft:redstone_producer`.
    #[component(name = "minecraft:redstone_producer", released = "1.21.120", requires = [UCF], parse = parse_component::<RedstoneProducer, BlockComponent>)]
    RedstoneProducer(RedstoneProducer),
    /// `minecraft:replaceable`.
    #[component(name = "minecraft:replaceable", released = "1.21.60", requires = [UCF], parse = parse_component::<Replaceable, BlockComponent>)]
    Replaceable(Replaceable),
    /// `minecraft:selection_box`.
    #[component(name = "minecraft:selection_box", shorthand = scalar, released = "1.19.60", requires = [LCF], parse = parse_component::<SelectionBox, BlockComponent>)]
    SelectionBox(SelectionBox),
    /// `minecraft:sound`. Its legacy row is released at 1.26.40 behind Upcoming Creator
    /// Features (the family's own gate below 1.26.20); ungated from 1.26.20.
    #[component(name = "minecraft:sound", fields = [{ name = "", since_engine = "1.26.50" }], parse = parse_component::<Sound, BlockComponent>)]
    Sound(Sound),
    /// `minecraft:support`.
    #[component(name = "minecraft:support", released = "1.26.0", requires = [UCF], parse = parse_component::<Support, BlockComponent>)]
    Support(Support),
    /// `minecraft:tags`.
    #[component(name = "minecraft:tags", parse = parse_component::<Tags, BlockComponent>)]
    Tags(Tags),
    /// `minecraft:tick`.
    #[component(name = "minecraft:tick", parse = parse_component::<Tick, BlockComponent>)]
    Tick(Tick),
    /// `minecraft:transformation`.
    #[component(name = "minecraft:transformation", released = "1.19.80", requires = [LCF], parse = parse_component::<Transformation, BlockComponent>)]
    Transformation(Transformation),
    /// `minecraft:vibration_properties` (1.26.60). Behind Upcoming Creator Features, by the
    /// validator of 1.26.60 and below 1.26.20 by its legacy row (released 1.26.60): the
    /// family gates it itself.
    #[component(name = "minecraft:vibration_properties", parse = parse_component::<VibrationProperties, BlockComponent>)]
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
        parse_component::<OneCollisionBox, BlockComponent>(slice, mode, cx, d)
    } else {
        parse_component::<CollisionBox, BlockComponent>(slice, mode, cx, d)
    }
}

/// One collision box written as an object.
#[derive(facet::Facet)]
#[facet(transparent)]
struct OneCollisionBox(CollisionBoundingBox);

impl From<OneCollisionBox> for BlockComponent {
    fn from(one: OneCollisionBox) -> Self {
        BlockComponent::CollisionBox(CollisionBox::Boxes(vec![one.0]))
    }
}

/// `minecraft:material_instances`, whose instances keep their unknown members.
fn parse_material_instances(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent> {
    let mut value: MaterialInstances = parse_typed(slice, mode, cx, d)?;
    keep_instances(&mut value, slice, None, is_cereal(cx), d);
    Some(value.into())
}

/// `minecraft:item_visual`, whose instances keep their unknown members.
fn parse_item_visual(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent> {
    let mut value: ItemVisual = parse_typed(slice, mode, cx, d)?;
    keep_instances(
        &mut value.material_instances,
        slice,
        Some("material_instances"),
        is_cereal(cx),
        d,
    );
    Some(value.into())
}

/// `minecraft:embedded_visual`, whose instances keep their unknown members.
fn parse_embedded_visual(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<BlockComponent> {
    let mut value: EmbeddedVisual = parse_typed(slice, mode, cx, d)?;
    keep_instances(
        &mut value.material_instances,
        slice,
        Some("material_instances"),
        is_cereal(cx),
        d,
    );
    Some(value.into())
}

/// `parse_component` for a type that is then given back to the caller.
fn parse_typed<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T>
where
    T: for<'a> facet::Facet<'a>
        + Into<BlockComponent>
        + bedrock_addon_core::document::Component<Set = BlockComponent>,
{
    let component = parse_component::<T, BlockComponent>(slice, mode, cx, d)?;
    <T as bedrock_addon_core::document::Component>::try_from_set(component).ok()
}

/// Whether the load is a cereal slice's (rule N2): the legacy slice has no null rule.
fn is_cereal(cx: &LoadContext) -> bool {
    cx.null_rule == Some(bedrock_addon_core::json::NullRule::Rejected)
}

/// Keeps the unknown members of each material instance (below the member `within`, if any)
/// and reports them: in a cereal document the marker the strict mask turns into the error
/// that refuses the block; in a legacy one the game says nothing of an unknown member, so the
/// crate reports it as an information.
fn keep_instances(
    instances: &mut MaterialInstances,
    slice: JsonSlice<'_>,
    within: Option<&str>,
    cereal: bool,
    d: &mut Diagnostics,
) {
    let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text) else {
        return;
    };
    let json = match within {
        Some(key) => match json.as_object().and_then(|o| o.get(key)) {
            Some(inner) => inner.clone(),
            None => return,
        },
        None => json,
    };
    let mut at = JsonPath::new();
    if let Some(key) = within {
        at.push(key);
    }
    for (instance, member) in instances.keep_unknown(&json, &at, d.unknown_mut()) {
        let mut path = JsonPath::new();
        if let Some(key) = within {
            path.push(key);
        }
        path.push(&instance);
        path.push(&member);
        d.push(
            Diagnostic::error(DiagnosticCode::UnrecognizedField { text: None })
                .severity(if cereal {
                    Severity::Warning
                } else {
                    Severity::Info
                })
                .at(path),
        );
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
