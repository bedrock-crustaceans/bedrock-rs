use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV924;

impl ProtoHelper for ProtoHelperV924 {
    type GamePacketType = GamePackets;
}
