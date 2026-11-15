//! Reading the family's JSON files: the root as the game's reader takes it, then the structs of
//! the model read themselves (`#[derive(Facet, Schema)]`, every failure told), with the family's
//! own findings where the game would skip, default or convert in silence.
//!
//! The model holds a member only in the JSON type the game's reader expects. A member of
//! another type is reported (`InvalidType`) and kept as written in the struct's `extra`
//! (`schema::or_kept`).
//!
//! A meta file is read as the 1.26.40+ reader reads it: as a Minecraft document (`null` is a
//! member of the wrong type) parsed by one strict slice, in which an unknown member is an
//! error. Each such finding is an error, and a meta file with one is rejected: the client
//! skips the piece.
//! A store-info file is read by the client's own JSON code: there `null` is absent and every
//! finding is an information (the game says nothing).

use super::codes;
use super::names::Named;
use super::{
    ArmSize, BodySize, FaceFlag, GeometrySource, HexColor, PieceMetadata, PieceSide,
    PieceStoreInfo, PieceType, TextureSource, TintColor, TintColorOrString, Written,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, last_wins, strip_bom,
};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

/// The payload key of a meta file written as a Minecraft document.
pub(super) const ENVELOPE: &str = "minecraft:persona_piece_meta";

/// Why a file has no root: the message is what the game's reader says.
#[derive(Debug, thiserror::Error)]
pub(super) enum RootError {
    /// The nesting passes the reader's limit.
    #[error("nesting too deep at byte {offset}")]
    TooDeep {
        /// The byte offset of the first bracket past the limit.
        offset: usize,
    },
    /// The bytes are not UTF-8.
    #[error(transparent)]
    NotUtf8(#[from] std::str::Utf8Error),
    /// The text is not JSON; the parser's message.
    #[error("{0}")]
    Json(String),
}

/// A JSON file's root the way the game's reader takes it: a BOM skipped, comments accepted,
/// the last of repeated members kept, content after the root value reported and ignored. The
/// error is the reader's message ([`RootError`]).
pub(super) fn read_root(bytes: &[u8], findings: &mut Vec<Diagnostic>) -> Result<Value, RootError> {
    // Stripped, not `Prepared`: blanking would shift every span by three, so that switch is
    // a behaviour-bearing change for later.
    let stripped = strip_bom(bytes);
    let bom = bytes.len() - stripped.len();
    check_depth(stripped, MAX_JSON_DEPTH).map_err(|deep| RootError::TooDeep {
        offset: deep.offset,
    })?;
    let text = std::str::from_utf8(stripped)?;
    // Of a member written twice the last stands, as in jsoncpp's map.
    let mut root = bedrock_addon_core::json::from_str_jsonc(&last_wins(text))
        .map_err(|error| RootError::Json(error.kind.to_string()))?;
    if let Some(mut found) =
        bedrock_addon_core::json::trailing_content_finding(JsonSlice::new(text))
    {
        // The span counts from the start of the file, the byte-order mark included.
        if let Some(span) = &mut found.location.span {
            span.offset = span
                .offset
                .saturating_add(u32::try_from(bom).unwrap_or(u32::MAX));
        }
        findings.push(found);
    }
    // A number past what a double holds (`1e999`) has no JSON spelling, so a file holding
    // one could not be written back: it is held as the largest finite double of its sign,
    // with an information of the crate's own (the game's reader holds the infinity in
    // silence).
    for clamped in bedrock_addon_core::json::clamp_non_finite(&mut root, None) {
        findings.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(clamped.path),
        );
    }
    Ok(root)
}

/// How a meta file is read: the 1.26.40+ strict slice (module documentation), in which a
/// `null` is a member of the wrong type, a scalar must be of its own JSON type and every
/// finding of the reader is an error.
const META_READING: Reading = Reading::DOCUMENT.strict();

/// How a store-info file is read: by the client's own JSON code, in which `null` is absent
/// and a member of another type is taken without a line (an information).
const STORE_INFO_READING: Reading = Reading::NULL_ABSENT
    .strict()
    .with_on_member_failure(MemberFailure::DefaultAndTell);

