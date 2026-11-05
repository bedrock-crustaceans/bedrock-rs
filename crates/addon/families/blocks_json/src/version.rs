//! The effective version of a `blocks.json` file: what the
//! client makes of `format_version`, which is not what the member says when it is missing,
//! unreadable, partial or above the client's ceiling.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, known};

/// The version the client reads a `blocks.json` file as when its `format_version` is missing,
/// unreadable or above [`BLOCKS_JSON_MAX_FORMAT_VERSION`].
pub const BLOCKS_JSON_DEFAULT_VERSION: FormatVersion = FormatVersion::new(1, 1, 0);

/// The newest `format_version` the client reads a `blocks.json` file as. A file above it is read
/// as [`BLOCKS_JSON_DEFAULT_VERSION`], not as this version.
pub const BLOCKS_JSON_MAX_FORMAT_VERSION: FormatVersion = FormatVersion::new(1, 21, 40);

/// What the client's semantic-version parser (wildcards refused) made of a `format_version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Match {
    /// A whole version (`"1.21.40"`, `[1, 21, 40]`, `"beta"`).
    Full(Effective),
    /// A string with the major only, or major and minor: the missing parts are 0.
    Partial(Effective),
    /// Not a version; `zeroed` for `"*"`, which leaves 0.0.0 behind.
    None {
        /// The output was zeroed.
        zeroed: bool,
    },
}

/// A version as the gates compare it: the three numbers and whether a pre-release tag puts it
/// below its release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Effective {
    /// The three numbers.
    pub(super) version: FormatVersion,
    /// The pre-release tag, without `-`; empty for a release.
    pub(super) pre_release: String,
}

impl Effective {
    /// A release.
    pub(super) fn release(version: FormatVersion) -> Self {
        Self {
            version,
            pre_release: String::new(),
        }
    }

    /// `self < gate` in semantic-version order: a pre-release is below its release.
    pub(super) fn is_below(&self, gate: FormatVersion) -> bool {
        self.version < gate || (self.version == gate && !self.pre_release.is_empty())
    }

    /// `gate < self`.
    pub(super) fn is_above(&self, gate: FormatVersion) -> bool {
        self.version > gate
    }
}

/// The version as vanilla prints it.
impl std::fmt::Display for Effective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pre_release.is_empty() {
            write!(f, "{}", self.version)
        } else {
            write!(f, "{}-{}", self.version, self.pre_release)
        }
    }
}

/// What the client does with a file's `format_version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Outcome {
    /// The version the gates compare against.
    pub(super) effective: Effective,
    /// What the client logs, if anything.
    pub(super) log: Option<Log>,
}

/// The client's warning about a `format_version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Log {
    /// The member is absent.
    NotGiven,
    /// The member is not a version. `partial` holds the text of a partial version, which
    /// the client keeps although it says it uses 1.1.0.
    Invalid {
        /// The partial version as written, when the string was one.
        partial: Option<String>,
    },
    /// The version is above the ceiling; the text of the version as vanilla prints it.
    AboveCeiling(String),
}

/// The effective version of a file whose root has `member` as its `format_version`. Only a whole
/// version is compared with the ceiling; a partial version or no version at all logs the
/// "invalid" warning and keeps whatever version the parse left behind.
pub(super) fn effective(member: Option<&Value>) -> Outcome {
    let Some(value) = member else {
        return Outcome {
            effective: Effective::release(BLOCKS_JSON_DEFAULT_VERSION),
            log: Some(Log::NotGiven),
        };
    };
    match from_json(value) {
        Match::Full(version) if version.is_above(BLOCKS_JSON_MAX_FORMAT_VERSION) => Outcome {
            effective: Effective::release(BLOCKS_JSON_DEFAULT_VERSION),
            log: Some(Log::AboveCeiling(version.to_string())),
        },
        Match::Full(version) => Outcome {
            effective: version,
            log: None,
        },
        Match::Partial(version) => Outcome {
            effective: version,
            log: Some(Log::Invalid {
                partial: value.as_string().map(|s| s.as_str().to_owned()),
            }),
        },
        Match::None { zeroed } => Outcome {
            effective: Effective::release(if zeroed {
                known::V0_0_0
            } else {
                BLOCKS_JSON_DEFAULT_VERSION
            }),
            log: Some(Log::Invalid { partial: None }),
        },
    }
}

