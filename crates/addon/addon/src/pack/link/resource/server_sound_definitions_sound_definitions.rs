//! A sound event the behaviour packs give server metadata to is a sound event the resource
//! packs define.
//!
//! The server reads `sounds/sound_definitions.json` of the behaviour packs alone and never
//! looks at the resource packs' file of the same name, so the game checks nothing: metadata
//! for a sound no client can play is kept and unused. Told here as an information, at the
//! definition's `sound_event_name`, with the name as written (compared byte for byte); a lint,
//! left out when the lints are off.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::{Content, Origin};
use bedrock_addon_core::pack_identity::PackSide;

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use crate::families::server_sound_definitions::SoundMetadataFile;

/// The payload member of a server sound definitions file: the array of the definitions.
const PAYLOAD: &str = "minecraft:server_sound_definitions";

/// The name of every server sound definition the behaviour stack keeps is a key of the
/// resource stack's merged `sound_definitions` (or a sound event the host says the game has).
/// One that is none is an `UnknownSound` information at `<payload>/<index>/sound_event_name`
/// of the definition's file.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    // The names of each file as the file has them, read when a finding needs its index.
    let mut written: HashMap<Option<usize>, Option<Vec<String>>> = HashMap::new();
    // How many definitions of each file the registry has given so far.
    let mut given: HashMap<Option<usize>, usize> = HashMap::new();
    for (id, document) in cx.behavior().server_sound_definitions() {
        let origin = document.origin();
        let file = origin.pack.index();
        let place = given.entry(file).or_default();
        let kept = *place;
        *place += 1;
        if !cx.sound_definition(id).is_missing() {
            continue;
        }
        let name = document.value().sound_event_name.as_str();
        // The registry keeps the first definition of a name in a file, so the first element
        // with the name is this one. Without the file's bytes the index is the definition's
        // place among the ones the registry kept of that file, which is the file's index
        // unless an element before it was dropped as a duplicate.
        let names = written
            .entry(file)
            .or_insert_with(|| names_as_written(cx, origin, out));
        let index = names
            .as_ref()
            .and_then(|names| names.iter().position(|written| written == name))
            .unwrap_or(kept);
        let mut path = JsonPath::new();
        path.push(PAYLOAD);
        path.push_index(index);
        path.push("sound_event_name");
        let at = At::new(origin, path, Severity::Info);
        out.push(unknown_sound(at, kinds::SOUND_EVENT, name));
    }
}

/// The `sound_event_name` of each definition of the file of `origin`, in the file's order:
/// the registry does not keep a definition's index. `None` when the pack does not hold the
/// file's bytes (told once by the pass) or the file does not read again.
fn names_as_written(cx: &LinkCx<'_>, origin: &Origin, out: &mut Findings) -> Option<Vec<String>> {
    let pack = cx.stack_pack(PackSide::Behavior, &origin.pack)?;
    let bytes = cx.bytes(PackSide::Behavior, &origin.pack, &origin.path, out)?;
    let file = SoundMetadataFile::parse(&origin.path, &bytes, &pack.context).ok()?;
    let names = file.value().0.iter();
    Some(names.map(|entry| entry.sound_event_name.clone()).collect())
}
