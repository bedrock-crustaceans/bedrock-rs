//! `WorldPacks::parse`: the world's `world_behavior_packs.json` and `world_resource_packs.json`.

use bedrock_addon::diagnostic::Code;
use bedrock_addon::document::Rejected;
use bedrock_addon::pack::WorldPacks;
use bedrock_addon::version::SemVersion;
use uuid::Uuid;

/// A behaviour list as vanilla writes it (jsoncpp's styled writer: tabs, `" : "`).
const BEHAVIOR: &str = "[\n\t{\n\t\t\"pack_id\" : \"5f9e8f0a-2c1e-4b5a-9d3e-1a2b3c4d5e6f\",\n\t\t\"version\" : [ 1, 0, 3 ]\n\t},\n\t{\n\t\t\"pack_id\" : \"0B1C2D3E-4F50-6172-8394-A5B6C7D8E9F0\",\n\t\t\"version\" : [ 2, 1, 0 ]\n\t}\n]\n";

/// A resource list with a subpack and a string version.
const RESOURCE: &str = r#"[
  { "pack_id": "11111111-2222-3333-4444-555555555555", "version": "1.2.0", "subpack": "high_res" }
]"#;

fn uuid(text: &str) -> Uuid {
    Uuid::parse_str(text).expect("a UUID")
}

#[test]
fn both_lists_parse_in_file_order() {
    let world = WorldPacks::parse(Some(BEHAVIOR.as_bytes()), Some(RESOURCE.as_bytes()))
        .expect("both lists parse");
    let behavior: Vec<(Uuid, SemVersion, Option<String>)> = world
        .behavior
        .iter()
        .map(|entry| (entry.pack_id, entry.version.clone(), entry.subpack.clone()))
        .collect();
    assert_eq!(
        behavior,
        [
            (
                uuid("5f9e8f0a-2c1e-4b5a-9d3e-1a2b3c4d5e6f"),
                SemVersion::new(1, 0, 3),
                None
            ),
            (
                uuid("0b1c2d3e-4f50-6172-8394-a5b6c7d8e9f0"),
                SemVersion::new(2, 1, 0),
                None
            ),
        ]
    );
    assert_eq!(world.resource.len(), 1);
    assert_eq!(
        world.resource[0].pack_id,
        uuid("11111111-2222-3333-4444-555555555555")
    );
    assert_eq!(world.resource[0].version, SemVersion::new(1, 2, 0));
    assert_eq!(world.resource[0].subpack.as_deref(), Some("high_res"));
}

#[test]
fn a_missing_file_is_an_empty_list() {
    let world = WorldPacks::parse(None, Some(RESOURCE.as_bytes())).expect("parses");
    assert!(world.behavior.is_empty());
    assert_eq!(world.resource.len(), 1);
    let neither = WorldPacks::parse(None, None).expect("parses");
    assert!(neither.behavior.is_empty() && neither.resource.is_empty());
}

#[test]
fn a_bom_and_comments_are_read_past() {
    let text = format!("\u{feff}// written by a tool\n{RESOURCE}");
    let world = WorldPacks::parse(None, Some(text.as_bytes())).expect("parses");
    assert_eq!(world.resource.len(), 1);
}

#[test]
fn an_empty_array_is_an_empty_list() {
    let world = WorldPacks::parse(Some(b"[]"), Some(b"[ ]\n")).expect("parses");
    assert!(world.behavior.is_empty() && world.resource.is_empty());
}

/// The one finding of a rejection, and the file it names.
fn rejection(rejected: &Rejected) -> (&Code, String, String) {
    let diagnostics: Vec<_> = rejected.diagnostics.iter().collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    let location = &diagnostics[0].location;
    (
        &diagnostics[0].code,
        location
            .path
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        location.json_path.to_string(),
    )
}

#[test]
fn malformed_json_is_rejected() {
    let rejected = WorldPacks::parse(Some(b"[ { \"pack_id\": "), None).expect_err("malformed");
    let (code, file, _) = rejection(&rejected);
    assert!(matches!(code.id(), "core/syntax"));
    assert_eq!(file, WorldPacks::BEHAVIOR_FILE);
    assert_eq!(rejected.origin.path.as_str(), WorldPacks::BEHAVIOR_FILE);
}

