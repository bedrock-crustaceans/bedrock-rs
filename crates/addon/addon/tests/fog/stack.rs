//! `n01`–`n05`, `s01`–`s03`), the biome binding
//! rows `b01`–`b02`, the accessor and where a definition is written.

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Placed, Registry};
use bedrock_addon::families::fog::FogSettings;
use bedrock_addon::id::FogId;

use super::common::{count, expect_at, line, lines, loaded, rejected, stacks};
use super::support::{base_game, doc, named, pack, parse};

const NAMESPACE: &str = "Invalid fog identifier namespace. \"minecraft\" namespace cannot be used.";

fn id(name: &str) -> FogId {
    FogId::parse(name)
}

fn found<'r>(
    registry: &'r Registry<FogSettings>,
    name: &str,
    severity: Severity,
) -> Vec<&'r Diagnostic> {
    registry
        .diagnostics()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .collect()
}

fn all(registry: &Registry<FogSettings>) -> Vec<String> {
    registry.diagnostics().map(line).collect()
}

#[test]
fn n01_minecraft_namespace() {
    for version in ["1.21.90", "1.26.20"] {
        let d = rejected(parse(&named(version, "minecraft:fog_x", ""), &pack()));
        expect_at(
            &d,
            "NamespaceRule",
            Severity::Error,
            NAMESPACE,
            "minecraft:fog_settings/description/identifier",
        );
        // The namespace rule has its own line and no closing one.
        assert_eq!(d.len(), 1, "{version}: {:?}", lines(&d));
    }
    // Another namespace that merely starts like it passes.
    loaded(parse(&named("1.26.20", "minecraft2:fog_x", ""), &pack()));
    loaded(parse(&named("1.26.20", "my:minecraft", ""), &pack()));
}

#[test]
fn n02_minecraft_namespace_uppercase() {
    for version in ["1.21.90", "1.26.20"] {
        let d = rejected(parse(&named(version, "Minecraft:fog_x", ""), &pack()));
        assert_eq!(count(&d, "NamespaceRule", Severity::Error), 1, "{version}");
    }
}

#[test]
fn n03_minecraft_namespace_base_game() {
    for version in ["1.21.90", "1.26.20"] {
        let document = loaded(parse(&named(version, "minecraft:fog_x", ""), &base_game()));
        assert!(document.diagnostics().is_empty());
        assert_eq!(document.namespace(), "minecraft");
    }
}

#[test]
fn n04_bare_minecraft_identifier() {
    // Without a colon the whole identifier is the namespace.
    for identifier in ["minecraft", "MINECRAFT"] {
        let d = rejected(parse(&named("1.26.20", identifier, ""), &pack()));
        assert_eq!(
            count(&d, "NamespaceRule", Severity::Error),
            1,
            "{identifier}"
        );
    }
    let document = loaded(parse(&named("1.26.20", "myfog", ""), &pack()));
    assert_eq!(document.namespace(), "myfog");
    loaded(parse(&named("1.26.20", "minecraft", ""), &base_game()));
}

#[test]
fn a_document_that_fails_validation_is_not_asked_for_its_namespace() {
    // Step 8 comes after the parser: a refused document reports the parser's reason only.
    let d = rejected(parse(
        &named(
            "1.26.20",
            "minecraft:x",
            r#""distance": {"air": {"fog_start": 9, "fog_end": 1}}"#,
        ),
        &pack(),
    ));
    assert_eq!(
        count(&d, "NamespaceRule", Severity::Error),
        0,
        "{:?}",
        lines(&d)
    );
    assert_eq!(count(&d, "DocumentRejected", Severity::Error), 1);
}

#[test]
fn n05_identifier_case() {
    let upper = named("1.21.90", "Demo:Fog", "");
    let lower = named("1.21.90", "demo:fog", "");
    let set = stacks(&[&[("fogs/a.json", &upper)], &[("fogs/a.json", &lower)]]);
    let registry = set.resource.fogs();
    assert_eq!(registry.len(), 2, "{:?}", all(registry));
    assert_eq!(
        registry.get(&id("Demo:Fog")).unwrap().origin().pack.index(),
        Some(0)
    );
    assert_eq!(
        registry.get(&id("demo:fog")).unwrap().origin().pack.index(),
        Some(1)
    );
    assert!(found(registry, "ShadowedDefinition", Severity::Info).is_empty());
}

