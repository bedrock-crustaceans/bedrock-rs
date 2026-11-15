//! Expansion tests for `#[derive(Schema)]`: the plan's `Boss`, `BreakDoor`, `EquipmentSlot`
//! and `Friction`, and toys for each attribute. A test checks what the derive emits (rows,
//! `check` under the slice's rule, the Molang walk, the trigger members, `Default`, the
//! getters, a string enum's names) and that the JSON names it emits are the ones facet's own
//! `Shape` carries.

use bedrock_addon_core::diagnostic::cereal::NumericRange;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity, codes};
use bedrock_addon_core::molang::{
    Molang, MolangFieldSpec, MolangVersionSource, visit_fields_under,
};
use bedrock_addon_core::schema::{
    self, Bound, BoundOp, FieldKind, FieldRow, Limit, Lit, Schema, SchemaCx, UnknownRule,
};
use bedrock_addon_core::shared::EventTrigger;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{MemberFailure, Reading};
use facet::{Facet, Shape, Type, UserType};
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Boss {
    #[facet(default = 55)]
    pub hud_range: i32,
    #[facet(default = true)]
    pub should_darken_sky: bool,
    #[facet(default = 2.0, schema::min = 0.0)]
    pub radius: f32,
    pub on_x: Option<EventTrigger>,
    pub cond: Option<Molang>,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct BreakDoor {
    #[facet(schema::fallback = 0)]
    pub priority: Option<i32>,
    pub control_flags: Option<Vec<String>>,
    #[facet(schema::fallback = 1.0, schema::min = 0.0)]
    pub speed: Option<f32>,
    pub slot: Option<EquipmentSlot>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(transparent)]
pub struct Friction(#[facet(default = 0.4, schema::min = 0.0, schema::max = 0.9)] pub f32);

fn check<T: Schema>(value: &mut T, failure: MemberFailure) -> (Diagnostics, bool) {
    let mut out = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(failure);
    let failed = {
        let mut cx = SchemaCx::new(reading, JsonPath::new(), &mut out);
        value.check(&mut cx);
        cx.failed()
    };
    (out, failed)
}

fn paths(out: &Diagnostics) -> Vec<(String, Severity)> {
    out.iter()
        .map(|d| (d.location.json_path.to_string(), d.severity))
        .collect()
}

fn boss() -> Boss {
    Boss {
        hud_range: 55,
        should_darken_sky: true,
        radius: 2.0,
        on_x: Some(EventTrigger::new("x")),
        cond: Some(Molang::Const(1.0)),
        color: Color { r: 1, g: 2, b: 3 },
    }
}

#[test]
fn boss_rows_state_each_fields_kind_default_and_bound() {
    let rows = Boss::FIELDS;
    let names: Vec<(&str, FieldKind)> = rows.iter().map(|r| (r.json, r.kind)).collect();
    assert_eq!(
        names,
        [
            ("hud_range", FieldKind::Filled),
            ("should_darken_sky", FieldKind::Filled),
            ("radius", FieldKind::Filled),
            ("on_x", FieldKind::Optional),
            ("cond", FieldKind::Optional),
            ("color", FieldKind::Required),
        ]
    );
    assert_eq!(rows[0].default, Some(Lit::Int(55)));
    assert_eq!(rows[1].default, Some(Lit::Bool(true)));
    assert_eq!(
        rows[2],
        FieldRow {
            default: Some(Lit::Float(2.0)),
            bounds: &[Bound {
                op: BoundOp::Min,
                limit: Limit::Num(0.0)
            }],
            ..FieldRow::plain("radius", "radius", "f32", FieldKind::Filled)
        }
    );
    assert_eq!(rows[5].ty, "Color");
}

#[test]
fn a_rows_type_is_printed_as_rust_writes_it() {
    let types: Vec<&str> = BreakDoor::FIELDS.iter().map(|r| r.ty).collect();
    assert_eq!(
        types,
        [
            "Option<i32>",
            "Option<Vec<String>>",
            "Option<f32>",
            "Option<EquipmentSlot>"
        ]
    );
    assert_eq!(Outer::FIELDS[1].ty, "IndexMap<String, Inner>");
}

#[test]
fn a_broken_bound_fails_the_owner_under_refuse_and_resets_with_an_information_under_default_and_tell()
 {
    let mut refused = Boss {
        radius: -1.0,
        ..boss()
    };
    let (out, failed) = check(&mut refused, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(refused.radius, -1.0, "the value stays");
    assert_eq!(paths(&out), [("radius".to_owned(), Severity::Error)]);

    let mut told = Boss {
        radius: -1.0,
        ..boss()
    };
    let (out, failed) = check(&mut told, MemberFailure::DefaultAndTell);
    assert!(!failed);
    assert_eq!(told.radius, 2.0, "the field's facet default");
    assert_eq!(paths(&out), [("radius".to_owned(), Severity::Info)]);

    let mut fine = boss();
    let (out, failed) = check(&mut fine, MemberFailure::Refuse);
    assert!(out.is_empty() && !failed);
}

#[test]
fn boss_walks_its_molang_field_under_the_default_spec() {
    const FAMILY: MolangFieldSpec =
        MolangFieldSpec::DEFAULT.version(MolangVersionSource::FormatVersion);
    let mut value = boss();
    let mut seen = Vec::new();
    visit_fields_under(&mut value, &JsonPath::new(), &FAMILY, |field| {
        seen.push((field.path.to_string(), *field.spec));
    });
    assert_eq!(seen, [("cond".to_owned(), FAMILY)]);
}

#[test]
fn an_event_trigger_field_is_a_trigger_member() {
    assert_eq!(Boss::triggers(), ["on_x"]);
}

#[test]
fn a_struct_with_no_required_field_gets_default_and_getters_answer_the_fallback() {
    let door = BreakDoor::default();
    assert_eq!(
        door,
        BreakDoor {
            priority: None,
            control_flags: None,
            speed: None,
            slot: None,
        }
    );
    assert_eq!(door.priority(), 0);
    assert_eq!(door.speed(), 1.0);
    let set = BreakDoor {
        priority: Some(3),
        speed: Some(0.5),
        ..door
    };
    assert_eq!((set.priority(), set.speed()), (3, 0.5));
    assert_eq!(BreakDoor::FIELDS[0].fallback, Some(Lit::Int(0)));
    assert_eq!(BreakDoor::FIELDS[0].kind, FieldKind::Optional);
}

#[test]
fn an_option_bound_resets_to_absence_and_an_absent_member_is_not_checked() {
    let mut door = BreakDoor {
        speed: Some(-2.0),
        ..BreakDoor::default()
    };
    let (out, _) = check(&mut door, MemberFailure::DefaultAndTell);
    assert_eq!(
        door.speed, None,
        "the fallback is never written into the data"
    );
    assert_eq!(door.speed(), 1.0, "absence stands for the fallback");
    assert_eq!(paths(&out), [("speed".to_owned(), Severity::Info)]);

    let mut absent = BreakDoor::default();
    let (out, _) = check(&mut absent, MemberFailure::Refuse);
    assert!(out.is_empty());
}

#[test]
fn a_string_enum_names_its_values_through_rename_all() {
    assert_eq!(EquipmentSlot::ALL.len(), 5);
    assert_eq!(
        EquipmentSlot::NAMES,
        ["head", "chest", "legs", "feet", "main_hand"]
    );
    assert_eq!(EquipmentSlot::MainHand.name(), "main_hand");
    assert_eq!(EquipmentSlot::MainHand.to_string(), "main_hand");
    assert_eq!(EquipmentSlot::from_name("feet"), Some(EquipmentSlot::Feet));
    assert_eq!(EquipmentSlot::from_name("Feet"), None);
    let error = EquipmentSlot::parse_name("tail").unwrap_err();
    assert_eq!(
        error.to_string(),
        "expected one of: head, chest, legs, feet, main_hand, found `tail`"
    );
    let shape: Vec<&str> = match EquipmentSlot::SHAPE.ty {
        Type::User(UserType::Enum(e)) => e
            .variants
            .iter()
            .map(facet::Variant::effective_name)
            .collect(),
        _ => panic!("an enum"),
    };
    assert_eq!(shape, EquipmentSlot::NAMES, "facet's names");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(schema::expected = "a control flag name", schema::case_insensitive)]
#[repr(u8)]
pub enum ControlFlag {
    #[facet(rename = "move")]
    Move,
    #[facet(rename = "look")]
    Look,
}

#[test]
fn expected_and_case_insensitive_shape_the_name_lookup_and_its_error() {
    assert_eq!(ControlFlag::from_name("MOVE"), Some(ControlFlag::Move));
    assert_eq!(ControlFlag::EXPECTED, "a control flag name");
    assert_eq!(
        ControlFlag::parse_name("jump").unwrap_err().to_string(),
        "expected a control flag name, found `jump`"
    );
}

#[test]
fn a_transparent_newtype_checks_at_its_owners_path() {
    assert_eq!(Friction::default(), Friction(0.4));
    assert_eq!(Friction::FIELDS[0].json, "");
    let mut high = Friction(1.5);
    let (out, _) = check(&mut high, MemberFailure::DefaultAndTell);
    assert_eq!(high, Friction(0.4));
    assert_eq!(paths(&out), [(String::new(), Severity::Info)]);
}

fn shape_names(shape: &'static Shape) -> Vec<&'static str> {
    match shape.ty {
        Type::User(UserType::Struct(st)) => st
            .fields
            .iter()
            .filter(|f| !f.flags.contains(facet::FieldFlags::SKIP))
            .map(facet::Field::effective_name)
            .collect(),
        _ => panic!("a struct"),
    }
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(rename_all = "camelCase", schema::unknown = "report")]
pub struct Renamed {
    #[facet(default = 1)]
    pub hud_range: i32,
    #[facet(
        rename = "Other",
        schema::since_format = "1.21.60",
        schema::unknown = "keep"
    )]
    pub other_value: Option<i32>,
    #[facet(skip)]
    pub side: u8,
    #[facet(schema::since_engine = "1.20.0")]
    pub r#type: Option<String>,
}

#[test]
fn the_json_names_are_facets_and_the_stored_keys_are_rows() {
    let rows: Vec<&str> = Renamed::FIELDS.iter().map(|r| r.json).collect();
    assert_eq!(rows, shape_names(Renamed::SHAPE));
    assert_eq!(rows, ["hudRange", "Other", "type"]);
    assert_eq!(Renamed::UNKNOWN, Some(UnknownRule::Report));
    assert_eq!(
        Renamed::FIELDS[1].since_format,
        Some(FormatVersion::new(1, 21, 60))
    );
    assert_eq!(Renamed::FIELDS[1].unknown, Some(UnknownRule::Keep));
    assert_eq!(
        Renamed::FIELDS[2].since_engine,
        Some(FormatVersion::new(1, 20, 0))
    );
    let renamed = Renamed::default();
    assert_eq!((renamed.hud_range, renamed.side), (1, 0));
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Span {
    #[facet(default = 0.0)]
    pub low: f32,
    #[facet(default = 1.0, schema::min = low)]
    pub high: f32,
    #[facet(schema::fallback = 2, schema::above = floor)]
    pub top: Option<i32>,
    #[facet(schema::fallback = 1)]
    pub floor: Option<i32>,
}

#[test]
fn a_cross_field_bound_compares_with_the_other_member_and_falls_back_to_it() {
    assert_eq!(
        Span::FIELDS[1].bounds,
        [Bound {
            op: BoundOp::Min,
            limit: Limit::Field("low")
        }]
    );
    let mut span = Span {
        low: 3.0,
        high: 1.0,
        top: Some(1),
        floor: None,
    };
    let (out, _) = check(&mut span, MemberFailure::DefaultAndTell);
    assert_eq!(span.high, 3.0, "the other member");
    assert_eq!(
        span.top,
        Some(1),
        "the floor's fallback, 1: top is not above it"
    );
    assert_eq!(
        paths(&out),
        [
            ("high".to_owned(), Severity::Info),
            ("top".to_owned(), Severity::Info)
        ]
    );
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Named {
    #[facet(schema::lowercase)]
    pub id: String,
    #[facet(schema::lowercase)]
    pub alias: Option<String>,
}

#[test]
fn lowercase_lower_cases_ascii_in_silence_as_the_game_does() {
    let mut named = Named {
        id: "Zombie".to_owned(),
        alias: Some("ÉCLAIR".to_owned()),
    };
    let (out, failed) = check(&mut named, MemberFailure::Refuse);
    assert!(!failed);
    assert!(out.is_empty(), "no finding: {out:?}");
    assert_eq!(named.id, "zombie");
    assert_eq!(named.alias.as_deref(), Some("Éclair"), "ASCII letters only");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Inner {
    #[facet(default = 0, schema::max = 9)]
    pub x: i32,
    pub on_y: Option<EventTrigger>,
    #[facet(schema::foreign_event)]
    pub on_spawn: Option<EventTrigger>,
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Outer {
    pub list: Vec<Inner>,
    pub by_name: IndexMap<String, Inner>,
    pub events: Option<Vec<EventTrigger>>,
    #[facet(flatten)]
    pub shared: Inner,
}

#[test]
fn check_and_triggers_walk_into_lists_maps_and_flattened_fields() {
    assert_eq!(
        Outer::triggers(),
        ["list[].on_y", "by_name.*.on_y", "events[]", "on_y"]
    );
    let inner = |x| Inner {
        x,
        on_y: None,
        on_spawn: None,
    };
    let mut outer = Outer {
        list: vec![inner(1), inner(10)],
        by_name: IndexMap::from([("a".to_owned(), inner(11))]),
        events: None,
        shared: inner(12),
    };
    let (out, _) = check(&mut outer, MemberFailure::Refuse);
    let at: Vec<String> = out
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["list/1/x", "by_name/a/x", "x"]);
    assert_eq!(Outer::FIELDS[3].json, "");
    assert!(Outer::FIELDS[3].flatten);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Gated {
    #[cfg_attr(all(), facet(schema::min = 0))]
    pub on: Option<i32>,
    #[cfg_attr(any(), facet(schema::min = 0))]
    pub off: Option<i32>,
}

#[test]
fn a_cfg_attr_key_counts_only_when_its_condition_holds() {
    assert_eq!(Gated::FIELDS[0].bounds.len(), 1);
    assert!(Gated::FIELDS[1].bounds.is_empty());
}

const TAGS: MolangFieldSpec = MolangFieldSpec::DEFAULT.tags();

fn walk_tags(tags: &mut [Molang], visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {
    for (index, tag) in tags.iter_mut().enumerate() {
        visitor.field(index, &TAGS, tag);
    }
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Specs {
    #[facet(schema::molang = TAGS)]
    pub tagged: Option<Molang>,
    #[facet(schema::molang_with = walk_tags)]
    pub tags: Vec<Molang>,
    pub boss: Option<Boss>,
}

#[test]
fn molang_and_molang_with_override_the_default_walk() {
    let mut specs = Specs {
        tagged: Some(Molang::Const(1.0)),
        tags: vec![Molang::Const(2.0)],
        boss: Some(boss()),
    };
    let mut seen = Vec::new();
    visit_fields_under(
        &mut specs,
        &JsonPath::new(),
        &MolangFieldSpec::DEFAULT,
        |field| {
            seen.push((field.path.to_string(), field.spec.admission));
        },
    );
    assert_eq!(
        seen,
        [
            ("tagged".to_owned(), TAGS.admission),
            ("tags/0".to_owned(), TAGS.admission),
            ("boss/cond".to_owned(), MolangFieldSpec::DEFAULT.admission),
        ]
    );
}

#[test]
fn the_namespace_resolves_through_core() {
    // `schema` is used by the attributes above; this keeps the import named.
    let _ = schema::BoundOp::Min;
}

#[test]
fn a_field_with_two_bounds_is_one_range_and_one_finding_with_no_text_by_default() {
    let mut high = Friction(1.5);
    let (out, failed) = check(&mut high, MemberFailure::Refuse);
    assert!(failed);
    let codes: Vec<&Code> = out.iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&Code::from(&codes::INVALID_VALUE)]);

    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out).with_range_text(true);
    Friction(-1.0).check(&mut cx);
    let range = NumericRange::inclusive(0.0, f64::from(0.9_f32));
    let codes: Vec<&Code> = out.iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&range.code()], "the game's range text, both sides");
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Exclusive {
    #[facet(default = 0.5, schema::above = 0.0, schema::below = 1.0)]
    pub chance: f64,
}

#[test]
fn an_exclusive_range_states_its_open_sides() {
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out).with_range_text(true);
    Exclusive { chance: 1.0 }.check(&mut cx);
    let range = NumericRange {
        exclusive_min: true,
        exclusive_max: true,
        ..NumericRange::inclusive(0.0, 1.0)
    };
    assert_eq!(
        out.iter().map(|d| &d.code).collect::<Vec<_>>(),
        [&range.code()]
    );
}

#[test]
fn nan_fails_every_bound() {
    let mut nan = Friction(f32::NAN);
    let (out, failed) = check(&mut nan, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(out.iter().count(), 1);
    let mut door = BreakDoor {
        speed: Some(f32::NAN),
        ..BreakDoor::default()
    };
    let (_, failed) = check(&mut door, MemberFailure::Refuse);
    assert!(failed, "a NaN is not >= 0");
}

/// The plan's `[0, other]` shape: `value` is at least 0 and at most `limit`, which stands for 10
/// where the document has none. `value` comes first, so the order of the checks is not the
/// order of the fields.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Window {
    #[facet(default = 0, schema::min = 0, schema::max = limit)]
    pub value: i32,
    #[facet(schema::fallback = 10, schema::min = 0)]
    pub limit: Option<i32>,
}

#[test]
fn a_cross_field_bound_compares_with_the_other_member_when_it_is_written() {
    let mut window = Window {
        value: 7,
        limit: Some(5),
    };
    let mut out = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut cx = SchemaCx::new(reading, JsonPath::new(), &mut out).with_range_text(true);
    window.check(&mut cx);
    assert_eq!(window.value, 5, "the other member");
    let codes: Vec<&Code> = out.iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&NumericRange::inclusive(0.0, 5.0).code()]);
}

#[test]
fn a_cross_field_bound_compares_with_what_an_absent_member_stands_for() {
    let mut window = Window {
        value: 12,
        limit: None,
    };
    let (out, _) = check(&mut window, MemberFailure::DefaultAndTell);
    assert_eq!(window.value, 10, "the other member's fallback");
    assert_eq!(paths(&out), [("value".to_owned(), Severity::Info)]);

    let mut low = Window {
        value: -1,
        limit: None,
    };
    let (_, _) = check(&mut low, MemberFailure::DefaultAndTell);
    assert_eq!(
        low.value, 0,
        "the numeric side resets to the field's own default"
    );
}

#[test]
fn a_cross_field_bound_runs_after_the_other_members_own_check() {
    // `limit` breaks its own bound and is reset first; `value` is then within what it stands
    // for, where in field order it would have been compared with -3.
    let mut window = Window {
        value: 7,
        limit: Some(-3),
    };
    let (out, _) = check(&mut window, MemberFailure::DefaultAndTell);
    assert_eq!(window.value, 7);
    assert_eq!(paths(&out), [("limit".to_owned(), Severity::Info)]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Needs {
    #[facet(schema::min = 1)]
    pub count: i32,
}

#[test]
fn a_required_field_with_nothing_to_reset_to_fails_its_owner_even_under_default_and_tell() {
    let mut needs = Needs { count: 0 };
    let (out, failed) = check(&mut needs, MemberFailure::DefaultAndTell);
    assert!(failed, "no silent acceptance");
    assert_eq!(needs.count, 0, "the value stays");
    assert_eq!(paths(&out), [("count".to_owned(), Severity::Info)]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Holders {
    pub boxed: Box<Inner>,
    pub hashed: std::collections::HashMap<String, Inner>,
    pub sorted: std::collections::BTreeMap<String, Inner>,
}

#[test]
fn check_and_triggers_walk_into_boxes_and_std_maps() {
    assert_eq!(
        Holders::triggers(),
        ["boxed.on_y", "hashed.*.on_y", "sorted.*.on_y"]
    );
    let inner = |x| Inner {
        x,
        on_y: None,
        on_spawn: None,
    };
    let mut holders = Holders {
        boxed: Box::new(inner(10)),
        hashed: [("h".to_owned(), inner(11))].into(),
        sorted: [("s".to_owned(), inner(12))].into(),
    };
    let (out, _) = check(&mut holders, MemberFailure::Refuse);
    let at: Vec<String> = out
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["boxed/x", "hashed/h/x", "sorted/s/x"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Placed {
    pub block: bedrock_addon_core::id::BlockId,
    pub corners: [Inner; 2],
    pub offset: Option<[f32; 3]>,
    pub filter: Option<bedrock_addon_core::shared::Filter>,
    pub replaces: Option<Vec<bedrock_addon_shared_shapes::BlockDescriptor>>,
    pub height: Option<bedrock_addon_core::shared::IntRange>,
}

#[test]
fn check_and_triggers_walk_into_fixed_arrays_past_identifiers_and_shared_values() {
    assert_eq!(Placed::triggers(), ["corners[].on_y"]);
    let inner = |x| Inner {
        x,
        on_y: None,
        on_spawn: None,
    };
    let mut placed = Placed {
        block: bedrock_addon_core::id::BlockId::parse("minecraft:stone"),
        corners: [inner(1), inner(12)],
        offset: Some([0.0; 3]),
        filter: None,
        replaces: None,
        height: None,
    };
    let (out, _) = check(&mut placed, MemberFailure::Refuse);
    let at: Vec<String> = out
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["corners/1/x"]);
}

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Shared {
    pub filter: Option<bedrock_addon_core::shared::Filter>,
    pub block: bedrock_addon_shared_shapes::BlockSpecifier,
    pub range: Option<bedrock_addon_core::shared::FloatRange>,
    pub inner: Inner,
}

#[test]
fn filters_block_specifiers_and_ranges_are_leaves_of_the_walk() {
    assert_eq!(Shared::triggers(), ["inner.on_y"]);
    let mut shared = Shared {
        filter: None,
        block: bedrock_addon_shared_shapes::BlockSpecifier::new(
            bedrock_addon_core::id::BlockId::parse("minecraft:stone"),
        ),
        range: None,
        inner: Inner {
            x: 12,
            on_y: None,
            on_spawn: None,
        },
    };
    let (out, _) = check(&mut shared, MemberFailure::Refuse);
    let at: Vec<String> = out
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["inner/x"]);
}

/// The core value types the families hold in their typed structs, each walked by the derive
/// without a family's own impl.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct CoreValues {
    pub item: bedrock_addon_shared_shapes::ItemDescriptor,
    pub items: Option<Vec<bedrock_addon_shared_shapes::ItemDescriptor>>,
    pub sound: Option<bedrock_addon_core::id::SoundEventName>,
    pub entity: Option<bedrock_addon_core::id::EntityId>,
    pub block: Option<bedrock_addon_core::id::BlockId>,
    pub menu: Option<bedrock_addon_shared_shapes::MenuCategory>,
    pub subject: Option<bedrock_addon_core::shared::FilterSubject>,
    pub ints: Option<bedrock_addon_core::shared::IntRange>,
    pub floats: Option<bedrock_addon_core::shared::FloatRange>,
    pub specifier: Option<bedrock_addon_shared_shapes::BlockSpecifier>,
    pub render_controllers: Option<Vec<bedrock_addon_shared_shapes::ControllerUse>>,
    pub animate: Option<Vec<bedrock_addon_shared_shapes::AnimateEntry>>,
    pub size: Option<[u32; 2]>,
    pub tint: Option<[u8; 3]>,
    pub inner: Inner,
}

/// Every member of [`CoreValues`] written, and an `inner.x` past its bound.
const CORE_VALUES: &str = r#"{
    "item": {"tags": "query.any_tag('wood')"},
    "items": ["minecraft:stick", {"tags": "query.all_tags('a', 'b')"}, {"item_tag": "minecraft:logs"}],
    "sound": "random.click",
    "entity": "minecraft:pig",
    "block": "minecraft:stone",
    "menu": {"category": "nature", "group": "itemGroup.name.log"},
    "subject": "other",
    "ints": {"range_min": 1, "range_max": 3},
    "floats": [0.5, 1.5],
    "specifier": {"name": "minecraft:stone", "states": {"stone_type": "granite"}},
    "render_controllers": ["controller.render.pig", {"controller.render.saddle": "query.is_saddled"}],
    "animate": ["walk", {"look": "query.is_awake"}],
    "size": [16, 32],
    "tint": [255, 128, 0],
    "inner": {"x": 12}
}"#;

fn core_values() -> CoreValues {
    bedrock_addon_core::json::from_str_jsonc(CORE_VALUES).expect("the document reads")
}

#[test]
fn core_value_types_are_leaves_of_the_check_and_hold_no_trigger() {
    assert_eq!(CoreValues::triggers(), ["inner.on_y"]);
    let mut values = core_values();
    let before = values.clone();
    let (out, _) = check(&mut values, MemberFailure::Refuse);
    let at: Vec<String> = out
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(at, ["inner/x"]);
    assert_eq!(values, before, "a leaf is left as read");
}

#[test]
fn item_descriptor_tags_and_named_conditions_are_molang_fields_under_the_default_spec() {
    const FAMILY: MolangFieldSpec =
        MolangFieldSpec::DEFAULT.version(MolangVersionSource::FormatVersion);
    let mut values = core_values();
    let mut seen = Vec::new();
    visit_fields_under(&mut values, &JsonPath::new(), &FAMILY, |field| {
        seen.push((field.path.to_string(), *field.spec));
    });
    let paths: Vec<&str> = seen.iter().map(|(path, _)| path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "item/tags",
            "items/1/tags",
            "render_controllers/1/controller.render.saddle",
            "animate/1/look",
        ]
    );
    assert!(seen.iter().all(|(_, spec)| *spec == FAMILY), "{seen:?}");
}

/// Dimension's height: a range and a multiple of 16.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Height {
    #[facet(
        default = 256,
        schema::min = 16,
        schema::max = 1024,
        schema::multiple_of = 16
    )]
    pub height_range: i32,
    #[facet(schema::multiple_of = 16)]
    pub min_y: Option<i32>,
    #[facet(schema::multiple_of = 0.5)]
    pub scale: Option<f32>,
}

