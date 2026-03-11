use super::gamepackets::GamePackets;
use crate::helper::ProtoHelper;

pub struct ProtoHelperV859;

impl ProtoHelper for ProtoHelperV859 {
    type GamePacketType = GamePackets;
}
