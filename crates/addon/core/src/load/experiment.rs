//! Experiments by save name and the requirements content is gated on.

use uuid::Uuid;

use super::LoadContext;
use crate::molang::{Experiment as MolangExperiment, ExperimentMask};

/// One of vanilla's experiments, keyed by its save name (`gametest` for Beta APIs), never by
/// ordinal.
///
/// The union of every engine build's experiments (vanilla renumbers them between builds), in
/// the order of the experiment table; the discriminant is the crate's own number,
/// [`bit`](Self::bit), which indexes [`Experiments`] and the Molang experiment mask. The save name
/// is the key in a world's `experiments` compound and in the experiment lists the game sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum Experiment {
    /// `test_experiment`: an automated-test setting.
    TestExperiment = 0,
    /// `next_update`: the next update's content (and the legacy `experimentalgameplay` flag).
    NextUpdate = 1,
    /// A placeholder experiment vanilla never turns on; it has no save name.
    DisabledExperiment = 2,
    /// `experimental_text`.
    ExperimentalText = 3,
    /// `allow_seed_change`.
    AllowSeedChange = 4,
    /// `upcoming_creator_features`: the Upcoming Creator Features toggle.
    UpcomingCreatorFeatures = 5,
    /// `gametest`: the Beta APIs toggle.
    BetaApis = 6,
    /// `data_driven_vanilla_blocks_and_items`: on only in the game's internal builds, never
    /// in the released game.
    DataDrivenVanillaBlocksAndItems = 7,
    /// `disable_data_driven_vanilla_blocks_and_items` (internal).
    DisableDataDrivenVanillaBlocksAndItems = 8,
    /// `experimental_creator_cameras`.
    ExperimentalCreatorCameras = 9,
    /// `minecraft_explorer` (internal).
    MinecraftExplorer = 10,
    /// `deferred_technical_preview`.
    DeferredTechnicalPreview = 11,
    /// `villager_trades_rebalance`.
    VillagerTradesRebalance = 12,
    /// `vanilla_block_geometry` (internal).
    VanillaBlockGeometry = 13,
    /// `simplified_spawn_rules` (internal).
    SimplifiedSpawnRules = 14,
    /// `voxel_shapes`.
    VoxelShapes = 15,
    /// `poi` (internal).
    Poi = 16,
    /// `brain_prototype` (internal).
    BrainPrototype = 17,
    /// `furnace_recipe_book` (released in 1.26.30; a deprecated name since).
    FurnaceRecipeBook = 18,
    /// `custom_projectiles` (released in 1.26.30; a deprecated name since).
    CustomProjectiles = 19,
    /// `y_2026_drop_2` (released in 1.26.30; a deprecated name since).
    Y2026Drop2 = 20,
    /// `lwe`: lightweight entities (internal).
    Lwe = 21,
    /// `gameplay_ui` (internal).
    GameplayUi = 22,
    /// `game_event_system_refactor` (internal).
    GameEventSystemRefactor = 23,
    /// `offhand_parity` (internal).
    OffhandParity = 24,
    /// `y_2026_drop_3`.
    Y2026Drop3 = 25,
    /// `mao` (internal).
    Mao = 26,
    /// `creator_world_clocks` (internal).
    CreatorWorldClocks = 27,
    /// `y_2026_drop_4`: the experiment of the vanilla pack `experimental_y_2026_drop_4`.
    Y2026Drop4 = 28,
}

