//! The validation rules that no fixture pack reaches, and both
//! sides of the ones a fixture shows one side of: severities by reachability, the hints, the
//! native-module answers of a host, `ScriptBundle::scan` outside a stack.

use std::fmt::Write as _;

use bedrock_addon::diagnostic::{DiagnosticCode, DiagnosticDetail, NotFoundHint, Severity};
use bedrock_addon::families::script::{
    ImportKind, NativeModuleAccess, Resolution, ScanOptions, ScriptBundle, ScriptHost, ScriptPath,
    ScriptRoot, ScriptSyntaxKind,
};
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

use super::common::{Finding, finding_of};

/// A script module written by the test.
struct Host {
    entry: Option<&'static str>,
    available: &'static [&'static str],
    restricted: &'static [&'static str],
    script_eval: bool,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            entry: Some("scripts/main.js"),
            available: &["@minecraft/server"],
            restricted: &[],
            script_eval: false,
        }
    }
}

impl ScriptHost for Host {
    fn plugin(&self) -> &'static str {
        "probe - 1.0.0"
    }
    fn entry(&self) -> Option<&str> {
        self.entry
    }
    fn native(&self, name: &str) -> NativeModuleAccess {
        if self.available.contains(&name) {
            NativeModuleAccess::Available
        } else if self.restricted.contains(&name) {
            NativeModuleAccess::Restricted
        } else {
            NativeModuleAccess::Unknown
        }
    }
    fn script_eval(&self) -> bool {
        self.script_eval
    }
}

/// The bundle of `files` (paths in the pack) run by `host`.
fn scan_with(
    files: &[(&str, &[u8])],
    host: Option<&dyn ScriptHost>,
    options: ScanOptions,
) -> ScriptBundle {
    let files: Vec<(PackPath, &[u8])> = files
        .iter()
        .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), *bytes))
        .collect();
    ScriptBundle::scan(
        files.iter().map(|(path, bytes)| (path, *bytes)),
        host,
        options,
    )
}

fn scan(files: &[(&str, &[u8])]) -> ScriptBundle {
    scan_with(files, Some(&Host::default()), ScanOptions::default())
}

/// The findings of `bundle` as `(code, severity, file)`.
fn findings(bundle: &ScriptBundle) -> Vec<(String, String, String)> {
    bundle
        .findings()
        .map(finding_of)
        .map(
            |Finding {
                 code,
                 severity,
                 file,
                 ..
             }| (code, severity, file),
        )
        .collect()
}

fn found(code: &str, severity: &str, file: &str) -> (String, String, String) {
    (code.to_owned(), severity.to_owned(), file.to_owned())
}

const SYNTAX: &str = "ScriptSyntaxError";
const NOT_FOUND: &str = "ScriptImportNotFound";

#[test]
fn script_files_are_the_dot_js_files_below_scripts() {
    let name = |path: &str| {
        ScriptPath::of_pack_path(&PackPath::new(path).unwrap()).map(|name| name.as_str().to_owned())
    };
    assert_eq!(name("scripts/main.js").as_deref(), Some("main.js"));
    assert_eq!(name("scripts/lib/a.b.js").as_deref(), Some("lib/a.b.js"));
    assert_eq!(name("scripts/.hidden.js").as_deref(), Some(".hidden.js"));
    for other in [
        "scripts/main.JS",
        "scripts/main.mjs",
        "scripts/main.cjs",
        "scripts/main.ts",
        "scripts/data.json",
        "scripts/main.js.map",
        "scripts/noext",
        "scripts/js",
        "main.js",
        "other/scripts/main.js",
        "Scripts/main.js",
    ] {
        assert_eq!(name(other), None, "{other}");
    }
    assert_eq!(ScriptRoot.as_str(), "scripts/");
    assert_eq!(ScriptRoot.to_string(), "scripts/");
}

#[test]
fn a_bundle_outside_a_stack_sorts_its_files_and_is_no_pack_s() {
    let bundle = scan(&[
        ("scripts/z.js", b""),
        ("scripts/main.js", b"import \"./z.js\";"),
        ("scripts/notes.txt", b"n"),
        ("scripts/a.ts", b"t"),
        ("textures/x.png", b"p"),
    ]);
    let names: Vec<&str> = bundle.files.keys().map(ScriptPath::as_str).collect();
    assert_eq!(names, ["main.js", "z.js"]);
    let others: Vec<&str> = bundle.others.iter().map(PackPath::as_str).collect();
    assert_eq!(others, ["scripts/a.ts", "scripts/notes.txt"]);
    assert_eq!(bundle.pack, PackRef::app_data());
    assert_eq!(findings(&bundle), []);
    assert_eq!(
        bundle.files[&ScriptPath::new("z.js")].path.as_str(),
        "scripts/z.js"
    );
}

