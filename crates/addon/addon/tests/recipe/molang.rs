//! Ingredient `data` as Molang: read with the pack's
//! version, compiled with the `molang` feature, resolved to a number at load when it folds.

#![allow(dead_code, reason = "some helpers serve only the `molang` build")]

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::recipe::{Ingredient, IngredientObject};
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::{Molang, MolangList};

use super::common::*;

/// `cx` with the load-time compile check.
pub fn with_check(cx: LoadContext) -> LoadContext {
    #[cfg(feature = "molang")]
    {
        let mut cx = cx;
        cx.molang = Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        ));
        cx
    }
    #[cfg(not(feature = "molang"))]
    {
        cx
    }
}

/// A third-party pack of the engine version with the compile check.
pub fn cx_molang() -> LoadContext {
    with_check(cx())
}

fn object(ingredient: &Ingredient) -> &IngredientObject {
    match ingredient {
        Ingredient::Object(object) => object,
        Ingredient::Name(name) => panic!("not an object: {name}"),
    }
}

fn not_checked(
    document: &bedrock_addon::document::Document<bedrock_addon::families::recipe::Recipe>,
) -> u32 {
    document
        .diagnostics()
        .iter()
        .filter_map(|d| match d.code {
            DiagnosticCode::MolangNotChecked { fields } => {
                assert_eq!(d.severity, Severity::Info);
                Some(fields)
            }
            _ => None,
        })
        .sum()
}

fn invalid_molang(
    document: &bedrock_addon::document::Document<bedrock_addon::families::recipe::Recipe>,
) -> Vec<&bedrock_addon::diagnostic::Diagnostic> {
    document
        .diagnostics()
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::InvalidMolang { .. }))
        .collect()
}

#[test]
fn data_molang_takes_the_packs_molang_version() {
    for cx in [cx_mev(1, 16, 0), cx_mev(1, 21, 100), cx()] {
        let document = loaded("data_molang.json", &with_check(cx.clone()));
        let data = object(&shaped(&document).key["#"]).data.as_ref().unwrap();
        let MolangList::One(Molang::Expr(source)) = data else {
            panic!("{data:?}");
        };
        assert_eq!(source.as_str(), "query.is_baby");
        assert_eq!(source.raw_version(), Some(cx.molang_version));
    }
}

#[cfg(feature = "molang")]
#[test]
fn data_molang_forms_resolve_to_3_1_and_2() {
    let document = loaded("data_molang_forms.json", &cx_molang());
    silent(document.diagnostics());
    let key = &shaped(&document).key;
    assert_eq!([key["#"].aux(), key["$"].aux(), key["%"].aux()], [3, 1, 2]);
}

#[test]
fn data_molang_forms_without_the_check_resolve_constants_only() {
    let document = loaded("data_molang_forms.json", &cx());
    let key = &shaped(&document).key;
    assert_eq!(
        [key["#"].aux(), key["$"].aux(), key["%"].aux()],
        [3, 1, 32767]
    );
    assert_eq!(not_checked(&document), 1);
}

#[cfg(feature = "molang")]
#[test]
fn data_molang_unparsable_is_a_kept_error_and_reads_as_zero() {
    let document = loaded("data_molang_unparsable.json", &cx_molang());
    let found = invalid_molang(&document);
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    // The parser's line is level 3 and the recipe loads: a kept error.
    assert_eq!(found[0].severity, Severity::Error);
    assert!(found[0].kept);
    let Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(detail)) =
        found[0].detail.as_deref()
    else {
        panic!("{:#?}", found[0]);
    };
    assert_eq!(
        detail.code(),
        bedrock_addon::molang::DiagCode::StatementForm
    );
    assert_eq!(
        detail.message(),
        "Error: binary Add '+' operator at end of expression\n"
    );
    assert_eq!(shaped(&document).key["#"].aux(), 0);
}

/// Vanilla evaluates `q.is_baby` with no actor to 0; evaluation needs molang's `vm`, which is
/// out of scope: the expression compiles, is not evaluated, and `data` stays "any".
#[cfg(feature = "molang")]
#[test]
fn data_molang_query_compiles_but_is_not_evaluated_deviation() {
    let document = loaded("data_molang_query.json", &cx_molang());
    assert!(
        invalid_molang(&document).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    clean(document.diagnostics());
    assert_eq!(not_checked(&document), 1);
    assert_eq!(shaped(&document).key["#"].aux(), 32767);
}

/// `ExpressionNode::parse` logs the line at level 3 (`_contentLog(true, 3, 22)`,
/// then `clear()`) and the recipe loads: a kept error.
#[test]
fn data_molang_object_is_not_a_molang_value() {
    let document = loaded("data_molang_object.json", &cx_molang());
    let finding = one(
        document.diagnostics(),
        "Type not convertible to Molang expression: {}",
        Severity::Error,
    );
    assert!(finding.kept);
    assert!(matches!(finding.code, DiagnosticCode::InvalidType { .. }));
    assert_eq!(args(finding), ["7"]);
    let ingredient = &shaped(&document).key["#"];
    assert_eq!(ingredient.aux(), 0);
    // kept as written
    assert!(object(ingredient).extra.get("data").is_some());
}

#[cfg(feature = "molang")]
#[test]
fn data_molang_version_compiles_in_an_old_pack_and_not_in_a_new_one() {
    let old = loaded("data_molang_version.json", &with_check(cx_mev(1, 16, 0)));
    assert!(invalid_molang(&old).is_empty(), "{:#?}", old.diagnostics());
    let new = loaded("data_molang_version.json", &with_check(cx_mev(1, 21, 100)));
    let found = invalid_molang(&new);
    assert!(!found.is_empty(), "{:#?}", new.diagnostics());
    assert!(
        found
            .iter()
            .all(|d| d.severity == Severity::Error && d.kept),
        "{found:#?}"
    );
    assert!(found.iter().any(|d| matches!(
        d.detail.as_deref(),
        Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(m))
            if m.code() == bedrock_addon::molang::DiagCode::StringMisuse
    )));
    assert_eq!(shaped(&new).key["#"].aux(), 0);
}

