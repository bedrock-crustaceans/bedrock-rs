//! Stage 2: plugin discovery. One test per fixture folder; `common::check` requires
//! everything the folder's `expected.toml` states.

use crate::common::check;

#[test]
fn sm20_two_script_modules() {
    check("sm20_two_script_modules");
}

#[test]
fn sm21_language_cases_absent() {
    check("sm21_language_cases/absent");
}

#[test]
fn sm21_language_cases_empty() {
    check("sm21_language_cases/empty");
}

#[test]
fn sm21_language_cases_upper() {
    check("sm21_language_cases/upper");
}

#[test]
fn sm21_language_cases_mixed() {
    check("sm21_language_cases/mixed");
}

#[test]
fn sm22_language_bad_typescript() {
    check("sm22_language_bad/typescript");
}

#[test]
fn sm22_language_bad_js() {
    check("sm22_language_bad/js");
}

#[test]
fn sm22_language_bad_java_script() {
    check("sm22_language_bad/java_script");
}

#[test]
fn sm23_entry_extension_ts() {
    check("sm23_entry_extension/ts");
}

#[test]
fn sm23_entry_extension_upper_js() {
    check("sm23_entry_extension/upper_js");
}

#[test]
fn sm23_entry_extension_mjs() {
    check("sm23_entry_extension/mjs");
}

#[test]
fn sm23_entry_extension_absent() {
    check("sm23_entry_extension/absent");
}

#[test]
fn sm23_entry_extension_absent_v3() {
    check("sm23_entry_extension/absent_v3");
}

#[test]
fn sm24_entry_missing_file() {
    check("sm24_entry_missing_file");
}

#[test]
fn sm25_entry_without_root() {
    check("sm25_entry_without_root");
}

#[test]
fn sm26_entry_root_only() {
    check("sm26_entry_root_only");
}

#[test]
fn sm27_beta_selector_off() {
    check("sm27_beta_selector/off");
}

#[test]
fn sm27_beta_selector_off_v3() {
    check("sm27_beta_selector/off_v3");
}

#[test]
fn sm27_beta_selector_on() {
    check("sm27_beta_selector/on");
}

#[test]
fn sm27_beta_selector_on_explicit() {
    check("sm27_beta_selector/on_explicit");
}

#[test]
fn sm28_beta_explicit_unshipped_minor_8_off() {
    check("sm28_beta_explicit_unshipped/minor_8_off");
}

#[test]
fn sm28_beta_explicit_unshipped_minor_0_off() {
    check("sm28_beta_explicit_unshipped/minor_0_off");
}

#[test]
fn sm28_beta_explicit_unshipped_minor_8_on() {
    check("sm28_beta_explicit_unshipped/minor_8_on");
}

#[test]
fn sm29_major_zero_off() {
    check("sm29_major_zero/off");
}

#[test]
fn sm29_major_zero_on() {
    check("sm29_major_zero/on");
}

#[test]
fn sm29_major_zero_unshipped_on() {
    check("sm29_major_zero/unshipped_on");
}

#[test]
fn sm30_beta_optional_v3_off() {
    check("sm30_beta_optional_v3/off");
}

#[test]
fn sm30_beta_optional_v3_on() {
    check("sm30_beta_optional_v3/on");
}

#[test]
fn sm30_beta_optional_v3_not_optional() {
    check("sm30_beta_optional_v3/not_optional");
}

#[test]
fn sm31_internal_selector() {
    check("sm31_internal/selector");
}

#[test]
fn sm31_internal_explicit() {
    check("sm31_internal/explicit");
}

#[test]
fn sm31_internal_beta_on() {
    check("sm31_internal/beta_on");
}

#[test]
fn sm31_internal_package_origin() {
    check("sm31_internal/package_origin");
}

#[test]
fn sm31_internal_allow_internal() {
    check("sm31_internal/allow_internal");
}

#[test]
fn sm31_internal_optional_v3() {
    check("sm31_internal/optional_v3");
}

#[test]
fn sm32_alpha_selector() {
    check("sm32_alpha/selector");
}

#[test]
fn sm32_alpha_explicit() {
    check("sm32_alpha/explicit");
}

#[test]
fn sm32_alpha_beta_on() {
    check("sm32_alpha/beta_on");
}

#[test]
fn sm33_gametest_explicit() {
    check("sm33_gametest/explicit");
}

#[test]
fn sm33_gametest_selector() {
    check("sm33_gametest/selector");
}

#[test]
fn sm33_gametest_on() {
    check("sm33_gametest/on");
}

#[test]
fn sm33_gametest_optional_v3() {
    check("sm33_gametest/optional_v3");
}

#[test]
fn sm34_unknown_module_nonexistent() {
    check("sm34_unknown_module/nonexistent");
}

#[test]
fn sm34_unknown_module_case() {
    check("sm34_unknown_module/case");
}

#[test]
fn sm34_unknown_module_no_permissions() {
    check("sm34_unknown_module/no_permissions");
}

#[test]
fn sm35_not_allowed() {
    check("sm35_not_allowed");
}

#[test]
fn sm36_uuid_dependency() {
    check("sm36_uuid_dependency");
}

#[test]
fn sm37_script_eval_legacy() {
    check("sm37_script_eval/legacy");
}

#[test]
fn sm37_script_eval_v3() {
    check("sm37_script_eval/v3");
}

#[test]
fn sm37_script_eval_without() {
    check("sm37_script_eval/without");
}
