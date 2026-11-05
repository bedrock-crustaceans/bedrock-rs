//! Format versions 1.8.0 up to 1.10.0, and the step to 1.10.0.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Phase};
use bedrock_addon_core::id::{
    AnimationRef, AttachableId, GeometryId, MaterialId, ParticleId, RenderControllerId,
    SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_core::visual::history::{v1_8_0::Scripts, v1_10_0};
use bedrock_addon_core::visual::{ControllerUse, SpawnEgg};
use facet::Facet;
use indexmap::IndexMap;

use super::super::read::{self, is_map_key, not_valid_here};
use super::super::validate::NAMED_SCRIPT;
use super::super::write;

/// An attachable document of format version 1.8.0 up to 1.10.0.
#[derive(Clone, Debug, PartialEq)]
pub struct Attachable {
    /// `format_version` as written (1.8.0 for a document without one).
    pub format_version: FormatVersion,
    /// `description`.
    pub description: Description,
}

/// `description` at format versions 1.8.0 up to 1.10.0: animation controllers are a list of
/// their own (which an attachable cannot carry on to 1.10.0), render controllers are names
/// only, and `queryable_geometry`, `held_item_scale` and `item` do not exist.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Description {
    /// `identifier`.
    pub identifier: AttachableId,
    /// `min_engine_version`.
    pub min_engine_version: Option<Value>,
    /// `materials`.
    pub materials: Option<IndexMap<String, MaterialId>>,
    /// `textures`.
    pub textures: Option<IndexMap<String, TexturePath>>,
    /// `geometry`.
    pub geometry: Option<IndexMap<String, GeometryId>>,
    /// `spawn_egg`.
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`.
    pub scripts: Option<Scripts>,
    /// `particle_effects`.
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    pub particle_emitters: Option<IndexMap<String, String>>,
    /// `animations`.
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `animation_controllers`: a list of `{ name: controller }` objects, of which vanilla
    /// reads one member each (the key that sorts first byte by byte).
    pub animation_controllers: Option<Vec<IndexMap<String, String>>>,
    /// `render_controllers`: names only.
    pub render_controllers: Option<Vec<String>>,
    /// `sound_effects`.
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`.
    pub enable_attachables: Option<bool>,
    /// `held_item_ignores_lighting`.
    pub held_item_ignores_lighting: Option<bool>,
    /// `hide_armor`.
    pub hide_armor: Option<bool>,
}

impl Versioned for Attachable {
    const VERSION: FormatVersion = known::V1_8_0;
}

impl VersionStruct for Attachable {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.8 schema.
        let described = read::describe(document, cx, d)?;
        let description: Description = read::typed(&described, true, &[], true, d)?;
        let at = &described.at;
        let before = d.len();
        let de = &description;
        read::check_identifier(de.identifier.as_str(), at, d);
        read::check_min_engine_version(de.min_engine_version.as_ref(), at, d);
        read::check_map(de.materials.as_ref(), "materials", true, at, d);
        read::check_map(de.textures.as_ref(), "textures", true, at, d);
        read::check_map(de.geometry.as_ref(), "geometry", true, at, d);
        read::check_map(
            de.particle_effects.as_ref(),
            "particle_effects",
            false,
            at,
            d,
        );
        read::check_map(
            de.particle_emitters.as_ref(),
            "particle_emitters",
            false,
            at,
            d,
        );
        read::check_map(de.animations.as_ref(), "animations", false, at, d);
        read::check_map(de.sound_effects.as_ref(), "sound_effects", false, at, d);
        // Element members match `[a-zA-Z0-9_.-]+` in full.
        for (index, element) in de.animation_controllers.iter().flatten().enumerate() {
            let mut element_at = at.with(&["animation_controllers"]);
            element_at.push_index(index);
            for key in element.keys().filter(|key| !is_map_key(key)) {
                d.push(not_valid_here(key, element_at.with(&[key])));
            }
        }
        (d.len() == before).then_some(Self {
            format_version: cx.original_version,
            description,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write::document(self.format_version, &self.description)
    }
}

impl MolangFields for Attachable {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let Some(scripts) = &mut self.description.scripts else {
            return;
        };
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        visitor.enter(Step::Key("scripts"));
        visitor.field("pre_animation", &NAMED_SCRIPT, &mut scripts.pre_animation);
        visitor.field("scale", &NAMED_SCRIPT, &mut scripts.scale);
        visitor.field("scaleX", &NAMED_SCRIPT, &mut scripts.scale_x);
        visitor.field("scalex", &NAMED_SCRIPT, &mut scripts.scalex);
        visitor.field("scaleY", &NAMED_SCRIPT, &mut scripts.scale_y);
        visitor.field("scaley", &NAMED_SCRIPT, &mut scripts.scaley);
        visitor.field("scaleZ", &NAMED_SCRIPT, &mut scripts.scale_z);
        visitor.field("scalez", &NAMED_SCRIPT, &mut scripts.scalez);
        visitor.field("parent_setup", &NAMED_SCRIPT, &mut scripts.parent_setup);
        visitor.leave();
        visitor.leave();
        visitor.leave();
    }
}

/// The 1.8 → 1.10 step (vanilla `upgrade_v1_8_to_v1_10`). For an attachable vanilla stamps the
/// version and converts nothing: its converter visits `minecraft:client_entity` only. An
/// `animation_controllers` list therefore stays, as a member 1.10.0 does not have: the
/// document is rejected.
impl Upgrade for Attachable {
    type Next = super::v1_10_0::Attachable;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        // The step from 1.8 to 1.10.
        let d = self.description;
        if d.animation_controllers.is_some() {
            // The converter that moves the list into `animations` and `scripts.animate`
            // visits `minecraft:client_entity` only; the list stays, and
            // the v1_10 schema that runs next does not declare it.
            let found = not_valid_here(
                "animation_controllers",
                description_at().with(&["animation_controllers"]),
            )
            .phase(Phase::Upgrade);
            cx.diagnostics.push(found);
        }
        let scripts = d.scripts.map(|scripts| v1_10_0::Scripts {
            variables: None,
            initialize: None,
            pre_animation: scripts.pre_animation,
            scale: scripts.scale,
            scale_x: scripts.scale_x,
            scalex: scripts.scalex,
            scale_y: scripts.scale_y,
            scaley: scripts.scaley,
            scale_z: scripts.scale_z,
            scalez: scripts.scalez,
            parent_setup: scripts.parent_setup,
            should_update_bones_and_effects_offscreen: None,
            should_update_effects_offscreen: None,
            animate: None,
        });
        super::v1_10_0::Attachable {
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
                animations: d.animations,
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
                item: None,
            },
        }
    }
}

/// `minecraft:attachable/description`.
pub(super) fn description_at() -> JsonPath {
    JsonPath::of_keys([read::PAYLOAD, "description"])
}
