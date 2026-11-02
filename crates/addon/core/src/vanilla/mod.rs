//! Generated vanilla tables: the identifier lists and command table (`cargo xtask script-meta`)
//! and the actor types (`cargo xtask actor-types`). Always built; the `function` and `camera`
//! lints and `script::meta` read them from here.

// The generated files; `cargo xtask` writes them, nothing edits them.
mod generated {
    pub(super) mod actor_type;
    pub(super) mod commands;
    pub(super) mod vanilla_ids;
}

pub use generated::actor_type::ActorType;

impl ActorType {
    /// Vanilla's numeric value of the type (the game's `ActorType` enumeration).
    #[must_use]
    pub const fn id(self) -> u32 {
        self as u32
    }

    /// The type a vanilla entity identifier names (as the game's entity-type lookup
    /// resolves it), or `None` when it names no vanilla entity. The `minecraft:` namespace may be
    /// left out (`"pig"` is `minecraft:pig`); the comparison is case-sensitive.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Option<Self> {
        // The game adds the `minecraft:` namespace to a type's name only on request, so the name
        // table is bare and the namespace is optional here; any other namespace names no vanilla
        // entity.
        let name = match identifier.split_once(':') {
            Some(("minecraft", name)) => name,
            Some(_) => return None,
            None => identifier,
        };
        Self::from_vanilla_name(name)
    }

    /// The type vanilla gives an entity document: its identifier's type; for a custom
    /// identifier, the type its `runtime_identifier` names; and `Mob` when neither names a
    /// vanilla entity, never "no type".
    #[must_use]
    pub fn for_entity(identifier: &str, runtime_identifier: Option<&str>) -> Self {
        // The game looks the identifier up first, then the runtime identifier, and gives a custom
        // entity `Mob` (0x100).
        Self::from_identifier(identifier)
            .or_else(|| runtime_identifier.and_then(Self::from_identifier))
            .unwrap_or(Self::Mob)
    }

    /// The vanilla identifier of the type (`minecraft:pig`); `None` for `Mob`.
    #[must_use]
    pub fn identifier(self) -> Option<&'static str> {
        self.vanilla_identifier()
    }
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
#[must_use]
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
#[must_use]
pub fn commands() -> &'static [Command] {
    generated::commands::COMMANDS
}

/// The command typed as `name`: its name or one of its aliases, without `/`.
#[must_use]
pub fn command(name: &str) -> Option<&'static Command> {
    commands()
        .iter()
        .find(|command| command.name == name || command.aliases.contains(&name))
}

/// Every command enum, sorted by name.
#[must_use]
pub fn command_enums() -> &'static [CommandEnum] {
    generated::commands::ENUMS
}

/// The command enum called `name`, compared without ASCII case (a parameter's `type.name` is
/// the enum's name upper-cased).
#[must_use]
pub fn command_enum(name: &str) -> Option<&'static CommandEnum> {
    command_enums()
        .iter()
        .find(|command_enum| command_enum.name.eq_ignore_ascii_case(name))
}
