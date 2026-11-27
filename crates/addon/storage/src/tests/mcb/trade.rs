//! `tiers` bodies.

use super::{Body, assert_unsupported, document};
use crate::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 26, 10);

#[expect(
    clippy::needless_pass_by_value,
    reason = "a test hands over what it built"
)]
fn table(body: Body) -> Result<String, crate::StorageError> {
    to_json_text(&document("tiers", VERSION, &body.0))
}

/// One item of a choice: quantity, name, no function, the pinned bytes, the multiplier.
fn item(body: Body, quantity: (i32, i32), name: &str, price_multiplier: f32) -> Body {
    body.i32(quantity.0)
        .i32(quantity.1)
        .string(name)
        .raw(&[0])
        .raw(&[2, 0, 0, 0, 0, 0, 0, 0, 0])
        .f32(price_multiplier)
}

#[test]
fn a_member_at_its_default_is_left_out() {
    // one tier, one group, one trade: one emerald for one stick, everything at its default
    let mut body = Body::default().raw(&[1, 1]).i32(1).raw(&[1]);
    body = item(body.raw(&[1, 1]), (1, 1), "minecraft:emerald", 0.0);
    body = item(body.raw(&[1, 1]), (1, 1), "minecraft:stick", 0.0);
    body = body.i32(1).i32(7).raw(&[1]).i32(1).i32(0);
    assert_eq!(
        table(body).unwrap(),
        concat!(
            r#"{"format_version":"1.26.10","tiers":[{"groups":[{"num_to_select":1,"trades":["#,
            r#"{"wants":[{"choice":[{"item":"minecraft:emerald","functions":[]}]}],"#,
            r#""gives":[{"choice":[{"item":"minecraft:stick","functions":[]}]}]}"#,
            r#"]}],"total_exp_required":0}]}"#
        )
    );
}

#[test]
fn a_member_off_its_default_is_written() {
    let mut body = Body::default().raw(&[1, 1]).i32(-1).raw(&[1]);
    // wants: a choice of two items
    body = item(body.raw(&[1, 2]), (3, 5), "minecraft:wheat", 0.05);
    body = item(body, (1, 2), "minecraft:carrot", 0.5);
    body = item(body.raw(&[1, 1]), (4, 4), "minecraft:emerald", 0.0);
    body = body.i32(10).i32(16).raw(&[1]).i32(6).i32(70);
    assert_eq!(
        table(body).unwrap(),
        concat!(
            r#"{"format_version":"1.26.10","tiers":[{"groups":[{"num_to_select":-1,"trades":["#,
            r#"{"wants":[{"choice":["#,
            r#"{"quantity":{"min":3,"max":5},"item":"minecraft:wheat","functions":[],"#,
            r#""price_multiplier":0.05},"#,
            r#"{"quantity":{"max":2},"item":"minecraft:carrot","functions":[],"#,
            r#""price_multiplier":0.5}]}],"#,
            r#""gives":[{"choice":[{"quantity":{"min":4,"max":4},"#,
            r#""item":"minecraft:emerald","functions":[]}]}],"#,
            r#""trader_exp":10,"max_uses":16,"weight":6}"#,
            r#"]}],"total_exp_required":70}]}"#
        )
    );
}

#[test]
fn an_item_with_a_function_is_unsupported() {
    // no vanilla table with a function is baked, so a function's layout is not known
    let body = Body::default()
        .raw(&[1, 1])
        .i32(1)
        .raw(&[1, 1, 1])
        .i32(1)
        .i32(1)
        .string("minecraft:book")
        .raw(&[1])
        .raw(&[2, 0, 0, 0, 0, 0, 0, 0, 0])
        .f32(0.0);
    assert_unsupported(table(body));
}

#[test]
fn a_pinned_field_off_its_value_is_unsupported() {
    // the byte after `max_uses` is 01 in every vanilla trade
    let mut body = Body::default().raw(&[1, 1]).i32(1).raw(&[1]);
    body = item(body.raw(&[1, 1]), (1, 1), "minecraft:emerald", 0.0);
    body = item(body.raw(&[1, 1]), (1, 1), "minecraft:stick", 0.0);
    body = body.i32(1).i32(7).raw(&[0]).i32(1).i32(0);
    assert_unsupported(table(body));
}
