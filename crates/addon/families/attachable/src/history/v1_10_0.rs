//! Format versions 1.10.0 up to 1.26.0, and the step to 1.26.0.

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::id::{
    AnimationRef, AttachableId, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_shared_visual::history::v1_10_0::FoldedLiterals;
use bedrock_addon_shared_visual::history::v1_10_0::Scripts;
use bedrock_addon_shared_visual::{ControllerUse, SpawnEgg};
use facet::Facet;
use indexmap::IndexMap;

use super::super::ItemBinding;
use super::super::read;
use super::super::validate::{self};
use super::super::write;
use super::v1_8_0::description_at;

/// An attachable document of format version 1.10.0 up to 1.26.0.
#[derive(Clone, Debug, PartialEq)]
pub struct Attachable {
    /// `format_version` as written (1.10.0 for a document upgraded from 1.8.0).
    pub format_version: FormatVersion,
    /// `description`.
    pub description: Description,
}

/// `description` at format versions 1.10.0 up to 1.26.0: as the newest
/// [`Description`](super::super::Description), except that each `scripts.initialize` and
/// `scripts.pre_animation` element is a Molang value of its own and `held_item_scale` and
/// `scripts.hide_held_items` do not exist.
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
    /// `queryable_geometry`.
    pub queryable_geometry: Option<String>,
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
    /// `render_controllers`: a name, or `{ name: condition }`.
    pub render_controllers: Option<Vec<ControllerUse>>,
    /// `sound_effects`.
    pub sound_effects: Option<IndexMap<String, SoundEventName>>,
    /// `enable_attachables`.
    pub enable_attachables: Option<bool>,
    /// `held_item_ignores_lighting`.
    pub held_item_ignores_lighting: Option<bool>,
    /// `hide_armor`.
    pub hide_armor: Option<bool>,
    /// `item`: the item the attachable is drawn for; first declared at 1.10.0.
    pub item: Option<ItemBinding>,
}

impl Versioned for Attachable {
    const VERSION: FormatVersion = known::V1_10_0;
}

impl VersionStruct for Attachable {
    fn parse(document: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        // The 1.10 schema.
        let described = read::describe(document, cx, d)?;
        let description: Description = read::typed(&described, false, &[], true, d)?;
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
        read::check_controllers(de.render_controllers.as_deref(), at, d);
        read::check_item(de.item.as_ref(), &described.json, at, d);
        if let Some(scripts) = &de.scripts {
            let scripts_at = at.with(&["scripts"]);
            read::check_variables(scripts.variables.as_ref(), &scripts_at, d);
            read::check_animate(scripts.animate.as_deref(), &scripts_at, d);
        }
        // A dropped member is a warning; only an error fails the schema.
        let failed = d
            .iter()
            .skip(before)
            .any(|found| found.severity == Severity::Error);
        (!failed).then_some(Self {
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
        let d = &mut self.description;
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        validate::visit_controllers(d.render_controllers.as_mut(), visitor);
        if let Some(scripts) = &mut d.scripts {
            visitor.enter(Step::Key("scripts"));
            scripts.visit_molang(visitor);
            visitor.leave();
        }
        validate::visit_item(&mut d.item, visitor);
        visitor.leave();
        visitor.leave();
    }
}

/// The 1.10 → 1.26 step. Vanilla rewrites nothing at 1.26.0; the version selects another
/// schema. So the step is the typed conversion and the one rule the loader keys on the
/// original version:
/// - each `initialize` and `pre_animation` element, a Molang value of its own until now, is
///   checked as one (a rejected element fails the document, as the 1.10 schema does) and
///   becomes a plain string; an element that is a number or a boolean, which the loader skips,
///   is dropped;
/// - in a document written below 1.18.20 the text of every named script is lower-cased, as
///   the loader does before it parses them. `animate` and `render_controllers` conditions and
///   variable names stay as written.
impl Upgrade for Attachable {
    type Next = super::super::Attachable;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        // The shared step: a literal the lower-casing below 1.18.20 changes is folded and not
        // told (D4).
        let d = self.description;
        let at = description_at().with(&["scripts"]);
        let scripts = d
            .scripts
            .map(|scripts| scripts.upgrade(FoldedLiterals::Silent, &at, cx));
        super::super::Attachable {
            format_version: <super::super::Attachable as Versioned>::VERSION,
            description: super::super::Description {
                identifier: d.identifier,
                min_engine_version: d.min_engine_version,
                slots: bedrock_addon_shared_visual::VisualSlots {
                    materials: d.materials.unwrap_or_default(),
                    textures: d.textures.unwrap_or_default(),
                    geometry: d.geometry.unwrap_or_default(),
                    queryable_geometry: d.queryable_geometry,
                    spawn_egg: d.spawn_egg,
                    scripts,
                    particle_effects: d.particle_effects,
                    particle_emitters: d.particle_emitters,
                    animations: d.animations,
                    render_controllers: d.render_controllers,
                    sound_effects: d.sound_effects,
                    enable_attachables: d.enable_attachables,
                    held_item_ignores_lighting: d.held_item_ignores_lighting,
                    held_item_scale: None,
                    hide_armor: d.hide_armor,
                },
                item: d.item,
            },
            stack_bindings: Vec::new(),
        }
    }
}
