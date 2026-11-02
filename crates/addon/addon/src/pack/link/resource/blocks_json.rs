//! What the rules about `blocks.json` share: which pack's file wrote a member of a block's merged
//! entry, and where an entry is told.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use super::super::{Findings, LinkCx};
use crate::families::blocks_json::entry_members;

/// The one `blocks.json` of a pack, by the path the collection keys it with.
pub(super) const BLOCKS_JSON: &str = "blocks.json";

/// The path of the member `member` of the `blocks.json` entry written under `key`, or of the
/// entry itself.
pub(super) fn entry_path(key: &str, member: Option<&str>) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(key);
    if let Some(member) = member {
        path.push(member);
    }
    path
}

/// The key a `blocks.json` entry is written under: the registry holds a block by its
/// identifier, the entry's origin has the key as the file has it.
pub(super) fn entry_key<'d>(id: &'d BlockId, origin: &'d Origin) -> &'d str {
    origin.key.as_deref().unwrap_or(id.as_str())
}

/// The members of a `blocks.json` entry a rule of this group tells a finding at.
pub(super) const WRITTEN: [&str; 4] = [
    "textures",
    "carried_textures",
    "sound",
    "ambient_occlusion_exponent",
];

/// Which pack's `blocks.json` wrote each member of a block's merged entry. The registry holds
/// one entry per block, merged over every pack and placed at the highest pack that has an
/// entry for the block: a member a lower pack wrote (the ordinary shape of an add-on over
/// another pack) is that pack's, and a finding about it is told at that pack's file.
///
/// Built on the first finding of a pass, from the `blocks.json` of every pack of the
/// resource stack: a pass without a finding reads nothing again.
#[derive(Debug, Default)]
pub(super) struct Writers(Option<HashMap<(BlockId, &'static str), Origin>>);

impl Writers {
    /// The place of the entry that wrote `member` (one of [`WRITTEN`]) of the block `id`:
    /// the highest pack whose entry for the block has the member. `merged`, the place of the
    /// merged entry, when no file that could be read has it (a file that is not resident is
    /// told once, as an information).
    pub(super) fn of(
        &mut self,
        cx: &LinkCx<'_>,
        out: &mut Findings,
        id: &BlockId,
        member: &'static str,
        merged: &Origin,
    ) -> Origin {
        let written = self.0.get_or_insert_with(|| {
            let mut written = HashMap::new();
            // The stack's packs, the top one first: the first entry with a member wins.
            for pack in cx.resource().stack_packs() {
                for file in pack.files() {
                    if file.key.as_str() != BLOCKS_JSON {
                        continue;
                    }
                    let Some(bytes) = cx.bytes(PackSide::Resource, &pack.pack, &file.path, out)
                    else {
                        continue;
                    };
                    let base_game = pack.context.is_base_game_pack;
                    // Inside one file a later entry is read over an earlier one (both
                    // mushroom blocks' entries go to the stem): the last one that has the
                    // member wrote it.
                    let mut of_file = HashMap::new();
                    for entry in entry_members(&bytes, base_game, &WRITTEN) {
                        for member in entry.members {
                            of_file.insert((entry.id.clone(), member), entry.key.clone());
                        }
                    }
                    for (place, key) in of_file {
                        written
                            .entry(place)
                            .or_insert_with(|| creator_origin(pack, &file.path, key));
                    }
                }
            }
            written
        });
        written.get(&(id.clone(), member)).unwrap_or(merged).clone()
    }
}

/// The origin of the entry `key` of the `blocks.json` at `path` of `pack`.
pub(super) fn creator_origin(
    pack: &bedrock_addon_core::load::StackPack,
    path: &PackPath,
    key: String,
) -> Origin {
    Origin::new(
        pack.pack.clone(),
        PackSide::Resource,
        path.clone(),
        Some(key.into()),
        Vec::new(),
    )
}
