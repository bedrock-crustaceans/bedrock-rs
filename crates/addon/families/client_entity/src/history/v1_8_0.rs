//! Format versions 1.8.0 up to 1.10.0, and the step to 1.10.0.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, RenderControllerId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictBool, StrictString, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_core::visual::history::{v1_8_0::Scripts, v1_10_0};
use bedrock_addon_core::visual::{AnimateEntry, ControllerUse, SpawnEgg};
use facet::Facet;
use indexmap::IndexMap;

use super::super::read::{self, below, is_map_key, not_valid_here};
use super::super::validate::NAMED_SCRIPT;
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
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub min_engine_version: Option<Value>,
    /// `materials`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub materials: Option<IndexMap<String, MaterialId>>,
    /// `textures`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub textures: Option<IndexMap<String, TexturePath>>,
    /// `geometry`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub geometry: Option<IndexMap<String, GeometryId>>,
    /// `spawn_egg`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub spawn_egg: Option<SpawnEgg>,
    /// `scripts`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub scripts: Option<Scripts>,
    /// `particle_effects`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_effects: Option<IndexMap<String, ParticleId>>,
    /// `particle_emitters`: accepted and read by nothing.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub particle_emitters: Option<IndexMap<String, StrictString>>,
    /// `animations`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animations: Option<IndexMap<String, AnimationRef>>,
    /// `animation_controllers`: a list of `{ name: controller }` objects, of which vanilla
    /// reads one member each (the key that sorts first byte by byte).
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub animation_controllers: Option<Vec<IndexMap<String, StrictString>>>,
    /// `render_controllers`: names only.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<StrictString>>,
    /// `sound_effects`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub enable_attachables: Option<StrictBool>,
    /// `held_item_ignores_lighting`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub held_item_ignores_lighting: Option<StrictBool>,
    /// `hide_armor`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub hide_armor: Option<StrictBool>,
}

