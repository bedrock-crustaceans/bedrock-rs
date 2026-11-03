//! Versions: `SemVersion` for packs, `FormatVersion` for documents, the engine version and the
//! manifest's `min_engine_version` and `base_game_version`.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::json::Value;
use crate::molang::{EngineVersion, MolangVersion};

/// A pack version: the manifest's `header.version` and `dependencies[].version`.
///
/// Vanilla's `SemVersion`: three `u16` numbers, a pre-release identifier list and build
/// metadata, or the any-version `*`. Ordering: the any-version is the maximum; otherwise the
/// numbers compare numerically, then a version with a pre-release sorts below the same numbers
/// without one, and pre-release lists compare identifier by identifier (numeric identifiers
/// numerically and below alphanumeric ones, a strict prefix lower). Build metadata takes no part
/// in ordering or equality, and two any-versions are equal only to each other.
#[derive(Clone, Debug)]
pub struct SemVersion {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
    /// The pre-release identifiers joined with `.` (`beta.3`); empty for a release.
    pub pre_release: String,
    /// The build metadata (`build.7`); empty when there is none.
    pub build: String,
    /// The any-version `*`; the numbers are then 0 and ignored.
    pub any: bool,
}

impl SemVersion {
    /// The any-version `*`, greater than every other version.
    pub const ANY: SemVersion = SemVersion {
        major: 0,
        minor: 0,
        patch: 0,
        pre_release: String::new(),
        build: String::new(),
        any: true,
    };

