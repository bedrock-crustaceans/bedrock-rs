//! The resolver of vanilla's shared client-entity / attachable identifier map: the
//! `EngineVersionMerge` combine rule, keyed on `description.min_engine_version`.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use indexmap::IndexMap;

use super::{ControllerUse, HeldItemScale, Scripts, SpawnEgg, scripts::script_source};
use crate::id::{AnimationRef, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath};
use crate::json::{StrictBool, StrictString, Value};
use crate::molang::Molang;
use crate::version::{FormatVersion, SemVersion};

/// What the shared identifier map does with a document offered for an identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MergeStep {
    /// No definition had the identifier: the document defines it.
    Create,
    /// The definition has the same `min_engine_version`: the document is merged into it.
    Merge,
    /// The document's `min_engine_version` is higher: it replaces the definition whole.
    Replace,
    /// The document is dropped; vanilla says nothing.
    Ignore(ShadowReason),
}

/// Why [`MergeStep::Ignore`] dropped a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShadowReason {
    /// Its `min_engine_version` is above the highest pack `min_engine_version` of the stack.
    MinEngineVersionAboveStack,
    /// Its `min_engine_version` is below that of the definition already loaded.
    OlderMinEngineVersion,
}

/// The step for a document whose `description.min_engine_version` is `offered`, against the
/// definition already loaded for its identifier (with `existing`, its `min_engine_version`) and
/// the stack's `full_stack` version (the highest pack-manifest `min_engine_version` of the
/// stack). Above the stack → ignored; no definition → created; lower → ignored; equal →
/// merged; higher → replaced. Equal means equal numbers and pre-release (build metadata aside).
#[must_use]
pub fn merge_step(
    full_stack: &SemVersion,
    offered: &SemVersion,
    existing: Option<&SemVersion>,
) -> MergeStep {
    if full_stack < offered {
        return MergeStep::Ignore(ShadowReason::MinEngineVersionAboveStack);
    }
    match existing {
        None => MergeStep::Create,
        Some(existing) if offered < existing => {
            MergeStep::Ignore(ShadowReason::OlderMinEngineVersion)
        }
        Some(existing) if offered == existing => MergeStep::Merge,
        Some(_) => MergeStep::Replace,
    }
}

/// The version a document competes with: its `description.min_engine_version` read as a
/// version string or a `[major, minor, patch]` array, or 0.0.0 when it is absent, does not
/// read (silently: the schema lets a malformed string through), is the any-version `"*"`, or
/// the document's `format_version` is below 1.8.0.
pub fn description_min_engine_version(
    format_version: FormatVersion,
    value: Option<&Value>,
) -> SemVersion {
    // The version is read as a semantic version without wildcards, and only from 1.8.0.
    const ZERO: SemVersion = SemVersion::new(0, 0, 0);
    if format_version < FormatVersion::new(1, 8, 0) {
        return ZERO;
    }
    value
        .and_then(SemVersion::from_json)
        .filter(|version| !version.is_any())
        .unwrap_or(ZERO)
}

/// The description members the shared identifier map merges, borrowed from one document so
/// the client-entity and attachable families share one merge without sharing a struct.
///
/// Each family builds it from its own description; a member it does not have is not merged.
#[derive(Debug)]
pub struct DescriptionSlots<'a> {
    /// `materials`: merged key by key. A plain map, like `textures` and `geometry`: the
    /// schema refuses an empty object there, so an empty map is a document without the member.
    pub materials: &'a mut IndexMap<String, MaterialId>,
    /// `textures`: merged key by key.
    pub textures: &'a mut IndexMap<String, TexturePath>,
    /// `geometry`: merged key by key.
    pub geometry: &'a mut IndexMap<String, GeometryId>,
    /// `animations`: merged key by key. `None` is a document without the member, which an
    /// authored empty object is not.
    pub animations: &'a mut Option<IndexMap<String, AnimationRef>>,
    /// `sound_effects`: merged key by key; `None` as for `animations`.
    pub sound_effects: &'a mut Option<IndexMap<String, SoundEventName>>,
    /// `particle_effects`: replaced when the later document has it.
    pub particle_effects: &'a mut Option<IndexMap<String, ParticleId>>,
    /// `queryable_geometry`: replaced when the later document has it.
    pub queryable_geometry: &'a mut Option<StrictString>,
    /// `render_controllers`: replaced when the later document has it.
    pub render_controllers: &'a mut Option<Vec<ControllerUse>>,
    /// `spawn_egg`: replaced whole when the later document has it.
    pub spawn_egg: &'a mut Option<SpawnEgg>,
    /// `scripts`: merged member by member ([`merge_slots`]).
    pub scripts: &'a mut Option<Scripts>,
    /// `enable_attachables`: replaced when the later document has it.
    pub enable_attachables: &'a mut Option<StrictBool>,
    /// `held_item_ignores_lighting`: replaced when the later document has it.
    pub held_item_ignores_lighting: &'a mut Option<StrictBool>,
    /// `held_item_scale`: replaced when the later document has a value vanilla stores.
    pub held_item_scale: &'a mut Option<HeldItemScale>,
    /// `hide_armor`: replaced when the later document has it.
    pub hide_armor: &'a mut Option<StrictBool>,
}

