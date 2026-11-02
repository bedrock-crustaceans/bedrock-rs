//! `textures/ui/<image>.json`: the side file of a UI image, a nine-slice description or an
//! Aseprite sprite sheet, and the stack's loader of them (`rp.ui_texture_infos()`): each the
//! highest pack's copy whichever pack supplies the image.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod content;
mod load;
mod write;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::jsoncpp::{member, number};

#[doc(hidden)]
pub use load::load_texture_infos;

/// The side file of a UI image.
///
/// JSON: with a `meta` member an Aseprite sheet (`{"frames": [..], "meta": {..}}`), otherwise
/// a nine-slice description (`{"nineslice_size": 4, "base_size": [16, 16]}`). The game loads
/// the side file of the highest pack that has one, whichever pack supplies the image.
/// A `meta` member that is present and not null selects the sheet.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum UiTextureInfo {
    /// An Aseprite sprite sheet the game accepts.
    Aseprite(AsepriteSheet),
    /// A nine-slice description.
    Nineslice(Nineslice),
    /// A file with `meta` that fails one of the sheet's tests, as written: the game rejects
    /// the sheet (base size 0 × 0, no frames).
    RejectedAseprite(Value),
}

/// A nine-slice description.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Nineslice {
    /// The border widths (JSON `nineslice_size`: a number, or `[left, top, right, bottom]`).
    pub nineslice_size: Option<SliceSize>,
    /// The size the UV rectangle is measured against (JSON `base_size`: `[width, height]`).
    pub base_size: Option<[f32; 2]>,
    /// JSON `tiled`. Vanilla files carry it; the game does not read it.
    pub tiled: Option<bool>,
    /// The other members. Also a `nineslice_size`, `base_size` or `tiled` as the file wrote
    /// it, when the typed field cannot say it (a value the game ignores, `4.0`, a third
    /// element, a `null`): it is written back while the typed field still is its reading, and
    /// the typed field is written once it was changed.
    pub extra: Extra,
}

/// The border widths of a nine-slice image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliceSize {
    /// One width for all four sides (a JSON number).
    All(f32),
    /// `[left, top, right, bottom]` (a JSON array of exactly four).
    Sides([f32; 4]),
}

/// An Aseprite sprite sheet (the array export; the game rejects the hash export).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AsepriteSheet {
    /// The frames (JSON `frames`), in play order.
    pub frames: Vec<AsepriteFrame>,
    /// The sheet's description (JSON `meta`).
    pub meta: AsepriteMeta,
    /// The other members.
    pub extra: Extra,
}

/// `meta` of an Aseprite sheet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AsepriteMeta {
    /// The image name (JSON `image`); the game only checks that it is a string.
    pub image: String,
    /// The size of the sheet image, `[w, h]` (JSON `size`: `{"w", "h"}`): the base UV size.
    pub size: [i32; 2],
    /// The other members of `meta`, and of `meta.size` under `size`.
    pub extra: Extra,
}

/// One frame of an Aseprite sheet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AsepriteFrame {
    /// Where the frame starts, `[x, y]` (JSON `frame`: `{"x", "y"}`, each at least 0).
    pub position: [i32; 2],
    /// How long the frame shows, in milliseconds (JSON `duration`, at least 1).
    pub duration: i32,
    /// The other members of the frame; the other members of `frame` (`w`, `h`, which the
    /// game does not read) under `frame`.
    pub extra: Extra,
}

impl UiTextureInfo {
    /// The size the UV rectangle is measured against: a nine-slice's `base_size`, a sheet's
    /// `meta.size`; 0 × 0 when absent and for a rejected sheet.
    #[must_use]
    pub fn base_size(&self) -> [f32; 2] {
        match self {
            #[allow(clippy::cast_precision_loss, reason = "an image size is small")]
            Self::Aseprite(sheet) => [sheet.meta.size[0] as f32, sheet.meta.size[1] as f32],
            Self::Nineslice(nineslice) => nineslice.base_size.unwrap_or([0.0, 0.0]),
            Self::RejectedAseprite(_) => [0.0, 0.0],
        }
    }

