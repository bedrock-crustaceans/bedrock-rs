#![cfg(feature = "full")]
//! Pins of present behaviour, part C (the client families): a refused definition in the pack
//! that would win an identifier (or a path), over a valid definition of the same identifier in
//! another pack. Either the other pack's definition is effective ("fallback") or the identifier
//! stays without an effective definition ("taken").
//!
//! Per family: a two-pack form (the winner's place refused, one valid definition beside it)
//! and a three-pack form (the winner's place refused, then a valid `M`, then a valid `L`),
//! which says WHICH definition is effective. Each test also requires the refusal to be
//! reported, in the refused pack and at its file.
//!
//! Families whose refusals are of two kinds that behave differently, or whose stack has a
//! refusal of its own, have both kinds pinned (client entities, sounds, `blocks.json`).

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::server_sound_definitions::SoundDuration;
use bedrock_addon::families::texture_set::{LayerKind, Rgba};
use bedrock_addon::id::{
    AnimationRef, AtomicClientEntityId, BlockId, EntityId, FogId, RenderControllerId,
    SoundEventName, TexturePath,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

// ---- packs and stacks ----

/// A pack of `module` type (`resources`, `data`) with one file; pack `n` is named `pack <n>`.
fn pack(n: u128, module: &str, engine: &str, path: &str, text: &str) -> Pack {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": {engine}}}, "modules": [{{"type": "{module}", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(n),
        Uuid::from_u128(n + 1000),
    );
    let files = vec![
        (
            PackPath::new("manifest.json").unwrap(),
            manifest.into_bytes(),
        ),
        (PackPath::new(path).unwrap(), text.as_bytes().to_vec()),
    ];
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// A resource stack whose packs each hold the one file `path`, the first text on top.
fn resource(path: &str, texts: &[&str]) -> PackStack<Resource> {
    let packs: Vec<Pack> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| pack(i as u128 + 1, "resources", "[1, 21, 100]", path, text))
        .collect();
    PackStack::<Resource>::of(packs, LoadOptions::default()).expect("a resource stack")
}

/// A behaviour stack with Beta APIs on whose packs each hold `path`, the first text on top.
fn behavior(path: &str, texts: &[&str]) -> PackStack<Behavior> {
    let packs: Vec<Pack> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| pack(i as u128 + 1, "data", "[1, 26, 40]", path, text))
        .collect();
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

// ---- findings ----

/// A finding as `severity code pack path | text`, for failure messages.
fn line(finding: &Diagnostic) -> String {
    format!(
        "{:?} {} pack {:?} {:?} | {}",
        finding.severity,
        finding.code.id(),
        finding
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        finding.location.path.as_ref().map(PackPath::as_str),
        finding.code
    )
}

/// Requires a finding named `code` at `severity`, located in the pack at `pack` (0 = top) and
/// at the file `path`.
#[track_caller]
fn reported<'a>(
    findings: impl Iterator<Item = &'a Diagnostic>,
    code: &str,
    severity: Severity,
    pack: usize,
    path: &str,
) {
    let all: Vec<&Diagnostic> = findings.collect();
    let hit = all.iter().any(|f| {
        f.code.id() == code
            && f.severity == severity
            && f.location
                .pack
                .as_ref()
                .and_then(bedrock_addon::pack::PackRef::index)
                == Some(pack)
            && f.location.path.as_ref().map(PackPath::as_str) == Some(path)
    });
    assert!(
        hit,
        "no {severity:?} {code} in pack {pack} at {path}: {:#?}",
        all.iter().map(|f| line(f)).collect::<Vec<_>>()
    );
}

// ---- client animations ----
//
// Every failure skips the whole file, never one animation. The slot is replaced only by a file
// that parses (the game constructs a new animation and replaces the slot's), so a skipped file
// replaces nothing. Present behaviour: fallback.

const ANIMATIONS: &str = "animations/x.json";

/// `animation.x` with the one bone `bone`.
fn animation(bone: &str) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "animations": {{"animation.x": {{"bones": {{"{bone}": {{"scale": 1}}}}}}}}}}"#
    )
}

