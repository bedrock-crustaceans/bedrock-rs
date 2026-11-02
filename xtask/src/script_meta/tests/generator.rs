//! `cargo xtask script-meta` over a tiny hand-written tag: the catalog derivation (rules 1–7),
//! the vendoring checks (1–18) and the model and header checks. Each check gets one broken copy
//! of the tag.

use std::collections::BTreeMap;

use super::step::{Check, Finding, Generated, Level, Samples, VANILLA_DATA_FILES, generate};

/// A class declaration in the generator's shape.
fn class(name: &str) -> String {
    format!(
        r#"{{ "base_types": [], "constants": [], "functions": [], "name": "{name}", "properties": [],
            "type": {{ "is_bind_type": true, "is_errorable": false, "name": "{name}" }} }}"#
    )
}

/// A native module file with `classes` (JSON array items) and `peers` (JSON array items).
fn native(name: &str, uuid: &str, classes: &[String], peers: &str) -> String {
    format!(
        r#"{{ "classes": [{}], "constants": [], "dependencies": [], "enums": [], "errors": [], "functions": [],
            "interfaces": [], "module_type": "script", "name": "{name}", "objects": [],
            "peer_dependencies": [{peers}], "type_aliases": [], "uuid": "{uuid}" }}"#,
        classes.join(", ")
    )
}

/// A stub file.
fn stub(name: &str, uuid: &str, parent: &str, version: &str, wrapper: &str, peers: &str) -> String {
    format!(
        r#"{{ "module_type": "script", "name": "{name}", "parentModule": {{ "name": "{parent}", "version": "{version}" }},
            "peer_dependencies": [{peers}], "uuid": "{uuid}", "wrapper_module": "{wrapper}" }}"#
    )
}

const COMMON: &str = "77ec12b4-1b2b-4c98-8d34-d1cd63f849d5";
const BINDINGS: &str = "0196a2fa-8fb5-79a7-b29e-ce3cbc644f82";
const SERVER: &str = "b26a4d4c-afdf-4690-88f8-931846312678";
const UI_BINDINGS: &str = "ceded7ee-4d9d-4bcc-badf-0ecddb8e238a";
const UI: &str = "2BD50A27-AB5F-4F40-A596-3641627C635E";

const PEER_COMMON: &str = r#"{ "name": "@minecraft/common", "uuid": "77ec12b4-1b2b-4c98-8d34-d1cd63f849d5",
    "versions": [ { "version": "1.0.0" } ] }"#;
const PEER_VANILLA_DATA: &str = r#"{ "name": "@minecraft/vanilla-data", "types_only": true,
    "version": "1.20.70", "versions": [ { "version": "1.20.70" } ] }"#;

/// A whole tag, file by file, so a test can break one file.
struct Tag {
    script_modules: BTreeMap<String, String>,
    after_events: String,
    vanilla_data: BTreeMap<String, String>,
    commands: String,
    molang_queries: String,
    json_schemas: BTreeMap<String, String>,
}

impl Tag {
    fn samples(&self) -> Samples {
        let bytes = |map: &BTreeMap<String, String>| {
            map.iter()
                .map(|(name, text)| (name.clone(), text.clone().into_bytes()))
                .collect()
        };
        Samples {
            tag: "v0.0.1".to_owned(),
            script_modules: bytes(&self.script_modules),
            after_events: self.after_events.clone().into_bytes(),
            vanilla_data: bytes(&self.vanilla_data),
            commands: self.commands.clone().into_bytes(),
            molang_queries: self.molang_queries.clone().into_bytes(),
            json_schemas: bytes(&self.json_schemas),
        }
    }

    fn generate(&self) -> Result<Generated, Vec<Finding>> {
        generate(&self.samples())
    }

    fn module(&mut self, file: &str) -> &mut String {
        self.script_modules.get_mut(file).unwrap()
    }
}

