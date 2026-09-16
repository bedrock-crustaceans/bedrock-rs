//! Orchestrates one differential-testing run (Phase 7 of the level crate's
//! roadmap): dump a fixture world through this crate, dump it again through
//! whichever oracles the caller has available, and report the diff.
//!
//! Two independent oracle paths, matching what is actually runnable rather
//! than a single shape forced onto both:
//!
//! - **Java oracle.** Its CLI converts worlds; it has no dump mode of its
//!   own. So this harness runs it as a Bedrock -> Bedrock round trip
//!   (`--outputFormat INPUT --keepOriginalNBT`) over a scratch copy of the
//!   fixture, then dumps *this crate's own decode* of both the original and
//!   the round-tripped copy and diffs those two dumps against each other.
//!   A divergence there means the two independent readers (this crate's,
//!   and the oracle's, which produced the round-tripped bytes) disagree
//!   about what the original world contains.
//! - **Python oracle.** Driven by a short script
//!   (`python_dump.py`, next to this file) that walks the world through the
//!   oracle's own public load API and emits the same line format this
//!   module's Rust-side dump produces, so the two dumps diff directly.
//!
//!   **Gated behind `--python-oracle-permission-acknowledged`.** The
//!   recorded clearance for this reference (see this crate's `TODO.md`
//!   References section) covers reading its source and reimplementing what
//!   it teaches; it does not cover *executing* the installed package, whose
//!   own licence ("all rights reserved, a licence must be purchased") does
//!   not carve out an exception for comparison tooling. Running it locally
//!   is on hold pending explicit permission from its authors, which is
//!   being sought separately. This code path stays in the tree (it has
//!   real value once that permission lands, and there is nothing unsafe
//!   about the code itself) but refuses to execute without the flag, so
//!   `cargo xtask diff-oracle --python-bin ...` cannot accidentally run it.
//!
//! Either oracle is optional -- pass only the ones available on the running
//! machine (see the crate root's usage text). With neither, this still
//! writes this crate's own dump, which has standalone value as a fixture
//! snapshot even with nothing to diff it against yet.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bedrock_level::level::Level;

use super::diff::{self, ExpectedDifference, Side};
use super::dump::{self, dump_level};

pub struct OracleConfig {
    pub world_dir: PathBuf,
    pub work_dir: PathBuf,
    pub java_jar: Option<PathBuf>,
    pub python_bin: Option<PathBuf>,
    /// See this module's doc comment: the python oracle refuses to run
    /// without this being explicitly set to `true`.
    pub python_oracle_permission_acknowledged: bool,
}

/// A message printed whenever the python oracle path is reachable at all
/// (whether or not it actually runs), per review: this needs to be visible
/// on every invocation that could have taken it, not only in `oracle.md`.
const PYTHON_ORACLE_NOT_CONTENT_INFORMATIVE: &str = "note: even when it runs, the python oracle comparison is not content-informative today -- \
     it always translates a loaded chunk into its own cross-edition universal namespace before \
     handing it back, so a direct identifier diff against this crate (which never does that \
     translation) differs by namespace prefix on nearly every line. See oracle.md.";

pub fn run(config: &OracleConfig) {
    if let Err(err) = fs::create_dir_all(&config.work_dir) {
        eprintln!(
            "could not create --work-dir {}: {err}",
            config.work_dir.display()
        );
        return;
    }
    clear_stale_dumps(&config.work_dir);

    let Some(level) = open_level(&config.world_dir, "<world-dir>") else {
        return;
    };
    let ours = dump_level(&level);
    write_lines(&config.work_dir.join("ours.txt"), &ours);
    println!(
        "this crate's dump: {} lines ({})",
        ours.len(),
        config.work_dir.join("ours.txt").display()
    );

    if let Some(jar) = &config.java_jar {
        run_java_oracle(config, jar, &ours, &level);
    } else {
        println!("java oracle: skipped (no --java-jar given)");
    }

    if let Some(python) = &config.python_bin {
        println!("{PYTHON_ORACLE_NOT_CONTENT_INFORMATIVE}");
        if config.python_oracle_permission_acknowledged {
            run_python_oracle(config, python, &ours);
        } else {
            println!(
                "python oracle: not run -- local execution is pending explicit permission from \
                 the authors (the orchestrator is raising it with the project owner); pass \
                 --python-oracle-permission-acknowledged once that permission is granted"
            );
        }
    } else {
        println!("python oracle: skipped (no --python-bin given)");
    }
}

