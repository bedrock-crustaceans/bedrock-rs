//! The Molang members of both file kinds: their specs, pass 1 (versions) and pass 2 (the compile
//! check with each member's failure rule).

use super::types::{EventString, ParticleEffect};
use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::diagnostic::{Diagnostics, Location};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangContext, MolangFailure, MolangFieldSpec, MolangFields,
    MolangVersionSource, MolangVisitor, OpsRule, ReaderKind, Step, not_checked, pass_one,
};
use bedrock_addon_core::version::FormatVersionInput;

/// A typed `ExpressionNode` member: read as number, bool or string; an empty text is compiled.
const fn spec(
    version: MolangVersionSource,
    on_failure: MolangFailure,
    error_severity: Severity,
) -> MolangFieldSpec {
    MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        version,
        admission: Admission::Default,
        ops: OpsRule::All,
        empty: EmptyRule::Compile,
        on_failure,
        error_severity,
        error_kept: false,
    }
}

/// Animation typed members: the pack's version; a rejection fails the schema node and drops the
/// file.
pub const ANIMATION_VALUE: MolangFieldSpec = spec(
    MolangVersionSource::Pack,
    MolangFailure::RejectDocument,
    Severity::Error,
);
/// `timeline` Molang strings: the pack's version; the game logs the parser's error at
/// level 3 and ignores the parse result: a kept error.
pub const ANIMATION_EVENT: MolangFieldSpec = spec(
    MolangVersionSource::Pack,
    MolangFailure::KeepLogged,
    Severity::Error,
);
/// Controller typed members: the effective version; a rejection drops the file.
pub const CONTROLLER_VALUE: MolangFieldSpec = spec(
    MolangVersionSource::ControllerBackCompat,
    MolangFailure::RejectDocument,
    Severity::Error,
);
/// v1.10 `transitions[]` values: a rejection is the parser's content-log error, the constant 0
/// and the file loads: a kept error.
pub const CONTROLLER_TRANSITION: MolangFieldSpec = spec(
    MolangVersionSource::ControllerBackCompat,
    MolangFailure::ReplaceWith(0.0),
    Severity::Error,
);
/// A 1.8 file's `transitions[]` values: typed nodes of the `v1_8` schema, so a rejection drops
/// the file.
pub const CONTROLLER_TRANSITION_1_8: MolangFieldSpec = CONTROLLER_VALUE;
/// `on_entry` / `on_exit` Molang strings: the effective version; the parser's content-log error
/// and the parse result ignored: a kept error.
pub const CONTROLLER_EVENT: MolangFieldSpec = spec(
    MolangVersionSource::ControllerBackCompat,
    MolangFailure::KeepLogged,
    Severity::Error,
);

/// The default `anim_time_update` (`query.anim_time + query.delta_time`, Molang version 13).
#[cfg(feature = "molang")]
pub(crate) const DEFAULT_ANIM_TIME_UPDATE: &str = "query.anim_time + query.delta_time";

// ---- the walk ----------------------------------------------------------------------------------

/// Visit the Molang of a particle effect's `pre_effect_script`.
pub fn particle(
    visitor: &mut MolangVisitor<'_>,
    spec: &MolangFieldSpec,
    effect: &mut ParticleEffect,
) {
    visitor.field("pre_effect_script", spec, &mut effect.pre_effect_script);
}

/// Visit the Molang of an event string (a plain event is left alone).
pub fn event(
    visitor: &mut MolangVisitor<'_>,
    step: Step<'_>,
    spec: &MolangFieldSpec,
    e: &mut EventString,
) {
    if let EventString::Molang(source) = e {
        visitor.enter(step);
        visitor.emit(
            spec,
            bedrock_addon_core::molang::MolangFieldMut::Source(source),
        );
        visitor.leave();
    }
}

// ---- the passes --------------------------------------------------------------------------------

/// How a file's Molang is checked: the context its versions come from and the controller gates.
#[cfg_attr(
    not(feature = "molang"),
    allow(dead_code, reason = "pass 2 reads the gates")
)]
#[derive(Clone, Debug)]
pub struct MolangRun<'a> {
    /// The load context the versions come from.
    pub load: &'a LoadContext,
    /// The file's `format_version` as written (the controller's effective version below pack
    /// 1.20.50).
    pub written: FormatVersionInput,
    /// Controllers: lower-case transition expressions before compiling (`fv < 1.17.30`).
    pub lowercase_transitions: bool,
    /// Controllers: the file was a 1.8 file, so its transitions were typed nodes.
    pub transitions_typed: bool,
}

impl MolangRun<'_> {
    fn context(&self) -> MolangContext {
        MolangContext::new(
            self.load.min_engine_version.as_ref(),
            self.load.molang_experiments,
            &self.written,
            Some(self.load.molang_version),
        )
    }
}

/// Pass 1 and pass 2 over `document`; `false` when a member whose failure drops the file was
/// rejected.
pub fn run<T: MolangFields>(
    document: &mut T,
    run: &MolangRun<'_>,
    default_anim_time_updates: usize,
    diagnostics: &mut Diagnostics,
) -> bool {
    let cx = run.context();
    let base = Location::default();
    let pass = pass_one(document, &cx, &base, diagnostics);
    if pass.failed > 0 {
        return false;
    }
    #[cfg(feature = "molang")]
    if let Some(check) = &run.load.molang {
        return check::pass_two(
            document,
            check,
            run,
            &cx,
            default_anim_time_updates,
            diagnostics,
        );
    }
    let _ = default_anim_time_updates;
    not_checked(document, &base, diagnostics);
    true
}

