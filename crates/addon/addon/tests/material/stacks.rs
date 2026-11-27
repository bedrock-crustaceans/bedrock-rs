//! The in-line resource stacks of the material tests (the lowest pack first: `pack_0` is the
//! lowest priority).

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;

/// A format-2 resource manifest with UUID `n`.
fn manifest(n: u128) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "resources", "uuid": "{}", "version": [1, 0, 0] }}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

/// A resource pack of `files` (path, bytes) with UUID `n`.
fn pack_of(n: u128, files: &[(&str, &[u8])]) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(n).into_bytes(),
    )];
    for (path, bytes) in files {
        all.push((PackPath::new(path).expect("a pack path"), bytes.to_vec()));
    }
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).unwrap_or_else(|error| panic!("pack {n}: {error}"))
}

/// One pack of a test stack: its files as (path, text).
pub type Files<'a> = &'a [(&'a str, &'a str)];

/// A resource stack of in-line packs, the **lowest** first: `packs[0]` is
/// `pack_0`. The stack index of `packs[i]` is `packs.len() - 1 - i`.
pub fn stack(packs: &[Files<'_>]) -> PackStack<Resource> {
    let opened: Vec<Pack> = packs
        .iter()
        .enumerate()
        .rev()
        .map(|(n, files)| {
            let files: Vec<(&str, &[u8])> = files
                .iter()
                .map(|(path, text)| (*path, text.as_bytes()))
                .collect();
            pack_of(n as u128 + 1, &files)
        })
        .collect();
    PackStack::<Resource>::of(opened, LoadOptions::default()).expect("resource packs")
}