    /// The release `major.minor.patch`.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: String::new(),
            build: String::new(),
            any: false,
        }
    }

    /// Reads vanilla's string form, whose grammar is the engine's semantic-version regular
    /// expression: `major[.minor[.patch[-pre.release][+build]]]`, each number `0` or one to
    /// five digits without a leading zero and at most 65 535; missing numbers are 0 (`"1.10"`
    /// is 1.10.0); a pre-release or build only after a patch; a numeric pre-release identifier
    /// has no leading zero, a build identifier may. `"*"` is the any-version and `"beta"` the
    /// beta sentinel 9999.9999.9999-beta. `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "*" => return Some(Self::ANY),
            "beta" => {
                return Some(Self {
                    pre_release: "beta".to_owned(),
                    ..Self::new(9999, 9999, 9999)
                });
            }
            _ => {}
        }
        let (rest, build) = match text.split_once('+') {
            Some((rest, build)) => (rest, Some(build)),
            None => (text, None),
        };
        // The numbers hold no `-`, so the first one starts the pre-release.
        let (core, pre_release) = match rest.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (rest, None),
        };
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() > 3 || ((pre_release.is_some() || build.is_some()) && parts.len() != 3) {
            return None;
        }
        let mut numbers = [0_u16; 3];
        for (slot, part) in numbers.iter_mut().zip(&parts) {
            *slot = regex_number(part)?;
        }
        let pre_release = match pre_release {
            None => String::new(),
            Some(list) => list
                .split('.')
                .all(pre_release_identifier)
                .then(|| list.to_owned())?,
        };
        let build = match build {
            None => String::new(),
            Some(list) => list
                .split('.')
                .all(|id| !id.is_empty() && id.bytes().all(identifier_byte))
                .then(|| list.to_owned())?,
        };
        Some(Self {
            pre_release,
            build,
            ..Self::new(numbers[0], numbers[1], numbers[2])
        })
    }

    /// Reads a version from JSON: a string by [`parse`](Self::parse)'s rules, or an array of
    /// exactly three numbers, each non-negative, reals truncated,
    /// and none above 65 535 (0xFFFF). Anything else is `None`.
    pub fn from_json(value: &Value) -> Option<Self> {
        if let Some(text) = value.as_string() {
            return Self::parse(text.as_str());
        }
        let [major, minor, patch] = value.as_array()?.as_slice() else {
            return None;
        };
        let number = |value: &Value| -> Option<u16> {
            let number = value.as_number()?;
            if number.is_integer() {
                u16::try_from(number.to_u64()?).ok()
            } else {
                let real = number.to_f64_lossy();
                // jsoncpp rejects a negative real before truncating it (`-0.5` is not 0).
                if real < 0.0 {
                    return None;
                }
                let real = real.trunc();
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a truncated real in 0..=65535 is exactly one u16"
                )]
                (0.0..=f64::from(u16::MAX))
                    .contains(&real)
                    .then_some(real as u16)
            }
        };
        Some(Self::new(number(major)?, number(minor)?, number(patch)?))
    }

    /// Reads a version array as vanilla does (the manifest's legacy versions, a dialogue's
    /// `format_version`): exactly three elements, each
    /// convertible to an unsigned integer as jsoncpp converts (`null` is 0, `false` / `true`
    /// are 0 / 1, a non-negative real is truncated), none above 65 535. Anything else is
    /// `None`. [`from_json`](Self::from_json) takes numbers only.
    pub fn from_json_array(elements: &[Value]) -> Option<Self> {
        let [major, minor, patch] = elements else {
            return None;
        };
        let element = |value: &Value| -> Option<u16> {
            if value.is_null() {
                return Some(0);
            }
            if let Some(flag) = value.as_bool() {
                return Some(u16::from(flag));
            }
            let number = value.as_number()?;
            if number.is_integer() {
                return u16::try_from(number.to_u64()?).ok();
            }
            let real = number.to_f64_lossy();
            if !(0.0..f64::from(u16::MAX) + 1.0).contains(&real) {
                return None;
            }
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a real in 0..65536 truncates to one u16"
            )]
            Some(real.trunc() as u16)
        };
        Some(Self::new(element(major)?, element(minor)?, element(patch)?))
    }

    /// This is the any-version `*`.
    pub fn is_any(&self) -> bool {
        self.any
    }

    /// The version has a pre-release tag (`1.21.60-beta.3`).
    pub fn is_pre_release(&self) -> bool {
        !self.any && !self.pre_release.is_empty()
    }

    /// This version can stand in for `other` (vanilla's compatibility rule, used for
    /// premium-pack version matching): the same major and a minor at least `other`'s; the patch,
    /// pre-release and build take no part.
    pub fn satisfies(&self, other: &SemVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    /// The three numbers as a document version (pre-release and build dropped).
    pub fn format_version(&self) -> FormatVersion {
        FormatVersion::new(self.major, self.minor, self.patch)
    }
}

impl From<FormatVersion> for SemVersion {
    fn from(version: FormatVersion) -> Self {
        Self::new(version.major, version.minor, version.patch)
    }
}

impl PartialEq for SemVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for SemVersion {}

/// Hashes what equality compares: build metadata is left out, the any-version hashes alone, and
/// each pre-release identifier hashes as the ordering compares it (a numeric one by
/// value, so `beta.01` and `beta.1` hash alike).
impl Hash for SemVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.any.hash(state);
        if !self.any {
            (self.major, self.minor, self.patch).hash(state);
            for identifier in self.pre_release.split('.').filter(|id| !id.is_empty()) {
                match numeric(identifier) {
                    Some(value) => (0_u8, value).hash(state),
                    None => (1_u8, identifier).hash(state),
                }
            }
        }
    }
}

/// One number of `SemVerRegex`: `0|[1-9]\d{0,4}`, read as a `u16` (above 65 535 fails).
fn regex_number(part: &str) -> Option<u16> {
    let leading_zero = part.len() > 1 && part.starts_with('0');
    if part.is_empty()
        || part.len() > 5
        || leading_zero
        || !part.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    part.parse().ok()
}

/// A byte of a pre-release or build identifier: `[0-9A-Za-z-]`.
fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-'
}

