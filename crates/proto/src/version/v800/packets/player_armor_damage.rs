use bedrockrs_macros::gamepacket;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::Cursor;

#[gamepacket(id = 149)]
#[derive(Clone, Debug)]
pub struct PlayerArmorDamagePacket {
    pub slot_bitset: i8,
    pub damage: [i32; 5],
}

pub enum PlayerArmorDamageFlag {
    Helmet = 1 << 0,
    Chestplate = 1 << 1,
    Leggings = 1 << 2,
    Boots = 1 << 3,
    Body = 1 << 4,
}

impl ProtoCodec for PlayerArmorDamagePacket {
    fn proto_serialize(&self, stream: &mut Vec<u8>) -> Result<(), ProtoCodecError> {
        self.slot_bitset.proto_serialize(stream)?;
        for i in 0..5 {
            let flag = 1 << i;
            if (self.slot_bitset & flag) != 0 {
                ProtoCodecVAR::proto_serialize(&self.damage[i], stream)?;
            }
        }

        Ok(())
    }

    fn proto_deserialize(stream: &mut Cursor<&[u8]>) -> Result<Self, ProtoCodecError> {
        let slot_bitset = i8::proto_deserialize(stream)?;
        let damage = {
            let mut damage = [0; 5];
            for i in 0..5 {
                let flag = 1 << i;
                if (slot_bitset & flag) != 0 {
                    damage[i] = <i32 as ProtoCodecVAR>::proto_deserialize(stream)?;
                }
            }
            damage
        };

        Ok(Self {
            slot_bitset,
            damage,
        })
    }

    fn get_size_prediction(&self) -> usize {
        self.slot_bitset.get_size_prediction()
            + (0..5)
            .filter_map(|i| {
                let flag = 1 << i;
                if (self.slot_bitset & flag) != 0 {
                    Some(ProtoCodecVAR::get_size_prediction(&self.damage[i]))
                } else {
                    None
                }
            })
            .sum::<usize>()
    }
}
