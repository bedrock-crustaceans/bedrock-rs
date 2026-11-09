//! The audio files the sound entries of the sound definitions name.
//!
//! The client resolves an entry's `name` to the first of `.fsb`, `.ogg` and `.wav` the highest
//! pack has under that name; a name that resolves to nothing is silent when it is played, and
//! nothing happens at load. Told as informations at the entry.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, ReferenceHint, Severity};

use crate::families::sound_file::{AudioExt, SOUND_FILE_KIND, SoundAsset};
use crate::pack::link::prelude::*;
use indexmap::IndexMap;

/// The longest name the client's 1024-byte path buffer still holds with an extension.
const MAX_RESOLVED_NAME: usize = 1019;

/// Whether `name` ends in an audio extension the client would append again.
fn has_extension(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".fsb", ".ogg", ".wav", ".mp3"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

/// Every sound name of the merged definitions is checked against the audio files of the stack:
/// a name that ends in an extension is told, and one that resolves to no file is told only when
/// the client's own files are in view: built-ins to ask, or a
/// base-game pack of the stack that lists audio. A stack of add-on packs alone (definitions
/// that reuse the client's audio, with or without some audio of their own) would otherwise
/// report every name the client resolves itself.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    let definitions = cx.resource().sound_definitions();
    let files = cx.resource().sound_files();
    let built_ins = cx.host_built_ins();
    let folded: IndexMap<String, &SoundAsset> = files
        .by_name
        .iter()
        .map(|(name, asset)| (name.to_lowercase(), asset))
        .collect();
    for (event, definition) in &definitions.sound_definitions {
        let Some(origin) = definitions.origin(event) else {
            continue;
        };
        for (index, entry) in definition.sounds.iter().enumerate() {
            let name = entry.name.as_str();
            if name.is_empty() {
                continue;
            }
            let mut path = JsonPath::new();
            path.push("sound_definitions");
            path.push(event.as_str());
            path.push("sounds");
            path.push_index(index);
            if has_extension(name) {
                let at = At::new(origin, path.clone(), Severity::Info);
                out.push(at.diagnostic(Code::from(&core_codes::INVALID_VALUE)));
            }
            let built_in = || {
                built_ins.is_some_and(|built_ins| {
                    AudioExt::PROBE_ORDER.iter().any(|ext| {
                        built_ins.contains(SOUND_FILE_KIND, &format!("{name}{}", ext.as_str()))
                    })
                })
            };
            if name.len() <= MAX_RESOLVED_NAME && (files.by_name.contains_key(name) || built_in()) {
                continue;
            }
            if !files.client_audio && built_ins.is_none() {
                continue;
            }
            // Nothing happens at load, and the sound is silent when played.
            let hint = folded
                .get(&name.to_lowercase())
                .map(|asset| ReferenceHint::CaseMismatch(asset.path.as_str().to_owned()));
            let at = At::new(origin, path, Severity::Info);
            let finding: Diagnostic = at.reference(
                Code::from(&core_codes::UNRESOLVED_REFERENCE),
                SOUND_FILE_KIND,
                name,
                hint,
            );
            out.push(finding);
        }
    }
}
