//! The namespace checks of the aim-assist documents, which vanilla runs after the cereal parse.

use super::{AimAssistCategories, AimAssistPreset, codes};
use bedrock_addon_core::diagnostic::{CodeDef, Diagnostic, JsonPath, Phase, Severity};
use bedrock_addon_core::versioning::ValidateCx;

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
    let mut finding = Diagnostic::error(code.with([id]))
        .severity(Severity::Error)
        .at(at);
    finding.phase = Phase::Validate;
    Some(finding)
}

/// A1–A4: the preset's identifier, then `default_item_settings`, `hand_settings` and each
/// `item_settings` value; the preset is skipped at the first failure.
pub(crate) fn preset_namespaces(preset: &AimAssistPreset, cx: &mut ValidateCx<'_>) {
    let at = cx.at.child("minecraft:aim_assist_preset");
    let identifier = namespaced(
        preset.identifier.as_str(),
        &codes::PRESET_NAMESPACE,
        &codes::PRESET_FORMAT,
        at.child("identifier"),
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
        id.and_then(|id| category(id.as_str(), at.child(key)))
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
            let items = at.child("item_settings");
            preset
                .item_settings
                .iter()
                .flatten()
                .find_map(|(item, id)| category(id.as_str(), items.child(item)))
        });
    if let Some(finding) = first {
        cx.diagnostics.push(finding);
    }
}

/// A5 and A6: every category name is checked before any is added, so the first bad name drops
/// the whole file.
pub(crate) fn category_namespaces(file: &AimAssistCategories, cx: &mut ValidateCx<'_>) {
    let categories = cx
        .at
        .child("minecraft:aim_assist_categories")
        .child("categories");
    let first = file
        .categories
        .iter()
        .enumerate()
        .find_map(|(i, category)| {
            namespaced(
                category.name.as_str(),
                &codes::CATEGORY_NAMESPACE,
                &codes::CATEGORY_FORMAT,
                categories.index(i).child("name"),
            )
        });
    if let Some(finding) = first {
        cx.diagnostics.push(finding);
    }
}
