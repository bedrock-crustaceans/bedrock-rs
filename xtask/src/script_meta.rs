//! `cargo xtask script-meta --samples <dir>`: generates
//! `crates/addon/families/script/src/meta/generated/versions.rs` and
//! `crates/addon/core/src/vanilla/generated/{vanilla_ids,commands}.rs` from the game's public
//! script metadata (the `metadata/` folder of a checkout of Mojang's `bedrock-samples`
//! repository at a release tag).
//!
//! The checkout must be at a tag; the tag goes into `versions.rs` as `SAMPLES_TAG`.
//! Every family is parsed with the strict model of `bedrock_addon::families::script::meta::model` (an
//! unknown key fails the run), every type descriptor is lowered and every privilege list read,
//! and the vendoring checks run before anything is written: an error row fails the run, warnings
//! and info rows are printed. The raw metadata files are never committed.
//!
//! `bedrock_addon`'s test `script_meta.rs` compiles this file with `#[path]` and runs
//! [`generate`] and [`format`] in-process to check the committed output.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::io::{self, Write as _};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs, str};

use bedrock_addon::families::script::meta::model::commands::CommandsFile;
use bedrock_addon::families::script::meta::model::engine::AfterEventsOrdering;
use bedrock_addon::families::script::meta::model::molang::MolangQueriesFile;
use bedrock_addon::families::script::meta::model::script::{
    Class, ClassConstant, ConstValue, Function, Interface, PeerDependency, Property, RawTypeDesc,
};
use bedrock_addon::families::script::meta::model::vanilla::{
    BiomeEntry, BlockEntry, EnchantmentEntry, ItemEntry, NameEntry, VanillaDataFile,
};
use bedrock_addon::families::script::meta::{
    MetaError, ModuleMetadata, NativeModule, PrivilegeList, TypeDescriptor,
};
use bedrock_addon::version::SemVersion;

/// The generated catalog, relative to the workspace root.
pub const VERSIONS_OUT: &str = "crates/addon/families/script/src/meta/generated/versions.rs";
/// The generated identifier lists, relative to the workspace root.
pub const VANILLA_IDS_OUT: &str = "crates/addon/core/src/vanilla/generated/vanilla_ids.rs";
/// The generated command table, relative to the workspace root.
pub const COMMANDS_OUT: &str = "crates/addon/core/src/vanilla/generated/commands.rs";

/// The script-module directory, relative to the samples checkout.
pub const SCRIPT_MODULES: &str = "metadata/script_modules/@minecraft";
/// The after-events ordering file.
pub const AFTER_EVENTS: &str = "metadata/engine_modules/engine-after-events-ordering.json";
/// The vanilla data directory.
pub const VANILLA_DATA: &str = "metadata/vanilladata_modules";
/// The commands file.
pub const COMMANDS: &str = "metadata/command_modules/mojang-commands.json";
/// The Molang queries file.
pub const MOLANG_QUERIES: &str = "metadata/molang_modules/mojang-molang-queries.json";
/// The JSON schema directory.
pub const JSON_SCHEMAS: &str = "metadata/json_schemas";

/// The vanilla data files and the `vanilla_data_type` each has.
pub const VANILLA_DATA_FILES: [(&str, &str); 12] = [
    ("mojang-biomes.json", "biome"),
    ("mojang-blocks.json", "block"),
    ("mojang-camera-presets.json", "cameraPresets"),
    ("mojang-cooldown-category.json", "cooldownCategory"),
    ("mojang-dimensions.json", "dimension"),
    ("mojang-effects.json", "effect"),
    ("mojang-enchantments.json", "enchantment"),
    ("mojang-entities.json", "entity"),
    ("mojang-features.json", "feature"),
    ("mojang-items.json", "item"),
    ("mojang-potion-effects.json", "potionEffect"),
    ("mojang-potion-types.json", "potionDelivery"),
];

/// The blocks the game registers that `mojang-blocks.json` leaves out: technical and retired
/// blocks no script constant names. They are blocks all the same (the game's own files name
/// `minecraft:end_gateway` as one), so the generated block list ends with them.
// These fourteen are the names of the game's block registry that `mojang-blocks.json` lacks;
// `bedrock_addon_core`'s test `every_block_the_game_registers_is_in_the_block_list` holds the
// generated list against the game's own block snapshots. Sorted bytewise.
pub const UNLISTED_BLOCKS: [&str; 14] = [
    "minecraft:chalkboard",
    "minecraft:client_request_placeholder_block",
    "minecraft:deprecated_anvil",
    "minecraft:deprecated_purpur_block_1",
    "minecraft:deprecated_purpur_block_2",
    "minecraft:end_gateway",
    "minecraft:glowingobsidian",
    "minecraft:info_update",
    "minecraft:info_update2",
    "minecraft:invisible_bedrock",
    "minecraft:moving_block",
    "minecraft:netherreactor",
    "minecraft:reserved6",
    "minecraft:stonecutter",
];

/// The identifiers the game registers, one per line, sorted bytewise, kept as data files beside
/// this module. Each list of the metadata is followed by the names of the game's list it lacks,
/// after its own and after [`UNLISTED_BLOCKS`], each once, so that an older metadata tag misses
/// no name the game knows.
pub mod game {
    /// `items.txt`: `VanillaItemNames` and the 119 element items.
    pub const ITEMS: &str = include_str!("script_meta/game/items.txt");
    /// `block-items.txt`: every registered block but air has an item of its name.
    pub const BLOCK_ITEMS: &str = include_str!("script_meta/game/block-items.txt");
    /// `blocks.txt`: `VanillaBlockTypeIds`, the element blocks, air and unknown.
    pub const BLOCKS: &str = include_str!("script_meta/game/blocks.txt");
    /// `potions.txt`: the `Potion` statics, bare names (`freezing`).
    pub const POTIONS: &str = include_str!("script_meta/game/potions.txt");
}

/// Appends each line of `game` (with `prefix` before it) that `list` lacks, in order.
fn append_game_names(list: &mut Vec<String>, game: &str, prefix: &str) {
    for name in game.lines().filter(|line| !line.is_empty()) {
        let name = format!("{prefix}{name}");
        if !list.contains(&name) {
            list.push(name);
        }
    }
}

/// The built-in parameter type tokens of the commands file.
pub const COMMAND_TOKENS: [&str; 26] = [
    "SELECTION",
    "WILDCARDSELECTION",
    "INT",
    "VAL",
    "RVAL",
    "ID",
    "POSITION",
    "POSITION_FLOAT",
    "RAWTEXT",
    "MESSAGE_ROOT",
    "JSON_OBJECT",
    "BLOCK_STATE_ARRAY",
    "COMPAREOPERATOR",
    "OPERATOR",
    "FULLINTEGERRANGE",
    "WILDCARDINT",
    "PATHCOMMAND",
    "CODEBUILDERARGS",
    "EXECUTECHAINEDOPTION_0",
    "CLOCK_TIMEMARKER_NAME",
    "POI_TAG",
    "POI_TYPE",
    "postfix_d",
    "postfix_l",
    "postfix_s",
    "postfix_t",
];

/// The npm type package every bindings file from 1.18.0 peers; it ships no metadata file.
const VANILLA_DATA_PACKAGE: &str = "@minecraft/vanilla-data";

