//! The single-struct chain of spawn groups: contiguity, the range model's selection, floor and
//! ceiling, no beta.

use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    Family, RangeModel, Selection, VersionChain, check_chain, select,
};
use bedrock_addon_family_spawn_groups::history::{GroupAny, GroupFamily};

fn engine() -> String {
    ENGINE_VERSION.to_string()
}

fn selects<F: Family>(version: &str) -> Selection {
    select::<F>(&FormatVersionInput::parse(version).expect("a version"))
}

#[test]
fn the_chain_is_one_struct_without_beta() {
    check_chain::<GroupFamily>().unwrap();
    assert_eq!(
        <GroupAny as VersionChain<GroupFamily>>::VERSIONS,
        [FormatVersion::new(1, 8, 0)]
    );
    const { assert!(!<GroupAny as VersionChain<GroupFamily>>::HAS_BETA) };
}

#[test]
fn spawn_groups_is_closed_to_the_engine() {
    assert_eq!(GroupFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(selects::<GroupFamily>("1.7.0"), Selection::BelowFloor);
    assert_eq!(selects::<GroupFamily>("1.11.0"), Selection::Struct(0));
    assert_eq!(selects::<GroupFamily>(&engine()), Selection::Struct(0));
}
