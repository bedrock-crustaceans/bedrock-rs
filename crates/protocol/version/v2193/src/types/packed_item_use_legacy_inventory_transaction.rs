use crate::types::ItemUseTransactionData;
use bedrock_protocol_v662::types::ItemUseTransactionWithActions;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, true, false>;