/// Runs the step over a checkout of the metadata repository at `samples`.
pub fn run(samples: &Path) -> io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask has no parent directory"))?;
    let input = read_samples(samples)?;
    let generated = match generate(&input) {
        Ok(generated) => generated,
        Err(errors) => {
            for finding in &errors {
                eprintln!("{finding}");
            }
            return Err(io::Error::other(format!(
                "script-meta: {} check(s) failed; nothing written",
                errors.len()
            )));
        }
    };
    for finding in &generated.report {
        eprintln!("{finding}");
    }
    fs::write(root.join(VERSIONS_OUT), format(&generated.versions)?)?;
    fs::write(root.join(VANILLA_IDS_OUT), format(&generated.vanilla_ids)?)?;
    fs::write(root.join(COMMANDS_OUT), format(&generated.commands)?)
}

/// The metadata files of one samples tag, as bytes.
#[derive(Debug, Default)]
pub struct Samples {
    /// The tag the checkout is at (`v1.26.50.4`).
    pub tag: String,
    /// `script_modules/@minecraft/*.json`: file name and bytes, sorted by name.
    pub script_modules: Vec<(String, Vec<u8>)>,
    /// `engine-after-events-ordering.json`.
    pub after_events: Vec<u8>,
    /// `vanilladata_modules/*.json`: file name and bytes, sorted by name.
    pub vanilla_data: Vec<(String, Vec<u8>)>,
    /// `mojang-commands.json`.
    pub commands: Vec<u8>,
    /// `mojang-molang-queries.json`.
    pub molang_queries: Vec<u8>,
    /// `json_schemas/**/*.json`: the path below `json_schemas/` with `/` separators, and the
    /// bytes, sorted by path.
    pub json_schemas: Vec<(String, Vec<u8>)>,
}

/// Reads the metadata files of the checkout at `dir`, and its tag through `git`.
pub fn read_samples(dir: &Path) -> io::Result<Samples> {
    let git = |args: &[&str]| -> io::Result<String> {
        let output = Command::new("git").arg("-C").arg(dir).args(args).output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    let directory = |path: &Path| -> io::Result<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.strip_suffix(".json").is_some() && entry.file_type()?.is_file() {
                files.push((name, fs::read(entry.path())?));
            }
        }
        files.sort();
        Ok(files)
    };
    let mut json_schemas = Vec::new();
    let schemas = dir.join(JSON_SCHEMAS);
    if schemas.is_dir() {
        walk_schemas(&schemas, "", &mut json_schemas)?;
    }
    json_schemas.sort();
    Ok(Samples {
        tag: git(&["describe", "--tags", "--exact-match", "HEAD"])?,
        script_modules: directory(&dir.join(SCRIPT_MODULES))?,
        after_events: fs::read(dir.join(AFTER_EVENTS))?,
        vanilla_data: directory(&dir.join(VANILLA_DATA))?,
        commands: fs::read(dir.join(COMMANDS))?,
        molang_queries: fs::read(dir.join(MOLANG_QUERIES))?,
        json_schemas,
    })
}

fn walk_schemas(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{prefix}{name}");
        if entry.file_type()?.is_dir() {
            walk_schemas(&entry.path(), &format!("{path}/"), out)?;
        } else if name.strip_suffix(".json").is_some() {
            out.push((path, fs::read(entry.path())?));
        }
    }
    Ok(())
}

/// Passes generated Rust through `rustfmt` (edition 2024, default style), the form
/// `cargo fmt` keeps.
pub fn format(source: &str) -> io::Result<String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("rustfmt has no stdin"))?;
    let text = source.to_owned();
    let writer = std::thread::spawn(move || stdin.write_all(text.as_bytes()));
    let output = child.wait_with_output()?;
    writer
        .join()
        .map_err(|_| io::Error::other("rustfmt writer panicked"))??;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "rustfmt: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(io::Error::other)
}

/// The three generated files (before `rustfmt`) and the warning and info rows.
#[derive(Debug)]
pub struct Generated {
    /// `versions.rs`.
    pub versions: String,
    /// `vanilla_ids.rs`.
    pub vanilla_ids: String,
    /// `commands.rs`.
    pub commands: String,
    /// Warning and info findings, in check order.
    pub report: Vec<Finding>,
}

/// A vendoring check, or one of the model and table checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Check {
    /// A file does not parse with the strict model, a descriptor does not lower, or a privilege
    /// list breaks the generator's lowering rules.
    Model,
    /// 1: file name `<m>_<v>` against `name` `@minecraft/<m>`.
    FileName,
    /// 2: a stub's `parentModule` file exists and has the stub's version.
    Parent,
    /// 3: a stub carries declarations (warning; they become its overlay).
    StubDeclarations,
    /// 4: every `from_module` names a shipped file that declares the type, with an equal UUID.
    FromModule,
    /// 5: every unqualified bind-type name, base types included, is declared in its module.
    Unresolved,
    /// 6: no duplicate declaration name in a module, no duplicate member in a declaration.
    Duplicate,
    /// 7: every peer (module, version) is shipped, the npm `@minecraft/vanilla-data` aside.
    PeerShipped,
    /// 8: `error_types` only with `is_errorable`, never empty.
    ErrorTypes,
    /// 9: `details.min_value <= max_value`, and both exact in an `f64`.
    DetailBounds,
    /// 10: one beta per module and no other pre-release tag in file names.
    PreRelease,
    /// 11: one UUID per module across its versions.
    Uuid,
    /// 12: a `from_module` module is not a declared peer (info, by design).
    FromModulePeer,
    /// 13: a `from_module` version is not among the peer's versions (info).
    FromModuleVersion,
    /// 14: after-events names are classes of the matching bindings file.
    AfterEvents,
    /// 15: after-events versions without a metadata file (info; `ENGINE_ONLY`).
    EngineOnly,
    /// 16: every block's state names a `block_properties` entry (unused entries are info).
    BlockProperties,
    /// 17: every command parameter type is a command enum or a built-in token (an enum no
    /// parameter names is info).
    CommandTypes,
    /// 18: `json_schemas` `$ref`s resolve after percent-decoding (info; not vendored).
    SchemaRefs,
    /// The vanilla data headers (`module_type`, `name`, `vanilla_data_type`, `block_properties`
    /// only on blocks, state values of their property's type), the other files' headers, and
    /// data the model keeps but never saw (a `dependencies` entry, a string module constant).
    Headers,
    /// The catalog derivation: every file of a module has one shape and one
    /// parent.
    Catalog,
}

/// How a finding weighs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Fails the run.
    Error,
    /// Printed.
    Warning,
    /// Printed.
    Info,
}

/// One result of a check.
#[derive(Clone, Debug)]
pub struct Finding {
    /// The check.
    pub check: Check,
    /// Its weight.
    pub level: Level,
    /// What was found, for the person running the step.
    pub detail: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.level {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Info => "info",
        };
        write!(f, "{level}: {:?}: {}", self.check, self.detail)
    }
}

/// Collects findings while the checks run.
#[derive(Default)]
struct Findings(Vec<Finding>);

impl Findings {
    fn push(&mut self, check: Check, level: Level, detail: impl Into<String>) {
        self.0.push(Finding {
            check,
            level,
            detail: detail.into(),
        });
    }

    fn error(&mut self, check: Check, detail: impl Into<String>) {
        self.push(check, Level::Error, detail);
    }
}

/// One parsed script-module file.
struct ModuleFile {
    /// The file name.
    file: String,
    /// The version from the file name.
    version: String,
    /// Its `SemVersion` reading.
    sem: SemVersion,
    /// The file.
    meta: ModuleMetadata,
}

