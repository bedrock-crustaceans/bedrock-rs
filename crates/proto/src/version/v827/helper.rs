use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV827;

impl ProtoHelper for ProtoHelperV827 {
    type GamePacketType = GamePackets;
}
