//! `ModuleMetadata::parse`: the strict read and the native / stub discriminator
//!. The documents are hand-written in the
//! generator's shapes, abbreviated.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it and serializes a value to compare it with the written text; not yet moved to the door"
)]

use bedrock_addon::families::script::meta::model::script::ConstValue;
use bedrock_addon::families::script::meta::{MetaError, ModuleMetadata};

/// A stub in the shape of `server-ui_2.2.0.json` (v1.26.50.4).
pub const STUB: &str = r#"{
  "module_type": "script",
  "name": "@minecraft/server-ui",
  "parentModule": { "name": "@minecraft/server-ui-bindings", "version": "2.2.0" },
  "peer_dependencies": [
    { "name": "@minecraft/server", "versions": [ { "version": "2.0.0" } ], "wrapperName": "@minecraft/server-wrapper" }
  ],
  "uuid": "2BD50A27-AB5F-4F40-A596-3641627C635E",
  "wrapper_module": "@minecraft/server-ui-bindings-v2-wrapper"
}"#;

/// A native module with one declaration of each kind, in the generator's shapes.
pub const NATIVE: &str = r#"{
  "classes": [
    {
      "base_types": [],
      "constants": [
        { "is_read_only": true, "is_static": true, "name": "maxValue",
          "type": { "is_bind_type": false, "is_errorable": false, "name": "int32" }, "value": 7 }
      ],
      "functions": [
        {
          "arguments": [
            { "details": null, "name": "plain",
              "type": { "is_bind_type": false, "is_errorable": false, "name": "string" } },
            { "details": { "default_value": "null", "max_value": 15, "min_value": 0.009999999776482582 }, "name": "bounded",
              "type": { "is_bind_type": false, "is_errorable": false, "name": "float" } },
            { "name": "generated",
              "type": { "is_bind_type": false, "is_errorable": false, "name": "T" } }
          ],
          "call_privilege": [ { "name": "restricted_execution" }, { "name": "early_execution" } ],
          "is_constructor": false,
          "is_static": false,
          "name": "doThing",
          "return_type": { "is_bind_type": false, "is_errorable": true, "name": "undefined" }
        }
      ],
      "name": "Thing",
      "properties": [
        { "get_privilege": [ { "name": "restricted_execution" } ], "is_baked": false, "is_read_only": false,
          "name": "size", "set_privilege": [ { "name": "default" } ],
          "type": { "is_bind_type": false, "is_errorable": false, "name": "int32" } }
      ],
      "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" }
    }
  ],
  "constants": [
    { "is_read_only": true, "is_static": true, "name": "TicksPerDay",
      "type": { "is_bind_type": false, "is_errorable": false, "name": "int32" }, "value": 24000 }
  ],
  "dependencies": [],
  "enums": [
    { "constants": [
        { "is_read_only": true, "is_static": true, "name": "Red",
          "type": { "is_bind_type": false, "is_errorable": false, "name": "string" }, "value": "red" }
      ], "name": "Colour" }
  ],
  "errors": [
    { "functions": [], "name": "ThingError", "properties": [],
      "type": { "is_bind_type": true, "is_errorable": false, "name": "ThingError" } }
  ],
  "functions": [],
  "interfaces": [
    { "base_types": [], "name": "ThingOptions", "properties": [
        { "is_baked": false, "is_read_only": false, "name": "size",
          "type": { "is_bind_type": false, "is_errorable": false, "name": "int32" } } ],
      "type": { "is_bind_type": true, "is_errorable": false, "name": "ThingOptions" } }
  ],
  "module_type": "script",
  "name": "@minecraft/server-bindings",
  "objects": [
    { "is_read_only": true, "is_static": true, "name": "thing",
      "type": { "is_bind_type": true, "is_errorable": false, "name": "Thing" } }
  ],
  "peer_dependencies": [
    { "name": "@minecraft/common", "uuid": "77ec12b4-1b2b-4c98-8d34-d1cd63f849d5", "versions": [ { "version": "1.2.0" } ] },
    { "name": "@minecraft/vanilla-data", "types_only": true, "version": "1.20.70", "versions": [ { "version": "1.20.70" } ] }
  ],
  "type_aliases": [
    { "alias_type": "type_map", "mappings": [ { "name": "minecraft:thing", "value": "Thing" } ], "name": "ThingTypeMap",
      "type": { "is_bind_type": false, "is_errorable": false, "name": "ThingTypeMap" } }
  ],
  "uuid": "0196a2fa-8fb5-79a7-b29e-ce3cbc644f82"
}"#;

