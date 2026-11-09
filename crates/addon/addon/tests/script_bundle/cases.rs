//! One test per row of the fixtures: the fixture pack loaded through
//! `scripts()` and compared, part by part, with its expected file.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::script::{Resolution, ScriptPath};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackOrigin, PackStack};

use super::common::{
    bundle_of, case_files, check, check_in, pack_of, script_manifest, script_stack, stack_of,
};

/// A test named for its fixture pack.
macro_rules! cases {
    ($($name:ident),* $(,)?) => {
        $(
            #[test]
            #[allow(
                non_snake_case,
                reason = "the test is named for its fixture folder, and some fixtures are named with capitals (lang_JS_upper)"
            )]
            fn $name() {
                check(stringify!($name));
            }
        )*
    };
}

// Entry and discovery.
cases!(
    missing_entry,
    entry_case,
    entry_ts,
    entry_mjs,
    entry_upper_ext,
    entry_empty,
    entry_absent,
    entry_dotslash,
    entry_outside,
    entry_backslash,
    entry_subdir,
    lang_lua,
    lang_JS_upper,
    lang_absent,
    two_modules,
    data_with_scripts,
);

// Files and encoding.
cases!(bom_entry, latin1_entry, names);

// Static imports.
cases!(
    static_ext,
    static_noext,
    static_dotdot,
    static_bare,
    static_at_unknown,
    static_undeclared,
    static_json,
    static_json_attr,
    syntax_entry,
    syntax_import,
    import_throw,
    toplevel_throw,
    circular,
    long_spec,
    dots,
    sourcemap,
);

/// A test named for its fixture pack, which pins a listed deviation from vanilla.
macro_rules! deviations {
    ($($test:ident => $name:literal),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                check($name);
            }
        )*
    };
}

// Where ours says more than vanilla does: an entry without the `scripts/` prefix is warned
// of where vanilla accepts it; a file nothing reaches, which vanilla never reads, gets a
// note for its bad bytes; a specifier that ends vanilla's process is an error found before
// anything runs.
deviations!(
    entry_noprefix_deviation => "entry_noprefix",
    unused_bad_deviation => "unused_bad",
    crash_dot_deviation => "crash_dot",
    crash_dotslash_deviation => "crash_dotslash",
    crash_dotdot_deviation => "crash_dotdot",
    crash_dotdotslash_deviation => "crash_dotdotslash",
    crash_static_deviation => "crash_static",
);

/// The dynamic matrix: the `dyn` pack, one expected row per specifier of the observed table
/// (`dyn` is a keyword, so the test has a longer name).
#[test]
fn dyn_matrix() {
    check("dyn");
}

// The native-module rows of the second run, and capabilities.
cases!(ui_dep, eval_nocap, eval_cap);

#[test]
fn bom_entry_keeps_the_mark_as_a_fact_of_the_file() {
    let stack = stack_of(&["bom_entry"]);
    let bundle = bundle_of(&stack, "bom_entry");
    let main = bundle.main().expect("the main file");
    assert!(main.content.as_ref().unwrap().bom);
    assert_eq!(main.content.as_ref().unwrap().utf8, Ok(()));
    // The first line's columns do not count the mark: `"@minecraft/server"` opens at 23.
    assert_eq!((main.imports()[0].line, main.imports()[0].column), (1, 23));
    assert_eq!(main.imports()[0].offset, 25);
}

#[test]
fn latin1_entry_names_the_byte() {
    let stack = stack_of(&["latin1_entry"]);
    let main = bundle_of(&stack, "latin1_entry")
        .main()
        .expect("the main file");
    assert!(!main.content.as_ref().unwrap().bom);
    // `console.warn("[latin1_entry] caf` is 32 bytes.
    assert_eq!(
        main.content
            .as_ref()
            .unwrap()
            .utf8
            .map_err(|error| (error.offset, error.line)),
        Err((32, 1))
    );
    assert!(main.imports().is_empty());
}

