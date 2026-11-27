//! The writer's cost on a large structure that keeps its file's layout.

use bedrock_addon_family_structure::{BlockPositionData, Palette, Structure, TickQueueEntry};
use nbtx::Value;

use super::common::loads;

/// A structure of `count` block positions read from a file whose root is not in vanilla's
/// order, so its tree is kept and every compound is laid over the file's; and that file.
fn kept_layout(count: i32) -> (Structure, Vec<u8>) {
    let mut built = Structure::new([0, 0, 0]);
    let positions = (0..count)
        .map(|index| {
            let data = BlockPositionData {
                tick_queue_data: Some(vec![TickQueueEntry {
                    tick_delay: Some(index),
                    ..TickQueueEntry::default()
                }]),
                ..BlockPositionData::default()
            };
            (index, data)
        })
        .collect();
    built.structure.palette.insert(
        "default".to_owned(),
        Palette {
            block_position_data: positions,
            ..Palette::default()
        },
    );
    let vanilla = built.to_bytes().unwrap();
    let Value::Compound(mut root) = nbtx::from_le_bytes::<Value>(&mut vanilla.as_slice()).unwrap()
    else {
        panic!("a compound");
    };
    let version = root.shift_remove(b"format_version".as_slice()).unwrap();
    root.insert("format_version".into(), version);
    let input = nbtx::to_le_bytes(&Value::Compound(root)).unwrap();
    assert!(input != vanilla, "the file is not in vanilla's layout");
    (loads(&input).into_value(), input)
}

#[test]
fn writing_a_kept_layout_is_linear_in_the_members_of_a_compound() {
    // The same write at n and at 4n members of one compound, by the crate's one protocol.
    // A linear overlay takes four times as long; one where each member taken shifts
    // the rest takes sixteen times. The ratio does not depend on how fast or how busy the
    // machine is, where a ceiling in seconds does.
    const MEMBERS: usize = 10_000;
    let members = |count: usize| i32::try_from(count).expect("a small count");
    crate::cost::assert_linear_at(
        "Structure::to_bytes, a kept layout",
        MEMBERS,
        |count| kept_layout(members(count)),
        |(structure, input)| {
            let written = structure.to_bytes().unwrap();
            assert!(written == *input, "the kept layout is written back");
        },
    );
    let (large, _) = kept_layout(members(2 * MEMBERS));
    // A change among them is still written.
    let mut changed = large;
    changed.structure.palette["default"]
        .block_position_data
        .remove(&members(MEMBERS));
    let again = loads(&changed.to_bytes().unwrap()).into_value();
    assert!(again == changed, "the changed structure round trips");
}
