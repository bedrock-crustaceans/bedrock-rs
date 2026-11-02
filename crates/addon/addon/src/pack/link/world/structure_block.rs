//! The blocks of a structure's palettes (`structure.palette.<name>.block_palette[]`,
//! each raw block NBT `{name, states, version}`).

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use crate::diagnostic::{JsonPath, Severity};
use crate::id::BlockId;

/// A palette entry whose `name` is no block of the stack and none of the game's is an
/// `UnresolvedReference` information at the name: the game puts an unknown-block placeholder
/// in its place when it places the structure, in silence. An entry without a name that is text names
/// nothing.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().structures() {
        for (palette_name, palette) in &document.value().structure.palette {
            for (index, entry) in palette.block_palette.iter().enumerate() {
                let Some(nbtx::Value::String(name)) = entry.get(b"name".as_slice()) else {
                    continue;
                };
                let Ok(name) = std::str::from_utf8(name.as_slice()) else {
                    continue;
                };
                if let Target::Missing(hint) = cx.block(&BlockId::parse(name)) {
                    let mut path = JsonPath::new();
                    for key in ["structure", "palette", palette_name, "block_palette"] {
                        path.push(key);
                    }
                    path.push_index(index);
                    path.push("name");
                    // The game places the unknown placeholder in silence: the crate reports an
                    // information.
                    let at = At::new(document.origin(), path, Severity::Info);
                    out.push(unresolved(at, kinds::BLOCK, name, hint));
                }
            }
        }
    }
}
