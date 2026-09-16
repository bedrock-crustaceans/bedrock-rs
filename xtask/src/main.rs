pub mod codegen;
pub mod oracle;

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
        Some("diff-oracle") => run_diff_oracle(args.collect()),
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
    eprintln!(
        "  diff-oracle        dump a fixture world through this crate and diff it against \
         whichever reference-implementation oracles are available; see \
         crates/level/tests/support/oracle.md"
    );
}

fn run_diff_oracle(args: Vec<String>) {
    let mut world_dir: Option<PathBuf> = None;
    let mut work_dir: Option<PathBuf> = None;
    let mut java_jar: Option<PathBuf> = None;
    let mut python_bin: Option<PathBuf> = None;
    let mut python_oracle_permission_acknowledged = false;

    let usage = "usage: cargo xtask diff-oracle <world-dir> [--work-dir <dir>] \
                 [--java-jar <path/to/jar>] [--python-bin <path/to/python>] \
                 [--python-oracle-permission-acknowledged]";

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--work-dir" => match iter.next() {
                Some(value) => work_dir = Some(PathBuf::from(value)),
                None => {
                    eprintln!("--work-dir needs a value\n{usage}");
                    process::exit(1);
                }
            },
            "--java-jar" => match iter.next() {
                Some(value) => java_jar = Some(PathBuf::from(value)),
                None => {
                    eprintln!("--java-jar needs a value\n{usage}");
                    process::exit(1);
                }
            },
            "--python-bin" => match iter.next() {
                Some(value) => python_bin = Some(PathBuf::from(value)),
                None => {
                    eprintln!("--python-bin needs a value\n{usage}");
                    process::exit(1);
                }
            },
            "--python-oracle-permission-acknowledged" => {
                python_oracle_permission_acknowledged = true;
            }
            other if world_dir.is_none() => world_dir = Some(PathBuf::from(other)),
            other => {
                eprintln!("unrecognized diff-oracle argument `{other}`\n{usage}");
                process::exit(1);
            }
        }
    }

    let Some(world_dir) = world_dir else {
        eprintln!("{usage}");
        process::exit(1);
    };
    let work_dir = work_dir.unwrap_or_else(|| PathBuf::from("target/diff-oracle"));

    oracle::run::run(&oracle::run::OracleConfig {
        world_dir,
        work_dir,
        java_jar,
        python_bin,
        python_oracle_permission_acknowledged,
    });
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