fn height(height_range: i32, min_y: Option<i32>) -> Height {
    Height {
        height_range,
        min_y,
        scale: None,
    }
}

/// The findings of `value`'s check under `failure`, with the game's range text.
fn check_worded<T: Schema>(value: &mut T, failure: MemberFailure) -> (Diagnostics, bool) {
    let mut out = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(failure);
    let failed = {
        let mut cx = SchemaCx::new(reading, JsonPath::new(), &mut out).with_range_text(true);
        value.check(&mut cx);
        cx.failed()
    };
    (out, failed)
}

/// A step `f64` holds only near: `0.1`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Tenths {
    #[facet(schema::multiple_of = 0.1)]
    pub x: f64,
}

#[test]
fn a_step_f64_holds_only_near_tells_a_decimal_multiple_of_it() {
    // `0.5 % 0.1` and `1.0 % 0.1` are about 0.1 in `f64`, not 0: the check compares with
    // the remainder. `0.2 % 0.1` is 0.
    for x in [0.5, 1.0] {
        let (out, _) = check(&mut Tenths { x }, MemberFailure::Refuse);
        assert_eq!(paths(&out), [("x".to_owned(), Severity::Error)], "{x}");
    }
    let (out, _) = check(&mut Tenths { x: 0.2 }, MemberFailure::Refuse);
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn multiple_of_is_a_row_fact_and_a_value_off_the_step_is_one_finding() {
    assert_eq!(Height::FIELDS[0].multiple_of, Some(16.0));
    assert_eq!(Height::FIELDS[1].multiple_of, Some(16.0));
    assert_eq!(Height::FIELDS[2].multiple_of, Some(0.5));
    assert_eq!(Boss::FIELDS[2].multiple_of, None);
    let (out, failed) = check(&mut height(256, Some(-64)), MemberFailure::Refuse);
    assert!(out.is_empty() && !failed, "{out:?}");

    let mut off = height(40, None);
    let (out, failed) = check(&mut off, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(paths(&out), [("height_range".to_owned(), Severity::Error)]);
    let codes: Vec<&Code> = out.iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&Code::from(&codes::INVALID_VALUE)]);
    assert_eq!(off.height_range, 40, "the value stays under Refuse");

    // Worded as the game words it, where the family opts into the range text.
    let (out, _) = check_worded(&mut height(40, None), MemberFailure::Refuse);
    let worded: Vec<Code> = out.iter().map(|d| d.code.clone()).collect();
    assert_eq!(
        worded,
        [bedrock_addon_core::diagnostic::cereal::not_multiple_of(
            40.0, 16.0
        )]
    );

    // The range comes first: a value outside it is told for the range alone.
    let (out, _) = check_worded(&mut height(2000, None), MemberFailure::Refuse);
    let worded: Vec<Code> = out.iter().map(|d| d.code.clone()).collect();
    assert_eq!(worded, [NumericRange::inclusive(16.0, 1024.0).code()]);
}

