//! Reading and writing the camera documents. Each is read member by member by core's reader as
//! vanilla's strict cereal loader reads it (`chain_file::read`: every failure told in file order,
//! an unknown member always an error, a wrong type worded as the game words it). The bounds the
//! types state (`schema::min`, `schema::max`) are checked by the derived `Schema::check`
//! (`chain_file::check`) once the document is read, as the loader checks them once it is bound.
//!
//! What is the family's own: a name that must be `namespace:name`, a time that must not be
//! negative (each with the game's line, `schema::read_with`), and the rotation limits, whose
//! constraints the game checks in an order of its own. Its enums are read from a string and
//! nothing else (`schema::string_only`), a spline type outside the two with the game's line
//! (`schema::invalid`).

use super::history::v1_19_50;
use super::{CameraEntity, CameraList, CameraPreset, CameraSpline};
use super::{V1_19_50, V1_26_50, codes};
use bedrock_addon_core::diagnostic::cereal::{self, NumericRange};
use bedrock_addon_core::diagnostic::{Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, Schema};
use bedrock_addon_core::versioning::chain_file::{check, document, read};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned, write_payload};

/// The note of a scalar the legacy preset reader reads as the other JSON type: write-back
/// writes the member's own type there, so nothing is changed in silence.
const REWRITTEN: &str = "a number read as a bool (0 is false, any other number true) or a bool read as a number (1 or 0), as vanilla's non-strict reader of presets below 1.26.50 reads it; written back in the member's own type";

/// An error of the document at `at`.
fn violation(line: &'static CodeDef<0>, at: JsonPath, d: &mut Diagnostics) {
    d.push(
        Diagnostic::error(Code::from(line))
            .severity(Severity::Error)
            .at(at),
    );
}

/// Whether `text` is `^\w+:\w+$`: a namespace and a name, each one or more of `A-Z`, `a-z`,
/// `0-9` and `_`, around one `:`.
fn namespace_name(text: &str) -> bool {
    let word = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    };
    text.split_once(':')
        .is_some_and(|(namespace, name)| word(namespace) && word(name))
}

/// Tells `line` at `node`, a constraint the value breaks: the game's bind tells every violated
/// constraint of the document, so the member is kept and the read goes on.
fn violated(read: &mut Read<'_>, node: &Node<'_>, line: &'static CodeDef<0>) {
    let mut finding = Diagnostic::error(Code::from(line))
        .severity(Severity::Error)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// A preset's `identifier` and `inherit_from` (`schema::read_with`): `^(\w+:\w+)?$`, so the
/// empty string passes.
pub(crate) fn preset_name<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Id<K>> {
    let id = Id::<K>::from_json(node, read)?;
    if !id.as_str().is_empty() && !namespace_name(id.as_str()) {
        violated(read, node, &codes::IDENTIFIER);
    }
    Some(id)
}

/// A list's `identifier`: `^\w+:\w+$`, which the empty string does not match.
pub(crate) fn list_name<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Id<K>> {
    let id = Id::<K>::from_json(node, read)?;
    if !namespace_name(id.as_str()) {
        violated(read, node, &codes::IDENTIFIER);
    }
    Some(id)
}

/// A string member that must be `namespace:name`, told as `line` when it is not.
fn named(node: &Node<'_>, read: &mut Read<'_>, line: &'static CodeDef<0>) -> Option<String> {
    let name = String::from_json(node, read)?;
    if !namespace_name(&name) {
        violated(read, node, line);
    }
    Some(name)
}

/// A camera entity's `description.identifier`.
pub(crate) fn entity_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    named(node, read, &codes::ENTITY_IDENTIFIER)
}

/// A spline's `name`.
pub(crate) fn spline_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    named(node, read, &codes::SPLINE_NAME)
}

/// A number member that must be `0` or more, told as `line` when it is less.
fn not_below_zero(node: &Node<'_>, read: &mut Read<'_>, line: &'static CodeDef<0>) -> Option<f32> {
    let value = f32::from_json(node, read)?;
    if value < 0.0 {
        violated(read, node, line);
    }
    Some(value)
}

/// A spline's `total_time`.
pub(crate) fn total_time(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    not_below_zero(node, read, &codes::SPLINE_TOTAL_TIME)
}

/// A progress key frame's `time`.
pub(crate) fn progress_time(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    not_below_zero(node, read, &codes::SPLINE_PROGRESS_TIME)
}

/// A rotation key frame's `time`.
pub(crate) fn rotation_time(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    not_below_zero(node, read, &codes::SPLINE_ROTATION_TIME)
}

/// A progress key frame's `progress`: `0` to `1`.
pub(crate) fn progress(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let value = f32::from_json(node, read)?;
    if !(0.0..=1.0).contains(&value) {
        violated(read, node, &codes::SPLINE_PROGRESS);
    }
    Some(value)
}

/// Tells that an element of `limit` is outside `range`, at that element: the first one is the
/// one finding.
fn outside(limit: [f32; 2], range: NumericRange, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let Some(index) = limit
        .iter()
        .position(|value| !range.contains(f64::from(*value)))
    else {
        return false;
    };
    d.push(
        Diagnostic::error(range.code())
            .severity(Severity::Error)
            .at(at.index(index)),
    );
    true
}