/// Runs every check over `samples` and renders the three files; the error findings when any
/// check fails.
pub fn generate(samples: &Samples) -> Result<Generated, Vec<Finding>> {
    let mut findings = Findings::default();
    let files = read_modules(samples, &mut findings);
    check_modules(&files, &mut findings);
    let engine_only = check_after_events(samples, &files, &mut findings);
    let vanilla = read_vanilla_data(samples, &mut findings);
    let commands = read_commands(samples, &mut findings);
    read_molang_queries(samples, &mut findings);
    check_schema_refs(samples, &mut findings);
    let catalog = catalog(&files, &mut findings);
    let (errors, mut report): (Vec<_>, Vec<_>) = findings
        .0
        .into_iter()
        .partition(|finding| finding.level == Level::Error);
    if !errors.is_empty() {
        return Err(errors);
    }
    report.sort_by_key(|finding| finding.check);
    let (Some(vanilla), Some(commands), Some(catalog)) = (vanilla, commands, catalog) else {
        // Every `None` comes with an error finding.
        return Err(Vec::new());
    };
    let header = |what: &str| {
        format!(
            "// @generated by `cargo xtask script-meta --samples <dir>` (xtask/src/script_meta.rs) from\n\
             // the game's {what}. Do not edit; regenerate.\n"
        )
    };
    Ok(Generated {
        versions: render_versions(
            &header("script module metadata (`script_modules/`)"),
            &samples.tag,
            &catalog,
            &engine_only,
        ),
        vanilla_ids: render_vanilla_ids(
            &header("identifier metadata (`vanilladata_modules/`)"),
            &vanilla,
        ),
        commands: render_commands(&header("command metadata (`command_modules/`)"), &commands),
        report,
    })
}

// --- script modules -----------------------------------------------------------------------

/// Parses every script-module file (check 1 and the model), sorted by module then version.
fn read_modules(samples: &Samples, findings: &mut Findings) -> Vec<ModuleFile> {
    let mut files = Vec::new();
    for (file, bytes) in &samples.script_modules {
        let Some((prefix, version)) = file
            .strip_suffix(".json")
            .and_then(|stem| stem.split_once('_'))
        else {
            findings.error(
                Check::FileName,
                format!("{file}: not `<module>_<version>.json`"),
            );
            continue;
        };
        let Some(sem) =
            SemVersion::parse(version).filter(|sem| !sem.is_any() && sem.build.is_empty())
        else {
            findings.error(
                Check::FileName,
                format!("{file}: `{version}` is not a version"),
            );
            continue;
        };
        let meta = match ModuleMetadata::parse(bytes) {
            Ok(meta) => meta,
            Err(error) => {
                findings.error(Check::Model, format!("{file}: {error}"));
                continue;
            }
        };
        if meta.name() != format!("@minecraft/{prefix}") {
            findings.error(
                Check::FileName,
                format!(
                    "{file}: `name` is `{}`, not `@minecraft/{prefix}`",
                    meta.name()
                ),
            );
        }
        files.push(ModuleFile {
            file: file.clone(),
            version: version.to_owned(),
            sem,
            meta,
        });
    }
    files.sort_by(|a, b| {
        a.meta
            .name()
            .cmp(b.meta.name())
            .then_with(|| a.sem.cmp(&b.sem))
    });
    files
}

/// Checks 2–13 over the parsed files, and the model checks that need the whole file
/// (descriptor lowering, privilege lists).
fn check_modules(files: &[ModuleFile], findings: &mut Findings) {
    let find = |name: &str, version: &str| {
        files
            .iter()
            .find(|f| f.meta.name() == name && f.version == version)
    };
    let mut uuids: BTreeMap<&str, (u128, &str)> = BTreeMap::new();
    let mut betas: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for file in files {
        let name = file.meta.name();
        // Check 11: one UUID per module.
        let uuid = file.meta.uuid().as_u128();
        match uuids.get(name) {
            Some((first_uuid, first)) if *first_uuid != uuid => findings.error(
                Check::Uuid,
                format!("{}: uuid differs from {first}", file.file),
            ),
            Some(_) => {}
            None => {
                uuids.insert(name, (uuid, &file.file));
            }
        }
        // Check 10: only `-beta`, once per module.
        match file.sem.pre_release.as_str() {
            "" => {}
            "beta" => betas.entry(name).or_default().push(&file.file),
            other => findings.error(
                Check::PreRelease,
                format!("{}: pre-release `{other}` is not `beta`", file.file),
            ),
        }
        // Check 7: every peer (module, version) is shipped.
        for peer in file.meta.peer_dependencies() {
            if peer.name == VANILLA_DATA_PACKAGE {
                continue;
            }
            for version in &peer.versions {
                if find(&peer.name, &version.version).is_none() {
                    findings.error(
                        Check::PeerShipped,
                        format!(
                            "{}: peer {} {} is not shipped",
                            file.file, peer.name, version.version
                        ),
                    );
                }
            }
        }
        match &file.meta {
            ModuleMetadata::Stub(stub) => {
                // Check 2: the parent file exists and has the stub's version.
                if stub.parent_module.version != file.version {
                    findings.error(
                        Check::Parent,
                        format!(
                            "{}: parentModule version {} is not the file's",
                            file.file, stub.parent_module.version
                        ),
                    );
                }
                if find(&stub.parent_module.name, &stub.parent_module.version).is_none() {
                    findings.error(
                        Check::Parent,
                        format!(
                            "{}: parent {} {} is not shipped",
                            file.file, stub.parent_module.name, stub.parent_module.version
                        ),
                    );
                }
                // Check 3: declarations on a stub are overlay.
                if let Some(overlay) = &stub.overlay {
                    findings.push(
                        Check::StubDeclarations,
                        Level::Warning,
                        format!(
                            "{}: {} classes, {} interfaces, {} enums on a stub (overlay)",
                            file.file,
                            overlay.classes.len(),
                            overlay.interfaces.len(),
                            overlay.enums.len()
                        ),
                    );
                    let mut declarations = Declarations::default();
                    declarations.classes(&overlay.classes);
                    declarations.interfaces(&overlay.interfaces);
                    for constant in overlay.enums.iter().flat_map(|e| &e.constants) {
                        declarations.descriptors.push(&constant.ty);
                    }
                    declarations.lower(&file.file, findings);
                }
            }
            ModuleMetadata::Native(native) => check_native(file, native, files, findings),
        }
    }
    for (module, list) in betas {
        if list.len() > 1 {
            findings.error(
                Check::PreRelease,
                format!("{module}: {} beta files ({})", list.len(), list.join(", ")),
            );
        }
    }
}

/// The descriptors and privilege lists of one module's declarations.
#[derive(Default)]
struct Declarations<'a> {
    descriptors: Vec<&'a RawTypeDesc>,
    base_types: Vec<&'a RawTypeDesc>,
    privileges: Vec<(
        &'a str,
        &'a [bedrock_addon::families::script::meta::model::script::PrivilegeName],
    )>,
    details: Vec<(&'a str, Option<f64>, Option<f64>)>,
}

impl<'a> Declarations<'a> {
    fn classes(&mut self, classes: &'a [Class]) {
        for class in classes {
            self.descriptors.push(&class.ty);
            self.base_types.extend(&class.base_types);
            self.descriptors.extend(&class.iterator);
            self.constants(&class.constants);
            self.functions(&class.functions);
            self.properties(&class.properties);
            for param in class.generic_class_types.iter().flatten() {
                self.descriptors.extend(&param.constraint);
                self.descriptors.extend(&param.default);
            }
        }
    }

