//! Saved pack settings: the values a world stored for a pack's `settings` definitions (the
//! definitions themselves are `manifest::Setting`).

pub mod codes;
pub mod history;

// Pack settings are user-adjustable controls a format-3 manifest declares (`manifest::Setting`).
// The values a player chose are kept outside the pack, in a small versioned save document: the
// behaviour packs' values per world (`world_behavior_pack_settings.json` in the world folder),
// the resource packs' values per game (`global_resource_pack_settings.json` in the settings
// directory). `SavedSettings` is that document at its newest format (1.26.40); the runtime
// values of one pack, built from its definitions and its saved entry, are `PackSettings`; and
// `SavedSettings::save` is the game's save algorithm over them.

mod read;
mod runtime;
mod save;
mod types;
mod write;

#[cfg(test)]
mod tests;

pub use runtime::unmatched_entries;
pub use runtime::{PackSettings, SetError, SettingValueAndDefault};
pub use save::{FileState, SaveRefused};
pub use types::{SavedSettings, SettingValue, Settings, SettingsFile};

/// The file a world keeps its behaviour packs' setting values in, in the world folder.
pub const WORLD_FILE: &str = "world_behavior_pack_settings.json";
/// The file the game keeps its resource packs' setting values in, in its settings directory;
/// every world shares it.
pub const GLOBAL_FILE: &str = "global_resource_pack_settings.json";
/// The most entries a save keeps in the global file: the most recently used ones.
pub const MAX_GLOBAL_ENTRIES: usize = 128;

use std::collections::HashSet;
use std::convert::Infallible;

use crate::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity, Span};
use crate::document::{Document, Origin, Rejected};
use crate::json::{
    EnvelopeVersion, JsonSlice, MAX_JSON_DEPTH, RootProbe, check_depth, probe_envelope, strip_bom,
};
use crate::version::{FormatVersion, FormatVersionInput};
use crate::versioning::{
    BelowFloor, Envelope, EnvelopeCheck, Family, Identity, Loaded, MissingVersion, NoComponents,
    RangeModel, Selection, ValidateCx, Versioned, finish, parse_as_written, select,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use history::{PackSettingsFamily, PackSettingsHistory};

/// The floor, and the only version the released game reads.
const FLOOR: FormatVersion = FormatVersion::new(1, 21, 100);

impl Family for PackSettingsFamily {
    const NAME: &'static str = "pack_settings";
    const PAYLOAD_KEY: &'static str = "minecraft:pack_settings";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // Below 1.21.100 (or without a version) the text is kept raw and no envelope check runs.
        check: EnvelopeCheck::From(FLOOR),
        // The caller passes an empty version: the game's message is "no valid version passed by the
        // caller or found in the input".
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    // The loader's last range is the newest struct's version exactly.
    const RANGE_MODEL: RangeModel = RangeModel::ClosedToLatest;
    type Latest = SavedSettings;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = PackSettingsHistory;

    /// The document has no `description`.
    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    /// A second entry for a pack UUID: the game reads and updates only the first and carries
    /// the others along silently. The crate reports it as an information.
    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        let Loaded::Stable(saved) = v else {
            return;
        };
        let mut seen = HashSet::new();
        for (index, entry) in saved.settings.iter().enumerate() {
            if seen.insert(entry.pack_id) {
                continue;
            }
            let mut at = cx.at.clone();
            at.push("settings");
            at.push_index(index);
            at.push("pack_id");
            // The game takes the first match and carries the others in silence, so the crate
            // reports it as an information.
            cx.diagnostics.push(
                Diagnostic::of(&core_codes::DUPLICATE_IDENTIFIER)
                    .severity(Severity::Info)
                    .at(at)
                    .phase(Phase::Validate),
            );
        }
    }
}

