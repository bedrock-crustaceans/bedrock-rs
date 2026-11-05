//! Format versions 1.10.0 up to 1.26.0, and the step to 1.26.0.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::id::{
    AnimationRef, EntityId, GeometryId, MaterialId, ParticleId, SoundEventName, TexturePath,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, StrictBool, StrictString, Value};
use bedrock_addon_core::molang::{Molang, MolangFields, MolangSource, MolangVisitor, Step};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};
use bedrock_addon_core::visual::history::v1_10_0::Scripts;
use bedrock_addon_core::visual::{ControllerUse, SpawnEgg};
use facet::Facet;
use indexmap::IndexMap;

use super::super::read;
use super::super::validate::{self, CONDITION, NAMED_SCRIPT};
use super::super::write;
use super::v1_8_0::description_at;

/// The version from which vanilla stops lower-casing the named scripts: a document written
/// below it has them lower-cased when it loads.
const NO_LOWER_CASING: FormatVersion = FormatVersion::new(1, 18, 20);

/// A client entity document of format version 1.10.0 up to 1.26.0.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientEntity {
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
    /// `queryable_geometry`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub queryable_geometry: Option<StrictString>,
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
    /// `render_controllers`: a name, or `{ name: condition }`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub render_controllers: Option<Vec<ControllerUse>>,
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
    const VERSION: FormatVersion = known::V1_10_0;
}

impl VersionStruct for ClientEntity {
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

impl MolangFields for ClientEntity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        let d = &mut self.description;
        visitor.enter(Step::Key(read::PAYLOAD));
        visitor.enter(Step::Key("description"));
        validate::visit_controllers(d.render_controllers.as_mut(), visitor);
        if let Some(scripts) = &mut d.scripts {
            visitor.enter(Step::Key("scripts"));
            visitor.field("initialize", &NAMED_SCRIPT, &mut scripts.initialize);
            visitor.field("pre_animation", &NAMED_SCRIPT, &mut scripts.pre_animation);
            visitor.field("scale", &NAMED_SCRIPT, &mut scripts.scale);
            visitor.field("scaleX", &NAMED_SCRIPT, &mut scripts.scale_x);
            visitor.field("scalex", &NAMED_SCRIPT, &mut scripts.scalex);
            visitor.field("scaleY", &NAMED_SCRIPT, &mut scripts.scale_y);
            visitor.field("scaley", &NAMED_SCRIPT, &mut scripts.scaley);
            visitor.field("scaleZ", &NAMED_SCRIPT, &mut scripts.scale_z);
            visitor.field("scalez", &NAMED_SCRIPT, &mut scripts.scalez);
            visitor.field("parent_setup", &NAMED_SCRIPT, &mut scripts.parent_setup);
            visitor.field(
                "should_update_bones_and_effects_offscreen",
                &NAMED_SCRIPT,
                &mut scripts.should_update_bones_and_effects_offscreen,
            );
            visitor.field(
                "should_update_effects_offscreen",
                &NAMED_SCRIPT,
                &mut scripts.should_update_effects_offscreen,
            );
            validate::visit_animate(scripts.animate.as_mut(), &CONDITION, visitor);
            visitor.leave();
        }
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
impl Upgrade for ClientEntity {
    type Next = super::super::ClientEntity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        // Named scripts are lower-cased below 1.18.20, as the game does when it loads them.
        let d = self.description;
        let fold = cx.original_version < NO_LOWER_CASING;
        let at = description_at().with(&["scripts"]);
        let scripts = d.scripts.map(|s| bedrock_addon_core::visual::Scripts {
            variables: s.variables,
            initialize: elements(s.initialize, "initialize", fold, &at, cx),
            pre_animation: elements(s.pre_animation, "pre_animation", fold, &at, cx),
            scale: named(s.scale, "scale", fold, &at, cx),
            scale_x: named(s.scale_x, "scaleX", fold, &at, cx),
            scalex: named(s.scalex, "scalex", fold, &at, cx),
            scale_y: named(s.scale_y, "scaleY", fold, &at, cx),
            scaley: named(s.scaley, "scaley", fold, &at, cx),
            scale_z: named(s.scale_z, "scaleZ", fold, &at, cx),
            scalez: named(s.scalez, "scalez", fold, &at, cx),
            parent_setup: named(s.parent_setup, "parent_setup", fold, &at, cx),
            should_update_bones_and_effects_offscreen: named(
                s.should_update_bones_and_effects_offscreen,
                "should_update_bones_and_effects_offscreen",
                fold,
                &at,
                cx,
            ),
            should_update_effects_offscreen: named(
                s.should_update_effects_offscreen,
                "should_update_effects_offscreen",
                fold,
                &at,
                cx,
            ),
            hide_held_items: None,
            animate: s.animate,
        });
        super::super::ClientEntity {
            format_version: <super::super::ClientEntity as Versioned>::VERSION,
            description: super::super::Description {
                identifier: d.identifier,
                min_engine_version: d.min_engine_version,
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
        }
    }
}

/// The elements of `initialize` or `pre_animation` as the strings 1.26.0 holds.
fn elements(
    list: Option<Vec<Molang>>,
    name: &str,
    fold: bool,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) -> Option<Vec<StrictString>> {
    let list = list?;
    let mut strings = Vec::with_capacity(list.len());
    for (index, element) in list.into_iter().enumerate() {
        let mut element_at = at.with(&[name]);
        element_at.push_index(index);
        if let Molang::Expr(source) = element {
            // The schema's `ExpressionNode` parses each element as written.
            validate::check_element(&source, &element_at, cx);
            if fold && has_cased_literal(source.as_str()) {
                cx.diagnostics.push(literal_folded(element_at));
            }
            strings.push(StrictString(folded(source.as_str(), fold)));
        } else {
            // The game joins the string elements and skips the ones that are not strings.
            cx.diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(element_at)
                    .phase(Phase::Upgrade),
            );
        }
    }
    Some(strings)
}

/// A named script with its text lower-cased when `fold` is set. A number or a boolean, which
/// builds no script, stays.
fn named(
    value: Option<Molang>,
    name: &str,
    fold: bool,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) -> Option<Molang> {
    match value {
        Some(Molang::Expr(source)) if fold => {
            let lower = folded(source.as_str(), true);
            if lower == source.as_str() {
                return Some(Molang::Expr(source));
            }
            if has_cased_literal(source.as_str()) {
                cx.diagnostics.push(literal_folded(at.with(&[name])));
            }
            Some(Molang::Expr(match source.raw_version() {
                Some(version) => MolangSource::string(lower, version),
                None => MolangSource::string_without_context(lower),
            }))
        }
        other => other,
    }
}

/// Whether a script's text has a single-quoted literal with an upper-case ASCII letter: the
/// one part of it the lower-casing changes the meaning of (Molang reads names in any case).
fn has_cased_literal(text: &str) -> bool {
    text.split('\'')
        .skip(1)
        .step_by(2)
        .any(|literal| literal.bytes().any(|byte| byte.is_ascii_uppercase()))
}

/// The note that the script at `at` is held and written with a literal in another case than
/// the file's: the game's own lower-casing, reported as an information because the game
/// discards the original case itself.
fn literal_folded(at: JsonPath) -> Diagnostic {
    Diagnostic::info(DiagnosticCode::ValueRewritten { text: None })
        .at(at)
        .phase(Phase::Upgrade)
}

/// The whole text lower-cased, quoted literals included, as the game does it.
fn folded(text: &str, fold: bool) -> String {
    if fold {
        text.to_ascii_lowercase()
    } else {
        text.to_owned()
    }
}
