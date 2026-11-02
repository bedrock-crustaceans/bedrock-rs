//! An ingredient's `data` holding a number beyond `f32` (as everywhere): read as the
//! largest finite float of its sign with core's one warning at the number, in every recipe kind
//! and every place an ingredient stands, as a scalar and as an element of a list.

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, Phase, Severity};
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::recipe::{Ingredient, Recipe};
use bedrock_addon::load::LoadContext;
use bedrock_addon::molang::{Molang, MolangList};

use super::common::*;

/// One place an ingredient stands: the name of the case, the document with `@` where the
/// ingredient is, and the path of the ingredient.
const PLACES: &[(&str, &str, &str)] = &[
    (
        "shaped key",
        r##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": @}, "result": "minecraft:torch", "unlock": [{"item": "minecraft:stick"}]}}"##,
        "minecraft:recipe_shaped/key/#",
    ),
    (
        "shaped result",
        r##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": @, "unlock": [{"item": "minecraft:stick"}]}}"##,
        "minecraft:recipe_shaped/result",
    ),
    (
        "shaped result list",
        r##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": ["minecraft:torch", @], "unlock": [{"item": "minecraft:stick"}]}}"##,
        "minecraft:recipe_shaped/result/1",
    ),
    (
        "shaped unlock",
        r##"{"format_version": "1.20.10", "minecraft:recipe_shaped": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "pattern": ["#"], "key": {"#": "minecraft:stick"}, "result": "minecraft:torch", "unlock": [@]}}"##,
        "minecraft:recipe_shaped/unlock/0",
    ),
    (
        "shapeless ingredient",
        r#"{"format_version": "1.20.10", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stick", @], "result": "minecraft:torch", "unlock": [{"item": "minecraft:stick"}]}}"#,
        "minecraft:recipe_shapeless/ingredients/1",
    ),
    (
        "shapeless result",
        r#"{"format_version": "1.20.10", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stick"], "result": @, "unlock": [{"item": "minecraft:stick"}]}}"#,
        "minecraft:recipe_shapeless/result",
    ),
    (
        "shapeless unlock",
        r#"{"format_version": "1.20.10", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:r"}, "tags": ["crafting_table"], "ingredients": ["minecraft:stick"], "result": "minecraft:torch", "unlock": [@]}}"#,
        "minecraft:recipe_shapeless/unlock/0",
    ),
    (
        "furnace input",
        r#"{"format_version": "1.20.10", "minecraft:recipe_furnace": {"description": {"identifier": "demo:r"}, "tags": ["furnace"], "input": @, "output": "minecraft:cooked_beef"}}"#,
        "minecraft:recipe_furnace/input",
    ),
    (
        "furnace output",
        r#"{"format_version": "1.20.10", "minecraft:recipe_furnace": {"description": {"identifier": "demo:r"}, "tags": ["furnace"], "input": "minecraft:beef", "output": @}}"#,
        "minecraft:recipe_furnace/output",
    ),
    (
        "brewing mix input",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_mix": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": @, "reagent": "minecraft:fermented_spider_eye", "output": "minecraft:potion_type:weakness"}}"#,
        "minecraft:recipe_brewing_mix/input",
    ),
    (
        "brewing mix reagent",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_mix": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": "minecraft:potion_type:awkward", "reagent": @, "output": "minecraft:potion_type:weakness"}}"#,
        "minecraft:recipe_brewing_mix/reagent",
    ),
    (
        "brewing mix output",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_mix": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": "minecraft:potion_type:awkward", "reagent": "minecraft:fermented_spider_eye", "output": @}}"#,
        "minecraft:recipe_brewing_mix/output",
    ),
    (
        "brewing container input",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_container": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": @, "reagent": "minecraft:gunpowder", "output": "minecraft:splash_potion"}}"#,
        "minecraft:recipe_brewing_container/input",
    ),
    (
        "brewing container reagent",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_container": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": @, "output": "minecraft:splash_potion"}}"#,
        "minecraft:recipe_brewing_container/reagent",
    ),
    (
        "brewing container output",
        r#"{"format_version": "1.20.10", "minecraft:recipe_brewing_container": {"description": {"identifier": "demo:r"}, "tags": ["brewing_stand"], "input": "minecraft:potion", "reagent": "minecraft:gunpowder", "output": @}}"#,
        "minecraft:recipe_brewing_container/output",
    ),
    (
        "material reduction input",
        r#"{"format_version": "1.26.0", "minecraft:recipe_material_reduction": {"description": {"identifier": "demo:r"}, "tags": "material_reducer", "input": @, "output": [{"item": "minecraft:element_16", "count": 4}]}}"#,
        "minecraft:recipe_material_reduction/input",
    ),
    (
        "material reduction output",
        r#"{"format_version": "1.26.0", "minecraft:recipe_material_reduction": {"description": {"identifier": "demo:r"}, "tags": "material_reducer", "input": "minecraft:sulfur", "output": @}}"#,
        "minecraft:recipe_material_reduction/output",
    ),
    (
        "material reduction output list",
        r#"{"format_version": "1.26.0", "minecraft:recipe_material_reduction": {"description": {"identifier": "demo:r"}, "tags": "material_reducer", "input": "minecraft:sulfur", "output": [{"item": "minecraft:element_16", "count": 4}, @]}}"#,
        "minecraft:recipe_material_reduction/output/1",
    ),
    (
        "smithing transform template",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_transform": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": @, "base": "minecraft:diamond_axe", "addition": "minecraft:netherite_ingot", "result": "minecraft:netherite_axe"}}"#,
        "minecraft:recipe_smithing_transform/template",
    ),
    (
        "smithing transform base",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_transform": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": @, "addition": "minecraft:netherite_ingot", "result": "minecraft:netherite_axe"}}"#,
        "minecraft:recipe_smithing_transform/base",
    ),
    (
        "smithing transform addition",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_transform": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": "minecraft:diamond_axe", "addition": @, "result": "minecraft:netherite_axe"}}"#,
        "minecraft:recipe_smithing_transform/addition",
    ),
    (
        "smithing transform result",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_transform": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": "minecraft:netherite_upgrade_smithing_template", "base": "minecraft:diamond_axe", "addition": "minecraft:netherite_ingot", "result": @}}"#,
        "minecraft:recipe_smithing_transform/result",
    ),
    (
        "smithing trim template",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_trim": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": @, "base": {"tag": "minecraft:trimmable_armors"}, "addition": {"tag": "minecraft:trim_materials"}}}"#,
        "minecraft:recipe_smithing_trim/template",
    ),
    (
        "smithing trim base",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_trim": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": {"tag": "minecraft:trim_templates"}, "base": @, "addition": {"tag": "minecraft:trim_materials"}}}"#,
        "minecraft:recipe_smithing_trim/base",
    ),
    (
        "smithing trim addition",
        r#"{"format_version": "1.20.10", "minecraft:recipe_smithing_trim": {"description": {"identifier": "demo:r"}, "tags": ["smithing_table"], "template": {"tag": "minecraft:trim_templates"}, "base": {"tag": "minecraft:trimmable_armors"}, "addition": @}}"#,
        "minecraft:recipe_smithing_trim/addition",
    ),
];