    fn interfaces(&mut self, interfaces: &'a [Interface]) {
        for interface in interfaces {
            self.descriptors.push(&interface.ty);
            self.base_types.extend(&interface.base_types);
            self.properties(&interface.properties);
            self.constants(interface.constants.as_deref().unwrap_or_default());
            self.functions(interface.functions.as_deref().unwrap_or_default());
        }
    }

    fn constants(&mut self, constants: &'a [ClassConstant]) {
        for constant in constants {
            self.descriptors.push(&constant.ty);
        }
    }

    fn functions(&mut self, functions: &'a [Function]) {
        for function in functions {
            self.descriptors.push(&function.return_type);
            self.privileges
                .push((&function.name, &function.call_privilege));
            for argument in &function.arguments {
                self.descriptors.push(&argument.ty);
                if let Some(details) = &argument.details {
                    self.details
                        .push((&argument.name, details.min_value, details.max_value));
                }
            }
            for param in function.generic_function_types.iter().flatten() {
                self.descriptors.extend(&param.constraint);
                self.descriptors.extend(&param.default);
            }
        }
    }

    fn properties(&mut self, properties: &'a [Property]) {
        for property in properties {
            self.descriptors.push(&property.ty);
            for list in [&property.get_privilege, &property.set_privilege]
                .into_iter()
                .flatten()
            {
                self.privileges.push((&property.name, list));
            }
        }
    }

    /// Lowers every descriptor (check 8 and the lowering rules), reads every privilege list,
    /// and checks argument bounds (check 9).
    fn lower(&self, file: &str, findings: &mut Findings) {
        // 2^53: above it an integer token may not survive the `f64` reading.
        const EXACT: f64 = 9_007_199_254_740_992.0;
        for descriptor in self.descriptors.iter().chain(&self.base_types) {
            if let Err(error) = TypeDescriptor::lower(descriptor) {
                let check = match error {
                    MetaError::ErrorTypes => Check::ErrorTypes,
                    _ => Check::Model,
                };
                findings.error(check, format!("{file}: `{}`: {error}", descriptor.name));
            }
        }
        for (member, names) in &self.privileges {
            if let Err(error) = PrivilegeList::from_names(names) {
                findings.error(Check::Model, format!("{file}: `{member}`: {error}"));
            }
        }
        for (argument, min, max) in &self.details {
            if let (Some(min), Some(max)) = (min, max)
                && min > max
            {
                findings.error(
                    Check::DetailBounds,
                    format!("{file}: argument `{argument}`: min_value {min} > max_value {max}"),
                );
            }
            if [min, max].into_iter().flatten().any(|x| x.abs() >= EXACT) {
                findings.error(
                    Check::DetailBounds,
                    format!("{file}: argument `{argument}`: a bound is not exact in an f64"),
                );
            }
        }
    }
}

/// Calls `visit` on `descriptor` and on every descriptor nested in it.
fn each_descriptor<'a>(descriptor: &'a RawTypeDesc, visit: &mut impl FnMut(&'a RawTypeDesc)) {
    visit(descriptor);
    let boxed = [
        &descriptor.optional_type,
        &descriptor.element_type,
        &descriptor.key_type,
        &descriptor.value_type,
        &descriptor.promise_type,
        &descriptor.generic_base,
    ];
    for inner in boxed.into_iter().flatten() {
        each_descriptor(inner, visit);
    }
    let lists = [
        &descriptor.error_types,
        &descriptor.variant_types,
        &descriptor.generic_types,
    ];
    for inner in lists.into_iter().flatten().flatten() {
        each_descriptor(inner, visit);
    }
    if let Some(closure) = &descriptor.closure_type {
        for inner in &closure.argument_types {
            each_descriptor(inner, visit);
        }
        each_descriptor(&closure.return_type, visit);
    }
    if let Some(generator) = &descriptor.generator_type {
        each_descriptor(&generator.yield_type, visit);
        each_descriptor(&generator.return_type, visit);
        each_descriptor(&generator.next_type, visit);
    }
}

/// The names a native module declares: classes, interfaces, enums, errors and type aliases.
fn declared_names(native: &NativeModule) -> Vec<&str> {
    native
        .classes
        .iter()
        .map(|c| c.name.as_str())
        .chain(native.interfaces.iter().map(|i| i.name.as_str()))
        .chain(native.enums.iter().map(|e| e.name.as_str()))
        .chain(native.errors.iter().map(|e| e.name.as_str()))
        .chain(native.type_aliases.iter().map(|a| a.name.as_str()))
        .collect()
}

/// The declarations of one native module, with the lowering findings and the informational
/// notes about its constants and `dependencies` already recorded.
fn native_declarations<'a>(
    file: &ModuleFile,
    native: &'a NativeModule,
    findings: &mut Findings,
) -> Declarations<'a> {
    let mut declarations = Declarations::default();
    declarations.classes(&native.classes);
    declarations.interfaces(&native.interfaces);
    for constant in native.enums.iter().flat_map(|e| &e.constants) {
        declarations.descriptors.push(&constant.ty);
    }
    for error in &native.errors {
        declarations.descriptors.push(&error.ty);
        declarations.properties(&error.properties);
        declarations.functions(&error.functions);
    }
    declarations.functions(&native.functions);
    for object in &native.objects {
        declarations.descriptors.push(&object.ty);
    }
    for constant in &native.constants {
        declarations.descriptors.push(&constant.ty);
        if let ConstValue::Str(value) = &constant.value {
            findings.push(
                Check::Headers,
                Level::Info,
                format!(
                    "{}: module constant `{}` is a string ({value:?})",
                    file.file, constant.name
                ),
            );
        }
    }
    for alias in &native.type_aliases {
        declarations.descriptors.push(&alias.ty);
    }
    declarations.lower(&file.file, findings);
    if !native.dependencies.is_empty() {
        findings.push(
            Check::Headers,
            Level::Info,
            format!(
                "{}: {} `dependencies` entries",
                file.file,
                native.dependencies.len()
            ),
        );
    }
    declarations
}

/// Check 6: duplicate declarations and members.
fn check_duplicates(file: &ModuleFile, native: &NativeModule, findings: &mut Findings) {
    let names = declared_names(native);
    let mut seen = BTreeSet::new();
    for name in &names {
        if !seen.insert(*name) {
            findings.error(
                Check::Duplicate,
                format!("{}: `{name}` declared twice", file.file),
            );
        }
    }
    let mut members = |owner: &str, list: Vec<&str>| {
        let mut seen = BTreeSet::new();
        for member in list {
            if !seen.insert(member) {
                findings.error(
                    Check::Duplicate,
                    format!("{}: `{owner}.{member}` declared twice", file.file),
                );
            }
        }
    };
    for class in &native.classes {
        members(
            &class.name,
            class
                .constants
                .iter()
                .map(|c| c.name.as_str())
                .chain(class.functions.iter().map(|f| f.name.as_str()))
                .chain(class.properties.iter().map(|p| p.name.as_str()))
                .collect(),
        );
    }
    for interface in &native.interfaces {
        members(
            &interface.name,
            interface
                .properties
                .iter()
                .map(|p| p.name.as_str())
                .collect(),
        );
    }
    for enumeration in &native.enums {
        members(
            &enumeration.name,
            enumeration
                .constants
                .iter()
                .map(|c| c.name.as_str())
                .collect(),
        );
    }
    for error in &native.errors {
        members(
            &error.name,
            error
                .properties
                .iter()
                .map(|p| p.name.as_str())
                .chain(error.functions.iter().map(|f| f.name.as_str()))
                .collect(),
        );
    }
    members(
        "<module>",
        native
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .chain(native.objects.iter().map(|o| o.name.as_str()))
            .chain(native.constants.iter().map(|c| c.name.as_str()))
            .collect(),
    );
}

