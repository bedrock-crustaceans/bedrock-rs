use bedrock_protocol_v662::types::ItemUseTransactionWithActions;
use crate::types::ItemUseTransactionData;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, false, false>;
