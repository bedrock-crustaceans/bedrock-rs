//! `shorthand = scalar` at the door: `versioning::parse_component` reads a bare scalar of a
//! one-member payload as that member, and leaves an untagged enum's or a transparent newtype's
//! read as it was; the shape test a family holds over its rows.

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{Diagnostics, Severity, Span};
use bedrock_addon_core::json::{JsonSlice, ScalarRules, Scalars, Value};
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

enum ToyAny {}

impl VersionChain<Toy> for ToyAny {
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
    type Any = ToyAny;

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

enum OtherAny {}

impl VersionChain<Other> for OtherAny {
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
    type Any = OtherAny;

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
    Unknown(Value),
}

/// One finding as the test compares it.
type Told = (String, String, Severity, Option<Span>);

/// `text` placed at offset 30 of a file, read as `T` of `C` under `rules`.
fn read<T, C>(text: &str, rules: ScalarRules) -> (Option<C>, Vec<Told>)
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
    let mut found = Diagnostics::default();
    let value = parse_component::<T, C>(slice, ConvertMode::Strict, &cx, &mut found);
    let told = found
        .iter()
        .map(|d| {
            (
                d.location.json_path.to_string(),
                d.code.name().to_owned(),
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
                "InvalidType".to_owned(),
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
