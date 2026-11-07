//! Unit tests of the item module: the chain and its ranges, the computed item-level gate, and
//! the stack's override gates.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::{LoadContext, LoadOptions, StackCx, StackPack, WalkFile};
use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
};
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{RangeModel, Selection, VersionChain, check_chain, select};
use indexmap::IndexMap;

use super::Item;
use super::LOADER;
use super::history::ItemFamily;
use super::history::ItemHistory;
use super::history::{GateRow, gate_with, release_threshold};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::Value;

fn input(version: FormatVersion) -> FormatVersionInput {
    FormatVersionInput {
        version: Some(version),
        beta_literal: false,
        prerelease: false,
    }
}

fn v(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

#[test]
fn the_chain_is_contiguous_from_0_0_0_to_the_engine() {
    assert_eq!(check_chain::<ItemFamily>(), Ok(()));
    let versions = <ItemHistory as VersionChain<ItemFamily>>::VERSIONS;
    assert_eq!(versions.len(), 21);
    assert_eq!(versions.first(), Some(&v(1, 10, 0)));
    assert_eq!(versions.last(), Some(&v(1, 26, 50)));
    let has_beta = <ItemHistory as VersionChain<ItemFamily>>::HAS_BETA;
    assert!(has_beta, "the chain has a beta struct");
    assert_eq!(
        <ItemFamily as bedrock_addon_core::versioning::Family>::RANGE_MODEL,
        RangeModel::ClosedToEngine
    );
}

#[test]
fn each_version_selects_its_struct() {
    let index = |version| match select::<ItemFamily>(&input(version)) {
        Selection::Struct(i) => Some(<ItemHistory as VersionChain<ItemFamily>>::VERSIONS[i]),
        _ => None,
    };
    // the floor is 0.0.0: everything below 1.16.100 is the legacy struct
    assert_eq!(index(v(0, 0, 0)), Some(v(1, 10, 0)));
    assert_eq!(index(v(1, 16, 0)), Some(v(1, 10, 0)));
    assert_eq!(index(v(1, 16, 100)), Some(v(1, 16, 100)));
    // the gap is the 1.16.100 range; the routing hook sends it to the legacy struct
    assert_eq!(index(v(1, 16, 101)), Some(v(1, 16, 100)));
    assert_eq!(index(v(1, 19, 82)), Some(v(1, 16, 100)));
    assert_eq!(index(v(1, 19, 83)), Some(v(1, 19, 83)));
    assert_eq!(index(v(1, 21, 0)), Some(v(1, 20, 80)));
    assert_eq!(index(v(1, 21, 109)), Some(v(1, 21, 90)));
    assert_eq!(index(v(1, 26, 20)), Some(v(1, 26, 0)));
    assert_eq!(index(v(1, 26, 29)), Some(v(1, 26, 0)));
    assert_eq!(index(v(1, 26, 30)), Some(v(1, 26, 30)));
    assert_eq!(index(v(1, 26, 49)), Some(v(1, 26, 30)));
    assert_eq!(index(v(1, 26, 50)), Some(v(1, 26, 50)));
    // the ceiling is the engine version
    assert_eq!(index(ENGINE_VERSION), Some(v(1, 26, 50)));
    // 1.26.50 is the newest struct: every version up to the engine selects it
    assert_eq!(index(v(1, 26, 51)), Some(v(1, 26, 50)));
    let above = FormatVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1,
    );
    assert_eq!(select::<ItemFamily>(&input(above)), Selection::Unsupported);
}

/// Loads `json` with `cx`.
fn load(
    json: &str,
    cx: &LoadContext,
) -> Result<bedrock_addon_core::document::Document<Item>, bedrock_addon_core::document::Rejected> {
    Item::parse(&PackPath::from_static("items/x.json"), json.as_bytes(), cx)
}

/// A 1.26.55 item loads through the newest struct, as does one at the engine's own version.
#[test]
fn an_item_between_1_26_51_and_the_engine_version_loads() {
    for (version, patch) in [("1.26.55", 55), ("1.26.60", 60)] {
        let text = format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 2}}}}}}"#
        );
        let document = load(&text, &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{version}: {:?}", rejected.diagnostics));
        assert_eq!(document.version().written, Some(v(1, 26, patch)));
    }
}

#[test]
fn a_version_above_the_engine_is_unsupported_and_beta_is_the_literal() {
    let above = format!(
        r#"{{"format_version": "{}.{}.{}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:max_stack_size": 2}}}}}}"#,
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch + 1
    );
    let rejected = load(&above, &LoadContext::detached()).unwrap_err();
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|f| f.code.name() == "UnsupportedVersion")
    );
    // a pre-release selects the struct below its release and needs no toggle
    let pre = r#"{"format_version": "1.21.60-beta.3", "minecraft:item": {"description": {"identifier": "demo:x", "menu_category": {"group": "bundles"}}, "components": {"minecraft:max_stack_size": 2}}}"#;
    let document = load(pre, &LoadContext::detached()).expect("1.21.50 has no namespace rule");
    assert_eq!(document.version().parsed_as, v(1, 21, 50));
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|f| f.code.name() == "PrereleaseFormatVersion")
    );
    assert_eq!(
        super::history::unpack(super::history::pack(1, 21, 110)),
        v(1, 21, 110)
    );
}