#[test]
fn l13_a_non_literal_dynamic_import_is_a_note() {
    let bundle = scan(&[(
        "scripts/main.js",
        b"const n = \"./a.js\";\nimport(n);\nimport(\"./\" + n);\n",
    )]);
    assert_eq!(
        findings(&bundle),
        [
            found("ScriptImportNotChecked", "info", "scripts/main.js"),
            found("ScriptImportNotChecked", "info", "scripts/main.js"),
        ]
    );
    let main = bundle.main().unwrap();
    assert_eq!(main.imports()[0].kind, ImportKind::DynamicNonLiteral);
    assert_eq!(main.imports()[0].resolution, Resolution::NotChecked);
    assert_eq!((main.imports()[0].line, main.imports()[0].column), (2, 1));
    assert_eq!((main.imports()[1].line, main.imports()[1].column), (3, 1));
    // The span of a finding is its site's offset.
    let offsets: Vec<u32> = bundle
        .findings()
        .map(|finding| finding.location.span.expect("a span").offset)
        .collect();
    assert_eq!(offsets, [20, 31]);
}

#[test]
fn l15_a_js_file_outside_scripts_is_a_note_per_file_of_a_pack_with_a_module() {
    let files: [(&str, &[u8]); 4] = [
        ("scripts/main.js", b""),
        ("src/a.js", b""),
        ("b.js", b""),
        ("src/c.JS", b""),
    ];
    let with = scan(&files);
    assert_eq!(
        findings(&with),
        [
            found("ScriptFileNotLoadable", "info", "b.js"),
            found("ScriptFileNotLoadable", "info", "src/a.js"),
        ]
    );
    // Without a module nothing runs at all: one note for the pack (L20), none per file.
    let without = scan_with(&files, None, ScanOptions::default());
    assert_eq!(
        findings(&without),
        [found("ScriptFileNotLoadable", "info", "")]
    );
}

#[test]
fn l20_is_one_note_however_many_files_and_none_without_script_files() {
    let two = scan_with(
        &[("scripts/a.js", b""), ("scripts/b.js", b"")],
        None,
        ScanOptions::default(),
    );
    assert_eq!(findings(&two), [found("ScriptFileNotLoadable", "info", "")]);
    assert_eq!(two.entry, None);
    assert_eq!(two.graph, None);
    let none = scan_with(&[("scripts/readme.txt", b"")], None, ScanOptions::default());
    assert_eq!(findings(&none), []);
}

#[test]
fn the_parser_reads_only_the_files_the_entry_reaches() {
    // M5 of the review. `x = ;` scans whole in the lexer: only the parser of `script-syntax`
    // refuses it.
    let files: [(&str, &[u8]); 4] = [
        (
            "scripts/main.js",
            b"import \"./a.js\";\nimport(\"./b.js\");\n",
        ),
        ("scripts/a.js", b"export const a = 1;\n"),
        ("scripts/b.js", b"x = ;\n"),
        ("scripts/c.js", b"x = ;\n"),
    ];
    let parsed = |bundle: &ScriptBundle| -> Vec<bool> {
        bundle
            .files
            .values()
            .map(|file| file.content.as_ref().unwrap().parsed)
            .collect()
    };
    let checked = ScriptBundle::SYNTAX_CHECKED;
    // The entry and what it reaches, statically or through a literal `import()`; not `c.js`,
    // which nothing imports and vanilla never reads.
    let runs = scan(&files);
    assert_eq!(parsed(&runs), [checked, checked, false, checked]);
    let unreached = &runs.files[&ScriptPath::new("c.js")];
    assert_eq!(unreached.content.as_ref().unwrap().syntax, None);
    let expected = if checked {
        vec![found(SYNTAX, "info", "scripts/b.js")]
    } else {
        Vec::new()
    };
    assert_eq!(findings(&runs), expected);
    // A pack without a script module runs nothing, and nothing of it is parsed.
    let data_only = scan_with(&files, None, ScanOptions::default());
    assert_eq!(parsed(&data_only), [false; 4]);
    assert_eq!(
        findings(&data_only),
        [found("ScriptFileNotLoadable", "info", "")]
    );
    // Nor where the main file is missing, or the imports are not scanned.
    let no_main = scan_with(&files[1..], Some(&Host::default()), ScanOptions::default());
    assert_eq!(parsed(&no_main), [false; 3]);
    let off = scan_with(
        &files,
        Some(&Host::default()),
        ScanOptions::default().with_imports(false),
    );
    assert_eq!(parsed(&off), [false; 4]);
}

