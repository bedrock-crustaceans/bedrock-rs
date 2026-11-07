//! Spawn rules: `SpawnRules`, `StandardSpawnRules`, `SimplifiedSpawnRules`, `Condition`.
//!
//! Spawn rules (`spawn_rules/**/*.json` of a behaviour pack, `minecraft:spawn_rules`). A
//! spawn-rules file is read in one of two shapes, never upgraded from one to the other: the 1.8.0
//! shape ([`StandardSpawnRules`]) always, the 1.21.60 shape ([`SimplifiedSpawnRules`]) when the
//! Simplified Spawn Rules experiment is on and the file is 1.21.60 or newer.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
mod legacy;
mod simplified;
mod validate;

pub use bedrock_addon_shared_spawn::{
    AboveBlockFilter, Condition, DelayFilter, DensityLimit, Difficulty, DifficultyFilter, Herd,
    HerdInfo, MobEventFilter, MobSpawnRules, OneOrMany, Permutation, PopulationControl,
    RangeFilter, VillageFilter, Weight, WorldAgeFilter,
};
pub use simplified::{
    BiomeConditions, PermuteType, Placement, SimplifiedDescription, SimplifiedHerd,
    SimplifiedSpawnRules,
};

use crate::simplified::V1_21_60;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Placed, Registry, Rejected, default_path,
};
use bedrock_addon_core::id::SpawnRulesId;
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, JsonSlice, WriteStyle, write_json};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{CollectionLoader, Walk, WalkFile};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, BeyondWidth, Envelope, EnvelopeCheck, Family, InputKind, Loaded, NoComponents,
    RangeModel, Reading, Scalars, Single, StructIndex, ValidateCx, VersionStruct, Versioned,
    file_version, write_envelope,
};
use bedrock_addon_shared_spawn::legacy::{RULES_KEY, V1_8_0};
use bedrock_addon_shared_spawn::load::{
    FileLoad, Parsed, combine, contribution, load_family, load_file, origin,
};

/// How one entity spawns (`minecraft:spawn_rules`): the rules in the shape the file was read in.
///
/// A file is read as [`SpawnRules::Simplified`] when the Simplified Spawn Rules experiment is on
/// and its `format_version` is 1.21.60 or newer, and as [`SpawnRules::Standard`] otherwise; one
/// shape is never converted into the other.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:spawn_rules")]
#[doc(alias = "spawn_rules")]
pub enum SpawnRules {
    /// The 1.8.0 shape.
    Standard(StandardSpawnRules),
    /// The 1.21.60 shape (Simplified Spawn Rules experiment).
    Simplified(SimplifiedSpawnRules),
}

impl SpawnRules {
    /// The entity the rules are for (`description.identifier`).
    pub fn identifier(&self) -> &SpawnRulesId {
        match self {
            Self::Standard(rules) => &rules.description.identifier,
            Self::Simplified(rules) => &rules.description.identifier,
        }
    }
}

/// Spawn rules in the 1.8.0 shape: a spawn pool and a list of conditions, each a weight and the
/// filters and herds of one way the entity spawns.
///
/// JSON: `{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier",
/// "population_control"}, "conditions": [{"minecraft:weight": …, …}]}}`. Vanilla reads the whole
/// file with a hand-written parser: the envelope is not checked, a member of the wrong JSON type
/// takes the value vanilla's accessors give it, and `format_version` only selects the Molang
/// version of block descriptor `tags`. Spawn rules whose pool is missing or unknown load but are
/// not registered.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardSpawnRules {
    /// The document's `format_version` (JSON `format_version`; `None` when absent, which reads
    /// as 1.8.0 with a warning): selects the Molang version of the block descriptors' `tags`.
    pub format_version: Option<FormatVersion>,
    /// The entity and its pool (JSON `description`).
    pub description: StandardDescription,
    /// The conditions (JSON `conditions`).
    pub conditions: Option<Vec<Condition>>,
    /// Unknown members of `minecraft:spawn_rules`.
    pub extra: UnknownMembers,
    /// Root members other than `format_version` and `minecraft:spawn_rules` (vanilla ignores
    /// them; each is an `UnrecognizedField` warning).
    pub root_extra: UnknownMembers,
}