/// `common` 1.0.0; `server-bindings` 1.0.0 and 2.0.0-beta; the `server` stubs over them;
/// `server-ui-bindings` 1.0.0 and its `server-ui` stub, whose own peer carries a `wrapperName`.
fn tiny_tag() -> Tag {
    let bindings_peers = format!("{PEER_COMMON}, {PEER_VANILLA_DATA}");
    let script_modules = [
        ("common_1.0.0.json", native("@minecraft/common", COMMON, &[], "")),
        (
            "server-bindings_1.0.0.json",
            native(
                "@minecraft/server-bindings",
                BINDINGS,
                &[class("World"), class("ButtonPushAfterEvent")],
                PEER_COMMON,
            ),
        ),
        (
            "server-bindings_2.0.0-beta.json",
            native(
                "@minecraft/server-bindings",
                BINDINGS,
                &[class("World")],
                &bindings_peers,
            ),
        ),
        (
            "server_1.0.0.json",
            stub(
                "@minecraft/server",
                SERVER,
                "@minecraft/server-bindings",
                "1.0.0",
                "@minecraft/server-bindings-v1-wrapper",
                "",
            ),
        ),
        (
            "server_2.0.0-beta.json",
            stub(
                "@minecraft/server",
                SERVER,
                "@minecraft/server-bindings",
                "2.0.0-beta",
                "@minecraft/server-bindings-wrapper",
                "",
            ),
        ),
        (
            "server-ui-bindings_1.0.0.json",
            native(
                "@minecraft/server-ui-bindings",
                UI_BINDINGS,
                &[],
                &format!(
                    r#"{PEER_COMMON}, {{ "name": "@minecraft/server", "uuid": "{SERVER}", "versions": [ {{ "version": "1.0.0" }} ] }}"#
                ),
            ),
        ),
        (
            "server-ui_1.0.0.json",
            stub(
                "@minecraft/server-ui",
                UI,
                "@minecraft/server-ui-bindings",
                "1.0.0",
                "@minecraft/server-ui-bindings-v1-wrapper",
                r#"{ "name": "@minecraft/server", "versions": [ { "version": "1.0.0" } ], "wrapperName": "@minecraft/server-wrapper" }"#,
            ),
        ),
    ]
    .into_iter()
    .map(|(file, text)| (file.to_owned(), text))
    .collect();
    let after_events = r#"{ "after_events_order_by_version": [
        { "event_order": [ { "name": "ButtonPushAfterEvent" } ], "name": "@minecraft/server 1.0.0", "version": "1.0.0" },
        { "event_order": [ { "name": "ButtonPushAfterEvent" } ], "name": "@minecraft/server 3.0.0-alpha", "version": "3.0.0-alpha" } ],
      "module_type": "after_events_ordering", "name": "engine-after_events_ordering" }"#
        .to_owned();
    let vanilla_data = VANILLA_DATA_FILES
        .iter()
        .map(|(file, ty)| {
            let items = match *ty {
                "block" => r#"{ "name": "minecraft:stone", "properties": [ { "name": "stone_type" } ], "raw_id": 1, "serialization_id": "tile.stone" }"#.to_owned(),
                "item" => r#"{ "command_name": "minecraft:stick", "name": "minecraft:stick", "raw_id": 320, "serialization_id": "item.stick", "serialization_name": "minecraft:stick" }"#.to_owned(),
                "biome" => r#"{ "id": 1, "name": "minecraft:plains" }"#.to_owned(),
                "enchantment" => r#"{ "name": "minecraft:infinity" }, { "name": "minecraft:bow_infinity", "value": "minecraft:infinity" }"#.to_owned(),
                "potionDelivery" => r#"{ "name": "Consume" }"#.to_owned(),
                other => format!(r#"{{ "name": "minecraft:{}" }}"#, other.to_ascii_lowercase()),
            };
            let properties = if *ty == "block" {
                r#""block_properties": [ { "name": "stone_type", "type": "string", "values": [ { "value": "granite" } ] } ],"#
            } else {
                ""
            };
            let text = format!(
                r#"{{ {properties} "data_items": [ {items} ], "module_type": "vanilla_data", "name": "mojang-{ty}", "vanilla_data_type": "{ty}" }}"#
            );
            ((*file).to_owned(), text)
        })
        .collect();
    let commands = r#"{ "command_enums": [ { "name": "GameMode", "values": [ { "value": "survival" } ] } ],
      "commands": [
        { "aliases": [ { "name": "gm" } ], "description": "Sets a game mode.", "name": "gamemode",
          "overloads": [ { "name": "1", "params": [
            { "is_optional": false, "name": "gameMode", "type": { "name": "GAMEMODE" } },
            { "is_optional": true, "name": "player", "type": { "name": "SELECTION" } } ] } ],
          "permission_level": 1, "requires_cheats": true },
        { "aliases": [], "description": "Lists players.", "name": "list",
          "overloads": [ { "name": "1", "params": [] } ], "permission_level": 0, "requires_cheats": false } ],
      "module_type": "commands", "name": "mojang-commands" }"#
        .to_owned();
    let molang_queries = r#"{ "math_functions": [], "module_type": "molang", "name": "engine-molang", "queries": [] }"#.to_owned();
    let json_schemas = [
        (
            "server/block/1.26.0/Block.json",
            r##"{ "properties": { "a": { "$ref": "Block%20Part.json#/definitions/x" }, "b": { "$ref": "#/definitions/y" } } }"##,
        ),
        ("server/block/1.26.0/Block Part.json", r#"{ "definitions": {} }"#),
    ]
    .into_iter()
    .map(|(path, text)| (path.to_owned(), text.to_owned()))
    .collect();
    Tag {
        script_modules,
        after_events,
        vanilla_data,
        commands,
        molang_queries,
        json_schemas,
    }
}

/// The checks that failed, for a tag that must not generate.
fn failed(tag: &Tag) -> Vec<Check> {
    match tag.generate() {
        Ok(_) => panic!("the tag generated"),
        Err(errors) => errors.iter().map(|finding| finding.check).collect(),
    }
}

