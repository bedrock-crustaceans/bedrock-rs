use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV898;

impl ProtoHelper for ProtoHelperV898 {
    type GamePacketType = GamePackets;
}
