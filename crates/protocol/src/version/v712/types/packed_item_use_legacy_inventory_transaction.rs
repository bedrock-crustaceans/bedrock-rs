use crate::version::v662::types::ItemUseTransactionWithActions;
use crate::version::v712::types::ItemUseTransactionData;

pub type PackedItemUseLegacyInventoryTransaction<V> = ItemUseTransactionWithActions<V, ItemUseTransactionData<V>, false, false>;