fn report(tag: &Tag) -> Vec<Finding> {
    match tag.generate() {
        Ok(generated) => generated.report,
        Err(errors) => panic!("the tag failed: {errors:#?}"),
    }
}

/// The unformatted `versions.rs` line of the version `major.minor.patch[-pre]` of `module`.
fn version_line<'a>(generated: &'a Generated, module: &str, version: &str) -> &'a str {
    let entry = generated
        .versions
        .split("ModuleEntry {")
        .find(|entry| entry.contains(&format!("name: \"{module}\",")))
        .unwrap_or_else(|| panic!("{module} is not in the table"));
    let (numbers, pre) = version
        .split_once('-')
        .map_or((version, None), |(n, p)| (n, Some(p)));
    let mut numbers = numbers.split('.');
    let wanted = format!(
        "ModuleVersion {{ major: {}, minor: {}, patch: {}, pre: {},",
        numbers.next().unwrap(),
        numbers.next().unwrap(),
        numbers.next().unwrap(),
        pre.map_or_else(|| "None".to_owned(), |p| format!("Some(\"{p}\")"))
    );
    entry
        .lines()
        .find(|line| line.starts_with(&wanted))
        .unwrap_or_else(|| panic!("{module} {version} is not in the table"))
}

#[test]
fn the_tiny_tag_generates() {
    let generated = tiny_tag().generate().unwrap();
    assert!(
        generated
            .versions
            .contains("SAMPLES_TAG: &str = \"v0.0.1\"")
    );
    assert!(
        generated
            .versions
            .contains("from\n// the game's script module metadata")
    );
}

#[test]
fn rule_1_one_entry_per_module_sorted_by_name() {
    let generated = tiny_tag().generate().unwrap();
    let names: Vec<&str> = generated
        .versions
        .split("ModuleEntry { name: \"")
        .skip(1)
        .map(|entry| entry.split('"').next().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "@minecraft/common",
            "@minecraft/server",
            "@minecraft/server-bindings",
            "@minecraft/server-ui",
            "@minecraft/server-ui-bindings",
        ]
    );
}

#[test]
fn rule_2_the_shape_comes_from_the_discriminator_and_agrees_across_files() {
    let generated = tiny_tag().generate().unwrap();
    assert!(generated.versions.contains(
        "name: \"@minecraft/server\", uuid: 0xb26a_4d4c_afdf_4690_88f8_9318_4631_2678, // b26a4d4c-afdf-4690-88f8-931846312678\nshape: ModuleShape::Stub { parent: \"@minecraft/server-bindings\" }"
    ));
    assert!(
        generated
            .versions
            .contains("// 77ec12b4-1b2b-4c98-8d34-d1cd63f849d5\nshape: ModuleShape::Native")
    );
    // A server version that is a native module: the module's files disagree.
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "server_1.1.0.json".to_owned(),
        native("@minecraft/server", SERVER, &[], ""),
    );
    assert!(failed(&tag).contains(&Check::Catalog));
}

#[test]
fn rule_3_a_stub_version_points_at_its_parents_version() {
    let generated = tiny_tag().generate().unwrap();
    assert!(
        version_line(&generated, "@minecraft/server", "1.0.0").contains("parent_version: Some(0)")
    );
    assert!(
        version_line(&generated, "@minecraft/server", "2.0.0-beta")
            .contains("parent_version: Some(1)")
    );
    assert!(
        version_line(&generated, "@minecraft/server-bindings", "1.0.0")
            .contains("parent_version: None")
    );
}

#[test]
fn rule_4_a_native_version_keeps_its_own_peers() {
    let generated = tiny_tag().generate().unwrap();
    assert!(
        version_line(&generated, "@minecraft/server-bindings", "1.0.0")
            .contains("peers: &[P_COMMON_1_0_0]")
    );
}

#[test]
fn rule_5_a_stub_takes_its_parents_peers_with_its_own_replacing_them() {
    let generated = tiny_tag().generate().unwrap();
    // `server` stubs have no peers of their own: the bindings' peers.
    assert!(
        version_line(&generated, "@minecraft/server", "2.0.0-beta")
            .contains("peers: &[P_COMMON_1_0_0, P_VANILLA_DATA_1_20_70]")
    );
    // `server-ui` replaces the bindings' `server` peer with its own, keeping `wrapperName`.
    assert!(
        version_line(&generated, "@minecraft/server-ui", "1.0.0")
            .contains("peers: &[P_COMMON_1_0_0, P_SERVER_1_0_0_WRAPPED]")
    );
    assert!(
        version_line(&generated, "@minecraft/server-ui-bindings", "1.0.0")
            .contains("peers: &[P_COMMON_1_0_0, P_SERVER_1_0_0]")
    );
    assert!(generated.versions.contains(
        "const P_SERVER_1_0_0_WRAPPED: PeerReq = PeerReq { module: \"@minecraft/server\", versions: &[\"1.0.0\"], types_only: false, wrapper_name: Some(\"@minecraft/server-wrapper\") };"
    ));
}