/// Opens a world directory through this crate, printing a plain-language
/// Removes whatever this harness may have left in `work_dir` from an
/// earlier invocation, before writing anything new into it. Each dump file
/// is also overwritten in place by the step that produces it, but that
/// alone is not enough: a step that does not run this time (say, the java
/// oracle was passed last time but not now) would otherwise leave its old
/// output sitting next to this run's, which reads as if it were still
/// current. `--work-dir` is always a directory this harness fully owns
/// (never the world directory itself), so clearing its known contents
/// cannot touch anything else.
fn clear_stale_dumps(work_dir: &Path) {
    for name in ["ours.txt", "java-oracle.txt", "python-oracle.txt"] {
        let path = work_dir.join(name);
        if path.exists()
            && let Err(err) = fs::remove_file(&path)
        {
            eprintln!("could not clear stale {}: {err}", path.display());
        }
    }
    let java_out = work_dir.join("java-oracle-out");
    if java_out.exists()
        && let Err(err) = fs::remove_dir_all(&java_out)
    {
        eprintln!("could not clear stale {}: {err}", java_out.display());
    }
}

/// Opens a world directory through this crate, printing a plain-language
/// error naming which flag/argument supplied the bad path rather than
/// panicking on a missing or malformed fixture.
fn open_level(path: &Path, arg_name: &str) -> Option<Level> {
    match Level::open(path) {
        Ok(level) => Some(level),
        Err(err) => {
            eprintln!(
                "could not open {arg_name} `{}` as a Bedrock world: {err} (expected a \
                 directory containing `db/` and `level.dat`)",
                path.display()
            );
            None
        }
    }
}

fn run_java_oracle(config: &OracleConfig, jar: &Path, ours: &[String], ours_level: &Level) {
    // `clear_stale_dumps` (called once, up front in `run`) has already
    // removed any leftover `java-oracle-out` from a previous invocation.
    let out_dir = config.work_dir.join("java-oracle-out");

    let status = match Command::new("java")
        .arg("-jar")
        .arg(jar)
        .arg("-i")
        .arg(&config.world_dir)
        .arg("-f")
        .arg("INPUT")
        .arg("-k")
        .arg("-o")
        .arg(&out_dir)
        .status()
    {
        Ok(status) => status,
        Err(err) => {
            eprintln!(
                "java oracle: could not run `java -jar {}` ({err}); check --java-jar points at \
                 a built jar and `java` is on PATH",
                jar.display()
            );
            return;
        }
    };
    if !status.success() {
        println!("java oracle: conversion failed ({status})");
        return;
    }

    let Some(oracle_level) = open_level(&out_dir, "--java-jar output") else {
        return;
    };
    let java_dump = dump_level(&oracle_level);
    write_lines(&config.work_dir.join("java-oracle.txt"), &java_dump);

    let mut report = diff::diff(ours, &java_dump, java_expected_differences());
    apply_paired_rules(ours_level, &oracle_level, ours, &java_dump, &mut report);
    print_report("java oracle", &report);
}

