//! Building a finding in one expression: a code, then where and how it is told.
//!
//! ```
//! use bedrock_addon_core::diagnostic::{
//!     Class, CodeDef, Diagnostic, DiagnosticCode, JsonPath, Kind, Phase, Severity,
//! };
//!
//! static DENSITY_OUT_OF_UNIT: CodeDef<1> = CodeDef::new(
//!     "fog/density_out_of_unit",
//!     Kind::InvalidValue,
//!     Class::Error,
//!     "fog density {} is outside [0, 1]",
//! );
//!
//! let at = JsonPath::of_keys(["minecraft:fog_settings", "distance"]);
//! // A specific code: its class grades it.
//! let finding = Diagnostic::of(DENSITY_OUT_OF_UNIT.with(["1.5"])).at(at.clone());
//! assert_eq!(finding.severity, Severity::Error);
//! // A legacy code says its severity.
//! let legacy = Diagnostic::warn(DiagnosticCode::InvalidValue { text: None })
//!     .at(at)
//!     .phase(Phase::Validate);
//! assert_eq!(legacy.severity, Severity::Warning);
//! ```

use super::{
    Class, Code, Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, Location, Phase, Severity,
    Span,
};
use crate::document::Origin;

/// The phase a built finding is told in until [`Diagnostic::phase`] says another: the phase
/// most findings are raised in. A site outside parsing that leaves it is wrong where a phase is
/// branched on (see [`Diagnostic::of`]).
const DEFAULT_PHASE: Phase = Phase::Parse;

impl Diagnostic {
    /// A finding told in [`Phase::Parse`]: every site that raises it outside parsing must chain
    /// [`phase`](Self::phase), since three places branch on the phase
    /// (`core/src/versioning/pipeline.rs:627`, `families/entity/src/load.rs:289`,
    /// `families/entity/src/read.rs:384`).
    ///
    /// The finding is of a specific code, graded by the code's [`Class`]: an error, a kept
    /// error (an error that rejects nothing), a warning, or information for a lint. It is at the
    /// document root of no file until the builder says otherwise.
    pub fn of(code: impl Into<Code>) -> Self {
        let code = code.into();
        let class = code.class();
        let mut finding = Self::told(code.into(), class.default_severity());
        finding.kept = class == Class::KeptError;
        finding
    }

    /// An error-level finding told in [`Phase::Parse`]: every site that raises it outside
    /// parsing must chain [`phase`](Self::phase) (see [`of`](Self::of) for the three places
    /// that branch on the phase).
    ///
    /// For a legacy code, which has no class. The finding is at the document root of no file
    /// until the builder says otherwise.
    ///
    /// A specific code is not taken here, so that its class grades it: it goes through
    /// [`of`](Self::of), and [`severity`](Self::severity) when the raise site overrides.
    ///
    /// ```compile_fail
    /// use bedrock_addon_core::diagnostic::{Class, CodeDef, Diagnostic, Kind};
    /// static LINT: CodeDef<0> = CodeDef::new("fog/lint", Kind::Lint, Class::Lint, "lint");
    /// // A specific code bypassing its class: refused.
    /// let _ = Diagnostic::error(&LINT);
    /// ```
    pub fn error(code: DiagnosticCode) -> Self {
        Self::told(code, Severity::Error)
    }

    /// A warning told in [`Phase::Parse`]: every site that raises it outside parsing must
    /// chain [`phase`](Self::phase). For a legacy code; see [`error`](Self::error).
    pub fn warn(code: DiagnosticCode) -> Self {
        Self::told(code, Severity::Warning)
    }

    /// An information finding told in [`Phase::Parse`]: every site that raises it outside
    /// parsing must chain [`phase`](Self::phase). For a legacy code; see
    /// [`error`](Self::error).
    pub fn info(code: DiagnosticCode) -> Self {
        Self::told(code, Severity::Info)
    }

    fn told(code: DiagnosticCode, severity: Severity) -> Self {
        Self::new(code, severity, DEFAULT_PHASE, Location::default())
    }

    /// The finding at `path` inside its file.
    #[must_use]
    pub fn at(mut self, path: JsonPath) -> Self {
        self.location.json_path = path;
        self
    }

    /// The finding at the byte range `span` of its file.
    #[must_use]
    pub fn span(mut self, span: Span) -> Self {
        self.location.span = Some(span);
        self
    }

    /// The finding told in `phase`.
    #[must_use]
    pub fn phase(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }

