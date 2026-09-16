# Differential testing against reference implementations

`cargo xtask diff-oracle` is the Phase 7 standing harness: it dumps a
fixture world through this crate and, for whichever oracles are available
on the machine, diffs that dump against the oracle's own understanding of
the same world. It never runs as part of `cargo test` -- both oracles are
external tools this repository does not vendor or depend on, so the
harness is a manual `xtask` subcommand rather than an `#[ignore]` test:
there is orchestration here (building/locating a jar, activating a venv,
shelling out, comparing three different outputs) that reads more naturally
as a small CLI tool a developer runs on demand than as a test case, and
keeping it out of the `bedrock_level`/`xtask` test binaries entirely (not
even behind `#[ignore]`) means `cargo test --workspace` never needs `java`
or `python3` on `PATH` to compile.

## What it does

For a world directory:

1. Opens it through this crate's ordinary `Level` API and writes a sorted,
   deduplicated text dump (`xtask/src/oracle/dump.rs`'s line format --
   documented in that file) to `<work-dir>/ours.txt`.
2. If `--java-jar` is given, runs that jar as a Bedrock -> Bedrock round
   trip (`-f INPUT -k`, i.e. convert to the detected input format and keep
   original NBT) into `<work-dir>/java-oracle-out`, then dumps *that*
   output through this crate's own decode and diffs it against step 1's
   dump. A divergence here means this crate's decode of the original bytes
   disagrees with what the same crate decodes from bytes the java oracle
   itself produced after reading and re-writing them -- i.e. the two
   independent readers disagree about the original world's content.
3. If `--python-bin` is given **and** `--python-oracle-permission-acknowledged`
   is also given, runs `xtask/src/oracle/python_dump.py` (a short script
   against the python oracle's own public load API) over the original world
   directory and diffs its output directly against step 1's dump. See the
   blocker below -- without the second flag this step does not run.

> **BLOCKER: the python oracle path does not run without explicit
> permission, which has not been granted yet.** The installed `amulet-core`
> and `pymctranslate` packages are both "all rights reserved, a licence
> must be purchased to use this software." The clearance recorded in this
> crate's `TODO.md` References section covers *reading* their source and
> *reimplementing* what they teach independently -- it does not say
> anything about *executing* the installed packages, and their licence text
> carves out no exception for comparison tooling. Local execution is
> therefore on hold pending explicit permission from their authors, which
> the orchestrator is raising with the project owner separately. Until that
> permission lands, `cargo xtask diff-oracle --python-bin ...` prints a
> message saying so and does not invoke the interpreter; pass
> `--python-oracle-permission-acknowledged` only once that permission is
> actually granted, not as a way to silence the message. Do not resolve
> this yourself by deciding the clearance is broad enough -- it is an open
> question for the authors, not an engineering judgment call.

Every diff is filtered through an explicit, documented expected-difference
set (`java_expected_differences`/`python_expected_differences` in
`xtask/src/oracle/run.rs`) before being reported as `[UNEXPLAINED]` --
see that file for what is expected and why.

## Running it

Build the java oracle once (writes only to its own `build/` directory,
never touched by this harness otherwise):

```sh
(cd /path/to/the/java-converter-checkout && ./gradlew :cli:shadowJar)
```

Set up a scratch virtualenv once for the python oracle:

```sh
python3 -m venv /path/to/scratch/venv
/path/to/scratch/venv/bin/pip install amulet-core
```

Then, with a **copy** of a fixture world (never point this at a reference
checkout's own test-world directory -- see the ground rules in this
crate's `TODO.md`):

```sh
cargo xtask diff-oracle /path/to/scratch/copy-of-world \
  --work-dir target/diff-oracle \
  --java-jar /path/to/the/java-converter-checkout/cli/build/libs/<name>.jar \
  --python-bin /path/to/scratch/venv/bin/python3
  # --python-oracle-permission-acknowledged intentionally omitted -- see the blocker above
```

Either `--java-jar` or `--python-bin` can be omitted; the harness reports
which oracle it skipped and still writes this crate's own dump, which has
standalone value as a fixture snapshot even with nothing to diff it
against. `--python-bin` without `--python-oracle-permission-acknowledged`
prints the blocker message and skips execution rather than running.

## Known harness limitations (not format facts)

- **Paletted 3D biome fragments are reported structurally, not
  per-voxel.** A fragment's per-voxel indices are not reachable through
  the public API this harness restricts itself to; only the fragment's
  palette contents are. See `xtask/src/oracle/dump.rs`'s module doc.
- **The `PALETTE` line includes the palette's raw index.** Two decodes
  that agree on every block's *content* can still disagree on `PALETTE`
  lines alone if the two palettes list the same entries in a different
  order (a real possibility whenever either side rebuilds a palette from
  its own in-memory block-placement order rather than preserving the
  on-disk one). Treat `PALETTE`-only divergences as lower-confidence than
  `BLOCK` ones for exactly this reason, and check `BLOCK` first when
  triaging.
- **The python oracle's block/biome identifiers carry a
  `universal_minecraft:` namespace prefix**, because it always translates
  a loaded chunk into its own cross-edition intermediate representation on
  load, even for a Bedrock-only comparison. This crate has no such
  translation and never will (architecture decision 1), so a direct
  string diff against it is not informative at the identifier level today
  -- the expected-difference set explains away the prefix, but does not
  (yet) reverse-map the rest of the identifier or its properties' NBT-tag
  formatting, so a real content-level disagreement with the python oracle
  cannot currently be told apart from a formatting one. Closing that gap
  needs a small identifier/property canonicalizer in `python_dump.py` and
  is future work, not part of this pass.
