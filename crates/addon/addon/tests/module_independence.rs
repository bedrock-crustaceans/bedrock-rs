//! A family lifted into its own crate (`crates/addon/families/<name>/`, the package
//! `bedrock_addon_family_<name>`, which `families/mod.rs` re-exports) depends on no
//! `bedrock_addon` crate but core, the derive and the shared crates (and support's
//! `biome_json`, for the three biome families until it goes, and script for `server_config`,
//! the one family on a family): its manifest names no other, and its sources do not either.
//! The compiler enforces the rest (a family crate cannot name `crate::pack`, `crate::storage`,
//! another family or `bedrock_addon`'s `load`).
//!
//! The second rule: library code never reads the text of a finding's path.
//!
//! The tests read the crate's own source files, which library code may never do.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::fs;
use std::path::{Path, PathBuf};

/// The directories directly below `dir`, sorted by name; none when `dir` does not exist.
fn directories(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The directory of the lifted families: `crates/addon/families/`.
fn lifted_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../families")
}

/// Every family lifted into its own crate: a directory of `crates/addon/families/` with a
/// `Cargo.toml`, sorted by name; none when the crate is packaged alone.
fn lifted_families() -> &'static [String] {
    static LIFTED: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    LIFTED.get_or_init(|| {
        directories(&lifted_dir())
            .into_iter()
            .filter(|name| lifted_dir().join(name).join("Cargo.toml").is_file())
            .collect()
    })
}

/// The dependency names of a `Cargo.toml` (every `[*dependencies]` table, target-specific ones
/// included, and `[dependencies.<name>]` tables), each with its line.
fn manifest_dependencies(manifest: &str) -> Vec<(usize, String)> {
    let mut names = Vec::new();
    let mut in_dependencies = false;
    for (index, line) in manifest.lines().enumerate() {
        let line = line.trim();
        if let Some(table) = line.strip_prefix('[') {
            let table = table.trim_end_matches(']').trim();
            let (head, last) = table.rsplit_once('.').unwrap_or(("", table));
            if last.ends_with("dependencies") {
                in_dependencies = true;
            } else if head.ends_with("dependencies") {
                // `[dependencies.name]`
                in_dependencies = false;
                names.push((index + 1, last.trim_matches('"').to_owned()));
            } else {
                in_dependencies = false;
            }
            continue;
        }
        if in_dependencies && let Some((key, _)) = line.split_once('=') {
            let key = key.trim().trim_matches('"');
            if !key.is_empty() && !key.starts_with('#') {
                // `name.workspace = true` names `name`.
                names.push((index + 1, key.split('.').next().unwrap().to_owned()));
            }
        }
    }
    names
}

/// The support crate, which holds `biome_json` and nothing else.
const SUPPORT: &str = "bedrock_addon_support";

/// The families that still depend on [`SUPPORT`], for its `biome_json` alone: the digit-key
/// path builder of the biome documents, whose deletion is behaviour-bearing and not yet
/// landed. They may name `bedrock_addon_support::biome_json` and nothing else of it;
/// `support_allowance_is_biome_json_s` goes red when `biome_json` goes and this list stays.
const BIOME_JSON_ALLOWANCE: [&str; 3] = ["biome", "biomes_client", "client_biome"];

/// The one family that may depend on another family: `server_config` on script. The server's
/// configuration is held against script modules: its dependency filter
/// (`ScriptConfig::filter`) takes the plugin discovery made, its unmatched-folder rule takes
/// the modules' identities, its permission list goes into the gates of a discovery
/// (`ScriptConfig::context`), and its near-miss note asks script's module table whether a name
/// is a module. Script names nothing of it, so the edge runs one way; a shared crate holding
/// the four would be a script crate of its own (the module table alone is two thousand lines).
const SCRIPT_ALLOWANCE: (&str, &str) = ("server_config", "bedrock_addon_family_script");

/// What `server_config`'s library may name of script: the module table (`meta::module`), the
/// gates of a discovery, a module's identity and a plugin.
const SCRIPT_ITEMS: [&str; 4] = ["meta", "ScriptContext", "ScriptModuleRef", "ScriptPlugin"];

/// What `server_config`'s tests may name of script besides [`SCRIPT_ITEMS`]: the parts of a
/// plugin built by hand.
const SCRIPT_TEST_ITEMS: [&str; 2] = ["ResolvedDependency", "ScriptVersion"];

/// Whether the lifted family `family` may depend on the `bedrock_addon*` crate `name`: itself,
/// core, the derive (six families derive their component tables with it), the shared crates,
/// support for the families of [`BIOME_JSON_ALLOWANCE`], and script for `server_config`
/// ([`SCRIPT_ALLOWANCE`]).
fn family_may_name(family: &str, name: &str) -> bool {
    name == format!("bedrock_addon_family_{family}")
        || shared_may_name(name)
        || (name == SUPPORT && BIOME_JSON_ALLOWANCE.contains(&family))
        || (family, name) == SCRIPT_ALLOWANCE
}

/// The items a path names below its crate: the first segment after the crate's `::`
/// (`meta` of `meta::module`), or each first segment of a group (`{A, b::C}`).
fn items_named(path: &str) -> Vec<&str> {
    fn first(segment: &str) -> &str {
        let segment = segment.trim();
        let end = segment
            .find(|c: char| !is_ident(c))
            .unwrap_or(segment.len());
        &segment[..end]
    }
    let Some(group) = path.trim_start().strip_prefix('{') else {
        return vec![first(path)];
    };
    let mut items = Vec::new();
    let (mut depth, mut start) = (0_usize, 0);
    for (at, c) in group.char_indices() {
        match c {
            '{' => depth += 1,
            '}' if depth == 0 => {
                items.push(first(&group[start..at]));
                break;
            }
            '}' => depth -= 1,
            ',' if depth == 0 => {
                items.push(first(&group[start..at]));
                start = at + 1;
            }
            _ => {}
        }
    }
    items.retain(|item| !item.is_empty());
    items
}

