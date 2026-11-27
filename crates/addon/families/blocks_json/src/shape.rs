//! `blockshape`: the 90 names the client knows, the shape a vanilla block starts with, and the
//! three conversions the client refuses.

use super::tables::{DEFAULT_SHAPES, LEGACY_ALIASES};

/// The names `blockshape` takes, each with vanilla's shape number, in the client's own
/// order. Three pairs of names share a number (`barrier` / `camera_facing_sprite`, `azalea` /
/// `flowering_azalea`, `glow_lichen` / `sculk_vein`).
pub const BLOCK_SHAPES: &[(&str, i16)] = &[
    ("invisible", -1),
    ("block", 0),
    ("cross_texture", 1),
    ("torch", 2),
    ("fire", 3),
    ("water", 4),
    ("red_dust", 5),
    ("rows", 6),
    ("door", 7),
    ("ladder", 8),
    ("rail", 9),
    ("stairs", 10),
    ("fence", 11),
    ("lever", 12),
    ("cactus", 13),
    ("bed", 14),
    ("diode", 15),
    ("iron_fence", 18),
    ("stem", 19),
    ("vine", 20),
    ("fence_gate", 21),
    ("chest", 22),
    ("lilypad", 23),
    ("brewing_stand", 25),
    ("portal_frame", 26),
    ("cocoa", 28),
    ("tree", 31),
    ("cobblestone_wall", 32),
    ("double_plant", 40),
    ("flower_pot", 42),
    ("anvil", 43),
    ("kelp", 98),
    ("coral_fan", 96),
    ("block_half", 67),
    ("trapdoor", 99),
    ("top_snow", 68),
    ("tripwire", 69),
    ("tripwire_hook", 70),
    ("cauldron", 71),
    ("repeater", 72),
    ("comparator", 73),
    ("hopper", 74),
    ("slime_block", 75),
    ("piston", 76),
    ("beacon", 77),
    ("chorus_plant", 78),
    ("chorus_flower", 79),
    ("dragon_egg", 44),
    ("end_portal", 80),
    ("end_rod", 81),
    ("structure_void", 48),
    ("skull", 83),
    ("facing_block", 84),
    ("command_block", 85),
    ("terracotta", 86),
    ("double_side_fence", 87),
    ("frame", 88),
    ("glow_frame", 133),
    ("shulker_box", 89),
    ("doublesided_cross_texture", 90),
    ("doublesided_double_plant", 91),
    ("doublesided_rows", 92),
    ("element_block", 93),
    ("chemistry_table", 94),
    ("seagrass", 97),
    ("sea_pickle", 100),
    ("conduit", 101),
    ("bubble_column", 105),
    ("turtle_egg", 102),
    ("barrier", 106),
    ("camera_facing_sprite", 106),
    ("sign", 107),
    ("bamboo", 108),
    ("bamboo_sapling", 109),
    ("scaffolding", 110),
    ("grindstone", 111),
    ("bell", 112),
    ("lantern", 113),
    ("campfire", 114),
    ("lectern", 115),
    ("sweet_berry_bush", 116),
    ("cartography_table", 117),
    ("stonecutter_block", 119),
    ("chain", 123),
    ("sculk_sensor", 125),
    ("azalea", 132),
    ("flowering_azalea", 132),
    ("glow_lichen", 134),
    ("sculk_vein", 134),
    ("sculk_shrieker", 138),
];

/// The shape number of a `blockshape` name; `None` for a name the client does not know.
/// Names compare as written.
#[must_use]
pub fn block_shape_value(name: &str) -> Option<i16> {
    static BY_NAME: std::sync::LazyLock<std::collections::HashMap<&'static str, i16>> =
        std::sync::LazyLock::new(|| BLOCK_SHAPES.iter().copied().collect());
    BY_NAME.get(name).copied()
}

/// The conversions the client refuses, `(current, requested)`: `terracotta` to `tree`, `block`
/// to `tree`, `shulker_box` to `command_block`. The client looks the refusal up by the block's
/// current shape.
pub(super) const REFUSED_CONVERSIONS: [(i16, i16); 3] = [(86, 31), (0, 31), (89, 85)];

