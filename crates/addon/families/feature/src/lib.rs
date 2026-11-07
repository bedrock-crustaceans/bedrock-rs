//! Features: `features/**/*.json` of a behaviour pack and the app's `definitions/features`
//! (one struct per feature type, gathered in [`Feature`]). A document is `{"format_version",
//! "minecraft:<type>": {…}}`; each payload key is a feature type with its own schema and chain.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codes;
pub mod history;
pub use blocks::{
    AttachConditions, ExposedFace, FossilFeature, GeodeFeature, GrowingPlantFeature,
    GrowthDirection, HeightWeight, HorizontalTreeDecorationFeature, MultiBlockFeature,
    MultifaceFeature, MultipartBlockColumnFeature, OreFeature, PartiallyExposedBlobFeature,
    PlacesBlock, ReplaceRule, SculkPatchFeature, SingleBlockFeature, VegetationPatchFeature,
    VegetationSurface, WeightedHeight, WeightedSpecifier,
};
pub use carver::{CaveCarverFeature, NetherCaveCarverFeature, UnderwaterCaveCarverFeature};
#[doc(hidden)]
pub use molang::{Reference, references};
pub use placement::{
    AggregateFeature, ConditionalFeature, ConditionalListFeature, EarlyOut, EarlyOutScheme,
    FeatureArea, HeightDifferenceFilterFeature, RectLayoutFeature, ScanSurfaceFeature, SearchAxis,
    SearchFeature, SearchVolume, SequenceFeature, SurfaceRelativeThresholdFeature, WeightedFeature,
    WeightedRandomFeature,
};
pub use scatter::ScatterFeature;
pub use structure::{
    BeardsAndShaversFeature, BlockIntersection, EmptyConstraint, FacingDirection,
    LeveledConstraint, SnapToSurfaceFeature, StructureConstraints, StructureTemplateFeature,
    Surface,
};
pub use tree::{
    AboveRoot, AcaciaBranches, AcaciaCanopy, AcaciaTrunk, AcaciaTrunkHeight, AltitudeFactor,
    BaseCluster, BranchCanopy, CanopyOffset, CanopySlope, CherryBranches, CherryCanopy,
    CherryTrunk, CherryTrunkHeight, Decoration, DecorationBlock, FallenTrunk, FancyBranches,
    FancyCanopy, FancyTrunk, FancyTrunkHeight, MangroveBranches, MangroveCanopy, MangroveRoots,
    MangroveTrunk, MangroveTrunkHeight, MegaBranches, MegaCanopy, MegaPineCanopy, MegaTrunk,
    MegaTrunkHeight, PineCanopy, PoplarCanopy, PoplarTrunk, RandomSpreadCanopy, RoofedCanopy,
    SimpleCanopy, SimpleTrunk, SpruceCanopy, StepDirection, Submerged, TreeFeature,
    TreeTypeWeights, TrunkLean,
};
pub use values::{BlockList, CarverRange, Chance, ChanceList, IntToken, JsonInt, WeightedBlock};

// First: the macros the feature types are declared with.
mod family;

mod blocks;
mod carver;
mod molang;
mod placement;
mod read;
mod scatter;
mod structure;
mod tree;
mod types;
mod validate;
mod values;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Phase, Severity};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected, UnknownMembers, default_path,
};
use bedrock_addon_core::id::{FeatureId, kind};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::write_envelope;
use family::{Fam, FeatureFn, FeatureType};
use read::{DuplicateText, FileSource, parse_file, parse_file_as_written, upgrade_document};

#[cfg(test)]
use family::{KEYS, TypeFn, with_type};
pub(crate) use family::{Rules, V1_13_0, strict_struct};

pub use family::Feature;
pub use types::Description;

// ---- Content, writing --------------------------------------------------------------------------

impl bedrock_addon_core::__private::Sealed for Feature {}

impl JsonContent for Feature {}

impl Content for Feature {
    const NAME: &'static str = "feature";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        parse_file(origin, bytes, cx, FileSource::Pack).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        let document = self.dispatch(WholeDocument)?;
        write_json(&document, out, style)
    }
}

/// The document of a type's newest struct as [`Feature`] writes it ([`Content::to_json`]).
pub(crate) fn newest_json<T: FeatureType>(value: &T) -> Result<String, EncodeError> {
    let document = WholeDocument.call(value)?;
    let mut out = Vec::new();
    write_json(&document, &mut out, WriteStyle::default())?;
    // The JSON writer emits only UTF-8.
    Ok(String::from_utf8(out).unwrap_or_default())
}

/// A feature's document: `{"format_version": <its type's newest struct>, "<key>": payload}`,
/// the envelope of its type's family (whose payload key is the type's).
struct WholeDocument;

