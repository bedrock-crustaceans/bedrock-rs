//! Pass 2, the load-time compile check (feature `molang`): every expression compiled
//! once per distinct text and options through one `CompileCache`, its diagnostics reported as
//! `InvalidMolang` / `MolangLint` at the field's path, its outcome handed to the family.

use std::fmt;
use std::sync::Arc;

use molang::catalog::AllowListError;
use molang::stdlib;

use super::fields::{MolangFieldMut, at, visit_fields};
use super::{
    Admission, BLOCK_STATE_QUERIES, CheckOptions, Checked, CompileCache, CompileFailure,
    CompileOptions, EmptyRule, Molang, MolangContext, MolangDiagnostic, MolangFailure,
    MolangFieldSpec, MolangFields, MolangList, MolangOutcome, MolangSeverity, MolangSource,
    MolangVersionSource, OnFailure, OpSet, OpsRule, PROPERTY_DEFAULT_QUERIES, QueryAdmission,
    QueryAllowList, QueryCatalog, QuerySetMask, RawVersion, Side, SourceForm, VersionFrom,
};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
};

/// The load-time compile check: the query catalogue of each side and the compile cache a load
/// compiles its expressions against (`LoadOptions::molang`).
///
/// One per load, shared by every parse job: the cache compiles each distinct text once per set
/// of options. A host that later compiles the same expressions itself passes its own
/// catalogues and cache through [`MolangCheck::with_catalogs`], so its compiles are cache hits.
pub struct MolangCheck {
    cache: Arc<CompileCache>,
    server: QueryCatalog,
    client: QueryCatalog,
}