/// `minecraft:spawn_rules/description` of standard spawn rules.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardDescription {
    /// The entity (JSON `identifier`, required).
    pub identifier: SpawnRulesId,
    /// The spawn pool's name (JSON `population_control`); see
    /// [`StandardDescription::population_control`].
    pub population_control: Option<String>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

impl StandardSpawnRules {
    /// The version the document is read at: its `format_version`, 1.8.0 when absent.
    pub fn effective_version(&self) -> FormatVersion {
        self.format_version
            .unwrap_or(bedrock_addon_core::version::known::V1_8_0)
    }
}

impl StandardDescription {
    /// The spawn pool: `population_control` after lower-casing, `None` when it is missing,
    /// empty or unknown (then vanilla does not register the rules).
    pub fn population_control(&self) -> Option<PopulationControl> {
        self.population_control
            .as_deref()
            .and_then(PopulationControl::from_name)
    }
}

/// The pack directory of spawn rules.
const RULES_DIRECTORY: &str = "spawn_rules";

/// The `spawn` family: standard spawn rules, one struct (`v1_8_0`) with no `Upgrade` step.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct SpawnFamily;

/// The `spawn_simplified` family: simplified spawn rules, one struct (`v1_21_60`) with no
/// `Upgrade` step.
#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct SimplifiedFamily;

/// A standard spawn-rules document as written: the one struct, the newest.
#[doc(hidden)]
pub type SpawnHistory = Single<SpawnFamily>;

/// A simplified spawn-rules document as written: the one struct, the newest.
#[doc(hidden)]
pub type SimplifiedHistory = Single<SimplifiedFamily>;

impl Family for SpawnFamily {
    const NAME: &'static str = "spawn rules";
    const PAYLOAD_KEY: &'static str = RULES_KEY;
    const FLOOR: FormatVersion = V1_8_0;
    const ENVELOPE: Envelope = Envelope {
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // The game reads the raw text and never checks the envelope.
        check: EnvelopeCheck::Never,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        // The default parses a missing version at 1.8.0 with an information (the crate's own
        // leniency; vanilla errors).
        ..Envelope::DEFAULT
    };
    // The legacy slice covers `[1.8.0, engine version]`, both ends included: the default
    // range model, `ClosedToEngine`.
    type Latest = StandardSpawnRules;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type History = Single<Self>;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rules) = v {
            validate::standard(rules, cx);
        }
    }
}

impl Family for SimplifiedFamily {
    const NAME: &'static str = "spawn rules";
    const PAYLOAD_KEY: &'static str = RULES_KEY;
    const FLOOR: FormatVersion = V1_21_60;
    const ENVELOPE: Envelope = Envelope {
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        // The default checks every routed document as a modern envelope, from 1.8.0 on.
        ..Envelope::DEFAULT
    };
    // The simplified slice covers `[1.21.60, engine version)`: the engine version itself has no
    // parser.
    const RANGE_MODEL: RangeModel = RangeModel::HalfOpenToEngine;
    type Latest = SimplifiedSpawnRules;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type History = Single<Self>;

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        SIMPLIFIED_READING
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rules) = v {
            validate::simplified(rules, cx);
        }
    }
}

/// The reading of the simplified slice, which its typed parse takes: the profile of a document
/// without an input kind, with vanilla's strict cereal scalars (a member takes only a token of
/// its own type, a number beyond its float's width is a type error). Every struct of the rules
/// keeps its unknown members for write-back.
pub(crate) const SIMPLIFIED_READING: Reading = InputKind::FirstRun
    .profile()
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error)
    .with_keeps_unknown(true);

/// Whether the simplified family takes a file of version `file`: the experiment is on and the
/// file says 1.21.60 or newer. The standard family takes every other file.
fn simplified_selected(file: Option<FormatVersion>, cx: &LoadContext) -> bool {
    cx.experiments.contains(Experiment::SimplifiedSpawnRules) && file.is_some_and(|v| v >= V1_21_60)
}

