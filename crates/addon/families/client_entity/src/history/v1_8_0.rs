//! Format versions 1.8.0 up to 1.10.0, and the step to 1.10.0.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, RenderControllerId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_shared_visual::history::v1_8_0::Scripts;
use bedrock_addon_shared_visual::history::v1_8_0::{ControllerList, animation_controllers};
use bedrock_addon_shared_visual::{ControllerUse, SpawnEgg};
use facet::Facet;
use indexmap::IndexMap;

use super::super::read::{self, is_map_key, not_valid_here};
use super::super::write;

/// A client entity document of format version 1.8.0 up to 1.10.0.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientEntity {
    /// `format_version` as written (1.8.0 for a document without one).
    pub format_version: FormatVersion,
    /// `description`.
    pub description: Description,
}

/// `description` at format versions 1.8.0 up to 1.10.0: animation controllers are a list of
/// their own, render controllers are names only, and `queryable_geometry` and
/// `held_item_scale` do not exist.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Description {
    /// `identifier`.
    pub identifier: EntityId,
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

impl Versioned for ClientEntity {
    const VERSION: FormatVersion = known::V1_8_0;
}

impl VersionStruct for ClientEntity {
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
        // Element members must match `[a-zA-Z0-9_.-]+` in full.
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

impl MolangFields for ClientEntity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let Some(scripts) = &mut self.description.scripts else {
            return;
        };
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        visitor.enter(Step::Key("scripts"));
        scripts.visit_molang(visitor);
        visitor.leave();
        visitor.leave();
        visitor.leave();
    }
}

/// The 1.8 → 1.10 step (vanilla `upgrade_v1_8_to_v1_10`): when the description has
/// `animations`, each element of `animation_controllers` becomes the animation
/// `controller__<name>` and an entry of `scripts.animate`, and the list is removed. Only the
/// member whose key sorts first byte by byte is read from an element; the others are dropped
/// with a warning. Without `animations` nothing moves, and the list that stays is a member
/// 1.10.0 does not have: the document is rejected.
impl Upgrade for ClientEntity {
    type Next = super::v1_10_0::ClientEntity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let d = self.description;
        let mut animations = d.animations;
        let mut scripts = d.scripts.map(Scripts::into_v1_10);
        animation_controllers(
            ControllerList::Moved,
            d.animation_controllers,
            &mut animations,
            &mut scripts,
            &description_at(),
            cx,
        );
        super::v1_10_0::ClientEntity {
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
            },
        }
    }
}

/// `minecraft:client_entity/description`.
pub(super) fn description_at() -> JsonPath {
    JsonPath::of_keys([read::PAYLOAD, "description"])
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::vanilla::ActorType;

    use super::*;

    /// The findings of the 1.8 → 1.10 step on a description with `members`, with the moves
    /// the step recorded.
    fn stepped(members: &str) -> Diagnostics {
        let text = format!(r#"{{"identifier": "demo:ghost", {members}}}"#);
        let description =
            bedrock_addon_core::json::from_str::<Description>(&text).expect("a description");
        let entity = ClientEntity {
            format_version: FormatVersion::new(1, 8, 0),
            description,
        };
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let _ = entity.upgrade(&mut UpgradeCx {
            load: &load,
            original_version: FormatVersion::new(1, 8, 0),
            identifier: "demo:ghost",
            actor_type: ActorType::Mob,
            skip_section_a: false,
            reading: load.reading,
            diagnostics: &mut diagnostics,
        });
        diagnostics
    }

    /// Where the file has the member the upgraded document has at `at` (below the
    /// description).
    fn in_the_file(told: &Diagnostics, at: &[&str]) -> String {
        told.source_of(&description_at().with(at)).to_string()
    }

    /// [`in_the_file`] for the entry `index` of `scripts.animate`.
    fn animate_in_the_file(told: &Diagnostics, index: usize) -> String {
        let mut at = description_at().with(&["scripts", "animate"]);
        at.push_index(index);
        told.source_of(&at).to_string()
    }

    /// What a later check tells of an animation or an
    /// `animate` entry the step made of a controller is told at the controller's element.
    #[test]
    fn the_animation_and_the_animate_entry_made_of_a_controller_are_where_the_file_has_it() {
        let told = stepped(
            r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": [{"move": "controller.animation.x.a"}, {"zz": "controller.animation.x.z", "look": "controller.animation.x.b"}]"#,
        );
        let controllers = "minecraft:client_entity/description/animation_controllers";
        assert_eq!(
            in_the_file(&told, &["animations", "controller__move"]),
            format!("{controllers}/0/move")
        );
        assert_eq!(
            in_the_file(&told, &["animations", "controller__look"]),
            format!("{controllers}/1/look")
        );
        assert_eq!(
            animate_in_the_file(&told, 0),
            format!("{controllers}/0/move")
        );
        assert_eq!(
            animate_in_the_file(&told, 1),
            format!("{controllers}/1/look")
        );
        // An animation of the file is where it was.
        assert_eq!(
            in_the_file(&told, &["animations", "walk"]),
            "minecraft:client_entity/description/animations/walk"
        );
    }

    /// Of two elements of one key the later one made the animation that stands; the note
    /// of the earlier one and of an animation of the file stay where they are.
    #[test]
    fn an_animation_made_twice_is_where_the_later_controller_is() {
        let told = stepped(
            r#""animations": {"controller__move": "animation.x.own"}, "animation_controllers": [{"move": "controller.animation.x.a"}, {"move": "controller.animation.x.b"}]"#,
        );
        let description = "minecraft:client_entity/description";
        assert_eq!(
            in_the_file(&told, &["animations", "controller__move"]),
            format!("{description}/animation_controllers/1/move")
        );
        let mut told = told;
        told.name_as_the_file_has_it();
        let notes: Vec<String> = told
            .iter()
            .map(|f| {
                format!(
                    "{} {:?} {}",
                    f.code.name(),
                    f.severity,
                    f.location.json_path
                )
            })
            .collect();
        assert_eq!(
            notes,
            [
                format!("DroppedField Info {description}/animations/controller__move"),
                format!("DroppedField Info {description}/animation_controllers/0/move"),
            ]
        );
    }
}
