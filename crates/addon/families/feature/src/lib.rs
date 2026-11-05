//! Features: `features/**/*.json` of a behaviour pack and the app's `definitions/features`
//! (one struct per feature type, gathered in [`Feature`]). A document is `{"format_version",
//! "minecraft:<type>": {…}}`; each payload key is a feature type with its own schema and chain.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use blocks::{
    AttachConditions, ExposedFace, FossilFeature, GeodeFeature, GrowingPlantFeature,
    GrowthDirection, HeightWeight, HorizontalTreeDecorationFeature, MultiBlockFeature,
    MultifaceFeature, MultipartBlockColumnFeature, OreFeature, PartiallyExposedBlobFeature,
    PlacesBlock, ReplaceRule, SculkPatchFeature, SingleBlockFeature, VegetationPatchFeature,
    VegetationSurface, WeightedHeight, WeightedSpecifier,
};
pub use carver::{CaveCarverFeature, NetherCaveCarverFeature, UnderwaterCaveCarverFeature};
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
#[doc(hidden)]
pub use walk::{Reference, references};

mod blocks;
mod carver;
mod placement;
mod read;
mod scatter;
mod structure;
mod tree;
mod types;
mod validate;
mod values;
mod walk;

#[cfg(test)]
mod tests;

use std::convert::Infallible;
use std::marker::PhantomData;

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, Extra, JsonContent, NoBeta, Origin,
    Placed, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{FeatureId, kind};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, PrepareError, Prepared, Value, WriteStyle, write_json,
};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, LoadContext, StackCx, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    Envelope, Family, InputKind, Loaded, MissingVersion, NoComponents, Parsed, ValidateCx,
    VersionChain, VersionStruct, file_version, finish, load, parse_as_written, write_envelope,
};
use read::formatted;

pub use types::{Description, Feature};

/// Format 1.13.0, the initial release (`[1.13.0, 1.21.10)`) and the floor of every feature
/// type but `multi_block_feature` and `multipart_block_column_feature`, which are registered
/// at 1.26.40 (`[1.26.40, 1.26.50)`). The format versions at which a feature type's schema
/// changes are 1.13.0, 1.21.10 (scatter), 1.21.40 (single block), 1.26.40 (multi block) and
/// 1.26.50 (snap to surface), the last one running to the current version.
pub(crate) const V1_13_0: FormatVersion = FormatVersion::new(1, 13, 0);

/// `impl Versioned + VersionStruct` for a struct that parses strictly and writes plainly.
macro_rules! strict_struct {
    ($ty:ty, $version:expr) => {
        impl ::bedrock_addon_core::versioning::Versioned for $ty {
            const VERSION: ::bedrock_addon_core::version::FormatVersion = $version;
        }

        impl ::bedrock_addon_core::versioning::VersionStruct for $ty {
            fn parse(
                payload: ::bedrock_addon_core::json::JsonSlice<'_>,
                cx: &::bedrock_addon_core::versioning::ParseCx<'_>,
                d: &mut ::bedrock_addon_core::diagnostic::Diagnostics,
            ) -> Option<Self> {
                $crate::read::parse_strict(payload, cx, d)
            }

            fn write(
                &self,
            ) -> Result<::bedrock_addon_core::json::Value, ::bedrock_addon_core::json::EncodeError>
            {
                ::bedrock_addon_core::versioning::write_payload(self)
            }
        }
    };
}

pub(crate) use strict_struct;

/// What a feature type's newest struct says about itself, and its rules on a loaded value.
pub(crate) trait Rules {
    /// Only base-game packs may use the type.
    const INTERNAL: bool = false;
    /// The engine release that registers the type; below it the payload key is unknown.
    const SINCE_ENGINE: Option<FormatVersion> = None;
    /// The type's lowest format version.
    const FLOOR: FormatVersion = V1_13_0;
    /// The type's schema rules on the loaded value, at the payload's path.
    #[allow(unused_variables)]
    fn rules(&mut self, check: &mut ValidateCx<'_>) {}
    /// The value in an older struct's layout and that struct's version, when the newest
    /// struct's schema would reject it (an upgraded document the newest layout cannot hold, for
    /// example a single block feature whose `places_block` is a `{tags}` descriptor, which has
    /// no 1.21.40 form); `None` writes the newest struct. The older layout reads back to the
    /// same value.
    fn older_form(&self) -> Option<Result<(FormatVersion, Value), EncodeError>> {
        None
    }
}

/// A feature type: its payload key, its chain and its place in [`Feature`].
pub(crate) trait FeatureType: Rules + VersionStruct + Send + Sync + 'static {
    /// The payload key, `minecraft:ore_feature`.
    const KEY: &'static str;
    /// The type's chain of version structs.
    type Chain: VersionChain<Fam<Self>>;
    /// The feature of this type.
    fn into_feature(self) -> Feature;
    /// A chain value as the public history enum.
    fn to_history(chain: Self::Chain) -> history::Any;
    /// The chain value a history value of this type holds; the value back otherwise.
    #[allow(
        clippy::result_large_err,
        reason = "the value is handed back whole, once per document"
    )]
    fn from_history(any: history::Any) -> Result<Self::Chain, history::Any>;
}