#[test]
fn a_bundle_says_once_how_many_files_the_parser_was_kept_from() {
    // M4 of the review. Two files the entry reaches nest more deeply than the parser has
    // stack for; a third that does is not reached.
    let deep = format!("x = {}1{};\n", "(".repeat(1_000), ")".repeat(1_000));
    let main = format!("import \"./a.js\";\nimport \"./b.js\";\n{deep}");
    let files: [(&str, &[u8]); 4] = [
        ("scripts/main.js", main.as_bytes()),
        ("scripts/a.js", deep.as_bytes()),
        ("scripts/b.js", b"export const b = 1;\n"),
        ("scripts/c.js", deep.as_bytes()),
    ];
    let bundle = scan(&files);
    let parsed: Vec<bool> = bundle
        .files
        .values()
        .map(|file| file.content.as_ref().unwrap().parsed)
        .collect();
    let checked = ScriptBundle::SYNTAX_CHECKED;
    assert_eq!(parsed, [false, checked, false, false]);
    let notes: Vec<_> = bundle
        .findings()
        .map(|finding| {
            (
                finding.code.clone(),
                finding.severity,
                finding.location.path.clone(),
            )
        })
        .collect();
    let expected = if checked {
        vec![(
            DiagnosticCode::from(
                bedrock_addon::diagnostic::codes::SCRIPT_SYNTAX_NOT_CHECKED.with([2.to_string()]),
            ),
            Severity::Info,
            None,
        )]
    } else {
        // Without the feature no file is parsed, which `SYNTAX_CHECKED` says of the build.
        Vec::new()
    };
    assert_eq!(notes, expected);
    assert_eq!(
        DiagnosticCode::from(
            bedrock_addon::diagnostic::codes::SCRIPT_SYNTAX_NOT_CHECKED.with([2.to_string()])
        )
        .to_string(),
        "syntax of 2 script files not checked: nested too deeply for the parser"
    );
    // A bundle whose files are all read says nothing.
    let flat = scan(&[("scripts/main.js", b"x = (1);\n")]);
    assert_eq!(findings(&flat), []);
}

#[test]
fn l19_scanning_off_says_so_once_and_resolves_nothing() {
    let files: [(&str, &[u8]); 2] = [
        (
            "scripts/main.js",
            b"import \"./missing.js\";\nimport(\"./\");\n",
        ),
        ("scripts/other.js", b"import \"./missing.js\";\n"),
    ];
    let host = Host::default();
    let off = scan_with(
        &files,
        Some(&host),
        ScanOptions::default().with_imports(false),
    );
    assert_eq!(findings(&off), [found("ScriptsNotChecked", "info", "")]);
    assert_eq!(off.graph, None);
    assert!(off.entry.is_some());
    assert!(
        off.imports()
            .all(|(_, site)| site.resolution == Resolution::NotChecked)
    );
    // On, the same files have a graph and their findings, and no such note.
    assert_eq!(
        ScanOptions::default(),
        ScanOptions::default().with_imports(true)
    );
    let on = scan_with(&files, Some(&host), ScanOptions::default());
    assert!(on.graph.is_some());
    assert_eq!(
        findings(&on),
        [
            found(NOT_FOUND, "error", "scripts/main.js"),
            found("ScriptImportCrashesEngine", "error", "scripts/main.js"),
        ]
    );
}

/// A file with something wrong in it, reached statically, through `import()` only, and not
/// at all.
fn by_reach(broken: &[u8]) -> Vec<(String, String, String)> {
    let bundle = scan(&[
        (
            "scripts/main.js",
            b"import \"./s.js\";\nimport(\"./d.js\");\n",
        ),
        ("scripts/s.js", broken),
        ("scripts/d.js", broken),
        ("scripts/u.js", broken),
    ]);
    let graph = bundle.graph.as_ref().expect("a graph");
    let names = |set: &std::collections::BTreeSet<ScriptPath>| -> Vec<String> {
        set.iter().map(|n| n.as_str().to_owned()).collect()
    };
    assert_eq!(names(&graph.reachable_static), ["main.js", "s.js"]);
    assert_eq!(names(&graph.reachable_dynamic), ["d.js"]);
    assert!(graph.reaches(&ScriptPath::new("d.js")));
    assert!(graph.reaches(&ScriptPath::new("s.js")));
    assert!(!graph.reaches(&ScriptPath::new("u.js")));
    findings(&bundle)
}

#[test]
fn l6_l7_invalid_utf8_is_an_error_where_the_entry_reaches_and_a_note_elsewhere_deviation() {
    assert_eq!(
        by_reach(b"export const v = \"\xff\";"),
        [
            found(SYNTAX, "error", "scripts/d.js"),
            found(SYNTAX, "error", "scripts/s.js"),
            found(SYNTAX, "info", "scripts/u.js"),
        ]
    );
}

