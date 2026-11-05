//! Writing: `Manifest::new` / `with_ids`, write in the read format, parse → write → parse equal,
//! and what a legacy format cannot hold. Also the synthetic casing and legacy-name manifests.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Content;
use bedrock_addon::json::{EncodeError, WriteStyle};
use bedrock_addon::load::LoadContext;
use bedrock_addon::manifest::{
    Dependency, Manifest, ManifestFormat, MemoryTier, ModuleType, NativeModuleVersion, Setting,
};
use bedrock_addon::pack::{PackScope, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{MinEngineVersion, SemVersion};
use uuid::Uuid;

use super::{fixture, format_2, format_3, loads, parse, parse_str};

const HEADER: Uuid = Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111);
const MODULE: Uuid = Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222);

/// Parse, write, parse again: the values must be equal.
fn round_trip(text: &str) -> Manifest {
    let first = loads(text).into_value();
    let written = first.to_json().unwrap();
    let second = parse_str(&written)
        .unwrap_or_else(|rejected| panic!("{written}\n{:#?}", rejected.diagnostics))
        .into_value();
    assert_eq!(first, second, "written:\n{written}");
    second
}

#[test]
fn with_ids_writes_a_deterministic_format_2_manifest() {
    let manifest = Manifest::with_ids("Ruby lamps", PackType::Behavior, HEADER, MODULE);
    let expected = r#"{
  "format_version": 2,
  "header": {
    "name": "Ruby lamps",
    "uuid": "11111111-1111-1111-1111-111111111111",
    "version": [
      1,
      0,
      0
    ],
    "min_engine_version": [
      1,
      26,
      60
    ]
  },
  "modules": [
    {
      "type": "data",
      "uuid": "22222222-2222-2222-2222-222222222222",
      "version": [
        1,
        0,
        0
      ]
    }
  ]
}
"#;
    assert_eq!(manifest.to_json().unwrap(), expected);
    assert_eq!(
        Manifest::with_ids("Ruby lamps", PackType::Behavior, HEADER, MODULE),
        manifest
    );
    // It reads back to itself.
    assert_eq!(parse_str(expected).unwrap().into_value(), manifest);
}

#[test]
fn new_gives_fresh_uuids_and_the_engine_version() {
    let first = Manifest::new("Ruby", PackType::Resource);
    let second = Manifest::new("Ruby", PackType::Resource);
    assert_ne!(first.header.uuid, second.header.uuid);
    assert_ne!(first.header.uuid, first.modules[0].uuid);
    assert_eq!(first.format, ManifestFormat::V2);
    assert_eq!(first.modules[0].module_type, ModuleType::Resources);
    assert_eq!(first.header.version, SemVersion::new(1, 0, 0));
    assert_eq!(
        first.header.min_engine_version,
        Some(MinEngineVersion::new(1, 26, 60))
    );
    assert!(parse_str(&first.to_json().unwrap()).is_ok());
}

#[test]
fn every_vanilla_manifest_round_trips_in_its_format() {
    let mut count = 0;
    for format in [
        "vanilla/behavior_packs",
        "vanilla/resource_packs",
        "vanilla/skin_packs",
        "vanilla/test",
    ] {
        let mut stack = vec![super::fixtures().join(format)];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name().is_some_and(|n| n == "manifest.json") {
                    let text = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
                    let manifest = round_trip(&text);
                    let read = loads(&text);
                    assert_eq!(manifest.format, read.format, "{}", path.display());
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 221);
}

#[test]
fn a_format_1_manifest_is_written_as_format_1() {
    let text = format_2("", "").replacen("\"format_version\": 2", "\"format_version\": 1", 1);
    let written = round_trip(&text).to_json().unwrap();
    assert!(
        written.starts_with("{\n  \"format_version\": 1,"),
        "{written}"
    );
}

/// A format-1 behaviour pack manifest whose header `min_engine_version` member is `member`
/// (`""` for none).
fn format_1_with_min_engine(member: &str) -> String {
    format_2("", "")
        .replacen("\"format_version\": 2", "\"format_version\": 1", 1)
        .replacen(",\n    \"min_engine_version\": [1, 26, 20]", member, 1)
}

/// The pretty-printed `min_engine_version` member of a written legacy header.
fn written_min_engine(major: u16, minor: u16, patch: u16) -> String {
    format!("\"min_engine_version\": [\n      {major},\n      {minor},\n      {patch}\n    ]")
}

// The writer writes back what was read; loading uses the capped value.
#[test]
fn a_format_1_min_engine_version_above_the_cap_is_written_as_read() {
    let text = format_1_with_min_engine(",\n    \"min_engine_version\": [1, 16, 0]");
    let manifest = round_trip(&text);
    assert_eq!(
        manifest.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_CAP),
        "loading uses the capped value"
    );
    let written = manifest.to_json().unwrap();
    assert!(
        written.contains(&written_min_engine(1, 16, 0)),
        "the value read is written back:\n{written}"
    );
    let again = round_trip(&written);
    assert_eq!(again, manifest);
    assert_eq!(again.to_json().unwrap(), written);
}