/// The family of one feature type (hidden: only the loaders name it).
pub(crate) struct Fam<T>(PhantomData<fn() -> T>);

impl<T: FeatureType> Family for Fam<T> {
    const NAME: &'static str = "feature";
    const PAYLOAD_KEY: &'static str = T::KEY;
    const FLOOR: FormatVersion = T::FLOOR;
    type Latest = T;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = T::Chain;
    const ENVELOPE: Envelope = Envelope {
        // The client reads a feature through a jsoncpp input: the version comes from the root
        // member, and rule N1 holds (a member written as `null` is absent);
        // `read::parse_strict_with` reads under the selected version's rule.
        input: InputKind::JsonCpp,
        // A block descriptor keeps the value of a state as written (a float is not a state's
        // type, and is told): a number no double holds is the largest finite one there, with a
        // warning at it, or the feature would load and not be written.
        clamps_beyond_double: true,
        // The client passes no version and finds none in a file without `format_version`: the
        // file is skipped.
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(value) = v {
            value.rules(cx);
        }
    }
}

/// The value of a `Feature` variant, held inline or (`Feature::Tree`) boxed.
trait Held<T> {
    /// The value, moved out.
    fn into_inner(self) -> T;
    /// The value.
    fn inner(&self) -> &T;
}

impl<T> Held<T> for T {
    fn into_inner(self) -> T {
        self
    }

    fn inner(&self) -> &T {
        self
    }
}

impl<T> Held<T> for Box<T> {
    fn into_inner(self) -> T {
        *self
    }

    fn inner(&self) -> &T {
        self
    }
}

/// A computation generic over the feature type, picked by payload key.
pub(crate) trait TypeFn<R> {
    /// Runs for the type `T`.
    fn call<T: FeatureType>(self) -> R;
}

/// A computation over a feature's value, generic over its type.
pub(crate) trait FeatureFn<R> {
    /// Runs for the value `value` of type `T`.
    fn call<T: FeatureType>(self, value: &T) -> R;
}