/// `animation.x` with a member the schema does not know: the file is refused.
const REFUSED_ANIMATION: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.x": {"loops": true}}}"#;

#[test]
fn client_animation_two_packs_a_refused_top_file_leaves_the_lower_animation_effective() {
    let stack = resource(ANIMATIONS, &[REFUSED_ANIMATION, &animation("l")]);
    let registry = stack.animations();
    let x = registry.get(&AnimationRef::parse("animation.x")).unwrap();
    assert_eq!(x.origin().pack.index(), Some(1));
    assert!(x.bone("l").is_some());
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        ANIMATIONS,
    );
}

#[test]
fn client_animation_three_packs_a_refused_top_file_leaves_the_middle_animation_effective() {
    let stack = resource(
        ANIMATIONS,
        &[REFUSED_ANIMATION, &animation("m"), &animation("l")],
    );
    let registry = stack.animations();
    let x = registry.get(&AnimationRef::parse("animation.x")).unwrap();
    assert_eq!(x.origin().pack.index(), Some(1));
    assert!(x.bone("m").is_some() && x.bone("l").is_none());
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        ANIMATIONS,
    );
}

// ---- client animation controllers ----
//
// A file that fails parsing or validation contributes nothing and removes nothing, so the
// controller of a lower pack stays. Present behaviour: fallback.

const CONTROLLERS: &str = "animation_controllers/x.json";

/// `controller.animation.x` with the one state `state`.
fn controller(state: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "animation_controllers": {{"controller.animation.x": {{"states": {{"{state}": {{}}}}}}}}}}"#
    )
}

/// `controller.animation.x` whose state has a member the schema does not know.
const REFUSED_CONTROLLER: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.x": {"states": {"top": {"x": 1}}}}}"#;

#[test]
fn client_animation_controller_two_packs_a_refused_top_file_leaves_the_lower_controller_effective()
{
    let stack = resource(CONTROLLERS, &[REFUSED_CONTROLLER, &controller("l")]);
    let registry = stack.animation_controllers();
    let x = registry
        .get(&AnimationRef::parse("controller.animation.x"))
        .unwrap();
    assert_eq!(x.origin().pack.index(), Some(1));
    assert!(x.states.contains_key("l"));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        CONTROLLERS,
    );
}

#[test]
fn client_animation_controller_three_packs_a_refused_top_file_leaves_the_middle_controller_effective()
 {
    let stack = resource(
        CONTROLLERS,
        &[REFUSED_CONTROLLER, &controller("m"), &controller("l")],
    );
    let registry = stack.animation_controllers();
    let x = registry
        .get(&AnimationRef::parse("controller.animation.x"))
        .unwrap();
    assert_eq!(x.origin().pack.index(), Some(1));
    assert!(x.states.contains_key("m") && !x.states.contains_key("l"));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        CONTROLLERS,
    );
}

// ---- fogs ----
//
// A file that fails any later step registers nothing, so a lower-priority file with the same
// identifier can still register. Present behaviour: fallback.

const FOGS: &str = "fogs/a.json";

/// `demo:fog` at 1.26.20 with `members` beside its description.
fn fog(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:fog_settings": {{"description": {{"identifier": "demo:fog"}}{members}}}}}"#
    )
}

/// `demo:fog` whose air distance starts behind its end: the validation refuses the document
/// after its identifier is read.
fn refused_fog() -> String {
    fog(r#", "distance": {"air": {"fog_start": 9, "fog_end": 1}}"#)
}

#[test]
fn fog_two_packs_a_refused_top_definition_leaves_the_lower_fog_effective() {
    let stack = resource(FOGS, &[&refused_fog(), &fog("")]);
    let registry = stack.fogs();
    let fog = registry.get(&FogId::parse("demo:fog")).unwrap();
    assert_eq!(fog.origin().pack.index(), Some(1));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "fog/failed_to_validate",
        Severity::Error,
        0,
        FOGS,
    );
}