#[test]
fn rule_6_from_module_does_not_feed_the_peers() {
    let mut tag = tiny_tag();
    let foreign = r#"{ "base_types": [], "constants": [], "functions": [], "name": "Thing", "properties": [ { "get_privilege": [ { "name": "default" } ],
        "is_baked": false, "is_read_only": true, "name": "world", "type": { "from_module": { "name": "@minecraft/server-bindings",
        "uuid": "0196a2fa-8fb5-79a7-b29e-ce3cbc644f82", "version": "1.0.0" }, "is_bind_type": true, "is_errorable": false, "name": "World" } } ],
        "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" } }"#;
    *tag.module("server-ui-bindings_1.0.0.json") = native(
        "@minecraft/server-ui-bindings",
        UI_BINDINGS,
        &[foreign.to_owned()],
        &format!(
            r#"{PEER_COMMON}, {{ "name": "@minecraft/server", "uuid": "{SERVER}", "versions": [ {{ "version": "1.0.0" }} ] }}"#
        ),
    );
    let generated = tag.generate().unwrap();
    assert!(
        version_line(&generated, "@minecraft/server-ui-bindings", "1.0.0")
            .contains("peers: &[P_COMMON_1_0_0, P_SERVER_1_0_0]")
    );
}

#[test]
fn rule_7_types_only_peers_stay_in_the_table() {
    let generated = tiny_tag().generate().unwrap();
    assert!(generated.versions.contains(
        "const P_VANILLA_DATA_1_20_70: PeerReq = PeerReq { module: \"@minecraft/vanilla-data\", versions: &[\"1.20.70\"], types_only: true, wrapper_name: None };"
    ));
}

#[test]
fn a_beta_file_is_the_beta_channel_and_sorts_last() {
    let generated = tiny_tag().generate().unwrap();
    assert!(
        version_line(&generated, "@minecraft/server-bindings", "2.0.0-beta")
            .contains("channel: Channel::Beta")
    );
    assert!(
        version_line(&generated, "@minecraft/server-bindings", "1.0.0")
            .contains("channel: Channel::Stable")
    );
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "common_1.10.0.json".to_owned(),
        native("@minecraft/common", COMMON, &[], ""),
    );
    tag.script_modules.insert(
        "common_1.2.0.json".to_owned(),
        native("@minecraft/common", COMMON, &[], ""),
    );
    let generated = tag.generate().unwrap();
    let common = generated
        .versions
        .split("ModuleEntry {")
        .find(|e| e.contains("name: \"@minecraft/common\","))
        .unwrap();
    let minors: Vec<&str> = common
        .split("minor: ")
        .skip(1)
        .map(|rest| rest.split(',').next().unwrap())
        .collect();
    assert_eq!(
        minors,
        ["0", "2", "10"],
        "SemVersion order, not file-name order"
    );
}

#[test]
fn check_15_engine_only_versions_are_listed() {
    let tag = tiny_tag();
    let generated = tag.generate().unwrap();
    assert!(
        generated
            .versions
            .contains("(\"@minecraft/server\", \"3.0.0-alpha\"),")
    );
    assert!(
        generated
            .report
            .iter()
            .any(|f| f.check == Check::EngineOnly && f.level == Level::Info)
    );
}

#[test]
fn model_an_unknown_key_fails_the_run() {
    let mut tag = tiny_tag();
    let text = tag.module("common_1.0.0.json");
    *text = text.replacen("\"module_type\"", "\"brand_new\": 1, \"module_type\"", 1);
    // The unread file is missing for the checks after it (its peers are not shipped).
    assert_eq!(failed(&tag)[0], Check::Model);
}

#[test]
fn model_a_descriptor_that_does_not_lower_fails_the_run() {
    let mut tag = tiny_tag();
    let broken = r#"{ "base_types": [], "constants": [], "functions": [], "name": "Bad", "properties": [ { "get_privilege": [ { "name": "default" } ],
        "is_baked": false, "is_read_only": true, "name": "p", "type": { "is_bind_type": false, "is_errorable": false, "name": "optional" } } ],
        "type": { "is_bind_type": true, "is_errorable": false, "name": "Bad" } }"#;
    *tag.module("common_1.0.0.json") =
        native("@minecraft/common", COMMON, &[broken.to_owned()], "");
    assert_eq!(failed(&tag), [Check::Model]);
}

#[test]
fn model_a_privilege_list_out_of_order_fails_the_run() {
    let mut tag = tiny_tag();
    let broken = r#"{ "base_types": [], "constants": [], "functions": [ { "arguments": [],
        "call_privilege": [ { "name": "early_execution" }, { "name": "restricted_execution" } ], "is_constructor": false,
        "is_static": false, "name": "f", "return_type": { "is_bind_type": false, "is_errorable": false, "name": "undefined" } } ],
        "name": "Bad", "properties": [], "type": { "is_bind_type": true, "is_errorable": false, "name": "Bad" } }"#;
    *tag.module("common_1.0.0.json") =
        native("@minecraft/common", COMMON, &[broken.to_owned()], "");
    assert_eq!(failed(&tag), [Check::Model]);
}