fn parse(json: &str) -> Result<ModuleMetadata, MetaError> {
    ModuleMetadata::parse(json.as_bytes())
}

#[test]
fn a_file_with_parent_module_is_a_stub() {
    let ModuleMetadata::Stub(stub) = parse(STUB).unwrap() else {
        panic!("not a stub")
    };
    assert_eq!(stub.name, "@minecraft/server-ui");
    assert_eq!(stub.parent_module.name, "@minecraft/server-ui-bindings");
    assert_eq!(stub.parent_module.version, "2.2.0");
    assert_eq!(
        stub.wrapper_module,
        "@minecraft/server-ui-bindings-v2-wrapper"
    );
    assert!(stub.overlay.is_none());
    assert_eq!(
        stub.peer_dependencies[0].wrapper_name.as_deref(),
        Some("@minecraft/server-wrapper")
    );
}

#[test]
fn the_uuid_is_read_whatever_its_case() {
    // `@minecraft/server-ui` is the only upper-case module UUID.
    let stub = parse(STUB).unwrap();
    assert_eq!(
        stub.uuid().to_string(),
        "2bd50a27-ab5f-4f40-a596-3641627c635e"
    );
}

#[test]
fn a_file_without_parent_module_is_a_native_module() {
    let ModuleMetadata::Native(native) = parse(NATIVE).unwrap() else {
        panic!("not native")
    };
    assert_eq!(native.name, "@minecraft/server-bindings");
    assert_eq!(native.classes[0].name, "Thing");
    assert_eq!(native.classes[0].constants[0].value, ConstValue::Int(7));
    assert_eq!(
        native.enums[0].constants[0].value,
        ConstValue::Str("red".into())
    );
    assert_eq!(native.constants[0].value, ConstValue::Int(24000));
    assert_eq!(native.peer_dependencies[1].types_only, Some(true));
    assert_eq!(
        native.type_aliases[0].mappings.as_ref().unwrap()[0].value,
        "Thing"
    );
}

#[test]
fn arguments_tell_null_details_from_absent_ones() {
    let ModuleMetadata::Native(native) = parse(NATIVE).unwrap() else {
        panic!("not native")
    };
    let arguments = &native.classes[0].functions[0].arguments;
    // `"details": null`: the key is there, no details.
    assert!(arguments[0].details.is_none());
    assert!(arguments[0].details_key_present);
    // An object: the f32 artefact of `SoundInstance.setPitch` kept as f64.
    let details = arguments[1].details.as_ref().unwrap();
    assert!(arguments[1].details_key_present);
    assert_eq!(details.min_value, Some(0.009_999_999_776_482_582));
    assert_eq!(details.max_value, Some(15.0));
    // Script-generated arguments have no `details` key.
    assert!(arguments[2].details.is_none());
    assert!(!arguments[2].details_key_present);
}

#[test]
fn an_unknown_key_is_an_error() {
    let text = STUB.replacen(
        "\"module_type\"",
        "\"new_generator_key\": 1, \"module_type\"",
        1,
    );
    let error = parse(&text).unwrap_err();
    let MetaError::UnknownMember { path } = error else {
        panic!("not an unknown member: {error:?}")
    };
    assert_eq!(path.to_string(), "new_generator_key");
}

#[test]
fn an_unknown_key_deep_in_a_descriptor_is_an_error() {
    let text = NATIVE.replacen(
        "\"name\": \"maxValue\"",
        "\"name\": \"maxValue\", \"is_deprecated\": true",
        1,
    );
    let MetaError::UnknownMember { path } = parse(&text).unwrap_err() else {
        panic!("not an unknown member")
    };
    assert_eq!(path.to_string(), "classes/0/constants/0/is_deprecated");
}

