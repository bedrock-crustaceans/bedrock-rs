//! `ScriptVersion`: the version of a native script module as the script engine compares it.

use std::cmp::Ordering;
use std::fmt;

use bedrock_addon_core::version::SemVersion;

/// The version of a native script module as the script engine holds it: three numbers and a
/// pre-release tag, without build metadata.
///
/// Versions order as `SemVer` 2.0 does: by the numbers, then a version with a pre-release tag
/// below the one without, then the tags identifier by identifier (split at `.`; two numeric
/// identifiers by value, a numeric one below any other, otherwise bytewise, and the tag with
/// fewer identifiers first). Two versions are equal when the numbers and the tag's bytes are.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScriptVersion {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
    /// The pre-release tag (`beta`, `rc.1`); empty for a release.
    pub pre: String,
}

impl ScriptVersion {
    /// The release `major.minor.patch`.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: String::new(),
        }
    }

    /// `major.minor.patch-pre`.
    pub fn with_pre(major: u16, minor: u16, patch: u16, pre: &str) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: pre.to_owned(),
        }
    }

    /// Reads `major.minor.patch[-pre][+build]`; the build metadata is dropped. `None` for
    /// anything else (a partial version, the any-version `*`).
    pub fn parse(text: &str) -> Option<Self> {
        let core = text.split(['-', '+']).next().unwrap_or_default();
        if core.split('.').count() != 3 {
            return None;
        }
        let version = SemVersion::parse(text).filter(|version| !version.is_any())?;
        Some(Self::from(&version))
    }

    /// Whether this version, as a candidate, serves `request` as the
    /// script engine decides it.
    ///
    /// A candidate of major 0, or one whose pre-release tag is the request's, serves only an
    /// equal request. Otherwise the candidate must not be lower and must have the request's
    /// major, and then serves it when neither has a tag; when `strict` is off, a candidate
    /// with a tag also serves a request without one.
    pub fn satisfies(&self, request: &ScriptVersion, strict: bool) -> bool {
        // A pre-release tag is compared as the script engine compares it, not as semver does.
        if self.major == 0 || (!self.pre.is_empty() && self.pre == request.pre) {
            return self == request;
        }
        if self < request || self.major != request.major {
            return false;
        }
        if self.pre == request.pre {
            return true;
        }
        !strict && !self.pre.is_empty() && request.pre.is_empty()
    }

    /// Whether the Beta APIs gate covers this version: the tag is exactly `beta`, or the
    /// major is 0.
    pub fn is_beta(&self) -> bool {
        // Every 0.x version belongs to the Beta APIs, whatever its tag.
        self.pre == "beta" || self.major == 0
    }

    /// Whether the tag is exactly `internal`.
    pub fn is_internal(&self) -> bool {
        // Only the exact tag counts: `internal.1` is an ordinary pre-release.
        self.pre == "internal"
    }
}

/// The numbers and the pre-release tag; the build metadata is dropped, as the engine does.
impl From<&SemVersion> for ScriptVersion {
    fn from(version: &SemVersion) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            pre: version.pre_release.clone(),
        }
    }
}

/// `2.9.0-beta`, as vanilla prints it.
impl fmt::Display for ScriptVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}

impl PartialOrd for ScriptVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScriptVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // Numbers first, then the tag, as the script engine orders versions.
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => cmp_pre_release(&self.pre, &other.pre),
            })
            // `01` and `1` are one number above and different bytes to `==`.
            .then_with(|| self.pre.cmp(&other.pre))
    }
}

/// Two pre-release tags, identifier by identifier; with every shared identifier equal, the
/// tag with fewer identifiers is the lower.
fn cmp_pre_release(a: &str, b: &str) -> Ordering {
    let (mut left, mut right) = (a.split('.'), b.split('.'));
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match cmp_identifier(x, y) {
                Ordering::Equal => {}
                unequal => return unequal,
            },
        }
    }
}

/// Two identifiers: two all-digit ones by value (an empty one is all-digit, and zero), an
/// all-digit one below any other, otherwise bytewise.
fn cmp_identifier(a: &str, b: &str) -> Ordering {
    let digits = |text: &str| text.bytes().all(|byte| byte.is_ascii_digit());
    match (digits(a), digits(b)) {
        (true, true) => {
            // By value without converting: vanilla's `atoi` overflows on a long identifier.
            let (a, b) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
            a.len().cmp(&b.len()).then_with(|| a.cmp(b))
        }
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => a.as_bytes().cmp(b.as_bytes()),
    }
}