    /// The finding in the pack and file of `origin`.
    #[must_use]
    pub fn in_file(mut self, origin: &Origin) -> Self {
        self.location.pack = Some(origin.pack.clone());
        self.location.path = Some(origin.path.clone());
        self
    }

    /// The finding with `detail` attached.
    #[must_use]
    pub fn detail(mut self, detail: DiagnosticDetail) -> Self {
        self.detail = Some(Box::new(detail));
        self
    }

    /// The finding at `severity` whatever its code's class says: the raise site's explicit
    /// override. A finding below error level is never [`kept`](Self::kept); one at error level
    /// keeps what it was.
    #[must_use]
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self.kept &= severity == Severity::Error;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{CodeDef, Kind};
    use crate::pack_identity::{PackPath, PackRef, PackSide};

    static DENSITY: CodeDef<1> = CodeDef::new(
        "fog/density",
        Kind::InvalidValue,
        Class::Error,
        "density {}",
    );
    static KEPT: CodeDef<0> =
        CodeDef::new("fog/kept", Kind::InvalidValue, Class::KeptError, "kept");
    static UNKNOWN: CodeDef<0> = CodeDef::new(
        "fog/unknown",
        Kind::UnknownMember,
        Class::Unknown,
        "unknown",
    );
    static LINT: CodeDef<0> = CodeDef::new("fog/lint", Kind::Lint, Class::Lint, "lint");

    fn friction() -> JsonPath {
        JsonPath::of_keys(["minecraft:block", "components", "minecraft:friction"])
    }

    fn fog_origin() -> Origin {
        Origin::file(
            PackRef::app_data(),
            PackSide::Resource,
            PackPath::from_static("fogs/a.json"),
        )
    }

    /// The builder and the five-argument constructor build the same finding.
    #[test]
    fn a_built_legacy_finding_equals_the_constructed_one() {
        let span = Span { offset: 3, len: 4 };
        let built = Diagnostic::warn(DiagnosticCode::InvalidValue { text: None })
            .at(friction())
            .span(span)
            .phase(Phase::Validate);
        let made = Diagnostic::new(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Warning,
            Phase::Validate,
            Location::new(None, None, friction(), Some(span)),
        );
        assert_eq!(built, made);
    }

    #[test]
    fn a_built_finding_is_told_at_parse_at_the_root_of_no_file() {
        let d = Diagnostic::error(DiagnosticCode::MissingRequiredField { text: None });
        assert_eq!(d.phase, Phase::Parse);
        assert_eq!(d.location, Location::default());
        assert_eq!(d.severity, Severity::Error);
        assert!(!d.kept);
        assert_eq!(
            Diagnostic::info(DiagnosticCode::NoEffect { text: None }).severity,
            Severity::Info
        );
    }

    #[test]
    fn a_specific_finding_is_graded_by_its_class() {
        let error = Diagnostic::of(DENSITY.with(["2"]));
        assert_eq!(error.severity, Severity::Error);
        assert!(error.rejects());
        let kept = Diagnostic::of(&KEPT);
        assert_eq!(kept.severity, Severity::Error);
        assert!(kept.kept && !kept.rejects());
        assert_eq!(Diagnostic::of(&UNKNOWN).severity, Severity::Warning);
        assert_eq!(Diagnostic::of(&LINT).severity, Severity::Info);
    }

    #[test]
    fn severity_overrides_the_class() {
        let d = Diagnostic::of(&LINT).severity(Severity::Error);
        assert_eq!(d.severity, Severity::Error);
        assert!(d.rejects());
        let below = Diagnostic::of(&KEPT).severity(Severity::Warning);
        assert_eq!(below.severity, Severity::Warning);
        assert!(!below.kept);
        assert!(Diagnostic::of(&KEPT).severity(Severity::Error).kept);
    }

    #[test]
    fn in_file_names_the_pack_and_file_of_the_origin() {
        let origin = fog_origin();
        let d = Diagnostic::of(&LINT).at(friction()).in_file(&origin);
        assert_eq!(d.location.pack, Some(origin.pack.clone()));
        assert_eq!(d.location.path, Some(origin.path.clone()));
        assert_eq!(d.location.json_path, friction());
    }

    #[test]
    fn detail_is_attached() {
        let d = Diagnostic::of(&LINT).detail(DiagnosticDetail::Note("why"));
        assert_eq!(d.detail.as_deref(), Some(&DiagnosticDetail::Note("why")));
    }
}
