//! Format versions 1.8.0 up to 1.10.0.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::id::{
    AnimationRef, GeometryId, MaterialId, ParticleId, RenderControllerId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor, Step};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use indexmap::IndexMap;

use crate::family::{Visual, document, read_description};
use crate::read::{
    Named, animation_controllers as controller_names, identifier, min_engine_version,
    molang_elements, name_map, not_valid_here, slot_map,
};
use crate::rules::{CONDITION, NAMED_SCRIPT};
use crate::{AnimateEntry, ControllerUse, SpawnEgg};

/// `scripts` at format versions 1.8.0 up to 1.10.0: `pre_animation`, the scale scripts and
/// `parent_setup` only. `variables`, `initialize`, `animate` and the off-screen scripts do not
/// exist yet.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::molang = NAMED_SCRIPT)]
pub struct Scripts {
    /// `pre_animation`: each element a Molang value.
    #[facet(schema::read_with = molang_elements, skip_serializing_if = Option::is_none)]
    pub pre_animation: Option<Vec<Molang>>,
    /// `scale`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scale: Option<Molang>,
    /// `scaleX` (the same script as `scalex`).
    #[facet(rename = "scaleX", skip_serializing_if = Option::is_none)]
    pub scale_x: Option<Molang>,
    /// `scalex`; wins over `scaleX`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scalex: Option<Molang>,
    /// `scaleY` (the same script as `scaley`).
    #[facet(rename = "scaleY", skip_serializing_if = Option::is_none)]
    pub scale_y: Option<Molang>,
    /// `scaley`; wins over `scaleY`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scaley: Option<Molang>,
    /// `scaleZ` (the same script as `scalez`).
    #[facet(rename = "scaleZ", skip_serializing_if = Option::is_none)]
    pub scale_z: Option<Molang>,
    /// `scalez`; wins over `scaleZ`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub scalez: Option<Molang>,
    /// `parent_setup`.
    #[facet(skip_serializing_if = Option::is_none)]
    pub parent_setup: Option<Molang>,
}

impl Scripts {
    /// The same members at format version 1.10.0, which adds `variables`, `initialize`,
    /// `animate` and the off-screen scripts (absent here).
    #[must_use]
    pub fn into_v1_10(self) -> super::v1_10_0::Scripts {
        super::v1_10_0::Scripts {
            variables: None,
            initialize: None,
            pre_animation: self.pre_animation,
            scale: self.scale,
            scale_x: self.scale_x,
            scalex: self.scalex,
            scale_y: self.scale_y,
            scaley: self.scaley,
            scale_z: self.scale_z,
            scalez: self.scalez,
            parent_setup: self.parent_setup,
            should_update_bones_and_effects_offscreen: None,
            should_update_effects_offscreen: None,
            animate: None,
        }
    }
}

/// What vanilla's 1.8 → 1.10 step (`upgrade_v1_8_to_v1_10`) does with a description's
/// `animation_controllers`: its converter visits `minecraft:client_entity` only, so the list
/// of a client entity moves and an attachable's stays. A game rule of the family, not a drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerList {
    /// A client entity's: when the description has `animations`, each element becomes the
    /// animation `controller__<name>` and an entry of `scripts.animate`, and the list is
    /// removed. Only the member whose key sorts first byte by byte is read from an element;
    /// the others are dropped with an information. Without `animations` nothing moves.
    Moved,
    /// An attachable's: nothing moves.
    Kept,
}

