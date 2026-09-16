//! Cross-version block-schema diffing: what changed between two adjacent pinned dumps,
//! plus the hand-authored rename supplement that fills the one gap the dumps cannot
//! settle on their own (architecture decision 12's fallback clause).
//!
//! A dump names every block that exists *at* one version; it carries no mapping between
//! versions. So a name that disappears between two adjacent pinned dumps and a different
//! name that appears in the same diff are two independent facts -- `blocks_removed` and
//! `blocks_added` -- never merged into a rename by this module on its own. Guessing from
//! name similarity is exactly the kind of fabrication architecture decision 11 rules out:
//! a removed name plus an added name is not provably a rename, so this generator never
//! treats it as one. [`load_renames`] reads the one place such a claim can come from
//! instead: `xtask/dumps/bedrock-samples/renames.json`, a small hand-authored table,
//! empty until a real entry with real provenance is added to it. [`diff`] merges it in --
//! and hard-fails if an entry does not actually resolve against the step it claims to
//! apply to, so a stale entry cannot silently stop taking effect.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

use super::GeneratorError;
use super::model::{PropertyKind, VersionPalette};

/// One confirmed rename, read from `renames.json`. See that file's own `_comment` for the
/// provenance bar an entry must clear.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRenameEntry {
    pub from: String,
    pub to: String,
    pub from_version: [u8; 3],
    pub to_version: [u8; 3],
    /// Required and non-empty -- see `renames.json`'s `requirements` field. Checked by
    /// [`load_renames`], not left to a downstream consumer to notice is missing.
    pub provenance: String,
}

// `_comment`/`requirements` are hand-authored provenance prose read by a human opening
// `renames.json`, not by this generator -- same treatment `manifest.rs` gives its own
// `_comment` field. `#[allow(dead_code)]` rather than dropping the fields: keeping them
// named here is what makes an unexpected-shape edit to that prose fail loudly (serde
// still type-checks the field even though nothing here reads it back), instead of
// silently parsing as a stray top-level string no one is looking at.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawRenameFile {
    #[serde(default)]
    _comment: Option<String>,
    #[serde(default)]
    requirements: Option<String>,
    renames: Vec<RawRenameEntry>,
}

/// Reads and validates `xtask/dumps/bedrock-samples/renames.json`. Every entry's
/// `provenance` must be non-empty; an entry that fails that (or the file that fails to
/// parse) is a hard error, the same "loud failure" bar the rest of this generator holds
/// to. An empty `renames` list -- the checked-in default -- parses fine and yields an
/// empty `Vec`.
///
/// Does not itself check for two entries reusing the same `from` or `to` at the same
/// step (e.g. two entries both claiming `from: "minecraft:old_name"` for the same
/// `from_version`/`to_version`): [`diff`] already retains only names actually present in
/// `blocks_removed`/`blocks_added` and removes each as it is consumed, so a second entry
/// reusing an already-consumed name surfaces there as
/// [`GeneratorError::RenameEntryNotApplicable`] rather than needing a separate check
/// here.
pub fn load_renames(dumps_dir: &Path) -> Result<Vec<RawRenameEntry>, GeneratorError> {
    let path = dumps_dir.join("renames.json");
    let text = std::fs::read_to_string(&path).map_err(|source| GeneratorError::ReadRenames {
        path: path.clone(),
        source,
    })?;
    let raw: RawRenameFile =
        serde_json::from_str(&text).map_err(|source| GeneratorError::ParseRenames {
            path: path.clone(),
            source,
        })?;

    for entry in &raw.renames {
        if entry.provenance.trim().is_empty() {
            return Err(GeneratorError::RenameMissingProvenance {
                path: path.clone(),
                from: entry.from.clone(),
                to: entry.to.clone(),
            });
        }
    }

    Ok(raw.renames)
}

/// One confirmed rename that applied at one specific step, carried into the emitted
/// diff so a downstream consumer (the upgrade walker) can resolve a name change without
/// re-reading `renames.json` itself.
#[derive(Debug, Clone, PartialEq)]
pub struct RenameChange {
    pub from: String,
    pub to: String,
}

/// One block whose declared property *set* changed between two adjacent versions.
/// Property-set changes for a renamed block (`from` -> `to` via [`RenameChange`]) are not
/// folded in here -- the rename list is empty today, so there is nothing real to carry,
/// and adding that join is deferred until a real rename entry exists to design it against
/// rather than guessing the shape now.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockChange {
    pub block: String,
    /// Sorted, deduplicated (properties are already unique per block).
    pub properties_added: Vec<String>,
    /// Sorted, deduplicated.
    pub properties_removed: Vec<String>,
}

