//! Type-descriptor lowering. The descriptors are
//! written in the generator's shapes.

#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use bedrock_addon::families::script::meta::model::script::RawTypeDesc;
use bedrock_addon::families::script::meta::{
    Errorability, MetaError, Prim, Privilege, TypeDescriptor, TypeKind, TypeOverlay,
};

fn raw(json: &str) -> RawTypeDesc {
    facet_json::from_str(json).unwrap()
}

fn lower(json: &str) -> Result<TypeDescriptor, MetaError> {
    TypeDescriptor::lower(&raw(json))
}

const INT32: &str = r#"{ "is_bind_type": false, "is_errorable": false, "name": "int32",
    "valid_range": { "max": 2147483647, "min": -2147483648 } }"#;
const STRING: &str = r#"{ "is_bind_type": false, "is_errorable": false, "name": "string" }"#;

#[test]
fn rule_1_a_composite_tag_takes_its_payload() {
    let optional = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "name": "optional", "optional_type": {STRING} }}"#
    ))
    .unwrap();
    assert_eq!(
        optional.kind,
        TypeKind::Optional(Box::new(lower(STRING).unwrap()))
    );
    let map = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "key_type": {STRING}, "name": "map", "value_type": {INT32} }}"#
    ))
    .unwrap();
    assert!(matches!(map.kind, TypeKind::Map { .. }));
    let variant = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "name": "variant", "variant_types": [{STRING}, {INT32}] }}"#
    ))
    .unwrap();
    let TypeKind::Variant(arms) = variant.kind else {
        panic!("not a variant: {variant:?}")
    };
    assert_eq!(arms.len(), 2);
    let array = lower(&format!(
        r#"{{ "element_type": {STRING}, "is_bind_type": false, "is_errorable": false, "name": "array" }}"#
    ))
    .unwrap();
    assert!(matches!(array.kind, TypeKind::Array(_)));
    let promise = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "name": "promise", "promise_type": {STRING} }}"#
    ))
    .unwrap();
    assert!(matches!(promise.kind, TypeKind::Promise(_)));
    let generator = lower(&format!(
        r#"{{ "generator_type": {{ "next_type": {STRING}, "return_type": {STRING}, "yield_type": {INT32} }},
            "is_bind_type": false, "is_errorable": false, "name": "generator" }}"#
    ))
    .unwrap();
    assert!(matches!(generator.kind, TypeKind::Generator { .. }));
}

#[test]
fn rule_1_a_closure_keeps_its_call_privilege() {
    // `subscribe(callback: (arg) => void)`: the callback runs with `early_execution`.
    let closure = lower(&format!(
        r#"{{ "closure_type": {{ "argument_types": [{STRING}], "call_privilege": {{ "name": "early_execution" }},
            "return_type": {{ "is_bind_type": false, "is_errorable": false, "name": "undefined" }} }},
            "is_bind_type": false, "is_errorable": false, "name": "closure" }}"#
    ))
    .unwrap();
    let TypeKind::Closure {
        args,
        ret,
        privilege,
    } = closure.kind
    else {
        panic!("not a closure: {closure:?}")
    };
    assert_eq!(args.len(), 1);
    assert_eq!(ret.kind, TypeKind::Prim(Prim::Undefined));
    assert_eq!(privilege.unwrap().bits, Privilege::EARLY_EXECUTION);
}

#[test]
fn rule_1_a_composite_without_its_payload_is_an_error() {
    let error = lower(r#"{ "is_bind_type": false, "is_errorable": false, "name": "optional" }"#)
        .unwrap_err();
    assert!(matches!(
        error,
        MetaError::MissingPayload {
            tag: "optional",
            key: "optional_type"
        }
    ));
    let error = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "key_type": {STRING}, "name": "map" }}"#
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        MetaError::MissingPayload {
            tag: "map",
            key: "value_type"
        }
    ));
}

#[test]
fn rule_1_a_composite_with_another_payload_is_an_error() {
    let error = lower(&format!(
        r#"{{ "element_type": {STRING}, "is_bind_type": false, "is_errorable": false, "name": "optional", "optional_type": {STRING} }}"#
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        MetaError::UnexpectedPayload {
            key: "element_type"
        }
    ));
}

#[test]
fn rule_2_a_payload_under_another_name_is_an_error() {
    let error = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "name": "string", "optional_type": {STRING} }}"#
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        MetaError::UnexpectedPayload {
            key: "optional_type"
        }
    ));
}

#[test]
fn rule_3_a_bind_type_is_a_declared_name_with_its_module() {
    let bind = lower(
        r#"{ "from_module": { "name": "@minecraft/server-bindings", "uuid": "0196a2fa-8fb5-79a7-b29e-ce3cbc644f82",
            "version": "2.10.0" }, "is_bind_type": true, "is_errorable": false, "name": "Entity" }"#,
    )
    .unwrap();
    let TypeKind::Bind { name, from } = bind.kind else {
        panic!("not a bind type: {bind:?}")
    };
    assert_eq!(name, "Entity");
    assert_eq!(from.unwrap().version, "2.10.0");
}

