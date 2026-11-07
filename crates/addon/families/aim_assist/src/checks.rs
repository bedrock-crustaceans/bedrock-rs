//! The checks of the aim-assist documents beyond their shape: the priority constraint of the
//! cereal parse and the namespace checks vanilla runs after it.

use indexmap::IndexMap;

use super::{AimAssistCategories, AimAssistPreset, codes};
use bedrock_addon_core::diagnostic::{CodeDef, Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::versioning::ValidateCx;
use bedrock_addon_core::versioning::shape::{child, element, error, in_range, violation};

/// One priority: a number in `[0, 100]`. A negative number is cereal's "invalid numeric
/// value"; a number above 100 is a range violation.
fn priority(value: i32, at: JsonPath, d: &mut Diagnostics) -> bool {
    if value < 0 {
        violation(
            bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT,
            at,
            d,
        );
        return false;
    }
    in_range(f64::from(value), 0.0, 100.0, at, d)
}

/// Every priority of `map`.
fn priority_map(map: Option<&IndexMap<String, i32>>, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let mut passed = true;
    for (key, value) in map.into_iter().flatten() {
        passed &= priority(*value, child(at, key), d);
    }
    passed
}

/// The priority constraints of a categories file, in bind order; a violation fails the whole
/// file. `at` is the root.
pub(crate) fn priorities(file: &AimAssistCategories, at: &JsonPath, d: &mut Diagnostics) -> bool {
    let categories = child(&child(at, "minecraft:aim_assist_categories"), "categories");
    let mut passed = true;
    for (i, category) in file.categories.iter().enumerate() {
        let at = child(&element(&categories, i), "priorities");
        let p = &category.priorities;
        for (key, map) in [
            ("entities", &p.entities),
            ("blocks", &p.blocks),
            ("block_tags", &p.block_tags),
            ("entity_type_families", &p.entity_type_families),
        ] {
            passed &= priority_map(map.as_ref(), &child(&at, key), d);
        }
        for (key, value) in [
            ("entity_default", p.entity_default),
            ("block_default", p.block_default),
        ] {
            if let Some(value) = value {
                passed &= priority(value, child(&at, key), d);
            }
        }
    }
    passed
}

/// The namespace of an identifier: the text before the last `:`, empty when there is no `:` or it
/// is the first character.
fn namespace(id: &str) -> &str {
    id.rsplit_once(':').map_or("", |(namespace, _)| namespace)
}

/// Whether an identifier has a valid namespace format: exactly one `:` with text on both sides.
fn valid_namespace_format(id: &str) -> bool {
    id.split_once(':').is_some_and(|(namespace, name)| {
        !namespace.is_empty() && !name.is_empty() && !name.contains(':')
    })
}

/// The finding of `id` when it has no namespace (`missing`) or is not `namespace:name`
/// (`format`): an `InvalidValue` error with vanilla's text.
fn namespaced(
    id: &str,
    missing: &'static CodeDef<1>,
    format: &'static CodeDef<1>,
    at: JsonPath,
) -> Option<Diagnostic> {
    let code = if namespace(id).is_empty() {
        missing
    } else if valid_namespace_format(id) {
        return None;
    } else {
        format
    };
    let mut finding = error(code.with([id]), at);
    finding.phase = Phase::Validate;
    Some(finding)
}

/// A1–A4: the preset's identifier, then `default_item_settings`, `hand_settings` and each
/// `item_settings` value; the preset is skipped at the first failure.
pub(crate) fn preset_namespaces(preset: &AimAssistPreset, cx: &mut ValidateCx<'_>) {
    let at = child(&cx.at, "minecraft:aim_assist_preset");
    let identifier = namespaced(
        preset.identifier.as_str(),
        &codes::PRESET_NAMESPACE,
        &codes::PRESET_FORMAT,
        child(&at, "identifier"),
    );
    let category = |id: &str, at: JsonPath| {
        namespaced(
            id,
            &codes::CATEGORY_ID_NAMESPACE,
            &codes::CATEGORY_ID_FORMAT,
            at,
        )
    };
    let single = |key: &str, id: Option<&bedrock_addon_core::id::AimAssistCategoryId>| {
        id.and_then(|id| category(id.as_str(), child(&at, key)))
    };
    let first = identifier
        .or_else(|| {
            single(
                "default_item_settings",
                preset.default_item_settings.as_ref(),
            )
        })
        .or_else(|| single("hand_settings", preset.hand_settings.as_ref()))
        .or_else(|| {
            let items = child(&at, "item_settings");
            preset
                .item_settings
                .iter()
                .flatten()
                .find_map(|(item, id)| category(id.as_str(), child(&items, item)))
        });
    if let Some(finding) = first {
        cx.diagnostics.push(finding);
    }
}

/// A5 and A6: every category name is checked before any is added, so the first bad name drops
/// the whole file.
pub(crate) fn category_namespaces(file: &AimAssistCategories, cx: &mut ValidateCx<'_>) {
    let categories = child(
        &child(&cx.at, "minecraft:aim_assist_categories"),
        "categories",
    );
    let first = file
        .categories
        .iter()
        .enumerate()
        .find_map(|(i, category)| {
            namespaced(
                category.name.as_str(),
                &codes::CATEGORY_NAMESPACE,
                &codes::CATEGORY_FORMAT,
                child(&element(&categories, i), "name"),
            )
        });
    if let Some(finding) = first {
        cx.diagnostics.push(finding);
    }
}
