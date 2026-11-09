mod actor_types;
mod addon_features;
pub mod codegen;
mod entity_schemas;
mod goal_numeric;
mod script_meta;

use proc_macro2::TokenStream;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, io, process};
use syn::File;

const USAGE: &str = "usage: cargo xtask [protocol | actor-types --samples <dir> | script-meta --samples <dir> | entity-schemas | addon-features]";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        [] | ["protocol"] => {
            protocol();
            Ok(())
        }
        ["actor-types", "--samples", dir] => actor_types::run(Path::new(dir)),
        ["script-meta", "--samples", dir] => script_meta::run(Path::new(dir)),
        ["entity-schemas"] => entity_schemas::run(),
        ["addon-features"] => addon_features::run(),
        _ => {
            eprintln!("{USAGE}");
            process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("{error}");
        process::exit(1);
    }
}

/// Regenerates `crates/protocol/src/generated/` from `crates/protocol/def/versions.def.rs`.
fn protocol() {
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

#[cfg(test)]
mod tests {
    #[test]
    fn xtask_does_not_depend_on_bedrock_addon() {
        // A Rust error in the composing crate must not take the generators down with it.
        let manifest = include_str!("../Cargo.toml");
        let line = manifest
            .lines()
            .find(|line| line.trim_start().starts_with("bedrock_addon ="));
        assert_eq!(line, None, "xtask/Cargo.toml depends on bedrock_addon");
    }
}
