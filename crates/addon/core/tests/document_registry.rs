#![allow(clippy::pedantic, missing_docs)]
//! `Registry<T>` and the combine rules on synthetic documents: load order,
//! `layers` with and without `keep_shadowed`, `beta()` and `resolve`.

use std::sync::Arc;

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, EngineVersionDecision, NoBeta,
    Notice, Origin, Registry, RegistryBuilder, Rejected, Resolution, VersionInfo,
};
use bedrock_addon_core::id::{BlockId, Id, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{BuiltIns, LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use uuid::Uuid;

/// A toy definition: a name, a version an `EngineVersionMerge` rule decides by, a side list
/// and two optional fields a `FieldMerge` rule merges.
#[derive(Clone, Debug, PartialEq, Default)]
struct Toy {
    from: &'static str,
    version: u32,
    list: Vec<&'static str>,
    a: Option<u32>,
    b: Option<u32>,
}

impl Sealed for Toy {}

impl Content for Toy {
    const NAME: &'static str = "toy";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        unimplemented!("the tests build documents by hand")
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Ok(())
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Ok(String::new())
    }
}

impl Definition for Toy {
    type Kind = kind::Block;
    type File = Toy;
    type Beta = ToyBeta;
}

/// The toy family's beta struct.
#[derive(Clone, Debug, PartialEq, Default)]
struct ToyBeta {
    from: &'static str,
}

impl Sealed for ToyBeta {}

impl Content for ToyBeta {
    const NAME: &'static str = "toy beta";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        unimplemented!("the tests build documents by hand")
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Ok(())
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Ok(String::new())
    }
}

impl Definition for ToyBeta {
    type Kind = kind::Block;
    type File = ToyBeta;
    type Beta = NoBeta<kind::Block>;
}

/// The pack at stack `index` (0 is the top).
fn pack(index: usize) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        format!("pack {index}"),
    )
}

fn origin(pack: &PackRef, path: &'static str) -> Origin {
    Origin::new(
        pack.clone(),
        PackSide::Behavior,
        PackPath::from_static(path),
        None,
        Vec::new(),
    )
}

fn version() -> VersionInfo {
    let v = FormatVersion::new(1, 21, 0);
    VersionInfo::new(Some(v), false, v, v, false)
}

fn id(text: &'static str) -> BlockId {
    Id::from_static(text)
}

/// `value` from `pack`, at `blocks/<from>.json`.
fn document(pack: &PackRef, value: Toy) -> Document<Toy> {
    Document::new(
        value,
        origin(pack, "blocks/toy.json"),
        version(),
        Diagnostics::default(),
    )
}

fn toy(from: &'static str) -> Toy {
    Toy {
        from,
        ..Toy::default()
    }
}

fn stable(pack_index: usize, text: &'static str, from: &'static str) -> Contribution<Toy> {
    Contribution::Stable(id(text), document(&pack(pack_index), toy(from)))
}

fn beta(pack_index: usize, text: &'static str, from: &'static str) -> Contribution<Toy> {
    Contribution::Beta(
        id(text),
        Document::new(
            ToyBeta { from },
            origin(&pack(pack_index), "blocks/toy_beta.json"),
            version(),
            Diagnostics::default(),
        ),
    )
}

fn an_error() -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(bedrock_addon_core::diagnostic::Diagnostic::new(
        DiagnosticCode::MissingRequiredField { text: None },
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, JsonPath::new(), None),
    ));
    diagnostics
}

fn rejected(pack_index: usize, text: Option<&'static str>) -> Contribution<Toy> {
    Contribution::Rejected(
        text.map(id),
        Rejected::new(origin(&pack(pack_index), "blocks/broken.json"), an_error()),
    )
}

fn build(
    rule: CombineRule<Toy>,
    options: &LoadOptions,
    walk: Vec<Contribution<Toy>>,
) -> Registry<Toy> {
    let mut builder = RegistryBuilder::new(rule, CombineRule::top_wins(), options);
    for contribution in walk {
        builder.push(contribution);
    }
    builder.finish()
}

fn keep_shadowed() -> LoadOptions {
    LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    }
}

