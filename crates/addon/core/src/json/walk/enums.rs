//! The walk of an enum: the variant the value holds, or the arm facet reads.

use facet::Shape;

use super::scalars::scalar_arm;
use super::untagged::untagged_arm;
use super::{Cursor, Step, Typed, Unknown, Value, rule_of, walk_object, walk_value, with_step};
use crate::json::scalar::Token;

pub(super) fn walk_enum(
    shape: &'static Shape,
    variants: &'static [facet::Variant],
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let typed_enum = typed.and_then(|t| t.into_enum().ok());
    let active = typed_enum.and_then(|e| e.variant_index().ok());
    // The typed value's fields when `index` is its variant.
    let typed_payload = |index: usize| typed_enum.filter(|_| active == Some(index));
    if shape.is_untagged() {
        if active.is_none()
            && let Some(rules) = cursor.rules
            && let Some(token) = Token::of(json)
        {
            if let Some(index) = scalar_arm(variants, token, rules, cursor) {
                walk_variant(index, &variants[index], json, None, cursor, out);
            }
            return;
        }
        if let Some(index) = active.or_else(|| untagged_arm(variants, json)) {
            walk_variant(
                index,
                &variants[index],
                json,
                typed_payload(index),
                cursor,
                out,
            );
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return; // a unit variant written as a string
    };
    if let Some(tag) = shape.get_tag_attr() {
        // Internally tagged (`{"type": "a", …}`): the payload's fields sit beside the tag.
        let Some(name) = object.get(tag).and_then(|t| t.as_string()) else {
            return;
        };
        if let Some(index) = variants
            .iter()
            .position(|v| v.effective_name() == name.as_str())
        {
            let mut rest = object.clone();
            rest.remove(tag);
            let payload = Value::from(rest);
            walk_variant(
                index,
                &variants[index],
                &payload,
                typed_payload(index),
                cursor,
                out,
            );
        }
        return;
    }
    // Externally tagged: `{"Variant": payload}`.
    let mut members = object.iter();
    let (Some((name, payload)), None) = (members.next(), members.next()) else {
        return;
    };
    if let Some(index) = variants
        .iter()
        .position(|v| v.effective_name() == name.as_str())
    {
        cursor.path.push(name.as_str());
        walk_variant(
            index,
            &variants[index],
            payload,
            typed_payload(index),
            cursor,
            out,
        );
        cursor.path.pop();
    }
}

/// A variant's payload: a struct variant's fields, or a newtype variant's one field. `typed` is
/// the enum value when this variant is its active one.
fn walk_variant(
    index: usize,
    variant: &'static facet::Variant,
    json: &Value,
    typed: Option<facet::PeekEnum<'_, '_>>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let fields = variant.data.fields;
    let typed_field = |field: usize| typed.and_then(|e| e.field(field).ok().flatten());
    match variant.data.kind {
        facet::StructKind::Struct => walk_object(
            fields,
            rule_of(variant.attributes.iter()),
            &|field| Step::VariantField(index, field),
            &typed_field,
            json,
            cursor,
            out,
        ),
        facet::StructKind::TupleStruct | facet::StructKind::Tuple if fields.len() == 1 => {
            with_step(cursor, Some(Step::VariantField(index, 0)), |cursor| {
                walk_value(fields[0].shape(), json, typed_field(0), cursor, out);
            });
        }
        _ => {}
    }
}
