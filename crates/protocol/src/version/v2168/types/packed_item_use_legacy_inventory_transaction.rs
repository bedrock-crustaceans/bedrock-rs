use crate::version::v1001::types::ItemUseTransactionData;
use crate::version::v662::types::ItemUseTransactionWithActions;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, true, true>;
