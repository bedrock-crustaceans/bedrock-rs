//! The steps of the feature chains, run on a hand-built input: what the step leaves absent.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::testing::{Checks, Run, step_output};
use bedrock_addon_family_feature::history::v1_13_0;

/// The JSON at `path` (`/`-separated) of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/')
        .try_fold(value, |v, key| v.as_object().and_then(|o| o.get(key)))
}

fn input(json: &str) -> Value {
    facet_json::from_str(json).unwrap()
}

#[test]
fn the_scatter_step_inserts_no_parameter_the_file_does_not_have() {
    let out = step_output::<v1_13_0::ScatterFeature>(
        &input(
            r#"{"description": {"identifier": "demo:x"}, "places_feature": "demo:y", "iterations": 1, "x": 0, "y": 0, "z": 0}"#,
        ),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs");
    for member in ["scatter_chance", "coordinate_eval_order"] {
        assert_eq!(
            at(&out.payload, &format!("distribution/{member}")),
            None,
            "{member}"
        );
    }
    assert!(
        at(&out.payload, "places_feature").is_some(),
        "places_feature left the root"
    );
}

#[test]
fn the_snap_to_surface_step_inserts_no_allow_non_air_placement() {
    let out = step_output::<v1_13_0::SnapToSurfaceFeature>(
        &input(
            r#"{"description": {"identifier": "demo:x"}, "feature_to_snap": "demo:y", "vertical_search_range": 4}"#,
        ),
        Run::default(),
        &Checks::default(),
    )
    .expect("the step runs");
    assert_eq!(at(&out.payload, "allow_non_air_placement"), None);
    assert!(
        at(&out.payload, "search_range").is_some(),
        "vertical_search_range was not renamed"
    );
}