#[test]
fn rule_3_a_bind_type_with_a_payload_is_an_error() {
    let error = lower(&format!(
        r#"{{ "is_bind_type": true, "is_errorable": false, "name": "optional", "optional_type": {STRING} }}"#
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        MetaError::UnexpectedPayload {
            key: "optional_type"
        }
    ));
}

#[test]
fn rule_4_generic_base_makes_a_generic_whatever_the_name() {
    let generic = lower(&format!(
        r#"{{ "generic_base": {{ "is_bind_type": false, "is_core": false, "is_errorable": false, "is_type": true,
            "name": "BlockStateArg" }}, "generic_types": [{STRING}], "is_bind_type": false, "is_errorable": false,
            "name": "BlockStateArg<T>" }}"#
    ))
    .unwrap();
    let TypeKind::Generic { base, args } = generic.kind else {
        panic!("not a generic: {generic:?}")
    };
    assert_eq!(base.kind, TypeKind::Opaque("BlockStateArg".to_owned()));
    assert!(base.overlay.is_type);
    assert!(!base.overlay.is_core);
    assert_eq!(args, [lower(STRING).unwrap()]);
}

#[test]
fn rule_5_primitives_and_everything_else_opaque() {
    assert_eq!(lower(INT32).unwrap().kind, TypeKind::Prim(Prim::Int32));
    let error_class = lower(r#"{ "is_bind_type": false, "is_errorable": false, "name": "Error" }"#);
    assert_eq!(error_class.unwrap().kind, TypeKind::Prim(Prim::JsError));
    // Lower-case `error` (once in v1.26.50.4) is not the primitive.
    let lower_case = lower(r#"{ "is_bind_type": false, "is_errorable": false, "name": "error" }"#);
    assert_eq!(
        lower_case.unwrap().kind,
        TypeKind::Opaque("error".to_owned())
    );
    let ts = "T extends keyof BlockComponentTypeMap ? BlockComponentTypeMap[T] : BlockCustomComponentInstance";
    let conditional = lower(&format!(
        r#"{{ "is_bind_type": false, "is_errorable": false, "name": "{ts}" }}"#
    ));
    assert_eq!(conditional.unwrap().kind, TypeKind::Opaque(ts.to_owned()));
}

#[test]
fn rule_6_valid_range_is_dropped() {
    let with_range = lower(INT32).unwrap();
    let without = lower(r#"{ "is_bind_type": false, "is_errorable": false, "name": "int32" }"#);
    assert_eq!(with_range, without.unwrap());
}

#[test]
fn errorability_has_three_forms() {
    let no = lower(STRING).unwrap();
    assert_eq!(no.errors, Errorability::No);
    let unlisted = lower(r#"{ "is_bind_type": true, "is_errorable": true, "name": "Dimension" }"#);
    assert_eq!(unlisted.unwrap().errors, Errorability::Unlisted);
    let listed = lower(
        r#"{ "error_types": [ { "is_bind_type": true, "is_errorable": false, "name": "InvalidIteratorError" } ],
            "is_bind_type": false, "is_errorable": true, "name": "undefined" }"#,
    )
    .unwrap();
    let Errorability::Listed(errors) = listed.errors else {
        panic!("not listed: {listed:?}")
    };
    assert!(
        matches!(&errors[0].kind, TypeKind::Bind { name, .. } if name == "InvalidIteratorError")
    );
}

#[test]
fn check_8_error_types_need_is_errorable_and_are_never_empty() {
    let not_errorable = lower(
        r#"{ "error_types": [ { "is_bind_type": true, "is_errorable": false, "name": "E" } ],
            "is_bind_type": false, "is_errorable": false, "name": "undefined" }"#,
    );
    assert!(matches!(not_errorable.unwrap_err(), MetaError::ErrorTypes));
    let empty = lower(
        r#"{ "error_types": [], "is_bind_type": false, "is_errorable": true, "name": "undefined" }"#,
    );
    assert!(matches!(empty.unwrap_err(), MetaError::ErrorTypes));
}

#[test]
fn overlay_flags_default_to_false() {
    let plain = lower(STRING).unwrap();
    assert_eq!(plain.overlay, TypeOverlay::default());
    let flagged = lower(
        r#"{ "is_bind_type": false, "is_class": true, "is_errorable": false, "is_optional": true, "name": "T" }"#,
    )
    .unwrap();
    assert!(flagged.overlay.is_class && flagged.overlay.is_optional);
    assert!(!flagged.overlay.is_enum);
}