/// The 1.8 → 1.10 step of `animation_controllers` (`controllers`), into `animations` and
/// `scripts` (already the 1.10 shape) as `list` says; `at` is the description. A list that
/// stays is a member 1.10.0 does not have: "child '`animation_controllers`' not valid here.",
/// and the document is rejected.
pub fn animation_controllers(
    list: ControllerList,
    controllers: Option<Vec<IndexMap<String, String>>>,
    animations: &mut Option<IndexMap<String, AnimationRef>>,
    scripts: &mut Option<super::v1_10_0::Scripts>,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) {
    if list == ControllerList::Kept {
        if controllers.is_some() {
            refused(at, cx);
        }
        return;
    }
    match (animations, controllers) {
        (Some(animations), Some(controllers)) => {
            let mut animate = Vec::with_capacity(controllers.len());
            // The element and key each made name comes from.
            let mut made_at: HashMap<String, (usize, String)> = HashMap::new();
            for (index, element) in controllers.iter().enumerate() {
                // The game takes the first member of the element in its map order, which
                // is the smallest key by bytes.
                let Some((first, controller)) = element
                    .iter()
                    .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
                else {
                    cx.diagnostics.push(empty_element(at, index));
                    continue;
                };
                for key in element.keys().filter(|key| *key != first) {
                    cx.diagnostics.push(
                        Diagnostic::of(&core_codes::DROPPED_FIELD)
                            .at(controller_at(at, index, key))
                            .phase(Phase::Upgrade),
                    );
                }
                let name = format!("controller__{first}");
                let entry = animate.len();
                animate.push(AnimateEntry::Always(name.clone()));
                let made = AnimationRef::parse(controller.clone());
                // An animation of the file under the name is written over, as the
                // game's step writes over it; the crate notes it where it is (the game says
                // nothing). Where the animation is one an earlier element of this list
                // made, the file has it there, not under the made name.
                if animations
                    .get(&name)
                    .is_some_and(|own| own.as_str() != made.as_str())
                {
                    cx.diagnostics
                        .push(written_over(at, &name, made_at.get(&name)));
                }
                made_at.insert(name.clone(), (index, first.clone()));
                // The notes above are of members the file has and stay where they are.
                made_of(cx, at, controller_at(at, index, first), &name, entry);
                animations.insert(name, made);
            }
            if !animate.is_empty() {
                scripts
                    .get_or_insert_with(super::v1_10_0::Scripts::default)
                    .animate
                    .get_or_insert_with(Vec::new)
                    .extend(animate);
            }
        }
        (None, Some(_)) => refused(at, cx),
        (_, None) => {}
    }
}

/// The list stays: the `v1_10` schema that runs next does not declare the member.
fn refused(at: &JsonPath, cx: &mut UpgradeCx<'_>) {
    let found = not_valid_here("animation_controllers", at.with(&["animation_controllers"]))
        .phase(Phase::Upgrade);
    cx.diagnostics.push(found);
}

/// Records that the animation `name` and the entry `entry` of `scripts.animate` were made of
/// the controller at `from` (`at` is the description): what is told of them from here on is
/// told at the controller.
fn made_of(cx: &mut UpgradeCx<'_>, at: &JsonPath, from: JsonPath, name: &str, entry: usize) {
    let mut entry_at = at.with(&["scripts", "animate"]);
    entry_at.push_index(entry);
    cx.diagnostics
        .moved(from.clone(), at.with(&["animations", name]));
    cx.diagnostics.moved(from, entry_at);
}

/// The note of the animation `name` a controller writes over, where the file has it: the
/// element and key of `animation_controllers` an earlier controller made it from (`made`),
/// or `animations` (`at` is the description).
fn written_over(at: &JsonPath, name: &str, made: Option<&(usize, String)>) -> Diagnostic {
    let replaced = made.map_or_else(
        || at.with(&["animations", name]),
        |(earlier, key)| controller_at(at, *earlier, key),
    );
    Diagnostic::of(&core_codes::DROPPED_FIELD)
        .at(replaced)
        .phase(Phase::Upgrade)
}

/// The note of an `animation_controllers` element without a member: it moves nothing and
/// goes with the list (the crate's own note; the game is silent).
fn empty_element(at: &JsonPath, index: usize) -> Diagnostic {
    let mut element_at = at.with(&["animation_controllers"]);
    element_at.push_index(index);
    Diagnostic::of(&core_codes::DROPPED_FIELD)
        .at(element_at)
        .phase(Phase::Upgrade)
}

/// The member `key` of the element `index` of `animation_controllers` (`at` is the
/// description).
fn controller_at(at: &JsonPath, index: usize, key: &str) -> JsonPath {
    at.child("animation_controllers").index(index).child(key)
}