/// The findings about the syntax of `file` in `bundle`: severity and text, in order.
fn syntax_of(bundle: &ScriptBundle, file: &str) -> Vec<(String, String)> {
    bundle
        .findings()
        .map(finding_of)
        .filter(|finding| finding.code == SYNTAX && finding.file == file)
        .map(|finding| (finding.severity, finding.text))
        .collect()
}

// K124: the engine's sentence for invalid UTF-8 was probed once, for a main file with the
// byte on line 1. A file nothing imports is never read by the engine, so
// the note about it is ours and does not quote the engine.
#[test]
fn l7_a_file_nothing_reaches_does_not_carry_the_engine_s_sentence() {
    let bundle = scan(&[
        ("scripts/main.js", b"import \"@minecraft/server\";\n"),
        ("scripts/sub/u.js", b"// one\n// two\nconst v = \"\xff\";\n"),
    ]);
    assert_eq!(
        syntax_of(&bundle, "scripts/sub/u.js"),
        [(
            "info".to_owned(),
            "sub/u.js is not UTF-8 at line 3; nothing imports the file, so the engine never \
             reads it"
                .to_owned()
        )]
    );
}

// K124: for a file the entry reaches the sentence is the engine's. That it names a member of
// a sub-folder by its path below `scripts/`, and a line other than 1 by the line of the bad
// byte, is inferred from the one probe (`main.js:1`).
#[test]
fn l6_names_the_member_below_scripts_and_the_line_of_the_byte_inferred() {
    let bundle = scan(&[
        ("scripts/main.js", b"import \"./sub/s.js\";\n"),
        ("scripts/sub/s.js", b"// one\n// two\nconst v = \"\xff\";\n"),
    ]);
    assert_eq!(
        syntax_of(&bundle, "scripts/sub/s.js"),
        [(
            "error".to_owned(),
            "SyntaxError: invalid UTF-8 sequence    at sub/s.js:3".to_owned()
        )]
    );
}

// K124: the engine stops at a file's first error. A string its line ends inside, before the
// byte that is no UTF-8, is that first error: the file gets the syntax finding, and the
// finding about the byte does not quote a sentence the engine would not print.
#[test]
fn l6_behind_an_earlier_syntax_error_does_not_carry_the_engine_s_sentence() {
    let bundle = scan(&[
        ("scripts/main.js", b"import \"./s.js\";\n"),
        ("scripts/s.js", b"const a = \"open\nconst v = \"\xff\";\n"),
    ]);
    assert_eq!(
        syntax_of(&bundle, "scripts/s.js"),
        [
            (
                "error".to_owned(),
                "s.js is not UTF-8 at line 2; a syntax error comes before it".to_owned()
            ),
            (
                "error".to_owned(),
                "syntax error in a script file".to_owned()
            ),
        ]
    );
    let content = bundle.files[&ScriptPath::new("s.js")]
        .content
        .as_ref()
        .unwrap();
    assert_eq!(
        content
            .syntax
            .as_ref()
            .map(|syntax| (&syntax.kind, syntax.at.offset)),
        Some((&ScriptSyntaxKind::UnterminatedString, 10))
    );
    assert_eq!(
        content.utf8.map_err(|error| (error.offset, error.line)),
        Err((27, 2))
    );
    // A string, a template or a comment still open where the bytes stop being UTF-8 is no
    // error of its own: the engine's first error is the byte.
    for open in [
        &b"const v = \"\xff\";\n"[..],
        b"const v = `one\ntwo \xff`;\n",
        b"/* one\n \xff */\n",
    ] {
        let bundle = scan(&[
            ("scripts/main.js", b"import \"./s.js\";\n"),
            ("scripts/s.js", open),
        ]);
        let told = syntax_of(&bundle, "scripts/s.js");
        assert_eq!(told.len(), 1, "{told:?}");
        assert!(
            told[0]
                .1
                .starts_with("SyntaxError: invalid UTF-8 sequence    at s.js:"),
            "{told:?}"
        );
    }
}

#[test]
fn a_lexical_error_is_an_error_in_the_static_graph_and_a_note_elsewhere() {
    assert_eq!(
        by_reach(b"export const v = \"open\n"),
        [
            found(SYNTAX, "info", "scripts/d.js"),
            found(SYNTAX, "error", "scripts/s.js"),
            found(SYNTAX, "info", "scripts/u.js"),
        ]
    );
    let bundle = scan(&[("scripts/main.js", b"import \"./a.js\";\n/* open")]);
    let syntax = bundle
        .main()
        .unwrap()
        .content
        .as_ref()
        .unwrap()
        .syntax
        .clone()
        .expect("the error");
    assert_eq!(syntax.kind, ScriptSyntaxKind::UnterminatedComment);
    assert_eq!(
        (syntax.at.line, syntax.at.column, syntax.at.offset),
        (2, 1, 17)
    );
    // The site before the error is kept and resolved.
    assert_eq!(bundle.main().unwrap().imports().len(), 1);
}

