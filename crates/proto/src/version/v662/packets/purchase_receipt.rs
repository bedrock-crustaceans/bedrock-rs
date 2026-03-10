use bedrockrs_macros::{ProtoCodec, gamepacket};

#[gamepacket(id = 92)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PurchaseReceiptPacket {
    pub purchase_receipts: Vec<String>,
}
