//! What the `Debug` of the loaded collections leaves out, read through public accessors, so
//! that the comparison of two loads sees it (`collections.rs`, "What the text does not show").
//! Each value gives one [`Unprinted`]: its name and the part its `Debug` is silent about.
//!
//! Two sorts. Five values of single families (a material's variants, what a geometry
//! inherits, a jigsaw metadata body, a structure's layout, what a UI definition inherits).
//! And every registry and path map: their `Debug` prints the effective documents, and of a
//! registry's beta registry the identifiers of the effective ones. [`registries`] gives one
//! value per registry and path map with the whole table behind it, from
//! `Registry::all_layers` and `Files::all_layers`: every identifier or path anything is kept
//! of, whether or not it has an effective document, with every layer in order; the same of
//! the beta registry, with its rejected files and findings.
//!
//! The driver crate includes this file from `bedrock_addon`'s tests, as it does
//! `collections.rs`. It names
//! the content modules of `bedrock_addon`, each of which is behind a feature there and always
//! compiled in for the driver crate's tests. So the file that includes it defines, before
//! its `mod unprinted;`, the macro `family!("<feature>" => …)` over a block or an item: in
//! `bedrock_addon` it keeps the block or item where the feature is on, in the driver crate it
//! keeps every one.
//!
//! Three of the five values are of documents in a registry or a path map, and a layer that
//! is not effective is of the same type: one function per type gives the part its `Debug`
//! leaves out ([`body_of`], [`bytes_of`], [`inherited_by`]), for the effective document's
//! value and for every other layer in the table alike. With
//! a partial feature set the values of the families compiled in are compared, whatever the
//! others are.
//!
//! [`registries`] lists the accessors by hand. `tests/determinism.rs` checks the list
//! against the stacks: every registry and path map their `Debug` shows, at any depth, is one
//! this file gave a value for (`every_registry_and_path_map_gives_its_unprinted_part`).

#![allow(
    dead_code,
    unused_variables,
    reason = "each test file uses part of it, and a family's block is behind its feature"
)]

use std::fmt::{Debug, Write as _};

use bedrock_addon::document::{Content, Definition, Document, Files, Registry};
use bedrock_addon::pack::StackSet;

use crate::collections::Unprinted;

/// The start of the name of each kind of value, in the order [`of`] gives them. Of the last
/// kind there is one per registry and path map, with nothing in it where nothing is kept.
pub const KINDS: [&str; 6] = [
    "the variants of material ",
    "what geometry ",
    "the body of worldgen/metadata/",
    "the bytes structures/",
    "the body of the UI definition ",
    LAYERS,
];

/// The start of the name of a registry's or a path map's value: `<that> <Collection> of the
/// <side> stack`, or `<Collection>.<member>` for a member of a group.
pub const LAYERS: &str = "the layers of ";

