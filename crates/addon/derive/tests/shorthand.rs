//! `shorthand = scalar` at the door: `versioning::parse_component` reads a bare scalar of a
//! one-member payload as that member, and leaves an untagged enum's or a transparent newtype's
//! read as it was; the shape test a family holds over its rows.

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{Diagnostics, Severity, Span};
use bedrock_addon_core::json::{JsonSlice, NullRule, ScalarRules, Scalars, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{
    ShorthandRows, ShorthandShape, misshapen_shorthand_rows, shorthand_shape,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, ConvertMode, Family, Loaded, NoStable, UpgradeCx, VersionChain, parse_component,
};
use bedrock_addon_derive::Components;
use facet::Facet;

struct Toy;

enum ToyHistory {}

impl VersionChain<Toy> for ToyHistory {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _: JsonSlice<'_>,
        _: FormatVersion,
        _: &LoadContext,
        _: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Toy> {
        match self {}
    }
}

impl Family for Toy {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = Shorthands;
    type History = ToyHistory;

    fn components() -> &'static ComponentTable<Self> {
        Shorthands::TABLE
    }
}

/// `{"value": n}`, or `n`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Damage {
    #[facet(default)]
    pub value: i32,
}

/// `true`, `false` or `{"resistance": x}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum Explode {
    Enabled(bool),
    Params(Params),
}

#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Params {
    pub resistance: f32,
}

/// A bare number.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(transparent)]
pub struct Friction(pub f32);

#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = Toy, unknown = Unknown)]
pub enum Shorthands {
    #[component(name = "minecraft:damage", shorthand = scalar, parse = parse_component::<Damage, Shorthands>)]
    Damage(Damage),
    #[component(name = "minecraft:explode", shorthand = scalar, parse = parse_component::<Explode, Shorthands>)]
    Explode(Explode),
    #[component(name = "minecraft:friction", shorthand = scalar, parse = parse_component::<Friction, Shorthands>)]
    Friction(Friction),
    Unknown(Value),
}

/// The same payloads, no row with a shorthand: the reads as they are without it.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(from_only, unknown = Unknown)]
pub enum Plain {
    Damage(Damage),
    Explode(Explode),
    Friction(Friction),
    Unknown(Value),
}

/// `{"count": n}`, or `n`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Count {
    pub count: u8,
}

/// A payload of two members, which no bare scalar stands for.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Pair {
    pub a: i32,
    pub b: i32,
}

struct Other;

enum OtherHistory {}

impl VersionChain<Other> for OtherHistory {
    const VERSIONS: &'static [FormatVersion] = &[];

    fn parse(
        _: JsonSlice<'_>,
        _: FormatVersion,
        _: &LoadContext,
        _: &mut Diagnostics,
    ) -> Option<Self> {
        None
    }

    fn upgrade_to_end(self, _: &mut UpgradeCx<'_>) -> Loaded<Other> {
        match self {}
    }
}

impl Family for Other {
    const NAME: &'static str = "other";
    const PAYLOAD_KEY: &'static str = "minecraft:other";
    const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);
    type Latest = NoStable;
    type Beta = Infallible;
    type Component = Misshapen;
    type History = OtherHistory;

    fn components() -> &'static ComponentTable<Self> {
        Misshapen::TABLE
    }
}

#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = Other, unknown = Unknown)]
pub enum Misshapen {
    #[component(name = "minecraft:count", shorthand = scalar, parse = parse_component::<Count, Misshapen>)]
    Count(Count),
    #[component(name = "minecraft:pair", shorthand = scalar, parse = parse_component::<Pair, Misshapen>)]
    Pair(Pair),
    #[component(name = "minecraft:blast", shorthand = scalar, parse = parse_component::<Blast, Misshapen>)]
    Blast(Blast),
    #[component(name = "minecraft:maybe", shorthand = scalar, parse = parse_component::<Maybe, Misshapen>)]
    Maybe(Maybe),
    Unknown(Value),
}

/// `{"value": n}`, `{}` or `n`: a one-member payload whose member is optional.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Maybe {
    pub value: Option<i32>,
}

/// One finding as the test compares it.
type Told = (String, String, Severity, Option<Span>);

