//! Writing a structure back as little-endian NBT.
//!
//! The typed value is turned into the tree vanilla's writer produces (every compound in
//! `std::map` order, an empty list of End tags). A loaded file whose bytes differ from that
//! keeps its own tree in the structure's `Layout`, and the writer lays the new tree over it:
//! the file's tag order and empty-list tags are kept, and so is the tag of one of the format's
//! own integers (which vanilla reads with `getInt`) while it still reads as the typed value,
//! so an unchanged structure is written back byte for byte. Raw
//! compounds (entities, palette entries, block entities) and unknown tags are opaque: they are
//! written as the structure holds them.

use nbtx::{Compound, Named, Value, ValueList};

use super::{BlockPositionData, Contents, Palette, Structure, TickQueueEntry};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{EncodeError, Unrepresentable};

impl Structure {
    /// The structure's file: little-endian NBT with an empty root name, tags in vanilla's order.
    /// A loaded structure that was not changed is written back byte for byte.
    ///
    /// Fails (`NotRepresentable`) only for what NBT cannot hold: a string or tag name longer
    /// than 32 767 bytes, or containers nested deeper than 512.
    ///
    /// # Errors
    ///
    /// Returns an [`EncodeError`] when a string or tag name is longer than 32 767 bytes or containers
    /// nest deeper than 512.
    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        let tree = tree(self);
        let (root, trailing): (Named<Value>, &[u8]) = match &self.layout.source {
            Some(source) => (
                Named::new(source.name.clone(), patch(tree, &source.value)),
                &self.layout.trailing,
            ),
            None => (Named::new("", tree), &self.layout.trailing),
        };
        let mut out = encode(&root)?;
        out.extend_from_slice(trailing);
        Ok(out)
    }
}

/// Little-endian NBT of `root`.
pub(super) fn encode(root: &Named<Value>) -> Result<Vec<u8>, EncodeError> {
    nbtx::to_le_bytes(root).map_err(|_| EncodeError::NotRepresentable {
        at: JsonPath::new(),
        what: Unrepresentable::Unsupported,
    })
}

/// The tree vanilla's writer produces for `structure`.
pub(super) fn tree(structure: &Structure) -> Value {
    let mut root = structure.extra.clone();
    root.insert(
        "format_version".into(),
        Value::Int(structure.format_version),
    );
    if let Some(version) = structure.neighbor_aware_block_upgrade_version {
        root.insert(
            "neighbor_aware_block_upgrade_version".into(),
            Value::Int(version),
        );
    }
    root.insert("size".into(), ints(&structure.size));
    root.insert(
        "structure_world_origin".into(),
        ints(&structure.structure_world_origin),
    );
    root.insert("structure".into(), contents(&structure.structure));
    sorted(root)
}

/// `structure`.
fn contents(contents: &Contents) -> Value {
    let mut body = contents.extra.clone();
    let layers = contents
        .block_indices
        .iter()
        .map(|layer| list_of_ints(layer))
        .collect();
    body.insert("block_indices".into(), Value::List(ValueList::List(layers)));
    body.insert("entities".into(), compounds(&contents.entities));
    let palettes = contents
        .palette
        .iter()
        .map(|(name, palette)| (name.as_str().into(), palette_tree(palette)))
        .collect();
    body.insert("palette".into(), sorted(palettes));
    sorted(body)
}

/// One palette.
fn palette_tree(palette: &Palette) -> Value {
    let mut out = palette.extra.clone();
    out.insert("block_palette".into(), compounds(&palette.block_palette));
    let positions = palette
        .block_position_data
        .iter()
        .map(|(index, data)| (index.to_string().into(), position(data)))
        .collect();
    out.insert("block_position_data".into(), sorted(positions));
    sorted(out)
}

/// One entry of `block_position_data`.
fn position(data: &BlockPositionData) -> Value {
    let mut out = data.extra.clone();
    if let Some(entity) = &data.block_entity_data {
        out.insert("block_entity_data".into(), Value::Compound(entity.clone()));
    }
    if let Some(ticks) = &data.tick_queue_data {
        let ticks: Vec<Compound> = ticks.iter().map(tick).collect();
        out.insert("tick_queue_data".into(), compounds(&ticks));
    }
    sorted(out)
}

/// One scheduled tick.
fn tick(tick: &TickQueueEntry) -> Compound {
    let mut out = tick.extra.clone();
    if let Some(delay) = tick.tick_delay {
        out.insert("tick_delay".into(), Value::Int(delay));
    }
    out.sort_keys();
    out
}

/// A compound in `std::map` order.
fn sorted(mut compound: Compound) -> Value {
    compound.sort_keys();
    Value::Compound(compound)
}

/// A list of Ints; an empty one of End tags.
fn list_of_ints(values: &[i32]) -> ValueList {
    if values.is_empty() {
        ValueList::End
    } else {
        ValueList::Int(values.to_vec())
    }
}

/// [`list_of_ints`] as a tag.
fn ints(values: &[i32]) -> Value {
    Value::List(list_of_ints(values))
}