#[test]
fn check_1_the_file_name_matches_name() {
    let mut tag = tiny_tag();
    let text = tag.script_modules.remove("common_1.0.0.json").unwrap();
    tag.script_modules
        .insert("commons_1.0.0.json".to_owned(), text);
    assert!(failed(&tag).contains(&Check::FileName));
}

#[test]
fn check_2_a_stub_parent_is_shipped_with_the_stubs_version() {
    // A shipped parent version that is not the stub's own.
    let mut tag = tiny_tag();
    let text = tag.module("server_2.0.0-beta.json");
    *text = text.replace("\"version\": \"2.0.0-beta\"", "\"version\": \"1.0.0\"");
    assert_eq!(failed(&tag), [Check::Parent]);
    // The stub's own version, which the parent does not ship.
    let mut tag = tiny_tag();
    let text = tag.script_modules["server_1.0.0.json"].clone();
    tag.script_modules.insert(
        "server_1.5.0.json".to_owned(),
        text.replace("\"version\": \"1.0.0\"", "\"version\": \"1.5.0\""),
    );
    assert_eq!(failed(&tag), [Check::Parent]);
}

#[test]
fn check_3_declarations_on_a_stub_warn() {
    let mut tag = tiny_tag();
    let text = tag.module("server_1.0.0.json");
    *text = text.replacen("\"module_type\"", "\"enums\": [], \"module_type\"", 1);
    let report = report(&tag);
    assert!(
        report
            .iter()
            .any(|f| f.check == Check::StubDeclarations && f.level == Level::Warning)
    );
}

#[test]
fn check_4_from_module_names_a_shipped_declaring_file_with_its_uuid() {
    let with_from = |module: &str, uuid: &str, version: &str, type_name: &str| {
        let property = format!(
            r#"{{ "base_types": [], "constants": [], "functions": [], "name": "Thing", "properties": [ {{ "get_privilege": [ {{ "name": "default" }} ],
            "is_baked": false, "is_read_only": true, "name": "p", "type": {{ "from_module": {{ "name": "{module}", "uuid": "{uuid}",
            "version": "{version}" }}, "is_bind_type": true, "is_errorable": false, "name": "{type_name}" }} }} ],
            "type": {{ "is_bind_type": true, "is_errorable": false, "name": "Thing" }} }}"#
        );
        let mut tag = tiny_tag();
        *tag.module("common_1.0.0.json") = native("@minecraft/common", COMMON, &[property], "");
        tag
    };
    // Upper case UUID: compared without case.
    let good = with_from(
        "@minecraft/server-bindings",
        &BINDINGS.to_ascii_uppercase(),
        "1.0.0",
        "World",
    );
    assert!(good.generate().is_ok());
    let unshipped = with_from("@minecraft/server-bindings", BINDINGS, "9.0.0", "World");
    assert_eq!(failed(&unshipped), [Check::FromModule]);
    let undeclared = with_from("@minecraft/server-bindings", BINDINGS, "1.0.0", "Nothing");
    assert_eq!(failed(&undeclared), [Check::FromModule]);
    let wrong_uuid = with_from("@minecraft/server-bindings", COMMON, "1.0.0", "World");
    assert_eq!(failed(&wrong_uuid), [Check::FromModule]);
}

#[test]
fn check_5_an_unqualified_bind_type_is_declared_in_the_module() {
    let mut tag = tiny_tag();
    let broken = r#"{ "base_types": [ { "is_bind_type": true, "is_errorable": false, "name": "Missing" } ], "constants": [],
        "functions": [], "name": "Thing", "properties": [], "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" } }"#;
    *tag.module("common_1.0.0.json") =
        native("@minecraft/common", COMMON, &[broken.to_owned()], "");
    assert_eq!(failed(&tag), [Check::Unresolved]);
}

#[test]
fn check_5_an_unqualified_member_type_is_declared_in_the_module() {
    let with_members = |property_type: &str, argument_type: &str| {
        let class = format!(
            r#"{{ "base_types": [], "constants": [], "functions": [ {{ "arguments": [ {{ "details": null, "name": "a",
            "type": {argument_type} }} ], "call_privilege": [ {{ "name": "default" }} ], "is_constructor": false,
            "is_static": false, "name": "f", "return_type": {{ "is_bind_type": false, "is_errorable": false, "name": "undefined" }} }} ],
            "name": "Thing", "properties": [ {{ "get_privilege": [ {{ "name": "default" }} ], "is_baked": false,
            "is_read_only": true, "name": "p", "type": {property_type} }} ],
            "type": {{ "is_bind_type": true, "is_errorable": false, "name": "Thing" }} }}"#
        );
        let mut tag = tiny_tag();
        *tag.module("common_1.0.0.json") = native("@minecraft/common", COMMON, &[class], "");
        tag
    };
    let declared = r#"{ "is_bind_type": true, "is_errorable": false, "name": "Thing" }"#;
    let missing = r#"{ "is_bind_type": true, "is_errorable": false, "name": "Missing" }"#;
    let optional = |inner: &str| {
        format!(
            r#"{{ "is_bind_type": false, "is_errorable": false, "name": "optional", "optional_type": {inner} }}"#
        )
    };
    assert!(
        with_members(declared, &optional(declared))
            .generate()
            .is_ok()
    );
    // A property's type.
    assert_eq!(
        failed(&with_members(missing, &optional(declared))),
        [Check::Unresolved]
    );
    // An argument's type, nested in a composite.
    assert_eq!(
        failed(&with_members(declared, &optional(missing))),
        [Check::Unresolved]
    );
}

