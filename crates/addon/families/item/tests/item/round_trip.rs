//! The round-trip level for items: Value (parse, write, parse equal) at the
//! newest version, Upgraded for older documents, and writing as written through
//! `history::ItemHistory::to_json`; plus construction, `default_path` and the network NBT.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::JsonDocument;
use bedrock_addon_core::document::{ComponentRef, Content, Placed};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_item::component::{Glint, MaxStackSize};
use bedrock_addon_family_item::{Item, ItemFormat};

use super::common::*;

/// The newest-format document every round trip starts from: shorthands and a custom component.
/// (An unknown member or `events` rejects a modern document:
/// `an_older_documents_unknown_members_are_written_at_their_level`.)
const RICH: &str = r#"{
  "format_version": "1.26.30",
  "minecraft:item": {
    "description": {"identifier": "demo:rich", "menu_category": {"category": "equipment", "group": "demo:tools"}},
    "components": {
      "minecraft:max_stack_size": 16,
      "minecraft:glint": true,
      "minecraft:icon": "rich",
      "minecraft:display_name": {"value": "item.rich.name"},
      "minecraft:interact_button": "Use me",
      "minecraft:damage": {"value": 7},
      "minecraft:durability": {"max_durability": 100, "damage_chance": {"min": 10, "max": 50}},
      "minecraft:repairable": {"repair_items": [{"items": ["minecraft:stick", {"tags": "query.any_tag('planks')"}], "repair_amount": "context.other->query.max_durability * 0.25"}]},
      "minecraft:digger": {"use_efficiency": true, "destroy_speeds": [{"block": {"tags": "query.any_tag('stone')"}, "speed": 4}]},
      "minecraft:use_modifiers": {"use_duration": 1.25, "movement_modifier": 0.5, "start_using": "always"},
      "minecraft:wearable": {"slot": "slot.armor.chest", "protection": 4, "hides_player_location": true},
      "minecraft:cooldown": {"category": "rich", "duration": 0.5, "type": "attack"},
      "minecraft:kinetic_weapon": {"reach": {"min": 0.0, "max": 4.5}, "damage_conditions": {"max_duration": 20, "min_speed": 2.0}},
      "minecraft:use_animation": "spear",
      "minecraft:tags": {"tags": ["demo:shiny"]},
      "demo:spin": {"speed": 2}
    }
  }
}"#;

/// The document loaded: its value without the legacy reading (derived from the JSON).
fn value(json: &str) -> Item {
    let mut item = loads(json, &third_party()).into_value();
    item.legacy.shared.clear();
    item
}

#[test]
fn a_newest_format_document_round_trips_at_the_value_level() {
    let first = value(RICH);
    let written = first.to_json().unwrap();
    let second = value(&written);
    assert_eq!(first, second, "{written}");
    // shorthands are written in the object form
    assert!(written.contains("\"minecraft:glint\": {"), "{written}");
}

/// An older document's unknown members and `events` (warnings in its own slice) are held at
/// their level; the strict 1.26.50 format refuses each of them, so the newest writer
/// refuses the item, and the as-written writer writes them all.
#[test]
fn an_older_documents_unknown_members_are_written_at_their_level() {
    let older = r#"{"format_version": "1.21.90", "minecraft:item": {
        "description": {"identifier": "demo:old", "mystery": 1},
        "components": {"minecraft:tags": {"tags": ["demo:shiny"], "note": "kept"}, "minecraft:icon": "x"},
        "events": {"demo:e": {"run_command": {"command": ["say hi"]}}}}}"#;
    let first = loads(older, &third_party());
    let held = newest_payload(&first);
    for member in ["\"mystery\"", "\"note\"", "\"events\""] {
        assert!(held.contains(member), "{member}: {held}");
    }
    // the newest writer refuses the first of them; the as-written writer keeps them all
    assert!(refused_at(
        &first.to_json(),
        "minecraft:item/description/mystery"
    ));
    let as_read = written_as_read(older, &third_party());
    for member in ["\"mystery\"", "\"note\"", "\"events\""] {
        assert!(as_read.contains(member), "{member}: {as_read}");
    }
}