#[test]
fn a_root_that_is_not_an_array_is_rejected() {
    let rejected =
        WorldPacks::parse(None, Some(br#"{ "pack_id": "x" }"#)).expect_err("not an array");
    let (code, file, _) = rejection(&rejected);
    assert!(matches!(code.id(), "core/invalid_type"));
    assert_eq!(file, WorldPacks::RESOURCE_FILE);
}

#[test]
fn an_entry_without_a_version_is_rejected() {
    let rejected = WorldPacks::parse(
        Some(br#"[ { "pack_id": "11111111-2222-3333-4444-555555555555" } ]"#),
        None,
    )
    .expect_err("no version");
    let (code, _, json_path) = rejection(&rejected);
    assert!(matches!(code.id(), "core/missing_required_field"));
    assert_eq!(json_path, "0/version");
}

#[test]
fn an_entry_with_a_malformed_uuid_or_version_is_rejected_at_the_member() {
    let bad_uuid = WorldPacks::parse(
        Some(br#"[ { "pack_id": "not-a-uuid", "version": [1, 0, 0] } ]"#),
        None,
    )
    .expect_err("bad uuid");
    let (code, _, json_path) = rejection(&bad_uuid);
    assert!(matches!(code.id(), "core/invalid_value"));
    assert_eq!(json_path, "0/pack_id");
    let bad_version = WorldPacks::parse(
        None,
        Some(
            br#"[ { "pack_id": "11111111-2222-3333-4444-555555555555", "version": [1, 0] },
                 { "pack_id": "11111111-2222-3333-4444-555555555555", "version": [1, 0, 0] } ]"#,
        ),
    )
    .expect_err("bad version");
    let (code, _, json_path) = rejection(&bad_version);
    assert!(matches!(code.id(), "core/invalid_value"));
    assert_eq!(json_path, "0/version");
}

#[test]
fn a_subpack_that_is_not_a_string_is_rejected() {
    let rejected = WorldPacks::parse(
        None,
        Some(
            br#"[ { "pack_id": "11111111-2222-3333-4444-555555555555", "version": [1, 0, 0], "subpack": 3 } ]"#,
        ),
    )
    .expect_err("numeric subpack");
    let (code, _, json_path) = rejection(&rejected);
    assert!(matches!(code.id(), "core/invalid_type"));
    assert_eq!(json_path, "0/subpack");
}

#[test]
fn a_pack_id_takes_only_the_32_and_36_character_forms() {
    let list = |pack_id: &str| format!(r#"[ {{ "pack_id": "{pack_id}", "version": [1, 0, 0] }} ]"#);
    for accepted in [
        "11111111-2222-3333-4444-555555555555",
        "11111111222233334444555555555555",
        "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
    ] {
        let world = WorldPacks::parse(Some(list(accepted).as_bytes()), None)
            .unwrap_or_else(|rejected| panic!("{accepted}: {rejected:?}"));
        assert_eq!(world.behavior[0].pack_id, uuid(accepted));
    }
    for refused in [
        "{11111111-2222-3333-4444-555555555555}",
        "urn:uuid:11111111-2222-3333-4444-555555555555",
        "1111111-2222-3333-4444-5555555555555",
    ] {
        let rejected = WorldPacks::parse(Some(list(refused).as_bytes()), None).expect_err(refused);
        let (code, _, json_path) = rejection(&rejected);
        assert!(matches!(code.id(), "core/invalid_value"), "{refused}");
        assert_eq!(json_path, "0/pack_id");
    }
}

#[test]
fn a_pack_id_that_is_not_a_string_is_the_wrong_type() {
    let rejected = WorldPacks::parse(Some(br#"[ { "pack_id": 7, "version": [1, 0, 0] } ]"#), None)
        .expect_err("numeric pack_id");
    let (code, _, json_path) = rejection(&rejected);
    assert!(matches!(code.id(), "core/invalid_type"));
    assert_eq!(json_path, "0/pack_id");
}

#[test]
fn unknown_members_of_an_entry_are_ignored() {
    let world = WorldPacks::parse(
        None,
        Some(
            br#"[ { "pack_id": "11111111-2222-3333-4444-555555555555", "version": [1, 0, 0],
                    "priority": 3, "extra": { "a": [] } } ]"#,
        ),
    )
    .expect("unknown members are ignored");
    assert_eq!(world.resource.len(), 1);
    assert_eq!(world.resource[0].subpack, None);
}

#[test]
fn the_file_names_are_vanillas() {
    assert_eq!(WorldPacks::BEHAVIOR_FILE, "world_behavior_packs.json");
    assert_eq!(WorldPacks::RESOURCE_FILE, "world_resource_packs.json");
}