#[test]
fn check_6_no_duplicate_declarations_or_members() {
    let mut tag = tiny_tag();
    *tag.module("common_1.0.0.json") = native(
        "@minecraft/common",
        COMMON,
        &[class("Thing"), class("Thing")],
        "",
    );
    assert_eq!(failed(&tag), [Check::Duplicate]);
    let mut tag = tiny_tag();
    let doubled = r#"{ "base_types": [], "constants": [], "functions": [], "name": "Thing", "properties": [
        { "get_privilege": [ { "name": "default" } ], "is_baked": false, "is_read_only": true, "name": "p",
          "type": { "is_bind_type": false, "is_errorable": false, "name": "string" } },
        { "get_privilege": [ { "name": "default" } ], "is_baked": false, "is_read_only": true, "name": "p",
          "type": { "is_bind_type": false, "is_errorable": false, "name": "string" } } ],
        "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" } }"#;
    *tag.module("common_1.0.0.json") =
        native("@minecraft/common", COMMON, &[doubled.to_owned()], "");
    assert_eq!(failed(&tag), [Check::Duplicate]);
}

#[test]
fn check_7_every_peer_version_is_shipped_but_vanilla_data() {
    let mut tag = tiny_tag();
    let text = tag.module("server-bindings_1.0.0.json");
    *text = text.replace(
        "\"versions\": [ { \"version\": \"1.0.0\" } ]",
        "\"versions\": [ { \"version\": \"1.1.0\" } ]",
    );
    assert_eq!(failed(&tag), [Check::PeerShipped]);
}

#[test]
fn check_8_error_types_need_is_errorable() {
    let mut tag = tiny_tag();
    let broken = r#"{ "base_types": [], "constants": [], "functions": [], "name": "Thing", "properties": [ { "get_privilege": [ { "name": "default" } ],
        "is_baked": false, "is_read_only": true, "name": "p", "type": { "error_types": [], "is_bind_type": false, "is_errorable": true,
        "name": "string" } } ], "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" } }"#;
    *tag.module("common_1.0.0.json") =
        native("@minecraft/common", COMMON, &[broken.to_owned()], "");
    assert_eq!(failed(&tag), [Check::ErrorTypes]);
}

