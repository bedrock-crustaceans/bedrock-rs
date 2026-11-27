//! The registry of `rp.client_biomes()`: the walk of `biomes/`, the
//! claim-first rule, duplicates, beta documents and `Placed::default_path`.

use bedrock_addon::diagnostic::codes;
use bedrock_addon::diagnostic::{Code, Diagnostic, Severity};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::client_biome::ClientBiome;
use bedrock_addon::families::client_biome::component::SkyColor;
use bedrock_addon::id::BiomeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, Pack, PackOrigin, PackSide, PackStack, Resource};

use super::common::{biome, document};
#[cfg(feature = "biome")]
use super::stacks::world_stacks;
use super::stacks::{manifest, pack};

/// A client biome named `identifier` whose sky colour tells the packs apart.
fn sky(identifier: &str, color: &str) -> String {
    biome(
        "1.21.130",
        identifier,
        &format!(r#""minecraft:sky_color": {{"sky_color": "{color}"}}"#),
    )
}

/// The resource stack of `packs`, top first.
fn stack(packs: Vec<Pack>) -> PackStack<Resource> {
    PackStack::<Resource>::of(packs, LoadOptions::default()).expect("a stack")
}

/// The sky colour of the effective `identifier`.
fn sky_of(stack: &PackStack<Resource>, identifier: &str) -> Option<String> {
    let biomes = stack.client_biomes();
    let biome = biomes.get(&BiomeId::parse(identifier))?;
    match &biome.components.get::<SkyColor>()?.sky_color {
        bedrock_addon::families::client_biome::Color255Rgb::Hex(text) => Some(text.clone()),
        other @ bedrock_addon::families::client_biome::Color255Rgb::Array(_) => panic!("{other:?}"),
    }
}

/// Whether `code` is a definition the legacy `ShadowedDefinition` was.
fn is_shadowed_definition(code: &Code) -> bool {
    matches!(
        code.id(),
        "core/shadowed_definition"
            | "core/shadowed_definition_worded"
            | "dimension/replaced_registration"
            | "item/overridden_by_a_higher_pack"
            | "material/variant_replaced"
    )
}

/// The findings of the registry with a code `code` accepts.
fn found(stack: &PackStack<Resource>, code: impl Fn(&Code) -> bool) -> Vec<Diagnostic> {
    stack
        .client_biomes()
        .diagnostics()
        .filter(|d| code(&d.code))
        .cloned()
        .collect()
}

/// The file a finding is at.
fn file(diagnostic: &Diagnostic) -> (String, String) {
    (
        diagnostic.location.pack.as_ref().unwrap().name().to_owned(),
        diagnostic
            .location
            .path
            .as_ref()
            .unwrap()
            .as_str()
            .to_owned(),
    )
}

/// `st01_first_wins`: the highest pack's document wins whole; the other is shadowed.
#[test]
fn st01_first_wins() {
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![(
            "biomes/plains.client_biome.json",
            sky("minecraft:plains", "#111111"),
        )],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![
            (
                "biomes/plains.json",
                biome(
                    "1.21.130",
                    "minecraft:plains",
                    r##""minecraft:sky_color": {"sky_color": "#222222"}, "minecraft:water_appearance": {"surface_opacity": 0.5}"##,
                ),
            ),
            ("biomes/desert.json", sky("minecraft:desert", "#333333")),
        ],
    );
    let stack = stack(vec![top, bottom]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#111111")
    );
    // No component merge: the lower pack's water is not the biome's.
    let biomes = stack.client_biomes();
    let plains = biomes
        .get(&BiomeId::from_static("minecraft:plains"))
        .unwrap();
    assert_eq!(plains.components.len(), 1);
    assert_eq!(plains.origin().pack.name(), "top");
    assert_eq!(plains.origin().side, PackSide::Resource);
    assert_eq!(
        sky_of(&stack, "minecraft:desert").as_deref(),
        Some("#333333")
    );
    assert_eq!(biomes.len(), 2);
    let shadowed = found(&stack, |c| c.id() == "core/shadowed_definition");
    assert_eq!(shadowed.len(), 1, "{shadowed:#?}");
    assert_eq!(shadowed[0].severity, Severity::Info);
    assert_eq!(
        file(&shadowed[0]),
        ("bottom".to_owned(), "biomes/plains.json".to_owned())
    );
    assert!(
        stack
            .client_biomes()
            .diagnostics()
            .all(|d| d.severity < Severity::Warning)
    );
}