/// What a lifted family crate depends on or names that it may not: any `bedrock_addon*` crate
/// but itself, core, the derive and the shared crates (and `biome_json` of support for the
/// families of [`BIOME_JSON_ALLOWANCE`], and the [`SCRIPT_ITEMS`] of script for
/// `server_config`, its tests also the [`SCRIPT_TEST_ITEMS`]), in its manifest (`manifest`) or its sources
/// (`sources`, each with its path below the crate's `src/`).
fn lifted_violations(family: &str, manifest: &str, sources: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (line, name) in manifest_dependencies(manifest) {
        if name.starts_with("bedrock_addon") && !family_may_name(family, &name) {
            violations.push(format!("{family}/Cargo.toml:{line} depends on {name}"));
        }
    }
    for (relative, source) in sources {
        let code = code_of(source).replace("r#", "");
        for (index, _) in code.match_indices("bedrock_addon") {
            if code[..index].chars().next_back().is_some_and(is_ident) {
                continue;
            }
            let rest = &code[index + "bedrock_addon".len()..];
            let end = rest.find(|c: char| !is_ident(c)).unwrap_or(rest.len());
            let name = format!("bedrock_addon{}", &rest[..end]);
            let Some(path) = rest[end..].trim_start().strip_prefix("::") else {
                continue;
            };
            let biome_json = path.trim_start().starts_with("biome_json")
                && !path.trim_start()["biome_json".len()..]
                    .chars()
                    .next()
                    .is_some_and(is_ident);
            let allowed = family_may_name(family, &name) && (name != SUPPORT || biome_json);
            let line = code[..index].matches('\n').count() + 1;
            if !allowed {
                violations.push(format!("{family}/src/{relative}:{line} names {name}::"));
            } else if (family, name.as_str()) == SCRIPT_ALLOWANCE {
                let tests = is_test_file(Path::new(relative));
                for item in items_named(path) {
                    let allowed = SCRIPT_ITEMS.contains(&item)
                        || (tests && SCRIPT_TEST_ITEMS.contains(&item));
                    if !allowed {
                        violations.push(format!(
                            "{family}/src/{relative}:{line} names {name}::{item}"
                        ));
                    }
                }
            }
        }
    }
    violations
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Whether `c` can be part of an identifier.
fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `source` with every comment, string literal and character literal blanked (each byte of
/// one replaced by a space, newlines kept), so the scans read code only and offsets stay.
fn code_of(source: &str) -> String {
    blanked(source, true)
}

/// `source` with every comment blanked, and with `literals` every string and character
/// literal too (each byte replaced by a space, newlines kept).
fn blanked(source: &str, literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let blank = |out: &mut Vec<u8>, from: usize, to: usize| {
        for byte in &mut out[from..to] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    };
    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        let comment = rest.starts_with(b"//") || rest.starts_with(b"/*");
        let end = if rest.starts_with(b"//") {
            i + rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len())
        } else if rest.starts_with(b"/*") {
            let mut depth = 0_usize;
            let mut j = i;
            loop {
                if bytes[j..].starts_with(b"/*") {
                    depth += 1;
                    j += 2;
                } else if bytes[j..].starts_with(b"*/") {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break j;
                    }
                } else if j >= bytes.len() {
                    break bytes.len();
                } else {
                    j += 1;
                }
            }
        } else if rest[0] == b'"' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'"' {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            (j + 1).min(bytes.len())
        } else if rest[0] == b'r'
            && (i == 0 || !is_ident(char::from(bytes[i - 1])))
            && raw_string_hashes(rest).is_some()
        {
            let hashes = raw_string_hashes(rest).unwrap_or_default();
            let close: Vec<u8> = std::iter::once(b'"')
                .chain(std::iter::repeat_n(b'#', hashes))
                .collect();
            let body = i + 2 + hashes;
            bytes[body..]
                .windows(close.len())
                .position(|window| window == close)
                .map_or(bytes.len(), |at| body + at + close.len())
        } else if rest[0] == b'\'' {
            // A character literal (`'x'`, `'\n'`, `'\u{1F600}'`); anything else is a lifetime.
            let close = if rest.get(1) == Some(&b'\\') {
                rest[2..].iter().position(|&b| b == b'\'').map(|at| at + 3)
            } else {
                let width = source[i + 1..].chars().next().map_or(1, char::len_utf8);
                (rest.get(1 + width) == Some(&b'\'')).then_some(width + 2)
            };
            if let Some(len) = close {
                i + len
            } else {
                i += 1;
                continue;
            }
        } else {
            i += 1;
            continue;
        };
        if comment || literals {
            blank(&mut out, i, end);
        }
        i = end;
    }
    String::from_utf8(out).expect("blanking keeps the text UTF-8 outside the blanked spans")
}

/// The number of `#` of a raw string literal starting at `rest` (`r"…"`, `r#"…"#`), if one
/// starts there.
fn raw_string_hashes(rest: &[u8]) -> Option<usize> {
    let hashes = rest[1..].iter().take_while(|&&b| b == b'#').count();
    (rest.get(1 + hashes) == Some(&b'"')).then_some(hashes)
}

/// A family in its own crate is held to the rule across crates.
#[test]
fn lifted_families_depend_on_no_other_family_and_not_on_bedrock_addon() {
    let mut violations = Vec::new();
    for family in lifted_families() {
        let dir = lifted_dir().join(family);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        let package = format!("name = \"bedrock_addon_family_{family}\"");
        assert!(
            manifest.lines().any(|line| line.trim() == package),
            "{family}/Cargo.toml is not the package bedrock_addon_family_{family}"
        );
        assert!(
            !src_families_dir().join(family).exists(),
            "{family} is both a crate and a directory of src/families/"
        );
        let mut files = Vec::new();
        rust_files(&dir.join("src"), &mut files);
        assert!(!files.is_empty(), "{family} has no source files");
        let sources: Vec<(String, String)> = files
            .iter()
            .map(|file| {
                let relative = file.strip_prefix(dir.join("src")).unwrap();
                (
                    relative.display().to_string(),
                    fs::read_to_string(file).unwrap(),
                )
            })
            .collect();
        violations.extend(lifted_violations(family, &manifest, &sources));
    }
    assert!(
        violations.is_empty(),
        "a lifted family depends on bedrock_addon_core, bedrock_addon_derive and the \
         bedrock_addon_shared_* crates only (and biome_json of bedrock_addon_support for the \
         biome_json allowance, and SCRIPT_ITEMS of bedrock_addon_family_script for \
         server_config):\n{}",
        violations.join("\n")
    );
}

/// `src/families/`.
fn src_families_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/families")
}