#[cfg(feature = "molang")]
mod check {
    use super::ANIMATION_VALUE;
    use super::{
        CONTROLLER_TRANSITION, CONTROLLER_TRANSITION_1_8, DEFAULT_ANIM_TIME_UPDATE, MolangRun,
    };
    use bedrock_addon_core::diagnostic::{
        Diagnostic, DiagnosticCode, Diagnostics, Location, Phase, Severity,
    };
    use bedrock_addon_core::molang::{
        CompileFailure, DiagCode, MolangCheck, MolangContext, MolangFailure, MolangFieldMut,
        MolangFieldSpec, MolangFields, MolangSource, RawVersion, Side, is_pack_setting_lint,
        molang_diagnostic, visit_fields,
    };
    use bedrock_addon_core::pack_identity::PackType;

    /// The compile check of every member, each with its own failure rule, then the default
    /// `anim_time_update` once per animation that has none (a quiet compile that fills the cache).
    pub(super) fn pass_two<T: MolangFields>(
        document: &mut T,
        check: &MolangCheck,
        run: &MolangRun<'_>,
        cx: &MolangContext,
        default_anim_time_updates: usize,
        diagnostics: &mut Diagnostics,
    ) -> bool {
        let side = if run.load.pack_type == PackType::Resource {
            Side::Client
        } else {
            Side::Server
        };
        let mut keep = true;
        visit_fields(
            document,
            &bedrock_addon_core::diagnostic::JsonPath::new(),
            |field| {
                let source = match field.value {
                    MolangFieldMut::Value(value) => value.source(),
                    MolangFieldMut::Source(source) => Some(&*source),
                    MolangFieldMut::List(_) => None,
                };
                let Some(source) = source else { return };
                let transition = *field.spec == CONTROLLER_TRANSITION;
                let spec = if transition && run.transitions_typed {
                    &CONTROLLER_TRANSITION_1_8
                } else {
                    field.spec
                };
                // Below 1.17.30 the transition node lower-cases the whole text before it is
                // compiled; the stored source stays as written.
                let folded;
                let compiled_source = if transition && run.lowercase_transitions {
                    folded = MolangSource::string(
                        source.as_str().to_lowercase(),
                        source.raw_version().unwrap_or(RawVersion(-1)),
                    );
                    &folded
                } else {
                    source
                };
                let location = Location::new(None, None, field.path.clone(), None);
                if !compile(
                    check,
                    side,
                    cx,
                    spec,
                    source,
                    compiled_source,
                    &location,
                    diagnostics,
                ) && spec.on_failure == MolangFailure::RejectDocument
                {
                    keep = false;
                }
            },
        );
        if side == Side::Server && default_anim_time_updates > 0 {
            // The absent `anim_time_update` is vanilla's default expression at version 13;
            // one cache entry serves every animation.
            let source = MolangSource::string(DEFAULT_ANIM_TIME_UPDATE, RawVersion(13));
            let mut quiet = Diagnostics::default();
            for _ in 0..default_anim_time_updates {
                compile(
                    check,
                    side,
                    cx,
                    &ANIMATION_VALUE,
                    &source,
                    &source,
                    &Location::default(),
                    &mut quiet,
                );
            }
        }
        keep
    }

    /// Compiles one member; `false` when it was rejected.
    #[allow(clippy::too_many_arguments, reason = "one member's whole context")]
    fn compile(
        check: &MolangCheck,
        side: Side,
        cx: &MolangContext,
        spec: &MolangFieldSpec,
        written: &MolangSource,
        source: &MolangSource,
        location: &Location,
        diagnostics: &mut Diagnostics,
    ) -> bool {
        let options = check.options(spec, side, cx);
        let compiled = check.cache().compile_source(source, &options);
        let start = diagnostics.len();
        let mut pack_setting_told = false;
        for diagnostic in compiled.diagnostics() {
            let mut finding =
                molang_diagnostic(diagnostic, written, spec.error_severity, location.clone());
            // A member the game keeps after the parser's error: the document loads.
            if finding.severity == Severity::Error
                && matches!(
                    spec.on_failure,
                    MolangFailure::KeepLogged | MolangFailure::ReplaceWith(_)
                )
            {
                finding = finding.into_kept();
            }
            if side == Side::Server && diagnostic.code() == DiagCode::QueryClientOnly {
                // The crate: vanilla's own `anim_time_update` default reads `query.anim_time` on
                // the server, so the client-only note stays off. But for the pack-setting queries,
                // which fail in a behaviour pack: one warning for the expression, as in any other
                // behaviour-pack Molang.
                if !is_pack_setting_lint(diagnostic)
                    || std::mem::replace(&mut pack_setting_told, true)
                {
                    continue;
                }
                // An information of the crate's own.
                finding.severity = Severity::Info;
            }
            diagnostics.push(finding);
        }
        match compiled.failure() {
            None => true,
            Some(CompileFailure::UsesArrays | CompileFailure::UsesResources) => {
                if side == Side::Server {
                    // The crate's own information: a render-only name in a behaviour pack evaluates
                    // to 0 on the server.
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::InvalidMolang {
                            expression: written.as_str().to_owned(),
                        },
                        Severity::Info,
                        Phase::Validate,
                        location.clone(),
                    ));
                }
                true
            }
            Some(_) => {
                // A rejection with only an info note (an empty text below version 4) still takes
                // the member's severity.
                let reached = diagnostics
                    .iter()
                    .skip(start)
                    .any(|d| d.severity >= spec.error_severity);
                if !reached {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::InvalidMolang {
                            expression: written.as_str().to_owned(),
                        },
                        spec.error_severity,
                        Phase::Validate,
                        location.clone(),
                    ));
                }
                false
            }
        }
    }
}
