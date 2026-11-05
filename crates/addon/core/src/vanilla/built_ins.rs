//! The generated tables as the default [`BuiltIns`] provider, with indexed lookups.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::{Command, CommandEnum, command_enums, commands, ids};
use crate::load::BuiltIns;

/// The identifier lists, commands and enchantment aliases the crate was generated with, as a
/// [`BuiltIns`] provider: what answers when the host gives none
/// (`LoadContext::built_ins`). Every lookup is a hash lookup, built once.
///
/// [`contains`](BuiltIns::contains) answers for the kinds with a list: `block`, `item`,
/// `entity`, `biome`, `dimension`, `camera preset` and `jigsaw structure` (the game's
/// structure features), each a `Kind::NAME`, and for the lists with no identifier kind under
/// the names `effect`, `enchantment`, `potion effect`, `potion type` and `cooldown category`.
/// Any other kind is not built in.
///
/// Where it differs from what the link pass answers today:
///
/// - `contains("feature", ..)` is `false` for every id: the link pass answers the features the
///   game registers in code from addon's `CODE_FEATURES` (`addon/src/pack/link/lookup.rs`),
///   not from a provider.
/// - The legacy block and item names and the entities with no file are checked by the link
///   pass before it asks a provider (`LinkCx::built_in`, `lookup.rs:722-728`), so they are not
///   in these lists.
/// - [`command`](Self::command) compares without ASCII case, like function's `find_command`;
///   the plain [`super::command`] compares exactly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeneratedBuiltIns;

/// The tables, indexed.
struct Index {
    lists: HashMap<&'static str, HashSet<&'static str>>,
    /// Every command by its ASCII-lower-cased name and each alias; the first command in table
    /// order wins, as in function's `find_command`.
    commands: HashMap<Box<str>, &'static Command>,
    /// Every command enum by its ASCII-lower-cased name; the first in table order wins.
    command_enums: HashMap<Box<str>, &'static CommandEnum>,
    /// Every enchantment alias; the first in table order wins.
    enchantment_aliases: HashMap<&'static str, &'static str>,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let ids = ids();
        let lists = [
            ("block", ids.blocks),
            ("item", ids.items),
            ("entity", ids.entities),
            ("biome", ids.biomes),
            ("dimension", ids.dimensions),
            ("camera preset", ids.camera_presets),
            ("jigsaw structure", ids.features),
            ("effect", ids.effects),
            ("enchantment", ids.enchantments),
            ("potion effect", ids.potion_effects),
            ("potion type", ids.potion_types),
            ("cooldown category", ids.cooldown_categories),
        ]
        .into_iter()
        .map(|(kind, list)| (kind, list.iter().copied().collect()))
        .collect();
        let mut by_name = HashMap::new();
        for command in commands() {
            for name in std::iter::once(command.name).chain(command.aliases.iter().copied()) {
                by_name
                    .entry(name.to_ascii_lowercase().into_boxed_str())
                    .or_insert(command);
            }
        }
        let mut enums = HashMap::new();
        for command_enum in command_enums() {
            enums
                .entry(command_enum.name.to_ascii_lowercase().into_boxed_str())
                .or_insert(command_enum);
        }
        let mut aliases = HashMap::new();
        for (alias, target) in ids.enchantment_aliases {
            aliases.entry(*alias).or_insert(*target);
        }
        Index {
            lists,
            commands: by_name,
            command_enums: enums,
            enchantment_aliases: aliases,
        }
    })
}

/// The longest name [`folded`] lowers on the stack; a longer one is lowered on the heap.
const FOLD_BUFFER: usize = 64;

/// The value of `name` in a map keyed by ASCII-lower-cased names. A name with no ASCII
/// upper-case letter is looked up as it is; another is lowered first, on the stack when it fits.
fn folded<V: Copy>(map: &HashMap<Box<str>, V>, name: &str) -> Option<V> {
    if !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return map.get(name).copied();
    }
    let mut buffer = [0u8; FOLD_BUFFER];
    match buffer.get_mut(..name.len()) {
        Some(lowered) => {
            lowered.copy_from_slice(name.as_bytes());
            lowered.make_ascii_lowercase();
            // Lowering ASCII letters keeps the bytes UTF-8.
            std::str::from_utf8(lowered)
                .ok()
                .and_then(|lowered| map.get(lowered).copied())
        }
        None => map.get(name.to_ascii_lowercase().as_str()).copied(),
    }
}

impl GeneratedBuiltIns {
    /// The command typed as `name` (its name or an alias), compared without ASCII case, as
    /// function's `find_command` compares a line's first word: the first command in table
    /// order that answers. [`super::command`] compares exactly.
    pub fn command(self, name: &str) -> Option<&'static Command> {
        folded(&index().commands, name)
    }

    /// The command enum called `name`, compared without ASCII case: [`super::command_enum`],
    /// indexed.
    pub fn command_enum(self, name: &str) -> Option<&'static CommandEnum> {
        folded(&index().command_enums, name)
    }

    /// The enchantment the alias names (`minecraft:bow_infinity` → `minecraft:infinity`).
    pub fn enchantment_alias(self, alias: &str) -> Option<&'static str> {
        index().enchantment_aliases.get(alias).copied()
    }
}

impl BuiltIns for GeneratedBuiltIns {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        index()
            .lists
            .get(kind)
            .is_some_and(|list| list.contains(id))
    }

    fn command(&self, name: &str) -> Option<&Command> {
        Self::command(*self, name)
    }

    fn command_enum(&self, name: &str) -> Option<&CommandEnum> {
        Self::command_enum(*self, name)
    }

    fn enchantment_alias(&self, alias: &str) -> Option<&str> {
        Self::enchantment_alias(*self, alias)
    }
}