impl MolangCheck {
    /// The standard catalogues of molang (`stdlib::queries`) and a new, empty cache.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(CompileCache::new()),
            server: stdlib::queries(Side::Server).clone(),
            client: stdlib::queries(Side::Client).clone(),
        }
    }

    /// The caller's catalogues and cache. Fails when a catalogue does not declare a query the
    /// load's allow-lists name (`query.block_state`, `query.had_component_group`).
    ///
    /// # Errors
    ///
    /// Returns `AllowListError` when a catalogue does not declare a query the load's allow-lists name.
    pub fn with_catalogs(
        server: QueryCatalog,
        client: QueryCatalog,
        cache: Arc<CompileCache>,
    ) -> Result<Self, AllowListError> {
        for catalog in [&server, &client] {
            for names in [BLOCK_STATE_QUERIES, PROPERTY_DEFAULT_QUERIES] {
                QueryAllowList::new(catalog, names)?;
            }
        }
        Ok(Self {
            cache,
            server,
            client,
        })
    }

    /// The compile cache; its `hits()` and `misses()` count the load's compiles.
    pub fn cache(&self) -> &Arc<CompileCache> {
        &self.cache
    }

    /// The catalogue expressions of `side` compile against.
    pub fn catalog(&self, side: Side) -> &QueryCatalog {
        match side {
            Side::Server => &self.server,
            Side::Client => &self.client,
        }
    }

    /// The compile options of a field: its side's catalogue at the pack's version (which
    /// `compile_source` replaces by the source's own), its admission and operations, the world's
    /// experiments; `keep_source` off and every deviation on.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    pub fn options(
        &self,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
    ) -> CompileOptions {
        let catalog = self.catalog(side);
        let admission = match spec.admission {
            Admission::Default => QueryAdmission::Sets(QuerySetMask::DEFAULT),
            Admission::Tags => QueryAdmission::Sets(QuerySetMask::TAGS),
            Admission::WorldGen => QueryAdmission::Sets(QuerySetMask::WORLD_GEN),
            // A list a catalogue does not declare admits nothing; `with_catalogs` checked the
            // lists the families use.
            Admission::Only(names) => QueryAllowList::new(catalog, names).map_or(
                QueryAdmission::Sets(QuerySetMask::empty()),
                QueryAdmission::Only,
            ),
        };
        let allowed_ops = match spec.ops {
            OpsRule::All => OpSet::all(),
            OpsRule::NoAssignments => OpSet::all().without_assignments(),
            OpsRule::NoAssignmentsOrRandom => OpSet::all().without_assignments_or_random(),
        };
        CompileOptions::from_raw_version(catalog.clone(), cx.pack)
            .with_admission(admission)
            .with_allowed_ops(allowed_ops)
            .with_experiments(cx.experiments)
    }

    /// Checks one `Molang` field at `location`: a constant or a bool needs no compile; an
    /// expression is checked as [`MolangCheck::check_source`] checks it.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    pub fn check(
        &self,
        value: &Molang,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
        location: &Location,
        diagnostics: &mut Diagnostics,
    ) -> MolangOutcome {
        match value.source() {
            Some(source) => self.check_source(source, spec, side, cx, location, diagnostics),
            None => MolangOutcome::Ok,
        }
    }

    /// Checks one value-or-array field: every element, under the field's options, its findings at
    /// the element's index below `location` (vanilla logs a failing element by itself). The field
    /// fails when an element fails.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    pub fn check_list(
        &self,
        list: &MolangList,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
        location: &Location,
        diagnostics: &mut Diagnostics,
    ) -> MolangOutcome {
        match list {
            MolangList::One(value) => self.check(value, spec, side, cx, location, diagnostics),
            MolangList::Many(elements) => {
                let mut outcome = MolangOutcome::Ok;
                let mut at = location.clone();
                for (index, element) in elements.iter().enumerate() {
                    at.json_path.push_index(index);
                    let element = self.check_list(element, spec, side, cx, &at, diagnostics);
                    at.json_path.pop();
                    if matches!(element, MolangOutcome::Failed(_)) {
                        outcome = element;
                    }
                }
                outcome
            }
        }
    }

    /// The check options of a field: [`MolangCheck::options`], the field's failure rule as a
    /// policy (`ReplaceWith` keeps the expression, the family substitutes the value), arrays and
    /// resources accepted, and the version from the source, or from the options for
    /// `PackIgnoringObject`.
    pub(crate) fn check_options(
        &self,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
    ) -> CheckOptions {
        let on_failure = match spec.on_failure {
            MolangFailure::RejectDocument => OnFailure::Reject,
            MolangFailure::DropEntry => OnFailure::Drop,
            MolangFailure::KeepLogged | MolangFailure::ReplaceWith(_) => OnFailure::Keep,
        };
        let version_from = match spec.version {
            MolangVersionSource::PackIgnoringObject => VersionFrom::Options,
            _ => VersionFrom::Source,
        };
        CheckOptions::new(self.options(spec, side, cx))
            .with_on_failure(on_failure)
            .with_version_from(version_from)
    }

    /// Compiles one expression under its field's [`check_options`](Self::check_options) through
    /// the cache; nothing is reported.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    pub fn compile(
        &self,
        source: &MolangSource,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
    ) -> Checked {
        self.cache
            .check(source, &self.check_options(spec, side, cx))
    }

    /// Checks one expression at `location`: what the field's
    /// `EmptyRule` makes no expression (an empty text in a cereal field; in block files also an
    /// object form at version −1) is compiled and logged nothing; otherwise the expression
    /// compiles at its own version (the pack's for `PackIgnoringObject`) and every compiler
    /// diagnostic is reported; a rejecting message of a field whose spec has `error_kept` is a
    /// kept error.
    /// Only a rejection fails the field: an expression that uses render-controller
    /// arrays or resources parsed, and in a behaviour pack (the server side) gets one
    /// `MolangLint` warning because molang cannot evaluate it. On the server side an
    /// expression that calls `query.get_pack_setting`, `query.is_pack_setting_enabled` or
    /// `query.is_pack_setting_selected` gets one `MolangLint` warning (the compiler's
    /// client-only note about the first of them): those queries fail in a behaviour pack.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    pub fn check_source(
        &self,
        source: &MolangSource,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
        location: &Location,
        diagnostics: &mut Diagnostics,
    ) -> MolangOutcome {
        let (outcome, _) = self.check_source_with(
            source,
            source,
            spec,
            side,
            cx,
            location,
            &ReportRules::LOAD,
            diagnostics,
        );
        outcome
    }

    /// [`check_source`](Self::check_source) for a family's own pass 2: `source` is compiled,
    /// the findings quote `written` (the text as the file holds it), and `rules` say what the
    /// family reports beyond the shared mapping. Returns the outcome and the compile, `None` for
    /// no expression.
    // hidden: Molang pass machinery of the crate family, outside SemVer
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments, reason = "one member's whole context")]
    pub fn check_source_with(
        &self,
        written: &MolangSource,
        source: &MolangSource,
        spec: &MolangFieldSpec,
        side: Side,
        cx: &MolangContext,
        location: &Location,
        rules: &ReportRules,
        diagnostics: &mut Diagnostics,
    ) -> (MolangOutcome, Option<Checked>) {
        // Pass 1 gave every loaded source its version; one without would compile at `Invalid`
        // rules with molang's "made without its load context" warning.
        debug_assert!(
            source.raw_version().is_some(),
            "a Molang source reached the compile pass without its version: {source:?}"
        );
        if is_no_expression(source, spec.empty) {
            // Vanilla clears the expression node: no compile, no log.
            return (MolangOutcome::NoExpression, None);
        }
        let checked = self.compile(source, spec, side, cx);
        let outcome = report(&checked, written, spec, side, location, *rules, diagnostics);
        (outcome, Some(checked))
    }
}