/// The spellings of a number beyond `f32`, each with whether it is below zero: past `f32` and
/// inside a double, past a double, and an integer token of 42 digits.
const WIDE: &[(&str, bool)] = &[
    ("1e39", false),
    ("-1e39", true),
    ("1e999", false),
    ("-1e999", true),
    ("100000000000000000000000000000000000000000", false),
    ("-100000000000000000000000000000000000000000", true),
];

/// How `data` holds the number: the JSON around the number (`@`), and the path of the number
/// below `data`.
const FORMS: &[(&str, &str)] = &[("@", ""), ("[@]", "/0"), ("[2, [1, @]]", "/1/1")];

/// The largest finite float of the sign, as this crate's writer spells it.
fn limit(negative: bool) -> (&'static str, f32) {
    if negative {
        ("-3.4028235e38", f32::MIN)
    } else {
        ("3.4028235e38", f32::MAX)
    }
}

fn world() -> LoadContext {
    cx_world(1, 26)
}

/// The document of `place` whose ingredient is an item with `data`.
fn document_text(place: &str, data: &str) -> String {
    place.replace(
        '@',
        &format!(r#"{{"item": "minecraft:wool", "data": {data}}}"#),
    )
}

fn parsed(name: &str, text: &str) -> Document<Recipe> {
    parse_bytes("wide.json", text.as_bytes(), &world())
        .unwrap_or_else(|rejected| panic!("{name}: rejected\n{text}\n{:#?}", rejected.diagnostics))
}

/// The findings of `document` without their order.
fn findings(document: &Document<Recipe>) -> Vec<String> {
    let mut found: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|finding| format!("{finding:?}"))
        .collect();
    found.sort();
    found
}

/// Core's finding of a number read as the largest finite float, as this family raises it: the
/// family's findings carry no span.
fn is_the_warning(finding: &Diagnostic, path: &str) -> bool {
    matches!(finding.code, DiagnosticCode::InvalidValue { text: None })
        && finding.severity == Severity::Info
        && finding.phase == Phase::Parse
        && finding.location.json_path.to_string() == path
        && finding.location.span.is_none()
}

