use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, packet};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::{Read, Write};

#[packet(id = 5)]
#[derive(Clone, Debug)]
pub struct DisconnectPacket<V: ProtoVersion> {
    pub reason: V::ConnectionFailReason,
    pub message: Option<DisconnectMessage>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct DisconnectMessage {
    pub kick_message: String,
    pub filtered_message: String,
}

impl<V: ProtoVersion> ProtoCodec for DisconnectPacket<V> {
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.reason.serialize(stream)?;

        // Normally an optional type is prefixed by a bool indicating if the following type has a value,
        // but for the message in the DisconnectPacket<V> it is the other way around,
        // indicating if the following value should be skipped
        bool::serialize(&self.message.is_none(), stream)?;

        if let Some(ref message) = self.message {
            message.serialize(stream)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let reason = V::ConnectionFailReason::deserialize(stream)?;

        let skip_message = bool::deserialize(stream)?;

        let message = if !skip_message {
            Some(DisconnectMessage::deserialize(stream)?)
        } else {
            None
        };

        Ok(Self { reason, message })
    }

    fn size_hint(&self) -> usize {
        self.reason.size_hint() + self.message.size_hint()
    }
}
