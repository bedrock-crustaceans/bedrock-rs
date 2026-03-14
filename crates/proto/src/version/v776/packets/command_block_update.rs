use crate::version::versions::ProtoVersion;
use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecLE};
use std::io::{Read, Write};
use std::mem::size_of;

#[packet(id = 78)]
#[derive(Clone, Debug)]
pub struct CommandBlockUpdatePacket<V: ProtoVersion> {
    pub is_block: bool, // TODO: probably worth turning this into an enum
    pub target_runtime_id: Option<V::ActorRuntimeID>, // Only if is_block is false
    pub block_position: Option<V::NetworkBlockPosition>, // Only if is_block is true
    pub command_block_mode: Option<V::CommandBlockMode>, // Only if is_block is true
    pub redstone_mode: Option<bool>, // Only if is_block is true
    pub is_conditional: Option<bool>, // Only if is_block is true
    pub command: String,
    pub last_output: String,
    pub name: String,
    pub filtered_name: String,
    pub track_output: bool,
    pub tick_delay: u32,
    pub should_execute_on_first_tick: bool,
}

impl<V: ProtoVersion> ProtoCodec for CommandBlockUpdatePacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        <bool as ProtoCodec>::serialize(&self.is_block, stream)?;
        match &self.is_block {
            false => {
                <V::ActorRuntimeID as ProtoCodec>::serialize(
                    self.target_runtime_id.as_ref().unwrap(),
                    stream,
                )?;
            }
            true => {
                <V::NetworkBlockPosition as ProtoCodec>::serialize(
                    self.block_position.as_ref().unwrap(),
                    stream,
                )?;
                <V::CommandBlockMode as ProtoCodec>::serialize(
                    self.command_block_mode.as_ref().unwrap(),
                    stream,
                )?;
                <bool as ProtoCodec>::serialize(self.redstone_mode.as_ref().unwrap(), stream)?;
                <bool as ProtoCodec>::serialize(self.is_conditional.as_ref().unwrap(), stream)?;
            }
        }
        <String as ProtoCodec>::serialize(&self.command, stream)?;
        <String as ProtoCodec>::serialize(&self.last_output, stream)?;
        <String as ProtoCodec>::serialize(&self.name, stream)?;
        <String as ProtoCodec>::serialize(&self.filtered_name, stream)?;
        <bool as ProtoCodec>::serialize(&self.track_output, stream)?;
        <u32 as ProtoCodecLE>::serialize(&self.tick_delay, stream)?;
        <bool as ProtoCodec>::serialize(&self.should_execute_on_first_tick, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let is_block = <bool as ProtoCodec>::deserialize(stream)?;
        let (target_runtime_id, block_position, command_block_mode, redstone_mode, is_conditional) =
            match &is_block {
                false => {
                    let target_runtime_id =
                        Some(<V::ActorRuntimeID as ProtoCodec>::deserialize(stream)?);
                    (target_runtime_id, None, None, None, None)
                }
                true => {
                    let block_position = Some(
                        <V::NetworkBlockPosition as ProtoCodec>::deserialize(stream)?,
                    );
                    let command_block_mode =
                        Some(<V::CommandBlockMode as ProtoCodec>::deserialize(stream)?);
                    let redstone_mode = Some(<bool as ProtoCodec>::deserialize(stream)?);
                    let is_conditional = Some(<bool as ProtoCodec>::deserialize(stream)?);
                    (
                        None,
                        block_position,
                        command_block_mode,
                        redstone_mode,
                        is_conditional,
                    )
                }
            };

        let command = <String as ProtoCodec>::deserialize(stream)?;
        let last_output = <String as ProtoCodec>::deserialize(stream)?;
        let name = <String as ProtoCodec>::deserialize(stream)?;
        let filtered_name = <String as ProtoCodec>::deserialize(stream)?;
        let track_output = <bool as ProtoCodec>::deserialize(stream)?;
        let tick_delay = <u32 as ProtoCodecLE>::deserialize(stream)?;
        let should_execute_on_first_tick = <bool as ProtoCodec>::deserialize(stream)?;

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

    fn size_hint(&self) -> usize {
        size_of::<bool>()
            + match &self.is_block {
                false => self.target_runtime_id.as_ref().unwrap().size_hint(),
                true => {
                    self.block_position.as_ref().unwrap().size_hint()
                        + self.command_block_mode.as_ref().unwrap().size_hint()
                        + size_of::<bool>()
                        + size_of::<bool>()
                }
            }
            + self.command.size_hint()
            + self.last_output.size_hint()
            + self.name.size_hint()
            + self.filtered_name.size_hint()
            + size_of::<bool>()
            + size_of::<u32>()
            + size_of::<bool>()
    }
}

// TODO: verify ProtoCodec impl
