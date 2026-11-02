//! Writing [`VoxelShape`]: the payload, and the document with its envelope.

use facet_value::VObject;

use super::parse::parse_shape;
use super::{FLOOR, VoxelShape};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

impl Versioned for VoxelShape {
    const VERSION: FormatVersion = FLOOR;
}

impl VersionStruct for VoxelShape {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_shape(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("shape", write_payload(&self.shape)?);
        Ok(object.into())
    }
}

impl VoxelShape {
    /// The whole document: `format_version` (the shape's own, never below 1.21.110, which the
    /// game would not read) and the payload.
    fn document(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert(
            "format_version",
            self.source_format_version.max(FLOOR).to_string(),
        );
        object.insert("minecraft:voxel_shape", self.write()?);
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
