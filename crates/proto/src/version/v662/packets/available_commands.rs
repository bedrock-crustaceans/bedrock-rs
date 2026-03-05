use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Cursor, Read, Write};
use std::mem::size_of;

#[gamepacket(id = 76)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AvailableCommandsPacket<V: ProtoVersion> {
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
    pub commands: Vec<CommandsEntry<V>>,
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
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <String as ProtoCodec>::serialize(&self.name, stream)?;
        {
            let len: u32 = self.values.len().try_into()?;
            <u32 as ProtoCodecVAR>::serialize(&len, stream)?;
            for i in &self.values {
                <u32 as ProtoCodecVAR>::serialize(i, stream)?;
                // VERIFY: If this varint works
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let name = <String as ProtoCodec>::deserialize(stream)?;
        let values = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<u32 as ProtoCodecVAR>::deserialize(stream)?);
            }
            vec
        };

        Ok(Self { name, values })
    }

    fn size_hint(&self) -> usize {
        self.name.size_hint() + self.values.len() * size_of::<u32>()
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
pub struct CommandsEntry<V: ProtoVersion> {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission_level: V::CommandPermissionLevel,
    pub alias_enum: i32,
    pub chained_sub_command_indices: Vec<u16>,
    pub overloads: Vec<OverloadsEntry>,
}

impl<V: ProtoVersion> ProtoCodec for CommandsEntry<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <String as ProtoCodec>::serialize(&self.name, stream)?;
        <String as ProtoCodec>::serialize(&self.description, stream)?;
        <u16 as ProtoCodecLE>::serialize(&self.flags, stream)?;
        <V::CommandPermissionLevel as ProtoCodec>::serialize(&self.permission_level, stream)?;
        <i32 as ProtoCodecLE>::serialize(&self.alias_enum, stream)?;
        {
            let len: u32 = self.chained_sub_command_indices.len().try_into()?;
            <u32 as ProtoCodecVAR>::serialize(&len, stream)?;
            for i in &self.chained_sub_command_indices {
                <u16 as ProtoCodecLE>::serialize(i, stream)?;
            }
        }
        {
            let len: u32 = self.overloads.len().try_into()?;
            <u32 as ProtoCodecVAR>::serialize(&len, stream)?;
            for i in &self.overloads {
                <OverloadsEntry as ProtoCodec>::serialize(i, stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let name = <String as ProtoCodec>::deserialize(stream)?;
        let description = <String as ProtoCodec>::deserialize(stream)?;
        let flags = <u16 as ProtoCodecLE>::deserialize(stream)?;
        let permission_level = <V::CommandPermissionLevel as ProtoCodec>::deserialize(stream)?;
        let alias_enum = <i32 as ProtoCodecLE>::deserialize(stream)?;
        let chained_sub_command_indices = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<u16 as ProtoCodecLE>::deserialize(stream)?);
            }
            vec
        };
        let overloads = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
            let mut vec = Vec::with_capacity(len.try_into()?);
            for _ in 0..len {
                vec.push(<OverloadsEntry as ProtoCodec>::deserialize(stream)?);
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

    fn size_hint(&self) -> usize {
        self.name.size_hint()
            + self.description.size_hint()
            + size_of::<u32>()
            + self.chained_sub_command_indices.len() * size_of::<u16>()
            + size_of::<u32>()
            + self.overloads.iter().map(|i| i.size_hint()).sum::<usize>()
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
