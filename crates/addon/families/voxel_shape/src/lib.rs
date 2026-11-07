//! Voxel shapes (`VoxelShape`): `shapes/**/*.json` of a behaviour pack (`minecraft:voxel_shape`),
//! the boxes a block's `minecraft:geometry.culling_shape` names. One struct covers every
//! `format_version` from 1.21.110 to the engine.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod read;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Phase, Severity};
use bedrock_addon_core::document::{
    CombineRule, Contribution, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Notice, Registry,
    default_path,
};
use bedrock_addon_core::id::{VoxelShapeId, kind};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::{self, CollectionLoader, StackCx, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, Envelope, EnvelopeCheck, Family, InputKind, Loaded, MissingVersion, NoComponents,
    Single, ValidateCx,
};

pub use types::{Description, Shape, ShapeBox, VoxelShape};

/// The first and only `format_version` vanilla has a voxel-shape parser for
/// (1.21.110); it reads every version from it up to the engine's.
pub(crate) const FLOOR: FormatVersion = bedrock_addon_core::version::known::V1_21_110;

/// The built-in shape without boxes (`minecraft:empty`): what a block culls with when its
/// geometry names no `culling_shape`. No pack defines it.
pub const EMPTY: VoxelShapeId = VoxelShapeId::from_static("minecraft:empty");

/// The built-in full block (`minecraft:unit_cube`): one box from 0 to 16 on every axis. No
/// pack defines it.
pub const UNIT_CUBE: VoxelShapeId = VoxelShapeId::from_static("minecraft:unit_cube");

/// The voxel-shape family of the version-chain machinery (public only as
/// `JsonDocument::Family`).
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct VoxelShapeFamily;

bedrock_addon_core::impl_no_molang!(VoxelShape);

impl Family for VoxelShapeFamily {
    const NAME: &'static str = "voxel shape";
    const PAYLOAD_KEY: &'static str = "minecraft:voxel_shape";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a shape through the document-input reader of the cereal-based families:
        // the version is read from the document, the size limits apply, and a member written
        // `null` fails its type.
        input: InputKind::Document,
        // The document input takes the raw text below 1.21.110 or without a version, and exactly
        // two members from 1.21.110 on.
        check: EnvelopeCheck::From(FLOOR),
        // Without a version the loader falls back to the caller's, which is none: "no valid version
        // passed by the caller or found in the input" and the file is skipped.
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = VoxelShape;
    type Beta = NoBeta<kind::VoxelShape>;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(shape) = v {
            validate::rules(shape, cx.load.is_base_game_pack, &cx.at, cx.diagnostics);
        }
    }

    /// After a file the schema loader rejected (a parse error; not a namespace or geometry
    /// failure, which log one line of their own), vanilla's "Failed to load voxel shape file" with
    /// the file's path in its pack.
    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, path: &PackPath, _: bool) {
        let rejected_by_schema = diagnostics
            .iter()
            .any(|finding| finding.severity >= Severity::Error && finding.phase == Phase::Parse);
        if rejected_by_schema {
            diagnostics.push(Diagnostic::of(codes::FAILED_TO_LOAD.with([path.as_str()])));
        }
    }
}

impl JsonDocument for VoxelShape {
    type Family = VoxelShapeFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// The shape's own version, never below 1.21.110, which the game would not read.
    fn write_version(&self) -> FormatVersion {
        self.source_format_version.max(FLOOR)
    }
}

impl JsonPlaced for VoxelShape {
    /// `shapes/<identifier>.json`, the identifier's `:` written `_`.
    fn place(&self) -> PackPath {
        default_path("shapes", &self.description.identifier)
    }
}

impl JsonDefinition for VoxelShape {
    type Kind = kind::VoxelShape;

    fn identifier(&self) -> &VoxelShapeId {
        &self.description.identifier
    }

    /// Per identifier the shape loaded last, each replaced one noted with `ShadowedDefinition`
    /// at info (the crate's own note; vanilla replaces silently). A file that failed binds
    /// nothing.
    fn combine_rule(_: &StackCx<'_>) -> CombineRule<Self> {
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        }
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `shapes/**/*.json` (the extension compared case-sensitively), packs bottom-up: vanilla walks
/// the stack from its last pack to its first and binds an identifier to the shape it loads
/// last, so the highest pack's shape is the effective one. Sub-folders are read in every pack
/// (the game reads them only in packs outside the app package).
const WALK: Walk = Walk::json_tree_in(&["shapes"]);

/// The loader of `bp.voxel_shapes()`: `shapes/**/*.json` of every behaviour pack, the shape
/// loaded last winning per identifier.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<VoxelShape>, Contribution<VoxelShape>> =
    load::registry::<VoxelShape>(WALK);

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn voxel_shape_reads_as_pinned() {
        assert_reading_table(
            "VoxelShapeFamily",
            &reading_table::<super::VoxelShapeFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 110 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.110..=1.21.110 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            ",
        );
    }
}