/// Which pack's value each layer of `text` is.
fn layers(registry: &Registry<Toy>, text: &'static str) -> Vec<&'static str> {
    registry
        .layers(&id(text))
        .map(|document| document.from)
        .collect()
}

fn codes(registry: &Registry<Toy>) -> Vec<(DiagnosticCode, Severity, Option<usize>)> {
    registry
        .diagnostics()
        .map(|d| {
            (
                d.code.clone(),
                d.severity,
                d.location.pack.as_ref().and_then(PackRef::index),
            )
        })
        .collect()
}

// ---- load order ----

#[test]
fn iteration_is_by_first_definition_and_an_override_takes_its_place() {
    // Bottom-up walk: pack 2 (lowest) first.
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![
            stable(2, "demo:a", "2a"),
            stable(2, "demo:b", "2b"),
            stable(2, "demo:c", "2c"),
            stable(1, "demo:b", "1b"),
            stable(1, "demo:d", "1d"),
        ],
    );
    let order: Vec<(&str, &str)> = registry
        .iter()
        .map(|(id, document)| (id.as_str(), document.from))
        .collect();
    assert_eq!(
        order,
        [
            ("demo:a", "2a"),
            ("demo:b", "1b"),
            ("demo:c", "2c"),
            ("demo:d", "1d")
        ]
    );
    assert_eq!(registry.len(), 4);
    assert!(!registry.is_empty());
    assert_eq!((&registry).into_iter().count(), 4);
}

#[test]
fn identifiers_compare_canonically() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![
            stable(1, "minecraft:stone", "vanilla"),
            stable(0, "stone", "override"),
        ],
    );
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.get(&id("stone")).unwrap().from, "override");
    assert_eq!(
        registry.get(&id("minecraft:stone")).unwrap().from,
        "override"
    );
}

// ---- TopWins ----

#[test]
fn top_wins_whatever_the_walk_order() {
    for walk in [
        vec![stable(1, "demo:x", "lower"), stable(0, "demo:x", "top")],
        vec![stable(0, "demo:x", "top"), stable(1, "demo:x", "lower")],
    ] {
        let registry = build(CombineRule::top_wins(), &keep_shadowed(), walk);
        assert_eq!(registry.get(&id("demo:x")).unwrap().from, "top");
        assert_eq!(layers(&registry, "demo:x"), ["top", "lower"]);
        assert_eq!(registry.diagnostics().count(), 0, "no notice by default");
    }
}

#[test]
fn top_wins_can_report_each_shadowed_definition() {
    let registry = build(
        CombineRule::TopWins {
            shadowed: Some(Notice {
                code: DiagnosticCode::ClientEntityShadowed { reason: None },
                severity: Severity::Info,
            }),
        },
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "lower"), stable(0, "demo:x", "top")],
    );
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::ClientEntityShadowed { reason: None },
            Severity::Info,
            Some(1)
        )]
    );
}

#[test]
fn two_definitions_in_one_pack_are_a_duplicate_and_the_later_wins() {
    let registry = build(
        CombineRule::top_wins(),
        &keep_shadowed(),
        vec![stable(0, "demo:x", "first"), stable(0, "demo:x", "second")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "second");
    assert_eq!(layers(&registry, "demo:x"), ["second", "first"]);
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::DuplicateIdentifier { text: None },
            Severity::Info,
            Some(0)
        )]
    );
}

#[test]
fn a_failed_top_document_does_not_reserve_its_identifier_under_top_wins() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "lower"), rejected(0, Some("demo:x"))],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "lower");
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::MissingRequiredField { text: None },
            Severity::Error,
            None
        )]
    );
}

// ---- layers and keep_shadowed ----

#[test]
fn layers_without_keep_shadowed_is_the_effective_one_only() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![
            stable(2, "demo:x", "bottom"),
            stable(1, "demo:x", "middle"),
            stable(0, "demo:x", "top"),
        ],
    );
    assert_eq!(layers(&registry, "demo:x"), ["top"]);
    assert_eq!(registry.layers(&id("demo:x")).len(), 1);
    assert_eq!(layers(&registry, "demo:missing"), Vec::<&str>::new());
}

#[test]
fn layers_with_keep_shadowed_lists_the_shadowed_top_first() {
    let registry = build(
        CombineRule::top_wins(),
        &keep_shadowed(),
        vec![
            stable(2, "demo:x", "bottom"),
            stable(0, "demo:x", "top"),
            stable(1, "demo:x", "middle"),
        ],
    );
    assert_eq!(layers(&registry, "demo:x"), ["top", "middle", "bottom"]);
}

// ---- BottomWins ----

