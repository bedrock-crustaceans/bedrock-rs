//! Format 1.21.110: `minecraft:precipitation` and `biome_music.underwater_music`; the last
//! format whose `ambient_sounds.addition` is a plain sound.

use super::v1_21_40::{AmbientSounds as OlderAmbientSounds, Component};
use super::{insert, legacy_document, note_component, v1_21_120};
use crate::component::{AmbientSounds, ClientBiomeComponent, SoundAddition};
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::document::{Component as _, ComponentMap, ComponentRef, Extra};
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::json::StrictFloat;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

legacy_document! {
    /// A client biome of format `[1.21.110, 1.21.120)`.
    version = (1, 21, 110), set = Component
}

/// The version below which a document gets the underwater ambient sounds: the next struct's
/// version, so every document that reaches the step is below it.
const UNDERWATER_AMBIENT_SOUNDS_COMPONENT_BEFORE: FormatVersion = known::V1_21_120;

/// The chance the game gave every biome's `addition` before 1.21.120.
const ADDITION_CHANCE: f32 = 0.0111;
/// The underwater ambience of every vanilla biome.
const UNDERWATER_ADDITION: &str = "ambient.underwater.additions";
const UNDERWATER_ADDITION_CHANCE: f32 = 0.01;
const UNDERWATER_LOOP: &str = "ambient.underwater.loop";

/// `minecraft:ambient_sounds` in the 1.21.120 type: `addition` becomes the sound at the chance
/// the game used; `loop` and `mood` move. The underwater members start absent. An `addition`
/// the document left out, or wrote empty, stays absent: vanilla stores chance 0.0111 beside
/// the empty name, which plays nothing and has no JSON the 1.21.120 struct would read back
/// (`asset` rejects an empty string).
fn convert(older: OlderAmbientSounds) -> AmbientSounds {
    AmbientSounds {
        addition: older
            .addition
            .filter(|asset| !asset.as_str().is_empty())
            .map(|asset| addition(asset, ADDITION_CHANCE)),
        loop_sound: older.loop_sound,
        mood: older.mood,
        underwater_addition: None,
        underwater_loop: None,
        underwater_mood: None,
        extra: older.extra,
    }
}

/// `{asset, chance}`.
fn addition(asset: SoundEventName, chance: f32) -> SoundAddition {
    SoundAddition {
        asset,
        chance: StrictFloat(chance),
        extra: Extra::default(),
    }
}

/// Whether the identifier's namespace is exactly `minecraft`.
fn is_vanilla(identifier: &str) -> bool {
    split_namespace(identifier).is_some_and(|(namespace, _)| namespace == "minecraft")
}

/// The step to 1.21.120: `ambient_sounds` moves into the type with underwater sounds. A
/// vanilla biome gets the game's underwater ambience (and the component, when it has none);
/// another biome's sounds also play under water, as they did before 1.21.120. Every document
/// that reaches this step is below 1.21.120, so the underwater sounds always apply.
impl Upgrade for ClientBiome {
    type Next = v1_21_120::ClientBiome;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_21_120::ClientBiome {
        let vanilla = is_vanilla(self.description.identifier.as_str());
        let early = cx.original_version < UNDERWATER_AMBIENT_SOUNDS_COMPONENT_BEFORE;
        let mut components: ComponentMap<ClientBiomeComponent> = ComponentMap::new();
        let mut converted = false;
        for entry in self.components.iter() {
            match entry {
                ComponentRef::Typed {
                    value: Component::AmbientSounds(older),
                    synthesized,
                    ..
                } => {
                    // An `addition` that is the empty text names no sound and has no place
                    // in the newer shape: noted at the member.
                    if older
                        .addition
                        .as_ref()
                        .is_some_and(|asset| asset.as_str().is_empty())
                    {
                        let mut at = super::component_path(AmbientSounds::NAME);
                        at.push("addition");
                        super::note(cx, DiagnosticCode::DroppedField { text: None }, at);
                    }
                    let sounds = convert(older.clone());
                    let sounds = if early {
                        underwater(sounds, vanilla, cx)
                    } else {
                        sounds
                    };
                    components.insert_typed(sounds.into(), synthesized);
                    converted = true;
                }
                // No document below 1.21.120 reads the component in the newest shape, but a
                // struct built by hand can hold it: there is nothing to convert, and the
                // underwater sounds are given all the same.
                ComponentRef::Typed {
                    value: Component::Newest(ClientBiomeComponent::AmbientSounds(sounds)),
                    synthesized,
                    ..
                } => {
                    let sounds = if early {
                        underwater(sounds.clone(), vanilla, cx)
                    } else {
                        sounds.clone()
                    };
                    components.insert_typed(sounds.into(), synthesized);
                    converted = true;
                }
                ComponentRef::Typed {
                    value: Component::Newest(component),
                    synthesized,
                    ..
                } => {
                    components.insert_typed(component.clone(), synthesized);
                }
                ComponentRef::Unknown { name, json } => {
                    components.insert_unknown(name, json.clone());
                }
                _ => {}
            }
        }
        if early && vanilla && !converted && !super::has(&components, AmbientSounds::NAME) {
            let mut sounds = AmbientSounds::default();
            underwater_ambience(&mut sounds);
            insert(&mut components, sounds, cx);
        }
        v1_21_120::ClientBiome {
            format_version: self.format_version,
            description: self.description,
            components,
            extra: self.extra,
            root_extra: self.root_extra,
        }
    }
}

/// The underwater sounds of a component the document has: the game's for a vanilla biome,
/// the component's own for any other. Reported as a change of the component.
fn underwater(mut sounds: AmbientSounds, vanilla: bool, cx: &mut UpgradeCx<'_>) -> AmbientSounds {
    if vanilla {
        underwater_ambience(&mut sounds);
    } else {
        sounds.underwater_addition.clone_from(&sounds.addition);
        sounds.underwater_loop.clone_from(&sounds.loop_sound);
        sounds.underwater_mood.clone_from(&sounds.mood);
    }
    note_component(
        cx,
        DiagnosticCode::SynthesizedComponent,
        AmbientSounds::NAME,
    );
    sounds
}

/// The underwater ambience of a vanilla biome, over whatever the component had.
fn underwater_ambience(sounds: &mut AmbientSounds) {
    sounds.underwater_addition = Some(addition(
        SoundEventName::from_static(UNDERWATER_ADDITION),
        UNDERWATER_ADDITION_CHANCE,
    ));
    sounds.underwater_loop = Some(SoundEventName::from_static(UNDERWATER_LOOP));
}
