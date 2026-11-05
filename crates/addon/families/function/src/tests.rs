//! Unit tests of the module's private parts: the texts' arity.

use super::texts;

#[test]
fn every_formatted_text_has_a_placeholder_per_argument() {
    for (template, arguments) in texts::FORMATTED {
        assert_eq!(template.matches("{}").count(), *arguments, "{template}");
    }
}

#[test]
fn the_hinted_texts_extend_the_unknown_command_text() {
    for hinted in [texts::UNKNOWN_COMMAND_COMMENT, texts::UNKNOWN_COMMAND_SLASH] {
        assert!(hinted.starts_with(texts::UNKNOWN_COMMAND), "{hinted}");
        assert!(hinted.ends_with(')'), "{hinted}");
    }
}

/// Every finding of `result` (loaded or rejected), each of which must name the pack of no pack
/// and `path`: the loader locates them all in the file.
fn assert_located<T>(
    result: &Result<
        bedrock_addon_core::document::Document<T>,
        bedrock_addon_core::document::Rejected,
    >,
    path: &bedrock_addon_core::pack_identity::PackPath,
) {
    use bedrock_addon_core::pack_identity::PackRef;
    let found = match result {
        Ok(document) => document.diagnostics(),
        Err(rejected) => &rejected.diagnostics,
    };
    assert!(!found.is_empty(), "no finding to check");
    for finding in found {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:#?}"
        );
        assert_eq!(finding.location.path.as_ref(), Some(path), "{finding:#?}");
    }
}

#[test]
fn every_finding_of_a_function_file_carries_its_pack_and_path() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    let cx = LoadContext::detached();
    // A file name the game ignores: rejected.
    let nameless = PackPath::from_static("functions/.mcfunction");
    let result = super::Function::parse(&nameless, b"say hi", &cx);
    assert!(result.is_err());
    assert_located(&result, &nameless);
    // Bytes that are not UTF-8: loaded with an information.
    let path = PackPath::from_static("functions/a.mcfunction");
    let result = super::Function::parse(&path, b"say \xff", &cx);
    assert!(result.is_ok());
    assert_located(&result, &path);
}

#[test]
fn every_finding_of_a_tick_list_carries_its_pack_and_path() {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    let cx = LoadContext::detached();
    let path = PackPath::from_static("functions/tick.json");
    // `values` that is no array: rejected.
    let result = super::TickList::parse(&path, br#"{"values": 3}"#, &cx);
    assert!(result.is_err());
    assert_located(&result, &path);
    // A number no double holds: loaded with the clamp's warning.
    let result = super::TickList::parse(&path, br#"{"values": [], "n": 1e400}"#, &cx);
    assert!(result.is_ok(), "{result:#?}");
    assert_located(&result, &path);
}
