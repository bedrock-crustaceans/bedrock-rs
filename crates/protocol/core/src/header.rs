use crate::error::ProtoCodecError;
use crate::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PacketHeader {
    pub packet_id: u16,
    pub sender_sub_client_id: u8,
    pub target_sub_client_id: u8,
}

impl ProtoCodec for PacketHeader {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        let header = u32::from(self.packet_id & 0x3FF)
            | u32::from(self.sender_sub_client_id & 0x3) << 10
            | u32::from(self.target_sub_client_id & 0x3) << 12;
        <u32 as ProtoCodecVAR>::serialize(&header, stream)?;

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let header = <u32 as ProtoCodecVAR>::deserialize(stream)?;

        Ok(Self {
            packet_id: u16::try_from(header & 0x3FF)?,
            sender_sub_client_id: u8::try_from((header >> 10) & 0x3)?,
            target_sub_client_id: u8::try_from((header >> 12) & 0x3)?,
        })
    }

    fn size_hint(&self) -> usize {
        let header = u32::from(self.packet_id & 0x3FF)
            | u32::from(self.sender_sub_client_id & 0x3) << 10
            | u32::from(self.target_sub_client_id & 0x3) << 12;
        <u32 as ProtoCodecVAR>::size_hint(&header)
    }
}