#[test]
fn a_value_off_the_step_resets_as_a_broken_bound_does_under_default_and_tell() {
    let mut off = height(40, Some(8));
    let (out, failed) = check(&mut off, MemberFailure::DefaultAndTell);
    assert!(!failed);
    assert_eq!(
        paths(&out),
        [
            ("height_range".to_owned(), Severity::Info),
            ("min_y".to_owned(), Severity::Info),
        ]
    );
    assert_eq!(off, height(256, None));
    // A float steps by its own literal.
    let mut scaled = Height {
        scale: Some(1.25),
        ..height(256, None)
    };
    let (out, _) = check(&mut scaled, MemberFailure::Refuse);
    assert_eq!(paths(&out), [("scale".to_owned(), Severity::Error)]);
    let mut stepped = Height {
        scale: Some(1.5),
        ..height(256, None)
    };
    assert!(check(&mut stepped, MemberFailure::Refuse).0.is_empty());
}

/// Block's `minecraft:destructible_by_explosion` in the shape of an untagged enum: a bool, a
/// bare number, an object, and a unit read as its name.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(clippy::large_enum_variant, reason = "a toy, built a few times")]
pub enum Explosion {
    Enabled(bool),
    Resistance(#[facet(schema::min = 0.0)] f32),
    Params {
        #[facet(default = 0.0, schema::min = 0.0)]
        explosion_resistance: f32,
        #[facet(rename = "on_boom")]
        on_explode: Option<EventTrigger>,
        cond: Option<Molang>,
        #[facet(schema::fallback = 1, schema::min = 1)]
        count: Option<i32>,
        inner: Option<Inner>,
    },
    Off,
}

fn params(explosion_resistance: f32, count: Option<i32>, x: i32) -> Explosion {
    Explosion::Params {
        explosion_resistance,
        on_explode: None,
        cond: Some(Molang::Const(1.0)),
        count,
        inner: Some(Inner {
            x,
            on_y: None,
            on_spawn: None,
        }),
    }
}

#[test]
fn an_untagged_enum_states_its_rows_per_variant() {
    assert!(
        Explosion::FIELDS.is_empty(),
        "no one object: the rows are per variant"
    );
    let variants: Vec<(&str, Vec<&str>)> = Explosion::VARIANT_FIELDS
        .iter()
        .map(|(variant, rows)| (*variant, rows.iter().map(|row| row.json).collect()))
        .collect();
    assert_eq!(
        variants,
        [
            ("Enabled", vec![""]),
            ("Resistance", vec![""]),
            (
                "Params",
                vec!["explosion_resistance", "on_boom", "cond", "count", "inner"]
            ),
            ("Off", vec![]),
        ]
    );
    let resistance = Explosion::VARIANT_FIELDS[1].1[0];
    assert_eq!(
        resistance,
        FieldRow {
            bounds: &[Bound {
                op: BoundOp::Min,
                limit: Limit::Num(0.0)
            }],
            ..FieldRow::plain("0", "", "f32", FieldKind::Required)
        }
    );
    let count = Explosion::VARIANT_FIELDS[2].1[3];
    assert_eq!(count.fallback, Some(Lit::Int(1)));
    // The names are facet's own.
    let Type::User(UserType::Enum(en)) = Explosion::SHAPE.ty else {
        panic!("an enum");
    };
    let facet_params: Vec<&str> = en.variants[2]
        .data
        .fields
        .iter()
        .map(facet::Field::effective_name)
        .collect();
    assert_eq!(facet_params, variants[2].1);
}

#[test]
fn an_untagged_enum_checks_the_active_variant_at_the_owners_path() {
    let (out, failed) = check(&mut Explosion::Enabled(true), MemberFailure::Refuse);
    assert!(out.is_empty() && !failed);
    let (out, _) = check(&mut Explosion::Off, MemberFailure::Refuse);
    assert!(out.is_empty());

    let mut bare = Explosion::Resistance(-1.0);
    let (out, failed) = check(&mut bare, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(paths(&out), [(String::new(), Severity::Error)]);
    // A newtype's value has nothing to reset to: it stays and fails its owner.
    let (out, failed) = check(&mut bare, MemberFailure::DefaultAndTell);
    assert!(failed);
    assert_eq!(paths(&out), [(String::new(), Severity::Info)]);
    assert_eq!(bare, Explosion::Resistance(-1.0));

    let mut object = params(-2.0, Some(0), 12);
    let (out, failed) = check(&mut object, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(
        paths(&out),
        [
            ("explosion_resistance".to_owned(), Severity::Error),
            ("count".to_owned(), Severity::Error),
            ("inner/x".to_owned(), Severity::Error),
        ]
    );
    let (_, failed) = check(&mut object, MemberFailure::DefaultAndTell);
    assert!(!failed);
    assert_eq!(object, params(0.0, None, 0));
}

#[test]
fn an_untagged_enum_lists_every_variants_triggers_and_walks_the_active_ones_molang() {
    const FAMILY: MolangFieldSpec =
        MolangFieldSpec::DEFAULT.version(MolangVersionSource::FormatVersion);
    assert_eq!(Explosion::triggers(), ["on_boom", "inner.on_y"]);
    let walked = |value: &mut Explosion| {
        let mut seen = Vec::new();
        visit_fields_under(value, &JsonPath::new(), &FAMILY, |field| {
            seen.push(field.path.to_string());
        });
        seen
    };
    assert_eq!(walked(&mut params(1.0, None, 0)), ["cond"]);
    assert!(walked(&mut Explosion::Enabled(false)).is_empty());
}

/// Two untagged arms that share a trigger member list it once.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(untagged)]
#[repr(u8)]
#[allow(clippy::large_enum_variant, reason = "a toy, built a few times")]
pub enum Either {
    Short { on_y: Option<EventTrigger> },
    Long { on_y: Option<EventTrigger>, n: i32 },
    Nested(Inner),
}

#[test]
fn a_trigger_two_arms_share_is_one_member() {
    assert_eq!(Either::triggers(), ["on_y"]);
}

/// Block's horizontal direction: the game's order is not the declaration's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
#[facet(rename_all = "snake_case", schema::order = "game")]
#[repr(u8)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// The four directions, in the game's order.
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
}

#[test]
fn a_string_enum_in_the_games_order_keeps_its_own_all() {
    assert_eq!(
        Direction::ALL,
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West
        ]
    );
    assert_eq!(Direction::NAMES, ["north", "south", "east", "west"]);
    assert_eq!(Direction::parse_name("east"), Ok(Direction::East));
    assert_eq!(Direction::West.to_string(), "west");
}

/// `schema::molang` on fields whose types walk Molang of their own: the spec is the nested
/// walk's default, for every field below that has none of its own.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct NestedSpecs {
    #[facet(schema::molang = TAGS)]
    pub item: Option<bedrock_addon_shared_shapes::ItemDescriptor>,
    #[facet(schema::molang = TAGS)]
    pub tagged_boss: Option<Boss>,
    pub boss: Option<Boss>,
}

