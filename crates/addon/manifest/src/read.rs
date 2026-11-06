//! What both manifest readers share: the finding sink, the primitives each member is read
//! with, and small `Value` accessors.
//!
//! The game has two manifest parsers, and the primitives follow the one the sink's
//! [`Profile`] names: the legacy parser (formats 0, 1, 2) takes `null` for an absent member,
//! keeps unknown members and has its own codes; the comprehensive document (format 3) is
//! strict. Every difference of a primitive between the two is a branch on the profile; what
//! differs beyond a primitive is a section reader of its own in `legacy` or `comprehensive`.
//!
//! Manifests are read as an untyped [`Value`] (text through facet-json's jsonc entry point) and
//! walked by hand, the way vanilla's legacy parser walks its jsoncpp tree: each member has its
//! own coercion and its own parse-error type, which a typed deserialisation cannot report.
//! Findings on a `Value` have a JSON path and no byte offset.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::{ENGINE_VERSION, SemVersion};
use uuid::Uuid;

use super::{DependencyPlace, ids};

use bedrock_addon_core::pack_identity::PackScope;

/// The lowest `min_engine_version` (and world-template `base_game_version`) a format-2 or
/// format-3 pack may declare.
pub(super) const LOWEST_ENGINE: SemVersion = SemVersion::new(1, 13, 0);

/// Which of the game's two manifest parsers a read follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Profile {
    /// Vanilla's hand-written legacy parser (formats 0, 1 and 2) over a jsoncpp tree: a member
    /// written `null` is absent, unknown members are kept for write-back, and the codes are
    /// the pack report's (`WrongTypeProperty`, `MissingProperty`, `EmptyProperty`).
    Legacy,
    /// The comprehensive document (format 3), read with a strict mask: `null` is a value of
    /// the wrong type, an unknown member is `UnrecognizedField`, and the codes are the
    /// schema's (`InvalidType`, `MissingRequiredField`).
    Comprehensive,
}

/// Collects the findings of one manifest read, at JSON paths of the file at `path`, and reads
/// members by the rules of its [`Profile`].
pub(super) struct Sink<'p> {
    path: &'p PackPath,
    profile: Profile,
    pub(super) diagnostics: Diagnostics,
}

impl<'p> Sink<'p> {
    /// A sink for the file at `path`; it reads by the legacy rules until the format picks.
    pub(super) fn new(path: &'p PackPath) -> Self {
        Self {
            path,
            profile: Profile::Legacy,
            diagnostics: Diagnostics::default(),
        }
    }

    /// Reads by the rules of `profile` from now on.
    pub(super) fn set_profile(&mut self, profile: Profile) {
        self.profile = profile;
    }

    /// The rules the sink reads by.
    pub(super) fn profile(&self) -> Profile {
        self.profile
    }

    /// Records `code` at `at`.
    pub(super) fn push(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        phase: Phase,
        at: &JsonPath,
    ) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            phase,
            Location::new(None, Some(self.path.clone()), at.clone(), None),
        ));
    }

    /// Records an error at the byte range `span` of the file (the root's JSON path): a finding
    /// on the text itself, before there is a value to walk.
    pub(super) fn error_in_text(&mut self, code: DiagnosticCode, span: Span) {
        self.diagnostics.push(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, Some(self.path.clone()), JsonPath::new(), Some(span)),
        ));
    }

    pub(super) fn error(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.push(code, Severity::Error, Phase::Parse, at);
    }

    /// A warning: one of the game's pack-report warnings, the player-facing list a pack's
    /// problems go to instead of the content log (shown through the `packupgradewarning.*`
    /// keys: `.malformed_uuid`, `.multiple_modules`, `.manifest_upgraded`,
    /// `.malformed_version`, `.invalidpacktype`, `.duplicate_uuid`). A pack-report entry has
    /// no content-log level, so it is taken as the level-2 equivalent.
    pub(super) fn warning(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.push(code, Severity::Warning, Phase::Parse, at);
    }

    /// An information from the crate: the game reads on in silence.
    pub(super) fn note(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.push(code, Severity::Info, Phase::Parse, at);
    }

    /// An error, or a warning when the manifest is being upgraded from format 0 (vanilla's
    /// "Upgraded" twin codes are one code here, with the severity picked by the read; the
    /// twin is a pack-report warning, as [`Self::warning`] says).
    pub(super) fn error_or_upgraded(
        &mut self,
        code: DiagnosticCode,
        upgraded: bool,
        at: &JsonPath,
    ) {
        let severity = if upgraded {
            Severity::Warning
        } else {
            Severity::Error
        };
        self.push(code, severity, Phase::Parse, at);
    }

    /// [`version_array`], telling what it converts (the crate's own report; vanilla is
    /// silent): an
    /// element that is `null` or a bool is read as 0 or 1 with an `InvalidType` warning at
    /// the element, a real with a fraction is truncated with an `InvalidValue` warning. The
    /// version is the one vanilla reads; the written manifest holds it, not the file's
    /// element.
    pub(super) fn version_array(&mut self, value: &Value, at: &JsonPath) -> Option<SemVersion> {
        let version = version_array(value)?;
        for (index, element) in value.as_array()?.as_slice().iter().enumerate() {
            let fraction = element
                .as_number()
                .is_some_and(|number| number.to_f64_lossy().fract() != 0.0);
            // Vanilla reads the element in silence; the crate reports an information.
            if element.is_null() || element.as_bool().is_some() {
                self.note(DiagnosticCode::InvalidType { text: None }, &at.index(index));
            } else if fraction {
                self.note(
                    DiagnosticCode::InvalidValue { text: None },
                    &at.index(index),
                );
            }
        }
        Some(version)
    }

    pub(super) fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// The number of findings so far, to tell whether a step added an error.
    pub(super) fn errors(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }
}

