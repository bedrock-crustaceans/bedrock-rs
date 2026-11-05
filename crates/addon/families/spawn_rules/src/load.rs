//! Loading spawn rules: the two families' parameters, the routing between them, `Content` /
//! `Definition`, the walk and the combine rule.

use super::history::{Any, SimplifiedAny, SimplifiedFamily, SpawnFamily};
use super::simplified::V1_21_60;
use super::texts;
use super::validate;
use super::{SimplifiedSpawnRules, SpawnRules, StandardSpawnRules};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{
    Content, Definition, Document, JsonContent, NoBeta, Placed, Registry, Rejected, default_path,
};
use bedrock_addon_core::id::{SpawnRulesId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::load::{CollectionLoader, Walk, WalkFile};
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, EnvelopeCheck, Family, Loaded, MissingVersion, NoComponents, RangeModel,
    ValidateCx, file_version,
};
use bedrock_addon_support::spawn::legacy::{RULES_KEY, V1_8_0};
use bedrock_addon_support::spawn::load::{
    FileLoad, Parsed, combine, contribution, load_family, load_file, origin,
};

/// The pack directory of spawn rules.
const RULES_DIRECTORY: &str = "spawn_rules";

impl Family for SpawnFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "spawn rules";
    const PAYLOAD_KEY: &'static str = RULES_KEY;
    const DIRECTORIES: &'static [&'static str] = &[RULES_DIRECTORY];
    const FLOOR: FormatVersion = V1_8_0;
    // Below the first slice the versioned loader has no parser at all.
    const BELOW_FLOOR: BelowFloor = BelowFloor::NoParser;
    // The legacy slice covers `[1.8.0, engine version]`, both ends included.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToEngine;
    type Latest = StandardSpawnRules;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The game reads the raw text and never checks the envelope.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // A missing version is parsed at 1.8.0 with a warning (the crate's own leniency; vanilla
    // errors).
    const MISSING_VERSION: MissingVersion = MissingVersion::Floor;

    fn selected(file: Option<FormatVersion>, cx: &LoadContext) -> bool {
        !SimplifiedFamily::selected(file, cx)
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rules) = v {
            validate::standard(rules, cx);
        }
    }
}

impl Family for SimplifiedFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "spawn rules";
    const PAYLOAD_KEY: &'static str = RULES_KEY;
    const DIRECTORIES: &'static [&'static str] = &[RULES_DIRECTORY];
    const FLOOR: FormatVersion = V1_21_60;
    // Below the first slice the versioned loader has no parser at all.
    const BELOW_FLOOR: BelowFloor = BelowFloor::NoParser;
    // The simplified slice covers `[1.21.60, engine version)`: the engine version itself has no
    // parser.
    const RANGE_MODEL: RangeModel = RangeModel::HalfOpenToEngine;
    type Latest = SimplifiedSpawnRules;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = SimplifiedAny;
    // Every routed document is checked as a modern envelope, from 1.8.0 on.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Always;

    /// The experiment is on and the file says 1.21.60 or newer.
    fn selected(file: Option<FormatVersion>, cx: &LoadContext) -> bool {
        cx.experiments.contains(Experiment::SimplifiedSpawnRules)
            && file.is_some_and(|v| v >= V1_21_60)
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(rules) = v {
            validate::simplified(rules, cx);
        }
    }
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
    let (identifier, result) = load_file(&origin, bytes, texts::FAILED_RULES, |text, origin| {
        let version = file_version(JsonSlice::new(text));
        if SimplifiedFamily::selected(version, cx) {
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
                let text = SimplifiedAny::V1_21_60(rules.clone()).to_json()?;
                // The chain writes the envelope; re-laid out in `style`.
                let value = bedrock_addon_core::json::from_str::<Value>(&text).map_err(|_| {
                    EncodeError::NotRepresentable {
                        at: JsonPath::new(),
                        what: bedrock_addon_core::json::Unrepresentable::Unsupported,
                    }
                })?;
                write_json(&value, out, style)
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

/// The loader of `bp.spawn_rules()`: `spawn_rules/**/*.json` of every behaviour pack, packs
/// bottom-up, the higher pack's rules for an entity replacing the lower's (a later document with
/// the same identifier replaces the earlier one).
pub const fn load_rules() -> CollectionLoader<Registry<SpawnRules>, Parsed<SpawnRules>> {
    CollectionLoader {
        walk: Walk::json_tree_in(&[RULES_DIRECTORY]),
        parse: parse_rules_file,
        combine,
        needs: &[],
        then: None,
    }
}
