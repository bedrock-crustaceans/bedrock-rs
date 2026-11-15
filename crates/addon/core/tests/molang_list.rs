//! `MolangList`, the value-or-array Molang field: `Option<MolangList>` reads `null`
//! as no expression, a scalar as `One`, an array as `Many`, element by element.

use bedrock_addon_core::json::{
    EncodeError, Indent, Value, WriteStyle, from_str_jsonc, write_json,
};
use bedrock_addon_core::molang::{Molang, MolangList, MolangSource};
use bedrock_addon_core::schema::read_value;

/// `json` read as the field: `None` (outer) when the field refuses it.
#[allow(
    clippy::option_option,
    reason = "the outer `None` is a refusal, the inner one the field's absence"
)]
fn read(json: &str) -> Option<Option<MolangList>> {
    read_value::<Option<MolangList>>(&from_str_jsonc::<Value>(json).unwrap())
}

fn list(json: &str) -> Option<MolangList> {
    read(json).unwrap()
}

/// `value` written on one line.
fn compact(value: &MolangList) -> Result<String, EncodeError> {
    let mut out = Vec::new();
    let style = WriteStyle {
        indent: Indent::Compact,
        trailing_newline: false,
    };
    write_json(value, &mut out, style)?;
    Ok(String::from_utf8(out).unwrap())
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
        assert_eq!(compact(&value).unwrap(), json);
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
        assert!(read(json).is_none(), "{json}");
    }
}

#[test]
fn a_non_finite_constant_is_not_written() {
    let value = MolangList::Many(vec![MolangList::One(Molang::Const(f32::NAN))]);
    assert!(compact(&value).is_err());
}