#[test]
fn fog_three_packs_a_refused_top_definition_leaves_the_middle_fog_effective() {
    let middle = fog(r#", "distance": {"air": {"fog_end": 1}}"#);
    let stack = resource(FOGS, &[&refused_fog(), &middle, &fog("")]);
    let registry = stack.fogs();
    let fog = registry.get(&FogId::parse("demo:fog")).unwrap();
    assert_eq!(fog.origin().pack.index(), Some(1));
    assert!(fog.distance.is_some(), "the middle pack's member");
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "fog/failed_to_validate",
        Severity::Error,
        0,
        FOGS,
    );
}

// ---- client entities ----
//
// Two kinds. The stack's own refusal: a document whose engine version is below that of the
// definition already held is ignored silently (ours adds the `ClientEntityShadowed` info). And
// a file the reader refuses: file-level, the file contributes nothing. Present behaviour for both: fallback; the two
// lower documents have the same `min_engine_version`, so they merge and the definition is the
// middle pack's with the low pack contributing.

const ENTITIES: &str = "entity/ghost.entity.json";
const GHOST: &str = "demo:ghost";

/// A 1.10.0 `demo:ghost` with `members` after its identifier.
fn ghost(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "minecraft:client_entity": {{"description": {{"identifier": "demo:ghost", {members}}}}}}}"#
    )
}

/// `demo:ghost` with the one geometry key `key`.
fn ghost_with_geometry(key: &str) -> String {
    ghost(&format!(r#""geometry": {{"{key}": "geometry.{key}"}}"#))
}

/// `demo:ghost` at a `min_engine_version` above every manifest of the stack (1.21.100).
fn ghost_above_the_stack() -> String {
    ghost(r#""min_engine_version": "1.30.0", "geometry": {"top": "geometry.top"}"#)
}

/// `demo:ghost` with a member no schema declares: the file is refused.
fn refused_ghost() -> String {
    ghost(r#""render_distance": 3, "geometry": {"top": "geometry.top"}"#)
}

#[test]
fn client_entity_two_packs_a_top_document_above_the_stack_version_leaves_the_lower_entity_effective()
 {
    let stack = resource(
        ENTITIES,
        &[&ghost_above_the_stack(), &ghost_with_geometry("l")],
    );
    let registry = stack.client_entities();
    let ghost = registry.get(&EntityId::parse(GHOST)).unwrap();
    assert_eq!(ghost.origin().pack.index(), Some(1));
    assert!(ghost.description.slots.geometry.contains_key("l"));
    assert!(!ghost.description.slots.geometry.contains_key("top"));
    // The stack's "Ignore" is no rejection and no error: an info on the ignored file.
    assert!(registry.rejected().is_empty());
    assert!(registry.diagnostics().all(|f| f.severity < Severity::Error));
    reported(
        registry.diagnostics(),
        "core/client_entity_shadowed_above_stack",
        Severity::Info,
        0,
        ENTITIES,
    );
}

#[test]
fn client_entity_three_packs_a_top_document_above_the_stack_version_leaves_the_middle_entity_effective()
 {
    let stack = resource(
        ENTITIES,
        &[
            &ghost_above_the_stack(),
            &ghost_with_geometry("m"),
            &ghost_with_geometry("l"),
        ],
    );
    let registry = stack.client_entities();
    let ghost = registry.get(&EntityId::parse(GHOST)).unwrap();
    assert_eq!(ghost.origin().pack.index(), Some(1));
    assert_eq!(ghost.origin().contributors.len(), 1);
    assert_eq!(ghost.origin().contributors[0].index(), Some(2));
    let geometry = &ghost.description.slots.geometry;
    assert!(geometry.contains_key("m") && geometry.contains_key("l"));
    assert!(!geometry.contains_key("top"));
    assert!(registry.rejected().is_empty());
    reported(
        registry.diagnostics(),
        "core/client_entity_shadowed_above_stack",
        Severity::Info,
        0,
        ENTITIES,
    );
}

#[test]
fn client_entity_two_packs_a_refused_top_file_leaves_the_lower_entity_effective() {
    let stack = resource(ENTITIES, &[&refused_ghost(), &ghost_with_geometry("l")]);
    let registry = stack.client_entities();
    let ghost = registry.get(&EntityId::parse(GHOST)).unwrap();
    assert_eq!(ghost.origin().pack.index(), Some(1));
    assert!(ghost.description.slots.geometry.contains_key("l"));
    assert!(!ghost.description.slots.geometry.contains_key("top"));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        ENTITIES,
    );
}

#[test]
fn client_entity_three_packs_a_refused_top_file_leaves_the_middle_entity_effective() {
    let stack = resource(
        ENTITIES,
        &[
            &refused_ghost(),
            &ghost_with_geometry("m"),
            &ghost_with_geometry("l"),
        ],
    );
    let registry = stack.client_entities();
    let ghost = registry.get(&EntityId::parse(GHOST)).unwrap();
    assert_eq!(ghost.origin().pack.index(), Some(1));
    assert_eq!(ghost.origin().contributors.len(), 1);
    assert_eq!(ghost.origin().contributors[0].index(), Some(2));
    let geometry = &ghost.description.slots.geometry;
    assert!(geometry.contains_key("m") && geometry.contains_key("l"));
    assert!(!geometry.contains_key("top"));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/child_not_valid",
        Severity::Error,
        0,
        ENTITIES,
    );
}

// ---- atomic client entities ----
//
// A failed document does not remove an earlier definition with the same identifier (it never
// reaches the assignment). Present behaviour: fallback, whole (no merge of the two lower
// documents).

const ATOMIC: &str = "atomic_client_entities/wisp.json";

/// A beta `demo:wisp` with `rest` after its description.
fn wisp(geometry: &str, rest: &str) -> String {
    format!(
        r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{"description": {{"identifier": "demo:wisp", "geometry": {{"default": "geometry.{geometry}"}}}}{rest}}}}}"#
    )
}

