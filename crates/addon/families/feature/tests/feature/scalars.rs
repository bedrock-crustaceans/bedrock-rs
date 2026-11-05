//! How the plain scalar members of a feature read each JSON type: a bool member
//! (`can_place_on_floor`), a float member (`chance_of_spreading`) and a string member
//! (`structure_name`), each given every kind of token. Pinned when the members were strict
//! wrappers; the slice's strict scalar rule reads them the same.

use super::common::*;

const MULTIFACE: &str = r#""places_block": "minecraft:glow_lichen", "search_range": 20, "can_place_on_ceiling": true, "can_place_on_wall": true, "can_place_on": ["minecraft:stone"]"#;

/// Each finding of loading `json`: code, severity, path and span; `loaded` when it loaded.
fn read(json: &str) -> (bool, Vec<String>) {
    let (loaded, d) = match load(json, &cx()) {
        Ok(document) => (true, document.diagnostics().clone()),
        Err(rejected) => (false, rejected.diagnostics),
    };
    let found = d
        .iter()
        .map(|f| {
            format!(
                "{:?}|{}|{:?}|{:?}",
                f.severity,
                f.location.json_path,
                f.code.name(),
                f.location.span.map(|s| (s.offset, s.len))
            )
        })
        .collect();
    (loaded, found)
}

fn multiface(floor: &str, chance: &str) -> String {
    doc(
        "1.13.0",
        "multiface_feature",
        &format!(r#"{MULTIFACE}, "can_place_on_floor": {floor}, "chance_of_spreading": {chance}"#),
    )
}

fn template(name: &str) -> String {
    doc(
        "1.13.0",
        "structure_template_feature",
        &format!(r#""structure_name": {name}"#),
    )
}

#[test]
fn a_bool_float_and_string_member_read_every_token_as_pinned() {
    let tokens = [
        "true", "1", "0", "1.5", "\"true\"", "\"1\"", "null", "1e39", "[]",
    ];
    let mut seen = Vec::new();
    for token in tokens {
        seen.push((format!("floor {token}"), read(&multiface(token, "0.5"))));
        seen.push((format!("chance {token}"), read(&multiface("true", token))));
        seen.push((format!("name {token}"), read(&template(token))));
    }
    let rendered: Vec<String> = seen
        .iter()
        .map(|(what, (loaded, found))| format!("{what}: {loaded} {found:?}"))
        .collect();
    let expected: Vec<&str> = EXPECTED
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(rendered, expected, "{}", rendered.join("\n"));
}

const EXPECTED: &str = r#"
    floor true: true []
    chance true: false ["Error|minecraft:multiface_feature/chance_of_spreading|\"InvalidType\"|Some((285, 21))"]
    name true: false ["Error|minecraft:structure_template_feature/structure_name|\"InvalidType\"|Some((111, 16))"]
    floor 1: false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance 1: true []
    name 1: false ["Error|minecraft:structure_template_feature/structure_name|\"InvalidType\"|Some((111, 16))"]
    floor 0: false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance 0: true []
    name 0: false ["Error|minecraft:structure_template_feature/structure_name|\"InvalidType\"|Some((111, 16))"]
    floor 1.5: false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance 1.5: false ["Error|minecraft:multiface_feature/chance_of_spreading|\"InvalidValue\"|None"]
    name 1.5: false ["Error|minecraft:structure_template_feature/structure_name|\"InvalidType\"|Some((111, 16))"]
    floor "true": false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance "true": false ["Error|minecraft:multiface_feature/chance_of_spreading|\"InvalidType\"|Some((285, 21))"]
    name "true": false ["Error|minecraft:structure_template_feature/constraints|\"MissingRequiredField\"|Some((135, 1))"]
    floor "1": false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance "1": false ["Error|minecraft:multiface_feature/chance_of_spreading|\"InvalidType\"|Some((285, 21))"]
    name "1": false ["Error|minecraft:structure_template_feature/constraints|\"MissingRequiredField\"|Some((132, 1))"]
    floor null: false ["Error|minecraft:multiface_feature/can_place_on_floor|\"MissingRequiredField\"|Some((279, 4))", "Info|minecraft:multiface_feature/can_place_on_floor|\"ValueRewritten\"|None"]
    chance null: false ["Error|minecraft:multiface_feature/chance_of_spreading|\"MissingRequiredField\"|Some((312, 1))", "Info|minecraft:multiface_feature/chance_of_spreading|\"ValueRewritten\"|None"]
    name null: false ["Error|minecraft:structure_template_feature/structure_name|\"MissingRequiredField\"|Some((133, 1))", "Info|minecraft:structure_template_feature/structure_name|\"ValueRewritten\"|None"]
    floor 1e39: false ["Error|minecraft:multiface_feature/can_place_on_floor|\"InvalidType\"|Some((257, 20))"]
    chance 1e39: false ["Error|minecraft:multiface_feature/chance_of_spreading|\"InvalidType\"|Some((285, 21))"]
    name 1e39: false ["Error|minecraft:structure_template_feature/structure_name|\"InvalidType\"|Some((111, 16))"]
    floor []: false ["Error|minecraft:multiface_feature/can_place_on_floor/0|\"InvalidType\"|Some((280, 1))"]
    chance []: false ["Error|minecraft:multiface_feature/chance_of_spreading/0|\"InvalidType\"|Some((309, 1))"]
    name []: false ["Error|minecraft:structure_template_feature/structure_name/0|\"InvalidType\"|Some((130, 1))"]
"#;