/// How one property's value domain (or type) changed between two adjacent versions, for
/// a property name declared at both.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValueDelta {
    /// The property's declared *type* changed (e.g. `int` -> `string`) -- unobserved in
    /// the pinned dumps, but a fact this generator refuses to silently drop if it ever
    /// happens, rather than only modelling the same-type case.
    KindChanged {
        from: &'static str,
        to: &'static str,
    },
    /// Both versions declare this property as `int`; its integer domain changed.
    Int { added: Vec<i64>, removed: Vec<i64> },
    /// Both versions declare this property as `string`; its string domain changed.
    Str {
        added: Vec<String>,
        removed: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyChange {
    pub property: String,
    pub delta: PropertyValueDelta,
}

/// The full diff between two adjacent pinned versions.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionDiff {
    pub from_tag: String,
    pub to_tag: String,
    pub from_version: (u8, u8, u8),
    pub to_version: (u8, u8, u8),
    /// Sorted. A name in `next` with no counterpart in `prev` that a [`RenameChange`]
    /// does not account for.
    pub blocks_added: Vec<String>,
    /// Sorted. A name in `prev` with no counterpart in `next` that a [`RenameChange`]
    /// does not account for.
    pub blocks_removed: Vec<String>,
    /// Sorted by [`RenameChange::from`]. Always empty against the checked-in
    /// `renames.json` -- see this module's doc comment.
    pub renames: Vec<RenameChange>,
    /// Sorted by [`BlockChange::block`]. Only blocks present at both versions whose
    /// declared property set actually differs.
    pub block_changes: Vec<BlockChange>,
    /// Sorted by [`PropertyChange::property`]. Only properties declared at both versions
    /// whose kind or value domain actually differs.
    pub property_changes: Vec<PropertyChange>,
}

/// Diffs two adjacent pinned versions, merging in whichever `renames` entries apply to
/// exactly this step (`from_version`/`to_version` matching `prev`/`next`).
///
/// Fails loudly if a rename entry claims to apply to this step but does not actually
/// resolve: `from` must be a name `prev` declares and `next` does not, and `to` must be a
/// name `next` declares and `prev` does not. A stale entry (the block was not actually
/// removed the way it claims, or the target name does not exist) is exactly the kind of
/// silently-ignored input this generator's `deny_unknown_fields` discipline exists to
/// rule out elsewhere; the rename supplement gets the same treatment.
pub fn diff(
    prev: &VersionPalette,
    next: &VersionPalette,
    renames: &[RawRenameEntry],
) -> Result<VersionDiff, GeneratorError> {
    let prev_names: BTreeSet<&str> = prev.blocks.iter().map(|b| b.name.as_str()).collect();
    let next_names: BTreeSet<&str> = next.blocks.iter().map(|b| b.name.as_str()).collect();

    let mut blocks_removed: Vec<String> = prev_names
        .difference(&next_names)
        .map(|s| s.to_string())
        .collect();
    let mut blocks_added: Vec<String> = next_names
        .difference(&prev_names)
        .map(|s| s.to_string())
        .collect();

    let mut step_renames = Vec::new();
    for entry in renames.iter().filter(|r| {
        r.from_version
            == [
                prev.game_version.0,
                prev.game_version.1,
                prev.game_version.2,
            ]
            && r.to_version
                == [
                    next.game_version.0,
                    next.game_version.1,
                    next.game_version.2,
                ]
    }) {
        if !blocks_removed.iter().any(|n| n == &entry.from) {
            return Err(GeneratorError::RenameEntryNotApplicable {
                from: entry.from.clone(),
                to: entry.to.clone(),
                reason: format!(
                    "`{}` was not removed between {:?} and {:?}",
                    entry.from, entry.from_version, entry.to_version
                ),
            });
        }
        if !blocks_added.iter().any(|n| n == &entry.to) {
            return Err(GeneratorError::RenameEntryNotApplicable {
                from: entry.from.clone(),
                to: entry.to.clone(),
                reason: format!(
                    "`{}` was not added between {:?} and {:?}",
                    entry.to, entry.from_version, entry.to_version
                ),
            });
        }
        blocks_removed.retain(|n| n != &entry.from);
        blocks_added.retain(|n| n != &entry.to);
        step_renames.push(RenameChange {
            from: entry.from.clone(),
            to: entry.to.clone(),
        });
    }
    blocks_removed.sort();
    blocks_added.sort();
    step_renames.sort_by(|a, b| a.from.cmp(&b.from));

    let mut block_changes = Vec::new();
    for name in prev_names.intersection(&next_names) {
        let pb = prev
            .block(name)
            .expect("name came from prev's own block set");
        let nb = next
            .block(name)
            .expect("name came from next's own block set");
        let pprops: BTreeSet<&str> = pb.properties.iter().map(String::as_str).collect();
        let nprops: BTreeSet<&str> = nb.properties.iter().map(String::as_str).collect();
        let properties_added: Vec<String> =
            nprops.difference(&pprops).map(|s| s.to_string()).collect();
        let properties_removed: Vec<String> =
            pprops.difference(&nprops).map(|s| s.to_string()).collect();
        if !properties_added.is_empty() || !properties_removed.is_empty() {
            block_changes.push(BlockChange {
                block: name.to_string(),
                properties_added,
                properties_removed,
            });
        }
    }
    block_changes.sort_by(|a, b| a.block.cmp(&b.block));

    let prev_props: BTreeSet<&str> = prev.properties.iter().map(|p| p.name.as_str()).collect();
    let next_props: BTreeSet<&str> = next.properties.iter().map(|p| p.name.as_str()).collect();
    let mut property_changes = Vec::new();
    for name in prev_props.intersection(&next_props) {
        let pp = prev
            .property(name)
            .expect("name came from prev's own property set");
        let np = next
            .property(name)
            .expect("name came from next's own property set");
        let delta = match (&pp.kind, &np.kind) {
            (PropertyKind::Bool, PropertyKind::Bool) => None,
            (PropertyKind::Int(pv), PropertyKind::Int(nv)) => {
                let added: Vec<i64> = nv.iter().filter(|v| !pv.contains(v)).copied().collect();
                let removed: Vec<i64> = pv.iter().filter(|v| !nv.contains(v)).copied().collect();
                (!added.is_empty() || !removed.is_empty())
                    .then_some(PropertyValueDelta::Int { added, removed })
            }
            (PropertyKind::Str(pv), PropertyKind::Str(nv)) => {
                let added: Vec<String> = nv.iter().filter(|v| !pv.contains(v)).cloned().collect();
                let removed: Vec<String> = pv.iter().filter(|v| !nv.contains(v)).cloned().collect();
                (!added.is_empty() || !removed.is_empty())
                    .then_some(PropertyValueDelta::Str { added, removed })
            }
            (pk, nk) => Some(PropertyValueDelta::KindChanged {
                from: kind_label(pk),
                to: kind_label(nk),
            }),
        };
        if let Some(delta) = delta {
            property_changes.push(PropertyChange {
                property: name.to_string(),
                delta,
            });
        }
    }
    property_changes.sort_by(|a, b| a.property.cmp(&b.property));

    Ok(VersionDiff {
        from_tag: prev.tag.clone(),
        to_tag: next.tag.clone(),
        from_version: prev.game_version,
        to_version: next.game_version,
        blocks_added,
        blocks_removed,
        renames: step_renames,
        block_changes,
        property_changes,
    })
}

