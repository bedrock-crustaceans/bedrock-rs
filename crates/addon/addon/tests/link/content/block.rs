//! The references of a block (20 B5 to B7, B9 to B12) and the block rows that are not the
//! link pass's (B1 to B4, B13).

#![allow(
    unused_imports,
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]

use bedrock_addon::diagnostic::{ReferenceHint, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::hosted;
use super::{block, hint, one, said, said_of, text};

const NO_LINTS: LinkOptions = LinkOptions {
    lints: false,
    client: true,
};

/// `geometry.Lamp` with the bones `Body` and `shade`.
const LAMP_GEOMETRY: (&str, &str) = (
    "models/blocks/lamp.geo.json",
    r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.Lamp"}, "bones": [{"name": "Body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}]}, {"name": "shade"}]}]}"#,
);

/// A terrain atlas with the tile `demo_lamp`.
const TERRAIN: (&str, &str) = (
    "textures/terrain_texture.json",
    r#"{"texture_name": "atlas.terrain", "texture_data": {"demo_lamp": {"textures": "textures/blocks/lamp"}}}"#,
);

const FILE: &str = "blocks/lamp.json";

/// The link findings of a world whose behaviour pack must load `blocks` blocks: a fixture
/// the load rejects would make a test of "no finding" pass for nothing.
fn linked(
    blocks: usize,
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    load: LoadOptions,
    options: LinkOptions,
) -> bedrock_addon::diagnostic::Diagnostics {
    // The terrain atlas of these worlds is a pack the game ships: B9 is tested at
    // its own severity. `tests/link/view.rs` has the other views.
    let set = super::super::common::game_stacks(behavior, resource, load);
    let loaded = set.behavior.blocks();
    assert_eq!(
        loaded.len(),
        blocks,
        "{:#?}",
        loaded
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    set.link_with(&options)
}

/// [`linked`] with one block, no host and every rule.
fn link(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
) -> bedrock_addon::diagnostic::Diagnostics {
    let blocks = behavior
        .iter()
        .filter(|(path, _)| path.starts_with("blocks/"))
        .count();
    linked(
        blocks,
        behavior,
        resource,
        LoadOptions::default(),
        LinkOptions::default(),
    )
}

/// [`linked`] with the load options `load` and the link options `options`.
fn link_with(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    load: LoadOptions,
    options: LinkOptions,
) -> bedrock_addon::diagnostic::Diagnostics {
    let blocks = behavior
        .iter()
        .filter(|(path, _)| path.starts_with("blocks/"))
        .count();
    linked(blocks, behavior, resource, load, options)
}

// ---- B5: the geometry ----------------------------------------------------------------------

#[cfg(feature = "geometry")]
mod geometry {
    use super::*;

    const KINDS: [&str; 2] = ["geometry", "bone"];

    fn lamp(geometry: &str) -> String {
        block(
            "demo:lamp",
            &format!(
                r#""minecraft:geometry": {geometry}, "minecraft:material_instances": {{"*": {{"texture": "demo_lamp"}}}}"#
            ),
        )
    }

    #[test]
    fn b5_a_geometry_a_resource_pack_defines_is_no_finding() {
        for written in [r#""geometry.Lamp""#, r#"{"identifier": "geometry.Lamp"}"#] {
            let found = link(&[(FILE, &lamp(written))], &[LAMP_GEOMETRY]);
            assert_eq!(said(&found, &KINDS), [], "{written}");
        }
    }

    /// The identifier kind folds case: a reference
    /// that differs from the definition only in case resolves.
    #[test]
    fn b5_a_geometry_named_in_another_case_resolves_open_o3() {
        let found = link(&[(FILE, &lamp(r#""geometry.lamp""#))], &[LAMP_GEOMETRY]);
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b5_the_geometries_of_the_game_are_no_finding_with_or_without_a_host() {
        for name in [
            "minecraft:geometry.full_block",
            "minecraft:geometry.cross",
            "minecraft:geometry.full_block_v1",
        ] {
            let file = lamp(&format!(r#""{name}""#));
            let found = link(&[(FILE, &file)], &[]);
            assert_eq!(said(&found, &KINDS), [], "{name}");
            // A host that knows no geometry is not asked about the two the row names.
            let found = link_with(&[(FILE, &file)], &[], hosted(&[]), LinkOptions::default());
            assert_eq!(said(&found, &KINDS), [], "{name} hosted");
        }
        // What the host says the game has is the game's.
        let file = lamp(r#""geometry.host_only""#);
        let host = hosted(&[("geometry", "geometry.host_only")]);
        let found = link_with(&[(FILE, &file)], &[], host, LinkOptions::default());
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b5_a_geometry_nothing_defines_is_an_information_at_the_member_that_names_it() {
        let found = link(&[(FILE, &lamp(r#""geometry.none""#))], &[LAMP_GEOMETRY]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                "minecraft:block/components/minecraft:geometry",
                "geometry",
                "geometry.none"
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "geometry")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        assert_eq!(text(finding), None);
        // The object form names it in `identifier`.
        let object = lamp(r#"{"identifier": "geometry.none"}"#);
        let found = link(&[(FILE, &object)], &[]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                "minecraft:block/components/minecraft:geometry/identifier",
                "geometry",
                "geometry.none"
            )]
        );
    }

    #[test]
    fn b5_is_a_lint_and_is_left_out_without_the_lints() {
        let file = lamp(r#""geometry.none""#);
        let found = link_with(&[(FILE, &file)], &[], LoadOptions::default(), NO_LINTS);
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b5_a_permutations_geometry_is_told_at_the_permutation() {
        let file = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}}, "components": {}, "permutations": [{"condition": "q.block_state('demo:lit') == true", "components": {"minecraft:geometry": "geometry.none", "minecraft:material_instances": {"*": {"texture": "demo_lamp"}}}}]}}"#;
        let found = link(&[(FILE, file)], &[]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                "minecraft:block/permutations/0/components/minecraft:geometry",
                "geometry",
                "geometry.none"
            )]
        );
    }

    // ---- B6: the bones ---------------------------------------------------------------------

    #[test]
    fn b6_a_bone_of_the_geometry_is_no_finding_whatever_its_case() {
        // The geometry keeps `Body` lower-cased; the block may write either.
        let file = lamp(
            r#"{"identifier": "geometry.lamp", "bone_visibility": {"body": true, "Shade": false, "BODY": true}}"#,
        );
        let found = link(&[(FILE, &file)], &[LAMP_GEOMETRY]);
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }

    /// The client's bone-visibility check logs
    /// "Geometry Component `bone_visibilities` bone %s is invalid, in block %s" at level 3
    /// and goes on with the expression: a kept error.
    #[test]
    fn b6_a_bone_the_geometry_does_not_have_is_a_kept_error() {
        let file = lamp(
            r#"{"identifier": "geometry.lamp", "bone_visibility": {"body": true, "tail": false}}"#,
        );
        let found = link(&[(FILE, &file)], &[LAMP_GEOMETRY]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                "minecraft:block/components/minecraft:geometry/bone_visibility/tail",
                "bone",
                "tail"
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "bone")
            .unwrap();
        assert!(finding.kept, "{finding:?}");
        // A warning is told without the lints too.
        let found = link_with(
            &[(FILE, &file)],
            &[LAMP_GEOMETRY],
            LoadOptions::default(),
            NO_LINTS,
        );
        assert_eq!(said(&found, &KINDS).len(), 1);
    }

    #[test]
    fn b6_the_bones_of_a_geometry_no_pack_defines_are_not_judged() {
        // Only the geometry is missing (B5): its bones cannot be known.
        let file = lamp(r#"{"identifier": "geometry.none", "bone_visibility": {"tail": false}}"#);
        let found = link(&[(FILE, &file)], &[LAMP_GEOMETRY]);
        let found = said(&found, &KINDS);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_eq!(found[0].4, "geometry");
        // Nor are those of a geometry the host says the game has.
        let file = lamp(r#"{"identifier": "geometry.host", "bone_visibility": {"tail": false}}"#);
        let host = hosted(&[("geometry", "geometry.host")]);
        let found = link_with(&[(FILE, &file)], &[], host, LinkOptions::default());
        assert_eq!(said(&found, &KINDS), []);
    }
}

/// With the `geometry` feature off no geometry rule is compiled: nothing is told.
#[cfg(not(feature = "geometry"))]
#[test]
fn b5_b6_without_the_geometry_feature_nothing_is_told() {
    let file = block(
        "demo:lamp",
        r#""minecraft:geometry": {"identifier": "geometry.none", "bone_visibility": {"tail": false}}, "minecraft:material_instances": {"*": {"texture": "demo_lamp"}}"#,
    );
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["geometry", "bone"]), []);
}

// ---- B7: the culling shape -----------------------------------------------------------------

const SLAB: (&str, &str) = (
    "shapes/slab.json",
    r#"{"format_version": "1.21.110", "minecraft:voxel_shape": {"description": {"identifier": "demo:slab"}, "shape": {"boxes": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}}"#,
);

fn culled(shape: &str) -> String {
    block(
        "demo:lamp",
        &format!(
            r#""minecraft:geometry": {{"identifier": "minecraft:geometry.cross", "culling_shape": "{shape}"}}, "minecraft:material_instances": {{"*": {{"texture": "demo_lamp"}}}}"#
        ),
    )
}

#[test]
fn b7_a_shape_of_the_stack_and_the_two_of_the_game_are_no_finding() {
    for shape in ["demo:slab", "minecraft:empty", "minecraft:unit_cube"] {
        let found = link(&[(FILE, &culled(shape)), SLAB], &[]);
        assert_eq!(said(&found, &["voxel shape"]), [], "{shape}");
        // The two built-in shapes are the row's own: a host is not asked about them.
        let found = link_with(
            &[(FILE, &culled(shape)), SLAB],
            &[],
            hosted(&[]),
            LinkOptions::default(),
        );
        assert_eq!(said(&found, &["voxel shape"]), [], "{shape} hosted");
    }
    let host = hosted(&[("voxel shape", "host:shape")]);
    let found = link_with(
        &[(FILE, &culled("host:shape"))],
        &[],
        host,
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &["voxel shape"]), []);
}

/// The game logs the line at level 3
///  and assigns the default shape: a
/// kept error.
#[test]
fn b7_a_shape_nothing_defines_is_vanillas_kept_error() {
    let found = link_with(
        &[(FILE, &culled("demo:slabs")), SLAB],
        &[],
        LoadOptions::default(),
        NO_LINTS,
    );
    assert_eq!(
        said(&found, &["voxel shape"]),
        [one(
            "UnresolvedReference",
            Severity::Error,
            FILE,
            "minecraft:block/components/minecraft:geometry/culling_shape",
            "voxel shape",
            "demo:slabs"
        )]
    );
    let finding = found
        .iter()
        .find(|finding| said_of(finding).4 == "voxel shape")
        .unwrap();
    assert!(finding.kept, "{finding:?}");
    assert_eq!(
        text(finding).as_deref(),
        Some(
            "Voxel shape with name \"demo:slabs\" doesn't exist in the registry. Defaulting to minecraft:empty for identifier."
        )
    );
    assert_eq!(
        finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name),
        Some("bp 1")
    );
}

// ---- B9: the textures ----------------------------------------------------------------------

#[cfg(feature = "texture_atlas")]
mod texture {
    use super::*;

    const KINDS: [&str; 1] = ["atlas key"];

    fn textured(texture: &str) -> String {
        block(
            "demo:lamp",
            &format!(
                r#""minecraft:geometry": "minecraft:geometry.cross", "minecraft:material_instances": {{"*": {{"texture": "{texture}"}}, "up": "*"}}, "minecraft:destruction_particles": {{"texture": "{texture}"}}"#
            ),
        )
    }

    #[test]
    fn b9_a_tile_of_the_terrain_atlas_is_no_finding() {
        let found = link(&[(FILE, &textured("demo_lamp"))], &[TERRAIN]);
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }

    #[test]
    fn b9_a_tile_the_host_says_the_game_has_is_no_finding_and_without_a_host_it_is_missing() {
        let host = hosted(&[("atlas key", "stone")]);
        let found = link_with(
            &[(FILE, &textured("stone"))],
            &[],
            host,
            LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
        // The crate has no list of the game's tiles: without a host, and without the game's
        // resource pack on the stack, the game's own tile is missing like any other.
        let found = link(&[(FILE, &textured("stone"))], &[TERRAIN]);
        assert_eq!(said(&found, &KINDS).len(), 2);
    }

    /// a kind is judged only where the stack or a host has a view of it. Without
    /// `textures/terrain_texture.json` on the resource stack and without a host, a tile of
    /// the game cannot be told from a missing one, and nothing is said; with the atlas, or
    /// with a host, the same name is told as the row says.
    #[test]
    fn b9_a_stack_without_the_terrain_atlas_and_without_a_host_judges_no_tile() {
        let file = textured("demo_lamps");
        let tiles = |found: &bedrock_addon::diagnostic::Diagnostics| {
            said(found, &KINDS)
                .into_iter()
                .map(|said| (said.0, said.1, said.5))
                .collect::<Vec<_>>()
        };
        let told = vec![
            (
                "UnresolvedReference",
                Severity::Error,
                "demo_lamps".to_owned()
            );
            2
        ];
        // No atlas, no host: nothing is said of the kind.
        assert_eq!(tiles(&link(&[(FILE, &file)], &[])), []);
        // The atlas is on the stack and lacks the tile.
        assert_eq!(tiles(&link(&[(FILE, &file)], &[TERRAIN])), told);
        // A host answers for the kind: its no is believed.
        let host = hosted(&[("atlas key", "stone")]);
        let found = link_with(&[(FILE, &file)], &[], host, LinkOptions::default());
        assert_eq!(tiles(&found), told);
    }

    #[test]
    fn b9_a_tile_nothing_defines_is_an_error_at_each_member_that_names_it() {
        let found = link_with(
            &[(FILE, &textured("demo_lamps"))],
            &[TERRAIN],
            LoadOptions::default(),
            NO_LINTS,
        );
        assert_eq!(
            said(&found, &KINDS),
            [
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    "minecraft:block/components/minecraft:material_instances/*/texture",
                    "atlas key",
                    "demo_lamps"
                ),
                one(
                    "UnresolvedReference",
                    Severity::Error,
                    FILE,
                    "minecraft:block/components/minecraft:destruction_particles/texture",
                    "atlas key",
                    "demo_lamps"
                ),
            ]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "atlas key")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        assert_eq!(
            text(finding).as_deref(),
            Some(
                "-> components -> minecraft:material_instances -> * -> texture:  Missing referenced asset demo_lamps"
            )
        );
        assert_eq!(hint(finding), None);
    }

    /// A key is compared as written; the tile that differs only in case is the hint.
    #[test]
    fn b9_a_tile_named_in_another_case_is_missing_with_the_case_hint() {
        let found = link(&[(FILE, &textured("Demo_Lamp"))], &[TERRAIN]);
        let hints: Vec<_> = found
            .iter()
            .filter(|finding| said_of(finding).4 == "atlas key")
            .map(hint)
            .collect();
        assert_eq!(
            hints,
            [
                Some(ReferenceHint::CaseMismatch("demo_lamp".to_owned())),
                Some(ReferenceHint::CaseMismatch("demo_lamp".to_owned()))
            ]
        );
    }

    /// The alias half of the row: an instance that names another instance is no texture.
    #[test]
    fn b9_an_instance_that_names_another_instance_is_not_looked_up_in_the_atlas() {
        let file = block(
            "demo:lamp",
            r#""minecraft:geometry": "minecraft:geometry.cross", "minecraft:material_instances": {"*": {"texture": "demo_lamp"}, "up": "*", "side": {"texture": "demo_lamp"}, "north": "side"}"#,
        );
        let found = link(&[(FILE, &file)], &[TERRAIN]);
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }
}

#[cfg(not(feature = "texture_atlas"))]
#[test]
fn b9_without_the_texture_feature_nothing_is_told() {
    let file = block(
        "demo:lamp",
        r#""minecraft:geometry": "minecraft:geometry.cross", "minecraft:material_instances": {"*": {"texture": "demo_none"}}"#,
    );
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["atlas key"]), []);
}

// ---- B10: the loot table -------------------------------------------------------------------

#[cfg(feature = "loot")]
mod loot {
    use super::*;

    const KINDS: [&str; 1] = ["loot table"];
    const TABLE: (&str, &str) = ("loot_tables/blocks/lamp.json", r#"{"pools": []}"#);

    fn dropping(table: &str) -> String {
        block("demo:lamp", &format!(r#""minecraft:loot": "{table}""#))
    }

    #[test]
    fn b10_a_table_of_the_stack_is_no_finding_with_or_without_case_and_extension() {
        for written in [
            "loot_tables/blocks/lamp.json",
            "loot_tables/Blocks/Lamp.json",
            "loot_tables/blocks/lamp",
        ] {
            let found = link(&[(FILE, &dropping(written)), TABLE], &[]);
            assert_eq!(said(&found, &KINDS), [], "{written}");
        }
    }

    #[test]
    fn b10_a_table_the_host_says_the_game_has_is_no_finding() {
        let host = hosted(&[("loot table", "loot_tables/blocks/stone.json")]);
        let found = link_with(
            &[(FILE, &dropping("loot_tables/blocks/stone.json"))],
            &[],
            host,
            LinkOptions::default(),
        );
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b10_a_table_nothing_defines_is_an_error_with_the_validators_line() {
        // and drops nothing: a warning, not a silent lint.
        let file = dropping("loot_tables/blocks/lamps.json");
        let found = link(&[(FILE, &file), TABLE], &[]);
        assert_eq!(
            said(&found, &KINDS),
            [one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                "minecraft:block/components/minecraft:loot",
                "loot table",
                "loot_tables/blocks/lamps.json"
            )]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "loot table")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        assert_eq!(
            text(finding).as_deref(),
            Some(
                "-> components -> minecraft:loot:  Invalid asset path loot_tables/blocks/lamps.json"
            )
        );
        // No lint: told with the lints off too.
        let found = link_with(&[(FILE, &file)], &[], LoadOptions::default(), NO_LINTS);
        assert_eq!(said(&found, &KINDS).len(), 1);
    }
}

#[cfg(not(feature = "loot"))]
#[test]
fn b10_without_the_loot_feature_nothing_is_told() {
    let file = block("demo:lamp", r#""minecraft:loot": "loot_tables/none.json""#);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["loot table"]), []);
}

// ---- B11: the localisation keys ------------------------------------------------------------

#[cfg(all(feature = "lang", feature = "languages"))]
mod lang {
    use super::*;

    const KINDS: [&str; 1] = ["lang key"];
    const LANGUAGES: (&str, &str) = ("texts/languages.json", r#"["en_US"]"#);

    fn named(display: &str, table: &str) -> String {
        block(
            "demo:lamp",
            &format!(
                r#""minecraft:display_name": "{display}", "minecraft:crafting_table": {{"crafting_tags": ["demo_bench"], "table_name": "{table}"}}"#
            ),
        )
    }

    #[test]
    fn b11_a_key_a_lang_file_of_either_stack_defines_is_no_finding() {
        let file = named("tile.demo.lamp.name", "Demo.Bench");
        // The resource stack's file, the behaviour stack's file; keys compare without case.
        let resource = ("texts/en_US.lang", "tile.demo.lamp.name=Lamp\n");
        let behavior = ("texts/en_US.lang", "demo.bench=Bench\n");
        let found = link(
            &[(FILE, &file), LANGUAGES, behavior],
            &[LANGUAGES, resource],
        );
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }

    /// K115: the usual add-on has `texts/en_US.lang` and no `texts/languages.json`; the
    /// game loads the file all the same (its own packs list `en_US`), so its keys resolve.
    #[test]
    fn b11_a_key_of_a_lang_file_in_a_pack_without_languages_json_is_no_finding() {
        let file = named("tile.demo.lamp.name", "Demo.Bench");
        let resource = ("texts/en_US.lang", "tile.demo.lamp.name=Lamp\n");
        let behavior = ("texts/en_US.lang", "demo.bench=Bench\n");
        let found = link(&[(FILE, &file), behavior], &[resource]);
        assert_eq!(said(&found, &KINDS), [], "{found:#?}");
    }

    #[test]
    fn b11_a_key_the_host_says_the_game_has_is_no_finding() {
        let file = named("tile.stone.name", "tile.stone.name");
        let host = hosted(&[("lang key", "tile.stone.name")]);
        let found = link_with(&[(FILE, &file)], &[], host, LinkOptions::default());
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b11_a_key_no_lang_file_defines_is_an_information() {
        let file = named("tile.demo.lamp.name", "demo.bench");
        let found = link(&[(FILE, &file)], &[]);
        assert_eq!(
            said(&found, &KINDS),
            [
                one(
                    "UnresolvedLangKey",
                    Severity::Info,
                    FILE,
                    "minecraft:block/components/minecraft:display_name",
                    "lang key",
                    "tile.demo.lamp.name"
                ),
                one(
                    "UnresolvedLangKey",
                    Severity::Info,
                    FILE,
                    "minecraft:block/components/minecraft:crafting_table/table_name",
                    "lang key",
                    "demo.bench"
                ),
            ]
        );
        let finding = found
            .iter()
            .find(|finding| said_of(finding).4 == "lang key")
            .unwrap();
        assert_eq!(
            finding
                .location
                .pack
                .as_ref()
                .map(bedrock_addon::pack::PackRef::name),
            Some("bp 1")
        );
        // A lint.
        let found = link_with(&[(FILE, &file)], &[], LoadOptions::default(), NO_LINTS);
        assert_eq!(said(&found, &KINDS), []);
    }

    #[test]
    fn b11_a_crafting_table_without_a_name_names_no_key() {
        let file = block(
            "demo:lamp",
            r#""minecraft:crafting_table": {"crafting_tags": ["demo_bench"]}"#,
        );
        let found = link(&[(FILE, &file)], &[]);
        assert_eq!(said(&found, &KINDS), []);
    }
}

#[cfg(not(all(feature = "lang", feature = "languages")))]
#[test]
fn b11_without_the_text_feature_nothing_is_told() {
    let file = block("demo:lamp", r#""minecraft:display_name": "tile.none""#);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(said(&found, &["lang key"]), []);
}

// ---- B12: the block descriptors ------------------------------------------------------------

const BLOCK_KINDS: [&str; 2] = ["block", "block state"];

fn placed_on(filter: &str) -> String {
    block(
        "demo:lamp",
        &format!(
            r#""minecraft:placement_filter": {{"conditions": [{{"allowed_faces": ["up"], "block_filter": [{filter}]}}]}}"#
        ),
    )
}

const BASE: (&str, &str) = (
    "blocks/base.json",
    r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:base", "states": {"demo:lit": [false, true]}}, "components": {}}}"#,
);

#[test]
fn b12_a_block_of_the_stack_or_of_the_game_is_no_finding() {
    let file = placed_on(
        r#""demo:base", {"name": "demo:base", "states": {"demo:lit": true}}, "minecraft:stone", "stone""#,
    );
    let found = link(&[(FILE, &file), BASE], &[]);
    assert_eq!(said(&found, &BLOCK_KINDS), [], "{found:#?}");
    let host = hosted(&[("block", "host:block")]);
    let found = link_with(
        &[(FILE, &placed_on(r#""host:block""#))],
        &[],
        host,
        LinkOptions::default(),
    );
    assert_eq!(said(&found, &BLOCK_KINDS), []);
}

#[test]
fn b12_a_block_nothing_defines_is_an_error_at_the_member_that_names_it() {
    const FILTER: &str =
        "minecraft:block/components/minecraft:placement_filter/conditions/0/block_filter";

    let file = placed_on(r#""demo:none", {"name": "demo:nothing"}"#);
    let found = link_with(&[(FILE, &file)], &[], LoadOptions::default(), NO_LINTS);
    assert_eq!(
        said(&found, &BLOCK_KINDS),
        [
            one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                &format!("{FILTER}/0"),
                "block",
                "demo:none"
            ),
            one(
                "UnresolvedReference",
                Severity::Error,
                FILE,
                &format!("{FILTER}/1/name"),
                "block",
                "demo:nothing"
            ),
        ]
    );
    let finding = found
        .iter()
        .find(|finding| said_of(finding).4 == "block")
        .unwrap();
    assert_eq!(
        finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name),
        Some("bp 1")
    );
    // The game's link pass holds the descriptor: its line.
    assert_eq!(
        text(finding).as_deref(),
        Some(
            "-> components -> minecraft:placement_filter -> conditions[0] -> block_filter[0]:  Missing referenced asset demo:none"
        )
    );
}

/// The states are the crate's own check: the game's block descriptor resolution has only
/// internal assertions for them, no content-log line: informations.
#[test]
fn b12_a_state_the_block_does_not_have_and_a_value_it_cannot_take_are_informations() {
    const FILTER: &str =
        "minecraft:block/components/minecraft:placement_filter/conditions/0/block_filter";

    let file = placed_on(
        r#"{"name": "demo:base", "states": {"demo:dim": true}}, {"name": "demo:base", "states": {"demo:lit": 3}}"#,
    );
    let found = link(&[(FILE, &file), BASE], &[]);
    assert_eq!(
        said(&found, &BLOCK_KINDS),
        [
            one(
                "UnresolvedReference",
                Severity::Info,
                FILE,
                &format!("{FILTER}/0/states/demo:dim"),
                "block state",
                "demo:dim"
            ),
            one(
                "InvalidReference",
                Severity::Info,
                FILE,
                &format!("{FILTER}/1/states/demo:lit"),
                "block state",
                "demo:lit"
            ),
        ]
    );
}

/// `{tags}` is a Molang expression: the game evaluates it, the pass does not read it.
#[test]
fn b12_a_tag_expression_stays_run() {
    let file = placed_on(r#"{"tags": "q.any_tag('demo:no_such_tag')"}"#);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

#[test]
fn b12_the_blocks_a_corner_placement_connects_with_are_resolved() {
    let file = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "traits": {"minecraft:placement_direction": {"enabled_states": ["minecraft:corner_and_cardinal_direction"], "blocks_to_corner_with": ["minecraft:stone", "demo:none"]}}}, "components": {}}}"#;
    let found = link(&[(FILE, file)], &[]);
    assert_eq!(
        said(&found, &BLOCK_KINDS),
        [one(
            "UnresolvedReference",
            Severity::Error,
            FILE,
            "minecraft:block/description/traits/minecraft:placement_direction/blocks_to_corner_with/1",
            "block",
            "demo:none"
        )],
        "{found:#?}"
    );
}

// ---- the rows the load checks: B9's alias half, B13's deprecated triggers ---------------------

/// The findings of the load of the one block `file`, as (code, severity, JSON path, name).
fn load_references(file: &str) -> Vec<(&'static str, Severity, String, String)> {
    let set = super::super::common::stacks(&[(FILE, file)], &[], LoadOptions::default());
    let blocks = set.behavior.blocks();
    assert_eq!(
        blocks.len(),
        1,
        "{:#?}",
        blocks
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    blocks
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(|finding| {
            assert_eq!(finding.phase, bedrock_addon::diagnostic::Phase::Validate);
            let id = match finding.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Reference { id, .. }) => {
                    id.clone()
                }
                _ => String::new(),
            };
            (
                finding.code.name(),
                finding.severity,
                finding.location.json_path.to_string(),
                id,
            )
        })
        .collect()
}

/// 20 B9, the alias half (a load row): a material instance that is the name of another
/// instance must name one of the same component.
#[test]
fn b9_an_instance_that_names_no_instance_is_a_load_information() {
    let file = block(
        "demo:lamp",
        r#""minecraft:geometry": "minecraft:geometry.cross", "minecraft:material_instances": {"*": {"texture": "demo_lamp"}, "up": "*", "side": {"texture": "demo_lamp"}, "north": "side", "south": "sides"}"#,
    );
    assert_eq!(
        load_references(&file),
        [(
            "UnresolvedReference",
            Severity::Info,
            "minecraft:block/components/minecraft:material_instances/south".to_owned(),
            "sides".to_owned()
        )]
    );
    // The pass itself tells nothing of an alias.
    let found = link(&[(FILE, &file)], &[TERRAIN]);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

/// 20 B13, the deprecated half (a load row): a deprecated trigger names an event of the
/// block's own event handlers.
#[test]
fn b13_a_deprecated_trigger_that_names_no_event_of_the_block_is_a_load_information() {
    // The legacy triggers and events of an allow-listed pack's legacy file (no cereal
    // document has `deprecated` or `deprecated_event_handlers`), each told
    // at the member the file has.
    let file = r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {
        "minecraft:on_interact": {"event": "demo:missing"}, "minecraft:on_step_on": {"event": "demo:there"}, "minecraft:on_placed": {"event": "demo:elsewhere", "target": "other"}, "minecraft:random_ticking": {"on_tick": {"event": "demo:gone"}}},
        "events": {"demo:there": {"sequence": [{"trigger": {"event": "demo:nowhere"}}, {"trigger": {"event": "demo:there"}}]}}}}"#;
    assert_eq!(
        allow_listed_load_references(file),
        [
            (
                "UnresolvedReference",
                Severity::Info,
                "minecraft:block/components/minecraft:on_interact/event".to_owned(),
                "demo:missing".to_owned()
            ),
            (
                "UnresolvedReference",
                Severity::Info,
                "minecraft:block/components/minecraft:random_ticking/on_tick/event".to_owned(),
                "demo:gone".to_owned()
            ),
            (
                "UnresolvedReference",
                Severity::Info,
                "minecraft:block/events/demo:there/sequence/0/trigger/event".to_owned(),
                "demo:nowhere".to_owned()
            ),
        ]
    );
    let found = linked_allow_listed_stack(file);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

/// [`load_references`] for the one block `file` in an allow-listed pack.
fn allow_listed_load_references(file: &str) -> Vec<(&'static str, Severity, String, String)> {
    let pack = super::super::common::behavior_pack(LEGACY_ALLOWED, &[(FILE, file)]);
    let set = super::super::common::stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let blocks = set.behavior.blocks();
    assert_eq!(
        blocks.len(),
        1,
        "{:#?}",
        blocks
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    blocks
        .diagnostics()
        .filter(|finding| finding.code.name() == "UnresolvedReference")
        .map(|finding| {
            let id = match finding.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Reference { id, .. }) => {
                    id.clone()
                }
                _ => String::new(),
            };
            (
                finding.code.name(),
                finding.severity,
                finding.location.json_path.to_string(),
                id,
            )
        })
        .collect()
}

/// The link findings of the one block `file` in an allow-listed pack.
fn linked_allow_listed_stack(file: &str) -> bedrock_addon::diagnostic::Diagnostics {
    let pack = super::super::common::behavior_pack(LEGACY_ALLOWED, &[(FILE, file)]);
    let set = super::super::common::stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    set.link_with(&NO_LINTS)
}

/// 20 B13, the custom-component half, and B4: a custom component is the scripts' to
/// register, and a state named inside a Molang expression is the game's to look up when the
/// expression runs. Neither the load nor the pass tells of them.
#[test]
fn b13_a_custom_component_stays_run() {
    let file = block(
        "demo:lamp",
        r#""demo:no_such_component": {"block": "demo:none"}"#,
    );
    assert_eq!(load_references(&file), []);
    let found = link(&[(FILE, &file)], &[]);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

#[test]
fn b4_a_state_named_in_a_molang_expression_stays_run() {
    let file = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}}, "components": {}, "permutations": [{"condition": "q.block_state('demo:no_such_state') == true", "components": {"minecraft:friction": 0.5}}]}}"#;
    assert_eq!(load_references(file), []);
    let found = link(&[(FILE, file)], &[]);
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

/// 20 B12: the item and feature members of a block archetype are the base game's alone and
/// are not looked up.
#[test]
fn b12_the_members_of_a_block_archetype_are_not_checked() {
    let file = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp", "internal_vanilla_data": {"block_archetype": {"minecraft:crop": {"seed_item": "demo:no_item", "grows_into_feature": "demo:no_feature"}}}}, "components": {}}}"#;
    let set = super::super::common::stacks(&[(FILE, file)], &[], LoadOptions::default());
    let found = set.link();
    assert_eq!(found.iter().map(said_of).collect::<Vec<_>>(), []);
}

// ---- the index of a permutation is the file's ------------------------------------------------

/// The paths of the findings about a reference of one of `kinds`, in order.
fn paths(found: &bedrock_addon::diagnostic::Diagnostics, kinds: &[&str]) -> Vec<String> {
    said(found, kinds).into_iter().map(|said| said.3).collect()
}

/// A permutation behind one the load takes out (`minecraft:breathability` below its legacy
/// release, without the experiment, skips its permutation with the block kept; a
/// `minecraft:sound` there refuses the block since (d)): what the link pass finds in it is at the index the file has it
/// at.
#[test]
fn a_finding_in_a_permutation_behind_a_skipped_one_is_at_the_index_of_the_file() {
    const FILTER: &str = "components/minecraft:placement_filter/conditions/0/block_filter/0";

    // A legacy file: no component is gated in a cereal one.
    let file = r#"{"format_version": "1.21.60", "minecraft:block": {
        "description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}},
        "components": {},
        "permutations": [
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == true", "components": {
                "minecraft:placement_filter": {"conditions": [{"allowed_faces": ["up"], "block_filter": ["demo:none"]}]}}},
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == true", "components": {
                "minecraft:placement_filter": {"conditions": [{"allowed_faces": ["up"], "block_filter": ["demo:nothing"]}]}}}
        ]}}"#;
    let set = super::super::common::game_stacks(&[(FILE, file)], &[], LoadOptions::default());
    let blocks = set.behavior.blocks();
    let block = blocks
        .iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                blocks
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1;
    assert_eq!(block.permutations.len(), 2, "{:#?}", block.diagnostics());
    let found = set.link_with(&NO_LINTS);
    assert_eq!(
        paths(&found, &BLOCK_KINDS),
        [
            format!("minecraft:block/permutations/1/{FILTER}"),
            format!("minecraft:block/permutations/3/{FILTER}"),
        ],
        "{found:#?}"
    );
}

/// The same for a block of an older file, whose permutation is taken out by the last step
/// of the chain (a legacy component of a pack that is not allow-listed), with either value.
/// A rule of the load (`minecraft:sound` below its release) does not skip: it refuses the
/// block.
#[test]
fn a_finding_in_a_permutation_of_an_older_file_behind_a_skipped_one_is_at_the_index_of_the_file() {
    const FILTER: &str = "components/minecraft:placement_filter/conditions/0/block_filter/0";

    for skipped in [
        r#"{"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}}"#,
        r#"{"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "solid"}}"#,
    ] {
        let file = format!(
            r#"{{"format_version": "1.21.60", "minecraft:block": {{
            "description": {{"identifier": "demo:lamp", "states": {{"demo:lit": [false, true]}}}},
            "components": {{}},
            "permutations": [
                {skipped},
                {{"condition": "q.block_state('demo:lit') == true", "components": {{
                    "minecraft:placement_filter": {{"conditions": [{{"allowed_faces": ["up"], "block_filter": ["demo:none"]}}]}}}}}},
                {skipped},
                {{"condition": "q.block_state('demo:lit') == true", "components": {{
                    "minecraft:placement_filter": {{"conditions": [{{"allowed_faces": ["up"], "block_filter": ["demo:nothing"]}}]}}}}}}
            ]}}}}"#
        );
        let set = super::super::common::game_stacks(&[(FILE, &file)], &[], LoadOptions::default());
        let blocks = set.behavior.blocks();
        let block = blocks
            .iter()
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "{:#?}",
                    blocks
                        .diagnostics()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            })
            .1;
        assert_eq!(
            block.permutations.len(),
            2,
            "{skipped}: {:#?}",
            block.diagnostics()
        );
        let found = set.link_with(&NO_LINTS);
        assert_eq!(
            paths(&found, &BLOCK_KINDS),
            [
                format!("minecraft:block/permutations/1/{FILTER}"),
                format!("minecraft:block/permutations/3/{FILTER}"),
            ],
            "{skipped}"
        );
    }
}