/// `st02_failed_load_does_not_claim`: a document that does not load claims nothing.
#[test]
fn st02_failed_load_does_not_claim() {
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![(
            "biomes/plains.json",
            biome(
                "1.21.130",
                "minecraft:plains",
                r#""minecraft:sky_color": {}"#,
            ),
        )],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#222222"))],
    );
    let stack = stack(vec![top, bottom]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#222222")
    );
    assert_eq!(stack.client_biomes().rejected().len(), 1);
    assert!(found(&stack, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS)).is_empty());
    assert!(found(&stack, is_shadowed_definition).is_empty());
}

/// A document the client's own component checks reject has already claimed its identifier:
/// the biome gets no client data, and the lower pack's document is shadowed (the part of
/// `st03` that needs no other registry).
#[test]
fn a_glue_failure_claims_the_identifier() {
    let failing = |identifier: &str| {
        biome(
            "1.21.40",
            identifier,
            r#""minecraft:biome_music": {"volume_multiplier": 2}"#,
        )
    };
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        // Written without its namespace: the claim is the upgraded identifier.
        vec![("biomes/plains.json", failing("plains"))],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#222222"))],
    );
    let stack = stack(vec![top, bottom]);
    assert_eq!(sky_of(&stack, "minecraft:plains"), None);
    assert!(stack.client_biomes().is_empty());
    assert_eq!(BiomeId::parse("plains"), BiomeId::parse("minecraft:plains"));
    let claims = found(&stack, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS));
    assert_eq!(claims.len(), 1, "{claims:#?}");
    // The client has no line for the shadowing: an information of ours.
    assert_eq!(claims[0].severity, Severity::Info);
    assert_eq!(file(&claims[0]).0, "top");
    assert!(
        stack
            .client_biomes()
            .diagnostics()
            .any(|d| d.code.to_string() == "Volume multiplier must be between 0 and 1, inclusive.")
    );
    // Beside the claim's warning the lower pack's document says it is the shadowed one.
    let shadowed = found(&stack, |c| c.id() == "core/shadowed_definition");
    assert_eq!(shadowed.len(), 1, "{shadowed:#?}");
    assert_eq!(shadowed[0].severity, Severity::Info);
    assert_eq!(file(&shadowed[0]).0, "bottom");
    // The other way round the failing document is below: it shadows nothing.
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#111111"))],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![("biomes/plains.json", failing("minecraft:plains"))],
    );
    let stack = self::stack(vec![top, bottom]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#111111")
    );
    assert!(found(&stack, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS)).is_empty());
}

/// The findings of `report` with a code `code` accepts.
#[cfg(feature = "biome")]
fn linked(
    report: &bedrock_addon::diagnostic::Diagnostics,
    code: impl Fn(&Code) -> bool,
) -> Vec<Diagnostic> {
    // The client's findings; the game's validator line beside a miss is the link tests'.
    report
        .iter()
        .filter(|d| code(&d.code) && !d.code.to_string().contains(":  Missing referenced asset "))
        .cloned()
        .collect()
}