/// What a pass 2 reports beyond the shared mapping of [`molang_diagnostic`]. The load's own
/// rules are [`ReportRules::LOAD`]; a family whose game loader logs differently says so here.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReportRules {
    /// Leave out molang's argument-count lint for the calls vanilla's own files make
    /// (`query.equipped_item_any_tag` with a slot and a tag, `query.anger_level(this)`).
    pub vanilla_arities: bool,
    /// On the server side, report the client-only notes about queries other than the
    /// pack-setting ones (those are told once per expression either way).
    pub other_client_only: bool,
    /// On the server side, the information for an expression that uses arrays or resources.
    pub render_note: RenderNote,
    /// A rejection whose findings all stay below the field's severity gets one `InvalidMolang`
    /// at it.
    pub rejection_floor: bool,
}

impl ReportRules {
    /// The load's rules ([`MolangCheck::check_source`]).
    pub const LOAD: Self = Self {
        vanilla_arities: true,
        other_client_only: true,
        render_note: RenderNote::MolangLint,
        rejection_floor: false,
    };
}

/// The code of the server-side information for an expression molang cannot evaluate.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderNote {
    /// `MolangLint`, the load's.
    MolangLint,
    /// `InvalidMolang`, the animation loaders'.
    InvalidMolang,
}

