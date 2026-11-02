//! What vanilla makes of a legacy condition at load: its `MobSpawnRules` (defaults applied, ranges
//! swapped and clamped) and the condition's weight.

use super::condition::{
    Condition, KEYS, MAX_DEFAULT_SPAWN_DISTANCE, MIN_DEFAULT_SPAWN_DISTANCE, Profile, sorted,
};
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::shared::BlockDescriptor;

/// A world difficulty, as `minecraft:difficulty_filter` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Difficulty {
    /// `peaceful` (0).
    Peaceful,
    /// `easy` (1).
    Easy,
    /// `normal` (2).
    Normal,
    /// `hard` (3).
    Hard,
}

impl Difficulty {
    /// The difficulty a filter names, compared after lower-casing (`"EASY"` is easy); `None` for
    /// any other name.
    // The name is lower-cased first.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "peaceful" => Some(Self::Peaceful),
            "easy" => Some(Self::Easy),
            "normal" => Some(Self::Normal),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }

    /// The JSON name (`"peaceful"`, …).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }
}

/// The spawn pool an entity's spawn rules count against (JSON
/// `description.population_control`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PopulationControl {
    /// `animal`.
    Animal,
    /// `monster`.
    Monster,
    /// `water_animal`.
    WaterAnimal,
    /// `villager`.
    Villager,
    /// `ambient`.
    Ambient,
    /// `cat`.
    Cat,
    /// `pillager`.
    Pillager,
}

impl PopulationControl {
    /// The pool a name selects, compared after lower-casing (`"Monster"` is the monster pool);
    /// `None` for an empty or unknown name, which leaves the spawn rules unregistered.
    // The name is lower-cased first; an unknown name leaves the rules unregistered.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "animal" => Some(Self::Animal),
            "monster" => Some(Self::Monster),
            "water_animal" => Some(Self::WaterAnimal),
            "villager" => Some(Self::Villager),
            "ambient" => Some(Self::Ambient),
            "cat" => Some(Self::Cat),
            "pillager" => Some(Self::Pillager),
            _ => None,
        }
    }

    /// The JSON name (`"animal"`, …).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Animal => "animal",
            Self::Monster => "monster",
            Self::WaterAnimal => "water_animal",
            Self::Villager => "villager",
            Self::Ambient => "ambient",
            Self::Cat => "cat",
            Self::Pillager => "pillager",
        }
    }
}

/// One herd as vanilla stores it: the sizes and counts are unsigned, so a
/// negative number in the document wraps.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct HerdInfo {
    /// The smallest herd.
    pub min_size: u32,
    /// The largest herd.
    pub max_size: u32,
    /// The event sent to the herd's members (`""` for none).
    pub event: String,
    /// How many members skip `event`.
    pub event_skip_count: u32,
    /// The event sent to the first members (`""` for none).
    pub initial_event: String,
    /// How many members get `initial_event`.
    pub initial_event_count: u32,
}