/// The three position-paired checks a plain substring rule could not make
/// safely narrow (see the doc comment above `java_expected_differences`).
/// Each one re-derives the specific claim its class makes from the two
/// dumps (and, for the jungle_door check, a second decode pass restricted
/// to the one field -- `BlockDef::version` -- that check needs and the
/// main dump deliberately omits) rather than matching on content alone, so
/// a divergence in the same block name or record type that does *not* fit
/// the verified shape is left unexplained instead of being swallowed.
///
/// Runs after the static substring rules in [`java_expected_differences`]
/// and moves anything it can verify from `report.unexplained` into
/// `report.explained`.
fn apply_paired_rules(
    ours_level: &Level,
    oracle_level: &Level,
    ours: &[String],
    java_dump: &[String],
    report: &mut diff::DiffReport,
) {
    use std::collections::{HashMap, HashSet};

    // Position -> content, for BLOCK lines only, one map per side. Used by
    // both the kelp and jungle_door checks to find "what does the other
    // side have at this exact position", which a substring rule has no
    // way to ask at all.
    fn block_by_pos(lines: &[String]) -> HashMap<String, &str> {
        lines
            .iter()
            .filter_map(|line| dump::parse_block_line(line))
            .map(|bl| {
                (
                    dump::position_key(bl.dim, bl.cx, bl.cz, bl.x, bl.y, bl.z, bl.layer),
                    bl.content,
                )
            })
            .collect()
    }
    let ours_by_pos = block_by_pos(ours);
    let java_by_pos = block_by_pos(java_dump);

    // "{dim}\t{cx}\t{cz}\t{subchunk_y}\t{layer}\t{content}" for every real
    // block this crate's own dump has anywhere -- what the palette-orphan
    // check means by "referenced". Built once, from BLOCK lines, the same
    // way a PALETTE line's own subchunk/layer/content triple is built.
    let ours_referenced: HashSet<String> = ours
        .iter()
        .filter_map(|line| dump::parse_block_line(line))
        .map(|bl| {
            let subchunk_y = bl.y.div_euclid(16);
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                bl.dim, bl.cx, bl.cz, subchunk_y, bl.layer, bl.content
            )
        })
        .collect();

    // The jungle_door check's one extra ingredient: `BlockDef::version` at
    // every jungle_door position, on both sides. A second, narrowly-scoped
    // decode pass over the *same already-open* levels (see
    // `dump::state_versions`'s doc comment for why this is not just added
    // to the main dump) -- reopening either world by path here instead
    // would contend with the `Level` the caller already holds open on it
    // for the underlying store's exclusive lock and fail.
    let jungle_door_versions_ours = dump::state_versions(ours_level, &["minecraft:jungle_door"]);
    let jungle_door_versions_oracle =
        dump::state_versions(oracle_level, &["minecraft:jungle_door"]);

    // "{dim}\t{cx}\t{cz}\t{subchunk_y}\t{layer}" for every subchunk/layer
    // whose oracle-side palette contains the normalized kelp_age=25 entry
    // -- what `explain_kelp_palette_pairing` matches an ours-only kelp
    // PALETTE line's own region against.
    let java_kelp25_regions: HashSet<String> = java_dump
        .iter()
        .filter_map(|line| dump::parse_palette_line(line))
        .filter(|pl| pl.content == "minecraft:kelp;kelp_age=Int(25)")
        .map(|pl| {
            format!(
                "{}\t{}\t{}\t{}\t{}",
                pl.dim, pl.cx, pl.cz, pl.subchunk_y, pl.layer
            )
        })
        .collect();

    // "{dim}\t{cx}\t{cz}\t{subchunk_y}\t{layer}" for every subchunk/layer
    // containing a jungle_door position where the version-only decode pass
    // confirmed the verified stamp transition -- what
    // `explain_jungle_door_palette_pairing` matches a jungle_door PALETTE
    // line's own region against. Derived from the position-level version
    // maps rather than "any jungle_door in this region on both sides",
    // which is the name-only shape this whole rewrite exists to avoid.
    const VERIFIED_OLD_VERSION: i32 = 17825808;
    const VERIFIED_NEW_VERSION: i32 = 17876992;
    let verified_jungle_door_regions: HashSet<String> = jungle_door_versions_ours
        .iter()
        .filter(|(key, version)| {
            **version == Some(VERIFIED_OLD_VERSION)
                && jungle_door_versions_oracle.get(key.as_str())
                    == Some(&Some(VERIFIED_NEW_VERSION))
        })
        .filter_map(|(key, _)| {
            let mut fields = key.split('\t');
            let dim = fields.next()?;
            let cx = fields.next()?;
            let cz = fields.next()?;
            fields.next()?; // x
            let y: i32 = fields.next()?.parse().ok()?;
            fields.next()?; // z
            let layer = fields.next()?;
            Some(format!("{dim}\t{cx}\t{cz}\t{}\t{layer}", y.div_euclid(16)))
        })
        .collect();

    let unexplained = std::mem::take(&mut report.unexplained);
    for divergence in unexplained {
        let explanation = explain_kelp_pairing(&divergence, &ours_by_pos, &java_by_pos)
            .or_else(|| explain_kelp_palette_pairing(&divergence, &java_kelp25_regions))
            .or_else(|| {
                explain_jungle_door_pairing(
                    &divergence,
                    &ours_by_pos,
                    &java_by_pos,
                    &jungle_door_versions_ours,
                    &jungle_door_versions_oracle,
                )
            })
            .or_else(|| {
                explain_jungle_door_palette_pairing(&divergence, &verified_jungle_door_regions)
            })
            .or_else(|| explain_palette_orphan(&divergence, &ours_referenced));

        match explanation {
            Some(reason) => report.explained.push((divergence, reason)),
            None => report.unexplained.push(divergence),
        }
    }
}

