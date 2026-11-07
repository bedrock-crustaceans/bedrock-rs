//! `Document<T>` and its provenance, `Rejected` and the `Placed::default_path`
//! helper.

use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};
use bedrock_addon_core::document::{
    Content, Document, NoBeta, Origin, Rejected, VersionInfo, default_path,
};
use bedrock_addon_core::id::{BiomeId, BlockId, EntityId, GeometryId, Id, ItemId, kind};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use uuid::Uuid;

fn ruby_pack() -> PackRef {
    PackRef::new(
        0,
        PackIdVersion::new(
            Uuid::from_u128(7),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        "Ruby Pack",
    )
}

fn ruby_origin() -> Origin {
    Origin::new(
        ruby_pack(),
        PackSide::Behavior,
        PackPath::from_static("blocks/ruby.json"),
        None,
        Vec::new(),
    )
}

fn version_1_21_60() -> VersionInfo {
    VersionInfo::new(
        Some(FormatVersion::new(1, 21, 60)),
        false,
        FormatVersion::new(1, 21, 60),
        FormatVersion::new(1, 21, 60),
        true,
    )
}

fn a_warning() -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        Code::from(&codes::SHADOWED_DEFINITION),
        Severity::Warning,
        Phase::Validate,
        Location::new(None, None, JsonPath::new(), None),
    ));
    diagnostics
}

/// A root type of the test: a block with a public field.
#[derive(Clone, Debug, PartialEq)]
struct Ruby {
    identifier: &'static str,
    light: u8,
}

fn ruby_document() -> Document<Ruby> {
    Document::new(
        Ruby {
            identifier: "demo:ruby",
            light: 15,
        },
        ruby_origin(),
        version_1_21_60(),
        a_warning(),
    )
}

#[test]
fn a_document_reads_through_to_its_value() {
    let document = ruby_document();
    assert_eq!(document.identifier, "demo:ruby");
    assert_eq!(document.light, 15);
    assert_eq!(document.value().light, 15);
}

#[test]
fn a_document_keeps_its_provenance() {
    let document = ruby_document();
    assert_eq!(document.origin(), &ruby_origin());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 60))
    );
    assert!(document.version().upgraded);
    assert_eq!(document.diagnostics(), &a_warning());
}

#[test]
fn map_keeps_the_provenance() {
    let mapped = ruby_document().map(|ruby| u32::from(ruby.light) * 2);
    assert_eq!(*mapped, 30);
    assert_eq!(mapped.origin(), &ruby_origin());
    assert_eq!(mapped.diagnostics().len(), 1);
    assert_eq!(mapped.into_value(), 30);
}

#[test]
fn version_info_from_its_parts() {
    let info = VersionInfo::new(
        None,
        true,
        FormatVersion::new(1, 26, 0),
        FormatVersion::new(1, 26, 10),
        false,
    );
    assert_eq!(info.written, None);
    assert!(info.beta);
    assert_eq!(info.effective, FormatVersion::new(1, 26, 0));
    assert_eq!(info.parsed_as, FormatVersion::new(1, 26, 10));
    assert!(!info.upgraded);
}

#[test]
fn origin_displays_pack_name_and_path() {
    assert_eq!(ruby_origin().to_string(), "Ruby Pack/blocks/ruby.json");
}

#[test]
fn origin_displays_the_member_key_after_a_hash() {
    let origin = Origin::new(
        ruby_pack(),
        PackSide::Resource,
        PackPath::from_static("animations/zombie.json"),
        Some("animation.zombie.walk".into()),
        vec![ruby_pack()],
    );
    assert_eq!(origin.side, PackSide::Resource);
    assert_eq!(origin.contributors, [ruby_pack()]);
    assert_eq!(
        origin.to_string(),
        "Ruby Pack/animations/zombie.json#animation.zombie.walk"
    );
}

#[test]
fn rejected_displays_its_origin_and_summary() {
    let rejected = Rejected::new(ruby_origin(), a_warning());
    assert_eq!(rejected.origin, ruby_origin());
    assert_eq!(rejected.diagnostics.len(), 1);
    let text = rejected.to_string();
    assert!(text.starts_with("Ruby Pack/blocks/ruby.json: "), "{text}");
    assert!(text.ends_with(&a_warning().summary().to_string()), "{text}");
}

#[test]
fn default_path_of_the_17_example() {
    let ruby: BlockId = Id::from_static("demo:ruby_block");
    assert_eq!(
        default_path("blocks", &ruby).as_str(),
        "blocks/demo/ruby_block.json"
    );
}

#[test]
fn default_path_of_a_bare_vanilla_identifier_uses_minecraft() {
    let stone: BlockId = Id::from_static("stone");
    assert_eq!(
        default_path("blocks", &stone).as_str(),
        "blocks/minecraft/stone.json"
    );
}

#[test]
fn default_path_of_an_identifier_without_namespace_is_its_text() {
    let geometry: GeometryId = Id::from_static("geometry.ruby_lamp");
    assert_eq!(
        default_path("models/entity", &geometry).as_str(),
        "models/entity/geometry.ruby_lamp.json"
    );
}

#[test]
fn default_path_ignores_suffixes_the_identifier_compares_without() {
    let pig: EntityId = Id::parse("minecraft:pig<minecraft:entity_born>");
    assert_eq!(
        default_path("entities", &pig).as_str(),
        "entities/minecraft/pig.json"
    );
    let wool: ItemId = Id::parse("minecraft:wool:3");
    assert_eq!(
        default_path("items", &wool).as_str(),
        "items/minecraft/wool.json"
    );
}

#[test]
fn default_path_keeps_a_malformed_identifier_inside_the_directory() {
    // The loader keeps malformed identifiers as written; the writer still needs a valid path.
    let odd: BiomeId = Id::parse("demo:../a:b\u{1}");
    let path = default_path("biomes", &odd);
    assert_eq!(path.as_str(), "biomes/demo/_/a_b_.json");
    assert!(PackPath::new(path.as_str()).is_ok());
}

#[test]
fn no_beta_rejects_every_file() {
    let path = PackPath::from_static("blocks/ruby.json");
    let rejected = NoBeta::<kind::Block>::parse(&path, b"{}", &LoadContext::detached())
        .expect_err("a family without a beta struct parses no beta file");
    assert_eq!(rejected.origin.path, path);
    assert!(
        rejected
            .diagnostics
            .first_error()
            .is_some_and(|d| d.is(&codes::UNSUPPORTED_VERSION))
    );
    assert!(NoBeta::<kind::Block>::SIDES.is_empty());
}
