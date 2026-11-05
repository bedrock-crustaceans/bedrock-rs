//! Reading [`VoxelShape`] from the text of a payload (the game's cereal parse): every failure
//! of the schema fails the file, an unknown member included.

use facet::Facet;
use facet_json::RawJson;

use super::{Description, Shape, ShapeBox, VoxelShape, texts};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Span};
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
            d.push(failure.into_diagnostic(&cx.at, |code, _| code));
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
            let at = cx.at.with(&["shape", "boxes"]).index(index);
            let min = corner(payload, &shell.min, &at.child("min"), d);
            let max = corner(payload, &shell.max, &at.child("max"), d);
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
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                .at(at.with(&["description", "identifier"])),
        );
    }
    // A shape needs a box; the game's text for it is not reproduced here.
    if boxes < MIN_BOXES {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text: None })
                .at(at.with(&["shape", "boxes"])),
        );
    }
    if boxes > MAX_BOXES {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(texts::too_many_boxes()),
            })
            .at(at.with(&["shape", "boxes"])),
        );
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
        d.push(unknown(at.join(path)));
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
    // A corner's own finding is at the corner's text, when the payload holds it.
    let at_corner = |code: DiagnosticCode| {
        let finding = Diagnostic::error(code).at(at.clone());
        match span {
            Some(span) => finding.span(span),
            None => finding,
        }
    };
    let expected = |d: &mut Diagnostics| {
        d.push(at_corner(DiagnosticCode::InvalidType {
            text: Some(texts::expected_x_y_z()),
        }));
    };
    let Some(object) = json.and_then(Value::as_object) else {
        expected(d);
        d.push(at_corner(
            bedrock_addon_core::diagnostic::cereal::expected_object(),
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
        d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at.child(key)));
    }
    for key in &unbound {
        d.push(unknown(at.child(key)));
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
    Diagnostic::warn(DiagnosticCode::UnrecognizedField {
        text: Some(texts::not_in_schema()),
    })
    .at(at)
}