/// Position-paired half of the kelp_age normalization: explains an
/// ours-only kelp `BLOCK` line only when the oracle has a `BLOCK` line for
/// the exact same position whose content is the normalized
/// `minecraft:kelp;kelp_age=Int(25)` -- the specific, verified
/// substitution (see the value-anchored oracle-side rule in
/// `java_expected_differences` for the corpus-wide evidence). Any other
/// ours-only kelp line -- paired with a non-25 value, or with no oracle
/// line at that position at all -- is left unexplained.
fn explain_kelp_pairing(
    divergence: &diff::Divergence,
    ours_by_pos: &std::collections::HashMap<String, &str>,
    java_by_pos: &std::collections::HashMap<String, &str>,
) -> Option<&'static str> {
    let _ = ours_by_pos; // only the oracle side is looked up for this direction
    if !matches!(divergence.side, Side::Ours) {
        return None;
    }
    let bl = dump::parse_block_line(&divergence.line)?;
    if !bl.content.starts_with("minecraft:kelp;kelp_age=Int(") {
        return None;
    }
    let key = dump::position_key(bl.dim, bl.cx, bl.cz, bl.x, bl.y, bl.z, bl.layer);
    if java_by_pos.get(key.as_str()) == Some(&"minecraft:kelp;kelp_age=Int(25)") {
        Some(
            "position-paired kelp_age normalization: the oracle has a BLOCK line at this exact \
             position reading kelp_age=25, the specific verified substitution (see the \
             value-anchored oracle-side rule in this file for the corpus evidence)",
        )
    } else {
        None
    }
}

/// Region-paired half of the kelp_age normalization for `PALETTE` lines:
/// the same phenomenon [`explain_kelp_pairing`] explains at the `BLOCK`
/// level also empties a subchunk's palette down to the single normalized
/// `kelp_age=25` entry, which turns every *other* kelp_age this crate's
/// own palette for that subchunk still carries (each one genuinely
/// referenced by a real block, so [`explain_palette_orphan`] correctly
/// declines to touch them) into its own PALETTE-only divergence. Explains
/// an ours-only kelp `PALETTE` line only when the oracle's palette for the
/// exact same subchunk and layer contains the normalized entry.
fn explain_kelp_palette_pairing(
    divergence: &diff::Divergence,
    java_kelp25_regions: &std::collections::HashSet<String>,
) -> Option<&'static str> {
    if !matches!(divergence.side, Side::Ours) {
        return None;
    }
    let pl = dump::parse_palette_line(&divergence.line)?;
    if !pl.content.starts_with("minecraft:kelp;kelp_age=Int(") {
        return None;
    }
    let region = format!(
        "{}\t{}\t{}\t{}\t{}",
        pl.dim, pl.cx, pl.cz, pl.subchunk_y, pl.layer
    );
    if java_kelp25_regions.contains(&region) {
        Some(
            "region-paired kelp_age normalization: the oracle's palette for this exact \
             subchunk and layer contains the normalized kelp_age=25 entry, so this is the same \
             verified substitution as the BLOCK-level pairing rule, showing up as a palette \
             divergence in a subchunk whose kelp entries are all genuinely referenced (see \
             the orphan check, which correctly declined to touch this line for that reason)",
        )
    } else {
        None
    }
}