#[test]
fn molang_on_a_nested_walk_sets_that_walks_default_spec() {
    let mut nested: NestedSpecs =
        bedrock_addon_core::json::from_str_jsonc(r#"{"item": {"tags": "q.any_tag('a')"}}"#)
            .expect("the document reads");
    nested.tagged_boss = Some(boss());
    nested.boss = Some(boss());
    let mut seen = Vec::new();
    visit_fields_under(
        &mut nested,
        &JsonPath::new(),
        &MolangFieldSpec::DEFAULT,
        |field| {
            seen.push((field.path.to_string(), *field.spec));
        },
    );
    assert_eq!(
        seen,
        [
            ("item/tags".to_owned(), TAGS),
            ("tagged_boss/cond".to_owned(), TAGS),
            ("boss/cond".to_owned(), MolangFieldSpec::DEFAULT),
        ]
    );
}

/// `schema::molang` on a struct: the default spec of the walk below it, for the fields of its
/// own and of the structs it holds, unless one of them states another.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang = TAGS)]
pub struct RootSpec {
    pub item: Option<bedrock_addon_shared_shapes::ItemDescriptor>,
    pub boss: Option<Boss>,
    pub plain: Option<Molang>,
    #[facet(schema::molang = MolangFieldSpec::KEPT)]
    pub kept: Option<Molang>,
}

#[test]
fn molang_on_a_struct_is_the_default_spec_of_the_walk_below_it() {
    let mut root: RootSpec =
        bedrock_addon_core::json::from_str_jsonc(r#"{"item": {"tags": "q.any_tag('a')"}}"#)
            .expect("the document reads");
    root.boss = Some(boss());
    root.plain = Some(Molang::Const(1.0));
    root.kept = Some(Molang::Const(2.0));
    let mut seen = Vec::new();
    visit_fields_under(
        &mut root,
        &JsonPath::new(),
        &MolangFieldSpec::DEFAULT,
        |field| {
            seen.push((field.path.to_string(), *field.spec));
        },
    );
    assert_eq!(
        seen,
        [
            ("item/tags".to_owned(), TAGS),
            ("boss/cond".to_owned(), TAGS),
            ("plain".to_owned(), TAGS),
            ("kept".to_owned(), MolangFieldSpec::KEPT),
        ]
    );
}

/// A bound on a fixed array or a list is the bound of each element.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Offsets {
    #[facet(schema::min = -64.0, schema::max = 64.0)]
    pub view: Option<[f32; 2]>,
    #[facet(schema::min = 0)]
    pub counts: Vec<i32>,
}

#[test]
fn a_bound_on_an_array_or_a_list_is_the_bound_of_each_element_and_tells_the_first_one_outside() {
    let mut inside = Offsets {
        view: Some([-64.0, 64.0]),
        counts: vec![0, 3],
    };
    assert!(check(&mut inside, MemberFailure::Refuse).0.is_empty());
    let mut outside = Offsets {
        view: Some([65.0, -65.0]),
        counts: vec![1, -2, -3],
    };
    let (out, failed) = check(&mut outside, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(
        paths(&out),
        [
            ("view/0".to_owned(), Severity::Error),
            ("counts/1".to_owned(), Severity::Error)
        ]
    );
    // An absent member has no element to check, and an `Option` resets whole.
    let mut absent = Offsets {
        view: None,
        counts: Vec::new(),
    };
    assert!(check(&mut absent, MemberFailure::Refuse).0.is_empty());
    let mut told = Offsets {
        view: Some([65.0, 0.0]),
        counts: Vec::new(),
    };
    let (out, failed) = check(&mut told, MemberFailure::DefaultAndTell);
    assert!(!failed);
    assert_eq!(paths(&out), [("view/0".to_owned(), Severity::Info)]);
    assert_eq!(told.view, None);
}

/// A limit written as a double is compared, and worded, as the double it is.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Yaw {
    #[facet(schema::min = -180.0, schema::max = 179.9_f64)]
    pub least: Option<f32>,
    #[facet(schema::min = -180.0, schema::max = 179.9)]
    pub plain: Option<f32>,
}

#[test]
fn a_limit_with_the_f64_suffix_is_compared_and_worded_as_a_double() {
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out).with_range_text(true);
    Yaw {
        least: Some(180.0),
        plain: Some(180.0),
    }
    .check(&mut cx);
    let double = NumericRange::inclusive(-180.0, 179.9);
    let single = NumericRange::inclusive(-180.0, f64::from(179.9_f32));
    assert_eq!(
        out.iter().map(|d| &d.code).collect::<Vec<_>>(),
        [&double.code(), &single.code()]
    );
    assert_ne!(double.code(), single.code());
    // The field's own 179.9 is below the double 179.9.
    let mut at_the_limit = Yaw {
        least: Some(179.9),
        plain: Some(179.9),
    };
    assert!(check(&mut at_the_limit, MemberFailure::Refuse).0.is_empty());
}