#[test]
fn bottom_wins_keeps_the_lowest_and_reports_the_higher() {
    let registry = build(
        CombineRule::bottom_wins(),
        &keep_shadowed(),
        vec![stable(1, "demo:x", "lower"), stable(0, "demo:x", "top")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "lower");
    assert_eq!(layers(&registry, "demo:x"), ["lower", "top"]);
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::ShadowedDefinition { text: None },
            Severity::Info,
            Some(0)
        )]
    );
}

#[test]
fn bottom_wins_keeps_the_first_of_one_pack() {
    let registry = build(
        CombineRule::bottom_wins(),
        &LoadOptions::default(),
        vec![stable(0, "demo:x", "first"), stable(0, "demo:x", "second")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "first");
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::DuplicateIdentifier { text: None },
            Severity::Info,
            Some(0)
        )]
    );
}

// ---- LastLoadedWins ----

#[test]
fn last_loaded_wins_whatever_the_priority() {
    // A top-down walk: the lower pack loads last and wins.
    let registry = build(
        CombineRule::last_loaded_wins(),
        &LoadOptions::default(),
        vec![stable(0, "demo:x", "top"), stable(1, "demo:x", "lower")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "lower");
    assert_eq!(
        codes(&registry),
        [(
            DiagnosticCode::DuplicateIdentifier { text: None },
            Severity::Info,
            Some(0)
        )]
    );
}

// ---- TopWinsFallback ----

#[test]
fn top_wins_fallback_keeps_the_lower_valid_definition_and_reports_it() {
    let registry = build(
        CombineRule::TopWinsFallback {
            fallback: Some(Notice {
                code: DiagnosticCode::RenderControllerFallback,
                severity: Severity::Warning,
            }),
        },
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "lower"), rejected(0, Some("demo:x"))],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "lower");
    assert_eq!(registry.rejected()[0].origin.pack, pack(0));
    assert_eq!(
        codes(&registry),
        [
            (
                DiagnosticCode::MissingRequiredField { text: None },
                Severity::Error,
                None
            ),
            (
                DiagnosticCode::RenderControllerFallback,
                Severity::Warning,
                Some(0)
            ),
        ]
    );
}

#[test]
fn top_wins_fallback_is_silent_when_the_failure_is_lower() {
    let registry = build(
        CombineRule::TopWinsFallback {
            fallback: Some(Notice {
                code: DiagnosticCode::RenderControllerFallback,
                severity: Severity::Warning,
            }),
        },
        &LoadOptions::default(),
        vec![rejected(1, Some("demo:x")), stable(0, "demo:x", "top")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "top");
    assert!(
        !codes(&registry)
            .iter()
            .any(|(code, ..)| *code == DiagnosticCode::RenderControllerFallback)
    );
}

// ---- ClaimBeforeValidate ----

#[test]
fn claim_before_validate_a_failed_top_claim_leaves_the_identifier_undefined() {
    let registry = build(
        CombineRule::ClaimBeforeValidate {
            claimed: Some(Notice {
                code: DiagnosticCode::ClientBiomeShadowsLowerPacks,
                severity: Severity::Warning,
            }),
        },
        &keep_shadowed(),
        vec![
            stable(1, "demo:x", "lower"),
            rejected(0, Some("demo:x")),
            stable(1, "demo:y", "lower y"),
        ],
    );
    assert_eq!(registry.get(&id("demo:x")), None);
    assert!(!registry.contains(&id("demo:x")));
    assert_eq!(
        layers(&registry, "demo:x"),
        ["lower"],
        "listed, not effective"
    );
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["demo:y"]
    );
    assert_eq!(
        codes(&registry).last(),
        Some(&(
            DiagnosticCode::ClientBiomeShadowsLowerPacks,
            Severity::Warning,
            Some(0)
        ))
    );
}

