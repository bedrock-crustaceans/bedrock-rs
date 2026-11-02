//! Hostile bodies: cut short, counts and lengths past the body, bytes after the end, floats
//! JSON cannot write. Each is a fault, never a panic.

use super::{Body, assert_unsupported, document};
use crate::json::{MAX_JSON_DEPTH, check_depth};
use crate::storage::mcb::to_json_text;
use crate::storage::{McbFault, StorageError};

/// One small document of every payload kind, each with strings, counts and floats.
fn samples() -> Vec<Vec<u8>> {
    let shape = Body::default()
        .string("demo:step")
        .raw(&[1])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0]);
    let table = Body::default()
        .raw(&[1, 1])
        .i32(1)
        .raw(&[1, 1, 1])
        .i32(2)
        .i32(3)
        .string("minecraft:wheat")
        .raw(&[0, 2, 0, 0, 0, 0, 0, 0, 0, 0])
        .f32(0.05)
        .raw(&[1, 1])
        .i32(1)
        .i32(1)
        .string("minecraft:emerald")
        .raw(&[0, 2, 0, 0, 0, 0, 0, 0, 0, 0])
        .f32(0.0)
        .i32(2)
        .i32(16)
        .raw(&[1])
        .i32(1)
        .i32(0);
    let item = Body::default()
        .string("demo:ruby")
        .string("items")
        .raw(&[0, 0])
        .u32(3)
        .component("minecraft:cooldown")
        .string("gem")
        .f32(0.25)
        .string("use")
        .component("minecraft:icon")
        .raw(&[1])
        .string("default")
        .string("ruby")
        .component("minecraft:tags")
        .raw(&[1])
        .string("demo:gem");
    let effect = Body::default()
        .string("demo:spark")
        .string("particles_alpha")
        .string("textures/particle/demo")
        .u32(3)
        .component("minecraft:emitter_rate_steady")
        .molang("v.rate")
        .number(20.0)
        .component("minecraft:particle_appearance_tinting")
        .raw(&[1, 1, 1])
        .string("0.000000")
        .number(1.0)
        .number(0.5)
        .number(0.25)
        .number(1.0)
        .molang("v.t")
        .component("minecraft:particle_motion_collision")
        .number(1.0)
        .f32s(&[0.0, 0.0, 0.25])
        .raw(&[0, 1, 1])
        .f32(2.0)
        .string("hit")
        .raw(&[1, 1])
        .string("variable.c")
        .raw(&[0])
        .string("linear")
        .raw(&[2])
        .number(0.0)
        .number(1.0)
        .molang("v.age")
        .number(1.0)
        .raw(&[0]);
    vec![
        document("minecraft:voxel_shape", (1, 21, 110), &shape.0),
        document("tiers", (1, 26, 10), &table.0),
        document("minecraft:item", (1, 26, 30), &item.0),
        document("particle_effect", (1, 26, 10), &effect.0),
    ]
}

#[test]
fn every_sample_decodes() {
    for sample in samples() {
        to_json_text(&sample).unwrap();
    }
}

#[test]
fn a_document_cut_anywhere_is_a_fault() {
    for sample in samples() {
        for len in 0..sample.len() {
            assert!(
                matches!(
                    to_json_text(&sample[..len]),
                    Err(StorageError::Mcb {
                        fault: McbFault::Header | McbFault::UnsupportedBody
                    })
                ),
                "{len} of {}",
                sample.len()
            );
        }
    }
}

#[test]
fn a_document_with_bytes_after_its_end_is_unsupported() {
    for mut sample in samples() {
        sample.push(0);
        assert_unsupported(to_json_text(&sample));
    }
}

#[test]
fn a_document_with_any_byte_changed_decodes_or_is_a_fault() {
    // no value of any byte panics, loops or allocates past the document
    for sample in samples() {
        for at in 4..sample.len() {
            for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
                let mut changed = sample.clone();
                changed[at] = value;
                if let Ok(text) = to_json_text(&changed) {
                    assert!(text.len() < 16 * sample.len(), "{at} = {value:#x}");
                }
            }
        }
    }
}

