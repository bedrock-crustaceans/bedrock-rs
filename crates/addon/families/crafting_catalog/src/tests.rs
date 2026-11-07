//! Unit tests of the crate-private parts: the link checkers the link pass calls (the placement
//! rules and the group-name lint).

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::SemVersion;
use indexmap::IndexMap;
use uuid::Uuid;

use super::link::{Definitions, link_group_names, link_placements};
use super::{
    CatalogItem, CatalogKey, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier, PATH,
};
use bedrock_addon_core::diagnostic::{Diagnostics, Kind, Phase, Severity};
use bedrock_addon_core::document::{Document, Merged, Origin, VersionInfo};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_shared_shapes::{CreativeCategory, MenuCategory};

/// Item definitions by identifier.
#[derive(Default)]
struct Defined(HashMap<ItemId, (MenuCategory, bool)>);

impl Defined {
    /// A definition whose author wrote `category` (when it is `Some`).
    fn with(self, id: &str, category: Option<CreativeCategory>, group: Option<&str>) -> Self {
        self.defined(id, category, group, category.is_some())
    }

    /// A definition of 1.21.110 or later without `category`: it holds the `none` an absent
    /// member means, unwritten (`ItemDescription::category_is_written` is `false`).
    fn without_category(self, id: &str, group: Option<&str>) -> Self {
        self.defined(id, Some(CreativeCategory::ItemCommandOnly), group, false)
    }

    fn defined(
        mut self,
        id: &str,
        category: Option<CreativeCategory>,
        group: Option<&str>,
        written: bool,
    ) -> Self {
        self.0.insert(
            ItemId::parse(id),
            (
                MenuCategory {
                    category,
                    group: group.map(str::to_owned),
                    ..MenuCategory::default()
                },
                written,
            ),
        );
        self
    }
}

impl Definitions for Defined {
    fn menu_category(&self, id: &ItemId) -> Option<(&MenuCategory, bool)> {
        self.0.get(id).map(|(menu, written)| (menu, *written))
    }
}

fn item(name: &str) -> CatalogItem {
    CatalogItem {
        name: ItemId::parse(name),
    }
}

fn group(name: Option<&str>, items: &[&str]) -> Group {
    Group {
        group_identifier: name.map(|name| GroupIdentifier {
            name: name.to_owned(),
            icon: None,
        }),
        items: items.iter().copied().map(item).collect(),
    }
}

/// A merged catalogue with `groups` in `category`, every entry placed by `pack1`.
fn merged(category: CategoryName, groups: Vec<Group>) -> Merged<CraftingCatalog> {
    let mut value = super::merge::empty();
    let origin = Origin::file(
        PackRef::new(
            0,
            PackIdVersion::new(
                Uuid::from_u128(1),
                SemVersion::new(1, 0, 0),
                PackType::Behavior,
            ),
            "pack1",
        ),
        PackSide::Behavior,
        PackPath::from_static(PATH),
    );
    let mut origins = IndexMap::new();
    for group in &groups {
        if let Some(identifier) = &group.group_identifier {
            origins.insert(
                CatalogKey::Group {
                    category,
                    name: identifier.name.clone(),
                },
                origin.clone(),
            );
        }
        for item in &group.items {
            origins.insert(
                CatalogKey::Item {
                    id: item.name.clone(),
                    aux: item.name.aux().unwrap_or(0),
                },
                origin.clone(),
            );
        }
    }
    let placed = Category {
        category_name: category,
        groups,
    };
    // pack1's file: an empty entry first, so that the paths are not all zeros.
    let file = CraftingCatalog {
        categories: vec![
            Category {
                category_name: CategoryName::Equipment,
                groups: Vec::new(),
            },
            placed.clone(),
        ],
    };
    let version = VersionInfo::new(
        Some(super::V1_21_60),
        false,
        super::V1_21_60,
        super::V1_21_60,
        false,
    );
    let layer = Document::new(file, origin, version, Diagnostics::default());
    value.categories[category as usize] = placed;
    Merged::new(value, origins, vec![layer], Diagnostics::default())
}