/// Whether the client refuses to change a block's shape from `current` to `requested`.
pub(super) fn is_refused(current: i16, requested: i16) -> bool {
    REFUSED_CONVERSIONS.contains(&(current, requested))
}

/// The groups of blocks the client registers with one shape, as far as their members follow
/// from a name: a suffix of the block's name and the shape. The client fills the groups at run
/// time, so which block is in which group is inferred from the group's name. No vanilla file
/// carries a `blockshape`, so these shapes cannot be checked against the game's data.
const SUFFIX_GROUPS: &[(&str, i16)] = &[
    ("_stained_glass_pane", 87),
    ("_shulker_box", 89),
    ("lightning_rod", 127),
    ("candle_cake", 137),
    ("candle", 136),
    ("copper_chest", 22),
    ("_shelf", 158),
    ("copper_golem_statue", 145),
    ("copper_bars", 18),
    ("copper_chain", 123),
    ("copper_lantern", 113),
    ("_sapling", 155),
];

/// The legacy names whose post-split blocks form one of those groups, and the shape. Like
/// `SUFFIX_GROUPS`, these shapes cannot be checked against the game's data.
const ALIAS_GROUPS: &[(&str, i16)] = &[
    ("sapling", 155),
    ("chemistry_table", 94),
    ("colored_torch_rg", 2),
    ("colored_torch_bp", 2),
    ("stone_block_slab", 67),
    ("stone_block_slab2", 67),
    ("stone_block_slab3", 67),
    ("stone_block_slab4", 67),
    ("wooden_slab", 67),
    ("shulker_box", 89),
    ("cobblestone_wall", 32),
    ("anvil", 43),
    ("skull", 83),
    ("double_plant", 91),
    ("coral_fan", 96),
    ("coral_fan_dead", 96),
    ("coral_fan_hang", 96),
    ("coral_fan_hang2", 96),
    ("coral_fan_hang3", 96),
    ("light_block", 106),
    ("wood", 31),
];

/// The shape a vanilla block has before any `blocks.json` entry is applied (the shape number
/// the vanilla game uses): the one the client registers the block with, 0 (`block`) for a block it
/// registers without one. `name` is the block's name without the `minecraft:` namespace.
///
/// The client refuses `"blockshape": "tree"` on a block whose shape is 0 or 86, and
/// `"command_block"` on one whose shape is 89.
///
/// Every value this function returns is unchecked against vanilla's data, since no vanilla file
/// carries a `blockshape` to check it with. The blocks the client registers through a group
/// (the stained glass panes, the shulker boxes, the candles, the slabs, the saplings and the
/// like) are matched by name, which is a second inference; for 12 of the 470 blocks registered
/// one by one, the name itself is a further inference, since no vanilla `blocks.json` has an
/// entry of that name.
#[must_use]
pub fn default_block_shape(name: &str) -> i16 {
    static BY_NAME: std::sync::LazyLock<std::collections::HashMap<&'static str, i16>> =
        std::sync::LazyLock::new(|| {
            let mut shapes = std::collections::HashMap::new();
            for (alias, shape) in ALIAS_GROUPS {
                let names = LEGACY_ALIASES
                    .iter()
                    .find(|entry| entry.name == *alias)
                    .map_or(&[][..], |entry| entry.post_split);
                for (_, name) in names {
                    shapes.insert(*name, *shape);
                }
            }
            // A call site that names one block is the block's own registration.
            shapes.extend(DEFAULT_SHAPES.iter().copied());
            shapes
        });
    if let Some(shape) = BY_NAME.get(name) {
        return *shape;
    }
    // The chemistry elements: `element_0` … `element_118`.
    if name
        .strip_prefix("element_")
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
    {
        return 93;
    }
    SUFFIX_GROUPS
        .iter()
        .find(|(suffix, _)| name.ends_with(suffix))
        .map_or(0, |(_, shape)| *shape)
}
