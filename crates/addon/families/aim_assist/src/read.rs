//! Reading and writing the aim-assist documents. Each is read member by member by core's reader
//! as vanilla's strict cereal loader reads it (`chain_file::read`): every failure told in file
//! order, an unknown member always an error, a wrong type worded as the game words it. The
//! family's own is a priority, a number from 0 to 100 whose two ends have two lines.

use super::history::v1_21_50;
use super::{AimAssistCategories, AimAssistPreset};
use bedrock_addon_core::diagnostic::cereal::{self, NumericRange};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, Schema, read_entries};
use bedrock_addon_core::versioning::chain_file::{check, document, read};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};
use indexmap::IndexMap;

pub(crate) use super::checks::{category_namespaces, preset_namespaces};

/// A priority (`schema::read_with`): a number in `[0, 100]`. A negative number is cereal's
/// "invalid numeric value"; a number above 100 is a range violation. Each is told and the member
/// kept: the game's bind tells every violated constraint.
pub(crate) fn priority(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let value = i32::from_json(node, read)?;
    let code = if value < 0 {
        Some(core_codes::INVALID_VALUE_WORDED.with([cereal::INVALID_NUMERIC_VALUE_TEXT]))
    } else {
        let range = NumericRange::inclusive(0.0, 100.0);
        (!range.contains(f64::from(value))).then(|| range.code())
    };
    if let Some(code) = code {
        let mut finding = Diagnostic::error(code)
            .severity(Severity::Error)
            .at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    Some(value)
}

/// A map of priorities (`schema::read_with`): each value a [`priority`]; one that is no number
/// fails the map.
pub(crate) fn priority_map(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, i32>> {
    read_entries(node, read, priority)
}

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
