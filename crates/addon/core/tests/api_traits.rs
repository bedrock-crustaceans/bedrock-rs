#![allow(clippy::pedantic, missing_docs)]
//! The std traits the public types of the core implement: `Debug` without a content bound,
//! `Clone`/`Hash`/`Eq` where the data allows, conversions and iterator traits.

use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{Id, IdError, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, StrictBool, StrictFloat, StrictInt, StrictString, Value, WriteStyle,
};
use bedrock_addon_core::load::{AppData, Experiment, Experiments, LoadOptions};
use bedrock_addon_core::pack_identity::{PackPath, PathRejected};
use bedrock_addon_core::versioning::testing::{Checks, FailureKind};
use bedrock_addon_core::versioning::{UpgradeCx, ValidateCx};
use bedrock_addon_core::visual::{Color, DescriptionSlots, Visuals};

fn assert_debug<T: Debug>() {}
fn assert_clone<T: Clone>() {}
fn assert_copy<T: Copy>() {}
fn assert_eq_hash<T: Eq + std::hash::Hash>() {}
fn assert_error<T: Error>() {}

#[test]
fn the_borrowing_and_context_types_are_debug() {
    assert_debug::<LoadOptions>();
    assert_debug::<UpgradeCx<'_>>();
    assert_debug::<ValidateCx<'_>>();
    assert_debug::<Checks<'_>>();
    assert_debug::<DescriptionSlots<'_>>();
    assert_debug::<Visuals<'_>>();
    assert_clone::<Checks<'_>>();
    assert_copy::<Visuals<'_>>();
}

#[test]
fn load_options_debug_shows_whether_built_ins_are_set() {
    let text = format!("{:?}", LoadOptions::default());
    assert!(text.contains("built_ins: \"absent\""), "{text}");
}

#[test]
fn the_error_types_derive_what_the_data_allows() {
    assert_error::<FailureKind>();
    assert_clone::<IdError>();
    assert_eq_hash::<IdError>();
    assert_clone::<EncodeError>();
    assert_eq_hash::<WriteStyle>();
    assert_eq_hash::<JsonSlice<'_>>();
    assert_eq_hash::<bedrock_addon_core::diagnostic::Span>();
    assert_eq!(
        bedrock_addon_core::diagnostic::Span::default(),
        bedrock_addon_core::diagnostic::Span::default()
    );
    assert_eq!(AppData::default(), AppData::default());
}

#[test]
fn color_defaults_to_nil() {
    assert_eq!(Color::default(), Color::NIL);
}

#[test]
fn extra_collects_extends_and_counts() {
    let mut extra: Extra = [("b".to_owned(), Value::from(1_i64))].into_iter().collect();
    extra.extend([("a".to_owned(), Value::from(2_i64))]);
    assert_eq!(extra.len(), 2);
    let keys: Vec<&str> = extra.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["b", "a"]);
}

#[test]
fn an_id_and_a_pack_path_are_as_ref_str() {
    fn text<S: AsRef<str>>(value: S) -> String {
        value.as_ref().to_owned()
    }
    let id = Id::<kind::Block>::new("minecraft:stone").unwrap();
    assert_eq!(text(&id), id.as_str());
    let path = PackPath::new("blocks/ruby.json").unwrap();
    assert_eq!(text(&path), "blocks/ruby.json");
}

#[test]
fn a_pack_path_parses_from_str_and_try_from() {
    assert_eq!(
        PackPath::from_str("blocks\\ruby.json").unwrap(),
        PackPath::new("blocks/ruby.json").unwrap()
    );
    assert_eq!(
        PackPath::try_from("a/b").unwrap(),
        PackPath::new("a/b").unwrap()
    );
    assert!(matches!(
        "a/../b".parse::<PackPath>(),
        Err(PathRejected::DotSegment { .. })
    ));
}

#[test]
fn the_strict_scalars_convert_from_their_inner_type() {
    assert_eq!(StrictInt::from(3_i32), StrictInt(3));
    assert_eq!(StrictFloat::from(1.5_f32), StrictFloat(1.5));
    assert_eq!(StrictBool::from(true), StrictBool(true));
    assert_eq!(
        StrictString::from("stone".to_owned()),
        StrictString("stone".to_owned())
    );
}

#[test]
fn experiments_iterate_by_reference() {
    let experiments: Experiments = [Experiment::ALL[0]].into_iter().collect();
    let seen: Vec<Experiment> = (&experiments).into_iter().collect();
    assert_eq!(seen, [Experiment::ALL[0]]);
    let mut count = 0;
    for _ in &experiments {
        count += 1;
    }
    assert_eq!(count, 1);
    let _ = HashSet::<Experiment>::new();
}
