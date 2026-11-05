//! Building a catalogue by hand  and the inherent helpers, `Placed` and the
//! content constants.

use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::pack_identity::PackSide;
use bedrock_addon_family_crafting_catalog::{
    CatalogItem, CatalogKey, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier,
};
use bedrock_addon_shared_shapes::CreativeCategory;

use super::common::*;

#[test]
fn a_catalogue_built_with_the_constructors_equals_the_parsed_one() {
    let built = CraftingCatalog {
        categories: vec![
            Category {
                groups: vec![Group {
                    group_identifier: Some(GroupIdentifier {
                        icon: Some(CatalogItem::new(ItemId::from_static("custom:item1"))),
                        ..GroupIdentifier::new("custom:building")
                    }),
                    items: vec![
                        ItemId::from_static("custom:item1").into(),
                        CatalogItem::new(ItemId::from_static("custom:item2")),
                    ],
                }],
                ..Category::new(CategoryName::Construction)
            },
            Category {
                groups: vec![Group {
                    group_identifier: Some(GroupIdentifier {
                        icon: Some(ItemId::from_static("custom:my_icon:1").into()),
                        ..GroupIdentifier::new("custom:tools")
                    }),
                    items: vec![ItemId::from_static("custom:item3").into()],
                }],
                ..Category::new(CategoryName::Equipment)
            },
            Category {
                groups: vec![Group {
                    items: vec![
                        ItemId::from_static("custom:item4").into(),
                        ItemId::from_static("minecraft:stone").into(),
                    ],
                    ..Group::loose()
                }],
                ..Category::new(CategoryName::Items)
            },
            Category {
                groups: vec![Group::named("custom:plants")],
                ..Category::new(CategoryName::Nature)
            },
        ],
    };
    assert_eq!(built, cat01());
    // What is built is what loads.
    let written = built.to_json().unwrap();
    assert_eq!(loads(&written).value(), &built);
}

#[test]
fn the_empty_values() {
    assert_eq!(CraftingCatalog::new(), CraftingCatalog::default());
    assert!(CraftingCatalog::new().categories.is_empty());
    assert_eq!(Group::loose(), Group::default());
    assert!(Group::loose().group_identifier.is_none());
    assert!(Group::loose().items.is_empty());
    let category = Category::new(CategoryName::Nature);
    assert_eq!(category.category_name, CategoryName::Nature);
    assert!(category.groups.is_empty());
    let identifier = GroupIdentifier::new("custom:g");
    assert_eq!(identifier.name, "custom:g");
    assert!(identifier.icon.is_none());
    let named = Group::named("custom:g");
    assert_eq!(named.group_identifier, Some(identifier));
    assert!(named.items.is_empty());
}

#[test]
fn group_names_and_lookups() {
    let catalog = catalog(vec![
        category(
            CategoryName::Items,
            vec![named("custom:a", None, &["x:1"]), loose(&["x:2"])],
        ),
        category(
            CategoryName::Nature,
            vec![named("custom:a", None, &["x:3"])],
        ),
        category(
            CategoryName::Items,
            vec![
                named("custom:b", None, &[]),
                named("custom:a", None, &["x:4"]),
            ],
        ),
    ]);
    assert_eq!(Group::named("custom:a").name(), Some("custom:a"));
    assert_eq!(Group::loose().name(), None);
    let names: Vec<Option<&str>> = catalog
        .groups(CategoryName::Items)
        .map(Group::name)
        .collect();
    assert_eq!(
        names,
        [Some("custom:a"), None, Some("custom:b"), Some("custom:a")]
    );
    assert_eq!(catalog.groups(CategoryName::Nature).count(), 1);
    assert_eq!(catalog.groups(CategoryName::Equipment).count(), 0);
    // The first of that name in that category.
    let first = catalog.group(CategoryName::Items, "custom:a").unwrap();
    assert_eq!(first.items, [item("x:1")]);
    assert_eq!(
        catalog
            .group(CategoryName::Nature, "custom:a")
            .unwrap()
            .items,
        [item("x:3")]
    );
    assert!(catalog.group(CategoryName::Nature, "custom:b").is_none());
    assert!(catalog.group(CategoryName::Items, "custom:A").is_none());
}

#[test]
fn items_compare_by_the_name_as_written() {
    assert_eq!(item("custom:a"), item("custom:a"));
    // `ItemId` alone ignores the aux value; a catalogue item does not.
    assert_eq!(ItemId::parse("custom:a:1"), ItemId::parse("custom:a:2"));
    assert_ne!(item("custom:a:1"), item("custom:a:2"));
    assert_ne!(item("custom:a"), item("custom:a:0"));
    assert_ne!(item("custom:a"), item("custom:b"));
}

#[test]
fn category_names() {
    let names: Vec<&str> = CategoryName::ALL.iter().map(|c| c.name()).collect();
    assert_eq!(names, ["construction", "nature", "equipment", "items"]);
    for (value, category) in CategoryName::ALL.into_iter().enumerate() {
        assert_eq!(category as usize, value, "vanilla's enum value");
        assert_eq!(CategoryName::from_name(category.name()), Some(category));
        assert_eq!(category.to_string(), category.name());
    }
    assert_eq!(CategoryName::from_name("Construction"), None);
    assert_eq!(CategoryName::from_name("none"), None);
    assert!(CategoryName::Construction < CategoryName::Nature);
    assert!(CategoryName::Equipment < CategoryName::Items);
    // The creative category of an item definition: vanilla's value + 1.
    assert_eq!(
        CategoryName::ALL.map(CategoryName::creative_category),
        [
            CreativeCategory::Construction,
            CreativeCategory::Nature,
            CreativeCategory::Equipment,
            CreativeCategory::Items,
        ]
    );
    for category in CategoryName::ALL {
        assert_eq!(category.creative_category() as u8, category as u8 + 1);
    }
}

#[test]
fn the_content_constants_and_the_one_path() {
    assert_eq!(<CraftingCatalog as Content>::NAME, "crafting item catalog");
    assert_eq!(<CraftingCatalog as Content>::SIDES, [PackSide::Behavior]);
    assert_eq!(
        CraftingCatalog::new().default_path().as_str(),
        "item_catalog/crafting_item_catalog.json"
    );
    assert_eq!(cat01().default_path().as_str(), PATH);
}

#[test]
fn catalog_keys_compare_the_identifier_without_its_aux_text() {
    let key = |name: &str, aux| CatalogKey::Item {
        id: ItemId::parse(name),
        aux,
    };
    assert_eq!(key("custom:a:3", 3), key("custom:a", 3));
    assert_ne!(key("custom:a", 3), key("custom:a", 0));
    assert_ne!(key("custom:a", 0), key("custom:b", 0));
    let group = |category, name: &str| CatalogKey::Group {
        category,
        name: name.to_owned(),
    };
    assert_ne!(
        group(CategoryName::Items, "custom:g"),
        group(CategoryName::Nature, "custom:g")
    );
    assert_eq!(
        group(CategoryName::Items, "custom:g"),
        group(CategoryName::Items, "custom:g")
    );
}
