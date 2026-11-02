//! One keyframed member with very many keys costs time near linear in their number (review
//! round 3): the merge of equal times was quadratic (80 000 keys read in 33.9 s).

use std::fmt::Write as _;

use bedrock_addon::families::fog::{FogSettings, Keyframed};

use super::common::loaded;
use super::support::{density, doc, pack, parse};

/// The `max_density` of the document's `volumetric.density.air`.
fn max_density(document: &FogSettings) -> Keyframed<f32> {
    let density = document.volumetric.as_ref().unwrap().density.as_ref();
    let air = density.unwrap().air.as_ref().unwrap();
    air.max_density.clone().unwrap()
}

/// A document whose `max_density` has `n` keyframes at the times `n - 1` down to 0.
fn keyframes(n: usize) -> String {
    let mut keys = String::new();
    for time in (0..n).rev() {
        write!(keys, r#""{time}": 0.5,"#).unwrap();
    }
    keys.pop();
    doc(
        "1.26.20",
        &density(&format!(r#""uniform": true, "max_density": {{{keys}}}"#)),
    )
}

/// Requires a load of `build(20 000)` to take under eight times a load of `build(5 000)`,
/// by the crate's one protocol: four times the keys are four times the time when
/// linear and sixteen times when quadratic. The text is built before the clock starts and the
/// document is checked after it stops.
#[track_caller]
fn near_linear(what: &str, build: impl Fn(usize) -> String, check: impl Fn(&FogSettings, usize)) {
    crate::cost::assert_linear_timed_at(what, 5_000, |n| {
        let text = build(n);
        let (document, took) = crate::cost::timed(|| loaded(parse(&text, &pack())));
        check(&document, n);
        took
    });
}

#[test]
fn many_keyframes_of_one_member_are_read_in_near_linear_time() {
    near_linear("keyframes of one member", keyframes, |document, n| {
        let Keyframed::Keys(keys) = max_density(document) else {
            panic!("a single value");
        };
        assert_eq!(keys.len(), n);
        assert_eq!(keys.first().map(|key| key.0), Some(0.0));
        assert!(keys.windows(2).all(|pair| pair[0].0 < pair[1].0));
    });
}

#[test]
fn equal_times_in_several_positions_merge_to_the_later_one() {
    let keys = r#""1": 0.1, "0": 0.2, "1.0": 0.3, "0.5": 0.4, "0.0": 0.5, "5e-1": 0.6, "2": 0.7, "0.50": 0.8, "00": 0.9"#;
    let text = doc(
        "1.26.20",
        &density(&format!(r#""uniform": true, "max_density": {{{keys}}}"#)),
    );
    let document = loaded(parse(&text, &pack()));
    assert_eq!(
        max_density(&document),
        Keyframed::Keys(vec![(0.0, 0.9), (0.5, 0.8), (1.0, 0.3), (2.0, 0.7)])
    );
}

/// A document whose two height members have `n` keyframes each, at the same times.
fn keyed_heights(n: usize) -> String {
    let mut zero = String::new();
    let mut max = String::new();
    for time in (0..n).rev() {
        write!(zero, r#""{time}": 100,"#).unwrap();
        write!(max, r#""{time}": 50,"#).unwrap();
    }
    zero.pop();
    max.pop();
    doc(
        "1.26.20",
        &density(&format!(
            r#""max_density": 0.5, "zero_density_height": {{{zero}}}, "max_density_height": {{{max}}}"#
        )),
    )
}

#[test]
fn many_keyframes_of_both_heights_are_compared_in_near_linear_time() {
    near_linear("keyframes of both heights", keyed_heights, |_, _| {});
}
