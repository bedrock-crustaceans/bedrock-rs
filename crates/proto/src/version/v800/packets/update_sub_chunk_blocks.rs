use super::super::enums::ActorBlockSyncMessageID;
use super::super::types::NetworkBlockPosition;
use bedrockrs_macros::{gamepacket, ProtoCodec};
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::Cursor;
use std::mem::size_of;
use varint_rs::{VarintReader, VarintWriter};

#[gamepacket(id = 172)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateSubChunkBlocksPacket {
    pub sub_chunk_block_position: NetworkBlockPosition,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub standard_blocks_changed: Vec<BlocksChangedEntry>,
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub extra_blocks_changed: Vec<BlocksChangedEntry>,
}

#[derive(Clone, Debug)]
pub struct BlocksChangedEntry {
    pub pos: NetworkBlockPosition,
    pub runtime_id: u32,
    pub update_flags: u32,
    pub sync_message_entity_unique_id: u64,
    pub sync_message: ActorBlockSyncMessageID, // This is sent as unsigned varint, needs to be varint64
}

impl ProtoCodec for BlocksChangedEntry {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.pos.proto_serialize(stream)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&self.runtime_id, stream)?;
        <u32 as ProtoCodecVAR>::proto_serialize(&self.update_flags, stream)?;
        <u64 as ProtoCodecVAR>::proto_serialize(&self.sync_message_entity_unique_id, stream)?;

        let mut sync_message_stream: Vec<u8> = Vec::new();
        self.sync_message.proto_serialize(&mut sync_message_stream)?;
        let mut sync_message_cursor = Cursor::new(sync_message_stream.as_slice());

        stream.write_u32_varint(sync_message_cursor.read_i64_varint()? as u32)?;

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let pos = NetworkBlockPosition::proto_deserialize(stream)?;
        let runtime_id = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let update_flags = <u32 as ProtoCodecVAR>::proto_deserialize(stream)?;
        let sync_message_entity_unique_id = <u64 as ProtoCodecVAR>::proto_deserialize(stream)?;

        let mut sync_message_stream: Vec<u8> = Vec::new();
        sync_message_stream.write_i64_varint(stream.read_u32_varint()? as i64)?;
        let mut sync_message_cursor = Cursor::new(sync_message_stream.as_slice());

        let sync_message = ActorBlockSyncMessageID::proto_deserialize(&mut sync_message_cursor)?;

        Ok(Self {
            pos,
            runtime_id,
            update_flags,
            sync_message_entity_unique_id,
            sync_message,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.pos.get_size_prediction()
            + self.runtime_id.get_size_prediction()
            + self.update_flags.get_size_prediction()
            + self.sync_message_entity_unique_id.get_size_prediction()
            + size_of::<u32>()
    }
}

// VERIFY: ProtoCodec impl