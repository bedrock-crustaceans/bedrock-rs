//! The game's own texts for this family, with `{}` where the game
//! formats an argument, and the two small text readers the load needs: translation keys of a
//! `.lang` file and the name key of a piece.

use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, VanillaText,
};

pub(super) const META_NOT_FOUND: &str = "Could not find expected metafile json file {}";
pub(super) const META_NOT_PARSED: &str = "Could not parse metafile json file {}";
pub(super) const GEOMETRY_NOT_FOUND: &str = "Missing expected geometry file {}";
pub(super) const GEOMETRY_NOT_PARSED: &str = "Could not parse geometry file {}";
pub(super) const GEOMETRY_NAME_NOT_FOUND: &str = "Couldn't find geometry name {} on {}.";
pub(super) const SIDE_NOT_ARM_OR_LEG: &str = "Trying to assign the side attribute to a piece that is not an arm or leg (pieceType: {}, side: '{}')";
pub(super) const ANIMATION_FILE_MISSING: &str =
    "Trying to parse emote {} with missing anim file: {}";
pub(super) const ANIMATION_NOT_IN_FILE: &str =
    "Animation {} declared by emote piece {} was not found in file {}";
pub(super) const EMOTE_INVALID: &str =
    "Trying to parse invalid emote animation definition with pieceId {}";

/// Every template of the family with the number of arguments it takes.
#[cfg(test)]
pub(super) const TEMPLATES: &[(&str, usize)] = &[
    (META_NOT_FOUND, 1),
    (META_NOT_PARSED, 1),
    (GEOMETRY_NOT_FOUND, 1),
    (GEOMETRY_NOT_PARSED, 1),
    (GEOMETRY_NAME_NOT_FOUND, 2),
    (SIDE_NOT_ARM_OR_LEG, 2),
    (ANIMATION_FILE_MISSING, 2),
    (ANIMATION_NOT_IN_FILE, 3),
    (EMOTE_INVALID, 1),
];

/// The game's text `template` with `args`.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills a code's optional text"
)]
pub(super) fn vanilla<const N: usize>(
    template: &'static str,
    args: [&str; N],
) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    })
}

/// `finding` with the name a reference could not be resolved to.
pub(super) fn unresolved(
    kind: &'static str,
    name: &str,
    text: Option<VanillaText>,
    at: JsonPath,
) -> Diagnostic {
    // Every caller sets the severity it means (an information of the crate's own, or an error
    // the game logs for a missing geometry name or animation); the helper's own is the least.
    Diagnostic::info(DiagnosticCode::UnresolvedReference { text })
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
