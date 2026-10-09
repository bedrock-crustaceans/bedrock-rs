use bedrock_protocol_core::ProtoCodecVAR;
use bedrock_macros::packet;
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Read, Write};
use std::mem::size_of;

#[packet(id = 8)]
#[derive(Clone, Debug)]
pub enum ResourcePackClientResponsePacket {
    Cancel,
    Downloading(Vec<String>),
    DownloadingFinished,
    ResourcePackStackFinished,
}

impl ProtoCodec for ResourcePackClientResponsePacket {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match self {
            Self::Cancel => {
                <u32 as ProtoCodecVAR>::serialize(&0, stream)?;
                <String as ProtoCodec>::serialize(&String::from("cancel"), stream)?;
            }
            Self::Downloading(downloading_packs) => {
                <u32 as ProtoCodecVAR>::serialize(&1, stream)?;
                <String as ProtoCodec>::serialize(&String::from("downloading"), stream)?;
                <Vec<String> as ProtoCodec>::serialize(downloading_packs, stream)?;
            }
            Self::DownloadingFinished => {
                <u32 as ProtoCodecVAR>::serialize(&2, stream)?;
                <String as ProtoCodec>::serialize(&String::from("downloadingfinished"), stream)?;
            }
            Self::ResourcePackStackFinished => {
                <u32 as ProtoCodecVAR>::serialize(&3, stream)?;
                <String as ProtoCodec>::serialize(
                    &String::from("resourcepackstackfinished"),
                    stream,
                )?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let response = <u32 as ProtoCodecVAR>::deserialize(stream)?;
        let _response_id = <String as ProtoCodec>::deserialize(stream)?;

        Ok(match response {
            0 => Self::Cancel,
            1 => Self::Downloading(<Vec<String> as ProtoCodec>::deserialize(stream)?),
            2 => Self::DownloadingFinished,
            3 => Self::ResourcePackStackFinished,
            other => {
                return Err(ProtoCodecError::InvalidEnumID(
                    other.to_string(),
                    "ResourcePackClientResponsePacket",
                ));
            }
        })
    }

    fn size_hint(&self) -> usize {
        let header = size_of::<u32>() + size_of::<u32>() + 25;

        header
            + match self {
                Self::Downloading(downloading_packs) => downloading_packs.size_hint(),
                _ => 0,
            }
    }
}
