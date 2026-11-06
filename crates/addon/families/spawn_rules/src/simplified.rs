//! Simplified spawn rules (behind the Simplified Spawn Rules experiment): a cereal struct, read
//! typed from the payload; its numeric constraints and the placement-name validator.

use facet::Facet;

use super::texts;
use bedrock_addon_core::diagnostic::{
    Class, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::id::{EntityId, SpawnRulesId};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, ScalarFailure, Value, keep_unknown_members_with, parse_with_rules,
    unknown_members,
};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_shared_spawn::read::finding;

/// Spawn rules in the simplified shape (format 1.21.60 and later, with the Simplified Spawn Rules
/// experiment on): where an entity spawns is a named placement, and each condition is a weight
/// with herds, permutations, a mob event and a biome filter.
///
/// JSON: `{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier"},
/// "placement"?, "conditions"?}}`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[doc(alias = "spawn_simplified")]
pub struct SimplifiedSpawnRules {
    /// What the rules are for (JSON `description`, required).
    pub description: SimplifiedDescription,
    /// Where the entity is placed; absent leaves every placement to the runtime (JSON
    /// `placement`).
    pub placement: Option<Placement>,
    /// The conditions (JSON `conditions`).
    pub conditions: Option<Vec<BiomeConditions>>,
}

/// `minecraft:spawn_rules/description` of simplified spawn rules.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SimplifiedDescription {
    /// The entity the rules are for (JSON `identifier`, required).
    pub identifier: SpawnRulesId,
}

/// `placement`: four names from the placement registries, each `""` or absent for the runtime
/// default. An unregistered name is a warning and falls back to the same default.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Placement {
    /// The placement type: `minecraft:on_ground`, `minecraft:in_water`, `minecraft:in_lava`,
    /// `minecraft:no_restrictions` (the default) (JSON `type`).
    #[facet(rename = "type")]
    pub r#type: Option<String>,
    /// The heightmap: `minecraft:world_surface`, `minecraft:ocean_floor`,
    /// `minecraft:motion_blocking`, `minecraft:motion_blocking_no_leaves` (the default) (JSON
    /// `heightmap`).
    pub heightmap: Option<String>,
    /// The entity spawn predicate, `minecraft:mob` by default (JSON `spawn_predicate`).
    pub spawn_predicate: Option<String>,
    /// The obstruction predicate, `minecraft:mob` by default (JSON
    /// `spawn_obstruction_predicate`).
    pub spawn_obstruction_predicate: Option<String>,
}

/// One entry of `conditions` (vanilla `BiomeConditions`).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct BiomeConditions {
    /// The spawn weight, at least 0 (JSON `weight`, required).
    pub weight: i32,
    /// The herds; the runtime herd size is the first one's (JSON `herds`).
    pub herds: Option<Vec<SimplifiedHerd>>,
    /// Entities spawned in the entity's place (JSON `permute_types`).
    pub permute_types: Option<Vec<PermuteType>>,
    /// A mob event name that must be on (JSON `mob_event_filter`).
    pub mob_event_filter: Option<String>,
    /// The biomes the condition applies in; absent or empty = every biome (JSON `biome_filter`).
    pub biome_filter: Option<Filter>,
}

/// One entry of `herds`. No `min ≤ max` check.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct SimplifiedHerd {
    /// The smallest herd, at least 0; absent = 1 (JSON `min`).
    #[facet(schema::fallback = 1)]
    pub min: Option<i32>,
    /// The largest herd, at least 0; absent = 1 (JSON `max`).
    #[facet(schema::fallback = 1)]
    pub max: Option<i32>,
    /// An entity event sent to the first members (JSON `initial_event`).
    pub initial_event: Option<String>,
    /// How many members get `initial_event`; absent = 0 (JSON `initial_event_count`).
    pub initial_event_count: Option<i32>,
    /// An entity event sent to the herd's members (JSON `event`).
    pub event: Option<String>,
    /// How many members skip `event`; absent = 0 (JSON `event_skip_count`).
    pub event_skip_count: Option<i32>,
}

/// One entry of `permute_types` (vanilla `PermuteType`).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct PermuteType {
    /// The weight in the pick, at least 0 (JSON `weight`, required).
    pub weight: i32,
    /// How many are spawned regardless of the pick, at least 0; absent = 0 (JSON
    /// `min_guaranteed`).
    pub min_guaranteed: Option<i32>,
    /// The entity (JSON `entity_type`).
    pub entity_type: Option<EntityId>,
}

