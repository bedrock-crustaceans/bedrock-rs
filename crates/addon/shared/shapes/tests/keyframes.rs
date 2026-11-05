//! `Keyframes`: the accessors the fog and the deferred settings read their keyed values by.

use bedrock_addon_shared_shapes::{Keyframe, Keyframes};

#[test]
fn a_single_value_is_one_keyframe_at_noon() {
    let single = Keyframes::Single(2.0_f32);
    assert_eq!(single.single(), Some(&2.0));
    assert_eq!(single.values(), [(None, &2.0)]);
    assert_eq!(
        single.frames(),
        [Keyframe {
            time: 0.0,
            value: 2.0
        }]
    );
    assert_eq!(Keyframes::at_noon(2.0_f32).frames(), single.frames());
}

#[test]
fn keys_keep_their_times_and_map_their_values() {
    let keys = Keyframes::Keys(vec![
        Keyframe {
            time: 0.0,
            value: 1_u8,
        },
        Keyframe {
            time: 0.5,
            value: 3,
        },
    ]);
    assert_eq!(keys.single(), None);
    assert_eq!(keys.values(), [(Some(0.0), &1), (Some(0.5), &3)]);
    assert_eq!(Keyframes::from_pairs([(0.0, 1_u8), (0.5, 3)]), keys);
    let doubled = keys.map(|v| f32::from(v) * 2.0);
    assert_eq!(
        doubled.frames(),
        [
            Keyframe {
                time: 0.0,
                value: 2.0
            },
            Keyframe {
                time: 0.5,
                value: 6.0
            }
        ]
    );
}