/// The JSON paths of the findings.
fn paths(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect()
}

/// The findings as `(code, severity, text)`, each checked to be a link finding at pack1's
/// catalogue file.
fn listed(found: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    found
        .iter()
        .map(|finding| {
            assert_eq!(finding.phase, Phase::Link);
            assert_eq!(
                finding.location.pack.as_ref().map(PackRef::name),
                Some("pack1")
            );
            assert_eq!(
                finding.location.path.as_ref().map(PackPath::as_str),
                Some(PATH)
            );
            (
                finding.code.id(),
                finding.severity,
                finding.code.to_string(),
            )
        })
        .collect()
}

const CATEGORY: &str = "The item custom:item1 was created with the category set to 'nature', and is now being set to 'construction'";
const GROUP: &str = "The item custom:item1 was created with the group set to 'custom:other_group', but is now being set to 'custom:my_name'";

/// The registry of `merge/items.json`: `custom:item1` is defined for `nature` and
/// `custom:other_group`.
fn item1() -> Defined {
    Defined::default().with(
        "custom:item1",
        Some(CreativeCategory::Nature),
        Some("custom:other_group"),
    )
}

/// Row `merge/m10_category_group_warnings`.
#[test]
fn m10_another_category_and_another_group_are_two_warnings_with_vanilla_s_texts() {
    let catalog = merged(
        CategoryName::Construction,
        vec![group(Some("custom:my_name"), &["custom:item1"])],
    );
    let found = link_placements(&catalog, &item1());
    assert_eq!(
        listed(&found),
        [
            (
                "crafting_catalog/category_mismatch",
                Severity::Warning,
                CATEGORY.to_owned()
            ),
            (
                "crafting_catalog/group_mismatch",
                Severity::Warning,
                GROUP.to_owned()
            ),
        ]
    );
    assert!(found.iter().all(|f| f.code.kind() == Kind::InvalidValue));
}

#[test]
fn m4_the_same_category_is_no_warning() {
    let catalog = merged(
        CategoryName::Nature,
        vec![group(Some("custom:my_name"), &["custom:item1"])],
    );
    assert_eq!(
        listed(&link_placements(&catalog, &item1())),
        [(
            "crafting_catalog/group_mismatch",
            Severity::Warning,
            GROUP.to_owned()
        )]
    );
}

#[test]
fn m5_the_same_group_a_loose_target_and_an_empty_group_are_no_warning() {
    let same = merged(
        CategoryName::Construction,
        vec![group(Some("custom:other_group"), &["custom:item1"])],
    );
    assert_eq!(
        listed(&link_placements(&same, &item1())),
        [(
            "crafting_catalog/category_mismatch",
            Severity::Warning,
            CATEGORY.to_owned()
        )]
    );
    let loose = merged(
        CategoryName::Construction,
        vec![group(None, &["custom:item1"])],
    );
    assert_eq!(
        listed(&link_placements(&loose, &item1())),
        [(
            "crafting_catalog/category_mismatch",
            Severity::Warning,
            CATEGORY.to_owned()
        )]
    );
    let named = merged(
        CategoryName::Nature,
        vec![group(Some("custom:my_name"), &["custom:item1"])],
    );
    for own_group in [None, Some("")] {
        let definitions =
            Defined::default().with("custom:item1", Some(CreativeCategory::Nature), own_group);
        assert_eq!(
            listed(&link_placements(&named, &definitions)),
            [],
            "{own_group:?}"
        );
    }
}

