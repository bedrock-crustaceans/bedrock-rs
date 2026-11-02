//! One `blocks.json` entry read field by field, as the client applies it: a field the client
//! refuses is `None` with a finding, and the rest of the entry stays.

use facet_value::VObject;

use super::texts::{
    BAD_BLOCKSHAPE, BAD_CARRIED_TEXTURES, BAD_TEXTURES, CUSTOM_BLOCKSHAPE, formatted,
};
use super::{ClientBlock, FaceBools3, FaceBools6, FaceTextures, Isotropic, block_shape_value};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::json::Value;

/// The six members the client reads; every other member of an entry is ignored.
const READ: [&str; 6] = [
    "textures",
    "carried_textures",
    "isotropic",
    "blockshape",
    "ambient_occlusion_exponent",
    "sound",
];

/// An entry as read: the block's fields, and the shape it asks for (applied or refused where
/// the block's current shape is known, in the stack).
#[derive(Clone, Debug)]
pub(super) struct ReadEntry {
    /// The fields; `blockshape` is unset until the stack applies [`requested_shape`].
    ///
    /// [`requested_shape`]: Self::requested_shape
    pub(super) block: ClientBlock,
    /// The `blockshape` name the entry asks for, with its number.
    pub(super) requested_shape: Option<(String, i16)>,
}

/// Reads the entry `object`, written under `key`, for the block `block_name` (its full name,
/// which vanilla's texts print). `vanilla` says whether the block can be a vanilla one: only
/// those take a `blockshape`.
pub(super) fn read(
    object: &VObject,
    key: &str,
    block_name: &str,
    vanilla: bool,
    d: &mut Diagnostics,
) -> ReadEntry {
    let at = |member: &str| {
        let mut path = JsonPath::new();
        path.push(key);
        path.push(member);
        path
    };
    let mut block = ClientBlock::default();
    let mut requested_shape = None;
    if let Some(value) = object.get("textures") {
        block.textures = textures(value, BAD_TEXTURES, block_name, &at("textures"), d);
    }
    if let Some(value) = object.get("carried_textures") {
        block.carried_textures = textures(
            value,
            BAD_CARRIED_TEXTURES,
            block_name,
            &at("carried_textures"),
            d,
        );
    }
    if let Some(value) = object.get("isotropic") {
        block.isotropic = isotropic(value, &at("isotropic"), d);
    }
    if let Some(value) = object.get("blockshape") {
        if vanilla {
            requested_shape = blockshape(value, block_name, &at("blockshape"), d);
        } else {
            // The member's presence is the test, so `null` counts. The game logs this as an
            // error and loads the entry without a shape, so it is a kept error: the finding is
            // an error although the file still loads.
            d.push_kept_error(kept(
                DiagnosticCode::IgnoredMember {
                    text: Some(formatted(CUSTOM_BLOCKSHAPE, &[block_name])),
                },
                at("blockshape"),
            ));
        }
    }
    if let Some(value) = object.get("ambient_occlusion_exponent") {
        block.ambient_occlusion_exponent =
            ambient_occlusion(value, &at("ambient_occlusion_exponent"), d);
    }
    if let Some(value) = object.get("sound") {
        block.sound = sound(value, &at("sound"), d);
    }
    for (name, value) in object {
        if !READ.contains(&name.as_str()) {
            block.extra.insert(name.as_str().to_owned(), value.clone());
        }
    }
    ReadEntry {
        block,
        requested_shape,
    }
}

/// Reports every member of the entry `object` under `key` that the client never reads, as an
/// information. The client reads `textures`, `carried_textures`, `isotropic`, `blockshape`,
/// `ambient_occlusion_exponent` and `sound` and has no unknown-member check, so `sounds`
/// (vanilla's own `coral_block` has one) and `brightness_gamma` are ignored without a word; the
/// finding is a lint of this crate's own, hence an information. Raised once per entry, whether
/// or not the entry applies to a block.
pub(super) fn unread_members(object: &VObject, key: &str, d: &mut Diagnostics) {
    for (name, _) in object {
        let name = name.as_str();
        if READ.contains(&name) {
            continue;
        }
        let mut at = JsonPath::new();
        at.push(key);
        at.push(name);
        d.push(finding(
            DiagnosticCode::IgnoredMember { text: None },
            Severity::Info,
            at,
        ));
    }
}

/// `textures` / `carried_textures`: a string, exactly `{up, down, side}` or exactly the six
/// faces, every face a string. `null` changes nothing; an object is told apart by its member
/// count (6 or 3) and each face must be a string.
fn textures(
    value: &Value,
    template: &'static str,
    block_name: &str,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<FaceTextures> {
    if value.is_null() {
        return None;
    }
    if let Some(all) = value.as_string() {
        return Some(FaceTextures::All(all.as_str().to_owned()));
    }
    let face = |object: &VObject, name: &str| {
        object
            .get(name)
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned())
    };
    let read = value.as_object().and_then(|object| match object.len() {
        3 => Some(FaceTextures::UpDownSide {
            up: face(object, "up")?,
            down: face(object, "down")?,
            side: face(object, "side")?,
        }),
        6 => Some(FaceTextures::SixFaces {
            up: face(object, "up")?,
            down: face(object, "down")?,
            north: face(object, "north")?,
            south: face(object, "south")?,
            west: face(object, "west")?,
            east: face(object, "east")?,
        }),
        _ => None,
    });
    if read.is_none() {
        d.push(warning(
            DiagnosticCode::InvalidType {
                text: Some(formatted(template, &[block_name])),
            },
            at.clone(),
        ));
    }
    read
}