/// A document the shared identifier map holds: a client entity or an attachable.
pub trait SharedDescription {
    /// The members [`merge_slots`] merges.
    fn slots(&mut self) -> DescriptionSlots<'_>;

    /// Merges the members only this family has, after the shared ones; nothing by default.
    fn merge_own(&mut self, later: &mut Self) {
        let _ = later;
    }
}

/// Merges the `later` document into `earlier`, as vanilla merges two documents with the same
/// identifier and `min_engine_version`:
/// - the maps `materials`, `textures`, `geometry`, `animations`, `sound_effects` and
///   `scripts.variables` key by key, a later key replacing an earlier one that differs only in
///   ASCII case (vanilla lower-cases keys); the merged definition has `animations` or
///   `sound_effects` when one of the two documents has the member;
/// - `particle_effects`, `render_controllers`, `spawn_egg` (whole, so its absent members fall
///   back to their defaults) and `scripts.animate` replaced when the later document has them;
/// - the booleans and `queryable_geometry` replaced when the later document has them, so a
///   later `false` overrides an earlier `true`;
/// - a named script replaced when the later document's member builds a script (a non-empty
///   expression; a number or a boolean does not), `scaleX` / `scalex` as one script;
/// - `held_item_scale` replaced unless the later value is a number ≤ 0, which vanilla does not
///   store.
///
/// The caller removes scripts that fail to compile before merging: vanilla drops them.
pub fn merge_slots(earlier: &mut DescriptionSlots<'_>, later: &mut DescriptionSlots<'_>) {
    // The game writes a later definition into a reused one member by member: maps entry by
    // entry, the render-controller list and the particle map replaced whole, the spawn egg
    // assigned whole, `animate` replaced as a list, booleans written when present, and
    // `held_item_scale` only when above 0.
    merge_map(earlier.materials, later.materials);
    merge_map(earlier.textures, later.textures);
    merge_map(earlier.geometry, later.geometry);
    merge_present_map(earlier.animations, later.animations);
    merge_present_map(earlier.sound_effects, later.sound_effects);
    replace_if_present(earlier.particle_effects, later.particle_effects);
    replace_if_present(earlier.queryable_geometry, later.queryable_geometry);
    replace_if_present(earlier.render_controllers, later.render_controllers);
    replace_if_present(earlier.spawn_egg, later.spawn_egg);
    replace_if_present(earlier.enable_attachables, later.enable_attachables);
    replace_if_present(
        earlier.held_item_ignores_lighting,
        later.held_item_ignores_lighting,
    );
    replace_if_present(earlier.hide_armor, later.hide_armor);
    match later.held_item_scale.take() {
        Some(HeldItemScale::Number(scale)) if scale <= 0.0 => {}
        Some(scale) => *earlier.held_item_scale = Some(scale),
        None => {}
    }
    if let Some(later) = later.scripts.take() {
        merge_scripts(earlier.scripts.get_or_insert_with(Scripts::default), later);
    }
}

fn merge_scripts(earlier: &mut Scripts, later: Scripts) {
    if let Some(mut later) = later.variables {
        merge_map(
            earlier.variables.get_or_insert_with(IndexMap::new),
            &mut later,
        );
    }
    for (earlier, later) in [
        (&mut earlier.initialize, later.initialize),
        (&mut earlier.pre_animation, later.pre_animation),
    ] {
        if later
            .as_ref()
            .is_some_and(|parts| parts.iter().any(|part| !part.0.is_empty()))
        {
            *earlier = later;
        }
    }
    for (earlier, later) in [
        (&mut earlier.scale, later.scale),
        (&mut earlier.parent_setup, later.parent_setup),
        (
            &mut earlier.should_update_bones_and_effects_offscreen,
            later.should_update_bones_and_effects_offscreen,
        ),
        (
            &mut earlier.should_update_effects_offscreen,
            later.should_update_effects_offscreen,
        ),
        (&mut earlier.hide_held_items, later.hide_held_items),
    ] {
        if later.as_ref().and_then(script_source).is_some() {
            *earlier = later;
        }
    }
    for ((earlier_lower, earlier_upper), (later_lower, later_upper)) in [
        (
            (&mut earlier.scalex, &mut earlier.scale_x),
            (later.scalex, later.scale_x),
        ),
        (
            (&mut earlier.scaley, &mut earlier.scale_y),
            (later.scaley, later.scale_y),
        ),
        (
            (&mut earlier.scalez, &mut earlier.scale_z),
            (later.scalez, later.scale_z),
        ),
    ] {
        let builds = |value: &Option<Molang>| value.as_ref().and_then(script_source).is_some();
        if builds(&later_lower) || builds(&later_upper) {
            *earlier_lower = later_lower;
            *earlier_upper = later_upper;
        }
    }
    if later.animate.is_some() {
        earlier.animate = later.animate;
    }
}

