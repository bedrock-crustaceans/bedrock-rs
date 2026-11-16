//! One `blocks.json` entry read as the client applies it: [`ClientBlock`]'s derived read, member
//! by member, with the leaves below. A member the client refuses is `None` with a finding, and
//! the rest of the entry stays; every finding is told in [`Phase::Validate`], and the codes of
//! vanilla's texts are fed the block's name (the read's subject).

use super::codes;
use super::{ClientBlock, FaceBools3, FaceBools6, FaceTextures, Isotropic, block_shape_value};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

/// The six members the client reads; every other member of an entry is ignored.
const READ: [&str; 6] = [
    "textures",
    "carried_textures",
    "isotropic",
    "blockshape",
    "ambient_occlusion_exponent",
    "sound",
];

/// How the client reads an entry: a member it refuses changes nothing and the rest is applied
/// (a failed member is reset, told at `Info` where its leaf does not tell its own), and a `null`
/// member is absent without a word.
const READING: Reading = Reading::OWN
    .with_on_member_failure(MemberFailure::DefaultAndTell)
    .with_null(NullHandling::AbsentSilently);

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
/// which vanilla's texts print; a vanilla block's is in the `minecraft` namespace, and only
/// those take a `blockshape`).
pub(super) fn read(object: &Value, key: &str, block_name: &str, d: &mut Diagnostics) -> ReadEntry {
    let node = Node::new(object, JsonPath::new().child(key));
    let mut read = Read::new(READING, d)
        .profile(Profile::DECLARED_ORDER)
        .phase(Phase::Validate)
        .subject(block_name);
    // Every member is optional and a failed one is reset: the entry always reads.
    let mut block = read_document::<ClientBlock>(&node, &mut read).unwrap_or_default();
    let requested_shape = block
        .blockshape
        .take()
        .and_then(|name| block_shape_value(&name).map(|value| (name, value)));
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
/// or not the entry applies to a block, so not by the entry's read (which keeps them silently in
/// [`ClientBlock::extra`]).
pub(super) fn unread_members(object: &facet_value::VObject, key: &str, d: &mut Diagnostics) {
    for (name, _) in object {
        let name = name.as_str();
        if READ.contains(&name) {
            continue;
        }
        let at = JsonPath::new().child(key).child(name);
        d.push(
            Diagnostic::of(&core_codes::IGNORED_MEMBER)
                .at(at)
                .phase(Phase::Validate),
        );
    }
}

/// `finding` at `node`, spanning it.
fn at(finding: Diagnostic, node: &Node<'_>) -> Diagnostic {
    let mut finding = finding.at(node.path().clone());
    finding.location.span = node.span();
    finding
}

/// Tells `code` at `node` as an information, the value read all the same.
fn note(read: &mut Read<'_>, node: &Node<'_>, code: Code) {
    read.push(at(Diagnostic::of(code).severity(Severity::Info), node));
}

/// Tells the member's own refusal (`schema::invalid`, vanilla's warning fed the block's name) at
/// its code's severity, as the client logs it.
fn refuse(read: &mut Read<'_>, node: &Node<'_>) {
    match read.invalid_code(node) {
        Some(code) => read.push(at(Diagnostic::of(code), node)),
        None => read.wrong_type(node),
    }
}

/// `textures` / `carried_textures`: a string, exactly `{up, down, side}` or exactly the six
/// faces, every face a string; an object is told apart by its member count (6 or 3). Anything
/// else is the member's code (vanilla's warning).
impl FromJson for FaceTextures {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(all) = value.as_string() {
            return Some(Self::All(all.as_str().to_owned()));
        }
        let face = |object: &facet_value::VObject, name: &str| {
            object
                .get(name)
                .and_then(Value::as_string)
                .map(|s| s.as_str().to_owned())
        };
        let faces = value.as_object().and_then(|object| match object.len() {
            3 => Some(Self::UpDownSide {
                up: face(object, "up")?,
                down: face(object, "down")?,
                side: face(object, "side")?,
            }),
            6 => Some(Self::SixFaces {
                up: face(object, "up")?,
                down: face(object, "down")?,
                north: face(object, "north")?,
                south: face(object, "south")?,
                west: face(object, "west")?,
                east: face(object, "east")?,
            }),
            _ => None,
        });
        if faces.is_none() {
            refuse(read, node);
        }
        faces
    }
}

