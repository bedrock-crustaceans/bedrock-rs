//! `p01`–`p08`: which definition of
//! an identifier a stack ends up with, by `description.min_engine_version`. Packs are listed
//! highest priority first; every manifest is at 1.21.0 unless the row says otherwise.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Document, Registry};
use bedrock_addon::families::client_entity::ClientEntity;
use bedrock_addon::id::EntityId;
use bedrock_addon::pack::StackSet;
use bedrock_addon::visual::{Color, HasVisuals};

use super::common::*;

fn ghost_of(set: &StackSet) -> Option<&Document<ClientEntity>> {
    entities(set).get(&EntityId::parse("demo:ghost"))
}

fn entities(set: &StackSet) -> &Registry<ClientEntity> {
    set.resource.client_entities()
}

/// The files of the `ClientEntityShadowed` infos, as `pack name/path`.
fn shadowed(set: &StackSet) -> Vec<String> {
    entities(set)
        .diagnostics()
        .filter(|f| f.code.name() == "ClientEntityShadowed")
        .map(|f| {
            assert_eq!(f.severity, Severity::Info);
            format!(
                "{}/{}",
                f.location.pack.as_ref().unwrap().name(),
                f.location.path.as_ref().unwrap()
            )
        })
        .collect()
}

/// The texts of the `ClientEntityShadowed` infos.
fn shadow_texts(set: &StackSet) -> Vec<String> {
    entities(set)
        .diagnostics()
        .filter(|f| f.code.name() == "ClientEntityShadowed")
        .map(|f| f.code.to_string())
        .collect()
}

