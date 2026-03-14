use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 76)]
#[derive(Clone, Debug)]
pub struct AvailableCommandsPacket<V: ProtoVersion> {
    pub enum_values: Vec<String>,
    pub sub_command_values: Vec<String>,
    pub post_fixes: Vec<String>,
    pub enum_data: Vec<EnumDataEntry>,
    pub chained_sub_command_data: Vec<ChainedSubCommandDataEntry>,
    pub commands: Vec<CommandsEntry<V>>,
    pub soft_enums: Vec<SoftEnumsEntry>,
    pub constraints: Vec<ConstraintsEntry>,
}

impl<V: ProtoVersion> ProtoCodec for AvailableCommandsPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <Vec<_> as ProtoCodec>::serialize(&self.enum_values, stream)?;
        <Vec<_> as ProtoCodec>::serialize(&self.sub_command_values, stream)?;
        <Vec<_> as ProtoCodec>::serialize(&self.post_fixes, stream)?;
        {
            let len: u32 = self.enum_data.len() as u32;
            <u32 as ProtoCodecVAR>::serialize(&len, stream)?;
            for i in &self.enum_data {
                i.name.serialize(stream)?;
                <u32 as ProtoCodecVAR>::serialize(&(i.values.len() as u32), stream)?;
                for &j in &i.values {
                    match self.enum_values.len() {
                        len if len <= u8::MAX as usize => {
                            <u8 as ProtoCodec>::serialize(&(j as u8), stream)?
                        }
                        len if len <= u16::MAX as usize => {
                            <u16 as ProtoCodecLE>::serialize(&(j as u16), stream)?
                        }
                        _ => <u32 as ProtoCodecLE>::serialize(&j, stream)?,
                    };
                }
            }
        }
        <Vec<_> as ProtoCodec>::serialize(&self.chained_sub_command_data, stream)?;
        <Vec<_> as ProtoCodec>::serialize(&self.commands, stream)?;
        <Vec<_> as ProtoCodec>::serialize(&self.soft_enums, stream)?;
        <Vec<_> as ProtoCodec>::serialize(&self.constraints, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let enum_values = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let sub_command_values = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let post_fixes = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let enum_data = {
            let len = <u32 as ProtoCodecVAR>::deserialize(stream)? as usize;
            let mut vec = Vec::with_capacity(len);
            for _ in 0..len {
                let name = String::deserialize(stream)?;
                let j_len = <u32 as ProtoCodecVAR>::deserialize(stream)?;
                let mut j_vec = Vec::with_capacity(j_len as usize);
                for _ in 0..j_len {
                    let i = match enum_values.len() {
                        len if len <= u8::MAX as usize => {
                            <u8 as ProtoCodec>::deserialize(stream)? as u32
                        }
                        len if len <= u16::MAX as usize => {
                            <u16 as ProtoCodecLE>::deserialize(stream)? as u32
                        }
                        _ => <u32 as ProtoCodecLE>::deserialize(stream)?,
                    };
                    j_vec.push(i);
                }
                vec.push(EnumDataEntry {
                    name,
                    values: j_vec,
                })
            }
            vec
        };
        let chained_sub_command_data = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let commands = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let soft_enums = <Vec<_> as ProtoCodec>::deserialize(stream)?;
        let constraints = <Vec<_> as ProtoCodec>::deserialize(stream)?;

        Ok(Self {
            enum_values,
            sub_command_values,
            post_fixes,
            enum_data,
            chained_sub_command_data,
            commands,
            soft_enums,
            constraints,
        })
    }

    fn size_hint(&self) -> usize {
        self.enum_values.size_hint()
            + self.sub_command_values.size_hint()
            + self.post_fixes.size_hint()
            + self
                .enum_data
                .iter()
                .map(|i| {
                    i.name.size_hint()
                        + i.values
                            .iter()
                            .map(|&j| match self.enum_values.len() {
                                len if len <= u8::MAX as usize => {
                                    <u8 as ProtoCodec>::size_hint(&(j as u8))
                                }
                                len if len <= u16::MAX as usize => {
                                    <u16 as ProtoCodecLE>::size_hint(&(j as u16))
                                }
                                _ => <u32 as ProtoCodecLE>::size_hint(&j),
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.chained_sub_command_data.size_hint()
            + self.commands.size_hint()
            + self.soft_enums.size_hint()
            + self.constraints.size_hint()
    }
}

#[derive(Clone, Debug)]
pub struct EnumDataEntry {
    name: String,
    values: Vec<u32>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct SubCommandValues {
    #[endianness(le)]
    pub index: u16,
    #[endianness(le)]
    pub value: u16,
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
pub struct CommandsEntry<V: ProtoVersion> {
    pub name: String,
    pub description: String,
    #[endianness(le)]
    pub flags: u16,
    pub permission_level: V::CommandPermissionLevel,
    #[endianness(le)]
    pub alias_enum: i32,
    #[endianness(le)]
    pub chained_sub_command_indices: Vec<u16>,
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