/// Tells `code` at `at`, graded `severity`.
fn tell(read: &mut Read<'_>, at: JsonPath, code: Code, severity: Severity) {
    read.push(Diagnostic::error(code).severity(severity).at(at));
}

bedrock_addon_core::impl_schema_leaf!(
    Written<PieceType>,
    Written<BodySize>,
    Written<ArmSize>,
    Written<PieceSide>,
    HexColor,
    FaceFlag,
    TintColorOrString,
);
bedrock_addon_core::impl_no_molang!(
    Written<PieceType>,
    Written<BodySize>,
    Written<ArmSize>,
    Written<PieceSide>,
    HexColor,
    FaceFlag,
    TintColorOrString,
);

/// A colour string, told when it is not `#` and six or eight hex digits. The empty text is the
/// colour a `TintColor` holds when one is left out (no colour), and is not told.
impl FromJson for HexColor {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let Some(text) = node.value().as_string() else {
            read.wrong_type(node);
            return None;
        };
        let color = Self(text.as_str().to_owned());
        if !color.as_str().is_empty() && !color.is_well_formed() {
            // The game reads what is there and logs nothing: an information of the
            // crate's own.
            let code = Code::from(&core_codes::INVALID_VALUE);
            tell(read, node.path().clone(), code, Severity::Info);
        }
        Some(color)
    }
}

/// The four slots (an object), or one colour for all (a string).
impl FromJson for TintColorOrString {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_object() {
            TintColor::from_json(node, read).map(Self::Channels)
        } else {
            HexColor::from_json(node, read).map(Self::String)
        }
    }
}

/// A string, as 1.26.40 types it (the string vanilla's files write); a boolean is a member of
/// the wrong type.
impl FromJson for FaceFlag {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let Some(text) = node.value().as_string() else {
            read.wrong_type(node);
            return None;
        };
        Some(Self::Text(text.as_str().to_owned()))
    }
}

/// A list of entries (`schema::read_with` of `texture_sources`, `geometry_sources` and
/// `animation_sources`): an entry that is no object leaves the whole list unread with that one
/// finding, and the findings of the entries read before it are dropped. The entries after it
/// are not read.
pub(super) fn entries<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<T>> {
    let elements = read.array(node)?;
    let mut attempt = read.fork();
    let mut out = Vec::new();
    let mut failed = false;
    for element in elements.iter() {
        match T::from_json(&element, &mut attempt) {
            Some(entry) => out.push(entry),
            // An object that fails has told why.
            None if element.value().is_object() => failed = true,
            None => {
                read.wrong_type(&element);
                return None;
            }
        }
    }
    read.commit(attempt);
    (!failed).then_some(out)
}

/// A string, or `null`, which the client's own JSON code takes for absent: not read, in silence
/// (`schema::read_with` of the store-info members, which keep the `null` as written).
pub(super) fn text_or_absent(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    if node.value().is_null() {
        return None;
    }
    String::from_json(node, read)
}

/// An entry the texture builder skips, in silence: an information of the crate's own.
fn no_texture(node: &Node<'_>, read: &mut Read<'_>) {
    let code = Code::from(&core_codes::MISSING_REQUIRED_FIELD);
    tell(read, node.path().clone(), code, Severity::Info);
}

/// `texture` is required; an empty one is a texture the builder skips, reading nothing else of
/// the entry (`schema::after_read` of [`TextureSource`]).
pub(super) fn texture_rules(source: &mut TextureSource, node: &Node<'_>, read: &mut Read<'_>) {
    if source.texture.as_deref() == Some("") {
        no_texture(node, read);
    }
}

/// The same skip, for an entry that gives the texture builder something: every vanilla geometry
/// source names no texture and loses nothing by being skipped (`schema::after_read` of
/// [`GeometrySource`]).
pub(super) fn geometry_rules(source: &mut GeometrySource, node: &Node<'_>, read: &mut Read<'_>) {
    let unread = written_in(&source.extra, "texture");
    let texture = source.texture.as_deref();
    let lost = source.tint_map.is_some() || source.animated.is_some() || source.frames.is_some();
    if !unread && (texture == Some("") || (texture.is_none() && lost)) {
        no_texture(node, read);
    }
}