/// The parts the `Debug` of `set`'s collections leaves out, in the order of [`KINDS`] and
/// within a kind in the collection's own order.
pub fn of(set: &StackSet) -> Vec<Unprinted> {
    let mut found = Vec::new();
    // A material prints how many variants it has: the names, its own and the inherited ones,
    // one per line.
    family!("material" => {
        use bedrock_addon::families::material::MaterialGroupKind;
        let materials = set.resource.materials();
        for kind in [
            MaterialGroupKind::Common,
            MaterialGroupKind::Fancy,
            MaterialGroupKind::Sad,
        ] {
            for material in materials.group(kind).iter() {
                let variants: Vec<&str> = material.variants().collect();
                found.push(Unprinted::new(
                    format!(
                        "the variants of material {} of the {kind:?} group",
                        material.name
                    ),
                    variants.join("\n"),
                ));
            }
        }
    });
    // What a geometry inherits prints as an identifier and two numbers: every geometry of
    // the chain with its own bones, nearest parent first.
    family!("geometry" => {
        let geometries = set.resource.geometries();
        for (id, geometry) in geometries {
            if let Some(chain) = inherited_by(geometry) {
                found.push(Unprinted::new(
                    format!("what geometry {} inherits", id.as_str()),
                    chain,
                ));
            }
        }
    });
    // A jigsaw metadata file prints the length of its body: the bytes.
    family!("jigsaw" => {
        let jigsaw = set.behavior.jigsaw();
        for (path, file) in &jigsaw.metadata {
            found.push(Unprinted::new(
                format!("the body of {path}"),
                body_of(file).unwrap_or_default(),
            ));
        }
    });
    // A structure prints of its layout only whether the file's tree is kept. The layout is
    // what makes the written file the read one (tag order, integer widths, the bytes after
    // the root tag): the bytes the structure is written as show all of it.
    family!("structure" => {
        let structures = set.behavior.structures();
        for (path, file) in structures {
            found.push(Unprinted::new(
                format!("the bytes {path} is written as"),
                bytes_of(file).unwrap_or_default(),
            ));
        }
    });
    // A UI definition prints the name of the definition it inherits from: the flattened
    // definition, its own members over everything its chain hands on.
    family!("json_ui" => {
        let ui = set.resource.ui();
        for control in ui.controls() {
            // A definition without a base inherits nothing: its `Debug` is all of it.
            if control.key.base.is_none() {
                continue;
            }
            found.push(Unprinted::new(
                format!(
                    "the body of the UI definition {}.{}",
                    control.namespace, control.key.name
                ),
                format!("{:?}", control.body()),
            ));
        }
    });
    registries(set, &mut found);
    found
}

// The bytes of a jigsaw metadata body, which its `Debug` gives the length of.
family!("jigsaw" => #[expect(
    clippy::unnecessary_wraps,
    reason = "`files` takes readers that return `Option`"
)]
fn body_of(
    file: &Document<bedrock_addon::families::jigsaw::JigsawMetadata>,
) -> Option<String> {
    Some(file.value().body.escape_ascii().to_string())
});

// The bytes a structure is written as. Its `Debug` prints of the layout only whether the
// file's tree is kept; the layout is what makes the written file the read one (tag order,
// integer widths, the bytes after the root tag), so the written bytes show all of it.
family!("structure" => #[expect(
    clippy::unnecessary_wraps,
    reason = "`files` takes readers that return `Option`"
)]
fn bytes_of(
    file: &Document<bedrock_addon::families::structure::Structure>,
) -> Option<String> {
    Some(match file.value().to_bytes() {
        Ok(bytes) => bytes.escape_ascii().to_string(),
        Err(error) => format!("not written: {error:?}"),
    })
});

// What a geometry inherits, which its `Debug` gives as an identifier and two numbers: every
// geometry of the chain with its own bones, nearest parent first. `None` for a geometry
// without a parent (a layer that is not effective has none resolved).
family!("geometry" => fn inherited_by(
    geometry: &Document<bedrock_addon::families::geometry::Geometry>,
) -> Option<String> {
    let inherited = geometry.inherited.as_ref()?;
    let mut chain = String::new();
    for parent in inherited.chain() {
        writeln!(chain, "{}: {:?}", parent.identifier(), parent.bones())
            .expect("a string takes any text");
    }
    Some(chain)
});

/// The part of a layer its `Debug` leaves out, for the types that have none.
fn nothing<D>(_: &D) -> Option<String> {
    None
}

/// One slot of a table: its key, how many layers it has and whether the first is the
/// effective one, then each layer by its full `Debug` (the pack and path are in its origin)
/// and by `rest`, the part that `Debug` leaves out. With `first_is_printed` the first layer
/// is not repeated: it is the entry the collection's own `Debug` prints under this key, and
/// what that leaves out is a value of its own.
fn slot<'a, D: Debug + 'a>(
    text: &mut String,
    key: &str,
    first_is_effective: bool,
    first_is_printed: bool,
    layers: impl Iterator<Item = &'a D>,
    rest: &dyn Fn(&D) -> Option<String>,
) {
    let layers: Vec<&D> = layers.collect();
    let first = if first_is_effective {
        "effective"
    } else {
        "not effective"
    };
    writeln!(text, "{key}: {} layers, the first {first}", layers.len())
        .expect("a string takes any text");
    for (position, layer) in layers
        .iter()
        .enumerate()
        .skip(usize::from(first_is_printed))
    {
        writeln!(text, "  {position}: {layer:?}").expect("a string takes any text");
        if let Some(rest) = rest(layer) {
            writeln!(text, "  {position}, not in its Debug: {rest}")
                .expect("a string takes any text");
        }
    }
}