    /// The frames of an accepted sheet; none otherwise.
    #[must_use]
    pub fn frames(&self) -> &[AsepriteFrame] {
        match self {
            Self::Aseprite(sheet) => &sheet.frames,
            _ => &[],
        }
    }

    /// The summed duration of the frames, in milliseconds.
    #[must_use]
    pub fn total_duration(&self) -> i64 {
        self.frames()
            .iter()
            .map(|frame| i64::from(frame.duration))
            .sum()
    }
}

/// A finding at `path`: an information of the crate's own (the loader is silent) or the text of
/// the game's debug assert.
fn warn(d: &mut Diagnostics, code: DiagnosticCode, path: &[&str]) {
    let mut json_path = JsonPath::new();
    for segment in path {
        json_path.push(segment);
    }
    d.push(Diagnostic::new(
        code,
        Severity::Info,
        Phase::Validate,
        Location::new(None, None, json_path, None),
    ));
}

fn invalid(d: &mut Diagnostics, text: Option<&'static str>, path: &[&str]) {
    warn(
        d,
        DiagnosticCode::InvalidValue {
            text: text.map(VanillaText::Fixed),
        },
        path,
    );
}

/// jsoncpp's `asFloat` with default 0: a number converted, a bool 0 or 1, anything else 0.
#[allow(clippy::cast_possible_truncation, reason = "the game reads an f32")]
fn as_float(value: &Value) -> f32 {
    match value.as_bool() {
        Some(flag) => f32::from(u8::from(flag)),
        None => number(value).unwrap_or(0.0) as f32,
    }
}

/// jsoncpp's `isInt`: an integral number in `i32`.
fn as_int(value: Option<&Value>) -> Option<i32> {
    let number = value?.as_number()?;
    if number.is_float() {
        let float = number.to_f64()?;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "checked to be integral and in range"
        )]
        return (float.fract() == 0.0 && float.abs() <= f64::from(i32::MAX))
            .then_some(float as i32);
    }
    number.to_i32()
}

/// Every member of `value` but the ones in `read`.
fn others(value: &Value, read: &[&str]) -> Extra {
    let mut extra = Extra::default();
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            if !read.contains(&key.as_str()) {
                extra.insert(key.as_str().to_owned(), member.clone());
            }
        }
    }
    extra
}

impl UiTextureInfo {
    /// Reads a side file's root object.
    pub(crate) fn from_value(root: &Value, d: &mut Diagnostics) -> Self {
        if member(root, "meta").is_some() {
            match read_sheet(root) {
                Ok(sheet) => Self::Aseprite(sheet),
                Err(path) => {
                    // The sheet is rejected; the finding names the failing member.
                    let path: Vec<&str> = path.iter().map(String::as_str).collect();
                    invalid(d, None, &path);
                    Self::RejectedAseprite(root.clone())
                }
            }
        } else {
            Self::Nineslice(read_nineslice(root, d))
        }
    }
}

/// The members a nine-slice file is read for.
pub(crate) const NINESLICE_KEYS: [&str; 3] = ["nineslice_size", "base_size", "tiled"];

/// `nineslice_size` as the game reads it: a number (a bool counts as one) for all four sides,
/// or an array of exactly four, each element through `asFloat`.
pub(crate) fn read_slice_size(value: &Value) -> Option<SliceSize> {
    if value.is_number() || value.is_bool() {
        return Some(SliceSize::All(as_float(value)));
    }
    match value.as_array().map(facet_value::VArray::as_slice) {
        Some([left, top, right, bottom]) => {
            Some(SliceSize::Sides([left, top, right, bottom].map(as_float)))
        }
        _ => None,
    }
}

/// `base_size` as the game reads it: the first two elements of an array, through `asFloat`.
pub(crate) fn read_base_size(value: &Value) -> Option<[f32; 2]> {
    match value.as_array().map(facet_value::VArray::as_slice) {
        Some([width, height, ..]) => Some([as_float(width), as_float(height)]),
        _ => None,
    }
}