/// One pre-release identifier of `SemVerRegex`: `[0-9A-Za-z-]+`, and a numeric one is `0` or
/// has no leading zero.
fn pre_release_identifier(id: &str) -> bool {
    let numeric = id.bytes().all(|b| b.is_ascii_digit());
    !id.is_empty()
        && id.bytes().all(identifier_byte)
        && !(numeric && id.len() > 1 && id.starts_with('0'))
}

/// The value of a numeric pre-release identifier; `None` for an alphanumeric one.
fn numeric(identifier: &str) -> Option<u64> {
    identifier.parse().ok()
}

impl PartialOrd for SemVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.any, other.any) {
            (true, true) => return Ordering::Equal,
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            (false, false) => {}
        }
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| compare_pre_release(&self.pre_release, &other.pre_release))
    }
}

/// A release sorts above any pre-release of it; two lists compare identifier by identifier.
fn compare_pre_release(a: &str, b: &str) -> Ordering {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }
    let mut a = a.split('.');
    let mut b = b.split('.');
    loop {
        match (a.next(), b.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let order = match (numeric(x), numeric(y)) {
                    (Some(x), Some(y)) => x.cmp(&y),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => x.cmp(y),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

/// `1.21.60-beta.3+build`, or `*`.
impl fmt::Display for SemVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.any {
            return f.write_str("*");
        }
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre_release.is_empty() {
            write!(f, "-{}", self.pre_release)?;
        }
        if !self.build.is_empty() {
            write!(f, "+{}", self.build)?;
        }
        Ok(())
    }
}

/// A document version: `format_version` as three numbers. Copy, const-constructible; documents
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FormatVersion {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
}

impl FormatVersion {
    /// `major.minor.patch`.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// `1.21.60`.
impl fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// What the envelope actually said about its version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatVersionInput {
    /// The version, when one parsed (`None` for the literal `"beta"`).
    pub version: Option<FormatVersion>,
    /// The literal `"beta"`.
    pub beta_literal: bool,
    /// A prerelease tag (`1.21.60-beta.3`).
    pub prerelease: bool,
}

impl FormatVersionInput {
    /// Reads a string `format_version` by [`SemVersion::parse`]'s rules: `"beta"` and
    /// `"9999.9999.9999-beta"` are the beta literal, a pre-release tag sets `prerelease`, build
    /// metadata is ignored. `None` (vanilla's "Malformed format version") for a string that is
    /// not a version and for the any-version `"*"`, which no document version can hold.
    pub fn parse(text: &str) -> Option<Self> {
        // `"9999.9999.9999-beta"` is the sentinel itself: every beta check compares the three
        // numbers and the pre-release `beta`, build metadata aside.
        let sentinel = || {
            SemVersion::parse(text).is_some_and(|v| {
                !v.any
                    && (v.major, v.minor, v.patch) == (9999, 9999, 9999)
                    && v.pre_release == "beta"
            })
        };
        if text == "beta" || sentinel() {
            return Some(Self {
                version: None,
                beta_literal: true,
                prerelease: false,
            });
        }
        let version = SemVersion::parse(text).filter(|v| !v.is_any())?;
        Some(Self {
            version: Some(version.format_version()),
            beta_literal: false,
            prerelease: version.is_pre_release(),
        })
    }

    /// Reads a JSON `format_version` as vanilla does: a string by [`parse`](Self::parse), or an
    /// array of exactly three integer-tagged, non-negative numbers that fit `u32`, each taken
    /// modulo 65 536. Anything else (a number, a bool, null,
    /// an object, an array with a real element) is `None`, "Malformed format version".
    pub fn from_json(value: &Value) -> Option<Self> {
        if let Some(text) = value.as_string() {
            return Self::parse(text.as_str());
        }
        let [major, minor, patch] = value.as_array()?.as_slice() else {
            return None;
        };
        let number = |value: &Value| -> Option<u64> {
            value.as_number().filter(|n| n.is_integer())?.to_u64()
        };
        Self::from_array([number(major)?, number(minor)?, number(patch)?])
    }

    /// The version of a three-integer array: each element must fit `u32` (`RapidJSON`'s
    /// unsigned-integer flag) and is stored in 16 bits, so a value above 65 535 is taken modulo
    /// 65 536 rather than refused.
    pub fn from_array(parts: [u64; 3]) -> Option<Self> {
        let [major, minor, patch] = parts.map(|part| {
            u32::try_from(part)
                .ok()
                .map(|part| u16::try_from(part & 0xffff).unwrap_or_default())
        });
        Some(Self {
            version: Some(FormatVersion::new(major?, minor?, patch?)),
            beta_literal: false,
            prerelease: false,
        })
    }

    /// Reads the version of a document the way the game's document input does (a scan of the
    /// raw text): the value of the first textual
    /// `"format_version"` key of `document`, wherever it sits, cut at the next `,` or `}` (an
    /// array at its `]`), and read with numbers as their text, so `1` is 1.0.0 and `1.21` is
    /// 1.21.0.
    pub fn pick(document: &str) -> PickedVersion {
        const KEY: &str = "\"format_version\"";
        let mut from = 0;
        while let Some(found) = document.get(from..).and_then(|rest| rest.find(KEY)) {
            let after = from + found + KEY.len();
            from = after;
            let rest = document
                .get(after..)
                .unwrap_or_default()
                .trim_start_matches(json_space);
            // A `"format_version"` that is not a key (a string value) is passed over.
            let Some(value) = rest.strip_prefix(':') else {
                continue;
            };
            let value = value.trim_start_matches(json_space);
            let fragment = if value.starts_with('[') {
                match value.find(']') {
                    Some(end) => value.get(..=end).unwrap_or_default(),
                    None => return PickedVersion::Malformed,
                }
            } else {
                let end = value.find([',', '}']).unwrap_or(value.len());
                value
                    .get(..end)
                    .unwrap_or_default()
                    .trim_end_matches(json_space)
            };
            return match Self::from_fragment(fragment) {
                Some((input, written)) => PickedVersion::Version { input, written },
                None => PickedVersion::Malformed,
            };
        }
        PickedVersion::Absent
    }

    /// One `format_version` value as the game's raw-text scan cut it from the text, and the
    /// version as written for messages (a string's text, a number's token, an array's
    /// numbers).
    fn from_fragment(fragment: &str) -> Option<(Self, String)> {
        if fragment.starts_with('"') || fragment.starts_with('[') {
            let json = crate::json::from_str_jsonc::<Value>(fragment).ok()?;
            let input = Self::from_json(&json)?;
            let written = match (json.as_string(), input.version) {
                (Some(text), _) => text.as_str().to_owned(),
                (None, Some(version)) => version.to_string(),
                (None, None) => String::new(),
            };
            return Some((input, written));
        }
        // A number token, read as its text.
        let input = is_json_number(fragment).then(|| Self::parse(fragment))??;
        Some((input, fragment.to_owned()))
    }
}

/// What [`FormatVersionInput::pick`] found in a document's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickedVersion {
    /// No `"format_version"` key in the text.
    Absent,
    /// The key with a value that gives no version ("Malformed format version").
    Malformed,
    /// The version.
    Version {
        /// What the value says.
        input: FormatVersionInput,
        /// The version as written, for messages.
        written: String,
    },
}