#[test]
fn claim_before_validate_a_failed_lower_claim_changes_nothing() {
    let registry = build(
        CombineRule::ClaimBeforeValidate { claimed: None },
        &LoadOptions::default(),
        vec![rejected(1, Some("demo:x")), stable(0, "demo:x", "top")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "top");
}

// ---- EngineVersionMerge ----

/// Vanilla's client-entity rule in miniature: above the ceiling is ignored, a higher version
/// replaces, an equal one merges, a lower one is ignored.
fn engine_version_merge(ceiling: u32) -> CombineRule<Toy> {
    CombineRule::EngineVersionMerge {
        decide: Box::new(move |existing, incoming, _diagnostics| {
            if incoming.version > ceiling {
                return EngineVersionDecision::Ignore;
            }
            match existing {
                None => EngineVersionDecision::Replace,
                Some(existing) if incoming.version > existing.version => {
                    EngineVersionDecision::Replace
                }
                Some(existing) if incoming.version == existing.version => {
                    EngineVersionDecision::Merge
                }
                Some(_) => EngineVersionDecision::Ignore,
            }
        }),
        merge: |existing, incoming| {
            existing.from = incoming.from;
            existing.list.extend(incoming.list);
        },
    }
}

fn versioned(pack_index: usize, from: &'static str, version: u32) -> Contribution<Toy> {
    Contribution::Stable(
        id("demo:x"),
        document(
            &pack(pack_index),
            Toy {
                from,
                version,
                list: vec![from],
                ..Toy::default()
            },
        ),
    )
}

#[test]
fn engine_version_merge_merges_equal_versions_and_records_the_contributor() {
    let registry = build(
        engine_version_merge(10),
        &LoadOptions::default(),
        vec![versioned(1, "lower", 5), versioned(0, "top", 5)],
    );
    let effective = registry.get(&id("demo:x")).unwrap();
    assert_eq!(effective.list, ["lower", "top"]);
    assert_eq!(effective.origin().pack, pack(0));
    assert_eq!(effective.origin().contributors, [pack(1)]);
}

#[test]
fn engine_version_merge_replaces_with_a_higher_version_and_ignores_a_lower_one() {
    let registry = build(
        engine_version_merge(10),
        &keep_shadowed(),
        vec![
            versioned(2, "bottom", 5),
            versioned(1, "middle", 7),
            versioned(0, "top", 6),
        ],
    );
    let effective = registry.get(&id("demo:x")).unwrap();
    assert_eq!(effective.from, "middle");
    assert_eq!(effective.list, ["middle"]);
    assert_eq!(layers(&registry, "demo:x"), ["middle", "top", "bottom"]);
}

#[test]
fn engine_version_merge_can_ignore_the_first_definition() {
    let registry = build(
        engine_version_merge(10),
        &LoadOptions::default(),
        vec![versioned(0, "too new", 11)],
    );
    assert_eq!(registry.get(&id("demo:x")), None);
    assert!(registry.is_empty());
}

// ---- FieldMerge ----

fn fields(
    pack_index: usize,
    from: &'static str,
    a: Option<u32>,
    b: Option<u32>,
) -> Contribution<Toy> {
    Contribution::Stable(
        id("demo:x"),
        document(
            &pack(pack_index),
            Toy {
                from,
                a,
                b,
                ..Toy::default()
            },
        ),
    )
}

fn field_merge() -> CombineRule<Toy> {
    CombineRule::FieldMerge {
        merge: |lower, higher| {
            lower.from = higher.from;
            if higher.a.is_some() {
                lower.a = higher.a;
            }
            if higher.b.is_some() {
                lower.b = higher.b;
            }
        },
    }
}

#[test]
fn field_merge_writes_the_higher_fields_over_the_lower_ones_in_either_walk_order() {
    for walk in [
        vec![
            fields(1, "lower", Some(1), Some(2)),
            fields(0, "top", None, Some(3)),
        ],
        vec![
            fields(0, "top", None, Some(3)),
            fields(1, "lower", Some(1), Some(2)),
        ],
    ] {
        let registry = build(field_merge(), &LoadOptions::default(), walk);
        let effective = registry.get(&id("demo:x")).unwrap();
        assert_eq!(
            (effective.from, effective.a, effective.b),
            ("top", Some(1), Some(3))
        );
        assert_eq!(effective.origin().pack, pack(0));
        assert_eq!(effective.origin().contributors, [pack(1)]);
    }
}

// ---- Accumulate ----

#[test]
fn accumulate_keeps_the_top_definition_and_collects_the_side_list() {
    let registry = build(
        CombineRule::Accumulate {
            accumulate: |winner, other| winner.list.extend(other.list.iter().copied()),
        },
        &LoadOptions::default(),
        vec![versioned(1, "lower", 0), versioned(0, "top", 0)],
    );
    let effective = registry.get(&id("demo:x")).unwrap();
    assert_eq!(effective.from, "top");
    assert_eq!(effective.list, ["top", "lower"]);
    assert_eq!(effective.origin().contributors, [pack(1)]);
}

#[test]
fn accumulate_over_three_packs_keeps_every_contributor() {
    let registry = build(
        CombineRule::Accumulate {
            accumulate: |winner, other| winner.list.extend(other.list.iter().copied()),
        },
        &LoadOptions::default(),
        vec![
            versioned(2, "bottom", 0),
            versioned(1, "middle", 0),
            versioned(0, "top", 0),
        ],
    );
    let effective = registry.get(&id("demo:x")).unwrap();
    assert_eq!(effective.from, "top");
    assert_eq!(effective.list, ["top", "middle", "bottom"]);
    assert_eq!(effective.origin().contributors, [pack(1), pack(2)]);
}

// ---- beta ----

#[test]
fn beta_is_none_without_beta_apis_or_beta_documents() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(0, "demo:x", "x")],
    );
    assert!(registry.beta().is_none());
}

