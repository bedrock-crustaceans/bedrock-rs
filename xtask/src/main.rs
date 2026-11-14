mod actor_types;
mod addon_features;
pub mod codegen;
mod entity_schemas;
mod goal_numeric;
mod markers;
mod script_meta;

use codegen::protocol;
use std::ffi::OsStr;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs, io, process};

const USAGE: &str = "usage: cargo xtask [protocol | actor-types --samples <dir> | script-meta --samples <dir> | entity-schemas | addon-features]";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        [] | ["protocol"] => protocol(),
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

/// The workspace root: the directory above this crate's.
fn workspace_root() -> io::Result<&'static Path> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask has no parent directory"))
}

/// The edition the generated Rust files are formatted as: the workspace's.
const EDITION: &str = "2024";

/// Regenerates the protocol's version crates, the traits and the facade from
/// `crates/protocol/def/versions.def.rs`.
fn protocol() -> io::Result<()> {
    write_protocol(&workspace_root()?.join(protocol::ROOT), rustfmt).map(drop)
}

/// `text`, a Rust file, as `rustfmt` formats it: `$RUSTFMT` if set, as `cargo fmt` has it, or
/// the `rustfmt` of the toolchain.
fn rustfmt(text: &str) -> io::Result<String> {
    let program = env::var_os("RUSTFMT").unwrap_or_else(|| "rustfmt".into());
    format_with(&program, text)
}

/// `text`, a Rust file, as the formatter `program` prints it from its standard input.
fn format_with(program: &OsStr, text: &str) -> io::Result<String> {
    let name = program.display();
    let mut child = Command::new(program)
        .args(["--edition", EDITION])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            let hint = "the generated Rust files are formatted with rustfmt; install it with \
                        `rustup component add rustfmt`";
            io::Error::new(
                error.kind(),
                format!("cannot run `{name}`: {error}\n{hint}"),
            )
        })?;
    // rustfmt reads its whole input before it prints, so the pipes cannot fill up against each
    // other. A formatter that exits early breaks the pipe; its exit status says why.
    let fed = child
        .stdin
        .take()
        .map(|mut stdin| stdin.write_all(text.as_bytes()));
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!(
            "`{name}` failed ({}):\n{}",
            output.status,
            stderr.trim_end()
        )));
    }
    fed.transpose()?;
    String::from_utf8(output.stdout).map_err(io::Error::other)
}

/// Writes what the protocol generator renders for the tree under `dir` (`crates/protocol`), says
/// which files it wrote, and returns their paths below `dir`. A Rust file is written as `format` returns it, so
/// what is on disk is final and a later `cargo fmt` leaves it alone. A file that is already what
/// would be written is not touched, and so nothing is rebuilt for it.
///
/// Everything is rendered and formatted before anything is written, so an error of the
/// definition, of a source or of the formatter leaves the tree as it was. The files are then
/// written one after another, the version crates before the facade that names them: an I/O
/// error among them leaves those before it written, and the next run writes the rest. A
/// directory that is no version of the definition is an error too, but one returned after the
/// files are written: only then does no manifest name the directory, and it can be deleted.
fn write_protocol(
    dir: &Path,
    format: impl Fn(&str) -> io::Result<String>,
) -> io::Result<Vec<String>> {
    let protocol::Rendering {
        mut outputs,
        left_behind,
    } = render_protocol(dir)?;
    // The message is made here, on the thread that rendered, like that of any other error.
    let left_behind = left_behind.map(|error| error.to_string());
    for output in &mut outputs {
        if is_rust(&output.path) {
            output.contents = format(&output.contents).map_err(|error| {
                let path = format!("{}/{}", protocol::ROOT, output.path);
                io::Error::new(error.kind(), format!("{path}: {error}"))
            })?;
        }
    }

    let mut written = Vec::new();
    for output in outputs {
        let path = dir.join(&output.path);
        if fs::read_to_string(&path).is_ok_and(|current| current == output.contents) {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &output.contents)?;
        if output.scaffold {
            eprintln!(
                "cargo xtask: {}/{} is the start of a new version crate. It is hand-written \
                 from here on: write the modules it declares, then run `cargo xtask` again, so \
                 that the crate's manifest lists what they use.",
                protocol::ROOT,
                output.path
            );
        }
        eprintln!("cargo xtask: wrote {}/{}", protocol::ROOT, output.path);
        written.push(output.path);
    }
    left_behind.map_or(Ok(written), |message| Err(io::Error::other(message)))
}

