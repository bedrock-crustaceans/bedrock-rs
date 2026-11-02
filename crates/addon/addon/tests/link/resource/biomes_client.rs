//! 20 R45 to R49: what a client biome and `biomes_client.json` name outside their own files
//! (the biome, a fog, the deferred-rendering settings, the ambient sounds), what a claim the
//! pass refuses costs the packs below it, and biomes_client V6.10.

use bedrock_addon::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, Phase, ReferenceHint, Severity,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, StackSet};

use crate::common::{hosted, link, link_with, resource_pack, stacks_of};

const FILE: &str = "biomes/glade.client_biome.json";
const IDENTIFIER: &str = "minecraft:client_biome/description/identifier";

/// A client biome of `version` for `identifier` with `components` (a JSON object's members).
fn client(version: &str, identifier: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:client_biome": {{
            "description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}}}"#
    )
}

/// A client biome of the newest format for a biome of the game.
fn plains(components: &str) -> String {
    client("1.21.130", "minecraft:plains", components)
}

/// A behaviour-pack biome `identifier`.
fn biome(identifier: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:biome": {{"description": {{"identifier": "{identifier}"}},
            "components": {{"minecraft:climate": {{"temperature": 0.5}}}}}}}}"#
    )
}

/// A fog definition `identifier`.
#[cfg(feature = "fog")]
fn fog(identifier: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:fog_settings": {{"description": {{"identifier": "{identifier}"}}}}}}"#
    )
}

/// What a finding says: its code, severity, phase, pack, file, JSON path and the kind and
/// name of its detail.
type Told = (
    &'static str,
    Severity,
    Phase,
    String,
    String,
    String,
    &'static str,
    String,
);

