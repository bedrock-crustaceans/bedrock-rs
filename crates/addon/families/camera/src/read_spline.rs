//! Reading and writing camera spline files.

use super::parse::VEC3;
use super::{CameraSpline, Spline, texts};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, write_payload};
use bedrock_addon_support::chain_file as file;
use bedrock_addon_support::shape::{
    self, Member, Shape, child, element, namespace_name, opt, req, violation,
};

/// One control point.
const CONTROL_POINT: &[Member] = &[req("position", VEC3)];

/// One progress key frame.
const PROGRESS_KEY_FRAME: &[Member] = &[
    req("progress", Shape::Float),
    req("time", Shape::Float),
    opt("easing", Shape::Str),
];

/// One rotation key frame.
const ROTATION_KEY_FRAME: &[Member] = &[
    req("rotation", VEC3),
    req("time", Shape::Float),
    opt("easing", Shape::Str),
];

/// One spline. `spline_type` is a string with a pattern, not an enum: a name outside the two
/// is a constraint violation with its own text.
const SPLINE: &[Member] = &[
    req("name", Shape::Str),
    req("total_time", Shape::Float),
    opt("spline_type", Shape::Str),
    req("control_points", Shape::List(&Shape::Object(CONTROL_POINT))),
    req(
        "progress_key_frames",
        Shape::List(&Shape::Object(PROGRESS_KEY_FRAME)),
    ),
    req(
        "rotation_key_frames",
        Shape::List(&Shape::Object(ROTATION_KEY_FRAME)),
    ),
];

/// The spline file's payload.
const SPLINES: &[Member] = &[req("splines", Shape::List(&Shape::Object(SPLINE)))];

/// The `spline_type` members of the payload that are neither `catmullrom` nor `linear`
/// (`^(?:catmullrom|linear)$`), by spline index: checked on the JSON, before the typed parse
/// reads the member as an enum.
fn spline_types(payload: &Value, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let splines = payload
        .as_object()
        .and_then(|object| object.get("splines"))
        .and_then(Value::as_array);
    let mut passed = true;
    for (i, spline) in splines.into_iter().flatten().enumerate() {
        let written = spline
            .as_object()
            .and_then(|object| object.get("spline_type"))
            .and_then(Value::as_string);
        if written.is_some_and(|name| !matches!(name.as_str(), "catmullrom" | "linear")) {
            let at = child(&element(&child(at, "splines"), i), "spline_type");
            violation(texts::SPLINE_TYPE, at, d);
            passed = false;
        }
    }
    passed
}

impl Spline {
    /// The bind constraints of one spline, in bind order. `false` when one failed.
    fn check(&self, at: &JsonPath, d: &mut Diagnostics) -> bool {
        let mut passed = true;
        if !namespace_name(&self.name.0) {
            violation(texts::SPLINE_NAME, child(at, "name"), d);
            passed = false;
        }
        if self.total_time.0 < 0.0 {
            violation(texts::SPLINE_TOTAL_TIME, child(at, "total_time"), d);
            passed = false;
        }
        let frames = child(at, "progress_key_frames");
        for (i, frame) in self.progress_key_frames.iter().enumerate() {
            let at = element(&frames, i);
            if !(0.0..=1.0).contains(&frame.progress.0) {
                violation(texts::SPLINE_PROGRESS, child(&at, "progress"), d);
                passed = false;
            }
            if frame.time.0 < 0.0 {
                violation(texts::SPLINE_PROGRESS_TIME, child(&at, "time"), d);
                passed = false;
            }
        }
        let frames = child(at, "rotation_key_frames");
        for (i, frame) in self.rotation_key_frames.iter().enumerate() {
            if frame.time.0 < 0.0 {
                let at = child(&element(&frames, i), "time");
                violation(texts::SPLINE_ROTATION_TIME, at, d);
                passed = false;
            }
        }
        passed
    }
}

impl VersionStruct for CameraSpline {
    /// Reads the payload (the envelope is checked from 1.26.0, the only version range).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let file_shape = Shape::Object(SPLINES);
        // The pattern of `spline_type` is checked on a payload of the right shape, before the
        // typed parse reads the member as an enum.
        if let Some(json) = file::last_wins(payload.text)
            && shape::check(
                &json,
                &file_shape,
                &cx.at,
                shape::NOT_IN_SCHEMA,
                &mut Diagnostics::default(),
            )
            && !spline_types(&json, &cx.at, d)
        {
            return None;
        }
        let file: Self = file::typed(payload, &file_shape, None, shape::NOT_IN_SCHEMA, cx, d)?;
        let splines = child(&cx.at, "splines");
        let mut passed = true;
        for (i, spline) in file.splines.iter().enumerate() {
            passed &= spline.check(&element(&splines, i), d);
        }
        passed.then_some(file)
    }

    /// The payload.
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}