/// Position-paired jungle_door check, both directions: explains a
/// jungle_door `BLOCK` line only when (a) the other side has a jungle_door
/// `BLOCK` line at the exact same position (so the divergence is confined
/// to that block's own properties -- direction/hinge/open bits are the
/// only properties a jungle_door ever carries, so a shared block name at a
/// shared position already confines the diff to them) and (b) a second,
/// version-only decode pass confirms the exact verified stamp transition
/// at that position: `17825808` on this crate's side, `17876992` on the
/// oracle's. A jungle_door divergence missing either condition -- no paired
/// door at that position, or a version pair other than the verified one --
/// surfaces instead of being explained.
fn explain_jungle_door_pairing(
    divergence: &diff::Divergence,
    ours_by_pos: &std::collections::HashMap<String, &str>,
    java_by_pos: &std::collections::HashMap<String, &str>,
    ours_versions: &std::collections::BTreeMap<String, Option<i32>>,
    oracle_versions: &std::collections::BTreeMap<String, Option<i32>>,
) -> Option<&'static str> {
    const VERIFIED_OLD_VERSION: i32 = 17825808;
    const VERIFIED_NEW_VERSION: i32 = 17876992;

    let bl = dump::parse_block_line(&divergence.line)?;
    if !bl.content.starts_with("minecraft:jungle_door;") {
        return None;
    }
    let key = dump::position_key(bl.dim, bl.cx, bl.cz, bl.x, bl.y, bl.z, bl.layer);

    let other_content = match divergence.side {
        Side::Ours => java_by_pos.get(key.as_str()),
        Side::Oracle => ours_by_pos.get(key.as_str()),
    }?;
    if !other_content.starts_with("minecraft:jungle_door;") {
        return None;
    }

    let ours_version = ours_versions.get(&key).copied().flatten();
    let oracle_version = oracle_versions.get(&key).copied().flatten();
    if ours_version == Some(VERIFIED_OLD_VERSION) && oracle_version == Some(VERIFIED_NEW_VERSION) {
        Some(
            "position-paired jungle_door neighbour resolution: both sides have a jungle_door at \
             this exact position, and a version-only decode pass confirms the verified stamp \
             transition (17825808 -> 17876992) at this position specifically",
        )
    } else {
        None
    }
}

/// Region-paired half of the jungle_door check, for `PALETTE` lines: the
/// same neighbour re-resolution [`explain_jungle_door_pairing`] explains
/// at the `BLOCK` level also changes which hinge/direction/open-bit
/// combinations appear in a subchunk's palette at all, so a jungle_door
/// `PALETTE` entry this crate has can go missing from the oracle's palette
/// (or vice versa) even though every jungle_door *position* in that
/// subchunk was already explained. Explains a jungle_door `PALETTE` line
/// only when its own subchunk/layer region is one where the position-level
/// check above actually confirmed the verified version transition for at
/// least one door -- not merely "some jungle_door exists on both sides
/// somewhere in this region", which would be the name-only rule a review
/// already rejected once.
fn explain_jungle_door_palette_pairing(
    divergence: &diff::Divergence,
    verified_jungle_door_regions: &std::collections::HashSet<String>,
) -> Option<&'static str> {
    let pl = dump::parse_palette_line(&divergence.line)?;
    if !pl.content.starts_with("minecraft:jungle_door;") {
        return None;
    }
    let region = format!(
        "{}\t{}\t{}\t{}\t{}",
        pl.dim, pl.cx, pl.cz, pl.subchunk_y, pl.layer
    );
    if verified_jungle_door_regions.contains(&region) {
        Some(
            "region-paired jungle_door neighbour resolution: this subchunk/layer contains at \
             least one jungle_door position where the position-level pairing check already \
             confirmed the verified version-stamp transition, so this palette entry is the \
             same phenomenon showing up as a palette-shape change rather than a per-position one",
        )
    } else {
        None
    }
}

/// Genuine-orphan check for the palette-compaction class: explains an
/// ours-only `PALETTE` line only when no `BLOCK` line anywhere in this
/// crate's own dump, within that same subchunk and layer, resolves to the
/// entry's exact content -- i.e. the entry really is unreferenced, which is
/// the only case the java oracle's compaction (see this file's other
/// palette-related rules) is evidence for. A `PALETTE` line whose content
/// *is* referenced by a real block surfaces instead: something other than
/// compaction is going on, and this rule has nothing to say about what.
fn explain_palette_orphan(
    divergence: &diff::Divergence,
    ours_referenced: &std::collections::HashSet<String>,
) -> Option<&'static str> {
    if !matches!(divergence.side, Side::Ours) {
        return None;
    }
    let pl = dump::parse_palette_line(&divergence.line)?;
    let key = format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        pl.dim, pl.cx, pl.cz, pl.subchunk_y, pl.layer, pl.content
    );
    if ours_referenced.contains(&key) {
        None
    } else {
        Some(
            "verified palette orphan: no BLOCK line anywhere in this crate's own dump, within \
             this same subchunk and layer, resolves to this palette entry's exact content",
        )
    }
}

