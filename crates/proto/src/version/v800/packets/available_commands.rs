use super::super::enums::CommandPermissionLevel;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::Cursor;
use std::mem::size_of;

#[gamepacket(id = 76)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AvailableCommandsPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub enum_values: Vec<String>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub post_fixes: Vec<String>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub enum_data: Vec<EnumDataEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub chained_sub_command_data: Vec<ChainedSubCommandDataEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub commands: Vec<CommandsEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub soft_enums: Vec<SoftEnumsEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub constraints: Vec<ConstraintsEntry>,
}

#[derive(Clone, Debug)]
pub struct EnumDataEntry {
    name: String,
    values: Vec<u32>,
}

impl ProtoCodec for EnumDataEntry {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <String as ProtoCodec>::proto_serialize(&self.name, stream)?;
        {
            let len: u32 = self.values.len().try_into()?;
            <u32 as ProtoCodecVAR>::proto_serialize(&len, stream)?;
            for i in &self.values {
                <u32 as ProtoCodecVAR>::proto_serialize(i, stream)?;
                // VERIFY: If this varint works
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let name = <String as ProtoCodec>::proto_deserialize(stream)?;
        let values = {
            let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<u32 as ProtoCodecVAR>::proto_deserialize(stream)?);
            }
            vec
        };

        Ok(Self { name, values })
    }

    fn get_size_prediction(&self) -> usize {
        self.name.get_size_prediction() + self.values.len() * size_of::<u32>()
    }
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubCommandValues {
    #[endianness(le)]
    pub sub_command_first_value: u16,
    #[endianness(le)]
    pub sub_command_second_value: u16,
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
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub parameter_data: Vec<ParameterDataEntry>,
}

#[derive(Clone, Debug)]
pub struct CommandsEntry {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission_level: CommandPermissionLevel,
    pub alias_enum: i32,
    pub chained_sub_command_indices: Vec<u16>,
    pub overloads: Vec<OverloadsEntry>,
}

impl ProtoCodec for CommandsEntry {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <String as ProtoCodec>::proto_serialize(&self.name, stream)?;
        <String as ProtoCodec>::proto_serialize(&self.description, stream)?;
        <u16 as ProtoCodecLE>::proto_serialize(&self.flags, stream)?;
        <CommandPermissionLevel as ProtoCodec>::proto_serialize(&self.permission_level, stream)?;
        <i32 as ProtoCodecLE>::proto_serialize(&self.alias_enum, stream)?;
        {
            let len: u32 = self.chained_sub_command_indices.len().try_into()?;
            <u32 as ProtoCodecVAR>::proto_serialize(&len, stream)?;
            for i in &self.chained_sub_command_indices {
                <u16 as ProtoCodecLE>::proto_serialize(i, stream)?;
            }
        }
        {
            let len: u32 = self.overloads.len().try_into()?;
            <u32 as ProtoCodecVAR>::proto_serialize(&len, stream)?;
            for i in &self.overloads {
                <OverloadsEntry as ProtoCodec>::proto_serialize(i, stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let name = <String as ProtoCodec>::proto_deserialize(stream)?;
        let description = <String as ProtoCodec>::proto_deserialize(stream)?;
        let flags = <u16 as ProtoCodecLE>::proto_deserialize(stream)?;
        let permission_level = <CommandPermissionLevel as ProtoCodec>::proto_deserialize(stream)?;
        let alias_enum = <i32 as ProtoCodecLE>::proto_deserialize(stream)?;
        let chained_sub_command_indices = {
            let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<u16 as ProtoCodecLE>::proto_deserialize(stream)?);
            }
            vec
        };
        let overloads = {
            let len = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<OverloadsEntry as ProtoCodec>::proto_deserialize(stream)?);
            }
            vec
        };

        Ok(Self {
            name,
            description,
            flags,
            permission_level,
            alias_enum,
            chained_sub_command_indices,
            overloads,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.name.get_size_prediction()
            + self.description.get_size_prediction()
            + size_of::<u32>()
            + self.chained_sub_command_indices.len() * size_of::<u16>()
            + size_of::<u32>()
            + self
            .overloads
            .iter()
            .map(|i| i.get_size_prediction())
            .sum::<usize>()
    }
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SoftEnumsEntry {
    pub enum_name: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub enum_options: Vec<String>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ConstraintsEntry {
    #[endianness(le)]
    pub enum_value_symbol: u32,
    #[endianness(le)]
    pub enum_symbol: u32,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub constraint_indices: Vec<i8>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ChainedSubCommandDataEntry {
    pub sub_command_name: String,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub sub_command_values: Vec<SubCommandValues>,
}

// VERIFY: ProtoCodec impl
