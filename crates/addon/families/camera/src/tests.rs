//! Unit tests of the camera module's private parts.

use super::texts;

#[test]
fn every_formatted_text_has_as_many_placeholders_as_arguments() {
    for (template, arity) in texts::FORMATTED {
        assert_eq!(template.matches("{}").count(), *arity, "{template}");
    }
}

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
    assert_eq!(all.len(), super::Easing::NAMES.len());
    for (easing, name) in all.into_iter().zip(super::Easing::NAMES) {
        assert_eq!(easing.as_str(), *name);
    }
}

#[test]
fn splines_iterate_by_reference() {
    let splines = super::Splines::default();
    assert_eq!((&splines).into_iter().count(), 0);
}
