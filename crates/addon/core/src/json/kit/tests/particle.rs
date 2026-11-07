//! Particle's description reader in the kit's terms: `Description` and
//! `BasicRenderParameters` as `families/particle/src/lib.rs` declares them (`req` members,
//! unknown members told and not kept, `read::render_parameters` after), read by
//! `families/particle/src/read.rs` (`Members`, `Opt::NONE`, `req`, `Members::extra`,
//! `Reader::wrong_type` and `Reader::missing`) under each of its two eras.
//!
//! The 1.20.80 era is the kit's own reading: the strict document, whose unknown members the
//! raw reader tells as warnings (the pipeline's strict mask makes them errors). The legacy
//! parser's era keeps particle's own lines through [`Read::push`]: a value of the wrong type is
//! the crate's `InvalidValue` information, an absent member says nothing, an unknown member is
//! an information.

use super::*;
use crate::diagnostic::codes as core_codes;

/// Which particle reader reads the document.
#[derive(Clone, Copy, PartialEq)]
enum Era {
    Legacy,
    Cereal,
}

/// A read under `era`.
fn reader(era: Era, found: &mut Diagnostics) -> Read<'_> {
    match era {
        Era::Cereal => Read::new(Reading::DOCUMENT.with_unknown(UnknownMask::Warn), found),
        Era::Legacy => Read::new(
            Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell),
            found,
        )
        .unknown_severity(Severity::Info),
    }
}

/// `Reader::wrong_type(None)`.
fn wrong_type(era: Era, read: &mut Read<'_>, node: &Node<'_>) {
    match era {
        Era::Cereal => read.wrong_type(node),
        Era::Legacy => read.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(node.path().clone()),
        ),
    }
}

/// `<String as Node>::read`.
fn string(era: Era, read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let text = node
        .value()
        .as_string()
        .map(|text| text.as_str().to_owned());
    if text.is_none() {
        wrong_type(era, read, node);
    }
    text
}

/// `req(m, key, Opt::NONE, r)`: the member, read by `each`; its fallback when it is absent
/// (the 1.20.80 reader's `MissingRequiredField`, the legacy parser's silence) or unusable.
fn req<'v, T: Default>(
    era: Era,
    read: &mut Read<'_>,
    members: &Members<'v>,
    key: &str,
    each: impl FnOnce(Era, &mut Read<'_>, &Node<'v>) -> Option<T>,
) -> T {
    let node = match era {
        Era::Cereal => read.required(members, key),
        Era::Legacy => read.opt(members, key),
    };
    node.and_then(|node| each(era, read, &node))
        .unwrap_or_default()
}

/// The members of a `Fields` object; `None` (and the wrong type) for another value.
fn object<'v>(era: Era, read: &mut Read<'_>, node: &Node<'v>) -> Option<Members<'v>> {
    if era == Era::Legacy && node.value().as_object().is_none() {
        wrong_type(era, read, node);
        return None;
    }
    read.object(node)
}

/// `BasicRenderParameters`: `material` and `texture`, then the length rule on `texture`.
fn render_parameters(era: Era, read: &mut Read<'_>, node: &Node<'_>) -> Option<(String, String)> {
    let members = object(era, read, node)?;
    let material = req(era, read, &members, "material", string);
    let texture = req(era, read, &members, "texture", string);
    read.unknown(&members, &["material", "texture"], Disposition::Keep);
    if texture.chars().count() > 256 {
        read.push(
            Diagnostic::of(&core_codes::INVALID_VALUE).at(members.node().path().child("texture")),
        );
    }
    Some((material, texture))
}

/// `Description`: `identifier` and `basic_render_parameters`.
fn description(
    era: Era,
    read: &mut Read<'_>,
    node: &Node<'_>,
) -> Option<(String, (String, String))> {
    let members = object(era, read, node)?;
    let identifier = req(era, read, &members, "identifier", string);
    let parameters = req(
        era,
        read,
        &members,
        "basic_render_parameters",
        render_parameters,
    );
    read.unknown(
        &members,
        &["identifier", "basic_render_parameters"],
        Disposition::Keep,
    );
    Some((identifier, parameters))
}

