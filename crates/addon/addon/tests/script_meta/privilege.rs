//! Privilege lists.

use bedrock_addon::families::script::meta::model::script::PrivilegeName;
use bedrock_addon::families::script::meta::{MetaError, Privilege, PrivilegeList};

fn names(list: &[&str]) -> Vec<PrivilegeName> {
    list.iter()
        .map(|name| PrivilegeName {
            name: (*name).to_owned(),
        })
        .collect()
}

#[test]
fn default_alone_is_the_empty_set() {
    let list = PrivilegeList::from_names(&names(&["default"])).unwrap();
    assert_eq!(list.bits, Privilege::DEFAULT);
    assert!(list.bits.is_default());
    assert!(list.overlay.is_empty());
}

#[test]
fn each_flag_sets_its_bit() {
    let restricted = PrivilegeList::from_names(&names(&["restricted_execution"])).unwrap();
    assert_eq!(restricted.bits, Privilege::RESTRICTED_EXECUTION);
    assert_eq!(restricted.bits.bits(), 1);
    let early = PrivilegeList::from_names(&names(&["early_execution"])).unwrap();
    assert_eq!(early.bits, Privilege::EARLY_EXECUTION);
    assert_eq!(early.bits.bits(), 2);
}

#[test]
fn both_flags_in_the_generator_order_make_three() {
    let list =
        PrivilegeList::from_names(&names(&["restricted_execution", "early_execution"])).unwrap();
    assert_eq!(list.bits.bits(), 3);
    assert!(list.bits.contains(Privilege::RESTRICTED_EXECUTION));
    assert!(list.bits.contains(Privilege::EARLY_EXECUTION));
}

#[test]
fn default_beside_another_name_is_an_error() {
    let error = PrivilegeList::from_names(&names(&["default", "early_execution"])).unwrap_err();
    assert!(matches!(error, MetaError::DefaultNotAlone));
    let error = PrivilegeList::from_names(&names(&["read_only", "default"])).unwrap_err();
    assert!(matches!(error, MetaError::DefaultNotAlone));
}

#[test]
fn a_repeated_or_reordered_flag_is_an_error() {
    let error = PrivilegeList::from_names(&names(&["early_execution", "restricted_execution"]))
        .unwrap_err();
    assert!(matches!(error, MetaError::PrivilegeOrder));
    let error =
        PrivilegeList::from_names(&names(&["restricted_execution", "restricted_execution"]))
            .unwrap_err();
    assert!(matches!(error, MetaError::PrivilegeOrder));
}

#[test]
fn an_unknown_name_goes_to_the_overlay() {
    // `read_only`: script-generated members of v1.26.20.4.
    let list = PrivilegeList::from_names(&names(&["restricted_execution", "read_only"])).unwrap();
    assert_eq!(list.bits, Privilege::RESTRICTED_EXECUTION);
    assert_eq!(list.overlay, ["read_only"]);
    let list = PrivilegeList::from_names(&names(&["read_only"])).unwrap();
    assert_eq!(list.bits, Privilege::DEFAULT);
    assert_eq!(list.overlay, ["read_only"]);
}

#[test]
fn an_empty_list_is_an_error() {
    // The generator writes nothing only for an unregistered privilege value (its debug assert).
    let error = PrivilegeList::from_names(&[]).unwrap_err();
    assert!(matches!(error, MetaError::EmptyPrivilegeList));
}

#[test]
fn from_bits_takes_only_the_two_flags() {
    assert_eq!(
        Privilege::from_bits(3),
        Some(Privilege::RESTRICTED_EXECUTION | Privilege::EARLY_EXECUTION)
    );
    assert_eq!(Privilege::from_bits(0), Some(Privilege::DEFAULT));
    assert_eq!(Privilege::from_bits(4), None);
}
