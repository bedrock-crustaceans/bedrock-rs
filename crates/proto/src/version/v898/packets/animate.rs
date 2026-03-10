use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::{ProtoCodec, gamepacket};
use bedrockrs_proto_core::ProtoCodec;
use bedrockrs_proto_core::error::ProtoCodecError;
use std::io::Cursor;

#[gamepacket(id = 44)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct AnimatePacket<V: ProtoVersion> {
    pub action: AnimatePacketAction,
    pub target_runtime_id: V::ActorRuntimeID,
    #[endianness(le)]
    pub data: f32,
    pub swing_source: Option<SwingSource>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum AnimatePacketAction {
    NoAction = 0,
    Swing = 1,
    WakeUp = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

#[derive(Clone, Debug)]
pub enum SwingSource {
    Build,
    Mine,
    Interact,
    Attack,
    UseItem,
    ThrowItem,
    DropItem,
    Event,
}

impl TryFrom<String> for SwingSource {
    type Error = ProtoCodecError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "build" => Ok(SwingSource::Build),
            "mine" => Ok(SwingSource::Mine),
            "interact" => Ok(SwingSource::Interact),
            "attack" => Ok(SwingSource::Attack),
            "useitem" => Ok(SwingSource::UseItem),
            "throwitem" => Ok(SwingSource::ThrowItem),
            "dropitem" => Ok(SwingSource::DropItem),
            "event" => Ok(SwingSource::Event),
            invalid => Err(ProtoCodecError::InvalidEnumID(
                invalid.to_string(),
                "SwingSource",
            )),
        }
    }
}

impl From<SwingSource> for String {
    fn from(source: SwingSource) -> Self {
        match source {
            SwingSource::Build => "build",
            SwingSource::Mine => "mine",
            SwingSource::Interact => "interact",
            SwingSource::Attack => "attack",
            SwingSource::UseItem => "useitem",
            SwingSource::ThrowItem => "throwitem",
            SwingSource::DropItem => "dropitem",
            SwingSource::Event => "event",
        }
        .to_string()
    }
}

impl ProtoCodec for SwingSource {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        String::proto_serialize(&String::from(self.clone()), stream)
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        Self::try_from(String::proto_deserialize(stream)?)
    }

    fn get_size_prediction(&self) -> usize {
        String::from(self.clone()).get_size_prediction()
    }
}
