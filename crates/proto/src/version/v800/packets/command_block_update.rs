use super::super::enums::CommandBlockMode;
use super::super::types::{ActorRuntimeID, NetworkBlockPosition};
use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE};
use std::io::Cursor;
use std::mem::size_of;

#[gamepacket(id = 78)]
#[derive(Clone, Debug)]
pub struct CommandBlockUpdatePacket {
    pub is_block: bool,
    pub target_runtime_id: Option<ActorRuntimeID>, // Only if is_block is false
    pub block_position: Option<NetworkBlockPosition>, // Only if is_block is true
    pub command_block_mode: Option<CommandBlockMode>, // Only if is_block is true
    pub redstone_mode: Option<bool>,               // Only if is_block is true
    pub is_conditional: Option<bool>,              // Only if is_block is true
    pub command: String,
    pub last_output: String,
    pub name: String,
    pub filtered_name: String,
    pub track_output: bool,
    pub tick_delay: u32,
    pub should_execute_on_first_tick: bool,
}

impl ProtoCodec for CommandBlockUpdatePacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        <bool as ProtoCodec>::proto_serialize(&self.is_block, stream)?;
        match &self.is_block {
            false => {
                <ActorRuntimeID as ProtoCodec>::proto_serialize(
                    &self.target_runtime_id.as_ref().unwrap(),
                    stream,
                )?;
            }
            true => {
                <NetworkBlockPosition as ProtoCodec>::proto_serialize(
                    &self.block_position.as_ref().unwrap(),
                    stream,
                )?;
                <CommandBlockMode as ProtoCodec>::proto_serialize(
                    &self.command_block_mode.as_ref().unwrap(),
                    stream,
                )?;
                <bool as ProtoCodec>::proto_serialize(
                    &self.redstone_mode.as_ref().unwrap(),
                    stream,
                )?;
                <bool as ProtoCodec>::proto_serialize(
                    &self.is_conditional.as_ref().unwrap(),
                    stream,
                )?;
            }
        }
        <String as ProtoCodec>::proto_serialize(&self.command, stream)?;
        <String as ProtoCodec>::proto_serialize(&self.last_output, stream)?;
        <String as ProtoCodec>::proto_serialize(&self.name, stream)?;
        <String as ProtoCodec>::proto_serialize(&self.filtered_name, stream)?;
        <bool as ProtoCodec>::proto_serialize(&self.track_output, stream)?;
        <u32 as ProtoCodecLE>::proto_serialize(&self.tick_delay, stream)?;
        <bool as ProtoCodec>::proto_serialize(&self.should_execute_on_first_tick, stream)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let is_block = <bool as ProtoCodec>::proto_deserialize(stream)?;
        let (target_runtime_id, block_position, command_block_mode, redstone_mode, is_conditional) =
            match &is_block {
                false => {
                    let target_runtime_id =
                        Some(<ActorRuntimeID as ProtoCodec>::proto_deserialize(stream)?);
                    (target_runtime_id, None, None, None, None)
                }
                true => {
                    let block_position = Some(
                        <NetworkBlockPosition as ProtoCodec>::proto_deserialize(stream)?,
                    );
                    let command_block_mode =
                        Some(<CommandBlockMode as ProtoCodec>::proto_deserialize(stream)?);
                    let redstone_mode = Some(<bool as ProtoCodec>::proto_deserialize(stream)?);
                    let is_conditional = Some(<bool as ProtoCodec>::proto_deserialize(stream)?);
                    (
                        None,
                        block_position,
                        command_block_mode,
                        redstone_mode,
                        is_conditional,
                    )
                }
            };

        let command = <String as ProtoCodec>::proto_deserialize(stream)?;
        let last_output = <String as ProtoCodec>::proto_deserialize(stream)?;
        let name = <String as ProtoCodec>::proto_deserialize(stream)?;
        let filtered_name = <String as ProtoCodec>::proto_deserialize(stream)?;
        let track_output = <bool as ProtoCodec>::proto_deserialize(stream)?;
        let tick_delay = <u32 as ProtoCodecLE>::proto_deserialize(stream)?;
        let should_execute_on_first_tick = <bool as ProtoCodec>::proto_deserialize(stream)?;

        Ok(Self {
            is_block,
            target_runtime_id,
            block_position,
            command_block_mode,
            redstone_mode,
            is_conditional,
            command,
            last_output,
            name,
            filtered_name,
            track_output,
            tick_delay,
            should_execute_on_first_tick,
        })
    }

    fn get_size_prediction(&self) -> usize {
        size_of::<bool>()
            + match &self.is_block {
            true => self
                .target_runtime_id
                .as_ref()
                .unwrap()
                .get_size_prediction(),
            false => {
                self.block_position.as_ref().unwrap().get_size_prediction()
                    + self
                    .command_block_mode
                    .as_ref()
                    .unwrap()
                    .get_size_prediction()
                    + size_of::<bool>()
                    + size_of::<bool>()
            }
        }
            + &self.command.get_size_prediction()
            + &self.last_output.get_size_prediction()
            + &self.name.get_size_prediction()
            + size_of::<bool>()
            + size_of::<u32>()
            + size_of::<bool>()
    }
}

// VERIFY: ProtoCodec impl
