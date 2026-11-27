//! The two tables compiled into the engine,
//! in vanilla's build order, and the version each entry first shipped in.
//!
//! A newer engine that adds an entry changes these constants; nothing else in the module
//! knows a pattern or a material by name.

use std::sync::OnceLock;

use super::{FormatCode, TrimData, TrimMaterial, TrimPattern};
use bedrock_addon_core::id::{ItemId, TrimMaterialId, TrimPatternId};
use bedrock_addon_core::version::SemVersion;

// The versioned vanilla packs that first ship an entry's assets.
const TRAILS_AND_TALES: SemVersion = SemVersion::new(1, 20, 0);
const V1_21_0: SemVersion = SemVersion::new(1, 21, 0);
const V1_21_50: SemVersion = SemVersion::new(1, 21, 50);

/// `(pattern id, template item, first shipped)`.
// 18 pairs, in the game's build order.
const PATTERNS: [(&str, &str, SemVersion); 18] = [
    (
        "ward",
        "minecraft:ward_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "snout",
        "minecraft:snout_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "dune",
        "minecraft:dune_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "spire",
        "minecraft:spire_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "wild",
        "minecraft:wild_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "rib",
        "minecraft:rib_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "tide",
        "minecraft:tide_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "sentry",
        "minecraft:sentry_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "coast",
        "minecraft:coast_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "eye",
        "minecraft:eye_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "vex",
        "minecraft:vex_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "silence",
        "minecraft:silence_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "wayfinder",
        "minecraft:wayfinder_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "raiser",
        "minecraft:raiser_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "shaper",
        "minecraft:shaper_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "host",
        "minecraft:host_armor_trim_smithing_template",
        TRAILS_AND_TALES,
    ),
    (
        "flow",
        "minecraft:flow_armor_trim_smithing_template",
        V1_21_0,
    ),
    (
        "bolt",
        "minecraft:bolt_armor_trim_smithing_template",
        V1_21_0,
    ),
];

/// `(material id, colour, ingredient item, first shipped)`.
// 11 entries, in the game's build order; each colour is the formatting code of the
// material's hover text.
const MATERIALS: [(&str, &str, &str, SemVersion); 11] = [
    ("quartz", "§h", "minecraft:quartz", TRAILS_AND_TALES),
    ("iron", "§i", "minecraft:iron_ingot", TRAILS_AND_TALES),
    (
        "netherite",
        "§j",
        "minecraft:netherite_ingot",
        TRAILS_AND_TALES,
    ),
    ("redstone", "§m", "minecraft:redstone", TRAILS_AND_TALES),
    ("copper", "§n", "minecraft:copper_ingot", TRAILS_AND_TALES),
    ("gold", "§p", "minecraft:gold_ingot", TRAILS_AND_TALES),
    ("emerald", "§q", "minecraft:emerald", TRAILS_AND_TALES),
    ("diamond", "§s", "minecraft:diamond", TRAILS_AND_TALES),
    ("lapis", "§t", "minecraft:lapis_lazuli", TRAILS_AND_TALES),
    (
        "amethyst",
        "§u",
        "minecraft:amethyst_shard",
        TRAILS_AND_TALES,
    ),
    ("resin", "§v", "minecraft:resin_brick", V1_21_50),
];

/// The engine's table.
pub(super) fn vanilla() -> &'static TrimData {
    static VANILLA: OnceLock<TrimData> = OnceLock::new();
    VANILLA.get_or_init(|| {
        let mut data = TrimData::default();
        for (pattern, item, _) in PATTERNS {
            data.insert_pattern(TrimPattern {
                pattern_id: TrimPatternId::from_static(pattern),
                item_name: ItemId::from_static(item),
            });
        }
        data.materials = MATERIALS
            .iter()
            .map(|(material, color, item, _)| TrimMaterial {
                material_id: TrimMaterialId::from_static(material),
                color: FormatCode::new(*color),
                item_name: ItemId::from_static(item),
            })
            .collect();
        data
    })
}

/// When the vanilla packs first ship the pattern `id`.
pub(super) fn pattern_since(id: &str) -> Option<SemVersion> {
    PATTERNS
        .iter()
        .find(|(pattern, ..)| *pattern == id)
        .map(|(.., since)| since.clone())
}

/// When the vanilla packs first ship the material `id`.
pub(super) fn material_since(id: &str) -> Option<SemVersion> {
    MATERIALS
        .iter()
        .find(|(material, ..)| *material == id)
        .map(|(.., since)| since.clone())
}
