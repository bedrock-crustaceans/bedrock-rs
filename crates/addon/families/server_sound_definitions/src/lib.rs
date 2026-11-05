//! Server sound definitions: a behaviour pack's `sounds/sound_definitions.json` with the
//! `minecraft:server_sound_definitions` payload, the metadata the server keeps about a sound
//! it plays. One stable struct at exactly 1.26.60; no other version, `"beta"` included, has a
//! parser. The file is read strictly (an
//! unknown member, a wrong type, a `null` member or a failed constraint rejects the whole
//! file).

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;

#[doc(hidden)]
pub use load::load_registry;

use facet::Facet;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, NullRule, StrictFloat, StrictString, Value, keep_unknown_members,
    parse_at, reject_nulls,
};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    EnvelopeCheck, ErrorMask, Family, InputKind, Loaded, MissingVersion, NoComponents, ParseCx,
    StructIndex, ValidateCx, VersionStruct, Versioned, write_payload,
};

/// The version of the one struct, and of the one slice: the game's document format is
/// 1.26.60.
pub(crate) const V1_26_60: FormatVersion = FormatVersion::new(1, 26, 60);

/// The engine version whose struct first binds `broadcast_range`; an older engine does not
/// know the member.
pub(crate) const BROADCAST_RANGE_SINCE: FormatVersion = FormatVersion::new(1, 26, 50);

/// The regex of the string constraint on a sound event name.
const NO_WHITESPACE_REGEX: &str = "^\\S+$";
/// The namespace constraint's text for a name without `namespace:value`.
const NEEDS_NAMESPACE: &str = "string must be prefixed with a namespace (eg. namespace:value)";
/// The namespace constraint's text for a disallowed namespace.
const INVALID_NAMESPACE: &str = "string has invalid namespace ({})";
/// The namespace no pack may define server sounds in, compared without case.
const DISALLOWED_NAMESPACE: &str = "minecraft";

/// A behaviour pack's server sound definitions file, `sounds/sound_definitions.json`
/// (`#[doc(alias)]` `server_sound_definitions`): the payload is the array itself.
#[doc(alias = "server_sound_definitions")]
#[doc(alias = "sound_definitions.json")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct SoundMetadataFile(pub Vec<SoundMetadata>);

/// The server's metadata of one sound event (`bp.server_sound_definitions()`): how long it lasts, what
/// music it is, the tags scripts filter on, and how far a broadcast reaches. The audio itself
/// is the resource pack's.
#[doc(alias = "server_sound_definitions")]
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct SoundMetadata {
    /// `sound_event_name`: the namespaced sound event this is the metadata of. No whitespace;
    /// a namespace other than `minecraft` is required.
    pub sound_event_name: StrictString,
    /// `duration_info`: how long the sound lasts, so the server knows when it ends.
    pub duration_info: Option<DurationInfo>,
    /// `music_info`.
    pub music_info: Option<MusicInfo>,
    /// `tags`: pack-defined tags, each one value or a list of values; no key or value holds
    /// whitespace.
    pub tags: Option<IndexMap<String, TagValueList>>,
    /// `broadcast_range`: how far from the sound a broadcast play reaches a player (from
    /// engine 1.26.50; an older engine rejects the file).
    pub broadcast_range: Option<StrictFloat<f32>>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl SoundMetadata {
    /// The sound event name as an identifier.
    pub fn id(&self) -> SoundEventName {
        SoundEventName::parse(self.sound_event_name.0.as_str())
    }

    /// The duration as the server stores it; `None` without `duration_info`.
    pub fn duration(&self) -> Option<SoundDuration> {
        self.duration_info.as_ref().map(DurationInfo::duration)
    }
}

/// `duration_info`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct DurationInfo {
    /// `mode`: what the duration counts in.
    pub mode: DurationMode,
    /// `seconds`.
    pub seconds: StrictFloat<f32>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl DurationInfo {
    /// The duration as vanilla converts it at load: game ticks (`seconds` × 20) or
    /// milliseconds (`seconds` × 1000), computed in `f32` and truncated. A product outside
    /// the target's range is the target's minimum, as the x86 conversion gives it.
    // The game multiplies in single precision and converts with a truncating cast.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "vanilla converts with a truncating cast"
    )]
    pub fn duration(&self) -> SoundDuration {
        match self.mode {
            DurationMode::GameTime => {
                let ticks = self.seconds.0 * 20.0_f32;
                // 2^31 is exact in f32; anything at or past it, or NaN, is out of range.
                SoundDuration::Ticks(if (-2_147_483_648.0..2_147_483_648.0).contains(&ticks) {
                    ticks as i32
                } else {
                    i32::MIN
                })
            }
            DurationMode::RealTime => {
                let millis = self.seconds.0 * 1000.0_f32;
                SoundDuration::Millis(
                    if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&millis)
                    {
                        millis as i64
                    } else {
                        i64::MIN
                    },
                )
            }
        }
    }
}