/// The whole table of a registry: every identifier anything is kept of, in load order, with
/// its layers (the effective document itself is in the registry's `Debug` and not repeated),
/// each with `rest`; then the beta registry's table with every layer in full, its rejected
/// files and its findings.
fn registry<T>(
    at: &str,
    registry: &Registry<T>,
    rest: &dyn Fn(&Document<T>) -> Option<String>,
    found: &mut Vec<Unprinted>,
) where
    T: Definition + Debug,
    T::Beta: Debug,
{
    let mut text = String::new();
    for (id, layers) in registry.all_layers() {
        let effective = registry.get(id).is_some();
        slot(
            &mut text,
            &format!("{id:?}"),
            effective,
            effective,
            layers,
            rest,
        );
    }
    if let Some(beta) = registry.beta() {
        for (id, layers) in beta.all_layers() {
            let effective = beta.get(id).is_some();
            slot(
                &mut text,
                &format!("beta {id:?}"),
                effective,
                false,
                layers,
                &nothing,
            );
        }
        if !beta.rejected().is_empty() {
            writeln!(text, "rejected by the beta registry: {:?}", beta.rejected())
                .expect("a string takes any text");
        }
        let findings: Vec<_> = beta.diagnostics().collect();
        if !findings.is_empty() {
            writeln!(text, "findings of the beta registry: {findings:?}")
                .expect("a string takes any text");
        }
    }
    found.push(Unprinted::new(format!("{LAYERS}{at}"), text));
}

/// The whole table of a path map: every path a copy is kept of, with its layers, each with
/// `rest`.
fn files<T: Content + Debug>(
    at: &str,
    files: &Files<T>,
    rest: &dyn Fn(&Document<T>) -> Option<String>,
    found: &mut Vec<Unprinted>,
) {
    let mut text = String::new();
    for (path, layers) in files.all_layers() {
        let effective = files.get(path).is_some();
        slot(
            &mut text,
            &format!("{path}"),
            effective,
            effective,
            layers,
            rest,
        );
    }
    found.push(Unprinted::new(format!("{LAYERS}{at}"), text));
}