fn run_python_oracle(config: &OracleConfig, python_bin: &Path, ours: &[String]) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/oracle/python_dump.py");

    let output = match Command::new(python_bin)
        .arg(&script)
        .arg(&config.world_dir)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!(
                "python oracle: could not run `{} {}` ({err}); check --python-bin points at a \
                 venv interpreter with the reference package installed",
                python_bin.display(),
                script.display()
            );
            return;
        }
    };
    if !output.status.success() {
        println!(
            "python oracle: script failed ({})\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut python_dump: Vec<String> = text.lines().map(str::to_owned).collect();
    python_dump.sort();
    python_dump.dedup();
    write_lines(&config.work_dir.join("python-oracle.txt"), &python_dump);

    let report = diff::diff(ours, &python_dump, python_expected_differences());
    print_report("python oracle", &report);
}

/// Prints a summary line, then a per-rule breakdown of what each
/// expected-difference rule explained (a count, not every matching line --
/// a rule can legitimately match millions of lines, and printing them all
/// once cost several gigabytes of log for no benefit over the count), and
/// finally a bounded sample of whatever remains unexplained, which is the
/// actual finding this harness exists to produce.
fn print_report(label: &str, report: &diff::DiffReport) {
    println!(
        "{label}: {} lines ours, {} lines oracle, {} agree, {} explained divergences, {} \
         unexplained divergences",
        report.ours_total,
        report.oracle_total,
        report.agree_total,
        report.explained.len(),
        report.unexplained.len()
    );

    let mut by_reason: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (_, reason) in &report.explained {
        *by_reason.entry(reason).or_default() += 1;
    }
    for (reason, count) in &by_reason {
        println!("  [{count} explained by] {reason}");
    }

    // A `CHUNKERROR` line means the oracle side did not decode that chunk
    // at all -- once that has happened for most or all of a world, every
    // other line in this report (explained or not) has nothing real on
    // the oracle side to compare against: it is not verified agreement or
    // a separate disagreement, it is the mechanical fallout of the one
    // decode failure already named above. Flag that plainly rather than
    // let a huge unexplained (or explained) count read as many findings.
    let oracle_chunkerror_lines = report
        .explained
        .iter()
        .map(|(d, _)| d)
        .chain(&report.unexplained)
        .filter(|d| matches!(d.side, Side::Oracle) && d.line.starts_with("CHUNKERROR\t"))
        .count();
    if report.oracle_total > 0 && oracle_chunkerror_lines * 2 >= report.oracle_total {
        println!(
            "  note: {oracle_chunkerror_lines} of {} oracle-side lines are CHUNKERROR entries \
             -- the oracle-side world did not decode through this crate's own reader for most \
             or all of its chunks, so the counts above are not per-content findings: they are \
             the mechanical consequence of that one decode failure, not separate evidence of \
             agreement or disagreement. Treat this as a single finding, not as {} of them.",
            report.oracle_total,
            report.unexplained.len() + report.explained.len()
        );
    }

    for divergence in report.unexplained.iter().take(200) {
        println!(
            "  [UNEXPLAINED] {} {}",
            side_label(&divergence.side),
            divergence.line
        );
    }
    if report.unexplained.len() > 200 {
        println!(
            "  ... {} more unexplained divergences",
            report.unexplained.len() - 200
        );
    }
}

fn side_label(side: &Side) -> &'static str {
    match side {
        Side::Ours => "ours-only",
        Side::Oracle => "oracle-only",
    }
}

fn write_lines(path: &Path, lines: &[String]) {
    let mut text = lines.join("\n");
    if !lines.is_empty() {
        text.push('\n');
    }
    if let Err(err) = fs::write(path, text) {
        eprintln!("could not write {}: {err}", path.display());
    }
}

