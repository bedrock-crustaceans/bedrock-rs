//! The sound of a `blocks.json` entry against the block sound types of `sounds.json`.

use bedrock_addon_core::diagnostic::{Kind, Severity};

use super::super::{At, Findings, LinkCx, kinds, unknown_sound};
use super::blocks_json::{Writers, entry_key, entry_path};
use super::told::Told;

/// What the client reads of a `sound` that is no string: `""` for a number, an array or an
/// object, the word for a boolean.
const NOT_A_STRING: [&str; 3] = ["", "true", "false"];

/// The `sound` of a `blocks.json` entry is a block sound type: a key of `block_sounds` in the
/// merged `sounds.json`, compared as written. Any other name is an `UnknownSound` information
/// at the member of the entry of the pack that wrote it: the client plays the sounds of
/// `normal` and says nothing.
///
/// A stack whose `sounds.json` has no `block_sounds` has nothing to compare with. A `sound`
/// that was no string is told when the file is read, and not again here.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let bindings = &cx.resource().sounds_json();
    if bindings.origin(&"block_sounds".to_owned()).is_none() {
        return;
    }
    let mut told = Told::default();
    let mut writers = Writers::default();
    for (id, document) in cx.resource().client_blocks() {
        let Some(sound) = document.value().sound.as_deref() else {
            continue;
        };
        if !cx.block_sound(sound).is_missing() {
            continue;
        }
        // The merged entry holds what the client read, not what the file wrote: one of the
        // three texts a value of another type gives, on an entry the load told of such a
        // value, is that value.
        let refused = NOT_A_STRING.contains(&sound)
            && document.diagnostics().iter().any(|finding| {
                finding.code.kind() == Kind::WrongType
                    && finding.location.json_path.len() > 1
                    && finding.location.json_path.last_key() == Some("sound")
            });
        if refused {
            continue;
        }
        let origin = writers.of(cx, out, id, "sound", document.origin());
        let key = entry_key(id, &origin);
        if !told.first(&origin, key, "sound", "") {
            continue;
        }
        // The game logs the miss at the lowest level only, at play time, and retries with
        // `normal`: the crate reports an information.
        let at = At::new(&origin, entry_path(key, Some("sound")), Severity::Info);
        out.push(unknown_sound(at, kinds::BLOCK_SOUND, sound));
    }
}