/// The emitted read names its locals apart from the family's own items: a read function named
/// `read` and a default from a function named `tag` are the family's, not the read's.
mod hygiene {
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::kit::{Node, Read};
    use bedrock_addon_core::json::{Value, from_str_jsonc};
    use bedrock_addon_core::schema::{self, FromJson, Schema};
    use bedrock_addon_core::versioning::Reading;
    use facet::Facet;

    /// A level, read doubled.
    fn read(node: &Node<'_>, read: &mut Read<'_>) -> Option<u8> {
        u8::from_json(node, read).map(|level| level * 2)
    }

    fn tag() -> u8 {
        9
    }

    #[derive(Debug, PartialEq, Facet, Schema)]
    struct Shadowed {
        #[facet(schema::read_with = read)]
        level: u8,
        #[facet(default = tag())]
        key: u8,
        member: Option<u8>,
    }

    #[test]
    fn a_family_s_own_names_are_not_the_read_s_locals() {
        let json: Value = from_str_jsonc(r#"{"level": 2, "member": 1}"#).unwrap();
        let mut found = Diagnostics::default();
        let mut reader = Read::new(Reading::DOCUMENT, &mut found);
        let value = Shadowed::from_json(&Node::new(&json, JsonPath::new()), &mut reader);
        assert_eq!(
            value,
            Some(Shadowed {
                level: 4,
                key: 9,
                member: Some(1)
            })
        );
    }
}

// ---- the validate stage: since_engine, the older tree's wording, length, hooks -----------------

use bedrock_addon_core::diagnostic::Phase;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{Bounded, End, FromJson, TreeWording};
use bedrock_addon_core::shared::IntRange;

/// An integer that keeps whether it was written as a real (`2.0`): a wrapper a bound sees
/// through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Facet)]
#[facet(opaque)]
pub struct Counted(pub i32, pub bool);

impl Bounded for Counted {
    type Number = i32;