/// `text` placed at offset 30 of a file, read as `T` of `C` under `rules`.
fn read<T, C>(text: &str, rules: ScalarRules) -> (Option<C>, Vec<Told>)
where
    T: for<'a> Facet<'a> + Into<C>,
    C: ShorthandRows,
{
    read_under::<T, C>(text, rules, None)
}

/// [`read`] under the slice's null rule `null`.
fn read_under<T, C>(
    text: &str,
    rules: ScalarRules,
    null: Option<NullRule>,
) -> (Option<C>, Vec<Told>)
where
    T: for<'a> Facet<'a> + Into<C>,
    C: ShorthandRows,
{
    let file = format!("{}{text}", " ".repeat(30));
    let slice = JsonSlice {
        text: &file[30..],
        offset: 30,
    };
    let mut cx = LoadContext::detached();
    cx.scalar_rules = rules;
    cx.null_rule = null;
    let mut found = Diagnostics::default();
    let value = parse_component::<T, C>(slice, ConvertMode::Strict, &cx, &mut found);
    let told = found
        .iter()
        .map(|d| {
            (
                d.location.json_path.to_string(),
                d.code.id().to_owned(),
                d.severity,
                d.location.span,
            )
        })
        .collect();
    (value, told)
}

const STRICT: ScalarRules = ScalarRules::PLAIN.with_scalars(Scalars::Strict);

fn at(offset: u32, len: u32) -> Span {
    Span { offset, len }
}

#[test]
fn every_shorthand_row_of_the_toy_is_one_of_the_shapes() {
    assert_eq!(
        Shorthands::SHORTHAND_PAYLOADS
            .iter()
            .map(|(key, shape)| (*key, shorthand_shape(shape)))
            .collect::<Vec<_>>(),
        [
            (
                "minecraft:damage",
                Some(ShorthandShape::OneField { key: "value" })
            ),
            ("minecraft:explode", Some(ShorthandShape::Untagged)),
            ("minecraft:friction", Some(ShorthandShape::Transparent)),
        ]
    );
    assert!(misshapen_shorthand_rows::<Shorthands>().is_empty());
    assert_eq!(misshapen_shorthand_rows::<Misshapen>(), ["minecraft:pair"]);
    assert!(
        Plain::SHORTHAND_PAYLOADS.is_empty(),
        "a from_only enum has no rows"
    );
}

#[test]
fn a_bare_scalar_of_a_one_member_payload_is_that_member() {
    let damage = |n| Some(Shorthands::Damage(Damage { value: n }));
    assert_eq!(read::<Damage, Shorthands>("5", STRICT), (damage(5), vec![]));
    assert_eq!(
        read::<Damage, Shorthands>(" /* bare */ 7 // trailing", STRICT),
        (damage(7), vec![])
    );
    // The object form reads as it does without the shorthand.
    let object = r#"{"value": 5, "x": 1}"#;
    let (value, told) = read::<Damage, Shorthands>(object, STRICT);
    assert_eq!(value, damage(5));
    let (_, plain) = read::<Damage, Plain>(object, STRICT);
    assert_eq!(told, plain);
    // What is told of the member is told at the component, the scalar's span the file's.
    assert_eq!(
        read::<Damage, Shorthands>(r#""5""#, ScalarRules::PLAIN),
        (
            damage(5),
            vec![(
                String::new(),
                "core/invalid_type".to_owned(),
                Severity::Error,
                None
            )]
        )
    );
    let (value, told) = read::<Damage, Shorthands>("true", STRICT);
    assert_eq!(value, None);
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!((told[0].0.as_str(), told[0].2), ("", Severity::Error));
    assert_eq!(told[0].3, Some(at(30, 4)), "{told:?}");
    // Without the row's shorthand a bare scalar is no struct.
    let (value, told) = read::<Damage, Plain>("5", STRICT);
    assert_eq!(value, None);
    assert_eq!(told.len(), 1, "{told:?}");
}

#[test]
fn an_untagged_enum_and_a_newtype_read_their_scalar_as_without_the_shorthand() {
    for text in ["true", "5", r#"{"resistance": 2}"#, r#""no""#, "null"] {
        let (value, told) = read::<Explode, Shorthands>(text, STRICT);
        let (plain_value, plain_told) = read::<Explode, Plain>(text, STRICT);
        assert_eq!(told, plain_told, "{text}");
        assert_eq!(
            value.map(|v| format!("{v:?}")),
            plain_value.map(|v| format!("{v:?}").replace("Plain", "Shorthands")),
            "{text}"
        );
    }
    for text in ["0.5", r#""x""#] {
        let (value, told) = read::<Friction, Shorthands>(text, STRICT);
        let (plain_value, plain_told) = read::<Friction, Plain>(text, STRICT);
        assert_eq!(told, plain_told, "{text}");
        assert_eq!(value.is_some(), plain_value.is_some(), "{text}");
    }
}

/// `{"mode": …}`, or the mode bare: a one-member payload whose member is an untagged enum.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Blast {
    pub mode: Explode,
}