/// Row `merge/m11_minecraft_no_warning`: a catalogue string starting with `minecraft:` is
/// exempt from both rules.
#[test]
fn m11_a_minecraft_item_is_exempt() {
    let catalog = merged(
        CategoryName::Construction,
        vec![group(
            Some("custom:my_name"),
            &["minecraft:stone", "minecraft:arrow:6"],
        )],
    );
    let definitions = Defined::default()
        .with(
            "minecraft:stone",
            Some(CreativeCategory::Nature),
            Some("minecraft:g"),
        )
        .with(
            "minecraft:arrow",
            Some(CreativeCategory::Items),
            Some("minecraft:g"),
        );
    assert_eq!(listed(&link_placements(&catalog, &definitions)), []);
    // The prefix is the whole of `minecraft:`: `minecraftx:` and `Minecraft:` are not exempt.
    for name in ["minecraftx:stone", "Minecraft:stone", "minecraf:stone"] {
        let catalog = merged(CategoryName::Construction, vec![group(None, &[name])]);
        let definitions = Defined::default().with(name, Some(CreativeCategory::Nature), None);
        assert_eq!(
            listed(&link_placements(&catalog, &definitions)).len(),
            1,
            "{name}"
        );
    }
}

/// An item whose definition gave no `menu_category` (vanilla's `Undefined` category), and an
/// item no pack defines, are exempt.
#[test]
fn m4_m5_an_item_without_a_category_or_without_a_definition_is_exempt() {
    let catalog = merged(
        CategoryName::Construction,
        vec![group(
            Some("custom:my_name"),
            &["custom:item1", "custom:item2"],
        )],
    );
    for category in [Some(CreativeCategory::Undefined), None] {
        let definitions =
            Defined::default().with("custom:item1", category, Some("custom:other_group"));
        assert_eq!(
            listed(&link_placements(&catalog, &definitions)),
            [],
            "{category:?}"
        );
    }
    assert_eq!(listed(&link_placements(&catalog, &Defined::default())), []);
}

#[test]
fn m4_the_definition_is_found_whatever_the_aux_value_and_the_text_has_the_catalogue_string() {
    let catalog = merged(CategoryName::Items, vec![group(None, &["custom:item1:3"])]);
    let found = link_placements(&catalog, &item1());
    assert_eq!(
        listed(&found),
        [(
            "crafting_catalog/category_mismatch",
            Severity::Warning,
            "The item custom:item1:3 was created with the category set to 'nature', and is now being set to 'items'"
                .to_owned()
        )]
    );
}

#[test]
fn m4_every_category_is_named_as_item_definitions_spell_it() {
    for (placed, name) in [
        (CategoryName::Construction, "construction"),
        (CategoryName::Nature, "nature"),
        (CategoryName::Equipment, "equipment"),
        (CategoryName::Items, "items"),
    ] {
        let (own, own_name) = if placed == CategoryName::Nature {
            (CreativeCategory::Items, "items")
        } else {
            (CreativeCategory::Nature, "nature")
        };
        let catalog = merged(placed, vec![group(None, &["custom:item1"])]);
        let definitions = Defined::default().with("custom:item1", Some(own), None);
        let found = link_placements(&catalog, &definitions);
        assert_eq!(
            listed(&found),
            [(
                "crafting_catalog/category_mismatch",
                Severity::Warning,
                format!(
                    "The item custom:item1 was created with the category set to '{own_name}', and is now being set to '{name}'"
                )
            )]
        );
        assert_eq!(placed.creative_category().name(), Some(name));
    }
}

/// An item definition from 1.21.110 without `category` holds the `none` an absent member
/// means: exempt, like vanilla's `Undefined`. A `none` the author wrote is not exempt (see
/// the next test).
#[test]
fn m4_m5_a_definition_whose_category_is_unwritten_none_is_exempt() {
    let catalog = merged(
        CategoryName::Construction,
        vec![group(Some("custom:my_name"), &["custom:item1"])],
    );
    let definitions =
        Defined::default().without_category("custom:item1", Some("custom:other_group"));
    assert_eq!(listed(&link_placements(&catalog, &definitions)), []);
}

