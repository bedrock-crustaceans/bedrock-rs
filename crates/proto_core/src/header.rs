use crate::error::ProtoCodecError;
use crate::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct PacketHeader {
    pub packet_id: u16,
    pub sender_sub_client_id: u8,
    pub target_sub_client_id: u8,
}

impl ProtoCodec for PacketHeader {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let header = (self.packet_id & 0x3FF) as u32
            | ((self.sender_sub_client_id & 0x3) as u32) << 10
            | ((self.target_sub_client_id & 0x3) as u32) << 12;
        <u32 as ProtoCodecVAR>::serialize(&header, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let header = <u32 as ProtoCodecVAR>::deserialize(stream)?;

        Ok(Self {
            packet_id: (header & 0x3FF) as u16,
            sender_sub_client_id: ((header >> 10) & 0x3) as u8,
            target_sub_client_id: ((header >> 12) & 0x3) as u8,
        })
    }

    fn size_hint(&self) -> usize {
        let header = (self.packet_id & 0x3FF) as u32
            | ((self.sender_sub_client_id & 0x3) as u32) << 10
            | ((self.target_sub_client_id & 0x3) as u32) << 12;
        <u32 as ProtoCodecVAR>::size_hint(&header)
    }
}
