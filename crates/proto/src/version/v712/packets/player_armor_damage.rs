use bedrockrs_macros::packet;
use bedrockrs_proto_core::error::ProtoCodecError;
use bedrockrs_proto_core::{ProtoCodec, ProtoCodecVAR};
use std::io::{Read, Write};

#[packet(id = 149)]
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
    fn serialize<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        self.slot_bitset.serialize(stream)?;
        for i in 0..5 {
            let flag = 1 << i;
            if (self.slot_bitset & flag) != 0 {
                ProtoCodecVAR::serialize(&self.damage[i], stream)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(stream: &mut R) -> Result<Self, ProtoCodecError> {
        let slot_bitset = i8::deserialize(stream)?;
        let damage = {
            let mut damage = [0; 5];
            for (i, slot) in damage.iter_mut().enumerate() {
                let flag = 1 << i;
                if (slot_bitset & flag) != 0 {
                    *slot = <i32 as ProtoCodecVAR>::deserialize(stream)?;
                }
            }
            damage
        };

        Ok(Self {
            slot_bitset,
            damage,
        })
    }

    fn size_hint(&self) -> usize {
        self.slot_bitset.size_hint()
            + (0..5)
                .filter_map(|i| {
                    let flag = 1 << i;
                    if (self.slot_bitset & flag) != 0 {
                        Some(ProtoCodecVAR::size_hint(&self.damage[i]))
                    } else {
                        None
                    }
                })
                .sum::<usize>()
    }
}