    fn numbers(&self) -> impl IntoIterator<Item = (End, i32)> {
        [(End::Only, self.0)]
    }
}

impl FromJson for Counted {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        i32::from_json(node, read).map(|n| Counted(n, false))
    }
}

bedrock_addon_core::impl_schema_leaf!(Counted);
bedrock_addon_core::impl_no_molang!(Counted);

/// What the game's older schema tree bounds: a wrapped integer, a range, a list.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct TreeBounds {
    #[facet(schema::min = 1)]
    pub count: Counted,
    #[facet(schema::min = 0, schema::max = 4)]
    pub span: Option<IntRange>,
    #[facet(schema::min_len = 1, schema::max_len = 2)]
    pub items: Option<Vec<i32>>,
    #[facet(schema::min = 1)]
    pub steps: Option<Vec<Counted>>,
    #[facet(schema::min = 0.0, schema::max = 1.0)]
    pub chance: Option<f32>,
}

fn tree_bounds() -> TreeBounds {
    TreeBounds {
        count: Counted(1, false),
        span: Some(IntRange::new(0, 4, schema_range_form())),
        items: Some(vec![1]),
        steps: Some(vec![Counted(1, false)]),
        chance: Some(0.5),
    }
}

fn schema_range_form() -> bedrock_addon_core::shared::RangeForm {
    bedrock_addon_core::shared::RangeForm::MinMax
}

