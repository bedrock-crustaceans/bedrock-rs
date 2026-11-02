//! The crate's own load-time rule on a controller that loaded: an `array.<name>` an expression
//! indexes is declared in the `arrays` kind of the member that uses it. The game checks nothing
//! when it loads the file; the lookup fails when the controller is first used.

use indexmap::{IndexMap, IndexSet};

use super::RenderController;
use super::read::controller_path;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, Location, Phase, Severity,
};
use bedrock_addon_core::molang::MolangSource;

/// The `array.<name>` references of a Molang text, lower-cased, outside its string literals:
/// `array.` not preceded by an identifier character, then the name's identifier characters.
pub(crate) fn array_names(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let name_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'.';
    let mut names = Vec::new();
    let mut quoted = false;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            quoted = !quoted;
            index += 1;
            continue;
        }
        let starts = !quoted
            && bytes[index..]
                .get(..6)
                .is_some_and(|head| head.eq_ignore_ascii_case(b"array."))
            && (index == 0 || !name_byte(bytes[index - 1]));
        if !starts {
            index += 1;
            continue;
        }
        let end = bytes[index..]
            .iter()
            .position(|b| !name_byte(*b))
            .map_or(bytes.len(), |n| index + n);
        if end > index + 6 {
            names.push(text[index..end].to_ascii_lowercase());
        }
        index = end;
    }
    names
}

/// Reports each array a source of `member` indexes and `declared` lacks.
fn check<'s>(
    key: &str,
    member: &str,
    sources: impl Iterator<Item = &'s MolangSource>,
    declared: Option<&IndexMap<String, Vec<MolangSource>>>,
    diagnostics: &mut Diagnostics,
) {
    // The declared names, lower-cased once: a reference is compared ignoring case.
    let declared: IndexSet<String> = declared
        .into_iter()
        .flat_map(IndexMap::keys)
        .map(|name| name.to_ascii_lowercase())
        .collect();
    for source in sources {
        for name in array_names(source.as_str()) {
            if declared.contains(&name) {
                continue;
            }
            // The game makes no load-time check; its "Error processing expression" is a draw-time
            // line, so the crate reports an information.
            let mut finding = Diagnostic::new(
                DiagnosticCode::UnresolvedReference { text: None },
                Severity::Info,
                Phase::Validate,
                Location::new(None, None, controller_path(key, Some(member)), None),
            );
            finding.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: "array",
                id: name,
                hint: None,
            }));
            diagnostics.push(finding);
        }
    }
}

/// Rule 26 on the controller `key`.
pub(crate) fn array_references(
    key: &str,
    controller: &RenderController,
    diagnostics: &mut Diagnostics,
) {
    let arrays = controller.arrays.as_ref();
    check(
        key,
        "geometry",
        std::iter::once(&controller.geometry),
        arrays.map(|a| &a.geometries),
        diagnostics,
    );
    check(
        key,
        "textures",
        controller.textures.iter(),
        arrays.map(|a| &a.textures),
        diagnostics,
    );
    check(
        key,
        "materials",
        controller.materials.iter().map(|(_, source)| source),
        arrays.map(|a| &a.materials),
        diagnostics,
    );
}

#[cfg(test)]
mod tests {
    use super::array_names;

    #[test]
    fn array_names_are_found_outside_string_literals() {
        assert_eq!(
            array_names("q.x == 'array.no' ? array.skins[q.variant] : Array.Other[0]"),
            ["array.skins", "array.other"]
        );
    }

    #[test]
    fn a_name_inside_another_identifier_is_no_reference() {
        assert!(array_names("my_array.skins[0]").is_empty());
        assert!(array_names("array.").is_empty());
        assert!(array_names("texture.default").is_empty());
    }
}
