//! Test support for the content crates: runs one `Upgrade` step on a payload and gives what it
//! made of it, without a pack or a load around it. No I/O.
//!
//! [`step_output`] takes the payload of a step's own struct (`S`), written as JSON, parses it as
//! `S` (a parse that fails or reports an error is [`FailureKind::InputRejected`]), runs
//! `S::upgrade` and gives the next struct's payload with the diagnostics the step raised, each
//! named as the input has its member. [`Run`] sets the original version of the document and
//! whether the version-gated upgrade ops are skipped; [`Checks`] sets the load context, the
//! identifier and the actor type the step sees. Both have a `Default`.
//!
//! ```ignore
//! let output = step_output::<V1_0_0>(&input, Run::default(), &Checks::default())?;
//! assert_eq!(output.payload, expected);
//! ```

use std::fmt;

use super::{ParseCx, Upgrade, UpgradeCx, VersionStruct};
use crate::diagnostic::{Diagnostics, JsonPath};
use crate::json::{JsonSlice, Value};
use crate::load::LoadContext;
use crate::vanilla::ActorType;
use crate::version::{FormatVersion, FormatVersionInput};

/// Why a step could not be run.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum FailureKind {
    /// The input does not parse as the step's struct, or parses with an error.
    InputRejected {
        /// What the parse reported.
        diagnostics: Diagnostics,
    },
    /// The input or the output could not be written as JSON.
    NotWritten,
}

impl fmt::Display for FailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputRejected { diagnostics } => {
                write!(f, "the input does not parse: {}", diagnostics.summary())
            }
            Self::NotWritten => f.write_str("the output could not be written"),
        }
    }
}

impl std::error::Error for FailureKind {}

/// How a step is run: the original version (`None`: the step's own version) and the
/// suppression of the version-gated upgrade ops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Run {
    /// `UpgradeCx::original_version`.
    pub original_version: Option<FormatVersion>,
    /// `UpgradeCx::skip_section_a`.
    pub skip_section_a: bool,
}

/// What a step produced.
#[derive(Clone, Debug, PartialEq)]
pub struct StepOutput {
    /// The next struct, written.
    pub payload: Value,
    /// What the step reported.
    pub diagnostics: Diagnostics,
}

/// The context [`step_output`] runs a step in.
#[derive(Clone, Debug)]
pub struct Checks<'a> {
    /// The load context the step sees.
    pub load: LoadContext,
    /// `UpgradeCx::identifier`.
    pub identifier: &'a str,
    /// `UpgradeCx::actor_type`.
    pub actor_type: ActorType,
}

impl Default for Checks<'_> {
    fn default() -> Self {
        Self {
            load: LoadContext::detached(),
            identifier: "",
            actor_type: ActorType::Mob,
        }
    }
}

/// Runs `S`'s one step on `input`, a payload of `S`'s version, and gives the payload of the
/// next struct with what the step reported, each finding at the member as `input` has it.
///
/// # Errors
///
/// [`FailureKind::InputRejected`] when `input` does not parse as `S` without an error,
/// [`FailureKind::NotWritten`] when the input or the output has no JSON text.
pub fn step_output<S>(
    input: &Value,
    settings: Run,
    checks: &Checks<'_>,
) -> Result<StepOutput, FailureKind>
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    run_step::<S>(input, settings, S::VERSION, checks)
}

/// Runs `S::upgrade` on `input` written as JSON.
fn run_step<S>(
    input: &Value,
    settings: Run,
    from: FormatVersion,
    checks: &Checks<'_>,
) -> Result<StepOutput, FailureKind>
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let text = crate::json::to_json(input).map_err(|_| FailureKind::NotWritten)?;
    let original_version = settings.original_version.unwrap_or(from);
    let parse = ParseCx {
        load: &checks.load,
        written: FormatVersionInput {
            version: Some(original_version),
            beta_literal: false,
            prerelease: false,
        },
        original_version,
        at: JsonPath::new(),
    };
    let mut diagnostics = Diagnostics::default();
    let Some(value) = S::parse(JsonSlice::new(&text), &parse, &mut diagnostics) else {
        return Err(FailureKind::InputRejected { diagnostics });
    };
    if diagnostics.has_errors() {
        return Err(FailureKind::InputRejected { diagnostics });
    }
    let mut out = Diagnostics::default();
    let next = value.upgrade(&mut UpgradeCx {
        load: &checks.load,
        original_version,
        identifier: checks.identifier,
        actor_type: checks.actor_type,
        skip_section_a: settings.skip_section_a,
        diagnostics: &mut out,
    });
    let payload = next.write().map_err(|_| FailureKind::NotWritten)?;
    // As a load does when its chain is done: what the step moved is named as its input has
    // it.
    out.name_as_the_file_has_it();
    Ok(StepOutput {
        payload,
        diagnostics: out,
    })
}