/// The fixtures, the descriptions of `families/particle`'s rule tests.
fn fixtures() -> Vec<String> {
    let long = "t".repeat(300);
    vec![
        r#"{"identifier": "demo:p", "basic_render_parameters": {"material": "m", "texture": "t"}}"#.to_owned(),
        r#"{"identifier": 5, "basic_render_parameters": {"material": 1, "texture": "t", "zz": 0}, "x": 1}"#.to_owned(),
        r"{}".to_owned(),
        r#"{"basic_render_parameters": []}"#.to_owned(),
        r#"{"identifier": "a", "basic_render_parameters": {"material": "m"}}"#.to_owned(),
        format!(r#"{{"identifier": "a", "basic_render_parameters": {{"material": "m", "texture": "{long}"}}}}"#),
        r#"{"identifier": null, "basic_render_parameters": {"material": null, "texture": true}}"#.to_owned(),
        r"[]".to_owned(),
    ]
}

fn reads_as(era: Era, expected: &[(bool, &[&str])]) {
    let fixtures = fixtures();
    let fixtures: Vec<&str> = fixtures.iter().map(String::as_str).collect();
    assert_reads_as(&fixtures, expected, |text, found| {
        let json = value(text);
        let node = Node::new(&json, at(&["particle_effect", "description"]));
        let mut read = reader(era, found);
        description(era, &mut read, &node).is_some()
    });
}

#[test]
fn particle_s_1_20_80_description_reader_reads_as_the_kit() {
    reads_as(Era::Cereal, CEREAL);
}

#[test]
fn particle_s_legacy_description_reader_reads_as_the_kit_with_its_own_lines() {
    reads_as(Era::Legacy, LEGACY);
}

/// What `<Description as Node>::read` tells under `Era::Cereal`.
const CEREAL: &[(bool, &[&str])] = &[
    // {"identifier": "demo:p", "basic_render_parameters": {"material": "m", "texture": "t"}}
    (true, &[]),
    // {"identifier": 5, "basic_render_parameters": {"material": 1, "texture": "t", "zz": 0}, "x": 1
    (
        true,
        &[
            r"Error|particle_effect/description/identifier|core/invalid_type|None|false",
            r"Error|particle_effect/description/basic_render_parameters/material|core/invalid_type|None|false",
            r"Warning|particle_effect/description/basic_render_parameters/zz|core/unrecognized_field|None|false",
            r"Warning|particle_effect/description/x|core/unrecognized_field|None|false",
        ],
    ),
    // {}
    (
        true,
        &[
            r"Error|particle_effect/description/identifier|core/missing_required_field|None|false",
            r"Error|particle_effect/description/basic_render_parameters|core/missing_required_field|None|false",
        ],
    ),
    // {"basic_render_parameters": []}
    (
        true,
        &[
            r"Error|particle_effect/description/identifier|core/missing_required_field|None|false",
            r"Error|particle_effect/description/basic_render_parameters|core/invalid_type|None|false",
        ],
    ),
    // {"identifier": "a", "basic_render_parameters": {"material": "m"}}
    (
        true,
        &[
            r"Error|particle_effect/description/basic_render_parameters/texture|core/missing_required_field|None|false",
        ],
    ),
    // {"identifier": "a", "basic_render_parameters": {"material": "m", "texture": "tttttttttttttttt
    (
        true,
        &[
            r"Error|particle_effect/description/basic_render_parameters/texture|core/invalid_value|None|false",
        ],
    ),
    // {"identifier": null, "basic_render_parameters": {"material": null, "texture": true}}
    (
        true,
        &[
            r"Error|particle_effect/description/identifier|core/invalid_type|None|false",
            r"Error|particle_effect/description/basic_render_parameters/material|core/invalid_type|None|false",
            r"Error|particle_effect/description/basic_render_parameters/texture|core/invalid_type|None|false",
        ],
    ),
    // []
    (
        false,
        &[r"Error|particle_effect/description|core/invalid_type|None|false"],
    ),
];

/// What `<Description as Node>::read` tells under `Era::Legacy`.
const LEGACY: &[(bool, &[&str])] = &[
    // {"identifier": "demo:p", "basic_render_parameters": {"material": "m", "texture": "t"}}
    (true, &[]),
    // {"identifier": 5, "basic_render_parameters": {"material": 1, "texture": "t", "zz": 0}, "x": 1
    (
        true,
        &[
            r"Info|particle_effect/description/identifier|core/invalid_value|None|false",
            r"Info|particle_effect/description/basic_render_parameters/material|core/invalid_value|None|false",
            r"Info|particle_effect/description/basic_render_parameters/zz|core/unrecognized_field|None|false",
            r"Info|particle_effect/description/x|core/unrecognized_field|None|false",
        ],
    ),
    // {}
    (true, &[]),
    // {"basic_render_parameters": []}
    (
        true,
        &[
            r"Info|particle_effect/description/basic_render_parameters|core/invalid_value|None|false",
        ],
    ),
    // {"identifier": "a", "basic_render_parameters": {"material": "m"}}
    (true, &[]),
    // {"identifier": "a", "basic_render_parameters": {"material": "m", "texture": "tttttttttttttttt
    (
        true,
        &[
            r"Error|particle_effect/description/basic_render_parameters/texture|core/invalid_value|None|false",
        ],
    ),
    // {"identifier": null, "basic_render_parameters": {"material": null, "texture": true}}
    (
        true,
        &[
            r"Info|particle_effect/description/identifier|core/invalid_value|None|false",
            r"Info|particle_effect/description/basic_render_parameters/material|core/invalid_value|None|false",
            r"Info|particle_effect/description/basic_render_parameters/texture|core/invalid_value|None|false",
        ],
    ),
    // []
    (
        false,
        &[r"Info|particle_effect/description|core/invalid_value|None|false"],
    ),
];
