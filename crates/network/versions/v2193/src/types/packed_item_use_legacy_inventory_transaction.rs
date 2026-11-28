use crate::types::ItemUseTransactionData;
use bedrock_network_version_v662::types::ItemUseTransactionWithActions;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, true, false>;