// The family's own lines for the two ends of a range.
static TEST_RANGE_MIN: bedrock_addon_core::diagnostic::CodeDef<3> =
    bedrock_addon_core::diagnostic::CodeDef::new(
        "test/range_min",
        bedrock_addon_core::diagnostic::Kind::InvalidValue,
        bedrock_addon_core::diagnostic::Class::Error,
        "range_min {} is outside [{}, {}]",
    );
static TEST_RANGE_MAX: bedrock_addon_core::diagnostic::CodeDef<3> =
    bedrock_addon_core::diagnostic::CodeDef::new(
        "test/range_max",
        bedrock_addon_core::diagnostic::Kind::InvalidValue,
        bedrock_addon_core::diagnostic::Class::Error,
        "range_max {} is outside [{}, {}]",
    );

const TREE_WORDING: TreeWording = TreeWording {
    range_min: &TEST_RANGE_MIN,
    range_max: &TEST_RANGE_MAX,
};

fn told_as_the_older_tree(value: &mut TreeBounds) -> Vec<(String, String)> {
    let load = LoadContext::detached();
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::validating(
        Reading::DOCUMENT,
        JsonPath::of_keys(["feature"]),
        &load,
        FormatVersion::new(1, 0, 0),
        &mut out,
    )
    .with_tree_wording(TREE_WORDING);
    value.check(&mut cx);
    out.iter()
        .map(|d| (d.location.json_path.to_string(), d.code.to_string()))
        .collect()
}

#[test]
fn a_bound_on_a_wrapper_or_a_range_is_the_bound_of_the_numbers_it_holds() {
    assert!(told_as_the_older_tree(&mut tree_bounds()).is_empty());
    let mut outside = tree_bounds();
    outside.count = Counted(0, false);
    outside.span = Some(IntRange::new(-1, 9, schema_range_form()));
    outside.steps = Some(vec![
        Counted(2, false),
        Counted(0, false),
        Counted(-3, false),
    ]);
    // The first number outside is the one finding of a range, the first element of a list.
    assert_eq!(
        told_as_the_older_tree(&mut outside),
        [
            (
                "feature/count".to_owned(),
                "Value '0' outside valid range [1, 2147483647]".to_owned()
            ),
            (
                "feature/span".to_owned(),
                "range_min -1 is outside [0, 4]".to_owned()
            ),
            (
                "feature/steps/1".to_owned(),
                "Value '0' outside valid range [1, 2147483647]".to_owned()
            ),
        ]
    );
    let mut upper = tree_bounds();
    upper.span = Some(IntRange::new(2, 5, schema_range_form()));
    assert_eq!(
        told_as_the_older_tree(&mut upper),
        [(
            "feature/span".to_owned(),
            "range_max 5 is outside [0, 4]".to_owned()
        )]
    );
}

#[test]
fn a_float_outside_its_range_states_the_value_in_its_own_type() {
    let mut value = tree_bounds();
    value.chance = Some(1.1);
    assert_eq!(
        told_as_the_older_tree(&mut value),
        [(
            "feature/chance".to_owned(),
            "Value '1.1' outside valid range [0, 1]".to_owned()
        )]
    );
}

#[test]
fn the_findings_of_a_validate_check_are_validate_phase_errors_unless_the_plain_wording_is_on() {
    let load = LoadContext::detached();
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::validating(
        Reading::DOCUMENT,
        JsonPath::new(),
        &load,
        FormatVersion::new(1, 0, 0),
        &mut out,
    );
    let mut value = tree_bounds();
    value.count = Counted(0, false);
    value.check(&mut cx);
    assert!(cx.failed());
    let found: Vec<_> = out.iter().collect();
    let [finding] = found.as_slice() else {
        panic!("one finding: {out:?}")
    };
    assert_eq!(finding.phase, Phase::Validate);
    assert_eq!(finding.code, Code::from(&codes::INVALID_VALUE));
}