/// Reports `checked` at `location` under `rules` and gives the field's outcome: only a
/// rejection fails it.
fn report(
    checked: &Checked,
    written: &MolangSource,
    spec: &MolangFieldSpec,
    side: Side,
    location: &Location,
    rules: ReportRules,
    diagnostics: &mut Diagnostics,
) -> MolangOutcome {
    let start = diagnostics.len();
    // Diagnostics are cached with the program; a hit reports them again at this field.
    let mut pack_setting_reported = false;
    for diagnostic in checked.diagnostics() {
        let mut finding =
            molang_diagnostic(diagnostic, written, spec.error_severity, location.clone());
        // The game logs the parser's line at error level and goes on.
        if spec.error_kept && finding.severity == Severity::Error {
            finding = finding.into_kept();
        }
        if rules.vanilla_arities
            && (is_slot_and_tag_arity(diagnostic) || is_anger_level_arity(diagnostic))
        {
            // Vanilla's own files call these with a slot and a tag, and
            // `query.anger_level` with `this`.
            continue;
        }
        if side == Side::Server && is_pack_setting_lint(diagnostic) {
            // One information per expression, however many of the queries it calls. The
            // crate's own finding: the game's line comes from the query itself, when the
            // query runs, not at the load.
            if std::mem::replace(&mut pack_setting_reported, true) {
                continue;
            }
            finding.severity = Severity::Info;
        } else if side == Side::Server
            && !rules.other_client_only
            && diagnostic.code() == super::DiagCode::QueryClientOnly
        {
            continue;
        }
        diagnostics.push(finding);
    }
    match checked.failure() {
        None => MolangOutcome::Ok,
        Some(CompileFailure::UsesArrays | CompileFailure::UsesResources) => {
            if side == Side::Server {
                // The crate's own finding: the game has no line for it.
                let expression = written.as_str().to_owned();
                let code = match rules.render_note {
                    RenderNote::MolangLint => DiagnosticCode::MolangLint { expression },
                    RenderNote::InvalidMolang => DiagnosticCode::InvalidMolang { expression },
                };
                diagnostics.push(Diagnostic::new(
                    code,
                    Severity::Info,
                    Phase::Validate,
                    location.clone(),
                ));
            }
            MolangOutcome::Ok
        }
        Some(_) => {
            // A rejection with only an info note (an empty text below version 4) takes the
            // field's severity.
            if rules.rejection_floor
                && !diagnostics
                    .iter()
                    .skip(start)
                    .any(|d| d.severity >= spec.error_severity)
            {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::InvalidMolang {
                        expression: written.as_str().to_owned(),
                    },
                    spec.error_severity,
                    Phase::Validate,
                    location.clone(),
                ));
            }
            MolangOutcome::Failed(spec.on_failure)
        }
    }
}

impl Default for MolangCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MolangCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MolangCheck")
            .field("cache", &self.cache)
            .field("server", &self.server)
            .field("client", &self.client)
            .finish()
    }
}

/// The pack-setting queries: in a behaviour pack vanilla fails each at run time ("pack settings
/// query functions are not supported in behavior packs.").
const PACK_SETTING_QUERIES: [&str; 3] = [
    "query.get_pack_setting",
    "query.is_pack_setting_enabled",
    "query.is_pack_setting_selected",
];

/// Whether `diagnostic` is molang's client-only lint about one of the three pack-setting
/// queries: the lint names its query first, by its declared name. [`MolangCheck::check_source`]
/// tells it apart (a warning, once for an expression, on the server side).
// molang `lint_query_side`: "<name> is a client-only query in this crate's table; …".
#[must_use]
pub(crate) fn is_pack_setting_lint(diagnostic: &MolangDiagnostic) -> bool {
    diagnostic.code() == super::DiagCode::QueryClientOnly
        && diagnostic
            .message()
            .split_whitespace()
            .next()
            .is_some_and(|name| PACK_SETTING_QUERIES.contains(&name))
}

/// The queries vanilla's files call with a slot and a tag, where molang's table registers at
/// most one argument (`query_table.rs`: `args(0, 1)`); molang is not changed, so its lint is
/// not passed on for exactly that count.
const SLOT_AND_TAG_QUERIES: [&str; 2] = [
    "query.equipped_item_any_tag",
    "query.equipped_item_all_tags",
];

/// Whether `diagnostic` is molang's argument-count lint about one of
/// [`SLOT_AND_TAG_QUERIES`] called with two arguments.
// molang: "<name> is registered with <n> arguments, <m> given (this crate's check)".
fn is_slot_and_tag_arity(diagnostic: &MolangDiagnostic) -> bool {
    let message = diagnostic.message();
    diagnostic.code() == super::DiagCode::QueryArity
        && message
            .split_whitespace()
            .next()
            .is_some_and(|name| SLOT_AND_TAG_QUERIES.contains(&name))
        && message.contains(", 2 given")
}