#[test]
fn l12_attributes_fail_the_file_they_are_in_by_its_reach() {
    assert_eq!(
        by_reach(b"import \"@minecraft/server\" with { type: \"json\" };"),
        [
            found(SYNTAX, "info", "scripts/d.js"),
            found(SYNTAX, "error", "scripts/s.js"),
        ]
    );
    // Without attributes the same import is nothing.
    assert_eq!(by_reach(b"import \"@minecraft/server\";"), []);
}

#[test]
fn l8_l9_a_missing_import_is_an_error_when_static_in_the_static_graph_and_a_note_otherwise() {
    assert_eq!(
        by_reach(b"import \"./gone.js\";\nimport(\"./gone.js\");\nexport * from \"./gone2.js\";\n"),
        [
            // Reached through `import()` only: all three reject that one promise.
            found(NOT_FOUND, "info", "scripts/d.js"),
            found(NOT_FOUND, "info", "scripts/d.js"),
            found(NOT_FOUND, "info", "scripts/d.js"),
            // Linked with the entry: the static import and the re-export fail the plugin.
            found(NOT_FOUND, "error", "scripts/s.js"),
            found(NOT_FOUND, "info", "scripts/s.js"),
            found(NOT_FOUND, "error", "scripts/s.js"),
        ]
    );
}

#[test]
fn l10_l11_unknown_and_restricted_modules_by_how_they_are_imported() {
    let host = Host {
        restricted: &["@minecraft/server-bindings"],
        ..Host::default()
    };
    let source = b"import \"@minecraft/nope\";\nimport(\"@minecraft/nope\");\n\
        import \"@minecraft/server-bindings\";\nimport(\"@minecraft/server-bindings\");\n\
        import \"@minecraft/server\";\n";
    let bundle = scan_with(
        &[("scripts/main.js", source)],
        Some(&host),
        ScanOptions::default(),
    );
    assert_eq!(
        findings(&bundle),
        [
            found("ScriptModuleUnrecognized", "error", "scripts/main.js"),
            found("ScriptModuleUnrecognized", "info", "scripts/main.js"),
            found("ScriptModuleRestricted", "error", "scripts/main.js"),
            found("ScriptModuleRestricted", "info", "scripts/main.js"),
        ]
    );
    let restricted = bundle.findings().nth(2).unwrap();
    assert_eq!(
        restricted.code,
        DiagnosticCode::from(
            bedrock_addon::diagnostic::codes::SCRIPT_MODULE_RESTRICTED
                .with(["@minecraft/server-bindings", "main.js"])
        )
    );
    assert_eq!(
        bundle.main().unwrap().imports()[4].resolution,
        Resolution::Native {
            name: "@minecraft/server".to_owned()
        }
    );
}

#[test]
fn a_native_name_wins_over_a_file_and_an_at_name_falls_back_to_a_file() {
    let host = Host {
        available: &["@minecraft/server", "plain-native"],
        ..Host::default()
    };
    let bundle = scan_with(
        &[
            (
                "scripts/main.js",
                b"import \"plain-native\";\nimport \"@scope/file.js\";\nimport \"@scope/gone.js\";\nimport \"\";\n",
            ),
            ("scripts/plain-native.js", b""),
            ("scripts/@scope/file.js", b""),
        ],
        Some(&host),
        ScanOptions::default(),
    );
    let resolutions: Vec<&Resolution> = bundle
        .main()
        .unwrap()
        .imports()
        .iter()
        .map(|site| &site.resolution)
        .collect();
    assert_eq!(
        resolutions,
        [
            // The exact name of a native module is returned as written, before any file.
            &Resolution::Native {
                name: "plain-native".to_owned()
            },
            // An `@` name the context does not have is loaded as a file of that exact name.
            &Resolution::File(ScriptPath::new("@scope/file.js")),
            &Resolution::Unrecognized {
                name: "@scope/gone.js".to_owned()
            },
            // The empty specifier is looked up as the empty name.
            &Resolution::NotFound {
                looked_up: String::new(),
                hint: None,
                near: None
            },
        ]
    );
}