/// Which piece types take which `side`, and which members only an emote plays
/// (`schema::after_read` of [`PieceMetadata`]). A type that is no arm or leg gets the game's
/// text, an arm or leg with a side it does not take is skipped in silence.
pub(super) fn meta_rules(meta: &mut PieceMetadata, node: &Node<'_>, read: &mut Read<'_>) {
    let piece_type = meta.piece_type();
    let sources = node.path().clone().child("geometry_sources");
    for (index, source) in meta.geometry_sources.iter().flatten().enumerate() {
        let Some(side) = &source.side else {
            continue;
        };
        if piece_type.accepts_side(side.as_str()) {
            continue;
        }
        let code = if piece_type.is_arm_or_leg() {
            Code::from(&core_codes::INVALID_VALUE)
        } else {
            codes::SIDE_NOT_ARM_OR_LEG.with([piece_type.name(), side.as_str()])
        };
        // The game's text is not a content-log line (it goes to the error display only),
        // and the other case has no text at all. Both are informations of the crate's own.
        tell(
            read,
            sources.clone().index(index).child("side"),
            code,
            Severity::Info,
        );
    }
    // Only an emote's animations are ever played.
    if meta.animation_sources.is_some() && piece_type != PieceType::Emote {
        let at = node.path().clone().child("animation_sources");
        tell(
            read,
            at,
            Code::from(&core_codes::IGNORED_MEMBER),
            Severity::Info,
        );
    }
}

/// The object blanks what it does not set. The game reads the member as "" without a line (it
/// has no text for `piece_store_name` or `piece_thumbnail`): an information of the crate's own
/// (`schema::after_read` of [`PieceStoreInfo`]).
pub(super) fn store_rules(store: &mut PieceStoreInfo, node: &Node<'_>, read: &mut Read<'_>) {
    for (key, set) in [
        ("piece_store_name", store.piece_store_name.is_some()),
        ("piece_thumbnail", store.piece_thumbnail.is_some()),
    ] {
        if !set && !written_in(&store.extra, key) {
            let code = Code::from(&core_codes::MISSING_REQUIRED_FIELD);
            tell(read, node.path().clone().child(key), code, Severity::Info);
        }
    }
}

/// `Json::Value::asInt(default)`: `null` (and a missing member) is the default; an integer
/// keeps its low 32 bits; a real is cut toward zero and held to the bounds of 32 bits; a
/// boolean is 0 or 1; a string, an array or an object is 0.
// The texture family reads the same call the same way; it is mirrored here because a
// content module cannot call another and the function is not in core.
pub(super) fn as_int(value: Option<&Value>, default: i32) -> i32 {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        return i32::from(flag);
    }
    let Some(number) = value.as_number() else {
        return 0;
    };
    if number.is_float() {
        return saturating(number.to_f64_lossy());
    }
    match (number.to_i64(), number.to_u64()) {
        (Some(wide), _) => low_word(wide.to_le_bytes()),
        (None, Some(wide)) => low_word(wide.to_le_bytes()),
        // Past 64 bits the game's reader holds a real.
        (None, None) => saturating(number.to_f64_lossy()),
    }
}

/// The low 32 bits of a 64-bit integer, as `asInt` loads them.
fn low_word(bytes: [u8; 8]) -> i32 {
    let [a, b, c, d, ..] = bytes;
    i32::from_le_bytes([a, b, c, d])
}

/// Toward zero, held to the bounds of an `i32`; NaN is 0.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a float-to-int `as` saturates, which is the conversion the game's instruction does"
)]
fn saturating(real: f64) -> i32 {
    real as i32
}

/// Whether `extra` holds the member `key` with a value other than `null`: a known member of
/// a JSON type the reader did not take.
pub(super) fn written_in(extra: &UnknownMembers, key: &str) -> bool {
    extra.get(key).is_some_and(|value| !value.is_null())
}

