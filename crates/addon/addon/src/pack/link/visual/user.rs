//! What a client entity, an attachable and an atomic client entity share: a description whose
//! maps name materials, textures, geometries, animations, particles and sounds and whose list
//! names render controllers. [`User`] is that description of one document, borrowed; the
//! checks of the maps are here once, each behind the feature of its target, and the files of
//! the pairs (`entity_render.rs`, `item_geometry.rs`, …) call them for their documents.

#![allow(
    dead_code,
    reason = "which checks and which members are used depends on the enabled features"
)]

use std::borrow::Cow;

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::id::{
    AnimationRef, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::visual::{AnimateEntry, ControllerUse};
use indexmap::IndexMap;

/// Which family a [`User`] is of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Family {
    /// `minecraft:client_entity`.
    ClientEntity,
    /// `minecraft:attachable`.
    Attachable,
    /// `minecraft:atomic_client_entity`.
    Atomic,
}

/// The description of one client entity, attachable or atomic client entity, as the rules
/// read it.
#[derive(Debug)]
pub(super) struct User<'a> {
    /// The document.
    pub(super) origin: &'a Origin,
    /// Its family.
    pub(super) family: Family,
    /// `materials`.
    pub(super) materials: Option<&'a IndexMap<String, MaterialId>>,
    /// `textures`.
    pub(super) textures: Option<&'a IndexMap<String, TexturePath>>,
    /// `geometry`.
    pub(super) geometry: Option<&'a IndexMap<String, GeometryId>>,
    /// `animations`.
    pub(super) animations: Option<&'a IndexMap<String, AnimationRef>>,
    /// `particle_effects`.
    pub(super) particle_effects: Option<&'a IndexMap<String, ParticleId>>,
    /// `sound_effects`.
    pub(super) sound_effects: Option<&'a IndexMap<String, SoundEventName>>,
    /// `render_controllers`, in order.
    pub(super) render_controllers: &'a [ControllerUse],
    /// `scripts.animate` of a document written before 1.10.0 that no other document was
    /// merged into: its entries are the elements of the file's `animation_controllers` list,
    /// in order ([`User::animation_path`]). `None` for any other document.
    pub(super) made_controllers: Option<&'a [AnimateEntry]>,
}

/// The prefix of the name an upgraded 1.8.0 document gives the animation controller of an
/// `animation_controllers` element.
const MADE_CONTROLLER: &str = "controller__";

impl User<'_> {
    /// The payload key of the family.
    pub(super) fn payload(&self) -> &'static str {
        match self.family {
            Family::ClientEntity => "minecraft:client_entity",
            Family::Attachable => "minecraft:attachable",
            Family::Atomic => "minecraft:atomic_client_entity",
        }
    }

    /// Whether the game lower-cases the names of the description's maps before it looks them
    /// up: it does for a client entity and an attachable, and for nothing of an atomic client
    /// entity.
    pub(super) fn folds(&self) -> bool {
        self.family != Family::Atomic
    }

    /// `name` as the game reads it: in ASCII lower case where the family lower-cases.
    pub(super) fn folded<'n>(&self, name: &'n str) -> Cow<'n, str> {
        if self.folds() && name.bytes().any(|byte| byte.is_ascii_uppercase()) {
            Cow::Owned(name.to_ascii_lowercase())
        } else {
            Cow::Borrowed(name)
        }
    }

    /// The path of `description.<member>`.
    pub(super) fn member(&self, member: &str) -> JsonPath {
        let mut path = JsonPath::new();
        path.push(self.payload());
        path.push("description");
        path.push(member);
        path
    }

    /// The path of the entry `key` of the map `description.<member>`.
    pub(super) fn entry(&self, member: &str, key: &str) -> JsonPath {
        let mut path = self.member(member);
        path.push(key);
        path
    }

    /// The path the `animations` entry `key` has in the file. A document written before
    /// 1.10.0 lists its animation controllers in `animation_controllers`, each element one
    /// `{ name: controller }` object that the upgrade turns into the animation
    /// `controller__<name>` and one entry of `scripts.animate`: such an entry is told where
    /// the file has it, `animation_controllers/<index>/<name>`.
    ///
    /// The index is the entry's place among the made entries of `scripts.animate`; an empty
    /// element of the file's list, which makes no entry, is not counted.
    pub(super) fn animation_path(&self, key: &str) -> JsonPath {
        let made = self.made_controllers.and_then(|animate| {
            let name = key.strip_prefix(MADE_CONTROLLER)?;
            let index = animate
                .iter()
                .filter_map(|entry| match entry {
                    AnimateEntry::Always(made) if made.starts_with(MADE_CONTROLLER) => Some(made),
                    _ => None,
                })
                .enumerate()
                .filter(|(_, made)| made.as_str() == key)
                .map(|(index, _)| index)
                .last()?;
            Some((index, name))
        });
        match made {
            Some((index, name)) => {
                let mut path = self.member("animation_controllers");
                path.push_index(index);
                path.push(name);
                path
            }
            None => self.entry("animations", key),
        }
    }
}

/// The keys of a description's map in ASCII lower case, as the game keys its slots and its
/// short names; empty for a description without the map.
pub(super) fn lowered_keys<V>(
    map: Option<&IndexMap<String, V>>,
) -> std::collections::HashSet<String> {
    map.into_iter()
        .flat_map(IndexMap::keys)
        .map(|key| key.to_ascii_lowercase())
        .collect()
}
