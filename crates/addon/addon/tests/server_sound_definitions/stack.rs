//! The stack: the accessor, the registry and the link pass.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Document, Registry};
use bedrock_addon::families::server_sound_definitions::{SoundDuration, SoundMetadata};
use bedrock_addon::id::SoundEventName;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

use super::common::*;

/// `"functional test" ServerSoundDefinitionsLoadFunctionalTests::LoadedLevel_TryGetSoundEvent_
/// ReturnsDefinition` loads an on-disk `SoundsBP` whose JSON is not in the binary
/// and expects `test:loaded_from_pack` registered; this is that pack written at the one slice.
#[test]
fn a_pack_definition_named_test_loaded_from_pack_registers() {
    let text = file(r#"{"sound_event_name": "test:loaded_from_pack"}"#);
    let stack = PackStack::<Behavior>::of(vec![pack(10, &[(PATH, &text)])], LoadOptions::default())
        .expect("a behaviour stack");
    let registry = stack.server_sound_definitions();
    assert!(
        registry.contains(&SoundEventName::parse("test:loaded_from_pack")),
        "Sound definition from on-disk behavior pack should be registered after level load: {:#?}",
        registry
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(registry.diagnostics().count(), 0);
}

/// A format-2 behaviour-pack manifest.
fn manifest(uuid: u128) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {uuid}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 40]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

fn pack(uuid: u128, files: &[(&str, &str)]) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(uuid).into_bytes(),
    )];
    all.extend(
        files
            .iter()
            .map(|(p, t)| (PackPath::new(p).unwrap(), t.as_bytes().to_vec())),
    );
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A behaviour stack of `packs`, top first, with Beta APIs on (which the family does not ask).
fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

fn timed(seconds: u32) -> String {
    file(&format!(
        r#"{{"sound_event_name": "demo:click", "duration_info": {{"mode": "game_time", "seconds": {seconds}}}}}"#
    ))
}

fn named(registry: &Registry<SoundMetadata>, code: &str) -> Vec<String> {
    registry
        .diagnostics()
        .filter(|d| d.code.name() == code)
        .map(ToString::to_string)
        .collect()
}

fn click(registry: &Registry<SoundMetadata>) -> &Document<SoundMetadata> {
    registry
        .get(&SoundEventName::parse("demo:click"))
        .expect("demo:click")
}

const DUPLICATE: &str = "Duplicate server sound definition for sound event 'demo:click'; the later definition will be ignored.";

#[test]
fn ssd17_a_duplicate_in_one_file_keeps_the_first() {
    let text = file(&format!(
        r#"{{"sound_event_name": "demo:click", "duration_info": {{"mode": "game_time", "seconds": 1}}}}, {CLICK}"#
    ));
    let stack = stack(vec![pack(10, &[(PATH, &text)])]);
    let registry = stack.server_sound_definitions();
    assert_eq!(registry.len(), 1);
    assert!(click(registry).duration_info.is_some());
    let duplicates = named(registry, "DuplicateIdentifier");
    assert_eq!(duplicates.len(), 1);
    assert!(duplicates[0].contains(DUPLICATE) && duplicates[0].starts_with("[error]"));
}

#[test]
fn ssd18_across_packs_the_lowest_pack_wins() {
    let stack = stack(vec![
        pack(10, &[(PATH, &timed(10))]),
        pack(20, &[(PATH, &timed(20))]),
    ]);
    let registry = stack.server_sound_definitions();
    assert_eq!(click(registry).duration(), Some(SoundDuration::Ticks(400)));
    assert_eq!(click(registry).origin().pack.index(), Some(1));
    let duplicates: Vec<_> = registry
        .diagnostics()
        .filter(|d| d.code.name() == "DuplicateIdentifier")
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(
        duplicates[0].location.pack.as_ref().unwrap().index(),
        Some(0),
        "pack A's"
    );
    assert!(duplicates[0].to_string().contains(DUPLICATE));
}

#[test]
fn ssd25_three_packs_report_each_later_definition_deviation() {
    // vanilla = "logOnlyOnce": one line per distinct text.
    let stack = stack(vec![
        pack(10, &[(PATH, &timed(10))]),
        pack(20, &[(PATH, &timed(20))]),
        pack(30, &[(PATH, &timed(30))]),
    ]);
    let registry = stack.server_sound_definitions();
    assert_eq!(
        click(registry).duration(),
        Some(SoundDuration::Ticks(600)),
        "pack C's"
    );
    assert_eq!(named(registry, "DuplicateIdentifier").len(), 2);
}