#[test]
fn a_format_1_min_engine_version_changed_after_reading_is_written_as_changed() {
    let text = format_1_with_min_engine(",\n    \"min_engine_version\": [1, 16, 0]");
    let mut manifest = loads(&text).into_value();
    manifest.header.min_engine_version = Some(MinEngineVersion::new(1, 10, 0));
    let written = manifest.to_json().unwrap();
    assert!(written.contains(&written_min_engine(1, 10, 0)), "{written}");
    assert_eq!(
        written.matches("min_engine_version").count(),
        1,
        "{written}"
    );
    manifest.header.min_engine_version = None;
    let written = manifest.to_json().unwrap();
    assert!(!written.contains("min_engine_version"), "{written}");
}

#[test]
fn a_format_1_pack_without_min_engine_version_is_written_without_one() {
    let manifest = round_trip(&format_1_with_min_engine(""));
    assert_eq!(
        manifest.header.min_engine_version,
        Some(MinEngineVersion::FORMAT_1_DEFAULT),
        "loading uses the default"
    );
    let written = manifest.to_json().unwrap();
    assert!(!written.contains("min_engine_version"), "{written}");
}

#[test]
fn a_format_1_min_engine_version_of_1_2_5_is_written_as_read() {
    let manifest = round_trip(&format_1_with_min_engine(
        ",\n    \"min_engine_version\": [1, 2, 5]",
    ));
    let written = manifest.to_json().unwrap();
    assert!(written.contains(&written_min_engine(1, 2, 5)), "{written}");
}

#[test]
fn a_format_3_manifest_round_trips_with_every_member() {
    let text = r#"{
  "format_version": 3,
  "header": {
    "name": "Ruby",
    "description": "rich",
    "uuid": "11111111-1111-1111-1111-111111111111",
    "version": "1.2.3-beta.1",
    "min_engine_version": "1.26.50",
    "pack_scope": "world",
    "platform_locked": false,
    "pack_optimization_version": "0.1.0"
  },
  "modules": [
    { "type": "data", "uuid": "22222222-2222-2222-2222-222222222222", "version": "1.0.0" },
    { "type": "script", "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0.0",
      "language": "javascript", "entry": "scripts/main.js", "description": "s" }
  ],
  "dependencies": [
    { "uuid": "44444444-4444-4444-4444-444444444444", "version": "1.0.0" },
    { "module_name": "@minecraft/server", "version": "2.1.0", "optional": false },
    { "module_name": "@minecraft/server-ui", "version": "internal" }
  ],
  "capabilities": ["script_eval"],
  "settings": [
    { "type": "label", "text": "L" },
    { "type": "slider", "name": "a:s", "text": "S", "min": 0.1, "max": 2.5, "step": 0.1, "default": 1 },
    { "type": "toggle", "name": "a:t", "text": "T", "default": false },
    { "type": "dropdown", "name": "a:d", "text": "D", "default": "x",
      "options": [ { "name": "x", "text": "X" }, { "name": "y", "text": "Y" } ] },
    { "type": "multiselect", "name": "a:m", "text": "M", "defaults": ["y"],
      "options": [ { "name": "x", "text": "X" }, { "name": "y", "text": "Y" } ] }
  ],
  "subpacks": [ { "folder_name": "high", "name": "High", "memory_performance_tier": 3 } ],
  "metadata": { "authors": ["me"], "license": "MIT", "url": "https://example.com",
                "product_type": "addon", "generated_with": { "tool": ["1.0.0", "2.0"] } },
  "has_education_metadata": true
}"#;
    let manifest = round_trip(text);
    assert_eq!(manifest.format, ManifestFormat::V3);
    assert_eq!(manifest.settings.len(), 5);
    assert_eq!(manifest.subpacks[0].memory_tier, MemoryTier::Tier3);
    assert_eq!(manifest.has_education_metadata, Some(true));
    assert!(matches!(
        manifest.dependencies[2],
        Dependency::NativeModule {
            version: NativeModuleVersion::LatestInternal,
            ..
        }
    ));
    // Written as the integer 3: the comprehensive document's `format_version` is a `u8`
    // and the engine's own format-3 manifests spell it so.
    let written = manifest.to_json().unwrap();
    assert!(written.contains("\"format_version\": 3,"), "{written}");
    assert!(written.contains("\"min\": 0.1,"), "{written}");
    assert_eq!(super::parse_str(&written).unwrap().into_value(), manifest);
}