#[cfg(feature = "molang")]
#[test]
fn data_molang_empty_is_a_rejected_expression_reported_as_an_information() {
    let document = loaded("data_molang_empty.json", &with_check(cx_mev(1, 16, 0)));
    let found = invalid_molang(&document);
    assert!(!found.is_empty(), "{:#?}", document.diagnostics());
    // Below Molang version 4 the game rejects an empty text without a line: an information of
    // ours.
    assert!(
        found.iter().all(|d| d.severity == Severity::Info),
        "{found:#?}"
    );
    assert!(found.iter().any(|d| matches!(
        d.detail.as_deref(),
        Some(bedrock_addon::diagnostic::DiagnosticDetail::Molang(m))
            if m.code() == bedrock_addon::molang::DiagCode::Syntax
    )));
    assert_eq!(shaped(&document).key["#"].aux(), 0);
}

/// `data_value` is derived at load and not written: a hand-built ingredient equals its
/// re-parse, whatever its `data` evaluated to.
#[cfg(feature = "molang")]
#[test]
fn a_hand_built_data_expression_equals_its_reparse() {
    use bedrock_addon::document::Content;
    use bedrock_addon::families::recipe::{Description, Output, Recipe, Shaped, Tags};
    use bedrock_addon::molang::MolangSource;

    let cx = cx_molang();
    let source = MolangSource::string("1 + 1", cx.molang_version.0);
    let ingredient = Ingredient::Object(IngredientObject {
        item: Some("minecraft:wool".to_owned()),
        data: Some(MolangList::One(Molang::Expr(source))),
        ..IngredientObject::default()
    });
    let recipe = Recipe::Shaped(Shaped {
        format_version: Some(bedrock_addon::version::FormatVersion::new(1, 12, 0)),
        description: Description {
            identifier: bedrock_addon::id::RecipeId::new("demo:wool").unwrap(),
            ..Description::default()
        },
        tags: Some(Tags::One("crafting_table".to_owned())),
        pattern: vec!["#".to_owned()],
        key: [("#".to_owned(), ingredient)].into_iter().collect(),
        result: Output::One(Ingredient::Name("minecraft:torch".to_owned())),
        ..Shaped::default()
    });
    let document = parse_bytes("wool.json", recipe.to_json().unwrap().as_bytes(), &cx).unwrap();
    assert_eq!(shaped(&document).key["#"].aux(), 2);
    assert_eq!(document.value(), &recipe);
}

/// A list with an element that is no value: the game's line for it was not traced, so the
/// crate's note is an information; `data` reads 0.
#[test]
fn a_data_list_with_a_null_element_is_an_information() {
    let text = br##"{"format_version": "1.12", "minecraft:recipe_shaped": {"description": {"identifier": "demo:list"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": {"item": "minecraft:wool", "data": [1, null]}}, "result": {"item": "minecraft:torch"}}}"##;
    let document = parse_bytes("list.json", text, &cx_molang()).unwrap();
    let found: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::InvalidType { text: None }))
        .collect();
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    assert_eq!(found[0].severity, Severity::Info);
    assert_eq!(shaped(&document).key["#"].aux(), 0);
}

#[test]
fn a_data_list_is_read_but_not_a_number() {
    let text = br##"{"format_version": "1.12", "minecraft:recipe_shaped": {"description": {"identifier": "demo:list"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": {"item": "minecraft:wool", "data": [1, "2"]}}, "result": {"item": "minecraft:torch"}}}"##;
    let document = parse_bytes("list.json", text, &cx_molang()).unwrap();
    let finding = document
        .diagnostics()
        .iter()
        .find(|d| matches!(d.code, DiagnosticCode::InvalidType { text: None }))
        .unwrap_or_else(|| panic!("{:#?}", document.diagnostics()));
    // The game has no line for it: an information of ours.
    assert_eq!(finding.severity, Severity::Info);
    let ingredient = &shaped(&document).key["#"];
    assert!(matches!(object(ingredient).data, Some(MolangList::Many(_))));
    assert_eq!(ingredient.aux(), 32767);
}
