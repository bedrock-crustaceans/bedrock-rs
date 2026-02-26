use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 92)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PurchaseReceiptPacket {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub purchase_receipts: Vec<String>,
}