/// `st03_glue_failure_shadows`: an unknown `fog_identifier` fails the top document,
/// which still claims `minecraft:plains`: the pass tells the error, and the notice names the
/// pack it shadows.
#[cfg(all(
    feature = "biome",
    feature = "biomes_client",
    feature = "fog",
    feature = "deferred_settings"
))]
#[test]
fn st03_glue_failure_shadows() {
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![(
            "biomes/plains.json",
            biome(
                "1.21.130",
                "minecraft:plains",
                r#""minecraft:fog_appearance": {"fog_identifier": "myns:no_such_fog"}"#,
            ),
        )],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#222222"))],
    );
    let stacks = world_stacks(vec![top, bottom], LoadOptions::default());
    let report = stacks.link();
    let unresolved = linked(&report, |c| c.id() == "core/unresolved_reference_worded");
    assert_eq!(unresolved.len(), 1, "{unresolved:#?}");
    assert_eq!(unresolved[0].severity, Severity::Error);
    assert_eq!(
        unresolved[0].code.to_string(),
        "fog_identifier: cannot find fog definition 'myns:no_such_fog'"
    );
    assert_eq!(file(&unresolved[0]).0, "top");
    assert_eq!(
        unresolved[0].location.json_path.to_string(),
        "minecraft:client_biome/components/minecraft:fog_appearance/fog_identifier"
    );
    // Nothing of the document is applied, and the lower pack's is not used either: the pass
    // says so with the error above and the warning below. It returns findings and changes
    // no registry, so the registry still holds the top pack's claim,
    // never the lower pack's document.
    let kept = stacks
        .resource
        .client_biomes()
        .get(&BiomeId::from_static("minecraft:plains"))
        .expect("the claiming document");
    assert_eq!(kept.origin().pack.name(), "top");
    let claims = linked(&report, |c| {
        c.is(&bedrock_addon::diagnostic::codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS)
    });
    assert_eq!(claims.len(), 1, "{claims:#?}");
    assert_eq!(claims[0].severity, Severity::Info);
    assert_eq!(file(&claims[0]).0, "top");
    // The code has no payload: the shadowed packs are in the finding's detail.
    let detail = claims[0].detail.as_ref().map(ToString::to_string);
    assert!(
        detail.as_deref().is_some_and(|d| d.contains("bottom")),
        "{detail:?}"
    );
}

/// `st04_unknown_biome`: a client biome for a biome the server does not send.
#[cfg(feature = "biome")]
#[test]
fn st04_unknown_biome() {
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![("biomes/nowhere.json", sky("myns:nowhere", "#111111"))],
    );
    let stacks = world_stacks(vec![only], LoadOptions::default());
    let report = stacks.link();
    let unresolved = linked(&report, |c| c.id() == "core/unresolved_reference_worded");
    assert_eq!(unresolved.len(), 1, "{unresolved:#?}");
    assert_eq!(unresolved[0].severity, Severity::Error);
    assert_eq!(
        unresolved[0].code.to_string(),
        "Loaded client biome but no biome with that name exists: myns:nowhere"
    );
    assert_eq!(
        file(&unresolved[0]),
        ("only".to_owned(), "biomes/nowhere.json".to_owned())
    );
    assert_eq!(
        unresolved[0].location.json_path.to_string(),
        "minecraft:client_biome/description/identifier"
    );
}

/// `st05_glue_messages`: the rows that look a name up, each with vanilla's text.
/// V4.7–V4.10 need no registry and are checked in `rules.rs`.
#[cfg(all(
    feature = "biome",
    feature = "biomes_client",
    feature = "fog",
    feature = "deferred_settings"
))]
#[test]
fn st05_glue_messages() {
    let rows = [
        (
            "fog_appearance",
            "fog_identifier",
            "fog_identifier: cannot find fog definition 'x:none'",
        ),
        (
            "water_identifier",
            "water_identifier",
            "water_identifier: cannot find water definition 'x:none'",
        ),
        (
            "lighting_identifier",
            "lighting_identifier",
            "lighting_identifier: cannot find lighting definition 'x:none'",
        ),
        (
            "atmosphere_identifier",
            "atmosphere_identifier",
            "atmosphere_identifier: cannot find atmosphere definition 'x:none'",
        ),
        (
            "color_grading_identifier",
            "color_grading_identifier",
            "color_grading_identifier: cannot find color grading definition 'x:none'",
        ),
        (
            "cubemap_identifier",
            "cubemap_identifier",
            "cubemap_identifier: cannot find cubemap definition 'x:none'",
        ),
    ];
    for (index, (component, member, text)) in rows.into_iter().enumerate() {
        let only = pack(
            10 + index as u128,
            "only",
            PackOrigin::User,
            vec![(
                "biomes/glade.json",
                biome(
                    "1.21.130",
                    "myns:glade",
                    &format!(r#""minecraft:{component}": {{"{member}": "x:none"}}"#),
                ),
            )],
        );
        let stacks = world_stacks(vec![only], LoadOptions::default());
        let report = stacks.link();
        let unresolved: Vec<_> = linked(&report, |c| c.id() == "core/unresolved_reference_worded")
            .into_iter()
            .filter(|d| d.code.to_string() == text)
            .collect();
        assert_eq!(unresolved.len(), 1, "{component}");
        assert_eq!(unresolved[0].severity, Severity::Error);
        assert_eq!(
            unresolved[0].location.json_path.to_string(),
            format!("minecraft:client_biome/components/minecraft:{component}/{member}")
        );
    }
}

/// `st06_non_json_extension`: only `.json` files are client biomes. A file in a sub-folder is
/// read: the walk's recursion flag is `false`, and a pack outside the app package recurses
/// ("not recursive" is the family's reading of that flag).
#[test]
fn st06_non_json_extension() {
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![
            ("biomes/plains.txt", sky("minecraft:plains", "#111111")),
            ("biomes/plains.json5", sky("minecraft:plains", "#111111")),
            ("biomes/sub/desert.json", sky("minecraft:desert", "#222222")),
            ("biomes/short.json", sky("myns:short", "#333333")),
            ("biome/elsewhere.json", sky("myns:elsewhere", "#444444")),
            ("glade.json", sky("myns:root", "#555555")),
        ],
    );
    let stack = stack(vec![only]);
    assert_eq!(sky_of(&stack, "minecraft:plains"), None);
    assert_eq!(
        sky_of(&stack, "minecraft:desert").as_deref(),
        Some("#222222")
    );
    assert_eq!(sky_of(&stack, "myns:short").as_deref(), Some("#333333"));
    assert_eq!(stack.client_biomes().len(), 2);
    assert!(stack.client_biomes().rejected().is_empty());
}