/// `bigfile`: a 20 MiB comment in a file the entry imports. The file is built here, not kept
/// in the repository.
// [bds-run 2] [bigfile] OK ./big.js -> big ok.
#[test]
fn bigfile() {
    let mut big = b"/*".to_vec();
    big.resize(20 * 1024 * 1024, b'x');
    big.extend_from_slice(b"*/\nexport const v = \"big ok\";\n");
    let stack = script_stack(&[
        (
            "scripts/main.js",
            b"import(\"./big.js\").then((m) => console.warn(m.v));\n",
        ),
        ("scripts/big.js", &big),
    ]);
    let bundle = &stack.scripts()[0];
    assert_eq!(bundle.findings().count(), 0);
    let big_js = &bundle.files[&ScriptPath::new("big.js")];
    assert_eq!(big_js.content.as_ref().unwrap().size, big.len() as u64);
    assert_eq!(big_js.content.as_ref().unwrap().syntax, None);
    let graph = bundle.graph.as_ref().expect("the graph");
    assert!(graph.reachable_dynamic.contains(&ScriptPath::new("big.js")));
}

/// `fail_with_function` / `syntax_with_function`: the plugin fails, the pack's data part is
/// intact.
// [bds-run 2] both `function` commands ran: `Successfully executed 1 function entries.`
#[cfg(feature = "function")]
#[test]
fn fail_with_function() {
    use bedrock_addon::id::FunctionPath;
    let stack = stack_of(&["fail_with_function"]);
    check_in(&stack, "fail_with_function");
    let function = stack
        .functions()
        .get(&FunctionPath::parse("sbprobe_fn"))
        .expect("the function of the pack whose plugin failed");
    assert_eq!(function.commands.len(), 1);
    assert_eq!(stack.functions().diagnostics().count(), 0);
}

#[cfg(feature = "function")]
#[test]
fn syntax_with_function() {
    use bedrock_addon::id::FunctionPath;
    let stack = stack_of(&["syntax_with_function"]);
    check_in(&stack, "syntax_with_function");
    assert!(
        stack
            .functions()
            .get(&FunctionPath::parse("sbprobe_fn2"))
            .is_some()
    );
}

/// Without the `function` reader the two packs open alike and their bundles are the same:
/// the bundle reads nothing of `functions/`.
#[cfg(not(feature = "function"))]
#[test]
fn fail_with_function_has_its_bundle_without_the_function_reader() {
    check("fail_with_function");
}

#[cfg(not(feature = "function"))]
#[test]
fn syntax_with_function_has_its_bundle_without_the_function_reader() {
    check("syntax_with_function");
}

/// Cross-pack: `libpack` (data module with `scripts/libx.js`) below `consumer`, which declares
/// a pack dependency on it: none of the three specifiers reaches the other pack's file.
#[test]
fn libpack_and_consumer() {
    for order in [["consumer", "libpack"], ["libpack", "consumer"]] {
        let stack = stack_of(&order);
        check_in(&stack, "consumer");
        check_in(&stack, "libpack");
        // One bundle per pack, the highest pack first.
        let names: Vec<&str> = stack.scripts().iter().map(|b| b.pack.name()).collect();
        assert_eq!(names, order.map(|case| format!("SB_{case}")));
    }
}

/// Cross-pack: `scriptlib` (a script pack) and `consumer2`, which depends on it.
#[test]
fn scriptlib_and_consumer2() {
    for order in [["consumer2", "scriptlib"], ["scriptlib", "consumer2"]] {
        let stack = stack_of(&order);
        check_in(&stack, "consumer2");
        check_in(&stack, "scriptlib");
    }
}

/// Three packs: a file of the same name in a higher and in a lower pack never satisfies the
/// import of the pack between them, and each pack keeps its own bundle.
#[test]
fn no_import_crosses_packs_in_a_stack_of_three() {
    let pack = |name: &str, n: u128, files: &[(&str, &[u8])]| {
        let mut all = vec![(
            "manifest.json".to_owned(),
            script_manifest(name, n, "scripts/main.js"),
        )];
        all.extend(files.iter().map(|(p, b)| ((*p).to_owned(), b.to_vec())));
        pack_of(all, PackOrigin::User)
    };
    let lib: (&str, &[u8]) = ("scripts/lib.js", b"export const v = 1;\n");
    let stack = PackStack::<Behavior>::of(
        [
            pack(
                "top",
                10,
                &[("scripts/main.js", b"import \"./lib.js\";\n"), lib],
            ),
            pack(
                "middle",
                20,
                &[("scripts/main.js", b"import \"./lib.js\";\n")],
            ),
            pack(
                "bottom",
                30,
                &[("scripts/main.js", b"import \"./lib.js\";\n"), lib],
            ),
        ],
        LoadOptions::default(),
    )
    .expect("behaviour packs");
    let bundles = stack.scripts();
    let names: Vec<&str> = bundles.iter().map(|b| b.pack.name()).collect();
    assert_eq!(names, ["top", "middle", "bottom"]);
    let first = |index: usize| &bundles[index].main().expect("main").imports()[0].resolution;
    assert_eq!(first(0), &Resolution::File(ScriptPath::new("lib.js")));
    assert!(matches!(first(1), Resolution::NotFound { looked_up, .. } if looked_up == "lib.js"));
    assert_eq!(first(2), &Resolution::File(ScriptPath::new("lib.js")));
    let errors = |index: usize| {
        bundles[index]
            .findings()
            .filter(|finding| finding.severity == Severity::Error)
            .count()
    };
    assert_eq!([errors(0), errors(1), errors(2)], [0, 1, 0]);
    let finding = bundles[1].findings().next().expect("the finding");
    assert_eq!(
        finding
            .code
            .args_of(&bedrock_addon::diagnostic::codes::SCRIPT_IMPORT_NOT_FOUND),
        Some(&["lib.js".to_owned()][..])
    );
    assert!(finding.detail.is_none());
    assert_eq!(
        finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name),
        Some("middle")
    );
}

