//! Flipbooks: `textures/flipbook_textures.json`, the animated tiles of the terrain atlas.
//! The game merges every pack's array entry by entry and reads the merged array once.

use std::collections::{HashMap, HashSet};

use facet::Facet;
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity, builders};
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, RawLayer, Rejected,
    UnknownMembers, unversioned,
};
use bedrock_addon_core::id::AtlasKey;
use bedrock_addon_core::json::jsoncpp::lenient::{as_int, read_lenient};
use bedrock_addon_core::json::jsoncpp::to_text;
use bedrock_addon_core::json::kit::{Node, Profile, Read, UnknownAt};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::WalkFile;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading, Scalars};

/// The file's path in a resource pack.
pub(crate) const PATH: &str = "textures/flipbook_textures.json";

/// The flipbooks of a resource stack: the merged entries of every pack's
/// `textures/flipbook_textures.json`, lowest pack first, each in the place of its first
/// appearance.
#[doc(alias = "flipbook_textures.json")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flipbooks {
    /// The entries.
    pub entries: Vec<Flipbook>,
}

/// What two flipbook entries of one tile, index and variant have in common: vanilla merges
/// the higher pack's entry into the lower one's. An absent variant and variant 0 are one key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlipbookKey {
    /// `atlas_tile`: the tile's name; `""` when absent.
    pub tile: String,
    /// `atlas_index`; 0 when absent.
    pub index: i32,
    /// `atlas_tile_variant`; 0 when absent.
    pub variant: i32,
}

impl Flipbooks {
    /// The entries in order.
    pub fn iter(&self) -> std::slice::Iter<'_, Flipbook> {
        self.entries.iter()
    }
}