/// The primitives: one member each, by the rules of the sink's profile.
impl Sink<'_> {
    /// The code of a member of the wrong JSON type.
    pub(super) fn wrong_type(&self) -> DiagnosticCode {
        match self.profile {
            Profile::Legacy => DiagnosticCode::WrongTypeProperty { text: None },
            Profile::Comprehensive => DiagnosticCode::InvalidType { text: None },
        }
    }

    /// The code of a required member that is absent.
    fn missing(&self) -> DiagnosticCode {
        match self.profile {
            Profile::Legacy => DiagnosticCode::MissingProperty { text: None },
            Profile::Comprehensive => DiagnosticCode::MissingRequiredField { text: None },
        }
    }

    /// The member `key` of `object` when it is there: the legacy parser takes one written
    /// `null` for an absent one, the comprehensive document reads it as a value.
    pub(super) fn present<'v>(&self, object: &'v Value, key: &str) -> Option<&'v Value> {
        let value = member(object, key)?;
        match self.profile {
            Profile::Legacy if value.is_null() => None,
            _ => Some(value),
        }
    }

    /// The member `key` of `object` at `below(at, key)`, or `None`; an absent one is an error
    /// when `required`.
    fn required<'v>(
        &mut self,
        object: &'v Value,
        key: &str,
        at: &JsonPath,
        required: bool,
    ) -> Option<&'v Value> {
        let value = self.present(object, key);
        if value.is_none() && required {
            let code = self.missing();
            self.error(code, at);
        }
        value
    }

    /// The object member `key` of `container`; absent (an error when `required`) or of
    /// another type (an error) is `None`.
    pub(super) fn object<'v>(
        &mut self,
        container: &'v Value,
        key: &str,
        at: &JsonPath,
        required: bool,
    ) -> Option<&'v Value> {
        let at = below(at, key);
        let value = self.required(container, key, &at, required)?;
        if value.as_object().is_none() {
            let code = self.wrong_type();
            self.error(code, &at);
            return None;
        }
        Some(value)
    }

    /// The array member `key` of `container`; absent (an error when `required`) or of another
    /// type (an error) is `None`.
    pub(super) fn array<'v>(
        &mut self,
        container: &'v Value,
        key: &str,
        at: &JsonPath,
        required: bool,
    ) -> Option<&'v [Value]> {
        let at = below(at, key);
        let value = self.required(container, key, &at, required)?;
        let Some(array) = value.as_array() else {
            let code = self.wrong_type();
            self.error(code, &at);
            return None;
        };
        Some(array.as_slice())
    }

    /// A string member; another type is an error, so is an absent one when `required`. The
    /// legacy parser also refuses an empty required string (`EmptyProperty`, an error, the
    /// empty text still read); the comprehensive document accepts it.
    pub(super) fn string(
        &mut self,
        object: &Value,
        key: &str,
        at: &JsonPath,
        required: bool,
    ) -> Option<String> {
        let at = below(at, key);
        let value = self.required(object, key, &at, required)?;
        let Some(read) = text(value) else {
            let code = self.wrong_type();
            self.error(code, &at);
            return None;
        };
        if read.is_empty() && required && self.profile == Profile::Legacy {
            // An empty required string is reported as `empty_property:%s;` (skipped for an
            // optional member), and the pack is rejected, so it is an error.
            self.error(DiagnosticCode::EmptyProperty, &at);
        }
        Some(read.to_owned())
    }

    /// A boolean member; another type is an error, so is an absent one when `required`.
    pub(super) fn boolean(
        &mut self,
        object: &Value,
        key: &str,
        at: &JsonPath,
        required: bool,
    ) -> Option<bool> {
        let at = below(at, key);
        let value = self.required(object, key, &at, required)?;
        let read = value.as_bool();
        if read.is_none() {
            let code = self.wrong_type();
            self.error(code, &at);
        }
        read
    }

    /// A required UUID member. A malformed or nil text is replaced by its hash with
    /// `MalformedPropertyUuid` (an error; a warning when `upgraded`, format 0). The legacy
    /// parser reads the empty text as `EmptyProperty` and gives no UUID; to the comprehensive
    /// document it is malformed.
    pub(super) fn uuid(
        &mut self,
        object: &Value,
        key: &str,
        at: &JsonPath,
        upgraded: bool,
    ) -> Option<Uuid> {
        // Vanilla gives a legacy pack with no UUID a random one; the crate reports
        // `missing_property` and rejects the manifest instead.
        let raw = self.string(object, key, at, true)?;
        let at = below(at, key);
        if raw.is_empty() && self.profile == Profile::Legacy {
            // `string` reported the empty text, as for any required string.
            return None;
        }
        if let Some(uuid) = ids::parse(&raw) {
            return Some(uuid);
        }
        self.error_or_upgraded(DiagnosticCode::MalformedPropertyUuid, upgraded, &at);
        Some(ids::salvage(&raw))
    }

    /// A required version member. The legacy parser reads the strict string form first, then
    /// the array form, and a format-0 (`upgraded`) manifest substitutes what it cannot read
    /// (absent: 0.0.0, a bad string: 1.0.0, another value: 0.0.0) with a warning; its errors
    /// name the member as a pointer. The comprehensive document reads a relaxed string only
    /// (`"2.0"`), and an array is the wrong type.
    pub(super) fn version(
        &mut self,
        object: &Value,
        key: &str,
        at: &JsonPath,
        upgraded: bool,
    ) -> Option<SemVersion> {
        if self.profile == Profile::Comprehensive {
            let raw = self.string(object, key, at, true)?;
            let parsed = relaxed_version_text(&raw);
            if parsed.is_none() {
                self.error(
                    DiagnosticCode::MalformedPropertyVersion { text: None },
                    &below(at, key),
                );
            }
            return parsed;
        }
        let at = below(at, key);
        let Some(value) = self.present(object, key) else {
            if upgraded {
                self.warning(DiagnosticCode::MissingProperty { text: None }, &at);
                return Some(SemVersion::new(0, 0, 0));
            }
            self.error(DiagnosticCode::MissingProperty { text: None }, &at);
            return None;
        };
        if let Some(raw) = text(value) {
            return match strict_version_text(raw) {
                Some(version) => Some(version),
                None if upgraded => {
                    self.warning(DiagnosticCode::MalformedPropertyVersion { text: None }, &at);
                    Some(SemVersion::new(1, 0, 0))
                }
                None => {
                    let text = Some(VanillaText::ManifestNotSemVer { path: pointer(&at) });
                    self.error(DiagnosticCode::MalformedPropertyVersion { text }, &at);
                    None
                }
            };
        }
        if let Some(version) = self.version_array(value, &at) {
            return Some(version);
        }
        // An array of the wrong length or with an unconvertible element, or another type.
        let text = (!upgraded).then(|| VanillaText::ManifestWrongType { path: pointer(&at) });
        self.error_or_upgraded(DiagnosticCode::WrongTypeProperty { text }, upgraded, &at);
        upgraded.then(|| SemVersion::new(0, 0, 0))
    }

    /// The members of `object` outside `known`. The legacy parser reads past them, and they
    /// are kept, in document order, for write-back; the comprehensive document refuses each
    /// with `UnrecognizedField` (an error, its mask is strict), and none is kept.
    pub(super) fn unknown(
        &mut self,
        object: &Value,
        at: &JsonPath,
        known: &[&str],
    ) -> UnknownMembers {
        let mut extra = UnknownMembers::default();
        let Some(members) = object.as_object() else {
            return extra;
        };
        for (key, value) in members {
            if known.contains(&key.as_str()) {
                continue;
            }
            match self.profile {
                Profile::Legacy => {
                    extra.insert(key.as_str().to_owned(), value.clone());
                }
                Profile::Comprehensive => self.error(
                    DiagnosticCode::UnrecognizedField { text: None },
                    &below(at, key.as_str()),
                ),
            }
        }
        extra
    }

    /// `false`, with an error at `at`, when `version` is above the engine's or below
    /// [`LOWEST_ENGINE`]: the bounds of a format-2 or format-3 `min_engine_version` and of a
    /// world template's `base_game_version`.
    pub(super) fn engine_bounds(&mut self, version: &SemVersion, at: &JsonPath) -> bool {
        let engine = SemVersion::from(ENGINE_VERSION);
        if *version > engine {
            self.error(
                DiagnosticCode::VersionTooHigh {
                    version: version.to_string(),
                    limit: engine.to_string(),
                },
                at,
            );
            return false;
        }
        if *version < LOWEST_ENGINE {
            self.error(
                DiagnosticCode::VersionTooLow {
                    version: version.to_string(),
                    limit: LOWEST_ENGINE.to_string(),
                },
                at,
            );
            return false;
        }
        true
    }
}