/// The one version the meta reader of 1.26.40+ has a slice for: the loader's end version is
/// 1.26.40 too, so its range is `[1.26.40, 1.26.40]`.
pub(super) const META_VERSION: FormatVersion = known::V1_26_40;

/// One meta file from its root object, as the 1.26.40+ loader routes it:
///
/// - no `format_version` (every vanilla file): the whole root is the payload, read at the
///   caller's version 1.26.40;
/// - a `format_version` below 1.26.40: also the whole root, with no envelope check, and no
///   slice for the version, so the file is refused;
/// - from 1.26.40: the two-member envelope, read from its payload at exactly 1.26.40
///   and refused at any other version;
/// - a value that is no version is malformed; `"beta"` has no slice and is refused.
pub(super) fn read_meta(root: &Value, findings: &mut Vec<Diagnostic>) -> PieceMetadata {
    let mut told = Diagnostics::default();
    let mut read = Read::new(META_READING, &mut told);
    let meta = meta_file(root, &mut read);
    // The reader keeps no member in the document's table: it holds findings only.
    findings.extend(told);
    meta
}

/// [`read_meta`] with its reader.
fn meta_file(root: &Value, read: &mut Read<'_>) -> PieceMetadata {
    let Some(members) = root.as_object() else {
        return PieceMetadata::default();
    };
    let Some(version) = members.get("format_version") else {
        return read_document(&Node::new(root, JsonPath::new()), read).unwrap_or_default();
    };
    let at_version = JsonPath::new().child("format_version");
    let refuse = |read: &mut Read<'_>, code: Code, at: JsonPath| {
        read.push(Diagnostic::error(code).at(at));
        PieceMetadata::default()
    };
    let Some(input) = FormatVersionInput::from_json(version) else {
        let code = Code::from(&core_codes::MALFORMED_FORMAT_VERSION);
        return refuse(read, code, at_version);
    };
    let written = version.as_string().map_or_else(
        || input.version.map(|v| v.to_string()).unwrap_or_default(),
        |text| text.as_str().to_owned(),
    );
    let unsupported = |written: &str| core_codes::NO_PARSER_FOR_VERSION.with([written]);
    let below = input
        .version
        .is_some_and(|v| v < META_VERSION || (input.prerelease && v == META_VERSION));
    if input.beta_literal || below {
        // Below the loader's minimum the envelope is not counted, and no slice takes it.
        return refuse(read, unsupported(&written), at_version);
    }
    if members.len() != 2 {
        read.push(
            Diagnostic::of(&core_codes::DOCUMENT_MEMBER_COUNT)
                .at(JsonPath::new())
                .detail(DiagnosticDetail::Syntax(format!(
                    "{} root members, not \"format_version\" and a payload member",
                    members.len()
                ))),
        );
        return PieceMetadata::default();
    }
    let Some(payload) = members.get(ENVELOPE) else {
        return refuse(
            read,
            Code::from(&core_codes::MISSING_PAYLOAD),
            JsonPath::new(),
        );
    };
    if input.version != Some(META_VERSION) || input.prerelease {
        return refuse(read, unsupported(&written), at_version);
    }
    let node = Node::new(payload, JsonPath::new().child(ENVELOPE));
    let mut meta: PieceMetadata = read_document(&node, read).unwrap_or_default();
    meta.envelope = Some(version.clone());
    meta
}

/// One store-info file from its root object.
pub(super) fn read_store_info(root: &Value, findings: &mut Vec<Diagnostic>) -> PieceStoreInfo {
    if !root.is_object() {
        return PieceStoreInfo::default();
    }
    let mut told = Diagnostics::default();
    let mut read = Read::new(STORE_INFO_READING, &mut told).unknown_severity(Severity::Info);
    let store = read_document(&Node::new(root, JsonPath::new()), &mut read);
    // The reader keeps no member in the document's table: it holds findings only.
    findings.extend(told);
    store.unwrap_or_default()
}