/// Checks 4, 5, 6, 8, 9, 12 and 13 over one native module, and its model checks.
fn check_native(
    file: &ModuleFile,
    native: &NativeModule,
    files: &[ModuleFile],
    findings: &mut Findings,
) {
    let declarations = native_declarations(file, native, findings);

    check_duplicates(file, native, findings);
    let names = declared_names(native);

    // Checks 4, 5, 12 and 13 over every descriptor, nested ones included.
    let mut not_peer = 0_usize;
    let mut not_peer_version = 0_usize;
    let mut visit = |descriptor: &RawTypeDesc, base: bool| {
        if let Some(from) = &descriptor.from_module {
            match files
                .iter()
                .find(|f| f.meta.name() == from.name && f.version == from.version)
            {
                None => findings.error(
                    Check::FromModule,
                    format!(
                        "{}: `{}` from {} {}, which is not shipped",
                        file.file, descriptor.name, from.name, from.version
                    ),
                ),
                Some(source) => {
                    let declares = match &source.meta {
                        ModuleMetadata::Native(native) => {
                            declared_names(native).contains(&descriptor.name.as_str())
                        }
                        ModuleMetadata::Stub(_) => false,
                    };
                    if !declares {
                        findings.error(
                            Check::FromModule,
                            format!(
                                "{}: `{}` is not declared by {}",
                                file.file, descriptor.name, source.file
                            ),
                        );
                    }
                    // Case-insensitive: `Uuid`'s `Display` is lower-case and hyphenated.
                    if from.uuid.to_ascii_lowercase() != source.meta.uuid().to_string() {
                        findings.error(
                            Check::FromModule,
                            format!(
                                "{}: `{}`: from_module uuid {} is not {}'s",
                                file.file, descriptor.name, from.uuid, source.file
                            ),
                        );
                    }
                }
            }
            match native
                .peer_dependencies
                .iter()
                .find(|p| p.name == from.name)
            {
                None => not_peer += 1,
                Some(peer) if !peer.versions.iter().any(|v| v.version == from.version) => {
                    not_peer_version += 1;
                }
                Some(_) => {}
            }
        } else if (descriptor.is_bind_type || base) && !names.contains(&descriptor.name.as_str()) {
            findings.error(
                Check::Unresolved,
                format!(
                    "{}: `{}` is not declared in the module",
                    file.file, descriptor.name
                ),
            );
        }
    };
    for descriptor in &declarations.descriptors {
        each_descriptor(descriptor, &mut |d| visit(d, false));
    }
    for descriptor in &declarations.base_types {
        visit(descriptor, true);
        each_descriptor(descriptor, &mut |d| {
            if !std::ptr::eq(d, *descriptor) {
                visit(d, false);
            }
        });
    }
    if not_peer > 0 {
        findings.push(
            Check::FromModulePeer,
            Level::Info,
            format!(
                "{}: {not_peer} from_module modules are not peers",
                file.file
            ),
        );
    }
    if not_peer_version > 0 {
        findings.push(
            Check::FromModuleVersion,
            Level::Info,
            format!(
                "{}: {not_peer_version} from_module versions are not among the peer's",
                file.file
            ),
        );
    }
}

// --- after-events ordering ----------------------------------------------------------------

/// Parses the after-events ordering (check 14) and returns the versions without a file
/// (check 15).
fn check_after_events(
    samples: &Samples,
    files: &[ModuleFile],
    findings: &mut Findings,
) -> Vec<(String, String)> {
    let ordering = match AfterEventsOrdering::parse(&samples.after_events) {
        Ok(ordering) => ordering,
        Err(error) => {
            findings.error(Check::Model, format!("{AFTER_EVENTS}: {error}"));
            return Vec::new();
        }
    };
    if ordering.module_type != "after_events_ordering"
        || ordering.name != "engine-after_events_ordering"
    {
        findings.error(
            Check::Headers,
            format!(
                "{AFTER_EVENTS}: module_type `{}`, name `{}`",
                ordering.module_type, ordering.name
            ),
        );
    }
    let find = |name: &str, version: &str| {
        files
            .iter()
            .find(|f| f.meta.name() == name && f.version == version)
    };
    let mut engine_only = Vec::new();
    for order in &ordering.after_events_order_by_version {
        let module = order.module();
        if order.name != format!("{module} {}", order.version) {
            findings.error(
                Check::AfterEvents,
                format!(
                    "`{}` does not end in its version {}",
                    order.name, order.version
                ),
            );
        }
        let Some(mut source) = find(module, &order.version) else {
            findings.push(
                Check::EngineOnly,
                Level::Info,
                format!("{module} {}: no metadata file", order.version),
            );
            engine_only.push((module.to_owned(), order.version.clone()));
            continue;
        };
        if let ModuleMetadata::Stub(stub) = &source.meta {
            match find(&stub.parent_module.name, &stub.parent_module.version) {
                Some(parent) => source = parent,
                None => continue, // check 2 reports it
            }
        }
        let ModuleMetadata::Native(native) = &source.meta else {
            continue;
        };
        for event in &order.event_order {
            if !native.classes.iter().any(|class| class.name == event.name) {
                findings.error(
                    Check::AfterEvents,
                    format!(
                        "{module} {}: `{}` is not a class of {}",
                        order.version, event.name, source.file
                    ),
                );
            }
        }
    }
    engine_only
}

// --- vanilla data -------------------------------------------------------------------------

/// The identifier lists of the vanilla data files.
#[derive(Debug, Default)]
struct VanillaLists {
    blocks: Vec<String>,
    items: Vec<String>,
    entities: Vec<String>,
    biomes: Vec<String>,
    effects: Vec<String>,
    enchantments: Vec<String>,
    enchantment_aliases: Vec<(String, String)>,
    dimensions: Vec<String>,
    camera_presets: Vec<String>,
    cooldown_categories: Vec<String>,
    features: Vec<String>,
    potion_effects: Vec<String>,
    potion_types: Vec<String>,
}

/// The strict reader of a vanilla data file with entries `T`
/// (`VanillaDataFile::<T>::parse`).
type VanillaParse<T> = fn(&[u8]) -> Result<VanillaDataFile<T>, MetaError>;

/// One vanilla data file read with `parse`, its header checked.
fn vanilla_file<T>(
    samples: &Samples,
    file: &str,
    parse: VanillaParse<T>,
    findings: &mut Findings,
) -> Option<VanillaDataFile<T>> {
    let (_, bytes) = samples.vanilla_data.iter().find(|(name, _)| name == file)?;
    let parsed = match parse(bytes) {
        Ok(parsed) => parsed,
        Err(error) => {
            findings.error(Check::Model, format!("{file}: {error}"));
            return None;
        }
    };
    let expected = VANILLA_DATA_FILES
        .iter()
        .find(|(name, _)| *name == file)
        .map(|(_, ty)| *ty);
    if parsed.module_type != "vanilla_data"
        || Some(parsed.vanilla_data_type.as_str()) != expected
        || parsed.name != format!("mojang-{}", parsed.vanilla_data_type)
    {
        findings.error(
            Check::Headers,
            format!(
                "{file}: module_type `{}`, name `{}`, vanilla_data_type `{}`",
                parsed.module_type, parsed.name, parsed.vanilla_data_type
            ),
        );
    }
    if parsed.block_properties.is_some() != (file == "mojang-blocks.json") {
        findings.error(
            Check::Headers,
            format!("{file}: block_properties only belongs to mojang-blocks.json"),
        );
    }
    Some(parsed)
}