/// The element at `at` as the legacy reader's messages name it: a JSON pointer
/// (`/dependencies/0/version`).
pub(super) fn pointer(at: &JsonPath) -> String {
    format!("/{at}")
}

/// The place of the kept `dependencies[]` element `element` at `at`, with what its text says
/// that the value does not hold: whether it has an `optional` member (whatever its value), and,
/// for a `native` module, whether its `version` is a string of fewer than three numbers that
/// is no selector.
pub(super) fn dependency_place(element: &Value, at: JsonPath, native: bool) -> DependencyPlace {
    let relaxed = native
        && member(element, "version")
            .and_then(text)
            .is_some_and(|version| {
                !matches!(version, "alpha" | "beta" | "internal") && number_count(version) != 3
            });
    DependencyPlace {
        at,
        has_optional: member(element, "optional").is_some(),
        relaxed,
    }
}

/// How many numbers a version string writes before its pre-release tag or build metadata.
fn number_count(version: &str) -> usize {
    version
        .split(['-', '+'])
        .next()
        .unwrap_or_default()
        .split('.')
        .count()
}

/// `pack_scope` from its text: a vanilla spelling (`global`, `world`, `any`) is its scope. Other
/// text is a `code` warning, reads as no scope (the default, any) and is kept in `extra`, so it
/// is written back as it was.
pub(super) fn pack_scope(
    sink: &mut Sink<'_>,
    text: Option<String>,
    code: DiagnosticCode,
    extra: &mut UnknownMembers,
) -> Option<PackScope> {
    let text = text?;
    let scope = PackScope::from_json_name(&text);
    if scope.is_none() {
        // Vanilla reads the text in silence; the crate reports an information.
        sink.note(code, &path(&["header", "pack_scope"]));
        extra.insert("pack_scope".to_owned(), Value::from(text.as_str()));
    }
    scope
}

