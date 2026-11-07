//! The typed entity components (F-ENTITY-COMP): each component read strictly and through the
//! lenient conversion, written back, and checked by its rules. Fixtures live under
//! `tests/fixtures/entity/entity/component/`.

use bedrock_addon_core::diagnostic::{Code, Diagnostics, Severity};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::json::{JsonSlice, Value, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::EntityComponent;

mod corpus;
mod health;
mod interact;
mod latest;
mod lifetime;
mod marker;
mod molang;
mod movement;
mod projectile;
mod push;
mod trade;
mod trigger;

/// The text of fixture `name`.
pub fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../addon/tests/fixtures/entity/entity/component/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Runs the table's `parse` of `T`'s row on `json`.
pub fn read_as<T>(json: &str, mode: ConvertMode) -> (Option<T>, Diagnostics)
where
    T: Component<Set = EntityComponent>,
{
    let (value, d) = read(T::NAME, json, mode);
    let typed = value.map(|v| {
        T::try_from_set(v).unwrap_or_else(|other| panic!("{} read as {other:?}", T::NAME))
    });
    (typed, d)
}

/// Runs the table's `parse` of row `key` on `json`.
pub fn read(key: &str, json: &str, mode: ConvertMode) -> (Option<EntityComponent>, Diagnostics) {
    read_with(key, json, mode, &LoadContext::detached())
}

/// [`read`] in the load context `cx`.
pub fn read_with(
    key: &str,
    json: &str,
    mode: ConvertMode,
    cx: &LoadContext,
) -> (Option<EntityComponent>, Diagnostics) {
    let meta = EntityComponent::TABLE
        .get(key)
        .unwrap_or_else(|| panic!("{key} has a row"));
    let mut d = Diagnostics::default();
    let value = (meta.parse)(
        JsonSlice {
            text: json,
            offset: 0,
        },
        mode,
        cx,
        &mut d,
    );
    (value, d)
}

/// A strict read that must succeed without an error.
pub fn strict<T>(json: &str) -> (T, Diagnostics)
where
    T: Component<Set = EntityComponent>,
{
    let (value, d) = read_as::<T>(json, ConvertMode::Strict);
    assert!(!d.has_errors(), "{}: {d:?}", T::NAME);
    (
        value.unwrap_or_else(|| panic!("{} was dropped: {d:?}", T::NAME)),
        d,
    )
}

/// A lenient read that must keep the component.
pub fn lenient<T>(json: &str) -> (T, Diagnostics)
where
    T: Component<Set = EntityComponent>,
{
    let (value, d) = read_as::<T>(json, ConvertMode::Lenient);
    assert!(!d.has_errors(), "the lenient conversion never errs: {d:?}");
    (
        value.unwrap_or_else(|| panic!("{} was dropped: {d:?}", T::NAME)),
        d,
    )
}

/// The value as JSON, written the way documents are.
pub fn write<T: for<'a> facet::Facet<'a>>(value: &T) -> Value {
    let text = to_json(value).expect("writes");
    facet_json::from_str::<Value>(&text).expect("the writer's JSON reads back")
}

/// [`write`] with the unknown members a read kept (paths below the value).
pub fn write_with<T: for<'a> facet::Facet<'a>>(
    value: &T,
    unknown: &bedrock_addon_core::document::Unknown,
) -> Value {
    let mut out = Vec::new();
    bedrock_addon_core::json::write_json_with(
        value,
        unknown,
        &mut out,
        bedrock_addon_core::json::WriteStyle::default(),
    )
    .expect("writes");
    facet_json::from_slice::<Value>(&out).expect("the writer's JSON reads back")
}

/// Strict read, write, strict read again: the two values are equal; returns the first.
pub fn round_trip<T>(json: &str) -> T
where
    T: Component<Set = EntityComponent> + for<'a> facet::Facet<'a> + PartialEq + std::fmt::Debug,
{
    let (first, _) = strict::<T>(json);
    let text = to_json(&first).expect("writes");
    let (second, _) = strict::<T>(&text);
    assert_eq!(first, second, "{} round trip through {text}", T::NAME);
    first
}

/// The JSON value of `text`.
pub fn json(text: &str) -> Value {
    facet_json::from_str::<Value>(text).expect("valid JSON")
}

/// Whether `d` holds a finding of `code` (by variant) at `path`.
pub fn has(d: &Diagnostics, code: fn(&Code) -> bool, severity: Severity, path: &str) -> bool {
    d.iter().any(|f| {
        code(&f.code) && f.severity == severity && f.location.json_path.to_string() == path
    })
}