/// Loads one spawn-rules file, routed by the experiment and its version.
pub(crate) fn parse_rules(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> FileLoad<SpawnRules> {
    let origin = origin(pack, side, path);
    let (identifier, result) = load_file(&origin, bytes, &codes::FAILED_RULES, |text, origin| {
        let version = file_version(JsonSlice::new(text));
        if simplified_selected(version, cx) {
            load_family::<SimplifiedFamily, _>(text, cx, origin, SpawnRules::Simplified)
        } else {
            load_family::<SpawnFamily, _>(text, cx, origin, SpawnRules::Standard)
        }
    });
    // A standard document without a known pool loads but is not registered (rule 8).
    let registered = result
        .as_ref()
        .is_ok_and(|document| match document.value() {
            SpawnRules::Standard(rules) => rules.description.population_control().is_some(),
            SpawnRules::Simplified(_) => true,
        });
    FileLoad {
        identifier,
        result,
        registered,
    }
}

// The quartet is hand-written: `SpawnRules` is either family's newest struct, so it is the
// `Latest` of neither and cannot be a `JsonDocument`.
impl bedrock_addon_core::__private::Sealed for SpawnRules {}

impl Content for SpawnRules {
    const NAME: &'static str = "spawn rules";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Loads one spawn-rules file. A standard document whose pool is missing or unknown loads
    /// (with its warning) although a stack does not register it.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_rules(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).result
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        match self {
            Self::Standard(rules) => write_json(&rules.to_value(), out, style),
            Self::Simplified(rules) => {
                let document = write_envelope::<SimplifiedFamily>(
                    <SimplifiedSpawnRules as Versioned>::VERSION,
                    VersionStruct::write(rules)?,
                    &UnknownMembers::default(),
                );
                write_json(&document, out, style)
            }
        }
    }
}

impl JsonContent for SpawnRules {}

impl Definition for SpawnRules {
    type Kind = kind::SpawnRules;
    type File = SpawnRules;
    type Beta = NoBeta<kind::SpawnRules>;
}

impl Placed for SpawnRules {
    /// `spawn_rules/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(RULES_DIRECTORY, self.identifier())
    }
}

fn parse_rules_file(file: &WalkFile, cx: &LoadContext) -> Parsed<SpawnRules> {
    contribution(
        parse_rules(file.pack.clone(), file.side, &file.path, &file.bytes, cx),
        |id| SpawnRulesId::parse(id),
        |document| document.identifier().clone(),
    )
}

/// The loader of `bp.spawn_rules()`, the family's own over `bedrock_addon_shared_spawn`'s kit (a standard
/// document without a known pool loads but is not registered, `Parsed::Unregistered`): `spawn_rules/**/*.json` of every behaviour pack, packs
/// bottom-up, the higher pack's rules for an entity replacing the lower's (a later document with
/// the same identifier replaces the earlier one).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<SpawnRules>, Parsed<SpawnRules>> = CollectionLoader {
    walk: Walk::json_tree_in(&[RULES_DIRECTORY]),
    parse: parse_rules_file,
    combine,
    needs: &[],
    then: None,
};

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn spawn_reads_as_pinned() {
        assert_reading_table(
            "SpawnFamily",
            &reading_table::<super::SpawnFamily>(),
            r"
                text: Root, clamps beyond a double: true, duplicates kept in: []
                envelope: Never
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.8.0..=1.8.0 (0..=0): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }

    #[test]
    fn simplified_reads_as_pinned() {
        assert_reading_table(
            "SimplifiedFamily",
            &reading_table::<super::SimplifiedFamily>(),
            r"
                text: Root, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: at the floor, told at Info: None
                below floor: NoParser
                empty document: Error
                1.21.60..=1.21.60 (0..=0): json Own/Strict/Error/Warn/keeps/lenient/Refuse, mcb Own/Strict/Error/Warn/keeps/lenient/Refuse
            ",
        );
    }
}