/// JSON white space.
fn json_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// The text is one JSON number token: `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
fn is_json_number(text: &str) -> bool {
    let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
    let rest = text.strip_prefix('-').unwrap_or(text);
    let int = digits(rest);
    if int == 0 || (int > 1 && rest.starts_with('0')) {
        return false;
    }
    let mut rest = &rest[int..];
    if let Some(fraction) = rest.strip_prefix('.') {
        let n = digits(fraction);
        if n == 0 {
            return false;
        }
        rest = &fraction[n..];
    }
    if let Some(exponent) = rest.strip_prefix(['e', 'E']) {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        let n = digits(exponent);
        if n == 0 {
            return false;
        }
        rest = &exponent[n..];
    }
    rest.is_empty()
}

/// A range of document versions, as a version chain selects structs by: `[min, max]` or
/// `[min, max)`. The four range models of a chain are built from these: inner ranges right-open,
/// the last closed (to the engine or the newest struct) or right-open, or every range exact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VersionRange {
    /// The lowest version in the range.
    pub min: FormatVersion,
    /// The upper bound.
    pub max: FormatVersion,
    /// `max` itself is in the range.
    pub max_inclusive: bool,
}

impl VersionRange {
    /// `[min, max]`.
    pub const fn closed(min: FormatVersion, max: FormatVersion) -> Self {
        Self {
            min,
            max,
            max_inclusive: true,
        }
    }

