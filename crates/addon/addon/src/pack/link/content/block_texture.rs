//! The terrain-atlas tiles a block's materials name.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::Id;

use super::super::{At, Findings, LinkCx, Target, Validator, View, kinds, unresolved};
use super::block::{placed, sections};
use crate::families::block::Block;
use crate::families::block::component::{
    DestructionParticles, EmbeddedVisual, ItemVisual, MaterialInstanceEntry, MaterialInstances,
};

/// The `texture` of each material instance and of `minecraft:destruction_particles`
/// is a key of the terrain atlas (`textures/terrain_texture.json`). An instance that names
/// another instance is the block's own to resolve, when it loads.
///
/// The tiles are judged by the view the pass has of the terrain atlas
/// ([`LinkCx::terrain_atlas_view`]): without the atlas file on the resource stack and
/// without a host nothing is said of any tile; with an atlas of an add-on's own alone a
/// miss is an information (the tile may be the game's); with the game's atlas or a host it
/// is the linked-asset check's error. `blocks.json` is judged the same way.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    let view = cx.terrain_atlas_view(out);
    if !view.judges() || (view == View::Partial && !cx.lints()) {
        return;
    }
    for (_, document) in cx.behavior().blocks() {
        for (section, components) in sections(document) {
            if let Some(instances) = components.get::<MaterialInstances>() {
                let path = at![&section; "minecraft:material_instances"];
                materials(cx, out, view, document, &path, instances);
            }
            if let Some(visual) = components.get::<ItemVisual>() {
                let path = at![&section; "minecraft:item_visual", "material_instances"];
                materials(cx, out, view, document, &path, &visual.material_instances);
            }
            if let Some(visual) = components.get::<EmbeddedVisual>() {
                let path = at![&section; "minecraft:embedded_visual", "material_instances"];
                materials(cx, out, view, document, &path, &visual.material_instances);
            }
            if let Some(particles) = components.get::<DestructionParticles>()
                && let Some(texture) = &particles.texture
            {
                let path = at![&section; "minecraft:destruction_particles", "texture"];
                tile(cx, out, view, document, &path, &texture.0);
            }
        }
    }
}

/// The textures of the instances of one `material_instances` object the block holds at `path`.
fn materials(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    view: View,
    document: &Document<Block>,
    path: &JsonPath,
    instances: &MaterialInstances,
) {
    for (name, entry) in &instances.0 {
        if let MaterialInstanceEntry::Instance(instance) = entry
            && let Some(texture) = &instance.texture
        {
            tile(cx, out, view, document, &at![path; name, "texture"], &texture.0);
        }
    }
}

/// One texture name against the terrain atlas.
fn tile(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    view: View,
    document: &Document<Block>,
    held: &JsonPath,
    name: &str,
) {
    if let Target::Missing(hint) = cx.terrain_atlas_key(&Id::parse(name)) {
        // The linked-asset check's grading, an information under a partial view.
        let severity = view.severity(Severity::Error);
        let place = At::new(document.origin(), placed(document, held), severity)
            .validator(Validator::Missing, 1, name);
        out.push(unresolved(place, kinds::ATLAS_KEY, name, hint));
    }
}
