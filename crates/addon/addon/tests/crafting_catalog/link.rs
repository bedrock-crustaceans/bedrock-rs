//! The catalogue's link rows: the placement warnings
//! and the group-name lint run in `StackSet::link`. Their bodies are written against it; the checkers themselves are tested in
//! `src/item/catalog/tests.rs`.

use bedrock_addon::diagnostic::{Diagnostics, Kind, Severity, codes};
use bedrock_addon::pack::{PackRepository, PackSelection, WorldPacks};

use super::stacks::*;

/// The link findings of a world whose behaviour packs are `packs`, bottom first: the packs
/// from the package are the base game's, the others the world's.
fn link(packs: &[Spec]) -> Diagnostics {
    let mut repository = PackRepository::new();
    let mut behavior = Vec::new();
    for spec in packs.iter().rev() {
        let pack = spec.open();
        if spec.origin != bedrock_addon::pack::PackOrigin::Package {
            behavior.push(PackSelection {
                pack_id: pack.id().uuid,
                version: pack.id().version.clone(),
                subpack: None,
            });
        }
        repository.add_pack(pack, spec.origin);
    }
    let world = WorldPacks {
        behavior,
        resource: Vec::new(),
    };
    repository.stacks(&world, registry_options()).link()
}

/// The findings of `d` at `severity` of the kind `kind`, as texts.
fn texts_of_kind(d: &Diagnostics, kind: Kind, severity: Severity) -> Vec<String> {
    d.iter()
        .filter(|f| f.code.kind() == kind && f.severity == severity)
        .map(|f| f.code.to_string())
        .collect()
}

#[test]
fn m10_an_item_placed_outside_its_own_category_and_group_is_two_warnings() {
    let found = link(&fixture_packs("m10_category_group_warnings"));
    assert_eq!(
        texts_of_kind(&found, Kind::InvalidValue, Severity::Warning),
        [
            "The item custom:item1 was created with the category set to 'nature', and is now being set to 'construction'",
            "The item custom:item1 was created with the group set to 'custom:other_group', but is now being set to 'custom:my_name'",
        ]
    );
    assert!(
        found
            .iter()
            .filter(|f| f.severity > Severity::Info)
            .all(|f| f.code.kind() == Kind::InvalidValue)
    );
}

#[test]
fn m11_a_minecraft_item_in_another_group_is_no_link_warning() {
    let found = link(&fixture_packs("m11_minecraft_no_warning"));
    assert_eq!(
        texts_of_kind(&found, Kind::InvalidValue, Severity::Warning),
        [] as [&str; 0]
    );
}

#[test]
fn a_new_group_name_no_lang_file_defines_is_an_info() {
    let found = link(&fixture_packs("m10_category_group_warnings"));
    let lints: Vec<_> = found
        .iter()
        .filter(|f| f.is(&codes::UNRESOLVED_LANG_KEY))
        .collect();
    assert_eq!(lints.len(), 1, "{found:#?}");
    assert_eq!(lints[0].severity, Severity::Info);
}
