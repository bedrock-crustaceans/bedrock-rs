//! `minecraft:collision_box` of the 1.26.60 document: `true` / `false` or up to 16
//! boxes, each by `origin`/`size` (the 1.26.20 reading) or by `min`/`max`.

use bedrock_addon_core::json::{StrictFloat, Value};
use facet::Facet;
use facet_json::RawJson;

use super::BlockComponent;
use super::types::{Vec3, vec3};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::version::FormatVersion;

/// `minecraft:collision_box`: what entities collide with.
///
/// JSON: `true` / `false`, or an array of at most 16 boxes. A 1.26.20 document may write one
/// box as an object; the 1.26.20 → 1.26.60 step makes it an array of one, and a document of
/// 1.26.60 or later may not: the 1.26.60 component is a `bool` or an array of boxes, with no
/// single-object alternative.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum CollisionBox {
    /// The full block (`true`) or none (`false`).
    Enabled(bool),
    /// The boxes.
    Boxes(Vec<CollisionBoundingBox>),
}

impl CollisionBox {
    /// The boxes it describes: none for `false`, the full block for `true`.
    pub fn boxes(&self) -> Vec<CollisionBoundingBox> {
        match self {
            CollisionBox::Enabled(true) => vec![CollisionBoundingBox::default()],
            CollisionBox::Enabled(false) => Vec::new(),
            CollisionBox::Boxes(boxes) => boxes.clone(),
        }
    }
}

/// One box of `minecraft:collision_box` in sixteenths of a block.
///
/// JSON: `{"origin", "size"}` (relative to the block's bottom centre, as 1.26.20 reads a box)
/// or `{"min", "max"}` (from the block's bottom-left corner, 1.26.60); every member optional,
/// and the two pairs exclusive. With neither pair the box is the full block. A box of a
/// document below 1.26.60 is converted to `min`/`max` by the upgrade
/// ([`CollisionBoundingBox::from_origin_size`]).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct CollisionBoundingBox {
    /// The corner nearest `[-8, 0, -8]` (JSON `origin`); x and z in `-8..=8`, y in `0..=24`.
    #[facet(default)]
    pub origin: Option<Vec3>,
    /// The extent (JSON `size`); x and z in `-16..=16`, y in `-24..=24`.
    #[facet(default)]
    pub size: Option<Vec3>,
    /// The minimum corner (JSON `min`, 1.26.60); x and z in `0..=16`, y in `0..=24`.
    #[facet(default)]
    pub min: Option<Vec3>,
    /// The maximum corner (JSON `max`, 1.26.60); x and z in `0..=16`, y in `0..=24`.
    #[facet(default)]
    pub max: Option<Vec3>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl CollisionBoundingBox {
    /// The minimum corner of the 1.26.20 reading: `origin`, `[-8, 0, -8]` when absent.
    pub fn origin(&self) -> Vec3 {
        self.origin.unwrap_or(vec3(-8.0, 0.0, -8.0))
    }

    /// The extent of the 1.26.20 reading: `size`, `[16, 16, 16]` when absent.
    pub fn size(&self) -> Vec3 {
        self.size.unwrap_or(vec3(16.0, 16.0, 16.0))
    }

    /// Whether the box is written by `min`/`max` (either of them).
    pub fn has_min_max(&self) -> bool {
        self.min.is_some() || self.max.is_some()
    }

    /// The box `origin`/`size` names as the 1.26.20 → 1.26.60 step gives it.
    /// `a = origin + (8, 0, 8)`, `b = a + size`, `lo`/`hi` their componentwise minimum and
    /// maximum, `min = (16 − hi.x, lo.y, lo.z)`, `max = (16 − lo.x, hi.y, hi.z)`: the x axis
    /// is mirrored. Only `min` and `max` are set.
    pub fn from_origin_size(origin: Vec3, size: Vec3) -> CollisionBoundingBox {
        let (min, max) = legacy_aabb(origin, size);
        CollisionBoundingBox {
            min: Some(to_vec3(min)),
            max: Some(to_vec3(max)),
            ..CollisionBoundingBox::default()
        }
    }

    /// The box's corners in sixteenths from the block's bottom-left corner: `min`/`max` when
    /// either is written (an absent one `[0, 0, 0]` or `[16, 16, 16]`, the game's
    /// constructor defaults), each axis sorted so the lower value is the minimum; else the conversion of
    /// `origin`/`size` (with their defaults).
    pub fn corners(&self) -> ([f32; 3], [f32; 3]) {
        if self.has_min_max() {
            // The game sorts each axis, so a `min` above its `max` is still a box.
            let a = self.min.map_or([0.0; 3], from_vec3);
            let b = self.max.map_or([16.0; 3], from_vec3);
            (
                [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])],
                [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])],
            )
        } else {
            legacy_aabb(self.origin(), self.size())
        }
    }

    /// The box as the 1.26.20 reading names it, `(origin, size)` with a size of no negative
    /// axis: the inverse of [`from_origin_size`](Self::from_origin_size) on the box's
    /// [`corners`](Self::corners), `origin = (8 − max.x, min.y, min.z − 8)`,
    /// `size = max − min`.
    pub fn legacy_origin_size(&self) -> (Vec3, Vec3) {
        let (min, max) = self.corners();
        let origin = [8.0 - max[0], min[1], min[2] - 8.0];
        let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        (to_vec3(origin), to_vec3(size))
    }
}

