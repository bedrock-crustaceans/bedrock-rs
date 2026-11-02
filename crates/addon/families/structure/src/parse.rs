//! Reading a structure file: little-endian NBT, then the game's load steps in its order, each
//! failure ending the load with vanilla's text.

use std::collections::BTreeMap;

use bedrock_addon_core::load::LoadContext;
use indexmap::IndexMap;
use nbtx::{Compound, Named, Value, ValueList};
use std::sync::Arc;

use super::{BlockPositionData, Contents, Layout, Palette, Structure, TickQueueEntry, text};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};

/// A step's result: the value, or the finding that ends the load (boxed: one per file).
type Step<T> = Result<T, Box<Diagnostic>>;

/// Reads `bytes` as a structure: the structure, or `None` with the finding that failed it.
pub(crate) fn read(bytes: &[u8], cx: &LoadContext) -> (Option<Structure>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let result = decode(bytes).and_then(|(file, read)| {
        let Value::Compound(root) = &file.value else {
            return Err(malformed("the root tag is not a compound".to_owned()));
        };
        let mut structure = structure(root, cx)?;
        structure.layout = layout(&structure, file, &bytes[..read], &bytes[read..]);
        Ok(structure)
    });
    match result {
        Ok(structure) => {
            diagnostics.extend(super::validate::index_notes(&structure));
            (Some(structure), diagnostics)
        }
        Err(finding) => {
            diagnostics.push(*finding);
            (None, diagnostics)
        }
    }
}

/// The root tag of little-endian NBT bytes and how many bytes it took; bytes after it are not
/// read.
fn decode(bytes: &[u8]) -> Step<(Named<Value>, usize)> {
    let mut reader = bytes;
    match nbtx::from_le_bytes::<Named<Value>>(&mut reader) {
        Ok(root) => Ok((root, bytes.len() - reader.len())),
        Err(error) => Err(malformed(error.to_string())),
    }
}

/// Bytes that are not an NBT compound. The game reads them as an empty root, which fails the
/// `format_version` step; the crate says the input is malformed, with that text.
fn malformed(message: String) -> Box<Diagnostic> {
    fail(
        DiagnosticCode::MalformedInput {
            message,
            text: Some(text::required("format_version")),
        },
        &JsonPath::new(),
    )
}

/// What the writer needs to reproduce the file: its tree when the typed value's own tree
/// encodes differently, and the bytes after the root.
fn layout(structure: &Structure, file: Named<Value>, read: &[u8], trailing: &[u8]) -> Layout {
    let own = super::write::encode(&Named::new("", super::write::tree(structure)));
    let source = (own.ok().as_deref() != Some(read)).then(|| Arc::new(file));
    Layout {
        source,
        trailing: trailing.into(),
    }
}

/// The steps of the structure load.
fn structure(root: &Compound, cx: &LoadContext) -> Step<Structure> {
    let format_version = format_version(root)?;
    let neighbor_aware_block_upgrade_version = neighbor_aware_version(root, cx)?;
    let size = size(root)?;
    let structure_world_origin = triple(root, "structure_world_origin")?;
    // A missing `structure` tag returns no code: the load just ends.
    let Some(body) = compound(root, "structure") else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required("structure")),
            },
            &path(&["structure"]),
        ));
    };
    let structure = contents(body, size)?;
    Ok(Structure {
        format_version,
        neighbor_aware_block_upgrade_version,
        size,
        structure_world_origin,
        structure,
        extra: extra(
            root,
            &[
                "format_version",
                "neighbor_aware_block_upgrade_version",
                "size",
                "structure_world_origin",
                "structure",
            ],
        ),
        layout: super::Layout::default(),
    })
}

/// The format version is read as an Int, so absent, 0 and any tag other than Int are the
/// required-field error; anything else but the current format version is larger or smaller
/// than it.
fn format_version(root: &Compound) -> Step<i32> {
    const FIELD: &str = "format_version";
    let version = int(root, FIELD);
    let current = Structure::FORMAT_VERSION;
    let text = match version {
        0 => {
            return Err(fail(
                DiagnosticCode::MissingRequiredField {
                    text: Some(text::required(FIELD)),
                },
                &path(&[FIELD]),
            ));
        }
        v if v > current => text::larger_version(FIELD, current, v),
        v if v < 0 => text::smaller_version(FIELD, current, v),
        v => return Ok(v),
    };
    Err(fail(
        DiagnosticCode::UnsupportedVersion { text: Some(text) },
        &path(&[FIELD]),
    ))
}