#[test]
fn the_lifted_scan_reads_manifests_and_paths() {
    let manifest = "[package]\nname = \"bedrock_addon_family_fog\"\n\n[dependencies]\n\
        bedrock_addon_core = { path = \"../../core\" }\nbedrock_addon = { path = \"x\" }\n\
        [target.'cfg(unix)'.dependencies]\nbedrock_addon_family_lang.workspace = true\n\
        [dependencies.bedrock_addon_family_block]\npath = \"../block\"\n[features]\n\
        bedrock_addon = []\n[dev-dependencies]\nbedrock_addon_support = { path = \"y\" }\n\
        bedrock_addon_shared_shapes = { path = \"z\" }\nbedrock_addon_derive = { path = \"w\" }\n";
    let sources = [(
        "lib.rs".to_owned(),
        "use bedrock_addon_core::id::Id; use bedrock_addon::pack::X;\n\
         // bedrock_addon::pack in a comment\nuse bedrock_addon_family_fog::A;\n\
         fn f() { bedrock_addon_family_item::Item::new(); }\n\
         fn g() { bedrock_addon_support::x(); bedrock_addon_shared_scatter::y(); bedrock_addon_derive::z(); }"
            .to_owned(),
    )];
    assert_eq!(
        lifted_violations("fog", manifest, &sources),
        [
            "fog/Cargo.toml:6 depends on bedrock_addon",
            "fog/Cargo.toml:8 depends on bedrock_addon_family_lang",
            "fog/Cargo.toml:9 depends on bedrock_addon_family_block",
            "fog/Cargo.toml:14 depends on bedrock_addon_support",
            "fog/src/lib.rs:1 names bedrock_addon::",
            "fog/src/lib.rs:4 names bedrock_addon_family_item::",
            "fog/src/lib.rs:5 names bedrock_addon_support::",
        ]
    );
    // A family of the biome_json allowance may depend on support and name its biome_json, and
    // nothing else of it.
    let manifest = "[package]\nname = \"bedrock_addon_family_biome\"\n\n[dependencies]\n\
        bedrock_addon_support = { path = \"../../support\" }\n";
    let sources = [(
        "lib.rs".to_owned(),
        "use bedrock_addon_support::biome_json::child;\nuse bedrock_addon_support::jsoncpp::read;\n"
            .to_owned(),
    )];
    assert_eq!(
        lifted_violations("biome", manifest, &sources),
        ["biome/src/lib.rs:2 names bedrock_addon_support::"]
    );
    // `server_config` may depend on script and name the items of `SCRIPT_ITEMS`, its tests
    // also those of `SCRIPT_TEST_ITEMS`: nothing else of script, and no other family.
    let manifest = "[package]\nname = \"bedrock_addon_family_server_config\"\n\n[dependencies]\n\
        bedrock_addon_family_script = { path = \"../script\" }\n\
        bedrock_addon_family_lang = { path = \"../lang\" }\n";
    let sources = [
        (
            "lib.rs".to_owned(),
            "use bedrock_addon_family_script::meta;\n\
             use bedrock_addon_family_script::{ScriptContext, ScriptModuleRef, ScriptPlugin};\n\
             fn f() -> bedrock_addon_family_script::ScriptBundle { todo!() }\n\
             use bedrock_addon_family_script::{ScriptPlugin, meta::module, ScriptVersion};\n\
             use bedrock_addon_family_script::bundle::{ScriptBundle as B};\n"
                .to_owned(),
        ),
        (
            "tests.rs".to_owned(),
            "use bedrock_addon_family_script::{ResolvedDependency, ScriptPlugin, ScriptVersion};\n\
             fn g() { bedrock_addon_family_script::ScriptBundle::default(); }\n"
                .to_owned(),
        ),
    ];
    assert_eq!(
        lifted_violations("server_config", manifest, &sources),
        [
            "server_config/Cargo.toml:6 depends on bedrock_addon_family_lang",
            "server_config/src/lib.rs:3 names bedrock_addon_family_script::ScriptBundle",
            "server_config/src/lib.rs:4 names bedrock_addon_family_script::ScriptVersion",
            "server_config/src/lib.rs:5 names bedrock_addon_family_script::bundle",
            "server_config/src/tests.rs:2 names bedrock_addon_family_script::ScriptBundle",
        ]
    );
    // The allowance is server_config's alone.
    let manifest = "[package]\nname = \"bedrock_addon_family_lang\"\n\n[dependencies]\n\
        bedrock_addon_family_script = { path = \"../script\" }\n";
    let sources = [(
        "lib.rs".to_owned(),
        "use bedrock_addon_family_script::ScriptPlugin;\n".to_owned(),
    )];
    assert_eq!(
        lifted_violations("lang", manifest, &sources),
        [
            "lang/Cargo.toml:5 depends on bedrock_addon_family_script",
            "lang/src/lib.rs:1 names bedrock_addon_family_script::",
        ]
    );
}

/// `server_config` depends on script and holds the server's configuration, which script's
/// library no longer names: the day `server_config` stops naming script, the allowance goes
/// with it.
#[test]
fn script_allowance_is_server_config_s() {
    if !lifted_dir().is_dir() {
        // A crate packaged alone has no sibling to scan.
        return;
    }
    let (family, script) = SCRIPT_ALLOWANCE;
    assert!(
        lifted_families().iter().any(|name| name == family),
        "{family} is no lifted family: drop SCRIPT_ALLOWANCE"
    );
    let dir = lifted_dir().join(family);
    let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(
        manifest_dependencies(&manifest)
            .iter()
            .any(|(_, name)| name == script),
        "{family} no longer depends on {script}: drop SCRIPT_ALLOWANCE"
    );
    let mut files = Vec::new();
    rust_files(&dir.join("src"), &mut files);
    assert!(
        files.iter().any(
            |file| code_of(&fs::read_to_string(file).unwrap()).contains(&format!("{script}::"))
        ),
        "{family} no longer names {script}: drop SCRIPT_ALLOWANCE"
    );
    // The edge runs one way: script names nothing of the configuration.
    assert!(
        !lifted_dir().join("script/src/config").exists(),
        "script/src/config is back: the server's configuration is the {family} family's"
    );
    let manifest = fs::read_to_string(lifted_dir().join("script/Cargo.toml")).unwrap();
    assert!(
        !manifest_dependencies(&manifest)
            .iter()
            .any(|(_, name)| name == &format!("bedrock_addon_family_{family}")),
        "script depends on {family}"
    );
    let mut files = Vec::new();
    rust_files(&lifted_dir().join("script/src"), &mut files);
    for file in files {
        let code = code_of(&fs::read_to_string(&file).unwrap());
        assert!(
            !code.contains("config::") && !code.contains("ScriptConfig"),
            "{} names the server's configuration",
            file.display()
        );
    }
}

/// The families whose shared bodies moved out of `bedrock_addon_support` into core or a
/// `shared/*` crate: they depend on core and the shared crates, and on support no more. Every
/// family but those of [`BIOME_JSON_ALLOWANCE`] that ever depended on support, and `lang`, which
/// never did and is kept here so it cannot start to.
const ON_SHARED_CRATES: [&str; 31] = [
    "aim_assist",
    "animation",
    "animation_controller",
    "camera",
    "credits",
    "dimension",
    "entity",
    "feature",
    "feature_rules",
    "flipbooks",
    "fog",
    "font",
    "geometry",
    "item",
    "json_ui",
    "lang",
    "legacy_path_mapping",
    "loading_messages",
    "music",
    "render_controller",
    "sound_definitions",
    "sounds_json",
    "spawn_groups",
    "spawn_rules",
    "splashes",
    "texture_atlas",
    "texture_file",
    "texture_list",
    "texture_set",
    "trade",
    "ui_texture_info",
];

/// Where a family of [`ON_SHARED_CRATES`] depends on or names `bedrock_addon_support`, in its
/// manifest (`manifest`) or its sources (`sources`, each with its path below the crate's `src/`).
fn support_violations(family: &str, manifest: &str, sources: &[(String, String)]) -> Vec<String> {
    const SUPPORT: &str = "bedrock_addon_support";
    let mut violations = Vec::new();
    for (line, name) in manifest_dependencies(manifest) {
        if name == SUPPORT {
            violations.push(format!("{family}/Cargo.toml:{line} depends on {name}"));
        }
    }
    for (relative, source) in sources {
        let code = code_of(source).replace("r#", "");
        for (index, _) in code.match_indices(SUPPORT) {
            let before = code[..index].chars().next_back().is_some_and(is_ident);
            let rest = &code[index + SUPPORT.len()..];
            let after = rest.chars().next().is_some_and(is_ident);
            if !before && !after && rest.trim_start().starts_with("::") {
                let line = code[..index].matches('\n').count() + 1;
                violations.push(format!("{family}/src/{relative}:{line} names {SUPPORT}::"));
            }
        }
    }
    violations
}

