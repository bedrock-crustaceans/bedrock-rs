//! The world gate and the template-item filter of world start, and the deviation for a pattern the filter leaves out.

use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail, Phase, Severity};
use bedrock_addon::families::item::trim::{ItemTags, TrimData};
use bedrock_addon::id::ItemId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{BaseGameVersion, SemVersion};
use bytes::Bytes;

use super::common::{
    item, material_entry, material_rows, pattern_entry, pattern_rows, vanilla_table,
};
use super::pack_common::bp;

const BOLT: &str = "minecraft:bolt_armor_trim_smithing_template";
const TRIM_TEMPLATES: &str = "minecraft:trim_templates";

/// A stack whose one pack defines no item.
fn stack_without_items() -> PackStack<Behavior> {
    PackStack::<Behavior>::of([bp(1).open()], LoadOptions::default()).unwrap()
}

/// A stack whose one pack, a base-game pack, defines the item of `file`: only a base-game pack
/// (or a pack above one that defines the same item) may define a `minecraft:` item.
fn stack_with_item(file: &'static [u8]) -> PackStack<Behavior> {
    let files = [
        ("manifest.json", Bytes::from(bp(1).manifest())),
        ("items/template.json", Bytes::from_static(file)),
    ]
    .map(|(path, bytes)| (PackPath::from_static(path), bytes));
    let options = OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    };
    let pack = Pack::from_files(files, &options).expect("the pack opens");
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).unwrap();
    assert_eq!(stack.items().len(), 1, "{:?}", stack.items().rejected());
    stack
}

fn tags(tags: &[&str]) -> ItemTags {
    ItemTags {
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
    }
}

/// The built-in items as vanilla has them: every item of the sample packs' item list
/// exists, and the template items of the table carry `minecraft:trim_templates`.
fn vanilla_items(id: &ItemId) -> Option<ItemTags> {
    let exists = bedrock_addon::vanilla::ids()
        .items
        .iter()
        .any(|known| item(known) == *id);
    exists.then(|| {
        if TrimData::vanilla().pattern_for_item(id).is_some() {
            tags(&[TRIM_TEMPLATES])
        } else {
            tags(&[])
        }
    })
}

/// The fixture's patterns without `bolt`, in build order.
fn all_but_bolt() -> Vec<super::common::PatternRow> {
    let mut expected = vanilla_table().patterns;
    assert_eq!(expected.pop().unwrap().item_name, BOLT);
    expected
}

/// The one finding of a filter that left `bolt` out: an info naming the item.
fn assert_left_out(data: &TrimData, kind: &str) {
    let findings: Vec<_> = data.diagnostics().iter().collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = findings[0];
    assert!(matches!(
        finding.code,
        DiagnosticCode::UnresolvedReference { text: None }
    ));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(finding.phase, Phase::Link);
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference {
            kind: found,
            id,
            hint,
        }) => {
            assert_eq!(*found, kind);
            assert_eq!(id, BOLT);
            assert_eq!(*hint, None);
        }
        other => panic!("a reference detail, found {other:?}"),
    }
}

/// Bedrock-samples' item list has every template and every ingredient of the table.
#[test]
fn every_item_of_the_table_is_a_vanilla_item() {
    for pattern in TrimData::vanilla().patterns() {
        assert!(
            vanilla_items(&pattern.item_name).is_some(),
            "{}",
            pattern.item_name
        );
    }
    for material in TrimData::vanilla().materials() {
        assert!(
            vanilla_items(&material.item_name).is_some(),
            "{}",
            material.item_name
        );
    }
}

#[test]
fn a_complete_tagged_item_set_keeps_the_whole_table() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), vanilla_items);
    assert_eq!(filtered, *TrimData::vanilla());
    assert_eq!(pattern_rows(&filtered), vanilla_table().patterns);
    assert_eq!(material_rows(&filtered), vanilla_table().materials);
    assert!(filtered.diagnostics().is_empty());
}

#[test]
fn tr03_a_missing_template_item_leaves_its_pattern_out() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        (*id != item(BOLT)).then(|| vanilla_items(id)).flatten()
    });

    assert_eq!(filtered.patterns().len(), 17);
    assert_eq!(pattern_rows(&filtered), all_but_bolt());
    assert_eq!(filtered.pattern("bolt"), None);
    assert_eq!(filtered.pattern_for_item(&item(BOLT)), None);
    assert!(filtered.pattern("flow").is_some());
    // The materials are not filtered.
    assert_eq!(material_rows(&filtered), vanilla_table().materials);
    assert_ne!(filtered, *TrimData::vanilla());
}

#[test]
fn tr04_an_untagged_template_item_leaves_its_pattern_out() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        if *id == item(BOLT) {
            Some(tags(&["minecraft:is_tool"]))
        } else {
            vanilla_items(id)
        }
    });

    assert_eq!(filtered.patterns().len(), 17);
    assert_eq!(pattern_rows(&filtered), all_but_bolt());
    assert_eq!(filtered.pattern("bolt"), None);
    assert_eq!(filtered.pattern_for_item(&item(BOLT)), None);
    assert_eq!(material_rows(&filtered), vanilla_table().materials);
}