/// What vanilla loads from one legacy condition: every value with its default
/// applied, `min > max` pairs swapped and the clamps done. The condition's weight and biome
/// filter are not part of it ([`Condition::spawn_weight`], [`Condition::biome_filter`]).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[allow(clippy::struct_excessive_bools, reason = "the game's spawn-rule flags")]
pub struct MobSpawnRules {
    /// The lowest light level, 0..=15.
    pub brightness_min: i32,
    /// The highest light level, 0..=15.
    pub brightness_max: i32,
    /// Darken the light level for weather.
    pub adjust_for_weather: bool,
    /// The lowest height (`i32::MIN` without a height filter).
    pub height_min: i32,
    /// The highest height (`i32::MAX` without a height filter).
    pub height_max: i32,
    /// The rarity.
    pub rarity: i32,
    /// The population cap on the surface (−1: none).
    pub surface_cap: i32,
    /// The population cap underground (−1: none).
    pub underground_cap: i32,
    /// The lowest difficulty.
    pub difficulty_min: Difficulty,
    /// The highest difficulty.
    pub difficulty_max: Difficulty,
    /// The smallest distance from a player.
    pub spawn_distance_min: i32,
    /// The largest distance from a player, at most `spawn_distance_cap`.
    pub spawn_distance_max: i32,
    /// The cap of `spawn_distance_max`: 128 for spawn rules, 1024 for spawn groups.
    pub spawn_distance_cap: i32,
    /// Spawns on the surface.
    pub surface_spawner: bool,
    /// Spawns underground.
    pub underground_spawner: bool,
    /// Spawns in water.
    pub underwater_spawner: bool,
    /// Spawns in lava.
    pub lava_spawner: bool,
    /// May spawn in a bubble column (true unless the condition disallows it).
    pub bubble_spawner: bool,
    /// Only with the experimental gameplay toggle.
    pub experimental: bool,
    /// Spawned entities never despawn.
    pub persistent: bool,
    /// The distance from the village a player must be in (0: no village check).
    pub village_distance: u32,
    /// How far outside the village border still counts.
    pub village_border_tolerance: u32,
    /// The lowest world age in ticks.
    pub world_age_min: u64,
    /// The highest world age in ticks.
    pub world_age_max: u64,
    /// The shortest delay (never negative).
    pub delay_min: i32,
    /// The longest delay.
    pub delay_max: i32,
    /// The entity the delay is for (`""` for none).
    pub delay_identifier: String,
    /// The delay's spawn chance in percent (0 without a delay filter).
    pub delay_spawn_chance: i32,
    /// The mob event that must be on (`""` for none).
    pub mob_event: String,
    /// How far above the spawn position `spawns_above_blocks` are looked for.
    pub above_block_distance: i32,
    /// The blocks the entity may spawn on.
    pub spawns_on_blocks: Vec<BlockDescriptor>,
    /// The blocks the entity may not spawn on.
    pub spawns_on_blocks_prevented: Vec<BlockDescriptor>,
    /// Blocks that must be above the spawn position.
    pub spawns_above_blocks: Vec<BlockDescriptor>,
    /// The herds, in document order.
    pub herds: Vec<HerdInfo>,
    /// The weighted pick of permutations: each entity with a weight of at least 1.
    pub permutations: Vec<(i32, EntityId)>,
    /// The permutations spawned regardless of the pick: each entity with a count of at least 1.
    pub guaranteed: Vec<(i32, EntityId)>,
}

impl Condition {
    /// The condition's spawn weight: 100 without `minecraft:weight`, else its `default` (100
    /// when absent), negative values 0.
    #[must_use]
    pub fn spawn_weight(&self) -> i32 {
        // Starts at 100 and is read with a default of 100, then clamped to be non-negative.
        self.weight
            .as_ref()
            .map_or(100, |w| w.default.unwrap_or(100).max(0))
    }

    /// What vanilla's spawn-rules parser loads from this condition (spawn-distance cap 128).
    #[must_use]
    pub fn to_spawn_rules(&self) -> MobSpawnRules {
        self.rules_with(super::condition::SPAWN_RULES)
    }

