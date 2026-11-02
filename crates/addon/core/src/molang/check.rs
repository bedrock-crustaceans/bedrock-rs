//! Pass 2, the load-time compile check (feature `molang`): every expression compiled
//! once per distinct text and options through one `CompileCache`, its diagnostics reported as
//! `InvalidMolang` / `MolangLint` at the field's path, its outcome handed to the family.

use std::fmt;
use std::sync::Arc;

use molangx::catalog::AllowListError;
use molangx::stdlib;

use super::fields::{MolangFieldMut, at, visit_fields};
use super::{
    Admission, BLOCK_STATE_QUERIES, CompileCache, CompileFailure, CompileOptions, EmptyRule,
    Molang, MolangContext, MolangDiagnostic, MolangFieldSpec, MolangFields, MolangList,
    MolangOutcome, MolangSeverity, MolangSource, MolangVersionSource, OpSet, OpsRule,
    PROPERTY_DEFAULT_QUERIES, QueryAdmission, QueryAllowList, QueryCatalog, QuerySetMask,
    RawVersion, Side, SourceForm,
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
    /// The standard catalogues of molangx (`stdlib::queries`) and a new, empty cache.
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
        CompileOptions {
            admission,
            allowed_ops,
            experiments: cx.experiments,
            ..CompileOptions::from_raw_version(catalog.clone(), cx.pack)
        }
    }

    /// Checks one `Molang` field at `location`: a constant or a bool needs no compile; an
    /// expression is checked as [`MolangCheck::check_source`] checks it.
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

    /// Checks one expression at `location`: what the field's
    /// `EmptyRule` makes no expression (an empty text in a cereal field; in block files also an
    /// object form at version −1) is compiled and logged nothing; otherwise the expression
    /// compiles at its own version (the pack's for `PackIgnoringObject`) and every compiler
    /// diagnostic is reported; a rejecting message of a field whose spec has `error_kept` is a
    /// kept error.
    /// Only a rejection fails the field: an expression that uses render-controller
    /// arrays or resources parsed, and in a behaviour pack (the server side) gets one
    /// `MolangLint` warning because molangx cannot evaluate it. On the server side an
    /// expression that calls `query.get_pack_setting`, `query.is_pack_setting_enabled` or
    /// `query.is_pack_setting_selected` gets one `MolangLint` warning (the compiler's
    /// client-only note about the first of them): those queries fail in a behaviour pack.
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
        // Pass 1 gave every loaded source its version; one without would compile at `Invalid`
        // rules with molangx's "made without its load context" warning.
        debug_assert!(
            source.raw_version().is_some(),
            "a Molang source reached the compile pass without its version: {source:?}"
        );
        if is_no_expression(source, spec.empty) {
            // Vanilla clears the expression node: no compile, no log.
            return MolangOutcome::NoExpression;
        }
        let options = self.options(spec, side, cx);
        let compiled = match spec.version {
            MolangVersionSource::PackIgnoringObject => {
                self.cache.compile_source_ignoring_version(source, &options)
            }
            _ => self.cache.compile_source(source, &options),
        };
        // Diagnostics are cached with the program; a hit reports them again at this field.
        let mut pack_setting_reported = false;
        for diagnostic in compiled.diagnostics() {
            let mut finding =
                molang_diagnostic(diagnostic, source, spec.error_severity, location.clone());
            // The game logs the parser's line at error level and goes on.
            if spec.error_kept && finding.severity == Severity::Error {
                finding = finding.into_kept();
            }
            if is_slot_and_tag_arity(diagnostic) || is_anger_level_arity(diagnostic) {
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
            }
            diagnostics.push(finding);
        }
        match compiled.failure() {
            None => MolangOutcome::Ok,
            Some(CompileFailure::UsesArrays | CompileFailure::UsesResources) => {
                if side == Side::Server {
                    // The crate's own finding: the game has no line for it.
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::MolangLint {
                            expression: source.as_str().to_owned(),
                        },
                        Severity::Info,
                        Phase::Validate,
                        location.clone(),
                    ));
                }
                MolangOutcome::Ok
            }
            Some(_) => MolangOutcome::Failed(spec.on_failure),
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

/// Whether `diagnostic` is molangx's client-only lint about one of the three pack-setting
/// queries: the lint names its query first, by its declared name. For a family that compiles its
/// expressions itself and tells the lint as [`MolangCheck::check_source`] does (a warning,
/// once for an expression, on the server side).
// molangx `lint_query_side`: "<name> is a client-only query in this crate's table; …".
#[doc(hidden)]
#[must_use]
pub fn is_pack_setting_lint(diagnostic: &MolangDiagnostic) -> bool {
    diagnostic.code() == super::DiagCode::QueryClientOnly
        && diagnostic
            .message()
            .split_whitespace()
            .next()
            .is_some_and(|name| PACK_SETTING_QUERIES.contains(&name))
}

/// The queries vanilla's files call with a slot and a tag, where molangx's table registers at
/// most one argument (`query_table.rs`: `args(0, 1)`); molangx is not changed, so its lint is
/// not passed on for exactly that count.
const SLOT_AND_TAG_QUERIES: [&str; 2] = [
    "query.equipped_item_any_tag",
    "query.equipped_item_all_tags",
];

/// Whether `diagnostic` is molangx's argument-count lint about one of
/// [`SLOT_AND_TAG_QUERIES`] called with two arguments.
// molangx: "<name> is registered with <n> arguments, <m> given (this crate's check)".
fn is_slot_and_tag_arity(diagnostic: &MolangDiagnostic) -> bool {
    let message = diagnostic.message();
    diagnostic.code() == super::DiagCode::QueryArity
        && message
            .split_whitespace()
            .next()
            .is_some_and(|name| SLOT_AND_TAG_QUERIES.contains(&name))
        && message.contains(", 2 given")
}

/// Whether `diagnostic` is molangx's argument-count lint about `query.anger_level` called with
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
/// survives (molangx's `Warning` or `Info`, a validator message below the root) is a
/// line the game logs at content-log error level and after which the parse goes on: a kept
/// error. A lint of molangx's own is `MolangLint`: at error level an `InvalidMolang` at
/// `error_severity` as before, below it an information, since the game has no such line.
/// The detail is the compiler's diagnostic, so the content-log line ends in
/// vanilla's `<expression> | <message>`.
#[doc(hidden)]
pub fn molang_diagnostic(
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