/// The neighbor-aware block upgrade version: optional; an Int in
/// `0..=latest` (an unsigned compare, so negatives fail too).
fn neighbor_aware_version(root: &Compound, cx: &LoadContext) -> Step<Option<i32>> {
    const FIELD: &str = "neighbor_aware_block_upgrade_version";
    let text = match root.get(key(FIELD)) {
        None => return Ok(None),
        Some(Value::Int(v)) => {
            let latest = neighbor_aware_latest(cx);
            if (0..=latest).contains(v) {
                return Ok(Some(*v));
            }
            text::invalid_value(FIELD, *v, 0, latest)
        }
        Some(_) => text::not_an_integer(FIELD),
    };
    Err(fail(
        DiagnosticCode::InvalidValue { text: Some(text) },
        &path(&[FIELD]),
    ))
}

/// The latest neighbor-aware block upgrade version of the targeted engine. The crate does not
/// know which release raised it, so every engine version gets 1 (`WallBlock`).
fn neighbor_aware_latest(_: &LoadContext) -> i32 {
    1
}

/// The size: three Ints, none negative; no upper bound.
fn size(root: &Compound) -> Step<[i32; 3]> {
    const FIELD: &str = "size";
    let size = triple(root, FIELD)?;
    if size.iter().any(|v| *v < 0) {
        return Err(fail(
            DiagnosticCode::InvalidValue {
                text: Some(text::negative(FIELD)),
            },
            &path(&[FIELD]),
        ));
    }
    Ok(size)
}

/// `structure`: block indices, palettes, entities.
fn contents(body: &Compound, size: [i32; 3]) -> Step<Contents> {
    let block_indices = block_indices(body)?;
    check_volume(&block_indices, size)?;
    let palette = palettes(body)?;
    let entities = entities(body)?;
    Ok(Contents {
        block_indices,
        entities,
        palette,
        extra: extra(body, &["block_indices", "entities", "palette"]),
    })
}

/// The block indices: a List of exactly two Lists of equal length, each element read as an
/// Int.
fn block_indices(body: &Compound) -> Step<[Vec<i32>; 2]> {
    const FIELD: &str = "block_indices";
    let invalid = |text| {
        fail(
            DiagnosticCode::InvalidValue { text: Some(text) },
            &path(&["structure", FIELD]),
        )
    };
    let Some(Value::List(layers)) = body.get(key(FIELD)) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(FIELD)),
            },
            &path(&["structure", FIELD]),
        ));
    };
    if layers.len() != 2 {
        return Err(invalid(text::two_arrays(FIELD, layers.len())));
    }
    // A list holds one tag type, so the two layers are Lists or neither is: vanilla's "second
    // array" text cannot be reached from a file.
    let ValueList::List(layers) = layers else {
        return Err(invalid(text::first_array(FIELD)));
    };
    let [first, second] = [ints(&layers[0]), ints(&layers[1])];
    if first.len() != second.len() {
        return Err(invalid(text::same_size(FIELD)));
    }
    Ok([first, second])
}

/// The volume check: a non-empty layer 0 has one index per
/// position; the volume is an `i32` product, which wraps.
fn check_volume(layers: &[Vec<i32>; 2], [x, y, z]: [i32; 3]) -> Step<()> {
    let count = layers[0].len();
    let volume = x.wrapping_mul(y).wrapping_mul(z);
    if count != 0 && usize::try_from(volume).ok() != Some(count) {
        return Err(fail(
            DiagnosticCode::InvalidValue {
                text: Some(text::volume("block_indices", "size")),
            },
            &path(&["structure", "block_indices"]),
        ));
    }
    Ok(())
}