/// The legacy reading of a newest-format document is part of its value, and the written
/// text gives the same one: a component the legacy reader also reads is held in the form the
/// writer writes it in (`{"value": 16}` for `16`), whatever form the file had.
#[test]
fn a_newest_format_document_reads_back_with_the_legacy_reading_it_had() {
    let first = loads(RICH, &third_party()).into_value();
    let written = first.to_json().unwrap();
    let second = loads(&written, &third_party()).into_value();
    assert_eq!(first.legacy.shared, second.legacy.shared, "{written}");
    assert_eq!(first, second, "{written}");
    // The form is the writer's, for the short and for the long form of the file alike.
    let stack = |form: &str| {
        let text = item(
            "1.26.30",
            "demo:x",
            &format!(r#""minecraft:max_stack_size": {form}, "minecraft:icon": "x""#),
        );
        loads(&text, &third_party()).into_value()
    };
    let (short, long) = (stack("16"), stack(r#"{"value": 16}"#));
    assert_eq!(short, long);
    assert_eq!(
        short.legacy.shared.get("minecraft:max_stack_size"),
        Some(&bedrock_addon_core::json::from_str(r#"{"value": 16}"#).unwrap())
    );
    // What the legacy reader computes from it is what it was.
    let timed = loads(
        &item(
            "1.26.30",
            "demo:x",
            r#""minecraft:use_duration": 1.6, "minecraft:icon": "x""#,
        ),
        &third_party(),
    )
    .into_value();
    assert_eq!(timed.legacy.use_duration_ticks(), Some(32));
    let again = loads(&timed.to_json().unwrap(), &third_party()).into_value();
    assert_eq!(again, timed);
}

#[test]
fn an_older_document_is_written_upgraded() {
    let old = item(
        "1.20.40",
        "demo:old",
        r#""minecraft:use_duration": 1.6, "minecraft:icon": {"texture": "old"}, "tag:demo:t": {}, "minecraft:max_stack_size": 1"#,
    );
    let first = value(&old);
    let written = first.to_json().unwrap();
    assert!(
        written.contains("\"format_version\": \"1.26.50\""),
        "{written}"
    );
    assert!(written.contains("\"textures\""), "{written}");
    let mut second = value(&written);
    // the side-car and the legacy reading are not read back at 1.26.50
    second.deprecated = first.deprecated.clone();
    second.legacy = first.legacy.clone();
    assert_eq!(first, second);
}

/// An item upgraded from the 1.21.60 custom component list is written at
/// 1.26.30 with keyed custom components only; read back, the output gives no finding (no
/// `__deprecated_components`, which 1.26.30 does not bind) and the same components.
#[test]
fn an_upgraded_custom_component_list_reads_back_without_findings() {
    let old = item(
        "1.21.60",
        "demo:x",
        r#""minecraft:max_stack_size": 2, "minecraft:custom_components": [{"name": "demo:shake", "data": {"power": 2}}, "demo:roll"]"#,
    );
    let first = value(&old);
    let written = first.to_json().unwrap();
    assert!(!written.contains("__deprecated_components"), "{written}");
    let reread = loads(&written, &third_party());
    assert!(
        above_info(reread.diagnostics()).is_empty(),
        "{:#?}",
        reread.diagnostics()
    );
    let names = |item: &Item| {
        item.components
            .iter()
            .filter_map(|e| match e {
                ComponentRef::Custom(c) => Some((c.name.clone(), c.parameters.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&first), names(reread.value()), "{written}");
}

#[test]
fn a_legacy_format_item_is_written_in_its_own_format() {
    let json = item(
        "1.10",
        "minecraft:camera",
        r#""minecraft:use_duration": 100000, "minecraft:block": "minecraft:camera", "minecraft:is_meat": false"#,
    );
    let first = loads(&json, &allowlisted()).into_value();
    let written = first.to_json().unwrap();
    assert!(written.contains("\"1.10.0\""), "{written}");
    let second = loads(&written, &allowlisted()).into_value();
    assert_eq!(second.legacy.format, ItemFormat::Legacy);
    assert_eq!(first.legacy.format_item, second.legacy.format_item);
}

#[test]
fn a_document_is_written_as_written_through_history() {
    let old = item(
        "1.20.40",
        "demo:old",
        r#""minecraft:use_duration": 1.6, "minecraft:max_stack_size": 1"#,
    );
    let document = Item::parse_as_written(
        &PackPath::from_static("items/x.json"),
        old.as_bytes(),
        &third_party(),
    )
    .unwrap();
    assert_eq!(document.value().parsed_version().to_string(), "1.20.40");
    let written = document.to_json().unwrap();
    assert!(written.contains("\"1.20.40\""), "{written}");
    assert!(written.contains("minecraft:use_duration"), "{written}");
    let upgraded =
        bedrock_addon_family_item::history::ItemHistory::upgrade(document, &third_party()).unwrap();
    assert_eq!(*upgraded.value(), *loads(&old, &third_party()).value());
}

#[test]
fn a_new_item_is_valid_and_placed_by_its_identifier() {
    let mut item = Item::new(ItemId::new("demo:ruby").unwrap());
    item.components.insert(MaxStackSize::new(16));
    item.components.insert(Glint::new(true));
    assert!(
        item.validate(&third_party()).is_empty(),
        "{:#?}",
        item.validate(&third_party())
    );
    assert_eq!(item.default_path().as_str(), "items/demo/ruby.json");
    let back = value(&item.to_json().unwrap());
    assert_eq!(back.components, item.components);
    assert_eq!(back.description, item.description);
}

#[test]
fn validate_reports_the_gates_of_a_constructed_item() {
    // the newest format is past every release: a third-party pack may use any component, but
    // the validators still run
    let mut item = Item::new(ItemId::new("demo:pouch").unwrap());
    item.components
        .insert(bedrock_addon_family_item::component::StorageItem::default());
    let d = item.validate(&third_party());
    assert!(d.has_errors(), "{d:#?}");
}

#[cfg(feature = "nbt")]
#[test]
fn network_properties_carry_the_components_and_item_properties() {
    let rich = loads(RICH, &third_party());
    let nbtx::Value::Compound(root) = rich.network_properties() else {
        panic!("not a compound");
    };
    let Some(nbtx::Value::Compound(components)) = root.get(&b"components"[..]) else {
        panic!("no components");
    };
    let Some(nbtx::Value::Compound(properties)) = components.get(&b"item_properties"[..]) else {
        panic!("no item_properties");
    };
    assert!(properties.get(&b"menu_category"[..]).is_some());
    assert!(properties.get(&b"minecraft:icon"[..]).is_some());
    // moved, not duplicated
    assert!(components.get(&b"minecraft:icon"[..]).is_none());
    assert_eq!(
        properties.get(&b"max_stack_size"[..]),
        Some(&nbtx::Value::Int(16))
    );
    // integers stay Int, reals Float
    let Some(nbtx::Value::Compound(damage)) = components.get(&b"minecraft:damage"[..]) else {
        panic!("no damage");
    };
    assert_eq!(damage.get(&b"value"[..]), Some(&nbtx::Value::Int(7)));
    let Some(nbtx::Value::Compound(cooldown)) = components.get(&b"minecraft:cooldown"[..]) else {
        panic!("no cooldown");
    };
    assert_eq!(
        cooldown.get(&b"duration"[..]),
        Some(&nbtx::Value::Float(0.5))
    );
    // custom components are not sent
    assert!(components.get(&b"demo:spin"[..]).is_none());
    // a legacy-format item is sent as a non-component-based entry
    let legacy = loads(
        &item("1.10", "minecraft:camera", r#""minecraft:use_duration": 1"#),
        &allowlisted(),
    );
    assert_eq!(
        legacy.network_properties(),
        nbtx::Value::Compound(nbtx::Compound::new())
    );
}

/// `Item::parse_as_written` of `bytes`.
fn as_written(
    bytes: &[u8],
) -> Result<
    bedrock_addon_core::document::Document<bedrock_addon_family_item::history::ItemHistory>,
    bedrock_addon_core::document::Rejected,
> {
    Item::parse_as_written(
        &PackPath::from_static("items/x.json"),
        bytes,
        &third_party(),
    )
}

#[test]
fn parse_as_written_strips_a_bom_and_refuses_what_parse_refuses() {
    let json = item("1.21.0", "demo:x", r#""minecraft:max_stack_size": 2"#);
    let mut bom = b"\xEF\xBB\xBF".to_vec();
    bom.extend_from_slice(json.as_bytes());
    assert!(as_written(&bom).is_ok());
    // not UTF-8: MalformedInput, as Item::parse
    let mut broken = json.clone().into_bytes();
    broken.insert(10, 0xFF);
    for result in [
        as_written(&broken).map(|_| ()),
        Item::parse(
            &PackPath::from_static("items/x.json"),
            &broken,
            &third_party(),
        )
        .map(|_| ()),
    ] {
        let d = result.unwrap_err().diagnostics;
        assert!(has(&d, |c| c.id() == "core/syntax"), "{d:#?}");
    }
    // nested past the depth limit: MalformedInput before any parse
    let deep = format!(
        r#"{{"format_version": "1.21.0", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 2}}, "events": {}{}}}}}"#,
        "[".repeat(200),
        "]".repeat(200)
    );
    let d = as_written(deep.as_bytes()).unwrap_err().diagnostics;
    assert!(has(&d, |c| c.id() == "core/syntax"), "{d:#?}");
}

// ---- `menu_category`: what the author wrote ---------------------------------

/// A newest-format item with `description_members` after the identifier (a JSON fragment
/// starting with `, `, or empty).
fn described(version: &str, description_members: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"{description_members}}}, "components": {{"minecraft:hand_equipped": true}}}}}}"#
    )
}

/// The `description` object the writer gives for `json`, as compact JSON text.
fn written_description(json: &str) -> String {
    let written = loads(json, &third_party()).to_json().unwrap();
    let value: bedrock_addon_core::json::Value =
        bedrock_addon_core::json::from_str(&written).unwrap();
    let description = &value.as_object().unwrap()["minecraft:item"]
        .as_object()
        .unwrap()["description"];
    compact(description)
}

/// From 1.21.110 an absent `category` is `none`, the format's own default: the writer does not
/// write a `menu_category` or a `category` the author left out, and keeps one the author wrote,
/// `none` included.
#[test]
fn a_menu_category_the_author_did_not_write_is_not_written_back() {
    for version in ["1.21.110", "1.26.30"] {
        assert_eq!(
            written_description(&described(version, "")),
            r#"{"identifier":"demo:x"}"#,
            "{version}: no menu_category"
        );
        assert_eq!(
            written_description(&described(version, r#", "menu_category": {}"#)),
            r#"{"identifier":"demo:x","menu_category":{}}"#,
            "{version}: an empty menu_category"
        );
        assert_eq!(
            written_description(&described(
                version,
                r#", "menu_category": {"group": "demo:tools"}"#
            )),
            r#"{"identifier":"demo:x","menu_category":{"group":"demo:tools"}}"#,
            "{version}: a group alone"
        );
        assert_eq!(
            written_description(&described(
                version,
                r#", "menu_category": {"category": "none"}"#
            )),
            r#"{"identifier":"demo:x","menu_category":{"category":"none"}}"#,
            "{version}: a written none"
        );
        assert_eq!(
            written_description(&described(
                version,
                r#", "menu_category": {"category": "items"}"#
            )),
            r#"{"identifier":"demo:x","menu_category":{"category":"items"}}"#,
            "{version}: a written category"
        );
    }
}

/// The description of `json` as loaded.
fn description_of(json: &str) -> bedrock_addon_family_item::ItemDescription {
    loads(json, &third_party()).into_value().description
}

/// `ItemDescription::menu_category` holds `none` for an absent and for a written `none`; the
/// presence record and `category_is_written` tell them apart.
#[test]
fn the_presence_record_tells_a_written_none_from_an_absent_category() {
    use bedrock_addon_family_item::MenuCategoryPresence as Presence;
    use bedrock_addon_shared_shapes::CreativeCategory;

    for (members, presence, category, written) in [
        (
            "",
            Presence::Absent,
            CreativeCategory::ItemCommandOnly,
            false,
        ),
        (
            r#", "menu_category": {}"#,
            Presence::WithoutCategory,
            CreativeCategory::ItemCommandOnly,
            false,
        ),
        (
            r#", "menu_category": {"group": "demo:tools"}"#,
            Presence::WithoutCategory,
            CreativeCategory::ItemCommandOnly,
            false,
        ),
        (
            r#", "menu_category": {"category": "none"}"#,
            Presence::Category,
            CreativeCategory::ItemCommandOnly,
            true,
        ),
        (
            r#", "menu_category": {"category": "items"}"#,
            Presence::Category,
            CreativeCategory::Items,
            true,
        ),
    ] {
        for version in ["1.21.110", "1.26.30"] {
            let json = described(version, members);
            let description = description_of(&json);
            assert_eq!(description.menu_category_presence, presence, "{json}");
            assert_eq!(description.menu_category.category, Some(category), "{json}");
            assert_eq!(description.category_is_written(), written, "{json}");
            // The written text reads back to the same description, presence included.
            let written_text = loads(&json, &third_party()).to_json().unwrap();
            assert_eq!(description_of(&written_text), description, "{written_text}");
        }
    }
}

/// A document below 1.21.110 is upgraded with the category its own format gives an absent
/// one (`Undefined` from 1.21.60, `items` below): the newest format would read the absence as
/// `none`, so that category is stated and written, whatever the document wrote. `Undefined`
/// has no spelling of its own at 1.21.110 and later: it is written as `none` and reads back
/// as a written `none`.
#[test]
fn an_upgraded_documents_category_is_stated_and_written() {
    use bedrock_addon_family_item::MenuCategoryPresence as Presence;
    use bedrock_addon_shared_shapes::CreativeCategory;

    for (version, category, reads_back) in [
        (
            "1.21.90",
            CreativeCategory::Undefined,
            CreativeCategory::ItemCommandOnly,
        ),
        ("1.21.50", CreativeCategory::Items, CreativeCategory::Items),
    ] {
        let json = described(version, "");
        let description = description_of(&json);
        assert_eq!(
            description.menu_category.category,
            Some(category),
            "{version}"
        );
        assert!(description.category_is_written(), "{version}");
        let written_text = loads(&json, &third_party()).to_json().unwrap();
        let again = description_of(&written_text);
        assert_eq!(
            again.menu_category.category,
            Some(reads_back),
            "{written_text}"
        );
        assert_eq!(again.menu_category_presence, Presence::Category);
        assert!(again.category_is_written());
    }
}

/// A category set on a loaded or new item is written although the document had none; a new
/// item, which is shown to commands only, writes no `menu_category` until one is set.
#[test]
fn a_category_set_by_the_caller_is_written() {
    use bedrock_addon_family_item::MenuCategoryPresence as Presence;
    use bedrock_addon_shared_shapes::CreativeCategory;

    let description = |item: &Item| {
        let value: bedrock_addon_core::json::Value =
            bedrock_addon_core::json::from_str(&item.to_json().unwrap()).unwrap();
        compact(
            &value.as_object().unwrap()["minecraft:item"]
                .as_object()
                .unwrap()["description"],
        )
    };
    let mut item = Item::new(ItemId::parse("demo:x"));
    // The test is of the category of a component-based item: without a component the item
    // is the legacy item format, which has no `menu_category` (it is refused there).
    item.description.menu_category.category = Some(CreativeCategory::Equipment);
    assert!(
        format!("{:?}", item.to_json()).contains(r#"Key("menu_category")"#),
        "{:?}",
        item.to_json()
    );
    item.description.menu_category.category = Some(CreativeCategory::ItemCommandOnly);
    item.components.insert(MaxStackSize::new(16));
    assert_eq!(item.description.menu_category_presence, Presence::Absent);
    assert!(!item.description.category_is_written());
    assert_eq!(description(&item), r#"{"identifier":"demo:x"}"#);

    item.description.menu_category.category = Some(CreativeCategory::Equipment);
    assert!(item.description.category_is_written());
    assert_eq!(
        description(&item),
        r#"{"identifier":"demo:x","menu_category":{"category":"equipment"}}"#
    );

    // A group alone is written without inventing the category.
    item.description.menu_category.category = Some(CreativeCategory::ItemCommandOnly);
    item.description.menu_category.group = Some("demo:tools".to_owned());
    assert_eq!(
        description(&item),
        r#"{"identifier":"demo:x","menu_category":{"group":"demo:tools"}}"#
    );

    // An explicit `none` is asked for through the presence record.
    item.description.menu_category.group = None;
    item.description.menu_category_presence = Presence::Category;
    assert_eq!(
        description(&item),
        r#"{"identifier":"demo:x","menu_category":{"category":"none"}}"#
    );
}

/// An item cannot be written with two members named `__deprecated_components`: the legacy
/// data it holds, and an unknown member of that name among its kept ones (a value only a
/// caller can build: a file's member of that name is read as the one or as the other). The
/// writer wrote the first and left the second out without a word; it refuses the item at
/// the member.
#[test]
fn an_unknown_side_car_member_beside_the_held_one_is_refused_not_dropped() {
    let document = loads(
        &item(
            "1.20.30",
            "demo:x",
            r#""minecraft:on_use": {"on_use": {"event": "e"}}, "minecraft:max_stack_size": 16"#,
        ),
        &allowlisted(),
    );
    let mut held = document;
    assert!(!held.deprecated.is_empty(), "the legacy trigger is held");
    // the 1.26.50 format binds no side-car and the 1.26.0 root binds it as a
    // deprecated name the game refuses: no text reads back as the item, and the
    // writer refuses it at the side-car
    match held.to_json() {
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:item/__deprecated_components");
        }
        other => panic!("{other:?}"),
    }
    let item_at = bedrock_addon_core::diagnostic::JsonPath::of_keys(["minecraft:item"]);
    held.unknown_mut().put(
        &item_at,
        "__deprecated_components",
        bedrock_addon_core::json::from_str(r#"{"zz": 1}"#).unwrap(),
    );
    match held.to_json() {
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:item/__deprecated_components");
        }
        other => panic!("{other:?}"),
    }
    // Alone, the unknown member is also refused: the strict 1.26.50 format does not bind it.
    let mut alone = held.map(|_| {
        let mut alone = Item::new(ItemId::parse("demo:x"));
        alone.components.insert(MaxStackSize::new(16));
        alone
    });
    *alone.unknown_mut() = bedrock_addon_core::document::Unknown::new();
    alone.unknown_mut().put(
        &item_at,
        "__deprecated_components",
        bedrock_addon_core::json::from_str(r#"{"zz": 1}"#).unwrap(),
    );
    assert!(refused_at(
        &alone.to_json(),
        "minecraft:item/__deprecated_components"
    ));
}

/// A new item has no component, and a document without one is the legacy item format to the
/// game at every version: the item is written as that document, the one the game's own
/// `honeycomb.json` is, and its text is what its own reading writes.
#[test]
fn a_new_item_without_a_component_is_written_as_the_legacy_document_it_is() {
    let new = Item::new(ItemId::parse("demo:honeycomb"));
    let written = new.to_json().expect("a new item is written");
    // The shape of the game's `honeycomb.json`, built in memory and read from a file: one text.
    let read = loads(
        r#"{"format_version": "1.16", "minecraft:item": {"description": {"identifier": "demo:honeycomb"}}}"#,
        &third_party(),
    );
    assert_eq!(read.legacy.format, ItemFormat::Legacy);
    assert_eq!(read.to_json().ok().as_ref(), Some(&written));
    let mut by_hand = Item::new(ItemId::parse("demo:honeycomb"));
    by_hand.legacy.format = ItemFormat::Legacy;
    by_hand.legacy.format_item = Some(bedrock_addon_family_item::LegacyFormatItem::default());
    assert_eq!(by_hand.to_json().ok().as_ref(), Some(&written));
    // It reads back as the legacy item the game makes of it, with no finding but the note
    // of the format, and is written as itself.
    let again = loads(&written, &third_party());
    assert_eq!(again.legacy.format, ItemFormat::Legacy);
    assert_eq!(again.description.identifier, new.description.identifier);
    assert_eq!(again.components.iter().count(), 0);
    assert!(
        again
            .diagnostics()
            .iter()
            .all(|f| f.code.id() == "core/legacy_item_format"),
        "{:?}",
        again.diagnostics()
    );
    assert_eq!(again.to_json().ok(), Some(written));
}

/// A JSON value on one line without spaces, as the tests above spell their expectations.
fn compact(value: &bedrock_addon_core::json::Value) -> String {
    let mut out = String::new();
    let mut in_text = false;
    let mut escaped = false;
    for c in bedrock_addon_core::json::to_json(value).unwrap().chars() {
        if in_text {
            out.push(c);
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_text = false,
                _ => {}
            }
        } else if c == '"' {
            in_text = true;
            out.push(c);
        } else if !c.is_whitespace() {
            out.push(c);
        }
    }
    out
}

/// What the load says of a component-based item it leaves without a registered component.
fn told_at_components(document: &bedrock_addon_core::document::Document<Item>) -> usize {
    document
        .diagnostics()
        .iter()
        .filter(|f| {
            f.code.id() == "core/invalid_value"
                && f.severity == bedrock_addon_core::diagnostic::Severity::Info
                && f.phase == bedrock_addon_core::diagnostic::Phase::Validate
                && f.location.json_path.to_string() == "minecraft:item/components"
        })
        .count()
}

/// Whether `written` refuses the item at `components`.
fn refused_at_components(written: &Result<String, bedrock_addon_core::json::EncodeError>) -> bool {
    matches!(
        written,
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. })
            if at.to_string() == "minecraft:item/components"
    )
}

/// A document is component-based to the game by a REGISTERED component of its `components`
/// (`isComponentBasedItemSchema`): keyed custom components are not registered, so the newest
/// format has no text for an item that holds nothing else. An older document whose only
/// component is the list `minecraft:custom_components` is such an item once it is upgraded:
/// the load says so at `components`, and the writer refuses it there instead of writing a text
/// that reads back as a legacy item without its components.
#[test]
fn an_item_of_custom_components_alone_is_told_at_the_load_and_not_written() {
    let list = r#""minecraft:custom_components": ["demo:shake"]"#;
    for version in ["1.21.10", "1.21.80"] {
        let document = loads(&item(version, "demo:x", list), &third_party());
        assert_eq!(
            told_at_components(&document),
            1,
            "{version}: {:?}",
            document.diagnostics()
        );
        // The finding says what is wrong, in words of ours (the code alone is "invalid
        // value").
        assert!(
            document.diagnostics().iter().any(|f| f
                .to_string()
                .ends_with("invalid value: no registered component is left, so the newest format reads the item as a legacy item: it is not written")),
            "{:?}",
            document.diagnostics()
        );
        // ours (the game is silent), an information
        assert!(
            document
                .diagnostics()
                .iter()
                .filter(|f| f.to_string().contains("no registered component is left"))
                .all(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Info)
        );
        let held = document.into_value();
        assert_eq!(held.legacy.format, ItemFormat::DataDriven, "{version}");
        assert_eq!(
            held.components.iter().count(),
            1,
            "{version}: the custom component is held"
        );
        assert!(
            refused_at_components(&held.to_json()),
            "{version}: {:?}",
            held.to_json()
        );
    }
    // The list of a newest-format document is deprecated and not read (told there); what is
    // left is a keyed custom component alone.
    let both = r#""minecraft:custom_components": ["demo:shake"], "demo:shake": {"a": 1}"#;
    // (a 1.26.30 file with the list is refused: the list is a deprecated name)
    let d = rejects(&item("1.26.30", "demo:x", both), &third_party());
    assert!(
        d.iter().any(
            |f| f.severity == bedrock_addon_core::diagnostic::Severity::Error
                && f.code.to_string() == "member \"minecraft:custom_components\" is deprecated"
        ),
        "{d:#?}"
    );
    // Beside one registered component the item is component-based: nothing is told, it is
    // written in the newest format and reads back as the item it was.
    for version in ["1.21.10", "1.21.80"] {
        let with = format!(r#"{list}, "minecraft:max_stack_size": 16"#);
        let document = loads(&item(version, "demo:x", &with), &third_party());
        assert_eq!(
            told_at_components(&document),
            0,
            "{version}: {:?}",
            document.diagnostics()
        );
        let held = document.into_value();
        let written = held.to_json().expect("written");
        let again = loads(&written, &third_party());
        assert_eq!(told_at_components(&again), 0, "{version}");
        let again = again.into_value();
        assert_eq!(
            again.legacy.format,
            ItemFormat::DataDriven,
            "{version}: {written}"
        );
        // (`from_v1_list` records that the file had the list: the written text has the keys.)
        assert_eq!(again.components.iter().count(), 2, "{version}: {written}");
        assert!(
            again.components.iter().any(
                |entry| matches!(entry, ComponentRef::Custom(custom) if custom.name == "demo:shake")
            ),
            "{version}: {written}"
        );
        assert_eq!(
            again.to_json().expect("written again"),
            written,
            "{version}"
        );
    }
    // A component-based item the caller completes with a registered component is written.
    let mut held = loads(&item("1.21.10", "demo:x", list), &third_party()).into_value();
    held.components.insert(MaxStackSize::new(16));
    let written = held.to_json().expect("written");
    let again = loads(&written, &third_party()).into_value();
    assert_eq!(again.legacy.format, ItemFormat::DataDriven, "{written}");
    assert_eq!(again.components.iter().count(), 2, "{written}");
}

/// The same for an item built in memory: what a legacy document cannot hold (a side-car
/// member, a component that is not registered) is refused at `components`, where the
/// registered component is missing, and never written into a text that reads back without it.
#[test]
fn a_hand_built_item_without_a_registered_component_has_no_text_but_the_empty_one() {
    let new = || Item::new(ItemId::parse("demo:x"));
    // Nothing at all: the legacy document.
    assert!(new().to_json().is_ok());
    let mut side_car = new();
    side_car.deprecated.render_offsets =
        Some(bedrock_addon_core::json::from_str(r#""tools""#).unwrap());
    assert!(
        refused_at_components(&side_car.to_json()),
        "{:?}",
        side_car.to_json()
    );
    let mut unknown = new();
    unknown.components.insert_unknown(
        "demo:what",
        bedrock_addon_core::json::from_str("{}").unwrap(),
    );
    assert!(
        refused_at_components(&unknown.to_json()),
        "{:?}",
        unknown.to_json()
    );
    let mut custom = new();
    custom
        .components
        .insert_custom(bedrock_addon_core::document::CustomComponent {
            name: "demo:shake".to_owned(),
            parameters: bedrock_addon_core::json::from_str("{}").unwrap(),
            from_v1_list: false,
        });
    assert!(
        refused_at_components(&custom.to_json()),
        "{:?}",
        custom.to_json()
    );
    // Each beside a registered component: the custom component and the unknown one (a
    // namespaced key, read back as a custom component) are written and read back
    // component-based; the side-car is no member of the strict 1.26.50 format, and its writer
    // refuses it there.
    for mut item in [unknown, custom] {
        item.components.insert(MaxStackSize::new(16));
        let written = item.to_json().expect("written");
        let again = loads(&written, &allowlisted()).into_value();
        assert_eq!(again.legacy.format, ItemFormat::DataDriven, "{written}");
    }
    // The side-car is a deprecated name of the 1.26.0 root: refused at it.
    let mut side_car = side_car;
    side_car.components.insert(MaxStackSize::new(16));
    match side_car.to_json() {
        Err(bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:item/__deprecated_components");
        }
        other => panic!("{other:?}"),
    }
}

/// A component-based item's `minecraft:block` and `minecraft:frame_count` are read by the
/// game's legacy reader at every version and are no component of any format: the item holds
/// them in its legacy reading, and the writer writes them from there.
#[test]
fn the_block_and_the_frame_count_of_a_component_based_item_are_written_back() {
    for (version, cx) in [
        ("1.26.30", third_party()),
        ("1.21.10", third_party()),
        ("1.20.0", allowlisted()),
        ("1.16.100", allowlisted()),
    ] {
        for (member, written_as) in [
            (
                r#""minecraft:block": "minecraft:stone""#,
                r#""minecraft:block": "minecraft:stone""#,
            ),
            (
                r#""minecraft:frame_count": 3"#,
                r#""minecraft:frame_count": 3"#,
            ),
        ] {
            let text = item(
                version,
                "demo:x",
                &format!(r#"{member}, "minecraft:max_stack_size": 16"#),
            );
            let read = loads(&text, &cx);
            let told: Vec<String> = read
                .diagnostics()
                .iter()
                .filter(|f| {
                    f.location.json_path.to_string().contains(":block")
                        || f.location.json_path.to_string().contains("frame_count")
                })
                .map(|f| f.code.id().to_owned())
                .collect();
            assert_eq!(told, [] as [&str; 0], "{version} {member}");
            let read = read.into_value();
            let written = read.to_json().expect("written");
            assert!(written.contains(written_as), "{version}: {written}");
            let again = loads(&written, &cx).into_value();
            assert_eq!(
                again.legacy.block, read.legacy.block,
                "{version}: {written}"
            );
            assert_eq!(
                again.legacy.frame_count, read.legacy.frame_count,
                "{version}"
            );
            assert_eq!(
                again.to_json().expect("written again"),
                written,
                "{version}"
            );
            if version == "1.26.30" {
                assert_eq!(again, read, "{written}");
            }
        }
    }
}

/// An item read at 1.21.60 to 1.21.109 without a category, or with `none`, has the category
/// `Undefined`, which the game keeps through its upgrade and which has no spelling in the
/// newest format: the item is written with `none`, and that reads back as the command-only
/// category, another value. The load says so, a warning of the upgrade at
/// `description/menu_category` (the library changes what the game keeps). An item of those
/// versions with a category that has a spelling, and an item from 1.21.110 on, say nothing.
#[test]
fn an_undefined_category_the_newest_format_cannot_spell_is_told_at_the_load() {
    use bedrock_addon_shared_shapes::CreativeCategory;
    let told = |version: &str, description: &str| -> (Vec<String>, Item) {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:copper_spear"{description}}}, "components": {{"minecraft:icon": {{"textures": {{"default": "copper_spear"}}}}, "minecraft:max_stack_size": 1}}}}}}"#
        );
        let document = loads(&text, &third_party());
        let told = document
            .diagnostics()
            .iter()
            .map(|f| {
                format!(
                    "{} {:?} {:?} at {}",
                    f.code.id(),
                    f.severity,
                    f.phase,
                    f.location.json_path
                )
            })
            .collect();
        (told, document.into_value())
    };
    // Ours: an information.
    let expected =
        ["core/value_rewritten Info Upgrade at minecraft:item/description/menu_category"];
    for version in ["1.21.60", "1.21.100"] {
        for description in ["", r#", "menu_category": {"category": "none"}"#] {
            let (found, item) = told(version, description);
            assert_eq!(found, expected, "{version} {description}");
            assert_eq!(
                item.description.menu_category.category,
                Some(CreativeCategory::Undefined)
            );
            // What is told: the written text is `none`, which reads back as another value.
            let written = item.to_json().unwrap();
            let again = loads(&written, &third_party());
            assert_eq!(
                again.description.menu_category.category,
                Some(CreativeCategory::ItemCommandOnly),
                "{written}"
            );
            assert!(again.diagnostics().is_empty(), "{:?}", again.diagnostics());
        }
        let (found, _) = told(version, r#", "menu_category": {"category": "items"}"#);
        assert_eq!(found, [] as [&str; 0], "{version}");
    }
    // Outside the band: `items` below it, `none` itself from 1.21.110.
    for version in ["1.21.50", "1.21.110", "1.26.30"] {
        let (found, _) = told(version, "");
        assert_eq!(found, [] as [&str; 0], "{version}");
    }
}

/// What becomes of one member of the legacy reading when the item is written.
enum Kept {
    /// The written text has it: this text is in what the document is written as.
    Written(&'static str),
    /// The load says, at this member, that the written text does not have it.
    Told(&'static str, &'static str),
    /// Nothing of the file is in it: why.
    Derived(&'static str),
}

use Kept::{Derived, Told, Written};

/// A row of [`LEGACY_ROWS`]: field, version, allow-listed, components (or a description member),
/// what becomes of it.
type Row = (&'static str, &'static str, bool, &'static str, Kept);

/// What becomes of each member of the legacy reading, one row per field of `LegacyItemData`.
const LEGACY_ROWS: &[Row] = &[
    (
        "format",
        "",
        false,
        "",
        Derived("which of the two formats the item is written in"),
    ),
    (
        "format_item",
        "",
        false,
        "",
        Derived("the legacy document, written whole (`a_new_item_…`, the 1.10 round trips)"),
    ),
    (
        "left_empty_by_the_load",
        "",
        false,
        "",
        Derived("a mark of the load; the writer refuses the item (the `k117_` tests)"),
    ),
    (
        "expressions",
        "",
        false,
        "",
        Derived("the compiled Molang of members that are written where they are"),
    ),
    // (at 1.26.30 an unknown description member rejects the item)
    (
        "category",
        "1.21.90",
        false,
        "description:\"category\": \"Nature\"",
        Written(r#""category": "Nature""#),
    ),
    (
        "category",
        "1.20.0",
        true,
        "description:\"category\": \"Nature\"",
        Told("core/invalid_value", "minecraft:item/description/category"),
    ),
    (
        "block",
        "1.26.30",
        false,
        r#""minecraft:block": "minecraft:stone""#,
        Written(r#""minecraft:block": "minecraft:stone""#),
    ),
    (
        "block",
        "1.20.0",
        true,
        r#""minecraft:block": "minecraft:stone""#,
        Written(r#""minecraft:block": "minecraft:stone""#),
    ),
    (
        "mining_speed",
        "1.20.0",
        true,
        r#""minecraft:mining_speed": 2"#,
        Told(
            "core/dropped_field",
            "minecraft:item/components/minecraft:mining_speed",
        ),
    ),
    (
        "mining_speed",
        "1.26.30",
        false,
        r#""minecraft:mining_speed": 2"#,
        Told(
            "item/mining_speed_deprecated",
            "minecraft:item/components/minecraft:mining_speed",
        ),
    ),
    (
        "frame_count",
        "1.26.30",
        false,
        r#""minecraft:frame_count": 3"#,
        Written(r#""minecraft:frame_count": 3"#),
    ),
    (
        "frame_count",
        "1.20.0",
        true,
        r#""minecraft:frame_count": 3"#,
        Written(r#""minecraft:frame_count": 3"#),
    ),
    (
        "creative_category",
        "1.20.0",
        true,
        r#""minecraft:creative_category": {"parent": "x"}"#,
        Told(
            "core/dropped_field",
            "minecraft:item/components/minecraft:creative_category",
        ),
    ),
    (
        "creative_category",
        "1.26.30",
        false,
        r#""minecraft:creative_category": {"parent": "x"}"#,
        Told(
            "item/creative_category_deprecated",
            "minecraft:item/components/minecraft:creative_category",
        ),
    ),
    (
        "tags",
        "1.20.0",
        true,
        r#""tag:a": {}"#,
        Written(r#""minecraft:tags": { "tags": [ "a" ] }"#),
    ),
    (
        "tags",
        "1.26.30",
        false,
        r#""tag:a": {}"#,
        Told("item/tag_deprecated", "minecraft:item/components/tag:a"),
    ),
    (
        "is_glint",
        "1.20.0",
        true,
        r#""minecraft:foil": true"#,
        Told(
            "core/dropped_field",
            "minecraft:item/components/minecraft:foil",
        ),
    ),
    (
        "is_glint",
        "1.26.30",
        false,
        r#""minecraft:foil": true"#,
        Told(
            "item/foil_deprecated",
            "minecraft:item/components/minecraft:foil",
        ),
    ),
    (
        "seed",
        "1.16.100",
        true,
        r#""minecraft:seed": {"crop_result": "a"}"#,
        Told(
            "core/dropped_field",
            "minecraft:item/components/minecraft:seed",
        ),
    ),
    (
        "seed",
        "1.26.30",
        false,
        r#""minecraft:seed": {"crop_result": "a"}"#,
        Told(
            "item/seed_deprecated",
            "minecraft:item/components/minecraft:seed",
        ),
    ),
    (
        "shared",
        "1.26.30",
        false,
        r#""minecraft:use_duration": 1.6"#,
        Written(r#""minecraft:use_duration": 1.6"#),
    ),
    (
        "shared",
        "1.20.0",
        true,
        r#""minecraft:use_modifiers": {"use_duration": 1.6, "movement_modifier": 0.5}"#,
        Told(
            "core/dropped_field",
            "minecraft:item/components/minecraft:use_modifiers",
        ),
    ),
    (
        "held_in_shared_alone",
        "",
        false,
        "",
        Derived("a mark of the newest parse: which keys of `shared` the writer writes"),
    ),
];

/// Every member of the legacy reading of a component-based item is written back, or the
/// load says at the member of the file that it is not; the table has a row for every field
/// of `LegacyItemData`, so a field added later fails here until it has one.
#[test]
fn every_member_of_the_legacy_reading_is_written_or_told() {
    let rows = LEGACY_ROWS;
    // Every field of the struct has a row, and every row names a field.
    let debug = format!("{:?}", bedrock_addon_family_item::LegacyItemData::default());
    let body = debug
        .strip_prefix("LegacyItemData { ")
        .and_then(|rest| rest.strip_suffix(" }"))
        .expect("the Debug of a struct");
    let mut fields: Vec<&str> = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, c) in body.char_indices() {
        match c {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ':' if depth == 0 && body[index..].starts_with(": ") => {
                if let Some(name) = body[start..index].rsplit(", ").next() {
                    fields.push(name);
                }
                start = index;
            }
            _ => {}
        }
    }
    let mut named: Vec<&str> = rows.iter().map(|row| row.0).collect();
    named.dedup();
    assert_eq!(
        named, fields,
        "a row for every field of `LegacyItemData`, in its order"
    );
    for (field, version, listed, member, kept) in rows {
        let (description, components) = match member.strip_prefix("description:") {
            Some(member) => (format!(", {member}"), String::new()),
            None => (String::new(), format!("{member}, ")),
        };
        let cx = if *listed {
            allowlisted()
        } else {
            third_party()
        };
        match kept {
            Derived(why) => assert!(!why.is_empty()),
            Written(text) => {
                // Kept: in the newest writer's text, or, for a member the strict 1.26.50 format
                // does not bind, in the payload the item holds, where the newest writer refuses
                // it.
                let document = loads(&legacy_probe(version, &description, &components), &cx);
                let written = document
                    .to_json()
                    .unwrap_or_else(|_| newest_payload(&document));
                let line = written.split_whitespace().collect::<Vec<_>>().join(" ");
                assert!(line.contains(text), "{field} at {version}: {line}");
            }
            Told(code, at) => {
                // The game discards a `seed` at 1.16.100 too (an information). A member the
                // library does not carry is an information of ours (the steps' notes); a deprecated member is the game's level-3 line with the item kept (a
                // kept error), an unknown one its level-2 line (a warning). The whole table,
                // by version and writer, is `legacy_audit.rs`.
                use bedrock_addon_core::diagnostic::Severity;
                let severity = match *code {
                    "item/mining_speed_deprecated"
                    | "item/creative_category_deprecated"
                    | "item/tag_deprecated"
                    | "item/foil_deprecated" => Severity::Error,
                    // (`seed`'s is the level-2 `upgradeJson` line)
                    "item/seed_deprecated"
                    | "core/unrecognized_field"
                    | "core/member_not_kept"
                    | "core/member_not_kept_worded" => Severity::Warning,
                    _ => Severity::Info,
                };
                let document = loads(&legacy_probe(version, &description, &components), &cx);
                assert!(
                    document.diagnostics().iter().any(|f| f.code.id() == *code
                        && f.severity == severity
                        && f.location.json_path.to_string() == *at),
                    "{field} at {version}: no {code} at {at} in {:?}",
                    document.diagnostics()
                );
            }
        }
    }
}

/// An item document with `description` members after the identifier and `components` before
/// a registered component.
fn legacy_probe(version: &str, description: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"{description}}}, "components": {{{components}"minecraft:max_stack_size": 16}}}}}}"#
    )
}

/// The newest writer never writes what the strict
/// 1.26.50 format refuses and never drops it silently. An upgraded item holding `events` (or
/// the side-car, or an unknown member) is written as a 1.26.0 legacy component document, the
/// newest format whose reader binds those members, when that text reads back as the same item.
#[test]
fn an_upgraded_item_with_events_is_written_at_1_26_0() {
    let older = r#"{"format_version": "1.20.50", "minecraft:item": {"description": {"identifier": "demo:old", "mystery": 1}, "components": {"minecraft:max_stack_size": 16, "minecraft:food": {"nutrition": 1, "on_consume": {"event": "demo:e"}}}, "events": {"demo:e": {"run_command": {"command": ["say hi"]}}}}}"#;
    let first = loads(older, &allowlisted());
    let written = first.to_json().expect("written at 1.26.0");
    assert!(
        written.contains("\"format_version\": \"1.26.0\""),
        "{written}"
    );
    for member in ["\"events\"", "\"mystery\""] {
        assert!(written.contains(member), "{member}: {written}");
    }
    let again = loads(&written, &allowlisted());
    let mut a = again.into_value();
    let mut b = first.into_value();
    a.legacy.shared.clear();
    b.legacy.shared.clear();
    assert_eq!(a, b, "{written}");
}

/// An item whose 1.26.0 text would not read back as itself (a use that starts
/// `if_first`, which the 1.26.30 step turns into `always`) and that holds `events` has no
/// text: `NotRepresentable` at the member the 1.26.50 format refuses.
#[test]
fn an_item_without_a_text_that_reads_back_is_not_representable() {
    let mut item = Item::new(ItemId::parse("demo:new"));
    item.components
        .insert(bedrock_addon_family_item::component::UseModifiers::new(1.0));
    item.events = Some(bedrock_addon_core::json::from_str(r#"{"demo:e": {}}"#).unwrap());
    assert!(
        refused_at(&item.to_json(), "minecraft:item/events"),
        "{:?}",
        item.to_json()
    );
}

/// `LoaderPerfTests-1.json` of 1.26.60 (`ComponentItemDataLoaderPerfTests::json`).
/// The game's test list reads "each document must load", but the test body
/// calls `CerealComponentItemDataLoader::load` and asserts nothing of
/// the result, and the item does not load in the game: its `menu_category.group`
/// "itemGroup.name.axe" has no namespace, and `v1_21_60::ItemMenuCategory` binds `group` with
/// `AllOf{NamespaceConstraint, locIdStringConstraint}`, whose
/// `NamespaceConstraint::validateValue` logs 0x40 "string must be prefixed with a
/// namespace (eg. namespace:value)", an error under either mask. The crate refuses
/// it with the same finding and nothing else.
#[test]
fn loader_perf_tests_1_the_perf_item_is_refused_for_its_group_as_in_the_game() {
    let path = format!(
        "{}/../../addon/tests/fixtures/item/item/loader_perf/LoaderPerfTests-1.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let d = rejects(&text, &third_party());
    let found = above_info(&d);
    assert_eq!(found.len(), 1, "{d:#?}");
    assert!(found[0].code.id() == "item/group_namespace", "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:item/description/menu_category/group"
    );
    assert_eq!(
        found[0].code.to_string(),
        "menu_category -> group: string must be prefixed with a namespace (eg. namespace:value)"
    );
    // With a namespace the same item loads without a warning and round-trips.
    let fixed = text.replace("itemGroup.name.axe", "minecraft:itemGroup.name.axe");
    let document = loads(&fixed, &third_party());
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let written = document.to_json().unwrap();
    let again = loads(&written, &third_party());
    assert!(
        above_info(again.diagnostics()).is_empty(),
        "{:#?}",
        again.diagnostics()
    );
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn upgrade_keeps_the_unknown_members_a_parse_keeps() {
    for version in ["1.10.0", "1.20.40"] {
        let old = format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{
                "description": {{"identifier": "demo:old", "zz_description": 1}},
                "components": {{"minecraft:max_stack_size": 1}},
                "zz_payload": 2
            }}}}"#
        );
        let path = PackPath::from_static("items/x.json");
        let parsed = loads(&old, &third_party());
        assert!(
            !parsed.unknown().is_empty(),
            "{version}: the parse keeps a member"
        );
        let document = Item::parse_as_written(&path, old.as_bytes(), &third_party()).unwrap();
        let upgraded =
            bedrock_addon_family_item::history::ItemHistory::upgrade(document, &third_party())
                .unwrap();
        assert_eq!(*upgraded.value(), *parsed.value(), "{version}");
        assert_eq!(upgraded.unknown(), parsed.unknown(), "{version}");
    }
}

/// A document the gates refuse (below the item formats' release, no legacy creator features)
/// after its description kept an unknown member is rejected with the findings of the read:
/// the members go with the rejection, and handing the findings on does not drop them.
#[test]
fn a_document_refused_by_the_gates_after_a_kept_member_is_rejected() {
    let text = r#"{"format_version": "1.16.100", "minecraft:item": {
        "description": {"identifier": "demo:old", "zz_description": 1},
        "components": {"minecraft:max_stack_size": 1}
    }}"#;
    let outcome = Item::parse(
        &PackPath::from_static("items/x.json"),
        text.as_bytes(),
        &bedrock_addon_core::load::LoadContext::detached(),
    );
    let Err(rejected) = outcome else {
        panic!("the gates refuse the item in a detached context");
    };
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|d| d.code.id() == "core/toggle_use_format_version"),
        "{:?}",
        rejected.diagnostics
    );
}