/// A lowered table: `max_stack_size` released at 1.20.10.
fn lowered(name: &str) -> Option<GateRow> {
    (name == "minecraft:max_stack_size").then_some(GateRow {
        name: "minecraft:max_stack_size",
        released: Some(FormatVersion::new(1, 20, 10)),
        requires: &[],
        deprecated_since: None,
        deprecated_member: false,
    })
}

#[test]
fn v12_the_item_level_threshold_is_computed_from_the_table() {
    let rows = [Some(v(1, 20, 10)), Some(v(1, 21, 0)), None];
    assert_eq!(release_threshold(rows.into_iter()), Some(v(1, 20, 10)));
    let mut components = IndexMap::new();
    components.insert("minecraft:max_stack_size".to_owned(), Value::from(true));
    let cx = LoadContext::detached();
    for (version, passes) in [(v(1, 20, 0), false), (v(1, 20, 10), true)] {
        let mut d = Diagnostics::default();
        let threshold = release_threshold([Some(v(1, 20, 10))].into_iter());
        let out = gate_with(
            &components,
            "demo:x",
            version,
            &cx,
            &JsonPath::new(),
            &mut d,
            threshold,
            lowered,
        );
        assert_eq!(out.is_some(), passes, "{version}");
    }
}

/// A pack of the stack.
fn pack(index: usize, name: &str) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            uuid::Uuid::from_u128(index as u128 + 7),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        name,
    )
}

/// One parsed file of `pack`, read in its pack's context: a base-game pack's when `base_game`.
fn file(
    pack: &PackRef,
    path: &str,
    json: &str,
    base_game: bool,
) -> bedrock_addon_core::document::Contribution<Item> {
    (LOADER.parse)(&walked(pack, path, json), &context(base_game))
}

/// The context a file of a pack is read with: a base-game pack's when `base_game`.
fn context(base_game: bool) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = base_game;
    cx
}

/// The walked file `path` of `pack`.
fn walked(pack: &PackRef, path: &str, json: &str) -> WalkFile {
    let path = PackPath::new(path).unwrap();
    WalkFile {
        pack: pack.clone(),
        side: PackSide::Behavior,
        path: path.clone(),
        key: path,
        bytes: bytes::Bytes::copy_from_slice(json.as_bytes()),
    }
}

/// The stack of `packs` (each with whether it is a base-game pack), each at the position its
/// index names (0 the top): what the combine reads a pack's base-game flag from. The indices
/// must be `0..packs.len()`, each once.
fn stack(packs: &[(&PackRef, bool)]) -> Vec<StackPack> {
    let mut slots: Vec<Option<StackPack>> = vec![None; packs.len()];
    for (pack, base_game) in packs {
        let index = pack.index().expect("a pack of the stack has an index");
        assert!(
            slots.get(index).is_some_and(Option::is_none),
            "pack index {index} is out of 0..{} or given twice",
            packs.len()
        );
        slots[index] = Some(StackPack::new(
            (*pack).clone(),
            PackOrigin::World,
            context(*base_game),
        ));
    }
    slots.into_iter().flatten().collect()
}

