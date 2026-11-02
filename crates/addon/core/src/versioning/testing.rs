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