#[test]
fn a_bare_scalar_no_arm_of_the_member_takes_is_refused_at_the_component_without_a_span() {
    let (value, told) = read::<Blast, Misshapen>(r#""no""#, STRICT);
    assert_eq!(value, None);
    assert_eq!(
        told,
        [(
            String::new(),
            "core/invalid_type".to_owned(),
            Severity::Error,
            None
        )],
        "a refusal is told at the component and, as every refusal at the door, without a span"
    );
    assert_eq!(
        read::<Blast, Misshapen>("true", STRICT).0,
        Some(Misshapen::Blast(Blast {
            mode: Explode::Enabled(true)
        }))
    );
}

fn told(path: &str, code: &str, severity: Severity, span: Option<Span>) -> Told {
    (path.to_owned(), code.to_owned(), severity, span)
}

#[test]
fn a_bare_null_follows_the_null_rule_as_the_object_form_does() {
    let damage = |n| Some(Shorthands::Damage(Damage { value: n }));
    let rejected = Some(NullRule::Rejected);
    let absent = Some(NullRule::Absent);
    // The object form, for the parity: the typed read fails at the member's key.
    assert_eq!(
        read_under::<Damage, Shorthands>(r#"{"value": null}"#, STRICT, rejected),
        (
            None,
            vec![told(
                "value",
                "core/invalid_type",
                Severity::Error,
                Some(at(31, 7))
            )]
        )
    );
    assert_eq!(
        read_under::<Damage, Shorthands>(r#"{"value": null}"#, STRICT, absent),
        (
            damage(0),
            vec![told("value", "core/value_rewritten", Severity::Info, None)]
        )
    );
    // The bare `null`: the same, at the component and spanning the text.
    assert_eq!(
        read_under::<Damage, Shorthands>("null", STRICT, rejected),
        (
            None,
            vec![told(
                "",
                "core/invalid_type",
                Severity::Error,
                Some(at(30, 4))
            )]
        ),
        "under `Rejected` a bare null is a type error, as the member's null is"
    );
    assert_eq!(
        read_under::<Damage, Shorthands>("null", STRICT, absent),
        (
            damage(0),
            vec![told("", "core/value_rewritten", Severity::Info, None)]
        ),
        "under `Absent` a bare null is the member left out"
    );
}

#[test]
fn a_one_member_payload_whose_member_is_optional_reads_a_bare_scalar_and_a_bare_null() {
    let maybe = |value| Some(Misshapen::Maybe(Maybe { value }));
    assert_eq!(
        read::<Maybe, Misshapen>("5", STRICT),
        (maybe(Some(5)), vec![])
    );
    assert_eq!(
        read_under::<Maybe, Misshapen>("null", STRICT, Some(NullRule::Rejected)),
        (
            None,
            vec![told(
                "",
                "core/invalid_type",
                Severity::Error,
                Some(at(30, 4))
            )]
        )
    );
    assert_eq!(
        read_under::<Maybe, Misshapen>("null", STRICT, Some(NullRule::Absent)),
        (
            maybe(None),
            vec![told("", "core/value_rewritten", Severity::Info, None)]
        )
    );
}

#[test]
fn under_default_and_tell_a_wrong_typed_bare_scalar_keeps_the_component_with_its_default() {
    let rules = STRICT.with_default_and_tell(true);
    let (value, found) = read::<Damage, Shorthands>("true", rules);
    assert_eq!(value, Some(Shorthands::Damage(Damage { value: 0 })));
    // Told as the member's defaulted scalar is, at the component.
    assert_eq!(
        found,
        [told("", "core/invalid_value", Severity::Info, None)]
    );
}