/// Whether the generated file at `path` is Rust.
fn is_rust(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension == "rs")
}

/// What the protocol generator renders from the definition and the tree under `dir`
/// (`crates/protocol`).
fn render_protocol(dir: &Path) -> io::Result<protocol::Rendering> {
    let definition = fs::read_to_string(dir.join(protocol::DEFINITION))?;
    let tree = read_protocol_tree(dir)?;
    // The message is made here: the place of a definition error is only known on this thread.
    protocol::render(&definition, &tree).map_err(|error| io::Error::other(error.to_string()))
}

/// What the protocol generator reads of the tree under `dir`: the facade's manifest, and every
/// directory of the versions directory with the `.rs` files under its `src/`. A directory
/// without a `src/` is recorded with no sources: the generator must see that it is there.
fn read_protocol_tree(dir: &Path) -> io::Result<protocol::Tree> {
    let mut tree = protocol::Tree {
        facade_manifest: fs::read_to_string(dir.join(protocol::FACADE_MANIFEST))?,
        ..protocol::Tree::default()
    };
    for entry in fs::read_dir(dir.join(protocol::VERSIONS))? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        let mut sources = protocol::Sources::new();
        let src = entry.path().join("src");
        if src.is_dir() {
            read_sources(&src, "", &mut sources)?;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        tree.sources.insert(name, sources);
    }
    Ok(tree)
}