/// `st07_behavior_pack_biomes_dir`: client biomes are a resource stack's; a behaviour pack is
/// no member of one, and its `biomes/` holds server biomes.
#[test]
fn st07_behavior_pack_biomes_dir() {
    assert_eq!(<ClientBiome as Content>::SIDES, &[PackSide::Resource]);
    let text = manifest(1, "data").replace("resources", "data");
    let files = vec![
        ("manifest.json".to_owned(), text.into_bytes()),
        (
            "biomes/x.json".to_owned(),
            sky("myns:x", "#111111").into_bytes(),
        ),
    ];
    let files = files
        .into_iter()
        .map(|(path, bytes)| (bedrock_addon::storage::PackPath::new(&path).unwrap(), bytes));
    let behavior = Pack::from_files(files, &bedrock_addon::pack::OpenOptions::default()).unwrap();
    assert!(PackStack::<Resource>::of([behavior.clone()], LoadOptions::default()).is_err());
    // On its own side the file is a server biome that does not load, never a client biome.
    let stack = PackStack::<Behavior>::of([behavior], LoadOptions::default()).unwrap();
    #[cfg(feature = "biome")]
    assert!(stack.biomes().is_empty());
    drop(stack);
}

/// V5.3 (ours: the file first by name; vanilla: the file its directory walk meets first): two
/// documents of one identifier in one pack.
#[test]
fn a_second_document_in_one_pack_is_dropped_deviation() {
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![
            ("biomes/b.json", sky("minecraft:plains", "#bbbbbb")),
            ("biomes/a.json", sky("minecraft:plains", "#aaaaaa")),
            ("biomes/c.json", sky("minecraft:plains", "#cccccc")),
        ],
    );
    let stack = stack(vec![only]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#aaaaaa")
    );
    let duplicates = found(&stack, |c| c.id() == "core/duplicate_identifier");
    assert_eq!(duplicates.len(), 2, "{duplicates:#?}");
    // The client has no line for it: an information of ours.
    assert!(duplicates.iter().all(|d| d.severity == Severity::Info));
    let files: Vec<String> = duplicates.iter().map(|d| file(d).1).collect();
    assert_eq!(files, ["biomes/b.json", "biomes/c.json"]);
    assert!(found(&stack, is_shadowed_definition).is_empty());
}

/// Beta documents go to the beta registry (from a base-game pack, or with Beta APIs).
#[test]
fn beta_documents_go_to_the_beta_registry() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "myns:beta"}, "components": {}}"#,
    );
    let vanilla = pack(
        1,
        "vanilla",
        PackOrigin::Package,
        vec![("biomes/beta.json", text.clone())],
    );
    let stack = stack(vec![vanilla]);
    let biomes = stack.client_biomes();
    assert!(biomes.is_empty());
    let beta = biomes.beta().expect("a beta registry");
    assert!(beta.get(&BiomeId::from_static("myns:beta")).is_some());
    // A user pack's beta document needs the toggle.
    let user = pack(
        2,
        "user",
        PackOrigin::User,
        vec![("biomes/beta.json", text)],
    );
    let stack = self::stack(vec![user]);
    assert_eq!(stack.client_biomes().rejected().len(), 1);
}