/// `vanilla = "silently skipped"`.
#[test]
fn s01_top_pack_wins_with_a_shadow_info_deviation() {
    let top = doc("1.26.20", r#""distance": {"air": {"fog_end": 1}}"#);
    let bottom = doc("1.21.90", "");
    let set = stacks(&[&[("fogs/a.json", &top)], &[("fogs/b.json", &bottom)]]);
    let registry = set.resource.fogs();
    assert_eq!(registry.len(), 1);
    let fog = registry.get(&id("demo:fog")).unwrap();
    assert_eq!(fog.origin().pack.index(), Some(0));
    assert!(fog.distance.is_some());
    let shadowed = found(registry, "ShadowedDefinition", Severity::Info);
    assert_eq!(shadowed.len(), 1, "{:?}", all(registry));
    assert_eq!(
        shadowed[0]
            .location
            .pack
            .as_ref()
            .and_then(bedrock_addon::pack::PackRef::index),
        Some(1)
    );
}

#[test]
fn s02_failed_top_falls_through() {
    let top = doc("1.16.0", "");
    let bottom = doc("1.21.90", "");
    let set = stacks(&[&[("fogs/a.json", &top)], &[("fogs/a.json", &bottom)]]);
    let registry = set.resource.fogs();
    let fog = registry.get(&id("demo:fog")).unwrap();
    assert_eq!(fog.origin().pack.index(), Some(1));
    assert_eq!(
        found(registry, "UnsupportedVersion", Severity::Error).len(),
        1
    );
    assert!(found(registry, "ShadowedDefinition", Severity::Info).is_empty());
    assert_eq!(registry.rejected().len(), 1);
}

#[test]
fn s03_empty_file() {
    let good = doc("1.26.20", "");
    let set = stacks(&[&[("fogs/a.json", ""), ("fogs/b.json", &good)]]);
    let registry = set.resource.fogs();
    let empty = found(registry, "EmptyDocument", Severity::Error);
    assert_eq!(empty.len(), 1, "{:?}", all(registry));
    assert_eq!(
        empty[0].code.to_string(),
        "The raw Json in was empty after getting resources."
    );
    assert_eq!(
        empty[0].location.path.as_ref().unwrap().as_str(),
        "fogs/a.json"
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn only_json_files_directly_in_fogs_are_read() {
    let good = doc("1.26.20", "");
    let other = named("1.26.20", "demo:other", "");
    let set = stacks(&[&[
        ("fogs/a.json", good.as_str()),
        ("fogs/sub/b.json", other.as_str()),
        ("fogs/c.JSON", other.as_str()),
        ("fogs/d.txt", other.as_str()),
    ]]);
    let registry = set.resource.fogs();
    assert_eq!(registry.len(), 1, "{:?}", all(registry));
    assert!(registry.get(&id("demo:fog")).is_some());
    assert!(all(registry).is_empty());
}

#[test]
fn a_definition_is_placed_by_its_identifier() {
    for (identifier, path) in [
        (
            "minecraft:fog_bamboo_jungle",
            "fogs/minecraft_fog_bamboo_jungle.json",
        ),
        ("demo:fog", "fogs/demo_fog.json"),
        ("myfog", "fogs/myfog.json"),
    ] {
        assert_eq!(
            FogSettings::new(id(identifier)).default_path().as_str(),
            path
        );
    }
    let fog = FogSettings::new(id("demo:fog"));
    let text = bedrock_addon::document::Content::to_json(&fog).unwrap();
    let again = loaded(parse(&text, &pack()));
    assert_eq!(again.value(), &fog);
    assert!(text.contains("\"format_version\": \"1.26.20\""), "{text}");
}

// ---- biome binding ----

const CLIENT_BIOME: &str = r#"{"format_version": "1.21.40", "minecraft:client_biome": {
    "description": {"identifier": "demo:swamp"},
    "components": {"minecraft:fog_appearance": {"fog_identifier": "FOG"}}}}"#;

/// The names of the unresolved fog references of `d`, as each detail gives them.
fn unresolved(d: &bedrock_addon::diagnostic::Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(|f| {
            format!(
                "{:?} {}{}",
                f.severity,
                f.code,
                f.detail
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            )
        })
        .collect()
}

#[test]
fn b01_client_biome_unresolved() {
    // The biome is one of the game's: a client biome of a biome nothing names is an
    // unresolved reference of its own.
    let biome = CLIENT_BIOME
        .replace("FOG", "demo:missing")
        .replace("demo:swamp", "minecraft:swampland");
    let fog = doc("1.26.20", "");
    let set = stacks(&[&[
        ("biomes/swamp.client_biome.json", biome.as_str()),
        ("fogs/a.json", fog.as_str()),
    ]]);
    // The game logs the miss twice, both to the content log at the error level: the glue's
    // resolver for the fog appearance component and the link pass's validator
    // (for `fog_identifier`).
    let found = unresolved(&set.link());
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found[0].starts_with("Error fog_identifier: cannot find fog definition 'demo:missing'"),
        "{found:?}"
    );
    assert!(
        found[1].starts_with(
            "Error -> components -> minecraft:fog_appearance -> fog_identifier:  Missing referenced asset demo:missing"
        ),
        "{found:?}"
    );
}