/// The families of [`ON_SHARED_CRATES`] read their shared bodies from the shared crates.
#[test]
fn families_on_shared_crates_do_not_depend_on_support() {
    let mut violations = Vec::new();
    for family in ON_SHARED_CRATES {
        let dir = lifted_dir().join(family);
        if !dir.is_dir() {
            // A crate packaged alone has no sibling to scan.
            continue;
        }
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        let mut files = Vec::new();
        rust_files(&dir.join("src"), &mut files);
        let sources: Vec<(String, String)> = files
            .iter()
            .map(|file| {
                let relative = file.strip_prefix(dir.join("src")).unwrap();
                (
                    relative.display().to_string(),
                    fs::read_to_string(file).unwrap(),
                )
            })
            .collect();
        violations.extend(support_violations(family, &manifest, &sources));
    }
    assert!(
        violations.is_empty(),
        "these families depend on bedrock_addon_core and the shared crates, not on \
         bedrock_addon_support:\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_support_scan_reads_manifests_and_paths() {
    let manifest = "[package]\nname = \"bedrock_addon_family_spawn_rules\"\n\n[dependencies]\n\
        bedrock_addon_core = { path = \"../../core\" }\nbedrock_addon_support = { path = \"x\" }\n\
        bedrock_addon_shared_spawn = { path = \"y\" }\n";
    let sources = [(
        "lib.rs".to_owned(),
        "use bedrock_addon_shared_spawn::Condition;\n\
         // bedrock_addon_support::spawn in a comment\nuse bedrock_addon_support::spawn::X;\n\
         fn f() { bedrock_addon_support_more::x(); }"
            .to_owned(),
    )];
    assert_eq!(
        support_violations("spawn_rules", manifest, &sources),
        [
            "spawn_rules/Cargo.toml:6 depends on bedrock_addon_support",
            "spawn_rules/src/lib.rs:3 names bedrock_addon_support::",
        ]
    );
}

/// The support crate holds `biome_json` for the families of [`BIOME_JSON_ALLOWANCE`], and each
/// of them still uses it: the day `biome_json` goes, or a family stops naming it, the
/// allowance goes with it.
#[test]
fn support_allowance_is_biome_json_s() {
    if !lifted_dir().is_dir() {
        // A crate packaged alone has no sibling to scan.
        return;
    }
    // The support crate going whole is the likely way `biome_json` goes: that is red too.
    let support = Path::new(env!("CARGO_MANIFEST_DIR")).join("../support/src");
    assert!(
        support.join("biome_json.rs").is_file(),
        "support's biome_json is gone: drop BIOME_JSON_ALLOWANCE and support with it"
    );
    for family in BIOME_JSON_ALLOWANCE {
        let dir = lifted_dir().join(family);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(
            manifest_dependencies(&manifest)
                .iter()
                .any(|(_, name)| name == SUPPORT),
            "{family} no longer depends on {SUPPORT}: drop it from BIOME_JSON_ALLOWANCE"
        );
        let mut files = Vec::new();
        rust_files(&dir.join("src"), &mut files);
        assert!(
            files
                .iter()
                .any(|file| code_of(&fs::read_to_string(file).unwrap())
                    .contains("bedrock_addon_support::biome_json")),
            "{family} no longer names {SUPPORT}::biome_json: drop it from BIOME_JSON_ALLOWANCE"
        );
    }
}

/// The directory of the shared crates: `crates/addon/shared/`.
fn shared_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared")
}

/// Whether a shared crate may name the `bedrock_addon*` crate `name`: core, the derive (its
/// tests derive) and another shared crate, never a family, `bedrock_addon`, the support crate
/// or the std driver.
fn shared_may_name(name: &str) -> bool {
    name == "bedrock_addon_core"
        || name == "bedrock_addon_derive"
        || name.starts_with("bedrock_addon_shared_")
}

/// What a shared crate (`crates/addon/shared/<name>/`) depends on or names that it may not:
/// a `bedrock_addon*` crate other than core, the derive and the shared crates, in its manifest
/// (`manifest`) or its sources (`sources`, each with its path below the crate's `src/`).
fn shared_violations(name: &str, manifest: &str, sources: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (line, dependency) in manifest_dependencies(manifest) {
        if dependency.starts_with("bedrock_addon") && !shared_may_name(&dependency) {
            violations.push(format!("{name}/Cargo.toml:{line} depends on {dependency}"));
        }
    }
    for (relative, source) in sources {
        let code = code_of(source).replace("r#", "");
        for (index, _) in code.match_indices("bedrock_addon") {
            if code[..index].chars().next_back().is_some_and(is_ident) {
                continue;
            }
            let rest = &code[index + "bedrock_addon".len()..];
            let end = rest.find(|c: char| !is_ident(c)).unwrap_or(rest.len());
            let named = format!("bedrock_addon{}", &rest[..end]);
            if rest[end..].trim_start().starts_with("::") && !shared_may_name(&named) {
                let line = code[..index].matches('\n').count() + 1;
                violations.push(format!("{name}/src/{relative}:{line} names {named}::"));
            }
        }
    }
    violations
}

