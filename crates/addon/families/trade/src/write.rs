//! Writing [`TradeTable`]: the payload (the `tiers` array); the document is
//! `JsonDocument::write_root`'s.

use super::TradeTable;
use super::read::{CEREAL, parse_table};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

impl Versioned for TradeTable {
    const VERSION: FormatVersion = CEREAL;
}

impl VersionStruct for TradeTable {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_table(payload, cx, d)
    }

    /// The payload: the `tiers` array.
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(&self.tiers)
    }
}