/// Reads a `format_version` value as the client does: an array is read as three numbers and a
/// string as a version text; anything else is not a version and leaves the output untouched.
pub(super) fn from_json(value: &Value) -> Match {
    if let Some(array) = value.as_array() {
        return from_array(array.as_slice());
    }
    match value.as_string() {
        Some(text) => from_string(text.as_str()),
        None => Match::None { zeroed: false },
    }
}

/// A version array: exactly three elements, each convertible to an unsigned integer below 65536.
/// jsoncpp converts a non-negative integer, a real in range (truncated), a boolean (0 or 1) and
/// `null` (0).
fn from_array(elements: &[Value]) -> Match {
    let [major, minor, patch] = elements else {
        return Match::None { zeroed: false };
    };
    match (part(major), part(minor), part(patch)) {
        (Some(major), Some(minor), Some(patch)) => {
            Match::Full(Effective::release(FormatVersion::new(major, minor, patch)))
        }
        _ => Match::None { zeroed: false },
    }
}

/// One element of a version array as jsoncpp's unsigned-integer conversion reads it, when it fits `u16`.
fn part(value: &Value) -> Option<u16> {
    if value.is_null() {
        return Some(0);
    }
    if let Some(flag) = value.as_bool() {
        return Some(u16::from(flag));
    }
    let number = value.as_number()?.to_f64_lossy();
    if !(0.0..65536.0).contains(&number) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // in range, checked above
    Some(number as u16)
}

/// A version text: `"*"` zeroes the output and is `None`; `"beta"` is the whole version
/// 9999.9999.9999-beta; otherwise the text must match
/// `^(0|[1-9]\d{0,4})(\.(0|[1-9]\d{0,4})(\.(0|[1-9]\d{0,4})(?:\-(…))?(?:\+(…))?)?)?` as a
/// whole and each part must fit `u16`. A missing minor or patch makes it `Partial`.
pub(super) fn from_string(text: &str) -> Match {
    const NONE: Match = Match::None { zeroed: false };
    if text == "*" {
        return Match::None { zeroed: true };
    }
    if text == "beta" {
        return Match::Full(Effective {
            version: FormatVersion::new(9999, 9999, 9999),
            pre_release: "beta".to_owned(),
        });
    }
    let (numbers, tags) = match text.find(['-', '+']) {
        Some(at) => (&text[..at], &text[at..]),
        None => (text, ""),
    };
    let mut parts = [0u16; 3];
    let mut count = 0;
    for number in numbers.split('.') {
        let Some(slot) = parts.get_mut(count) else {
            return NONE;
        };
        let Some(value) = number_part(number) else {
            return NONE;
        };
        *slot = value;
        count += 1;
    }
    let version = FormatVersion::new(parts[0], parts[1], parts[2]);
    if count < 3 {
        // The tags belong to the patch group: without a patch they do not match.
        return if tags.is_empty() {
            Match::Partial(Effective::release(version))
        } else {
            NONE
        };
    }
    let (pre_release, build) = match tags.strip_prefix('-') {
        Some(rest) => match rest.split_once('+') {
            Some((pre_release, build)) => (Some(pre_release), Some(build)),
            None => (Some(rest), None),
        },
        None => (None, tags.strip_prefix('+')),
    };
    if pre_release.is_some_and(|tag| !is_tag(tag)) || build.is_some_and(|tag| !is_tag(tag)) {
        return NONE;
    }
    Match::Full(Effective {
        version,
        pre_release: pre_release.unwrap_or_default().to_owned(),
    })
}

/// `0|[1-9]\d{0,4}`, fitting `u16`.
fn number_part(text: &str) -> Option<u16> {
    let digits = text.bytes().all(|b| b.is_ascii_digit());
    let leading_zero = text.len() > 1 && text.starts_with('0');
    if text.is_empty() || text.len() > 5 || !digits || leading_zero {
        return None;
    }
    text.parse().ok()
}

/// A pre-release or build tag: dot-separated runs of ASCII letters, digits and `-`, none empty.
fn is_tag(text: &str) -> bool {
    text.split('.')
        .all(|run| !run.is_empty() && run.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
}
