//! The references of an entity to other definitions (the link half): the members of its
//! components that name a block, an item, an entity, a loot or trade table, a sound, a
//! particle or a point of interest, and the particles its events emit.
//!
//! The members are read in the newest format's shape: a typed component is written back to
//! JSON once, when [`Entity::references`] is called (a component that cannot be written is
//! left out: it holds a value the format cannot express, which the load has told); a
//! component the game reads at run time is the JSON the file has. A custom component is a
//! script's and a key no table knows has no member that names a definition: neither is
//! walked. Each path is the member's place in the entity as it is held, from the payload key
//! on; a document maps it to the place in its file.

mod table;

use std::borrow::Cow;

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::id::{BlockId, EntityId};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::Family;
use bedrock_addon_shared_shapes::{BlockDescriptor, ItemDescriptor, ItemDescriptorForm};

use crate::Entity;
use crate::component::EntityComponent;
use crate::references::{event_nodes, members};
use table::Row;

/// What a member names: the tables of [`References::of`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ReferenceKind {
    /// A block: a block descriptor, or a name of a list of block names.
    Block,
    /// A block a goal reads through an item descriptor (every block has an item of its name).
    BlockAsItem,
    /// A loot table, by its path.
    LootTable,
    /// A trade table, by its path.
    TradeTable,
    /// An entity identifier, with or without an `<event>`; an offspring pair; a mate of
    /// `breeds_with`.
    Entity,
    /// An item: a name or an item descriptor.
    Item,
    /// A level sound event, by its name.
    Sound,
    /// A particle type of the game or a particle effect.
    Particle,
    /// A point of interest: a type or a tag.
    Poi,
}

impl ReferenceKind {
    /// The tables of the kind, walked as one in this order: within one component, the rows of
    /// the first table come first.
    fn tables(self) -> &'static [&'static [Row]] {
        match self {
            Self::Block => &[table::BLOCKS, table::BLOCK_NAMES],
            Self::BlockAsItem => &[table::BLOCKS_AS_ITEMS],
            Self::LootTable => &[table::LOOT_TABLES],
            Self::TradeTable => &[table::TRADE_TABLES],
            Self::Entity => &[table::ENTITIES],
            Self::Item => &[table::ITEMS],
            Self::Sound => &[table::SOUNDS],
            Self::Particle => &[table::PARTICLES],
            Self::Poi => &[table::POIS],
        }
    }
}

/// One member that holds a reference.
#[derive(Clone, Debug, PartialEq)]
pub struct Referenced<'a> {
    /// The key of the component that holds the member (`minecraft:loot`), or `emit_particle`
    /// for the response of an event node.
    pub component: &'a str,
    /// The member's path in the entity as it is held: for [`Reference::Offspring`] and
    /// [`Reference::Mate`], the pair's or the mate's member below the map.
    pub at: JsonPath,
    /// What the member names.
    pub reference: Reference<'a>,
}

