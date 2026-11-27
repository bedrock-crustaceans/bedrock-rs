//! The pack builders of the tests that need both the `.lang` files and the language list:
//! packs and stacks built from the fixtures under `tests/fixtures/text/text/`.

use super::common::fixtures;

// ---- packs and stacks ----------------------------------------------------------------------

/// Every file below fixture directory `dir`, as pack-relative paths, sorted.
pub fn files_below(dir: &str) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let root = fixtures().join(dir);
    let mut out = Vec::new();
    walk(&root, &root, &mut out);
    out.sort();
    out
}

/// A format-2 manifest of pack `n` with one module of `kind` (`resources` or `data`) and the
/// header `name`.
pub fn manifest(n: u128, kind: &str, name: &str) -> String {
    let uuid = uuid::Uuid::from_u128(n);
    let module = uuid::Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "{kind}", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    )
}

/// Pack `n` of `kind` holding `files`; a `manifest.json` among them is the pack's, otherwise
/// one named `pack <n>` is added.
pub fn pack_of(n: u128, kind: &str, files: Vec<(String, Vec<u8>)>) -> bedrock_addon::pack::Pack {
    use bedrock_addon::pack::{OpenOptions, Pack};
    use bedrock_addon::storage::PackPath;
    let mut all: Vec<(PackPath, bytes::Bytes)> = Vec::new();
    if !files.iter().any(|(path, _)| path == "manifest.json") {
        all.push((
            PackPath::from_static("manifest.json"),
            bytes::Bytes::from(manifest(n, kind, &format!("pack {n}"))),
        ));
    }
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), bytes::Bytes::from(bytes))),
    );
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// Pack `n` of `kind` with these text files.
pub fn pack_with(n: u128, kind: &str, files: &[(&str, &str)]) -> bedrock_addon::pack::Pack {
    pack_of(
        n,
        kind,
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
            .collect(),
    )
}

/// The packs of fixture case `case` (one subfolder per pack, bottom first on disk), top first
/// as a stack takes them.
pub fn fixture_packs(case: &str, kind: &str) -> Vec<bedrock_addon::pack::Pack> {
    let mut folders: Vec<String> = std::fs::read_dir(fixtures().join(case))
        .unwrap_or_else(|e| panic!("{case}: {e}"))
        .filter_map(|entry| entry.unwrap().file_name().into_string().ok())
        .collect();
    folders.sort();
    let mut packs: Vec<_> = folders
        .iter()
        .enumerate()
        .map(|(index, folder)| {
            pack_of(
                index as u128 + 1,
                kind,
                files_below(&format!("{case}/{folder}")),
            )
        })
        .collect();
    packs.reverse();
    packs
}

/// A resource stack of `packs`, top first.
pub fn resource_stack(
    packs: Vec<bedrock_addon::pack::Pack>,
) -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Resource> {
    bedrock_addon::pack::PackStack::of(packs, bedrock_addon::load::LoadOptions::default())
        .expect("resource packs")
}

/// A behaviour stack of `packs`, top first.
pub fn behavior_stack(
    packs: Vec<bedrock_addon::pack::Pack>,
) -> bedrock_addon::pack::PackStack<bedrock_addon::pack::Behavior> {
    bedrock_addon::pack::PackStack::of(packs, bedrock_addon::load::LoadOptions::default())
        .expect("behaviour packs")
}