/// `later`'s entries into `earlier`, key by key; a key equal but for ASCII case replaces the
/// earlier entry in place (its spelling and its value), at the earlier entry's position.
///
/// Linear in the two maps: each later key is looked up by its lower-case form, and the map is
/// built again once, so a respelt key is hashed as it is spelt (changing a key in place would
/// leave the map unable to find it).
fn merge_map<V>(earlier: &mut IndexMap<String, V>, later: &mut IndexMap<String, V>) {
    if later.is_empty() {
        return;
    }
    let mut entries: Vec<(String, V)> = std::mem::take(earlier).into_iter().collect();
    // Where the first entry of each lower-case key is.
    let mut by_lower: HashMap<String, usize> = HashMap::with_capacity(entries.len() + later.len());
    for (index, (key, _)) in entries.iter().enumerate() {
        by_lower.entry(key.to_ascii_lowercase()).or_insert(index);
    }
    for (key, value) in later.drain(..) {
        match by_lower.entry(key.to_ascii_lowercase()) {
            Entry::Occupied(found) => entries[*found.get()] = (key, value),
            Entry::Vacant(free) => {
                free.insert(entries.len());
                entries.push((key, value));
            }
        }
    }
    *earlier = entries.into_iter().collect();
}

/// [`merge_map`] for a member that keeps its presence: the merged definition has the member
/// when either document has it, and no document gains a member neither wrote.
fn merge_present_map<V>(
    earlier: &mut Option<IndexMap<String, V>>,
    later: &mut Option<IndexMap<String, V>>,
) {
    if let Some(mut later) = later.take() {
        merge_map(earlier.get_or_insert_with(IndexMap::new), &mut later);
    }
}

fn replace_if_present<T>(earlier: &mut Option<T>, later: &mut Option<T>) {
    if let Some(later) = later.take() {
        *earlier = Some(later);
    }
}

/// Vanilla's shared identifier map of client entities and attachables: documents offered in
/// vanilla's walk order (lowest-priority pack first, every `entity/` directory before any
/// `attachables/` one) are created, merged, replaced or ignored by
/// `description.min_engine_version` ([`merge_step`]). Identifiers compare in ASCII lower case.
///
/// Client entities and attachables each get their own map (vanilla shares one; the families
/// report a collision instead).
#[derive(Clone, Debug)]
pub struct EngineVersionMerge<T> {
    full_stack: SemVersion,
    entries: IndexMap<String, (SemVersion, T)>,
}

impl<T: SharedDescription> EngineVersionMerge<T> {
    /// An empty map for a stack whose highest pack-manifest `min_engine_version` is
    /// `full_stack` (vanilla: the current game version for an empty stack).
    #[must_use]
    pub fn new(full_stack: SemVersion) -> Self {
        Self {
            full_stack,
            entries: IndexMap::new(),
        }
    }

    /// Offers the next document, with the version [`description_min_engine_version`] read for
    /// it, and says what became of it. A replaced definition keeps its place in the order.
    pub fn offer(
        &mut self,
        identifier: &str,
        min_engine_version: SemVersion,
        mut document: T,
    ) -> MergeStep {
        let key = identifier.to_ascii_lowercase();
        let existing = self.entries.get_mut(&key);
        let step = merge_step(
            &self.full_stack,
            &min_engine_version,
            existing.as_ref().map(|(version, _)| version),
        );
        match (step, existing) {
            (MergeStep::Merge, Some((_, earlier))) => {
                merge_slots(&mut earlier.slots(), &mut document.slots());
                earlier.merge_own(&mut document);
            }
            (MergeStep::Replace, Some(entry)) => *entry = (min_engine_version, document),
            (MergeStep::Create, _) => {
                self.entries.insert(key, (min_engine_version, document));
            }
            _ => {}
        }
        step
    }

    /// The effective definition of `identifier`.
    #[must_use]
    pub fn get(&self, identifier: &str) -> Option<&T> {
        self.entries
            .get(&identifier.to_ascii_lowercase())
            .map(|(_, document)| document)
    }

    /// The `min_engine_version` the definition of `identifier` competes with.
    #[must_use]
    pub fn min_engine_version(&self, identifier: &str) -> Option<&SemVersion> {
        self.entries
            .get(&identifier.to_ascii_lowercase())
            .map(|(version, _)| version)
    }

    /// The number of identifiers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// No identifier is defined.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The lower-cased identifiers and their definitions, in order of first definition.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &T)> {
        self.entries
            .iter()
            .map(|(key, (_, document))| (key.as_str(), document))
    }

    /// The lower-cased identifiers and their definitions, in order of first definition.
    pub fn into_definitions(self) -> impl Iterator<Item = (String, T)> {
        self.entries
            .into_iter()
            .map(|(key, (_, document))| (key, document))
    }
}