/// What a member names, read the way the game reads that member. A name is never empty: an
/// empty name is what a member that was not written reads as, and names nothing.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Reference<'a> {
    /// A block descriptor: a name, `{name, states}` or `{tags}`.
    Block(BlockDescriptor),
    /// A name of a list of block names (`home.home_block_list`, the `raid_garden` goal's
    /// `blocks`): the game binds these as names, not as deferred descriptors; it may be empty.
    BlockName(BlockId),
    /// A block a goal reads through an item descriptor: a name that is a block or an item
    /// resolves.
    BlockAsItem(ItemDescriptor),
    /// An entity identifier, with or without an `<event>`: `name` is the member's text as
    /// written (it compares without the suffix) and `event` the event the suffix names.
    Entity {
        /// The identifier as written.
        name: EntityId,
        /// The event of an `ns:name<ns:event>` text.
        event: Option<&'a str>,
    },
    /// A pair of `offspring.offspring_pairs`: a mate's identifier (the key) and the child's
    /// (the value), each when it is written.
    Offspring {
        /// The mate, when the key is not empty.
        mate: Option<EntityId>,
        /// The child, when the value is a name.
        child: Option<EntityId>,
    },
    /// A mate of `breedable.breeds_with` (the 1.21.130+ map of a mate to its breed event):
    /// the key.
    Mate(EntityId),
    /// An item: a name (with the legacy `:aux`) or a descriptor object (`{item}`, `{name}`,
    /// `{item_tag}`, `{tags}`). `or_item_tag` when the member reads an item or an item tag
    /// there (`shareables.items[].item`: the game's own fox shares `minecraft:is_food`).
    Item {
        /// The descriptor.
        descriptor: ItemDescriptor,
        /// The name may be an item tag.
        or_item_tag: bool,
    },
    /// A loot table, by its path as written.
    LootTable(&'a str),
    /// A trade table, by its path as written.
    TradeTable(&'a str),
    /// The name of a level sound event as a plain string.
    Sound(&'a str),
    /// A particle: `effect` when the member holds an effect name the client spawns an emitter
    /// for (`explode.particle_effect`, an event's `emit_particle.particle`), not a particle
    /// type of the game (the enumeration the other members hold).
    Particle {
        /// The name as written.
        name: &'a str,
        /// An effect name, not a particle type.
        effect: bool,
    },
    /// The name of a point of interest: a type or a tag.
    Poi(&'a str),
}

/// One component of an entity as JSON in the newest format's shape.
#[derive(Debug)]
struct Component<'a> {
    /// The component's key.
    key: &'a str,
    /// Its path in the entity.
    at: JsonPath,
    /// Its value.
    json: Cow<'a, Value>,
}

/// The members of an entity that hold a reference, read from its components written back to
/// JSON once ([`Entity::references`]).
#[derive(Debug)]
pub struct References<'a> {
    /// The entity.
    entity: &'a Entity,
    /// Every component of `components` and of each component group, in document order.
    components: Vec<Component<'a>>,
}

impl<'a> References<'a> {
    /// Every member that holds a reference of `kind`, in document order; within one component,
    /// in the order of the kind's rows.
    #[must_use]
    pub fn of(&self, kind: ReferenceKind) -> Vec<Referenced<'_>> {
        let mut out = Vec::new();
        for component in &self.components {
            for table in kind.tables() {
                for row in rows(table, component.key) {
                    members(&component.json, row.1, &component.at, &mut |at, value| {
                        read(kind, row, component.key, at, value, &mut out);
                    });
                }
            }
        }
        out
    }

    /// The particle every `emit_particle` response of the entity's events emits (in an event's
    /// body or in any node nested in it), in document order: an effect name.
    #[must_use]
    pub fn emitted_particles(&self) -> Vec<Referenced<'a>> {
        let mut out = Vec::new();
        for (at, node) in self.entity.event_nodes() {
            members(node, "emit_particle.particle", &at, &mut |at, value| {
                if let Some(name) = name(value) {
                    out.push(Referenced {
                        component: EMIT_PARTICLE,
                        at,
                        reference: Reference::Particle { name, effect: true },
                    });
                }
            });
        }
        out
    }
}

/// The event response that emits a particle.
const EMIT_PARTICLE: &str = "emit_particle";

/// The component whose member holds an effect name, not a particle type.
const EXPLODE: &str = "minecraft:explode";

/// The component whose `items[].item` is an item or an item tag.
const SHAREABLES: &str = "minecraft:shareables";

/// The name a member holds: its string, unless it is empty.
fn name(value: &Value) -> Option<&str> {
    value
        .as_string()
        .map(facet_value::VString::as_str)
        .filter(|name| !name.is_empty())
}

