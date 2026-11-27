//! The game's own data, as plain tables: the identifier lists and the command table generated
//! from the game's script metadata (`cargo xtask script-meta`), and the hand-kept name lists
//! the metadata does not carry.
//!
//! The crate has no dependencies and no types beyond the table rows: every table is a
//! `&'static` array of `&'static str` (or a plain row struct of them), so a change to
//! `bedrock_addon_core` does not recompile the tables. Core reads them through its `BuiltIns`
//! provider (`bedrock_addon_core::vanilla::GeneratedBuiltIns`), which is what the families
//! ask. No family depends on this crate; the two readers that have no context to ask a
//! provider (`sounds.json`'s event names and entity's goal sounds) read [`sound_events`]
//! through core's re-export, `bedrock_addon_core::vanilla::sound_events`.
//!
//! Modules: [`names`], the hand-kept lists; [`sound_events`], the level sound events.
//! Features: none.

pub mod names;
pub mod sound_events;

// The generated files; `cargo xtask script-meta` writes them, nothing edits them.
mod generated {
    pub(crate) mod commands;
    pub(crate) mod vanilla_ids;
}

/// The vanilla identifier lists the tables were generated from (the game's `mojang-*.json`
/// script metadata): what cross-family lints compare against when no vanilla pack is loaded.
/// Each list is in the source file's order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct VanillaIds {
    /// `mojang-blocks.json`: block identifiers, and after them the blocks the game registers
    /// that the file leaves out (technical and retired blocks such as
    /// `minecraft:end_gateway`, which no script constant names).
    pub blocks: &'static [&'static str],
    /// `mojang-items.json`: item identifiers.
    pub items: &'static [&'static str],
    /// `mojang-entities.json`: entity identifiers.
    pub entities: &'static [&'static str],
    /// `mojang-biomes.json`: biome identifiers.
    pub biomes: &'static [&'static str],
    /// `mojang-effects.json`: effect identifiers.
    pub effects: &'static [&'static str],
    /// `mojang-enchantments.json`: enchantment identifiers.
    pub enchantments: &'static [&'static str],
    /// `mojang-enchantments.json` entries with a `value`: an alias and the enchantment it names
    /// (`minecraft:bow_infinity` → `minecraft:infinity`).
    pub enchantment_aliases: &'static [(&'static str, &'static str)],
    /// `mojang-dimensions.json`: dimension identifiers.
    pub dimensions: &'static [&'static str],
    /// `mojang-camera-presets.json`: camera preset identifiers.
    pub camera_presets: &'static [&'static str],
    /// `mojang-cooldown-category.json`: cooldown category identifiers.
    pub cooldown_categories: &'static [&'static str],
    /// `mojang-features.json`: structure feature identifiers.
    pub features: &'static [&'static str],
    /// `mojang-potion-effects.json`: potion effect identifiers.
    pub potion_effects: &'static [&'static str],
    /// `mojang-potion-types.json`: potion delivery types (`Consume`, `ThrownSplash`, …).
    pub potion_types: &'static [&'static str],
}

/// The vanilla identifier lists.
pub fn ids() -> &'static VanillaIds {
    &generated::vanilla_ids::IDS
}

/// One vanilla command (`command_modules/mojang-commands.json`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Command {
    /// `name`: the command as typed, without `/`.
    pub name: &'static str,
    /// `aliases[].name`: other names of the command (`tp` for `teleport`).
    pub aliases: &'static [&'static str],
    /// `permission_level`: 0 (any player) to 4.
    pub permission_level: u8,
    /// `requires_cheats`.
    pub requires_cheats: bool,
    /// `overloads`: the argument lists the command accepts, in order.
    pub overloads: &'static [Overload],
}

/// One argument list of a [`Command`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Overload {
    /// `params`, in order.
    pub params: &'static [Param],
}

/// One argument of an [`Overload`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Param {
    /// `name`, as the help text shows it (it may contain spaces: `x angle`).
    pub name: &'static str,
    /// `is_optional`.
    pub optional: bool,
    /// `type.name`: the upper-cased name of a [`CommandEnum`] or a built-in token (`INT`,
    /// `POSITION`, `SELECTION`, …).
    pub type_name: &'static str,
}

/// A named value set a command argument draws from (`command_enums[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CommandEnum {
    /// `name`.
    pub name: &'static str,
    /// `values[].value`, in order.
    pub values: &'static [&'static str],
}

/// Every vanilla command, sorted by name.
pub fn commands() -> &'static [Command] {
    generated::commands::COMMANDS
}

/// The command typed as `name`: its name or one of its aliases, without `/`.
pub fn command(name: &str) -> Option<&'static Command> {
    commands()
        .iter()
        .find(|command| command.name == name || command.aliases.contains(&name))
}

/// Every command enum, sorted by name.
pub fn command_enums() -> &'static [CommandEnum] {
    generated::commands::ENUMS
}

/// The command enum called `name`, compared without ASCII case (a parameter's `type.name` is
/// the enum's name upper-cased).
pub fn command_enum(name: &str) -> Option<&'static CommandEnum> {
    command_enums()
        .iter()
        .find(|command_enum| command_enum.name.eq_ignore_ascii_case(name))
}

/// The values a block state takes, by the state's type, in the metadata's order (the first is
/// the default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockStateValues {
    /// A `bool` state.
    Bool(&'static [bool]),
    /// An `int` state.
    Int(&'static [i64]),
    /// A `string` state.
    Str(&'static [&'static str]),
}

/// One block state (`mojang-blocks.json`, `block_properties`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BlockState {
    /// `name`: the state as saved (`weirdo_direction`, `minecraft:cardinal_direction`).
    pub name: &'static str,
    /// `values`: the values the state takes. The list is the state's, shared by every block that
    /// has it; a block may use fewer.
    pub values: BlockStateValues,
}

/// One block with its states (`mojang-blocks.json`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BlockStates {
    /// `name`: the identifier (`minecraft:oak_stairs`).
    pub name: &'static str,
    /// `raw_id`: the numeric id.
    pub raw_id: i32,
    /// `serialization_id`: the legacy name (`tile.*`); not unique.
    pub serialization_id: &'static str,
    /// `properties`: the states the block has, by name.
    pub states: &'static [BlockState],
}

/// The blocks of `mojang-blocks.json` with their states, sorted by identifier. The technical
/// blocks [`VanillaIds::blocks`] adds after the file's own are not here.
pub fn block_states() -> &'static [BlockStates] {
    generated::vanilla_ids::BLOCK_STATES
}

/// One block's states by its full identifier (`minecraft:…`), exactly.
pub fn block_state(name: &str) -> Option<&'static BlockStates> {
    let rows = block_states();
    rows.binary_search_by(|row| row.name.cmp(name))
        .ok()
        .map(|i| &rows[i])
}