/// Reads the twelve vanilla data files (check 16 and the header checks).
fn read_vanilla_data(samples: &Samples, findings: &mut Findings) -> Option<VanillaLists> {
    let names: BTreeSet<&str> = samples
        .vanilla_data
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    let expected: BTreeSet<&str> = VANILLA_DATA_FILES.iter().map(|(name, _)| *name).collect();
    for missing in expected.difference(&names) {
        findings.error(
            Check::Headers,
            format!("{VANILLA_DATA}/{missing} is missing"),
        );
    }
    for extra in names.difference(&expected) {
        findings.error(
            Check::Headers,
            format!("{VANILLA_DATA}/{extra} is a vanilla data file the step does not know"),
        );
    }
    let mut lists = VanillaLists::default();
    let mut complete = true;
    let mut names_of = |file: &str, findings: &mut Findings| -> Vec<String> {
        if let Some(parsed) =
            vanilla_file(samples, file, VanillaDataFile::<NameEntry>::parse, findings)
        {
            parsed
                .data_items
                .into_iter()
                .map(|item| item.name)
                .collect()
        } else {
            complete = false;
            Vec::new()
        }
    };
    lists.entities = names_of("mojang-entities.json", findings);
    lists.effects = names_of("mojang-effects.json", findings);
    lists.dimensions = names_of("mojang-dimensions.json", findings);
    lists.camera_presets = names_of("mojang-camera-presets.json", findings);
    lists.cooldown_categories = names_of("mojang-cooldown-category.json", findings);
    lists.features = names_of("mojang-features.json", findings);
    lists.potion_effects = names_of("mojang-potion-effects.json", findings);
    append_game_names(&mut lists.potion_effects, game::POTIONS, "minecraft:");
    lists.potion_types = names_of("mojang-potion-types.json", findings);
    match vanilla_file(
        samples,
        "mojang-blocks.json",
        VanillaDataFile::<BlockEntry>::parse,
        findings,
    ) {
        Some(blocks) => {
            check_block_properties(&blocks, findings);
            lists.blocks = blocks.data_items.into_iter().map(|b| b.name).collect();
            // The blocks the file leaves out, after its own, each once.
            for name in UNLISTED_BLOCKS {
                if !lists.blocks.iter().any(|listed| listed == name) {
                    lists.blocks.push(name.to_owned());
                }
            }
            append_game_names(&mut lists.blocks, game::BLOCKS, "");
        }
        None => complete = false,
    }
    let items = VanillaDataFile::<ItemEntry>::parse;
    match vanilla_file(samples, "mojang-items.json", items, findings) {
        Some(items) => {
            lists.items = items.data_items.into_iter().map(|i| i.name).collect();
            append_game_names(&mut lists.items, game::ITEMS, "");
            append_game_names(&mut lists.items, game::BLOCK_ITEMS, "");
        }
        None => complete = false,
    }
    let biomes = VanillaDataFile::<BiomeEntry>::parse;
    match vanilla_file(samples, "mojang-biomes.json", biomes, findings) {
        Some(biomes) => lists.biomes = biomes.data_items.into_iter().map(|b| b.name).collect(),
        None => complete = false,
    }
    let enchantments = VanillaDataFile::<EnchantmentEntry>::parse;
    match vanilla_file(samples, "mojang-enchantments.json", enchantments, findings) {
        Some(enchantments) => {
            for entry in enchantments.data_items {
                if let Some(target) = entry.value {
                    lists.enchantment_aliases.push((entry.name.clone(), target));
                }
                lists.enchantments.push(entry.name);
            }
        }
        None => complete = false,
    }
    complete.then_some(lists)
}

/// Check 16, and that every state value has its property's type.
fn check_block_properties(blocks: &VanillaDataFile<BlockEntry>, findings: &mut Findings) {
    let definitions = blocks.block_properties.as_deref().unwrap_or_default();
    let mut used = BTreeSet::new();
    for block in &blocks.data_items {
        for property in &block.properties {
            if definitions.iter().any(|d| d.name == property.name) {
                used.insert(property.name.as_str());
            } else {
                findings.error(
                    Check::BlockProperties,
                    format!(
                        "{}: state `{}` has no block_properties entry",
                        block.name, property.name
                    ),
                );
            }
        }
    }
    for definition in definitions {
        if let Some(value) = definition
            .values
            .iter()
            .find(|v| v.value.property_type() != definition.ty)
        {
            findings.error(
                Check::Headers,
                format!(
                    "block property `{}`: value {:?} is not of its type {:?}",
                    definition.name, value.value, definition.ty
                ),
            );
        }
    }
    let unused = definitions.len() - used.len();
    if unused > 0 {
        findings.push(
            Check::BlockProperties,
            Level::Info,
            format!(
                "{unused} of {} block_properties entries are unused",
                definitions.len()
            ),
        );
    }
}

// --- commands and Molang queries ----------------------------------------------------------

/// Reads the commands file (check 17).
fn read_commands(samples: &Samples, findings: &mut Findings) -> Option<CommandsFile> {
    let file = match CommandsFile::parse(&samples.commands) {
        Ok(file) => file,
        Err(error) => {
            findings.error(Check::Model, format!("{COMMANDS}: {error}"));
            return None;
        }
    };
    if file.module_type != "commands" || file.name != "mojang-commands" {
        findings.error(
            Check::Headers,
            format!(
                "{COMMANDS}: module_type `{}`, name `{}`",
                file.module_type, file.name
            ),
        );
    }
    let mut referenced = BTreeSet::new();
    for command in &file.commands {
        for param in command.overloads.iter().flat_map(|o| &o.params) {
            let type_name = param.ty.name.as_str();
            if let Some(found) = file
                .command_enums
                .iter()
                .find(|e| e.name.to_ascii_uppercase() == type_name)
            {
                referenced.insert(found.name.as_str());
            } else if !COMMAND_TOKENS.contains(&type_name) {
                findings.error(
                    Check::CommandTypes,
                    format!(
                        "{}: parameter `{}` has type `{type_name}`, neither an enum nor a token",
                        command.name, param.name
                    ),
                );
            }
        }
    }
    let unreferenced: Vec<&str> = file
        .command_enums
        .iter()
        .map(|e| e.name.as_str())
        .filter(|name| !referenced.contains(name))
        .collect();
    if !unreferenced.is_empty() {
        findings.push(
            Check::CommandTypes,
            Level::Info,
            format!(
                "command enums no parameter names: {}",
                unreferenced.join(", ")
            ),
        );
    }
    Some(file)
}

/// Reads the Molang queries file: a cross-check only, nothing is generated from it.
fn read_molang_queries(samples: &Samples, findings: &mut Findings) {
    match MolangQueriesFile::parse(&samples.molang_queries) {
        Ok(file) if file.module_type != "molang" || file.name != "engine-molang" => findings.error(
            Check::Headers,
            format!(
                "{MOLANG_QUERIES}: module_type `{}`, name `{}`",
                file.module_type, file.name
            ),
        ),
        Ok(_) => {}
        Err(error) => findings.error(Check::Model, format!("{MOLANG_QUERIES}: {error}")),
    }
}