/// What the member `value` at `at` of `component`, named by `row` of a table of `kind`, names:
/// pushed onto `out`, none, one, or one per pair of a map.
fn read<'v>(
    kind: ReferenceKind,
    row: &Row,
    component: &'v str,
    at: JsonPath,
    value: &'v Value,
    out: &mut Vec<Referenced<'v>>,
) {
    let mut push = |at: JsonPath, reference: Reference<'v>| {
        out.push(Referenced {
            component,
            at,
            reference,
        });
    };
    let reference = match kind {
        ReferenceKind::Block if table::BLOCK_NAMES.contains(row) => {
            // What is no string there, the load has refused.
            let Some(text) = value.as_string() else {
                return;
            };
            Reference::BlockName(BlockId::parse(text.as_str()))
        }
        ReferenceKind::Block => Reference::Block(BlockDescriptor::from(value.clone())),
        ReferenceKind::BlockAsItem => Reference::BlockAsItem(ItemDescriptor::from(value.clone())),
        ReferenceKind::Entity => {
            if let Some(text) = name(value) {
                let name = EntityId::parse(text);
                let event = text
                    .find('<')
                    .and_then(|open| text[open + 1..].strip_suffix('>'));
                Reference::Entity { name, event }
            } else if *row == table::OFFSPRING {
                for (mate, child) in value.as_object().into_iter().flatten() {
                    let mate_name = Some(mate.as_str()).filter(|mate| !mate.is_empty());
                    let reference = Reference::Offspring {
                        mate: mate_name.map(EntityId::parse),
                        child: name(child).map(EntityId::parse),
                    };
                    push(at.child(mate.as_str()), reference);
                }
                return;
            } else if *row == table::BREEDS_WITH {
                // The 1.21.130+ map: each key is a mate.
                for (mate, _) in value.as_object().into_iter().flatten() {
                    if !mate.as_str().is_empty() {
                        let reference = Reference::Mate(EntityId::parse(mate.as_str()));
                        push(at.child(mate.as_str()), reference);
                    }
                }
                return;
            } else {
                return;
            }
        }
        ReferenceKind::Item => {
            if value
                .as_string()
                .is_some_and(|text| text.as_str().is_empty())
            {
                return;
            }
            let descriptor = ItemDescriptor::from(value.clone());
            let or_item_tag = component == SHAREABLES
                && matches!(
                    descriptor,
                    ItemDescriptor::Item {
                        form: ItemDescriptorForm::Item,
                        ..
                    }
                );
            Reference::Item {
                descriptor,
                or_item_tag,
            }
        }
        ReferenceKind::LootTable => match name(value) {
            Some(path) => Reference::LootTable(path),
            None => return,
        },
        ReferenceKind::TradeTable => match name(value) {
            Some(path) => Reference::TradeTable(path),
            None => return,
        },
        ReferenceKind::Sound => match name(value) {
            Some(name) => Reference::Sound(name),
            None => return,
        },
        ReferenceKind::Particle => match name(value) {
            Some(name) => Reference::Particle {
                name,
                effect: component == EXPLODE,
            },
            None => return,
        },
        ReferenceKind::Poi => match name(value) {
            Some(name) => Reference::Poi(name),
            None => return,
        },
    };
    push(at, reference);
}

/// The rows of the sorted `table` for the component `key`.
fn rows<'t>(table: &'t [Row], key: &str) -> &'t [Row] {
    let from = table.partition_point(|(component, _)| *component < key);
    let len = table[from..].partition_point(|(component, _)| *component == key);
    &table[from..from + len]
}

/// The sections of an entity that hold components: `components` and each component group,
/// with the path of each.
fn sections(entity: &Entity) -> impl Iterator<Item = (JsonPath, &ComponentMap<EntityComponent>)> {
    let payload = JsonPath::new().child(<Entity as Family>::PAYLOAD_KEY);
    let own = std::iter::once((payload.child("components"), &entity.components));
    let groups_at = payload.child("component_groups");
    let groups = entity
        .component_groups
        .iter()
        .map(move |(name, group)| (groups_at.child(name), &group.components));
    own.chain(groups)
}

impl Entity {
    /// The members of the entity's components that hold a reference to another definition:
    /// each typed component is written back to JSON here, once, and [`References::of`] reads
    /// every kind from that.
    #[must_use]
    pub fn references(&self) -> References<'_> {
        let mut components = Vec::new();
        for (section, map) in sections(self) {
            for entry in map.iter() {
                let (key, json) = match entry {
                    ComponentRef::Typed { name, value, .. } => match crate::write::component(value)
                    {
                        Ok(json) => (name, Cow::Owned(json)),
                        Err(_) => continue,
                    },
                    ComponentRef::Dynamic { name, json } => (name, Cow::Borrowed(json)),
                    _ => continue,
                };
                let at = section.child(key);
                components.push(Component { key, at, json });
            }
        }
        References {
            entity: self,
            components,
        }
    }

    /// Every node of the entity's events with its path, from the payload key on: each event's
    /// body and each node nested in its `sequence`, `randomize` or `first_valid`, in document
    /// order.
    #[must_use]
    pub fn event_nodes(&self) -> Vec<(JsonPath, &Value)> {
        let events_at = JsonPath::new()
            .child(<Entity as Family>::PAYLOAD_KEY)
            .child("events");
        self.events
            .values()
            .flat_map(|event| event_nodes(&event.node, events_at.child(event.name())))
            .collect()
    }
}

#[cfg(test)]
mod tests;
