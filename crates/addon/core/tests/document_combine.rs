//! The collection-level combine entry points (`RegistryBuilder::combine`,
//! `FilesBuilder::combine`), each the builder's push loop in one call, and the within-pack tie
//! of a `CombineRule` (`CombineRule::within_pack`).

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, Files, FilesBuilder, FilesRule,
    NoBeta, Notice, Origin, PackTie, Registry, RegistryBuilder, Rejected, TieWinner, VersionInfo,
};
use bedrock_addon_core::id::{BlockId, Id, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use uuid::Uuid;

/// A toy definition: the name of the file it came from.
#[derive(Clone, Debug, PartialEq, Default)]
struct Toy(&'static str);

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
    type Beta = NoBeta<kind::Block>;
}

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

fn origin(pack_index: usize, path: &'static str) -> Origin {
    Origin::new(
        pack(pack_index),
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

/// The document of `from` (`blocks/<from>.json` in the pack at `pack_index`), with one
/// warning of its own.
fn document(pack_index: usize, from: &'static str) -> Document<Toy> {
    let path = Box::leak(format!("blocks/{from}.json").into_boxed_str());
    let mut at = JsonPath::new();
    at.push(from);
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        Code::from(&codes::UNRECOGNIZED_FIELD),
        Severity::Warning,
        Phase::Parse,
        Location::new(Some(pack(pack_index)), None, at, None),
    ));
    Document::new(Toy(from), origin(pack_index, path), version(), diagnostics)
}

fn id(text: &'static str) -> BlockId {
    Id::from_static(text)
}

fn stable(pack_index: usize, text: &'static str, from: &'static str) -> Contribution<Toy> {
    Contribution::Stable(id(text), document(pack_index, from))
}

fn rejected(pack_index: usize, text: &'static str) -> Contribution<Toy> {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        Code::from(&codes::MISSING_REQUIRED_FIELD),
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, JsonPath::new(), None),
    ));
    Contribution::Rejected(
        Some(id(text)),
        Rejected::new(origin(pack_index, "blocks/broken.json"), diagnostics),
    )
}

fn keep_shadowed() -> LoadOptions {
    LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    }
}