// --- json schemas -------------------------------------------------------------------------

/// Check 18: every `$ref` of a schema file names a schema file, after percent-decoding.
fn check_schema_refs(samples: &Samples, findings: &mut Findings) {
    let paths: BTreeSet<&str> = samples
        .json_schemas
        .iter()
        .map(|(path, _)| path.as_str())
        .collect();
    let mut total = 0_usize;
    let mut unresolved = Vec::new();
    for (path, bytes) in &samples.json_schemas {
        let text = String::from_utf8_lossy(bytes);
        for reference in schema_refs(&text) {
            total += 1;
            let target = reference.split('#').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let dir = path.rsplit_once('/').map_or("", |(dir, _)| dir);
            let resolved = resolve(dir, &percent_decode(target));
            if !resolved.as_deref().is_some_and(|r| paths.contains(r)) {
                unresolved.push(format!("{path}: {reference}"));
            }
        }
    }
    findings.push(
        Check::SchemaRefs,
        Level::Info,
        format!(
            "{} of {total} json_schemas $refs do not resolve{}",
            unresolved.len(),
            if unresolved.is_empty() {
                String::new()
            } else {
                format!(": {}", unresolved.join("; "))
            }
        ),
    );
}

/// The string values of every `"$ref"` member in `text`.
fn schema_refs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("\"$ref\"") {
        rest = &rest[at + "\"$ref\"".len()..];
        let after_colon = rest.trim_start().strip_prefix(':').map(str::trim_start);
        let Some(value) = after_colon.and_then(|r| r.strip_prefix('"')) else {
            continue;
        };
        let mut literal = String::new();
        let mut chars = value.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => literal.extend(chars.next()),
                c => literal.push(c),
            }
        }
        out.push(literal);
    }
    out
}

/// `%20` and the like decoded as UTF-8.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(byte) = text
                .get(i + 1..i + 3)
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `reference` resolved against the directory `dir` (`/`-separated, no leading `/`); `None`
/// when it climbs above the root.
fn resolve(dir: &str, reference: &str) -> Option<String> {
    let mut parts: Vec<&str> = if reference.starts_with('/') {
        Vec::new()
    } else {
        dir.split('/').filter(|p| !p.is_empty()).collect()
    };
    for part in reference.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            part => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

// --- the catalog --------------------------------------------------------------------------

/// One module of the catalog, before rendering.
struct Entry<'a> {
    name: &'a str,
    uuid: u128,
    /// The UUID in its lower-case hyphenated form, for the generated comment.
    uuid_text: String,
    parent: Option<&'a str>,
    versions: Vec<Version<'a>>,
}

/// One shipped version, before rendering.
struct Version<'a> {
    sem: &'a SemVersion,
    wrapper: Option<&'a str>,
    parent: Option<usize>,
    peers: Vec<Peer>,
}

/// A peer requirement, before rendering.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Peer {
    module: String,
    versions: Vec<String>,
    types_only: bool,
    wrapper_name: Option<String>,
}

impl From<&PeerDependency> for Peer {
    fn from(peer: &PeerDependency) -> Self {
        Self {
            module: peer.name.clone(),
            versions: peer.versions.iter().map(|v| v.version.clone()).collect(),
            types_only: peer.types_only == Some(true),
            wrapper_name: peer.wrapper_name.clone(),
        }
    }
}

/// Derives the catalog: one entry per module, the stubs' parent indices, and the
/// effective peers; `None` when a module's files disagree.
fn catalog<'a>(files: &'a [ModuleFile], findings: &mut Findings) -> Option<Vec<Entry<'a>>> {
    let mut entries: Vec<Entry<'a>> = Vec::new();
    let mut complete = true;
    // `files` is sorted by module, then version (SemVersion order).
    for file in files {
        let parent = match &file.meta {
            ModuleMetadata::Stub(stub) => Some(stub.parent_module.name.as_str()),
            ModuleMetadata::Native(_) => None,
        };
        match entries.last_mut() {
            Some(entry) if entry.name == file.meta.name() => {
                if entry.parent != parent {
                    findings.error(
                        Check::Catalog,
                        format!(
                            "{}: shape or parent differs from the module's other files",
                            file.file
                        ),
                    );
                    complete = false;
                }
            }
            _ => entries.push(Entry {
                name: file.meta.name(),
                uuid: file.meta.uuid().as_u128(),
                uuid_text: file.meta.uuid().to_string(),
                parent,
                versions: Vec::new(),
            }),
        }
        let wrapper = match &file.meta {
            ModuleMetadata::Stub(stub) => Some(stub.wrapper_module.as_str()),
            ModuleMetadata::Native(_) => None,
        };
        let entry = entries.last_mut()?;
        entry.versions.push(Version {
            sem: &file.sem,
            wrapper,
            parent: None,
            peers: file
                .meta
                .peer_dependencies()
                .iter()
                .map(Peer::from)
                .collect(),
        });
    }
    // Stubs: the parent's index and the merged peers (rules 3 and 5).
    let mut resolved = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let Some(parent) = entry.parent else {
            continue;
        };
        let Some(parent_entry) = entries.iter().find(|e| e.name == parent) else {
            complete = false; // check 2 reports it
            continue;
        };
        for (version_index, version) in entry.versions.iter().enumerate() {
            let Some(at) = parent_entry
                .versions
                .iter()
                .position(|v| v.sem.to_string() == version.sem.to_string())
            else {
                complete = false; // check 2 reports it
                continue;
            };
            let mut peers = parent_entry.versions[at].peers.clone();
            for own in &version.peers {
                match peers.iter_mut().find(|p| p.module == own.module) {
                    Some(slot) => *slot = own.clone(),
                    None => peers.push(own.clone()),
                }
            }
            resolved.push((index, version_index, at, peers));
        }
    }
    for (index, version_index, at, peers) in resolved {
        let version = &mut entries[index].versions[version_index];
        version.parent = Some(at);
        version.peers = peers;
    }
    complete.then_some(entries)
}

// --- rendering ----------------------------------------------------------------------------

/// A Rust string literal.
fn lit(text: &str) -> String {
    format!("{text:?}")
}

/// `Some("…")` or `None`.
fn opt_lit(text: Option<&str>) -> String {
    text.map_or_else(|| "None".to_owned(), |t| format!("Some({})", lit(t)))
}

/// The constant name of a peer requirement: `P_` and the module and versions in upper snake
/// case (`P_COMMON_1_2_0`), plus `_WRAPPED` / `_TYPES` where two requirements would share it.
fn peer_const_name(peer: &Peer) -> String {
    let module = peer
        .module
        .strip_prefix("@minecraft/")
        .unwrap_or(&peer.module);
    let mut name = format!("P_{module}");
    for version in &peer.versions {
        name.push('_');
        name.push_str(version);
    }
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// A distinct constant name for every peer requirement.
fn peer_const_names<'a>(peers: &BTreeSet<&'a Peer>) -> BTreeMap<&'a Peer, String> {
    let mut names: BTreeMap<&Peer, String> = BTreeMap::new();
    for peer in peers {
        let mut name = peer_const_name(peer);
        let clash = |name: &str, names: &BTreeMap<&Peer, String>| names.values().any(|n| n == name);
        let base_clash = peers
            .iter()
            .filter(|other| peer_const_name(other) == name)
            .count()
            > 1;
        if base_clash && peer.wrapper_name.is_some() {
            name.push_str("_WRAPPED");
        }
        if base_clash && peer.types_only {
            name.push_str("_TYPES");
        }
        while clash(&name, &names) {
            name.push('_');
        }
        names.insert(peer, name);
    }
    names
}