/// `isotropic`: a boolean, or an object of faces. An object always sets the mask, from the
/// members that are `true`; any member the client does not match is reported as an
/// information. The face set is chosen by whether `side` is `null`, each face member that is a
/// boolean counts as matched and `true` adds its bits to the mask; that a member went unmatched
/// does not stop the mask being applied.
fn isotropic(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<Isotropic> {
    if value.is_null() {
        return None;
    }
    if let Some(all) = value.as_bool() {
        return Some(Isotropic::All(all));
    }
    let Some(object) = value.as_object() else {
        d.push(info(DiagnosticCode::InvalidType { text: None }, at.clone()));
        return None;
    };
    let mut matched = 0;
    let mut face = |name: &str| match object.get(name).and_then(Value::as_bool) {
        Some(on) => {
            matched += 1;
            on
        }
        None => false,
    };
    let three = object.get("side").is_some_and(|side| !side.is_null());
    let read = if three {
        Isotropic::UpDownSide(FaceBools3 {
            up: face("up"),
            down: face("down"),
            side: face("side"),
        })
    } else {
        Isotropic::SixFaces(FaceBools6 {
            up: face("up"),
            down: face("down"),
            north: face("north"),
            south: face("south"),
            west: face("west"),
            east: face("east"),
        })
    };
    if matched != object.len() {
        d.push(info(
            DiagnosticCode::InvalidValue { text: None },
            at.clone(),
        ));
    }
    Some(read)
}

/// `blockshape` of a vanilla block: one of the 90 names. Whether the client then refuses the
/// conversion depends on the block's current shape, which the stack knows. `null` changes
/// nothing; a non-string or an unknown name is the invalid-blockshape warning.
fn blockshape(
    value: &Value,
    block_name: &str,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<(String, i16)> {
    if value.is_null() {
        return None;
    }
    let named = value
        .as_string()
        .map(facet_value::VString::as_str)
        .and_then(|name| Some((name.to_owned(), block_shape_value(name)?)));
    if named.is_none() {
        d.push(bad_blockshape(block_name, at.clone()));
    }
    named
}

/// The invalid-blockshape warning for `block_name` at `at`.
pub(super) fn bad_blockshape(block_name: &str, at: JsonPath) -> Diagnostic {
    warning(
        DiagnosticCode::InvalidValue {
            text: Some(formatted(BAD_BLOCKSHAPE, &[block_name])),
        },
        at,
    )
}

/// `ambient_occlusion_exponent`: a number; a boolean is read as 1.0 or 0.0 and anything else is
/// ignored, each with an information. The client applies the member when it is not `null` and is
/// numeric (an integer, a real or a boolean), converting it as a float.
fn ambient_occlusion(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<f32> {
    if value.is_null() {
        return None;
    }
    if let Some(flag) = value.as_bool() {
        d.push(info(DiagnosticCode::InvalidType { text: None }, at.clone()));
        return Some(if flag { 1.0 } else { 0.0 });
    }
    let Some(number) = value.as_number() else {
        d.push(info(DiagnosticCode::InvalidType { text: None }, at.clone()));
        return None;
    };
    let wide = number.to_f64_lossy();
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    let narrow = wide as f32;
    if narrow.is_finite() {
        return Some(narrow);
    }
    // Beyond `f32`: the largest finite float of that sign, never an infinity the writer would
    // refuse. A number beyond `f32` is clamped rather than refused so that one oversized value
    // does not cost the file its other values, and the written text reads back to the same value.
    d.push(info(
        DiagnosticCode::InvalidValue { text: None },
        at.clone(),
    ));
    Some(if wide < 0.0 { f32::MIN } else { f32::MAX })
}

/// `sound`: a string; any other value is what jsoncpp's string conversion makes of it, with an
/// information. The member is applied when it is not `null`; that conversion gives `"true"` or
/// `"false"` for a boolean and `""` for a number, array or object.
fn sound(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Option<String> {
    if value.is_null() {
        return None;
    }
    if let Some(name) = value.as_string() {
        return Some(name.as_str().to_owned());
    }
    d.push(info(DiagnosticCode::InvalidType { text: None }, at.clone()));
    Some(match value.as_bool() {
        Some(true) => "true".to_owned(),
        Some(false) => "false".to_owned(),
        None => String::new(),
    })
}

/// A warning at `at`: a line the game itself logs at warning level. Every caller is one of the
/// `format_version` texts (missing, invalid, above the ceiling) or the invalid `textures`,
/// `carried_textures` and `blockshape` texts, including a refused shape conversion; the entry
/// and the file load on.
pub(super) fn warning(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    finding(code, Severity::Warning, at)
}

/// A lint of this crate's own: an information. The game says nothing about these.
pub(super) fn info(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    finding(code, Severity::Info, at)
}

/// A content-log error of the game's after which the file loads, for `push_kept_error`: the
/// finding is an error although the entry or member it names is the only thing dropped.
pub(super) fn kept(code: DiagnosticCode, at: JsonPath) -> Diagnostic {
    finding(code, Severity::Error, at)
}

/// A finding of the validate phase at `at`.
pub(super) fn finding(code: DiagnosticCode, severity: Severity, at: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(None, None, at, None),
    )
}
