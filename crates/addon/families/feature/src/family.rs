//! The feature types: one row per type declares [`Feature`]'s variant, the [`FeatureType`] of
//! its payload key, its family and chain, the dispatch by payload key and `Feature`'s per-type
//! accessors ([`feature_types!`]); and the [`strict_struct!`] the types' structs are declared
//! with.

use std::convert::Infallible;
use std::marker::PhantomData;

use crate::{blocks, carver, history, placement, scatter, structure, tree};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BeyondWidth, Envelope, Family, InputKind, Loaded, MissingVersion, NoComponents, Reading,
    Scalars, StructIndex, ValidateCx, VersionChain, VersionStruct,
};

/// Format 1.13.0, the initial release (`[1.13.0, 1.21.10)`) and the floor of every feature
/// type but `multi_block_feature` and `multipart_block_column_feature`, which are registered
/// at 1.26.40 (`[1.26.40, 1.26.50)`). The format versions at which a feature type's schema
/// changes are 1.13.0, 1.21.10 (scatter), 1.21.40 (single block), 1.26.40 (multi block) and
/// 1.26.50 (snap to surface), the last one running to the current version.
pub(crate) const V1_13_0: FormatVersion = FormatVersion::new(1, 13, 0);

/// The reading of every slice of every feature type, which the payload's reader takes
/// (`read::parse_strict_with`): the jsoncpp input's profile (rule N1, unknown members warned,
/// lenient), with the strict scalars the types' members are read by: a member takes only a
/// token of its own type and a number beyond its float's width is a type error.
pub(crate) const READING: Reading = InputKind::JsonCpp
    .profile()
    .with_scalars(Scalars::Strict)
    .with_beyond_width(BeyondWidth::Error);

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

/// A feature type's rules on a loaded value.
pub(crate) trait Rules {
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

/// A feature type: its payload key, what its row says about it, its chain and its place in
/// [`Feature`].
pub(crate) trait FeatureType: Rules + VersionStruct + Send + Sync + 'static {
    /// The payload key, `minecraft:ore_feature`.
    const KEY: &'static str;
    /// Only base-game packs may use the type (the row's `internal`).
    const INTERNAL: bool;
    /// The engine release that registers the type; below it the payload key is unknown (the
    /// row's `since_engine`).
    const SINCE_ENGINE: Option<FormatVersion>;
    /// The type's lowest format version (the row's `floor`, 1.13.0 when it names none).
    const FLOOR: FormatVersion;
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

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        READING
    }

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