/// `isotropic`: a boolean, or an object of faces. An object always sets the mask, from the
/// members that are `true`; any member the client does not match is reported as an
/// information. The face set is chosen by whether `side` is `null`, each face member that is a
/// boolean counts as matched and `true` adds its bits to the mask; that a member went unmatched
/// does not stop the mask being applied. Another value is a wrong type and changes nothing.
impl FromJson for Isotropic {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let value = node.value();
        if let Some(all) = value.as_bool() {
            return Some(Self::All(all));
        }
        let Some(object) = value.as_object() else {
            read.wrong_type(node);
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
        let faces = if three {
            Self::UpDownSide(FaceBools3 {
                up: face("up"),
                down: face("down"),
                side: face("side"),
            })
        } else {
            Self::SixFaces(FaceBools6 {
                up: face("up"),
                down: face("down"),
                north: face("north"),
                south: face("south"),
                west: face("west"),
                east: face("east"),
            })
        };
        if matched != object.len() {
            note(read, node, (&core_codes::INVALID_VALUE).into());
        }
        Some(faces)
    }
}

bedrock_addon_core::impl_schema_leaf!(FaceTextures, Isotropic);
bedrock_addon_core::impl_no_molang!(FaceTextures, Isotropic);

/// `blockshape` (`schema::reads_null`: the member's presence is the test): on a vanilla block
/// one of the 90 names, whose conversion the stack then applies or refuses by the block's
/// current shape; `null` changes nothing, and a non-string or an unknown name is the member's
/// code (vanilla's invalid-blockshape warning). On a custom block any value, `null` too, is the
/// custom-blockshape error, which the game logs and loads the entry without a shape: a kept
/// error.
pub(super) fn blockshape(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let block_name = read.subject_text().unwrap_or_default();
    if !block_name.starts_with("minecraft:") {
        let custom = Diagnostic::of(codes::CUSTOM_BLOCKSHAPE.with([block_name]));
        read.push(at(custom, node));
        return None;
    }
    let value = node.value();
    if value.is_null() {
        return None;
    }
    let named = value
        .as_string()
        .map(|name| name.as_str().to_owned())
        .filter(|name| block_shape_value(name).is_some());
    if named.is_none() {
        refuse(read, node);
    }
    named
}

/// The invalid-blockshape warning for `block_name` at `at`, where the stack refuses the
/// conversion.
pub(super) fn bad_blockshape(block_name: &str, at: JsonPath) -> Diagnostic {
    Diagnostic::of(codes::BAD_BLOCKSHAPE.with([block_name]))
        .at(at)
        .phase(Phase::Validate)
}

/// `ambient_occlusion_exponent`: a number; a boolean is read as 1.0 or 0.0 and anything else is
/// ignored, each with an information. The client applies the member when it is not `null` and is
/// numeric (an integer, a real or a boolean), converting it as a float.
pub(super) fn exponent(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let value = node.value();
    if let Some(flag) = value.as_bool() {
        note(read, node, (&core_codes::INVALID_TYPE).into());
        return Some(if flag { 1.0 } else { 0.0 });
    }
    let Some(number) = value.as_number() else {
        read.wrong_type(node);
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
    note(read, node, (&core_codes::INVALID_VALUE).into());
    Some(if wide < 0.0 { f32::MIN } else { f32::MAX })
}

/// `sound`: a string; any other value is what jsoncpp's string conversion makes of it, with an
/// information. The member is applied when it is not `null`; that conversion gives `"true"` or
/// `"false"` for a boolean and `""` for a number, array or object.
#[allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` fn: the read's signature; this one reads every value"
)]
pub(super) fn sound(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let value = node.value();
    if let Some(name) = value.as_string() {
        return Some(name.as_str().to_owned());
    }
    note(read, node, (&core_codes::INVALID_TYPE).into());
    Some(match value.as_bool() {
        Some(true) => "true".to_owned(),
        Some(false) => "false".to_owned(),
        None => String::new(),
    })
}
