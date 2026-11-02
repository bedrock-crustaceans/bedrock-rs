//! What both manifest readers share: the finding sink and small `Value` accessors.
//!
//! Manifests are read as an untyped [`Value`] (text through facet-json's jsonc entry point) and
//! walked by hand, the way vanilla's legacy parser walks its jsoncpp tree: each member has its
//! own coercion and its own parse-error type, which a typed deserialisation cannot report.
//! Findings on a `Value` have a JSON path and no byte offset.

use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
};
use crate::document::Extra;
use crate::json::Value;
use crate::pack_identity::PackPath;
use crate::version::SemVersion;

use crate::pack_identity::PackScope;

/// Collects the findings of one manifest read, at JSON paths of the file at `path`.
pub(super) struct Sink<'p> {
    path: &'p PackPath,
    pub(super) diagnostics: Diagnostics,
}

impl<'p> Sink<'p> {
    pub(super) fn new(path: &'p PackPath) -> Self {
        Self {
            path,
            diagnostics: Diagnostics::default(),
        }
    }

    /// Records `code` at `at` (a `/`-separated member path, `""` for the root).
    pub(super) fn push(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        phase: Phase,
        at: &str,
    ) {
        self.diagnostics.push(Diagnostic::new(
            code,
            severity,
            phase,
            Location::new(None, Some(self.path.clone()), json_path(at), None),
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

    pub(super) fn error(&mut self, code: DiagnosticCode, at: &str) {
        self.push(code, Severity::Error, Phase::Parse, at);
    }

    /// A warning: one of the game's pack-report warnings, the player-facing list a pack's
    /// problems go to instead of the content log (shown through the `packupgradewarning.*`
    /// keys: `.malformed_uuid`, `.multiple_modules`, `.manifest_upgraded`,
    /// `.malformed_version`, `.invalidpacktype`, `.duplicate_uuid`). A pack-report entry has
    /// no content-log level, so it is taken as the level-2 equivalent.
    pub(super) fn warning(&mut self, code: DiagnosticCode, at: &str) {
        self.push(code, Severity::Warning, Phase::Parse, at);
    }

    /// An information from the crate: the game reads on in silence.
    pub(super) fn note(&mut self, code: DiagnosticCode, at: &str) {
        self.push(code, Severity::Info, Phase::Parse, at);
    }

    /// An error, or a warning when the manifest is being upgraded from format 0 (vanilla's
    /// "Upgraded" twin codes are one code here, with the severity picked by the read; the
    /// twin is a pack-report warning, as [`Self::warning`] says).
    pub(super) fn error_or_upgraded(&mut self, code: DiagnosticCode, upgraded: bool, at: &str) {
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
    pub(super) fn version_array(&mut self, value: &Value, at: &str) -> Option<SemVersion> {
        let version = version_array(value)?;
        for (index, element) in value.as_array()?.as_slice().iter().enumerate() {
            let fraction = element
                .as_number()
                .is_some_and(|number| number.to_f64_lossy().fract() != 0.0);
            // Vanilla reads the element in silence; the crate reports an information.
            if element.is_null() || element.as_bool().is_some() {
                self.note(DiagnosticCode::InvalidType { text: None }, &item(at, index));
            } else if fraction {
                self.note(
                    DiagnosticCode::InvalidValue { text: None },
                    &item(at, index),
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

/// `pack_scope` from its text: a vanilla spelling (`global`, `world`, `any`) is its scope. Other
/// text is a `code` warning, reads as no scope (the default, any) and is kept in `extra`, so it
/// is written back as it was.
pub(super) fn pack_scope(
    sink: &mut Sink<'_>,
    text: Option<String>,
    code: DiagnosticCode,
    extra: &mut Extra,
) -> Option<PackScope> {
    let text = text?;
    let scope = PackScope::from_json_name(&text);
    if scope.is_none() {
        // Vanilla reads the text in silence; the crate reports an information.
        sink.note(code, "header/pack_scope");
        extra.insert("pack_scope".to_owned(), Value::from(text.as_str()));
    }
    scope
}

/// `a/b/3/c` as a [`JsonPath`]; numeric segments are array indices. The text is the reader's
/// own ([`child`] builds it): a `/` or `~` inside a key is written `~1` or `~0` there, so a
/// key that holds one is still one segment.
pub(super) fn json_path(at: &str) -> JsonPath {
    let mut path = JsonPath::new();
    for segment in at.split('/').filter(|s| !s.is_empty()) {
        match segment.parse::<usize>() {
            Ok(index) => path.push_index(index),
            Err(_) => path.push(&segment.replace("~1", "/").replace("~0", "~")),
        }
    }
    path
}

/// `parent/key`, the key written so that [`json_path`] reads it back as one segment.
pub(super) fn child(parent: &str, key: &str) -> String {
    let key = key.replace('~', "~0").replace('/', "~1");
    if parent.is_empty() {
        key
    } else {
        format!("{parent}/{key}")
    }
}

/// `parent/index`.
pub(super) fn item(parent: &str, index: usize) -> String {
    child(parent, &index.to_string())
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
    SemVersion::parse(text).filter(|version| !version.any)
}

/// A relaxed version string (`"2.0"`, `"1"`): partial versions are filled with zeros; the
/// any-version `*` and the `beta` sentinel are refused.
pub(super) fn relaxed_version_text(text: &str) -> Option<SemVersion> {
    if text == "beta" {
        return None;
    }
    SemVersion::parse(text).filter(|version| !version.any)
}