#[test]
fn beta_is_some_with_beta_apis_even_when_empty() {
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    let registry = build(
        CombineRule::top_wins(),
        &options,
        vec![stable(0, "demo:x", "x")],
    );
    assert!(registry.beta().is_some_and(Registry::is_empty));
}

#[test]
fn a_base_game_packs_beta_documents_make_beta_some_without_beta_apis() {
    // The loader admits a beta document without Beta APIs only from a base-game pack.
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "x"), beta(1, "demo:y", "beta y")],
    );
    let beta = registry.beta().expect("beta documents were provided");
    assert_eq!(beta.get(&id("demo:y")).unwrap().from, "beta y");
    assert_eq!(
        registry.get(&id("demo:y")),
        None,
        "never in the stable registry"
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn a_higher_beta_definition_is_effective_in_beta_and_listed_in_the_stable_layers() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "stable"), beta(0, "demo:x", "beta")],
    );
    assert_eq!(registry.get(&id("demo:x")), None);
    assert_eq!(layers(&registry, "demo:x"), ["stable"]);
    assert_eq!(
        registry.beta().unwrap().get(&id("demo:x")).unwrap().from,
        "beta"
    );
}

#[test]
fn a_higher_stable_definition_stays_effective_and_the_beta_one_is_listed() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![beta(1, "demo:x", "beta"), stable(0, "demo:x", "stable")],
    );
    assert_eq!(registry.get(&id("demo:x")).unwrap().from, "stable");
    let beta = registry.beta().unwrap();
    assert_eq!(beta.get(&id("demo:x")), None);
    assert_eq!(
        beta.layers(&id("demo:x"))
            .map(|d| d.from)
            .collect::<Vec<_>>(),
        ["beta"]
    );
}

// ---- resolve ----

struct VanillaBlocks;

impl BuiltIns for VanillaBlocks {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "block" && id == "minecraft:air"
    }
}

#[test]
fn resolve_says_defined_built_in_or_missing() {
    let options = LoadOptions {
        built_ins: Some(Arc::new(VanillaBlocks)),
        ..LoadOptions::default()
    };
    let registry = build(
        CombineRule::top_wins(),
        &options,
        vec![stable(0, "demo:x", "x")],
    );
    assert!(matches!(registry.resolve(&id("demo:x")), Resolution::Defined(d) if d.from == "x"));
    assert!(matches!(
        registry.resolve(&id("minecraft:air")),
        Resolution::BuiltIn
    ));
    assert!(matches!(
        registry.resolve(&id("demo:nothing")),
        Resolution::Missing
    ));
    let without = build(CombineRule::top_wins(), &LoadOptions::default(), vec![]);
    assert!(matches!(
        without.resolve(&id("minecraft:air")),
        Resolution::Missing
    ));
}

#[test]
fn the_beta_registry_resolves_built_ins_too() {
    let options = LoadOptions {
        beta_apis: true,
        built_ins: Some(Arc::new(VanillaBlocks)),
        ..LoadOptions::default()
    };
    let registry = build(CombineRule::top_wins(), &options, vec![]);
    let beta = registry.beta().unwrap();
    assert!(matches!(
        beta.resolve(&id("minecraft:air")),
        Resolution::BuiltIn
    ));
}

// ---- diagnostics and priority ----

