//! The `schema::` keys a struct states are checked by facet's derive, and the three stored ones
//! (`since_format`, `since_engine`, `unknown`) are read back from the `Shape` the way the
//! JSON door reads them. A facet bump that changes how an extension attribute is stored fails
//! here first.

use bedrock_addon_core::schema;
use facet::{Facet, Field, Shape, Type, UserType};

#[derive(Facet)]
#[facet(schema::unknown = "report")]
struct Toy {
    #[facet(schema::since_format = "1.21.60", schema::since_engine = "1.20.0")]
    gated: Option<i32>,
    #[facet(default = 2.0, schema::min = 0.0, schema::below = 10.0)]
    radius: f32,
    #[facet(schema::fallback = 1, schema::above = lower)]
    upper: Option<i32>,
    lower: i32,
    #[facet(schema::lowercase, schema::unknown = "keep")]
    name: String,
    #[facet(schema::molang = SPEC, schema::molang_with = walk)]
    cond: Option<String>,
    #[facet(schema::foreign_event)]
    on_spawn: Option<String>,
}

fn fields(shape: &'static Shape) -> &'static [Field] {
    match shape.ty {
        Type::User(UserType::Struct(st)) => st.fields,
        _ => panic!("not a struct"),
    }
}

fn field(name: &str) -> &'static Field {
    fields(Toy::SHAPE)
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no field {name}"))
}

fn stored(field: &Field, key: &str) -> Option<&'static str> {
    field
        .get_attr(Some("schema"), key)
        .map(|attr| *attr.get_as::<&'static str>().expect("a string attribute"))
}

#[test]
fn stored_keys_are_read_back_from_the_field() {
    assert_eq!(stored(field("gated"), "since_format"), Some("1.21.60"));
    assert_eq!(stored(field("gated"), "since_engine"), Some("1.20.0"));
    assert_eq!(stored(field("name"), "unknown"), Some("keep"));
    assert_eq!(stored(field("radius"), "since_format"), None);
}

#[test]
fn a_container_key_is_in_the_shape_attributes() {
    let unknown = Toy::SHAPE
        .attributes
        .iter()
        .find(|a| a.ns() == Some("schema") && a.key() == "unknown")
        .expect("the container's unknown key");
    assert_eq!(*unknown.get_as::<&'static str>().unwrap(), "report");
}

#[test]
fn compile_time_keys_and_markers_are_present_but_carry_no_value() {
    for (name, key) in [
        ("radius", "min"),
        ("radius", "below"),
        ("upper", "fallback"),
        ("upper", "above"),
        ("name", "lowercase"),
        ("cond", "molang"),
        ("cond", "molang_with"),
        ("on_spawn", "foreign_event"),
    ] {
        let attr = field(name)
            .get_attr(Some("schema"), key)
            .unwrap_or_else(|| panic!("{name} carries schema::{key}"));
        assert!(
            attr.get_as::<()>().is_some(),
            "schema::{key} on {name} is stored as a unit marker"
        );
    }
    assert!(field("lower").get_attr(Some("schema"), "min").is_none());
}
