use crate::version::v2193::types::ItemUseTransactionData;
use crate::version::v662::types::ItemUseTransactionWithActions;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, true, false>;