impl FeatureFn<Result<Value, EncodeError>> for WholeDocument {
    fn call<T: FeatureType>(self, value: &T) -> Result<Value, EncodeError> {
        let (version, payload) = match value.older_form() {
            Some(older) => older?,
            None => (
                <T as bedrock_addon_core::versioning::Versioned>::VERSION,
                value.write()?,
            ),
        };
        Ok(write_envelope::<Fam<T>>(
            version,
            payload,
            &UnknownMembers::default(),
        ))
    }
}

impl Definition for Feature {
    type Kind = kind::Feature;
    type File = Feature;
    type Beta = NoBeta<kind::Feature>;
}

impl Placed for Feature {
    /// `features/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path("features", self.identifier())
    }
}

impl Feature {
    /// Parses one feature file without upgrading it: the struct its `format_version` selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<history::FeatureHistory>, Rejected> {
        parse_file_as_written(path, bytes, cx)
    }
}

// ---- the collection ----------------------------------------------------------------------------

/// `features/**/*.json`, packs top-down as the client walks them, then the app's
/// `definitions/features`.
const WALK: Walk = Walk::json_tree_in(&["features"])
    .top_down()
    .app_data(&["definitions/features"], AppDataPlace::After);

/// One file's contribution: a document claims its identifier even when its schema fails.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Feature> {
    let source = if file.pack.is_app_data() {
        FileSource::BuiltIn
    } else {
        FileSource::Pack
    };
    match parse_file(origin_of(file), &file.bytes, cx, source) {
        (Some(id), Ok(document)) => Contribution::Stable(id, document),
        (id, Ok(document)) => Contribution::Stable(
            id.unwrap_or_else(|| document.identifier().clone()),
            document,
        ),
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    }
}

/// The priority rank of a pack (0 is the top; the app-data layer is below every pack).
fn rank(pack: &PackRef) -> usize {
    pack.index().unwrap_or(usize::MAX)
}

/// The engine that adds the same-pack duplicate text.
const DUPLICATE_TEXT_ENGINE: FormatVersion = known::V1_26_40;

/// `DuplicateIdentifier` at `origin`: a second definition of `id` that is dropped; within one
/// pack on an engine from 1.26.40 an error with vanilla's text (the client logs "Feature with
/// identifier '%s' already exists" and does not add the second document), otherwise an
/// information without text (an older engine, or a definition shadowed by a lower pack, drops
/// the file silently, so the finding is the crate's own).
fn duplicate(origin: &Origin, id: &FeatureId, text: DuplicateText) -> Diagnostic {
    let text = text == DuplicateText::Vanilla;
    let severity = if text {
        Severity::Error
    } else {
        Severity::Info
    };
    let code = if text {
        codes::DUPLICATE.with([id.as_str()])
    } else {
        Code::from(&core_codes::DUPLICATE_IDENTIFIER)
    };
    Diagnostic::of(code)
        .severity(severity)
        .phase(Phase::Validate)
        .in_file(origin)
}

/// The registry: the first definition of an identifier in walk order (the highest-priority
/// pack's, the app's own after every pack) claims it, even when its schema fails; a later one
/// in the same pack is dropped, one in a lower pack is shadowed.
fn combine(parsed: Vec<Contribution<Feature>>, stack: &StackCx<'_>) -> Registry<Feature> {
    let mut builder = RegistryBuilder::new(
        CombineRule::ClaimBeforeValidate { claimed: None },
        CombineRule::top_wins(),
        stack.options,
    );
    let mut first: IndexMap<FeatureId, usize> = IndexMap::new();
    for contribution in parsed {
        let claim = match &contribution {
            Contribution::Stable(id, document) => Some((id, document.origin())),
            Contribution::Rejected(Some(id), rejected) => Some((id, &rejected.origin)),
            _ => None,
        };
        if let Some((id, origin)) = claim {
            let incoming = rank(&origin.pack);
            match first.get(id) {
                // The identifier is already in the map: the file is dropped.
                Some(&claimed) if claimed == incoming => {
                    let engine = stack
                        .packs
                        .iter()
                        .find(|pack| pack.pack.index() == origin.pack.index())
                        .map_or(bedrock_addon_core::version::ENGINE_VERSION, |pack| {
                            pack.context.engine_version
                        });
                    builder.push_diagnostic(duplicate(
                        origin,
                        id,
                        if engine >= DUPLICATE_TEXT_ENGINE {
                            DuplicateText::Vanilla
                        } else {
                            DuplicateText::Silent
                        },
                    ));
                    continue;
                }
                Some(_) if !origin.pack.is_app_data() => {
                    builder.push_diagnostic(duplicate(origin, id, DuplicateText::Silent));
                }
                Some(_) => {}
                None => {
                    first.insert(id.clone(), incoming);
                }
            }
        }
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.features()`: `features/**/*.json` of every behaviour pack and the app's
/// `definitions/features`, the first definition of an identifier winning.
///
/// By hand rather than `load::registry`: a `Feature` is read by the family of its payload key
/// (one of 23), so it is no `JsonDefinition`, and the loader stage names its identifier before
/// the schema runs.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Feature>, Contribution<Feature>> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};
