//! Vanilla game-content knowledge for Bedrock worlds: concrete block, item, and biome
//! data that `bedrock_level` deliberately does not contain.
//!
//! `bedrock_level` parses the on-disk format only. Anything that requires knowing what
//! a specific block, item, or biome *is* — the block-entity payload shapes, numeric-id
//! tables, block-state schemas — lives here instead, and is handed back to
//! `bedrock_level` through the `bedrock_level::provider::ContentProvider` trait.

pub mod biome;
pub mod block_entities;
pub mod diff;
pub mod palette;

#[cfg(feature = "generated-palettes")]
pub mod generated;

use bedrock_level::provider::{ContentProvider, StateUpgrade, VersionedBlockDef};
#[cfg(feature = "generated-palettes")]
use bedrock_level::provider::{MalformedState, States};
use bedrock_level::subchunk::BlockDef;
use bedrock_level::subchunk::PaletteEntry;

/// A `ContentProvider` backed by this crate's vanilla content tables.
///
/// [`legacy_block_state`](ContentProvider::legacy_block_state) and
/// [`numeric_block_state`](ContentProvider::numeric_block_state) always answer `None`:
/// the generated tables (architecture decision 12) are built from the game's own
/// published block-palette dumps, and those dumps only cover the post-flattening,
/// name-plus-properties era (the earliest upstream release that publishes one is well
/// after Bedrock's `nbt-blockstate` format began) — there is no official dump of a
/// pre-flattening `(name, val)` or `(numeric id, data)` table at all, at any version,
/// so this is not a gap the generator can close by running again. Closing it needs a
/// hand-authored supplement layered over the generated data, per architecture decision
/// 12's fallback clause; nothing here provides one yet.
///
/// [`upgrade_block_state`](ContentProvider::upgrade_block_state) walks the pinned
/// version ladder (`palette::PALETTES`, oldest-to-newest) forward from `source` to
/// `target`, one adjacent-pair step at a time, using each step's generated
/// [`diff::VersionDiff`] (`generated::DIFFS`):
///
/// 1. **The incoming `state` is validated against its own claimed `source` stamp
///    first**, exactly like the single-version check this method used to be limited to:
///    the block name must be declared at `source`, every property key `state.states`
///    carries must be declared on that block there, and every value must fall inside
///    that property's declared domain. A miss here is
///    [`StateUpgrade::Malformed`] — the entry does not match what its own stamp
///    promises (architecture decision 11) — not [`StateUpgrade::NoEntry`], and it is the
///    *only* place this method returns `Malformed`; see point 3 below for why a failure
///    further into the walk is not treated the same way.
/// 2. **Each step forward** resolves the current name through that step's
///    [`diff::VersionDiff::resolve_name`] (a confirmed rename, or the name unchanged),
///    looks it up in the next version's palette, and — if found — carries forward only
///    the properties that version's palette still declares for it, each revalidated
///    against that version's domain; a property the target no longer declares is
///    dropped, not carried (see point 4).
/// 3. **A step that cannot be taken — the resolved name is not declared at the next
///    version, or a still-declared property's value no longer fits that version's
///    domain — simply is not taken.** The walk stops there rather than either pressing
///    on with data that would misrepresent what that version accepts, or reporting
///    `Malformed`: nothing about `state` was ever inconsistent with its *own* stamp, the
///    schema data available from this version onward just does not cover taking it
///    further. That is exactly [`StateUpgrade::Upgraded`]'s "may fall short of the
///    requested target" case (architecture decision 6's honest achieved-version
///    report): the caller gets back the furthest version this method could actually
///    justify, stamped as such, never `target` stamped over a shortfall.
/// 4. **A property declared at one step but not the next is dropped, never carried
///    past that step.** The alternative — keep it anyway — would mean the returned
///    entry's `states` disagrees with what its own (correctly reported) achieved-version
///    stamp says the block accepts, which is worse than the data loss: a stamp that
///    overclaims is exactly what decision 6 exists to prevent, and unlike an unmodelled
///    *record* (decision 7, which this does not touch — record preservation is a
///    level-crate concern), a property the schema no longer declares for this block is a
///    positively known fact from the generated diff, not something this method merely
///    has not gotten round to modelling yet.
/// 5. **If the very first step cannot be taken** — the walk makes no progress past
///    `source` at all — the result is [`StateUpgrade::NoEntry`], matching this trait's
///    existing "no rule starting from this entry's source version at all" contract,
///    including the pre-existing behaviour for `source == target` with an unknown block.
/// 6. **A rename never invents an identity.** `resolve_name` only ever substitutes a
///    name the hand-authored supplement (`xtask/dumps/bedrock-samples/renames.json`)
///    confirms — empty today, so every step's walk currently resolves under the entry's
///    own unchanged name; see `diff::VersionDiff`'s module doc for why a coincidental
///    removed/added pair is never inferred as one.
///
/// `source`/`target` outside the pinned ladder (no generated palette matches that exact
/// `(major, minor, patch)`), or `target` behind `source`, answer `NoEntry` — this method
/// only ever upgrades forward across versions this crate actually has schema data for.
///
/// Without the `generated-palettes` feature, every lookup here honestly reports
/// "unknown" — the same "no provider entry" outcome architecture decision 9 describes,
/// which is the correct behaviour with no tables compiled in, not an approximation of
/// it.
#[derive(Debug, Default, Clone, Copy)]
pub struct VanillaContent;