impl SavedSettings {
    /// Parses the bytes of a saved-settings file (the file is read by the caller,
    /// `bedrock_addon_std::read_saved_settings`), upgraded to the newest struct. The findings
    /// name the world file ([`WORLD_FILE`]); [`parse_named`](Self::parse_named) names another.
    ///
    /// A missing file is no error: it holds no values (`SavedSettings::default()`). A file that
    /// fails is rejected whole, as the game drops it and every pack in it uses its defaults; its
    /// last finding is the game's "Pack Settings failed to load \[…\]".
    ///
    /// Rejected: no `format_version`, one that is neither a version string nor three integers,
    /// a version below 1.21.100 or above 1.26.40; at 1.21.100 and above, a root that has not
    /// exactly `format_version` and one other member; an unknown member, a missing `settings`,
    /// `pack_id` or `values`, a `pack_id` that is no string, a `last_used` that is no unsigned
    /// integer, and a value that is not a number, a boolean, a string or (at 1.26.40) an array
    /// of strings.
    ///
    /// Loaded with a finding, each an information: a `pack_id` string that is no UUID (it loads
    /// as the nil UUID), a second entry for a pack (kept), a payload member not named
    /// `minecraft:pack_settings`, a version written as three integers, and a version between
    /// 1.21.100 and 1.26.40, which the released game does not read.
    ///
    /// # Examples
    ///
    /// ```
    /// use bedrock_addon::pack::settings::{SavedSettings, SettingValue};
    /// use uuid::Uuid;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let text = br#"{
    ///   "format_version": "1.26.40",
    ///   "minecraft:pack_settings": {
    ///     "settings": [
    ///       { "pack_id": "00000000-0000-0000-0000-000000000001", "values": { "demo:glow": true } }
    ///     ]
    ///   }
    /// }"#;
    /// let document = SavedSettings::parse(text)?;
    /// let entry = document.value().get(Uuid::from_u128(1)).ok_or("no entry for the pack")?;
    /// assert_eq!(entry.values["demo:glow"], SettingValue::Bool(true));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Rejected`] when the file is not a valid settings document; the rejection carries every
    /// finding.
    pub fn parse(bytes: &[u8]) -> Result<Document<Self>, Rejected> {
        Self::parse_named(WORLD_FILE, bytes)
    }