#[test]
fn b02_client_biome_resolved() {
    // The biome is one of the game's.
    let biome = CLIENT_BIOME
        .replace("FOG", "demo:fog")
        .replace("demo:swamp", "minecraft:swampland");
    let fog = doc("1.26.20", "");
    let set = stacks(&[&[
        ("biomes/swamp.client_biome.json", biome.as_str()),
        ("fogs/a.json", fog.as_str()),
    ]]);
    assert!(set.resource.fogs().get(&id("demo:fog")).is_some());
    assert_eq!(unresolved(&set.link()), Vec::<String>::new());
}

/// The fog half of the binding, which needs no link pass: the registry answers by the
/// identifier as written.
#[test]
fn b02_the_registry_resolves_an_identifier_as_written() {
    let fog = doc("1.26.20", "");
    let set = stacks(&[&[("fogs/a.json", fog.as_str())]]);
    let registry = set.resource.fogs();
    assert!(registry.get(&id("demo:fog")).is_some());
    assert!(registry.get(&id("demo:missing")).is_none());
    assert!(registry.get(&id("Demo:Fog")).is_none());
}

/// inside one pack the first file to
/// register an identifier keeps it, as across packs; the later file is skipped with the
/// `ShadowedDefinition` info (ours) and keeps what its parse found. Files of a pack are read
/// in path order (the game's order inside a pack is not specified).
#[test]
fn s01_inside_one_pack_the_first_file_wins_deviation() {
    let first = doc("1.26.20", r#""distance": {"air": {"fog_end": 1}}"#);
    let later = doc("1.26.20", r#""unknown": 1"#);
    for files in [
        [
            ("fogs/a.json", first.as_str()),
            ("fogs/b.json", later.as_str()),
        ],
        [
            ("fogs/b.json", later.as_str()),
            ("fogs/a.json", first.as_str()),
        ],
    ] {
        let set = stacks(&[&files]);
        let registry = set.resource.fogs();
        assert_eq!(registry.len(), 1);
        let fog = registry.get(&id("demo:fog")).unwrap();
        assert_eq!(fog.origin().path.as_str(), "fogs/a.json");
        assert!(fog.distance.is_some());
        let shadowed = found(registry, "ShadowedDefinition", Severity::Info);
        assert_eq!(shadowed.len(), 1, "{:?}", all(registry));
        assert_eq!(
            shadowed[0].location.path.as_ref().unwrap().as_str(),
            "fogs/b.json"
        );
        // The game parsed the later file before it skipped it: its findings stay.
        let unknown = found(registry, "UnrecognizedField", Severity::Info);
        assert_eq!(unknown.len(), 1, "{:?}", all(registry));
        assert_eq!(
            unknown[0].location.path.as_ref().unwrap().as_str(),
            "fogs/b.json"
        );
        assert!(
            all(registry)
                .iter()
                .all(|l| !l.contains("DuplicateIdentifier")),
            "{:?}",
            all(registry)
        );
        assert!(registry.rejected().is_empty());
    }
}

#[test]
fn s01_three_files_of_one_pack_and_a_lower_pack() {
    let fog = doc("1.26.20", "");
    let set = stacks(&[
        &[
            ("fogs/a.json", fog.as_str()),
            ("fogs/b.json", fog.as_str()),
            ("fogs/c.json", fog.as_str()),
        ],
        &[("fogs/a.json", fog.as_str())],
    ]);
    let registry = set.resource.fogs();
    let winner = registry.get(&id("demo:fog")).unwrap();
    assert_eq!(winner.origin().pack.index(), Some(0));
    assert_eq!(winner.origin().path.as_str(), "fogs/a.json");
    let mut shadowed: Vec<(Option<usize>, String)> =
        found(registry, "ShadowedDefinition", Severity::Info)
            .iter()
            .map(|f| {
                (
                    f.location
                        .pack
                        .as_ref()
                        .and_then(bedrock_addon::pack::PackRef::index),
                    f.location.path.as_ref().unwrap().as_str().to_owned(),
                )
            })
            .collect();
    shadowed.sort();
    assert_eq!(
        shadowed,
        [
            (Some(0), "fogs/b.json".to_owned()),
            (Some(0), "fogs/c.json".to_owned()),
            (Some(1), "fogs/a.json".to_owned()),
        ]
    );
}

/// A lower pack's definition is the registry's to shadow (it stays in the identifier's layers);
/// only a later file of the same pack is settled before the registry sees it.
#[test]
fn s01_a_lower_pack_s_definition_stays_a_layer() {
    let fog = doc("1.26.20", "");
    let set = super::common::stacks_with(
        &[
            &[("fogs/a.json", fog.as_str()), ("fogs/b.json", fog.as_str())],
            &[("fogs/a.json", fog.as_str())],
        ],
        true,
    );
    let registry = set.resource.fogs();
    let layers: Vec<(Option<usize>, String)> = registry
        .layers(&id("demo:fog"))
        .map(|document| {
            (
                document.origin().pack.index(),
                document.origin().path.as_str().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        layers,
        [
            (Some(0), "fogs/a.json".to_owned()),
            (Some(1), "fogs/a.json".to_owned())
        ]
    );
}