/// `_parsePalettes`: a compound of palettes, each loaded inside the scope
/// `Parsing Structure Block Palette: <name>`; any failure fails the file. Vanilla walks its
/// sorted map, so a file with two bad palettes reports the first by name.
fn palettes(body: &Compound) -> Step<IndexMap<String, Palette>> {
    const FIELD: &str = "palette";
    let Some(palettes) = compound(body, FIELD) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(FIELD)),
            },
            &path(&["structure", FIELD]),
        ));
    };
    let mut read = IndexMap::new();
    for (name, value) in sorted(palettes) {
        let name = text_of(name);
        let at = path(&["structure", FIELD, &name]);
        let Value::Compound(value) = value else {
            return Err(fail(
                DiagnosticCode::InvalidValue {
                    text: Some(text::compound_elements(FIELD)),
                },
                &at,
            ));
        };
        read.insert(name, palette(value, &at)?);
    }
    // The file's order, for write-back.
    Ok(palettes
        .keys()
        .filter_map(|name| read.swap_remove_entry(&text_of(name)))
        .collect())
}

/// One palette.
fn palette(palette: &Compound, at: &JsonPath) -> Step<Palette> {
    let block_palette = block_palette(palette, at)?;
    let block_position_data = block_position_data(palette, at)?;
    Ok(Palette {
        block_palette,
        block_position_data,
        extra: extra(palette, &["block_palette", "block_position_data"]),
    })
}

/// A block palette: a list of compounds, cloned unchecked.
fn block_palette(palette: &Compound, at: &JsonPath) -> Step<Vec<Compound>> {
    const FIELD: &str = "block_palette";
    let at = child(at, FIELD);
    let Some(Value::List(list)) = palette.get(key(FIELD)) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(FIELD)),
            },
            &at,
        ));
    };
    compounds(list).ok_or_else(|| {
        fail(
            DiagnosticCode::InvalidValue {
                text: Some(text::invalid_block(0)),
            },
            &index(&at, 0),
        )
    })
}

/// The block position data: decimal keys, compound values; the index is
/// not checked against the volume.
fn block_position_data(
    palette: &Compound,
    at: &JsonPath,
) -> Step<BTreeMap<i32, BlockPositionData>> {
    const FIELD: &str = "block_position_data";
    let at = child(at, FIELD);
    let Some(positions) = compound(palette, FIELD) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(FIELD)),
            },
            &at,
        ));
    };
    let mut read = BTreeMap::new();
    for (name, value) in sorted(positions) {
        let name = text_of(name);
        let at = child(&at, &name);
        // The whole key is read as a decimal i32.
        let Ok(index) = name.parse::<i32>() else {
            return Err(fail(
                DiagnosticCode::InvalidValue {
                    text: Some(text::not_a_number(FIELD, &name)),
                },
                &at,
            ));
        };
        let Value::Compound(value) = value else {
            return Err(fail(
                DiagnosticCode::InvalidValue {
                    text: Some(text::compound_value(FIELD, &name)),
                },
                &at,
            ));
        };
        read.insert(index, position(value, &at)?);
    }
    Ok(read)
}

/// One block position entry: an optional block entity compound and an
/// optional list of scheduled ticks.
fn position(entry: &Compound, at: &JsonPath) -> Step<BlockPositionData> {
    const ENTITY: &str = "block_entity_data";
    const TICKS: &str = "tick_queue_data";
    let invalid = |text, at: &JsonPath| fail(DiagnosticCode::InvalidValue { text: Some(text) }, at);
    let block_entity_data = match entry.get(key(ENTITY)) {
        None => None,
        Some(Value::Compound(data)) => Some(data.clone()),
        Some(_) => return Err(invalid(text::compound_field(ENTITY), &child(at, ENTITY))),
    };
    let tick_queue_data = match entry.get(key(TICKS)) {
        None => None,
        Some(Value::List(list)) => {
            let at = child(at, TICKS);
            let ticks = compounds(list)
                .ok_or_else(|| invalid(text::compound_at(0, TICKS), &index(&at, 0)))?;
            Some(ticks.iter().map(tick).collect())
        }
        Some(_) => return Err(invalid(text::list_field(TICKS), &child(at, TICKS))),
    };
    Ok(BlockPositionData {
        block_entity_data,
        tick_queue_data,
        extra: extra(entry, &[ENTITY, TICKS]),
    })
}

