//! Writing [`TradeTable`]: the payload (the `tiers` array), and the document with its envelope.

use super::TradeTable;
use super::parse::{CEREAL, parse_table};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
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

impl TradeTable {
    /// The whole document: `format_version` (1.21.30, the only struct from it on) and `tiers`;
    /// a table that holds a legacy blob in the legacy shape instead (`v0::legacy_form`: the
    /// newest format has the blobs only under deprecated names the game rejects).
    pub(crate) fn document(&self) -> Result<Value, EncodeError> {
        if let Some(legacy) = super::history::v0::legacy_form(self)? {
            return write_payload(&legacy);
        }
        let mut object = facet_value::VObject::new();
        object.insert("format_version", CEREAL.to_string());
        object.insert("tiers", self.write()?);
        Ok(object.into())
    }

    /// Writes the document into `out`.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.document()?, out, style)
    }
}
