//! Writing the family's JSON files back as they were read, by core's generic writer
//! (`schema::write_value`): the members the model holds in the game's order (the declaration
//! order of the types), then the members kept as written. A member the author did not write
//! is never written; of a member held both ways (kept as written at load, then set in the
//! model) the model's stands.

use facet_value::VObject;

use super::read::ENVELOPE;
use super::{PieceMetadata, PieceStoreInfo};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::schema::write_value;

impl PieceMetadata {
    /// The file as a JSON value: flat, or inside the envelope it was read from.
    pub(super) fn to_value(&self) -> Result<Value, EncodeError> {
        let payload = write_value(self)?;
        Ok(match &self.envelope {
            None => payload,
            Some(version) => {
                let mut document = VObject::new();
                document.insert("format_version", version.clone());
                document.insert(ENVELOPE, payload);
                document.into()
            }
        })
    }
}

impl PieceStoreInfo {
    /// The file as a JSON value.
    pub(super) fn to_value(&self) -> Result<Value, EncodeError> {
        write_value(self)
    }
}