#[test]
fn check_9_argument_bounds_are_ordered_and_exact() {
    let with_details = |details: &str| {
        let function = format!(
            r#"{{ "base_types": [], "constants": [], "functions": [ {{ "arguments": [ {{ "details": {details}, "name": "a",
            "type": {{ "is_bind_type": false, "is_errorable": false, "name": "float" }} }} ],
            "call_privilege": [ {{ "name": "default" }} ], "is_constructor": false, "is_static": false, "name": "f",
            "return_type": {{ "is_bind_type": false, "is_errorable": false, "name": "undefined" }} }} ],
            "name": "Thing", "properties": [], "type": {{ "is_bind_type": true, "is_errorable": false, "name": "Thing" }} }}"#
        );
        let mut tag = tiny_tag();
        *tag.module("common_1.0.0.json") = native("@minecraft/common", COMMON, &[function], "");
        tag
    };
    assert!(
        with_details(r#"{ "max_value": 15, "min_value": 0 }"#)
            .generate()
            .is_ok()
    );
    assert_eq!(
        failed(&with_details(r#"{ "max_value": 0, "min_value": 15 }"#)),
        [Check::DetailBounds]
    );
    assert_eq!(
        failed(&with_details(r#"{ "max_value": 9007199254740993 }"#)),
        [Check::DetailBounds]
    );
}

#[test]
fn check_10_one_beta_per_module_and_no_other_pre_release() {
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "common_1.1.0-beta.json".to_owned(),
        native("@minecraft/common", COMMON, &[], ""),
    );
    tag.script_modules.insert(
        "common_1.2.0-beta.json".to_owned(),
        native("@minecraft/common", COMMON, &[], ""),
    );
    assert_eq!(failed(&tag), [Check::PreRelease]);
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "common_1.1.0-rc.json".to_owned(),
        native("@minecraft/common", COMMON, &[], ""),
    );
    assert_eq!(failed(&tag), [Check::PreRelease]);
}

#[test]
fn check_11_one_uuid_per_module() {
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "common_1.1.0.json".to_owned(),
        native("@minecraft/common", BINDINGS, &[], ""),
    );
    assert_eq!(failed(&tag), [Check::Uuid]);
}

#[test]
fn checks_12_and_13_from_module_outside_the_peers_is_information() {
    // `server-ui-bindings` peers `server` 1.0.0; a type from `server-bindings` (not a peer) and
    // one from `common` 1.0.0 at another version would both be information only.
    let mut tag = tiny_tag();
    tag.script_modules.insert(
        "common_1.1.0.json".to_owned(),
        native("@minecraft/common", COMMON, &[class("Vector")], ""),
    );
    let from = |module: &str, uuid: &str, version: &str, name: &str| {
        format!(
            r#"{{ "get_privilege": [ {{ "name": "default" }} ], "is_baked": false, "is_read_only": true, "name": "p{name}",
            "type": {{ "from_module": {{ "name": "{module}", "uuid": "{uuid}", "version": "{version}" }},
            "is_bind_type": true, "is_errorable": false, "name": "{name}" }} }}"#
        )
    };
    let class = format!(
        r#"{{ "base_types": [], "constants": [], "functions": [], "name": "Form", "properties": [ {}, {} ],
        "type": {{ "is_bind_type": true, "is_errorable": false, "name": "Form" }} }}"#,
        from("@minecraft/server-bindings", BINDINGS, "1.0.0", "World"),
        from("@minecraft/common", COMMON, "1.1.0", "Vector"),
    );
    *tag.module("server-ui-bindings_1.0.0.json") = native(
        "@minecraft/server-ui-bindings",
        UI_BINDINGS,
        &[class],
        &format!(
            r#"{PEER_COMMON}, {{ "name": "@minecraft/server", "uuid": "{SERVER}", "versions": [ {{ "version": "1.0.0" }} ] }}"#
        ),
    );
    let report = report(&tag);
    assert!(
        report
            .iter()
            .any(|f| f.check == Check::FromModulePeer && f.level == Level::Info)
    );
    assert!(
        report
            .iter()
            .any(|f| f.check == Check::FromModuleVersion && f.level == Level::Info)
    );
}

#[test]
fn check_14_after_events_are_classes_of_the_bindings_file() {
    let mut tag = tiny_tag();
    tag.after_events =
        tag.after_events
            .replacen("ButtonPushAfterEvent", "LeverActionAfterEvent", 1);
    assert_eq!(failed(&tag), [Check::AfterEvents]);
}

#[test]
fn check_16_block_states_name_a_block_property() {
    let mut tag = tiny_tag();
    let blocks = tag.vanilla_data.get_mut("mojang-blocks.json").unwrap();
    *blocks = blocks.replacen(
        "{ \"name\": \"stone_type\" }",
        "{ \"name\": \"colour\" }",
        1,
    );
    assert!(failed(&tag).contains(&Check::BlockProperties));
}

#[test]
fn check_16_unused_block_properties_are_information() {
    let mut tag = tiny_tag();
    let blocks = tag.vanilla_data.get_mut("mojang-blocks.json").unwrap();
    *blocks = blocks.replacen(
        "\"block_properties\": [",
        "\"block_properties\": [ { \"name\": \"age\", \"type\": \"int\", \"values\": [ { \"value\": 0 } ] },",
        1,
    );
    assert!(
        report(&tag)
            .iter()
            .any(|f| f.check == Check::BlockProperties && f.level == Level::Info)
    );
}

#[test]
fn check_17_command_parameter_types_are_enums_or_tokens() {
    let mut tag = tiny_tag();
    tag.commands = tag.commands.replacen("\"SELECTION\"", "\"SELECTOR\"", 1);
    assert_eq!(failed(&tag), [Check::CommandTypes]);
    let mut tag = tiny_tag();
    tag.commands = tag.commands.replacen(
        "\"command_enums\": [",
        "\"command_enums\": [ { \"name\": \"Unused\", \"values\": [] },",
        1,
    );
    assert!(
        report(&tag)
            .iter()
            .any(|f| f.check == Check::CommandTypes && f.level == Level::Info)
    );
}

#[test]
fn check_18_schema_refs_resolve_after_percent_decoding() {
    let tag = tiny_tag();
    let found = report(&tag)
        .into_iter()
        .find(|f| f.check == Check::SchemaRefs)
        .unwrap();
    assert_eq!(found.level, Level::Info);
    assert!(found.detail.starts_with("0 of 2 "), "{}", found.detail);
    let mut tag = tiny_tag();
    tag.json_schemas
        .remove("server/block/1.26.0/Block Part.json");
    let found = report(&tag)
        .into_iter()
        .find(|f| f.check == Check::SchemaRefs)
        .unwrap();
    assert!(found.detail.starts_with("1 of 2 "), "{}", found.detail);
}

