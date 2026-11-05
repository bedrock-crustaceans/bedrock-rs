//! The generated tables as the default [`BuiltIns`] provider, with indexed lookups.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::{Command, CommandEnum, command_enums, commands, ids};
use crate::load::BuiltIns;

/// The identifier lists, commands and enchantment aliases the crate was generated with, and
/// the hand-kept lists of `bedrock_addon_vanilla`, as a [`BuiltIns`] provider: what answers
/// when the host gives none (`LoadContext::built_ins`). Every lookup is a hash lookup, built
/// once.
///
/// [`contains`](BuiltIns::contains) answers for the kinds with a list: `block`, `item`,
/// `entity`, `biome`, `dimension`, `camera preset`, `jigsaw structure` (the game's
/// structure features), `feature` (the features the game registers in code,
/// [`names::CODE_FEATURES`](super::names::CODE_FEATURES)), `level sound event`,
/// `aim assist category` and `aim assist preset`, each a `Kind::NAME`, and for the lists with
/// no identifier kind under the names `effect`, `enchantment`, `potion effect`, `potion type`
/// and `cooldown category`. Any other kind is not built in.
///
/// The legacy block and item names and the entities with no file are not in these lists: the
/// link pass checks them before it asks a provider (`LinkCx::built_in`), so a host adds to
/// them and never removes one. [`command`](Self::command) compares without ASCII case, like
/// function's `find_command`; the plain [`super::command`] compares exactly.
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
    /// Every `minecraft:` potion effect by its name without the namespace.
    potions: HashMap<&'static str, &'static str>,
    /// The names without `minecraft:` of every enchantment, alias and alias target.
    enchantments: HashSet<&'static str>,
    /// Every effect by its ASCII-lower-cased name without `minecraft:`.
    effects: HashMap<Box<str>, &'static str>,
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
            ("feature", &super::names::CODE_FEATURES),
            ("level sound event", super::sound_events::LEVEL_SOUND_EVENTS),
            ("aim assist category", &super::names::AIM_ASSIST_CATEGORIES),
            ("aim assist preset", &super::names::AIM_ASSIST_PRESETS),
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
        let potions = ids
            .potion_effects
            .iter()
            .filter_map(|id| Some((id.strip_prefix("minecraft:")?, *id)))
            .collect();
        let enchantments = ids
            .enchantments
            .iter()
            .chain(
                ids.enchantment_aliases
                    .iter()
                    .flat_map(|(alias, target)| [alias, target]),
            )
            .filter_map(|id| id.strip_prefix("minecraft:"))
            .collect();
        let mut effects = HashMap::new();
        for effect in ids.effects {
            let name = effect.trim_start_matches("minecraft:");
            effects
                .entry(name.to_ascii_lowercase().into_boxed_str())
                .or_insert(*effect);
        }
        Index {
            lists,
            commands: by_name,
            command_enums: enums,
            enchantment_aliases: aliases,
            potions,
            enchantments,
            effects,
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

    /// The table's spelling of the level sound event `name`, compared without ASCII case.
    pub fn level_sound_event(self, name: &str) -> Option<&'static str> {
        super::sound_events::level_sound_event(name)
    }

    /// The identifier of the potion called `name` (`Awkward` → `minecraft:awkward`): the name
    /// lower-cased (Unicode, as recipe's reader always has) behind `minecraft:`, if it is one
    /// of the potion effects.
    pub fn potion(self, name: &str) -> Option<&'static str> {
        index().potions.get(name.to_lowercase().as_str()).copied()
    }

    /// Whether `name`, without a namespace and as written, names an enchantment, an
    /// enchantment alias or an alias's target.
    pub fn is_enchantment(self, name: &str) -> bool {
        index().enchantments.contains(name)
    }

    /// Whether `name`, without a namespace, names an effect, compared without ASCII case.
    pub fn effect(self, name: &str) -> bool {
        folded(&index().effects, name).is_some()
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

    fn level_sound_event(&self, name: &str) -> Option<&str> {
        Self::level_sound_event(*self, name)
    }

    fn potion(&self, name: &str) -> Option<&str> {
        Self::potion(*self, name)
    }

    fn is_enchantment(&self, name: &str) -> bool {
        Self::is_enchantment(*self, name)
    }

    fn effect(&self, name: &str) -> bool {
        Self::effect(*self, name)
    }
}
