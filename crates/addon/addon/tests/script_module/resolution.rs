//! Stage 3: version resolution. One test per fixture folder; `common::check` requires
//! everything the folder's `expected.toml` states.

use crate::common::check;

#[test]
fn sm40_caret_major_2() {
    check("sm40_caret/major_2");
}

#[test]
fn sm40_caret_major_1() {
    check("sm40_caret/major_1");
}

#[test]
fn sm41_unshipped_major_9() {
    check("sm41_unshipped/major_9");
}

#[test]
fn sm41_unshipped_minor_99() {
    check("sm41_unshipped/minor_99");
}

#[test]
fn sm41_unshipped_rc() {
    check("sm41_unshipped/rc");
}

#[test]
fn sm41_unshipped_beta_1() {
    check("sm41_unshipped/beta_1");
}

#[test]
fn sm42_optional_unshipped_v3() {
    check("sm42_optional_unshipped_v3");
}

#[test]
fn sm43_same_major_merge_two_versions() {
    check("sm43_same_major_merge/two_versions");
}

#[test]
fn sm43_same_major_merge_twice() {
    check("sm43_same_major_merge/twice");
}

#[test]
fn sm44_cross_major_conflict() {
    check("sm44_cross_major_conflict");
}

#[test]
fn sm45_transitive_conflict() {
    check("sm45_transitive_conflict");
}

#[test]
fn sm46_transitive_ok() {
    check("sm46_transitive_ok");
}

#[test]
fn sm47_beta_over_stable_release_first() {
    check("sm47_beta_over_stable/release_first");
}

#[test]
fn sm47_beta_over_stable_beta_first() {
    check("sm47_beta_over_stable/beta_first");
}

#[test]
fn sm48_conflict_and_unshipped() {
    check("sm48_conflict_and_unshipped");
}

#[test]
fn sm49_beta_with_transitive_stable() {
    check("sm49_beta_with_transitive_stable");
}

#[test]
fn sm50_transitive_beta_conflict() {
    check("sm50_transitive_beta_conflict");
}