#[test]
fn legacy_unknown_members_are_kept_after_the_known_ones() {
    let text = format_2(",\n    \"author\": \"me\"", ",\n  \"comment\": [1, 2]");
    let manifest = round_trip(&text);
    assert_eq!(
        manifest
            .header
            .extra
            .get("author")
            .and_then(|v| v.as_string())
            .map(facet_value::VString::as_str),
        Some("me")
    );
    let written = manifest.to_json().unwrap();
    assert!(written.find("\"modules\"").unwrap() < written.find("\"comment\"").unwrap());
}

#[test]
fn legacy_subpacks_write_the_smallest_memory_tier_of_their_tier() {
    let subpacks = r#",
  "subpacks": [
    { "folder_name": "a", "name": "A", "memory_tier": 40 },
    { "folder_name": "b", "name": "B", "memory_tier": 11 },
    { "folder_name": "c", "name": "C", "memory_performance_tier": 4 }
  ]"#;
    let manifest = round_trip(&format_2("", subpacks));
    let tiers: Vec<MemoryTier> = manifest.subpacks.iter().map(|s| s.memory_tier).collect();
    assert_eq!(
        tiers,
        [MemoryTier::Tier5, MemoryTier::Tier1, MemoryTier::Tier4]
    );
}

#[test]
fn a_legacy_manifest_cannot_hold_a_toggle() {
    let mut manifest = Manifest::with_ids("Ruby", PackType::Resource, HEADER, MODULE);
    manifest.settings.push(Setting::Toggle {
        name: "a:t".to_owned(),
        text: "T".to_owned(),
        default: true,
    });
    let mut out = Vec::new();
    let error = manifest
        .write_json(&mut out, WriteStyle::default())
        .unwrap_err();
    assert!(matches!(error, EncodeError::NotRepresentable { .. }));
    assert!(out.is_empty());
    // As format 3 it can.
    manifest.format = ManifestFormat::V3;
    let reread = parse_str(&manifest.to_json().unwrap())
        .unwrap()
        .into_value();
    assert_eq!(reread.settings, manifest.settings);
}

#[test]
fn a_non_finite_slider_number_is_not_representable() {
    let mut manifest = Manifest::with_ids("Ruby", PackType::Resource, HEADER, MODULE);
    manifest.format = ManifestFormat::V3;
    manifest.settings.push(Setting::Slider {
        name: "a:s".to_owned(),
        text: "S".to_owned(),
        min: 0.0,
        max: f32::INFINITY,
        step: 1.0,
        default: 0.0,
    });
    assert!(matches!(
        manifest.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
}

#[test]
fn content_parses_and_writes_like_the_inherent_methods() {
    let bytes = fixture("vanilla/behavior_packs/vanilla/manifest.json");
    let path = PackPath::from_static("manifest.json");
    let document = <Manifest as Content>::parse(&path, &bytes, &LoadContext::detached()).unwrap();
    assert_eq!(*document.value(), parse(&bytes).unwrap().into_value());
    assert_eq!(
        Content::to_json(document.value()).unwrap(),
        document.to_json().unwrap()
    );
}

#[test]
fn the_synthetic_casing_and_legacy_name_manifests_parse_identically() {
    let names = [
        "synthetic/casing/manifest_dir/Manifest.json",
        "synthetic/casing/manifest_dir_upper/MANIFEST.JSON",
        "synthetic/legacy_name/$pack_manifest.json",
    ];
    let header = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
    for name in names {
        let result = parse(&fixture(name));
        // The located bytes are an ordinary manifest: no finding at the header's identity.
        let diagnostics = super::findings(&result);
        assert!(
            !diagnostics
                .iter()
                .any(|d| d.location.json_path.to_string() == "header/uuid"),
            "{name}"
        );
        if let Ok(document) = &result {
            assert_eq!(document.header.uuid, header, "{name}");
        }
    }
    assert_eq!(fixture(names[0]), fixture(names[2]));
    // The empty-pack fixtures hold no manifest at all.
    assert!(
        !super::fixtures()
            .join("synthetic/empty/empty_directory/manifest.json")
            .exists()
    );
    assert!(
        !super::fixtures()
            .join("synthetic/empty/pack_icon_only/manifest.json")
            .exists()
    );
}

#[test]
fn pack_scope_reads_as_its_flags_and_writes_its_spelling() {
    for text in [
        format_2(",\n    \"pack_scope\": \"world\"", ""),
        format_3(",\n    \"pack_scope\": \"world\"", ""),
    ] {
        let manifest = round_trip(&text);
        assert_eq!(manifest.header.pack_scope, Some(PackScope::World));
        let written = manifest.to_json().unwrap();
        assert!(written.contains("\"pack_scope\": \"world\""), "{written}");
    }
}

/// The game's scope table maps `"global"` 1,
/// `"system"` 8, `"world"` 6 and `"any"` 7 with
/// no other spelling: `"System"` is unknown.
#[test]
fn the_system_pack_scope_reads_as_its_flag_and_writes_its_spelling() {
    for text in [
        format_2(",\n    \"pack_scope\": \"system\"", ""),
        format_3(",\n    \"pack_scope\": \"system\"", ""),
    ] {
        let document = loads(&text);
        assert!(
            document.diagnostics().is_empty(),
            "{:#?}",
            document.diagnostics()
        );
        let manifest = round_trip(&text);
        assert_eq!(manifest.header.pack_scope, Some(PackScope::System));
        assert_eq!(PackScope::System.bits(), 8);
        let written = manifest.to_json().unwrap();
        assert!(written.contains("\"pack_scope\": \"system\""), "{written}");
    }
    for text in [
        format_2(",\n    \"pack_scope\": \"System\"", ""),
        format_3(",\n    \"pack_scope\": \"System\"", ""),
    ] {
        let document = loads(&text);
        assert_eq!(document.header.pack_scope, None);
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|d| d.severity == Severity::Info
                    && d.location.json_path.to_string() == "header/pack_scope"),
            "{:#?}",
            document.diagnostics()
        );
    }
}

