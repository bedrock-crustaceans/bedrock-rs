//! `MolangList`, the value-or-array Molang field: `Option<MolangList>` reads `null`
//! as no expression, a scalar as `One`, an array as `Many`, element by element.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::molang::{Molang, MolangList, MolangSource};

fn list(json: &str) -> Option<MolangList> {
    facet_json::from_str::<Option<MolangList>>(json).unwrap()
}

#[test]
fn null_is_no_expression() {
    assert_eq!(list("null"), None);
}

#[test]
fn a_scalar_is_one_value() {
    assert_eq!(list("1.5"), Some(MolangList::One(Molang::Const(1.5))));
    assert_eq!(list("true"), Some(MolangList::One(Molang::Bool(true))));
    assert_eq!(
        list(r#""q.x""#),
        Some(MolangList::One(Molang::Expr(
            MolangSource::string_without_context("q.x")
        )))
    );
}

#[test]
fn an_array_is_read_element_by_element() {
    assert_eq!(
        list(r#"[1, "v.x", [2, false], []]"#),
        Some(MolangList::Many(vec![
            MolangList::One(Molang::Const(1.0)),
            MolangList::One(Molang::Expr(MolangSource::string_without_context("v.x"))),
            MolangList::Many(vec![
                MolangList::One(Molang::Const(2.0)),
                MolangList::One(Molang::Bool(false)),
            ]),
            MolangList::Many(vec![]),
        ]))
    );
}

#[test]
fn writes_back_the_form_it_read() {
    for json in ["1.5", "true", r#""q.x""#, r#"[1.5,"v.x",[2.5,false],[]]"#] {
        let value = list(json).unwrap();
        assert_eq!(facet_json::to_string(&value).unwrap(), json);
    }
}

#[test]
fn objects_null_elements_and_non_finite_numbers_are_rejected() {
    for json in [
        r#"{"expression": "1", "version": 1}"#,
        "{}",
        "[1, null]",
        r#"[{"a": 1}]"#,
        "1e39",
        "[1e39]",
    ] {
        assert!(
            facet_json::from_str::<Option<MolangList>>(json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn a_non_finite_constant_is_not_written() {
    let value = MolangList::Many(vec![MolangList::One(Molang::Const(f32::NAN))]);
    assert!(facet_json::to_string(&value).is_err());
}