/// A float as the shortest number that reads back to it; an integral one as an integer, but
/// the negative zero, which is written `-0.0`.
pub(crate) fn float(value: f32) -> Value {
    if value == 0.0 && value.is_sign_negative() {
        // A negative zero keeps its sign: an integer has none.
        return Value::from(-0.0_f64);
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "checked to be integral and small"
    )]
    if value.fract() == 0.0 && value.abs() < 16_777_216.0 {
        Value::from(value as i64)
    } else {
        Value::from(value.to_string().parse::<f64>().unwrap_or(f64::from(value)))
    }
}

pub(crate) fn floats(values: &[f32]) -> Value {
    let mut array = facet_value::VArray::new();
    for value in values {
        array.push(float(*value));
    }
    array.into_value()
}

pub(crate) fn slice_size_value(size: &SliceSize) -> Value {
    match size {
        SliceSize::All(all) => float(*all),
        SliceSize::Sides(sides) => floats(sides),
    }
}

/// Whether the typed form of a member writes back as the member was written, text for text.
fn writes_back(written: &Value, typed: Option<Value>) -> bool {
    let text = |value: &Value| bedrock_addon_core::json::to_json(value).unwrap_or_default();
    typed.is_some_and(|typed| text(&typed) == text(written))
}

/// Reads a nine-slice description.
fn read_nineslice(root: &Value, d: &mut Diagnostics) -> Nineslice {
    let mut nineslice = Nineslice {
        extra: others(root, &NINESLICE_KEYS),
        ..Nineslice::default()
    };
    let written = |key: &str| root.as_object().and_then(|object| object.get(key));
    // A member whose typed form would not write back as it was written (a value the game
    // ignores, `4.0`, a third element, a `null`) is kept as written beside the typed form.
    let keep = |nineslice: &mut Nineslice, key: &str, written: &Value, typed: Option<Value>| {
        if !writes_back(written, typed) {
            nineslice.extra.insert(key.to_owned(), written.clone());
        }
    };
    if let Some(size) = written("nineslice_size") {
        nineslice.nineslice_size = read_slice_size(size);
        match nineslice.nineslice_size {
            None if size.is_null() => {}
            // Neither a number nor an array of four: ignored.
            None => invalid(d, None, &["nineslice_size"]),
            Some(read) => {
                if size.is_bool() {
                    // The game counts a bool as numeric and reads it as 1 or 0.
                    invalid(d, None, &["nineslice_size"]);
                }
                let negative = match read {
                    SliceSize::All(all) => all < 0.0,
                    SliceSize::Sides(sides) => sides.iter().any(|side| *side < 0.0),
                };
                if negative {
                    // A debug assertion in vanilla; the value is kept.
                    invalid(d, Some("NineSlice cannot be negative"), &["nineslice_size"]);
                }
            }
        }
        let typed = nineslice.nineslice_size.as_ref().map(slice_size_value);
        keep(&mut nineslice, "nineslice_size", size, typed);
    }
    if let Some(size) = written("base_size") {
        nineslice.base_size = read_base_size(size);
        match nineslice.base_size {
            None if size.is_null() => {}
            // Not an array of at least two: ignored.
            None => invalid(d, None, &["base_size"]),
            Some(base) => {
                if base.iter().any(|side| *side < 0.0) {
                    // A debug assertion in vanilla; the value is kept.
                    invalid(d, Some("UV Base size can't be negative!"), &["base_size"]);
                }
            }
        }
        let typed = nineslice.base_size.as_ref().map(|base| floats(base));
        keep(&mut nineslice, "base_size", size, typed);
    }
    if let Some(tiled) = written("tiled") {
        nineslice.tiled = tiled.as_bool();
        let typed = nineslice.tiled.map(Value::from);
        keep(&mut nineslice, "tiled", tiled, typed);
    }
    nineslice
}