/// What a duration counts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[non_exhaustive]
#[repr(u8)]
pub enum DurationMode {
    /// `game_time`: the duration decrements by game tick.
    #[facet(rename = "game_time")]
    GameTime = 0,
    /// `real_time`: the duration decrements by wall-clock time.
    #[facet(rename = "real_time")]
    RealTime = 1,
}

/// A duration as the server tracks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SoundDuration {
    /// Game ticks.
    Ticks(i32),
    /// Milliseconds.
    Millis(i64),
}

/// `music_info`: free-form music metadata, every member optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MusicInfo {
    /// `genres`.
    pub genres: Option<Vec<StrictString>>,
    /// `moods`.
    pub moods: Option<Vec<StrictString>>,
    /// `artist`: display text or a translation key.
    pub artist: Option<StrictString>,
    /// `title`: display text or a translation key.
    pub title: Option<StrictString>,
    /// Unknown members, each of which rejects the file.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The value of one tag: one string or a list of strings.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum TagValueList {
    /// A single value.
    One(String),
    /// A list of values; it may be empty and may repeat a value.
    Many(Vec<String>),
}

impl TagValueList {
    /// The values: a single value is a list of one.
    pub fn values(&self) -> &[String] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::Many(values) => values,
        }
    }
}

/// The server sound definitions family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MetadataFamily;

bedrock_addon_core::version_chain! {
    family = MetadataFamily;
    /// A server sound definitions document: there is the 1.26.60 struct only.
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) enum MetadataAny {
        /// Format 1.26.60 exactly.
        V1_26_60(SoundMetadataFile),
    }
}

impl MolangFields for SoundMetadataFile {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl Versioned for SoundMetadataFile {
    const VERSION: FormatVersion = V1_26_60;
}

/// The tag values that are neither a string nor an array of strings.
/// facet reads a number into the untagged string arm, so the JSON is checked first.
fn mistyped_tags(payload: &Value) -> Vec<JsonPath> {
    let mut out = Vec::new();
    let definitions = payload.as_array().map(facet_value::VArray::as_slice);
    for (index, definition) in definitions.into_iter().flatten().enumerate() {
        let tags = definition
            .as_object()
            .and_then(|definition| definition.get("tags"))
            .and_then(Value::as_object);
        for (key, value) in tags.into_iter().flatten() {
            let strings = |value: &Value| value.is_string();
            let fits = strings(value)
                || value
                    .as_array()
                    .is_some_and(|values| values.as_slice().iter().all(strings));
            if !fits {
                out.push(
                    JsonPath::new()
                        .index(index)
                        .child("tags")
                        .child(key.as_str()),
                );
            }
        }
    }
    out
}

impl VersionStruct for SoundMetadataFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok();
        if json.as_ref().is_some_and(|json| !json.is_array()) {
            // The payload is the array itself.
            d.push(Diagnostic::error(DiagnosticCode::InvalidType { text: None }).at(cx.at.clone()));
            return None;
        }
        let mistyped = json.as_ref().map(mistyped_tags).unwrap_or_default();
        if !mistyped.is_empty() {
            for at in &mistyped {
                d.push(
                    Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                        .at(cx.at.join(at)),
                );
            }
            return None;
        }
        let mut value: Self = match parse_at::<Self>(payload) {
            Ok(value) => value,
            Err(error) => {
                d.push(error.into_diagnostic(&cx.at, |code, _| code));
                return None;
            }
        };
        // A member written as `null` is present and fails its type: the game reads the file
        // with null members rejected.
        if cx.load.null_rule == Some(NullRule::Rejected) && reject_nulls(&value, payload, &cx.at, d)
        {
            return None;
        }
        // The strict mask: an unknown member at any level rejects the file.
        let unknown = json
            .map(|json| keep_unknown_members(&mut value, &json))
            .unwrap_or_default();
        for (path, _) in &unknown {
            d.push(
                Diagnostic::error(DiagnosticCode::UnrecognizedField { text: None })
                    .at(cx.at.join(path)),
            );
        }
        // An error-level finding rejects the document, so the value goes no further.
        Some(value)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl Family for MetadataFamily {
    const NAME: &'static str = "server_sound_definitions";
    const PAYLOAD_KEY: &'static str = "minecraft:server_sound_definitions";
    // One fixed file per behaviour pack; the loader's walk names it.
    const DIRECTORIES: &'static [&'static str] = &[];
    // The one slice covers `[1.26.60, end]`, where the end is the larger of the slice's version
    // and the game's current version, which is 1.26.60: the engine version closes it.
    const FLOOR: FormatVersion = V1_26_60;
    type Latest = SoundMetadataFile;
    // No Beta slice: `"beta"` has no parser.
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = MetadataAny;
    // The game's loader is given no caller version, so a file without `format_version` is
    // rejected.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The envelope is checked only at or above `minModernVersion`, the slice's version.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(V1_26_60);
    // The loader reads the document as JSON input: the version as the game reads it from the
    // text, and `null` members rejected.
    const INPUT: InputKind = InputKind::Document;