/// Deliberate deviation: vanilla only asserts in a debug build ("Initializing
/// trim pattern with item that did not exist").
#[test]
fn a_missing_template_item_is_reported_as_an_info_deviation() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        (*id != item(BOLT)).then(|| vanilla_items(id)).flatten()
    });
    assert_left_out(&filtered, "item");
}

/// Deliberate deviation: vanilla only asserts in a debug build ("Initializing
/// trim pattern with a template item that is not a trim template").
#[test]
fn an_untagged_template_item_is_reported_as_an_info_deviation() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        if *id == item(BOLT) {
            Some(tags(&[]))
        } else {
            vanilla_items(id)
        }
    });
    assert_left_out(&filtered, "trim template");
}

/// The game compares the tag's hashed string: the full name, byte for byte.
#[test]
fn the_template_tag_is_compared_as_written() {
    let stack = stack_without_items();
    for wrong in [
        "trim_templates",
        "minecraft:Trim_Templates",
        "minecraft:trim_templates ",
        "minecraft:trim_materials",
    ] {
        let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
            if *id == item(BOLT) {
                Some(tags(&[wrong]))
            } else {
                vanilla_items(id)
            }
        });
        assert_eq!(filtered.pattern("bolt"), None, "{wrong:?}");
        assert_eq!(filtered.patterns().len(), 17, "{wrong:?}");
    }
}

#[test]
fn the_template_tag_may_be_any_of_the_items_tags() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        if *id == item(BOLT) {
            Some(tags(&["minecraft:is_tool", TRIM_TEMPLATES]))
        } else {
            vanilla_items(id)
        }
    });
    assert_eq!(filtered, *TrimData::vanilla());
    assert!(filtered.diagnostics().is_empty());
}

#[test]
fn item_tags_are_compared_as_written() {
    let own = tags(&["minecraft:trim_templates", "demo:shiny"]);
    assert!(own.contains("minecraft:trim_templates"));
    assert!(own.contains("demo:shiny"));
    assert!(!own.contains("trim_templates"));
    assert!(!own.contains("demo:Shiny"));
    assert!(!ItemTags::default().contains("minecraft:trim_templates"));
}

/// The stack's definition of a template item is the item the filter reads, not the built-in.
#[test]
fn a_stack_item_without_the_tag_replaces_the_tagged_built_in() {
    let stack = stack_with_item(
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "minecraft:bolt_armor_trim_smithing_template"}, "components": {"minecraft:max_stack_size": 16}}}"#,
    );
    let filtered = TrimData::vanilla().filtered(stack.items(), vanilla_items);
    assert_eq!(pattern_rows(&filtered), all_but_bolt());
    assert_left_out(&filtered, "trim template");
}

#[test]
fn a_stack_item_with_other_tags_only_is_no_template() {
    let stack = stack_with_item(
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "minecraft:bolt_armor_trim_smithing_template"}, "components": {"minecraft:tags": {"tags": ["minecraft:is_tool"]}}}}"#,
    );
    let filtered = TrimData::vanilla().filtered(stack.items(), vanilla_items);
    assert_eq!(pattern_rows(&filtered), all_but_bolt());
    assert_left_out(&filtered, "trim template");
}

/// A stack item's tag is compared as the built-in's is: the full name, byte for byte.
#[test]
fn a_stack_items_tag_is_compared_as_written() {
    let stack = stack_with_item(
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "minecraft:bolt_armor_trim_smithing_template"}, "components": {"minecraft:tags": {"tags": ["demo:trim_templates", "trim_templates", "minecraft:Trim_Templates", "minecraft:trim_templates2"]}}}}"#,
    );
    let filtered = TrimData::vanilla().filtered(stack.items(), vanilla_items);
    assert_eq!(pattern_rows(&filtered), all_but_bolt());
    assert_left_out(&filtered, "trim template");
}

#[test]
fn a_stack_item_with_the_tag_is_a_template_without_a_built_in() {
    let stack = stack_with_item(
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "minecraft:bolt_armor_trim_smithing_template"}, "components": {"minecraft:tags": {"tags": ["minecraft:is_tool", "minecraft:trim_templates"]}}}}"#,
    );
    let filtered = TrimData::vanilla().filtered(stack.items(), |id| {
        (*id != item(BOLT)).then(|| vanilla_items(id)).flatten()
    });
    assert_eq!(filtered, *TrimData::vanilla());
    assert!(filtered.diagnostics().is_empty());
}

/// A pack item that is not one of the table's templates maps to no pattern, tagged or not:
/// the list is hard-coded.
#[test]
fn a_tagged_pack_item_outside_the_table_adds_no_pattern() {
    let stack = stack_with_item(
        br#"{"format_version": "1.26.30", "minecraft:item": {"description": {"identifier": "demo:spiral_template"}, "components": {"minecraft:tags": {"tags": ["minecraft:trim_templates"]}}}}"#,
    );
    let filtered = TrimData::vanilla().filtered(stack.items(), vanilla_items);
    assert_eq!(filtered, *TrimData::vanilla());
    assert_eq!(
        filtered.pattern_for_item(&item("demo:spiral_template")),
        None
    );
}

