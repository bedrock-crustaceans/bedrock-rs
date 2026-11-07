//! Reading the family's JSON files: the root as the game's reader takes it, then each member
//! into the model, with a finding wherever the game would skip, default or convert in
//! silence.
//!
//! The model holds a member only in the JSON type the game's reader expects. A member of
//! another type is reported (`InvalidType`) and kept as written in the struct's `extra`.
//!
//! A meta file is read as the 1.26.40+ reader reads it: as a Minecraft document (`null` is a
//! member of the wrong type) parsed by one strict slice, in which an unknown member is an
//! error. Each such finding is an error, and a meta file with one is rejected: the client
//! skips the piece.
//! A store-info file is read by the client's own JSON code: there `null` is absent and every
//! finding is an information (the game says nothing).

use super::codes;
use super::names::{Named, Written};
use super::{
    AnimationSource, FaceFlag, GeometrySource, HexColor, PieceMetadata, PieceStoreInfo, PieceType,
    TextureSource, TintColor, TintColorOrString, Zones,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Severity,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Disposition, Node, Read};
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, last_wins, strip_bom,
};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

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
    let mut root = bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(text))
        .map_err(|error| RootError::Json(error.to_string()))?;
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

/// Why a member is not in the model.
enum Unread {
    /// Its JSON type is not the one the game's reader expects.
    Wrong,
    /// The same, and the reader has named the place (an element) itself.
    Reported,
}

/// How a meta file is read: the 1.26.40+ strict slice (module documentation), in which a
/// `null` is a member of the wrong type and every finding of the reader is an error.
const META_READING: Reading = Reading::DOCUMENT;

/// How a store-info file is read: by the client's own JSON code, in which `null` is absent
/// and a member of another type is taken without a line (an information).
const STORE_INFO_READING: Reading =
    Reading::NULL_ABSENT.with_on_member_failure(MemberFailure::DefaultAndTell);

/// The reader of one file: the findings so far, how the file is read and the place being
/// read.
struct Reader {
    findings: Diagnostics,
    reading: Reading,
    /// The severity of an unknown member, where it is not the reading's.
    unknown_severity: Option<Severity>,
    at: JsonPath,
}

impl Reader {
    fn new(reading: Reading, unknown_severity: Option<Severity>) -> Self {
        Self {
            findings: Diagnostics::default(),
            reading,
            unknown_severity,
            at: JsonPath::new(),
        }
    }