/// `brarchive_zero_size`: the layout of the BDS `server_library` pack, loose `scripts/*.js`
/// beside a `__brarchive/scripts.brarchive` whose entries all have size 0: the members'
/// content is the loose files.
// [bds-1.26.36.1] the library packs; [bds-run] the wrappers load with their full content.
#[test]
fn brarchive_zero_size() {
    let main: &[u8] = b"import { v } from \"./lib.js\";\nconsole.warn(v);\n";
    let lib: &[u8] = b"export const v = \"loose\";\n";
    let archive = brarchive::serialize([("main.js", &b""[..]), ("lib.js", &b""[..])]).unwrap();
    let stack = script_stack(&[
        ("scripts/main.js", main),
        ("scripts/lib.js", lib),
        ("__brarchive/scripts.brarchive", &archive),
    ]);
    let bundle = &stack.scripts()[0];
    let names: Vec<&str> = bundle.files.keys().map(ScriptPath::as_str).collect();
    assert_eq!(names, ["lib.js", "main.js"]);
    assert_eq!(
        bundle.files[&ScriptPath::new("main.js")]
            .content
            .as_ref()
            .unwrap()
            .size,
        main.len() as u64
    );
    assert_eq!(
        bundle.files[&ScriptPath::new("lib.js")]
            .content
            .as_ref()
            .unwrap()
            .size,
        lib.len() as u64
    );
    assert_eq!(
        bundle.main().expect("main").imports()[0].resolution,
        Resolution::File(ScriptPath::new("lib.js"))
    );
    assert_eq!(bundle.findings().count(), 0);
}

/// An archive entry with content is a member like a loose file, and stands in for the loose
/// file of its name (the stack's archive layer).
#[test]
fn a_brarchive_entry_with_content_is_a_member() {
    let archive = brarchive::serialize([
        ("main.js", &b"import \"./only_archived.js\";\n"[..]),
        ("only_archived.js", &b"export {};\n"[..]),
    ])
    .unwrap();
    let stack = script_stack(&[
        ("scripts/main.js", b"import \"./missing.js\";\n"),
        ("__brarchive/scripts.brarchive", &archive),
    ]);
    let bundle = &stack.scripts()[0];
    let names: Vec<&str> = bundle.files.keys().map(ScriptPath::as_str).collect();
    assert_eq!(names, ["main.js", "only_archived.js"]);
    assert_eq!(
        bundle.main().expect("main").imports()[0].resolution,
        Resolution::File(ScriptPath::new("only_archived.js"))
    );
    assert_eq!(bundle.findings().count(), 0);
}

/// Every fixture pack has a test above: a pack added to the folder without one fails here.
#[test]
fn every_fixture_pack_has_its_test() {
    let source = include_str!("cases.rs");
    let dir = super::common::fixtures().join("packs");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(!case_files(&name).is_empty(), "{name} is empty");
        let named = source.contains(&format!("    {name},"))
            || source.contains(&format!("\"{name}\""))
            || source.contains(&format!("({name},"))
            || source.contains(&format!(" {name})"))
            || source.contains(&format!(" {name},"));
        if !named {
            missing.push(name);
        }
    }
    assert_eq!(missing, Vec::<String>::new());
}