/// An author who wrote `"category": "none"` asked for no creative tab: a catalogue that places
/// the item is warned (M4), as for any other written category.
#[test]
fn m4_a_definition_whose_author_wrote_none_is_warned() {
    let catalog = merged(
        CategoryName::Construction,
        vec![group(None, &["custom:item1"])],
    );
    let definitions = Defined::default().with(
        "custom:item1",
        Some(CreativeCategory::ItemCommandOnly),
        None,
    );
    assert_eq!(
        listed(&link_placements(&catalog, &definitions)),
        [(
            "crafting_catalog/category_mismatch",
            Severity::Warning,
            "The item custom:item1 was created with the category set to 'none', and is now being set to 'construction'"
                .to_owned()
        )]
    );
}

/// A link finding is at the placement in the file that made it.
#[test]
fn link_findings_carry_the_path_of_the_placement_in_its_file() {
    let catalog = merged(
        CategoryName::Construction,
        vec![
            group(Some("custom:first"), &["custom:item2"]),
            group(Some("custom:my_name"), &["custom:item3", "custom:item1:4"]),
        ],
    );
    let found = link_placements(&catalog, &item1());
    assert_eq!(
        paths(&found),
        [
            "minecraft:crafting_items_catalog/categories/1/groups/1/items/1",
            "minecraft:crafting_items_catalog/categories/1/groups/1/items/1",
        ]
    );
    let lints = link_group_names(&catalog, &|key| key == "custom:first");
    assert_eq!(
        paths(&lints),
        ["minecraft:crafting_items_catalog/categories/1/groups/1/group_identifier/name"]
    );
}

/// The pack name and JSON path of each finding.
fn located(found: &Diagnostics) -> Vec<(String, String)> {
    found
        .iter()
        .map(|finding| {
            let pack = finding.location.pack.as_ref();
            (
                pack.map(|pack| pack.name().to_owned()).unwrap_or_default(),
                finding.location.json_path.to_string(),
            )
        })
        .collect()
}

/// With several layers the path is read from the layer of the entry's origin, not from the
/// first layer: a higher pack's file that names the same group and item elsewhere (its
/// placements lost to the lower pack's) does not give the path.
#[test]
fn link_findings_are_read_from_the_layer_that_placed_the_entry() {
    let origin_of = |index: usize, name: &str| {
        Origin::file(
            PackRef::new(
                index,
                PackIdVersion::new(
                    Uuid::from_u128(index as u128 + 1),
                    SemVersion::new(1, 0, 0),
                    PackType::Behavior,
                ),
                name,
            ),
            PackSide::Behavior,
            PackPath::from_static(PATH),
        )
    };
    let file = |groups: Vec<Group>| CraftingCatalog {
        categories: vec![Category {
            category_name: CategoryName::Construction,
            groups,
        }],
    };
    let version = || {
        VersionInfo::new(
            Some(super::V1_21_60),
            false,
            super::V1_21_60,
            super::V1_21_60,
            false,
        )
    };
    // The lower pack placed the group and the item: group 0, item 0.
    let lower = origin_of(1, "lower");
    let lower_file = file(vec![group(Some("custom:my_name"), &["custom:item1"])]);
    // The top pack names them again, at group 2 and item 1.
    let top = origin_of(0, "top");
    let top_file = file(vec![
        group(None, &[]),
        group(Some("custom:other"), &[]),
        group(Some("custom:my_name"), &["custom:item9", "custom:item1"]),
    ]);
    let mut value = super::merge::empty();
    value.categories[0].groups = vec![
        group(Some("custom:my_name"), &["custom:item1", "custom:item9"]),
        group(Some("custom:other"), &[]),
    ];
    let mut origins = IndexMap::new();
    let group_key = |name: &str| CatalogKey::Group {
        category: CategoryName::Construction,
        name: name.to_owned(),
    };
    let item_key = |name: &str| CatalogKey::Item {
        id: ItemId::parse(name),
        aux: 0,
    };
    origins.insert(group_key("custom:my_name"), lower.clone());
    origins.insert(item_key("custom:item1"), lower.clone());
    origins.insert(group_key("custom:other"), top.clone());
    origins.insert(item_key("custom:item9"), top.clone());
    // Layers are top first.
    let layers = vec![
        Document::new(top_file, top, version(), Diagnostics::default()),
        Document::new(lower_file, lower, version(), Diagnostics::default()),
    ];
    let catalog = Merged::new(value, origins, layers, Diagnostics::default());
    let item_at = "minecraft:crafting_items_catalog/categories/0/groups/0/items/0";
    assert_eq!(
        located(&link_placements(&catalog, &item1())),
        [
            ("lower".to_owned(), item_at.to_owned()),
            ("lower".to_owned(), item_at.to_owned()),
        ]
    );
    assert_eq!(
        located(&link_group_names(&catalog, &|_| false)),
        [
            (
                "lower".to_owned(),
                "minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/name"
                    .to_owned()
            ),
            (
                "top".to_owned(),
                "minecraft:crafting_items_catalog/categories/0/groups/1/group_identifier/name"
                    .to_owned()
            ),
        ]
    );
}