#[test]
fn ingredient_data_beyond_f32_is_the_largest_finite_float_with_one_warning_deviation() {
    for (name, place, at) in PLACES {
        for (form, below) in FORMS {
            for &(wide, negative) in WIDE {
                let (spelling, _) = limit(negative);
                let case = format!("{name}, {form} with {wide}");
                let text = document_text(place, &form.replace('@', wide));
                let document = parsed(&case, &text);
                // The same document with the largest float written out: what the wide one
                // must be read as, and the findings it has anyway.
                let plain = parsed(&case, &document_text(place, &form.replace('@', spelling)));

                // (1) One warning, core's, at the number; nothing else changes.
                let path = format!("{at}/data{below}");
                let raised: Vec<_> = document
                    .diagnostics()
                    .iter()
                    .filter(|finding| is_the_warning(finding, &path))
                    .collect();
                assert_eq!(
                    raised.len(),
                    1,
                    "{case}: one InvalidValue warning at {path}\n{:#?}",
                    document.diagnostics()
                );
                // It is core's finding, at this file of this pack.
                let mut core = bedrock_addon::molang::beyond_f32_warning(
                    raised[0].location.json_path.clone(),
                    None,
                );
                core.location.pack.clone_from(&raised[0].location.pack);
                core.location.path.clone_from(&raised[0].location.path);
                assert_eq!(raised[0], &core, "{case}");
                assert_eq!(
                    raised[0].location.path.as_ref().map(ToString::to_string),
                    Some("recipes/wide.json".to_owned()),
                    "{case}"
                );
                let mut expected = findings(&plain);
                expected.push(format!("{:?}", raised[0]));
                expected.sort();
                assert_eq!(findings(&document), expected, "{case}");
                assert!(
                    !plain
                        .diagnostics()
                        .iter()
                        .any(|finding| is_the_warning(finding, &path)),
                    "{case}: the largest float itself has no such warning"
                );

                // (2) The held value.
                assert_eq!(document.value(), plain.value(), "{case}");

                // (3) The written text spells the largest finite float.
                let written = document.to_json().unwrap();
                assert_eq!(written, plain.to_json().unwrap(), "{case}");
                assert!(
                    written.contains(spelling),
                    "{case}: {spelling} is not in\n{written}"
                );
                assert!(!written.contains(wide), "{case}:\n{written}");

                // (4) The written text reads without the warning, to the same value.
                let again = parsed(&case, &written);
                assert_eq!(findings(&again), findings(&plain), "{case}");
                assert_eq!(again.value(), document.value(), "{case}");
                assert_eq!(again.to_json().unwrap(), written, "{case}");
            }
        }
    }
}

/// The held value itself, where the comparison above holds it against another parse: a scalar
/// is the largest finite float of its sign and evaluates to the `i32` it truncates to, an
/// element of a list is that float.
#[test]
fn ingredient_data_beyond_f32_holds_the_largest_finite_float() {
    let (_, place, _) = PLACES[0];
    for &(wide, negative) in WIDE {
        let (_, value) = limit(negative);
        let document = parsed(wide, &document_text(place, wide));
        let ingredient = &shaped(&document).key["#"];
        let Ingredient::Object(object) = ingredient else {
            panic!("{wide}: {ingredient:?}");
        };
        assert_eq!(
            object.data,
            Some(MolangList::One(Molang::Const(value))),
            "{wide}"
        );
        assert_eq!(
            ingredient.aux(),
            if negative { i32::MIN } else { i32::MAX },
            "{wide}"
        );

        let document = parsed(wide, &document_text(place, &format!("[{wide}]")));
        let Ingredient::Object(object) = &shaped(&document).key["#"] else {
            panic!("{wide}");
        };
        assert_eq!(
            object.data,
            Some(MolangList::Many(vec![MolangList::One(Molang::Const(
                value
            ))])),
            "{wide}"
        );
    }
}

/// Both sides of the bound: the largest finite float and the double just above the last one
/// that rounds to it.
#[test]
fn ingredient_data_at_the_edge_of_f32() {
    let (_, place, at) = PLACES[0];
    let path = format!("{at}/data");
    let warned = |data: &str| {
        parsed(data, &document_text(place, data))
            .diagnostics()
            .iter()
            .filter(|finding| is_the_warning(finding, &path))
            .count()
    };
    // f32::MAX, and a double that still rounds to it.
    assert_eq!(warned("3.4028235e38"), 0);
    assert_eq!(warned("-3.4028235e38"), 0);
    assert_eq!(warned("3.40282356e38"), 0);
    // The first doubles whose nearest float is an infinity.
    assert_eq!(warned("3.4028236e38"), 1);
    assert_eq!(warned("-3.4028236e38"), 1);
}