/// The version of the 1.26.60 document, from which `min`/`max` are bound and the one-box
/// object is not.
const V1_26_60: FormatVersion = FormatVersion::new(1, 26, 60);
/// The first version vanilla reads with the cereal document.
const V1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);
/// The most boxes the game's constraint allows; the 1.26.20 constraint allows the same number.
const MAX_BOXES: usize = 16;

/// What the reader of a document of `version` says of the section's `minecraft:collision_box`
/// (`raw`, at `at`), and whether it is one box written as an object:
///
/// - below 1.26.60 (the 1.26.20 struct, and the conversion of a legacy document): `min` and
///   `max` are no members of the 1.26.20 box (an `UnrecognizedField` each, which the strict
///   slice makes an error; the value is kept with the box's unknown members); in a cereal
///   document the 1.26.20 constraint holds: at most 16 boxes, each member in its range, each
///   box inside `(0, 0, 0)`–`(16, 24, 16)` with the 1.26.20 text, an error that fails the
///   block (the game refuses to load a block whose collision description overflows these limits);
/// - from 1.26.60: the object is no alternative of the component (`InvalidType`, an error;
///   the component's schema is a variant of a bool or a vector of boxes, so an object matches
///   neither), and the 1.26.60 constraint holds.
pub(crate) fn read_collision_box(
    map: &mut ComponentMap<BlockComponent>,
    raw: Option<&RawJson<'_>>,
    at: &JsonPath,
    version: FormatVersion,
    d: &mut Diagnostics,
) -> bool {
    let one = raw.is_some_and(|raw| raw.as_str().trim_start().starts_with('{'));
    let Some(CollisionBox::Boxes(boxes)) = map.get_mut::<CollisionBox>() else {
        return false;
    };
    let box_at = |index: usize| {
        let mut path = at.clone();
        if !one {
            path.push_index(index);
        }
        path
    };
    if version >= V1_26_60 {
        if one {
            d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(at.clone()));
            return false;
        }
        newest_constraint(boxes, at, d);
        return false;
    }
    for (index, b) in boxes.iter_mut().enumerate() {
        for (name, slot) in [("min", &mut b.min), ("max", &mut b.max)] {
            if let Some(value) = slot.take() {
                let mut path = box_at(index);
                path.push(name);
                // `min` and `max` are unknown members of this box. The strict slice turns an
                // unknown member into the error that refuses the block. Below 1.26.20 the
                // game's description binds `origin` and `size` alone and says nothing of an
                // unknown member, so the crate reports it as an information.
                let severity = if version >= V1_26_20 {
                    Severity::Warning
                } else {
                    Severity::Info
                };
                d.push(
                    Diagnostic::error(DiagnosticCode::UnrecognizedField { text: None })
                        .severity(severity)
                        .at(path),
                );
                let json =
                    bedrock_addon_core::versioning::write_payload(&value).unwrap_or(Value::NULL);
                b.extra.insert(name.to_owned(), json);
            }
        }
    }
    if version >= V1_26_20
        && let Some(finding) = legacy_constraint(boxes, one, at)
    {
        d.push(finding);
    }
    one
}

