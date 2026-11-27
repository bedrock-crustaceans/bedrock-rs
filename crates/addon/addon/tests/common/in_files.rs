//! The check the step 4 pins share: every finding names its pack and file.

/// Every finding of `findings` names the pack `pack` and one of `files`, and each of `files`
/// has one: the loader gives each finding its document's pack and file (phase 1 step 4 pin).
pub fn assert_in_files<'a>(
    findings: impl IntoIterator<Item = &'a bedrock_addon::diagnostic::Diagnostic>,
    pack: &str,
    files: &[&str],
) {
    let mut named = Vec::new();
    for finding in findings {
        let in_pack = finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name);
        assert_eq!(in_pack, Some(pack), "{finding:?}");
        let path = finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str);
        assert!(path.is_some_and(|p| files.contains(&p)), "{finding:?}");
        named.extend(path.map(str::to_owned));
    }
    for file in files {
        assert!(named.iter().any(|p| p == file), "no finding names {file}");
    }
}