/// `Placed::default_path` is `biomes/<namespace>/<name>.json`, a path the walk reads back.
#[test]
fn the_default_path_is_below_biomes_and_loads() {
    let built = ClientBiome::new(BiomeId::from_static("demo:glade"));
    let path = built.default_path();
    assert_eq!(path.as_str(), "biomes/demo/glade.json");
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![(path.as_str(), built.to_json().unwrap())],
    );
    let stack = stack(vec![only]);
    let biomes = stack.client_biomes();
    let loaded = biomes
        .get(&BiomeId::from_static("demo:glade"))
        .expect("loaded");
    assert_eq!(*loaded.value(), built);
    let beta = bedrock_addon::families::client_biome::Beta::new(BiomeId::from_static("demo:glade"));
    assert_eq!(beta.default_path().as_str(), "biomes/demo/glade.json");
}

/// A second document of an identifier in one pack was parsed (and logged) before the client
/// ignored it: its parse findings stay, and the component checks never run on it.
#[test]
fn a_same_pack_loser_keeps_its_parse_findings() {
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![
            ("biomes/a.json", sky("minecraft:plains", "#aaaaaa")),
            (
                "biomes/b.json",
                biome(
                    "1.21.130",
                    "minecraft:plains",
                    r#""minecraft:sky_color": {"sky_color": [300, 0, 0]}, "minecraft:biome_music": {"volume_multiplier": 2.0}"#,
                ),
            ),
        ],
    );
    let stack = stack(vec![only]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#aaaaaa")
    );
    let duplicates = found(&stack, |c| c.id() == "core/duplicate_identifier");
    assert_eq!(duplicates.len(), 1, "{duplicates:#?}");
    assert_eq!(file(&duplicates[0]).1, "biomes/b.json");
    // The parse's information: a colour channel above 255, clamped.
    let clamped = found(&stack, |c| c.id() == "core/invalid_value");
    assert_eq!(clamped.len(), 1, "{clamped:#?}");
    assert_eq!(clamped[0].severity, Severity::Info);
    assert_eq!(file(&clamped[0]).1, "biomes/b.json");
    assert!(
        stack
            .client_biomes()
            .diagnostics()
            .all(|d| d.severity < Severity::Error),
        "the loser's component checks do not run"
    );
    assert!(stack.client_biomes().rejected().is_empty());
}

/// The client checks the components of the document that claims the identifier, and of no
/// other: a shadowed document that would fail them raises nothing.
#[test]
fn a_shadowed_document_is_not_glue_checked() {
    let bad = |file: &'static str| {
        (
            file,
            biome(
                "1.21.130",
                "minecraft:plains",
                r#""minecraft:biome_music": {"volume_multiplier": 2.0}"#,
            ),
        )
    };
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#111111"))],
    );
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![bad("biomes/plains.json")],
    );
    let stack = stack(vec![top, bottom]);
    assert_eq!(
        sky_of(&stack, "minecraft:plains").as_deref(),
        Some("#111111")
    );
    assert!(stack.client_biomes().rejected().is_empty());
    let all: Vec<_> = stack.client_biomes().diagnostics().collect();
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(all[0].code.id(), "core/shadowed_definition");
    assert_eq!(file(all[0]).0, "bottom");
    // The same document on top is the claiming one: it is checked, fails, and still claims.
    let top = pack(1, "top", PackOrigin::User, vec![bad("biomes/plains.json")]);
    let bottom = pack(
        2,
        "bottom",
        PackOrigin::User,
        vec![("biomes/plains.json", sky("minecraft:plains", "#222222"))],
    );
    let stack = self::stack(vec![top, bottom]);
    assert!(stack.client_biomes().is_empty());
    assert_eq!(stack.client_biomes().rejected().len(), 1);
    let glue: Vec<_> = stack
        .client_biomes()
        .diagnostics()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(glue.len(), 1, "{glue:#?}");
    assert_eq!(
        glue[0].location.json_path.to_string(),
        "minecraft:client_biome/components/minecraft:biome_music/volume_multiplier"
    );
    assert_eq!(
        file(glue[0]),
        ("top".to_owned(), "biomes/plains.json".to_owned())
    );
    // The lower document is shadowed all the same: its own info, beside the claim's warning.
    let shadowed = found(&stack, |c| c.id() == "core/shadowed_definition");
    assert_eq!(shadowed.len(), 1, "{shadowed:#?}");
    assert_eq!(shadowed[0].severity, Severity::Info);
    assert_eq!(
        file(&shadowed[0]),
        ("bottom".to_owned(), "biomes/plains.json".to_owned())
    );
    let claims = found(&stack, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS));
    assert_eq!(claims.len(), 1, "{claims:#?}");
    assert_eq!(claims[0].severity, Severity::Info);
    assert_eq!(file(&claims[0]).0, "top");
    assert_eq!(stack.client_biomes().diagnostics().count(), 3);
}