/// A preset's rotation limits, at the preset's `at`: the constraints are checked in the order
/// the game binds them, and the first that fails is the one reported. Each component of the
/// horizontal limit is in `[0, 360]` and together they are at most 360; the vertical limit's
/// components differ, `x` is below `y`, and each is in `[0, 180]`.
fn rotation_limits(
    horizontal: Option<[f32; 2]>,
    vertical: Option<[f32; 2]>,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    if let Some(limit) = horizontal {
        let at = at.child("horizontal_rotation_limit");
        if !outside(limit, NumericRange::inclusive(0.0, 360.0), &at, d)
            && limit[0] + limit[1] > 360.0
        {
            violation(&codes::VECTOR_TOTAL, at, d);
        }
    }
    if let Some([x, y]) = vertical {
        let at = at.child("vertical_rotation_limit");
        #[allow(
            clippy::float_cmp,
            reason = "vanilla compares the two components for equality"
        )]
        if x == y {
            violation(&codes::VECTOR_EQUAL, at, d);
        } else if x >= y {
            violation(&codes::VECTOR_ORDER, at, d);
        } else {
            outside([x, y], NumericRange::inclusive(0.0, 180.0), &at, d);
        }
    }
}

/// The preset file (1.26.50): the version and the payload.
#[derive(facet::Facet, Schema)]
struct PresetFile {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:camera_preset")]
    payload: CameraPreset,
}

/// The preset file below 1.26.50.
#[derive(facet::Facet, Schema)]
struct LegacyPresetFile {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:camera_preset")]
    payload: v1_19_50::CameraPreset,
}

/// The list file.
#[derive(facet::Facet, Schema)]
struct ListFile {
    #[allow(dead_code, reason = "the envelope's version, read by the pipeline")]
    format_version: String,
    #[facet(rename = "minecraft:camera_list")]
    payload: CameraList,
}

impl VersionStruct for v1_19_50::CameraPreset {
    /// Reads the whole document (the raw-text path). The legacy slice's reader is the
    /// non-strict one.
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: LegacyPresetFile = read(
            text,
            cx,
            d,
            codes::UNRECOGNIZED_FIELD.template(),
            Some(REWRITTEN),
        )?;
        check(&mut file, cx, d);
        let preset = &mut file.payload;
        preset.source_format_version = cx.original_version;
        let at = cx.at.child("minecraft:camera_preset");
        rotation_limits(
            preset.horizontal_rotation_limit,
            preset.vertical_rotation_limit,
            &at,
            d,
        );
        Some(file.payload)
    }

    /// The whole document, at the version it was read at: the struct covers a range of
    /// versions, so writing a fixed one would change the `format_version` of a document that
    /// is read and written without an upgrade.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        Ok(document(
            &self.source_format_version.max(Self::VERSION).to_string(),
            "minecraft:camera_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for CameraPreset {
    /// Reads the whole document (the raw-text path).
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: PresetFile = read(text, cx, d, codes::UNRECOGNIZED_FIELD.template(), None)?;
        check(&mut file, cx, d);
        let at = cx.at.child("minecraft:camera_preset");
        rotation_limits(
            file.payload.horizontal_rotation_limit,
            file.payload.vertical_rotation_limit,
            &at,
            d,
        );
        Some(file.payload)
    }

    /// The whole document, at format 1.26.50; at 1.19.50 when the preset applies the inherited
    /// starting rotation and has no `starting_rot`, the one value 1.26.50 cannot spell: the
    /// legacy slice reads it back and the step sets the flag again. With a `starting_rot` the
    /// flag decides nothing at the client, which tests the two together, and the 1.26.50 form
    /// is written.
    // The raw-text path: `write_envelope` hands the payload back as is, so the struct
    // writes its own root.
    fn write(&self) -> Result<Value, EncodeError> {
        let version = if self.apply_inherited_starting_rotation && self.starting_rot.is_none() {
            V1_19_50
        } else {
            V1_26_50
        };
        Ok(document(
            &version.to_string(),
            "minecraft:camera_preset",
            write_payload(self)?,
        ))
    }
}

impl VersionStruct for CameraList {
    /// Reads the whole document, strictly.
    fn parse(text: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: ListFile = read(text, cx, d, cereal::MEMBER_NOT_IN_SCHEMA_TEXT, None)?;
        check(&mut file, cx, d);
        Some(file.payload)
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

impl VersionStruct for CameraSpline {
    /// Reads the payload (the envelope is checked from 1.26.0, the only version range).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut file: Self = read(payload, cx, d, cereal::MEMBER_NOT_IN_SCHEMA_TEXT, None)?;
        check(&mut file, cx, d);
        Some(file)
    }

    /// The payload.
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl VersionStruct for CameraEntity {
    /// Reads the payload (the envelope is always checked).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut entity: Self = read(payload, cx, d, cereal::MEMBER_NOT_IN_SCHEMA_TEXT, None)?;
        check(&mut entity, cx, d);
        // Each component of the horizontal limit is in `[0, 360]` and together at most 360.
        if let Some(limit) = entity
            .components
            .target
            .as_ref()
            .and_then(|target| target.horizontal_rotation_limit)
        {
            let at = cx
                .at
                .child("components")
                .child("minecraft:target")
                .child("horizontal_rotation_limit");
            let range = NumericRange::inclusive(0.0, 360.0);
            if limit.iter().any(|value| !range.contains(f64::from(*value))) {
                violation(&codes::TARGET_HORIZONTAL_RANGE, at, d);
            } else if limit[0] + limit[1] > 360.0 {
                violation(&codes::TARGET_HORIZONTAL_SUM, at, d);
            }
        }
        Some(entity)
    }

    /// The payload.
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}
