//! Reading [`VoxelShape`] from the text of a payload (the game's cereal parse): every failure
//! of the schema fails the file, an unknown member included.

use facet::Facet;
use facet_json::RawJson;

use super::{Description, Shape, ShapeBox, VoxelShape, texts};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{self, JsonSlice, Value, parse_at};
use bedrock_addon_core::versioning::ParseCx;

/// What a payload is read into: the corners are held as raw slices, because a corner has two
/// JSON forms.
#[derive(Facet)]
struct Shell<'a> {
    description: Description,
    shape: ShapeShell<'a>,
}

/// What `shape` is read into.
#[derive(Facet)]
struct ShapeShell<'a> {
    boxes: Vec<BoxShell<'a>>,
}

/// What one box is read into.
#[derive(Facet)]
struct BoxShell<'a> {
    min: RawJson<'a>,
    max: RawJson<'a>,
}

/// Reads one `minecraft:voxel_shape` payload; `None` (with an error in `d`) when vanilla's
/// cereal pass would fail the file.
pub(crate) fn parse_shape(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<VoxelShape> {
    let shell: Shell<'_> = match parse_at(payload) {
        Ok(shell) => shell,
        Err(failure) => {
            let blank = payload.text.trim().is_empty();
            let code = code_for_deserialize_error_kind(
                &failure.error.kind,
                !failure.path.is_empty(),
                blank,
            );
            d.push(error(code, join(&cx.at, &failure.path), Some(failure.span)));
            return None;
        }
    };
    // Each of these is an error in `d`, and an error rejects the document.
    unknown_members(payload, &cx.at, d);
    constraints(
        shell.description.identifier.as_str(),
        shell.shape.boxes.len(),
        &cx.at,
        d,
    );
    // Every corner of the file is read, so every bad one is reported; a box with one has no
    // value and neither has the shape.
    let boxes: Vec<Option<ShapeBox>> = shell
        .shape
        .boxes
        .iter()
        .enumerate()
        .map(|(index, shell)| {
            let mut at = child(&cx.at, &["shape", "boxes"]);
            at.push_index(index);
            let min = corner(payload, &shell.min, &child(&at, &["min"]), d);
            let max = corner(payload, &shell.max, &child(&at, &["max"]), d);
            Some(ShapeBox {
                min: min?,
                max: max?,
            })
        })
        .collect();
    Some(VoxelShape {
        description: shell.description,
        shape: Shape {
            boxes: boxes.into_iter().collect::<Option<_>>()?,
        },
        source_format_version: cx.original_version,
    })
}

/// The longest identifier, in bytes (the length limit of the game's identifier strings).
pub(crate) const MAX_IDENTIFIER: usize = 256;

/// The fewest boxes a shape has.
pub(crate) const MIN_BOXES: usize = 1;

/// The most boxes a shape has.
pub(crate) const MAX_BOXES: usize = 32;

/// The schema's constraints on a shape named `identifier` with `boxes` boxes, each failure an
/// error below `at` (the payload's path).
pub(crate) fn constraints(identifier: &str, boxes: usize, at: &JsonPath, d: &mut Diagnostics) {
    // The identifier constraint is a length cap and no pattern; the game's text for it is not
    // reproduced here.
    if identifier.len() > MAX_IDENTIFIER {
        d.push(error(
            DiagnosticCode::InvalidValue { text: None },
            child(at, &["description", "identifier"]),
            None,
        ));
    }
    // A shape needs a box; the game's text for it is not reproduced here.
    if boxes < MIN_BOXES {
        d.push(error(
            DiagnosticCode::InvalidValue { text: None },
            child(at, &["shape", "boxes"]),
            None,
        ));
    }
    if boxes > MAX_BOXES {
        d.push(error(
            DiagnosticCode::InvalidValue {
                text: Some(texts::too_many_boxes()),
            },
            child(at, &["shape", "boxes"]),
            None,
        ));
    }
}

