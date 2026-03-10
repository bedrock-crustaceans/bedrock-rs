use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV818;

impl ProtoHelper for ProtoHelperV818 {
    type GamePacketType = GamePackets;
}