#[test]
fn diagnostics_are_in_walk_order_documents_rejected_and_combine_time() {
    let mut warned = document(&pack(1), toy("warned"));
    let mut with_warning = Diagnostics::default();
    with_warning.push(bedrock_addon_core::diagnostic::Diagnostic::new(
        DiagnosticCode::ClampedValue { text: None },
        Severity::Warning,
        Phase::Validate,
        Location::new(Some(pack(1)), None, JsonPath::new(), None),
    ));
    warned = Document::new(
        warned.value().clone(),
        warned.origin().clone(),
        version(),
        with_warning,
    );
    let registry = build(
        CombineRule::bottom_wins(),
        &LoadOptions::default(),
        vec![
            Contribution::Stable(id("demo:x"), warned),
            rejected(1, None),
            stable(0, "demo:x", "top"),
        ],
    );
    assert_eq!(
        codes(&registry),
        [
            (
                DiagnosticCode::ClampedValue { text: None },
                Severity::Warning,
                Some(1)
            ),
            (
                DiagnosticCode::MissingRequiredField { text: None },
                Severity::Error,
                None
            ),
            (
                DiagnosticCode::ShadowedDefinition { text: None },
                Severity::Info,
                Some(0)
            ),
        ]
    );
    assert_eq!(registry.rejected().len(), 1);
}

#[test]
fn the_app_data_layer_is_below_every_pack() {
    let app_data = Contribution::Stable(
        id("minecraft:raid_group_1"),
        Document::new(
            toy("app data"),
            origin(
                &PackRef::app_data(),
                "definitions/spawn_groups/raid_group_1.json",
            ),
            version(),
            Diagnostics::default(),
        ),
    );
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(40, "minecraft:raid_group_1", "pack 40"), app_data],
    );
    assert_eq!(
        registry.get(&id("minecraft:raid_group_1")).unwrap().from,
        "pack 40"
    );
}

#[test]
fn debug_lists_the_effective_documents() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(0, "demo:x", "x")],
    );
    let text = format!("{registry:?}");
    assert!(
        text.starts_with("Registry { entries: {Id<block>(\"demo:x\"): Document"),
        "{text}"
    );
}

// ---- what another collection's combine sees ----

#[test]
fn registered_lists_the_effective_definitions_stable_and_beta_with_their_packs() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![
            stable(2, "demo:x", "low"),
            stable(0, "demo:x", "top"),
            beta(1, "demo:y", "beta y"),
            // A higher beta definition replaces the stable one: effective in `beta()` only.
            stable(2, "demo:z", "stable z"),
            beta(1, "demo:z", "beta z"),
            rejected(0, Some("demo:broken")),
        ],
    );
    let registered = registry.registered();
    assert_eq!(registered.kind(), "block");
    assert_eq!(registered.len(), 3);
    assert!(!registered.is_empty());
    let by = |text: &'static str| registered.by(&id(text)).and_then(PackRef::index);
    assert_eq!(by("demo:x"), Some(0), "the winning definition's pack");
    assert_eq!(by("demo:y"), Some(1), "a beta definition is registered");
    assert_eq!(
        by("demo:z"),
        Some(1),
        "the beta definition that replaced a stable one"
    );
    assert_eq!(by("demo:broken"), None, "a rejected file registers nothing");
    assert_eq!(by("demo:absent"), None);
}

#[test]
fn registered_compares_as_the_kind_s_identifiers_do_and_only_within_the_kind() {
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(0, "minecraft:stone", "stone")],
    );
    let registered = registry.registered();
    // A bare block identifier is `minecraft:` (the kind's canonical form).
    assert!(registered.by(&id("stone")).is_some());
    assert!(registered.by(&id("minecraft:Stone")).is_none());
    // The same text as an identifier of another kind is not asked of this index.
    let item: bedrock_addon_core::id::ItemId = Id::from_static("minecraft:stone");
    assert!(registered.by(&item).is_none());
    let empty = build(CombineRule::top_wins(), &LoadOptions::default(), Vec::new());
    assert!(empty.registered().is_empty());
}

// ---- all_layers ----

/// Every identifier `all_layers` lists, with which pack's value each layer is.
fn all_layers(registry: &Registry<Toy>) -> Vec<(&str, Vec<&'static str>)> {
    registry
        .all_layers()
        .map(|(id, layers)| (id.as_str(), layers.map(|document| document.from).collect()))
        .collect()
}

