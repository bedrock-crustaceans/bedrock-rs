//! The command overloads vanilla registers for part of the command-version range only: which
//! syntax a line may use depends on its pack's command version.

use bedrock_addon_core::version::CURRENT_COMMAND_VERSION;

/// A group of overloads of one command that vanilla registers for a range of command versions
/// only. A line that uses the syntax under a command version outside the range fails to parse.
///
/// Every overload not listed by [`grammar_ranges`] is registered for every command version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct GrammarRange {
    /// The command, as typed.
    pub command: &'static str,
    /// The lowest command version that sees the overloads.
    pub from: u8,
    /// The highest command version that sees them; `None` when vanilla sets no upper bound.
    pub to: Option<u8>,
    /// How many overloads the range covers.
    pub overloads: u8,
    /// What the overloads are, in the command's own syntax.
    pub syntax: &'static str,
}

impl GrammarRange {
    /// Whether a line under `command_version` sees the overloads.
    // The game's parser shows an overload when `from <= command_version <= to`.
    #[must_use]
    pub fn contains(&self, command_version: u8) -> bool {
        self.from <= command_version && self.to.is_none_or(|to| command_version <= to)
    }
}

/// One row of the table.
const fn range(
    command: &'static str,
    from: u8,
    to: Option<u8>,
    overloads: u8,
    syntax: &'static str,
) -> GrammarRange {
    GrammarRange {
        command,
        from,
        to,
        overloads,
        syntax,
    }
}

/// The current command version as an upper bound: vanilla takes the upper bound of these
/// ranges from the current command version.
const CURRENT: Option<u8> = Some(CURRENT_COMMAND_VERSION);

// Every overload group the game registers for a range of command versions other than "every
// version". What each range covers is read from the names of the overload's parameters.
static RANGES: [GrammarRange; 18] = [
    range(
        "execute",
        1,
        Some(24),
        1,
        "execute <target> <position> [detect …] <command>",
    ),
    range(
        "execute",
        25,
        CURRENT,
        1,
        "execute <as|at|in|positioned|rotated|facing|align|anchored|if|unless|run> …",
    ),
    range(
        "execute",
        25,
        Some(28),
        1,
        "execute … <if|unless> block <position> <block> <data> …",
    ),
    range(
        "setblock",
        1,
        Some(28),
        1,
        "setblock <position> <tileName> <tileData> [mode]",
    ),
    range(
        "fill",
        1,
        Some(28),
        2,
        "fill <from> <to> <tileName> <tileData> [mode], fill … replace <replaceTileName> <replaceDataValue>",
    ),
    range(
        "clone",
        1,
        Some(28),
        1,
        "clone <begin> <end> <destination> filtered <mode> <tileName> <tileData>",
    ),
    range(
        "testforblock",
        1,
        Some(28),
        1,
        "testforblock <position> <tileName> <dataValue>",
    ),
    range(
        "locate",
        1,
        Some(20),
        1,
        "locate <structure> [useNewChunksOnly]",
    ),
    range("locate", 21, None, 2, "locate structure …, locate biome …"),
    range(
        "structure",
        1,
        Some(21),
        4,
        "structure load … (without waterlogged)",
    ),
    range(
        "structure",
        22,
        CURRENT,
        3,
        "structure load … [waterlogged] …",
    ),
    range(
        "summon",
        1,
        Some(30),
        1,
        "summon <entityType> [spawnPos] [spawnEvent] [nameTag]",
    ),
    range(
        "summon",
        31,
        None,
        3,
        "summon … [yRot] [xRot] …, summon … facing <lookAtPosition|lookAtEntity> …",
    ),
    range(
        "summon_noai",
        31,
        None,
        5,
        "summon_noai … (the rotation and facing forms)",
    ),
    range("damage", 1, Some(37), 4, "damage … (causes with suicide)"),
    range(
        "damage",
        38,
        CURRENT,
        3,
        "damage … (causes without suicide)",
    ),
    range("locate", 44, None, 2, "locate poi …"),
    range("agent", 1, CURRENT, 4, "agent …"),
];

/// Every range of command versions vanilla registers a command's overloads for, other than
/// "every version".
#[must_use]
pub fn grammar_ranges() -> &'static [GrammarRange] {
    &RANGES
}
