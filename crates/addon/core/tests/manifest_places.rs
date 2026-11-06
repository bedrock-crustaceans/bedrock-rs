//! `Manifest::places`: where the reader found each dependency it kept, whether the element has
//! `optional` and whether a native module's `version` is written relaxed. What a second read
//! of the text (`dependency_texts`, core's until the reader kept the places) told the script
//! stage is the oracle, kept here: over a spread of manifests the two agree.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, missing_docs)]

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{Value, from_slice_jsonc, from_str_jsonc, last_wins, strip_bom};
use bedrock_addon_core::manifest::{
    CapabilityRegistry, Dependency, DependencyPlace, Manifest, ManifestFormat,
};
use bedrock_addon_core::pack_identity::PackPath;

/// One `dependencies[]` element that names a native module, as the file has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NativeDependencyText {
    /// The element's index in the file's `dependencies`.
    index: usize,
    /// The element has an `optional` member.
    has_optional: bool,
    /// The element's `version` is a string of fewer than three numbers (`"2.0"`, `"2"`).
    relaxed: bool,
}

/// The dependencies of a manifest's text by kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DependencyTexts {
    /// The native-module dependencies the reader keeps, in file order.
    native: Vec<NativeDependencyText>,
    /// The file index of each pack dependency, in file order.
    packs: Vec<usize>,
}

/// The oracle: what the text of a manifest that `Manifest::parse` accepted says about its
/// dependencies, read a second time. `legacy` is a format 1 or 2 manifest, whose reader drops
/// an element with both `module_name` and `uuid`. (Core's `dependency_texts`, as it was.)
fn dependency_texts(bytes: &[u8], legacy: bool) -> DependencyTexts {
    let member = |value: &Value, key: &str| value.as_object()?.get(key).cloned();
    let mut texts = DependencyTexts::default();
    // The text as the manifest reader takes it: of a member written twice the last stands.
    let bytes = strip_bom(bytes);
    let parsed: Result<Value, _> = match std::str::from_utf8(bytes) {
        Ok(text) => from_str_jsonc(&last_wins(text)),
        Err(_) => from_slice_jsonc(bytes),
    };
    let Ok(root) = parsed else {
        return texts;
    };
    let Some(elements) = member(&root, "dependencies") else {
        return texts;
    };
    let Some(elements) = elements.as_array() else {
        return texts;
    };
    let present = |element: &Value, key: &str| {
        // The legacy reader takes a `null` member for an absent one.
        member(element, key).is_some_and(|value| !(legacy && value.is_null()))
    };
    for (index, element) in elements.as_slice().iter().enumerate() {
        if element.as_object().is_none() {
            continue;
        }
        if !present(element, "module_name") {
            texts.packs.push(index);
            continue;
        }
        if legacy && present(element, "uuid") {
            // The legacy reader drops such an element with an `InvalidProperty` warning.
            continue;
        }
        let relaxed = member(element, "version").is_some_and(|version| {
            version.as_string().is_some_and(|version| {
                let version = version.as_str();
                !matches!(version, "alpha" | "beta" | "internal")
                    && version
                        .split(['-', '+'])
                        .next()
                        .unwrap_or_default()
                        .split('.')
                        .count()
                        != 3
            })
        });
        texts.native.push(NativeDependencyText {
            index,
            has_optional: member(element, "optional").is_some(),
            relaxed,
        });
    }
    texts
}

/// A manifest of `format` (2 or 3) with the `dependencies` elements `elements`.
fn with_dependencies(format: u8, elements: &str) -> String {
    let (version, engine) = if format == 3 {
        (r#""1.0.0""#, r#""1.21.0""#)
    } else {
        ("[1, 0, 0]", "[1, 21, 0]")
    };
    format!(
        r#"{{
  "format_version": {format},
  // A comment, read as the game reads one.
  "header": {{ "name": "P", "uuid": "11111111-1111-1111-1111-111111111111", "version": {version}, "min_engine_version": {engine} }},
  "modules": [ {{ "type": "script", "uuid": "22222222-2222-2222-2222-222222222222", "version": {version}, "entry": "scripts/main.js" }} ],
  "dependencies": [ {elements} ]
}}"#
    )
}