/// What a registry holds, for comparing two: each id with the files of its layers, the
/// rejected files and every finding with the file it is at.
type Seen = (
    Vec<(String, Vec<&'static str>)>,
    usize,
    Vec<(Code, Severity, Option<String>)>,
);

fn seen(registry: &Registry<Toy>) -> Seen {
    let layers = registry
        .all_layers()
        .map(|(id, layers)| (id.to_string(), layers.map(|d| d.value().0).collect()))
        .collect();
    let findings = registry
        .diagnostics()
        .map(|d| {
            (
                d.code.clone(),
                d.severity,
                d.location.path.as_ref().map(ToString::to_string),
            )
        })
        .collect();
    (layers, registry.rejected().len(), findings)
}

/// The walk the comparisons use: one id over three packs (twice in one of them), a rejected
/// claim and a second id.
fn walk() -> Vec<Contribution<Toy>> {
    vec![
        stable(2, "demo:a", "bottom"),
        stable(1, "demo:a", "middle_first"),
        stable(1, "demo:a", "middle_second"),
        rejected(0, "demo:a"),
        stable(0, "demo:b", "top_b"),
    ]
}

fn pushed(rule: CombineRule<Toy>, options: &LoadOptions) -> Registry<Toy> {
    let mut builder = RegistryBuilder::new(rule, CombineRule::top_wins(), options);
    for contribution in walk() {
        builder.push(contribution);
    }
    builder.finish()
}

#[test]
fn registry_combine_is_the_push_loop_in_one_call() {
    let rules: [fn() -> CombineRule<Toy>; 4] = [
        CombineRule::top_wins,
        CombineRule::bottom_wins,
        || CombineRule::TopWinsFallback {
            fallback: Some(Notice::shadowed()),
        },
        || CombineRule::top_wins().within_pack(PackTie::first(Some(Notice::shadowed()))),
    ];
    for options in [LoadOptions::default(), keep_shadowed()] {
        for rule in rules {
            let combined =
                RegistryBuilder::combine(walk(), rule(), CombineRule::top_wins(), &options);
            assert_eq!(
                seen(&combined),
                seen(&pushed(rule(), &options)),
                "{:?}",
                rule()
            );
        }
    }
}

#[test]
fn files_combine_is_the_push_loop_in_one_call() {
    let copies = || {
        vec![
            (
                PackPath::from_static("loot/a.json"),
                Ok(document(1, "lower")),
            ),
            (
                PackPath::from_static("loot/a.json"),
                Ok(document(0, "upper")),
            ),
            (
                PackPath::from_static("loot/b.json"),
                Err(Rejected::new(
                    origin(0, "loot/b.json"),
                    Diagnostics::default(),
                )),
            ),
            (PackPath::from_static("loot/b.json"), Ok(document(1, "b"))),
        ]
    };
    let files_seen = |files: &Files<Toy>| {
        let layers: Vec<(String, Vec<&'static str>)> = files
            .all_layers()
            .map(|(path, layers)| (path.to_string(), layers.map(|d| d.value().0).collect()))
            .collect();
        let findings: Vec<Code> = files.diagnostics().map(|d| d.code.clone()).collect();
        (layers, files.len(), files.rejected().len(), findings)
    };
    for rule in [
        FilesRule::TopWins,
        FilesRule::TopWinsFallback {
            fallback: Some(Notice::shadowed()),
        },
    ] {
        let mut builder = FilesBuilder::new(rule.clone(), &keep_shadowed());
        for (path, copy) in copies() {
            builder.push(path, copy);
        }
        let pushed = builder.finish();
        let combined = FilesBuilder::combine(copies(), rule.clone(), &keep_shadowed());
        assert_eq!(files_seen(&combined), files_seen(&pushed), "{rule:?}");
    }
}

/// The files of the layers of `text`.
fn layers(registry: &Registry<Toy>, text: &'static str) -> Vec<&'static str> {
    registry.layers(&id(text)).map(|d| d.value().0).collect()
}

/// The combine-time findings (phase `Validate`) with the file each is at.
fn notices(registry: &Registry<Toy>) -> Vec<(Code, String)> {
    registry
        .diagnostics()
        .filter(|d| d.phase == Phase::Validate)
        .map(|d| {
            let path = d.location.path.as_ref().map(ToString::to_string);
            (d.code.clone(), path.unwrap_or_default())
        })
        .collect()
}

fn shadowed() -> Code {
    Code::from(&codes::SHADOWED_DEFINITION)
}

fn duplicate() -> Code {
    Code::from(&codes::DUPLICATE_IDENTIFIER)
}

fn tied(tie: PackTie, walk: Vec<Contribution<Toy>>) -> Registry<Toy> {
    let rule = CombineRule::TopWins {
        shadowed: Some(Notice::shadowed()),
    }
    .within_pack(tie);
    RegistryBuilder::combine(walk, rule, CombineRule::top_wins(), &keep_shadowed())
}

#[test]
fn a_first_tie_keeps_the_first_definition_of_a_pack() {
    let registry = tied(
        PackTie::first(Some(Notice::shadowed())),
        vec![
            stable(1, "demo:a", "first"),
            stable(1, "demo:a", "second"),
            stable(1, "demo:a", "third"),
        ],
    );
    assert_eq!(layers(&registry, "demo:a"), ["first", "second", "third"]);
    // Told at each later file of the pack: the losers.
    assert_eq!(
        notices(&registry),
        [
            (shadowed(), "blocks/second.json".to_owned()),
            (shadowed(), "blocks/third.json".to_owned()),
        ]
    );
}

#[test]
fn a_tie_decides_within_a_pack_only() {
    // Across packs the rule decides: the top pack's first file wins over the lower pack's, and
    // a later file of the top pack is the tie's.
    let registry = tied(
        PackTie::first(None),
        vec![
            stable(1, "demo:a", "lower"),
            stable(0, "demo:a", "top_first"),
            stable(0, "demo:a", "top_second"),
        ],
    );
    assert_eq!(
        layers(&registry, "demo:a"),
        ["top_first", "top_second", "lower"]
    );
    assert_eq!(
        notices(&registry),
        [(shadowed(), "blocks/lower.json".to_owned())],
        "the rule's notice across packs, none for the tie"
    );
}

#[test]
fn a_last_tie_tells_its_own_notice_at_the_earlier_file() {
    let registry = tied(
        PackTie::last(Some(Notice::shadowed())),
        vec![stable(1, "demo:a", "first"), stable(1, "demo:a", "second")],
    );
    assert_eq!(layers(&registry, "demo:a"), ["second", "first"]);
    assert_eq!(
        notices(&registry),
        [(shadowed(), "blocks/first.json".to_owned())],
        "not the DuplicateIdentifier the plain rule tells"
    );
}

#[test]
fn without_a_tie_the_rules_keep_their_own_within_pack_rule() {
    let walk = || vec![stable(1, "demo:a", "first"), stable(1, "demo:a", "second")];
    let top = RegistryBuilder::combine(
        walk(),
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &keep_shadowed(),
    );
    assert_eq!(layers(&top, "demo:a"), ["second", "first"]);
    assert_eq!(
        notices(&top),
        [(duplicate(), "blocks/first.json".to_owned())]
    );
    let bottom = RegistryBuilder::combine(
        walk(),
        CombineRule::bottom_wins(),
        CombineRule::top_wins(),
        &keep_shadowed(),
    );
    assert_eq!(layers(&bottom, "demo:a"), ["first", "second"]);
    assert_eq!(
        notices(&bottom),
        [(duplicate(), "blocks/second.json".to_owned())]
    );
    // The tie's constructors.
    assert_eq!(PackTie::first(None).winner, TieWinner::First);
    assert_eq!(
        PackTie::last(Some(Notice::duplicate())),
        PackTie {
            winner: TieWinner::Last,
            notice: Some(Notice::duplicate()),
        }
    );
}

#[test]
fn a_tied_claim_still_holds_its_identifier() {
    // `ClaimBeforeValidate` with a tie: the failed top claim still leaves the lower definition
    // shadowed.
    let rule = CombineRule::ClaimBeforeValidate {
        claimed: Some(Notice::shadowed()),
    }
    .within_pack(PackTie::first(None));
    let registry = RegistryBuilder::combine(
        vec![stable(1, "demo:a", "lower"), rejected(0, "demo:a")],
        rule,
        CombineRule::top_wins(),
        &keep_shadowed(),
    );
    assert!(registry.get(&id("demo:a")).is_none());
    assert_eq!(layers(&registry, "demo:a"), ["lower"]);
    assert_eq!(
        notices(&registry),
        [(shadowed(), "blocks/broken.json".to_owned())]
    );
}
