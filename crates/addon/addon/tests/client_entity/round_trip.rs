//! Round trips and
//! what write-back keeps and never invents.

use bedrock_addon::document::Content;
use bedrock_addon::families::client_entity::{ClientEntity, Description};
use bedrock_addon::id::{EntityId, MaterialId};
use bedrock_addon::json::{EncodeError, Value, WriteStyle};
use bedrock_addon::version::FormatVersion;

use super::common::*;

const FAMILY: &str = "client_entity";

fn json(text: &str) -> Value {
    bedrock_addon::json::from_str_jsonc(text).expect("test JSON")
}

/// `experimental_y_2026_drop_4/entity/frostbite.entity.json` of 1.26.60
/// (client entity 1.26.0): it loads without a warning, round-trips, and
/// writes back the JSON value it read.
#[test]
fn the_drop_4_frostbite_round_trips_and_writes_what_it_read() {
    let text = fixture(FAMILY, "vanilla_frostbite.entity.json");
    let document = loaded(entity(&text, &rp(1, 26, 30)));
    clean(document.diagnostics());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 0))
    );
    let written = round_trip(&document, |text| entity(text, &rp(1, 26, 30)));
    assert_eq!(json(&written), json(&text));
}

#[test]
fn a_vanilla_1_26_file_round_trips_and_writes_what_it_read() {
    let text = fixture(FAMILY, "vanilla_sulfur_cube.entity.json");
    let document = loaded(entity(&text, &rp(1, 26, 30)));
    clean(document.diagnostics());
    let written = round_trip(&document, |text| entity(text, &rp(1, 26, 30)));
    // Nothing is added, dropped or reordered beyond the members' declaration order.
    let (read, wrote) = (json(&text), json(&written));
    let description = |v: &Value| -> Value {
        v.as_object().unwrap()["minecraft:client_entity"]
            .as_object()
            .unwrap()["description"]
            .clone()
    };
    let mut keys_read: Vec<String> = description(&read)
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| k.as_str().to_owned())
        .collect();
    let mut keys_wrote: Vec<String> = description(&wrote)
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| k.as_str().to_owned())
        .collect();
    keys_read.sort();
    keys_wrote.sort();
    assert_eq!(keys_read, keys_wrote);
    for key in &keys_read {
        let member = |v: &Value| description(v).as_object().unwrap()[key.as_str()].clone();
        assert_eq!(member(&read), member(&wrote), "{key}");
    }
    assert!(
        document
            .description
            .slots
            .scripts
            .as_ref()
            .unwrap()
            .hide_held_items
            .is_some()
    );
}

