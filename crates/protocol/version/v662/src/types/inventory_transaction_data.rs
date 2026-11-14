use bedrock_protocol_version::ProtoVersion;
use crate::enums::ComplexInventoryTransactionType;
use bedrock_protocol_core::ProtoCodec;
use bedrock_protocol_core::error::ProtoCodecError;
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub enum InventoryTransactionData<V: ProtoVersion> {
    Normal,
    InventoryMismatch,
    ItemUse(V::ItemUseTransactionData),
    ItemUseOnEntity(V::ItemUseOnEntityTransactionData),
    ItemRelease(V::ItemReleaseTransactionData),
}

impl<V: ProtoVersion> InventoryTransactionData<V> {
    pub fn transaction_type(&self) -> ComplexInventoryTransactionType {
        match self {
            Self::Normal => ComplexInventoryTransactionType::NormalTransaction,
            Self::InventoryMismatch => ComplexInventoryTransactionType::InventoryMismatch,
            Self::ItemUse(_) => ComplexInventoryTransactionType::ItemUseTransaction,
            Self::ItemUseOnEntity(_) => ComplexInventoryTransactionType::ItemUseOnEntityTransaction,
            Self::ItemRelease(_) => ComplexInventoryTransactionType::ItemReleaseTransaction,
        }
    }

    /// Writes the transaction payload; the type tag is written separately.
    ///
    /// # Errors
    /// Returns a [`ProtoCodecError`] if the payload cannot be written to `stream`.
    pub fn serialize_payload<W: Write>(&self, stream: &mut W) -> Result<(), ProtoCodecError> {
        match self {
            Self::Normal | Self::InventoryMismatch => Ok(()),
            Self::ItemUse(data) => data.serialize(stream),
            Self::ItemUseOnEntity(data) => data.serialize(stream),
            Self::ItemRelease(data) => data.serialize(stream),
        }
    }

    /// Reads the payload selected by `transaction_type`.
    ///
    /// # Errors
    /// Returns a [`ProtoCodecError`] if `stream` ends early or the payload is malformed.
    pub fn deserialize_payload<R: Read>(
        transaction_type: &ComplexInventoryTransactionType,
        stream: &mut R,
    ) -> Result<Self, ProtoCodecError> {
        Ok(match transaction_type {
            ComplexInventoryTransactionType::NormalTransaction => Self::Normal,
            ComplexInventoryTransactionType::InventoryMismatch => Self::InventoryMismatch,
            ComplexInventoryTransactionType::ItemUseTransaction => {
                Self::ItemUse(ProtoCodec::deserialize(stream)?)
            }
            ComplexInventoryTransactionType::ItemUseOnEntityTransaction => {
                Self::ItemUseOnEntity(ProtoCodec::deserialize(stream)?)
            }
            ComplexInventoryTransactionType::ItemReleaseTransaction => {
                Self::ItemRelease(ProtoCodec::deserialize(stream)?)
            }
        })
    }

    pub fn size_hint(&self) -> usize {
        match self {
            Self::Normal | Self::InventoryMismatch => 0,
            Self::ItemUse(data) => data.size_hint(),
            Self::ItemUseOnEntity(data) => data.size_hint(),
            Self::ItemRelease(data) => data.size_hint(),
        }
    }
}
