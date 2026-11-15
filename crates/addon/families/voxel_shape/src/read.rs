//! Reading [`VoxelShape`] from the text of a payload (the game's cereal parse) with core's
//! reader (`versioning::read_payload_with`): every failure of the schema fails the file, an
//! unknown member included. The family's own is a corner, which has two JSON forms.

use super::{VoxelShape, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, cereal};
use bedrock_addon_core::json::kit::{Node, Read, Texts};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::UnknownRule;
use bedrock_addon_core::versioning::{ParseCx, read_payload_with};

/// An unknown member is cereal's line, as the game logs it.
const TEXTS: Texts = Texts {
    unknown_code: Some(&core_codes::MEMBER_NOT_IN_SCHEMA),
    ..Texts::CODES
};

/// Reads one `minecraft:voxel_shape` payload; `None` (with an error in `d`) when vanilla's
/// cereal pass would fail the file.
pub(crate) fn parse_shape(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<VoxelShape> {
    let mut shape: VoxelShape = read_payload_with(payload, cx, d, |read| read.texts(TEXTS))?;
    // Each of these is an error in `d`, and an error rejects the document.
    constraints(
        shape.description.identifier.as_str(),
        shape.shape.boxes.len(),
        &cx.at,
        d,
    );
    shape.source_format_version = cx.original_version;
    Some(shape)
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
            Diagnostic::of(&core_codes::INVALID_VALUE).at(at.with(&["description", "identifier"])),
        );
    }
    // A shape needs a box; the game's text for it is not reproduced here.
    if boxes < MIN_BOXES {
        d.push(Diagnostic::of(&core_codes::INVALID_VALUE).at(at.with(&["shape", "boxes"])));
    }
    if boxes > MAX_BOXES {
        d.push(Diagnostic::of(&codes::TOO_MANY_BOXES).at(at.with(&["shape", "boxes"])));
    }
}

/// One corner (vanilla's `Vec3`; `schema::read_with` of a box's `min` and `max`), in the two
/// forms the game tries in this order: `[x, y, z]`, exactly three numbers; then an object with
/// the optional members `x`, `y` and `z`, an absent one 0. `None` (with the errors told) for
/// anything else. A member no axis takes is an unknown member.
// The game logs the help text when the array form fails and clears it when the members load.
// Every finding is told, none stops the read: a file's every bad corner is reported.
pub(crate) fn corner(node: &Node<'_>, read: &mut Read<'_>) -> Option<[f32; 3]> {
    if read.stopped() {
        return None;
    }
    if let Some(array) = node.value().as_array()
        && array.len() == 3
        && let [Some(x), Some(y), Some(z)] =
            [0, 1, 2].map(|index| array.get(index).and_then(number))
    {
        return Some([x, y, z]);
    }
    let error = |node: &Node<'_>, code: Code| {
        let mut finding = Diagnostic::error(code).at(node.path().clone());
        finding.location.span = node.span();
        finding
    };
    let expected = error(node, Code::from(&codes::EXPECTED_X_Y_Z));
    if !node.value().is_object() {
        read.push(expected);
        read.push(error(node, cereal::expected_object()));
        return None;
    }
    let members = read.object(node)?;
    let mut at_axis = [0.0; 3];
    let mut mistyped = Vec::new();
    let mut unbound = Vec::new();
    for (key, member) in members.iter() {
        let Some(axis) = ["x", "y", "z"].iter().position(|name| *name == key) else {
            unbound.push(member);
            continue;
        };
        match number(member.value()) {
            Some(value) => at_axis[axis] = value,
            None => mistyped.push(member),
        }
    }
    if !mistyped.is_empty() {
        read.push(expected);
    }
    for member in &mistyped {
        read.push(error(member, Code::from(&core_codes::INVALID_TYPE)));
    }
    for member in &unbound {
        read.unknown_member(member, UnknownRule::Keep);
    }
    // With a finding above the value is not used: an error told rejects the document.
    Some(at_axis)
}

/// A JSON number (an integer or a real token) as the `f32` nearest to it; `None` for any other
/// JSON type and for a number no finite `f32` is near (as the crate's strict floats read).
fn number(json: &Value) -> Option<f32> {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    let x = json.as_number()?.to_f64_lossy() as f32;
    x.is_finite().then_some(x)
}