/// The order of the client's checks (ours: the component checks run at load, the lookup of the
/// biome at link; vanilla looks the biome up first and stops there with "Loaded client biome
/// but no biome with that name exists: %s", so it never checks the components of a document
/// for a biome the server does not send). Today such a document gets the component error.
#[test]
fn a_document_for_an_unknown_biome_is_still_glue_checked_deviation() {
    let only = pack(
        1,
        "only",
        PackOrigin::User,
        vec![(
            "biomes/nowhere.json",
            biome(
                "1.21.130",
                "myns:nowhere",
                r#""minecraft:biome_music": {"volume_multiplier": 2.0}"#,
            ),
        )],
    );
    let stack = stack(vec![only]);
    assert!(stack.client_biomes().is_empty());
    let all: Vec<_> = stack.client_biomes().diagnostics().collect();
    assert_eq!(all.len(), 1, "{all:#?}");
    assert_eq!(all[0].code.id(), "client_biome/volume");
    assert_eq!(all[0].severity, Severity::Error);
    assert_eq!(
        all[0].code.to_string(),
        "Volume multiplier must be between 0 and 1, inclusive."
    );
    assert_eq!(
        all[0].location.json_path.to_string(),
        "minecraft:client_biome/components/minecraft:biome_music/volume_multiplier"
    );
}

/// A stable document whose component checks fail, named `identifier`.
fn failing(identifier: &str) -> String {
    biome(
        "1.21.40",
        identifier,
        r#""minecraft:biome_music": {"volume_multiplier": 2}"#,
    )
}

/// A beta document named `identifier`.
fn beta_document(identifier: &str) -> String {
    document(
        "beta",
        &format!(r#"{{"description": {{"identifier": "{identifier}"}}, "components": {{}}}}"#),
    )
}

/// The pack of the effective beta document `identifier`.
fn beta_pack_of(stack: &PackStack<Resource>, identifier: &str) -> Option<String> {
    let beta = stack.client_biomes().beta()?;
    let document = beta.get(&BiomeId::parse(identifier))?;
    Some(document.origin().pack.name().to_owned())
}

// "the highest-priority pack that has a loadable document
// for an identifier wins the whole identifier: no component merge, and a lower pack is not
// consulted even when the winner is later rejected in steps 2–3". The claim is by identifier,
// whatever `format_version` the lower document has: a beta document below a failed claim is
// as shadowed as a stable one.
#[test]
fn two_packs_a_failed_claim_keeps_the_lower_beta_document_out() {
    let top = pack(
        1,
        "top",
        PackOrigin::User,
        vec![("biomes/b.json", failing("myns:b"))],
    );
    let vanilla = pack(
        2,
        "vanilla",
        PackOrigin::Package,
        vec![("biomes/b.json", beta_document("myns:b"))],
    );
    let stack = stack(vec![top, vanilla]);
    assert!(stack.client_biomes().is_empty());
    assert_eq!(beta_pack_of(&stack, "myns:b"), None);
    let claims = found(&stack, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS));
    assert_eq!(claims.len(), 1, "{claims:#?}");
    assert_eq!(claims[0].severity, Severity::Info);
    assert_eq!(
        file(&claims[0]),
        ("top".to_owned(), "biomes/b.json".to_owned())
    );
}

