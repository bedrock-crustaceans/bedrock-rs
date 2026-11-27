//! A CI gate over `Severity` and the findings' codes: load both stacks of a world, run the link
//! pass, print what a pack author must see and decide whether the build passes.
//!
//! The pack is held in memory and has three deliberate faults, of the two severities a gate
//! most often meets:
//! - an **error** of the load: `blocks/broken.json` has no `description`, so the block cannot
//!   load and is listed under `blocks().rejected()`;
//! - an **error** of the link pass: the entity `demo:keeper` leaves a trail of a block nothing
//!   defines; the game's link validator logs the miss at the content log's error level and
//!   removes nothing, so the entity is kept and the finding is not a
//!   rejection;
//! - a **lint** (an information): the same entity names a loot table no pack has; the game
//!   keeps the path as written and logs nothing, so only the link pass says it.
//!
//! The first comes from the load, the other two from `StackSet::link()`. A `Warning` is the
//! game's own level-2 line or a member it drops; no fault here is one.
//!
//! Run with `cargo run -p bedrock_addon --example diagnostics_gate --no-default-features --features block,entity,loot`.

mod common;

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Kind, Phase, Severity, codes};
use bedrock_addon::pack::{LinkOptions, StackSet};
use bedrock_addon::storage::PackPath;

/// What the gate decided, and the counts it decided on.
#[derive(Debug, PartialEq, Eq)]
struct Verdict {
    /// The build passes: nothing is an error, so `errors` is 0.
    passed: bool,
    /// Findings of `Severity::Error`: a document or definition was dropped, or the game logs
    /// an error and keeps it (a kept error, which `has_errors()` does not count).
    errors: usize,
    /// Findings of `Severity::Warning`: a member was dropped or replaced by a default.
    warnings: usize,
    /// Findings of `Severity::Info` the link pass raised: its lints.
    lints: usize,
    /// Files with a document that did not load.
    rejected: Vec<PackPath>,
}

/// Loads everything, links, prints every finding from a warning up and decides.
fn ci_gate(stacks: &StackSet) -> Verdict {
    let report = stacks.load_all();
    let links = stacks.link();

    for finding in report
        .at_least(Severity::Warning)
        .chain(links.at_least(Severity::Warning))
    {
        eprintln!("{finding}");
        if let Some(span) = finding.location.span {
            eprintln!("  at byte {}", span.offset);
        }
    }
    // A code is matched as data, never by its text.
    let deprecated = report
        .iter()
        // By kind: whichever rule raised it; the lint that an item is read with the legacy
        // schema is of the same kind and upgrades no member.
        .filter(|finding| {
            finding.kind() == Kind::Deprecated && !finding.is(&codes::LEGACY_ITEM_FORMAT)
        })
        .count();
    eprintln!("{deprecated} deprecated member(s) upgraded");
    let rejected: Vec<PackPath> = stacks
        .behavior
        .blocks()
        .rejected()
        .iter()
        .map(|rejected| rejected.origin.path.clone())
        .collect();
    for path in &rejected {
        eprintln!("skipped {path}");
    }

    let count = |severity: Severity| {
        report
            .iter()
            .chain(links.iter())
            .filter(|finding| finding.severity == severity)
            .count()
    };
    // Every error counts, a kept one too: it is an error in the game's log. (`has_errors()`
    // asks the narrower question, whether an error rejected something.)
    let errors = count(Severity::Error);
    Verdict {
        passed: errors == 0,
        errors,
        warnings: count(Severity::Warning),
        lints: links
            .iter()
            .filter(|finding| finding.severity == Severity::Info)
            .count(),
        rejected,
    }
}

/// The kind and the name of the reference a finding is about, when it is about one.
fn reference(finding: &Diagnostic) -> Option<(&'static str, &str)> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => Some((*kind, id.as_str())),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stacks = common::gate_stack_set()?;
    let verdict = ci_gate(&stacks);
    println!("{verdict:?}");

    // The gate's verdict: two errors, so the build does not pass.
    assert_eq!(
        verdict,
        Verdict {
            passed: false,
            errors: 2,
            warnings: 0,
            lints: 1,
            rejected: vec![PackPath::from_static("blocks/broken.json")],
        }
    );

    // The error is the load's, by its code and not by its text.
    let report = stacks.load_all();
    let [error] = report
        .at_least(Severity::Error)
        .collect::<Vec<_>>()
        .try_into()
        .expect("one error");
    // By kind: whichever rule raised it.
    assert_eq!(error.kind(), Kind::MissingRequired);
    assert_ne!(error.phase, Phase::Link);
    assert_eq!(
        error.location.path,
        Some(PackPath::from_static("blocks/broken.json"))
    );

    // The link error and the lint are the link pass's, both `UnresolvedReference`; what tells
    // them apart is the severity, and what they name is in the finding's detail.
    let links = stacks.link();
    let said: Vec<_> = links
        .iter()
        .map(|finding| {
            // By kind: whichever rule raised it.
            assert_eq!(finding.kind(), Kind::UnresolvedReference);
            assert_eq!(finding.phase, Phase::Link);
            (finding.severity, reference(finding).expect("a reference"))
        })
        .collect();
    assert_eq!(
        said,
        [
            (Severity::Error, ("block", "demo:missing_block")),
            (
                Severity::Info,
                ("loot table", "loot_tables/entities/keeper.json")
            ),
        ]
    );

    // A server that links on every start leaves the lints out: the rules whose finding is an
    // information do not run, and everything else is told as before.
    let quiet = stacks.link_with(&LinkOptions {
        lints: false,
        ..Default::default()
    });
    assert_eq!(quiet.len(), 1);
    assert_eq!(quiet.iter().next(), links.iter().next());
    println!(
        "{} link finding(s) with the lints, {} without",
        links.len(),
        quiet.len()
    );
    Ok(())
}