#[test]
fn ssd19_names_compare_with_case() {
    let text = file(r#"{"sound_event_name": "demo:Click"}, {"sound_event_name": "demo:click"}"#);
    let stack = stack(vec![pack(10, &[(PATH, &text)])]);
    let registry = stack.server_sound_definitions();
    assert_eq!(registry.len(), 2);
    assert!(named(registry, "DuplicateIdentifier").is_empty());
    assert_eq!(registry.diagnostics().count(), 0);
}

#[test]
fn ssd09_a_rejected_file_registers_nothing_and_the_rest_of_the_stack_loads() {
    let bad = file(&format!(
        r#"{CLICK}, {{"sound_event_name": "demo:two", "volume": 1.0}}"#
    ));
    let good = file(r#"{"sound_event_name": "demo:other"}"#);
    let stack = stack(vec![pack(10, &[(PATH, &bad)]), pack(20, &[(PATH, &good)])]);
    let registry = stack.server_sound_definitions();
    assert_eq!(registry.len(), 1);
    assert!(registry.contains(&SoundEventName::parse("demo:other")));
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(named(registry, "UnrecognizedField").len(), 1);
}

#[test]
fn ssd07_the_stack_loads_a_1_26_60_file_without_the_toggle() {
    let packs = || vec![pack(10, &[(PATH, &file(CLICK))])];
    let off = PackStack::<Behavior>::of(packs(), LoadOptions::default()).unwrap();
    assert_eq!(off.server_sound_definitions().len(), 1);
    assert_eq!(off.server_sound_definitions().diagnostics().count(), 0);
    assert_eq!(stack(packs()).server_sound_definitions().len(), 1);
}

#[test]
fn ssd21_only_the_fixed_path_of_a_behaviour_pack_is_read() {
    // An RP-style document elsewhere, and other files of `sounds/`, are not this family's.
    let stack = stack(vec![pack(
        10,
        &[
            ("sounds/other.json", &file(CLICK)),
            ("sounds/nested/sound_definitions.json", &file(CLICK)),
            ("sound_definitions.json", &file(CLICK)),
        ],
    )]);
    let registry = stack.server_sound_definitions();
    assert!(registry.is_empty());
    assert_eq!(registry.diagnostics().count(), 0);
    assert!(registry.rejected().is_empty());
}

#[test]
fn ssd20_a_subpack_copy_replaces_the_packs() {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "sub", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 40]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}], "subpacks": [{{"folder_name": "loud", "name": "Loud", "memory_tier": 0}}]}}"#,
        Uuid::from_u128(50),
        Uuid::from_u128(1050),
    );
    let files = vec![
        (
            PackPath::new("manifest.json").unwrap(),
            manifest.into_bytes(),
        ),
        (PackPath::new(PATH).unwrap(), timed(1).into_bytes()),
        (
            PackPath::new("subpacks/loud/sounds/sound_definitions.json").unwrap(),
            timed(2).into_bytes(),
        ),
    ];
    let pack = Pack::from_files(files, &OpenOptions::default()).expect("the pack opens");
    // `PackStack::of` selects no subpack: the pack's own file is the one read, and the
    // subpack's copy is no file of the pack itself.
    let stack = stack(vec![pack]);
    let registry = stack.server_sound_definitions();
    assert_eq!(click(registry).duration(), Some(SoundDuration::Ticks(20)));
    assert_eq!(registry.len(), 1);
    assert!(named(registry, "DuplicateIdentifier").is_empty());
}

#[test]
fn ssd23_a_name_without_a_resource_pack_definition_is_an_info() {
    use bedrock_addon::pack::{PackRepository, WorldPacks};
    // The behaviour pack declares `demo:missing` and `demo:present`; the resource pack's
    // `sounds/sound_definitions.json` defines `demo:present` only.
    let behavior = pack(
        10,
        &[(
            PATH,
            &file(r#"{"sound_event_name": "demo:present"}, {"sound_event_name": "demo:missing"}"#),
        )],
    );
    let resource_manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "rp", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 40]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(20),
        Uuid::from_u128(1020),
    );
    let resource = Pack::from_files(
        vec![
            (PackPath::new("manifest.json").unwrap(), resource_manifest.into_bytes()),
            (
                PackPath::new(PATH).unwrap(),
                br#"{"format_version": "1.20.20", "sound_definitions": {"demo:present": {"category": "neutral", "sounds": []}}}"#.to_vec(),
            ),
        ],
        &OpenOptions::default(),
    )
    .expect("the resource pack opens");
    let entry = |pack: &Pack| {
        format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            pack.id().uuid
        )
    };
    let world = WorldPacks::parse(
        Some(entry(&behavior).as_bytes()),
        Some(entry(&resource).as_bytes()),
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(behavior, PackOrigin::User);
    repository.add_pack(resource, PackOrigin::User);
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    let stacks = repository.stacks(&world, options);
    // Both definitions load whatever the resource packs define.
    assert_eq!(stacks.behavior.server_sound_definitions().len(), 2);
    // The link pass: one `UnknownSound` info, at the definition whose name the served
    // resource packs do not define (vanilla checks nothing).
    let report = stacks.link();
    let unknown: Vec<&Diagnostic> = report
        .iter()
        .filter(|d| d.code.name() == "UnknownSound")
        .collect();
    assert_eq!(unknown.len(), 1, "{:#?}", lines(&report));
    assert_eq!(unknown[0].severity, Severity::Info);
    assert_eq!(
        unknown[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon_core::pack_identity::PackPath::as_str),
        Some(PATH)
    );
    assert_eq!(
        unknown[0].location.json_path.to_string(),
        "minecraft:server_sound_definitions/1/sound_event_name"
    );
    assert_eq!(
        unknown[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon_core::pack_identity::PackRef::index),
        Some(0)
    );
}
