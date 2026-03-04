use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::ProtoCodec;
use std::io::Cursor;
use crate::version::proto_version::ProtoVersion;

#[gamepacket(id = 5)]
#[derive(Clone, Debug)]
pub struct DisconnectPacket<V: ProtoVersion> {
    pub reason: V::ConnectionFailReason,
    pub message: Option<String>,
}

impl<V: ProtoVersion> ProtoCodec for DisconnectPacket<V> {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.reason.proto_serialize(stream)?;

        // Normally an optional type is prefixed by a bool indicating if the following type has a value,
        // but for the message in the DisconnectPacket<V> it is the other way around,
        // indicating if the following value should be skipped
        bool::proto_serialize(&self.message.is_none(), stream)?;

        if let Some(ref message) = self.message {
            message.proto_serialize(stream)?;
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let reason = V::ConnectionFailReason::proto_deserialize(stream)?;

        let skip_message = bool::proto_deserialize(stream)?;

        let message = if !skip_message {
            Some(String::proto_deserialize(stream)?)
        } else {
            None
        };

        Ok(Self { reason, message })
    }

    fn get_size_prediction(&self) -> usize {
        self.reason.get_size_prediction() + self.message.get_size_prediction()
    }
}