/// Reads `text`, which must load.
fn read(text: &str) -> Manifest {
    match Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &CapabilityRegistry::default(),
    ) {
        Ok(document) => document.into_value(),
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// The places of `manifest` in the shape of `dependency_texts`.
fn as_texts(manifest: &Manifest) -> DependencyTexts {
    assert_eq!(
        manifest.places.dependencies.len(),
        manifest.dependencies.len(),
        "one place per dependency"
    );
    let mut texts = DependencyTexts::default();
    for (dependency, place) in manifest
        .dependencies
        .iter()
        .zip(&manifest.places.dependencies)
    {
        let index = place
            .at
            .index_after("dependencies")
            .expect("a place in the dependencies");
        assert_eq!(place.at, JsonPath::new().child("dependencies").index(index));
        match dependency {
            Dependency::Pack { .. } => texts.packs.push(index),
            Dependency::NativeModule { .. } => texts.native.push(NativeDependencyText {
                index,
                has_optional: place.has_optional,
                relaxed: place.relaxed,
            }),
        }
    }
    texts
}

const SERVER: &str = "b26a4d4c-afdf-4690-88f8-931846312678";

/// Elements of a legacy manifest: dropped ones, `null` members, `optional` in each spelling.
fn legacy_elements() -> Vec<String> {
    vec![
        String::new(),
        format!(
            r#"{{ "module_name": "@minecraft/server", "uuid": "{SERVER}", "version": "2.0.0" }},
               {{ "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] }},
               {{ "module_name": "@minecraft/server", "version": "beta" }},
               {{ "uuid": "{SERVER}", "version": [0, 1, 0] }}"#
        ),
        r#"{ "module_name": null, "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] },
           { "module_name": "@minecraft/server", "uuid": null, "version": "1.0.0", "optional": null },
           { "module_name": "@minecraft/server-ui", "version": [1, 2, 0], "optional": true },
           { "module_name": "a", "version": "alpha", "optional": false },
           { "module_name": "b", "version": "internal", "x": 1 }"#
            .to_owned(),
        r#"{ "module_name": "a", "version": "1.0.0", "module_name": "b" },
           { "module_name": "c", "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0.0", "uuid": null }"#
            .to_owned(),
    ]
}

/// Elements of a format-3 manifest: relaxed and strict versions, selectors, `optional`.
fn v3_elements() -> Vec<String> {
    vec![
        String::new(),
        r#"{ "module_name": "@minecraft/server", "version": "2.0" },
           { "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0" },
           { "module_name": "@minecraft/server-ui", "version": "2.1.0-beta", "optional": true },
           { "module_name": "a", "version": "2", "optional": false },
           { "module_name": "b", "version": "beta" },
           { "module_name": "c", "version": "1.2.3+build" },
           { "module_name": "d", "version": "1.2.0-rc" }"#
            .to_owned(),
        r#"{ "module_name": "a", "version": "1.0.0", "version": "1.0" }"#.to_owned(),
    ]
}

#[test]
fn the_places_agree_with_the_dependency_texts() {
    let cases = legacy_elements()
        .into_iter()
        .map(|elements| (2, elements))
        .chain(v3_elements().into_iter().map(|elements| (3, elements)));
    for (format, elements) in cases {
        let text = with_dependencies(format, &elements);
        let manifest = read(&text);
        let oracle = dependency_texts(text.as_bytes(), format != 3);
        assert_eq!(as_texts(&manifest), oracle, "{text}");
    }
}

#[test]
fn a_byte_order_mark_does_not_move_the_places() {
    let text = format!("\u{feff}{}", with_dependencies(3, &v3_elements()[1]));
    let manifest = read(&text);
    assert_eq!(
        as_texts(&manifest),
        dependency_texts(text.as_bytes(), false)
    );
}

#[test]
fn a_dropped_legacy_element_keeps_its_place_in_the_file() {
    let manifest = read(&with_dependencies(2, &legacy_elements()[1]));
    let place = |index: usize, has_optional| DependencyPlace {
        at: JsonPath::new().child("dependencies").index(index),
        has_optional,
        relaxed: false,
    };
    assert_eq!(
        manifest.places.dependencies,
        [place(1, false), place(2, false), place(3, false)]
    );
}

#[test]
fn a_relaxed_v3_version_and_an_optional_member_are_kept() {
    let manifest = read(&with_dependencies(
        3,
        r#"{ "module_name": "@minecraft/server", "version": "2.0", "optional": true }"#,
    ));
    assert_eq!(
        manifest.places.dependencies,
        [DependencyPlace {
            at: JsonPath::new().child("dependencies").index(0),
            has_optional: true,
            relaxed: true,
        }]
    );
}

#[test]
fn a_format_0_manifest_and_one_built_by_hand_have_no_places() {
    let text = r#"{ "format_version": 0, "header": { "name": "P", "pack_id": "11111111-1111-1111-1111-111111111111", "packs_version": [1, 0, 0] } }"#;
    let manifest = read(text);
    assert_eq!(manifest.original_format, ManifestFormat::V0);
    assert!(manifest.places.dependencies.is_empty());
    let built = Manifest::with_ids(
        "P",
        bedrock_addon_core::pack_identity::PackType::Behavior,
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
    );
    assert!(built.places.dependencies.is_empty());
}

#[test]
fn places_take_no_part_in_equality() {
    let text = with_dependencies(3, r#"{ "module_name": "a", "version": "2.0" }"#);
    let read = read(&text);
    let mut moved = read.clone();
    moved.places.dependencies.clear();
    assert_eq!(read, moved);
}

#[test]
fn reading_the_places_is_linear() {
    // The measurement of the second read of the text, which the places replace.
    bedrock_addon_core::test_cost::assert_linear_at(
        "Manifest::parse, n dependencies",
        5_000,
        |n| {
            let dependencies: Vec<String> = (0..n)
                .map(|i| {
                    format!(
                        r#"{{ "module_name": "@demo/m{i}", "version": "2.{}", "optional": true }}"#,
                        i % 7
                    )
                })
                .collect();
            with_dependencies(3, &dependencies.join(","))
        },
        |text| {
            let manifest = read(text);
            assert!(
                manifest
                    .places
                    .dependencies
                    .iter()
                    .all(|place| place.relaxed && place.has_optional)
            );
            manifest.places.dependencies.len()
        },
    );
}