#[test]
fn three_packs_a_failed_claim_keeps_the_stable_and_the_beta_document_below_it_out() {
    let layers = |top: String, middle: String, low: String| {
        stack(vec![
            pack(1, "top", PackOrigin::User, vec![("biomes/b.json", top)]),
            pack(
                2,
                "middle",
                PackOrigin::Package,
                vec![("biomes/b.json", middle)],
            ),
            pack(3, "low", PackOrigin::Package, vec![("biomes/b.json", low)]),
        ])
    };
    // A stable document, then a beta one, below the failed claim: neither is used, and the
    // claim is reported once.
    let loaded = layers(
        failing("myns:b"),
        sky("myns:b", "#222222"),
        beta_document("myns:b"),
    );
    assert_eq!(sky_of(&loaded, "myns:b"), None);
    assert_eq!(beta_pack_of(&loaded, "myns:b"), None);
    let claims = found(&loaded, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS));
    assert_eq!(claims.len(), 1, "{claims:#?}");
    // The other order below the claim.
    let loaded = layers(
        failing("myns:b"),
        beta_document("myns:b"),
        sky("myns:b", "#333333"),
    );
    assert_eq!(sky_of(&loaded, "myns:b"), None);
    assert_eq!(beta_pack_of(&loaded, "myns:b"), None);
    // A failed claim in the middle shadows nothing above it: the top beta document stays.
    let loaded = stack(vec![
        pack(
            1,
            "top",
            PackOrigin::Package,
            vec![("biomes/b.json", beta_document("myns:b"))],
        ),
        pack(
            2,
            "middle",
            PackOrigin::User,
            vec![("biomes/b.json", failing("myns:b"))],
        ),
        pack(
            3,
            "low",
            PackOrigin::Package,
            vec![("biomes/b.json", sky("myns:b", "#333333"))],
        ),
    ]);
    assert_eq!(beta_pack_of(&loaded, "myns:b").as_deref(), Some("top"));
    assert_eq!(sky_of(&loaded, "myns:b"), None);
    assert!(found(&loaded, |c| c.is(&codes::CLIENT_BIOME_SHADOWS_LOWER_PACKS)).is_empty());
}

// The game's walk over `biomes` takes a recursion flag set to false;
// the flag is overridden for a pack: a directory pack outside
// the app package is walked with its sub-folders, a pack shipped in the app package flat.
#[test]
fn a_sub_folder_of_biomes_is_read_in_a_user_pack_and_not_in_a_base_game_pack() {
    let files = || {
        vec![
            ("biomes/flat.json", sky("myns:flat", "#111111")),
            ("biomes/sub/nested.json", sky("myns:nested", "#222222")),
            ("biomes/sub/deeper/far.json", sky("myns:far", "#333333")),
        ]
    };
    let user = stack(vec![pack(1, "user", PackOrigin::User, files())]);
    assert_eq!(user.client_biomes().len(), 3);
    let shipped = stack(vec![pack(1, "vanilla", PackOrigin::Package, files())]);
    assert_eq!(sky_of(&shipped, "myns:flat").as_deref(), Some("#111111"));
    assert_eq!(shipped.client_biomes().len(), 1);
    assert!(shipped.client_biomes().rejected().is_empty());
    assert_eq!(shipped.client_biomes().diagnostics().count(), 0);
}

/// A `minecraft:` client biome is written to `biomes/<name>.json`, which every pack's walk
/// reads; a base-game pack reads no other place.
#[test]
fn the_default_path_of_a_minecraft_client_biome_is_read_back_by_a_base_game_pack() {
    let id = BiomeId::from_static("minecraft:plains");
    let built = ClientBiome::new(id.clone());
    let path = built.default_path();
    assert_eq!(path.as_str(), "biomes/plains.json");
    let beta = bedrock_addon::families::client_biome::Beta::new(id.clone());
    assert_eq!(beta.default_path().as_str(), "biomes/plains.json");
    for origin in [PackOrigin::Package, PackOrigin::User] {
        let only = pack(
            1,
            "only",
            origin,
            vec![(path.as_str(), built.to_json().unwrap())],
        );
        let stack = stack(vec![only]);
        let biomes = stack.client_biomes();
        assert_eq!(*biomes.get(&id).expect("loaded").value(), built);
    }
}
