use bedrock_macros::{packet, ProtoCodec};

#[packet(id = 92, direction = "client_to_server")]
#[derive(ProtoCodec, Clone, Debug)]
pub struct PurchaseReceiptPacket {
    pub purchase_receipts: Vec<String>,
}
