//! The family's unresolved-reference finding and the two small text readers the load needs: translation keys of a
//! `.lang` file and the name key of a piece.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath};

/// An unresolved reference at `at`, with the name it could not be resolved to.
pub(super) fn unresolved(
    kind: &'static str,
    name: &str,
    code: DiagnosticCode,
    at: JsonPath,
) -> Diagnostic {
    // Every caller sets the severity it means (an information of the crate's own, or an error
    // the game logs for a missing geometry name or animation); the helper's own is the least.
    Diagnostic::info(code)
        .at(at)
        .detail(DiagnosticDetail::Reference {
            kind,
            id: name.to_owned(),
            hint: None,
        })
}

/// The translation key the game shows a piece by: `persona.` + `name` + `.title`, the prefix
/// left out when `name` contains `persona.` and the suffix when it contains `.title`.
///
/// `name` is the piece's `piece_store_name` when it has a store-info object (the empty text
/// when that object lacks the member), else its `piece_name`.
#[must_use]
pub fn name_key(name: &str) -> String {
    let prefix = if name.contains("persona.") {
        ""
    } else {
        "persona."
    };
    let suffix = if name.contains(".title") {
        ""
    } else {
        ".title"
    };
    format!("{prefix}{name}{suffix}")
}

/// The keys of a `.lang` file with their values: one `key=value` per line, the key up to the
/// first `=`; blank lines and `##` comment lines hold none. Of a key written twice the first
/// stands.
pub(super) fn lang_entries(bytes: &[u8]) -> HashMap<String, String> {
    let bytes = bedrock_addon_core::json::strip_bom(bytes);
    let text = String::from_utf8_lossy(bytes);
    let mut entries = HashMap::new();
    for line in text.lines() {
        if line.trim_start().starts_with("##") {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // A value ends at a tab, where an end-of-line comment may follow.
        let value = value.split('\t').next().unwrap_or_default();
        entries
            .entry(key.to_owned())
            .or_insert_with(|| value.to_owned());
    }
    entries
}