fn told(finding: &Diagnostic) -> Told {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.phase,
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

/// The findings of the rules of this file: every finding of a client-biome file or of
/// `biomes_client.json`.
fn of_biomes(found: &Diagnostics) -> Vec<Told> {
    // The client's own findings: the game's validator line beside each is
    // `linked_asset.rs`'s.
    found
        .iter()
        .filter(|finding| !crate::common::is_second_validator_line(finding, found))
        .filter(|finding| {
            finding.location.path.as_ref().is_some_and(|path| {
                path.as_str().starts_with("biomes/") || path.as_str() == "biomes_client.json"
            })
        })
        .map(told)
        .collect()
}

/// The texts of the findings [`of_biomes`] selects.
#[cfg(all(feature = "fog", feature = "deferred_settings"))]
fn texts(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .filter(|finding| !crate::common::is_second_validator_line(finding, found))
        .filter(|finding| {
            finding.location.path.as_ref().is_some_and(|path| {
                path.as_str().starts_with("biomes/") || path.as_str() == "biomes_client.json"
            })
        })
        .map(|finding| finding.code.to_string())
        .collect()
}

/// A world of the resource packs `packs` (top first, `rp 2`, `rp 3`, …), linked.
fn linked(packs: &[&[(&'static str, &str)]]) -> Diagnostics {
    world(packs).link()
}

/// The stacks of [`linked`].
fn world(packs: &[&[(&'static str, &str)]]) -> StackSet {
    let packs = packs
        .iter()
        .zip(2..)
        .map(|(files, n)| resource_pack(n, files))
        .collect();
    stacks_of(Vec::new(), packs, LoadOptions::default())
}

// ---- R45: the biome ------------------------------------------------------------------------

#[test]
fn biome_r45_a_client_biome_of_a_biome_a_behaviour_pack_defines_is_no_finding() {
    let found = link(
        &[("biomes/moor.json", &biome("demo:moor"))],
        &[(FILE, &client("1.21.130", "demo:moor", ""))],
    );
    assert_eq!(of_biomes(&found), []);
}

#[test]
fn biome_r45_a_client_biome_of_a_biome_of_the_game_is_no_finding() {
    // Without a host the crate's own list of the game's biomes answers.
    assert_eq!(of_biomes(&link(&[], &[(FILE, &plains(""))])), []);
    // With a host, the host does: yes for a biome only it knows, no for one it does not list.
    let host = || hosted(&[("biome", "demo:hosted")]);
    let options = LinkOptions::default();
    let found = link_with(
        &[],
        &[(FILE, &client("1.21.130", "demo:hosted", ""))],
        host(),
        &options,
    );
    assert_eq!(of_biomes(&found), []);
    let found = link_with(&[], &[(FILE, &plains(""))], host(), &options);
    assert_eq!(
        of_biomes(&found)
            .iter()
            .map(|told| told.7.as_str())
            .collect::<Vec<_>>(),
        ["minecraft:plains"]
    );
}

#[test]
fn biome_r45_a_client_biome_of_no_biome_is_an_error_with_the_clients_text() {
    let found = link(&[], &[(FILE, &client("1.21.130", "demo:nowhere", ""))]);
    assert_eq!(
        of_biomes(&found),
        [(
            "UnresolvedReference",
            Severity::Error,
            Phase::Link,
            "rp 2".to_owned(),
            FILE.to_owned(),
            IDENTIFIER.to_owned(),
            "biome",
            "demo:nowhere".to_owned()
        )]
    );
    let finding = found
        .iter()
        .find(|finding| finding.code.name() == "UnresolvedReference")
        .expect("the finding");
    assert_eq!(
        finding.code.to_string(),
        "Loaded client biome but no biome with that name exists: demo:nowhere"
    );
}

#[test]
fn biome_r45_an_older_documents_bare_name_is_a_minecraft_biome() {
    // Both sides of the upgrade's `minecraft:` prefix: `plains` is the game's biome, and
    // `nowhere` is `minecraft:nowhere`, which nothing is.
    let found = link(&[], &[(FILE, &client("1.21.40", "plains", ""))]);
    assert_eq!(of_biomes(&found), []);
    let found = link(&[], &[(FILE, &client("1.21.40", "nowhere", ""))]);
    assert_eq!(
        of_biomes(&found)
            .iter()
            .map(|told| (told.0, told.1, told.5.as_str(), told.7.as_str()))
            .collect::<Vec<_>>(),
        [(
            "UnresolvedReference",
            Severity::Error,
            IDENTIFIER,
            "minecraft:nowhere"
        )]
    );
}

// the component checks of the load and the lookup of the link pass stand side by side.
#[test]
fn biome_r45_a_claim_the_component_checks_refused_is_still_asked_for_its_biome() {
    let opaque = r#""minecraft:water_appearance": {"surface_opacity": 2.0}"#;
    let set = world(&[&[(FILE, &client("1.21.130", "demo:nowhere", opaque))]]);
    // The load refused the document (V4.8) and keeps no document of it.
    let registry = set.resource.client_biomes();
    assert_eq!((registry.len(), registry.rejected().len()), (0, 1));
    assert_eq!(
        of_biomes(&set.link()),
        [(
            "UnresolvedReference",
            Severity::Error,
            Phase::Link,
            "rp 2".to_owned(),
            FILE.to_owned(),
            IDENTIFIER.to_owned(),
            "biome",
            "demo:nowhere".to_owned()
        )]
    );
    // The same document for a biome of the game: the load's error is all there is.
    let set = world(&[&[(FILE, &plains(opaque))]]);
    assert_eq!(set.resource.client_biomes().rejected().len(), 1);
    assert_eq!(of_biomes(&set.link()), []);
    // A file that does not load at all claims nothing and names no biome.
    let set = world(&[&[(FILE, "{ not json")]]);
    assert_eq!(set.resource.client_biomes().rejected().len(), 1);
    assert_eq!(of_biomes(&set.link()), []);
}

/// K122: a refused document claims its biome only where the client looks at it: below a
/// higher pack's document for the same biome it is shadowed, refused or not, and the unknown
/// biome is told once, at the claim.
#[test]
fn biome_r45_a_refused_document_a_higher_pack_shadows_is_not_asked_again() {
    let opaque = r#""minecraft:water_appearance": {"surface_opacity": 2.0}"#;
    let valid = client("1.21.130", "demo:nowhere", "");
    let refused = client("1.21.130", "demo:nowhere", opaque);
    let unknown = |set: &bedrock_addon::pack::StackSet| -> Vec<Told> {
        of_biomes(&set.link())
            .into_iter()
            .filter(|told| told.0 == "UnresolvedReference")
            .collect()
    };
    let at = |pack: &str, file: &str| {
        (
            "UnresolvedReference",
            Severity::Error,
            Phase::Link,
            pack.to_owned(),
            file.to_owned(),
            IDENTIFIER.to_owned(),
            "biome",
            "demo:nowhere".to_owned(),
        )
    };
    // A valid claim above a refused document.
    let set = world(&[&[(FILE, &valid)], &[("biomes/low.json", &refused)]]);
    assert_eq!(unknown(&set), [at("rp 2", FILE)]);
    // Two refused documents: the higher one claims.
    let set = world(&[&[(FILE, &refused)], &[("biomes/low.json", &refused)]]);
    eprintln!(
        "rejected: {}",
        set.resource.client_biomes().rejected().len()
    );
    assert_eq!(unknown(&set), [at("rp 2", FILE)]);
    // A refused document above a valid one still claims (the valid one is shadowed).
    let set = world(&[&[(FILE, &refused)], &[("biomes/low.json", &valid)]]);
    assert_eq!(unknown(&set), [at("rp 2", FILE)]);
}

// ---- the claim of a refused document ---------------------------------------------------------

/// The `ClientBiomeShadowsLowerPacks` findings: `(pack, file, JSON path, severity, detail)`.
fn shadows(found: &Diagnostics) -> Vec<(String, String, String, Severity, String)> {
    found
        .iter()
        .filter(|finding| finding.code.name() == "ClientBiomeShadowsLowerPacks")
        .map(|finding| {
            let told = told(finding);
            let detail = finding
                .detail
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            assert_eq!(finding.phase, Phase::Link);
            (told.3, told.4, told.5, told.1, detail)
        })
        .collect()
}

#[test]
fn biome_a_refused_claim_is_told_once_for_each_pack_it_shadows() {
    let top = client("1.21.130", "demo:nowhere", "");
    let lower = client(
        "1.21.130",
        "demo:nowhere",
        r##""minecraft:sky_color": {"sky_color": "#222222"}"##,
    );
    let other = client("1.21.130", "demo:elsewhere", "");
    let found = linked(&[
        &[(FILE, &top)],
        &[("biomes/a.json", &lower), ("biomes/other.json", &other)],
        &[("biomes/b.json", &lower)],
    ]);
    let shadow = |pack: &str| {
        (
            "rp 2".to_owned(),
            FILE.to_owned(),
            IDENTIFIER.to_owned(),
            Severity::Info,
            format!(": pack '{pack}' (shadowed by a higher-priority definition)"),
        )
    };
    assert_eq!(shadows(&found), [shadow("rp 3"), shadow("rp 4")]);
    // The detail is a reference to the pack, by its name.
    let finding = found
        .iter()
        .find(|finding| finding.code.name() == "ClientBiomeShadowsLowerPacks")
        .expect("the warning");
    assert!(
        matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { kind: "pack", id, hint: Some(ReferenceHint::Shadowed) })
                if id == "rp 3"
        ),
        "{finding:?}"
    );
    // The registry is as the load left it: the refused claim is still its document.
    let set = world(&[&[(FILE, &top)], &[("biomes/a.json", &lower)]]);
    let _ = set.link();
    let id = bedrock_addon::id::BiomeId::parse("demo:nowhere");
    let kept = set.resource.client_biomes().get(&id).expect("the claim");
    assert_eq!(kept.origin().pack.name(), "rp 2");
}

#[test]
fn biome_a_refused_claim_that_shadows_nothing_is_no_warning() {
    // Alone, and above a pack whose document is for another biome.
    let top = client("1.21.130", "demo:nowhere", "");
    let other = client("1.21.130", "demo:elsewhere", "");
    assert_eq!(shadows(&linked(&[&[(FILE, &top)]])), []);
    let found = linked(&[&[(FILE, &top)], &[("biomes/other.json", &other)]]);
    assert_eq!(shadows(&found), []);
    // A claim the pass accepts shadows as every higher definition does: no warning.
    let found = linked(&[&[(FILE, &plains(""))], &[(FILE, &plains(""))]]);
    assert_eq!(shadows(&found), []);
}

// The pass runs on the claiming document alone.
#[test]
fn biome_a_shadowed_document_is_not_asked() {
    let unknown = client("1.21.130", "demo:nowhere", "");
    let found = linked(&[
        &[("biomes/top.json", &client("1.21.130", "demo:nowhere", ""))],
        &[("biomes/lower.json", &unknown)],
    ]);
    assert_eq!(
        of_biomes(&found)
            .iter()
            .filter(|told| told.0 == "UnresolvedReference")
            .map(|told| told.4.as_str())
            .collect::<Vec<_>>(),
        ["biomes/top.json"]
    );
}

// ---- R46, R47: the fog and the settings ------------------------------------------------------

#[cfg(all(feature = "fog", feature = "deferred_settings"))]
mod render {
    use super::*;

    const FOG_MEMBER: &str =
        "minecraft:client_biome/components/minecraft:fog_appearance/fog_identifier";

    fn fogged(identifier: &str) -> String {
        plains(&format!(
            r#""minecraft:fog_appearance": {{"fog_identifier": "{identifier}"}}"#
        ))
    }

    /// The five settings kinds: the component (and member), the kind's name, the folder's
    /// payload key and file, the word of the client's text.
    const SETTINGS: [(&str, &str, &str, &str, &str); 5] = [
        (
            "water_identifier",
            "water settings",
            "minecraft:water_settings",
            "water/dusk.json",
            "water",
        ),
        (
            "lighting_identifier",
            "lighting settings",
            "minecraft:lighting_settings",
            "lighting/dusk.json",
            "lighting",
        ),
        (
            "atmosphere_identifier",
            "atmosphere settings",
            "minecraft:atmosphere_settings",
            "atmospherics/dusk.json",
            "atmosphere",
        ),
        (
            "color_grading_identifier",
            "color grading settings",
            "minecraft:color_grading_settings",
            "color_grading/dusk.json",
            "color grading",
        ),
        (
            "cubemap_identifier",
            "cubemap settings",
            "minecraft:cubemap_settings",
            "cubemaps/dusk.json",
            "cubemap",
        ),
    ];

    /// A settings document of the payload `key` named `demo:dusk`, at a version every kind
    /// has a struct for.
    fn settings(key: &str) -> String {
        let version = match key {
            "minecraft:lighting_settings" => "1.26.0",
            "minecraft:color_grading_settings" => "1.21.90",
            "minecraft:cubemap_settings" => "1.21.130",
            _ => "1.26.20",
        };
        format!(
            r#"{{"format_version": "{version}", "{key}": {{"description": {{"identifier": "demo:dusk"}}}}}}"#
        )
    }

    fn naming(member: &str, identifier: &str) -> String {
        plains(&format!(
            r#""minecraft:{member}": {{"{member}": "{identifier}"}}"#
        ))
    }

    #[test]
    fn biome_r46_a_fog_a_pack_defines_is_no_finding() {
        let found = link(
            &[],
            &[
                (FILE, &fogged("demo:mist")),
                ("fogs/mist.json", &fog("demo:mist")),
            ],
        );
        assert_eq!(of_biomes(&found), []);
    }

    #[test]
    fn biome_r46_a_fog_of_the_game_is_found_with_a_host_and_missing_without_one() {
        let file = fogged("minecraft:fog_plains");
        let host = hosted(&[
            ("fog", "minecraft:fog_plains"),
            ("biome", "minecraft:plains"),
        ]);
        let found = link_with(&[], &[(FILE, &file)], host, &LinkOptions::default());
        assert_eq!(of_biomes(&found), []);
        // The crate has no list of the game's fogs: without a host and without the game's
        // packs on the stack the name is missing.
        let found = link(&[], &[(FILE, &file)]);
        assert_eq!(
            of_biomes(&found)
                .iter()
                .map(|told| (told.0, told.6, told.7.as_str()))
                .collect::<Vec<_>>(),
            [("UnresolvedReference", "fog", "minecraft:fog_plains")]
        );
    }

    #[test]
    fn biome_r46_a_fog_nothing_defines_is_an_error_with_the_clients_text() {
        let found = link(
            &[],
            &[
                (FILE, &fogged("demo:none")),
                ("fogs/mist.json", &fog("demo:mist")),
            ],
        );
        assert_eq!(
            of_biomes(&found),
            [(
                "UnresolvedReference",
                Severity::Error,
                Phase::Link,
                "rp 2".to_owned(),
                FILE.to_owned(),
                FOG_MEMBER.to_owned(),
                "fog",
                "demo:none".to_owned()
            )]
        );
        let texts = texts(&found);
        assert_eq!(
            texts,
            ["fog_identifier: cannot find fog definition 'demo:none'"]
        );
    }

    #[test]
    fn biome_r46_a_fog_is_compared_as_written() {
        // The registry has `demo:mist`; `Demo:Mist` is another name, and the finding says
        // which definition is near.
        let found = link(
            &[],
            &[
                (FILE, &fogged("Demo:Mist")),
                ("fogs/mist.json", &fog("demo:mist")),
            ],
        );
        let finding = found
            .iter()
            .find(|finding| finding.code.name() == "UnresolvedReference")
            .expect("the finding");
        assert_eq!(finding.severity, Severity::Error);
        assert!(
            matches!(
                finding.detail.as_deref(),
                Some(DiagnosticDetail::Reference { kind: "fog", id, hint: Some(ReferenceHint::CaseMismatch(near)) })
                    if id == "Demo:Mist" && near == "demo:mist"
            ),
            "{finding:?}"
        );
    }

    #[test]
    fn biome_r47_a_settings_identifier_a_pack_defines_is_no_finding() {
        for (member, _, key, file, _) in SETTINGS {
            let found = link(
                &[],
                &[(FILE, &naming(member, "demo:dusk")), (file, &settings(key))],
            );
            assert_eq!(of_biomes(&found), [], "{member}");
        }
    }

    #[test]
    fn biome_r47_a_settings_identifier_nothing_defines_is_an_error_with_the_clients_text() {
        for (member, kind, key, file, word) in SETTINGS {
            // A definition of the kind is on the stack under another name.
            let found = link(
                &[],
                &[(FILE, &naming(member, "demo:none")), (file, &settings(key))],
            );
            assert_eq!(
                of_biomes(&found),
                [(
                    "UnresolvedReference",
                    Severity::Error,
                    Phase::Link,
                    "rp 2".to_owned(),
                    FILE.to_owned(),
                    format!("minecraft:client_biome/components/minecraft:{member}/{member}"),
                    kind,
                    "demo:none".to_owned()
                )],
                "{member}"
            );
            let texts = texts(&found);
            assert_eq!(
                texts,
                [format!(
                    "{member}: cannot find {word} definition 'demo:none'"
                )]
            );
        }
    }

    #[test]
    fn biome_r47_a_definition_of_another_kind_does_not_answer() {
        // `demo:dusk` is a water definition only: the lighting identifier misses it.
        let found = link(
            &[],
            &[
                (FILE, &naming("lighting_identifier", "demo:dusk")),
                ("water/dusk.json", &settings("minecraft:water_settings")),
            ],
        );
        assert_eq!(
            of_biomes(&found)
                .iter()
                .map(|told| (told.6, told.7.as_str()))
                .collect::<Vec<_>>(),
            [("lighting settings", "demo:dusk")]
        );
    }

    #[test]
    fn biome_r47_a_settings_identifier_of_the_game_is_found_with_a_host_and_missing_without_one() {
        for (member, kind, ..) in SETTINGS {
            let file = naming(member, "minecraft:of_the_game");
            let list: &'static [(&'static str, &'static str)] = Box::leak(Box::new([
                (kind, "minecraft:of_the_game"),
                ("biome", "minecraft:plains"),
            ]));
            let found = link_with(&[], &[(FILE, &file)], hosted(list), &LinkOptions::default());
            assert_eq!(of_biomes(&found), [], "{member}");
            let found = link(&[], &[(FILE, &file)]);
            assert_eq!(of_biomes(&found).len(), 1, "{member}");
        }
    }

    #[test]
    fn biome_r47_an_empty_settings_name_is_accepted() {
        for (member, ..) in SETTINGS {
            let found = link(&[], &[(FILE, &naming(member, ""))]);
            assert_eq!(of_biomes(&found), [], "{member}");
        }
    }

    // render_configs K2: the game has the default cubemap whether or not a pack defines it.
    #[test]
    fn biome_r47_the_default_cubemap_always_exists() {
        let found = link(
            &[],
            &[(
                FILE,
                &naming("cubemap_identifier", "minecraft:default_cubemap"),
            )],
        );
        assert_eq!(of_biomes(&found), []);
        // The other four kinds have no such name.
        let found = link(
            &[],
            &[(
                FILE,
                &naming("lighting_identifier", "minecraft:default_lighting"),
            )],
        );
        assert_eq!(of_biomes(&found).len(), 1);
    }

    #[test]
    fn biome_r47_the_identifiers_an_upgrade_step_inserted_are_not_asked() {
        // A 1.21.40 document gets the four render identifiers of the game from the upgrade
        // (`minecraft:default_lighting`, …); no pack of this world defines them, and the
        // author wrote none.
        let old = client("1.21.40", "minecraft:plains", "");
        let set = world(&[&[(FILE, &old)]]);
        let id = bedrock_addon::id::BiomeId::parse("minecraft:plains");
        let document = set.resource.client_biomes().get(&id).expect("plains");
        assert!(
            document
                .components
                .contains::<bedrock_addon::families::client_biome::component::LightingIdentifier>()
        );
        assert_eq!(of_biomes(&set.link()), []);
        // The same identifier written by the author is asked.
        let written = client(
            "1.21.70",
            "minecraft:plains",
            r#""minecraft:lighting_identifier": {"lighting_identifier": "minecraft:default_lighting"}"#,
        );
        let found = linked(&[&[(FILE, &written)]]);
        assert_eq!(
            of_biomes(&found)
                .iter()
                .map(|told| (told.6, told.7.as_str()))
                .collect::<Vec<_>>(),
            [("lighting settings", "minecraft:default_lighting")]
        );
    }

    #[test]
    fn biome_a_document_refused_for_its_biome_and_its_fog_tells_its_claim_once() {
        let top = client(
            "1.21.130",
            "demo:nowhere",
            r#""minecraft:fog_appearance": {"fog_identifier": "demo:none"}"#,
        );
        let lower = client("1.21.130", "demo:nowhere", "");
        let found = linked(&[&[(FILE, &top)], &[("biomes/a.json", &lower)]]);
        assert_eq!(
            of_biomes(&found)
                .iter()
                .map(|told| (told.0, told.1, told.6, told.7.as_str()))
                .collect::<Vec<_>>(),
            [
                (
                    "UnresolvedReference",
                    Severity::Error,
                    "biome",
                    "demo:nowhere"
                ),
                (
                    "ClientBiomeShadowsLowerPacks",
                    Severity::Info,
                    "pack",
                    "rp 3"
                ),
                ("UnresolvedReference", Severity::Error, "fog", "demo:none"),
            ]
        );
    }

    #[test]
    fn biome_a_claim_refused_for_a_setting_shadows_the_packs_below() {
        let top = naming("water_identifier", "demo:none");
        let found = linked(&[&[(FILE, &top)], &[("biomes/a.json", &plains(""))]]);
        assert_eq!(
            shadows(&found),
            [(
                "rp 2".to_owned(),
                FILE.to_owned(),
                IDENTIFIER.to_owned(),
                Severity::Info,
                ": pack 'rp 3' (shadowed by a higher-priority definition)".to_owned()
            )]
        );
    }

    #[test]
    fn biome_a_shadowed_documents_fog_is_not_asked() {
        let found = linked(&[
            &[(FILE, &plains(""))],
            &[("biomes/lower.json", &fogged("demo:none"))],
        ]);
        assert_eq!(of_biomes(&found), []);
    }

    // ---- R49: the fogs of biomes_client.json ---------------------------------------------

    const LEGACY: &str = "biomes_client.json";

    #[test]
    fn biome_r49_a_legacy_fog_a_pack_defines_is_no_finding() {
        let found = link(
            &[],
            &[
                (
                    LEGACY,
                    r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:mist"]}}}"#,
                ),
                ("fogs/mist.json", &fog("demo:mist")),
            ],
        );
        assert_eq!(of_biomes(&found), []);
    }

    #[test]
    fn biome_r49_a_legacy_fog_nothing_defines_is_an_information_at_its_member() {
        let found = link(
            &[],
            &[
                (
                    LEGACY,
                    r#"{"biomes": {"default": {"fog_ids_to_merge": ["demo:none"]},
                        "plains": {"fog_ids_to_merge": ["demo:mist", "demo:nofog", "demo:nofog"]}}}"#,
                ),
                ("fogs/mist.json", &fog("demo:mist")),
            ],
        );
        // The game resolves the fog without a content-log line (only
        // the `NonOwnerPointer` asserts): ours, an information.
        let warning = |path: &str, id: &str| {
            (
                "UnresolvedReference",
                Severity::Info,
                Phase::Link,
                "rp 2".to_owned(),
                LEGACY.to_owned(),
                path.to_owned(),
                "fog",
                id.to_owned(),
            )
        };
        // The key as the file has it (`plains`, which the game reads as `minecraft:plains`);
        // a name is told once per entry.
        assert_eq!(
            of_biomes(&found),
            [
                warning("biomes/default/fog_ids_to_merge/0", "demo:none"),
                warning("biomes/plains/fog_ids_to_merge/1", "demo:nofog"),
            ]
        );
        // Ours: the client logs nothing.
        assert_eq!(texts(&found), ["unresolved reference"; 2]);
    }

    #[test]
    fn biome_r49_a_fog_the_merge_took_from_fog_identifier_is_told_there() {
        // The lower pack has the entry, so the higher pack's `fog_identifier` becomes the
        // merged entry's one fog.
        let found = linked(&[
            &[(
                LEGACY,
                r#"{"biomes": {"minecraft:plains": {"fog_identifier": "demo:none"}}}"#,
            )],
            &[(
                LEGACY,
                r##"{"biomes": {"plains": {"water_surface_color": "#000001"}}}"##,
            )],
        ]);
        assert_eq!(
            of_biomes(&found)
                .iter()
                .filter(|told| told.0 == "UnresolvedReference")
                .map(|told| (told.3.as_str(), told.5.as_str(), told.7.as_str()))
                .collect::<Vec<_>>(),
            [(
                "rp 2",
                "biomes/minecraft:plains/fog_identifier",
                "demo:none"
            )]
        );
    }

    #[test]
    fn biome_r49_the_fogs_to_merge_are_compared_as_written_and_an_old_fogs_name_gets_minecraft() {
        let host = || hosted(&[("fog", "minecraft:mist"), ("biome", "minecraft:plains")]);
        let unresolved = |text: &str| {
            let found = link_with(&[], &[(LEGACY, text)], host(), &LinkOptions::default());
            of_biomes(&found)
                .into_iter()
                .filter(|told| told.0 == "UnresolvedReference")
                .map(|told| (told.5, told.7))
                .collect::<Vec<_>>()
        };
        // `fog_ids_to_merge`: the name as written, no namespace added.
        assert_eq!(
            unresolved(r#"{"biomes": {"plains": {"fog_ids_to_merge": ["minecraft:mist"]}}}"#),
            []
        );
        assert_eq!(
            unresolved(r#"{"biomes": {"plains": {"fog_ids_to_merge": ["mist"]}}}"#),
            [(
                "biomes/plains/fog_ids_to_merge/0".to_owned(),
                "mist".to_owned()
            )]
        );
        // An old-style fog: `fog_identifier` without a namespace is `minecraft:<name>`.
        assert_eq!(
            unresolved(
                r#"{"biomes": {"plains": {"make_back_compat_fog": true, "fog_identifier": "mist"}}}"#
            ),
            []
        );
        assert_eq!(
            unresolved(
                r#"{"biomes": {"plains": {"make_back_compat_fog": true, "fog_identifier": "haze"}}}"#
            ),
            [(
                "biomes/plains/fog_identifier".to_owned(),
                "minecraft:haze".to_owned()
            )]
        );
    }

    // ---- V6.10: the fog --------------------------------------------------------------------

    /// The `ClientBiomeOverriddenByBiomesClient` findings.
    fn overrides(found: &Diagnostics) -> Vec<Told> {
        found
            .iter()
            .filter(|finding| finding.code.name() == "ClientBiomeOverriddenByBiomesClient")
            .map(told)
            .collect()
    }

    #[test]
    fn biome_v6_10_a_legacy_entry_that_yields_fog_overrides_the_fog_appearance() {
        let mist = fog("demo:mist");
        let biome = fogged("demo:mist");
        let with = |legacy: &str| {
            let found = linked(&[
                &[(LEGACY, legacy)],
                &[(FILE, &biome), ("fogs/mist.json", &mist)],
            ]);
            overrides(&found)
        };
        // Vanilla overwrites the component's values in silence: ours, an information.
        let warning = [(
            "ClientBiomeOverriddenByBiomesClient",
            Severity::Info,
            Phase::Link,
            "rp 3".to_owned(),
            FILE.to_owned(),
            "minecraft:client_biome/components/minecraft:fog_appearance".to_owned(),
            "pack",
            "rp 2".to_owned(),
        )];
        // Another fog, two fogs, an old-style fog: the entry's fog replaces the component's.
        assert_eq!(
            with(r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:other"]}}}"#),
            warning
        );
        assert_eq!(
            with(r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:mist", "demo:mist"]}}}"#),
            warning
        );
        assert_eq!(
            with(
                r##"{"biomes": {"plains": {"make_back_compat_fog": true, "fog_color": "#000000"}}}"##
            ),
            warning
        );
        // The same one fog, an entry without fog, another biome's entry, the default entry.
        for same in [
            r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:mist"]}}}"#,
            r#"{"biomes": {"plains": {"fog_ids_to_merge": []}}}"#,
            r#"{"biomes": {"desert": {"fog_ids_to_merge": ["demo:other"]}}}"#,
            r#"{"biomes": {"default": {"fog_ids_to_merge": ["demo:other"]}}}"#,
        ] {
            assert_eq!(with(same), [], "{same}");
        }
    }

    #[test]
    fn biome_v6_10_a_biome_without_fog_appearance_has_no_fog_to_override() {
        let found = linked(&[
            &[(
                LEGACY,
                r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:other"]}}}"#,
            )],
            &[(FILE, &plains(""))],
        ]);
        assert_eq!(overrides(&found), []);
    }
}

#[cfg(not(all(feature = "fog", feature = "deferred_settings")))]
#[test]
fn biome_without_the_fog_and_deferred_settings_features_a_fog_and_a_setting_are_not_asked() {
    let file = plains(
        r#""minecraft:fog_appearance": {"fog_identifier": "demo:none"},
           "minecraft:water_identifier": {"water_identifier": "demo:none"}"#,
    );
    let found = link(
        &[],
        &[
            (FILE, &file),
            (
                "biomes_client.json",
                r#"{"biomes": {"plains": {"fog_ids_to_merge": ["demo:none"]}}}"#,
            ),
        ],
    );
    assert_eq!(of_biomes(&found), []);
}

// ---- V6.10: the water surface ----------------------------------------------------------------

#[test]
fn biome_v6_10_each_pack_whose_entry_replaces_the_water_surface_is_named() {
    let biome = plains(
        r##""minecraft:water_appearance": {"surface_color": "#123456", "surface_opacity": 0.5}"##,
    );
    let found = linked(&[
        &[(
            "biomes_client.json",
            r##"{"biomes": {"default": {"water_surface_color": "#44AFF5"}}}"##,
        )],
        &[
            (FILE, &biome),
            (
                "biomes_client.json",
                r#"{"biomes": {"plains": {"water_surface_transparency": 0.2}}}"#,
            ),
        ],
    ]);
    let warning = |by: &str| {
        (
            "ClientBiomeOverriddenByBiomesClient",
            Severity::Info,
            Phase::Link,
            "rp 3".to_owned(),
            FILE.to_owned(),
            "minecraft:client_biome/components/minecraft:water_appearance".to_owned(),
            "pack",
            by.to_owned(),
        )
    };
    assert_eq!(of_biomes(&found), [warning("rp 2"), warning("rp 3")]);
}

#[test]
fn biome_v6_10_entries_that_change_nothing_the_component_set_are_no_override() {
    let colored = plains(r##""minecraft:water_appearance": {"surface_color": "#123456"}"##);
    for legacy in [
        // The same colour.
        r##"{"biomes": {"default": {"water_surface_color": "#123456"}}}"##,
        // A member the component did not set.
        r#"{"biomes": {"plains": {"water_surface_transparency": 0.2}}}"#,
        // Another biome's entry.
        r##"{"biomes": {"desert": {"water_surface_color": "#000001"}}}"##,
    ] {
        let found = linked(&[&[(FILE, &colored), ("biomes_client.json", legacy)]]);
        assert_eq!(of_biomes(&found), [], "{legacy}");
    }
    // A biome without the component has nothing to override.
    let found = linked(&[&[
        (FILE, &plains("")),
        (
            "biomes_client.json",
            r##"{"biomes": {"default": {"water_surface_color": "#000001"}}}"##,
        ),
    ]]);
    assert_eq!(of_biomes(&found), []);
}

// ---- R49, the music half: an audit ---------------------------------------------------------

// 20 R49: `biome_music.music_definition` is held by the game's link pass: a
// name nothing defines is an `UnresolvedReference` warning at the member with the validator's
// line (it was not checked).
#[cfg(all(feature = "sounds_json", feature = "music"))]
#[test]
fn biome_r49_a_music_definition_nothing_defines_is_an_error_with_the_validators_line() {
    let file = plains(r#""minecraft:biome_music": {"music_definition": "no.such.music"}"#);
    let set = world(&[&[(FILE, &file)]]);
    let registry = set.resource.client_biomes();
    assert_eq!((registry.len(), registry.rejected().len()), (1, 0));
    let about_music = |finding: &&Diagnostic| {
        matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { id, .. }) if id == "no.such.music"
        )
    };
    let found = set.link();
    let music: Vec<_> = found
        .iter()
        .filter(about_music)
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        music,
        [(
            "UnresolvedReference",
            Severity::Error,
            "minecraft:client_biome/components/minecraft:biome_music/music_definition".to_owned(),
            "-> components -> minecraft:biome_music -> music_definition:  Missing referenced asset no.such.music".to_owned()
        )]
    );
}

// ---- R48: the ambient sounds -----------------------------------------------------------------

#[cfg(feature = "sounds_json")]
mod sound {
    use super::*;

    const SOUNDS: (&str, &str) = (
        "sounds.json",
        r#"{"individual_named_sounds": {"sounds": {"known.sound": "x", "other.sound": "y"}}}"#,
    );

    fn ambient(version: &str, members: &str) -> String {
        client(
            version,
            "minecraft:plains",
            &format!(r#""minecraft:ambient_sounds": {{{members}}}"#),
        )
    }

    /// The `UnknownSound` findings: `(severity, JSON path below the component, kind, name)`.
    fn unknown(found: &Diagnostics) -> Vec<(Severity, String, &'static str, String)> {
        of_biomes(found)
            .into_iter()
            .map(|told| {
                assert_eq!(
                    (told.0, told.2, told.3.as_str(), told.4.as_str()),
                    ("UnknownSound", Phase::Link, "rp 2", FILE)
                );
                let below = told
                    .5
                    .strip_prefix("minecraft:client_biome/components/minecraft:ambient_sounds/")
                    .expect("a member of the component")
                    .to_owned();
                (told.1, below, told.6, told.7)
            })
            .collect()
    }

    #[test]
    fn biome_r48_sounds_that_are_named_sounds_are_no_finding() {
        let file = ambient(
            "1.21.130",
            r#""addition": {"asset": "known.sound", "chance": 0.5}, "loop": "other.sound",
               "mood": "known.sound", "underwater_addition": {"asset": "other.sound", "chance": 1},
               "underwater_loop": "known.sound", "underwater_mood": "other.sound""#,
        );
        assert_eq!(unknown(&link(&[], &[(FILE, &file), SOUNDS])), []);
    }

    #[test]
    fn biome_r48_each_of_the_six_members_is_asked() {
        let file = ambient(
            "1.21.130",
            r#""addition": {"asset": "no.a", "chance": 0.5}, "loop": "no.b", "mood": "no.c",
               "underwater_addition": {"asset": "no.d", "chance": 1},
               "underwater_loop": "no.e", "underwater_mood": "no.f""#,
        );
        let warning = |path: &str, name: &str| {
            (
                Severity::Error,
                path.to_owned(),
                "named sound",
                name.to_owned(),
            )
        };
        assert_eq!(
            unknown(&link(&[], &[(FILE, &file), SOUNDS])),
            [
                warning("addition/asset", "no.a"),
                warning("loop", "no.b"),
                warning("mood", "no.c"),
                warning("underwater_addition/asset", "no.d"),
                warning("underwater_loop", "no.e"),
                warning("underwater_mood", "no.f"),
            ]
        );
    }

    /// each member is a reference of its own to the game's link pass, which
    /// logs each miss: a name two members share is told at both (supersedes X7b's "once").
    #[test]
    fn biome_r48_a_name_is_told_at_each_member_and_the_empty_name_is_no_sound() {
        let file = ambient(
            "1.21.130",
            r#""loop": "missing.sound", "mood": "missing.sound", "underwater_loop": "",
               "underwater_mood": "missing.sound""#,
        );
        let at = |member: &str| {
            (
                Severity::Error,
                member.to_owned(),
                "named sound",
                "missing.sound".to_owned(),
            )
        };
        assert_eq!(
            unknown(&link(&[], &[(FILE, &file), SOUNDS])),
            [at("loop"), at("mood"), at("underwater_mood")]
        );
    }

    #[test]
    fn biome_r48_a_sound_name_is_compared_with_case() {
        let file = ambient("1.21.130", r#""loop": "Known.Sound""#);
        assert_eq!(
            unknown(&link(&[], &[(FILE, &file), SOUNDS])),
            [(
                Severity::Error,
                "loop".to_owned(),
                "named sound",
                "Known.Sound".to_owned()
            )]
        );
    }

    #[test]
    fn biome_r48_an_older_files_addition_is_the_name_itself_and_its_underwater_sounds_are_the_upgrades()
     {
        // Below 1.21.120 `addition` is a string, and the upgrade gives a biome of the game
        // the game's underwater ambience (`ambient.underwater.loop`), which this world's
        // `sounds.json` does not name: not the author's, not asked.
        let file = ambient("1.21.40", r#""addition": "no.a", "loop": "known.sound""#);
        assert_eq!(
            unknown(&link(&[], &[(FILE, &file), SOUNDS])),
            [(
                Severity::Error,
                "addition".to_owned(),
                "named sound",
                "no.a".to_owned()
            )]
        );
        // A biome of the game without the component gets one from the upgrade: not asked.
        let bare = client("1.21.40", "minecraft:plains", "");
        assert_eq!(unknown(&link(&[], &[(FILE, &bare), SOUNDS])), []);
    }

    #[test]
    fn biome_r48_without_a_sounds_json_every_name_is_unknown() {
        let file = ambient("1.21.130", r#""loop": "known.sound""#);
        assert_eq!(unknown(&link(&[], &[(FILE, &file)])).len(), 1);
    }
}

#[cfg(not(feature = "sounds_json"))]
#[test]
fn biome_without_the_sounds_json_feature_an_ambient_sound_is_not_asked() {
    let file = plains(r#""minecraft:ambient_sounds": {"loop": "missing.sound"}"#);
    assert_eq!(of_biomes(&link(&[], &[(FILE, &file)])), []);
}

/// The lints are no part of these rules: without them every finding above is still told.
#[test]
fn biome_the_rules_are_no_lints() {
    let file = client("1.21.130", "demo:nowhere", "");
    let found = link_with(
        &[],
        &[(FILE, &file)],
        LoadOptions::default(),
        &LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(of_biomes(&found).len(), 1);
}