impl ContentProvider for VanillaContent {
    fn legacy_block_state(&self, _name: &str, _val: i16) -> Option<VersionedBlockDef> {
        None
    }

    fn numeric_block_state(&self, _id: u8, _data: u8) -> Option<VersionedBlockDef> {
        None
    }

    #[cfg(feature = "generated-palettes")]
    fn upgrade_block_state(&self, state: BlockDef, source: i32, target: i32) -> StateUpgrade {
        let palettes = generated::PALETTES;
        let source_version = unpack_version(source);
        let target_version = unpack_version(target);

        let (Some(source_idx), Some(target_idx)) = (
            ladder_index(palettes, source_version),
            ladder_index(palettes, target_version),
        ) else {
            return StateUpgrade::NoEntry(state);
        };
        if target_idx < source_idx {
            // This method only ever upgrades forward; a caller asking to go backward has
            // no rule to apply, the same as any other version this generator has no data
            // for.
            return StateUpgrade::NoEntry(state);
        }

        // Resolve indices done; validate the incoming state against its own claimed
        // source stamp next. A miss here is `Malformed` -- see this impl's doc comment
        // for why this is the only point that outcome is reachable from.
        let source_palette = &palettes[source_idx];
        let Some(source_block) = source_palette.block(&state.name) else {
            return StateUpgrade::NoEntry(state);
        };
        if let Err(reason) = validate_declared_properties(
            source_palette,
            source_block,
            &state.states,
            source_version,
        ) {
            return StateUpgrade::Malformed(state, reason);
        }

        if source_idx == target_idx {
            return StateUpgrade::Upgraded(VersionedBlockDef {
                block: state,
                version: target,
            });
        }

        // Walk forward one adjacent pair at a time: each step drops what the next
        // version no longer declares (point 4 on this impl's doc comment) and stops the
        // instant it cannot be taken (point 3).
        let mut current_name = state.name.clone();
        let mut current_states = state.states.clone();
        let mut achieved_idx = source_idx;

        for step in source_idx..target_idx {
            let diff = &generated::DIFFS[step];
            let to_palette = &palettes[step + 1];
            let Some((next_name, next_states)) =
                step_forward(diff, to_palette, &current_name, &current_states)
            else {
                break;
            };
            current_name = next_name;
            current_states = next_states;
            achieved_idx = step + 1;
        }

        if achieved_idx == source_idx {
            StateUpgrade::NoEntry(state)
        } else {
            let achieved_version = palettes[achieved_idx].game_version;
            let achieved_packed = BlockDef::pack_version([
                achieved_version.0,
                achieved_version.1,
                achieved_version.2,
                0,
            ]);
            StateUpgrade::Upgraded(VersionedBlockDef {
                block: BlockDef {
                    name: current_name,
                    version: None,
                    states: current_states,
                },
                version: achieved_packed,
            })
        }
    }

    #[cfg(not(feature = "generated-palettes"))]
    fn upgrade_block_state(&self, state: BlockDef, _source: i32, _target: i32) -> StateUpgrade {
        StateUpgrade::NoEntry(state)
    }

    fn biome_name(&self, id: i32) -> Option<&str> {
        biome_name_impl(id)
    }

    /// Classifies a [`PaletteEntry::Modern`] entry by its name against
    /// [`AIR_BLOCK_NAME`], so this is not a guess tied to one version: whichever schema
    /// version the entry is actually stamped at, the name comparison alone settles it. A
    /// `Legacy`/`Numeric` entry always answers `None`: the corpus has exactly one
    /// pre-flattening fixture (`v1_12`), so any `(name, val)`/`(id, data)` pair this crate
    /// could hardcode as "air" would be single-sourced against the same fixture that
    /// source-less legacy table already has no second reference for (see this type's
    /// crate-root doc comment) -- see `bedrock_vanilla_data`'s `tests/air_query.rs` for
    /// the survey that finding is based on.
    fn is_air(&self, entry: &PaletteEntry) -> Option<bool> {
        match entry {
            PaletteEntry::Modern(block) => Some(block.name == AIR_BLOCK_NAME),
            PaletteEntry::Legacy { .. } | PaletteEntry::Numeric { .. } => None,
        }
    }
}

