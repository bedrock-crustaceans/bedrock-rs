//! Reading [`SoundMetadataFile`]: the one struct's `VersionStruct`, its payload read strictly by
//! core's reader (`versioning::read_payload`) under the slice's reading.

use super::{SoundMetadataFile, V1_26_60};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ParseCx, VersionStruct, Versioned, read_payload, write_payload,
};

impl Versioned for SoundMetadataFile {
    const VERSION: FormatVersion = V1_26_60;
}

impl VersionStruct for SoundMetadataFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}