impl<'a> IntoIterator for &'a Flipbooks {
    type Item = &'a Flipbook;
    type IntoIter = std::slice::Iter<'a, Flipbook>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FromIterator<Flipbook> for Flipbooks {
    fn from_iter<I: IntoIterator<Item = Flipbook>>(iter: I) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

/// One flipbook: an image strip that animates a tile of the terrain atlas.
///
/// A member the entry does not write is `None`, and the accessor of the same name gives
/// vanilla's default. Read as the game reads it: each member coerced as jsoncpp's `asInt`,
/// `asString` and `asBool` coerce it, a `null` an absent member, the members it does not read kept in
/// [`extra`](Self::extra); what the game coerces or ignores in silence is told as an information.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
pub struct Flipbook {
    /// `flipbook_texture`: the strip image, pack-relative and without extension. Its frames
    /// are squares stacked top to bottom.
    pub flipbook_texture: Option<String>,
    /// `atlas_tile`: the tile's name in the terrain atlas.
    pub atlas_tile: Option<String>,
    /// `atlas_index`: the index into the tile's `textures`; 0 when absent.
    pub atlas_index: Option<i32>,
    /// `atlas_tile_variant`: one entry of the index's `variations`. Absent, every variant is
    /// animated.
    pub atlas_tile_variant: Option<i32>,
    /// `ticks_per_frame`; 1 when absent, and the game treats less than 1 as 1.
    pub ticks_per_frame: Option<i32>,
    /// `replicate`: the size of the atlas area drawn into, in tiles per axis; 1 when absent.
    pub replicate: Option<i32>,
    /// `blend_frames`: interpolate between frames; true when absent.
    #[facet(schema::read_with = blend_frames)]
    pub blend_frames: Option<bool>,
    /// `frames`: the frame indices in playback order. Absent (or empty, or no array), the
    /// frames play in the image's order.
    #[facet(schema::read_with = frames)]
    pub frames: Option<Vec<i32>>,
    /// Members vanilla does not read.
    pub extra: UnknownMembers,
}

/// `blend_frames`: the game reads a number as a bool too, and the crate tells it.
fn blend_frames(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let blend = bool::from_json(node, read)?;
    if node.value().is_number() {
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
    }
    Some(blend)
}

/// `frames`: the game reads it only when it is an array, and an empty one is no list of frames.
fn frames(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<i32>> {
    if !node.value().is_array() {
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
        return None;
    }
    Some(Vec::from_json(node, read)?).filter(|frames| !frames.is_empty())
}

impl Flipbook {
    /// `flipbook_texture`, `""` when absent.
    #[must_use]
    pub fn flipbook_texture(&self) -> &str {
        self.flipbook_texture.as_deref().unwrap_or_default()
    }

    /// `atlas_tile`, `""` when absent.
    #[must_use]
    pub fn atlas_tile(&self) -> &str {
        self.atlas_tile.as_deref().unwrap_or_default()
    }

    /// The tile as a key of the terrain atlas.
    #[must_use]
    pub fn tile(&self) -> AtlasKey {
        AtlasKey::parse(self.atlas_tile())
    }

    /// `atlas_index`, 0 when absent.
    #[must_use]
    pub fn atlas_index(&self) -> i32 {
        self.atlas_index.unwrap_or(0)
    }

    /// `ticks_per_frame`, 1 when absent.
    #[must_use]
    pub fn ticks_per_frame(&self) -> i32 {
        self.ticks_per_frame.unwrap_or(1)
    }

    /// The ticks a frame is shown for: `ticks_per_frame`, at least 1.
    #[must_use]
    pub fn effective_ticks_per_frame(&self) -> i32 {
        self.ticks_per_frame().max(1)
    }

    /// `replicate`, 1 when absent.
    #[must_use]
    pub fn replicate(&self) -> i32 {
        self.replicate.unwrap_or(1)
    }

    /// `blend_frames`, true when absent.
    #[must_use]
    pub fn blend_frames(&self) -> bool {
        self.blend_frames.unwrap_or(true)
    }

    /// The key vanilla merges entries by: the tile, the index and the variant (0 when absent).
    #[must_use]
    pub fn merge_key(&self) -> FlipbookKey {
        FlipbookKey {
            tile: self.atlas_tile().to_owned(),
            index: self.atlas_index(),
            variant: self.atlas_tile_variant.unwrap_or(0),
        }
    }
}

impl bedrock_addon_core::__private::Sealed for Flipbooks {}

/// Keyed by the merge key: `Merged::origin` names the highest pack that wrote the entry.
impl Merge for Flipbooks {
    type Key = FlipbookKey;
}

/// The merge key of a raw entry, as the game's merge reads it.
fn raw_key(entry: &VObject) -> FlipbookKey {
    FlipbookKey {
        tile: entry.get("atlas_tile").map_or_else(String::new, to_text),
        index: as_int(entry.get("atlas_index"), 0),
        variant: as_int(entry.get("atlas_tile_variant"), 0),
    }
}

/// The recursive merge: objects merge member by member, anything else is
/// replaced.
fn recursive_merge(target: &mut Value, source: &Value) {
    if let (Some(_), Some(from)) = (target.as_object(), source.as_object()) {
        let Some(into) = target.as_object_mut() else {
            return;
        };
        for (key, value) in from {
            match into.get_mut(key.as_str()) {
                Some(existing) => recursive_merge(existing, value),
                None => {
                    into.insert(key.as_str(), value.clone());
                }
            }
        }
    } else {
        *target = source.clone();
    }
}

/// Reads one typed entry with vanilla's coercions; the findings go to `d`. A member that
/// reads nothing resets, so the entry always reads.
fn read_entry(entry: &Value, at: JsonPath, d: &mut Diagnostics) -> Flipbook {
    let reading = Reading::OWN
        .with_scalars(Scalars::Lenient)
        .with_null(NullHandling::AbsentSilently)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut read = Read::new(reading, d)
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
        .unknown_severity(Severity::Info);
    schema::read_document(&Node::new(entry, at), &mut read).unwrap_or_default()
}

/// The typed entries of a merged array.
fn read_entries(root: &VArray) -> Vec<Flipbook> {
    let mut silence = Diagnostics::default();
    root.as_slice()
        .iter()
        .filter(|entry| entry.is_object())
        .map(|entry| read_entry(entry, JsonPath::new(), &mut silence))
        .collect()
}

/// The findings of one file, before the merge, so that each names its pack: a root that is no
/// array, elements that are no objects, members vanilla coerces or ignores, unknown members,
/// a `ticks_per_frame` below 1 and entries of one key. The game is silent on all of them.
fn check_file(root: &Value, origin: &Origin, out: &mut Diagnostics) -> bool {
    // All of these are the crate's own lints, informations: the game is silent, and the
    // `ticks_per_frame` clamp is silent too.
    let invalid = || Diagnostic::of(&core_codes::INVALID_VALUE).severity(Severity::Info);
    let Some(entries) = root.as_array() else {
        out.push(invalid().at(JsonPath::new()).in_file(origin));
        return false;
    };
    let mut seen: HashSet<FlipbookKey> = HashSet::new();
    for (index, entry) in entries.as_slice().iter().enumerate() {
        let at = JsonPath::new().index(index);
        if !entry.is_object() {
            out.push(invalid().at(at).in_file(origin));
            continue;
        }
        let mut told = Diagnostics::default();
        let flipbook = read_entry(entry, at.clone(), &mut told);
        for finding in told {
            out.push(finding.in_file(origin));
        }
        if flipbook.ticks_per_frame() < 1 {
            out.push(
                Diagnostic::of(&core_codes::CLAMPED_VALUE)
                    .at(at.child("ticks_per_frame"))
                    .in_file(origin),
            );
        }
        if !seen.insert(flipbook.merge_key()) {
            out.push(
                Diagnostic::of(&core_codes::DUPLICATE_IDENTIFIER)
                    .severity(Severity::Info)
                    .at(at)
                    .in_file(origin),
            );
        }
    }
    true
}

/// One pack's copy of the flipbook file, as the parse job leaves it.
#[derive(Debug)]
pub struct FlipbookCopy {
    /// The copy, when it is JSON with an array root.
    pub(super) layer: Option<RawLayer>,
    /// What reading the copy found.
    pub(super) findings: Diagnostics,
}

/// Reads one pack's copy. A copy that is no JSON is an error and is left out, as in vanilla
/// (whose text is its JSON library's); a root that is no array is left out with an information.
pub(crate) fn parse_copy(file: &WalkFile) -> FlipbookCopy {
    let origin = unversioned::origin_of(file);
    let mut findings = Diagnostics::default();
    // The copy is read with the numbers no double holds clamped, so neither its layer
    // nor the merged list keeps an infinity; the warning names this pack's file.
    let finite = bedrock_addon_core::document::FiniteText::of(&file.bytes);
    let layer = match read_lenient(finite.bytes()) {
        Ok(read) => check_file(&read.value, &origin, &mut findings)
            .then(|| RawLayer::new(origin.clone(), read.value)),
        Err(message) => {
            findings.push(Diagnostic::of(builders::syntax(message.to_string())).in_file(&origin));
            None
        }
    };
    finite.settle(&mut findings, Some(&origin));
    FlipbookCopy { layer, findings }
}

/// Merges `copy` into `root` the way the game merges a higher file: each
/// object entry into the first entry of its key, or appended. `wrote` hears the key of every
/// entry the copy wrote. `places` is where the first entry of each key stands in `root`; it is
/// kept up to date, so a file of any length merges in one pass (a merge never changes an
/// entry's key: both sides have the same one).
fn merge_copy(
    root: &mut VArray,
    places: &mut HashMap<FlipbookKey, usize>,
    copy: &Value,
    mut wrote: impl FnMut(FlipbookKey),
) {
    let Some(entries) = copy.as_array() else {
        return;
    };
    for entry in entries.as_slice() {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let key = raw_key(object);
        let place = places.get(&key).copied();
        if let Some(existing) = place.and_then(|place| root.as_mut_slice().get_mut(place)) {
            recursive_merge(existing, entry);
        } else {
            places.insert(key.clone(), root.len());
            root.push(entry.clone());
        }
        wrote(key);
    }
}

/// The merged flipbooks of the stack from its copies in walk order, lowest pack first.
///
/// Every file goes through the per-entry merge, the lowest one too (vanilla copies the lowest
/// parsable file whole, keeping its duplicates and turning its non-object elements into empty
/// entries).
pub(crate) fn merge_flipbooks(copies: Vec<FlipbookCopy>) -> Merged<Flipbooks, RawLayer> {
    let mut diagnostics = Diagnostics::default();
    let mut layers = Vec::new();
    let mut root = VArray::new();
    let mut places = HashMap::new();
    let mut origins: IndexMap<FlipbookKey, Origin> = IndexMap::new();
    for copy in copies {
        diagnostics.append(copy.findings);
        if let Some(layer) = copy.layer {
            merge_copy(&mut root, &mut places, &layer.json, |key| {
                origins.insert(key, layer.origin.clone());
            });
            layers.push(layer);
        }
    }
    layers.reverse();
    let value = Flipbooks {
        entries: read_entries(&root),
    };
    Merged::new(value, origins, layers, diagnostics)
}

impl Content for Flipbooks {
    const NAME: &'static str = "flipbook textures";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `flipbook_textures.json` on its own, as a stack of one pack would. Rejected
    /// when the file is no JSON or its root is no array.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = unversioned::lone(path);
            let mut diagnostics = Diagnostics::default();
            let read = match read_lenient(bytes) {
                Ok(read) => read,
                Err(message) => {
                    diagnostics.push(
                        Diagnostic::of(builders::syntax(message.to_string())).in_file(&origin),
                    );
                    return Err(unversioned::rejected(origin, diagnostics));
                }
            };
            if !check_file(&read.value, &origin, &mut diagnostics) {
                return Err(unversioned::rejected(origin, diagnostics));
            }
            let mut root = VArray::new();
            merge_copy(&mut root, &mut HashMap::new(), &read.value, |_| {});
            let value = Flipbooks {
                entries: read_entries(&root),
            };
            Ok(unversioned::document(value, origin, diagnostics))
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let mut root = VArray::new();
        for entry in &self.entries {
            root.push(entry_json(entry));
        }
        write_json(&root.into_value(), out, style)
    }
}

impl JsonContent for Flipbooks {}

impl Placed for Flipbooks {
    /// `textures/flipbook_textures.json`.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
}

/// One entry as JSON, in the member order of vanilla's own file.
fn entry_json(entry: &Flipbook) -> Value {
    let mut object = VObject::new();
    if let Some(path) = &entry.flipbook_texture {
        object.insert("flipbook_texture", path.as_str());
    }
    if let Some(tile) = &entry.atlas_tile {
        object.insert("atlas_tile", tile.as_str());
    }
    let ints = [
        ("atlas_index", entry.atlas_index),
        ("atlas_tile_variant", entry.atlas_tile_variant),
        ("ticks_per_frame", entry.ticks_per_frame),
    ];
    for (key, value) in ints {
        if let Some(value) = value {
            object.insert(key, i64::from(value));
        }
    }
    if let Some(frames) = &entry.frames {
        let mut array = VArray::new();
        for frame in frames {
            array.push(i64::from(*frame));
        }
        object.insert("frames", array.into_value());
    }
    if let Some(replicate) = entry.replicate {
        object.insert("replicate", i64::from(replicate));
    }
    if let Some(blend) = entry.blend_frames {
        object.insert("blend_frames", blend);
    }
    for (key, value) in entry.extra.iter() {
        object.insert(key, value.clone());
    }
    object.into_value()
}

#[cfg(test)]
mod collection_tests {
    use super::*;

    fn entry(tile: &str) -> Flipbook {
        Flipbook {
            atlas_tile: Some(tile.to_owned()),
            ..Flipbook::default()
        }
    }

    /// Flipbooks collect from an iterator and iterate by reference in order.
    #[test]
    fn flipbooks_collect_and_iterate_in_order() {
        let books: Flipbooks = [entry("a"), entry("b")].into_iter().collect();
        let tiles: Vec<&str> = (&books).into_iter().map(Flipbook::atlas_tile).collect();
        assert_eq!(tiles, ["a", "b"]);
    }

    /// The merge key names the tile, index and variant, and an absent variant is 0.
    #[test]
    fn the_merge_key_has_named_parts() {
        let key = entry("a").merge_key();
        assert_eq!(
            key,
            FlipbookKey {
                tile: "a".to_owned(),
                index: 0,
                variant: 0
            }
        );
    }
}
