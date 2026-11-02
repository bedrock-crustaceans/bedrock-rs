//! Simplified spawn rules (behind the Simplified Spawn Rules experiment): a cereal struct, read
//! typed from the payload; its numeric constraints and the placement-name validator.

use facet::Facet;

use super::texts;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{EntityId, SpawnRulesId};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, StrictInt, StrictString, Value, keep_unknown_members, parse_at,
    unknown_members,
};
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use bedrock_addon_support::spawn::read::finding;

/// Spawn rules in the simplified shape (format 1.21.60 and later, with the Simplified Spawn Rules
/// experiment on): where an entity spawns is a named placement, and each condition is a weight
/// with herds, permutations, a mob event and a biome filter.
///
/// JSON: `{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier"},
/// "placement"?, "conditions"?}}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[doc(alias = "spawn_simplified")]
pub struct SimplifiedSpawnRules {
    /// What the rules are for (JSON `description`, required).
    pub description: SimplifiedDescription,
    /// Where the entity is placed; absent leaves every placement to the runtime (JSON
    /// `placement`).
    #[facet(default)]
    pub placement: Option<Placement>,
    /// The conditions (JSON `conditions`).
    #[facet(default)]
    pub conditions: Option<Vec<BiomeConditions>>,
    /// Unknown members (each an `UnrecognizedField` warning).
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:spawn_rules/description` of simplified spawn rules.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SimplifiedDescription {
    /// The entity the rules are for (JSON `identifier`, required).
    pub identifier: SpawnRulesId,
    /// Unknown members (`population_control` is one in this shape).
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `placement`: four names from the placement registries, each `""` or absent for the runtime
/// default. An unregistered name is a warning and falls back to the same default.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Placement {
    /// The placement type: `minecraft:on_ground`, `minecraft:in_water`, `minecraft:in_lava`,
    /// `minecraft:no_restrictions` (the default) (JSON `type`).
    #[facet(default, rename = "type")]
    pub r#type: Option<StrictString>,
    /// The heightmap: `minecraft:world_surface`, `minecraft:ocean_floor`,
    /// `minecraft:motion_blocking`, `minecraft:motion_blocking_no_leaves` (the default) (JSON
    /// `heightmap`).
    #[facet(default)]
    pub heightmap: Option<StrictString>,
    /// The entity spawn predicate, `minecraft:mob` by default (JSON `spawn_predicate`).
    #[facet(default)]
    pub spawn_predicate: Option<StrictString>,
    /// The obstruction predicate, `minecraft:mob` by default (JSON
    /// `spawn_obstruction_predicate`).
    #[facet(default)]
    pub spawn_obstruction_predicate: Option<StrictString>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `conditions` (vanilla `BiomeConditions`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct BiomeConditions {
    /// The spawn weight, at least 0 (JSON `weight`, required).
    pub weight: StrictInt<i32>,
    /// The herds; the runtime herd size is the first one's (JSON `herds`).
    #[facet(default)]
    pub herds: Option<Vec<SimplifiedHerd>>,
    /// Entities spawned in the entity's place (JSON `permute_types`).
    #[facet(default)]
    pub permute_types: Option<Vec<PermuteType>>,
    /// A mob event name that must be on (JSON `mob_event_filter`).
    #[facet(default)]
    pub mob_event_filter: Option<StrictString>,
    /// The biomes the condition applies in; absent or empty = every biome (JSON `biome_filter`).
    #[facet(default)]
    pub biome_filter: Option<Filter>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `herds`. No `min ≤ max` check.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SimplifiedHerd {
    /// The smallest herd, at least 0; absent = 1 (JSON `min`).
    #[facet(default)]
    pub min: Option<StrictInt<i32>>,
    /// The largest herd, at least 0; absent = 1 (JSON `max`).
    #[facet(default)]
    pub max: Option<StrictInt<i32>>,
    /// An entity event sent to the first members (JSON `initial_event`).
    #[facet(default)]
    pub initial_event: Option<StrictString>,
    /// How many members get `initial_event`; absent = 0 (JSON `initial_event_count`).
    #[facet(default)]
    pub initial_event_count: Option<StrictInt<i32>>,
    /// An entity event sent to the herd's members (JSON `event`).
    #[facet(default)]
    pub event: Option<StrictString>,
    /// How many members skip `event`; absent = 0 (JSON `event_skip_count`).
    #[facet(default)]
    pub event_skip_count: Option<StrictInt<i32>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of `permute_types` (vanilla `PermuteType`).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PermuteType {
    /// The weight in the pick, at least 0 (JSON `weight`, required).
    pub weight: StrictInt<i32>,
    /// How many are spawned regardless of the pick, at least 0; absent = 0 (JSON
    /// `min_guaranteed`).
    #[facet(default)]
    pub min_guaranteed: Option<StrictInt<i32>>,
    /// The entity (JSON `entity_type`).
    #[facet(default)]
    pub entity_type: Option<EntityId>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The one struct of `spawn_simplified` and its floor.
pub(crate) const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);

impl Versioned for SimplifiedSpawnRules {
    const VERSION: FormatVersion = V1_21_60;
}

impl bedrock_addon_core::molang::MolangFields for SimplifiedSpawnRules {
    /// No Molang member.
    fn visit_molang(&mut self, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}
}

/// `at` followed by `relative`.
fn join(at: &JsonPath, relative: &JsonPath) -> JsonPath {
    let mut path = at.clone();
    path.append(relative);
    path
}

impl VersionStruct for SimplifiedSpawnRules {
    /// The typed parse; unknown members are `UnrecognizedField` warnings, reported also when the
    /// typed parse fails (vanilla's cereal parser logs both).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
        // The spawn-rule loader registers the slice with the lenient error mask, under which an
        // unknown member is written to the content log at the warning level.
        let unknown = |path: &JsonPath, d: &mut Diagnostics| {
            finding(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Warning,
                &join(&cx.at, path),
                d,
            );
        };
        match parse_at::<Self>(payload) {
            Ok(mut value) => {
                if let Some(json) = &json {
                    for (path, _) in keep_unknown_members(&mut value, json) {
                        unknown(&path, d);
                    }
                }
                Some(value)
            }
            Err(error) => {
                if let Some(json) = &json {
                    for (path, _) in unknown_members(Self::SHAPE, json) {
                        unknown(&path, d);
                    }
                }
                let blank = payload.text.trim().is_empty();
                let code = code_for_deserialize_error_kind(
                    &error.error.kind,
                    !error.path.is_empty(),
                    blank,
                );
                d.push(Diagnostic::new(
                    code,
                    Severity::Error,
                    Phase::Parse,
                    Location::new(None, None, join(&cx.at, &error.path), Some(error.span)),
                ));
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

fn resolved<'a>(written: Option<&'a StrictString>, names: &[&str], fallback: &'a str) -> &'a str {
    match written {
        Some(name) if names.contains(&name.0.as_str()) => name.0.as_str(),
        _ => fallback,
    }
}

impl SimplifiedHerd {
    /// The smallest herd (1 when absent).
    pub fn min_size(&self) -> i32 {
        self.min.map_or(1, |n| n.0)
    }

    /// The largest herd (1 when absent).
    pub fn max_size(&self) -> i32 {
        self.max.map_or(1, |n| n.0)
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
    let mut conditions_at = at.clone();
    conditions_at.push("conditions");
    for (i, condition) in rules.conditions.iter().flatten().enumerate() {
        let mut c_at = conditions_at.clone();
        c_at.push_index(i);
        let member = |base: &JsonPath, key: &str| {
            let mut path = base.clone();
            path.push(key);
            path
        };
        if condition.weight.0 < 0 {
            negative(&member(&c_at, "weight"), d);
        }
        for (k, herd) in condition.herds.iter().flatten().enumerate() {
            let mut h_at = member(&c_at, "herds");
            h_at.push_index(k);
            for (key, value) in [("min", herd.min), ("max", herd.max)] {
                if value.is_some_and(|n| n.0 < 0) {
                    negative(&member(&h_at, key), d);
                }
            }
        }
        for (j, permute) in condition.permute_types.iter().flatten().enumerate() {
            let mut p_at = member(&c_at, "permute_types");
            p_at.push_index(j);
            if permute.weight.0 < 0 {
                negative(&member(&p_at, "weight"), d);
            }
            if permute.min_guaranteed.is_some_and(|n| n.0 < 0) {
                negative(&member(&p_at, "min_guaranteed"), d);
            }
        }
    }
    // The placement validator, only for a present placement and non-empty members, in
    // vanilla's order: type, heightmap, obstruction, entity predicate.
    let Some(placement) = &rules.placement else {
        return;
    };
    let identifier = rules.description.identifier.as_str();
    let mut placement_at = at.clone();
    placement_at.push("placement");
    let checks: [(&str, Option<&StrictString>, &[&str], &'static str); 4] = [
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
        let Some(name) = written.filter(|n| !n.0.is_empty()) else {
            continue;
        };
        if !names.contains(&name.0.as_str()) {
            let mut path = placement_at.clone();
            path.push(key);
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