/// Declares [`Feature`] and, from one row per feature type, its variant, its chain, its
/// `FeatureType` impl (key, `internal`, `since_engine`, `floor`), the `VersionStruct` of its
/// newest struct where it parses strictly (`strict = version`, by [`strict_struct!`]), the
/// dispatch by payload key and `Feature`'s per-type accessors. A row is
/// `Variant(Type) = "key", Chain [Version(Struct), …]` and its keys in that order; a variant
/// that holds its type boxed says so, `Variant(Type => Box<Type>)`.
macro_rules! feature_types {
    (@or $value:expr ; $default:expr) => { $value };
    (@or ; $default:expr) => { $default };
    (@held $ty:ty) => { $ty };
    (@held $ty:ty, $held:ty) => { $held };
    (
        $(#[$enum_meta:meta])*
        pub enum Feature;
        $(
            $(#[$doc:meta])*
            $variant:ident($ty:ty $(=> $held:ty)?) = $key:literal,
                $chain:ident [ $($v:ident($t:ty)),+ ]
                $(, strict = $strict:expr)?
                $(, internal = $internal:literal)?
                $(, since_engine = $since:expr)?
                $(, floor = $floor:expr)?;
        )*
    ) => {
        $(#[$enum_meta])*
        pub enum Feature {
            $(
                $(#[$doc])*
                $variant(feature_types!(@held $ty $(, $held)?)),
            )*
        }

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

            $( strict_struct!($ty, $strict); )?

            impl FeatureType for $ty {
                const KEY: &'static str = $key;
                const INTERNAL: bool = feature_types!(@or $($internal)? ; false);
                const SINCE_ENGINE: Option<FormatVersion> =
                    feature_types!(@or $(Some($since))? ; None);
                const FLOOR: FormatVersion = feature_types!(@or $($floor)? ; V1_13_0);
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
        impl crate::molang::Refer for Feature {
            fn refer<'a>(&'a self, out: &mut crate::molang::References<'a>) {
                match self {
                    $( Feature::$variant(value) => {
                        crate::molang::Refer::refer(Held::<$ty>::inner(value), out)
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
    /// A feature: one placement rule of world generation, of one of vanilla's feature types, at the
    /// newest format version of its type.
    ///
    /// JSON: `{"format_version": "…", "minecraft:<type>_feature": {"description": {"identifier"},
    /// …}}`; the payload key names the type. A document of an older format version is upgraded
    /// when it loads (`history` has the older shapes); a document that breaks its type's schema in
    /// any way is not loaded, as vanilla's strict schema rejects it.
    #[derive(Clone, Debug, PartialEq)]
    #[non_exhaustive]
    #[doc(alias = "features")]
    #[allow(
        clippy::large_enum_variant,
        reason = "the variants are the types callers match on; the one far larger than the rest is boxed"
    )]
    pub enum Feature;

    /// `minecraft:aggregate_feature`.
    Aggregate(placement::AggregateFeature) = "minecraft:aggregate_feature",
        AggregateChain [V1_13_0(placement::AggregateFeature)],
        strict = V1_13_0;
    /// `minecraft:beards_and_shavers` (base-game packs only).
    BeardsAndShavers(structure::BeardsAndShaversFeature) = "minecraft:beards_and_shavers",
        BeardsAndShaversChain [V1_13_0(structure::BeardsAndShaversFeature)],
        strict = V1_13_0, internal = true;
    /// `minecraft:cave_carver_feature`.
    CaveCarver(carver::CaveCarverFeature) = "minecraft:cave_carver_feature",
        CaveCarverChain [V1_13_0(carver::CaveCarverFeature)];
    /// `minecraft:conditional_list`.
    ConditionalList(placement::ConditionalListFeature) = "minecraft:conditional_list",
        ConditionalListChain [V1_13_0(placement::ConditionalListFeature)],
        strict = V1_13_0;
    /// `minecraft:fossil_feature`.
    Fossil(blocks::FossilFeature) = "minecraft:fossil_feature",
        FossilChain [V1_13_0(blocks::FossilFeature)],
        strict = V1_13_0;
    /// `minecraft:geode_feature`.
    Geode(blocks::GeodeFeature) = "minecraft:geode_feature",
        GeodeChain [V1_13_0(blocks::GeodeFeature)],
        strict = V1_13_0;
    /// `minecraft:growing_plant_feature`.
    GrowingPlant(blocks::GrowingPlantFeature) = "minecraft:growing_plant_feature",
        GrowingPlantChain [V1_13_0(blocks::GrowingPlantFeature)];
    /// `minecraft:height_difference_filter_feature` (engine 1.26.30).
    HeightDifferenceFilter(placement::HeightDifferenceFilterFeature) = "minecraft:height_difference_filter_feature",
        HeightDifferenceFilterChain [V1_13_0(placement::HeightDifferenceFilterFeature)],
        strict = V1_13_0, since_engine = placement::HEIGHT_DIFFERENCE_ENGINE;
    /// `minecraft:horizontal_tree_decoration_feature` (engine 1.26.40).
    HorizontalTreeDecoration(blocks::HorizontalTreeDecorationFeature) = "minecraft:horizontal_tree_decoration_feature",
        HorizontalTreeDecorationChain [V1_13_0(blocks::HorizontalTreeDecorationFeature)],
        strict = V1_13_0, since_engine = blocks::HORIZONTAL_TREE_DECORATION_ENGINE;
    /// `minecraft:multi_block_feature` (engine 1.26.40).
    MultiBlock(blocks::MultiBlockFeature) = "minecraft:multi_block_feature",
        MultiBlockChain [V1_26_40(blocks::MultiBlockFeature)],
        strict = blocks::MULTI_BLOCK_FORMAT, since_engine = blocks::MULTI_BLOCK_FORMAT, floor = blocks::MULTI_BLOCK_FORMAT;
    /// `minecraft:multiface_feature`.
    Multiface(blocks::MultifaceFeature) = "minecraft:multiface_feature",
        MultifaceChain [V1_13_0(blocks::MultifaceFeature)],
        strict = V1_13_0;
    /// `minecraft:multipart_block_column_feature` (format 1.26.40).
    MultipartBlockColumn(blocks::MultipartBlockColumnFeature) = "minecraft:multipart_block_column_feature",
        MultipartBlockColumnChain [V1_26_40(blocks::MultipartBlockColumnFeature)],
        strict = blocks::MULTI_BLOCK_FORMAT, since_engine = blocks::MULTI_BLOCK_FORMAT, floor = blocks::MULTI_BLOCK_FORMAT;
    /// `minecraft:nether_cave_carver_feature`.
    NetherCaveCarver(carver::NetherCaveCarverFeature) = "minecraft:nether_cave_carver_feature",
        NetherCaveCarverChain [V1_13_0(carver::NetherCaveCarverFeature)];
    /// `minecraft:ore_feature`.
    Ore(blocks::OreFeature) = "minecraft:ore_feature",
        OreChain [V1_13_0(blocks::OreFeature)],
        strict = V1_13_0;
    /// `minecraft:partially_exposed_blob_feature`.
    PartiallyExposedBlob(blocks::PartiallyExposedBlobFeature) = "minecraft:partially_exposed_blob_feature",
        PartiallyExposedBlobChain [V1_13_0(blocks::PartiallyExposedBlobFeature)];
    /// `minecraft:rect_layout` (base-game packs only).
    RectLayout(placement::RectLayoutFeature) = "minecraft:rect_layout",
        RectLayoutChain [V1_13_0(placement::RectLayoutFeature)],
        strict = V1_13_0, internal = true;
    /// `minecraft:scan_surface` (base-game packs only).
    ScanSurface(placement::ScanSurfaceFeature) = "minecraft:scan_surface",
        ScanSurfaceChain [V1_13_0(placement::ScanSurfaceFeature)],
        strict = V1_13_0, internal = true;
    /// `minecraft:scatter_feature`.
    Scatter(scatter::ScatterFeature) = "minecraft:scatter_feature",
        ScatterChain [V1_13_0(history::v1_13_0::ScatterFeature), V1_21_10(scatter::ScatterFeature)];
    /// `minecraft:sculk_patch_feature` (base-game packs only).
    SculkPatch(blocks::SculkPatchFeature) = "minecraft:sculk_patch_feature",
        SculkPatchChain [V1_13_0(blocks::SculkPatchFeature)],
        strict = V1_13_0, internal = true;
    /// `minecraft:search_feature`.
    Search(placement::SearchFeature) = "minecraft:search_feature",
        SearchChain [V1_13_0(placement::SearchFeature)],
        strict = V1_13_0;
    /// `minecraft:sequence_feature`.
    Sequence(placement::SequenceFeature) = "minecraft:sequence_feature",
        SequenceChain [V1_13_0(placement::SequenceFeature)],
        strict = V1_13_0;
    /// `minecraft:single_block_feature`.
    SingleBlock(blocks::SingleBlockFeature) = "minecraft:single_block_feature",
        SingleBlockChain [V1_13_0(history::v1_13_0::SingleBlockFeature), V1_21_40(blocks::SingleBlockFeature)];
    /// `minecraft:snap_to_surface_feature`.
    SnapToSurface(structure::SnapToSurfaceFeature) = "minecraft:snap_to_surface_feature",
        SnapToSurfaceChain [V1_13_0(history::v1_13_0::SnapToSurfaceFeature), V1_26_50(structure::SnapToSurfaceFeature)];
    /// `minecraft:structure_template_feature`.
    StructureTemplate(structure::StructureTemplateFeature) = "minecraft:structure_template_feature",
        StructureTemplateChain [V1_13_0(structure::StructureTemplateFeature)],
        strict = V1_13_0;
    /// `minecraft:surface_relative_threshold_feature`.
    SurfaceRelativeThreshold(placement::SurfaceRelativeThresholdFeature) = "minecraft:surface_relative_threshold_feature",
        SurfaceRelativeThresholdChain [V1_13_0(placement::SurfaceRelativeThresholdFeature)],
        strict = V1_13_0;
    /// `minecraft:tree_feature`. Boxed: a `TreeFeature` holds every trunk and canopy shape
    /// inline (some 17 KB, four times the next type), and an unoptimised build keeps a copy of
    /// a `Feature` in each frame that moves one; held inline it took a 1 MiB thread past its
    /// stack on the base game's load.
    Tree(tree::TreeFeature => Box<tree::TreeFeature>) = "minecraft:tree_feature",
        TreeChain [V1_13_0(tree::TreeFeature)],
        strict = V1_13_0;
    /// `minecraft:underwater_cave_carver_feature`.
    UnderwaterCaveCarver(carver::UnderwaterCaveCarverFeature) = "minecraft:underwater_cave_carver_feature",
        UnderwaterCaveCarverChain [V1_13_0(carver::UnderwaterCaveCarverFeature)];
    /// `minecraft:vegetation_patch_feature`.
    VegetationPatch(blocks::VegetationPatchFeature) = "minecraft:vegetation_patch_feature",
        VegetationPatchChain [V1_13_0(blocks::VegetationPatchFeature)];
    /// `minecraft:weighted_random_feature`.
    WeightedRandom(placement::WeightedRandomFeature) = "minecraft:weighted_random_feature",
        WeightedRandomChain [V1_13_0(placement::WeightedRandomFeature)],
        strict = V1_13_0;
}