#[test]
fn a_count_larger_than_the_body_is_unsupported() {
    // 127 boxes declared, one present
    let shape = Body::default()
        .string("demo:step")
        .raw(&[0x7f])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0]);
    assert_unsupported(to_json_text(&document(
        "minecraft:voxel_shape",
        (1, 21, 110),
        &shape.0,
    )));
    // a count byte of 128 or more was never seen
    let shape = Body::default().string("demo:step").raw(&[0x80, 0x01]);
    assert_unsupported(to_json_text(&document(
        "minecraft:voxel_shape",
        (1, 21, 110),
        &shape.0,
    )));
    // four billion components declared in a body of a few bytes
    let effect = Body::default()
        .string("demo:spark")
        .string("m")
        .string("t")
        .u32(u32::MAX);
    assert_unsupported(to_json_text(&document(
        "particle_effect",
        (1, 26, 10),
        &effect.0,
    )));
}

#[test]
fn a_string_longer_than_the_body_is_unsupported() {
    let shape = Body::default().raw(&[0xff, 0x7f]).raw(b"demo:step");
    assert_unsupported(to_json_text(&document(
        "minecraft:voxel_shape",
        (1, 21, 110),
        &shape.0,
    )));
    let not_utf8 = Body::default().raw(&[2, 0xc3, 0x28, 0]);
    assert_unsupported(to_json_text(&document(
        "minecraft:voxel_shape",
        (1, 21, 110),
        &not_utf8.0,
    )));
}

#[test]
fn a_payload_or_a_version_no_vanilla_document_has_is_unsupported() {
    let shape = Body::default()
        .string("demo:step")
        .raw(&[1])
        .f32s(&[0.0, 0.0, 0.0, 16.0, 8.0, 16.0]);
    assert!(to_json_text(&document("minecraft:voxel_shape", (1, 21, 110), &shape.0)).is_ok());
    // the layout is known for the versions vanilla bakes, and a body does not say its own
    assert_unsupported(to_json_text(&document(
        "minecraft:voxel_shape",
        (1, 21, 120),
        &shape.0,
    )));
    assert_unsupported(to_json_text(&document(
        "minecraft:entity",
        (1, 21, 110),
        &shape.0,
    )));
}

#[test]
fn a_float_json_cannot_write_is_its_own_fault() {
    for value in [f32::NAN, f32::INFINITY] {
        let shape = Body::default()
            .string("demo:step")
            .raw(&[1])
            .f32s(&[0.0, 0.0, 0.0, 16.0, value, 16.0]);
        assert!(matches!(
            to_json_text(&document("minecraft:voxel_shape", (1, 21, 110), &shape.0)),
            Err(StorageError::Mcb {
                fault: McbFault::NonFiniteFloat
            })
        ));
    }
}

const _: () = assert!(12 < MAX_JSON_DEPTH);

#[test]
fn no_body_nests_deeper_than_its_layout() {
    // a layout fixes how deep its text nests (a body has no recursive member), far below
    // the depth the JSON readers accept
    for sample in samples() {
        let text = to_json_text(&sample).unwrap();
        assert!(check_depth(text.as_bytes(), 12).is_ok(), "{text}");
    }
}

#[test]
fn decoding_costs_in_proportion_to_the_body() {
    // `n` trades in 100 tiers of `n / 1000` groups of ten trades
    let table = |trades: usize| {
        let groups = u8::try_from(trades / 1000).unwrap();
        let mut body = Body::default().raw(&[100]);
        for _ in 0..100 {
            body = body.raw(&[groups]);
            for _ in 0..groups {
                body = body.i32(1).raw(&[10]);
                for _ in 0..10 {
                    for name in ["minecraft:emerald", "minecraft:enchanted_book"] {
                        body = body
                            .raw(&[1, 1])
                            .i32(2)
                            .i32(3)
                            .string(name)
                            .raw(&[0, 2, 0, 0, 0, 0, 0, 0, 0, 0])
                            .f32(0.05);
                    }
                    body = body.i32(2).i32(16).raw(&[1]).i32(1);
                }
            }
            body = body.i32(0);
        }
        document("tiers", (1, 26, 10), &body.0)
    };
    crate::test_cost::assert_linear_at("storage::mcb::to_json_text", 12_000, table, |document| {
        to_json_text(document).unwrap().len()
    });
}