/// Both at once: the step takes out a permutation and a rule one of those left; the link
/// pass names the last by the index the file has it at.
#[test]
fn a_finding_behind_permutations_a_step_and_a_rule_skip_is_at_the_index_of_the_file() {
    const FILTER: &str = "components/minecraft:placement_filter/conditions/0/block_filter/0";

    let file = r#"{"format_version": "1.21.60", "minecraft:block": {
        "description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}},
        "components": {},
        "permutations": [
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == true", "components": {
                "minecraft:placement_filter": {"conditions": [{"allowed_faces": ["up"], "block_filter": ["demo:none"]}]}}},
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:breathability": "air"}},
            {"condition": "q.block_state('demo:lit') == true", "components": {
                "minecraft:placement_filter": {"conditions": [{"allowed_faces": ["up"], "block_filter": ["demo:nothing"]}]}}}
        ]}}"#;
    let set = super::super::common::game_stacks(&[(FILE, file)], &[], LoadOptions::default());
    let blocks = set.behavior.blocks();
    let block = blocks
        .iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                blocks
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1;
    assert_eq!(block.permutations.len(), 2, "{:#?}", block.diagnostics());
    let found = set.link_with(&NO_LINTS);
    assert_eq!(
        paths(&found, &BLOCK_KINDS),
        [
            format!("minecraft:block/permutations/2/{FILTER}"),
            format!("minecraft:block/permutations/5/{FILTER}"),
        ]
    );
}