#[test]
fn a_list_of_the_wrong_length_is_one_finding_at_the_field() {
    let mut empty = tree_bounds();
    empty.items = Some(Vec::new());
    let mut long = tree_bounds();
    long.items = Some(vec![1, 2, 3]);
    assert_eq!(
        told_as_the_older_tree(&mut empty),
        [(
            "feature/items".to_owned(),
            "Array too small (0 < 1)".to_owned()
        )]
    );
    assert_eq!(
        told_as_the_older_tree(&mut long),
        [(
            "feature/items".to_owned(),
            "Array too large (3 > 2)".to_owned()
        )]
    );
    let mut absent = tree_bounds();
    absent.items = None;
    assert!(told_as_the_older_tree(&mut absent).is_empty());
    // Without the tree's wording the finding has no text; an `Option` resets whole under the
    // lenient rule.
    let (out, failed) = check(&mut empty, MemberFailure::Refuse);
    assert!(failed);
    assert_eq!(
        out.iter().next().map(|d| d.code.clone()),
        Some(Code::from(&codes::INVALID_VALUE))
    );
    let (out, failed) = check(&mut long, MemberFailure::DefaultAndTell);
    assert!(!failed);
    assert_eq!(paths(&out), [("items".to_owned(), Severity::Info)]);
    assert_eq!(long.items, None);
}

#[test]
fn the_rows_state_the_length_bounds() {
    let items = TreeBounds::FIELDS[2];
    assert_eq!((items.min_len, items.max_len), (Some(1), Some(2)));
    assert_eq!(
        (TreeBounds::FIELDS[0].min_len, TreeBounds::FIELDS[0].max_len),
        (None, None)
    );
}

/// A member the engine may not have yet, and a hook across the fields and one in place of the
/// walk.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::rule = ends_in_order)]
pub struct EngineGated {
    #[facet(schema::since_engine = "1.26.20")]
    pub fresh: Option<bool>,
    #[facet(schema::min = 1, schema::since_engine = "1.26.30")]
    pub newer: Option<Counted>,
    #[facet(schema::check_with = loose)]
    pub inner: Option<TreeBounds>,
    pub low: i32,
    pub high: i32,
}

/// A rule across `low` and `high`, told at the struct's own path.
fn ends_in_order(gated: &mut EngineGated, cx: &mut SchemaCx<'_>) {
    if gated.high < gated.low {
        cx.kept("high", Code::from(&codes::INVALID_VALUE));
        gated.high = gated.low;
    }
}

/// The check of `inner` in place of the walk into its type: it tells itself and leaves the
/// type's own bounds alone.
fn loose(_: &mut TreeBounds, cx: &mut SchemaCx<'_>) {
    cx.error("", Code::from(&codes::INVALID_TYPE));
}

fn engine_gated() -> EngineGated {
    EngineGated {
        fresh: Some(true),
        newer: Some(Counted(1, false)),
        inner: None,
        low: 1,
        high: 2,
    }
}

fn validate_on(engine: FormatVersion, value: &mut EngineGated) -> Diagnostics {
    let mut load = LoadContext::detached();
    load.engine_version = engine;
    load.min_engine_version = None;
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::validating(
        Reading::DOCUMENT,
        JsonPath::of_keys(["feature"]),
        &load,
        FormatVersion::new(1, 0, 0),
        &mut out,
    );
    value.check(&mut cx);
    out
}

#[test]
fn a_member_newer_than_the_engine_is_not_valid_here() {
    let out = validate_on(FormatVersion::new(1, 26, 10), &mut engine_gated());
    assert_eq!(
        out.iter()
            .map(|d| (
                d.location.json_path.to_string(),
                d.code.id().to_owned(),
                d.severity
            ))
            .collect::<Vec<_>>(),
        [
            (
                "feature/fresh".to_owned(),
                "core/child_not_valid".to_owned(),
                Severity::Error
            ),
            (
                "feature/newer".to_owned(),
                "core/child_not_valid".to_owned(),
                Severity::Error
            ),
        ]
    );
    // On an engine that has them, a pack that does not declare the engine is told, as
    // information.
    let out = validate_on(FormatVersion::new(1, 26, 30), &mut engine_gated());
    assert_eq!(
        out.iter()
            .map(|d| (
                d.location.json_path.to_string(),
                d.code.id().to_owned(),
                d.severity
            ))
            .collect::<Vec<_>>(),
        [
            (
                "feature/fresh".to_owned(),
                "core/field_since_engine".to_owned(),
                Severity::Info
            ),
            (
                "feature/newer".to_owned(),
                "core/field_since_engine".to_owned(),
                Severity::Info
            ),
        ]
    );
    let mut absent = EngineGated {
        fresh: None,
        newer: None,
        ..engine_gated()
    };
    assert!(validate_on(FormatVersion::new(1, 0, 0), &mut absent).is_empty());
    // Outside the validate stage nothing is gated.
    assert!(
        check(&mut engine_gated(), MemberFailure::Refuse)
            .0
            .is_empty()
    );
}

#[test]
fn a_gate_comes_before_the_bounds_of_its_member() {
    let mut value = EngineGated {
        newer: Some(Counted(0, false)),
        ..engine_gated()
    };
    let out = validate_on(FormatVersion::new(1, 26, 10), &mut value);
    let ids: Vec<_> = out
        .iter()
        .map(|d| (d.location.json_path.to_string(), d.code.id().to_owned()))
        .collect();
    assert_eq!(
        ids,
        [
            (
                "feature/fresh".to_owned(),
                "core/child_not_valid".to_owned()
            ),
            (
                "feature/newer".to_owned(),
                "core/child_not_valid".to_owned()
            ),
            ("feature/newer".to_owned(), "core/invalid_value".to_owned()),
        ]
    );
}

#[test]
fn a_rule_runs_after_the_fields_at_the_structs_path_and_may_keep_the_value() {
    let mut value = EngineGated {
        high: 0,
        ..engine_gated()
    };
    let out = validate_on(FormatVersion::new(1, 26, 30), &mut value);
    let kept: Vec<_> = out.iter().filter(|d| d.kept).collect();
    let [finding] = kept.as_slice() else {
        panic!("one kept finding: {out:?}")
    };
    assert_eq!(finding.location.json_path.to_string(), "feature/high");
    assert_eq!(finding.phase, Phase::Validate);
    assert_eq!(value.high, 1);
}

#[test]
fn check_with_stands_in_for_the_walk_into_the_field() {
    let mut value = EngineGated {
        inner: Some(TreeBounds {
            count: Counted(0, false),
            ..tree_bounds()
        }),
        ..engine_gated()
    };
    let out = validate_on(FormatVersion::new(1, 26, 30), &mut value);
    // The hook's finding; `count` is below its bound and is not told, the walk being replaced.
    let told: Vec<_> = out
        .iter()
        .filter(|d| d.location.json_path.to_string().ends_with("inner"))
        .map(|d| d.code.id().to_owned())
        .collect();
    assert_eq!(told, ["core/invalid_type"]);
    assert!(
        !out.iter()
            .any(|d| d.location.json_path.to_string().contains("inner/count"))
    );
}

#[test]
fn leaves_with_findings_of_their_own_are_told_only_where_the_family_opts_in() {
    let mut out = Diagnostics::default();
    let cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out);
    assert!(!cx.leaf_findings());
    assert_eq!(cx.unknown_severity(), Severity::Error);
    let cx = cx.with_leaf_findings(true);
    assert!(cx.leaf_findings());
}