/// `demo:wisp` with a collision box of negative width: a constraint the document fails after
/// its identifier is read.
fn refused_wisp() -> String {
    wisp(
        "top",
        r#", "components": {"minecraft:aabb_dimensions": {"width": -1, "height": 1}}"#,
    )
}

#[test]
fn atomic_client_entity_two_packs_a_refused_top_document_leaves_the_lower_entity_effective() {
    let stack = resource(ATOMIC, &[&refused_wisp(), &wisp("l", "")]);
    let registry = stack.atomic_client_entities();
    let wisp = registry
        .get(&AtomicClientEntityId::parse("demo:wisp"))
        .unwrap();
    assert_eq!(wisp.origin().pack.index(), Some(1));
    assert_eq!(
        wisp.description.geometry.as_ref().unwrap()["default"].as_str(),
        "geometry.l"
    );
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "atomic_client_entity/not_negative_text",
        Severity::Error,
        0,
        ATOMIC,
    );
}

#[test]
fn atomic_client_entity_three_packs_a_refused_top_document_leaves_the_middle_entity_effective() {
    let stack = resource(ATOMIC, &[&refused_wisp(), &wisp("m", ""), &wisp("l", "")]);
    let registry = stack.atomic_client_entities();
    let wisp = registry
        .get(&AtomicClientEntityId::parse("demo:wisp"))
        .unwrap();
    assert_eq!(wisp.origin().pack.index(), Some(1));
    assert!(wisp.origin().contributors.is_empty());
    assert_eq!(
        wisp.description.geometry.as_ref().unwrap()["default"].as_str(),
        "geometry.m"
    );
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "atomic_client_entity/not_negative_text",
        Severity::Error,
        0,
        ATOMIC,
    );
}

// ---- sounds: sounds/sound_definitions.json ----
//
// The copies are deep-merged, so a definition is refused only in the merged document. Two
// kinds, which behave differently.
//
// A definition that is no object: ours reports a definition that is still not an object in the
// merged document with the text above at every version, and skips the definition. The top pack's value replaces the lower
// object in the merge, so the name is TAKEN: no definition.
//
// A copy that is no JSON is skipped with a `MalformedInput` warning naming the pack (ours;
// vanilla: silent). Present behaviour: fallback (the merge of
// the lower copies stands). Neither kind is reported at error level.

const DEFINITIONS: &str = "sounds/sound_definitions.json";