fn doc(version: &str, id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

#[test]
fn v15_the_vanilla_override_gates() {
    let vanilla = pack(1, "vanilla");
    let addon = pack(0, "addon");
    let parsed = vec![
        file(
            &vanilla,
            "items/camera.json",
            &doc(
                "1.10",
                "minecraft:camera",
                r#""minecraft:use_duration": 100000"#,
            ),
            true,
        ),
        file(
            &vanilla,
            "items/apple.json",
            &doc(
                "1.26.30",
                "minecraft:apple",
                r#""minecraft:max_stack_size": 64"#,
            ),
            true,
        ),
        file(
            &addon,
            "items/apple.json",
            &doc(
                "1.10",
                "minecraft:apple",
                r#""minecraft:max_stack_size": 16"#,
            ),
            false,
        ),
        file(
            &addon,
            "items/camera.json",
            &doc(
                "1.21.0",
                "minecraft:camera",
                r#""minecraft:max_stack_size": 1"#,
            ),
            false,
        ),
        file(
            &addon,
            "items/odd.json",
            &doc(
                "1.21.0",
                "minecraft:not_an_item",
                r#""minecraft:max_stack_size": 1"#,
            ),
            false,
        ),
    ];
    let options = LoadOptions::default();
    let packs = stack(&[(&vanilla, true), (&addon, false)]);
    let registry = (LOADER.combine)(parsed, &StackCx::new(&options, &packs));
    let texts: Vec<String> = registry
        .rejected()
        .iter()
        .flat_map(|r| r.diagnostics.iter())
        .map(|f| f.code.to_string())
        .collect();
    assert!(texts.iter().any(|t| t == "Item 'minecraft:apple' (format_version: 1.10.0) cannot be registered because the Vanilla item only exists as a 1.16.100+ format version!"), "{texts:#?}");
    assert!(texts.iter().any(|t| t == "Item 'minecraft:camera' (format_version: 1.21.0) cannot be registered because no 1.16.100+ Vanilla format version exists!"), "{texts:#?}");
    // A component-format `minecraft:` item without a vanilla entry gets the "no 1.16.100+
    // Vanilla format version exists" text; the game writes no other text for it.
    assert!(texts.iter().any(|t| t == "Item 'minecraft:not_an_item' (format_version: 1.21.0) cannot be registered because no 1.16.100+ Vanilla format version exists!"), "{texts:#?}");
    // the vanilla definitions stay registered; none of the overrides is
    let apple = registry
        .get(&bedrock_addon_core::id::ItemId::parse("minecraft:apple"))
        .unwrap();
    assert_eq!(apple.origin().pack, vanilla);
    assert!(
        registry
            .get(&bedrock_addon_core::id::ItemId::parse(
                "minecraft:not_an_item"
            ))
            .is_none()
    );
}

#[test]
fn v15_the_override_gates_compare_the_version_not_the_format() {
    // a 1.21.0 document whose components name no registered component is the legacy format,
    // but at 1.16.100 or later: refused against a legacy vanilla item, and refused without one
    let vanilla = pack(1, "vanilla");
    let addon = pack(0, "addon");
    let parsed = vec![
        file(
            &vanilla,
            "items/camera.json",
            &doc("1.10", "minecraft:camera", ""),
            true,
        ),
        file(
            &addon,
            "items/camera.json",
            &doc("1.21.0", "minecraft:camera", ""),
            false,
        ),
        file(
            &addon,
            "items/odd.json",
            &doc("1.21.0", "minecraft:odd", ""),
            false,
        ),
        // a legacy-format override of a legacy vanilla item is allowed
        file(
            &addon,
            "items/camera_old.json",
            &doc("1.12.0", "minecraft:camera", ""),
            false,
        ),
    ];
    let options = LoadOptions::default();
    let packs = stack(&[(&vanilla, true), (&addon, false)]);
    let registry = (LOADER.combine)(parsed, &StackCx::new(&options, &packs));
    let texts: Vec<String> = registry
        .rejected()
        .iter()
        .flat_map(|r| r.diagnostics.iter())
        .filter(|f| f.severity == bedrock_addon_core::diagnostic::Severity::Error)
        .map(|f| f.code.to_string())
        .collect();
    assert_eq!(
        texts,
        [
            "Item 'minecraft:camera' (format_version: 1.21.0) cannot be registered because no 1.16.100+ Vanilla format version exists!",
            "Item 'minecraft:odd' (format_version: 1.21.0) cannot be registered because no 1.16.100+ Vanilla format version exists!",
        ]
    );
    let camera = registry
        .get(&bedrock_addon_core::id::ItemId::parse("minecraft:camera"))
        .unwrap();
    assert_eq!(camera.origin().pack, addon);
}

/// The game skips an item whose name a pack higher in the stack already defined, with a
/// warning, unless its pack is a base-game pack (inferred from the pack's base-game flag);
/// the higher definition is the one registered.
#[test]
fn v1_an_item_a_higher_pack_overrides_is_told() {
    let low = pack(2, "low");
    let base = pack(1, "base");
    let high = pack(0, "high");
    let text = doc("1.26.50", "demo:x", r#""minecraft:max_stack_size": 2"#);
    let parsed = vec![
        file(&low, "items/x.json", &text, false),
        file(&base, "items/x.json", &text, true),
        file(&high, "items/x.json", &text, false),
    ];
    let options = LoadOptions::default();
    let packs = stack(&[(&low, false), (&base, true), (&high, false)]);
    let registry = (LOADER.combine)(parsed, &StackCx::new(&options, &packs));
    let told: Vec<(
        String,
        Option<String>,
        bedrock_addon_core::diagnostic::Severity,
        String,
    )> = registry
        .diagnostics()
        .filter(|f| f.code.to_string().contains("overridden"))
        .map(|f| {
            (
                f.code.to_string(),
                f.location.pack.as_ref().map(|p| p.name().to_owned()),
                f.severity,
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(
        told,
        [(
            "Item 'demo:x' has already been overridden by a pack higher in the pack stack!"
                .to_owned(),
            Some("low".to_owned()),
            bedrock_addon_core::diagnostic::Severity::Warning,
            "minecraft:item/description/identifier".to_owned()
        )]
    );
    let x = registry
        .get(&bedrock_addon_core::id::ItemId::parse("demo:x"))
        .unwrap();
    assert_eq!(x.origin().pack, high);
}

/// The `tag:` deprecation carries the game's full text.
#[test]
fn v12_the_tag_deprecation_has_the_vanilla_text() {
    assert_eq!(
        super::codes::TAG_DEPRECATED.template(),
        "tag: has been deprecated in format versions 1.20.50 and above. Use the minecraft:tags component to store all tags instead."
    );
}