/// Reports every member of `payload` no schema type binds, in document order (the corners are
/// read on their own), each an `UnrecognizedField` warning that the slice's strict mask makes
/// the error that fails the file (`Family::error_mask`).
fn unknown_members(payload: JsonSlice<'_>, at: &JsonPath, d: &mut Diagnostics) {
    let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) else {
        return;
    };
    for (path, _) in &json::unknown_members(<Shell<'_> as Facet>::SHAPE, &json) {
        d.push(unknown(join(at, path)));
    }
}

/// One corner (vanilla's `Vec3`), in the two forms the game tries in this order: `[x, y, z]`,
/// exactly three numbers; then an object with the optional members `x`, `y` and `z`, an absent
/// one 0. `None` (with the errors in `d`) for anything else.
// The game logs the help text when the array form fails and clears it when the members load.
fn corner(
    payload: JsonSlice<'_>,
    raw: &RawJson<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<[f32; 3]> {
    let span = payload.nested(raw).map(|slice| Span {
        offset: u32::try_from(slice.offset).unwrap_or(u32::MAX),
        len: u32::try_from(slice.text.len()).unwrap_or(u32::MAX),
    });
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(raw.as_str()).ok();
    let json = json.as_ref();
    if let Some([x, y, z]) = json
        .and_then(Value::as_array)
        .map(facet_value::VArray::as_slice)
        && let (Some(x), Some(y), Some(z)) = (number(x), number(y), number(z))
    {
        return Some([x, y, z]);
    }
    let expected = |d: &mut Diagnostics| {
        d.push(error(
            DiagnosticCode::InvalidType {
                text: Some(texts::expected_x_y_z()),
            },
            at.clone(),
            span,
        ));
    };
    let Some(object) = json.and_then(Value::as_object) else {
        expected(d);
        d.push(error(
            bedrock_addon_core::diagnostic::cereal::expected_object(),
            at.clone(),
            span,
        ));
        return None;
    };
    let mut read = [0.0; 3];
    let mut unbound = Vec::new();
    let mut mistyped = Vec::new();
    for (key, value) in object {
        let Some(axis) = ["x", "y", "z"]
            .iter()
            .position(|name| *name == key.as_str())
        else {
            unbound.push(key.as_str());
            continue;
        };
        match number(value) {
            Some(value) => read[axis] = value,
            None => mistyped.push(key.as_str()),
        }
    }
    if !mistyped.is_empty() {
        expected(d);
    }
    for key in &mistyped {
        d.push(error(
            DiagnosticCode::InvalidType { text: None },
            child(at, &[key]),
            None,
        ));
    }
    for key in &unbound {
        d.push(unknown(child(at, &[key])));
    }
    // With a finding above the value is not used: an error in `d` rejects the document.
    Some(read)
}

/// A JSON number (an integer or a real token) as the `f32` nearest to it; `None` for any other
/// JSON type and for a number no finite `f32` is near (as the crate's strict floats read).
fn number(json: &Value) -> Option<f32> {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    let x = json.as_number()?.to_f64_lossy() as f32;
    x.is_finite().then_some(x)
}

/// A member the schema does not bind, at `at`: the marker `unknown_members_are_errors` turns
/// into the error that refuses the document (every voxel-shape slice is strict).
fn unknown(at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::UnrecognizedField {
            text: Some(texts::not_in_schema()),
        },
        Severity::Warning,
        Phase::Parse,
        Location::new(None, None, at, None),
    )
}

/// An error of the schema loader at `at`.
fn error(code: DiagnosticCode, at: JsonPath, span: Option<Span>) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, span),
    )
}

/// `base` followed by the segments of `relative`.
fn join(base: &JsonPath, relative: &JsonPath) -> JsonPath {
    let mut path = base.clone();
    path.append(relative);
    path
}

/// `base` followed by `keys`.
pub(crate) fn child(base: &JsonPath, keys: &[&str]) -> JsonPath {
    let mut path = base.clone();
    for key in keys {
        path.push(key);
    }
    path
}