/// Declares the chain of each feature type, its `FeatureType` impl, the dispatch by payload key
/// and `Feature`'s per-type accessors.
macro_rules! feature_types {
    ($( $variant:ident($ty:ty) = $key:literal, $chain:ident [ $($v:ident($t:ty)),+ ]; )*) => {
        $(
            bedrock_addon_core::version_chain! {
                family = Fam<$ty>;
                /// The type's document parsed by the struct its `format_version` selects.
                #[derive(Clone, Debug, PartialEq)]
                pub(crate) enum $chain {
                    $(
                        #[doc = stringify!($v)]
                        $v($t),
                    )+
                }
            }

            impl FeatureType for $ty {
                const KEY: &'static str = $key;
                type Chain = $chain;

                fn into_feature(self) -> Feature {
                    Feature::$variant(self.into())
                }

                fn to_history(chain: $chain) -> history::Any {
                    match chain {
                        $( $chain::$v(value) => history::Any::from(value), )+
                    }
                }

                fn from_history(any: history::Any) -> Result<$chain, history::Any> {
                    $(
                        let any = match <$t as TryFrom<history::Any>>::try_from(any) {
                            Ok(value) => return Ok($chain::$v(value)),
                            Err(any) => any,
                        };
                    )+
                    Err(any)
                }
            }

            impl From<$ty> for history::Any {
                fn from(value: $ty) -> Self {
                    history::Any::Latest(Feature::$variant(value.into()))
                }
            }

            impl TryFrom<history::Any> for $ty {
                type Error = history::Any;

                fn try_from(any: history::Any) -> Result<Self, history::Any> {
                    match any {
                        history::Any::Latest(Feature::$variant(value)) => Ok(Held::into_inner(value)),
                        other => Err(other),
                    }
                }
            }
        )*

        /// Runs `f` for the feature type of the payload key `key`; `None` for a key vanilla
        /// does not register.
        ///
        /// The match picks the function and the one call after it runs it: a call in each arm
        /// kept one `R` per type in this frame in an unoptimised build, and an `R` that holds a
        /// `Feature` was some 17 KB before the tree was boxed (504 016 bytes for 29 types).
        pub(crate) fn with_type<R, F: TypeFn<R>>(key: &str, f: F) -> Option<R> {
            let call: fn(F) -> R = match key {
                $( $key => <F as TypeFn<R>>::call::<$ty>, )*
                _ => return None,
            };
            Some(call(f))
        }

        /// Every payload key, in `Feature`'s order.
        #[cfg(test)]
        pub(crate) const KEYS: &[&str] = &[$($key),*];

        impl Feature {
            /// The payload key of the feature's type (`minecraft:ore_feature`).
            pub fn type_key(&self) -> &'static str {
                match self {
                    $( Feature::$variant(_) => $key, )*
                }
            }

            /// The feature's identifier (`description.identifier`).
            pub fn identifier(&self) -> &FeatureId {
                match self {
                    $( Feature::$variant(value) => &value.description.identifier, )*
                }
            }

            /// Runs `f` on the feature's value.
            pub(crate) fn dispatch<R>(&self, f: impl FeatureFn<R>) -> R {
                match self {
                    $( Feature::$variant(value) => f.call::<$ty>(value), )*
                }
            }
        }

        // The references of whichever type the feature is.
        impl walk::Refer for Feature {
            fn refer<'a>(&'a self, out: &mut walk::References<'a>) {
                match self {
                    $( Feature::$variant(value) => {
                        walk::Refer::refer(Held::<$ty>::inner(value), out)
                    } )*
                }
            }
        }
    };
}

/// Whether the type is one of the internal ones: a rule outside the game's packs cannot place
/// such a feature.
struct Internal;

impl FeatureFn<bool> for Internal {
    fn call<T: FeatureType>(self, _: &T) -> bool {
        T::INTERNAL
    }
}

impl Feature {
    /// Whether the feature is of an internal type: one only the game's own packs may define
    /// and only their rules may place.
    #[doc(hidden)]
    pub fn is_internal(&self) -> bool {
        self.dispatch(Internal)
    }
}