/// The 1.26.20 constraint on the boxes of a section (`one`: written as one object): `None`
/// when they keep it. The ranges of the members are the 1.26.60 box's (a 1.26.20 document
/// with a size of 19 in y loads, so the 1.26.20 ranges are no tighter), the bounds `(0, 0, 0)`–`(16, 24, 16)` with x and z shifted by 8.
pub(crate) fn legacy_constraint(
    boxes: &[CollisionBoundingBox],
    one: bool,
    at: &JsonPath,
) -> Option<Diagnostic> {
    if boxes.len() > MAX_BOXES {
        return Some(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(at.clone()));
    }
    let ranged = boxes
        .iter()
        .all(|b| in_ranges(b.origin, ORIGIN_RANGES) && in_ranges(b.size, SIZE_RANGES));
    let inside = boxes.iter().all(|b| within(b.corners()));
    if ranged && inside {
        return None;
    }
    let template = if one {
        "\"origin\" + \"size\", must be inside the following limit: {} and {}"
    } else {
        "The total of each \"origin\" + \"size\" pair, must be inside the following limit: {} and {}"
    };
    Some(
        Diagnostic::error(DiagnosticCode::InvalidValue {
            text: Some(VanillaText::Formatted {
                template,
                args: vec!["(0, 0, 0)".to_owned(), "(16, 24, 16)".to_owned()],
            }),
        })
        .at(at.clone()),
    )
}

/// The 1.26.60 constraint: at most 16 boxes; each member in its
/// range; `origin`/`size` not with `min`/`max`; the union of the other boxes inside
/// `MIN_BOUNDS`–`MAX_BOUNDS`. Each break is an error.
fn newest_constraint(boxes: &[CollisionBoundingBox], at: &JsonPath, d: &mut Diagnostics) {
    if boxes.len() > MAX_BOXES {
        d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(at.clone()));
        return;
    }
    // The union of the boxes that are no combined `origin`/`size` + `min`/`max` box: the
    // game logs a combined box and goes on without it.
    let mut union = Vec::with_capacity(boxes.len());
    for (index, b) in boxes.iter().enumerate() {
        let path = at.index(index);
        for (name, value, ranges) in [
            ("origin", b.origin, ORIGIN_RANGES),
            ("size", b.size, SIZE_RANGES),
            ("min", b.min, MIN_MAX_RANGES),
            ("max", b.max, MIN_MAX_RANGES),
        ] {
            if !in_ranges(value, ranges) {
                let member = path.child(name);
                d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(member));
            }
        }
        if (b.origin.is_some() || b.size.is_some()) && b.has_min_max() {
            d.push(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Fixed(
                        "\"origin\"/\"size\" cannot be combined with \"min\"/\"max\"",
                    )),
                })
                .at(path),
            );
            continue;
        }
        union.push(b.corners());
    }
    // The union must lie within the bounds; the game's one message is "Invalid box bounds"
    // inserted at the start of the area text.
    if !union.iter().all(|corners| within(*corners)) {
        d.push(Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Formatted {
                    template: "Invalid box boundsThe area covered by all boxes must be within {} to {}",
                    args: vec!["(0, 0, 0)".to_owned(), "(16, 24, 16)".to_owned()],
                }),
            }).at(at.clone()));
    }
}

/// `origin`: x and z in `-8..=8`, y in `0..=24`.
const ORIGIN_RANGES: [(f32, f32); 3] = [(-8.0, 8.0), (0.0, 24.0), (-8.0, 8.0)];
/// `size`: x and z in `-16..=16`, y in `-24..=24`.
const SIZE_RANGES: [(f32, f32); 3] = [(-16.0, 16.0), (-24.0, 24.0), (-16.0, 16.0)];
/// `min` and `max`: x and z in `0..=16`, y in `0..=24`.
const MIN_MAX_RANGES: [(f32, f32); 3] = [(0.0, 16.0), (0.0, 24.0), (0.0, 16.0)];