impl Versioned for ClientEntity {
    const VERSION: FormatVersion = FormatVersion::new(1, 8, 0);
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
            let mut element_at = below(at, &["animation_controllers"]);
            element_at.push_index(index);
            for key in element.keys().filter(|key| !is_map_key(key)) {
                d.push(not_valid_here(key, below(&element_at, &[key])));
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

/// The 1.8 → 1.10 step (vanilla `upgrade_v1_8_to_v1_10`): when the description has
/// `animations`, each element of `animation_controllers` becomes the animation
/// `controller__<name>` and an entry of `scripts.animate`, and the list is removed. Only the
/// member whose key sorts first byte by byte is read from an element; the others are dropped
/// with a warning. Without `animations` nothing moves, and the list that stays is a member
/// 1.10.0 does not have: the document is rejected.
impl Upgrade for ClientEntity {
    type Next = super::v1_10_0::ClientEntity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        // The game's 1.8 to 1.10 step, controller by controller.
        let d = self.description;
        let at = description_at();
        let mut animations = d.animations;
        let mut scripts = d.scripts.map(|scripts| v1_10_0::Scripts {
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
        match (&mut animations, d.animation_controllers) {
            (Some(animations), Some(controllers)) => {
                let mut animate = Vec::with_capacity(controllers.len());
                // The element and key each made name comes from.
                let mut made_at: std::collections::HashMap<String, (usize, String)> =
                    std::collections::HashMap::new();
                for (index, element) in controllers.iter().enumerate() {
                    // The game takes the first member of the element in its map order, which
                    // is the smallest key by bytes.
                    let Some((first, controller)) = element
                        .iter()
                        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
                    else {
                        cx.diagnostics.push(empty_element(&at, index));
                        continue;
                    };
                    for key in element.keys().filter(|key| *key != first) {
                        cx.diagnostics.push(step_finding(
                            DiagnosticCode::DroppedField { text: None },
                            Severity::Info,
                            controller_at(&at, index, key),
                        ));
                    }
                    let name = format!("controller__{first}");
                    let entry = animate.len();
                    animate.push(AnimateEntry::Always(name.clone()));
                    let made = AnimationRef::parse(controller.0.clone());
                    // An animation of the file under the name is written over, as the
                    // game's step writes over it; the crate notes it where it is (the game says
                    // nothing). Where the animation is one an earlier element of this list
                    // made, the file has it there, not under the made name.
                    if animations
                        .get(&name)
                        .is_some_and(|own| own.as_str() != made.as_str())
                    {
                        cx.diagnostics
                            .push(written_over(&at, &name, made_at.get(&name)));
                    }
                    made_at.insert(name.clone(), (index, first.clone()));
                    // The notes above are of members the file has and stay where they are.
                    made_of(cx, &at, controller_at(&at, index, first), &name, entry);
                    animations.insert(name, made);
                }
                if !animate.is_empty() {
                    scripts
                        .get_or_insert_with(v1_10_0::Scripts::default)
                        .animate
                        .get_or_insert_with(Vec::new)
                        .extend(animate);
                }
            }
            (None, Some(_)) => {
                // The v1_10 schema that runs next does not declare the member.
                let mut found = not_valid_here(
                    "animation_controllers",
                    below(&at, &["animation_controllers"]),
                );
                found.phase = Phase::Upgrade;
                cx.diagnostics.push(found);
            }
            (_, None) => {}
        }
        super::v1_10_0::ClientEntity {
            format_version: FormatVersion::new(1, 10, 0),
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
                        .map(|name| ControllerUse::Always(RenderControllerId::parse(name.0)))
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

/// Records that the animation `name` and the entry `entry` of `scripts.animate` were made of
/// the controller at `from` (`at` is the description): what is told of them from here on is
/// told at the controller.
fn made_of(cx: &mut UpgradeCx<'_>, at: &JsonPath, from: JsonPath, name: &str, entry: usize) {
    let mut entry_at = below(at, &["scripts", "animate"]);
    entry_at.push_index(entry);
    cx.diagnostics
        .moved(from.clone(), below(at, &["animations", name]));
    cx.diagnostics.moved(from, entry_at);
}

/// The note of the animation `name` a controller writes over, where the file has it: the
/// element and key of `animation_controllers` an earlier controller made it from (`made`),
/// or `animations` (`at` is the description).
fn written_over(at: &JsonPath, name: &str, made: Option<&(usize, String)>) -> Diagnostic {
    let replaced = made.map_or_else(
        || below(at, &["animations", name]),
        |(earlier, key)| controller_at(at, *earlier, key),
    );
    step_finding(
        DiagnosticCode::DroppedField { text: None },
        Severity::Info,
        replaced,
    )
}

/// The member `key` of the element `index` of `animation_controllers` (`at` is the
/// description).
/// The note of an `animation_controllers` element without a member: it moves nothing and
/// goes with the list (the crate's own note; the game is silent).
fn empty_element(at: &JsonPath, index: usize) -> Diagnostic {
    let mut element_at = below(at, &["animation_controllers"]);
    element_at.push_index(index);
    step_finding(
        DiagnosticCode::DroppedField { text: None },
        Severity::Info,
        element_at,
    )
}

fn controller_at(at: &JsonPath, index: usize, key: &str) -> JsonPath {
    let mut path = below(at, &["animation_controllers"]);
    path.push_index(index);
    path.push(key);
    path
}

/// `minecraft:client_entity/description`.
pub(super) fn description_at() -> JsonPath {
    below(&JsonPath::new(), &[read::PAYLOAD, "description"])
}

/// A finding of a step.
pub(super) fn step_finding(code: DiagnosticCode, severity: Severity, at: JsonPath) -> Diagnostic {
    let mut found = read::finding(code, severity, at);
    found.phase = Phase::Upgrade;
    found
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
            diagnostics: &mut diagnostics,
        });
        diagnostics
    }

    /// Where the file has the member the upgraded document has at `at` (below the
    /// description).
    fn in_the_file(told: &Diagnostics, at: &[&str]) -> String {
        told.source_of(&below(&description_at(), at)).to_string()
    }

    /// [`in_the_file`] for the entry `index` of `scripts.animate`.
    fn animate_in_the_file(told: &Diagnostics, index: usize) -> String {
        let mut at = below(&description_at(), &["scripts", "animate"]);
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