feature_types! {
    Aggregate(placement::AggregateFeature) = "minecraft:aggregate_feature",
        AggregateChain [V1_13_0(placement::AggregateFeature)];
    BeardsAndShavers(structure::BeardsAndShaversFeature) = "minecraft:beards_and_shavers",
        BeardsAndShaversChain [V1_13_0(structure::BeardsAndShaversFeature)];
    CaveCarver(carver::CaveCarverFeature) = "minecraft:cave_carver_feature",
        CaveCarverChain [V1_13_0(carver::CaveCarverFeature)];
    ConditionalList(placement::ConditionalListFeature) = "minecraft:conditional_list",
        ConditionalListChain [V1_13_0(placement::ConditionalListFeature)];
    Fossil(blocks::FossilFeature) = "minecraft:fossil_feature",
        FossilChain [V1_13_0(blocks::FossilFeature)];
    Geode(blocks::GeodeFeature) = "minecraft:geode_feature",
        GeodeChain [V1_13_0(blocks::GeodeFeature)];
    GrowingPlant(blocks::GrowingPlantFeature) = "minecraft:growing_plant_feature",
        GrowingPlantChain [V1_13_0(blocks::GrowingPlantFeature)];
    HeightDifferenceFilter(placement::HeightDifferenceFilterFeature)
        = "minecraft:height_difference_filter_feature",
        HeightDifferenceFilterChain [V1_13_0(placement::HeightDifferenceFilterFeature)];
    HorizontalTreeDecoration(blocks::HorizontalTreeDecorationFeature)
        = "minecraft:horizontal_tree_decoration_feature",
        HorizontalTreeDecorationChain [V1_13_0(blocks::HorizontalTreeDecorationFeature)];
    MultiBlock(blocks::MultiBlockFeature) = "minecraft:multi_block_feature",
        MultiBlockChain [V1_26_40(blocks::MultiBlockFeature)];
    Multiface(blocks::MultifaceFeature) = "minecraft:multiface_feature",
        MultifaceChain [V1_13_0(blocks::MultifaceFeature)];
    MultipartBlockColumn(blocks::MultipartBlockColumnFeature)
        = "minecraft:multipart_block_column_feature",
        MultipartBlockColumnChain [V1_26_40(blocks::MultipartBlockColumnFeature)];
    NetherCaveCarver(carver::NetherCaveCarverFeature) = "minecraft:nether_cave_carver_feature",
        NetherCaveCarverChain [V1_13_0(carver::NetherCaveCarverFeature)];
    Ore(blocks::OreFeature) = "minecraft:ore_feature",
        OreChain [V1_13_0(blocks::OreFeature)];
    PartiallyExposedBlob(blocks::PartiallyExposedBlobFeature)
        = "minecraft:partially_exposed_blob_feature",
        PartiallyExposedBlobChain [V1_13_0(blocks::PartiallyExposedBlobFeature)];
    RectLayout(placement::RectLayoutFeature) = "minecraft:rect_layout",
        RectLayoutChain [V1_13_0(placement::RectLayoutFeature)];
    ScanSurface(placement::ScanSurfaceFeature) = "minecraft:scan_surface",
        ScanSurfaceChain [V1_13_0(placement::ScanSurfaceFeature)];
    Scatter(scatter::ScatterFeature) = "minecraft:scatter_feature",
        ScatterChain [
            V1_13_0(history::v1_13_0::ScatterFeature),
            V1_21_10(scatter::ScatterFeature)
        ];
    SculkPatch(blocks::SculkPatchFeature) = "minecraft:sculk_patch_feature",
        SculkPatchChain [V1_13_0(blocks::SculkPatchFeature)];
    Search(placement::SearchFeature) = "minecraft:search_feature",
        SearchChain [V1_13_0(placement::SearchFeature)];
    Sequence(placement::SequenceFeature) = "minecraft:sequence_feature",
        SequenceChain [V1_13_0(placement::SequenceFeature)];
    SingleBlock(blocks::SingleBlockFeature) = "minecraft:single_block_feature",
        SingleBlockChain [
            V1_13_0(history::v1_13_0::SingleBlockFeature),
            V1_21_40(blocks::SingleBlockFeature)
        ];
    SnapToSurface(structure::SnapToSurfaceFeature) = "minecraft:snap_to_surface_feature",
        SnapToSurfaceChain [
            V1_13_0(history::v1_13_0::SnapToSurfaceFeature),
            V1_26_50(structure::SnapToSurfaceFeature)
        ];
    StructureTemplate(structure::StructureTemplateFeature)
        = "minecraft:structure_template_feature",
        StructureTemplateChain [V1_13_0(structure::StructureTemplateFeature)];
    SurfaceRelativeThreshold(placement::SurfaceRelativeThresholdFeature)
        = "minecraft:surface_relative_threshold_feature",
        SurfaceRelativeThresholdChain [V1_13_0(placement::SurfaceRelativeThresholdFeature)];
    Tree(tree::TreeFeature) = "minecraft:tree_feature",
        TreeChain [V1_13_0(tree::TreeFeature)];
    UnderwaterCaveCarver(carver::UnderwaterCaveCarverFeature)
        = "minecraft:underwater_cave_carver_feature",
        UnderwaterCaveCarverChain [V1_13_0(carver::UnderwaterCaveCarverFeature)];
    VegetationPatch(blocks::VegetationPatchFeature) = "minecraft:vegetation_patch_feature",
        VegetationPatchChain [V1_13_0(blocks::VegetationPatchFeature)];
    WeightedRandom(placement::WeightedRandomFeature) = "minecraft:weighted_random_feature",
        WeightedRandomChain [V1_13_0(placement::WeightedRandomFeature)];
}

// ---- the loader stage -------------------------------------------------------------------------

/// The client's text for a file that is no valid JSON.
const INVALID_JSON: &str = "[Feature Registry] Passed in contents could not be evaluated as valid JSON in the following file: {}";
/// The client's text for a file without a description identifier.
const MISSING_IDENTIFIER: &str =
    "[Feature Registry] Missing description identifier for JSON in the following file: {}";
/// The client's text for an empty description identifier.
const EMPTY_IDENTIFIER: &str =
    "[Feature Registry] Empty description identifier for JSON in the following file: {}";