impl Experiment {
    /// Every experiment, in discriminant order.
    pub const ALL: &'static [Experiment] = &[
        Self::TestExperiment,
        Self::NextUpdate,
        Self::DisabledExperiment,
        Self::ExperimentalText,
        Self::AllowSeedChange,
        Self::UpcomingCreatorFeatures,
        Self::BetaApis,
        Self::DataDrivenVanillaBlocksAndItems,
        Self::DisableDataDrivenVanillaBlocksAndItems,
        Self::ExperimentalCreatorCameras,
        Self::MinecraftExplorer,
        Self::DeferredTechnicalPreview,
        Self::VillagerTradesRebalance,
        Self::VanillaBlockGeometry,
        Self::SimplifiedSpawnRules,
        Self::VoxelShapes,
        Self::Poi,
        Self::BrainPrototype,
        Self::FurnaceRecipeBook,
        Self::CustomProjectiles,
        Self::Y2026Drop2,
        Self::Lwe,
        Self::GameplayUi,
        Self::GameEventSystemRefactor,
        Self::OffhandParity,
        Self::Y2026Drop3,
        Self::Mao,
        Self::CreatorWorldClocks,
        Self::Y2026Drop4,
    ];

    /// Names vanilla recognises in a world's experiments and never treats as toggles: the
    /// experiments it has retired.
    pub const DEPRECATED_NAMES: &'static [&'static str] = &[
        "caves_and_cliffs",
        "wild_update",
        "spectator_mode",
        "vanilla_experiments",
        "bundles",
        "data_driven_items",
        "data_driven_biomes",
        "scripting",
        "experimental_molang_features",
        "next_major_update",
        "sniffer",
        "updateAnnouncedLive2023",
        "armadillo",
        "jigsaw_structures",
        "camera_aim_assist",
        "y_2025_drop_1",
        "y_2025_drop_2",
        "locator_bar",
        "y_2025_drop_3",
        "y_2026_drop_1",
        "furnace_recipe_book",
        "custom_projectiles",
        "y_2026_drop_2",
    ];

    /// The save name: the key in a world's `experiments` and in the game's experiment lists
    /// (`gametest` for [`BetaApis`](Self::BetaApis)); `None` for
    /// [`DisabledExperiment`](Self::DisabledExperiment).
    // The names of `GameplayUi`, `GameEventSystemRefactor`, `OffhandParity`, `Mao` and
    // `CreatorWorldClocks` are the snake case of the enum name.
    pub const fn save_name(self) -> Option<&'static str> {
        Some(match self {
            Self::TestExperiment => "test_experiment",
            Self::NextUpdate => "next_update",
            Self::DisabledExperiment => return None,
            Self::ExperimentalText => "experimental_text",
            Self::AllowSeedChange => "allow_seed_change",
            Self::UpcomingCreatorFeatures => "upcoming_creator_features",
            Self::BetaApis => "gametest",
            Self::DataDrivenVanillaBlocksAndItems => "data_driven_vanilla_blocks_and_items",
            Self::DisableDataDrivenVanillaBlocksAndItems => {
                "disable_data_driven_vanilla_blocks_and_items"
            }
            Self::ExperimentalCreatorCameras => "experimental_creator_cameras",
            Self::MinecraftExplorer => "minecraft_explorer",
            Self::DeferredTechnicalPreview => "deferred_technical_preview",
            Self::VillagerTradesRebalance => "villager_trades_rebalance",
            Self::VanillaBlockGeometry => "vanilla_block_geometry",
            Self::SimplifiedSpawnRules => "simplified_spawn_rules",
            Self::VoxelShapes => "voxel_shapes",
            Self::Poi => "poi",
            Self::BrainPrototype => "brain_prototype",
            Self::FurnaceRecipeBook => "furnace_recipe_book",
            Self::CustomProjectiles => "custom_projectiles",
            Self::Y2026Drop2 => "y_2026_drop_2",
            Self::Lwe => "lwe",
            Self::GameplayUi => "gameplay_ui",
            Self::GameEventSystemRefactor => "game_event_system_refactor",
            Self::OffhandParity => "offhand_parity",
            Self::Y2026Drop3 => "y_2026_drop_3",
            Self::Mao => "mao",
            Self::CreatorWorldClocks => "creator_world_clocks",
            Self::Y2026Drop4 => "y_2026_drop_4",
        })
    }

    /// The experiment saved as `name`; case-sensitive, as vanilla's lookup.
    pub fn from_save_name(name: &str) -> Option<Experiment> {
        Self::ALL
            .iter()
            .copied()
            .find(|experiment| experiment.save_name() == Some(name))
    }

    /// The crate's number for the experiment: its bit in [`Experiments`] and in the Molang
    /// experiment mask.
    pub const fn bit(self) -> u8 {
        self as u8
    }

    const fn mask(self) -> u64 {
        1 << self.bit()
    }
}

/// What [`Experiments::insert_save_name`] made of a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveName {
    /// The save name of this experiment, now enabled.
    Known(Experiment),
    /// A retired experiment's name: kept, never a toggle, not worth a finding.
    Deprecated,
    /// A name no known game version uses (a newer game's experiment): kept so it survives.
    Unknown,
}

/// The set of enabled experiments, keeping unknown save names so a newer pack's experiment
/// does not fail to parse.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Experiments {
    /// Bit `n` is the experiment whose [`Experiment::bit`] is `n`.
    bits: u64,
    /// Names that are no [`Experiment`]'s save name, deprecated ones included, in insertion
    /// order, each once.
    unknown: Vec<String>,
}

impl Experiments {
    /// No experiment enabled; `Default` too.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `experiment` is enabled.
    pub fn contains(&self, experiment: Experiment) -> bool {
        self.bits & experiment.mask() != 0
    }

    /// Enables `experiment`; `false` if it already was.
    pub fn insert(&mut self, experiment: Experiment) -> bool {
        let added = !self.contains(experiment);
        self.bits |= experiment.mask();
        added
    }

    /// Disables `experiment`; `false` if it was not enabled.
    pub fn remove(&mut self, experiment: Experiment) -> bool {
        let removed = self.contains(experiment);
        self.bits &= !experiment.mask();
        removed
    }

