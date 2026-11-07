//! Unit tests of the camera module's private parts.

#[test]
fn easing_as_str_follows_the_names_table() {
    use super::Easing::*;
    let all = [
        Linear,
        Spring,
        InQuad,
        OutQuad,
        InOutQuad,
        InCubic,
        OutCubic,
        InOutCubic,
        InQuart,
        OutQuart,
        InOutQuart,
        InQuint,
        OutQuint,
        InOutQuint,
        InSine,
        OutSine,
        InOutSine,
        InExpo,
        OutExpo,
        InOutExpo,
        InCirc,
        OutCirc,
        InOutCirc,
        InBounce,
        OutBounce,
        InOutBounce,
        InBack,
        OutBack,
        InOutBack,
        InElastic,
        OutElastic,
        InOutElastic,
    ];
    // The game's 32 names of `easing_function`, written out: the names table is derived from
    // the variants, so it is checked against the spelling the game reads.
    assert_eq!(
        super::Easing::NAMES,
        [
            "linear",
            "spring",
            "in_quad",
            "out_quad",
            "in_out_quad",
            "in_cubic",
            "out_cubic",
            "in_out_cubic",
            "in_quart",
            "out_quart",
            "in_out_quart",
            "in_quint",
            "out_quint",
            "in_out_quint",
            "in_sine",
            "out_sine",
            "in_out_sine",
            "in_expo",
            "out_expo",
            "in_out_expo",
            "in_circ",
            "out_circ",
            "in_out_circ",
            "in_bounce",
            "out_bounce",
            "in_out_bounce",
            "in_back",
            "out_back",
            "in_out_back",
            "in_elastic",
            "out_elastic",
            "in_out_elastic",
        ]
    );
    assert_eq!(all.len(), super::Easing::NAMES.len());
    for (easing, name) in all.into_iter().zip(super::Easing::NAMES) {
        assert_eq!(easing.name(), *name);
    }
}

#[test]
fn splines_iterate_by_reference() {
    let splines = super::Splines::default();
    assert_eq!((&splines).into_iter().count(), 0);
}