/// The one struct of `spawn_simplified` and its floor.
pub(crate) const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);

impl Versioned for SimplifiedSpawnRules {
    const VERSION: FormatVersion = V1_21_60;
}

impl VersionStruct for SimplifiedSpawnRules {
    /// The typed parse; unknown members are `UnrecognizedField` warnings, reported also when the
    /// typed parse fails (vanilla's cereal parser logs both).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
        // The spawn-rule loader registers the slice with the lenient error mask, under which an
        // unknown member is written to the content log at the warning level: the grade.
        let severity = cx.load.grade(Class::Unknown);
        let unknown = |path: &JsonPath, d: &mut Diagnostics| {
            finding(
                DiagnosticCode::UnrecognizedField { text: None },
                severity,
                &cx.at.join(path),
                d,
            );
        };
        // The scalars are read by the slice's reading (`crate::SIMPLIFIED_READING`).
        let read =
            parse_with_rules::<Self>(payload, None, crate::SIMPLIFIED_READING.scalar_rules());
        match read.map(|read| read.value) {
            Ok(value) => {
                if let Some(json) = &json {
                    let mut kept = bedrock_addon_core::document::Unknown::new();
                    let keeps = crate::SIMPLIFIED_READING.keeps_unknown;
                    for (path, _) in keep_unknown_members_with(&value, json, keeps, &mut kept) {
                        unknown(&path, d);
                    }
                    d.unknown_mut().absorb(&cx.at, kept);
                }
                Some(value)
            }
            Err(failure) => {
                if let Some(json) = &json {
                    for (path, _) in unknown_members(Self::SHAPE, json) {
                        unknown(&path, d);
                    }
                }
                d.push(match failure {
                    ScalarFailure::Read(error) => error.into_diagnostic(&cx.at, |code, _| code),
                    // A scalar no arm of an untagged enum takes: a type error at the enum.
                    ScalarFailure::Refused(misread) => misread.spanned_finding(&cx.at),
                });
                None
            }
        }
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

// ---- the placement registries ----------------------------------------

/// The built-in placement types.
pub(crate) const PLACEMENT_TYPES: &[&str] = &[
    "minecraft:on_ground",
    "minecraft:in_water",
    "minecraft:in_lava",
    "minecraft:no_restrictions",
];

/// The built-in heightmaps (`HeightmapType`).
pub(crate) const HEIGHTMAPS: &[&str] = &[
    "minecraft:world_surface",
    "minecraft:ocean_floor",
    "minecraft:motion_blocking",
    "minecraft:motion_blocking_no_leaves",
];

/// The built-in entity spawn predicates (37 names).
pub(crate) const ENTITY_PREDICATES: &[&str] = &[
    "minecraft:mob",
    "minecraft:monster",
    "minecraft:light_monster",
    "minecraft:patrolling_monster",
    "minecraft:axolotl",
    "minecraft:surface_water_animal",
    "minecraft:ageable_water_creature",
    "minecraft:drowned",
    "minecraft:guardian",
    "minecraft:tropical_fish",
    "minecraft:glow_squid",
    "minecraft:animal",
    "minecraft:armadillo",
    "minecraft:bat",
    "minecraft:fox",
    "minecraft:frog",
    "minecraft:goat",
    "minecraft:mooshroom",
    "minecraft:ocelot",
    "minecraft:parrot",
    "minecraft:polar_bear",
    "minecraft:rabbit",
    "minecraft:skeleton_horse",
    "minecraft:turtle",
    "minecraft:wolf",
    "minecraft:zombie_horse",
    "minecraft:endermite",
    "minecraft:husk",
    "minecraft:ghast",
    "minecraft:magma_cube",
    "minecraft:strider",
    "minecraft:hoglin",
    "minecraft:piglin",
    "minecraft:silverfish",
    "minecraft:slime",
    "minecraft:stray",
    "minecraft:zombified_piglin",
];

/// The built-in obstruction predicates (`ObstructionTypes`).
pub(crate) const OBSTRUCTION_PREDICATES: &[&str] = &[
    "minecraft:mob",
    "minecraft:unobstructed",
    "minecraft:iron_golem",
    "minecraft:ravager",
    "minecraft:warden",
];

impl Placement {
    /// The placement type the runtime uses: the written one when registered, else
    /// `minecraft:no_restrictions`.
    pub fn resolved_type(&self) -> &str {
        resolved(
            self.r#type.as_ref(),
            PLACEMENT_TYPES,
            "minecraft:no_restrictions",
        )
    }