/// A file with a number no double holds is refused (the game's "number too big" parse error):
/// the block never loads, so no permutation of it is checked or linked.
#[test]
fn a_number_no_double_holds_refuses_the_block_before_any_permutation_is_checked() {
    // The number in a custom component's parameters, which every version keeps.
    let file = r#"{"format_version": "1.21.60", "minecraft:block": {
        "description": {"identifier": "demo:lamp", "states": {"demo:lit": [false, true]}},
        "components": {"demo:note": {"n": 1e999}},
        "permutations": [
            {"condition": "q.block_state('demo:lit') == false", "components": {"minecraft:sound": "stone"}},
            {"condition": "q.block_state('demo:lit') == true", "components": {
                "minecraft:placement_filter": {"conditions": [{"allowed_faces": ["up"], "block_filter": ["demo:none"]}]}}}
        ]}}"#;
    let set = super::super::common::game_stacks(&[(FILE, file)], &[], LoadOptions::default());
    let blocks = set.behavior.blocks();
    assert!(blocks.iter().next().is_none(), "the block is refused");
    let found = set.link_with(&NO_LINTS);
    assert_eq!(paths(&found, &BLOCK_KINDS), Vec::<String>::new());
}

/// A block of an older file whose permutations an upgrade step rebuilds (the legacy
/// components of an allow-listed pack move to `deprecated`, a `queued_ticking` with them)
/// and the retry of the load leaves some out of (a condition the block-state rule rejects and the default rule compiles): the moves of the step and the elements the rule keeps compose. What the load,
/// the upgrade and the Molang walk tell, and what the link pass finds, is at the path the
/// file has, through either door: the load, and the upgrade of the document as written.
#[cfg(feature = "molang")]
#[test]
fn a_finding_behind_a_permutation_left_out_and_inside_one_a_step_rebuilds_is_at_the_path_of_the_file()
 {
    const AT: &str = "minecraft:block/permutations";

    const TICKING: &str = "components/minecraft:queued_ticking";

    const FILTER: &str = "components/minecraft:placement_filter/conditions/0/block_filter/0";

    use bedrock_addon::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon::document::{Content, JsonDocument};
    use bedrock_addon::families::block::{Block, history};
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::pack::PackType;
    use bedrock_addon::storage::PackPath;
    // One of the packs allowed the legacy creator features.
    const ALLOW_LISTED: u128 = 0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166;
    const LEFT_OUT: &str =
        r#"{"condition": "q.is_baby", "components": {"minecraft:friction": 0.4}}"#;
    let ticking = |name: &str, filter: &str| {
        format!(
            r#"{{"condition": "q.block_property('demo:lit') == true", "components": {{
                "minecraft:queued_ticking": {{"interval_range": [1, 2], "looping": true, "bogus": 1,
                    "on_tick": {{"event": "{name}", "condition": "1 +"}}}},
                "minecraft:light_dampening": 16,
                "minecraft:placement_filter": {{"conditions": [{{"allowed_faces": ["up"], "block_filter": ["{filter}"]}}]}}}}}}"#
        )
    };
    let file = format!(
        r#"{{"format_version": "1.18.0", "minecraft:block": {{
        "description": {{"identifier": "demo:lamp", "properties": {{"demo:lit": [false, true]}}}},
        "components": {{}},
        "permutations": [{LEFT_OUT}, {}, {LEFT_OUT}, {}]}}}}"#,
        ticking("demo:tick", "demo:none"),
        ticking("demo:tock", "demo:nothing"),
    );
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        uuid::Uuid::from_u128(ALLOW_LISTED),
        PackType::Behavior,
        Some(bedrock_addon::version::MinEngineVersion::new(1, 26, 20)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx.molang = Some(std::sync::Arc::new(
        bedrock_addon::molang::MolangCheck::new(),
    ));
    let told = |found: &Diagnostics| -> Vec<String> {
        let mut told: Vec<String> = found
            .iter()
            .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
            .collect();
        told.sort();
        told.dedup();
        told
    };
    let mut expected = vec![
        format!("DroppedField {AT}/0"),
        format!("DroppedField {AT}/2"),
        // The block-state rule's rejection of the condition: a kept error.
        format!("InvalidMolang {AT}/0/condition"),
        format!("InvalidMolang {AT}/2/condition"),
    ];
    for index in [1, 3] {
        // The step's own warning, at the component the file has.
        expected.push(format!("DeprecatedMember {AT}/{index}/{TICKING}"));
        // Told by the reading of the rebuilt permutation, of `deprecated/queued_ticking`.
        expected.push(format!("UnrecognizedField {AT}/{index}/{TICKING}/bogus"));
        expected.push(format!(
            "UnresolvedReference {AT}/{index}/{TICKING}/on_tick/event"
        ));
        // The Molang walk, of the rebuilt member.
        expected.push(format!(
            "InvalidMolang {AT}/{index}/{TICKING}/on_tick/condition"
        ));
        // A rule of 1.26.20, told of the permutations as held once some are left out.
        expected.push(format!(
            "InvalidValue {AT}/{index}/components/minecraft:light_dampening"
        ));
    }
    expected.sort();
    let path = PackPath::from_static("blocks/lamp.json");
    let loaded = Block::parse(&path, file.as_bytes(), &cx).expect("the block loads");
    assert_eq!(loaded.permutations.len(), 2);
    assert_eq!(told(loaded.diagnostics()), expected);
    // A member of the second permutation held, inside what the step rebuilt.
    let mut member = JsonPath::new();
    for key in ["minecraft:block", "permutations"] {
        member.push(key);
    }
    member.push_index(1);
    for key in ["deprecated", "queued_ticking", "on_tick"] {
        member.push(key);
    }
    let written = Block::parse_as_written(&path, file.as_bytes(), &cx).expect("it is read");
    // Nothing is known of the file's places before the upgrade: a held path is itself.
    assert_eq!(written.diagnostics().place_of(&member), member);
    let upgraded = history::upgrade(written, &cx).expect("the block loads");
    assert_eq!(told(upgraded.diagnostics()), expected);
    for document in [&loaded, &upgraded] {
        assert_eq!(
            document.diagnostics().place_of(&member).to_string(),
            format!("{AT}/3/{TICKING}/on_tick")
        );
    }
    // The link pass, over the loaded block.
    let pack = super::super::common::behavior_pack(ALLOW_LISTED, &[(FILE, &file)]);
    let set = super::super::common::stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let found = set.link_with(&NO_LINTS);
    assert_eq!(
        paths(&found, &BLOCK_KINDS),
        [format!("{AT}/1/{FILTER}"), format!("{AT}/3/{FILTER}")],
        "{found:#?}"
    );
}