/// A 1.10.0 `demo:ghost` with `members`, at `min_engine_version` `mev` when given.
fn at_mev(mev: Option<&str>, members: &str) -> String {
    let mev = mev.map_or(String::new(), |v| format!(r#""min_engine_version": "{v}""#));
    let all: Vec<&str> = [mev.as_str(), members]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    ghost("1.10.0", &all.join(", "))
}

const FILE: &str = "entity/ghost.entity.json";

#[test]
fn p01_same_mev_merge() {
    let low = at_mev(
        None,
        r#""geometry": {"default": "geometry.low"}, "textures": {"default": "textures/low"}"#,
    );
    let high = at_mev(None, r#""textures": {"default": "textures/other"}"#);
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    let visuals = ghost.visuals();
    assert_eq!(
        visuals.geometry("default").unwrap().as_str(),
        "geometry.low"
    );
    assert_eq!(
        visuals.texture("default").unwrap().as_str(),
        "textures/other"
    );
    // The merged definition is the higher pack's document, with the lower one contributing.
    assert_eq!(ghost.origin().pack.name(), "pack 1");
    assert_eq!(ghost.origin().contributors.len(), 1);
    assert_eq!(ghost.origin().contributors[0].name(), "pack 2");
    assert!(shadowed(&set).is_empty());
    assert_eq!(entities(&set).len(), 1);
}

#[test]
fn p01_merge_details() {
    // A later `false` overrides an earlier `true`; a later scale at or below 0 is not stored;
    // keys that differ only in case are one key.
    let low = at_mev(
        None,
        r#""textures": {"Default": "textures/low"}, "enable_attachables": true, "hide_armor": true, "scripts": {"scale": "2.0", "parent_setup": "variable.a = 1;"}"#,
    );
    let high = at_mev(
        None,
        r#""textures": {"default": "textures/high"}, "enable_attachables": false, "scripts": {"scale": "3.0"}"#,
    );
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    assert_eq!(ghost.description.textures.len(), 1);
    assert_eq!(
        ghost.description.textures["default"].as_str(),
        "textures/high"
    );
    assert!(!ghost.enable_attachables());
    assert!(ghost.hide_armor());
    let scripts = ghost.description.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.scale.as_ref().unwrap().source().unwrap().as_str(),
        "3.0"
    );
    assert!(scripts.parent_setup.is_some());
    // Identifiers that differ only in case are one identifier.
    let upper = doc(
        "1.10.0",
        r#"{"identifier": "Demo:Ghost", "hide_armor": false}"#,
    );
    let set = stack(&[pack(&[(FILE, &upper)]), pack(&[(FILE, &low)])]);
    assert_eq!(entities(&set).len(), 1);
    assert!(!ghost_of(&set).unwrap().hide_armor());
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn p01_a_later_scale_at_or_below_zero_keeps_the_earlier_one() {
    let low = ghost("1.26.0", r#""held_item_scale": 2.0"#);
    let high = ghost("1.26.0", r#""held_item_scale": 0"#);
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    assert_eq!(ghost_of(&set).unwrap().held_item_scale(), 2.0);
    // Above 0 it replaces.
    let high = ghost("1.26.0", r#""held_item_scale": 0.5"#);
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    assert_eq!(ghost_of(&set).unwrap().held_item_scale(), 0.5);
}

#[test]
fn p02_higher_mev_replaces() {
    let old = at_mev(Some("1.8.0"), r#""geometry": {"a": "geometry.a"}"#);
    let new = at_mev(Some("1.16.0"), r#""textures": {"b": "textures/b"}"#);
    // The low pack has the higher version: the high pack's document is ignored.
    let set = stack(&[pack(&[(FILE, &old)]), pack(&[(FILE, &new)])]);
    let ghost = ghost_of(&set).unwrap();
    assert_eq!(ghost.origin().pack.name(), "pack 2");
    assert!(ghost.description.geometry.is_empty());
    assert_eq!(shadowed(&set), ["pack 1/entity/ghost.entity.json"]);
    // The reason is in the text: the ignored document is the higher pack's.
    assert_eq!(
        shadow_texts(&set),
        ["client entity ignored: its min_engine_version is below the loaded definition's"]
    );
    // The ignored document is no layer unless the load keeps shadowed definitions.
    assert_eq!(
        entities(&set)
            .layers(&EntityId::parse("demo:ghost"))
            .count(),
        1
    );
    // Swapped: the high pack replaces the definition whole; no key of the low pack survives.
    let set = stack(&[pack(&[(FILE, &new)]), pack(&[(FILE, &old)])]);
    let ghost = ghost_of(&set).unwrap();
    assert_eq!(ghost.origin().pack.name(), "pack 1");
    assert!(ghost.description.geometry.is_empty());
    assert_eq!(ghost.description.textures.len(), 1);
    assert!(ghost.origin().contributors.is_empty());
    assert!(shadowed(&set).is_empty());
}

#[test]
fn p03_above_full_stack() {
    let future = at_mev(Some("1.20.0"), "");
    let files = [(FILE, future.as_str())];
    let at_1_16 = |files| TestPack {
        min_engine_version: (1, 16, 0),
        files,
    };
    let set = stack(&[at_1_16(&files), at_1_16(&[])]);
    assert!(ghost_of(&set).is_none());
    assert!(entities(&set).is_empty());
    assert_eq!(shadowed(&set), ["pack 1/entity/ghost.entity.json"]);
    // The reason is in the text: no other document exists.
    assert_eq!(
        shadow_texts(&set),
        ["client entity ignored: its min_engine_version is above the stack's"]
    );
    // The stack's version is its highest manifest's: one pack at 1.20.0 lets the document in,
    // and exactly at the stack's version is not above it.
    let set = stack(&[
        at_1_16(&files),
        TestPack {
            min_engine_version: (1, 20, 0),
            files: &[],
        },
    ]);
    assert!(ghost_of(&set).is_some());
    assert!(shadowed(&set).is_empty());
}

#[test]
fn p04_vanilla_pair() {
    let modern = doc(
        "1.10.0",
        r#"{"identifier": "minecraft:guardian", "min_engine_version": "1.8.0", "textures": {"default": "textures/entity/guardian"}}"#,
    );
    let legacy = doc(
        "1.8.0",
        r#"{"identifier": "minecraft:guardian", "textures": {"default": "textures/entity/old"}}"#,
    );
    let id = EntityId::parse("minecraft:guardian");
    // Whichever file the walk reaches first, the 1.8.0 one wins.
    for (first, second) in [
        (
            ("entity/guardian.entity.json", modern.as_str()),
            ("entity/guardian.v1.0.entity.json", legacy.as_str()),
        ),
        (
            ("entity/a.json", legacy.as_str()),
            ("entity/b.json", modern.as_str()),
        ),
    ] {
        let set = stack(&[pack(&[first, second])]);
        let guardian = entities(&set).get(&id).unwrap();
        assert_eq!(
            guardian.description.textures["default"].as_str(),
            "textures/entity/guardian"
        );
        assert_eq!(entities(&set).len(), 1);
    }
}

#[test]
fn p05_render_controllers_replaced() {
    let low = at_mev(
        None,
        r#""render_controllers": ["controller.render.a", "controller.render.b"]"#,
    );
    let high = at_mev(None, r#""render_controllers": ["controller.render.c"]"#);
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    let controllers: Vec<String> = ghost
        .visuals()
        .render_controllers
        .iter()
        .map(|c| c.controller().unwrap().as_str().to_owned())
        .collect();
    assert_eq!(controllers, ["controller.render.c"]);
    // A later document without the member keeps the earlier list.
    let silent_high = at_mev(None, "");
    let set = stack(&[pack(&[(FILE, &silent_high)]), pack(&[(FILE, &low)])]);
    assert_eq!(
        ghost_of(&set).unwrap().visuals().render_controllers.len(),
        2
    );
}

#[test]
fn p06_legacy_mev_below_1_8() {
    let low = at_mev(Some("1.2.6"), r#""geometry": {"a": "geometry.a"}"#);
    let none = at_mev(None, r#""geometry": {"b": "geometry.b"}"#);
    // 1.2.6 is above an absent version's 0.0.0: the higher pack's document is ignored.
    let set = stack(&[pack(&[(FILE, &none)]), pack(&[(FILE, &low)])]);
    assert!(
        ghost_of(&set)
            .unwrap()
            .description
            .geometry
            .contains_key("a")
    );
    assert_eq!(shadowed(&set).len(), 1);
    // And it is 1.2.6, not more: 1.2.7 replaces it.
    let above = at_mev(Some("1.2.7"), r#""geometry": {"c": "geometry.c"}"#);
    let set = stack(&[pack(&[(FILE, &above)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    assert!(ghost.description.geometry.contains_key("c"));
    assert!(!ghost.description.geometry.contains_key("a"));
}

#[test]
fn p07_animate_replaced() {
    let animations =
        r#""animations": {"a": "animation.a", "b": "animation.b", "c": "animation.c"}"#;
    let low = at_mev(
        None,
        &format!(r#"{animations}, "scripts": {{"animate": ["a", "b"]}}"#),
    );
    let high = at_mev(
        None,
        &format!(r#"{animations}, "scripts": {{"animate": ["c"]}}"#),
    );
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    let animate = ghost
        .description
        .scripts
        .as_ref()
        .unwrap()
        .animate
        .as_ref()
        .unwrap();
    let names: Vec<_> = animate.iter().map(|entry| entry.name().unwrap()).collect();
    assert_eq!(names, ["c"]);
}

#[test]
fn p08_spawn_egg_replaced() {
    let low = at_mev(None, r#""spawn_egg": {"texture": "x", "texture_index": 3}"#);
    let high = at_mev(None, r##""spawn_egg": {"base_color": "#00ff00"}"##);
    let set = stack(&[pack(&[(FILE, &high)]), pack(&[(FILE, &low)])]);
    let ghost = ghost_of(&set).unwrap();
    let egg = ghost.description.spawn_egg.as_ref().unwrap().resolved();
    assert_eq!(egg.texture, "");
    assert_eq!(egg.texture_index, 0);
    assert_eq!(
        egg.base_color,
        Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 0.0
        }
    );
}

#[test]
fn a_rejected_document_takes_no_part_and_is_listed() {
    let good = at_mev(None, r#""geometry": {"a": "geometry.a"}"#);
    let bad = at_mev(Some("1.20.0"), r#""render_distance": 3"#);
    let set = stack(&[pack(&[(FILE, &bad)]), pack(&[(FILE, &good)])]);
    assert!(
        ghost_of(&set)
            .unwrap()
            .description
            .geometry
            .contains_key("a")
    );
    assert_eq!(entities(&set).rejected().len(), 1);
}

// "The directory does not matter: an
// `attachables/` file with a `minecraft:client_entity` payload is a client entity", and "every
// `entity/` directory is processed before any `attachables/` directory".
#[test]
fn a_client_entity_under_attachables_is_loaded_after_every_entity_folder() {
    const ELSEWHERE: &str = "attachables/ghost.json";
    let alone = at_mev(None, r#""textures": {"default": "textures/attached"}"#);
    let set = stack(&[pack(&[(ELSEWHERE, &alone)])]);
    let ghost = ghost_of(&set).expect("the client entity under attachables/");
    assert_eq!(ghost.origin().path.as_str(), ELSEWHERE);
    assert_eq!(entities(&set).diagnostics().count(), 0);
    // The lower pack's `attachables/` file is read after the higher pack's `entity/` file:
    // at one `min_engine_version` it is merged over it.
    let in_entity = at_mev(
        None,
        r#""geometry": {"default": "geometry.e"}, "textures": {"default": "textures/entity"}"#,
    );
    let set = stack(&[pack(&[(FILE, &in_entity)]), pack(&[(ELSEWHERE, &alone)])]);
    let visuals = ghost_of(&set).unwrap().visuals();
    assert_eq!(visuals.geometry.unwrap()["default"].as_str(), "geometry.e");
    assert_eq!(
        visuals.textures.unwrap()["default"].as_str(),
        "textures/attached"
    );
    assert_eq!(entities(&set).len(), 1);
    // A file with both payloads is the client entity, in either folder.
    let both = alone.replacen(
        r#""minecraft:client_entity""#,
        r#""minecraft:attachable": {"description": {"identifier": "demo:hat"}}, "minecraft:client_entity""#,
        1,
    );
    let set = stack(&[pack(&[(ELSEWHERE, &both)])]);
    assert!(ghost_of(&set).is_some());
}

/// What is no client entity under `attachables/` is the attachable family's: an attachable,
/// a file without a payload, a file that is no JSON. The client entities say nothing of it.
#[test]
fn an_attachables_file_that_is_no_client_entity_is_left_to_the_attachables() {
    let set = stack(&[pack(&[
        (
            "attachables/hat.json",
            r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:hat"}}}"#,
        ),
        ("attachables/empty.json", "{}"),
        ("attachables/broken.json", "{ not json"),
        ("attachables/notes.txt", "x"),
    ])]);
    assert!(entities(&set).is_empty());
    assert!(entities(&set).rejected().is_empty());
    assert_eq!(entities(&set).diagnostics().count(), 0);
}

// ---- a larger stack ----

#[test]
fn a_2_000_identifier_stack_merges_and_ignores_by_min_engine_version() {
    fn borrowed(files: &[(String, String)]) -> Vec<(&str, &str)> {
        files
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str()))
            .collect()
    }

    // 3 000 files in two packs: the lower pack defines 2 000 identifiers; the upper pack
    // defines the first 1 000 again, alternately with the same `min_engine_version` (merged
    // into the lower definition) and with a lower one (ignored, with a
    // `ClientEntityShadowed` info). What the registry holds, without a time bound: the cost
    // of the merge is pinned on the combine alone (`entity/client/load.rs`).
    const IDENTIFIERS: usize = 2_000;
    let file = |i: usize, mev: &str, texture: &str| {
        (
            format!("entity/e{i}.entity.json"),
            doc(
                "1.26.0",
                &format!(
                    r#"{{"identifier": "demo:e{i}", "min_engine_version": "{mev}", "textures": {{"default": "textures/{texture}"}}}}"#
                ),
            ),
        )
    };
    let lower: Vec<(String, String)> = (0..IDENTIFIERS)
        .map(|i| file(i, "1.20.0", "lower"))
        .collect();
    let upper: Vec<(String, String)> = (0..IDENTIFIERS / 2)
        .map(|i| file(i, if i % 2 == 0 { "1.20.0" } else { "1.10.0" }, "upper"))
        .collect();
    let (upper, lower) = (borrowed(&upper), borrowed(&lower));
    let set = stack(&[pack(&upper), pack(&lower)]);
    let registry = entities(&set);
    assert_eq!(registry.len(), IDENTIFIERS);
    assert_eq!(registry.rejected().len(), 0);
    // A quarter of the identifiers is merged from both packs, a quarter keeps the lower
    // pack's definition with the upper one ignored, and half is the lower pack's alone.
    assert_eq!(shadowed(&set).len(), IDENTIFIERS / 4);
    assert_eq!(registry.diagnostics().count(), IDENTIFIERS / 4);
    let mut merged = 0;
    for i in 0..IDENTIFIERS {
        let entity = registry
            .get(&EntityId::parse(format!("demo:e{i}")))
            .unwrap();
        let texture = entity
            .visuals()
            .texture("default")
            .unwrap()
            .as_str()
            .to_owned();
        let contributors = entity.origin().contributors.len();
        if i < IDENTIFIERS / 2 && i % 2 == 0 {
            assert_eq!(
                (texture.as_str(), contributors),
                ("textures/upper", 1),
                "{i}"
            );
            assert_eq!(entity.origin().pack.name(), "pack 1");
            merged += 1;
        } else {
            assert_eq!(
                (texture.as_str(), contributors),
                ("textures/lower", 0),
                "{i}"
            );
            assert_eq!(entity.origin().pack.name(), "pack 2");
        }
    }
    assert_eq!(merged, IDENTIFIERS / 4);
}

/// An `entity/` file is walked by the client entities and by the attachables, and the driver
/// tells each collection of a file it cannot read. A report lists the file once per pack:
/// the same file of two packs is two files, the same file of two collections one.
#[cfg(feature = "item")]
#[test]
fn an_unreadable_entity_file_is_reported_once_by_the_stack() {
    fn unread<'a>(findings: impl Iterator<Item = &'a Diagnostic>) -> Vec<(String, Severity)> {
        let mut found: Vec<(String, Severity)> = findings
            .filter(|f| f.code.name() == "ReadError")
            .map(|f| {
                (
                    format!(
                        "{}/{}",
                        f.location.pack.as_ref().unwrap().name(),
                        f.location.path.as_ref().unwrap()
                    ),
                    f.severity,
                )
            })
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    }

    use bedrock_addon::diagnostic::Diagnostic;

    let files = [("entity/ghost.json", "{}"), ("entity/other.json", "{}")];
    let set = stack_with_unread(
        &[pack(&files), pack(&files)],
        &["p1/entity/ghost.json", "p2/entity/ghost.json"],
    );
    let once = [
        ("pack 1/entity/ghost.json".to_owned(), Severity::Error),
        ("pack 2/entity/ghost.json".to_owned(), Severity::Error),
    ];
    // Both collections were loaded and neither has a definition from the unread file.
    assert!(entities(&set).iter().next().is_none());
    assert!(set.resource.attachables().iter().next().is_none());
    assert_eq!(unread(set.resource.report().iter()), once);
    assert_eq!(unread(set.load_all().iter()), once);
}
