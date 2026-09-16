# Palette-dump coverage

Source: `github.com/Mojang/bedrock-samples`, path `metadata/vanilladata_modules/mojang-blocks.json`,
one file per tagged release. Surveyed via the GitHub API (tag list, commit history filtered to
that path, and the git tree at specific tags) rather than assumed.

## The coverage floor is `v1.20.10.1`, not the repository's own start

`bedrock-samples`' oldest tag is `v1.19.30`. That release, and every tag before `v1.20.10.1`,
carries **no `metadata` directory at all** -- only the `documentation/*.html` reference pages,
`behavior_pack/`, and `resource_pack/`. `metadata/vanilladata_modules/` first appears at
`v1.20.10.1` (2023-07-11); confirmed directly by fetching both trees (`v1.20.0.1`: zero
`metadata` entries; `v1.20.10.1`: 23 entries under `metadata/`, including
`mojang-blocks.json`), not inferred from the commit history alone.

Consequence for this generator: it can cover the block-state schema from `1.20.10` onward and
**cannot cover anything earlier**, including every pre-flattening version this crate otherwise
aims to read (architecture decision 12's "widest possible version range" goal, and `bedrock_
level`'s own numeric/legacy palette forms). There is no official dump of a `(name, val)` or
`(numeric id, data)` table at any version -- the flattening update predates this repository's
metadata coverage entirely (Bedrock's `nbt-blockstate` era begins at 1.13.0; the dump begins
at 1.20.10). Closing that gap is not "generate further back," because there is nothing further
back to generate from; it needs a hand-authored supplement, per architecture decision 12's
fallback clause.

## What the dump contains, precisely

Each `mojang-blocks.json` is two tables, not a states-with-defaults palette:

- `block_properties`: every property name the game defines, its type (`bool` | `int` |
  `string`), and its full value domain (e.g. `attachment`: `string`, `["standing", "hanging",
  "side", "multiple"]`).
- `data_items`: every block name plus the property *names* it accepts, referencing
  `block_properties` by name (e.g. `minecraft:acacia_stairs` accepts `upside_down_bit` and
  `weirdo_direction`). Later dumps additionally carry `raw_id` (a numeric palette id) and
  `serialization_id` (an internal string tag); neither is consumed by the generator yet.

**No default value for any property, on any block, at any version, appears anywhere in the
file.** This was checked directly, not assumed from a schema description: every `data_items`
entry names properties only, never a value. A generator built from this data alone can
therefore answer "does this block, with this property set, exist at this version" -- it cannot
answer "what is this block's default state," because the dump does not record one. Filling
that in would be fabrication (architecture decision 11), not generation.

`minecraft_version` (top-level) is present and meaningful on early dumps (`"1.20.10-beta.0"`)
but `null` from somewhere in the `1.21.100`-`1.21.130` range onward -- not a dependable source
for which release a dump belongs to either way, since the file does not know which tag it was
read from. The manifest records the release tag and the `(major, minor, patch)` triple
alongside each vendored file instead of trusting the file's own content for either.

## Pinned subset, not the full ladder

As of the pin date in `manifest.json`, 26 stable (non-`-preview`) tags carry the dump,
`v1.20.10.1` through `v1.26.40.05`. This manifest pins six of them, spread across that range,
to keep the generated tree a reviewable size while still exercising the generator against real
data end to end: `v1.20.10.1` (the floor), `v1.21.0.3`, `v1.21.60.10`, `v1.21.130.3`,
`v1.26.0.2`, `v1.26.40.05` (the tip at pin time). Extending coverage to the rest of the ladder
is adding entries to `manifest.json` with the same fields and re-running
`cargo xtask generate-palette`; nothing about the generator is specific to which versions are
present.

One finding from the six that are pinned: `v1.21.130.3` and `v1.26.0.2` are byte-identical
(same git blob, same sha256) despite the jump in version numbering between them -- no block
changed between those two releases. That is corroborating evidence, not an assumption, for
generated Rust being the right shape here: most of the table does not change release to
release, which is exactly what makes per-version diffs worth keeping readable.

## Biome dump coverage

Surveyed separately from the block dump, at the same six tags this manifest already pins for
`mojang-blocks.json`, via the GitHub API tree listing for `metadata/vanilladata_modules` at
each tag:

- `v1.20.10.1`: no `mojang-biomes.json` at all -- the directory only carries
  `mojang-blocks.json`, `mojang-dimensions.json`, `mojang-effects.json`,
  `mojang-enchantments.json`, `mojang-entities.json`, `mojang-items.json`.
- `v1.21.0.3` and `v1.21.60.10`: `mojang-biomes.json` exists (`{"data_items": [{"name": ...}]}`),
  but every entry is name-only -- no numeric id anywhere in the file.
- `v1.21.130.3`, `v1.26.0.2`, `v1.26.40.05`: every entry carries a numeric `id` alongside
  `name` (e.g. `{"id": 48, "name": "minecraft:bamboo_jungle"}`). `v1.21.130.3` and `v1.26.0.2`
  are byte-identical, the same pairing already noted above for the block dump at those two
  tags.

Consequence: `ContentProvider::biome_name` resolves a numeric id to a name, so only the three
tags whose dump carries a numeric id are useful input, and only those three are vendored
(`manifest.json`'s `biome_entries`). The two name-only tags are recorded here as a fact about
upstream coverage, not vendored, since there is nothing an id-keyed table could be built from
in them.

## Licensing

The vendored files under this directory are Mojang's own published data, distributed from
`Mojang/bedrock-samples` under that repository's EULA-based terms -- not an open-source
licence. They are vendored here as pinned build inputs, the same way the rest of this
project treats provenance: each file's source tag and content hash is recorded in
`manifest.json` rather than the file being taken on trust, so what is checked in is
traceable back to exactly what Mojang published and when.