    // The slice is registered with the strict error mask: any parse error rejects the file.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        let Loaded::Stable(file) = v else { return };
        validate(file, cx);
    }
}

/// Whether `text` matches `^\S+$`: not empty and without whitespace.
fn no_whitespace(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(|c| c.is_whitespace() || c == '\u{feff}')
}

/// The namespace rule of the public area: `None` when `name` passes,
/// else the constraint's text.
// The game checks two case-insensitive patterns: the first forbids a `minecraft:` prefix, the
// second wants one `:` with text on both sides.
fn namespace_text(name: &str) -> Option<VanillaText> {
    let bytes = name.as_bytes();
    // A `:` with at least one character on each side.
    let prefixed = bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| *byte == b':' && index >= 1 && index + 1 < bytes.len());
    if !prefixed {
        return Some(VanillaText::Fixed(NEEDS_NAMESPACE));
    }
    let disallowed = name
        .split_once(':')
        .is_some_and(|(namespace, _)| namespace.eq_ignore_ascii_case(DISALLOWED_NAMESPACE));
    disallowed.then(|| VanillaText::Formatted {
        template: INVALID_NAMESPACE,
        args: vec![DISALLOWED_NAMESPACE.to_owned()],
    })
}

/// The rules on the loaded file. An error rejects it.
fn validate(file: &SoundMetadataFile, cx: &mut ValidateCx<'_>) {
    let base = cx.at.clone();
    let mut push = |code: DiagnosticCode, severity: Severity, index: usize, path: &[&str]| {
        let at = base.index(index).with(path);
        cx.diagnostics.push(
            Diagnostic::error(code)
                .at(at)
                .severity(severity)
                .phase(Phase::Validate),
        );
    };
    let pattern = || bedrock_addon_core::diagnostic::cereal::string_mismatch(NO_WHITESPACE_REGEX);
    for (index, definition) in file.0.iter().enumerate() {
        let name = definition.sound_event_name.0.as_str();
        if let Some(text) = namespace_text(name) {
            push(
                DiagnosticCode::NamespaceRule { text: Some(text) },
                Severity::Error,
                index,
                &["sound_event_name"],
            );
        }
        if !no_whitespace(name) {
            push(pattern(), Severity::Error, index, &["sound_event_name"]);
        }
        for (key, value) in definition.tags.iter().flatten() {
            if !no_whitespace(key) || !value.values().iter().all(|value| no_whitespace(value)) {
                push(pattern(), Severity::Error, index, &["tags", key.as_str()]);
            }
        }
        if let Some(duration) = &definition.duration_info {
            let seconds = duration.seconds.0;
            if !seconds.is_finite() || seconds <= 0.0 {
                // The crate's own information: vanilla converts any value without a check.
                push(
                    DiagnosticCode::InvalidValue { text: None },
                    Severity::Info,
                    index,
                    &["duration_info", "seconds"],
                );
            }
        }
        if let Some(range) = &definition.broadcast_range {
            if cx.load.engine_version < BROADCAST_RANGE_SINCE {
                // An engine below 1.26.50 does not bind the member.
                push(
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Error,
                    index,
                    &["broadcast_range"],
                );
            } else if range.0 <= 0.0 {
                // The crate's own information: the game binds any number without a check.
                push(
                    DiagnosticCode::InvalidValue { text: None },
                    Severity::Info,
                    index,
                    &["broadcast_range"],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::json::NullRule;
    use bedrock_addon_core::version::FormatVersionInput;
    use bedrock_addon_core::versioning::{
        ErrorMask, InputKind, Selection, StructIndex, VersionChain, select,
    };

    /// The `to_json` the family once wrote by hand, the chain's text as is, equals `Content`'s
    /// default, which lays that text out again: reals included.
    #[test]
    fn the_default_to_json_is_the_chain_text_as_is() {
        use bedrock_addon_core::document::Content;
        use bedrock_addon_core::load::LoadContext;
        use bedrock_addon_core::pack_identity::PackPath;
        let text = r#"{"format_version": "1.26.60", "minecraft:server_sound_definitions": [
            {"sound_event_name": "demo:x", "duration_info": {"mode": "real_time", "seconds": 1.2345},
             "broadcast_range": 64.0},
            {"sound_event_name": "demo:y", "duration_info": {"mode": "game_time", "seconds": 0.1},
             "broadcast_range": 16.5}
        ]}"#;
        let document = <SoundMetadataFile as Content>::parse(
            &PackPath::from_static("sounds/sound_definitions.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("loads");
        let file = document.value();
        assert_eq!(
            file.to_json().unwrap(),
            MetadataAny::V1_26_60(file.clone()).to_json().unwrap()
        );
    }

    #[test]
    fn the_chain_is_the_stable_struct_alone() {
        // One slice at 1.26.60 with the strict mask, then the engine version closes it; there
        // is no beta slice.
        assert_eq!(
            <MetadataAny as VersionChain<MetadataFamily>>::VERSIONS,
            [FormatVersion::new(1, 26, 60)]
        );
        const { assert!(!<MetadataAny as VersionChain<MetadataFamily>>::HAS_BETA) };
        assert_eq!(MetadataFamily::INPUT, InputKind::Document);
        assert_eq!(
            MetadataFamily::null_rule(StructIndex::Stable(0)),
            Some(NullRule::Rejected)
        );
        assert_eq!(
            MetadataFamily::error_mask(StructIndex::Stable(0)),
            ErrorMask::Strict
        );
        assert_eq!(
            MetadataFamily::PAYLOAD_KEY,
            "minecraft:server_sound_definitions"
        );
    }

    #[test]
    fn only_the_current_format_selects_the_struct() {
        let selects =
            |text: &str| select::<MetadataFamily>(&FormatVersionInput::parse(text).unwrap());
        assert_eq!(selects("1.26.60"), Selection::Struct(0));
        for other in [
            "1.26.40",
            "1.26.50",
            "1.26.60-beta.1",
            "1.26.61",
            "9999.9999.9999",
        ] {
            assert!(!matches!(selects(other), Selection::Struct(_)), "{other}");
        }
    }

    #[test]
    fn the_namespace_rule_follows_the_two_regexes() {
        let text = |name: &str| namespace_text(name).map(|text| text.to_string());
        assert_eq!(text("demo:click"), None);
        assert_eq!(text("a:b"), None);
        assert_eq!(text(":a:b"), None);
        for unprefixed in ["click", ":click", "demo:", ":", "", "minecraft:"] {
            assert_eq!(
                text(unprefixed).as_deref(),
                Some(NEEDS_NAMESPACE),
                "{unprefixed}"
            );
        }
        for disallowed in ["minecraft:a", "MineCraft:a"] {
            assert_eq!(
                text(disallowed).as_deref(),
                Some("string has invalid namespace (minecraft)"),
                "{disallowed}"
            );
        }
    }

    #[test]
    fn the_whitespace_pattern_is_non_empty_and_space_free() {
        assert!(no_whitespace("demo:x"));
        for bad in [
            "",
            " ",
            "a b",
            "a\tb",
            "a\n",
            "\u{a0}a",
            "a\u{2003}b",
            "a\u{feff}",
        ] {
            assert!(!no_whitespace(bad), "{bad:?}");
        }
    }
}