/// The client's text for an identifier that is not `namespace:id`.
const NOT_NAMESPACED: &str = "The feature identifier '{}' should be in the format namespace:id";
/// The built-in definitions' namespace rule.
const NOT_MINECRAFT: &str = "The feature identifer '{}' should be in the 'minecraft' namespace";
/// The client's text for an internal feature type used outside the game's packs.
const INTERNAL_TYPE: &str = "The feature uses the feature type '{}', which is considered internal and thus not compatible with this engine version of the game.";
/// The client's text for a second feature of one identifier in a pack (engine 1.26.40 and
/// later).
const DUPLICATE: &str = "Feature with identifier '{}' already exists";

/// The `description.identifier` string of the first root member that has one.
fn identifier_in(root: &Value) -> Option<String> {
    root.as_object()?.iter().find_map(|(_, member)| {
        let identifier = member
            .as_object()?
            .get("description")?
            .as_object()?
            .get("identifier")?
            .as_string()?;
        Some(identifier.as_str().to_owned())
    })
}

/// The payload key: the first root member other than `format_version`.
fn payload_key(root: &Value) -> String {
    root.as_object()
        .and_then(|object| {
            object
                .keys()
                .find(|key| key.as_str() != "format_version")
                .map(|key| key.as_str().to_owned())
        })
        .unwrap_or_default()
}

/// What the loader stage gives a type's parse: the text, its identifier and context.
struct Stage<'a> {
    text: &'a str,
    key: &'a str,
    cx: &'a LoadContext,
}

/// The checks the client makes before the schema (the internal-type gate and the engine gate
/// of the type) and the version floor, for type `T`; the findings when the document stops
/// here.
fn type_checks<T: FeatureType>(stage: &Stage<'_>) -> Result<(), Diagnostics> {
    let mut d = Diagnostics::default();
    let mut at = JsonPath::new();
    if T::SINCE_ENGINE.is_some_and(|since| stage.cx.engine_version < since) {
        // The engine has not registered the type.
        d.push(
            Diagnostic::error(DiagnosticCode::UnknownPayloadKey {
                name: stage.key.to_owned(),
            })
            .at(at),
        );
        return Err(d);
    }
    if T::INTERNAL && !stage.cx.is_base_game_pack {
        // An internal type is for the game's own packs.
        at.push(T::KEY);
        d.push(
            Diagnostic::error(DiagnosticCode::MissingToggles {
                text: Some(formatted(INTERNAL_TYPE, &[&stage.key])),
            })
            .at(at),
        );
        return Err(d);
    }
    let version = file_version(JsonSlice::new(stage.text));
    // A type registered at a later format version than the initial release (multi_block and
    // multipart_block_column) is in no schema below its floor: the client says "Could not find
    // registered feature type that matches name '%s'".
    if T::FLOOR > V1_13_0
        && let Some(version) = version
        && version < T::FLOOR
    {
        d.push(
            Diagnostic::error(DiagnosticCode::UnknownPayloadKey {
                name: stage.key.to_owned(),
            })
            .at(at),
        );
        return Err(d);
    }
    // Below the floor no parser's range matches: the client's "no parser available".
    if let Some(version) = version
        && version < T::FLOOR
    {
        at.push("format_version");
        d.push(
            Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion {
                    version: version.to_string(),
                }),
            })
            .at(at),
        );
        return Err(d);
    }
    Ok(())
}

/// The full load of a document of one type: the outcome as a `Feature`.
struct LoadType<'a>(Stage<'a>);

impl
    TypeFn<(
        Option<Feature>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    )> for LoadType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> (
        Option<Feature>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    ) {
        let stage = self.0;
        if let Err(d) = type_checks::<T>(&stage) {
            return (None, no_version(), d);
        }
        let outcome = load::<Fam<T>>(JsonSlice::new(stage.text), stage.cx);
        let value = match outcome.value {
            Some(Loaded::Stable(value)) => Some(value.into_feature()),
            _ => None,
        };
        (value, outcome.version, outcome.diagnostics)
    }
}

/// The parse of a document of one type without the upgrade.
struct ParseType<'a>(Stage<'a>);

impl
    TypeFn<(
        Option<history::Any>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    )> for ParseType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> (
        Option<history::Any>,
        bedrock_addon_core::document::VersionInfo,
        Diagnostics,
    ) {
        let stage = self.0;
        if let Err(d) = type_checks::<T>(&stage) {
            return (None, no_version(), d);
        }
        let parsed = parse_as_written::<Fam<T>>(JsonSlice::new(stage.text), stage.cx);
        (
            parsed.value.map(T::to_history),
            parsed.version,
            parsed.diagnostics,
        )
    }
}

