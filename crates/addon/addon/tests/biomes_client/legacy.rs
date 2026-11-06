//! `biomes_client.json`, merged over the resource stack with vanilla's
//! merge strategy and read from the merged value

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode as Code, Severity};
use bedrock_addon::document::{Content, Merged, Placed, RawLayer};
#[cfg(feature = "client_biome")]
use bedrock_addon::families::biomes_client::WaterSurface;
use bedrock_addon::families::biomes_client::{LegacyClientBiome, LegacyClientBiomes};
#[cfg(feature = "client_biome")]
use bedrock_addon::families::client_biome::ClientBiome;
use bedrock_addon::id::BiomeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::version::{BaseGameVersion, SemVersion};
use facet_value::Value;

use super::common::{biome, context, fixture, json, pack, path};

const FILE: &str = "biomes_client.json";

/// The water surface of `biome` after a client biome (if any) and the legacy file.
#[cfg(feature = "client_biome")]
fn effective(
    client: Option<&ClientBiome>,
    legacy: &LegacyClientBiomes,
    biome: &BiomeId,
) -> WaterSurface {
    let (color, opacity) = client.map_or((None, None), ClientBiome::water_surface);
    legacy.effective_water_surface(color, opacity, biome)
}
const PLAINS: &str = "minecraft:plains";

/// A user pack named `name` with `text` as its `biomes_client.json`.
fn user(uuid: u128, name: &str, text: &str) -> Pack {
    pack(uuid, name, PackOrigin::User, vec![(FILE, text.to_owned())])
}

/// The stack of `packs` (top first) in a world of `world`.
fn stack_in(packs: Vec<Pack>, world: BaseGameVersion) -> PackStack<Resource> {
    let options = LoadOptions {
        base_game_version: world,
        ..LoadOptions::default()
    };
    PackStack::<Resource>::of(packs, options).expect("a stack")
}

/// The stack of `packs` (top first), without a world version.
fn stack(packs: Vec<Pack>) -> PackStack<Resource> {
    stack_in(packs, BaseGameVersion::EMPTY)
}

/// The merged settings of files given lowest first, each in its own user pack.
fn merged(files: &[&str]) -> PackStack<Resource> {
    let packs: Vec<Pack> = files
        .iter()
        .enumerate()
        .map(|(index, text)| user(index as u128 + 1, &format!("pack{index}"), text))
        .rev()
        .collect();
    stack(packs)
}

fn legacy(stack: &PackStack<Resource>) -> &Merged<LegacyClientBiomes, RawLayer> {
    stack.legacy_client_biomes()
}

fn plains(stack: &PackStack<Resource>) -> &LegacyClientBiome {
    legacy(stack)
        .biomes
        .get(&BiomeId::from_static(PLAINS))
        .expect("a plains entry")
}

/// The merged JSON of the entry `key`.
fn raw(stack: &PackStack<Resource>, key: &str) -> Value {
    legacy(stack)
        .merged
        .as_object()
        .and_then(|root| root.get("biomes"))
        .and_then(Value::as_object)
        .and_then(|biomes| biomes.get(key))
        .cloned()
        .unwrap_or_else(|| panic!("no entry {key}"))
}

fn found(stack: &PackStack<Resource>, code: impl Fn(&Code) -> bool) -> Vec<Diagnostic> {
    legacy(stack)
        .diagnostics()
        .filter(|d| code(&d.code))
        .cloned()
        .collect()
}

fn ids(entry: &LegacyClientBiome) -> Vec<&str> {
    entry
        .fog_ids_to_merge
        .iter()
        .map(bedrock_addon::id::Id::as_str)
        .collect()
}

/// `#RRGGBB` as RGBA from 0 to 1.
fn rgba(r: u8, g: u8, b: u8) -> [f32; 4] {
    [
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        1.0,
    ]
}

