//! Spawn groups: `SpawnGroup`, `Condition`.
//!
//! Raid spawn groups (`definitions/spawn_groups/*.json` of the app data, `minecraft:spawn_groups`).
//! Spawn groups use the 1.8.0 condition ([`Condition`]) with a larger distance cap.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod legacy;
mod validate;

pub use bedrock_addon_shared_spawn::{
    AboveBlockFilter, Condition, DelayFilter, DensityLimit, Difficulty, DifficultyFilter, Herd,
    HerdInfo, MobEventFilter, MobSpawnRules, OneOrMany, Permutation, PopulationControl,
    RangeFilter, VillageFilter, Weight, WorldAgeFilter,
};

use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::document::{
    Document, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Origin, Registry, Rejected,
    default_path,
};
use bedrock_addon_core::id::SpawnGroupId;
use bedrock_addon_core::id::kind;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::{AppDataPlace, CollectionLoader, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, DocumentRead, Envelope, EnvelopeCheck, Family, Loaded, MissingVersion,
    NoComponents, Single, ValidateCx,
};
use bedrock_addon_shared_spawn::legacy::{GROUPS_KEY, V1_8_0};
use bedrock_addon_shared_spawn::load::{
    FileLoad, Parsed, combine, contribution, load_family, load_family_as_written, load_file, origin,
};

/// A raid spawn group (`minecraft:spawn_groups`): the conditions the server spawns as one batch
/// when a raid wave names the group (`minecraft:raid_group_<wave>`).
///
/// JSON: `{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier"},
/// "conditions": [{…}]}}`, read like standard spawn rules except that the spawn-distance cap is
/// 1024 and nothing is warned about the root, the payload or the description. Spawn groups come
/// only from the app data (`definitions/spawn_groups/*.json`), never from a pack.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:spawn_groups")]
#[doc(alias = "spawn_groups")]
pub struct SpawnGroup {
    /// The document's `format_version`: selects the Molang version of the block descriptors'
    /// `tags`.
    pub format_version: FormatVersion,
    /// The group (JSON `description`).
    pub description: GroupDescription,
    /// The conditions (JSON `conditions`); a group without any is never registered.
    pub conditions: Option<Vec<Condition>>,
    /// Other members of `minecraft:spawn_groups` (kept, ignored).
    pub extra: UnknownMembers,
    /// Root members other than `format_version` and `minecraft:spawn_groups` (kept, ignored).
    pub root_extra: UnknownMembers,
}

/// `minecraft:spawn_groups/description`.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupDescription {
    /// The group (JSON `identifier`, required), kept verbatim.
    pub identifier: SpawnGroupId,
    /// Read and kept; unused by spawn groups (JSON `population_control`).
    pub population_control: Option<String>,
    /// Other members (kept, ignored).
    pub extra: UnknownMembers,
}

impl SpawnGroup {
    /// What the runtime keeps of the group: one
    /// [`MobSpawnRules`] per condition, in document order, with the spawn-distance cap 1024. A
    /// condition's weight and biome filter are not part of it.
    pub fn spawn_rules(&self) -> Vec<MobSpawnRules> {
        self.conditions
            .iter()
            .flatten()
            .map(|c| c.rules_with(bedrock_addon_shared_spawn::condition::SPAWN_GROUPS))
            .collect()
    }
}

/// The app-data directory of spawn groups (`SPAWN_GROUPS_FILE_TYPE`).
const GROUPS_DIRECTORY: &str = "definitions/spawn_groups";

/// The `spawn_groups` family: one struct, `spawn_groups` (`v1_8_0`), with no `Upgrade` step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct GroupFamily;

/// A spawn-group document as written: the one struct, the newest.
#[doc(hidden)]
pub type GroupHistory = Single<GroupFamily>;

impl Family for GroupFamily {
    const NAME: &'static str = "spawn group";
    const PAYLOAD_KEY: &'static str = GROUPS_KEY;
    const FLOOR: FormatVersion = V1_8_0;
    const ENVELOPE: Envelope = Envelope {
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // The loader reads the raw text, never the cereal path.
        check: EnvelopeCheck::Never,
        // A missing version is an error, as in vanilla ("no valid version passed by the caller or
        // found in the input").
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = SpawnGroup;
    type Beta = NoBeta<kind::SpawnGroup>;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(group) = v {
            validate::group(group, cx);
        }
    }
}

