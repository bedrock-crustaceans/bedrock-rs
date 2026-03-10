use bedrockrs_macros::{ProtoCodec, gamepacket};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

#[gamepacket(id = 76)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AvailableCommandsPacket {
    pub enum_values: Vec<String>,
    pub sub_command_values: Vec<String>,
    pub post_fixes: Vec<String>,
    pub enum_data: Vec<EnumDataEntry>,
    pub chained_sub_command_data: Vec<ChainedSubCommandDataEntry>,
    pub commands: Vec<CommandsEntry>,
    pub soft_enums: Vec<SoftEnumsEntry>,
    pub constraints: Vec<ConstraintsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct EnumDataEntry {
    pub name: String,
    #[endianness(le)]
    pub values: Vec<u32>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubCommandValues {
    #[endianness(var)]
    pub index: u32,
    #[endianness(var)]
    pub value: u32,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ParameterDataEntry {
    pub name: String,
    #[endianness(le)]
    pub parse_symbol: u32,
    pub is_optional: bool,
    pub options: i8,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct OverloadsEntry {
    pub is_chaining: bool,
    pub parameter_data: Vec<ParameterDataEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct CommandsEntry {
    pub name: String,
    pub description: String,
    #[endianness(le)]
    pub flags: u16,
    pub permission_level: CommandPermissionLevelString,
    #[endianness(le)]
    pub alias_enum: i32,
    #[endianness(le)]
    pub chained_sub_command_indices: Vec<i32>,
    pub overloads: Vec<OverloadsEntry>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SoftEnumsEntry {
    pub enum_name: String,
    pub enum_options: Vec<String>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ConstraintsEntry {
    #[endianness(le)]
    pub enum_value_symbol: u32,
    #[endianness(le)]
    pub enum_symbol: u32,
    pub constraint_indices: Vec<i8>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ChainedSubCommandDataEntry {
    pub sub_command_name: String,
    pub sub_command_values: Vec<SubCommandValues>,
}

#[derive(Clone, Debug)]
#[repr(i8)]
pub enum CommandPermissionLevelString {
    Any = 0,
    GameDirectors = 1,
    Admin = 2,
    Host = 3,
    Owner = 4,
    Internal = 5,
}

impl ProtoCodec for CommandPermissionLevelString {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        String::proto_serialize(&String::from(self.clone()), stream)
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Self::try_from(String::proto_deserialize(stream)?)
    }

    fn get_size_prediction(&self) -> usize {
        String::from(self.clone()).get_size_prediction()
    }
}

impl TryFrom<String> for CommandPermissionLevelString {
    type Error = ProtoCodecError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "any" => Ok(CommandPermissionLevelString::Any),
            "gamedirectors" => Ok(CommandPermissionLevelString::GameDirectors),
            "admin" => Ok(CommandPermissionLevelString::Admin),
            "host" => Ok(CommandPermissionLevelString::Host),
            "owner" => Ok(CommandPermissionLevelString::Owner),
            "internal" => Ok(CommandPermissionLevelString::Internal),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                invalid.to_string(),
                "CommandPermissionLevel",
            )),
        }
    }
}

impl From<CommandPermissionLevelString> for String {
    fn from(value: CommandPermissionLevelString) -> Self {
        match value {
            CommandPermissionLevelString::Any => "any",
            CommandPermissionLevelString::GameDirectors => "gamedirectors",
            CommandPermissionLevelString::Admin => "admin",
            CommandPermissionLevelString::Host => "host",
            CommandPermissionLevelString::Owner => "owner",
            CommandPermissionLevelString::Internal => "internal",
        }
        .to_string()
    }
}