/// `bc01_single_file_raw`: a first entry is copied as written, so its `fog_identifier` names
/// no fog; `bc07_bare_key`: the bare key is namespaced.
#[test]
fn bc01_single_file_raw_and_bc07_bare_key() {
    let stack = merged(&[
        r##"{"biomes": {"plains": {"fog_identifier": "minecraft:fog_plains", "water_surface_color": "#44AFF5"}}}"##,
    ]);
    let value = legacy(&stack);
    assert_eq!(value.biomes.len(), 1);
    assert!(value.default.is_none());
    let entry = plains(&stack);
    assert!(entry.fog_ids_to_merge.is_empty());
    assert!(entry.old_fog.is_none());
    assert_eq!(entry.water_surface_color, Some(rgba(0x44, 0xAF, 0xF5)));
    assert_eq!(entry.water_surface_transparency, None);
    let ignored = found(&stack, |c| matches!(c, Code::LegacyFogIgnored));
    assert_eq!(ignored.len(), 1, "{ignored:#?}");
    assert_eq!(ignored[0].severity, Severity::Info);
    assert_eq!(
        ignored[0].location.json_path.to_string(),
        "biomes/minecraft:plains/fog_identifier"
    );
    assert_eq!(ignored[0].location.pack.as_ref().unwrap().name(), "pack0");
    assert_eq!(legacy(&stack).diagnostics().count(), 1);
    // The merged JSON keeps the member under the namespaced key only.
    assert_eq!(
        value.merged,
        json(
            r##"{"biomes": {"minecraft:plains": {"fog_identifier": "minecraft:fog_plains", "water_surface_color": "#44AFF5"}}}"##
        )
    );
    // Its pack is the entry's origin, and the layer is the file as read.
    let origin = value.origin(&BiomeId::from_static(PLAINS)).unwrap();
    assert_eq!(origin.pack.name(), "pack0");
    assert_eq!(origin.path.as_str(), FILE);
    assert_eq!(value.layers().len(), 1);
    assert!(
        value.layers()[0]
            .json
            .as_object()
            .and_then(|root| root.get("biomes"))
            .and_then(Value::as_object)
            .is_some_and(|biomes| biomes.contains_key("plains"))
    );
    // An entry that lists its fogs itself has them, and is not warned about.
    let listed = merged(&[
        r#"{"biomes": {"plains": {"fog_identifier": "a:f", "fog_ids_to_merge": ["a:f", "b:g"]}}}"#,
    ]);
    assert_eq!(ids(plains(&listed)), ["a:f", "b:g"]);
    assert!(found(&listed, |c| matches!(c, Code::LegacyFogIgnored)).is_empty());
    // A `null` entry under a bare key is dropped, not renamed.
    let null = merged(&[r#"{"biomes": {"plains": null, "default": {}}}"#]);
    assert!(legacy(&null).biomes.is_empty());
    assert!(legacy(&null).default.is_some());
    assert_eq!(legacy(&null).merged, json(r#"{"biomes": {"default": {}}}"#));
}

/// `bc02_second_file_replaces_fog`.
#[test]
fn bc02_second_file_replaces_fog() {
    let stack = merged(&[
        r##"{"biomes": {"minecraft:plains": {"fog_identifier": "a:fog", "fog_color": "#000000"}}}"##,
        r#"{"biomes": {"minecraft:plains": {"fog_identifier": "b:fog", "inherit_from_prior_fog": false, "water_fog_distance": 3}}}"#,
    ]);
    assert_eq!(ids(plains(&stack)), ["b:fog"]);
    // The top's fog name stands; the old-style members of both files are gone.
    assert_eq!(
        raw(&stack, PLAINS),
        json(
            r#"{"fog_identifier": "b:fog", "fog_ids_to_merge": ["b:fog"], "inherit_from_prior_fog": false}"#
        )
    );
    assert!(plains(&stack).old_fog.is_none());
    assert_eq!(
        found(&stack, |c| matches!(c, Code::LegacyFogIgnored)).len(),
        1
    );
    assert_eq!(
        legacy(&stack)
            .origin(&BiomeId::from_static(PLAINS))
            .unwrap()
            .pack
            .name(),
        "pack1"
    );
    // Layers are top-first.
    let names: Vec<&str> = legacy(&stack)
        .layers()
        .iter()
        .map(|layer| layer.origin.pack.name())
        .collect();
    assert_eq!(names, ["pack1", "pack0"]);
}

/// `bc03_inherit_appends`.
#[test]
fn bc03_inherit_appends() {
    let bottom = r#"{"biomes": {"minecraft:plains": {"fog_identifier": "a:fog"}}}"#;
    let inherit = |id: &str| {
        format!(
            r##"{{"biomes": {{"minecraft:plains": {{"fog_identifier": "{id}", "inherit_from_prior_fog": true, "fog_color": "#ffffff", "water_surface_transparency": 0.5}}}}}}"##
        )
    };
    let two = merged(&[bottom, &inherit("b:fog")]);
    assert_eq!(ids(plains(&two)), ["b:fog"]);
    // The bottom's name stays; the top's other members are copied, its old-style fog is not.
    assert_eq!(
        raw(&two, PLAINS),
        json(
            r#"{"fog_identifier": "a:fog", "fog_ids_to_merge": ["b:fog"], "inherit_from_prior_fog": true, "water_surface_transparency": 0.5}"#
        )
    );
    assert_eq!(plains(&two).water_surface_transparency, Some(0.5));
    let three = merged(&[bottom, &inherit("b:fog"), &inherit("c:fog")]);
    assert_eq!(ids(plains(&three)), ["b:fog", "c:fog"]);
    assert_eq!(
        raw(&three, PLAINS)
            .as_object()
            .unwrap()
            .get("fog_identifier"),
        Some(&"a:fog".into())
    );
    // Replacing after inheriting starts over.
    let replaced = merged(&[
        bottom,
        &inherit("b:fog"),
        r#"{"biomes": {"minecraft:plains": {"fog_identifier": "d:fog"}}}"#,
    ]);
    assert_eq!(ids(plains(&replaced)), ["d:fog"]);
}

/// `bc04_default_remove_all_prior`: the `default` entry's flag discards every lower entry.
#[test]
fn bc04_default_remove_all_prior() {
    let bottom = r##"{"biomes": {"default": {"water_surface_color": "#000001"}, "a:one": {}, "a:two": {}, "a:three": {}}}"##;
    let top = |flag: &str| {
        format!(
            r#"{{"biomes": {{"default": {{"remove_all_prior_fog": {flag}}}, "b:mine": {{}}}}}}"#
        )
    };
    let cleared = merged(&[bottom, &top("true")]);
    let keys: Vec<&str> = legacy(&cleared)
        .biomes
        .keys()
        .map(bedrock_addon::id::Id::as_str)
        .collect();
    assert_eq!(keys, ["b:mine"]);
    assert_eq!(
        legacy(&cleared)
            .default
            .as_ref()
            .unwrap()
            .water_surface_color,
        None
    );
    assert!(legacy(&cleared).origin(&BiomeId::parse("a:one")).is_none());
    let discarded = found(&cleared, |c| matches!(c, Code::ShadowedDefinition { .. }));
    assert_eq!(discarded.len(), 1);
    assert_eq!(discarded[0].severity, Severity::Info);
    assert_eq!(discarded[0].location.pack.as_ref().unwrap().name(), "pack0");
    // Without the flag everything merges.
    let kept = merged(&[bottom, &top("false")]);
    assert_eq!(legacy(&kept).biomes.len(), 4);
    assert_eq!(
        legacy(&kept).default.as_ref().unwrap().water_surface_color,
        Some(rgba(0, 0, 1))
    );
    assert!(found(&kept, |c| matches!(c, Code::ShadowedDefinition { .. })).is_empty());
}

/// `bc05_remove_all_prior_on_biome`: the flag is read on `default` only.
#[test]
fn bc05_remove_all_prior_on_biome() {
    let stack = merged(&[
        r#"{"biomes": {"a:one": {}, "minecraft:plains": {"water_surface_transparency": 0.25}}}"#,
        r#"{"biomes": {"minecraft:plains": {"remove_all_prior_fog": true}}}"#,
    ]);
    assert_eq!(legacy(&stack).biomes.len(), 2);
    assert_eq!(plains(&stack).water_surface_transparency, Some(0.25));
    let none = found(&stack, |c| matches!(c, Code::NoEffect { .. }));
    assert_eq!(none.len(), 1);
    assert_eq!(none[0].severity, Severity::Info);
    assert_eq!(
        none[0].location.json_path.to_string(),
        "biomes/minecraft:plains/remove_all_prior_fog"
    );
    assert_eq!(none[0].location.pack.as_ref().unwrap().name(), "pack1");
    // On `default`, and `false` on a biome, it is no finding.
    let fine = merged(&[
        r#"{"biomes": {"default": {"remove_all_prior_fog": true}, "minecraft:plains": {"remove_all_prior_fog": false}}}"#,
    ]);
    assert!(found(&fine, |c| matches!(c, Code::NoEffect { .. })).is_empty());
}

/// `bc06_back_compat_fog`: old-style fog members over an entry make it a back-compat fog; the
/// lower file's fog name survives the merge and is namespaced by the reader.
#[test]
fn bc06_back_compat_fog() {
    let stack = merged(&[
        r#"{"biomes": {"minecraft:plains": {"fog_identifier": "fog_plains"}}}"#,
        r##"{"biomes": {"minecraft:plains": {"fog_color": "#ff0000", "water_fog_distance": 15}}}"##,
    ]);
    assert_eq!(
        raw(&stack, PLAINS),
        json(
            r##"{"fog_identifier": "fog_plains", "fog_ids_to_merge": [], "make_back_compat_fog": true, "fog_color": "#ff0000", "water_fog_distance": 15}"##
        )
    );
    let entry = plains(&stack);
    assert!(entry.fog_ids_to_merge.is_empty());
    let old = entry.old_fog.as_ref().expect("a back-compat fog");
    assert_eq!(old.fog_color, Some(rgba(255, 0, 0)));
    assert_eq!(old.water_fog_color, None);
    assert_eq!(old.water_fog_distance, Some(15.0));
    assert_eq!(
        old.fog_identifier
            .as_ref()
            .map(bedrock_addon::id::Id::as_str),
        Some("minecraft:fog_plains")
    );
    // Each old-style member alone does it, and a namespaced name is kept.
    for member in [
        r##""fog_color": "#010203""##,
        r##""water_fog_color": "#010203""##,
        r#""water_fog_distance": 2"#,
    ] {
        let stack = merged(&[
            r#"{"biomes": {"minecraft:plains": {"fog_identifier": "a:fog", "fog_ids_to_merge": ["x:y"]}}}"#,
            &format!(r#"{{"biomes": {{"minecraft:plains": {{{member}}}}}}}"#),
        ]);
        let entry = plains(&stack);
        assert!(entry.fog_ids_to_merge.is_empty(), "{member}");
        let old = entry.old_fog.as_ref().expect(member);
        assert_eq!(
            old.fog_identifier
                .as_ref()
                .map(bedrock_addon::id::Id::as_str),
            Some("a:fog")
        );
    }
    // A fog name on top of a back-compat fog ends it.
    let named = merged(&[
        r#"{"biomes": {"minecraft:plains": {}}}"#,
        r##"{"biomes": {"minecraft:plains": {"fog_color": "#ff0000"}}}"##,
        r#"{"biomes": {"minecraft:plains": {"fog_identifier": "b:fog"}}}"#,
    ]);
    assert!(plains(&named).old_fog.is_none());
    assert_eq!(ids(plains(&named)), ["b:fog"]);
    // Members without any fog member merge plainly.
    let plain = merged(&[
        r#"{"biomes": {"minecraft:plains": {"fog_ids_to_merge": ["x:y"]}}}"#,
        r#"{"biomes": {"minecraft:plains": {"water_surface_transparency": 0.5}}}"#,
    ]);
    assert_eq!(ids(plains(&plain)), ["x:y"]);
    assert!(plains(&plain).old_fog.is_none());
}

/// `bc08_bare_and_namespaced`: in one file the bare key's value replaces the namespaced one's.
#[test]
fn bc08_bare_and_namespaced() {
    for text in [
        r##"{"biomes": {"minecraft:plains": {"water_surface_color": "#000001"}, "plains": {"water_surface_color": "#000002"}}}"##,
        r##"{"biomes": {"plains": {"water_surface_color": "#000002"}, "minecraft:plains": {"water_surface_color": "#000001"}}}"##,
    ] {
        let stack = merged(&[text]);
        assert_eq!(legacy(&stack).biomes.len(), 1);
        assert_eq!(plains(&stack).water_surface_color, Some(rgba(0, 0, 2)));
    }
    // A key with a namespace of its own is kept as written.
    let other = merged(&[r#"{"biomes": {"myns:plains": {}}}"#]);
    assert!(
        legacy(&other)
            .biomes
            .contains_key(&BiomeId::parse("myns:plains"))
    );
    assert!(legacy(&other).get(&BiomeId::parse("myns:plains")).is_some());
    assert!(legacy(&other).get(&BiomeId::from_static(PLAINS)).is_none());
}

/// `bc09_world_gate`: from world 1.21.40, and for `*`, a base-game pack's file takes no part.
#[test]
fn bc09_world_gate() {
    let packs = || {
        vec![
            user(1, "mine", r#"{"biomes": {"a:mine": {}}}"#),
            pack(
                2,
                "vanilla",
                PackOrigin::Package,
                vec![(FILE, r#"{"biomes": {"a:vanilla": {}}}"#.to_owned())],
            ),
        ]
    };
    let at = |world: BaseGameVersion| {
        let stack = stack_in(packs(), world);
        let keys: Vec<String> = legacy(&stack)
            .biomes
            .keys()
            .map(|id| id.as_str().to_owned())
            .collect();
        let skipped = found(&stack, |c| matches!(c, Code::VanillaBiomesClientSkipped));
        (keys, skipped)
    };
    let version = |a, b, c| BaseGameVersion::new(SemVersion::new(a, b, c));
    for world in [
        version(1, 21, 30),
        version(1, 21, 39),
        BaseGameVersion::INCOMPATIBLE,
    ] {
        let (keys, skipped) = at(world);
        assert_eq!(keys, ["a:vanilla", "a:mine"]);
        assert!(skipped.is_empty());
    }
    for world in [
        version(1, 21, 40),
        version(1, 26, 0),
        BaseGameVersion::ANY,
        // No world version: treated as the engine's (ours).
        BaseGameVersion::EMPTY,
    ] {
        let (keys, skipped) = at(world);
        assert_eq!(keys, ["a:mine"]);
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].severity, Severity::Info);
        assert_eq!(skipped[0].location.pack.as_ref().unwrap().name(), "vanilla");
        assert_eq!(skipped[0].location.path.as_ref().unwrap().as_str(), FILE);
    }
    // A user pack's file is read whatever the world.
    let stack = stack_in(
        vec![user(1, "mine", r#"{"biomes": {"a:mine": {}}}"#)],
        version(1, 26, 0),
    );
    assert_eq!(legacy(&stack).biomes.len(), 1);
}

/// `bc10_transparency`: clamped to `[0, 1]`; a non-number is ignored.
#[test]
fn bc10_transparency() {
    let with = |value: &str| {
        merged(&[&format!(
            r#"{{"biomes": {{"minecraft:plains": {{"water_surface_transparency": {value}}}}}}}"#
        )])
    };
    for (value, clamped) in [("1.7", 1.0), ("-0.5", 0.0)] {
        let stack = with(value);
        assert_eq!(plains(&stack).water_surface_transparency, Some(clamped));
        let warned = found(&stack, |c| matches!(c, Code::ClampedValue { .. }));
        assert_eq!(warned.len(), 1);
        assert_eq!(warned[0].severity, Severity::Info);
        assert_eq!(
            warned[0].location.json_path.to_string(),
            "biomes/minecraft:plains/water_surface_transparency"
        );
    }
    for (value, kept) in [("0", 0.0), ("1", 1.0), ("0.65", 0.65)] {
        let stack = with(value);
        assert_eq!(plains(&stack).water_surface_transparency, Some(kept));
        assert_eq!(legacy(&stack).diagnostics().count(), 0);
    }
    let text = with(r#""x""#);
    assert_eq!(plains(&text).water_surface_transparency, None);
    let wrong = found(&text, |c| matches!(c, Code::InvalidType { .. }));
    assert_eq!(wrong.len(), 1);
    assert_eq!(wrong[0].severity, Severity::Info);
    // The other members of the wrong type are ignored the same way.
    let others = merged(&[
        r#"{"biomes": {"minecraft:plains": {"water_surface_color": "blue", "fog_ids_to_merge": ["a:f", 3], "make_back_compat_fog": "yes", "water_fog_distance": "far"}}}"#,
    ]);
    assert_eq!(plains(&others).water_surface_color, None);
    assert_eq!(ids(plains(&others)), ["a:f"]);
    assert!(plains(&others).old_fog.is_none());
    assert_eq!(
        found(&others, |c| matches!(c, Code::InvalidType { .. })).len(),
        4
    );
}

/// `bc11_invalid_json_skipped` (ours: a warning; vanilla skips the file silently). Our reader
/// accepts a trailing comma, so the broken file misses a colon.
#[test]
fn bc11_invalid_json_skipped_deviation() {
    let stack = merged(&[
        r#"{"biomes": {"a:one": {}}}"#,
        r#"{"biomes" {"a:broken": {}}}"#,
        r#"{"biomes": {"a:two": {},}}"#,
    ]);
    let keys: Vec<&str> = legacy(&stack)
        .biomes
        .keys()
        .map(bedrock_addon::id::Id::as_str)
        .collect();
    assert_eq!(keys, ["a:one", "a:two"]);
    let malformed = found(&stack, |c| matches!(c, Code::MalformedInput { .. }));
    assert_eq!(malformed.len(), 1);
    assert_eq!(malformed[0].severity, Severity::Info);
    assert_eq!(malformed[0].location.pack.as_ref().unwrap().name(), "pack1");
    assert_eq!(legacy(&stack).layers().len(), 2);
    // A root that is no object is skipped too.
    let array = merged(&[r#"[{"biomes": {}}]"#, r#"{"biomes": {"a:two": {}}}"#]);
    assert_eq!(legacy(&array).biomes.len(), 1);
    assert_eq!(
        found(&array, |c| matches!(c, Code::InvalidType { .. })).len(),
        1
    );
}

/// `bc12_vanilla_1_19_30`: the 1.19 shape, where every entry lists its fogs itself (these
/// are entries of the vanilla 1.19.30 file's shape).
#[test]
fn bc12_explicit_fog_ids_to_merge() {
    let stack = merged(
        &[r##"/* a block comment, as vanilla's file starts with one */
        {"biomes": {
            "default": {"water_surface_color": "#44AFF5", "water_surface_transparency": 0.65,
                "fog_identifier": "minecraft:fog_default", "fog_ids_to_merge": ["minecraft:fog_default"]},
            "plains": {"water_surface_color": "#44AFF5", "fog_identifier": "minecraft:fog_plains",
                "fog_ids_to_merge": ["minecraft:fog_plains"], "remove_all_prior_fog": false,
                "inherit_from_prior_fog": false}}}"##],
    );
    assert_eq!(ids(plains(&stack)), ["minecraft:fog_plains"]);
    let default = legacy(&stack).default.as_ref().unwrap();
    assert_eq!(ids(default), ["minecraft:fog_default"]);
    assert_eq!(default.water_surface_transparency, Some(0.65));
    assert_eq!(legacy(&stack).diagnostics().count(), 0);
}

/// Retail's slices merged as a world below 1.21.40 merges them: the later slices meet the
/// entries of the earlier ones, so the predicate turns their old-style members into
/// back-compat fogs.
#[test]
fn retail_slices_merge_into_back_compat_fogs() {
    let base = fixture("biomes_client_json/bc13_vanilla_alone/biomes_client.json");
    let nether = fixture("biomes_client_json/bc12_vanilla_slices/vanilla_1.16.json");
    let stack = merged(&[&base, &nether]);
    let warped = legacy(&stack)
        .biomes
        .get(&BiomeId::parse("minecraft:warped_forest"))
        .expect("the 1.16 slice's entry");
    assert_eq!(warped.water_surface_color, Some(rgba(0x90, 0x59, 0x57)));
    assert!(
        legacy(&stack)
            .diagnostics()
            .all(|d| d.severity < Severity::Warning),
        "{:#?}",
        legacy(&stack).diagnostics().collect::<Vec<_>>()
    );
}

/// `bc13_vanilla_1_26_0_alone`: vanilla's current file alone gives water colours and no
/// per-biome fog.
#[test]
fn bc13_vanilla_alone() {
    let text = fixture("biomes_client_json/bc13_vanilla_alone/biomes_client.json");
    let vanilla = pack(
        1,
        "vanilla",
        PackOrigin::Package,
        vec![(FILE, text.clone())],
    );
    let stack = stack_in(
        vec![vanilla],
        BaseGameVersion::new(SemVersion::new(1, 21, 30)),
    );
    let value = legacy(&stack);
    assert!(value.biomes.len() > 60, "{}", value.biomes.len());
    assert!(
        value
            .biomes
            .values()
            .all(|entry| entry.fog_ids_to_merge.is_empty())
    );
    assert!(value.biomes.values().all(|entry| entry.old_fog.is_none()));
    assert!(
        value
            .biomes
            .values()
            .all(|entry| entry.water_surface_color.is_some())
    );
    let default = value.default.as_ref().expect("a default entry");
    assert_eq!(default.water_surface_color, Some(rgba(0x44, 0xAF, 0xF5)));
    assert_eq!(default.water_surface_transparency, Some(0.65));
    assert!(value.diagnostics().all(|d| d.severity < Severity::Warning));
    // The file on its own, outside a stack, reads the same and writes its merged JSON.
    let alone = LegacyClientBiomes::parse(&path(), text.as_bytes(), &context()).unwrap();
    assert_eq!(alone.value(), value.value());
    assert_eq!(json(&alone.to_json().unwrap()), value.merged);
    assert_eq!(alone.default_path().as_str(), "biomes_client.json");
    let again = LegacyClientBiomes::parse(&path(), alone.to_json().unwrap().as_bytes(), &context());
    assert_eq!(again.unwrap().value(), alone.value());
    assert!(LegacyClientBiomes::parse(&path(), b"{", &context()).is_err());
}

/// `bc14_overrides_client_biome`: what the client ends up with: the legacy
/// `default` entry's water surface replaces every biome's, a client biome's included.
#[cfg(feature = "client_biome")]
#[test]
fn bc14_effective_water_surface() {
    let client = ClientBiome::parse(
        &path(),
        biome(
            "1.21.130",
            PLAINS,
            r##""minecraft:water_appearance": {"surface_color": "#123456", "surface_opacity": 0.5}"##,
        )
        .as_bytes(),
        &context(),
    )
    .unwrap();
    let id = BiomeId::from_static(PLAINS);
    let none = LegacyClientBiomes::default();
    let own = effective(Some(client.value()), &none, &id);
    assert_eq!(own.color, Some(rgba(0x12, 0x34, 0x56)));
    assert_eq!(own.opacity, Some(0.5));
    assert!(!own.overridden);
    let stack = merged(&[r##"{"biomes": {"default": {"water_surface_color": "#44AFF5"}}}"##]);
    let overridden = effective(Some(client.value()), legacy(&stack), &id);
    assert_eq!(overridden.color, Some(rgba(0x44, 0xAF, 0xF5)));
    assert_eq!(overridden.opacity, Some(0.5));
    assert!(overridden.overridden);
    // Every other biome gets the default too, and a biome's own entry wins over it.
    let other = legacy(&stack).effective_water_surface(None, None, &BiomeId::parse("a:other"));
    assert_eq!(other.color, Some(rgba(0x44, 0xAF, 0xF5)));
    assert!(!other.overridden);
    let stack = merged(&[
        r##"{"biomes": {"default": {"water_surface_color": "#44AFF5", "water_surface_transparency": 0.65},
            "plains": {"water_surface_color": "#000001"}}}"##,
    ]);
    let entry = effective(Some(client.value()), legacy(&stack), &id);
    assert_eq!(entry.color, Some(rgba(0, 0, 1)));
    assert_eq!(entry.opacity, Some(0.65));
    assert!(entry.overridden);
    // The same values are no override.
    let same = merged(&[
        r##"{"biomes": {"plains": {"water_surface_color": "#123456", "water_surface_transparency": 0.5}}}"##,
    ]);
    assert!(!effective(Some(client.value()), legacy(&same), &id).overridden);
}

/// `bc14_overrides_client_biome`: the warning that names both packs.
#[cfg(all(feature = "client_biome", feature = "biome"))]
#[test]
fn bc14_overrides_client_biome() {
    let biomes = pack(
        1,
        "biomes",
        PackOrigin::User,
        vec![(
            "biomes/plains.json",
            biome(
                "1.21.130",
                PLAINS,
                r##""minecraft:water_appearance": {"surface_color": "#123456"}"##,
            ),
        )],
    );
    let overrides = user(
        2,
        "overrides",
        r##"{"biomes": {"default": {"water_surface_color": "#44AFF5"}}}"##,
    );
    let stacks = super::common::world_stacks(vec![biomes, overrides], LoadOptions::default());
    let report = stacks.link();
    let warned: Vec<_> = report
        .iter()
        .filter(|d| matches!(d.code, Code::ClientBiomeOverriddenByBiomesClient))
        .collect();
    assert_eq!(warned.len(), 1, "{warned:#?}");
    // Ours: vanilla overwrites in silence.
    assert_eq!(warned[0].severity, Severity::Info);
    // At the client biome's component, in its pack; the detail names the overriding pack.
    assert_eq!(warned[0].location.pack.as_ref().unwrap().name(), "biomes");
    assert_eq!(
        warned[0].location.path.as_ref().unwrap().as_str(),
        "biomes/plains.json"
    );
    assert_eq!(
        warned[0].location.json_path.to_string(),
        "minecraft:client_biome/components/minecraft:water_appearance"
    );
    let detail = warned[0].detail.as_ref().map(ToString::to_string);
    assert!(
        detail.as_deref().is_some_and(|d| d.contains("overrides")),
        "{detail:?}"
    );
    // The data outcome is vanilla's: the legacy default's colour.
    let client = stacks.resource.client_biomes();
    let plains = client.get(&BiomeId::from_static(PLAINS)).expect("plains");
    let surface = effective(
        Some(plains.value()),
        stacks.resource.legacy_client_biomes(),
        &BiomeId::from_static(PLAINS),
    );
    assert_eq!(surface.color, Some(rgba(0x44, 0xAF, 0xF5)));
    assert!(surface.overridden);
}

/// `bc15_ignored_members`: the two entry members of the row, which vanilla never
/// reads, among the ten it does. (Root members:
/// `a_root_format_version_and_another_root_member_are_each_an_info`.)
#[test]
fn bc15_ignored_members() {
    let stack = merged(&[
        r##"{"biomes": {"minecraft:plains": {"sky_color": "#000000", "foliage_color": "#000000",
            "water_surface_color": "#000001", "water_surface_transparency": 1, "fog_identifier": "a:f",
            "inherit_from_prior_fog": false, "remove_all_prior_fog": false, "fog_ids_to_merge": [],
            "make_back_compat_fog": false, "fog_color": "#000000", "water_fog_color": "#000000",
            "water_fog_distance": 1}}}"##,
    ]);
    let unknown = found(&stack, |c| matches!(c, Code::UnrecognizedField { .. }));
    let paths: Vec<String> = unknown
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(
        paths,
        [
            "biomes/minecraft:plains/sky_color",
            "biomes/minecraft:plains/foliage_color"
        ]
    );
    assert!(unknown.iter().all(|d| d.severity == Severity::Info));
    assert_eq!(legacy(&stack).diagnostics().count(), 2);
    assert_eq!(plains(&stack).water_surface_color, Some(rgba(0, 0, 1)));
}

/// V6.2: a merged file without `biomes` applies nothing; a stack without the file is silent.
#[test]
fn no_biomes_member_applies_nothing() {
    let stack = merged(&["{}"]);
    assert!(legacy(&stack).biomes.is_empty());
    let none = found(&stack, |c| matches!(c, Code::NoEffect { .. }));
    assert_eq!(none.len(), 1);
    assert_eq!(none[0].severity, Severity::Info);
    let empty = merged(&[]);
    assert!(legacy(&empty).biomes.is_empty());
    assert_eq!(legacy(&empty).diagnostics().count(), 0);
    assert!(legacy(&empty).layers().is_empty());
}

/// an entry is found under the biome's full name, else, for a `minecraft`
/// biome, under its bare name.
#[test]
fn the_lookup_falls_back_to_the_bare_name() {
    let mut value = LegacyClientBiomes::default();
    value
        .biomes
        .insert(BiomeId::parse("plains"), LegacyClientBiome::default());
    assert!(value.get(&BiomeId::from_static(PLAINS)).is_some());
    assert!(value.get(&BiomeId::parse("myns:plains")).is_none());
    assert!(value.get(&BiomeId::parse("minecraft:desert")).is_none());
}

/// the file has no version; a `format_version` written at its root is read
/// by nothing (ours: an info). Any other root member is an unknown one, an info too.
#[test]
fn a_root_format_version_and_another_root_member_are_each_an_info() {
    let stack = merged(&[r#"{"format_version": "1.21.40", "note": 1, "biomes": {"a:one": {}}}"#]);
    let unknown = found(&stack, |c| matches!(c, Code::UnrecognizedField { .. }));
    let seen: Vec<(String, Severity)> = unknown
        .iter()
        .map(|d| (d.location.json_path.to_string(), d.severity))
        .collect();
    assert_eq!(
        seen,
        [
            ("format_version".to_owned(), Severity::Info),
            ("note".to_owned(), Severity::Info)
        ]
    );
    assert_eq!(legacy(&stack).biomes.len(), 1);
    assert_eq!(legacy(&stack).diagnostics().count(), 2);
}

/// A key of `biomes` written without a namespace is written back as `minecraft:<key>`: it is
/// what the game makes of the key before it reads anything else of the file (the pre-merge
/// transform of its merge strategy renames every bare key but `default`), so the written
/// file is the same file to the game, and reads back as the same value.
// The game's pre-merge transform of `biomes_client.json`.
#[test]
fn k108_a_bare_biome_key_is_written_with_the_namespace_the_game_gives_it() {
    let text = r##"{"biomes": {
        "default": {"water_surface_color": "#44AFF5"},
        "plains": {"water_surface_color": "#112233", "fog_identifier": "minecraft:fog_plains"},
        "demo:marsh": {"water_surface_transparency": 0.5}
    }}"##;
    let read = LegacyClientBiomes::parse(&path(), text.as_bytes(), &context()).unwrap();
    let keys: Vec<&str> = read
        .biomes
        .keys()
        .map(bedrock_addon::id::Id::as_str)
        .collect();
    // (A renamed key follows the others: the game walks the keys of an object by name.)
    assert_eq!(keys, ["demo:marsh", "minecraft:plains"]);
    let written = read.to_json().unwrap();
    assert!(written.contains(r#""minecraft:plains""#), "{written}");
    assert!(!written.contains(r#""plains""#), "{written}");
    assert!(written.contains(r#""default""#) && written.contains(r#""demo:marsh""#));
    // The written file reads back as the value the file gave, says what it said, and is
    // written as itself.
    let again = LegacyClientBiomes::parse(&path(), written.as_bytes(), &context()).unwrap();
    assert_eq!(again.value(), read.value());
    assert_eq!(
        format!("{:?}", again.diagnostics()),
        format!("{:?}", read.diagnostics())
    );
    assert_eq!(again.to_json().unwrap(), written);
}