// ---- a member a step renamed inside a component is named as the file has it ----------------

/// A block of format 1.18.0 whose `minecraft:loot` is the object form (`{"loot_table": …}`)
/// in its own components and in a permutation behind one the load takes out: the 1.19.10
/// unwrap moves the table to the component itself, and the link pass names the member the
/// file has, `…/minecraft:loot/loot_table`, at the permutation's index in the file.
#[cfg(all(feature = "loot", feature = "molang"))]
#[test]
fn a_loot_table_an_unwrap_moved_is_named_at_the_member_the_file_has() {
    let file = r#"{"format_version": "1.18.0", "minecraft:block": {
        "description": {"identifier": "demo:lamp", "properties": {"demo:lit": [false, true]}},
        "components": {"minecraft:loot": {"loot_table": "loot_tables/blocks/none.json"}},
        "permutations": [
            {"condition": "q.is_baby", "components": {"minecraft:friction": 0.4}},
            {"condition": "q.block_property('demo:lit') == true", "components": {"minecraft:loot": {"loot_table": "loot_tables/blocks/nothing.json"}}}
        ]}}"#;
    let set = super::super::common::game_stacks(&[(FILE, file)], &[], molang_options());
    let blocks = set.behavior.blocks();
    let block = blocks
        .iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                blocks
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1;
    assert_eq!(block.permutations.len(), 1, "{:#?}", block.diagnostics());
    let found = set.link_with(&LinkOptions::default());
    assert_eq!(
        paths(&found, &["loot table"]),
        [
            "minecraft:block/components/minecraft:loot/loot_table",
            "minecraft:block/permutations/1/components/minecraft:loot/loot_table",
        ],
        "{found:#?}"
    );
}

