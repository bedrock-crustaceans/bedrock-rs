pub mod codegen;

use proc_macro2::TokenStream;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};
use syn::File;

fn main() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let protocol = project_root.join("crates/protocol/");

    let text =
        fs::read_to_string(protocol.join("def/versions.def.rs")).expect("versions file not found");
    let tokens = TokenStream::from_str(&text).unwrap();

    clear_directory(protocol.join("src/generated").to_str().unwrap()).unwrap();

    let version_tokens = codegen::protocol::build(tokens, protocol.as_path()).unwrap();
    let file = syn::parse2::<File>(version_tokens).unwrap();

    fs::write(
        protocol.join("src/generated/mod.rs"),
        prettyplease::unparse(&file),
    )
    .unwrap();
}

fn clear_directory(path: &str) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let path = entry.path();

        if meta.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}