#[test]
fn module_type_other_than_script_is_an_error() {
    let text = STUB.replacen("\"script\"", "\"data\"", 1);
    assert!(matches!(parse(&text).unwrap_err(), MetaError::Json(_)));
}

#[test]
fn a_constant_value_of_another_json_type_is_an_error() {
    let text = NATIVE.replacen("\"value\": 7", "\"value\": 7.5", 1);
    assert!(matches!(parse(&text).unwrap_err(), MetaError::Json(_)));
    let text = NATIVE.replacen("\"value\": 7", "\"value\": true", 1);
    assert!(matches!(parse(&text).unwrap_err(), MetaError::Json(_)));
}

#[test]
fn a_stub_without_wrapper_module_is_an_error() {
    let text = STUB.replacen(
        ",\n  \"wrapper_module\": \"@minecraft/server-ui-bindings-v2-wrapper\"",
        "",
        1,
    );
    assert!(matches!(
        parse(&text).unwrap_err(),
        MetaError::StubWithoutWrapper
    ));
}

#[test]
fn wrapper_module_without_parent_module_is_an_error() {
    let text = NATIVE.replacen(
        "\"module_type\"",
        "\"wrapper_module\": \"@minecraft/x-wrapper\", \"module_type\"",
        1,
    );
    assert!(matches!(
        parse(&text).unwrap_err(),
        MetaError::WrapperWithoutParent
    ));
}

#[test]
fn a_stub_with_a_native_only_member_is_an_error() {
    for member in [
        "dependencies",
        "constants",
        "errors",
        "functions",
        "objects",
        "type_aliases",
    ] {
        let text = STUB.replacen(
            "\"module_type\"",
            &format!("\"{member}\": [], \"module_type\""),
            1,
        );
        let error = parse(&text).unwrap_err();
        assert!(
            matches!(error, MetaError::StubMember { member: m } if m == member),
            "{member}: {error:?}"
        );
    }
}

#[test]
fn declarations_on_a_stub_are_its_overlay() {
    // v1.26.20.4 `server-ui_2.1.0-beta.json`; check 3 warns about it.
    let text = STUB.replacen(
        "\"module_type\"",
        r#""classes": [], "enums": [ { "constants": [], "is_script_generated": true, "name": "Mode",
            "raw_script_text": "enum Mode {}", "raw_tsdoc_text": "@beta" } ], "module_type""#,
        1,
    );
    let ModuleMetadata::Stub(stub) = parse(&text).unwrap() else {
        panic!("not a stub")
    };
    let overlay = stub.overlay.unwrap();
    assert!(overlay.classes.is_empty());
    assert!(overlay.interfaces.is_empty());
    assert_eq!(overlay.enums[0].name, "Mode");
}

#[test]
fn a_native_module_without_one_of_its_arrays_is_an_error() {
    for member in [
        "classes",
        "interfaces",
        "enums",
        "errors",
        "functions",
        "objects",
        "constants",
        "type_aliases",
        "dependencies",
    ] {
        let value: facet_value::Value = facet_json::from_str(NATIVE).unwrap();
        let mut object = value.as_object().unwrap().clone();
        object.remove(member);
        let text = facet_json::to_string(&facet_value::Value::from(object)).unwrap();
        let error = parse(&text).unwrap_err();
        assert!(
            matches!(error, MetaError::MissingArray { member: m } if m == member),
            "{member}: {error:?}"
        );
    }
}

#[test]
fn a_bom_is_accepted_and_bad_utf8_is_an_error() {
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(STUB.as_bytes());
    assert!(ModuleMetadata::parse(&bytes).is_ok());
    let bytes = STUB
        .replacen("server-ui\"", "server-\u{fffd}\"", 1)
        .replace('\u{fffd}', "\u{1}");
    let mut bytes = bytes.into_bytes();
    let at = bytes.iter().position(|b| *b == 1).unwrap();
    bytes[at] = 0xff;
    assert!(matches!(
        ModuleMetadata::parse(&bytes).unwrap_err(),
        MetaError::NotUtf8(_)
    ));
}
