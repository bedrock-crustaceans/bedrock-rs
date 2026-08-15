# Test-world fixtures

Thirteen Bedrock LevelDB worlds spanning storage version 8 through 10, chunk versions 15
through 40, and every biome/height-map format the game has shipped since 1.12. Each fixture is
a gzipped tarball that unpacks to `world/db/` and `world/level.dat`, matching the layout
`../level.tar.gz` already uses. `index.json` is the machine-readable catalogue tests select
fixtures from; see `crates/level/tests/support/` for the harness that reads it.

**The `db/` files inside each tarball are byte-identical to what the game itself wrote.** They
were copied straight out of the upstream zip/directory (unzip, or a plain recursive file copy)
into a tarball, with nothing in between reading them as a database: no LevelDB open, WAL
replay, or manifest rewrite ever touches a fixture before it ships. The *only* files left out
are the advisory `LOCK` file and the `LOG`/`LOG.old` diagnostic text logs -- pure runtime
artifacts the game itself regenerates on open and nothing ever reads back. Every numbered WAL
log (`000004.log` and so on) and the original `MANIFEST-*`/`CURRENT` ship exactly as upstream
wrote them, unreplayed and unrenumbered. This was verified by hashing every shipped file
against its upstream counterpart; all matched (see the import task's report for the check).

Chunk versions, subchunk versions, and per-dimension height ranges in `index.json` came from
extracting each already-built tarball into a temporary directory and scanning *that disposable
copy* with this crate's own `Database`, reading its keys for the `0x2c`/`0x76` chunk-version
records and `0x2f` subchunk records -- not from any upstream project's own documentation, and
never from a source world opened in place. All thirteen opened and scanned without error.

**Negative-Y note:** `v1_17_20_caves_and_cliffs` (saved Overworld range 0..176) and `v1_18_30`
(0..32) are built on a chunk/storage version that nominally supports negative-Y, but neither
fixture's saved area actually dips below y=0 -- `index.json`'s `negative_y: false` for both is
correct, reflecting what's really on disk rather than what the format allows in principle.
Tests that need a real negative-Y fixture should select `v1_17_40_caves_and_cliffs`, `v1_18`,
`v1_18_superflat`, or `v1_20_81` instead.

## Origin and permission

### Chunker (`v1_12`, `v1_18_30`, `v1_19_30`, `v1_20_81`)

Copied from Chunker's integration-test fixtures at
`cli/src/test/resources/integration/worlds/BEDROCK_R12.zip`, `BEDROCK_R18_30.zip`,
`BEDROCK_R19_30.zip`, and `BEDROCK_R20_80.zip`. Chunker is MIT licensed:

```
MIT License

Copyright (c) 2024 Hive Games

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

The MIT grant covers Chunker's own code; these world saves are game-generated content the
project did not author, so beyond the licence, Hive Games have given explicit permission to
copy these four world files into this crate's test suite.

### Amulet-Core (`v1_16`, `v1_17_20`, `v1_17_20_caves_and_cliffs`, `v1_17_40`,
### `v1_17_40_superflat`, `v1_17_40_caves_and_cliffs`, `v1_18`, `v1_18_superflat`,
### `v1_18_updated_superflat`)

Copied from Amulet-Core's test worlds at
`tests/data/worlds_src/bedrock/vanilla/{1_16, 1_17_20/vanilla, 1_17_20/caves_and_cliffs,
1_17_40/vanilla, 1_17_40/superflat, 1_17_40/caves_and_cliffs, 1_18/vanilla, 1_18/superflat,
1_18/updated_superflat}`. **Amulet-Core is not open source.** Its `LICENSE` reads, in full:

```
Copyright (c) 2025 Amulet-Team

All rights reserved.

A licence must be purchased to use this software.
See https://www.amuletmc.com/ for more details.
```

Amulet-Team have given explicit permission for these nine test worlds specifically. That
permission does not extend to Amulet-Core's code or its data tables, which stay off-limits;
only these world files were copied under the grant.

Each of these nine worlds ships with a `world_test_data.json` sidecar recording its declared
version and per-dimension height range. Those sidecars are preserved verbatim in `index.json`
under each fixture's `sidecar` field rather than as separate files in the tarballs.

## `v1_18` is imported whole, not trimmed

`v1_18` (Amulet-Core's `1_18/vanilla`) is 16 MiB raw and the largest fixture by a wide margin —
larger than the other twelve combined (~11 MiB raw). As a gzipped tarball it comes to about
15 MiB: LevelDB's table files are already internally deflate-compressed, so there was little
left for gzip to take out, as expected going in.

It was imported whole rather than trimmed. Trimming would mean deleting chunk keys outside a
small region through a real LevelDB write path and compacting the result, across three
dimensions' worth of chunk, subchunk, biome, block-entity, entity, and digest keys — a
nontrivial, error-prone operation, and a hand-damaged world is worse than a big one. `v1_18` is
also the only full-size, real-terrain 1.18 fixture in the set (`v1_18_superflat` and
`v1_18_updated_superflat` are both small, flat, and not representative of ordinary generated
terrain), so it carries unique value the smaller 1.18 fixtures don't. Given the modest actual
savings on offer and the risk of shipping a subtly-broken fixture, it stays whole.
