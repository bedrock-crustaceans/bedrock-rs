use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV800;

impl ProtoHelper for ProtoHelperV800 {
    type GamePacketType = GamePackets;
}