/// The path of the root members and their members `keys`, from the outermost down.
pub(super) fn path(keys: &[&str]) -> JsonPath {
    JsonPath::of_keys(keys)
}

/// The path of the member `key` below `at`, in the segments the manifest readers have always
/// given a member: a key that reads as an unsigned number (`"0"`, `"+5"`) is an array index
/// of the path, and the empty key adds no segment. Every other key is one segment as written.
// The two edges are the readers' paths as they were when built from `/`-joined text, pinned in
// `tests/characterize_manifest.rs`; they show only for a member of the file so named.
pub(super) fn below(at: &JsonPath, key: &str) -> JsonPath {
    if key.is_empty() {
        return at.clone();
    }
    match key.parse::<usize>() {
        Ok(index) => at.index(index),
        Err(_) => at.child(key),
    }
}

/// The member `key` of `value` when it is an object.
pub(super) fn member<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value.as_object()?.get(key)
}

/// The text of a string value.
pub(super) fn text(value: &Value) -> Option<&str> {
    value.as_string().map(facet_value::VString::as_str)
}

/// Vanilla's array form of a version, as jsoncpp reads it: exactly three elements, each
/// convertible to an unsigned integer (`null` is 0, `false` / `true` are 0 / 1, a non-negative
/// real is truncated), none above 65 535. `None` otherwise.
pub(super) fn version_array(value: &Value) -> Option<SemVersion> {
    // The array form `[a,b,c]` truncates reals and rejects any element above 0xFFFF; jsoncpp's
    // unsigned conversion admits null and booleans. The shared reader lives in the version
    // module.
    SemVersion::from_json_array(value.as_array()?.as_slice())
}

/// A version string of the strict form `major.minor.patch[-pre][+build]` (the any-version `*`,
/// the `beta` sentinel and partial versions are refused).
pub(super) fn strict_version_text(text: &str) -> Option<SemVersion> {
    let core = text.split(['-', '+']).next().unwrap_or_default();
    if core.split('.').count() != 3 {
        return None;
    }
    SemVersion::parse(text).filter(|version| !version.is_any())
}

/// A relaxed version string (`"2.0"`, `"1"`): partial versions are filled with zeros; the
/// any-version `*` and the `beta` sentinel are refused.
pub(super) fn relaxed_version_text(text: &str) -> Option<SemVersion> {
    if text == "beta" {
        return None;
    }
    SemVersion::parse(text).filter(|version| !version.is_any())
}