#[test]
fn l16_l17_are_raised_in_reached_files_only() {
    let source: &[u8] = b"eval(\"1\");\nimport.meta.url;\nimport.meta;\n";
    let files: [(&str, &[u8]); 3] = [
        ("scripts/main.js", b"import \"./s.js\";\n"),
        ("scripts/s.js", source),
        ("scripts/u.js", source),
    ];
    let bundle = scan(&files);
    assert_eq!(
        findings(&bundle),
        [
            found("ScriptEvalWithoutCapability", "info", "scripts/s.js"),
            found("ScriptImportMetaEmpty", "info", "scripts/s.js"),
        ]
    );
    let capable = Host {
        script_eval: true,
        ..Host::default()
    };
    let bundle = scan_with(&files, Some(&capable), ScanOptions::default());
    assert_eq!(
        findings(&bundle),
        [found("ScriptImportMetaEmpty", "info", "scripts/s.js")]
    );
}

#[test]
fn l18_a_static_import_of_an_overlong_path_is_an_error_too() {
    let long = "a".repeat(1100);
    let source = format!("import \"{long}\";\nimport \"./{long}\";\n");
    let bundle = scan(&[("scripts/main.js", source.as_bytes())]);
    assert_eq!(
        findings(&bundle),
        [
            found(NOT_FOUND, "error", "scripts/main.js"),
            found(NOT_FOUND, "error", "scripts/main.js"),
        ]
    );
    let looked_up: Vec<String> = bundle
        .findings()
        .map(|finding| {
            match finding
                .code
                .args_of(&bedrock_addon::diagnostic::codes::SCRIPT_IMPORT_NOT_FOUND)
            {
                Some([specifier]) => specifier.clone(),
                _ => panic!("{:?}", finding.code),
            }
        })
        .collect();
    assert_eq!(looked_up, [".js", ""]);
    // One byte under the limit is an ordinary missing import: an error.
    let fits = "a".repeat(1023);
    let source = format!("import \"{fits}\";\n");
    let bundle = scan(&[("scripts/main.js", source.as_bytes())]);
    assert_eq!(
        findings(&bundle),
        [found(NOT_FOUND, "error", "scripts/main.js")]
    );
}

#[test]
fn l21_a_crash_specifier_is_an_error_in_a_file_nothing_reaches() {
    let crashing: &[u8] =
        b"import(\"..\");\nexport * from \"./\";\nimport \"./ok.js\";\nimport \"./gone.js\";\n";
    let files: [(&str, &[u8]); 3] = [
        ("scripts/main.js", b""),
        ("scripts/unreached.js", crashing),
        ("scripts/ok.js", b""),
    ];
    let bundle = scan(&files);
    assert_eq!(
        findings(&bundle),
        [
            found("ScriptImportCrashesEngine", "error", "scripts/unreached.js"),
            found("ScriptImportCrashesEngine", "error", "scripts/unreached.js"),
        ]
    );
    let sites = &bundle.files[&ScriptPath::new("unreached.js")].imports();
    assert_eq!(sites[0].resolution, Resolution::CrashesEngine);
    assert_eq!(sites[1].resolution, Resolution::CrashesEngine);
    // The other imports of a file nothing reaches are not looked at, missing or not.
    assert_eq!(sites[2].resolution, Resolution::NotChecked);
    assert_eq!(sites[3].resolution, Resolution::NotChecked);
    // In a pack without a module nothing runs: the one note of L20.
    let without = scan_with(&files, None, ScanOptions::default());
    assert_eq!(
        findings(&without),
        [found("ScriptFileNotLoadable", "info", "")]
    );
}

