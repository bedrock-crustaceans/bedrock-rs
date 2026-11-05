//! The rows that need the whole stack (`bp.aim_assist()`): aa12 (nothing registered), aa18,
//! aa19, and the rows of the link pass: the lookup half of aa04b, C3 and C4.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::id::{AimAssistCategoryId, AimAssistPresetId};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::*;
use super::pack_common::bp;

fn preset_id(text: &str) -> AimAssistPresetId {
    AimAssistPresetId::parse(text)
}

#[test]
fn aa18_the_higher_pack_wins_per_identifier() {
    let stack = stack(&[
        &[
            (
                "aim_assist/presets/p.json",
                preset_json(
                    "1.21.120",
                    r#""identifier": "test:p", "hand_settings": "test:top""#,
                ),
            ),
            (
                "aim_assist/categories/c.json",
                categories_json("1.21.50", &category("test:c", r#""entity_default": 9"#)),
            ),
        ],
        &[
            (
                "aim_assist/presets/other_name.json",
                preset_json(
                    "1.21.120",
                    r#""identifier": "test:p", "hand_settings": "test:bottom", "default_item_settings": "test:d""#,
                ),
            ),
            (
                "aim_assist/presets/q.json",
                preset_json("1.21.50", r#""identifier": "test:q""#),
            ),
            (
                "aim_assist/categories/c.json",
                categories_json(
                    "1.21.50",
                    &format!(
                        "{}, {}",
                        category("test:c", r#""entity_default": 1, "block_default": 2"#),
                        category("test:only_below", "")
                    ),
                ),
            ),
        ],
    ]);
    let aim = stack.aim_assist();
    assert_eq!(aim.presets.len(), 2);
    let kept = aim.presets.get(&preset_id("test:p")).unwrap();
    assert_eq!(kept.hand_settings.as_ref().unwrap().as_str(), "test:top");
    assert_eq!(
        kept.default_item_settings, None,
        "replaced whole, not merged"
    );
    assert_eq!(kept.origin().pack.name(), "bp 1");
    assert!(aim.presets.contains(&preset_id("test:q")));
    // Categories are registered one by one: the higher pack's `test:c` wins, the lower pack's
    // other category of the same file path stays.
    assert_eq!(aim.categories.len(), 2);
    let category = aim
        .categories
        .get(&AimAssistCategoryId::parse("test:c"))
        .unwrap();
    assert_eq!(category.priorities.entity_default(), 9);
    assert_eq!(category.priorities.block_default, None);
    assert_eq!(category.origin().pack.name(), "bp 1");
    assert!(
        aim.categories
            .contains(&AimAssistCategoryId::parse("test:only_below"))
    );
    assert!(aim.diagnostics().all(|f| f.severity < Severity::Warning));
    assert_eq!(
        aim.diagnostics()
            .filter(|f| f.code.name() == "ShadowedDefinition")
            .count(),
        2
    );
    assert!(std::ptr::eq(aim, stack.aim_assist()));
}

#[test]
fn aa18_within_one_pack_the_later_file_replaces_the_earlier_one() {
    let stack = stack(&[&[
        (
            "aim_assist/presets/a.json",
            preset_json(
                "1.21.120",
                r#""identifier": "test:p", "hand_settings": "test:a""#,
            ),
        ),
        (
            "aim_assist/presets/b.json",
            preset_json(
                "1.21.120",
                r#""identifier": "test:p", "hand_settings": "test:b""#,
            ),
        ),
    ]]);
    let kept = stack
        .aim_assist()
        .presets
        .get(&preset_id("test:p"))
        .unwrap();
    assert_eq!(kept.hand_settings.as_ref().unwrap().as_str(), "test:b");
}

#[test]
fn aa19_only_json_files_directly_in_the_two_folders_are_read() {
    let preset =
        |identifier: &str| preset_json("1.21.120", &format!(r#""identifier": "{identifier}""#));
    let stack = stack(&[&[
        ("aim_assist/presets/x.json", preset("test:x")),
        ("aim_assist/presets/sub/x.json", preset("test:nested")),
        ("aim_assist/presets/x.JSON", preset("test:upper")),
        ("aim_assist/presets/x.jsonc", preset("test:jsonc")),
        ("aim_assist/x.json", preset("test:above")),
        ("cameras/presets/x.json", preset("test:in_cameras")),
        (
            "aim_assist/categories/c.json",
            categories_json("1.21.50", &category("test:c", "")),
        ),
        (
            "aim_assist/categories/sub/c.json",
            categories_json("1.21.50", &category("test:nested", "")),
        ),
        (
            "aim_assist/categories/c.JSON",
            categories_json("1.21.50", &category("test:upper", "")),
        ),
    ]]);
    let aim = stack.aim_assist();
    let presets: Vec<&str> = aim.presets.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(presets, ["test:x"]);
    let categories: Vec<&str> = aim.categories.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(categories, ["test:c"]);
    // The files that are not read raise nothing.
    assert_eq!(aim.diagnostics().count(), 0);
}

#[test]
fn aa12_a_rejected_categories_file_registers_none_of_its_categories() {
    let stack = stack(&[&[
        (
            "aim_assist/categories/bad.json",
            categories_json(
                "1.21.50",
                &format!(
                    "{}, {}",
                    category("test:melee", ""),
                    category("default", "")
                ),
            ),
        ),
        (
            "aim_assist/categories/good.json",
            categories_json("1.21.50", &category("test:ranged", "")),
        ),
        (
            "aim_assist/presets/bad.json",
            preset_json("1.21.120", r#""identifier": "nonamespace""#),
        ),
    ]]);
    let aim = stack.aim_assist();
    let categories: Vec<&str> = aim.categories.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(categories, ["test:ranged"]);
    assert_eq!(aim.categories.rejected().len(), 1);
    assert_eq!(
        aim.categories.rejected()[0].origin.path.as_str(),
        "aim_assist/categories/bad.json"
    );
    assert!(aim.presets.is_empty());
    assert_eq!(aim.presets.rejected().len(), 1);
    assert_eq!(
        errors(aim.diagnostics()),
        [
            finding(
                "InvalidValue",
                "minecraft:aim_assist_categories/categories/1/name",
                "Namespace required for category: default"
            ),
            finding(
                "InvalidValue",
                "minecraft:aim_assist_preset/identifier",
                "Namespace required for preset Id: nonamespace"
            )
        ]
    );
}

/// The link findings of one behaviour pack with `files`.
fn link(files: &[(&'static str, String)]) -> Diagnostics {
    let mut spec = bp(1);
    for (path, text) in files {
        spec = spec.file(path, leak(text.clone()).as_bytes());
    }
    let pack = spec.open();
    let world = WorldPacks::parse(
        Some(
            format!(
                r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
                pack.id().uuid
            )
            .as_bytes(),
        ),
        None,
    )
    .expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::User);
    repository.stacks(&world, LoadOptions::default()).link()
}

/// The names of the `UnresolvedReference` warnings of `d`, as each detail gives them.
fn unresolved(d: &Diagnostics) -> Vec<String> {
    d.iter()
        // A miss the rule tells (a warning) or the game's validator logs (a content-log
        // error, W3-A).
        .filter(|f| {
            f.code.name() == "UnresolvedReference"
                && matches!(f.severity, Severity::Warning | Severity::Error)
        })
        .map(|f| {
            f.detail
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(feature = "block")]
#[test]
fn aa04b_the_lookup_uses_the_normalised_item_and_entity_names() {
    // `minecraft:WOOL:3` is looked up as `minecraft:wool`, the entity without its event: both
    // are vanilla names, so the only miss is the block that does not exist.
    let d = link(&[(
        "aim_assist/presets/p.json",
        preset_json(
            "1.21.120",
            r#""identifier": "test:p", "item_settings": {"minecraft:WOOL:3": "minecraft:default"},
            "liquid_targeting_list": ["minecraft:Bucket:1"],
            "exclusion_settings": {"entities": ["minecraft:pig<minecraft:entity_born>"], "blocks": ["test:no_such_block"]}"#,
        ),
    )]);
    assert_eq!(unresolved(&d), [": block 'test:no_such_block'"], "{d:?}");
}

#[test]
fn an_unknown_category_reference_is_an_unresolved_reference_warning_deviation() {
    // vanilla = "linked-asset validator only": a category no categories file defines.
    let d = link(&[
        (
            "aim_assist/presets/p.json",
            preset_json(
                "1.21.120",
                r#""identifier": "test:p", "hand_settings": "test:known", "default_item_settings": "test:unknown""#,
            ),
        ),
        (
            "aim_assist/categories/c.json",
            categories_json("1.21.50", &category("test:known", "")),
        ),
    ]);
    assert_eq!(
        unresolved(&d),
        [": aim assist category 'test:unknown'"],
        "{d:?}"
    );
}

#[test]
fn a_camera_presets_aim_assist_preset_resolves_against_the_aim_assist_presets() {
    let d = link(&[(
        "cameras/presets/c.json",
        r#"{"format_version": "1.21.60", "minecraft:camera_preset": {"identifier": "test:c", "inherit_from": "minecraft:free", "aim_assist": {"preset": "test:no_such_preset"}}}"#.to_owned(),
    )]);
    // C3: neither folder checks it, an information of ours.
    let told: Vec<String> = d
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference" && f.severity == Severity::Info)
        .map(|f| {
            f.detail
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(told, [": aim assist preset 'test:no_such_preset'"], "{d:?}");
    assert!(unresolved(&d).is_empty(), "{d:?}");
}

/// What the link pass reads of an upgraded preset's places (`Diagnostics::place_of`): the step
/// that copies a 1.21.50 `exclusion_list` into both lists of `exclusion_settings` records no
/// move, so an entry is placed where the document holds it, and the link pass names the old
/// list itself (`told_as_the_file_has_it`). The findings of the file stay in it.
#[test]
fn an_upgraded_preset_places_each_member_where_the_document_holds_it() {
    use bedrock_addon::diagnostic::JsonPath;
    let stack = stack(&[&[(
        "aim_assist/presets/p.json",
        preset_json(
            "1.21.50",
            r#""identifier": "test:p", "exclusion_list": ["minecraft:stone", "test:nothing"]"#,
        ),
    )]]);
    let group = stack.aim_assist();
    let document = group
        .presets
        .get(&preset_id("test:p"))
        .expect("the preset loads");
    for list in ["blocks", "entities"] {
        let mut at = JsonPath::of_keys(["minecraft:aim_assist_preset", "exclusion_settings", list]);
        at.push_index(1);
        assert_eq!(document.diagnostics().place_of(&at), at, "{list}");
    }
    for finding in document.diagnostics() {
        assert_eq!(
            finding.location.path.as_ref(),
            Some(&document.origin().path),
            "{finding:?}"
        );
    }
}