/// A 1.20.20 file whose definition `x` is the JSON `definition`.
fn sounds(definition: &str) -> String {
    format!(r#"{{"format_version": "1.20.20", "sound_definitions": {{"x": {definition}}}}}"#)
}

/// A 1.20.20 file whose definition `x` plays the one sound `name`.
fn plays(name: &str) -> String {
    sounds(&format!(r#"{{"sounds": ["sounds/{name}"]}}"#))
}

#[test]
fn sounds_two_packs_a_top_definition_that_is_no_object_takes_the_name() {
    let stack = resource(DEFINITIONS, &[&sounds(r#""no object""#), &plays("l")]);
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    assert!(sounds.sound_definitions.is_empty());
    let malformed: Vec<&Diagnostic> = sounds
        .diagnostics()
        .filter(|f| f.code.id() == "sound_definitions/malformed_sound")
        .collect();
    assert_eq!(malformed.len(), 1, "{malformed:#?}");
    // at information in the merged read
    assert_eq!(malformed[0].severity, Severity::Info);
    // In the pack whose value is no object, at its file.
    reported(
        sounds.diagnostics(),
        "sound_definitions/malformed_sound",
        Severity::Info,
        0,
        DEFINITIONS,
    );
    assert!(
        malformed[0]
            .to_string()
            .contains("sound definitions has sound 'x'that is malformed")
    );
    assert!(sounds.diagnostics().all(|f| f.severity < Severity::Error));
}

#[test]
fn sounds_three_packs_a_top_definition_that_is_no_object_takes_the_name() {
    let stack = resource(
        DEFINITIONS,
        &[&sounds(r#""no object""#), &plays("m"), &plays("l")],
    );
    let sounds = stack.sound_definitions();
    assert!(sounds.get("x").is_none());
    assert!(sounds.sound_definitions.is_empty());
    let malformed: Vec<&Diagnostic> = sounds
        .diagnostics()
        .filter(|f| f.code.id() == "sound_definitions/malformed_sound")
        .collect();
    assert_eq!(malformed.len(), 1, "{malformed:#?}");
    // at information in the merged read
    assert_eq!(malformed[0].severity, Severity::Info);
    // In the pack whose value is no object, at its file.
    reported(
        sounds.diagnostics(),
        "sound_definitions/malformed_sound",
        Severity::Info,
        0,
        DEFINITIONS,
    );
    assert!(sounds.diagnostics().all(|f| f.severity < Severity::Error));
}

#[test]
fn sounds_two_packs_a_top_copy_that_is_no_json_leaves_the_lower_definition_effective() {
    let stack = resource(DEFINITIONS, &["{ not json", &plays("l")]);
    let sounds = stack.sound_definitions();
    let x = sounds.get("x").unwrap();
    assert_eq!(x.sounds.len(), 1);
    assert_eq!(x.sounds[0].name, "sounds/l");
    reported(
        sounds.diagnostics(),
        "core/syntax",
        Severity::Info, // a copy that is no JSON, ours
        0,
        DEFINITIONS,
    );
    assert!(sounds.diagnostics().all(|f| f.severity < Severity::Error));
}

#[test]
fn sounds_three_packs_a_top_copy_that_is_no_json_leaves_the_middle_definition_effective() {
    let stack = resource(DEFINITIONS, &["{ not json", &plays("m"), &plays("l")]);
    let sounds = stack.sound_definitions();
    let x = sounds.get("x").unwrap();
    // Arrays replace in the merge: the middle pack's list, whole.
    assert_eq!(x.sounds.len(), 1);
    assert_eq!(x.sounds[0].name, "sounds/m");
    reported(
        sounds.diagnostics(),
        "core/syntax",
        Severity::Info, // a copy that is no JSON, ours
        0,
        DEFINITIONS,
    );
    assert!(sounds.diagnostics().all(|f| f.severity < Severity::Error));
}

// ---- sound metadata: the behaviour pack's sounds/sound_definitions.json ----
//
// The LOWEST pack wins here: the definition from the lowest-priority pack that defines a name
// is kept, and every higher-priority pack's definition of the same name is dropped. So the
// refused winner is the bottom pack's. What a refused bottom file does to the packs above it
// is not defined by the game; ours rejects the whole file for an unknown member, a type error
// or a constraint failure. Present behaviour: fallback, upwards: the next pack above
// the refused one wins.

const METADATA: &str = "sounds/sound_definitions.json";

/// A beta file whose `demo:click` lasts `seconds` of game time.
fn click(seconds: u32) -> String {
    format!(
        r#"{{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{{"sound_event_name": "demo:click", "duration_info": {{"mode": "game_time", "seconds": {seconds}}}}}]}}"#
    )
}

/// A beta file whose `demo:click` has a member the schema does not know: the file is refused.
const REFUSED_CLICK: &str = r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [{"sound_event_name": "demo:click", "volume": 1.0}]}"#;

#[test]
fn server_sound_definitions_two_packs_a_refused_bottom_file_leaves_the_upper_definition_effective()
{
    let stack = behavior(METADATA, &[&click(1), REFUSED_CLICK]);
    let registry = stack.server_sound_definitions();
    let click = registry.get(&SoundEventName::parse("demo:click")).unwrap();
    assert_eq!(click.origin().pack.index(), Some(0));
    assert_eq!(click.duration(), Some(SoundDuration::Ticks(20)));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/unrecognized_field",
        Severity::Error,
        1,
        METADATA,
    );
}

#[test]
fn server_sound_definitions_three_packs_a_refused_bottom_file_leaves_the_middle_definition_effective()
 {
    let stack = behavior(METADATA, &[&click(1), &click(2), REFUSED_CLICK]);
    let registry = stack.server_sound_definitions();
    let click = registry.get(&SoundEventName::parse("demo:click")).unwrap();
    assert_eq!(click.origin().pack.index(), Some(1));
    assert_eq!(click.duration(), Some(SoundDuration::Ticks(40)));
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "core/unrecognized_field",
        Severity::Error,
        2,
        METADATA,
    );
}

// ---- texture sets ----
//
// Path-keyed. Exactly one pack's `.texture_set.json` (or its plain image) is used for a given
// texture path, chosen by stack index before the file is read; every failure ends with the
// texture replaced by the fallback image, which is the missing texture. Present behaviour: TAKEN: the lower packs' sets load, are listed, and
// are not what the client uses.

const STONE: &str = "textures/blocks/stone";
const STONE_SET: &str = "textures/blocks/stone.texture_set.json";

/// A 1.21.30 texture set with the layers `layers`.
fn texture_set(layers: &str) -> String {
    format!(r#"{{"format_version": "1.21.30", "minecraft:texture_set": {{{layers}}}}}"#)
}

/// A set without the colour layer: the post-validation refuses it.
fn refused_texture_set() -> String {
    texture_set(r#""metalness_emissive_roughness": [1, 2, 3]"#)
}

#[test]
fn texture_set_two_packs_a_refused_top_set_takes_the_texture_path() {
    let stack = resource(
        STONE_SET,
        &[
            &refused_texture_set(),
            &texture_set(r#""color": [1, 2, 3, 4]"#),
        ],
    );
    let textures = stack.texture_sets();
    let file = stack
        .resolve_texture(&TexturePath::parse(STONE.to_owned()))
        .unwrap();
    assert_eq!(file.stack_index, 0);
    assert_eq!(file.pack.index(), Some(0));
    assert!(file.is_missing(), "the client shows the missing texture");
    assert!(file.set.is_none() && file.image.is_none());
    // The lower set is loaded and listed, and not used.
    let listed: Vec<Option<usize>> = textures
        .sets()
        .map(|(_, set)| set.origin().pack.index())
        .collect();
    assert_eq!(listed, [Some(1)]);
    let rejected = file.rejected.unwrap();
    assert_eq!(rejected.origin.pack.index(), Some(0));
    assert_eq!(rejected.origin.path.as_str(), STONE_SET);
    assert!(rejected.diagnostics.iter().any(|f| {
        f.code.id() == "texture_set/set_no_color"
            && f.severity == Severity::Error
            && f.code.to_string() == "There must at least be a color layer in any TextureSet"
    }));
    reported(
        textures.diagnostics(),
        "texture_set/set_no_color",
        Severity::Error,
        0,
        STONE_SET,
    );
}

#[test]
fn texture_set_three_packs_a_refused_top_set_takes_the_texture_path() {
    let stack = resource(
        STONE_SET,
        &[
            &refused_texture_set(),
            &texture_set(r#""color": [10, 20, 30, 40]"#),
            &texture_set(r#""color": [1, 2, 3, 4]"#),
        ],
    );
    let textures = stack.texture_sets();
    let file = stack
        .resolve_texture(&TexturePath::parse(STONE.to_owned()))
        .unwrap();
    assert_eq!(file.stack_index, 0);
    assert!(file.is_missing(), "neither lower set is used");
    assert!(file.set.is_none() && file.image.is_none());
    let listed: Vec<(Option<usize>, Option<Rgba>)> = textures
        .sets()
        .map(|(_, set)| (set.origin().pack.index(), set.uniform(LayerKind::Color)))
        .collect();
    assert_eq!(listed.len(), 2, "{listed:?}");
    assert!(listed.iter().all(|(pack, _)| *pack != Some(0)));
    let rejected = file.rejected.unwrap();
    assert_eq!(rejected.origin.pack.index(), Some(0));
    reported(
        textures.diagnostics(),
        "texture_set/set_no_color",
        Severity::Error,
        0,
        STONE_SET,
    );
}

// ---- blocks.json ----
//
// A field-level merge: a higher pack overrides a lower pack field by field, not entry by
// entry, and an invalid field in a higher pack leaves the lower pack's value in place. Two kinds: a copy that is refused
// whole (no JSON), and one refused field of an entry. Present behaviour for both: fallback,
// per field: each field has the value of the highest pack below that holds a valid one.

const BLOCKS: &str = "blocks.json";

/// A 1.21.40 `blocks.json` whose `obsidian` entry has `members`.
fn obsidian(members: &str) -> String {
    format!(r#"{{"format_version": "1.21.40", "obsidian": {{{members}}}}}"#)
}

fn obsidian_id() -> BlockId {
    BlockId::parse("minecraft:obsidian")
}

#[test]
fn blocks_json_two_packs_a_refused_top_copy_leaves_the_lower_entry_effective() {
    let low = obsidian(r#""textures": "l", "sound": "l""#);
    let stack = resource(BLOCKS, &[r#"{"obsidian": }"#, &low]);
    let registry = stack.client_blocks();
    let block = registry.get(&obsidian_id()).unwrap();
    assert_eq!(block.origin().pack.index(), Some(1));
    assert_eq!(block.face_textures(), ["l"; 6]);
    assert_eq!(block.sound(), "l");
    reported(
        registry.diagnostics(),
        "blocks_json/not_json",
        Severity::Info, // L2, refused without a game line
        0,
        BLOCKS,
    );
}

#[test]
fn blocks_json_three_packs_a_refused_top_copy_leaves_the_middle_fields_over_the_low_ones() {
    let middle = obsidian(r#""sound": "m""#);
    let low = obsidian(r#""textures": "l", "sound": "l""#);
    let stack = resource(BLOCKS, &[r#"{"obsidian": }"#, &middle, &low]);
    let registry = stack.client_blocks();
    let block = registry.get(&obsidian_id()).unwrap();
    // The middle pack's field wins; the field it does not set is the low pack's.
    assert_eq!(block.sound(), "m");
    assert_eq!(block.face_textures(), ["l"; 6]);
    assert_eq!(block.origin().pack.index(), Some(1));
    assert_eq!(block.origin().contributors.len(), 1);
    assert_eq!(block.origin().contributors[0].index(), Some(2));
    reported(
        registry.diagnostics(),
        "blocks_json/not_json",
        Severity::Info, // L2, refused without a game line
        0,
        BLOCKS,
    );
}

#[test]
fn blocks_json_two_packs_a_refused_top_field_leaves_the_lower_value_effective() {
    let top = obsidian(r#""textures": 5, "sound": "t""#);
    let low = obsidian(r#""textures": "l", "sound": "l""#);
    let stack = resource(BLOCKS, &[&top, &low]);
    let registry = stack.client_blocks();
    let block = registry.get(&obsidian_id()).unwrap();
    // Only the refused field falls back; its valid neighbour is the top pack's.
    assert_eq!(block.face_textures(), ["l"; 6]);
    assert_eq!(block.sound(), "t");
    reported(
        registry.diagnostics(),
        "blocks_json/bad_textures",
        Severity::Warning,
        0,
        BLOCKS,
    );
}

#[test]
fn blocks_json_three_packs_a_refused_top_field_leaves_the_middle_value_effective() {
    let top = obsidian(r#""textures": 5"#);
    let middle = obsidian(r#""textures": "m""#);
    let low = obsidian(r#""textures": "l", "sound": "l""#);
    let stack = resource(BLOCKS, &[&top, &middle, &low]);
    let registry = stack.client_blocks();
    let block = registry.get(&obsidian_id()).unwrap();
    assert_eq!(block.face_textures(), ["m"; 6]);
    assert_eq!(block.sound(), "l");
    reported(
        registry.diagnostics(),
        "blocks_json/bad_textures",
        Severity::Warning,
        0,
        BLOCKS,
    );
}

// ---- render controllers ----
//
// A controller that fails to parse leaves any earlier definition of that name in place, so a
// broken override in a higher pack silently falls back to the lower pack's controller (ours adds the `RenderControllerFallback`
// warning). Present behaviour: fallback.

const RENDER_CONTROLLERS: &str = "render_controllers/cow.json";

/// A 1.8.0 file whose `controller.render.cow` has `members`.
fn cow(members: &str) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "render_controllers": {{"controller.render.cow": {{{members}}}}}}}"#
    )
}

/// A valid `controller.render.cow` with the geometry `geometry`.
fn cow_with_geometry(geometry: &str) -> String {
    cow(&format!(
        r#""geometry": "{geometry}", "textures": ["Texture.default"]"#
    ))
}

#[test]
fn render_controller_two_packs_a_refused_top_controller_leaves_the_lower_controller_effective() {
    // No `textures`: the controller is refused after its name is read.
    let top = cow(r#""geometry": "Geometry.top""#);
    let stack = resource(
        RENDER_CONTROLLERS,
        &[&top, &cow_with_geometry("Geometry.l")],
    );
    let registry = stack.render_controllers();
    let cow = registry
        .get(&RenderControllerId::parse("controller.render.cow"))
        .unwrap();
    assert_eq!(cow.origin().pack.index(), Some(1));
    assert_eq!(cow.geometry.as_str(), "geometry.l");
    assert_eq!(registry.rejected().len(), 1);
    // The refusal and the notice of the fallback: both in the refused pack, at its file.
    reported(
        registry.diagnostics(),
        "render_controller/must_contain_textures",
        Severity::Error,
        0,
        RENDER_CONTROLLERS,
    );
    reported(
        registry.diagnostics(),
        "core/render_controller_fallback",
        Severity::Info, // ours
        0,
        RENDER_CONTROLLERS,
    );
}

#[test]
fn render_controller_three_packs_a_refused_top_controller_leaves_the_middle_controller_effective() {
    // No `textures`: the controller is refused after its name is read.
    let top = cow(r#""geometry": "Geometry.top""#);
    let stack = resource(
        RENDER_CONTROLLERS,
        &[
            &top,
            &cow_with_geometry("Geometry.m"),
            &cow_with_geometry("Geometry.l"),
        ],
    );
    let registry = stack.render_controllers();
    let cow = registry
        .get(&RenderControllerId::parse("controller.render.cow"))
        .unwrap();
    assert_eq!(cow.origin().pack.index(), Some(1));
    assert_eq!(cow.geometry.as_str(), "geometry.m");
    assert_eq!(registry.rejected().len(), 1);
    reported(
        registry.diagnostics(),
        "render_controller/must_contain_textures",
        Severity::Error,
        0,
        RENDER_CONTROLLERS,
    );
    reported(
        registry.diagnostics(),
        "core/render_controller_fallback",
        Severity::Info, // ours
        0,
        RENDER_CONTROLLERS,
    );
}