/// The options of a load that compiles Molang: a block of a format below 1.19.70 drops a
/// permutation whose condition the block-state rule rejects and the default rule compiles
/// (`q.is_baby`; the game's retry). A `minecraft:sound`
/// below its release does not take a permutation out: it refuses the block.
#[cfg(feature = "molang")]
fn molang_options() -> LoadOptions {
    LoadOptions {
        molang: Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        )),
        ..LoadOptions::default()
    }
}

/// One of the packs allowed the legacy creator features: a component released after the
/// file's format with `requires = [LCF]` loads in it.
const LEGACY_ALLOWED: u128 = 0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166;

/// The link findings of the 1.18.0 block `file` in an allow-listed pack, which must load with
/// one permutation.
#[cfg(feature = "molang")]
fn linked_allow_listed(file: &str) -> bedrock_addon::diagnostic::Diagnostics {
    let pack = super::super::common::behavior_pack(LEGACY_ALLOWED, &[(FILE, file)]);
    let set = super::super::common::stacks_of(vec![pack], Vec::new(), molang_options());
    let blocks = set.behavior.blocks();
    let block = blocks
        .iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                blocks
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1;
    assert_eq!(block.permutations.len(), 1, "{:#?}", block.diagnostics());
    set.link_with(&LinkOptions::default())
}

