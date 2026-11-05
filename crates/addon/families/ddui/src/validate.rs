//! The loader's validators, which run on a document whose parse succeeded. The check that a
//! composition's extension point names an `ExtensionPoint` of a loaded root needs every root of
//! the stack, so it belongs to the combine step (`lib.rs`).

use super::UiComposition;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase, VanillaText};
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::versioning::ValidateCx;

/// The vanilla namespace, which the client looks for anywhere in the identifier: a substring
/// search over the whole string, with case.
const VANILLA_NAMESPACE: &str = "minecraft";

/// The client's text for a document of a pack the game ships that lacks the vanilla namespace.
const BUILTIN_TEXT: &str =
    "builtin packs must use the 'minecraft' namespace for DataDrivenUI documents";
/// The client's text for a document of any other pack that has the vanilla namespace.
const EXTERNAL_TEXT: &str =
    "external packs must not use the 'minecraft' namespace for DataDrivenUI documents";
/// The client's text for an extension point written by an external pack (vanilla's spelling).
const OVERRIDEN_TEXT: &str = "extension_point cannot be overriden at external packs.";

/// A document of a pack the game ships has `minecraft` somewhere in its identifier, a
/// document of any other pack has it nowhere. A slice pack counts as shipped in vanilla; the
/// pack model has no slice flag, so only `is_base_game_pack` decides here.
pub(crate) fn namespace(identifier: &str, cx: &mut ValidateCx<'_>) {
    let has = identifier.contains(VANILLA_NAMESPACE);
    let builtin = cx.load.is_base_game_pack;
    if has == builtin {
        return;
    }
    let text = if builtin { BUILTIN_TEXT } else { EXTERNAL_TEXT };
    let at = ["description", "identifier"];
    let error = Diagnostic::error(DiagnosticCode::NamespaceRule {
        text: Some(VanillaText::Fixed(text)),
    })
    .at(cx.at.with(&at))
    .phase(Phase::Validate);
    cx.diagnostics.push(error);
    // An add-on identifier refused although its namespace is not `minecraft` (the test is a
    // substring test) gets a second, crate-own finding that says so by standing there.
    let in_namespace =
        split_namespace(identifier).is_some_and(|(namespace, _)| namespace == VANILLA_NAMESPACE);
    if !builtin && !in_namespace {
        let hint = Diagnostic::info(DiagnosticCode::NamespaceRule { text: None })
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
    let error = Diagnostic::error(DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(OVERRIDEN_TEXT)),
    })
    .at(cx.at.with(&["attribs", "extension_point"]))
    .phase(Phase::Validate);
    cx.diagnostics.push(error);
}