/// One scheduled tick: `tick_delay` by `getInt`, other tags ignored by vanilla.
fn tick(tick: &Compound) -> TickQueueEntry {
    TickQueueEntry {
        tick_delay: tick.get(key("tick_delay")).map(|delay| match delay {
            Value::Int(delay) => *delay,
            _ => 0,
        }),
        extra: extra(tick, &["tick_delay"]),
    }
}

/// `_parseEntities`: a list of compounds, cloned unchecked; a non-compound
/// element fails the file.
fn entities(body: &Compound) -> Step<Vec<Compound>> {
    const FIELD: &str = "entities";
    let at = path(&["structure", FIELD]);
    let Some(Value::List(list)) = body.get(key(FIELD)) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(FIELD)),
            },
            &at,
        ));
    };
    compounds(list).ok_or_else(|| {
        fail(
            DiagnosticCode::InvalidValue {
                text: Some(text::invalid_entity(0)),
            },
            &index(&at, 0),
        )
    })
}

/// The error that ends the load, at `at`.
fn fail(code: DiagnosticCode, at: &JsonPath) -> Box<Diagnostic> {
    Box::new(Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at.clone(), None),
    ))
}

/// The tag path of `segments`.
fn path(segments: &[&str]) -> JsonPath {
    let mut at = JsonPath::new();
    for segment in segments {
        at.push(segment);
    }
    at
}

/// `at` followed by the tag `name`.
fn child(at: &JsonPath, name: &str) -> JsonPath {
    let mut at = at.clone();
    at.push(name);
    at
}

/// `at` followed by the list index `i`.
fn index(at: &JsonPath, i: usize) -> JsonPath {
    let mut at = at.clone();
    at.push_index(i);
    at
}

// ---- vanilla's tag reads ----------------------------------------------------------------

/// A tag name as a map key.
fn key(name: &str) -> &[u8] {
    name.as_bytes()
}

/// A tag name as text (lossy for bytes that are not UTF-8).
fn text_of(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

/// The Int at `name`, or 0 when the tag is absent or not an Int.
fn int(compound: &Compound, name: &str) -> i32 {
    match compound.get(key(name)) {
        Some(Value::Int(v)) => *v,
        _ => 0,
    }
}

/// The compound at `name`, or `None` when absent or of another type.
fn compound<'a>(compound: &'a Compound, name: &str) -> Option<&'a Compound> {
    match compound.get(key(name)) {
        Some(Value::Compound(found)) => Some(found),
        _ => None,
    }
}

/// Each Int of a list, 0 for an element of another type.
fn ints(list: &ValueList) -> Vec<i32> {
    match list {
        ValueList::Int(values) => values.clone(),
        other => vec![0; other.len()],
    }
}

/// A list of three Ints (`size`, `structure_world_origin`; `_parseSize`,
/// `_parseStructureWorldOrigin`): absent or not a List is the required-field
/// error, another length "expects 3 elements"; elements of another type read as 0.
fn triple(compound: &Compound, name: &str) -> Step<[i32; 3]> {
    let Some(Value::List(list)) = compound.get(key(name)) else {
        return Err(fail(
            DiagnosticCode::MissingRequiredField {
                text: Some(text::required(name)),
            },
            &path(&[name]),
        ));
    };
    ints(list).try_into().map_err(|_| {
        fail(
            DiagnosticCode::InvalidValue {
                text: Some(text::three_elements(name)),
            },
            &path(&[name]),
        )
    })
}

/// The compounds of a list, cloned; `None` when the list holds another tag (so its first
/// element is not a compound). An empty list of any tag is no compounds.
fn compounds(list: &ValueList) -> Option<Vec<Compound>> {
    match list {
        ValueList::Compound(items) => Some(items.clone()),
        _ if list.is_empty() => Some(Vec::new()),
        _ => None,
    }
}

/// The entries of `compound` in vanilla's `std::map` order (bytewise by name).
fn sorted(compound: &Compound) -> Vec<(&[u8], &Value)> {
    let mut entries: Vec<(&[u8], &Value)> = compound
        .iter()
        .map(|(name, value)| (name.as_slice(), value))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}

/// The tags of `compound` other than `known`, in file order.
fn extra(compound: &Compound, known: &[&str]) -> Compound {
    compound
        .iter()
        .filter(|(name, _)| !known.iter().any(|k| k.as_bytes() == name.as_slice()))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}
