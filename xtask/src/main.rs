pub mod codegen;

use proc_macro2::TokenStream;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, io, process};
use syn::File;

fn main() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("protocol") => run_protocol(&project_root),
        Some("generate-palette") => run_generate_palette(&project_root),
        Some(other) => {
            eprintln!("unrecognized xtask command `{other}`");
            print_usage();
            process::exit(1);
        }
        None => {
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("usage: cargo xtask <command>");
    eprintln!();
    eprintln!("commands:");
    eprintln!(
        "  protocol           regenerate crates/protocol/src/generated from def/versions.def.rs"
    );
    eprintln!(
        "  generate-palette   regenerate crates/vanilla_data/src/generated from the pinned \
         bedrock-samples dumps"
    );
}

fn run_protocol(project_root: &Path) {
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

fn run_generate_palette(project_root: &Path) {
    if let Err(err) = codegen::palette::generate(project_root) {
        eprintln!("generate-palette failed: {err}");
        process::exit(1);
    }
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