/// The group-name lint (20 M2).
#[test]
fn a_new_group_name_without_a_lang_entry_is_an_info() {
    let catalog = merged(
        CategoryName::Items,
        vec![
            group(Some("custom:known"), &[]),
            group(Some("custom:unknown"), &[]),
            group(Some("minecraft:itemGroup.name.unknown"), &[]),
            group(None, &[]),
        ],
    );
    let found = link_group_names(&catalog, &|key| key == "custom:known");
    assert_eq!(found.iter().count(), 1, "{found:#?}");
    let finding = found.iter().next().unwrap();
    assert!(finding.is(&core_codes::UNRESOLVED_LANG_KEY));
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(finding.phase, Phase::Link);
    assert_eq!(
        finding.location.pack.as_ref().map(PackRef::name),
        Some("pack1")
    );
    assert!(matches!(
        finding.detail.as_deref(),
        Some(bedrock_addon_core::diagnostic::DiagnosticDetail::Reference { kind: "lang key", id, .. }) if id == "custom:unknown"
    ));
    // Every key defined: nothing.
    assert!(link_group_names(&catalog, &|_| true).is_empty());
}

/// The chain is two structs, the floor's and the newest, read under the unstrict and the
/// strict mask, from a whole document.
#[test]
fn the_chain_is_two_structs_from_the_floor() {
    use bedrock_addon_core::versioning::{
        Family, RangeModel, VersionChain, Versioned, check_chain,
    };
    type Chain = <crate::CatalogFamily as Family>::History;
    assert!(check_chain::<crate::CatalogFamily>().is_ok());
    assert_eq!(
        <Chain as VersionChain<crate::CatalogFamily>>::VERSIONS,
        [super::V1_21_60, super::V1_26_50]
    );
    let has_beta = <Chain as VersionChain<crate::CatalogFamily>>::HAS_BETA;
    assert!(!has_beta);
    assert_eq!(<CraftingCatalog as Versioned>::VERSION, super::V1_26_50);
    {
        use bedrock_addon_core::json::NullRule;
        use bedrock_addon_core::load::DocumentSource;
        use bedrock_addon_core::versioning::{InputKind, StructIndex, UnknownMask};
        type F = crate::CatalogFamily;
        assert_eq!(F::ENVELOPE.input, InputKind::Document);
        assert_eq!(
            F::reading(StructIndex::Stable(0), DocumentSource::Json).unknown,
            UnknownMask::Warn
        );
        assert_eq!(
            F::reading(StructIndex::Stable(1), DocumentSource::Json).unknown,
            UnknownMask::Error
        );
        assert_eq!(
            F::reading(StructIndex::Stable(1), DocumentSource::Json).null_rule(),
            Some(NullRule::Rejected)
        );
    }
    assert_eq!(crate::CatalogFamily::FLOOR, super::V1_21_60);
    assert_eq!(
        crate::CatalogFamily::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
    assert_eq!(
        crate::CatalogFamily::PAYLOAD_KEY,
        "minecraft:crafting_items_catalog"
    );
    assert_eq!(super::LOADER.walk.directories, ["item_catalog"]);
}