/// The materials are an unfiltered list: no item has to exist for them.
#[test]
fn without_any_item_no_pattern_stays_and_every_material_does() {
    let stack = stack_without_items();
    let filtered = TrimData::vanilla().filtered(stack.items(), |_| None);
    assert_eq!(filtered.patterns().len(), 0);
    for row in vanilla_table().patterns {
        assert_eq!(filtered.pattern(&row.pattern_id), None);
    }
    assert_eq!(material_rows(&filtered), vanilla_table().materials);
    assert!(filtered.material("resin").is_some());
    assert_eq!(filtered.diagnostics().len(), 18);
    assert!(
        filtered
            .diagnostics()
            .iter()
            .all(|finding| finding.severity == Severity::Info)
    );
}

/// The filter works on whatever registries it is given: it keeps what building them found and
/// fills both maps from the template items that pass, as the game's server initialisation does.
#[test]
fn the_filter_keeps_earlier_findings_and_fills_both_maps_from_the_items_that_pass() {
    let stack = stack_without_items();
    let data = TrimData::from_packet(
        &[
            pattern_entry("demo:a_template", "one"),
            pattern_entry("demo:a_template", "three"),
            pattern_entry("demo:b_template", "two"),
        ],
        &[material_entry("ruby", "§c", "demo:ruby")],
    );
    assert_eq!(data.diagnostics().len(), 1);
    assert!(data.pattern("one").is_some());

    let filtered = data.filtered(stack.items(), |id| {
        (*id == item("demo:a_template")).then(|| tags(&[TRIM_TEMPLATES]))
    });

    assert_eq!(filtered.patterns().len(), 1);
    assert_eq!(
        filtered.pattern("three").unwrap().item_name.as_str(),
        "demo:a_template"
    );
    assert_eq!(
        filtered.pattern("one"),
        None,
        "the identifier of the overwritten entry is not in a map filled from the items"
    );
    assert_eq!(filtered.pattern("two"), None);
    assert_eq!(filtered.materials(), data.materials());
    let findings: Vec<_> = filtered
        .diagnostics()
        .iter()
        .map(|finding| (finding.severity, finding.phase))
        .collect();
    assert_eq!(
        findings,
        [
            (Severity::Error, Phase::Parse),
            (Severity::Info, Phase::Link)
        ]
    );
}

fn world(major: u16, minor: u16, patch: u16) -> BaseGameVersion {
    BaseGameVersion::new(SemVersion::new(major, minor, patch))
}

fn assert_empty(data: &TrimData) {
    assert_eq!(data.patterns().len(), 0);
    assert!(data.materials().is_empty());
    assert_eq!(data.pattern("ward"), None);
    assert_eq!(
        data.pattern_for_item(&item("minecraft:ward_armor_trim_smithing_template")),
        None
    );
    assert_eq!(data.material("quartz"), None);
    assert_eq!(data.material_for_item(&item("minecraft:quartz")), None);
    assert!(data.diagnostics().is_empty());
}

fn assert_full(data: &TrimData) {
    assert_eq!(pattern_rows(data), vanilla_table().patterns);
    assert_eq!(material_rows(data), vanilla_table().materials);
    assert_eq!(*data, *TrimData::vanilla());
}

#[test]
fn tr05_a_world_below_1_20_has_empty_registries_and_a_1_20_world_the_table() {
    assert_empty(TrimData::for_world(&world(1, 19, 80)));
    assert_full(TrimData::for_world(&world(1, 20, 0)));
}

/// `TrailsAndTalesVersion.isCompatibleWith(baseGameVersion)`: both sides of 1.20.0.
#[test]
fn the_world_gate_opens_at_exactly_1_20_0() {
    for below in [
        world(1, 19, 99),
        world(1, 19, 0),
        world(1, 0, 0),
        world(0, 20, 0),
    ] {
        assert_empty(TrimData::for_world(&below));
    }
    for from in [
        world(1, 20, 0),
        world(1, 20, 1),
        world(1, 21, 0),
        world(1, 26, 20),
        world(2, 0, 0),
    ] {
        assert_full(TrimData::for_world(&from));
    }
}

/// A world of every version (`*`) or without a version passes the gate as it passes the
/// other base-game gates; a never-compatible one does not.
#[test]
fn a_world_of_any_or_no_version_has_the_table_and_an_incompatible_one_has_not() {
    assert_full(TrimData::for_world(&BaseGameVersion::ANY));
    assert_full(TrimData::for_world(&BaseGameVersion::EMPTY));
    assert_full(TrimData::for_world(&BaseGameVersion::default()));
    assert_empty(TrimData::for_world(&BaseGameVersion::INCOMPATIBLE));
}

/// The gate and the filter together, as a server starts a world.
#[test]
fn an_old_world_stays_empty_through_the_filter() {
    let stack = stack_without_items();
    let filtered = TrimData::for_world(&world(1, 19, 80)).filtered(stack.items(), vanilla_items);
    assert_empty(&filtered);
}