    /// Enables the experiment saved as `name`, or keeps the name when it is no experiment's
    /// (a deprecated or a newer game's name). The result says which, so a reader of world
    /// settings can report an unknown name.
    ///
    /// A deprecated name is kept even when an older build had it as an experiment
    /// (`furnace_recipe_book`): the engine this crate tracks no longer toggles it.
    pub fn insert_save_name(&mut self, name: &str) -> SaveName {
        // A name on vanilla's deprecated list also goes to `unknown`.
        let deprecated = Experiment::DEPRECATED_NAMES.contains(&name);
        if !deprecated && let Some(experiment) = Experiment::from_save_name(name) {
            self.insert(experiment);
            return SaveName::Known(experiment);
        }
        if !self.unknown.iter().any(|kept| kept == name) {
            self.unknown.push(name.to_owned());
        }
        if deprecated {
            SaveName::Deprecated
        } else {
            SaveName::Unknown
        }
    }

    /// The enabled experiments, in discriminant order.
    pub fn iter(&self) -> impl Iterator<Item = Experiment> + '_ {
        Experiment::ALL
            .iter()
            .copied()
            .filter(|experiment| self.contains(*experiment))
    }

    /// The kept names that are no experiment's save name, in the order they were inserted.
    pub fn unknown(&self) -> &[String] {
        &self.unknown
    }

    /// No experiment enabled and no name kept.
    pub fn is_empty(&self) -> bool {
        self.bits == 0 && self.unknown.is_empty()
    }

    /// The enabled experiments as molangx's experiment mask: bit `n` is the experiment whose
    /// [`Experiment::bit`] is `n`.
    pub fn molang_mask(&self) -> ExperimentMask {
        self.iter()
            .filter_map(|experiment| MolangExperiment::new(experiment.bit()))
            .collect()
    }
}

impl IntoIterator for &Experiments {
    type Item = Experiment;
    type IntoIter = std::vec::IntoIter<Experiment>;

    /// The enabled experiments, as [`Experiments::iter`] gives them.
    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl FromIterator<Experiment> for Experiments {
    fn from_iter<I: IntoIterator<Item = Experiment>>(iter: I) -> Self {
        let mut experiments = Self::new();
        experiments.extend(iter);
        experiments
    }
}

impl Extend<Experiment> for Experiments {
    fn extend<I: IntoIterator<Item = Experiment>>(&mut self, iter: I) {
        for experiment in iter {
            self.insert(experiment);
        }
    }
}

/// What unlocks a component or member before its release version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LoadRequirement {
    /// The experiment is enabled.
    Experiment(Experiment),
    /// The pack is a base-game pack (one the game ships).
    BaseGamePack,
    /// Beta features: the Beta APIs toggle or a base-game pack. Kept for older metadata
    /// tables; no current component needs it.
    BetaFeatures,
    /// The pack is one of ten Mojang packs allowed the legacy creator features (legacy block
    /// events, the removed item components). No base-game or toggle shortcut.
    LegacyCreatorFeatures,
}

/// The packs the game's legacy-creator-features requirement allows: ten pack UUIDs.
const LEGACY_CREATOR_FEATURES_PACKS: [Uuid; 10] = [
    Uuid::from_u128(0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166),
    Uuid::from_u128(0x168d_15a6_78cd_4133_ad85_ecc2_4a0a_d6db),
    Uuid::from_u128(0x955c_2b94_306a_48df_bfc6_22f9_0d95_1d25),
    Uuid::from_u128(0x6d63_491a_ea68_446f_bf1a_4185_375d_b7d7),
    Uuid::from_u128(0x29a9_003d_4e59_4a1c_87c9_dea2_f898_8623),
    Uuid::from_u128(0x27f9_ef69_0dc9_48ba_a857_f9b8_c1d0_df5b),
    Uuid::from_u128(0x6c4a_4939_9f5a_45ad_bf93_013e_d60e_47c6),
    Uuid::from_u128(0xb4a0_f58d_fccb_4198_86f5_b44a_4fb1_62cd),
    Uuid::from_u128(0x81e5_d014_0f03_4cd0_aef0_ba69_14f6_ac2f),
    Uuid::from_u128(0x185f_05da_1408_4070_b3cf_75a0_baa1_7d8f),
];

impl LoadRequirement {
    /// Whether a document loaded with `cx` meets the requirement.
    pub fn satisfied(&self, cx: &LoadContext) -> bool {
        match *self {
            // The two toggles are also their experiments.
            Self::Experiment(experiment) => {
                cx.experiments.contains(experiment)
                    || (experiment == Experiment::BetaApis && cx.beta_apis)
                    || (experiment == Experiment::UpcomingCreatorFeatures
                        && cx.upcoming_creator_features)
            }
            Self::BaseGamePack => cx.is_base_game_pack,
            // The game's beta-features requirement holds with Beta APIs on or in a base game pack.
            Self::BetaFeatures => cx.beta_apis || cx.is_base_game_pack,
            Self::LegacyCreatorFeatures => LEGACY_CREATOR_FEATURES_PACKS.contains(&cx.pack_uuid),
        }
    }
}
