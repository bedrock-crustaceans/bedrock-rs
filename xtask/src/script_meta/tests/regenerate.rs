//! The committed tables against the metadata repository itself: `cargo xtask script-meta
//! --samples $BEDROCK_ADDON_CORPUS/bedrock-samples` must leave the three generated files
//! unchanged, and the tag must pass every check. Skipped when that checkout is absent.

use std::path::{Path, PathBuf};

use bedrock_addon_test_support::corpus;

use super::step::{self, Check, Level};

/// The committed file the step writes to `out` (a path from the workspace root).
fn committed(out: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::fs::read_to_string(root.join(out)).unwrap_or_else(|error| panic!("{out}: {error}"))
}

/// The metadata checkout, or `None` (with a message) without one.
fn samples_dir() -> Option<PathBuf> {
    let Ok(corpus) = std::env::var("BEDROCK_ADDON_CORPUS") else {
        corpus::skipped("the metadata checkout", corpus::not_set(corpus::CORPUS));
        return None;
    };
    let dir = PathBuf::from(corpus).join("bedrock-samples");
    if dir.is_dir() {
        Some(dir)
    } else {
        corpus::skipped(
            "the metadata checkout",
            format_args!("{} is missing", dir.display()),
        );
        None
    }
}

#[test]
fn the_generator_reproduces_the_committed_files() {
    let Some(dir) = samples_dir() else {
        return;
    };
    let samples = step::read_samples(&dir).unwrap();
    let generated = step::generate(&samples).unwrap_or_else(|errors| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        panic!("the samples tag fails its checks:\n{}", lines.join("\n"))
    });
    for (text, path) in [
        (&generated.versions, step::VERSIONS_OUT),
        (&generated.vanilla_ids, step::VANILLA_IDS_OUT),
        (&generated.commands, step::COMMANDS_OUT),
    ] {
        let formatted = step::format(text).unwrap();
        assert!(
            formatted == committed(path),
            "{path} is stale: run `cargo xtask script-meta --samples <metadata checkout>`"
        );
    }
}

#[test]
fn the_samples_tag_has_no_warning() {
    // Check 3 finds no stub that has declarations from v1.26.30.5 on.
    let Some(dir) = samples_dir() else {
        return;
    };
    let generated = step::generate(&step::read_samples(&dir).unwrap()).unwrap();
    for finding in &generated.report {
        eprintln!("{finding}");
    }
    assert!(
        generated
            .report
            .iter()
            .all(|finding| finding.level == Level::Info),
        "a warning"
    );
    // Check 15: the three engine-only versions of v1.26.50.4.
    let engine_only = generated
        .report
        .iter()
        .filter(|finding| finding.check == Check::EngineOnly)
        .count();
    assert_eq!(engine_only, 3);
}
