//! Stage 1: the manifest. One test per fixture folder; `common::check` requires
//! everything the folder's `expected.toml` states.

use crate::common::check;

#[test]
fn sm01_control_legacy() {
    check("sm01_control_legacy");
}

#[test]
fn sm02_type_javascript_legacy() {
    check("sm02_type_javascript_legacy");
}

#[test]
fn sm03_type_javascript_v3() {
    check("sm03_type_javascript_v3");
}

#[test]
fn sm04_version_star_legacy() {
    check("sm04_version_star_legacy");
}

#[test]
fn sm05_version_partial_legacy_two_parts() {
    check("sm05_version_partial_legacy/two_parts");
}

#[test]
fn sm05_version_partial_legacy_one_part() {
    check("sm05_version_partial_legacy/one_part");
}

#[test]
fn sm06_version_number_legacy() {
    check("sm06_version_number_legacy");
}

#[test]
fn sm07_version_array_legacy() {
    check("sm07_version_array_legacy");
}

#[test]
fn sm08_version_array_v3() {
    check("sm08_version_array_v3");
}

#[test]
fn sm09_version_invalid_v3_abc() {
    check("sm09_version_invalid_v3/abc");
}

#[test]
fn sm09_version_invalid_v3_star() {
    check("sm09_version_invalid_v3/star");
}

#[test]
fn sm10_version_relaxed_v3_engine_1_26_36() {
    check("sm10_version_relaxed_v3/engine_1_26_36");
}

#[test]
fn sm10_version_relaxed_v3_engine_1_26_30() {
    check("sm10_version_relaxed_v3/engine_1_26_30");
}

#[test]
fn sm10_version_relaxed_v3_engine_1_26_36_older_pack() {
    check("sm10_version_relaxed_v3/engine_1_26_36_older_pack");
}

#[test]
fn sm10_version_relaxed_v3_engine_1_26_20() {
    check("sm10_version_relaxed_v3/engine_1_26_20");
}

#[test]
fn sm10_version_relaxed_v3_engine_1_26_29() {
    check("sm10_version_relaxed_v3/engine_1_26_29");
}

#[test]
fn sm11_name_and_uuid_legacy() {
    check("sm11_name_and_uuid_legacy");
}

#[test]
fn sm12_name_and_uuid_v3() {
    check("sm12_name_and_uuid_v3");
}

#[test]
fn sm13_optional_legacy() {
    check("sm13_optional_legacy");
}

#[test]
fn sm14_build_metadata() {
    check("sm14_build_metadata");
}

#[test]
fn sm15_unknown_member_v3() {
    check("sm15_unknown_member_v3");
}
