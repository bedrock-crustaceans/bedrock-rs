//! Stack resolution and binding order, fixtures `a20`–`a29`: which files of a stack are attachables, and the item
//! bindings the stack gathers. Packs are listed highest priority first; every manifest is at
//! 1.21.0 unless the row says otherwise. `a24` waits for the link pass (`link.rs`).

use bedrock_addon::diagnostic::{Code, Kind, Severity};
use bedrock_addon::document::{Content, Document, Registry};
use bedrock_addon::families::attachable::{Attachable, Condition, ItemBindingEntry, ItemBindings};
use bedrock_addon::id::AttachableId;
use bedrock_addon::pack::StackSet;
use bedrock_addon::storage::PackPath;

use super::common::*;
use super::stacks::*;

const FAMILY: &str = "attachable";
const ITEM_AT: &str = "minecraft:attachable/description/item";

fn attachables(set: &StackSet) -> &Registry<Attachable> {
    set.resource.attachables()
}

fn get<'s>(set: &'s StackSet, identifier: &str) -> Option<&'s Document<Attachable>> {
    attachables(set).get(&AttachableId::parse(identifier))
}

/// A 1.10.0 attachable `identifier` with `members`, at `min_engine_version` `mev` when given.
fn named(identifier: &str, mev: Option<&str>, members: &str) -> String {
    let mut parts = vec![format!(r#""identifier": "{identifier}""#)];
    parts.extend(mev.map(|v| format!(r#""min_engine_version": "{v}""#)));
    if !members.is_empty() {
        parts.push(members.to_owned());
    }
    doc("1.10.0", &format!("{{{}}}", parts.join(", ")))
}

/// `example:wand` ([`named`]).
fn wand_doc(mev: Option<&str>, members: &str) -> String {
    named("example:wand", mev, members)
}

const WAND: &str = "attachables/wand.json";

/// One binding as `sequence item -> attachable [condition] @pack/file`, `!` when shadowed.
fn shown(entry: &ItemBindingEntry) -> String {
    let condition = match &entry.condition {
        Condition::Always => "always".to_owned(),
        Condition::Molang(source) => format!("`{}`", source.as_str()),
        other => panic!("{other:?}"),
    };
    format!(
        "{} {} -> {} [{condition}] @{}/{}{}",
        entry.sequence,
        entry.item,
        entry.attachable,
        entry.pack.name(),
        entry.file,
        if entry.shadowed { " !" } else { "" }
    )
}

/// The bindings of `item` across the stack, in the order vanilla gathered them.
fn bindings_of(set: &StackSet, item: &str) -> Vec<String> {
    ItemBindings::from_registry(attachables(set))
        .get(item)
        .iter()
        .map(|entry| shown(entry))
        .collect()
}

/// The registry's findings with code `name`, as `severity pack/path json-path`.
/// The texts of the findings named `name`.
fn texts(set: &StackSet, name: &str) -> Vec<String> {
    attachables(set)
        .diagnostics()
        .filter(|f| f.code.id() == name)
        .map(|f| f.code.to_string())
        .collect()
}

fn found(set: &StackSet, name: &str) -> Vec<String> {
    found_by(set, |code| code.id() == name)
}

/// The findings whose code `keep` accepts, as `severity pack/path json_path`.
fn found_by(set: &StackSet, keep: impl Fn(&Code) -> bool) -> Vec<String> {
    attachables(set)
        .diagnostics()
        .filter(|f| keep(&f.code))
        .map(|f| {
            format!(
                "{:?} {}/{} {}",
                f.severity,
                f.location.pack.as_ref().unwrap().name(),
                f.location.path.as_ref().unwrap(),
                f.location.json_path
            )
        })
        .collect()
}

/// Whether `code` is a definition the legacy `AttachableShadowed` was: the shadowing of an
/// attachable, whichever wording.
fn is_attachable_shadowed(code: &Code) -> bool {
    matches!(
        code.id(),
        "core/attachable_shadowed"
            | "core/attachable_shadowed_above_stack"
            | "core/attachable_shadowed_older"
    )
}

/// Requires the registry to have no finding at warning level or above.
#[track_caller]
fn no_warning(set: &StackSet) {
    let warnings: Vec<String> = attachables(set)
        .diagnostics()
        .filter(|f| f.severity >= Severity::Warning)
        .map(line)
        .collect();
    assert!(warnings.is_empty(), "{}", warnings.join("\n"));
}

// ---- rule 26: equal versions merge, and every binding stays ----

#[test]
fn a20_two_packs_equal_mev_merge() {
    let bottom = wand_doc(
        None,
        r#""textures": {"default": "a"}, "item": "example:wand""#,
    );
    let top = wand_doc(
        None,
        r#""textures": {"default": "b"}, "geometry": {"default": "geometry.g"}, "item": {"example:wand": "q.is_sneaking"}"#,
    );
    let set = stack(&[pack(&[(WAND, &top)]), pack(&[(WAND, &bottom)])]);
    assert_eq!(attachables(&set).len(), 1);
    let wand = get(&set, "example:wand").unwrap();
    assert_eq!(wand.description.slots.textures["default"].as_str(), "b");
    assert_eq!(
        wand.description.slots.geometry["default"].as_str(),
        "geometry.g"
    );
    // Two bindings, in load order: the lower pack's first. A client tries the top one first.
    let expected = [
        "0 example:wand -> example:wand [always] @pack 2/attachables/wand.json",
        "1 example:wand -> example:wand [`q.is_sneaking`] @pack 1/attachables/wand.json",
    ];
    assert_eq!(bindings_of(&set, "example:wand"), expected);
    let own: Vec<String> = wand.bound_items().iter().map(shown).collect();
    assert_eq!(own, expected);
    let index = ItemBindings::from_registry(attachables(&set));
    let tried: Vec<usize> = index.tried("example:wand").map(|e| e.sequence).collect();
    assert_eq!(tried, [1, 0]);
    assert!(found(&set, "core/item_binding_shadowed").is_empty());
    no_warning(&set);
    // The merged description holds the later document's `item`.
    assert_eq!(wand.binding().unwrap().1, wand.bound_items()[1].condition);
}

#[test]
fn a20_three_packs_merge_and_keep_three_bindings() {
    let low = wand_doc(None, r#""item": "example:a""#);
    let mid = wand_doc(None, r#""hide_armor": true"#);
    let high = wand_doc(None, r#""item": "example:c""#);
    let set = stack(&[
        pack(&[(WAND, &high)]),
        pack(&[(WAND, &mid)]),
        pack(&[(WAND, &low)]),
    ]);
    let wand = get(&set, "example:wand").unwrap();
    let own: Vec<String> = wand.bound_items().iter().map(shown).collect();
    assert_eq!(
        own,
        [
            "0 example:a -> example:wand [always] @pack 3/attachables/wand.json",
            "1 example:c -> example:wand [always] @pack 1/attachables/wand.json",
        ]
    );
    assert!(wand.hide_armor());
    assert_eq!(wand.origin().contributors.len(), 2);
    // A later document without `item` leaves the earlier one's in the description.
    let set = stack(&[pack(&[(WAND, &mid)]), pack(&[(WAND, &low)])]);
    let wand = get(&set, "example:wand").unwrap();
    assert_eq!(wand.binding().unwrap().0.as_str(), "example:a");
    assert_eq!(wand.bound_items().len(), 1);
}

#[test]
fn a20_a_merge_invents_no_member() {
    // `animations` and `sound_effects` are absent from both documents: the merged definition
    // has neither, and writes neither.
    let low = wand_doc(None, r#""textures": {"default": "a"}"#);
    let high = wand_doc(None, r#""textures": {"other": "b"}"#);
    let set = stack(&[pack(&[(WAND, &high)]), pack(&[(WAND, &low)])]);
    let wand = get(&set, "example:wand").unwrap();
    assert_eq!(wand.description.slots.textures.len(), 2);
    assert_eq!(wand.description.slots.animations, None);
    assert_eq!(wand.description.slots.sound_effects, None);
    let written = wand.to_json().unwrap();
    assert!(!written.contains("animations"), "{written}");
    assert!(!written.contains("sound_effects"), "{written}");
    // One side's member, empty or not, is the merged one's.
    let low = wand_doc(None, r#""animations": {}"#);
    let high = wand_doc(None, r#""sound_effects": {"s": "mob.wand"}"#);
    let set = stack(&[pack(&[(WAND, &high)]), pack(&[(WAND, &low)])]);
    let wand = get(&set, "example:wand").unwrap();
    assert!(
        wand.description
            .slots
            .animations
            .as_ref()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        wand.description.slots.sound_effects.as_ref().unwrap().len(),
        1
    );
}

// ---- rule 27: a higher version replaces, the earlier bindings stay ----

#[test]
fn a21_higher_mev_replaces() {
    let bottom = wand_doc(
        Some("1.16.0"),
        r#""textures": {"default": "a"}, "item": "example:wand""#,
    );
    let top = wand_doc(Some("1.20.0"), r#""geometry": {"default": "geometry.g"}"#);
    let set = stack(&[pack(&[(WAND, &top)]), pack(&[(WAND, &bottom)])]);
    let wand = get(&set, "example:wand").unwrap();
    // The definition is the top document alone.
    assert!(wand.description.slots.textures.is_empty());
    assert_eq!(wand.description.item, None);
    assert_eq!(wand.origin().pack.name(), "pack 1");
    // The bottom document's binding stays, and now selects the replacement.
    assert_eq!(
        bindings_of(&set, "example:wand"),
        ["0 example:wand -> example:wand [always] @pack 2/attachables/wand.json !"]
    );
    assert_eq!(wand.bound_items().len(), 1);
    assert!(wand.bound_items()[0].shadowed);
    assert_eq!(
        found(&set, "core/item_binding_shadowed"),
        [format!("Info pack 2/attachables/wand.json {ITEM_AT}")]
    );
    assert!(found_by(&set, is_attachable_shadowed).is_empty());
    no_warning(&set);
}

#[test]
fn a21_a_chain_of_replacements_keeps_every_binding_and_shadows_all_but_the_last() {
    let low = wand_doc(Some("1.16.0"), r#""item": "example:a""#);
    let mid = wand_doc(Some("1.18.0"), r#""item": "example:b""#);
    let high = wand_doc(Some("1.20.0"), r#""item": "example:c""#);
    let set = stack(&[
        pack(&[(WAND, &high)]),
        pack(&[(WAND, &mid)]),
        pack(&[(WAND, &low)]),
    ]);
    let wand = get(&set, "example:wand").unwrap();
    let own: Vec<String> = wand.bound_items().iter().map(shown).collect();
    assert_eq!(
        own,
        [
            "0 example:a -> example:wand [always] @pack 3/attachables/wand.json !",
            "1 example:b -> example:wand [always] @pack 2/attachables/wand.json !",
            "2 example:c -> example:wand [always] @pack 1/attachables/wand.json",
        ]
    );
    assert_eq!(found(&set, "core/item_binding_shadowed").len(), 2);
    // A document merged into the replacement is not shadowed.
    let merged = wand_doc(Some("1.20.0"), r#""item": "example:d""#);
    let set = stack(&[
        pack(&[(WAND, &merged)]),
        pack(&[(WAND, &high)]),
        pack(&[(WAND, &low)]),
    ]);
    let shadowed: Vec<bool> = get(&set, "example:wand")
        .unwrap()
        .bound_items()
        .iter()
        .map(|entry| entry.shadowed)
        .collect();
    assert_eq!(shadowed, [true, false, false]);
    assert_eq!(found(&set, "core/item_binding_shadowed").len(), 1);
}

// ---- rules 24 and 25: an ignored document adds no binding ----

#[test]
fn a22_lower_mev_ignored() {
    let bottom = wand_doc(Some("1.20.0"), r#""textures": {"default": "a"}"#);
    let top = wand_doc(Some("1.16.0"), r#""item": "example:wand""#);
    let set = stack(&[pack(&[(WAND, &top)]), pack(&[(WAND, &bottom)])]);
    let wand = get(&set, "example:wand").unwrap();
    assert_eq!(wand.origin().pack.name(), "pack 2");
    assert_eq!(wand.description.item, None);
    // The document ignored is the higher pack's, for its lower `min_engine_version`: the
    // finding says so.
    assert_eq!(
        found(&set, "core/attachable_shadowed_older"),
        ["Info pack 1/attachables/wand.json "]
    );
    assert_eq!(
        texts(&set, "core/attachable_shadowed_older"),
        ["attachable ignored: its min_engine_version is below the loaded definition's"]
    );
    // Vanilla returns before it reads `item`: no binding.
    assert!(wand.bound_items().is_empty());
    assert!(ItemBindings::from_registry(attachables(&set)).is_empty());
    assert!(found(&set, "core/item_binding_shadowed").is_empty());
}

#[test]
fn a22_an_ignored_document_between_two_that_count_adds_nothing() {
    let low = wand_doc(Some("1.18.0"), r#""item": "example:a""#);
    let mid = wand_doc(Some("1.16.0"), r#""item": "example:b""#);
    let high = wand_doc(Some("1.18.0"), r#""item": "example:c""#);
    let set = stack(&[
        pack(&[(WAND, &high)]),
        pack(&[(WAND, &mid)]),
        pack(&[(WAND, &low)]),
    ]);
    let index = ItemBindings::from_registry(attachables(&set));
    assert_eq!(
        index.items().collect::<Vec<_>>(),
        ["example:a", "example:c"]
    );
    assert_eq!(index.len(), 2);
    assert!(index.get("example:b").is_empty());
    // The sequence has no gap for the ignored one.
    assert_eq!(index.get("example:c")[0].sequence, 1);
}

#[test]
fn a23_mev_above_stack() {
    let future = wand_doc(Some("1.21.0"), r#""item": "example:wand""#);
    let files = [(WAND, future.as_str())];
    let at = |version, files| TestPack {
        min_engine_version: version,
        files,
    };
    let set = stack(&[at((1, 20, 0), &files)]);
    assert!(attachables(&set).is_empty());
    // No other pack exists: the document is above the stack's version, and the finding
    // says so.
    assert_eq!(
        found(&set, "core/attachable_shadowed_above_stack"),
        ["Info pack 1/attachables/wand.json "]
    );
    assert_eq!(
        texts(&set, "core/attachable_shadowed_above_stack"),
        ["attachable ignored: its min_engine_version is above the stack's"]
    );
    assert!(ItemBindings::from_registry(attachables(&set)).is_empty());
    // Exactly at the stack's version it is not above it; the stack's version is its highest
    // manifest's.
    let set = stack(&[at((1, 21, 0), &files)]);
    assert_eq!(bindings_of(&set, "example:wand").len(), 1);
    let set = stack(&[at((1, 20, 0), &files), at((1, 21, 0), &[])]);
    assert_eq!(bindings_of(&set, "example:wand").len(), 1);
    assert!(found_by(&set, is_attachable_shadowed).is_empty());
}

// ---- which files are attachables ----

const ENTITY: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "example:ghost"}}}"#;

#[test]
fn a25_attachable_under_entity_dir() {
    let sword = named("example:sword", None, r#""item": "example:blade""#);
    let knife = named("example:knife", None, r#""item": "example:blade""#);
    let axe = named("example:axe", None, r#""item": "example:blade""#);
    // The top pack's `entity/` file is loaded before the bottom pack's `attachables/` file:
    // every `entity/` directory comes before any `attachables/` one.
    let set = stack(&[
        pack(&[
            ("entity/sword.json", &sword),
            ("entity/ghost.entity.json", ENTITY),
            ("attachables/axe.json", &axe),
        ]),
        pack(&[("attachables/knife.json", &knife)]),
    ]);
    assert_eq!(attachables(&set).len(), 3);
    assert_eq!(
        bindings_of(&set, "example:blade"),
        [
            "0 example:blade -> example:sword [always] @pack 1/entity/sword.json",
            "1 example:blade -> example:knife [always] @pack 2/attachables/knife.json",
            "2 example:blade -> example:axe [always] @pack 1/attachables/axe.json",
        ]
    );
    assert_eq!(
        get(&set, "example:sword").unwrap().origin().path.as_str(),
        "entity/sword.json"
    );
    // The client entity beside it is the other family's: nothing is said of it here.
    assert!(attachables(&set).rejected().is_empty());
    assert!(found(&set, "core/ignored_payload").is_empty());
    assert!(found_by(&set, |code| code.kind() == Kind::MissingRequired).is_empty());
    assert!(client_entities(&set).is_none_or(|n| n == 1));
}

#[test]
fn a25_an_entity_file_that_is_no_attachable_is_left_to_the_client_entities() {
    // No JSON, an empty root, no payload at all: under `entity/` they are the client entity
    // family's to report; under `attachables/` they are this family's.
    for text in ["{", "{}", r#"{"format_version": "1.10.0"}"#, "[]"] {
        let set = stack(&[pack(&[("entity/x.json", text)])]);
        assert!(attachables(&set).rejected().is_empty(), "{text}");
        assert_eq!(attachables(&set).diagnostics().count(), 0, "{text}");
        let set = stack(&[pack(&[("attachables/x.json", text)])]);
        assert_eq!(attachables(&set).rejected().len(), 1, "{text}");
    }
    // A wrong attachable under `entity/` is this family's rejection.
    let bad = named("example:sword", None, r#""render_distance": 3"#);
    let set = stack(&[pack(&[("entity/sword.json", &bad)])]);
    assert_eq!(attachables(&set).rejected().len(), 1);
    // Only `.json` files are read, in sub-folders too.
    let sword = named("example:sword", None, "");
    let set = stack(&[pack(&[
        ("attachables/deep/er/sword.json", &sword),
        ("attachables/sword.json5", &sword),
        ("attachables/sword.JSON", &sword),
    ])]);
    assert_eq!(attachables(&set).len(), 1);
    assert_eq!(
        get(&set, "example:sword").unwrap().origin().path.as_str(),
        "attachables/deep/er/sword.json"
    );
    assert!(attachables(&set).rejected().is_empty());
}

const BOTH: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "example:ghost"}}, "minecraft:attachable": {"description": {"identifier": "example:sword", "item": "example:blade"}}}"#;

#[test]
fn a26_both_payloads() {
    // Rule 10: the file is the client entity; the attachable part is ignored, with a warning.
    for path in ["entity/both.json", "attachables/both.json"] {
        let set = stack(&[pack(&[(path, BOTH)])]);
        assert!(attachables(&set).is_empty(), "{path}");
        assert!(attachables(&set).rejected().is_empty(), "{path}");
        assert_eq!(
            found(&set, "core/ignored_payload"),
            [format!("Info pack 1/{path} minecraft:attachable")]
        );
        assert_eq!(attachables(&set).diagnostics().count(), 1);
        assert!(ItemBindings::from_registry(attachables(&set)).is_empty());
    }
    // Under `entity/` the client entity loads.
    let set = stack(&[pack(&[("entity/both.json", BOTH)])]);
    assert!(client_entities(&set).is_none_or(|n| n == 1));
    // A `minecraft:client_entity` that is no object does not make the file a client entity.
    let text = BOTH.replace(r#"{"description": {"identifier": "example:ghost"}}"#, "3");
    let set = stack(&[pack(&[("entity/both.json", &text)])]);
    assert!(found(&set, "core/ignored_payload").is_empty());
    // It is an attachable then, whose root holds a member of the wrong type.
    assert_eq!(
        attachables(&set).len() + attachables(&set).rejected().len(),
        1
    );
}

#[test]
fn a26_read_alone_a_file_with_both_payloads_is_refused() {
    let d = rejected(attachable(BOTH, &cx()));
    expect_at(
        &d,
        "core/ignored_payload",
        Severity::Error,
        "minecraft:attachable",
    );
    assert_eq!(d.len(), 1);
}

#[test]
fn a26_a_client_entity_under_attachables_is_the_client_entities() {
    // Vanilla loads it as a client entity, whatever the directory, and
    // so does `rp.client_entities()`. The file is no
    // attachable and the attachables say nothing of it, as of an `entity/` file.
    let set = stack(&[pack(&[("attachables/ghost.json", ENTITY)])]);
    assert!(attachables(&set).is_empty());
    assert!(attachables(&set).rejected().is_empty());
    assert_eq!(found(&set, "core/ignored_payload"), Vec::<String>::new());
    assert_eq!(attachables(&set).diagnostics().count(), 0);
    assert!(client_entities(&set).is_none_or(|n| n == 1));
    // The app's own `definitions/attachables` is read the same way, before every pack.
    let set = stack_over(&[], &[("definitions/attachables/ghost.json", ENTITY)]);
    assert!(attachables(&set).is_empty());
    assert_eq!(attachables(&set).diagnostics().count(), 0);
    assert!(client_entities(&set).is_none_or(|n| n == 1));
    // Read alone as an attachable it is refused.
    let d = rejected(attachable(ENTITY, &cx()));
    expect_at(
        &d,
        "core/ignored_payload",
        Severity::Error,
        "minecraft:client_entity",
    );
}

// ---- vanilla ----

/// The 55 files of the game's sample resource pack, as pack files.
fn vanilla() -> Vec<(String, String)> {
    let directory = fixtures(FAMILY).join("vanilla");
    let mut files: Vec<(String, String)> = std::fs::read_dir(&directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            (
                format!("attachables/{name}"),
                std::fs::read_to_string(&path).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

fn borrowed(files: &[(String, String)]) -> Vec<(&str, &str)> {
    files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect()
}

const PLAYER: &str = "query.owner_identifier == 'minecraft:player'";

#[test]
fn a27_vanilla_armor_pair() {
    let files = vanilla();
    let pair: Vec<(&str, &str)> = borrowed(&files)
        .into_iter()
        .filter(|(path, _)| path.contains("iron_chestplate"))
        .collect();
    assert_eq!(pair.len(), 2);
    let set = stack(&[pack(&pair)]);
    assert_eq!(
        bindings_of(&set, "minecraft:iron_chestplate"),
        [format!(
            "0 minecraft:iron_chestplate -> minecraft:iron_chestplate.player [`{PLAYER}`] @pack 1/attachables/iron_chestplate.player.json"
        )]
    );
    // The implicit route: the attachable whose identifier is the item's.
    let implicit = get(&set, "minecraft:iron_chestplate").unwrap();
    assert!(implicit.bound_items().is_empty());
    assert_eq!(implicit.binding(), None);
    no_warning(&set);
}

#[test]
fn a28_vanilla_corpus() {
    let files = vanilla();
    assert_eq!(files.len(), 55);
    let set = stack(&[pack(&borrowed(&files))]);
    let registry = attachables(&set);
    assert_eq!(registry.len(), 55);
    assert!(registry.rejected().is_empty());
    no_warning(&set);
    let index = ItemBindings::from_registry(registry);
    // 25 bindings, one per item, each the player override of an armour piece.
    assert_eq!(index.len(), 25);
    for item in index.items() {
        let entries = index.get(item);
        assert_eq!(entries.len(), 1, "{item}");
        assert_eq!(entries[0].attachable.as_str(), format!("{item}.player"));
        assert!(
            matches!(&entries[0].condition, Condition::Molang(source) if source.as_str() == PLAYER)
        );
        // And the default for everything that is no player.
        assert!(get(&set, item).is_some(), "{item}");
    }
    let implicit_only = registry
        .iter()
        .filter(|(_, document)| document.bound_items().is_empty())
        .count();
    assert_eq!(implicit_only, 30);
    // The identifier need not be the file's name.
    assert_eq!(
        get(&set, "minecraft:turtle_helmet")
            .unwrap()
            .origin()
            .path
            .as_str(),
        "attachables/turtle_shell_helmet.json"
    );
}

#[test]
fn a29_install_layering() {
    // The app's own `definitions/attachables` below every pack, and a pack that declares the
    // same armour again (vanilla_1.26.10 does, for 59 files): neither sets
    // `min_engine_version`, so each pair merges, and each `*.player` file binds a second
    // time. The retail layers themselves are loaded by `smoke_attachable`.
    let files = vanilla();
    let boots: Vec<(String, String)> = files
        .iter()
        .filter(|(path, _)| path.contains("chainmail_boots"))
        .cloned()
        .collect();
    assert_eq!(boots.len(), 2);
    let app: Vec<(String, String)> = boots
        .iter()
        .map(|(path, text)| (format!("definitions/{path}"), text.clone()))
        .collect();
    let set = stack_over(&[pack(&borrowed(&boots))], &borrowed(&app));
    assert_eq!(attachables(&set).len(), 2);
    assert_eq!(
        bindings_of(&set, "minecraft:chainmail_boots"),
        [
            format!(
                "0 minecraft:chainmail_boots -> minecraft:chainmail_boots.player [`{PLAYER}`] @app data/definitions/attachables/chainmail_boots.player.json"
            ),
            format!(
                "1 minecraft:chainmail_boots -> minecraft:chainmail_boots.player [`{PLAYER}`] @pack 1/attachables/chainmail_boots.player.json"
            ),
        ]
    );
    let player = get(&set, "minecraft:chainmail_boots.player").unwrap();
    assert_eq!(player.origin().contributors.len(), 1);
    no_warning(&set);
    let all: Vec<String> = attachables(&set)
        .diagnostics()
        .filter(|f| f.code.id() != "core/molang_not_checked")
        .map(line)
        .collect();
    assert!(all.is_empty(), "{}", all.join("\n"));
    // The app data is the lowest layer: a pack's higher `min_engine_version` replaces it.
    let newer = named(
        "minecraft:chainmail_boots.player",
        Some("1.20.0"),
        r#""textures": {"default": "textures/new"}"#,
    );
    let set = stack_over(
        &[pack(&[("attachables/new.json", &newer)])],
        &borrowed(&app),
    );
    let player = get(&set, "minecraft:chainmail_boots.player").unwrap();
    assert_eq!(player.origin().pack.name(), "pack 1");
    assert!(player.bound_items()[0].shadowed);
    // And app data is read from `definitions/attachables` only, with its sub-folders.
    let sword = named("example:sword", None, "");
    let set = stack_over(
        &[],
        &[
            ("definitions/attachables/deep/sword.json", &sword),
            ("definitions/entity/sword.json", &sword),
            ("attachables/sword.json", &sword),
        ],
    );
    assert_eq!(attachables(&set).len(), 1);
}

// ---- rule 29: two bindings for one item ----

#[test]
fn rule_29_two_always_bindings_of_one_item_overlap() {
    let sword = named("example:sword", None, r#""item": "example:blade""#);
    let knife = named("example:knife", None, r#""item": "example:blade""#);
    let axe = named("example:axe", None, r#""item": "example:blade""#);
    let set = stack(&[
        pack(&[("attachables/knife.json", &knife)]),
        pack(&[("attachables/sword.json", &sword)]),
    ]);
    // Only the later one can be selected: the note is at the one that never is.
    assert_eq!(
        found(&set, "core/item_binding_overlap"),
        [format!("Info pack 2/attachables/sword.json {ITEM_AT}")]
    );
    no_warning(&set);
    // Three: each but the last.
    let set = stack(&[
        pack(&[("attachables/axe.json", &axe)]),
        pack(&[("attachables/knife.json", &knife)]),
        pack(&[("attachables/sword.json", &sword)]),
    ]);
    assert_eq!(
        found(&set, "core/item_binding_overlap"),
        [
            format!("Info pack 3/attachables/sword.json {ITEM_AT}"),
            format!("Info pack 2/attachables/knife.json {ITEM_AT}"),
        ]
    );
    assert_eq!(bindings_of(&set, "example:blade").len(), 3);
}

#[test]
fn rule_29_a_conditional_binding_or_another_item_is_no_overlap() {
    let sword = named("example:sword", None, r#""item": "example:blade""#);
    let conditional = named(
        "example:knife",
        None,
        r#""item": {"example:blade": "q.is_sneaking"}"#,
    );
    let other = named("example:axe", None, r#""item": "example:Blade""#);
    for (top, bottom) in [(&conditional, &sword), (&sword, &conditional)] {
        let set = stack(&[
            pack(&[("attachables/top.json", top)]),
            pack(&[("attachables/bottom.json", bottom)]),
        ]);
        assert!(found(&set, "core/item_binding_overlap").is_empty());
        assert_eq!(bindings_of(&set, "example:blade").len(), 2);
    }
    // Items compare byte for byte.
    let set = stack(&[
        pack(&[("attachables/top.json", &other)]),
        pack(&[("attachables/bottom.json", &sword)]),
    ]);
    assert!(found(&set, "core/item_binding_overlap").is_empty());
    let index = ItemBindings::from_registry(attachables(&set));
    assert_eq!(index.get("example:blade").len(), 1);
    assert_eq!(index.get("example:Blade").len(), 1);
    assert!(index.get("minecraft:example:blade").is_empty());
}

// ---- a rejected document ----

#[test]
fn a_rejected_document_binds_nothing_and_leaves_the_lower_definition() {
    // "Every `item` problem is file-level": the file contributes nothing, and nothing it
    // would have replaced is touched. Two packs, then three.
    let good = wand_doc(
        Some("1.16.0"),
        r#""textures": {"default": "a"}, "item": "example:a""#,
    );
    let bad = wand_doc(Some("1.20.0"), r#""item": {}"#);
    let set = stack(&[pack(&[(WAND, &bad)]), pack(&[(WAND, &good)])]);
    let wand_ = get(&set, "example:wand").unwrap();
    assert_eq!(wand_.origin().pack.name(), "pack 2");
    assert_eq!(attachables(&set).rejected().len(), 1);
    assert_eq!(
        bindings_of(&set, "example:a"),
        ["0 example:a -> example:wand [always] @pack 2/attachables/wand.json"]
    );
    assert!(found(&set, "core/item_binding_shadowed").is_empty());
    let lowest = wand_doc(Some("1.10.0"), r#""item": "example:lowest""#);
    let set = stack(&[
        pack(&[(WAND, &bad)]),
        pack(&[(WAND, &good)]),
        pack(&[(WAND, &lowest)]),
    ]);
    let wand_ = get(&set, "example:wand").unwrap();
    assert_eq!(wand_.origin().pack.name(), "pack 2");
    let own: Vec<String> = wand_.bound_items().iter().map(shown).collect();
    assert_eq!(
        own,
        [
            "0 example:lowest -> example:wand [always] @pack 3/attachables/wand.json !",
            "1 example:a -> example:wand [always] @pack 2/attachables/wand.json",
        ]
    );
    // Rejected in the middle: the lowest stays the definition the top merges into or
    // replaces.
    let top = wand_doc(Some("1.10.0"), r#""item": "example:top""#);
    let set = stack(&[
        pack(&[(WAND, &top)]),
        pack(&[(WAND, &bad)]),
        pack(&[(WAND, &lowest)]),
    ]);
    let own: Vec<String> = get(&set, "example:wand")
        .unwrap()
        .bound_items()
        .iter()
        .map(shown)
        .collect();
    assert_eq!(
        own,
        [
            "0 example:lowest -> example:wand [always] @pack 3/attachables/wand.json",
            "1 example:top -> example:wand [always] @pack 1/attachables/wand.json",
        ]
    );
}

// ---- the index ----

#[test]
fn the_bindings_of_a_stack_are_numbered_in_load_order_across_identifiers() {
    let a = named("Example:A", None, r#""item": {"example:x": true}"#);
    let b = named("example:b", None, r#""item": {"example:x": 1}"#);
    let c = named("example:c", None, "");
    let d = named("example:d", None, r#""item": "example:y""#);
    let set = stack(&[
        pack(&[("attachables/a.json", &a), ("attachables/d.json", &d)]),
        pack(&[("attachables/b.json", &b), ("attachables/c.json", &c)]),
    ]);
    let index = ItemBindings::from_registry(attachables(&set));
    assert_eq!(
        index.items().collect::<Vec<_>>(),
        ["example:x", "example:y"]
    );
    assert_eq!(
        bindings_of(&set, "example:x"),
        [
            "0 example:x -> example:b [``] @pack 2/attachables/b.json",
            "1 example:x -> example:a [`true`] @pack 1/attachables/a.json",
        ]
    );
    assert_eq!(index.get("example:y")[0].sequence, 2);
    assert!(index.get("example:x")[0].condition.is_never());
    assert!(!index.is_empty());
    // A document read alone has no stack binding; its own is `binding()`.
    let alone = loaded(Attachable::parse(
        &PackPath::from_static("attachables/a.json"),
        a.as_bytes(),
        &cx(),
    ));
    assert!(alone.bound_items().is_empty());
    assert!(alone.binding().is_some());
}

#[test]
fn a_binding_s_condition_takes_its_pack_s_molang_version() {
    use bedrock_addon::molang::pack_molang_version;
    use bedrock_addon::version::MinEngineVersion;
    let a = named(
        "example:a",
        None,
        r#""item": {"example:x": "q.is_sneaking"}"#,
    );
    let b = named("example:b", None, r#""item": {"example:x": true}"#);
    let at = |version, files| TestPack {
        min_engine_version: version,
        files,
    };
    let top = [("attachables/a.json", a.as_str())];
    let bottom = [("attachables/b.json", b.as_str())];
    let set = stack(&[at((1, 21, 100), &top), at((1, 16, 0), &bottom)]);
    let index = ItemBindings::from_registry(attachables(&set));
    let versions: Vec<_> = index
        .get("example:x")
        .iter()
        .map(|entry| match &entry.condition {
            Condition::Molang(source) => source.raw_version(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        versions,
        [
            Some(pack_molang_version(Some(&MinEngineVersion::new(1, 16, 0)))),
            Some(pack_molang_version(Some(&MinEngineVersion::new(
                1, 21, 100
            )))),
        ]
    );
    assert_ne!(versions[0], versions[1]);
}

/// How many client entities the stack loaded; `None` without the `entity` feature.
#[cfg(feature = "entity")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the same signature as the build without the `entity` feature, which counts nothing"
)]
fn client_entities(set: &StackSet) -> Option<usize> {
    Some(set.resource.client_entities().len())
}

/// How many client entities the stack loaded; `None` without the `entity` feature.
#[cfg(not(feature = "entity"))]
fn client_entities(_: &StackSet) -> Option<usize> {
    None
}

#[test]
fn the_index_is_in_load_order_where_the_registry_s_order_is_another() {
    // `example:a` is defined first, without `item`; `example:b` binds the item; then a higher
    // pack's `example:a` is merged in and binds it too. The registry lists `a` before `b`;
    // the bindings were gathered `b` first.
    let a_low = named("example:a", None, "");
    let b = named(
        "example:b",
        None,
        r#""item": {"example:x": "q.is_sneaking"}"#,
    );
    let a_high = named("example:a", None, r#""item": {"example:x": "q.is_baby"}"#);
    let set = stack(&[
        pack(&[("attachables/a.json", &a_high)]),
        pack(&[("attachables/a.json", &a_low), ("attachables/b.json", &b)]),
    ]);
    let order: Vec<&str> = attachables(&set)
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(order, ["example:a", "example:b"]);
    assert_eq!(
        bindings_of(&set, "example:x"),
        [
            "0 example:x -> example:b [`q.is_sneaking`] @pack 2/attachables/b.json",
            "1 example:x -> example:a [`q.is_baby`] @pack 1/attachables/a.json",
        ]
    );
    let index = ItemBindings::from_registry(attachables(&set));
    let tried: Vec<&str> = index
        .tried("example:x")
        .map(|entry| entry.attachable.as_str())
        .collect();
    assert_eq!(tried, ["example:a", "example:b"]);
}