/// The version information of a document that stopped before its version was read.
fn no_version() -> bedrock_addon_core::document::VersionInfo {
    bedrock_addon_core::document::VersionInfo::new(None, false, V1_13_0, V1_13_0, false)
}

/// Whose definition a feature file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileSource {
    /// A pack's file.
    Pack,
    /// One of the app's own definitions: its identifier must be in the `minecraft` namespace.
    BuiltIn,
}

/// Whether a duplicate identifier is reported with vanilla's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuplicateText {
    /// An information without text.
    Silent,
    /// An error with vanilla's text.
    Vanilla,
}

/// The loader stage up to the identifier (valid JSON, an identifier that is present, not
/// empty and namespaced, and the built-in namespace rule): the text, its root and its
/// identifier; or the findings of a file that stops before the identifier map, so it claims no
/// identifier.
fn identify<'b>(
    prepared: &'b Result<Prepared<'b>, PrepareError>,
    path: &PackPath,
    source: FileSource,
) -> Result<(&'b str, Value, FeatureId), Diagnostics> {
    let file = path.to_string();
    let invalid = |message: String| {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message,
            text: Some(formatted(INVALID_JSON, &[&file])),
        }));
        d
    };
    let text = prepared
        .as_ref()
        .map(Prepared::as_str)
        .map_err(|_| invalid("not UTF-8 JSON within the nesting limit".to_owned()))?;
    // The identifier is read from the copy of a doubled member that stands, as the parse
    // reads it.
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )
    .map_err(|e| invalid(e.to_string()))?;
    let one = |code: DiagnosticCode| {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::error(code));
        d
    };
    let Some(identifier) = identifier_in(&root) else {
        return Err(one(DiagnosticCode::MissingRequiredField {
            text: Some(formatted(MISSING_IDENTIFIER, &[&file])),
        }));
    };
    if identifier.is_empty() {
        return Err(one(DiagnosticCode::InvalidValue {
            text: Some(formatted(EMPTY_IDENTIFIER, &[&file])),
        }));
    }
    let Some((namespace, _)) = identifier.split_once(':') else {
        return Err(one(DiagnosticCode::NamespaceRule {
            text: Some(formatted(NOT_NAMESPACED, &[&identifier])),
        }));
    };
    if source == FileSource::BuiltIn && namespace != "minecraft" {
        return Err(one(DiagnosticCode::NamespaceRule {
            text: Some(formatted(NOT_MINECRAFT, &[&identifier])),
        }));
    }
    Ok((text, root, FeatureId::parse(identifier)))
}

