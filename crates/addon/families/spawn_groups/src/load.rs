//! Loading spawn groups: the family's parameters, `Content` / `Definition`, the walk and the
//! combine rule.

use super::SpawnGroup;
use super::history::{GroupAny, GroupFamily};
use super::validate;
use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Placed, Registry, Rejected, default_path,
};
use bedrock_addon_core::id::{SpawnGroupId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    EnvelopeCheck, Family, Loaded, MissingVersion, NoComponents, RangeModel, ValidateCx,
};
use bedrock_addon_support::spawn::legacy::{GROUPS_KEY, V1_8_0};
use bedrock_addon_support::spawn::load::{
    FileLoad, Parsed, combine, contribution, is_json, load_family, load_file, origin,
};

/// The closing line of a spawn-group file that failed.
const FAILED_GROUPS: &str = "Failed to load spawn groups";
/// The app-data directory of spawn groups (`SPAWN_GROUPS_FILE_TYPE`).
const GROUPS_DIRECTORY: &str = "definitions/spawn_groups";

impl Family for GroupFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "spawn group";
    const PAYLOAD_KEY: &'static str = GROUPS_KEY;
    const DIRECTORIES: &'static [&'static str] = &[GROUPS_DIRECTORY];
    const FLOOR: FormatVersion = V1_8_0;
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToEngine;
    type Latest = SpawnGroup;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = GroupAny;
    // The loader reads the raw text, never the cereal path.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // A missing version is an error, as in vanilla ("no valid version passed by the caller or
    // found in the input").
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;

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
    let (identifier, result) = load_file(&origin, bytes, FAILED_GROUPS, |text, origin| {
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

impl bedrock_addon_core::__private::Sealed for SpawnGroup {}

impl Content for SpawnGroup {
    const NAME: &'static str = "spawn group";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Loads one spawn-group file (app data). A group without conditions loads (with its
    /// warning) although a stack does not register it.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_group(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).result
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.to_value(), out, style)
    }
}

impl JsonContent for SpawnGroup {}

impl Definition for SpawnGroup {
    type Kind = kind::SpawnGroup;
    type File = SpawnGroup;
    type Beta = NoBeta<kind::SpawnGroup>;
}

impl Placed for SpawnGroup {
    /// `definitions/spawn_groups/<name>.json` (an app-data path: no pack reads it).
    fn default_path(&self) -> PackPath {
        default_path(GROUPS_DIRECTORY, &self.description.identifier)
    }
}

fn parse_group_file(file: &WalkFile, cx: &LoadContext) -> Parsed<SpawnGroup> {
    contribution(
        parse_group(file.pack.clone(), file.side, &file.path, &file.bytes, cx),
        |id| SpawnGroupId::parse(id),
        |document| document.description.identifier.clone(),
    )
}

/// The loader of `bp.spawn_groups()`: only the app data's `definitions/spawn_groups/*.json`
/// (not recursive, extension exactly `json`), in file-name order, a later file with the same
/// identifier replacing the earlier one with a `DuplicateIdentifier` warning; no pack is read.
pub const fn load_groups() -> CollectionLoader<Registry<SpawnGroup>, Parsed<SpawnGroup>> {
    CollectionLoader {
        walk: Walk {
            directories: &[],
            depth: ListDepth::Children,
            accept: Accept::Fn(is_json),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[GROUPS_DIRECTORY],
            app_data_place: AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse: parse_group_file,
        combine,
        needs: &[],
        then: None,
    }
}
