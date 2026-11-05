//! Reading and writing camera lists.

use super::{CameraList, texts};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, write_payload};
use bedrock_addon_support::chain_file as file;
use bedrock_addon_support::chain_file::document;
use bedrock_addon_support::shape::{self, Member, Shape, child, namespace_name, req, violation};

/// The list payload: both members required.
const LIST: &[Member] = &[
    req("identifier", Shape::Str),
    req("cameras", Shape::List(&Shape::Str)),
];

/// The list file: both members required.
const FILE: &[Member] = &[
    req("format_version", Shape::Str),
    req("minecraft:camera_list", Shape::Object(LIST)),
];

impl VersionStruct for CameraList {
    /// Reads the whole document with a strict, typed load.
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let list: Self = file::typed(
            text,
            &Shape::Object(FILE),
            Some("minecraft:camera_list"),
            shape::NOT_IN_SCHEMA,
            cx,
            d,
        )?;
        // The identifier must match `^\w+:\w+$`; unlike a preset's pattern, the empty string
        // does not pass.
        if !namespace_name(list.identifier.as_str()) {
            let at = child(&child(&cx.at, "minecraft:camera_list"), "identifier");
            violation(texts::IDENTIFIER, at, d);
            return None;
        }
        Some(list)
    }

    /// The whole document, at format 1.19.50.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            "1.19.50",
            "minecraft:camera_list",
            write_payload(self)?,
        ))
    }
}
