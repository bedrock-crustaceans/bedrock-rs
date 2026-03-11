use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV819;

impl ProtoHelper for ProtoHelperV819 {
    type GamePacketType = GamePackets;
}
