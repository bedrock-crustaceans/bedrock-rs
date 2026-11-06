//! The fixture packs the writer's round trips run over, each with the files a copy of it must
//! hold, taken from the fixture itself and not from the pack that was opened.

use std::path::Path;

use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{OpenOptions, Pack};
use bytes::Bytes;

use super::{fixture, fixtures, parsed, path, ruby_manifest};

/// One fixture pack.
pub struct Case {
    /// The fixture's name, for messages.
    pub name: String,
    /// The pack, opened.
    pub pack: Pack,
    /// The manifest the fixture's `manifest.json` reads as.
    pub manifest: Manifest,
    /// Every file a copy holds besides `manifest.json`, bytewise by path.
    pub files: Vec<(String, Vec<u8>)>,
}

/// Every file below `dir`, sorted, as paths relative to it with `/`.
fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let at = entry.unwrap().path();
            if at.is_dir() {
                walk(root, &at, out);
            } else {
                let relative: Vec<String> = at
                    .strip_prefix(root)
                    .unwrap()
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect();
                out.push((relative.join("/"), std::fs::read(&at).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A pack opened from `files` (the manifest among them), as a folder in memory.
fn folder(name: &str, files: Vec<(String, Vec<u8>)>, dropped: &[&str]) -> Case {
    let pack = Pack::from_files(
        files
            .iter()
            .map(|(at, bytes)| (path(at), Bytes::from(bytes.clone()))),
        &OpenOptions::default(),
    )
    .unwrap_or_else(|error| panic!("{name}: {error}"));
    let manifest = files
        .iter()
        .find(|(at, _)| at == "manifest.json")
        .map_or_else(|| panic!("{name}: no manifest"), |(_, bytes)| parsed(bytes));
    let mut files: Vec<(String, Vec<u8>)> = files
        .into_iter()
        .filter(|(at, _)| at != "manifest.json" && !dropped.contains(&at.as_str()))
        .collect();
    files.sort();
    Case {
        name: name.to_owned(),
        pack,
        manifest,
        files,
    }
}

/// The 221 vanilla manifests, each a folder pack of one file: formats 1 and 2, CRLF line ends,
/// comments, subpack lists.
pub fn vanilla_manifests() -> Vec<Case> {
    let root = fixtures().join("manifest/vanilla");
    let manifests: Vec<(String, Vec<u8>)> = files_below(&root)
        .into_iter()
        .filter(|(at, _)| at.ends_with("manifest.json"))
        .collect();
    assert_eq!(manifests.len(), 221);
    // What a copy does not keep of the text is among them: CRLF line ends and comments.
    let with = |needle: &[u8]| {
        manifests
            .iter()
            .filter(|(_, bytes)| bytes.windows(needle.len()).any(|window| window == needle))
            .count()
    };
    assert!(with(b"\r\n") > 0 && with(b"\r\n") < manifests.len());
    assert!(with(b"// ") > 0);
    manifests
        .into_iter()
        .map(|(at, bytes)| folder(&at, vec![("manifest.json".to_owned(), bytes)], &[]))
        .collect()
}

/// A folder pack with a format-3 manifest and script files beside it.
pub fn v3_folder() -> Case {
    let dir = "script/script_module/sm42_optional_unshipped_v3";
    let files = files_below(&fixtures().join(dir));
    assert!(files.len() > 2, "{dir}: a manifest and more");
    folder(dir, files, &[])
}

/// The text of a new format-2 manifest.
fn ruby_manifest_json() -> Vec<u8> {
    ruby_manifest().to_json().unwrap().into_bytes()
}

/// A folder pack with a subpack folder, a plain `contents.json` and a `signatures.json`.
pub fn folder_with_subpacks() -> Case {
    let mut manifest = ruby_manifest();
    manifest.subpacks.push(bedrock_addon::manifest::Subpack {
        folder_name: "low".to_owned(),
        name: "Low".to_owned(),
        memory_tier: bedrock_addon::manifest::MemoryTier::default(),
    });
    let files = vec![
        (
            "manifest.json".to_owned(),
            manifest.to_json().unwrap().into_bytes(),
        ),
        (
            "contents.json".to_owned(),
            br#"{"version":1,"content":[{"path":"textures/a.png"}]}"#.to_vec(),
        ),
        ("signatures.json".to_owned(), b"[]".to_vec()),
        ("textures/a.png".to_owned(), b"\x89PNG root".to_vec()),
        (
            "subpacks/low/textures/a.png".to_owned(),
            b"\x89PNG low".to_vec(),
        ),
        // Only the two at the pack root describe the pack's bytes.
        ("subpacks/low/contents.json".to_owned(), b"{}".to_vec()),
    ];
    folder(
        "folder with subpacks",
        files,
        &["contents.json", "signatures.json"],
    )
}

/// A folder pack whose `blocks/` is served by `__brarchive/blocks.brarchive`
/// (`crates/addon/storage/tests/fixtures/brarchive/small.brarchive`: `ruby.json` 88 bytes, `main.js` empty,
/// `sapphire.json` 39 bytes) over two loose files. A copy holds the layer's files loose and no
/// archive: `ruby.json` and `sapphire.json` from the archive, `main.js` from the loose file its
/// empty entry does not shadow.
pub fn brarchive_folder() -> Case {
    let archive = fixture("../../../storage/tests/fixtures/brarchive/small.brarchive");
    // The data starts after the header and three descriptors.
    let data = &archive[16 + 3 * 256..];
    let (ruby, sapphire) = (data[..88].to_vec(), data[88..88 + 39].to_vec());
    let mut case = folder(
        "brarchive folder",
        vec![
            ("manifest.json".to_owned(), ruby_manifest_json()),
            ("__brarchive/blocks.brarchive".to_owned(), archive),
            ("blocks/main.js".to_owned(), b"loose script".to_vec()),
            ("blocks/ruby.json".to_owned(), b"shadowed".to_vec()),
            ("textures/a.png".to_owned(), b"\x89PNG".to_vec()),
        ],
        &["__brarchive/blocks.brarchive", "blocks/ruby.json"],
    );
    case.files.push(("blocks/ruby.json".to_owned(), ruby));
    case.files
        .push(("blocks/sapphire.json".to_owned(), sapphire));
    case.files.sort();
    case
}

/// The entries of a zip's `unzip -Z1` listing that are files.
#[cfg(feature = "zip")]
fn listing(zip: &str) -> Vec<String> {
    let text = String::from_utf8(fixture(&format!(
        "../../../storage/tests/fixtures/zip_listings/{zip}.txt"
    )))
    .unwrap();
    text.lines()
        .filter(|line| !line.ends_with('/') && !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The fixture zips that hold one pack in plain files, each read here with the `zip` crate
/// itself: 14 of the 23 (eight hold several manifests at one depth, `bakedPack.zip` is
/// encrypted). `BlockTextureSetTest_Stacking.zip` has subpack folders; several keep their pack
/// in a folder of the archive.
#[cfg(feature = "zip")]
pub fn zip_packs() -> Vec<Case> {
    use std::io::{Cursor, Read};

    let mut zips: Vec<String> =
        std::fs::read_dir(fixtures().join("../../../storage/tests/fixtures/zip_packs"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
    zips.sort();
    let mut cases = Vec::new();
    for zip in zips {
        let entries = listing(&zip);
        let manifests: Vec<&String> = entries
            .iter()
            .filter(|entry| entry.rsplit('/').next() == Some("manifest.json"))
            .collect();
        let least = manifests
            .iter()
            .map(|m| m.matches('/').count())
            .min()
            .unwrap();
        let shallowest: Vec<&&String> = manifests
            .iter()
            .filter(|m| m.matches('/').count() == least)
            .collect();
        if shallowest.len() != 1 || zip == "bakedPack.zip" {
            continue;
        }
        let prefix = shallowest[0].strip_suffix("manifest.json").unwrap();
        let bytes = fixture(&format!("../../../storage/tests/fixtures/zip_packs/{zip}"));
        let mut archive = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
        let mut manifest = None;
        let mut files = Vec::new();
        for entry in &entries {
            let Some(relative) = entry.strip_prefix(prefix) else {
                continue;
            };
            let mut content = Vec::new();
            archive
                .by_name(entry)
                .unwrap()
                .read_to_end(&mut content)
                .unwrap();
            match relative {
                "manifest.json" => manifest = Some(parsed(&content)),
                "contents.json" | "signatures.json" => {}
                _ => files.push((relative.to_owned(), content)),
            }
        }
        files.sort();
        cases.push(Case {
            pack: Pack::from_archive(bytes, &OpenOptions::default())
                .unwrap_or_else(|error| panic!("{zip}: {error}")),
            name: zip,
            manifest: manifest.unwrap(),
            files,
        });
    }
    assert_eq!(cases.len(), 14);
    assert!(
        cases
            .iter()
            .any(|case| case.files.iter().any(|(at, _)| at.starts_with("subpacks/"))),
        "a zip pack with subpack folders"
    );
    cases
}

/// Every fixture pack with plain files.
pub fn plain_packs() -> Vec<Case> {
    let mut cases = vanilla_manifests();
    cases.push(v3_folder());
    cases.push(folder_with_subpacks());
    cases.push(brarchive_folder());
    #[cfg(feature = "zip")]
    cases.extend(zip_packs());
    cases
}