#[test]
fn all_layers_lists_every_identifier_in_load_order_with_its_layers() {
    let registry = build(
        CombineRule::top_wins(),
        &keep_shadowed(),
        vec![
            stable(1, "demo:x", "lower x"),
            stable(1, "demo:y", "y"),
            stable(0, "demo:x", "top x"),
        ],
    );
    assert_eq!(
        all_layers(&registry),
        [("demo:x", vec!["top x", "lower x"]), ("demo:y", vec!["y"])]
    );
    // Without `keep_shadowed` the effective ones alone.
    let registry = build(
        CombineRule::top_wins(),
        &LoadOptions::default(),
        vec![stable(1, "demo:x", "lower x"), stable(0, "demo:x", "top x")],
    );
    assert_eq!(all_layers(&registry), [("demo:x", vec!["top x"])]);
}

#[test]
fn all_layers_lists_an_identifier_a_failed_definition_claimed() {
    let walk = || {
        vec![
            stable(1, "demo:x", "lower"),
            rejected(0, Some("demo:x")),
            stable(1, "demo:y", "lower y"),
        ]
    };
    let rule = || CombineRule::ClaimBeforeValidate { claimed: None };
    let registry = build(rule(), &keep_shadowed(), walk());
    assert_eq!(registry.get(&id("demo:x")), None);
    assert_eq!(
        all_layers(&registry),
        [("demo:x", vec!["lower"]), ("demo:y", vec!["lower y"])],
        "listed with its kept copy, though `iter` has no entry for it"
    );
    // Nothing is kept of it without `keep_shadowed`: not listed.
    let registry = build(rule(), &LoadOptions::default(), walk());
    assert_eq!(all_layers(&registry), [("demo:y", vec!["lower y"])]);
}

#[test]
fn all_layers_lists_an_identifier_whose_first_definition_its_family_ignores() {
    // A definition above the ceiling is ignored, also as the first of its identifier.
    let rule = || engine_version_merge(1);
    let walk = || {
        let ignored = Toy {
            from: "ignored",
            version: 5,
            ..Toy::default()
        };
        vec![
            Contribution::Stable(id("demo:x"), document(&pack(0), ignored)),
            stable(0, "demo:y", "y"),
        ]
    };
    let registry = build(rule(), &keep_shadowed(), walk());
    assert_eq!(registry.get(&id("demo:x")), None);
    assert_eq!(
        all_layers(&registry),
        [("demo:x", vec!["ignored"]), ("demo:y", vec!["y"])]
    );
    let registry = build(rule(), &LoadOptions::default(), walk());
    assert_eq!(all_layers(&registry), [("demo:y", vec!["y"])]);
}

#[test]
fn all_layers_lists_what_beta_and_stable_replaced_of_each_other() {
    // Whatever `keep_shadowed` says, the replaced definition stays in its registry's slot.
    for options in [LoadOptions::default(), keep_shadowed()] {
        // A higher stable definition: the beta one is listed in the beta registry alone.
        let registry = build(
            CombineRule::top_wins(),
            &options,
            vec![beta(1, "demo:x", "beta"), stable(0, "demo:x", "stable")],
        );
        assert_eq!(all_layers(&registry), [("demo:x", vec!["stable"])]);
        let beta_registry = registry.beta().unwrap();
        assert_eq!(beta_registry.iter().count(), 0);
        let listed: Vec<(&str, Vec<&str>)> = beta_registry
            .all_layers()
            .map(|(id, layers)| (id.as_str(), layers.map(|document| document.from).collect()))
            .collect();
        assert_eq!(listed, [("demo:x", vec!["beta"])]);
        // A higher beta definition: the stable one is listed, not effective.
        let registry = build(
            CombineRule::top_wins(),
            &options,
            vec![stable(1, "demo:x", "stable"), beta(0, "demo:x", "beta")],
        );
        assert_eq!(registry.iter().count(), 0);
        assert_eq!(all_layers(&registry), [("demo:x", vec!["stable"])]);
    }
}

/// C-DEBUG: a contribution prints its variant, identifier and origin, so a loader's trace of
/// what it handed the builder reads without a `T: Debug` bound.
#[test]
fn contribution_debug_names_variant_id_and_origin() {
    let stable = format!("{:?}", stable(0, "demo:a", "x"));
    assert!(stable.starts_with("Stable("), "{stable}");
    assert!(stable.contains("demo:a"), "{stable}");
    let beta = format!("{:?}", beta(1, "demo:b", "y"));
    assert!(beta.starts_with("Beta("), "{beta}");
    let rejected = format!("{:?}", rejected(2, None));
    assert!(rejected.starts_with("Rejected(None"), "{rejected}");
    assert!(rejected.contains("broken.json"), "{rejected}");
}