/// Adds every `.rs` file under `dir` to `sources`, under `prefix` and its path below `dir`.
fn read_sources(dir: &Path, prefix: &str, sources: &mut protocol::Sources) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            read_sources(&path, &format!("{prefix}{name}/"), sources)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.insert(format!("{prefix}{name}"), fs::read_to_string(&path)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Rust file as prettyplease prints its syntax tree, with every run of `use` items in one
    /// order; `None` if it does not parse. A generated file reads the same here whether it is
    /// as the generator rendered it or as rustfmt left it, so the test needs no rustfmt:
    /// prettyplease lays the tree out by its own rules (where a line breaks, where a trailing
    /// comma goes, whether a match arm of one expression gets braces), and the one thing
    /// rustfmt changes in the tree itself is the order of consecutive `use` items.
    fn normalised(text: &str) -> Option<String> {
        let is_use = |item: &syn::Item| matches!(item, syn::Item::Use(_));
        let mut file = syn::parse_file(text).ok()?;
        for run in file.items.chunk_by_mut(|a, b| is_use(a) && is_use(b)) {
            run.sort_by_cached_key(|item| quote::ToTokens::to_token_stream(item).to_string());
        }
        Some(prettyplease::unparse(&file))
    }

    #[test]
    fn the_comparison_reads_through_what_rustfmt_does_to_a_generated_file() {
        // As prettyplease prints it at this width, and as rustfmt would leave a deeper or a
        // longer copy: a wrapped variant and a wrapped call with their trailing commas, a
        // match arm of one expression in braces, the imports sorted.
        let rendered = "use b::Y;\nuse a::X;\n\
             pub enum V { A(Box<<Self as P>::A>), Unknown(Box<U>) }\n\
             fn f(r: R) -> V { match r { Ok(pk) => V::A(Box::new(pk)), Err(e) => V::Unknown(e) } }\n";
        let formatted = "use a::X;\nuse b::Y;\n\
             pub enum V {\n    A(\n        Box<<Self as P>::A>,\n    ),\n    Unknown(Box<U>),\n}\n\
             fn f(r: R) -> V {\n    match r {\n        Ok(pk) => {\n            V::A(\n                Box::new(pk),\n            )\n        }\n        Err(e) => V::Unknown(e),\n    }\n}\n";
        assert!(normalised(rendered).is_some());
        assert_eq!(normalised(rendered), normalised(formatted));
    }

    #[test]
    fn the_comparison_sees_a_change_of_the_code() {
        let rendered = "use a::X;\npub enum V { A(Box<X>), B(Box<X>) }\n";
        let without_a_variant = "use a::X;\npub enum V { A(Box<X>) }\n";
        let variants_swapped = "use a::X;\npub enum V { B(Box<X>), A(Box<X>) }\n";
        let another_path = "use b::X;\npub enum V { A(Box<X>), B(Box<X>) }\n";
        for changed in [without_a_variant, variants_swapped, another_path] {
            assert_ne!(normalised(rendered), normalised(changed), "{changed}");
        }
        assert_eq!(normalised("pub enum V {"), None);
    }

    #[test]
    fn the_protocol_files_are_what_cargo_xtask_generates() {
        let dir = workspace_root().unwrap().join(protocol::ROOT);
        let rendering = render_protocol(&dir).unwrap();
        if let Some(left_behind) = rendering.left_behind {
            panic!("{left_behind}");
        }
        assert!(rendering.outputs.len() > 3, "no version crate was rendered");
        for output in rendering.outputs {
            let path = format!("{}/{}", protocol::ROOT, output.path);
            let committed = fs::read_to_string(dir.join(&output.path));
            assert!(committed.is_ok(), "{path} is missing; run `cargo xtask`");
            // A manifest is written as rendered. A Rust file is written as rustfmt formats it,
            // which this test does not run: the two are compared as prettyplease prints them.
            let current = committed.is_ok_and(|committed| {
                if is_rust(&output.path) {
                    normalised(&committed) == normalised(&output.contents)
                } else {
                    committed == output.contents
                }
            });
            assert!(
                current,
                "{path} is not what `cargo xtask` generates; run it"
            );
        }
    }

    /// A `crates/protocol` with a definition of one version and nothing generated yet.
    fn new_protocol_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let file = |path: &str, text: &str| {
            let path = dir.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        file(
            protocol::DEFINITION,
            r#"versions![
                (1, "r/1", "1.0.0"): {
                    raknet_version: 10,
                    packets: [
                        + TextPacket: TextPacket,
                    ] in v1::packets,
                },
            ];"#,
        );
        file(
            protocol::FACADE_MANIFEST,
            "[dependencies]\n\
             # begin generated by `cargo xtask`: the version crates\n\
             # end generated: the version crates\n\
             [features]\n\
             # begin generated by `cargo xtask`: the version features\n\
             # end generated: the version features\n",
        );
        fs::create_dir_all(dir.path().join(protocol::VERSIONS)).unwrap();
        dir
    }

    /// A formatter that leaves the text as the generator rendered it. The tests run without a
    /// `rustfmt` binary.
    #[expect(clippy::unnecessary_wraps, reason = "the signature of a formatter")]
    fn unformatted(text: &str) -> io::Result<String> {
        Ok(text.to_owned())
    }

    /// Every file under `dir`, by its path below it.
    fn files_under(dir: &Path) -> protocol::Sources {
        fn walk(dir: &Path, prefix: &str, files: &mut protocol::Sources) {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
                if entry.path().is_dir() {
                    walk(&entry.path(), &format!("{name}/"), files);
                } else {
                    files.insert(name, fs::read_to_string(entry.path()).unwrap());
                }
            }
        }
        let mut files = protocol::Sources::new();
        walk(dir, "", &mut files);
        files
    }

    #[test]
    fn the_rust_files_are_formatted_before_they_are_compared_and_written() {
        let dir = new_protocol_dir();
        let marked = |text: &str| Ok(format!("// formatted\n{text}"));

        let written = write_protocol(dir.path(), marked).unwrap();
        for path in &written {
            let text = fs::read_to_string(dir.path().join(path)).unwrap();
            let is_rust = Path::new(path).extension().is_some_and(|e| e == "rs");
            assert_eq!(text.starts_with("// formatted\n"), is_rust, "{path}");
        }
        // The starting `lib.rs` of a new crate is formatted like the generated files.
        assert!(written.contains(&"version/v1/src/lib.rs".to_owned()));
        assert!(written.contains(&"version/v1/src/generated.rs".to_owned()));
        assert!(written.contains(&"version/v1/Cargo.toml".to_owned()));

        // What is on disk is compared with the formatted text: nothing is written again.
        assert!(write_protocol(dir.path(), marked).unwrap().is_empty());
    }

    #[test]
    fn a_formatter_that_fails_writes_nothing() {
        let dir = new_protocol_dir();
        let before = files_under(dir.path());
        let failing = |_: &str| Err(io::Error::other("no formatter"));

        let error = write_protocol(dir.path(), failing).unwrap_err();
        // The error says which file the formatter was given.
        let message = error.to_string();
        assert!(
            message.contains("version/traits/src/generated.rs"),
            "{message}"
        );
        assert!(message.contains("no formatter"), "{message}");
        assert_eq!(files_under(dir.path()), before);
    }

    #[test]
    fn a_rustfmt_that_cannot_be_run_says_how_to_install_it() {
        let error = format_with(OsStr::new("no-such-rustfmt-binary"), "fn main() {}").unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("rustup component add rustfmt"),
            "{message}"
        );
    }

    #[test]
    fn a_new_version_gets_a_crate_that_is_then_left_to_its_author() {
        let dir = new_protocol_dir();
        let v1 = dir.path().join(protocol::VERSIONS).join("v1");

        let written = write_protocol(dir.path(), unformatted).unwrap();
        assert!(written.contains(&"version/v1/src/lib.rs".to_owned()));
        assert!(v1.join("Cargo.toml").is_file());
        assert!(v1.join("src/generated.rs").is_file());
        let lib = fs::read_to_string(v1.join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod packets;"), "{lib}");

        // The author writes the module the `lib.rs` declares and makes the `lib.rs` their own.
        fs::create_dir_all(v1.join("src/packets")).unwrap();
        fs::write(v1.join("src/packets/mod.rs"), "use uuid::Uuid;").unwrap();
        fs::write(v1.join("src/lib.rs"), format!("// mine\n{lib}")).unwrap();

        // The manifest follows the sources; the `lib.rs` is not written again.
        assert_eq!(
            write_protocol(dir.path(), unformatted).unwrap(),
            ["version/v1/Cargo.toml"]
        );
        let manifest = fs::read_to_string(v1.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("uuid.workspace = true"), "{manifest}");

        // A tree that is what the generator renders is not written at all.
        assert!(write_protocol(dir.path(), unformatted).unwrap().is_empty());
        let lib = fs::read_to_string(v1.join("src/lib.rs")).unwrap();
        assert!(lib.starts_with("// mine\n"), "{lib}");
    }

    #[test]
    fn a_definition_error_writes_nothing() {
        let dir = new_protocol_dir();
        let definition = dir.path().join(protocol::DEFINITION);
        let text = fs::read_to_string(&definition).unwrap();
        fs::write(
            &definition,
            text.replace("in v1::packets", "in v2::packets"),
        )
        .unwrap();

        let error = write_protocol(dir.path(), unformatted).unwrap_err();
        assert!(error.to_string().contains("versions.def.rs:6:"), "{error}");
        assert!(!dir.path().join(protocol::VERSIONS).join("v1").exists());
        assert!(!dir.path().join("protocol/src").exists());
    }

    #[test]
    fn a_directory_that_is_no_version_is_reported_once_the_files_are_written() {
        // A crate left behind by a version that was removed, and what is left of one whose
        // files were deleted: a directory without a `src/` counts as well.
        for left_behind in ["v0/src", "v0"] {
            let dir = new_protocol_dir();
            let versions = dir.path().join(protocol::VERSIONS);
            fs::create_dir_all(versions.join(left_behind)).unwrap();

            let error = write_protocol(dir.path(), unformatted).unwrap_err();
            let message = error.to_string();
            assert!(
                message.starts_with("crates/protocol/version/v0: "),
                "{message}"
            );
            // The files are written all the same: the directory can only be deleted once no
            // generated manifest names it.
            assert!(versions.join("v1/Cargo.toml").is_file(), "{left_behind}");
            let facade = fs::read_to_string(dir.path().join(protocol::FACADE_MANIFEST)).unwrap();
            assert!(facade.contains("bedrock_protocol_v1 ="), "{facade}");

            // It is reported until it is gone, and nothing is left to write by then.
            assert!(write_protocol(dir.path(), unformatted).is_err());
            fs::remove_dir_all(versions.join("v0")).unwrap();
            assert!(write_protocol(dir.path(), unformatted).unwrap().is_empty());
        }

        // A file beside the crates is not a directory of a version.
        let dir = new_protocol_dir();
        fs::write(dir.path().join(protocol::VERSIONS).join("notes.md"), "").unwrap();
        assert!(write_protocol(dir.path(), unformatted).is_ok());
    }

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
