//! `metadata/command_modules/mojang-commands.json`: the vanilla commands, their argument lists
//! and the enums those draw values from. The command table generated
//! from it is [`bedrock_addon_core::vanilla::commands`].

use facet::Facet;

use crate::meta::MetaError;

/// The commands file.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct CommandsFile {
    /// `module_type`: `commands`.
    pub module_type: String,
    /// `name`: `mojang-commands`.
    pub name: String,
    /// `commands`.
    pub commands: Vec<Command>,
    /// `command_enums`.
    pub command_enums: Vec<CommandEnum>,
}

impl CommandsFile {
    /// Reads the file strictly: a key no field takes is an error.
    ///
    /// # Errors
    ///
    /// Returns a [`MetaError`] when the bytes are not UTF-8, are not JSON of the expected
    /// shape, nest deeper than the JSON depth limit, or hold a member no field takes.
    pub fn parse(bytes: &[u8]) -> Result<Self, MetaError> {
        crate::meta::parse::strict(bytes).map(|(file, _)| file)
    }
}

/// One command.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct Command {
    /// `name`: the command as typed, without `/`.
    pub name: String,
    /// `description`: the help text.
    pub description: String,
    /// `aliases`: other names of the command (`tp` for `teleport`).
    pub aliases: Vec<CommandAlias>,
    /// `permission_level`: 0 (any player) to 4.
    pub permission_level: u8,
    /// `requires_cheats`.
    pub requires_cheats: bool,
    /// `overloads`: the argument lists it accepts.
    pub overloads: Vec<Overload>,
}

/// One entry of `aliases`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct CommandAlias {
    /// `name`.
    pub name: String,
}

/// One argument list of a command.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct Overload {
    /// `name`: its number as a string (`"1"`, `"2"`, …).
    pub name: String,
    /// `params`.
    pub params: Vec<Param>,
}

/// One argument.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct Param {
    /// `name`: as the help text shows it; may contain spaces (`x angle`).
    pub name: String,
    /// `is_optional`.
    pub is_optional: bool,
    /// `type`.
    #[facet(rename = "type")]
    pub ty: ParamType,
}

/// `params[].type`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct ParamType {
    /// `name`: a command enum's name upper-cased (`AIMASSISTACTIONSET`) or a built-in token
    /// (`SELECTION`, `INT`, `POSITION`, `postfix_d`, …).
    pub name: String,
}

/// One entry of `command_enums`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct CommandEnum {
    /// `name`.
    pub name: String,
    /// `values`.
    pub values: Vec<EnumValue>,
}

/// One entry of `command_enums[].values`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct EnumValue {
    /// `value`.
    pub value: String,
}
