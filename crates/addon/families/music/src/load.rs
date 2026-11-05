//! Loading `sounds/music_definitions.json`: the walk, the JSON merge across the stack (the game's
//! merge strategy, with the music file's own predicate for a level that replaces) and the
//! typed read of the merged document.

use indexmap::IndexMap;

use super::{MusicDefinitions, replace_predicate};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{
    Content, Document, JsonContent, Merge, Merged, Origin, Placed, RawLayer, Rejected,
};
use bedrock_addon_core::id::MusicDefinitionId;
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::sound::fork;
use bedrock_addon_support::sound::merge::{
    SoundCopy, detached_origin, layers, located, not_json, parse_copy, path_member, read_json,
    readable, unversioned, walk, write_document,
};

/// Where a pack keeps its music definitions.
pub(crate) const MUSIC_PATH: &str = "sounds/music_definitions.json";

impl bedrock_addon_core::__private::Sealed for MusicDefinitions {}

impl Merge for MusicDefinitions {
    /// A music name.
    type Key = MusicDefinitionId;
}

/// Merges the copies of `sounds/music_definitions.json` and reads the result.
fn merge_music(copies: Vec<SoundCopy>, _: &StackCx<'_>) -> Merged<MusicDefinitions, RawLayer> {
    let mut diagnostics = Diagnostics::default();
    let copies = readable(MUSIC_PATH, copies, &mut diagnostics);
    let mut origins: IndexMap<MusicDefinitionId, Origin> = IndexMap::new();
    let mut target: Option<Value> = None;
    for (origin, json, _) in &copies {
        for name in fork::member_names(json) {
            origins.insert(MusicDefinitionId::parse(name), origin.clone());
        }
        match &mut target {
            None => target = Some(json.clone()),
            Some(target) => {
                fork::recursive_merge(target, json, Some(replace_predicate));
            }
        }
    }
    let merged = target.unwrap_or(Value::NULL);
    let mut found = Diagnostics::default();
    let value = MusicDefinitions::read(merged, &mut found);
    let top = copies.last().map(|(origin, _, _)| origin);
    for mut finding in found {
        let owner = path_member(&finding.location.json_path, 0)
            .and_then(|name| origins.get(&MusicDefinitionId::parse(name)))
            .or(top);
        if let Some(origin) = owner {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
        diagnostics.push(finding);
    }
    origins.retain(|name, _| value.definitions.contains_key(name));
    Merged::new(value, origins, layers(&copies), diagnostics)
}

/// The file the walk reads: `sounds/music_definitions.json` in every pack.
fn accepts(path: &PackPath) -> bool {
    path.as_str() == MUSIC_PATH
}

/// The loader of `rp.music()`: every resource pack's copy of the file, merged as JSON and read
/// once.
pub const LOADER: CollectionLoader<Merged<MusicDefinitions, RawLayer>, SoundCopy> =
    CollectionLoader {
        walk: walk(accepts),
        parse: parse_copy,
        combine: merge_music,
        needs: &[],
        then: None,
    };

impl Content for MusicDefinitions {
    const NAME: &'static str = "music definitions";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one pack's `sounds/music_definitions.json` on its own. A file that is no JSON is
    /// rejected.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        // A number no double holds is read as the largest finite one, with a warning.
        bedrock_addon_core::document::FiniteText::load(bytes, |bytes| {
            let origin = detached_origin(path);
            let json = match read_json(bytes) {
                Ok(json) => json,
                Err(message) => return Err(not_json(origin, &message)),
            };
            let mut found = Diagnostics::default();
            let value = Self::read(json, &mut found);
            let mut diagnostics = Diagnostics::default();
            located(found, &origin, &mut diagnostics);
            Ok(Document::new(value, origin, unversioned(), diagnostics))
        })
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_document(&self.json, out, style)
    }
}

impl JsonContent for MusicDefinitions {}

impl Placed for MusicDefinitions {
    /// `sounds/music_definitions.json`: the one path the client reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(MUSIC_PATH)
    }
}