    /// [`parse`](Self::parse) of the file the caller calls `name` (its path as the caller
    /// shows it, which the closing finding of a rejected file quotes). The document's origin
    /// is the file's last path segment: the global file ([`GLOBAL_FILE`]) is on the resource
    /// side, every other file on the behaviour side.
    ///
    /// # Errors
    ///
    /// [`Rejected`] when the file is not a valid settings document; the rejection carries every
    /// finding.
    pub fn parse_named(name: &str, bytes: &[u8]) -> Result<Document<Self>, Rejected> {
        let origin = origin(name);
        let mut found = Diagnostics::default();
        let loaded = load(bytes, &mut found);
        if found.has_errors() {
            // Once per file, after the loader's own entries (error level, the packs log area).
            found.push(error(codes::LOAD_FAILED.with([name])));
        }
        let diagnostics: Diagnostics = found
            .into_iter()
            .map(|mut finding| {
                finding.location.pack = Some(origin.pack.clone());
                finding.location.path = Some(origin.path.clone());
                finding
            })
            .collect();
        match loaded {
            Some((settings, version)) if !diagnostics.has_errors() => {
                Ok(Document::new(settings, origin, version, diagnostics))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// The text of `bytes` through the chain: the value and its version information, or `None`
/// (the reason is in `d`).
fn load(
    bytes: &[u8],
    d: &mut Diagnostics,
) -> Option<(SavedSettings, crate::document::VersionInfo)> {
    let bytes = &*crate::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    if let Err(too_deep) = check_depth(bytes, MAX_JSON_DEPTH) {
        d.push(error(malformed(too_deep.to_string())));
        return None;
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(failure) => {
            d.push(error(malformed(failure.to_string())));
            return None;
        }
    };
    let envelope = probe_envelope(JsonSlice::new(text)).ok();
    let written = envelope
        .as_ref()
        .map_or(EnvelopeVersion::Absent, |envelope| {
            envelope.format_version.clone()
        });
    // At or above the first modern version the payload member's name is not enforced (the game
    // only asserts it): the other member is the payload whatever it is called. The crate loads
    // it the same way and reports an information.
    let foreign = envelope
        .as_ref()
        .and_then(|envelope| foreign_payload_key(envelope, text));
    let renamed = foreign.as_ref().map(|key| key.rename(text));
    let document = JsonSlice::new(renamed.as_deref().unwrap_or(text));
    if let Some(key) = foreign.as_ref().filter(|key| key.warns) {
        let mut at = JsonPath::new();
        at.push(&key.name);
        // The game only asserts the payload key (in a development build) and the payload
        // loads, so the crate reports an information.
        d.push(
            Diagnostic::of(&core_codes::PAYLOAD_KEY_MISMATCH)
                .at(at)
                .span(key.span()),
        );
    }
    let cx = LoadContext::detached();
    let parsed = parse_as_written::<PackSettingsFamily>(document, &cx);
    let outcome = finish(parsed, &cx);
    for mut finding in outcome.diagnostics {
        if let (Some(key), Some(span)) = (&foreign, &mut finding.location.span) {
            key.restore(span);
        }
        d.push(finding);
    }
    let value = outcome.value?;
    // The game reads a version written as three integers too.
    if matches!(written, EnvelopeVersion::Array(_)) {
        d.push(info(
            Code::from(&core_codes::FORMAT_VERSION_ARRAY),
            version_at(),
        ));
    }
    // The released game's only range is [1.21.100, 1.21.100]; the crate reads up to the newest
    // struct.
    let effective = outcome.version.effective;
    if FLOOR < effective && effective < <SavedSettings as Versioned>::VERSION {
        d.push(info(
            Code::from(&core_codes::VANILLA_REJECTS_VERSION),
            version_at(),
        ));
    }
    match value {
        Loaded::Stable(settings) => Some((settings, outcome.version)),
        Loaded::Beta(never) => match never {},
    }
}

/// A payload member whose name is not `minecraft:pack_settings`, in a document whose envelope
/// vanilla checks (a version of at least 1.21.100, exactly two root members).
struct ForeignKey {
    /// The member's name.
    name: String,
    /// The byte range of its quoted name in the file.
    start: usize,
    end: usize,
    /// Whether the name is reported: the version has a parser.
    warns: bool,
}

/// The quoted name of the family's payload key, which replaces a foreign one.
const PAYLOAD_TOKEN: &str = "\"minecraft:pack_settings\"";

impl ForeignKey {
    /// The text with the member renamed to the family's payload key.
    fn rename(&self, text: &str) -> String {
        let mut renamed = String::with_capacity(text.len() + PAYLOAD_TOKEN.len());
        renamed.push_str(&text[..self.start]);
        renamed.push_str(PAYLOAD_TOKEN);
        renamed.push_str(&text[self.end..]);
        renamed
    }

    /// The quoted name's span in the file.
    fn span(&self) -> Span {
        Span {
            offset: u32::try_from(self.start).unwrap_or(u32::MAX),
            len: u32::try_from(self.end - self.start).unwrap_or(u32::MAX),
        }
    }

    /// Moves a span of the renamed text back to the file's offsets.
    fn restore(&self, span: &mut Span) {
        let renamed_end = self.start + PAYLOAD_TOKEN.len();
        let offset = usize::try_from(span.offset).unwrap_or(usize::MAX);
        let len = usize::try_from(span.len).unwrap_or(usize::MAX);
        if offset >= renamed_end {
            // After the name: moved by the difference of the two names.
            let original = offset - renamed_end + self.end;
            span.offset = u32::try_from(original).unwrap_or(u32::MAX);
        } else if offset <= self.start && offset.saturating_add(len) >= renamed_end {
            // Around the name (the whole document): as long as the file's text.
            let original = len - PAYLOAD_TOKEN.len() + (self.end - self.start);
            span.len = u32::try_from(original).unwrap_or(u32::MAX);
        }
    }
}

/// The envelope's foreign payload member, if it has one: one member besides `format_version`,
/// not named `minecraft:pack_settings`, under a version whose envelope is checked (the literal
/// `"beta"`, or a version of at least 1.21.100; below it the text is read raw and the name is
/// never looked at).
fn foreign_payload_key(envelope: &RootProbe<'_>, text: &str) -> Option<ForeignKey> {
    let version = written_version(&envelope.format_version)?;
    let checked = version.beta_literal || version.version.is_some_and(|v| v >= FLOOR);
    if !checked || envelope.members.len() != 1 {
        return None;
    }
    let (name, value) = envelope.members.first()?;
    if name == PackSettingsFamily::PAYLOAD_KEY {
        return None;
    }
    let (start, end) = key_token(text, value.offset)?;
    Some(ForeignKey {
        name: name.clone(),
        start,
        end,
        // Vanilla finds no parser before it would look at the payload: only a version that
        // selects a struct gets the finding.
        warns: matches!(select::<PackSettingsFamily>(&version), Selection::Struct(_)),
    })
}

/// The envelope's version as the loader reads it; `None` when absent or malformed.
fn written_version(version: &EnvelopeVersion) -> Option<FormatVersionInput> {
    match version {
        EnvelopeVersion::String(text) => FormatVersionInput::parse(text),
        EnvelopeVersion::Array([major, minor, patch]) => Some(FormatVersionInput {
            version: Some(FormatVersion::new(
                u16::try_from(*major).ok()?,
                u16::try_from(*minor).ok()?,
                u16::try_from(*patch).ok()?,
            )),
            beta_literal: false,
            prerelease: false,
        }),
        _ => None,
    }
}

/// The byte range of the quoted name of the root member whose value starts at `value_offset`:
/// the root object is walked token by token, over strings, comments and nested values.
fn key_token(text: &str, value_offset: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut depth = 0_usize;
    // The last string at the root level, and the one a colon followed.
    let mut last = None;
    let mut named = None;
    let mut i = 0;
    loop {
        i = skip_blank(bytes, i);
        let byte = *bytes.get(i)?;
        if let Some(token) = named.take()
            && i == value_offset
        {
            return Some(token);
        }
        match byte {
            b'"' => {
                let end = string_end(bytes, i)?;
                last = (depth == 1).then_some((i, end));
                i = end;
            }
            b':' => {
                if depth == 1 {
                    named = last.take();
                }
                i += 1;
            }
            b'{' | b'[' => {
                depth += 1;
                last = None;
                i += 1;
            }
            b'}' | b']' => {
                depth = depth.checked_sub(1)?;
                last = None;
                i += 1;
            }
            _ => {
                last = None;
                i += 1;
            }
        }
    }
}

/// The index after the whitespace and the `//` and `/* */` comments that start at `i`.
fn skip_blank(bytes: &[u8], mut i: usize) -> usize {
    loop {
        match (bytes.get(i), bytes.get(i + 1)) {
            (Some(byte), _) if byte.is_ascii_whitespace() => i += 1,
            (Some(b'/'), Some(b'/')) => {
                i = bytes[i..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .map_or(bytes.len(), |line| i + line + 1);
            }
            (Some(b'/'), Some(b'*')) => {
                i = bytes[i + 2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |close| i + 2 + close + 2);
            }
            _ => return i,
        }
    }
}

/// The index after the string whose opening quote is at `start`.
fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start + 1;
    loop {
        match *bytes.get(i)? {
            b'"' => return Some(i + 1),
            b'\\' => i += 2,
            _ => i += 1,
        }
    }
}

/// The path of the envelope's `format_version`.
fn version_at() -> JsonPath {
    let mut at = JsonPath::new();
    at.push("format_version");
    at
}

/// An info-level parse finding.
fn info(code: Code, at: JsonPath) -> Diagnostic {
    Diagnostic::info(code).at(at)
}

/// The document's origin: the app-data layer (the file belongs to no pack), the side of the
/// packs whose values the file holds, and the file's name.
fn origin(name: &str) -> Origin {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let side = if last == GLOBAL_FILE {
        PackSide::Resource
    } else {
        PackSide::Behavior
    };
    let path = PackPath::new(last).unwrap_or_else(|_| PackPath::from_static(WORLD_FILE));
    Origin::file(PackRef::app_data(), side, path)
}

/// A syntax finding without vanilla's text: bytes that are no JSON text at all.
fn malformed(message: String) -> Code {
    bedrock_addon_core::diagnostic::builders::syntax(message)
}

/// A file-level parse error.
fn error(code: Code) -> Diagnostic {
    Diagnostic::error(code)
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn pack_settings_reads_as_pinned() {
        assert_reading_table(
            "PackSettingsFamily",
            &reading_table::<super::PackSettingsFamily>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 100 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.100..=1.26.40 (0..=1): json Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
            ",
        );
    }
}

#[cfg(test)]
mod identity {
    //! What the family's documents say about themselves before they are parsed.

    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::{Family, Identity, parse_as_written};

    /// The family reads no identity from its payload, not even a `description.identifier`
    /// written in it (the default would).
    #[test]
    fn pack_settings_reads_no_identity_from_the_payload() {
        let text = format!(
            r#"{{"format_version": "1.21.0", "{}": {{"description": {{"identifier": "demo:x"}}}}}}"#,
            <super::PackSettingsFamily as Family>::PAYLOAD_KEY,
        );
        let parsed = parse_as_written::<super::PackSettingsFamily>(
            JsonSlice::new(&text),
            &LoadContext::detached(),
        );
        assert_eq!(parsed.identity, Identity::default());
    }
}