/// `versions.rs`.
fn render_versions(
    header: &str,
    tag: &str,
    entries: &[Entry<'_>],
    engine_only: &[(String, String)],
) -> String {
    let mut peers: BTreeSet<&Peer> = BTreeSet::new();
    for version in entries.iter().flat_map(|e| &e.versions) {
        peers.extend(&version.peers);
    }
    let names = peer_const_names(&peers);
    let mut out = String::new();
    out.push_str(header);
    out.push_str(
        "\nuse crate::meta::{Channel, ModuleEntry, ModuleShape, ModuleVersion, PeerReq};\n\n",
    );
    let _ = writeln!(
        out,
        "/// The release tag of the metadata the table was generated from.\n\
         pub(in crate::meta) const SAMPLES_TAG: &str = {};\n",
        lit(tag)
    );
    let mut sorted: Vec<(&String, &Peer)> =
        names.iter().map(|(peer, name)| (name, *peer)).collect();
    sorted.sort();
    for (name, peer) in sorted {
        let versions: Vec<String> = peer.versions.iter().map(|v| lit(v)).collect();
        let _ = writeln!(
            out,
            "const {name}: PeerReq = PeerReq {{ module: {}, versions: &[{}], types_only: {}, wrapper_name: {} }};",
            lit(&peer.module),
            versions.join(", "),
            peer.types_only,
            opt_lit(peer.wrapper_name.as_deref())
        );
    }
    out.push_str(
        "\n/// Every module, sorted by name.\npub(in crate::meta) static MODULES: &[ModuleEntry] = &[\n",
    );
    for entry in entries {
        let uuid = &entry.uuid_text;
        let digits = format!("{:032x}", entry.uuid);
        let grouped: Vec<&str> = (0..8).map(|i| &digits[i * 4..i * 4 + 4]).collect();
        let shape = entry.parent.map_or_else(
            || "ModuleShape::Native".to_owned(),
            |parent| format!("ModuleShape::Stub {{ parent: {} }}", lit(parent)),
        );
        let _ = writeln!(
            out,
            "ModuleEntry {{ name: {}, uuid: 0x{}, // {uuid}\nshape: {shape}, versions: &[",
            lit(entry.name),
            grouped.join("_"),
        );
        for version in &entry.versions {
            let pre =
                (!version.sem.pre_release.is_empty()).then_some(version.sem.pre_release.as_str());
            let channel = if pre.is_some() {
                "Channel::Beta"
            } else {
                "Channel::Stable"
            };
            let peers: Vec<&str> = version.peers.iter().map(|p| names[p].as_str()).collect();
            let _ = writeln!(
                out,
                "ModuleVersion {{ major: {}, minor: {}, patch: {}, pre: {}, channel: {channel}, wrapper: {}, parent_version: {}, peers: &[{}] }},",
                version.sem.major,
                version.sem.minor,
                version.sem.patch,
                opt_lit(pre),
                opt_lit(version.wrapper),
                version
                    .parent
                    .map_or_else(|| "None".to_owned(), |i| format!("Some({i})")),
                peers.join(", ")
            );
        }
        out.push_str("] },\n");
    }
    out.push_str("];\n\n");
    out.push_str(
        "/// Versions the engine knows (the after-events ordering lists them) but no metadata file ships;\n\
         /// never selectable by packs.\n\
         pub(in crate::meta) static ENGINE_ONLY: &[(&str, &str)] = &[\n",
    );
    for (module, version) in engine_only {
        let _ = writeln!(out, "({}, {}),", lit(module), lit(version));
    }
    out.push_str("];\n");
    out
}

/// `vanilla_ids.rs`.
fn render_vanilla_ids(header: &str, lists: &VanillaLists) -> String {
    let list = |items: &[String]| {
        let items: Vec<String> = items.iter().map(|i| lit(i)).collect();
        format!("&[{}]", items.join(", "))
    };
    let mut out = String::new();
    out.push_str(header);
    out.push_str(
        "// `blocks`, `items` and `potion_effects` end with the identifiers the game registers that\n\
         // the metadata lacks (technical and retired blocks and items).\n",
    );
    out.push_str("\nuse crate::vanilla::VanillaIds;\n\n/// The identifier lists.\n");
    out.push_str("pub(in crate::vanilla) static IDS: VanillaIds = VanillaIds {\n");
    let rows: [(&str, &[String]); 6] = [
        ("blocks", &lists.blocks),
        ("items", &lists.items),
        ("entities", &lists.entities),
        ("biomes", &lists.biomes),
        ("effects", &lists.effects),
        ("enchantments", &lists.enchantments),
    ];
    for (field, items) in rows {
        let _ = writeln!(out, "{field}: {},", list(items));
    }
    let aliases: Vec<String> = lists
        .enchantment_aliases
        .iter()
        .map(|(alias, target)| format!("({}, {})", lit(alias), lit(target)))
        .collect();
    let _ = writeln!(out, "enchantment_aliases: &[{}],", aliases.join(", "));
    let rows: [(&str, &[String]); 6] = [
        ("dimensions", &lists.dimensions),
        ("camera_presets", &lists.camera_presets),
        ("cooldown_categories", &lists.cooldown_categories),
        ("features", &lists.features),
        ("potion_effects", &lists.potion_effects),
        ("potion_types", &lists.potion_types),
    ];
    for (field, items) in rows {
        let _ = writeln!(out, "{field}: {},", list(items));
    }
    out.push_str("};\n");
    out
}

/// `commands.rs`: commands and enums sorted by name; descriptions are Mojang text and are not
/// generated.
fn render_commands(header: &str, file: &CommandsFile) -> String {
    let mut commands: Vec<_> = file.commands.iter().collect();
    commands.sort_by(|a, b| a.name.cmp(&b.name));
    let mut enums: Vec<_> = file.command_enums.iter().collect();
    enums.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\nuse crate::vanilla::{Command, CommandEnum, Overload, Param};\n\n");
    out.push_str("/// The commands, sorted by name.\npub(in crate::vanilla) static COMMANDS: &[Command] = &[\n");
    for command in commands {
        let aliases: Vec<String> = command.aliases.iter().map(|a| lit(&a.name)).collect();
        let _ = writeln!(
            out,
            "Command {{ name: {}, aliases: &[{}], permission_level: {}, requires_cheats: {}, overloads: &[",
            lit(&command.name),
            aliases.join(", "),
            command.permission_level,
            command.requires_cheats
        );
        for overload in &command.overloads {
            out.push_str("Overload { params: &[\n");
            for param in &overload.params {
                let _ = writeln!(
                    out,
                    "Param {{ name: {}, optional: {}, type_name: {} }},",
                    lit(&param.name),
                    param.is_optional,
                    lit(&param.ty.name)
                );
            }
            out.push_str("] },\n");
        }
        out.push_str("] },\n");
    }
    out.push_str("];\n\n/// The command enums, sorted by name.\n");
    out.push_str("pub(in crate::vanilla) static ENUMS: &[CommandEnum] = &[\n");
    for command_enum in enums {
        let values: Vec<String> = command_enum.values.iter().map(|v| lit(&v.value)).collect();
        let _ = writeln!(
            out,
            "CommandEnum {{ name: {}, values: &[{}] }},",
            lit(&command_enum.name),
            values.join(", ")
        );
    }
    out.push_str("];\n");
    out
}

#[cfg(test)]
mod tests;
