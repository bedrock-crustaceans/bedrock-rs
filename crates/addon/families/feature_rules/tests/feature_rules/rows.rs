//! Loading one feature rule file.

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::shared::Filter;
use bedrock_addon_family_feature_rules::{
    CoordinateRange, PlacementPass, RandomDistribution, ScatterChance,
};

use super::common::*;

#[test]
#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
fn r01_a_1_13_0_rule_with_distribution_and_biome_filter_loads_unchanged() {
    let text = fixture("r01_1_13_0_loads/feature_rules/overworld_surface_flowers.json");
    let document = load_at(
        "feature_rules/overworld_surface_flowers.json",
        &text,
        &LoadContext::detached(),
    )
    .unwrap_or_else(|rejected| panic!("{:#?}", rejected.diagnostics));
    assert!(
        above_info(document.diagnostics()).is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.identifier.as_str(),
        "demo:overworld_surface_flowers"
    );
    assert_eq!(
        document.description.places_feature.as_str(),
        "demo:flower_patch_feature"
    );
    assert_eq!(document.conditions.pass(), Some(PlacementPass::Surface));
    assert!(matches!(
        document.conditions.biome_filter,
        Some(Filter::List(ref tests)) if tests.len() == 1
    ));
    let distribution = document.distribution.as_ref().expect("a distribution");
    assert_eq!(distribution.iterations, Molang::Const(4.0));
    assert!(matches!(
        distribution.scatter_chance,
        Some(ScatterChance::Percent(Molang::Const(c))) if c == 50.0
    ));
    let Some(CoordinateRange::Object(x)) = &distribution.x else {
        panic!("x is an object: {:?}", distribution.x)
    };
    assert_eq!(x.distribution, RandomDistribution::Uniform);
    assert_eq!(x.extent, [Molang::Const(0.0), Molang::Const(15.0)]);
    assert!(
        matches!(&distribution.y, Some(CoordinateRange::Single(Molang::Expr(source)))
        if source.as_str() == "query.heightmap(variable.worldx, variable.worldz)")
    );
    assert_eq!(
        document.version().written,
        Some(bedrock_addon_core::version::FormatVersion::new(1, 13, 0))
    );
}

#[test]
fn r02_the_vanilla_test_input_at_1_11_0_is_unsupported_with_the_puv_text() {
    let d = rejects(&vanilla_input("without_distribution", "1.11.0"));
    let found = with_code(&d, "core/no_parser_for_version");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].severity,
        bedrock_addon_core::diagnostic::Severity::Error
    );
    assert_eq!(
        found[0].code.to_string(),
        "unsupported version: no parser available for version 1.11.0"
    );
}

#[test]
fn r03_the_vanilla_test_input_at_the_1_12_0_floor_loads() {
    for version in ["1.12.0", "1.13.0"] {
        let document = loads(&vanilla_input("custom_feature_rule_v1_13", version));
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{version}: {:#?}",
            document.diagnostics()
        );
        assert_eq!(document.conditions.pass(), Some(PlacementPass::Surface));
        let distribution = document.distribution.as_ref().unwrap();
        assert!(matches!(&distribution.iterations, Molang::Expr(s) if s.as_str() == "2"));
        let distributions: Vec<_> = [&distribution.x, &distribution.y, &distribution.z]
            .into_iter()
            .map(|c| match c {
                Some(CoordinateRange::Object(o)) => o.distribution,
                _ => panic!("an object"),
            })
            .collect();
        assert_eq!(
            distributions,
            [
                RandomDistribution::Uniform,
                RandomDistribution::Gaussian,
                RandomDistribution::Triangle
            ]
        );
    }
}