/// A shared crate holds the shapes the families compose, so it sits below every family: it
/// depends on core (and the other shared crates) only.
#[test]
fn shared_crates_depend_on_core_and_no_family() {
    let crates: Vec<String> = directories(&shared_dir())
        .into_iter()
        .filter(|name| shared_dir().join(name).join("Cargo.toml").is_file())
        .collect();
    assert!(
        !crates.is_empty(),
        "no crate below crates/addon/shared/ (the families' shared shapes live there)"
    );
    let mut violations = Vec::new();
    for name in &crates {
        let dir = shared_dir().join(name);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        let package = format!("name = \"bedrock_addon_shared_{name}\"");
        assert!(
            manifest.lines().any(|line| line.trim() == package),
            "shared/{name}/Cargo.toml is not the package bedrock_addon_shared_{name}"
        );
        let mut files = Vec::new();
        rust_files(&dir.join("src"), &mut files);
        let sources: Vec<(String, String)> = files
            .iter()
            .map(|file| {
                let relative = file.strip_prefix(dir.join("src")).unwrap();
                (
                    relative.display().to_string(),
                    fs::read_to_string(file).unwrap(),
                )
            })
            .collect();
        violations.extend(shared_violations(name, &manifest, &sources));
    }
    assert!(
        violations.is_empty(),
        "a shared crate depends on bedrock_addon_core and the other shared crates only:\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_shared_scan_reads_manifests_and_paths() {
    let manifest = "[package]\nname = \"bedrock_addon_shared_shapes\"\n\n[dependencies]\n\
        bedrock_addon_core = { path = \"../../core\" }\nbedrock_addon_family_block = { path = \"x\" }\n\
        bedrock_addon_shared_visual = { path = \"../visual\" }\nbedrock_addon_support.workspace = true\n\
        [dev-dependencies]\nbedrock_addon_derive = { path = \"../../derive\" }\nbedrock_addon = { path = \"y\" }\n";
    let sources = [(
        "lib.rs".to_owned(),
        "use bedrock_addon_core::id::Id; use bedrock_addon_shared_visual::A;\n\
         // bedrock_addon_family_item::Item in a comment\nuse bedrock_addon_family_fog::A;\n\
         fn f() { bedrock_addon_support::x(); }"
            .to_owned(),
    )];
    assert_eq!(
        shared_violations("shapes", manifest, &sources),
        [
            "shapes/Cargo.toml:6 depends on bedrock_addon_family_block",
            "shapes/Cargo.toml:8 depends on bedrock_addon_support",
            "shapes/Cargo.toml:11 depends on bedrock_addon",
            "shapes/src/lib.rs:3 names bedrock_addon_family_fog::",
            "shapes/src/lib.rs:4 names bedrock_addon_support::",
        ]
    );
}

#[test]
fn comments_and_literals_are_not_code() {
    let source = "// use crate::block::A;\n/* crate::item /* nested */ crate::pack */\nlet s = \"crate::entity\"; let r = r#\"crate::loot \"quoted\"\"#; let c = '\"'; let q = '\\''; fn f<'a>(x: &'a str) { crate::id::X }";
    let code = code_of(source);
    assert_eq!(code.len(), source.len(), "offsets are kept");
    assert_eq!(code.matches('\n').count(), source.matches('\n').count());
    assert_eq!(code.matches("crate::").count(), 1, "{code}");
    assert!(code.contains("crate::id::X"), "{code}");
    // An escaped quote does not end a string, and a lifetime opens no literal.
    let escaped = code_of("let s = \"a \\\" crate::block\"; crate::json::V");
    assert_eq!(escaped.matches("crate::").count(), 1, "{escaped}");
    assert!(escaped.contains("crate::json::V"), "{escaped}");
    let lifetime = code_of("impl<'a> T<'a> { fn f() { crate::block::B } }");
    assert!(lifetime.contains("crate::block::B"), "{lifetime}");
}

/// A family is the crate `crates/addon/families/<name>/`, never a module of `src/`.
#[test]
fn no_family_remains_a_module_of_src() {
    let dir = src_families_dir();
    assert!(
        dir.join("mod.rs").is_file(),
        "{} has no mod.rs",
        dir.display()
    );
    let directories = directories(&dir);
    assert!(
        directories.is_empty(),
        "a family is the crate crates/addon/families/<name>/, never a module of src/: \
         src/families/ holds the directories {directories:?}"
    );
    let lifted = lifted_families();
    assert!(lifted.iter().any(|name| name == "block"), "{lifted:?}");
    for not_a_family in ["load", "pack", "storage", "families", "support"] {
        assert!(
            !lifted.iter().any(|name| name == not_a_family),
            "{not_a_family} in {lifted:?}"
        );
    }
}

/// The library part of a source file: `source` without its comments and without the items
/// of its tests, each blanked (newlines kept, so a line of the result is a line of the file).
/// An item of the tests is what a `#[cfg(test)]` stands before (a module, a function, a
/// `use`, a constant, a field), wherever in the file; `#![cfg(test)]` makes the rest one.
fn library_part(source: &str) -> String {
    let code = code_of(source);
    let mut out = blanked(source, false).into_bytes();
    for item in test_items(&code) {
        for byte in &mut out[item] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(out).expect("an item starts and ends at an ASCII character")
}

/// The spans of the items of `code` (comments and literals blanked) behind a `#[cfg(test)]`
/// or a `#[cfg(all(test, …))]`, the attribute included.
fn test_items(code: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = code.as_bytes();
    let mut items = Vec::new();
    let mut at = 0;
    while let Some(found) = code[at..].find('#') {
        let start = at + found;
        at = start + 1;
        let Some((inner, after)) = attribute_at(code, start) else {
            continue;
        };
        let is_inner = code[start + 1..].trim_start().starts_with('!');
        if inner != "cfg(test)" && !inner.starts_with("cfg(all(test,") {
            continue;
        }
        let end = if is_inner {
            bytes.len()
        } else {
            item_end(bytes, after)
        };
        items.push(start..end);
        at = end.max(at);
    }
    items
}

/// The attribute that starts with the `#` at `start`: what stands between its brackets,
/// without white space, and the offset after its closing bracket.
fn attribute_at(code: &str, start: usize) -> Option<(String, usize)> {
    let rest = code[start + 1..].trim_start();
    let rest = rest.strip_prefix('!').map_or(rest, str::trim_start);
    let body = rest.strip_prefix('[')?;
    let open = code.len() - body.len();
    let mut depth = 1_usize;
    for (offset, byte) in body.bytes().enumerate() {
        match byte {
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            let inner = body[..offset].split_whitespace().collect();
            return Some((inner, open + offset + 1));
        }
    }
    None
}

/// Where the item that starts at `from` ends: after the block of its first brace, or after
/// the `;` or `,` that ends it where it has none (a `use`, a constant, a field, a variant),
/// or where what it stands in ends. `()`, `[]` and the `<>` of generics are stepped over; a
/// `<` after the `=` of a value is an operator, and a `<` left open (a comparison in a
/// condition) ends at the bracket, the `;` or the block that follows it.
fn item_end(bytes: &[u8], from: usize) -> usize {
    // The `(`, `[` and `<` the scan is in.
    let mut open: Vec<u8> = Vec::new();
    // After the `=` of a constant or a field's value: `<` and `>` compare or shift.
    let mut value = false;
    // After `where`: a `,` sets two bounds apart and ends nothing.
    let mut bounds = false;
    let mut at = from;
    while at < bytes.len() {
        let before = at.checked_sub(1).map(|at| bytes[at]);
        let after = bytes.get(at + 1).copied();
        let only_angles = open.iter().all(|bracket| *bracket == b'<');
        match bytes[at] {
            b'(' | b'[' => open.push(bytes[at]),
            b'<' if !value => open.push(b'<'),
            b'>' if open.last() == Some(&b'<') && !matches!(before, Some(b'-' | b'=')) => {
                open.pop();
            }
            b')' | b']' if only_angles => return at,
            b')' | b']' => {
                // The bracket closes, and with it every `<` left open in it.
                while open.pop() == Some(b'<') {}
            }
            b'}' => return at,
            b'=' if open.is_empty()
                && !matches!(after, Some(b'=' | b'>'))
                && !matches!(before, Some(b'=' | b'!' | b'<' | b'>')) =>
            {
                value = true;
            }
            b';' if only_angles => return at + 1,
            b',' if open.is_empty() && !bounds => return at + 1,
            b'w' if open.is_empty()
                && bytes[at..].starts_with(b"where")
                && !before.is_some_and(|byte| is_ident(char::from(byte)))
                && !bytes
                    .get(at + 5)
                    .is_some_and(|byte| is_ident(char::from(*byte))) =>
            {
                bounds = true;
            }
            b'{' if only_angles => {
                let mut depth = 0_usize;
                for (offset, byte) in bytes[at..].iter().enumerate() {
                    match byte {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 {
                        return at + offset + 1;
                    }
                }
                return bytes.len();
            }
            _ => {}
        }
        at += 1;
    }
    bytes.len()
}

/// Whether `relative` is a file of tests or of a test harness, not of the library.
fn is_test_file(relative: &Path) -> bool {
    relative.components().any(|part| {
        let part = part.as_os_str().to_string_lossy();
        part == "tests" || part == "tests.rs" || part.starts_with("tests_")
    })
}

/// Whether the module of `file` is declared by its parent behind `#[cfg(test)]`
/// (`#[cfg(test)] mod scalar_tests;` in `lib.rs`): a file the crate compiles for its tests
/// alone, whatever its name.
fn declared_for_tests(file: &Path) -> bool {
    let (Some(dir), Some(stem)) = (file.parent(), file.file_stem()) else {
        return false;
    };
    let stem = stem.to_string_lossy();
    let (module, parents) = if matches!(&*stem, "lib" | "main" | "mod") {
        // A crate root, or a directory's module declared by the file beside the directory.
        let Some(name) = dir.file_name() else {
            return false;
        };
        let up = dir.parent().map(Path::to_path_buf).unwrap_or_default();
        (
            name.to_string_lossy().into_owned(),
            vec![
                up.join("lib.rs"),
                up.join("mod.rs"),
                dir.with_extension("rs"),
            ],
        )
    } else {
        (
            stem.into_owned(),
            vec![
                dir.join("lib.rs"),
                dir.join("main.rs"),
                dir.join("mod.rs"),
                dir.with_extension("rs"),
            ],
        )
    };
    let declaration = format!("mod {module};");
    parents
        .iter()
        .filter(|parent| parent.as_path() != file)
        .any(|parent| {
            let Ok(source) = fs::read_to_string(parent) else {
                return false;
            };
            let code = code_of(&source);
            test_items(&code).into_iter().any(|item| {
                let item: Vec<&str> = code[item].split_whitespace().collect();
                item.join(" ").ends_with(&declaration)
            })
        })
}

/// The name every finding gives its path.
const JSON_PATH: &str = "json_path";

/// Where `text` (a source without its comments) takes the text of a path, as 1-based line
/// numbers, in order, each line once. A path is `json_path` anywhere, and in a file that
/// declares a name as a `JsonPath` (`at: JsonPath,`) that name too. Its text is taken by:
/// `.to_string()` on it, with any white space and after any `.clone()`; `{name}` or
/// `{name:…}` in a string literal; the path (or a clone, or a reference) as an argument
/// after the text of a macro that writes its arguments out (`format!("{}", d.json_path)`,
/// `assert!(ok, "{}", d.json_path)`, `log::warn!("{}", d.json_path)`); the path as the
/// argument of `ToString::to_string`; and `ToString::to_string` passed on in a statement
/// that names a path (`.map(|f| &f.json_path).map(ToString::to_string)`).
fn path_text_reads(text: &str) -> Vec<usize> {
    const PASSED: &str = "ToString::to_string";
    let code = code_of(text);
    let line_of = |at: usize| text[..at].matches('\n').count() + 1;
    let mut names = declared_paths(&code);
    names.push(JSON_PATH);
    names.sort_unstable();
    names.dedup();
    let macros = macro_arguments(&code);
    let mut found = Vec::new();
    for name in &names {
        let mut from = 0;
        while let Some(at) = text[from..].find(name) {
            let start = from + at;
            let end = start + name.len();
            from = end;
            if code.as_bytes()[start] == b' ' {
                // In a string literal: the name between the braces of a format.
                let opened = text[..start].ends_with('{') && !text[..start].ends_with("{{");
                if opened && matches!(text.as_bytes().get(end), Some(b'}' | b':')) {
                    found.push(line_of(start));
                }
                continue;
            }
            if code[..start].ends_with(is_ident) || code[end..].starts_with(is_ident) {
                continue;
            }
            if takes_text(text, &code, start, end, &macros) {
                found.push(line_of(start));
            }
        }
    }
    // `ToString::to_string` passed on, in a statement that names a path.
    let mut from = 0;
    while let Some(at) = code[from..].find(PASSED) {
        let start = from + at;
        from = start + PASSED.len();
        if code[from..].trim_start().starts_with('(') {
            continue;
        }
        let statement = &code[code[..start].rfind(';').map_or(0, |end| end + 1)..start];
        if names.iter().any(|name| has_word(statement, name)) {
            found.push(line_of(start));
        }
    }
    found.sort_unstable();
    found.dedup();
    found
}

/// Whether `code` holds `word` as a whole identifier.
fn has_word(code: &str, word: &str) -> bool {
    code.match_indices(word).any(|(at, _)| {
        !code[..at].ends_with(is_ident) && !code[at + word.len()..].starts_with(is_ident)
    })
}

/// Whether the path named at `start..end` of `code` has its text taken there (the ways of
/// [`path_text_reads`] that are code, not the inside of a string literal).
fn takes_text(
    text: &str,
    code: &str,
    start: usize,
    end: usize,
    macros: &[std::ops::Range<usize>],
) -> bool {
    // What follows the name, without white space and without the clones of it.
    let mut rest: String = code[end..]
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(256)
        .collect();
    while let Some(after) = rest.strip_prefix(".clone()") {
        rest = after.to_owned();
    }
    if rest.starts_with(".to_string()") {
        return true;
    }
    if !rest.starts_with([',', ')']) {
        return false;
    }
    // An argument after the text of a macro that writes its arguments out: a string
    // literal with a `{` that is no `{{` stands before it among the macro's arguments.
    let placeholder = |at: usize| {
        let bytes = text.as_bytes();
        bytes[at] == b'{'
            && code.as_bytes()[at] == b' '
            && bytes.get(at + 1) != Some(&b'{')
            && (at == 0 || bytes[at - 1] != b'{')
    };
    if macros
        .iter()
        .filter(|arguments| arguments.contains(&start))
        .any(|arguments| (arguments.start..start).any(placeholder))
    {
        return true;
    }
    // The argument of `to_string(…)`: `ToString::to_string(&d.json_path)`.
    let mut depth = 0_usize;
    for (at, byte) in code[..start].bytes().enumerate().rev() {
        match byte {
            b')' => depth += 1,
            b'(' if depth == 0 => return code[..at].trim_end().ends_with("to_string"),
            b'(' => depth -= 1,
            b';' | b'{' | b'}' if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

/// The spans of the arguments of every macro call written with parentheses (`name!(…)`).
fn macro_arguments(code: &str) -> Vec<std::ops::Range<usize>> {
    let mut spans = Vec::new();
    for (bang, _) in code.match_indices('!') {
        if !code[..bang].ends_with(is_ident) {
            continue;
        }
        let rest = &code[bang + 1..];
        let open = bang + 1 + rest.len() - rest.trim_start().len();
        if !code[open..].starts_with('(') {
            continue;
        }
        let mut depth = 0_usize;
        let close = code[open..].bytes().position(|byte| {
            match byte {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            depth == 0
        });
        spans.push(open + 1..close.map_or(code.len(), |close| open + close));
    }
    spans
}

/// The names `code` declares as a `JsonPath`: `name: JsonPath,` or `name: JsonPath }`, the
/// type by any path (`crate::diagnostic::JsonPath`). A field of a struct or of a variant.
fn declared_paths(code: &str) -> Vec<&str> {
    const TYPE: &str = "JsonPath";
    let mut names = Vec::new();
    for (at, _) in code.match_indices(TYPE) {
        let after = code[at + TYPE.len()..].trim_start();
        if code[at + TYPE.len()..].starts_with(is_ident) || !after.starts_with([',', '}']) {
            continue;
        }
        // Back over the path the type is named by, to the `:` after the name.
        let mut before = &code[..at];
        while let Some(module) = before.strip_suffix("::") {
            before = module.trim_end_matches(is_ident);
        }
        let Some(name) = before.trim_end().strip_suffix(':') else {
            continue;
        };
        let name = name.trim_end();
        if name.ends_with(':') {
            continue;
        }
        let start = name.len() - name.chars().rev().take_while(|c| is_ident(*c)).count();
        if start < name.len() {
            names.push(&name[start..]);
        }
    }
    names
}

/// Library code never takes a key, a prefix or a step out of the TEXT of
/// a `JsonPath`. The text escapes `/` and `~` in a key and cannot tell a digit key from an
/// index, so code that reads it back goes wrong for such keys; `JsonPath::segments`,
/// `last_key`, `starts_with`, `ends_with`, `strip_prefix`, `is_key` and `index_after` answer
/// from the segments. This scan catches the spellings of the mistake a text can show
/// ([`path_text_reads`]) in the library sources of this crate, of `bedrock_addon_core`, of the
/// families and of the shared crates:
/// all of each file but the items of its tests, wherever those stand. A path is a finding's
/// `json_path`, and in a file that declares a field as a `JsonPath` that field. A path held
/// under yet another name (a local, a parameter, a field of a type of another file) is not
/// seen by it (the type is not in the text), which is why the rule is also written on
/// `JsonPath`'s `Display`.
#[test]
fn library_code_does_not_read_the_text_of_a_finding_s_path() {
    /// The library files that write a path out for a reader, and on how many lines: the
    /// text of a finding (`Display for Diagnostic`), the texts of three errors that say
    /// where (`EncodeError::NotRepresentable`, `ShapeError`, `MetaError::UnknownMember`) and
    /// the JSON pointer the legacy manifest reader's messages name a member by
    /// (`ManifestNotSemVer`, `ManifestWrongType`, `ManifestInvalidValue`). They are messages;
    /// nothing reads a key, a prefix or a step back out of them. A line more in one of these
    /// files, or one in any other file, fails.
    const SHOWN: [(&str, usize); 5] = [
        ("core/src/diagnostic/mod.rs", 1),
        ("core/src/json/mod.rs", 1),
        ("core/src/manifest/read.rs", 1),
        ("core/src/shared/read.rs", 1),
        ("families/script/src/meta/error.rs", 1),
    ];
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();
    let mut scanned = 0usize;
    let lifted = lifted_families()
        .iter()
        .map(|family| lifted_dir().join(family).join("src"));
    let shared = directories(&shared_dir())
        .into_iter()
        .map(|name| shared_dir().join(name).join("src"));
    for src in [manifest.join("src"), manifest.join("../core/src")]
        .into_iter()
        .chain(lifted)
        .chain(shared)
    {
        if !src.is_dir() {
            // A crate packaged alone has no sibling to scan.
            continue;
        }
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        for file in files {
            let relative = file.strip_prefix(&src).unwrap();
            if is_test_file(relative) || declared_for_tests(&file) {
                continue;
            }
            scanned += 1;
            let source = fs::read_to_string(&file).unwrap();
            let reads = path_text_reads(&library_part(&source));
            let shown = SHOWN
                .iter()
                .find(|(shows, _)| file.ends_with(shows))
                .map_or(0, |(_, times)| *times);
            if reads.len() != shown {
                violations.push(format!(
                    "{}: lines {reads:?} ({shown} listed as written out for a reader)",
                    file.display()
                ));
            }
        }
    }
    assert!(scanned > 500, "{scanned} library files scanned");
    assert!(
        violations.is_empty(),
        "library code takes the text of a finding's path (use the path's segments):\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_scan_for_path_text_sees_the_call_and_nothing_else() {
    assert_eq!(
        path_text_reads("let a = finding.location.json_path.to_string();\nlet b = 1;"),
        [1]
    );
    assert_eq!(
        path_text_reads("x\n    .location\n    .json_path\n    .to_string()\n    .len()"),
        [3]
    );
    // Another member, another method, a longer name.
    assert_eq!(
        path_text_reads("a.json_path.len(); my_json_path.to_string();"),
        [0usize; 0]
    );
    assert_eq!(
        path_text_reads("a.json_path.clone().to_owned()"),
        [0usize; 0]
    );
    // The unit tests are no part of the library, and test files are no library.
    let code = "fn f() {}\n#[cfg(test)]\nmod tests { fn g(d: D) { d.json_path.to_string(); } }";
    assert_eq!(path_text_reads(&library_part(code)), [0usize; 0]);
    assert!(is_test_file(Path::new("server_config/tests.rs")));
    assert!(is_test_file(Path::new("pack/link/tests_findings.rs")));
    assert!(is_test_file(Path::new(
        "pack/stack/accessors/tests/forms.rs"
    )));
    assert!(!is_test_file(Path::new("entity/goal/check.rs")));
}

/// A file whose module its parent declares behind `#[cfg(test)]` (`mod scalar_tests;`) is a
/// file of tests, whatever its name; one declared for the library is not.
#[test]
fn a_module_its_parent_declares_for_tests_is_no_library_file() {
    let families = Path::new(env!("CARGO_MANIFEST_DIR")).join("../families");
    if !families.is_dir() {
        // A crate packaged alone has no sibling to look at.
        return;
    }
    let src = families.join("attachable/src");
    assert!(declared_for_tests(&src.join("scalar_tests.rs")));
    assert!(declared_for_tests(&src.join("peek_tests.rs")));
    assert!(!declared_for_tests(&src.join("read.rs")));
    assert!(!declared_for_tests(&src.join("lib.rs")));
}

/// The library part of a file is all of it but the items of its tests: a test-only item long
/// before the test module does not end it, and neither does the test module.
#[test]
fn the_library_part_of_a_file_goes_on_after_an_item_of_its_tests() {
    let source = "#[cfg(test)]
fn only_for_tests<A, B>(d: D) -> [u8; 2] { d.json_path.to_string(); }
fn f(d: D) { d.json_path.to_string(); }
#[cfg(test)]
#[allow(dead_code)]
mod tests { fn g(d: D) { if true { d.json_path.to_string(); } } }
fn h(d: D) { d.json_path.to_string(); }
# [ cfg ( test ) ] use std::fmt::Write;
#[cfg(test)]
const SHOWN: fn(&D) -> String = |d| d.json_path.to_string();
struct S {
    #[cfg(test)]
    seen: Vec<u8>,
    json_path: JsonPath,
}
fn k(s: S) { s.json_path.to_string(); }";
    assert_eq!(path_text_reads(&library_part(source)), [3, 7, 16]);
    // An attribute that is no `cfg(test)` hides nothing.
    let source = "#[cfg(feature = \"test\")]\nfn f(d: D) { d.json_path.to_string(); }";
    assert_eq!(path_text_reads(&library_part(source)), [2]);
}

/// The scan sees a path written out by a format, as an argument and as a name in the text.
#[test]
fn the_scan_for_path_text_sees_a_path_in_a_format() {
    assert_eq!(
        path_text_reads(r#"let a = format!("{}", finding.location.json_path);"#),
        [1]
    );
    assert_eq!(
        path_text_reads(
            "let a = format!(\"at {json_path}\");\nwrite!(f, \"{json_path:>4} | \")?;\nwriteln!(\n    out,\n    \"{} {}\",\n    code,\n    &d.json_path,\n);"
        ),
        [1, 2, 7]
    );
    // No format: a call on the path, a comparison, braces that are text, a comment.
    for code in [
        r#"let a = format!("{}", d.json_path.len());"#,
        "assert_eq!(a.json_path, b.json_path);",
        r#"let a = format!("{{json_path}}");"#,
        "// format!(\"{json_path}\")\nlet a = 1;",
        r#"let a = format!("{}", my_json_path);"#,
    ] {
        assert_eq!(path_text_reads(&library_part(code)), [0usize; 0], "{code}");
    }
}

/// The files a scan that stopped at the first `#[cfg(test)]` read least of: a test-only item
/// stands near their top. Their library part goes on to their last function.
#[test]
fn the_library_part_of_a_file_with_an_early_test_item_is_the_rest_of_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "src/pack/link/resource/block.rs",
        "../families/material/src/patch.rs",
        "src/pack/link/mod.rs",
        "../families/feature/src/lib.rs",
    ] {
        let source = fs::read_to_string(root.join(file)).unwrap();
        let library = library_part(&source);
        assert_eq!(library.len(), source.len(), "{file}");
        let first = source.find("#[cfg(test)]").expect(file);
        let functions = |text: &str| text.matches("fn ").count();
        let (before, after) = (functions(&library[..first]), functions(&library[first..]));
        assert!(
            after > 0 && before + after <= functions(&code_of(&source)),
            "{file}: {before} functions before its first test item, {after} after"
        );
        // And the item of the tests itself is no part of it.
        assert!(!library.contains("cfg(test)"), "{file}");
    }
}

/// A `<` that is no generic's (a comparison, a shift) in an item of the tests does not hide
/// the library code after the item, and the body of a test item with a `where` is the item's.
#[test]
fn a_comparison_in_an_item_of_the_tests_does_not_hide_the_library_after_it() {
    for item in [
        "#[cfg(test)]\nconst X: bool = 1 < 2;",
        "#[cfg(test)]\nconst X: u8 = 1 << 2;",
        "#[cfg(test)]\nconst X: bool = 1 <= 2;",
        "#[cfg(test)]\nstatic X: bool = f(1 < 2, 3);",
        "#[cfg(test)]\nconst X: Vec<u8> = Vec::<u8>::new();",
        "#[cfg(test)]\nfn t(a: u8) -> bool { a < 2 }",
    ] {
        let source = format!("{item}\nfn f(d: D) {{ d.json_path.to_string(); }}\nfn g() {{}}");
        assert_eq!(path_text_reads(&library_part(&source)), [3], "{item}");
    }
    let source = "#[cfg(test)]
fn t<A, B>(d: D) where A: X<u8>, B: Y { d.json_path.to_string(); }
fn f(d: D) { d.json_path.to_string(); }";
    assert_eq!(path_text_reads(&library_part(source)), [3]);
    let source = "fn f(d: D) {
    #[cfg(test)]
    if 1 < 2 { d.json_path.to_string(); }
    #[cfg(test)]
    assert!(1 < 2);
    d.json_path.to_string();
}";
    assert_eq!(path_text_reads(&library_part(source)), [6]);
}

/// The spellings that take a path's text by another way than `.to_string()` on it or a
/// format: a clone before either, `ToString::to_string` called or passed on, and a macro
/// that is no `format!` but writes its arguments out after a text (`assert!`, `log::warn!`).
#[test]
fn the_scan_for_path_text_sees_the_other_ways_to_the_text() {
    for code in [
        r#"let a = format!("{}", d.json_path.clone());"#,
        "let a = d.json_path.clone().to_string();",
        "let a = d.json_path\n    .clone()\n    .clone()\n    .to_string();",
        "let a = ToString::to_string(&d.json_path);",
        "let a = ToString::to_string(&d.location.json_path.clone());",
        "let a = found.map(|f| &f.location.json_path).map(ToString::to_string);",
        r#"assert!(ok, "at {}", d.json_path);"#,
        r#"debug_assert_eq!(a, b, "at {}", &d.json_path);"#,
        r#"log::warn!("dropped at {}", d.location.json_path);"#,
        r#"tracing::info!(target: "x", "dropped at {} ({})", d.location.json_path, 1);"#,
    ] {
        assert_eq!(path_text_reads(&library_part(code)), [1], "{code}");
    }
    // No text is taken: a path compared (before the text of the assertion), a macro without
    // a text, a clone kept, `ToString` of another value.
    for code in [
        r#"assert_eq!(a.json_path, b.json_path, "the two differ ({})", 1);"#,
        "assert!(a.json_path == b.json_path);",
        "let a = vec![d.json_path.clone()];",
        "let a = d.json_path.clone();",
        "let a = names.iter().map(ToString::to_string); let b = d.json_path.len();",
        "let a = ToString::to_string(&d.json_path.len());",
    ] {
        assert_eq!(path_text_reads(&library_part(code)), [0usize; 0], "{code}");
    }
}

/// A name a file declares as a `JsonPath` (a field of an error, of a finding) is a path in
/// that file under that name: its text in a format is seen, as `json_path`'s is anywhere.
#[test]
fn the_scan_for_path_text_knows_the_names_a_file_gives_its_paths() {
    let error = "#[derive(Debug, thiserror::Error)]
pub enum E {
    #[error(\"unknown member `{at}`\")]
    Unknown {
        at: crate::diagnostic::JsonPath,
    },
}";
    assert_eq!(path_text_reads(&library_part(error)), [3]);
    let shape = "pub struct ShapeError {
    kind: Kind,
    path: JsonPath,
}
impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, \"at `{}`: {}\", self.path, self.kind)
    }
}
fn text(e: &ShapeError) -> String {
    e.path.to_string()
}";
    assert_eq!(path_text_reads(&library_part(shape)), [7, 11]);
    // The same texts where the name is no path of the file: nothing.
    let other = "pub struct Place {
    path: PackPath,
    at: usize,
}
fn text(p: &Place) -> String {
    format!(\"{} {at}\", p.path, at = p.at)
}";
    assert_eq!(path_text_reads(&library_part(other)), [0usize; 0]);
}
