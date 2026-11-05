//! The sounds a client biome's `minecraft:ambient_sounds` names.
//!
//! The six members of the component (`addition`, `loop`, `mood` and their `underwater_`
//! twins; an addition holds its sound in `asset`) name sounds the client plays by name: the
//! keys of `individual_named_sounds` in the merged `sounds.json`. The empty name is "no
//! sound".

use bedrock_addon_core::document::ComponentRef;

use super::super::{At, Findings, LinkCx, Validator, kinds, unknown_sound};
#[cfg(feature = "music")]
use super::super::{Target, unresolved};
use super::biome::{Claim, claims, path};
use crate::diagnostic::Severity;
#[cfg(feature = "music")]
use crate::families::client_biome::component::BiomeMusic;
use crate::families::client_biome::component::{AmbientSounds, ClientBiomeComponent};
use crate::version::FormatVersion;

/// The component's key.
const COMPONENT: &str = "minecraft:ambient_sounds";

/// The first format whose file has the underwater members and writes an addition as an
/// object with `asset`; an older file's `addition` is the sound's name itself.
const WITH_UNDERWATER: FormatVersion = FormatVersion::new(1, 21, 120);

/// The names of `sounds` with the member each is written at, in the order of the members;
/// for a file older than 1.21.120 the three members such a file has (the underwater ones are
/// the upgrade's).
fn names<'s>(sounds: &'s AmbientSounds, old: bool) -> Vec<(&'static [&'static str], &'s str)> {
    let asset = |addition: &'s Option<_>| {
        addition.as_ref().map_or(
            "",
            |addition: &'s crate::families::client_biome::component::SoundAddition| {
                addition.asset.as_str()
            },
        )
    };
    let name = |name: &'s Option<crate::id::SoundEventName>| {
        name.as_ref().map_or("", crate::id::SoundEventName::as_str)
    };
    let addition: &[&str] = if old {
        &["addition"]
    } else {
        &["addition", "asset"]
    };
    let mut names = vec![
        (addition, asset(&sounds.addition)),
        (&["loop"][..], name(&sounds.loop_sound)),
        (&["mood"][..], name(&sounds.mood)),
    ];
    if !old {
        names.extend([
            (
                &["underwater_addition", "asset"][..],
                asset(&sounds.underwater_addition),
            ),
            (&["underwater_loop"][..], name(&sounds.underwater_loop)),
            (&["underwater_mood"][..], name(&sounds.underwater_mood)),
        ]);
    }
    names
}

/// The findings of one component: each member whose name is no named sound, with the game's
/// linked-asset line: the game holds each member as a reference of its own and logs each
/// miss, so a name two members share is told at both.
fn ambient(cx: &LinkCx<'_>, out: &mut Findings, claim: &Claim<'_>, sounds: &AmbientSounds) {
    let old = !claim.version.beta && claim.version.parsed_as < WITH_UNDERWATER;
    for (member, name) in names(sounds, old) {
        // The empty name is "none".
        if name.is_empty() || !cx.named_sound(name).is_missing() {
            continue;
        }
        let mut keys = vec!["components", COMPONENT];
        keys.extend(member);
        // The linked-asset check raises the finding to an error; it is written as the error
        // that comes out.
        let at = At::new(claim.origin, path(&keys), Severity::Error).validator(
            Validator::Missing,
            1,
            name,
        );
        out.push(unknown_sound(at, kinds::NAMED_SOUND, name));
    }
}

/// Each sound a claiming client biome's
/// `minecraft:ambient_sounds` names must be a key of `individual_named_sounds` of the merged
/// `sounds.json`, compared with case; else `UnknownSound`, an error at the member, once per
/// name and document. The empty name is no sound. A component an upgrade step inserted is
/// not the author's and is not asked.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for claim in claims(cx) {
        for entry in claim.components.iter() {
            match entry {
                ComponentRef::Typed {
                    value: ClientBiomeComponent::AmbientSounds(sounds),
                    synthesized: false,
                    ..
                } => ambient(cx, out, &claim, sounds),
                // The music definitions are the `music` family's.
                #[cfg(feature = "music")]
                ComponentRef::Typed {
                    value: ClientBiomeComponent::BiomeMusic(music),
                    synthesized: false,
                    ..
                } => music_definition(cx, out, &claim, music),
                _ => {}
            }
        }
    }
}

/// `minecraft:biome_music.music_definition` names a music definition of the merged
/// `sounds/music_definitions.json` or one of the game's; the game's linked-asset check holds the
/// reference and logs one it cannot resolve (the biome plays no music of it): an
/// `UnresolvedReference` error at the member with the check's line.
#[cfg(feature = "music")]
fn music_definition(cx: &LinkCx<'_>, out: &mut Findings, claim: &Claim<'_>, music: &BiomeMusic) {
    let Some(id) = &music.music_definition else {
        return;
    };
    if id.as_str().is_empty() {
        return;
    }
    if let Target::Missing(hint) = cx.music_definition(id) {
        let keys = ["components", "minecraft:biome_music", "music_definition"];
        // The linked-asset check raises the finding to an error; it is written as the error
        // that comes out.
        let at = At::new(claim.origin, path(&keys), Severity::Error).validator(
            Validator::Missing,
            1,
            id.as_str(),
        );
        out.push(unresolved(at, kinds::MUSIC_DEFINITION, id.as_str(), hint));
    }
}