#[test]
fn deviation_an_unknown_pack_scope_warns_and_is_written_back() {
    for text in [
        format_2(",\n    \"pack_scope\": \"level\"", ""),
        format_3(",\n    \"pack_scope\": \"level\"", ""),
    ] {
        let document = loads(&text);
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|d| d.severity == Severity::Info
                    && d.location.json_path.to_string() == "header/pack_scope"),
            "{:#?}",
            document.diagnostics()
        );
        let manifest = round_trip(&text);
        assert_eq!(manifest.header.pack_scope, None);
        let written = manifest.to_json().unwrap();
        assert!(written.contains("\"pack_scope\": \"level\""), "{written}");
    }
}

#[test]
fn a_scope_without_a_spelling_is_not_representable() {
    let mut manifest = Manifest::with_ids("Ruby", PackType::Resource, HEADER, MODULE);
    manifest.header.pack_scope = Some(PackScope::Level);
    assert!(matches!(
        manifest.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
}

#[test]
fn memory_tiers_are_0_to_5() {
    let tiers: Vec<Option<u8>> = (0..=6)
        .map(|n| MemoryTier::from_number(n).map(MemoryTier::number))
        .collect();
    assert_eq!(
        tiers,
        [Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), None]
    );
    assert_eq!(MemoryTier::from_number(255), None);
}

#[test]
fn generated_with_versions_are_written_as_spelled() {
    let metadata = r#",
  "metadata": { "generated_with": { "tool": ["1.0.0", "2.0", "3", [1, 2, 3]] } }"#;
    let written = round_trip(&format_2("", metadata)).to_json().unwrap();
    let compact: String = written.split_whitespace().collect();
    assert!(
        compact.contains(r#""tool":["1.0.0","2.0","3",[1,2,3]]"#),
        "{written}"
    );
    let metadata = r#",
  "metadata": { "generated_with": { "tool": ["2.0"] } }"#;
    let manifest = round_trip(&format_3("", metadata));
    let tool = &manifest
        .metadata
        .as_ref()
        .unwrap()
        .generated_with
        .as_ref()
        .unwrap()["tool"];
    assert_eq!(tool[0].version, SemVersion::new(2, 0, 0));
    let written = manifest.to_json().unwrap();
    let compact: String = written.split_whitespace().collect();
    assert!(compact.contains(r#""tool":["2.0"]"#), "{written}");
}

#[test]
fn pack_optimization_version_is_written_as_spelled() {
    for (text, expected) in [
        (
            format_2(",\n    \"pack_optimization_version\": [0, 1, 0]", ""),
            r#""pack_optimization_version":[0,1,0]"#,
        ),
        (
            format_2(",\n    \"pack_optimization_version\": \"0.1\"", ""),
            r#""pack_optimization_version":"0.1""#,
        ),
        (
            format_3(",\n    \"pack_optimization_version\": \"0.1\"", ""),
            r#""pack_optimization_version":"0.1""#,
        ),
    ] {
        let written = round_trip(&text).to_json().unwrap();
        let compact: String = written.split_whitespace().collect();
        assert!(compact.contains(expected), "{written}");
    }
}

#[test]
fn an_empty_name_is_kept_and_only_shown_as_noname() {
    for text in [format_2("", ""), format_3("", "")] {
        let text = text.replacen("\"name\": \"Ruby\"", "\"name\": \"\"", 1);
        let document = loads(&text);
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|d| d.severity == Severity::Info
                    && d.location.json_path.to_string() == "header/name"),
            "{:#?}",
            document.diagnostics()
        );
        assert_eq!(document.origin().pack.name(), "manifestvalidation.noname");
        let manifest = round_trip(&text);
        assert_eq!(manifest.header.name, "");
        let written = manifest.to_json().unwrap();
        assert!(written.contains("\"name\": \"\","), "{written}");
    }
}

/// A legacy manifest keeps the unknown members of its root, its header and its modules; its
/// `metadata` has no place for one, and the game reads none of them. The member is not
/// written back, and the load says so at the member (ours: the game is silent).
#[test]
fn an_unknown_member_of_a_legacy_metadata_is_told_as_not_kept() {
    let text = format_2(
        "",
        r#", "metadata": {"authors": ["a"], "x_meta": 1, "license": "MIT"}"#,
    );
    let document = loads(&text);
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|f| {
            format!(
                "{} {:?} at {}",
                f.code.name(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(told, ["MemberNotKept Info at metadata/x_meta"]);
    let written = document.to_json().unwrap();
    assert!(!written.contains("x_meta"), "{written}");
    assert!(written.contains(r#""license": "MIT""#), "{written}");
    // Without the member nothing is told.
    let known = loads(&format_2("", r#", "metadata": {"authors": ["a"]}"#));
    assert!(known.diagnostics().is_empty(), "{:?}", known.diagnostics());
    // Format 0 reads its metadata inside the header: told there.
    let v0 = loads(
        r#"{"header": {"pack_id": "11111111-1111-1111-1111-111111111111", "name": "x", "packs_version": "1.0.0", "modules": [{"type": "resources", "uuid": "22222222-2222-2222-2222-222222222222", "version": "1.0.0"}], "metadata": {"x_meta": 1}}}"#,
    );
    assert!(
        v0.diagnostics()
            .iter()
            .any(|f| f.code.name() == "MemberNotKept"
                && f.location.json_path.to_string() == "header/metadata/x_meta"),
        "{:?}",
        v0.diagnostics()
    );
}

/// A format-0 manifest (no `format_version`, `pack_id` and `packs_version` in its header) is
/// held and written as format 1, the format the game upgrades it to: the members move, so
/// the written text is another text. The load says so at the root (ours: the game upgrades
/// in silence), and the written manifest reads back as the same value.
#[test]
fn a_format_0_manifest_is_written_as_format_1_and_the_load_says_so() {
    let text = r#"{"header": {"pack_id": "11111111-1111-1111-1111-111111111111", "name": "x", "packs_version": "1.0.0", "modules": [{"type": "resources", "uuid": "22222222-2222-2222-2222-222222222222", "version": "1.0.0"}]}}"#;
    let document = loads(text);
    assert_eq!(document.original_format, ManifestFormat::V0);
    assert_eq!(document.format, ManifestFormat::V1);
    let told: Vec<String> = document
        .diagnostics()
        .iter()
        .map(|f| {
            format!(
                "{} {:?} at `{}`",
                f.code.name(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    // (the level-2 deprecation line of the legacy parser comes first)
    assert_eq!(
        told,
        [
            "DeprecatedMember Warning at `format_version`",
            "UnusualFormatVersion Info at ``"
        ]
    );
    let written = document.to_json().unwrap();
    assert!(written.contains(r#""format_version": 1"#), "{written}");
    assert!(
        written.contains(r#""uuid": "11111111-1111-1111-1111-111111111111""#)
            && !written.contains("pack_id"),
        "{written}"
    );
    // The written manifest is one of format 1: nothing more to tell, and the same value but
    // for the format it says it was read in.
    let again = loads(&written);
    assert!(again.diagnostics().is_empty(), "{:?}", again.diagnostics());
    assert_eq!(again.original_format, ManifestFormat::V1);
    let mut expected = document.into_value();
    expected.original_format = ManifestFormat::V1;
    assert_eq!(again.value(), &expected);
    assert_eq!(again.to_json().unwrap(), written);
}
