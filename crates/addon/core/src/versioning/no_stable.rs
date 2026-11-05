//! [`NoStable`] as a chain struct, so that every `Family::Latest` is a [`VersionStruct`].

use super::{NoStable, ParseCx, VersionStruct};
use crate::diagnostic::Diagnostics;
use crate::json::{EncodeError, JsonSlice, Value};

/// Never parsed: a beta-only family's chain lists no stable struct, so no version selects it.
impl VersionStruct for NoStable {
    fn parse(_: JsonSlice<'_>, _: &ParseCx<'_>, _: &mut Diagnostics) -> Option<Self> {
        None
    }

    fn write(&self) -> Result<Value, EncodeError> {
        match *self {}
    }
}