#[test]
fn every_hint_and_the_file_it_points_at() {
    let specs = [
        (
            "scripts/lib/b.js",
            Some(NotFoundHint::ScriptsPrefix),
            Some("lib/b.js"),
        ),
        (
            "scripts/lib/nope.js",
            Some(NotFoundHint::ScriptsPrefix),
            None,
        ),
        ("../up.js", Some(NotFoundHint::EscapesRoot), None),
        (
            "./LIB/B.js",
            Some(NotFoundHint::CaseMismatch),
            Some("lib/b.js"),
        ),
        (
            "./dir",
            Some(NotFoundHint::DirectoryIndex),
            Some("dir/index.js"),
        ),
        (
            "./dir.js",
            Some(NotFoundHint::DirectoryIndex),
            Some("dir/index.js"),
        ),
        ("./conf.json", Some(NotFoundHint::ImportedAsJson), None),
        ("./mod.mjs", Some(NotFoundHint::AppendedJs), None),
        ("./lib/../lib/b.js", Some(NotFoundHint::MidPathDots), None),
        ("lodash", Some(NotFoundHint::BarePackage), None),
        // A bare path with a folder is a path, not a package; a relative miss is just missing.
        ("lib/nope.js", None, None),
        ("./nope.js", None, None),
        ("./nope", None, None),
    ];
    let mut source = String::new();
    for (spec, _, _) in specs {
        writeln!(source, "import(\"{spec}\");").unwrap();
    }
    let bundle = scan(&[
        ("scripts/main.js", source.as_bytes()),
        ("scripts/lib/b.js", b""),
        ("scripts/dir/index.js", b""),
    ]);
    for ((spec, hint, near), site) in specs.iter().zip(bundle.main().unwrap().imports()) {
        match &site.resolution {
            Resolution::NotFound {
                hint: found,
                near: file,
                ..
            } => {
                assert_eq!(found, hint, "{spec}");
                assert_eq!(file.as_ref().map(ScriptPath::as_str), *near, "{spec}");
            }
            other => panic!("{spec}: {other:?}"),
        }
    }
    // The finding carries the hint; its text is vanilla's alone.
    let first = bundle.findings().next().unwrap();
    assert_eq!(
        first.code,
        DiagnosticCode::from(
            bedrock_addon::diagnostic::codes::SCRIPT_IMPORT_NOT_FOUND.with(["scripts/lib/b.js"])
        )
    );
    assert_eq!(
        first.detail.as_deref(),
        Some(&DiagnosticDetail::NotFound(NotFoundHint::ScriptsPrefix))
    );
    assert_eq!(
        first.code.to_string(),
        "ReferenceError: Import [scripts/lib/b.js] not found."
    );
    assert_eq!(first.severity, Severity::Info);
}

#[test]
fn re_exports_link_statically_and_a_dynamic_file_s_static_imports_stay_dynamic() {
    let bundle = scan(&[
        (
            "scripts/main.js",
            b"export * from \"./a.js\";\nimport(\"./d.js\");\n",
        ),
        ("scripts/a.js", b"export { x } from \"./b.js\";\n"),
        (
            "scripts/b.js",
            b"export const x = 1;\nimport(\"./a.js\");\n",
        ),
        ("scripts/d.js", b"import \"./e.js\";\nimport \"./b.js\";\n"),
        ("scripts/e.js", b"import(\"./f.js\");\n"),
        ("scripts/f.js", b""),
        ("scripts/g.js", b""),
    ]);
    let graph = bundle.graph.as_ref().expect("a graph");
    let names = |set: &std::collections::BTreeSet<ScriptPath>| -> Vec<String> {
        set.iter().map(|n| n.as_str().to_owned()).collect()
    };
    assert_eq!(names(&graph.reachable_static), ["a.js", "b.js", "main.js"]);
    assert_eq!(names(&graph.reachable_dynamic), ["d.js", "e.js", "f.js"]);
    assert!(!graph.reaches(&ScriptPath::new("g.js")));
    assert_eq!(findings(&bundle), []);
}

#[test]
fn the_entry_rules_run_in_vanilla_s_order() {
    // The extension is tested before the file set: a `.ts` entry that exists is still L2.
    let host = Host {
        entry: Some("scripts/main.ts"),
        ..Host::default()
    };
    let bundle = scan_with(
        &[("scripts/main.ts", b"")],
        Some(&host),
        ScanOptions::default(),
    );
    assert_eq!(findings(&bundle), [found("InvalidValue", "error", "")]);
    assert_eq!(bundle.graph, None);
    // An entry without the prefix whose file is missing is L3 alone, never L4.
    let host = Host {
        entry: Some("main.js"),
        ..Host::default()
    };
    let bundle = scan_with(
        &[("scripts/other.js", b"")],
        Some(&host),
        ScanOptions::default(),
    );
    assert_eq!(
        findings(&bundle),
        [found("UnresolvedReference", "error", "")]
    );
    let entry = bundle.entry.as_ref().expect("the entry as written");
    assert_eq!(
        (
            entry.raw.as_str(),
            entry.main.as_str(),
            entry.prefix_stripped
        ),
        ("main.js", "main.js", false)
    );
    // With the prefix and the file: nothing, and the prefix is recorded.
    let bundle = scan(&[("scripts/main.js", b"")]);
    assert_eq!(findings(&bundle), []);
    assert!(bundle.entry.as_ref().unwrap().prefix_stripped);
    // The prefix is matched byte for byte: `Scripts/` is no prefix.
    let host = Host {
        entry: Some("Scripts/main.js"),
        ..Host::default()
    };
    let bundle = scan_with(
        &[("scripts/main.js", b"")],
        Some(&host),
        ScanOptions::default(),
    );
    assert_eq!(
        findings(&bundle),
        [found("UnresolvedReference", "error", "")]
    );
    assert_eq!(
        bundle.entry.as_ref().unwrap().main.as_str(),
        "Scripts/main.js"
    );
}

