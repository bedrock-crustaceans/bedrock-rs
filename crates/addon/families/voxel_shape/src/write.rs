//! Writing [`VoxelShape`]: the payload (the envelope is core's `write_envelope`).

use facet_value::VObject;

use super::read::parse_shape;
use super::{FLOOR, VoxelShape};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
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