/// A list of compounds; an empty one of End tags.
fn compounds(values: &[Compound]) -> Value {
    Value::List(if values.is_empty() {
        ValueList::End
    } else {
        ValueList::Compound(values.to_vec())
    })
}

// ---- laying the new tree over the file's ------------------------------------------------

/// What a tag of the format is to the writer when it lays the new tree over the file's.
#[derive(Clone, Copy)]
enum Shape {
    /// A raw compound's or an unknown tag's value: written as the structure holds it.
    Opaque,
    /// An Int vanilla reads with `getInt`, which reads a tag of any other type as 0: such a
    /// file tag is kept while the value is still 0.
    Int,
    /// A list of this shape: the file's tag when both are empty, element by element when the
    /// lengths agree.
    List(&'static Shape),
    /// A compound whose members have the shape this gives for their name: the file's order.
    Compound(fn(&[u8]) -> Shape),
}

const INT: Shape = Shape::Int;
const INTS: Shape = Shape::List(&INT);
const OPAQUE: Shape = Shape::Opaque;
const TICK: Shape = Shape::Compound(tick_member);

/// The root's members.
fn root_member(name: &[u8]) -> Shape {
    match name {
        // `format_version` is the Int 1 in every file that loaded: nothing to keep.
        b"size" | b"structure_world_origin" => INTS,
        b"structure" => Shape::Compound(contents_member),
        _ => Shape::Opaque,
    }
}

/// The members of `structure`.
fn contents_member(name: &[u8]) -> Shape {
    match name {
        b"block_indices" => Shape::List(&INTS),
        b"entities" => Shape::List(&OPAQUE),
        b"palette" => Shape::Compound(|_| Shape::Compound(palette_member)),
        _ => Shape::Opaque,
    }
}

/// The members of one palette.
fn palette_member(name: &[u8]) -> Shape {
    match name {
        b"block_palette" => Shape::List(&OPAQUE),
        b"block_position_data" => Shape::Compound(|_| Shape::Compound(position_member)),
        _ => Shape::Opaque,
    }
}

/// The members of one entry of `block_position_data`.
fn position_member(name: &[u8]) -> Shape {
    match name {
        b"tick_queue_data" => Shape::List(&TICK),
        _ => Shape::Opaque,
    }
}

/// The members of one scheduled tick.
fn tick_member(name: &[u8]) -> Shape {
    match name {
        b"tick_delay" => Shape::Int,
        _ => Shape::Opaque,
    }
}

/// The new root laid over the file's root.
fn patch(new: Value, old: &Value) -> Value {
    lay(new, old, Shape::Compound(root_member))
}

/// `new` laid over the file's `old` tag, which has `shape`: the file's layout wherever it
/// still reads as `new`; a raw compound or an unknown tag is `new` as held.
fn lay(new: Value, old: &Value, shape: Shape) -> Value {
    match (shape, new, old) {
        (Shape::Int, Value::Int(0), old) if !matches!(old, Value::Int(_)) => old.clone(),
        (Shape::List(element), Value::List(new), Value::List(old)) => {
            Value::List(lay_list(new, old, *element))
        }
        (Shape::Compound(member), Value::Compound(new), Value::Compound(old)) => {
            Value::Compound(lay_compound(new, old, member))
        }
        (_, new, _) => new,
    }
}

/// The file's tags in the file's order, each laid over; then the new tags in their order.
///
/// Linear in the members: each new member is taken from its slot by index, never shifted.
fn lay_compound(new: Compound, old: &Compound, member: fn(&[u8]) -> Shape) -> Compound {
    let places: Vec<Option<usize>> = old.keys().map(|name| new.get_index_of(name)).collect();
    let mut slots: Vec<Option<Value>> = Vec::with_capacity(new.len());
    let mut names = Vec::with_capacity(new.len());
    for (name, value) in new {
        names.push(Some(name));
        slots.push(Some(value));
    }
    let mut out = Compound::with_capacity(slots.len());
    for ((name, old_value), place) in old.iter().zip(places) {
        let Some(index) = place else { continue };
        if let (Some(key), Some(value)) = (names[index].take(), slots[index].take()) {
            out.insert(key, lay(value, old_value, member(name)));
        }
    }
    out.extend(
        names
            .into_iter()
            .zip(slots)
            .filter_map(|(name, value)| Some((name?, value?))),
    );
    out
}

/// The file's list where it is equal or both are empty (an empty list keeps its tag); element
/// by element where both have the same length and the result is still one tag type; else the
/// new list.
fn lay_list(new: ValueList, old: &ValueList, element: Shape) -> ValueList {
    if new.is_empty() && old.is_empty() {
        return old.clone();
    }
    if matches!(element, Shape::Opaque) || new.len() != old.len() {
        return new;
    }
    if new == *old {
        return old.clone();
    }
    let laid: Vec<Value> = new
        .clone()
        .into_values()
        .into_iter()
        .zip(old.iter())
        .map(|(new, old)| lay(new, &old, element))
        .collect();
    ValueList::try_from(laid).unwrap_or(new)
}