    /// `[min, max)`.
    pub const fn right_open(min: FormatVersion, max: FormatVersion) -> Self {
        Self {
            min,
            max,
            max_inclusive: false,
        }
    }

    /// `[version, version]`: only `version`.
    pub const fn exact(version: FormatVersion) -> Self {
        Self::closed(version, version)
    }

    /// `version` lies in the range.
    pub fn contains(&self, version: FormatVersion) -> bool {
        self.min <= version
            && if self.max_inclusive {
                version <= self.max
            } else {
                version < self.max
            }
    }

    /// The version an envelope gave lies in the range. A pre-release of `v` sorts below `v` and
    /// above every release below it (vanilla's range check compares the
    /// pre-release too), so it lies in a range that reaches `v` without starting at it: the
    /// struct below `v` takes `v-beta.n`, and `min-beta.n` is outside. The literal `"beta"` is
    /// in no range.
    pub fn contains_input(&self, input: &FormatVersionInput) -> bool {
        match input.version {
            None => false,
            Some(version) if input.prerelease => self.min < version && version <= self.max,
            Some(version) => self.contains(version),
        }
    }
}

/// `[1.20.0, 1.21.60)`.
impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let close = if self.max_inclusive { ']' } else { ')' };
        write!(f, "[{}, {}{close}", self.min, self.max)
    }
}

/// The command version of the engine the crate tracks (vanilla's current command version),
/// which is also what a pack without a usable `min_engine_version` gets:
/// `min.map_or(CURRENT_COMMAND_VERSION, MinEngineVersion::command_version)`.
pub const CURRENT_COMMAND_VERSION: u8 = 46;

/// What every `min_engine_version` below the first entry of the map gets (the earliest command
/// version the game supports).
const EARLIEST_COMMAND_VERSION: u8 = 8;

/// Vanilla's map from `min_engine_version` to command version, verbatim and in storage order:
/// the lowest `min_engine_version` of each command version. Two keys are stored more than once
/// (1.19.70 six times, 1.21.10 twice); the lookup returns the last value of a key, so 27 to 31
/// and 40 are never the version of a pack.
const COMMAND_VERSION_MAP: [((u16, u16, u16), u8); 39] = [
    ((1, 8, 0), 8),
    ((1, 9, 0), 9),
    ((1, 13, 0), 10),
    ((1, 15, 0), 11),
    ((1, 16, 0), 12),
    ((1, 16, 100), 13),
    ((1, 16, 210), 14),
    ((1, 16, 220), 15),
    ((1, 17, 30), 16),
    ((1, 17, 40), 17),
    ((1, 18, 10), 18),
    ((1, 18, 20), 19),
    ((1, 19, 0), 20),
    ((1, 19, 10), 21),
    ((1, 19, 20), 22),
    ((1, 19, 30), 23),
    ((1, 19, 40), 24),
    ((1, 19, 50), 25),
    ((1, 19, 60), 26),
    ((1, 19, 70), 27),
    ((1, 19, 70), 28),
    ((1, 19, 70), 29),
    ((1, 19, 70), 30),
    ((1, 19, 70), 31),
    ((1, 19, 70), 32),
    ((1, 19, 80), 33),
    ((1, 20, 0), 34),
    ((1, 20, 10), 35),
    ((1, 20, 30), 36),
    ((1, 20, 60), 37),
    ((1, 20, 70), 38),
    ((1, 21, 0), 39),
    ((1, 21, 10), 40),
    ((1, 21, 10), 41),
    ((1, 21, 20), 42),
    ((1, 21, 70), 43),
    ((1, 21, 90), 44),
    ((1, 21, 130), 45),
    ((1, 26, 10), 46),
];