/// Loads one feature file, read from `origin`: the identifier it claims (when the loader stage
/// read one) and the document or its rejection. `source` says whose definition the file is.
pub(crate) fn parse_file(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
    source: FileSource,
) -> (Option<FeatureId>, Result<Document<Feature>, Rejected>) {
    let prepared = Prepared::new(bytes);
    let (text, root, identifier) = match identify(&prepared, &origin.path, source) {
        Ok(found) => found,
        Err(d) => return (None, Err(Rejected::located(origin, d))),
    };
    let key = payload_key(&root);
    let stage = Stage {
        text,
        key: &key,
        cx,
    };
    let Some((value, version, mut diagnostics)) = with_type(&key, LoadType(stage)) else {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::error(DiagnosticCode::UnknownPayloadKey {
            name: key.clone(),
        }));
        return (Some(identifier), Err(Rejected::located(origin, d)));
    };
    diagnostics.locate(&origin);
    let result = match value {
        Some(feature) => Ok(Document::new(feature, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    };
    (Some(identifier), result)
}

/// [`Feature::parse_as_written`] for a file of `pack`.
pub(crate) fn parse_file_as_written(
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<history::Any>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
    let prepared = Prepared::new(bytes);
    let (text, root, _) = match identify(&prepared, path, FileSource::Pack) {
        Ok(found) => found,
        Err(d) => return Err(Rejected::located(origin, d)),
    };
    let key = payload_key(&root);
    let stage = Stage {
        text,
        key: &key,
        cx,
    };
    let Some((value, version, mut diagnostics)) = with_type(&key, ParseType(stage)) else {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::error(DiagnosticCode::UnknownPayloadKey {
            name: key.clone(),
        }));
        return Err(Rejected::located(origin, d));
    };
    diagnostics.locate(&origin);
    match value {
        Some(value) => Ok(Document::new(value, origin, version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The upgrade of a document `parse_as_written` gave.
struct UpgradeType<'a> {
    any: history::Any,
    version: bedrock_addon_core::document::VersionInfo,
    diagnostics: Diagnostics,
    cx: &'a LoadContext,
}

impl
    TypeFn<
        Result<
            (
                Feature,
                bedrock_addon_core::document::VersionInfo,
                Diagnostics,
            ),
            Diagnostics,
        >,
    > for UpgradeType<'_>
{
    fn call<T: FeatureType>(
        self,
    ) -> Result<
        (
            Feature,
            bedrock_addon_core::document::VersionInfo,
            Diagnostics,
        ),
        Diagnostics,
    > {
        let identifier = self.any.identifier().as_str().to_owned();
        let Ok(chain) = T::from_history(self.any) else {
            return Err(self.diagnostics);
        };
        let at = JsonPath::new().child(T::KEY);
        let written = FormatVersionInput {
            version: Some(self.version.effective),
            beta_literal: false,
            prerelease: false,
        };
        let identity = bedrock_addon_core::versioning::Identity {
            identifier: Some(identifier),
            runtime_identifier: None,
        };
        let parsed =
            Parsed::<Fam<T>>::from_value(Some(chain), self.version, written, at, self.diagnostics)
                .with_identity(identity);
        let outcome = finish(parsed, self.cx);
        match outcome.value {
            Some(Loaded::Stable(value)) => {
                Ok((value.into_feature(), outcome.version, outcome.diagnostics))
            }
            _ => Err(outcome.diagnostics),
        }
    }
}

/// Rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<history::Any>,
    cx: &LoadContext,
) -> Result<Document<Feature>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let any = document.into_value();
    let key = any.type_key();
    let upgrade = UpgradeType {
        any,
        version,
        diagnostics: diagnostics.clone(),
        cx,
    };
    match with_type(key, upgrade) {
        Some(Ok((feature, version, mut diagnostics))) => {
            diagnostics.locate(&origin);
            Ok(Document::new(feature, origin, version, diagnostics))
        }
        Some(Err(diagnostics)) => Err(Rejected::located(origin, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

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
            &Extra::default(),
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
    ) -> Result<Document<history::Any>, Rejected> {
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
    let text = text.then(|| formatted(DUPLICATE, &[&id.as_str()]));
    Diagnostic::error(DiagnosticCode::DuplicateIdentifier { text })
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

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by,
    //! for every feature type.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    use super::{Fam, FeatureType, KEYS, TypeFn, with_type};

    /// The reading table of one feature type, headed by its payload key.
    struct Table;

    impl TypeFn<String> for Table {
        fn call<T: FeatureType>(self) -> String {
            format!("{}\n{}", T::KEY, reading_table::<Fam<T>>())
        }
    }

    /// The tables of every feature type, in `Feature`'s order.
    const TABLES: &str = r"
            minecraft:aggregate_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:beards_and_shavers
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:cave_carver_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:conditional_list
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:fossil_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:geode_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:growing_plant_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:height_difference_filter_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:horizontal_tree_decoration_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:multi_block_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.26.40..=1.26.40 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:multiface_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:multipart_block_column_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.26.40..=1.26.40 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:nether_cave_carver_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:ore_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:partially_exposed_blob_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:rect_layout
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:scan_surface
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:scatter_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.21.10 (0..=1): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:sculk_patch_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:search_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:sequence_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:single_block_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.21.40 (0..=1): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:snap_to_surface_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.26.50 (0..=1): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:structure_template_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:surface_relative_threshold_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:tree_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:underwater_cave_carver_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:vegetation_patch_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            minecraft:weighted_random_feature
            text: Root, clamps beyond a double: true, duplicates kept in: []
            envelope: Always
            missing version: rejected: NoValidVersion
            below floor: UseFormatVersion
            empty document: Error
            1.13.0..=1.13.0 (0..=0): json Absent/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
        ";

    #[test]
    fn feature_reads_as_pinned() {
        let tables: String = KEYS
            .iter()
            .map(|key| with_type(key, Table).expect("a registered key"))
            .collect();
        assert_reading_table("Fam<T>", &tables, TABLES);
    }
}