/// The reading table of `F`: what the family fixes for every file (the version input, the
/// clamp beyond a double, the duplicates kept, the envelope check, what an absent
/// `format_version` does, the line below the floor, the severity of an empty document) and
/// the reading of each of its structs from each source, one line per run of structs that read
/// alike: `null/scalars/beyond_width/unknown/keeps|reports/fatal|lenient/on_member_failure`,
/// where `keeps` is [`Reading::keeps_unknown`](super::Reading::keeps_unknown). A family crate pins it with
/// [`assert_reading_table`], so a change to how a family declares its reading cannot change
/// what the pipeline reads.
#[must_use]
pub fn reading_table<F: super::Family>() -> String {
    use std::fmt::Write as _;

    use super::{MissingVersion, StructIndex, VersionChain};
    use crate::diagnostic::VanillaText;
    use crate::load::DocumentSource;

    let mut out = String::new();
    let envelope = F::ENVELOPE;
    let _ = writeln!(
        out,
        "text: {:?}, clamps beyond a double: {}, duplicates kept in: {:?}",
        envelope.input.version_input(),
        envelope.clamps_beyond_double,
        envelope.duplicates_kept_in,
    );
    let _ = writeln!(out, "envelope: {:?}", envelope.check);
    let missing = match envelope.missing {
        MissingVersion::Reject { text } => format!(
            "rejected: {:?}",
            text.unwrap_or(VanillaText::NoValidVersion)
        ),
        MissingVersion::Floor {
            text,
            severity: Some(severity),
        } => format!("at the floor, told at {severity:?}: {text:?}"),
        MissingVersion::Floor { severity: None, .. } => "at the floor, silently".to_owned(),
    };
    let _ = writeln!(out, "missing version: {missing}");
    let _ = writeln!(out, "below floor: {:?}", envelope.below_floor);
    let _ = writeln!(out, "empty document: {:?}", F::EMPTY_DOCUMENT);

    let reading = |index: StructIndex| {
        let one = |source: DocumentSource| {
            let r = slice_reading::<F>(index, source);
            format!(
                "{:?}/{:?}/{:?}/{:?}/{}/{}/{:?}",
                r.null,
                r.scalars,
                r.beyond_width,
                r.unknown,
                if r.keeps_unknown { "keeps" } else { "reports" },
                if r.components_fatal {
                    "fatal"
                } else {
                    "lenient"
                },
                r.on_member_failure,
            )
        };
        format!(
            "json {}, mcb {}",
            one(DocumentSource::Json),
            one(DocumentSource::Mcb)
        )
    };
    let versions = <F::History as VersionChain<F>>::VERSIONS;
    let mut start = 0;
    while start < versions.len() {
        let line = reading(StructIndex::Stable(start));
        let mut end = start;
        while end + 1 < versions.len() && reading(StructIndex::Stable(end + 1)) == line {
            end += 1;
        }
        let _ = writeln!(
            out,
            "{}..={} ({start}..={end}): {line}",
            versions[start], versions[end]
        );
        start = end + 1;
    }
    if <F::History as VersionChain<F>>::HAS_BETA {
        let _ = writeln!(out, "beta: {}", reading(StructIndex::Beta));
    }
    out
}

pub use super::reading::slice_reading;

/// Asserts that `actual`, a [`reading_table`] (or several, joined), is `expected`, a table
/// written one line per line with any indentation and blank lines around it (no line of a
/// table starts with a space); on a mismatch the panic prints the actual table whole, between
/// marker lines, for review.
///
/// # Panics
///
/// When the tables differ.
#[track_caller]
pub fn assert_reading_table(name: &str, actual: &str, expected: &str) {
    let mut table = String::new();
    for line in expected
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        table.push_str(line);
        table.push('\n');
    }
    let expected = table;
    assert!(
        actual == expected,
        "the reading table of {name} changed\n--- actual reading table of {name} ---\n{actual}--- end ---"
    );
}