/// The Java oracle's own known gaps, and this crate's deliberate choices,
/// as they show up against a Bedrock -> Bedrock round trip through it. Each
/// entry documents *why* the divergence is expected rather than a bug --
/// see this module's doc comment for what "ours" and "oracle" mean in this
/// comparison.
///
/// A raw `CHUNKERROR` line is deliberately **not** matched by a catch-all
/// rule here: a chunk this crate fails to decode is never an expected
/// outcome in general, so the default is to surface it as unexplained,
/// where it is impossible to miss. The one exception below is narrowly
/// keyed to the *specific, root-caused* message this class of divergence
/// actually produces, not to `CHUNKERROR` as a category.
fn java_expected_differences() -> &'static [ExpectedDifference] {
    &[
        ExpectedDifference {
            side: Side::Oracle,
            contains: "unknown field `val`",
            reason: "confirmed root cause, 1.12 fixture: the java oracle's Bedrock writer, even \
                      under --keepOriginalNBT, re-encodes a legacy-sourced subchunk's palette \
                      entries into a hybrid shape -- `{name, val, states: {data}}` -- that \
                      mixes the pre-flattening `val` field with a `states` compound; no real \
                      game version ever writes that combination (pre-flattening writes `{name, \
                      val}` with no `states`; post-flattening writes `{name, version, states}` \
                      with no `val`, see `subchunk.rs`'s `PaletteEntry`/`BlockDef` doc comments \
                      for the two real shapes). This crate's decoder correctly rejects the \
                      hybrid as unmodelled (decision 11): every one of the fixture's 1118 \
                      chunks reproduced this exact error, and zero decoded from the original.",
        },
        ExpectedDifference {
            side: Side::Oracle,
            contains: "kelp_age=Int(25)",
            reason: "confirmed root cause, 1.16 fixture: the java oracle bumps a block's state \
                      version stamp on every round trip regardless of content (observed on \
                      plain `stone`, content-identical either side: 17825808 -> 17876992), and \
                      for `minecraft:kelp` specifically also normalizes `kelp_age` to its \
                      maximum (25) rather than preserving the stored growth stage -- verified \
                      against a chunk (-6,24) column whose original ages run 11..24 \
                      consecutively and whose oracle-side output is 25 at every one of those \
                      positions.",
        },
        ExpectedDifference {
            side: Side::Ours,
            contains: "\t1\tminecraft:air",
            reason: "confirmed root cause, 1.16 fixture, chunk (-4,2) subchunk 0: the liquid \
                      layer at this subchunk is real, on-disk data (a present, non-empty `0x2f` \
                      liquid layer record, uniformly air) that the java oracle's writer drops \
                      entirely on this round trip rather than re-emitting a trivial layer -- \
                      this crate correctly preserves it (decision 7). Matches both the BLOCK \
                      and PALETTE line shapes for a layer-1 (liquid) entry whose content is \
                      exactly `minecraft:air` with no properties.",
        },
        ExpectedDifference {
            side: Side::Ours,
            contains: "\t1\tminecraft:flowing_water;liquid_depth=Int(0)",
            reason: "confirmed root cause, 1.16 fixture, chunk (-3,1) and (1,7), both under a \
                      `minecraft:bubble_column`: the original stores `flowing_water` on the \
                      liquid layer beneath the bubble column; the oracle's round trip rewrites \
                      it to still `minecraft:water` at the same position (see the paired \
                      oracle-side rule below). This is a substitution, not a drop -- distinct \
                      from the trivial-air-liquid-layer rule above, which is a genuine omission.",
        },
        ExpectedDifference {
            side: Side::Oracle,
            contains: "\t1\tminecraft:water;liquid_depth=Int(0)",
            reason: "paired half of the bubble-column flowing_water->water substitution above: \
                      an oracle-only water-layer line only exists where this crate's line at \
                      the same position differs, and every instance found in the 1.16 fixture \
                      (3 positions, all under a bubble column) was the paired flowing_water \
                      case. Residual risk, stated plainly, same shape as the kelp pairing rule \
                      above: keyed on content, not position, so an unrelated genuine still-water \
                      divergence elsewhere would also be silently explained by this rule.",
        },
    ]
}

// The three classes whose static substring rule was too broad (a review
// found each would also explain away a *future*, unrelated bug in the
// same block name or record type): kelp-age normalization, jungle_door
// neighbour resolution, and palette compaction. All three now go through
// `apply_paired_rules` instead, which checks the actual claim -- same
// position, the specific paired value or a verified version transition,
// or genuine non-reference within the subchunk -- rather than matching on
// content alone. See that function's doc comment for each check.

/// The Python oracle's own known architectural difference, and this
/// crate's deliberate choices, as they show up against its own dump.
///
/// As with the Java rules above, a bare `CHUNKERROR` is not matched by a
/// catch-all: it always surfaces as unexplained by default.
fn python_expected_differences() -> &'static [ExpectedDifference] {
    &[ExpectedDifference {
        side: Side::Oracle,
        contains: "universal_minecraft:",
        reason: "the python oracle always translates a loaded chunk into its own cross-edition \
                  universal namespace before handing it back, even for a Bedrock-only \
                  comparison; this crate never does that translation (decision 1), so a \
                  block/biome identifier line is expected to differ by namespace prefix alone \
                  until this harness grows a universal-to-Bedrock reverse mapping for the \
                  comparison step specifically",
    }]
}