/// The sheet, or the path of the first member that fails a test.
fn read_sheet(root: &Value) -> Result<AsepriteSheet, Vec<String>> {
    let fail = |path: &[&str]| path.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let meta = member(root, "meta")
        .filter(|meta| meta.is_object())
        .ok_or_else(|| fail(&["meta"]))?;
    let image = member(meta, "image")
        .and_then(Value::as_string)
        .ok_or_else(|| fail(&["meta", "image"]))?;
    let size = member(meta, "size")
        .filter(|size| size.is_object())
        .ok_or_else(|| fail(&["meta", "size"]))?;
    let w = as_int(member(size, "w")).ok_or_else(|| fail(&["meta", "size", "w"]))?;
    let h = as_int(member(size, "h")).ok_or_else(|| fail(&["meta", "size", "h"]))?;
    let mut meta_extra = others(meta, &["image", "size"]);
    let size_extra = others(size, &["w", "h"]);
    if !size_extra.is_empty() {
        meta_extra.insert("size".to_owned(), extra_value(&size_extra));
    }
    let frames = member(root, "frames")
        .and_then(Value::as_array)
        .ok_or_else(|| fail(&["frames"]))?;
    let mut read = Vec::new();
    for (index, entry) in frames.as_slice().iter().enumerate() {
        let index = index.to_string();
        let duration = as_int(member(entry, "duration"))
            .filter(|duration| *duration >= 1)
            .ok_or_else(|| fail(&["frames", &index, "duration"]))?;
        let frame = member(entry, "frame")
            .filter(|frame| frame.is_object())
            .ok_or_else(|| fail(&["frames", &index, "frame"]))?;
        let axis = |key: &str| {
            as_int(member(frame, key))
                .filter(|at| *at >= 0)
                .ok_or_else(|| fail(&["frames", &index, "frame", key]))
        };
        let position = [axis("x")?, axis("y")?];
        let mut extra = others(entry, &["frame", "duration"]);
        let frame_extra = others(frame, &["x", "y"]);
        if !frame_extra.is_empty() {
            extra.insert("frame".to_owned(), extra_value(&frame_extra));
        }
        read.push(AsepriteFrame {
            position,
            duration,
            extra,
        });
    }
    Ok(AsepriteSheet {
        frames: read,
        meta: AsepriteMeta {
            image: image.as_str().to_owned(),
            size: [w, h],
            extra: meta_extra,
        },
        extra: others(root, &["frames", "meta"]),
    })
}

/// The members of `extra` as an object.
pub(crate) fn extra_value(extra: &Extra) -> Value {
    let mut object = facet_value::VObject::new();
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
    object.into_value()
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "the values are read from JSON, not computed"
)]
mod tests {
    use super::*;

    fn read(text: &str) -> (UiTextureInfo, Diagnostics) {
        let root = bedrock_addon_support::jsoncpp::read(text.as_bytes()).unwrap();
        let mut d = Diagnostics::default();
        (UiTextureInfo::from_value(&root, &mut d), d)
    }

    fn nineslice(text: &str) -> (Nineslice, Diagnostics) {
        match read(text) {
            (UiTextureInfo::Nineslice(nineslice), d) => (nineslice, d),
            other => panic!("not a nine-slice: {other:?}"),
        }
    }