/// The manifest's `header.min_engine_version`: the engine version a pack was written for, from
/// which vanilla derives the pack's Molang and command versions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MinEngineVersion {
    /// The version as read.
    pub version: SemVersion,
}

impl MinEngineVersion {
    /// What a format-1 (or 0) behaviour or resource pack without `min_engine_version` gets: 1.2.5.
    pub const FORMAT_1_DEFAULT: MinEngineVersion = MinEngineVersion::new(1, 2, 5);

    /// What a format-1 (or 0) manifest's `min_engine_version` of 1.13.0 or above is capped to:
    /// 1.12.0 (with vanilla's `min_engine_version_cap_format_version_1` warning).
    pub const FORMAT_1_CAP: MinEngineVersion = MinEngineVersion::new(1, 12, 0);

    /// `major.minor.patch`.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            version: SemVersion::new(major, minor, patch),
        }
    }

    /// The `min_engine_version` of a behaviour or resource pack whose manifest has format 1 or
    /// lower: [`FORMAT_1_DEFAULT`](Self::FORMAT_1_DEFAULT) when absent, and
    /// [`FORMAT_1_CAP`](Self::FORMAT_1_CAP) for a version of 1.13.0 or above.
    pub fn format_1(written: Option<SemVersion>) -> Self {
        match written {
            None => Self::FORMAT_1_DEFAULT,
            Some(version) => {
                let version = Self { version };
                if version.exceeds_format_1_cap() {
                    Self::FORMAT_1_CAP
                } else {
                    version
                }
            }
        }
    }

    /// The version is 1.13.0 or above, so a format-1 manifest caps it.
    pub fn exceeds_format_1_cap(&self) -> bool {
        self.version >= SemVersion::new(1, 13, 0)
    }

    /// The version as molang's engine version: the any-version, or the three numbers with a
    /// pre-release tag when the version has one (the tag text matters only as "present").
    pub fn engine_version(&self) -> EngineVersion {
        use semver::{Prerelease, Version};
        if self.version.any {
            return EngineVersion::Any;
        }
        let mut version = Version::new(
            u64::from(self.version.major),
            u64::from(self.version.minor),
            u64::from(self.version.patch),
        );
        if self.version.is_pre_release() {
            version.pre = Prerelease::new(&self.version.pre_release)
                .or_else(|_| Prerelease::new("pre"))
                .unwrap_or(Prerelease::EMPTY);
        }
        EngineVersion::Version(version)
    }

    /// The Molang version vanilla maps this engine version to: a
    /// pre-release sorts below its release and `*` is the newest.
    pub fn molang_version(&self) -> MolangVersion {
        MolangVersion::from(&self.engine_version())
    }

    /// The command version vanilla maps this engine version to, 8 to 46: the grammar and the
    /// behaviour of every command of the pack, in functions, tick lists, animation events and
    /// dialogue. It is the value of the last entry of vanilla's map at or below the version, 8
    /// below the first entry (1.8.0) and [`CURRENT_COMMAND_VERSION`] from the last (1.26.10). A
    /// pre-release sorts below its release (`1.16.0-beta` is 11) and `*` is the newest. An
    /// invalid version is the caller's `None`, which gets [`CURRENT_COMMAND_VERSION`].
    pub fn command_version(&self) -> u8 {
        COMMAND_VERSION_MAP
            .iter()
            .rev()
            .find(|((major, minor, patch), _)| {
                self.version >= SemVersion::new(*major, *minor, *patch)
            })
            .map_or(EARLIEST_COMMAND_VERSION, |(_, value)| *value)
    }
}