/// Whether `diagnostic` is molang's argument-count lint about `query.anger_level` called with
/// one argument: the game registers it with none (minimum and maximum 0) and its function never
/// reads the arguments, so vanilla's warden (`query.anger_level(this)`) runs without a word.
fn is_anger_level_arity(diagnostic: &MolangDiagnostic) -> bool {
    let message = diagnostic.message();
    diagnostic.code() == super::DiagCode::QueryArity
        && message
            .split_whitespace()
            .next()
            .is_some_and(|name| name == "query.anger_level")
        && message.contains(", 1 given")
}

/// The field's empty rule makes `source` no expression: an empty text for the cereal rules, and
/// for block files also an object form at version −1.
fn is_no_expression(source: &MolangSource, rule: EmptyRule) -> bool {
    match rule {
        EmptyRule::Compile => false,
        EmptyRule::NoExpression => source.is_empty(),
        EmptyRule::BlockNoExpression => {
            source.is_empty()
                || matches!(
                    source.form(),
                    SourceForm::Object {
                        version: RawVersion(-1)
                    }
                )
        }
    }
}

/// One compiler diagnostic as an addon finding at `location`: a parser
/// message is `InvalidMolang`. A rejecting one (`Error`) takes `error_severity` (the field's
/// spec decides whether it is kept, [`MolangFieldSpec::error_kept`]). One the expression
/// survives (molang's `Warning` or `Info`, a validator message below the root) is a
/// line the game logs at content-log error level and after which the parse goes on: a kept
/// error. A lint of molang's own is `MolangLint`: at error level an `InvalidMolang` at
/// `error_severity` as before, below it an information, since the game has no such line.
/// The detail is the compiler's diagnostic, so the content-log line ends in
/// vanilla's `<expression> | <message>`.
pub(crate) fn molang_diagnostic(
    diagnostic: &MolangDiagnostic,
    source: &MolangSource,
    error_severity: Severity,
    location: Location,
) -> Diagnostic {
    let expression = source.as_str().to_owned();
    let is_message = diagnostic.language_message().is_some();
    let (code, severity, kept) = match diagnostic.severity() {
        MolangSeverity::Error => (
            DiagnosticCode::InvalidMolang { expression },
            error_severity,
            false,
        ),
        MolangSeverity::Warning | MolangSeverity::Info if is_message => (
            DiagnosticCode::InvalidMolang { expression },
            Severity::Error,
            true,
        ),
        MolangSeverity::Warning | MolangSeverity::Info => (
            DiagnosticCode::MolangLint { expression },
            Severity::Info,
            false,
        ),
    };
    let mut finding = Diagnostic::new(code, severity, Phase::Validate, location);
    finding.detail = Some(Box::new(DiagnosticDetail::Molang(diagnostic.clone())));
    if kept { finding.into_kept() } else { finding }
}

/// A field the compile pass did not pass: its path and outcome.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct FieldOutcome {
    /// The field's JSON path.
    pub path: JsonPath,
    /// `NoExpression` or `Failed`.
    pub outcome: MolangOutcome,
}

/// Pass 2 over a document: every Molang field of `document` checked on `side`, its
/// findings at the field's path below `base`. Returns the fields whose outcome is not `Ok`, in
/// document order, for the family to apply their failure rules.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn check_fields<T: MolangFields + ?Sized>(
    document: &mut T,
    check: &MolangCheck,
    side: Side,
    cx: &MolangContext,
    base: &Location,
    diagnostics: &mut Diagnostics,
) -> Vec<FieldOutcome> {
    let mut outcomes = Vec::new();
    visit_fields(document, &base.json_path, |field| {
        let location = at(base, field.path);
        let spec = field.spec;
        let outcome = match field.value {
            MolangFieldMut::Value(value) => {
                check.check(value, spec, side, cx, &location, diagnostics)
            }
            MolangFieldMut::Source(source) => {
                check.check_source(source, spec, side, cx, &location, diagnostics)
            }
            MolangFieldMut::List(list) => {
                check.check_list(list, spec, side, cx, &location, diagnostics)
            }
        };
        if outcome != MolangOutcome::Ok {
            outcomes.push(FieldOutcome {
                path: field.path.clone(),
                outcome,
            });
        }
    });
    outcomes
}
