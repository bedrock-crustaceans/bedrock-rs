//! What only a manifest's text says about its script side: where each dependency is in the
//! file, a legacy `optional`, and a format-3 `version` written in the relaxed form.
//! [`Manifest`](super::Manifest) keeps none of the three; `script::ScriptModule` reads them
//! here for the findings of vanilla's stage 1 that depend on them.

use crate::json::{Value, last_wins, strip_bom};

use super::read::{member, text};

/// One `dependencies[]` element that names a native module, as the file has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeDependencyText {
    /// The element's index in the file's `dependencies`.
    pub index: usize,
    /// The element has an `optional` member.
    pub has_optional: bool,
    /// The element's `version` is a string of fewer than three numbers (`"2.0"`, `"2"`).
    pub relaxed: bool,
}

/// The dependencies of a manifest's text by kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DependencyTexts {
    /// The native-module dependencies the reader keeps, in file order.
    pub native: Vec<NativeDependencyText>,
    /// The file index of each pack dependency, in file order.
    pub packs: Vec<usize>,
}

/// What the text of a manifest that [`Manifest::parse`](super::Manifest::parse) accepted says
/// about its dependencies. `legacy` is a format 1 or 2 manifest, whose reader drops an
/// element with both `module_name` and `uuid`.
pub fn dependency_texts(bytes: &[u8], legacy: bool) -> DependencyTexts {
    let mut texts = DependencyTexts::default();
    // The text as the manifest reader takes it: of a member written twice the last stands.
    let bytes = strip_bom(bytes);
    let parsed: Result<Value, _> = match std::str::from_utf8(bytes) {
        Ok(text) => crate::json::from_str_jsonc(&last_wins(text)),
        Err(_) => crate::json::from_slice_jsonc(bytes),
    };
    let Ok(root) = parsed else {
        return texts;
    };
    let Some(elements) = member(&root, "dependencies").and_then(Value::as_array) else {
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
        let relaxed = member(element, "version")
            .and_then(text)
            .is_some_and(|version| !is_selector(version) && number_count(version) != 3);
        texts.native.push(NativeDependencyText {
            index,
            has_optional: member(element, "optional").is_some(),
            relaxed,
        });
    }
    texts
}

/// `alpha`, `beta` or `internal`, byte for byte.
fn is_selector(version: &str) -> bool {
    matches!(version, "alpha" | "beta" | "internal")
}

/// How many numbers a version string writes before its pre-release tag or build metadata.
fn number_count(version: &str) -> usize {
    version
        .split(['-', '+'])
        .next()
        .unwrap_or_default()
        .split('.')
        .count()
}
