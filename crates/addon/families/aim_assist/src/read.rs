//! Reading and writing the aim-assist documents. Each is read member by member by core's reader
//! as vanilla's strict cereal loader reads it (`chain_file::read`): every failure told in file
//! order, an unknown member always an error, a wrong type worded as the game words it; then the
//! bounds the types state (`chain_file::check`), the priorities' two ends among them.

use super::history::v1_21_50;
use super::{AimAssistCategories, AimAssistPreset};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::diagnostic::cereal;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::schema::Schema;
use bedrock_addon_core::versioning::chain_file::{check, document, read};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

pub(crate) use super::checks::{category_namespaces, preset_namespaces};

/// The preset file at 1.21.50: the version and the payload.
#[derive(facet::Facet, Schema)]
struct PresetFileV1_21_50 {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:aim_assist_preset")]
    payload: v1_21_50::AimAssistPreset,
}

/// The preset file at 1.21.120.
#[derive(facet::Facet, Schema)]
struct PresetFile {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:aim_assist_preset")]
    payload: AimAssistPreset,
}

/// The categories file.
#[derive(facet::Facet, Schema)]
struct CategoriesFile {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:aim_assist_categories")]
    payload: AimAssistCategories,
}

/// The line of an unknown member of any aim-assist document.
const UNKNOWN: &str = cereal::MEMBER_NOT_IN_SCHEMA_TEXT;

impl VersionStruct for v1_21_50::AimAssistPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: PresetFileV1_21_50 = read(text, cx, d, UNKNOWN, None)?;
        check(&mut file, cx, d);
        file.payload.source_format_version = cx.original_version;
        Some(file.payload)
    }

    /// The whole document, at the version it was read at.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            &self.source_format_version.max(Self::VERSION).to_string(),
            "minecraft:aim_assist_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for AimAssistPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: PresetFile = read(text, cx, d, UNKNOWN, None)?;
        check(&mut file, cx, d);
        Some(file.payload)
    }

    /// The whole document, at format 1.21.120.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            "1.21.120",
            "minecraft:aim_assist_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for AimAssistCategories {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: CategoriesFile = read(text, cx, d, UNKNOWN, None)?;
        check(&mut file, cx, d);
        Some(file.payload)
    }

    /// The whole document, at format 1.21.50.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            "1.21.50",
            "minecraft:aim_assist_categories",
            write_payload(self)?,
        ))
    }
}
