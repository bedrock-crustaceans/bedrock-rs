//! The loader's validators, which run on a document whose parse succeeded. The check that a
//! composition's extension point names an `ExtensionPoint` of a loaded root needs every root of
//! the stack, so it belongs to the combine step (`lib.rs`).

use super::UiComposition;
use super::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Phase, Severity};
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::versioning::ValidateCx;

/// The vanilla namespace, which the client looks for anywhere in the identifier: a substring
/// search over the whole string, with case.
const VANILLA_NAMESPACE: &str = "minecraft";

/// A document of a pack the game ships has `minecraft` somewhere in its identifier, a
/// document of any other pack has it nowhere. A slice pack counts as shipped in vanilla; the
/// pack model has no slice flag, so only `is_base_game_pack` decides here.
pub(crate) fn namespace(identifier: &str, cx: &mut ValidateCx<'_>) {
    let has = identifier.contains(VANILLA_NAMESPACE);
    let builtin = cx.load.is_base_game_pack;
    if has == builtin {
        return;
    }
    let rule = if builtin {
        &codes::BUILTIN_NAMESPACE
    } else {
        &codes::EXTERNAL_NAMESPACE
    };
    let at = ["description", "identifier"];
    let error = Diagnostic::of(rule)
        .at(cx.at.with(&at))
        .phase(Phase::Validate);
    cx.diagnostics.push(error);
    // An add-on identifier refused although its namespace is not `minecraft` (the test is a
    // substring test) gets a second, crate-own finding that says so by standing there.
    let in_namespace =
        split_namespace(identifier).is_some_and(|(namespace, _)| namespace == VANILLA_NAMESPACE);
    if !builtin && !in_namespace {
        let hint = Diagnostic::of(&core_codes::NAMESPACE_RULE)
            .severity(Severity::Info)
            .at(cx.at.with(&at))
            .phase(Phase::Validate);
        cx.diagnostics.push(hint);
    }
}

/// Only a pack the game ships may splice a composition in at an extension point. The client
/// goes on to look the name up either way.
pub(crate) fn external_extension_point(composition: &UiComposition, cx: &mut ValidateCx<'_>) {
    if composition.attribs.extension_point.is_none() || cx.load.is_base_game_pack {
        return;
    }
    let error = Diagnostic::of(&codes::EXTENSION_POINT_OVERRIDDEN)
        .at(cx.at.with(&["attribs", "extension_point"]))
        .phase(Phase::Validate);
    cx.diagnostics.push(error);
}