#[test]
fn headers_vanilla_data_type_matches_the_file() {
    let mut tag = tiny_tag();
    let effects = tag.vanilla_data.get_mut("mojang-effects.json").unwrap();
    *effects = effects.replace("\"effect\"", "\"potionEffect\"");
    assert!(failed(&tag).contains(&Check::Headers));
}

#[test]
fn headers_block_properties_only_on_blocks_and_typed() {
    let mut tag = tiny_tag();
    let items = tag.vanilla_data.get_mut("mojang-items.json").unwrap();
    *items = items.replacen(
        "\"data_items\"",
        "\"block_properties\": [], \"data_items\"",
        1,
    );
    assert_eq!(failed(&tag), [Check::Headers]);
    let mut tag = tiny_tag();
    let blocks = tag.vanilla_data.get_mut("mojang-blocks.json").unwrap();
    *blocks = blocks.replacen("\"value\": \"granite\"", "\"value\": 3", 1);
    assert_eq!(failed(&tag), [Check::Headers]);
}

#[test]
fn headers_a_missing_or_unknown_vanilla_data_file_fails() {
    let mut tag = tiny_tag();
    tag.vanilla_data.remove("mojang-features.json");
    assert!(failed(&tag).contains(&Check::Headers));
    let mut tag = tiny_tag();
    tag.vanilla_data.insert(
        "mojang-fogs.json".to_owned(),
        r#"{ "data_items": [], "module_type": "vanilla_data", "name": "mojang-fog", "vanilla_data_type": "fog" }"#
            .to_owned(),
    );
    assert_eq!(failed(&tag), [Check::Headers]);
}

#[test]
fn headers_unseen_data_is_information() {
    // A `dependencies` entry and a string module constant parse, and are reported.
    let mut tag = tiny_tag();
    let text = tag.module("common_1.0.0.json");
    *text = text
        .replacen(
            "\"dependencies\": []",
            "\"dependencies\": [ { \"name\": \"@minecraft/x\" } ]",
            1,
        )
        .replacen(
            "\"constants\": []",
            r#""constants": [ { "is_read_only": true, "is_static": true, "name": "Name",
                "type": { "is_bind_type": false, "is_errorable": false, "name": "string" }, "value": "text" } ]"#,
            1,
        );
    let infos: Vec<Check> = report(&tag)
        .iter()
        .filter(|f| f.check == Check::Headers)
        .map(|f| f.check)
        .collect();
    assert_eq!(infos.len(), 2);
}

#[test]
fn the_vanilla_tables_render_lists_in_file_order_and_commands_sorted() {
    let generated = tiny_tag().generate().unwrap();
    // The file's blocks, then the blocks the game registers and the file leaves out.
    let unlisted: Vec<String> = super::step::UNLISTED_BLOCKS
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect();
    assert_eq!(unlisted.len(), 14);
    // Then the names of the game's list the two leave out, in its order.
    let mut blocks = vec!["\"minecraft:stone\"".to_owned()];
    blocks.extend(unlisted.iter().cloned());
    for name in super::step::game::BLOCKS.lines() {
        let quoted = format!("\"{name}\"");
        if !blocks.contains(&quoted) {
            blocks.push(quoted);
        }
    }
    assert!(
        generated
            .vanilla_ids
            .contains(&format!("blocks: &[{}],", blocks.join(", ")))
    );
    assert!(unlisted.contains(&"\"minecraft:end_gateway\"".to_owned()));
    // The items and potions of the game's lists the metadata lacks follow theirs.
    for name in [
        "\"minecraft:element_0\"",
        "\"minecraft:frostbite_spawn_egg\"",
        "\"minecraft:fire\"",
        "\"minecraft:freezing\"",
    ] {
        assert!(generated.vanilla_ids.contains(name), "{name}");
    }
    assert!(
        generated
            .vanilla_ids
            .contains("enchantments: &[\"minecraft:infinity\", \"minecraft:bow_infinity\"],")
    );
    assert!(
        generated.vanilla_ids.contains(
            "enchantment_aliases: &[(\"minecraft:bow_infinity\", \"minecraft:infinity\")],"
        )
    );
    assert!(
        generated
            .vanilla_ids
            .contains("potion_types: &[\"Consume\"],")
    );
    let gamemode = generated.commands.find("name: \"gamemode\"").unwrap();
    let list = generated.commands.find("name: \"list\"").unwrap();
    assert!(gamemode < list);
    assert!(
        generated
            .commands
            .contains("Param { name: \"gameMode\", optional: false, type_name: \"GAMEMODE\" },")
    );
    assert!(
        generated
            .commands
            .contains("aliases: &[\"gm\"], permission_level: 1, requires_cheats: true")
    );
    assert!(
        generated
            .commands
            .contains("CommandEnum { name: \"GameMode\", values: &[\"survival\"] },")
    );
    // Descriptions are Mojang text and are not generated.
    assert!(!generated.commands.contains("Sets a game mode."));
}