/// `description` at format versions 1.8.0 up to 1.10.0: animation controllers are a list of
/// their own, render controllers are names only, and `queryable_geometry`, `held_item_scale` and
/// the family's own members do not exist.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::molang = CONDITION)]
pub struct Description<I: Named> {
    /// `identifier`.
    #[facet(schema::read_with = identifier)]
    pub identifier: I,
    /// `min_engine_version`.
    #[facet(schema::read_with = min_engine_version)]
    pub min_engine_version: Option<Value>,
    /// `materials`.
    #[facet(schema::read_with = slot_map)]
    pub materials: Option<IndexMap<String, MaterialId>>,
    /// `textures`.
    #[facet(schema::read_with = slot_map)]
    pub textures: Option<IndexMap<String, TexturePath>>,
    /// `geometry`.
    #[facet(schema::read_with = slot_map)]
    pub geometry: Option<IndexMap<String, GeometryId>>,
    /// `spawn_egg`.
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`.
    pub scripts: Option<Scripts>,
    /// `particle_effects`.
    #[facet(schema::read_with = name_map)]
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    #[facet(schema::read_with = name_map)]
    pub particle_emitters: Option<IndexMap<String, String>>,
    /// `animations`.
    #[facet(schema::read_with = name_map)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `animation_controllers`: a list of `{ name: controller }` objects, of which vanilla
    /// reads one member each (the key that sorts first byte by byte).
    #[facet(schema::read_with = controller_names)]
    pub animation_controllers: Option<Vec<IndexMap<String, String>>>,
    /// `render_controllers`: names only.
    pub render_controllers: Option<Vec<String>>,
    /// `sound_effects`.
    #[facet(schema::read_with = name_map)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`.
    pub enable_attachables: Option<bool>,
    /// `held_item_ignores_lighting`.
    pub held_item_ignores_lighting: Option<bool>,
    /// `hide_armor`.
    pub hide_armor: Option<bool>,
}

/// A document of format version 1.8.0 up to 1.10.0, of the family `V`.
#[derive(Clone, Debug, PartialEq)]
pub struct Described<V: Visual> {
    /// `format_version` as written (1.8.0 for a document without one).
    pub format_version: FormatVersion,
    /// `description`.
    pub description: Description<V::Id>,
}

impl<V: Visual> Versioned for Described<V> {
    const VERSION: FormatVersion = known::V1_8_0;
}

impl<V: Visual> VersionStruct for Described<V> {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.8 schema.
        let description = read_description::<V, _>(document, cx, d, true)?;
        Some(Self {
            format_version: cx.original_version,
            description,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        document::<V, _>(self.format_version, &self.description)
    }
}

impl<V: Visual> MolangFields for Described<V> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let Some(scripts) = &mut self.description.scripts else {
            return;
        };
        visitor.enter(Step::Key(V::PAYLOAD));
        visitor.enter(Step::Key("description"));
        visitor.enter(Step::Key("scripts"));
        scripts.visit_molang(visitor);
        visitor.leave();
        visitor.leave();
        visitor.leave();
    }
}

/// The 1.8 → 1.10 step (vanilla `upgrade_v1_8_to_v1_10`): what it does with the list of
/// `animation_controllers` is the family's ([`ControllerList`]); the rest is the stamp of the
/// version and the move of the members.
impl<V: Visual> Upgrade for Described<V> {
    type Next = super::v1_10_0::Described<V>;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let d = self.description;
        let mut animations = d.animations;
        let mut scripts = d.scripts.map(Scripts::into_v1_10);
        animation_controllers(
            V::CONTROLLERS,
            d.animation_controllers,
            &mut animations,
            &mut scripts,
            &JsonPath::of_keys([V::PAYLOAD, "description"]),
            cx,
        );
        super::v1_10_0::Described {
            format_version: known::V1_10_0,
            description: super::v1_10_0::Description {
                identifier: d.identifier,
                min_engine_version: d.min_engine_version,
                materials: d.materials,
                textures: d.textures,
                geometry: d.geometry,
                queryable_geometry: None,
                spawn_egg: d.spawn_egg,
                scripts,
                particle_effects: d.particle_effects,
                particle_emitters: d.particle_emitters,
                animations,
                render_controllers: d.render_controllers.map(|controllers| {
                    controllers
                        .into_iter()
                        .map(|name| ControllerUse::Always(RenderControllerId::parse(name)))
                        .collect()
                }),
                sound_effects: d.sound_effects,
                enable_attachables: d.enable_attachables,
                held_item_ignores_lighting: d.held_item_ignores_lighting,
                hide_armor: d.hide_armor,
                own: V::Own::default(),
            },
        }
    }
}