    /// The `MobSpawnRules` of this condition under `profile`.
    #[allow(
        clippy::too_many_lines,
        reason = "one value per `MobSpawnRules` member"
    )]
    pub fn rules_with(&self, profile: Profile) -> MobSpawnRules {
        let flag = |value: &Option<bedrock_addon_core::json::Value>| value.is_some();
        let descriptors = |list: Option<&super::condition::OneOrMany<BlockDescriptor>>| {
            list.map(|l| l.as_slice().to_vec()).unwrap_or_default()
        };
        let (brightness_min, brightness_max, adjust_for_weather) =
            self.brightness_filter.as_ref().map_or((0, 15, false), |b| {
                (
                    b.min.unwrap_or(0).clamp(0, 15),
                    b.max.unwrap_or(15).clamp(0, 15),
                    b.adjust_for_weather.unwrap_or(false),
                )
            });
        let (height_min, height_max) = self.height_filter.as_ref().map_or_else(
            // A height filter of the wrong shape is 24..128.
            || {
                if self.extra.get(KEYS[8]).is_some() {
                    (MIN_DEFAULT_SPAWN_DISTANCE, MAX_DEFAULT_SPAWN_DISTANCE)
                } else {
                    (i32::MIN, i32::MAX)
                }
            },
            |h| sorted(h.ends()),
        );
        let (spawn_distance_min, spawn_distance_max) = self.distance_filter.as_ref().map_or(
            (MIN_DEFAULT_SPAWN_DISTANCE, MAX_DEFAULT_SPAWN_DISTANCE),
            |f| {
                let (min, max) = sorted(f.ends());
                (min.max(0), max.min(profile.distance_cap))
            },
        );
        let (difficulty_min, difficulty_max) =
            self.difficulty_filter
                .as_ref()
                .map_or((Difficulty::Peaceful, Difficulty::Hard), |f| {
                    let end = |name: &Option<String>, default| {
                        name.as_deref()
                            .and_then(Difficulty::from_name)
                            .unwrap_or(default)
                    };
                    (
                        end(&f.min, Difficulty::Peaceful),
                        end(&f.max, Difficulty::Hard),
                    )
                });
        let (world_age_min, world_age_max) =
            self.world_age_filter.as_ref().map_or((0, u64::MAX), |f| {
                sorted((f.min.unwrap_or(0), f.max.unwrap_or(u64::MAX)))
            });
        let (delay_min, delay_max, delay_identifier, delay_spawn_chance) = self
            .delay_filter
            .as_ref()
            // A delay filter of the wrong shape still reads `spawn_chance` (100).
            .map_or_else(
                || {
                    let chance = if self.extra.get(KEYS[19]).is_some() {
                        100
                    } else {
                        0
                    };
                    (0, 0, String::new(), chance)
                },
                |f| {
                    // Swapped first, then the delay range clamps the new min.
                    let (min, max) = sorted((f.min.unwrap_or(0), f.max.unwrap_or(0)));
                    (
                        min.max(0),
                        max,
                        f.identifier
                            .as_ref()
                            .map(|id| id.as_str().to_owned())
                            .unwrap_or_default(),
                        f.spawn_chance.unwrap_or(100),
                    )
                },
            );
        let (village_distance, village_border_tolerance) =
            self.player_in_village_filter.as_ref().map_or((0, 0), |f| {
                (
                    f.distance.unwrap_or(0),
                    f.village_border_tolerance.unwrap_or(10),
                )
            });
        let (surface_cap, underground_cap) = self.density_limit.as_ref().map_or((-1, -1), |l| {
            (l.surface.unwrap_or(-1), l.underground.unwrap_or(-1))
        });
        #[allow(clippy::cast_sign_loss, reason = "vanilla passes the ints as unsigned")]
        let herds = self
            .herd
            .iter()
            .flat_map(super::condition::OneOrMany::as_slice)
            .map(|h| HerdInfo {
                min_size: h.min_size.unwrap_or(1) as u32,
                max_size: h.max_size.unwrap_or(1) as u32,
                event: h.event.clone().unwrap_or_default(),
                event_skip_count: h.event_skip_count.unwrap_or(0) as u32,
                initial_event: h.initial_event.clone().unwrap_or_default(),
                initial_event_count: h.initial_event_count.unwrap_or(0) as u32,
            })
            .collect();
        let mut permutations = Vec::new();
        let mut guaranteed = Vec::new();
        for p in self
            .permute_type
            .iter()
            .flat_map(super::condition::OneOrMany::as_slice)
        {
            // Each permutation is added with its weight, its guaranteed count and its entity.
            let entity = p.entity_type.clone().unwrap_or_else(|| EntityId::parse(""));
            let weight = p.weight.unwrap_or(1);
            if weight >= 1 {
                permutations.push((weight, entity.clone()));
            }
            let count = p.guaranteed_count.unwrap_or(0);
            if count >= 1 {
                guaranteed.push((count, entity));
            }
        }
        MobSpawnRules {
            brightness_min,
            brightness_max,
            adjust_for_weather,
            height_min,
            height_max,
            rarity: self.weight.as_ref().and_then(|w| w.rarity).unwrap_or(0),
            surface_cap,
            underground_cap,
            difficulty_min,
            difficulty_max,
            spawn_distance_min,
            spawn_distance_max,
            spawn_distance_cap: profile.distance_cap,
            surface_spawner: flag(&self.spawns_on_surface),
            underground_spawner: flag(&self.spawns_underground),
            underwater_spawner: flag(&self.spawns_underwater),
            lava_spawner: flag(&self.spawns_lava),
            bubble_spawner: !flag(&self.disallow_spawns_in_bubble),
            experimental: flag(&self.is_experimental),
            persistent: flag(&self.is_persistent),
            village_distance,
            village_border_tolerance,
            world_age_min,
            world_age_max,
            delay_min,
            delay_max,
            delay_identifier,
            delay_spawn_chance,
            mob_event: self
                .mob_event_filter
                .as_ref()
                .and_then(|f| f.event.clone())
                .unwrap_or_default(),
            above_block_distance: self
                .spawns_above_block_filter
                .as_ref()
                .map_or(0, |a| a.distance.unwrap_or(1)),
            spawns_on_blocks: descriptors(self.spawns_on_block_filter.as_ref()),
            spawns_on_blocks_prevented: descriptors(self.spawns_on_block_prevented_filter.as_ref()),
            spawns_above_blocks: descriptors(
                self.spawns_above_block_filter
                    .as_ref()
                    .and_then(|a| a.blocks.as_ref()),
            ),
            herds,
            permutations,
            guaranteed,
        }
    }
}
