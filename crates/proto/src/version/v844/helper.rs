use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV844;

impl ProtoHelper for ProtoHelperV844 {
    type GamePacketType = GamePackets;
}