fn kind_label(kind: &PropertyKind) -> &'static str {
    match kind {
        PropertyKind::Bool => "bool",
        PropertyKind::Int(_) => "int",
        PropertyKind::Str(_) => "string",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::palette::model::{BlockSpec, PropertySpec};

    fn palette(
        tag: &str,
        version: (u8, u8, u8),
        blocks: Vec<(&str, &[&str])>,
        properties: Vec<(&str, PropertyKind)>,
    ) -> VersionPalette {
        VersionPalette {
            tag: tag.to_string(),
            game_version: version,
            properties: properties
                .into_iter()
                .map(|(name, kind)| PropertySpec {
                    name: name.to_string(),
                    kind,
                })
                .collect(),
            blocks: blocks
                .into_iter()
                .map(|(name, props)| BlockSpec {
                    name: name.to_string(),
                    properties: props.iter().map(|p| p.to_string()).collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn reports_added_and_removed_blocks() {
        let a = palette(
            "a",
            (1, 0, 0),
            vec![("minecraft:stone", &[]), ("minecraft:old", &[])],
            vec![],
        );
        let b = palette(
            "b",
            (1, 1, 0),
            vec![("minecraft:stone", &[]), ("minecraft:new", &[])],
            vec![],
        );

        let d = diff(&a, &b, &[]).unwrap();
        assert_eq!(d.blocks_added, vec!["minecraft:new".to_string()]);
        assert_eq!(d.blocks_removed, vec!["minecraft:old".to_string()]);
        assert!(d.renames.is_empty());
    }

    #[test]
    fn does_not_infer_a_rename_from_a_coincidental_remove_and_add() {
        let a = palette("a", (1, 0, 0), vec![("minecraft:old_name", &[])], vec![]);
        let b = palette("b", (1, 1, 0), vec![("minecraft:new_name", &[])], vec![]);

        let d = diff(&a, &b, &[]).unwrap();
        assert_eq!(d.blocks_removed, vec!["minecraft:old_name".to_string()]);
        assert_eq!(d.blocks_added, vec!["minecraft:new_name".to_string()]);
        assert!(d.renames.is_empty());
    }

    #[test]
    fn a_supplied_rename_moves_the_pair_out_of_added_removed() {
        let a = palette("a", (1, 0, 0), vec![("minecraft:old_name", &[])], vec![]);
        let b = palette("b", (1, 1, 0), vec![("minecraft:new_name", &[])], vec![]);
        let renames = vec![RawRenameEntry {
            from: "minecraft:old_name".to_string(),
            to: "minecraft:new_name".to_string(),
            from_version: [1, 0, 0],
            to_version: [1, 1, 0],
            provenance: "test fixture".to_string(),
        }];

        let d = diff(&a, &b, &renames).unwrap();
        assert!(d.blocks_removed.is_empty());
        assert!(d.blocks_added.is_empty());
        assert_eq!(
            d.renames,
            vec![RenameChange {
                from: "minecraft:old_name".to_string(),
                to: "minecraft:new_name".to_string(),
            }]
        );
    }

    #[test]
    fn a_rename_entry_that_does_not_resolve_is_a_hard_error() {
        let a = palette("a", (1, 0, 0), vec![("minecraft:stone", &[])], vec![]);
        let b = palette("b", (1, 1, 0), vec![("minecraft:stone", &[])], vec![]);
        let renames = vec![RawRenameEntry {
            from: "minecraft:nonexistent".to_string(),
            to: "minecraft:also_nonexistent".to_string(),
            from_version: [1, 0, 0],
            to_version: [1, 1, 0],
            provenance: "test fixture".to_string(),
        }];

        let err = diff(&a, &b, &renames).unwrap_err();
        assert!(matches!(
            err,
            GeneratorError::RenameEntryNotApplicable { .. }
        ));
    }

    #[test]
    fn reports_property_sets_added_and_removed_per_block() {
        let a = palette(
            "a",
            (1, 0, 0),
            vec![("minecraft:door", &["facing"])],
            vec![("facing", PropertyKind::Bool)],
        );
        let b = palette(
            "b",
            (1, 1, 0),
            vec![("minecraft:door", &["facing", "half"])],
            vec![("facing", PropertyKind::Bool), ("half", PropertyKind::Bool)],
        );

        let d = diff(&a, &b, &[]).unwrap();
        assert_eq!(d.block_changes.len(), 1);
        assert_eq!(d.block_changes[0].block, "minecraft:door");
        assert_eq!(
            d.block_changes[0].properties_added,
            vec!["half".to_string()]
        );
        assert!(d.block_changes[0].properties_removed.is_empty());
    }

    #[test]
    fn reports_a_string_property_domain_gaining_a_value() {
        let a = palette(
            "a",
            (1, 0, 0),
            vec![],
            vec![("sapling_type", PropertyKind::Str(vec!["oak".to_string()]))],
        );
        let b = palette(
            "b",
            (1, 1, 0),
            vec![],
            vec![(
                "sapling_type",
                PropertyKind::Str(vec!["oak".to_string(), "pale_oak".to_string()]),
            )],
        );

        let d = diff(&a, &b, &[]).unwrap();
        assert_eq!(d.property_changes.len(), 1);
        assert_eq!(d.property_changes[0].property, "sapling_type");
        match &d.property_changes[0].delta {
            PropertyValueDelta::Str { added, removed } => {
                assert_eq!(added, &vec!["pale_oak".to_string()]);
                assert!(removed.is_empty());
            }
            other => panic!("expected Str delta, got {other:?}"),
        }
    }

    #[test]
    fn flags_a_property_kind_change_as_a_fact_rather_than_a_value_delta() {
        let a = palette("a", (1, 0, 0), vec![], vec![("age", PropertyKind::Bool)]);
        let b = palette(
            "b",
            (1, 1, 0),
            vec![],
            vec![("age", PropertyKind::Int(vec![0, 1]))],
        );

        let d = diff(&a, &b, &[]).unwrap();
        assert_eq!(d.property_changes.len(), 1);
        match &d.property_changes[0].delta {
            PropertyValueDelta::KindChanged { from, to } => {
                assert_eq!(*from, "bool");
                assert_eq!(*to, "int");
            }
            other => panic!("expected KindChanged, got {other:?}"),
        }
    }

    #[test]
    fn identical_adjacent_dumps_produce_an_empty_diff() {
        let a = palette("a", (1, 0, 0), vec![("minecraft:stone", &[])], vec![]);
        let b = palette("b", (1, 1, 0), vec![("minecraft:stone", &[])], vec![]);

        let d = diff(&a, &b, &[]).unwrap();
        assert!(d.blocks_added.is_empty());
        assert!(d.blocks_removed.is_empty());
        assert!(d.renames.is_empty());
        assert!(d.block_changes.is_empty());
        assert!(d.property_changes.is_empty());
    }
}