/// The manifest's `header.base_game_version` (world templates) and the world's base-game version.
///
/// Vanilla's `BaseGameVersion`: a version, the any-version `*` ([`ANY`](Self::ANY)), no version
/// ([`EMPTY`](Self::EMPTY), the default) or never compatible
/// ([`INCOMPATIBLE`](Self::INCOMPATIBLE)).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BaseGameVersion {
    /// The version; `None` when unset or invalid.
    pub version: Option<SemVersion>,
    /// Compatible with nothing.
    pub never_compatible: bool,
}

impl BaseGameVersion {
    /// `*`: every base-game version.
    pub const ANY: BaseGameVersion = BaseGameVersion {
        version: Some(SemVersion::ANY),
        never_compatible: false,
    };

    /// No version.
    pub const EMPTY: BaseGameVersion = BaseGameVersion {
        version: None,
        never_compatible: false,
    };

    /// Never compatible.
    pub const INCOMPATIBLE: BaseGameVersion = BaseGameVersion {
        version: None,
        never_compatible: true,
    };

    /// A world or template at `version`.
    pub const fn new(version: SemVersion) -> Self {
        Self {
            version: Some(version),
            never_compatible: false,
        }
    }

    /// The version is `*`.
    pub fn is_any(&self) -> bool {
        self.version.as_ref().is_some_and(SemVersion::is_any)
    }

    /// A pack versioned `version` (a base-game slice, a chemistry pack) applies to a world of this
    /// base-game version: always for `*` or no version, never when
    /// never compatible, otherwise when `version` is not above this one.
    pub fn includes(&self, version: &SemVersion) -> bool {
        if self.never_compatible {
            return false;
        }
        match &self.version {
            None => true,
            Some(world) => world.is_any() || version <= world,
        }
    }
}

/// The engine version the crate tracks, bumped together with the protocol version. It is the
/// game's current format version for items, blocks and manifests, and so the version at which
/// every open-ended slice sequence of a family closes.
pub const ENGINE_VERSION: FormatVersion = FormatVersion::new(1, 26, 60);

#[cfg(test)]
mod command_version_map {
    use super::{COMMAND_VERSION_MAP, CURRENT_COMMAND_VERSION, EARLIEST_COMMAND_VERSION};

    #[test]
    fn the_raw_map_is_vanillas_39_entries_in_order() {
        assert_eq!(COMMAND_VERSION_MAP.len(), 39);
        // Vanilla asserts that the last value is the current version.
        assert_eq!(
            COMMAND_VERSION_MAP[38],
            ((1, 26, 10), CURRENT_COMMAND_VERSION)
        );
        assert_eq!(
            COMMAND_VERSION_MAP[0],
            ((1, 8, 0), EARLIEST_COMMAND_VERSION)
        );
        // Keys never decrease and values rise by one: 8 to 46.
        for (index, pair) in COMMAND_VERSION_MAP.windows(2).enumerate() {
            assert!(pair[0].0 <= pair[1].0, "entry {index}");
            assert_eq!(pair[0].1 + 1, pair[1].1, "entry {index}");
        }
    }

    #[test]
    fn six_values_share_their_key_with_a_later_entry() {
        let shadowed: Vec<u8> = COMMAND_VERSION_MAP
            .windows(2)
            .filter(|pair| pair[0].0 == pair[1].0)
            .map(|pair| pair[0].1)
            .collect();
        assert_eq!(shadowed, [27, 28, 29, 30, 31, 40]);
    }
}