    /// The heightmap the runtime uses: the written one when registered, else
    /// `minecraft:motion_blocking_no_leaves`.
    pub fn resolved_heightmap(&self) -> &str {
        resolved(
            self.heightmap.as_ref(),
            HEIGHTMAPS,
            "minecraft:motion_blocking_no_leaves",
        )
    }

    /// The entity spawn predicate the runtime uses: the written one when registered (built-ins
    /// only: a predicate registered by a script is not known here), else `minecraft:mob`.
    pub fn resolved_spawn_predicate(&self) -> &str {
        resolved(
            self.spawn_predicate.as_ref(),
            ENTITY_PREDICATES,
            "minecraft:mob",
        )
    }

    /// The obstruction predicate the runtime uses: the written one when registered, else
    /// `minecraft:mob`.
    pub fn resolved_spawn_obstruction_predicate(&self) -> &str {
        resolved(
            self.spawn_obstruction_predicate.as_ref(),
            OBSTRUCTION_PREDICATES,
            "minecraft:mob",
        )
    }
}

fn resolved<'a>(written: Option<&'a String>, names: &[&str], fallback: &'a str) -> &'a str {
    match written {
        Some(name) if names.contains(&name.as_str()) => name.as_str(),
        _ => fallback,
    }
}

/// The checks vanilla's cereal constraints and validator make, below `at` (the payload): a
/// negative weight, herd size or guaranteed count is an `InvalidValue` error (a constraint
/// violation, fatal under the lenient mask too); an unregistered placement name a warning with
/// vanilla's text, in vanilla's order.
pub(crate) fn validate(rules: &SimplifiedSpawnRules, at: &JsonPath, d: &mut Diagnostics) {
    let negative = |path: &JsonPath, d: &mut Diagnostics| {
        finding(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Error,
            path,
            d,
        );
    };
    let conditions_at = at.child("conditions");
    for (i, condition) in rules.conditions.iter().flatten().enumerate() {
        let c_at = conditions_at.index(i);
        if condition.weight < 0 {
            negative(&c_at.child("weight"), d);
        }
        for (k, herd) in condition.herds.iter().flatten().enumerate() {
            let h_at = c_at.child("herds").index(k);
            for (key, value) in [("min", herd.min), ("max", herd.max)] {
                if value.is_some_and(|n| n < 0) {
                    negative(&h_at.child(key), d);
                }
            }
        }
        for (j, permute) in condition.permute_types.iter().flatten().enumerate() {
            let p_at = c_at.child("permute_types").index(j);
            if permute.weight < 0 {
                negative(&p_at.child("weight"), d);
            }
            if permute.min_guaranteed.is_some_and(|n| n < 0) {
                negative(&p_at.child("min_guaranteed"), d);
            }
        }
    }
    // The placement validator, only for a present placement and non-empty members, in
    // vanilla's order: type, heightmap, obstruction, entity predicate.
    let Some(placement) = &rules.placement else {
        return;
    };
    let identifier = rules.description.identifier.as_str();
    let placement_at = at.child("placement");
    let checks: [(&str, Option<&String>, &[&str], &'static str); 4] = [
        (
            "type",
            placement.r#type.as_ref(),
            PLACEMENT_TYPES,
            texts::PLACEMENT_TYPE,
        ),
        (
            "heightmap",
            placement.heightmap.as_ref(),
            HEIGHTMAPS,
            texts::PLACEMENT_HEIGHTMAP,
        ),
        (
            "spawn_obstruction_predicate",
            placement.spawn_obstruction_predicate.as_ref(),
            OBSTRUCTION_PREDICATES,
            texts::PLACEMENT_OBSTRUCTION,
        ),
        (
            "spawn_predicate",
            placement.spawn_predicate.as_ref(),
            ENTITY_PREDICATES,
            texts::PLACEMENT_PREDICATE,
        ),
    ];
    for (key, written, names, template) in checks {
        let Some(name) = written.filter(|n| !n.is_empty()) else {
            continue;
        };
        if !names.contains(&name.as_str()) {
            let path = placement_at.child(key);
            // An `UnresolvedReference` warning with vanilla's text: the validator logs it at the
            // content log's warning level.
            finding(
                DiagnosticCode::UnresolvedReference {
                    text: Some(VanillaText::Formatted {
                        template,
                        args: vec![identifier.to_owned()],
                    }),
                },
                Severity::Warning,
                &path,
                d,
            );
        }
    }
}
