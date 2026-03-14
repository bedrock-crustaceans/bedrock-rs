use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Cursor, Read, Write};
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[packet(id = 172)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSubChunkBlocksPacket<V: ProtoVersion> {
    pub sub_chunk_block_position: V::NetworkBlockPosition,
    pub standard_blocks_changed: Vec<BlocksChangedEntry<V>>,
    pub extra_blocks_changed: Vec<BlocksChangedEntry<V>>,
}

#[derive(Clone, Debug)]
pub struct BlocksChangedEntry<V: ProtoVersion> {
    pub pos: V::NetworkBlockPosition,
    pub runtime_id: u32,
    pub update_flags: u32,
    pub sync_message_entity_unique_id: u64,
    pub sync_message: V::ActorBlockSyncMessageID, // This is sent as unsigned varint, needs to be varint64
}

impl<V: ProtoVersion> ProtoCodec for BlocksChangedEntry<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.pos.serialize(stream)?;
        <u32 as ProtoCodecVAR>::serialize(&self.runtime_id, stream)?;
        <u32 as ProtoCodecVAR>::serialize(&self.update_flags, stream)?;
        <u64 as ProtoCodecVAR>::serialize(&self.sync_message_entity_unique_id, stream)?;

        let mut sync_message_stream: Vec<u8> = Vec::new();
        self.sync_message.serialize(&mut sync_message_stream)?;
        let mut sync_message_cursor = Cursor::new(sync_message_stream.as_slice());

        stream.write_u32_varint(sync_message_cursor.read_i64_varint()? as u32)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let pos = V::NetworkBlockPosition::deserialize(stream)?;
        let runtime_id = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let update_flags = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let sync_message_entity_unique_id = <u64 as ProtoCodecVAR>::deserialize(stream)?;

        let mut sync_message_stream: Vec<u8> = Vec::new();
        sync_message_stream.write_i64_varint(stream.read_u32_varint()? as i64)?;
        let mut sync_message_cursor = Cursor::new(sync_message_stream.as_slice());

        let sync_message = V::ActorBlockSyncMessageID::deserialize(&mut sync_message_cursor)?;

        Ok(Self {
            pos,
            runtime_id,
            update_flags,
            sync_message_entity_unique_id,
            sync_message,
        })
    }

    fn size_hint(&self) -> usize {
        self.pos.size_hint()
            + self.runtime_id.size_hint()
            + self.update_flags.size_hint()
            + self.sync_message_entity_unique_id.size_hint()
            + size_of::<u32>()
    }
}

// VERIFY: ProtoCodec impl