#[test]
fn scanning_a_deeply_nested_file_fits_a_small_stack() {
    // A megabyte of stack, as a world load is given (crates/addon/std/tests/small_stack.rs).
    let handle = std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let depth = 100_000;
            let mut source = "({[`${".repeat(depth);
            source.push_str("import(\"./a.js\")");
            source.push_str(&"}`]})".repeat(depth));
            // A chain of that many files, each importing the next.
            let mut files: Vec<(String, Vec<u8>)> = (0..20_000)
                .map(|n| {
                    (
                        format!("scripts/c{n}.js"),
                        format!("import \"./c{}.js\";", n + 1).into_bytes(),
                    )
                })
                .collect();
            files.push((
                "scripts/main.js".to_owned(),
                format!("import \"./c0.js\";\n{source}").into_bytes(),
            ));
            files.push(("scripts/a.js".to_owned(), Vec::new()));
            files.push(("scripts/c20000.js".to_owned(), Vec::new()));
            let files: Vec<(PackPath, Vec<u8>)> = files
                .into_iter()
                .map(|(path, bytes)| (PackPath::new(&path).unwrap(), bytes))
                .collect();
            let bundle = ScriptBundle::scan(
                files.iter().map(|(path, bytes)| (path, bytes.as_slice())),
                Some(&Host::default()),
                ScanOptions::default(),
            );
            let graph = bundle.graph.as_ref().expect("a graph");
            // With the `script-syntax` feature the parser, which recurses, reads the chain's
            // files and is not given the nested one.
            let parsed = |name: &str| {
                bundle.files[&ScriptPath::new(name)]
                    .content
                    .as_ref()
                    .unwrap()
                    .parsed
            };
            assert_eq!(parsed("c0.js"), ScriptBundle::SYNTAX_CHECKED);
            assert!(!parsed("main.js"));
            (
                graph.reachable_static.len(),
                graph.reachable_dynamic.len(),
                bundle.findings().count(),
            )
        })
        .unwrap();
    // With the feature, the one finding is the note that the parser was kept from a file.
    let notes = usize::from(ScriptBundle::SYNTAX_CHECKED);
    assert_eq!(handle.join().unwrap(), (20_002, 1, notes));
}

/// A bundle of `n` files in a ring of static imports, each with a dynamic import of the
/// entry and one of a missing file whose name differs from a member in case only.
fn ring(n: usize) -> Vec<(PackPath, Vec<u8>)> {
    let mut files: Vec<(PackPath, Vec<u8>)> = (0..n)
        .map(|index| {
            let next = (index + 1) % n;
            let source = format!(
                "import {{ v }} from \"./f{next}.js\";\nimport(\"./main.js\");\nimport(\"./F{index}.js\");\nexport const w = 1;\n"
            );
            (PackPath::new(&format!("scripts/f{index}.js")).unwrap(), source.into_bytes())
        })
        .collect();
    files.push((
        PackPath::new("scripts/main.js").unwrap(),
        b"import \"./f0.js\";\n".to_vec(),
    ));
    files
}

#[test]
fn building_a_bundle_is_linear_in_its_files() {
    super::cost::assert_linear_at("script bundle, a ring of files", 2_000, ring, |files| {
        ScriptBundle::scan(
            files.iter().map(|(path, bytes)| (path, bytes.as_slice())),
            Some(&Host::default()),
            ScanOptions::default(),
        )
    });
}

/// One file with `n` imports: each kind, found and missing with each hint.
fn many_imports(n: usize) -> Vec<(PackPath, Vec<u8>)> {
    let mut source = String::new();
    for index in 0..n {
        writeln!(
            source,
            "import \"./lib/f{index}.js\"; import(\"./LIB/F{index}.js\"); import(\"./gone{index}\"); import(\"pkg{index}\"); export * from \"./dir{index}\";"
        )
        .unwrap();
    }
    let mut files = vec![(
        PackPath::new("scripts/main.js").unwrap(),
        source.into_bytes(),
    )];
    for index in 0..n {
        files.push((
            PackPath::new(&format!("scripts/lib/f{index}.js")).unwrap(),
            Vec::new(),
        ));
        files.push((
            PackPath::new(&format!("scripts/dir{index}/index.js")).unwrap(),
            Vec::new(),
        ));
    }
    files
}

#[test]
fn building_a_bundle_is_linear_in_the_imports_of_one_file() {
    super::cost::assert_linear_at(
        "script bundle, many imports",
        2_000,
        many_imports,
        |files| {
            ScriptBundle::scan(
                files.iter().map(|(path, bytes)| (path, bytes.as_slice())),
                Some(&Host::default()),
                ScanOptions::default(),
            )
        },
    );
}