/// Whether each axis of a written member is in its range (an absent member is).
fn in_ranges(value: Option<Vec3>, ranges: [(f32, f32); 3]) -> bool {
    value.is_none_or(|v| (0..3).all(|axis| (ranges[axis].0..=ranges[axis].1).contains(&v[axis].0)))
}

/// Whether the corners lie inside `(0, 0, 0)`–`(16, 24, 16)`.
fn within((min, max): ([f32; 3], [f32; 3])) -> bool {
    let limit = [16.0, 24.0, 16.0];
    (0..3).all(|axis| {
        (0.0..=limit[axis]).contains(&min[axis]) && (0.0..=limit[axis]).contains(&max[axis])
    })
}

/// Converts a legacy `origin`/`size` box to its `min`/`max` corners, as the game's upgrade does.
fn legacy_aabb(origin: Vec3, size: Vec3) -> ([f32; 3], [f32; 3]) {
    let origin = from_vec3(origin);
    let size = from_vec3(size);
    let a = [origin[0] + 8.0, origin[1], origin[2] + 8.0];
    let b = [a[0] + size[0], a[1] + size[1], a[2] + size[2]];
    let lo = [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])];
    let hi = [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])];
    ([16.0 - hi[0], lo[1], lo[2]], [16.0 - lo[0], hi[1], hi[2]])
}

/// The three floats of a [`Vec3`].
fn from_vec3(v: Vec3) -> [f32; 3] {
    [v[0].0, v[1].0, v[2].0]
}

/// Three floats as a [`Vec3`].
fn to_vec3(v: [f32; 3]) -> Vec3 {
    [StrictFloat(v[0]), StrictFloat(v[1]), StrictFloat(v[2])]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A legacy box of origin `(0, 0, 0)` and size `(8, 12, 8)` loads as the game's own tests
    /// expect: `min` is (0, 0, 8) and `max` is (8, 12, 16).
    #[test]
    fn the_step_mirrors_x_as_the_game_expects() {
        let b = CollisionBoundingBox::from_origin_size(vec3(0.0, 0.0, 0.0), vec3(8.0, 12.0, 8.0));
        assert_eq!(b.min, Some(vec3(0.0, 0.0, 8.0)));
        assert_eq!(b.max, Some(vec3(8.0, 12.0, 16.0)));
        assert_eq!((b.origin, b.size), (None, None));
    }

    /// Two examples of the conversion.
    #[test]
    fn worked_examples_convert_as_stated() {
        let b =
            CollisionBoundingBox::from_origin_size(vec3(-8.0, 0.0, -8.0), vec3(4.0, 16.0, 16.0));
        assert_eq!(b.corners(), ([12.0, 0.0, 0.0], [16.0, 16.0, 16.0]));
        let full =
            CollisionBoundingBox::from_origin_size(vec3(-8.0, 0.0, -8.0), vec3(16.0, 16.0, 16.0));
        assert_eq!(full.corners(), ([0.0, 0.0, 0.0], [16.0, 16.0, 16.0]));
    }

    /// The writer's inverse gives back a box the step converts to the same corners.
    #[test]
    fn the_legacy_form_converts_back_to_the_same_corners() {
        for (origin, size) in [
            (vec3(-5.0, 4.0, 1.0), vec3(10.0, 7.0, 7.0)),
            (vec3(8.0, 0.0, 8.0), vec3(-16.0, 16.0, -16.0)),
            (vec3(-8.0, 5.0, -8.0), vec3(16.0, 19.0, 16.0)),
        ] {
            let converted = CollisionBoundingBox::from_origin_size(origin, size);
            let (o, s) = converted.legacy_origin_size();
            assert_eq!(
                CollisionBoundingBox::from_origin_size(o, s).corners(),
                converted.corners()
            );
        }
    }
}