/// A 1.18.0 block whose own components and permutation 1 (behind one the load takes out)
/// hold `components`.
#[cfg(feature = "molang")]
fn allow_listed_block(components: &str) -> String {
    format!(
        r#"{{"format_version": "1.18.0", "minecraft:block": {{
        "description": {{"identifier": "demo:lamp", "properties": {{"demo:lit": [false, true]}}}},
        "components": {{{components}}},
        "permutations": [
            {{"condition": "q.is_baby", "components": {{"minecraft:friction": 0.4}}}},
            {{"condition": "q.block_property('demo:lit') == true", "components": {{{components}}}}}
        ]}}}}"#
    )
}

/// In an allow-listed pack the 1.19.10 unwrap of `minecraft:display_name` and the rename of
/// the crafting table's `custom_description` reach the link pass: B11 names the members the
/// file has.
#[cfg(all(feature = "lang", feature = "languages", feature = "molang"))]
#[test]
fn a_display_name_and_a_table_name_a_step_moved_in_an_allow_listed_pack_are_named_as_the_file_has_them()
 {
    let file = allow_listed_block(
        r#""minecraft:display_name": {"display_name": "demo.lamp"}, "minecraft:crafting_table": {"crafting_tags": ["demo_bench"], "custom_description": "demo.bench"}"#,
    );
    let found = linked_allow_listed(&file);
    let mut told = paths(&found, &["lang key"]);
    told.sort();
    assert_eq!(
        told,
        [
            "minecraft:block/components/minecraft:crafting_table/custom_description",
            "minecraft:block/components/minecraft:display_name/display_name",
            "minecraft:block/permutations/1/components/minecraft:crafting_table/custom_description",
            "minecraft:block/permutations/1/components/minecraft:display_name/display_name",
        ],
        "{found:#?}"
    );
}

/// The same for the unwrap of `minecraft:geometry`: B5 names the identifier the file has.
#[cfg(all(feature = "geometry", feature = "molang"))]
#[test]
fn a_geometry_identifier_an_unwrap_moved_in_an_allow_listed_pack_is_named_as_the_file_has_it() {
    let file = allow_listed_block(
        r#""minecraft:geometry": {"geometry": {"identifier": "geometry.none"}}"#,
    );
    let found = linked_allow_listed(&file);
    assert_eq!(
        paths(&found, &["geometry"]),
        [
            "minecraft:block/components/minecraft:geometry/geometry/identifier",
            "minecraft:block/permutations/1/components/minecraft:geometry/geometry/identifier",
        ],
        "{found:#?}"
    );
}