/// The vanilla air block's modern name, confirmed present in every one of the eight pinned
/// generated tags (`v1.20.10.1` through `v1.26.60.5` -- see `generated::PALETTES`).
/// [`VanillaContent::is_air`] classifies a [`PaletteEntry::Modern`] entry by comparing
/// its name against this constant, per architecture decisions 8/9: this crate parses
/// content by consulting the generated tables, never by hardcoding what a block *means*
/// beyond a name this constant makes explicit and reviewable in one place.
const AIR_BLOCK_NAME: &str = "minecraft:air";

/// Splits a packed `(major << 24) | (minor << 16) | (patch << 8) | revision` word into
/// the `(major, minor, patch)` triple a [`palette::VersionPalette::game_version`] is
/// keyed on, discarding the revision byte -- the same split `source` and `target` both
/// need in [`ContentProvider::upgrade_block_state`].
#[cfg(feature = "generated-palettes")]
fn unpack_version(packed: i32) -> (u8, u8, u8) {
    let [major, minor, patch, _revision] = packed.to_be_bytes();
    (major, minor, patch)
}

/// The index into `palettes` (and, one less than the ladder's length, into
/// `generated::DIFFS`) whose [`palette::VersionPalette::game_version`] matches `version`
/// exactly, or `None` if this crate has no generated table for it.
#[cfg(feature = "generated-palettes")]
fn ladder_index(palettes: &[palette::VersionPalette], version: (u8, u8, u8)) -> Option<usize> {
    palettes.iter().position(|p| p.game_version == version)
}

/// Takes one adjacent-pair step of [`ContentProvider::upgrade_block_state`]'s walk:
/// resolves `name` through `diff`'s confirmed renames, looks the resolved name up in
/// `to_palette`, and -- if found -- carries forward only the properties `to_palette`
/// still declares for it, each revalidated against `to_palette`'s domain.
///
/// Returns `None` when this step cannot be taken at all: the resolved name is not
/// declared at `to_palette`, or a still-declared property's value no longer fits its
/// domain there. The caller stops the walk on `None` rather than treating it as an
/// error -- see [`VanillaContent`]'s doc comment, points 3 and 4, for why a stalled step
/// is not the same outcome as a malformed source state.
#[cfg(feature = "generated-palettes")]
fn step_forward(
    diff: &diff::VersionDiff,
    to_palette: &palette::VersionPalette,
    name: &str,
    states: &States,
) -> Option<(String, States)> {
    let resolved_name = diff.resolve_name(name).to_string();
    let to_block = to_palette.block(&resolved_name)?;

    let mut new_states = States::with_capacity(states.len());
    for (key, value) in states {
        if !to_block.properties.contains(&key.as_str()) {
            // Removed from the target's declared set: dropped, not carried -- see
            // `VanillaContent`'s doc comment, point 4.
            continue;
        }
        // `key` is declared, so this lookup always hits -- same guarantee
        // `validate_declared_properties` relies on.
        let spec = to_palette
            .property(key)
            .expect("a block-declared property is always in its own version's property table");
        if !spec.kind.accepts(value) {
            // The domain narrowed past this value between the last achieved version and
            // this one: this step cannot be taken (point 3), not a reason to call the
            // whole entry malformed.
            return None;
        }
        new_states.insert(key.clone(), value.clone());
    }

    Some((resolved_name, new_states))
}

/// Confirms every property `state` carries is declared on `block` (from `palette`, the
/// palette for `version`), with a value inside that property's declared domain. Shared by
/// both the `source == target` fast path and the walk's initial validation step.
#[cfg(feature = "generated-palettes")]
fn validate_declared_properties(
    palette: &palette::VersionPalette,
    block: &palette::BlockSpec,
    states: &States,
    version: (u8, u8, u8),
) -> Result<(), MalformedState> {
    let (major, minor, patch) = version;
    for (key, value) in states {
        if !block.properties.contains(&key.as_str()) {
            return Err(MalformedState(format!(
                "`{}` does not declare a `{key}` property at version {major}.{minor}.{patch}",
                block.name
            )));
        }
        let spec = palette
            .property(key)
            .expect("a block-declared property is always in the version's property table");
        if !spec.kind.accepts(value) {
            return Err(MalformedState(format!(
                "`{}`'s `{key}` value does not match its declared domain at version \
                 {major}.{minor}.{patch}",
                block.name
            )));
        }
    }
    Ok(())
}

#[cfg(feature = "generated-palettes")]
fn biome_name_impl(id: i32) -> Option<&'static str> {
    // Every generated biome table that carries this id; not narrowed to one version,
    // because `ContentProvider::biome_name`'s signature takes no version -- see this
    // trait method's doc comment on the level-crate side for why (a purely
    // presentation-layer lookup, keyed on the id alone). Coverage only begins at
    // `v1.21.130.3` (biome.rs's module doc), so an id from an in-range table resolves
    // even when called without version context; an id no pinned table has ever assigned
    // answers `None`, honestly, same as an id no table has ever heard of at all.
    generated::BIOMES.iter().find_map(|table| table.name(id))
}

#[cfg(not(feature = "generated-palettes"))]
fn biome_name_impl(_id: i32) -> Option<&'static str> {
    None
}