    fn texts(d: &Diagnostics) -> Vec<(String, Severity, String)> {
        d.iter()
            .map(|f| {
                (
                    f.location.json_path.to_string(),
                    f.severity,
                    f.code.to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn a_number_slices_all_sides_and_four_numbers_each() {
        let (all, d) = nineslice(r#"{ "nineslice_size": 1, "base_size": [4, 4] }"#);
        assert_eq!(all.nineslice_size, Some(SliceSize::All(1.0)));
        assert_eq!(all.base_size, Some([4.0, 4.0]));
        assert!(d.is_empty());
        let (sides, d) = nineslice(r#"{ "nineslice_size": [1, 2, 1, 2], "base_size": [3, 5, 9] }"#);
        assert_eq!(
            sides.nineslice_size,
            Some(SliceSize::Sides([1.0, 2.0, 1.0, 2.0]))
        );
        assert_eq!(sides.base_size, Some([3.0, 5.0]));
        assert!(d.is_empty());
    }

    #[test]
    fn a_slice_size_of_another_shape_is_ignored_with_a_warning() {
        for text in [
            r#"{ "nineslice_size": [1, 2, 3] }"#,
            r#"{ "nineslice_size": [1, 2, 3, 4, 5] }"#,
            r#"{ "nineslice_size": "4" }"#,
        ] {
            let (nineslice, d) = nineslice(text);
            assert_eq!(nineslice.nineslice_size, None, "{text}");
            assert!(nineslice.extra.get("nineslice_size").is_some());
            assert_eq!(
                texts(&d),
                vec![(
                    "nineslice_size".to_owned(),
                    Severity::Info,
                    "invalid value".to_owned()
                )]
            );
        }
    }

    #[test]
    fn a_bool_slice_size_is_read_as_a_number_deviation() {
        // Vanilla accepts it silently as a numeric value; the crate warns.
        let (on, d) = nineslice(r#"{ "nineslice_size": true }"#);
        assert_eq!(on.nineslice_size, Some(SliceSize::All(1.0)));
        assert_eq!(d.len(), 1);
        assert!(matches!(
            d.iter().next().unwrap().code,
            DiagnosticCode::InvalidValue { text: None }
        ));
        let (off, _) = nineslice(r#"{ "nineslice_size": false }"#);
        assert_eq!(off.nineslice_size, Some(SliceSize::All(0.0)));
    }

    #[test]
    fn negative_sizes_are_kept_with_vanilla_s_assertion_texts() {
        let (slice, d) = nineslice(r#"{ "nineslice_size": [-1, 2, 1, 2] }"#);
        assert_eq!(
            slice.nineslice_size,
            Some(SliceSize::Sides([-1.0, 2.0, 1.0, 2.0]))
        );
        assert_eq!(
            texts(&d),
            vec![(
                "nineslice_size".to_owned(),
                Severity::Info,
                "NineSlice cannot be negative".to_owned()
            )]
        );
        let (all, d) = nineslice(r#"{ "nineslice_size": -0.5 }"#);
        assert_eq!(all.nineslice_size, Some(SliceSize::All(-0.5)));
        assert_eq!(d.len(), 1);
        assert!(nineslice(r#"{ "nineslice_size": 0 }"#).1.is_empty());
        let (base, d) = nineslice(r#"{ "base_size": [-1, 4] }"#);
        assert_eq!(base.base_size, Some([-1.0, 4.0]));
        assert_eq!(
            texts(&d),
            vec![(
                "base_size".to_owned(),
                Severity::Info,
                "UV Base size can't be negative!".to_owned()
            )]
        );
        let (base, d) = nineslice(r#"{ "base_size": [4, -1] }"#);
        assert_eq!(base.base_size, Some([4.0, -1.0]));
        assert_eq!(d.len(), 1);
        assert!(nineslice(r#"{ "base_size": [0, 0] }"#).1.is_empty());
    }

    #[test]
    fn a_base_size_that_is_no_pair_is_ignored_with_a_warning() {
        for text in [r#"{ "base_size": [4] }"#, r#"{ "base_size": 4 }"#] {
            let (nineslice, d) = nineslice(text);
            assert_eq!(nineslice.base_size, None);
            assert_eq!(texts(&d)[0].0, "base_size");
            assert_eq!(d.len(), 1);
        }
        // Elements are read as floats: a string is 0, a bool 0 or 1.
        let (lenient, d) = nineslice(r#"{ "base_size": ["a", true] }"#);
        assert_eq!(lenient.base_size, Some([0.0, 1.0]));
        assert!(d.is_empty());
    }

    #[test]
    fn tiled_is_kept_and_not_reported() {
        let (tiled, d) = nineslice(r#"{ "nineslice_size": 2, "tiled": true, "other": 1 }"#);
        assert_eq!(tiled.tiled, Some(true));
        assert!(tiled.extra.get("other").is_some());
        assert!(d.is_empty());
    }

    const SHEET: &str = r#"{ "frames": [
        { "filename": "a 0.png", "frame": { "x": 0, "y": 0, "w": 8, "h": 8 }, "duration": 100 },
        { "frame": { "x": 8, "y": 0 }, "duration": 50 } ],
      "meta": { "app": "aseprite", "image": "a.png", "size": { "w": 8, "h": 8 } } }"#;

    #[test]
    fn a_sheet_reads_its_cell_size_and_frames() {
        let (info, d) = read(SHEET);
        assert!(d.is_empty());
        assert_eq!(info.base_size(), [8.0, 8.0]);
        assert_eq!(info.frames().len(), 2);
        assert_eq!(info.frames()[1].position, [8, 0]);
        assert_eq!(info.total_duration(), 150);
        let UiTextureInfo::Aseprite(sheet) = info else {
            panic!("a sheet")
        };
        assert_eq!(sheet.meta.image, "a.png");
        // `frame.w` and `frame.h` are not read, and are kept
        assert!(sheet.frames[0].extra.get("frame").is_some());
        assert!(sheet.frames[1].extra.get("frame").is_none());
        assert!(sheet.meta.extra.get("app").is_some());
    }

    /// `SHEET` with `from` replaced by `to`: rejected at `path`.
    fn rejected(from: &str, to: &str, path: &str) {
        let text = SHEET.replacen(from, to, 1);
        assert_ne!(text, SHEET, "{from}");
        let (info, d) = read(&text);
        assert!(
            matches!(info, UiTextureInfo::RejectedAseprite(_)),
            "{to}: {info:?}"
        );
        assert_eq!(info.base_size(), [0.0, 0.0]);
        assert!(info.frames().is_empty());
        assert_eq!(
            texts(&d),
            vec![(path.to_owned(), Severity::Info, "invalid value".to_owned())],
            "{to}"
        );
    }

    #[test]
    fn each_failed_test_rejects_the_sheet_and_names_its_member() {
        rejected(
            r#""meta": { "app": "aseprite", "image": "a.png", "size": { "w": 8, "h": 8 } }"#,
            r#""meta": 5"#,
            "meta",
        );
        rejected(r#""image": "a.png""#, r#""image": 5"#, "meta/image");
        rejected(
            r#""size": { "w": 8, "h": 8 }"#,
            r#""size": [8, 8]"#,
            "meta/size",
        );
        rejected(
            r#""w": 8, "h": 8 } }"#,
            r#""w": 8.5, "h": 8 } }"#,
            "meta/size/w",
        );
        rejected(
            r#""w": 8, "h": 8 } }"#,
            r#""w": 8, "h": "8" } }"#,
            "meta/size/h",
        );
        rejected(
            r#""duration": 100"#,
            r#""duration": 0"#,
            "frames/0/duration",
        );
        rejected(
            r#""duration": 50"#,
            r#""duration": 1.5"#,
            "frames/1/duration",
        );
        rejected(
            r#""frame": { "x": 8, "y": 0 }"#,
            r#""frame": 3"#,
            "frames/1/frame",
        );
        rejected(
            r#""x": 8, "y": 0"#,
            r#""x": -1, "y": 0"#,
            "frames/1/frame/x",
        );
        rejected(
            r#""x": 8, "y": 0"#,
            r#""x": 8, "y": -1"#,
            "frames/1/frame/y",
        );
        rejected(r#""x": 8, "y": 0"#, r#""x": 8"#, "frames/1/frame/y");
    }

    #[test]
    fn the_hash_export_is_rejected_and_an_empty_frame_list_accepted() {
        let hash = r#"{ "frames": { "a 0.png": { "frame": { "x": 0, "y": 0 }, "duration": 1 } },
            "meta": { "image": "a.png", "size": { "w": 8, "h": 8 } } }"#;
        let (info, d) = read(hash);
        assert!(matches!(info, UiTextureInfo::RejectedAseprite(_)));
        assert_eq!(texts(&d)[0].0, "frames");
        let empty = r#"{ "frames": [], "meta": { "image": "a.png", "size": { "w": 8, "h": 4 } } }"#;
        let (info, d) = read(empty);
        assert!(d.is_empty());
        assert_eq!(info.base_size(), [8.0, 4.0]);
        // duration 1 and position 0 are the lowest accepted
        let edge = r#"{ "frames": [ { "frame": { "x": 0, "y": 0 }, "duration": 1 } ],
            "meta": { "image": "", "size": { "w": 1, "h": 1 } } }"#;
        assert!(matches!(read(edge).0, UiTextureInfo::Aseprite(_)));
    }

    #[test]
    fn a_null_meta_is_a_nine_slice_file() {
        let (info, d) = read(r#"{ "meta": null, "nineslice_size": 2 }"#);
        assert!(matches!(info, UiTextureInfo::Nineslice(_)));
        assert!(d.is_empty());
    }
}