/// Loads one spawn-group file.
pub(crate) fn parse_group(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileLoad<SpawnGroup> {
    let origin = origin(pack, side, path);
    let (identifier, result) = load_file(&origin, bytes, &codes::FAILED_GROUPS, |text, origin| {
        load_family::<GroupFamily, _>(text, cx, origin, std::convert::identity)
    });
    // A group without conditions is never registered.
    let registered = result
        .as_ref()
        .is_ok_and(|document| document.conditions.as_ref().is_some_and(|c| !c.is_empty()));
    FileLoad {
        identifier,
        result,
        registered,
    }
}

impl JsonDocument for SpawnGroup {
    type Family = GroupFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Loads one spawn-group file (the shared spawn steps around the chain). A group without
    /// conditions loads (with its warning) although a stack does not register it.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<GroupFamily> {
        let loaded = parse_group(origin.pack.clone(), origin.side, &origin.path, bytes, cx);
        match loaded.result {
            Ok(document) => DocumentRead::Stable(document),
            Err(rejected) => DocumentRead::Rejected(loaded.identifier, rejected),
        }
    }

    /// Parses one spawn-group file without upgrading it, through the same spawn steps as
    /// [`read`](JsonDocument::read): a root that is no object, and the closing line of a file
    /// that fails, are told as a read tells them.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<GroupHistory>, Rejected> {
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        let (_, result) = load_file(&origin, bytes, &codes::FAILED_GROUPS, |text, origin| {
            load_family_as_written::<GroupFamily>(text, cx, origin)
        });
        result
    }
}

impl JsonPlaced for SpawnGroup {
    /// `definitions/spawn_groups/<name>.json` (an app-data path: no pack reads it).
    fn place(&self) -> PackPath {
        default_path(GROUPS_DIRECTORY, &self.description.identifier)
    }
}

impl JsonDefinition for SpawnGroup {
    type Kind = kind::SpawnGroup;

    fn identifier(&self) -> &SpawnGroupId {
        &self.description.identifier
    }
}

fn parse_group_file(file: &WalkFile, cx: &LoadContext) -> Parsed<SpawnGroup> {
    contribution(
        parse_group(file.pack.clone(), file.side, &file.path, &file.bytes, cx),
        |id| SpawnGroupId::parse(id),
        |document| document.description.identifier.clone(),
    )
}

/// The loader of `bp.spawn_groups()`, the family's own: a group that loads without conditions is
/// not registered but its findings are (`Parsed::Unregistered`), which `load::registry` has no
/// shape for. Only the app data's `definitions/spawn_groups/*.json`
/// (not recursive, extension exactly `json`), in file-name order, a later file with the same
/// identifier replacing the earlier one with a `DuplicateIdentifier` warning; no pack is read.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<SpawnGroup>, Parsed<SpawnGroup>> = CollectionLoader {
    walk: Walk::json_children_in(&[]).app_data(&[GROUPS_DIRECTORY], AppDataPlace::After),
    parse: parse_group_file,
    combine,
    needs: &[],
    then: None,
};

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn spawn_groups_reads_as_pinned() {
        assert_reading_table(
            "GroupFamily",
            &reading_table::<super::GroupFamily>(),
            r"
                text: Root, clamps beyond a double: true, duplicates kept in: []
                envelope: Never
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.8.0..=1.8.0 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Diagnostic;
    use bedrock_addon_core::document::Content;

    fn path() -> PackPath {
        PackPath::new("definitions/spawn_groups/raid.json").unwrap()
    }

    /// The findings of `text` read whole and read as written; both must be rejections.
    fn both_findings(text: &str) -> (Vec<Diagnostic>, Vec<Diagnostic>) {
        let cx = LoadContext::detached();
        let read = <SpawnGroup as Content>::parse(&path(), text.as_bytes(), &cx)
            .expect_err("the read rejects it");
        let written = <SpawnGroup as JsonDocument>::parse_as_written(&path(), text.as_bytes(), &cx)
            .expect_err("the as-written parse rejects it");
        (
            read.diagnostics.iter().cloned().collect(),
            written.diagnostics.iter().cloned().collect(),
        )
    }

    #[test]
    fn a_root_that_is_no_object_is_rejected_alike_read_whole_and_as_written() {
        for text in [
            r#"["format_version", 1]"#,
            r#"[{"format_version": "1.8.0"}]"#,
            "7",
        ] {
            let (read, written) = both_findings(text);
            assert!(
                read.iter().any(|d| d.code.name() == "DocumentRejected"),
                "{text}: {read:?}"
            );
            assert_eq!(written, read, "{text}");
        }
    }

    #[test]
    fn a_payload_that_fails_is_rejected_alike_read_whole_and_as_written() {
        let text = r#"{"format_version": "1.8.0", "minecraft:spawn_groups": {"description": {}}}"#;
        let (read, written) = both_findings(text);
        assert_eq!(written, read);
    }
}