/// One value per registry and path map of both stacks, in accessor-list order, a group's
/// members in the order of its fields.
#[allow(clippy::too_many_lines, reason = "one line per registry and path map")]
#[allow(
    clippy::ptr_arg,
    reason = "a build without a family has no registry to push"
)]
fn registries(set: &StackSet, found: &mut Vec<Unprinted>) {
    const B: &str = "of the behavior stack";
    const R: &str = "of the resource stack";
    let (b, r) = (&set.behavior, &set.resource);
    family!("block" => {
        registry(&format!("Blocks {B}"), b.blocks(), &nothing, found);
    });
    family!("item" => {
        registry(&format!("Items {B}"), b.items(), &nothing, found);
    });
    family!("entity" => {
        registry(&format!("Entities {B}"), b.entities(), &nothing, found);
    });
    family!("biome" => {
        registry(&format!("Biomes {B}"), b.biomes(), &nothing, found);
    });
    family!("spawn_rules" => {
        registry(&format!("SpawnRules {B}"), b.spawn_rules(), &nothing, found);
    });
    family!("spawn_groups" => {
        registry(&format!("SpawnGroups {B}"), b.spawn_groups(), &nothing, found);
    });
    family!("feature" => {
        registry(&format!("Features {B}"), b.features(), &nothing, found);
    });
    family!("jigsaw" => {
        let group = b.jigsaw();
        registry(&format!("Jigsaw.structures {B}"), &group.structures, &nothing, found);
        registry(&format!("Jigsaw.pools {B}"), &group.pools, &nothing, found);
        registry(&format!("Jigsaw.structure_sets {B}"), &group.structure_sets, &nothing, found);
        registry(&format!("Jigsaw.processors {B}"), &group.processors, &nothing, found);
        files(&format!("Jigsaw.metadata {B}"), &group.metadata, &body_of, found);
    });
    family!("recipe" => {
        let group = b.recipes();
        registry(&format!("Recipes.crafting {B}"), &group.crafting, &nothing, found);
        registry(&format!("Recipes.furnace {B}"), &group.furnace, &nothing, found);
        registry(&format!("Recipes.brewing_mix {B}"), &group.brewing_mix, &nothing, found);
        registry(&format!("Recipes.brewing_container {B}"), &group.brewing_container, &nothing, found);
        registry(&format!("Recipes.material_reduction {B}"), &group.material_reduction, &nothing, found);
        registry(&format!("Recipes.smithing_transform {B}"), &group.smithing_transform, &nothing, found);
        registry(&format!("Recipes.smithing_trim {B}"), &group.smithing_trim, &nothing, found);
    });
    family!("animation" => {
        registry(&format!("Animations {B}"), b.animations(), &nothing, found);
    });
    family!("animation" => {
        registry(&format!("AnimationControllers {B}"), b.animation_controllers(), &nothing, found);
    });
    family!("feature_rules" => {
        let group = b.feature_rules();
        registry(&format!("FeatureRules.pregeneration_pass {B}"), &group.pregeneration_pass, &nothing, found);
        registry(&format!("FeatureRules.first_pass {B}"), &group.first_pass, &nothing, found);
        registry(&format!("FeatureRules.before_underground_pass {B}"), &group.before_underground_pass, &nothing, found);
        registry(&format!("FeatureRules.underground_pass {B}"), &group.underground_pass, &nothing, found);
        registry(&format!("FeatureRules.after_underground_pass {B}"), &group.after_underground_pass, &nothing, found);
        registry(&format!("FeatureRules.before_surface_pass {B}"), &group.before_surface_pass, &nothing, found);
        registry(&format!("FeatureRules.surface_pass {B}"), &group.surface_pass, &nothing, found);
        registry(&format!("FeatureRules.after_surface_pass {B}"), &group.after_surface_pass, &nothing, found);
        registry(&format!("FeatureRules.before_sky_pass {B}"), &group.before_sky_pass, &nothing, found);
        registry(&format!("FeatureRules.sky_pass {B}"), &group.sky_pass, &nothing, found);
        registry(&format!("FeatureRules.after_sky_pass {B}"), &group.after_sky_pass, &nothing, found);
        registry(&format!("FeatureRules.final_pass {B}"), &group.final_pass, &nothing, found);
        // The registries of the passes the game does not define, inside the member's map.
        for (key, inside) in &group.unknown_passes {
            registry(&format!("FeatureRules.unknown_passes[{key}] {B}"), inside, &nothing, found);
        }
    });
    family!("dimension" => {
        registry(&format!("Dimensions {B}"), b.dimensions(), &nothing, found);
    });
    family!("poi" => {
        let group = b.poi();
        registry(&format!("Poi.types {B}"), &group.types, &nothing, found);
        registry(&format!("Poi.tags {B}"), &group.tags, &nothing, found);
    });
    family!("dialogue" => {
        registry(&format!("Dialogue {B}"), b.dialogue(), &nothing, found);
    });
    family!("trade" => {
        files(&format!("TradeTables {B}"), b.trade_tables(), &nothing, found);
    });
    family!("structure" => {
        files(&format!("Structures {B}"), b.structures(), &bytes_of, found);
    });
    family!("loot" => {
        files(&format!("LootTables {B}"), b.loot_tables(), &nothing, found);
    });
    family!("function" => {
        registry(&format!("Functions {B}"), b.functions(), &nothing, found);
    });
    family!("behavior_tree" => {
        registry(&format!("BehaviorTrees {B}"), b.behavior_trees(), &nothing, found);
    });
    family!("voxel_shape" => {
        registry(&format!("VoxelShapes {B}"), b.voxel_shapes(), &nothing, found);
    });
    family!("camera" => {
        let group = b.cameras();
        registry(&format!("Cameras.presets {B}"), &group.presets, &nothing, found);
        registry(&format!("Cameras.lists {B}"), &group.lists, &nothing, found);
    });
    family!("aim_assist" => {
        let group = b.aim_assist();
        registry(&format!("AimAssist.presets {B}"), &group.presets, &nothing, found);
        registry(&format!("AimAssist.categories {B}"), &group.categories, &nothing, found);
    });
    family!("server_sound_definitions" => {
        registry(&format!("ServerSoundDefinitions {B}"), b.server_sound_definitions(), &nothing, found);
    });
    family!("client_biome" => {
        registry(&format!("ClientBiomes {R}"), r.client_biomes(), &nothing, found);
    });
    family!("animation" => {
        registry(&format!("Animations {R}"), r.animations(), &nothing, found);
    });
    family!("animation_controller" => {
        registry(&format!("AnimationControllers {R}"), r.animation_controllers(), &nothing, found);
    });
    family!("geometry" => {
        registry(&format!("Geometries {R}"), r.geometries(), &inherited_by, found);
    });
    family!("deferred_settings" => {
        let group = r.deferred_settings();
        registry(&format!("DeferredSettings.lighting {R}"), &group.lighting, &nothing, found);
        registry(&format!("DeferredSettings.atmosphere {R}"), &group.atmosphere, &nothing, found);
        registry(&format!("DeferredSettings.water {R}"), &group.water, &nothing, found);
        registry(&format!("DeferredSettings.color_grading {R}"), &group.color_grading, &nothing, found);
        registry(&format!("DeferredSettings.cubemaps {R}"), &group.cubemaps, &nothing, found);
        files(&format!("DeferredSettings.shadows {R}"), &group.shadows, &nothing, found);
        files(&format!("DeferredSettings.pbr_fallback {R}"), &group.pbr_fallback, &nothing, found);
    });
    family!("blocks_json" => {
        registry(&format!("ClientBlocks {R}"), r.client_blocks(), &nothing, found);
    });
    family!("block_culling" => {
        registry(&format!("BlockCulling {R}"), r.block_culling(), &nothing, found);
    });
    family!("ddui" => {
        let group = r.data_driven_ui();
        registry(&format!("DataDrivenUi.roots {R}"), &group.roots, &nothing, found);
        registry(&format!("DataDrivenUi.compositions {R}"), &group.compositions, &nothing, found);
        registry(&format!("DataDrivenUi.styles {R}"), &group.styles, &nothing, found);
    });
    family!("ui_texture_info" => {
        files(&format!("UiTextureInfos {R}"), r.ui_texture_infos(), &nothing, found);
    });
    family!("render_controller" => {
        registry(&format!("RenderControllers {R}"), r.render_controllers(), &nothing, found);
    });
    family!("fog" => {
        registry(&format!("Fogs {R}"), r.fogs(), &nothing, found);
    });
    family!("client_entity" => {
        registry(&format!("ClientEntities {R}"), r.client_entities(), &nothing, found);
    });
    family!("atomic_client_entity" => {
        registry(&format!("AtomicClientEntities {R}"), r.atomic_client_entities(), &nothing, found);
    });
    family!("attachable" => {
        registry(&format!("Attachables {R}"), r.attachables(), &nothing, found);
    });
    family!("particle" => {
        registry(&format!("Particles {R}"), r.particles(), &nothing, found);
    });
}