#[test]
fn the_full_1_26_shape_round_trips() {
    let text = fixture(FAMILY, "s02_1_10_to_1_26_types.after.json");
    let document = loaded(entity(&text, &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert_eq!(json(&written), json(&text));
}

/// Requires both writers to refuse `entity` at `path` and to give out no text.
#[track_caller]
fn refused(entity: &ClientEntity, path: &str) {
    match entity.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => assert_eq!(at.to_string(), path),
        Err(other) => panic!("{other}"),
        Ok(text) => {
            let outcome = super::common::entity(&text, &cx());
            panic!(
                "written: {text}\nwhich this loader {}",
                if outcome.is_ok() { "reads" } else { "refuses" }
            );
        }
    }
    let mut out = Vec::new();
    assert!(entity.write_json(&mut out, WriteStyle::default()).is_err());
    assert!(out.is_empty());
}

#[test]
fn a_description_the_schema_refuses_is_not_written() {
    // A value built in code may hold what no file can; the text is given out only when this
    // loader reads it.
    const AT: &str = "minecraft:client_entity/description";
    refused(
        &ClientEntity::new(EntityId::parse("")),
        &format!("{AT}/identifier"),
    );
    let mut key = ClientEntity::new(EntityId::parse("demo:ghost"));
    key.description
        .slots
        .materials
        .insert("my:slot".to_owned(), MaterialId::parse("m"));
    refused(&key, &format!("{AT}/materials/my:slot"));
    let future = ClientEntity {
        format_version: FormatVersion::new(99, 0, 0),
        ..ClientEntity::new(EntityId::parse("demo:ghost"))
    };
    refused(&future, "format_version");
    // What the loader reads is written: the engine's own version, a held empty condition.
    let engine = ClientEntity {
        format_version: bedrock_addon::version::ENGINE_VERSION,
        ..ClientEntity::new(EntityId::parse("demo:ghost"))
    };
    loaded(entity(&engine.to_json().unwrap(), &cx()));
}

#[test]
fn write_back_writes_no_member_the_author_did_not_write() {
    // a minimal document writes its identifier and nothing else.
    let document = loaded(entity(&ghost("1.26.0", ""), &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert_eq!(
        json(&written),
        json(
            r#"{"format_version": "1.26.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}}"#
        )
    );
    // So does one built in code.
    let built = ClientEntity::new(EntityId::parse("demo:ghost"));
    assert_eq!(built.to_json().unwrap(), written);
    assert_eq!(
        built.description,
        Description::new(EntityId::parse("demo:ghost"))
    );
    // A written `false` and an empty object the schema allows are kept where the member has
    // a presence of its own.
    let members = r#""particle_effects": {}, "enable_attachables": false, "spawn_egg": {}, "scripts": {}, "render_controllers": []"#;
    let document = loaded(entity(&ghost("1.26.0", members), &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert_eq!(json(&written), json(&ghost("1.26.0", members)));
}

#[test]
fn an_authored_empty_slot_map_is_written_back() {
    // `animations` and `sound_effects` keep their
    // presence, so an empty object the author wrote is written as it was read.
    for members in [
        r#""animations": {}"#,
        r#""sound_effects": {}"#,
        r#""animations": {}, "sound_effects": {}"#,
    ] {
        let document = loaded(entity(&ghost("1.26.0", members), &cx()));
        let written = round_trip(&document, |text| entity(text, &cx()));
        assert_eq!(json(&written), json(&ghost("1.26.0", members)), "{members}");
    }
    // A document without the members gets none.
    let document = loaded(entity(&ghost("1.26.0", ""), &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert_eq!(json(&written), json(&ghost("1.26.0", "")));
}

#[test]
fn held_item_scale_is_written_as_the_shortest_number_that_reads_back() {
    // The vanilla sulfur cube's 2.678 must not come back as 2.677999973297119.
    let document = loaded(entity(
        &ghost("1.26.0", r#""held_item_scale": 2.678"#),
        &cx(),
    ));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert!(
        written.contains("\"held_item_scale\": 2.678\n"),
        "{written}"
    );
}

#[test]
fn held_item_scale_deviation_an_integer_token_is_written_as_a_real() {
    // the variant keeps no token kind.
    let document = loaded(entity(&ghost("1.26.0", r#""held_item_scale": 2"#), &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert!(written.contains("\"held_item_scale\": 2.0\n"), "{written}");
}

#[test]
fn the_version_written_is_the_document_s_own_and_never_below_1_26_0() {
    // A newer stamp stays.
    let document = loaded(entity(&ghost("1.26.30", ""), &cx()));
    assert_eq!(document.format_version, FormatVersion::new(1, 26, 30));
    assert!(document.to_json().unwrap().contains(r#""1.26.30""#));
    // An upgraded document is written at 1.26.0 ("Upgraded").
    let old = loaded(entity(&ghost("1.10.0", ""), &cx()));
    assert_eq!(old.format_version, FormatVersion::new(1, 26, 0));
    assert!(old.to_json().unwrap().contains(r#""1.26.0""#));
    // A value built with an older stamp is still written as the shape it is.
    let stale = ClientEntity {
        format_version: FormatVersion::new(1, 10, 0),
        ..ClientEntity::new(EntityId::parse("demo:ghost"))
    };
    assert!(stale.to_json().unwrap().contains(r#""1.26.0""#));
}

#[test]
fn an_upgraded_document_written_at_1_26_reads_back_to_the_same_value() {
    for name in [
        "s01_1_8_controllers_to_animate.before.json",
        "s02_1_10_to_1_26_types.before.json",
    ] {
        let document = loaded(entity(&fixture(FAMILY, name), &cx()));
        round_trip(&document, |text| entity(text, &cx()));
    }
    // Lower-cased scripts are written lower-cased, so the 1.26.0 text means what the old one
    // did.
    let members = r#""scripts": {"pre_animation": ["Variable.X = 1;"], "scale": "Query.Is_Baby"}"#;
    let document = loaded(entity(&ghost("1.16.0", members), &cx()));
    let written = round_trip(&document, |text| entity(text, &cx()));
    assert!(written.contains("variable.x = 1;"), "{written}");
    assert!(written.contains("query.is_baby"), "{written}");
}

#[test]
fn a_member_named_extra_is_an_undeclared_member_like_any_other() {
    // The structs have no `extra` field for facet to bind the name to.
    let d = rejected(entity(&ghost("1.26.0", r#""extra": 1"#), &cx()));
    expect(
        &d,
        "core/child_not_valid",
        bedrock_addon::diagnostic::Severity::Error,
        "child 'extra' not valid here.",
    );
}

#[test]
fn of_a_member_written_twice_the_last_stands() {
    // Of a member written twice, the last value stands.
    let text = r#"{"format_version": "1.26.0", "minecraft:client_entity": {"description": {"identifier": "demo:first", "identifier": "demo:ghost", "hide_armor": true, "hide_armor": false}}}"#;
    let document = loaded(entity(text, &cx()));
    assert_eq!(document.description.identifier.as_str(), "demo:ghost");
    assert!(!document.hide_armor());
    // Also for the version the loader's gate reads.
    let text = r#"{"format_version": "0.1.0", "format_version": "1.26.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}}"#;
    loaded(entity(text, &cx()));
    // The other way round the gate reads the last one too, and refuses it.
    let text = r#"{"format_version": "1.26.0", "format_version": "0.1.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}}"#;
    let d = rejected(entity(text, &cx()));
    assert!(
        find(
            &d,
            "visual/unknown_version",
            bedrock_addon::diagnostic::Severity::Error
        )
        .is_some()
    );
    // Which payload the file holds is read the same way: the last `minecraft:client_entity`
    // is no object, so with the attachable beside it the file is the attachable family's.
    let twice = r#"{"format_version": "1.26.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}, "minecraft:client_entity": 1, "minecraft:attachable": {}}"#;
    let once = r#"{"format_version": "1.26.0", "minecraft:client_entity": 1, "minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}, "minecraft:attachable": {}}"#;
    let files = [("entity/twice.json", twice), ("entity/once.json", once)];
    let set = stack(&[pack(&files)]);
    let entities = set.resource.client_entities();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities.rejected().len(), 0);
    assert_eq!(
        entities.iter().next().unwrap().1.origin().path.as_str(),
        "entity/once.json"
    );
}