    /// The kit's read, telling into the findings so far.
    fn read(&mut self) -> Read<'_> {
        let read = Read::new(self.reading, &mut self.findings);
        match self.unknown_severity {
            Some(severity) => read.unknown_severity(severity),
            None => read,
        }
    }

    /// `value` at the place being read.
    fn node<'v>(&self, value: &'v Value) -> Node<'v> {
        Node::new(value, self.at.clone())
    }

    /// A finding at the place being read.
    fn report(&mut self, code: DiagnosticCode, severity: Severity) {
        let finding = Diagnostic::error(code)
            .severity(severity)
            .at(self.at.clone());
        self.read().push(finding);
    }

    /// A finding at the member `key` of the object being read.
    fn report_at(&mut self, key: &str, code: DiagnosticCode, severity: Severity) {
        self.at.push(key);
        self.report(code, severity);
        self.at.pop();
    }

    /// `value`, at the place being read, is of an unexpected JSON type: an error in a meta
    /// file; in a store-info file the client's own JSON code takes the member without a line.
    fn wrong_type(&mut self, value: &Value) {
        let node = self.node(value);
        self.read().wrong_type(&node);
    }

    /// Reads the member `key` = `value` into `slot`; a value `read` does not take is kept in
    /// `extra` and reported, `null` only by the strict reader (the member is present
    /// and its type refuses `null`).
    fn take<T>(
        &mut self,
        key: &str,
        value: &Value,
        extra: &mut UnknownMembers,
        slot: &mut Option<T>,
        read: impl FnOnce(&mut Self, &Value) -> Result<T, Unread>,
    ) {
        self.at.push(key);
        match read(self, value) {
            Ok(read) => *slot = Some(read),
            Err(unread) => {
                let absent = value.is_null() && self.reading.null == NullHandling::Absent;
                if !absent && matches!(unread, Unread::Wrong) {
                    self.wrong_type(value);
                }
                extra.insert(key.to_owned(), value.clone());
            }
        }
        self.at.pop();
    }

    /// A member the schema does not declare (1.26.40+), kept as written. The strict slice
    /// makes the unknown member an error: the piece is invalid.
    /// In a store-info file the member is ignored (information).
    fn unknown(&mut self, key: &str, value: &Value, extra: &mut UnknownMembers) {
        self.at.push(key);
        let node = self.node(value);
        self.read().unknown_member(&node, Disposition::Keep);
        self.at.pop();
        extra.insert(key.to_owned(), value.clone());
    }

    /// A string.
    fn text(_: &mut Self, value: &Value) -> Result<String, Unread> {
        value
            .as_string()
            .map(|text| text.as_str().to_owned())
            .ok_or(Unread::Wrong)
    }

    /// A boolean.
    fn flag(_: &mut Self, value: &Value) -> Result<bool, Unread> {
        value.as_bool().ok_or(Unread::Wrong)
    }

    /// A whole number of 32 bits.
    fn int(_: &mut Self, value: &Value) -> Result<i32, Unread> {
        value
            .as_number()
            .filter(|number| !number.is_float())
            .and_then(facet_value::VNumber::to_i64)
            .and_then(|wide| i32::try_from(wide).ok())
            .ok_or(Unread::Wrong)
    }

    /// A name of a bound enum. A text outside it is an `invalid enum value` error in both
    /// readers, kept as written.
    fn name<T: Named>(&mut self, value: &Value) -> Result<Written<T>, Unread> {
        let written = Written::<T>::parse(value.as_string().ok_or(Unread::Wrong)?.as_str());
        if matches!(written, Written::Other(_)) {
            self.report(
                bedrock_addon_core::diagnostic::cereal::invalid_enum_value(),
                Severity::Error,
            );
        }
        Ok(written)
    }

    /// A `missing required field` error (in both readers) for each of `keys` that the object
    /// `object` does not have.
    fn required(&mut self, object: &Value, keys: &[&str]) {
        let node = self.node(object);
        let mut read = self.read();
        if let Some(members) = read.object(&node) {
            for key in keys {
                read.required(&members, key);
            }
        }
    }

    /// The elements of an array, each read by `read`; an element it does not take is named
    /// and leaves the whole member unread. Such a list is in the model by none of its
    /// entries, so what was found in the entries before the bad one is dropped: the member
    /// has the one finding of this reader. A number past a double inside the list keeps its
    /// own finding: that one is the root's (`read_root`), about the number the file holds
    /// in place of the author's, and the list is written back with it.
    fn elements<T>(
        &mut self,
        value: &Value,
        mut read: impl FnMut(&mut Self, &Value) -> Option<T>,
    ) -> Result<Vec<T>, Unread> {
        let array = value.as_array().ok_or(Unread::Wrong)?;
        // The entries are read apart, so that a bad one takes their findings with it.
        let before = std::mem::take(&mut self.findings);
        let mut elements = Vec::with_capacity(array.len());
        for (index, element) in array.as_slice().iter().enumerate() {
            self.at.push_index(index);
            let read = read(self, element);
            if read.is_none() {
                self.findings = Diagnostics::default();
                self.wrong_type(element);
            }
            self.at.pop();
            if let Some(read) = read {
                elements.push(read);
            } else {
                let entries = std::mem::replace(&mut self.findings, before);
                self.findings.extend_with_unknown(entries);
                return Err(Unread::Reported);
            }
        }
        let entries = std::mem::replace(&mut self.findings, before);
        self.findings.extend_with_unknown(entries);
        Ok(elements)
    }

    /// One zone or a list of zones.
    fn zones(&mut self, value: &Value) -> Result<Zones, Unread> {
        if let Some(zone) = value.as_string() {
            return Ok(Zones::One(zone.as_str().to_owned()));
        }
        self.elements(value, |_, element| {
            element.as_string().map(|zone| zone.as_str().to_owned())
        })
        .map(Zones::Many)
    }

    /// A colour string, reported when it is not `#` and six or eight hex digits. The
    /// empty text is the colour a `TintColor` holds when one is left out (no colour), and is
    /// not reported.
    fn color(&mut self, value: &Value) -> Result<HexColor, Unread> {
        let color = HexColor(value.as_string().ok_or(Unread::Wrong)?.as_str().to_owned());
        if !color.as_str().is_empty() && !color.is_well_formed() {
            // The game reads what is there and logs nothing: an information of the
            // crate's own.
            self.report(
                DiagnosticCode::from(&core_codes::INVALID_VALUE),
                Severity::Info,
            );
        }
        Ok(color)
    }

    /// The four slots, or one colour for all.
    fn tint(&mut self, value: &Value) -> Result<TintColorOrString, Unread> {
        let Some(members) = value.as_object() else {
            return self.color(value).map(TintColorOrString::String);
        };
        let mut tint = TintColor::default();
        for (key, value) in members {
            let extra = &mut tint.extra;
            match key.as_str() {
                "r_color" => self.take("r_color", value, extra, &mut tint.r_color, Self::color),
                "g_color" => self.take("g_color", value, extra, &mut tint.g_color, Self::color),
                "b_color" => self.take("b_color", value, extra, &mut tint.b_color, Self::color),
                "a_color" => self.take("a_color", value, extra, &mut tint.a_color, Self::color),
                other => self.unknown(other, value, extra),
            }
        }
        Ok(TintColorOrString::Channels(tint))
    }

    /// A string, as 1.26.40 types it (the string vanilla's files write); a boolean is a
    /// member of the wrong type.
    fn face(_: &mut Self, value: &Value) -> Result<FaceFlag, Unread> {
        let text = value.as_string().ok_or(Unread::Wrong)?;
        Ok(FaceFlag::Text(text.as_str().to_owned()))
    }

    /// An entry the texture builder skips.
    fn no_texture(&mut self) {
        // The builder skips the entry in silence: an information of the crate's own.
        self.report(
            DiagnosticCode::from(&core_codes::MISSING_REQUIRED_FIELD),
            Severity::Info,
        );
    }

    /// One entry of `texture_sources`.
    fn texture_source(&mut self, value: &Value) -> Option<TextureSource> {
        let members = value.as_object()?;
        let mut source = TextureSource::default();
        for (key, value) in members {
            let extra = &mut source.extra;
            match key.as_str() {
                "texture" => self.take("texture", value, extra, &mut source.texture, Self::text),
                "tint_map" => self.take("tint_map", value, extra, &mut source.tint_map, Self::text),
                "animated" => self.take("animated", value, extra, &mut source.animated, Self::flag),
                "frames" => self.take("frames", value, extra, &mut source.frames, Self::int),
                "use_face_uv" => {
                    self.take(
                        "use_face_uv",
                        value,
                        extra,
                        &mut source.use_face_uv,
                        Self::flag,
                    );
                }
                other => self.unknown(other, value, extra),
            }
        }
        // `texture` is required; an empty one is a texture the builder skips, reading
        // nothing else of the entry.
        self.required(value, &["texture"]);
        if source.texture.as_deref() == Some("") {
            self.no_texture();
        }
        Some(source)
    }

    /// One entry of `geometry_sources`.
    fn geometry_source(&mut self, value: &Value) -> Option<GeometrySource> {
        let members = value.as_object()?;
        let mut source = GeometrySource::default();
        for (key, value) in members {
            let extra = &mut source.extra;
            match key.as_str() {
                "geometry" => self.take("geometry", value, extra, &mut source.geometry, Self::text),
                "arm_size" => {
                    self.take("arm_size", value, extra, &mut source.arm_size, Self::name);
                }
                "body_size" => {
                    self.take("body_size", value, extra, &mut source.body_size, Self::name);
                }
                "side" => self.take("side", value, extra, &mut source.side, Self::name),
                "zone" => self.take("zone", value, extra, &mut source.zone, Self::zones),
                "is_face" => self.take("is_face", value, extra, &mut source.is_face, Self::face),
                "texture" => self.take("texture", value, extra, &mut source.texture, Self::text),
                "tint_map" => self.take("tint_map", value, extra, &mut source.tint_map, Self::text),
                "animated" => self.take("animated", value, extra, &mut source.animated, Self::flag),
                "frames" => self.take("frames", value, extra, &mut source.frames, Self::int),
                // `use_face_uv` is among the unknown members: it belongs to a texture source only.
                other => self.unknown(other, value, extra),
            }
        }
        self.required(value, &["geometry"]);
        // The same skip, for an entry that gives the texture builder something: every vanilla geometry
        // source names no texture and loses nothing by being skipped.
        let unread = written_in(&source.extra, "texture");
        let texture = source.texture.as_deref();
        let lost =
            source.tint_map.is_some() || source.animated.is_some() || source.frames.is_some();
        if !unread && (texture == Some("") || (texture.is_none() && lost)) {
            self.no_texture();
        }
        Some(source)
    }

    /// One entry of `animation_sources`.
    fn animation_source(&mut self, value: &Value) -> Option<AnimationSource> {
        let members = value.as_object()?;
        let mut source = AnimationSource::default();
        for (key, value) in members {
            let extra = &mut source.extra;
            match key.as_str() {
                "name" => self.take("name", value, extra, &mut source.name, Self::text),
                "animationFile" => {
                    self.take(
                        "animationFile",
                        value,
                        extra,
                        &mut source.animation_file,
                        Self::text,
                    );
                }
                other => self.unknown(other, value, extra),
            }
        }
        self.required(value, &["name", "animationFile"]);
        Some(source)
    }

    /// The members of a meta file (the root's, or the payload's).
    #[allow(clippy::too_many_lines, reason = "one arm per member of the file")]
    fn meta(&mut self, object: &Value, meta: &mut PieceMetadata) {
        let Some(members) = object.as_object() else {
            return;
        };
        for (key, value) in members {
            let extra = &mut meta.extra;
            match key.as_str() {
                "piece_id" => self.take("piece_id", value, extra, &mut meta.piece_id, Self::text),
                "piece_name" => {
                    self.take("piece_name", value, extra, &mut meta.piece_name, Self::text);
                }
                // A name of the 25 the 1.26.40+ enum binds.
                "piece_type" => {
                    self.take("piece_type", value, extra, &mut meta.piece_type, Self::name);
                }
                "piece_sub_type" => {
                    self.take(
                        "piece_sub_type",
                        value,
                        extra,
                        &mut meta.piece_sub_type,
                        Self::name,
                    );
                }
                "body_variant" => {
                    self.take(
                        "body_variant",
                        value,
                        extra,
                        &mut meta.body_variant,
                        Self::name,
                    );
                }
                "clothing_map" => {
                    self.take(
                        "clothing_map",
                        value,
                        extra,
                        &mut meta.clothing_map,
                        Self::text,
                    );
                }
                "face_clothing_map" => self.take(
                    "face_clothing_map",
                    value,
                    extra,
                    &mut meta.face_clothing_map,
                    Self::text,
                ),
                "zone" => self.take("zone", value, extra, &mut meta.zone, Self::zones),
                "texture_sources" => {
                    self.take(
                        "texture_sources",
                        value,
                        extra,
                        &mut meta.texture_sources,
                        |r, v| r.elements(v, Self::texture_source),
                    );
                }
                "geometry_sources" => self.take(
                    "geometry_sources",
                    value,
                    extra,
                    &mut meta.geometry_sources,
                    |r, v| r.elements(v, Self::geometry_source),
                ),
                "animation_sources" => self.take(
                    "animation_sources",
                    value,
                    extra,
                    &mut meta.animation_sources,
                    |r, v| r.elements(v, Self::animation_source),
                ),
                "tint_color" => {
                    self.take("tint_color", value, extra, &mut meta.tint_color, Self::tint);
                }
                "tint_base_color" => {
                    self.take(
                        "tint_base_color",
                        value,
                        extra,
                        &mut meta.tint_base_color,
                        Self::tint,
                    );
                }
                "allow_tint_override" => self.take(
                    "allow_tint_override",
                    value,
                    extra,
                    &mut meta.allow_tint_override,
                    Self::flag,
                ),
                other => self.unknown(other, value, extra),
            }
        }
        self.required(object, &["piece_id", "piece_name", "piece_type"]);
        self.side_rules(meta);
        // Only an emote's animations are ever played.
        if meta.animation_sources.is_some() && meta.piece_type() != PieceType::Emote {
            self.report_at(
                "animation_sources",
                DiagnosticCode::from(&core_codes::IGNORED_MEMBER),
                Severity::Info,
            );
        }
    }

    /// Which piece types take which `side`: a type that is no arm or leg gets the game's text, an arm or leg with a side it does not take is skipped in silence.
    fn side_rules(&mut self, meta: &PieceMetadata) {
        let piece_type = meta.piece_type();
        for (index, source) in meta.geometry_sources.iter().flatten().enumerate() {
            let Some(side) = &source.side else {
                continue;
            };
            if piece_type.accepts_side(side.as_str()) {
                continue;
            }
            // A type that is no arm or leg has the game's text; an arm or leg with a side it
            // does not take is skipped in silence and has none.
            let code = if piece_type.is_arm_or_leg() {
                DiagnosticCode::from(&core_codes::INVALID_VALUE)
            } else {
                codes::SIDE_NOT_ARM_OR_LEG
                    .with([piece_type.name(), side.as_str()])
                    .into()
            };
            self.at.push("geometry_sources");
            self.at.push_index(index);
            // The game's text is not a content-log line (it goes to the error display only),
            // and the other case has no text at all. Both are informations of the crate's own.
            self.report_at("side", code, Severity::Info);
            self.at.pop();
            self.at.pop();
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
    let mut reader = Reader::new(META_READING, None);
    let meta = meta_file(root, &mut reader);
    // The reader keeps no member in the document's table: it holds findings only.
    findings.extend(reader.findings);
    meta
}

/// [`read_meta`] with its reader.
fn meta_file(root: &Value, reader: &mut Reader) -> PieceMetadata {
    let mut meta = PieceMetadata::default();
    let Some(members) = root.as_object() else {
        return meta;
    };
    let Some(version) = members.get("format_version") else {
        reader.meta(root, &mut meta);
        return meta;
    };
    let at_version = JsonPath::new().child("format_version");
    let refuse = |reader: &mut Reader, code: DiagnosticCode, at: JsonPath| {
        reader.read().push(Diagnostic::error(code).at(at));
    };
    let Some(input) = FormatVersionInput::from_json(version) else {
        refuse(
            reader,
            DiagnosticCode::from(&core_codes::MALFORMED_FORMAT_VERSION),
            at_version,
        );
        return meta;
    };
    let written = version.as_string().map_or_else(
        || input.version.map(|v| v.to_string()).unwrap_or_default(),
        |text| text.as_str().to_owned(),
    );
    let unsupported =
        |written: &str| DiagnosticCode::from(core_codes::NO_PARSER_FOR_VERSION.with([written]));
    let below = input
        .version
        .is_some_and(|v| v < META_VERSION || (input.prerelease && v == META_VERSION));
    if input.beta_literal || below {
        // Below the loader's minimum the envelope is not counted, and no slice takes it.
        refuse(reader, unsupported(&written), at_version);
        return meta;
    }
    if members.len() != 2 {
        reader.read().push(
            Diagnostic::of(&core_codes::DOCUMENT_MEMBER_COUNT)
                .at(JsonPath::new())
                .detail(DiagnosticDetail::Syntax(format!(
                    "{} root members, not \"format_version\" and a payload member",
                    members.len()
                ))),
        );
        return meta;
    }
    let Some(payload) = members.get(ENVELOPE) else {
        refuse(
            reader,
            DiagnosticCode::from(&core_codes::MISSING_PAYLOAD),
            JsonPath::new(),
        );
        return meta;
    };
    if input.version != Some(META_VERSION) || input.prerelease {
        refuse(reader, unsupported(&written), at_version);
        return meta;
    }
    meta.envelope = Some(version.clone());
    reader.at.push(ENVELOPE);
    if !payload.is_object() {
        reader.wrong_type(payload);
        return meta;
    }
    reader.meta(payload, &mut meta);
    meta
}

/// One store-info file from its root object.
pub(super) fn read_store_info(root: &Value, findings: &mut Vec<Diagnostic>) -> PieceStoreInfo {
    let mut store = PieceStoreInfo::default();
    let Some(members) = root.as_object() else {
        return store;
    };
    let mut reader = Reader::new(STORE_INFO_READING, Some(Severity::Info));
    for (key, value) in members {
        let extra = &mut store.extra;
        match key.as_str() {
            "piece_store_name" => reader.take(
                "piece_store_name",
                value,
                extra,
                &mut store.piece_store_name,
                Reader::text,
            ),
            "piece_thumbnail" => reader.take(
                "piece_thumbnail",
                value,
                extra,
                &mut store.piece_thumbnail,
                Reader::text,
            ),
            other => reader.unknown(other, value, extra),
        }
    }
    // The object blanks what it does not set. The game reads the member as "" without a
    // line (it has no text for `piece_store_name` or `piece_thumbnail`): an information of
    // the crate's own.
    for (key, set) in [
        ("piece_store_name", store.piece_store_name.is_some()),
        ("piece_thumbnail", store.piece_thumbnail.is_some()),
    ] {
        if !set && !written_in(&store.extra, key) {
            reader.report_at(
                key,
                DiagnosticCode::from(&core_codes::MISSING_REQUIRED_FIELD),
                Severity::Info,
            );
        }
    }
    // The reader keeps no member in the document's table: it holds findings only.
    findings.extend(reader.findings);
    store
}
