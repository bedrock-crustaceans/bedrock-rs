use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 92)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PurchaseReceiptPacket {
    pub purchase_receipts: Vec<String>,
}
