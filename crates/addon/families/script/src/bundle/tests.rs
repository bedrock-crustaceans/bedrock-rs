//! Unit tests of the lexer: each lexer rule, and its cost.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the parser tests run on threads of a chosen stack and read the fixture scripts"
)]

use super::lexer::{Lexed, cook, lex};
use super::{ImportKind, ScriptSyntaxKind};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::version::known;

/// The import sites of `source` as `(kind, specifier)`.
fn sites(source: &str) -> Vec<(ImportKind, String)> {
    let lexed = lex(source);
    assert_eq!(lexed.error, None, "{source}");
    lexed
        .imports
        .into_iter()
        .map(|site| (site.kind, site.raw))
        .collect()
}

fn site(kind: ImportKind, raw: &str) -> (ImportKind, String) {
    (kind, raw.to_owned())
}

use ImportKind::{Dynamic, DynamicNonLiteral, ReExport, SideEffect, Static};

#[test]
fn rule_1_comments_strings_and_a_hashbang_hide_their_text() {
    let source = "#!/usr/bin/env node import \"./hash.js\"\n\
        // import \"./line.js\"\n\
        /* import \"./block.js\" */\n\
        const a = \"import './inner.js'\"; const b = 'import(\"./q.js\")';\n\
        import \"./real.js\";\n";
    assert_eq!(sites(source), [site(SideEffect, "./real.js")]);
}

#[test]
fn rule_1_html_like_comments_are_code_in_a_module() {
    // `<!--` hides nothing: the import after it is one.
    assert_eq!(
        sites("x <!-- y\nimport \"./a.js\";"),
        [site(SideEffect, "./a.js")]
    );
}

#[test]
fn rule_2_template_text_is_skipped_and_substitutions_are_code() {
    let source = "const t = `import \"./text.js\" ${ import(\"./in.js\") } tail ${ `n ${ import('./deep.js') }` }`;\n\
        import \"./after.js\";";
    assert_eq!(
        sites(source),
        [
            site(Dynamic, "./in.js"),
            site(Dynamic, "./deep.js"),
            site(SideEffect, "./after.js"),
        ]
    );
}

#[test]
fn rule_3_a_slash_after_an_expression_divides() {
    // Were the first `/` a regular expression, it would swallow the import up to the second.
    let source = "const a = b / c; import \"./x.js\"; const d = e / 2;\n\
        const f = g[0] / h(1) / `t` / 'str'.length;\n\
        import(\"./y.js\");";
    assert_eq!(
        sites(source),
        [site(SideEffect, "./x.js"), site(Dynamic, "./y.js")]
    );
}

#[test]
fn rule_3_a_slash_where_an_expression_starts_is_a_regular_expression() {
    // Each literal holds a quote or an `import(` that would derail a lexer taking it as code.
    let source = "const a = /\"/; const b = x.replace(/'/g, '');\n\
        if (a) /import(\"no\")/.test(b);\n\
        const c = [/[/\"]/, /\\//];\n\
        return /`/.test(c) ? /\"/ : /'/;\n\
        import \"./ok.js\";";
    assert_eq!(sites(source), [site(SideEffect, "./ok.js")]);
}

#[test]
fn rule_3_a_slash_after_a_keyword_that_an_expression_follows_is_a_regular_expression() {
    // Each line is valid, and its regular expression holds a quote. Read
    // as a division, the quote would open a string that the line does not close.
    let rows = [
        ("of", "for (const m of /\"/g.exec(s)) { }"),
        (
            "of, for await",
            "for await (const m of /\"/g.exec(s)) /\"/.test(m);",
        ),
        ("in", "if (\"a\" in /\"/) { }"),
        ("instanceof", "x = a instanceof /\"/.constructor;"),
        ("return", "function f() { return /\"/.test(s); }"),
        ("typeof", "x = typeof /\"/;"),
        ("case", "switch (x) { case /\"/.source: break; }"),
        ("do", "do /\"/.test(s); while (0);"),
        ("else", "if (a) b(); else /\"/.test(s);"),
        ("yield", "function* g() { yield /\"/; }"),
        ("await", "x = await /\"/;"),
        ("throw", "function f() { throw /\"/; }"),
        ("new", "x = new /\"/.constructor();"),
        ("delete", "delete /\"/.x;"),
        ("void", "void /\"/;"),
    ];
    for (keyword, line) in rows {
        let source = format!("{line}\nimport(\"./after.js\");");
        let lexed = lex(&source);
        assert_eq!(lexed.error, None, "{keyword}: {source}");
        let found: Vec<_> = lexed.imports.iter().map(|site| site.raw.as_str()).collect();
        assert_eq!(found, ["./after.js"], "{keyword}: {source}");
    }
    // `of` is a name everywhere else: a `/` after it divides.
    for source in [
        "const of = 4; x = of / 2; y = \"a/b\"; import(\"./after.js\");",
        "x = f(of / 2, \"a/b\"); import(\"./after.js\");",
        "for (const of of [1]) x = of / 2 + \"/\".length; import(\"./after.js\");",
    ] {
        assert_eq!(sites(source), [site(Dynamic, "./after.js")], "{source}");
    }
}

#[test]
fn rule_3_a_slash_on_the_line_after_a_declaration_or_an_import_is_a_regular_expression() {
    // Nothing can follow a declared name or an import's specifier but `=`, `,` or the end of
    // the statement: on the next line a `/` starts a statement, and a regular expression.
    for (line, before) in [
        ("var x", None),
        ("let x", None),
        ("import \"./a.js\"", Some(site(SideEffect, "./a.js"))),
        ("import a from \"./a.js\"", Some(site(Static, "./a.js"))),
        ("export * from \"./a.js\"", Some(site(ReExport, "./a.js"))),
    ] {
        let source = format!("{line}\n/\"/.test(s);\nimport(\"./after.js\");");
        let expected: Vec<_> = before
            .into_iter()
            .chain([site(Dynamic, "./after.js")])
            .collect();
        assert_eq!(sites(&source), expected, "{source}");
    }
    // On the same line, after an initialiser and after any other name it divides.
    for source in [
        "var x = a\n/ 2 + \"/\".length; import(\"./after.js\");",
        "var x; y = x / 2 + \"/\".length; import(\"./after.js\");",
        "x\n/ 2 + \"/\".length; import(\"./after.js\");",
        "f(\"./a.js\")\n/ 2 + \"/\".length; import(\"./after.js\");",
    ] {
        assert_eq!(sites(source), [site(Dynamic, "./after.js")], "{source}");
    }
}

#[test]
fn rule_3_the_brace_before_a_slash_decides() {
    // A block's `}` starts a statement (a regular expression follows); an object's ends an
    // expression (a division follows).
    let block = "function f() {}\n/\"/.test(s); import \"./a.js\";";
    assert_eq!(sites(block), [site(SideEffect, "./a.js")]);
    let object = "const r = {a: 1} / 2; const q = \"x\" / 1; import \"./b.js\";";
    assert_eq!(sites(object), [site(SideEffect, "./b.js")]);
}

#[test]
fn rule_4_static_imports_in_every_clause_form() {
    let source = "import d from \"./default.js\";\n\
        import * as ns from './ns.js';\n\
        import { a, b as c, \"string name\" as e } from \"./named.js\";\n\
        import f, { g } from \"./both.js\";\n\
        import h, * as i from \"./default_ns.js\";\n\
        import from from \"./from.js\";\n\
        import \"./side.js\";\n";
    assert_eq!(
        sites(source),
        [
            site(Static, "./default.js"),
            site(Static, "./ns.js"),
            site(Static, "./named.js"),
            site(Static, "./both.js"),
            site(Static, "./default_ns.js"),
            site(Static, "./from.js"),
            site(SideEffect, "./side.js"),
        ]
    );
}

#[test]
fn rule_5_re_exports_with_from_and_none_without() {
    let source = "export * from \"./all.js\";\n\
        export * as ns from \"./ns.js\";\n\
        export { a, b as c } from \"./named.js\";\n\
        const x = 1; export { x };\n\
        export default x;\n\
        export const from = \"./not.js\";\n\
        import \"./side.js\";";
    assert_eq!(
        sites(source),
        [
            site(ReExport, "./all.js"),
            site(ReExport, "./ns.js"),
            site(ReExport, "./named.js"),
            site(SideEffect, "./side.js"),
        ]
    );
}

#[test]
fn rule_6_dynamic_imports_literal_and_not() {
    let source = "import(\"./a.js\"); await import('./b.js', { with: {} });\n\
        import(`./c.js`); import(`./${d}.js`); import(e); import(\"./f\" + g);\n\
        import((h)); import();";
    assert_eq!(
        sites(source),
        [
            site(Dynamic, "./a.js"),
            site(Dynamic, "./b.js"),
            site(Dynamic, "./c.js"),
            site(DynamicNonLiteral, ""),
            site(DynamicNonLiteral, ""),
            site(DynamicNonLiteral, ""),
            site(DynamicNonLiteral, ""),
            site(DynamicNonLiteral, ""),
        ]
    );
}

#[test]
fn rule_7_import_meta_is_no_import_and_a_member_read_is_noted() {
    let lexed =
        lex("const m = import.meta;\nconst u = import.meta.url; const k = import.meta[\"k\"];");
    assert_eq!(lexed.imports, []);
    let lines: Vec<(u32, u32)> = lexed.metas.iter().map(|at| (at.line, at.column)).collect();
    assert_eq!(lines, [(2, 11), (2, 38)]);
    // `import.metadata` is another name.
    assert_eq!(lex("import.metadata.x").metas, []);
}

#[test]
fn rule_8_attributes_after_a_specifier() {
    let lexed = lex("import d from \"./d.json\" with { type: \"json\" };\n\
        import \"./s.js\" assert { type: \"json\" };\n\
        export * from \"./e.js\" with { type: \"json\" };\n\
        import \"./plain.js\";\nwith_ = 1;");
    let flags: Vec<(&str, bool)> = lexed
        .imports
        .iter()
        .map(|site| (site.raw.as_str(), site.attributes))
        .collect();
    assert_eq!(
        flags,
        [
            ("./d.json", true),
            ("./s.js", true),
            ("./e.js", true),
            ("./plain.js", false),
        ]
    );
}

#[test]
fn rule_9_the_specifier_is_the_cooked_value_at_its_opening_quote() {
    let lexed = lex(
        "\n  import \"./\\u{61}\\x62\\u0063\\\n.js\";\nimport('\u{fc}') ; import(\"./q\\\"uote.js\");",
    );
    let got: Vec<(&str, u32, u32, u64)> = lexed
        .imports
        .iter()
        .map(|site| {
            (
                site.raw.as_str(),
                site.at.line,
                site.at.column,
                site.at.offset,
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("./abc.js", 2, 10, 10),
            // Line 4 because the line continuation is a line of the file.
            ("\u{fc}", 4, 8, 44),
            ("./q\"uote.js", 4, 22, 59),
        ]
    );
    assert_eq!(cook(r"😀\n\t\0\q"), "\u{1F600}\n\t\0q");
}

#[test]
fn rule_9_columns_count_characters_and_a_byte_order_mark_is_none() {
    let lexed = lex("\u{feff}/* \u{fc}\u{fc} */ import \"./a.js\";");
    let at = lexed.imports[0].at;
    assert_eq!((at.line, at.column, at.offset), (1, 17, 21));
}

#[test]
fn rule_10_property_names_and_import_phases_are_no_imports() {
    let source = "a.import(\"./p.js\"); b?.import(\"./q.js\"); const o = { import: \"./k.js\", export: 1 };\n\
        class C { import(x) { return x; } }\n\
        import defer * as ns from \"./defer.js\";\n\
        import source s from \"./source.js\";\n\
        import defer from \"./named_defer.js\";\n\
        import source, { z } from \"./named_source.js\";";
    assert_eq!(
        sites(source),
        [
            site(Static, "./named_defer.js"),
            site(Static, "./named_source.js"),
        ]
    );
}

#[test]
fn rule_10_a_dynamic_import_before_a_block_on_the_next_line_is_an_import() {
    // A line ends between the `)` and the `{`, so the import is a statement
    // of its own (a semicolon is inserted) and the braces are a block.
    for (source, expected) in [
        (
            "import(\"./a.js\")\n{ let x = 1; }",
            site(Dynamic, "./a.js"),
        ),
        ("import(\"./a.js\")\r\n{ }", site(Dynamic, "./a.js")),
        ("import(\"./a.js\") // note\n{ }", site(Dynamic, "./a.js")),
        (
            "import(\"./a.js\") /* a\n b */ { }",
            site(Dynamic, "./a.js"),
        ),
        ("import(\"./a.js\")\u{2028}{ }", site(Dynamic, "./a.js")),
        ("import(x)\n{ }", site(DynamicNonLiteral, "")),
    ] {
        assert_eq!(sites(source), [expected], "{source:?}");
    }
    // On one line the braces are the body of a method named `import`.
    for source in [
        "class C { import(x) { return x; } }",
        "class C { import(x) /* body */ { return x; } }",
        "o = { import(x)   { return x; } }",
    ] {
        assert_eq!(sites(source), [], "{source:?}");
    }
}

#[test]
fn rule_11_eval_and_function_calls() {
    let lexed = lex(
        "eval(\"1\");\nnew Function(\"return 1\")();\nx.eval(\"2\"); function eval2() {}\nconst e = eval; Function.prototype;",
    );
    let lines: Vec<(u32, u32)> = lexed.evals.iter().map(|at| (at.line, at.column)).collect();
    assert_eq!(lines, [(1, 1), (2, 5)]);
}

#[test]
fn an_unterminated_literal_stops_the_scan_with_the_sites_before_it() {
    let cases = [
        (
            "import \"./a.js\";\nconst s = \"open\nimport \"./b.js\";",
            ScriptSyntaxKind::UnterminatedString,
            2,
            11,
        ),
        (
            "import \"./a.js\";\nconst t = `open ${ 1 } \nimport \"./b.js\";",
            ScriptSyntaxKind::UnterminatedTemplate,
            2,
            11,
        ),
        (
            "import \"./a.js\";\n /* open\nimport \"./b.js\";",
            ScriptSyntaxKind::UnterminatedComment,
            2,
            2,
        ),
        (
            "import \"./a.js\";\nconst t = `open ${ import(\"./c.js\")",
            ScriptSyntaxKind::UnterminatedTemplate,
            2,
            11,
        ),
    ];
    for (source, kind, line, column) in cases {
        let Lexed { imports, error, .. } = lex(source);
        let (found, at) = error.unwrap_or_else(|| panic!("no error in {source}"));
        assert_eq!(
            (found, at.line, at.column),
            (kind, line, column),
            "{source}"
        );
        assert_eq!(imports[0].raw, "./a.js", "{source}");
        assert!(imports.iter().all(|site| site.raw != "./b.js"), "{source}");
    }
}

#[test]
fn deep_nesting_needs_no_stack() {
    // 200 000 nested brackets and template substitutions: a recursive lexer would overflow.
    let depth = 200_000;
    let mut source = String::new();
    for _ in 0..depth {
        source.push_str("[({`${");
    }
    source.push_str("import(\"./deep.js\")");
    for _ in 0..depth {
        source.push_str("}`})]");
    }
    assert_eq!(sites(&source), [site(Dynamic, "./deep.js")]);
}

/// A file of `n` statements, each a mix of every literal the lexer skips and one import.
fn many_imports(n: usize) -> String {
    let mut source = String::new();
    for index in 0..n {
        source.push_str("import { a as b } from \"./lib/file.js\"; const x = y / 2 + /re[/]x/.test(`t${ z }`); /* c */ import(\"./d.js\"); import(q) {\n");
        if index % 2 == 0 {
            source.push_str("} // line\n");
        } else {
            source.push_str("}\n");
        }
    }
    source
}

#[test]
fn the_lexer_is_linear_in_the_imports() {
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, many imports",
        4_000,
        many_imports,
        |source| lex(source),
    );
}

#[test]
fn the_lexer_is_linear_in_one_long_line() {
    // One line: line and column are counted once, not from the line's start per site.
    let one_line = |n: usize| many_imports(n).replace('\n', " ").replace("// line", "");
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, one long line",
        4_000,
        one_line,
        |source| lex(source),
    );
}

/// One line of `piece` repeated, behind `head`: no regular expression of it closes.
fn unclosed(head: &'static str, piece: &'static str) -> impl FnMut(usize) -> String {
    move |n| format!("{head}{}", piece.repeat(n))
}

#[test]
fn the_lexer_is_linear_in_a_line_of_regular_expressions_that_do_not_close() {
    // After one scan that reached the line's end without a closing `/`,
    // no later `/` of the line scans to its end again.
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, unclosed classes",
        100_000,
        unclosed("", "/["),
        |source| lex(source),
    );
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, unclosed classes in arguments",
        40_000,
        unclosed("x=", "(1,/["),
        |source| lex(source),
    );
    // The same with nothing but escaped slashes up to the class, and with a class that
    // closes before the next one opens.
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, escaped slashes before an unclosed class",
        20_000,
        unclosed("", "(/\\/\\/["),
        |source| lex(source),
    );
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, classes that close and one that does not",
        60_000,
        unclosed("/[;", "(/[]x"),
        |source| lex(source),
    );
}

#[test]
fn a_slash_after_a_scan_that_failed_is_read_as_it_was_before() {
    // The first `/` finds no end (its class never closes); the later ones are read on their
    // own: a regular expression where one closes before the next bracket, else a division.
    assert_eq!(
        sites("x = (/[ , /a/.test(import('in.js')), import('a.js')"),
        [site(Dynamic, "in.js"), site(Dynamic, "a.js")]
    );
    assert_eq!(
        sites("x = (/[ , /import('in.js')/, import('a.js')"),
        [site(Dynamic, "a.js")]
    );
    assert_eq!(
        sites("x = (/[ , /a[import('in.js'), import('a.js')"),
        [site(Dynamic, "in.js"), site(Dynamic, "a.js")]
    );
    assert_eq!(
        sites("x = (/[ , /\\/import('in.js')/, import('a.js')"),
        [site(Dynamic, "a.js")]
    );
    // The next line starts afresh.
    assert_eq!(
        sites("x = (/[ ,\n /[import('in.js')]/, import('a.js')"),
        [site(Dynamic, "a.js")]
    );
}

#[test]
fn the_lexer_is_linear_in_a_large_comment_string_and_number() {
    let large = |n: usize| {
        let mut source = String::from("/*");
        source.push_str(&"c".repeat(n * 100));
        source.push_str("*/ const s = \"");
        source.push_str(&"s".repeat(n * 100));
        source.push_str("\"; const d = 0.");
        source.push_str(&"1".repeat(n * 100));
        source.push_str("; import \"./end.js\";");
        source
    };
    bedrock_addon_core::test_cost::assert_linear_at(
        "script lexer, large literals",
        20_000,
        large,
        |source| lex(source),
    );
}

/// The shapes a parser recurses on, each as `(name, what one level opens, the innermost
/// expression or statement, what one level closes)`.
#[cfg(feature = "script-syntax")]
const NESTED: [(&str, &str, &str, &str); 55] = [
    (
        "class heritage without parentheses",
        "class A extends ",
        "B",
        " {}",
    ),
    ("templates with quotes", "`\"${", "1", "}'`"),
    ("static blocks", "class A { static { ", "", " } }"),
    ("catch blocks", "try {} catch { ", "", " }"),
    ("finally blocks", "try {} finally { ", "", " }"),
    ("decorated classes", "(@", "a", " class {})"),
    (
        "regular expressions before blocks",
        "(/[)]/.test(a) || function () { return ",
        "1",
        " })",
    ),
    ("divisions by parentheses", "(a / ", "1", " / b)"),
    ("minus on lines", "-\n", "a", ""),
    ("await", "await ", "a", ""),
    ("complement", "~", "a", ""),
    ("async arrows without parentheses", "async a => ", "1", ""),
    ("spread in calls", "f(...", "a", ")"),
    ("parentheses", "(", "1", ")"),
    ("arrays", "[", "1", "]"),
    ("objects", "({a: ", "1", "})"),
    ("calls", "f(", "1", ")"),
    ("optional calls", "f?.(", "1", ")"),
    ("computed members", "a[", "1", "]"),
    ("templates", "`${", "1", "}`"),
    ("tagged templates", "t`a${", "1", "}b`"),
    ("function declarations", "function f() { ", "", " }"),
    ("function expressions", "(function () { return ", "1", " })"),
    ("generators", "(function* () { yield ", "1", " })"),
    ("async functions", "(async function () { await ", "1", " })"),
    ("arrows with a block", "(() => { return ", "1", " })"),
    ("async arrows", "(async () => await ", "1", ")"),
    ("classes", "class A { m() { ", "", " } }"),
    ("class expressions", "(class { static m = ", "1", " })"),
    ("class heritage", "(class extends ", "Object", " {})"),
    ("object methods", "({ m() { return ", "1", " } })"),
    ("getters", "({ get m() { return ", "1", " } })"),
    ("default parameters", "(function (a = ", "1", ") {})"),
    ("blocks", "{ ", "", " }"),
    ("if with blocks", "if (a) { ", "", " }"),
    ("else with blocks", "if (a) {} else { ", "", " }"),
    ("loops with blocks", "for (;;) { ", "", " }"),
    ("try blocks", "try { ", "", " } catch {}"),
    ("switches", "switch (a) { case 1: ", "", " }"),
    ("labelled blocks", "l: { ", "", " }"),
    ("imports", "import(", "'./a.js'", ")"),
    ("new with arguments", "new X(", "1", ")"),
    ("spread", "[...", "a", "]"),
    (
        "a precedence ladder in parentheses",
        "(b || c && d | e ^ f & g == h < i << j + k * l ** ",
        "1",
        ")",
    ),
    (
        "a precedence ladder in an array",
        "[a || b && c | d ^ e & f == g < h << i + j * k ** ",
        "1",
        "]",
    ),
    ("prefix operators", "!", "a", ""),
    ("keyword operators", "typeof void delete ", "a", ""),
    ("negations", "- + ~ ", "a", ""),
    ("new", "new ", "X", ""),
    ("assignments", "a = ", "1", ""),
    ("compound assignments", "a ||= b += ", "1", ""),
    ("ternaries", "a ? ", "1", " : 2"),
    ("ternaries on the right", "a ? 1 : ", "2", ""),
    ("arrows", "a => ", "1", ""),
    ("exponents", "a ** ", "1", ""),
];

/// Statements that nest without a bracket of their own.
#[cfg(feature = "script-syntax")]
const UNBRACED: [(&str, &str, &str); 10] = [
    ("with", "with (a) ", "x;"),
    ("labelled loops", "l: for (;;) ", "x;"),
    ("if", "if (a) ", "x;"),
    ("else if", "if (a) x; else ", "y;"),
    ("for", "for (;;) ", "x;"),
    ("while", "while (a) ", "x;"),
    ("with-less labels", "l: ", "x;"),
    ("for of", "for (const a of b) ", "x;"),
    ("if else if with blocks", "if (a) {} else ", "{}"),
    ("mixed", "if (a) for (;;) l: while (b) ", "x;"),
];

/// Parses `source` on a thread with as much stack as the budget and a little for the call:
/// returns only when the parser fits (an overflow aborts the test binary).
#[cfg(feature = "script-syntax")]
fn parse_within_the_budget(source: String) -> bool {
    use super::bound::STACK_BUDGET;
    use super::syntax::first_error;
    let stack = usize::try_from(STACK_BUDGET).unwrap() + 64 * 1024;
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || first_error(&source).is_none())
        .unwrap()
        .join()
        .unwrap()
}

/// The deepest `levels` for which the bound of `build(levels)` is within the budget.
#[cfg(feature = "script-syntax")]
fn deepest(build: &dyn Fn(usize) -> String) -> usize {
    let fits = |levels: usize| {
        super::bound::parser_stack(&build(levels), super::bound::STACK_BUDGET).is_some()
    };
    let (mut low, mut high) = (1, 2);
    while fits(high) {
        low = high;
        high *= 2;
        assert!(high < 1 << 20, "the bound does not grow with the nesting");
    }
    while high - low > 1 {
        let middle = usize::midpoint(low, high);
        if fits(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

#[cfg(feature = "script-syntax")]
#[test]
fn the_parser_fits_its_stack_budget_at_the_deepest_nesting_the_bound_allows() {
    // The frames are largest with full debug info. The workspace sets no profile, so a plain
    // `cargo test` has it (`debug = 2` is the default of `dev` and `test`); where a profile
    // or the environment lowers it, run this once with
    // CARGO_PROFILE_TEST_DEBUG=2 CARGO_PROFILE_DEV_DEBUG=2.
    for (name, open, inner, close) in NESTED {
        let build = |levels: usize| {
            format!(
                "x = {}{inner}{};",
                open.repeat(levels),
                close.repeat(levels)
            )
        };
        let statement =
            |levels: usize| format!("{}{inner}{}", open.repeat(levels), close.repeat(levels));
        // The shapes that are statements stand alone; the others are the right side of `x =`.
        let build: &dyn Fn(usize) -> String = if inner.is_empty() { &statement } else { &build };
        let levels = deepest(build);
        // The last line of a run that aborted names the shape that overflowed.
        println!("fitting: {name} at {levels} levels");
        assert!(levels >= 8, "{name}: only {levels} levels fit the bound");
        assert!(
            parse_within_the_budget(build(levels)),
            "{name} at {levels} is no valid script"
        );
    }
    for (name, open, inner) in UNBRACED {
        let build = |levels: usize| format!("{}{inner}", open.repeat(levels));
        let levels = deepest(&build);
        // The last line of a run that aborted names the shape that overflowed.
        println!("fitting: {name} at {levels} levels");
        assert!(levels >= 8, "{name}: only {levels} levels fit the bound");
        assert!(
            parse_within_the_budget(build(levels)),
            "{name} at {levels} is no valid script"
        );
    }
    // One shape inside another: the bound adds up across brackets.
    let mixed = |levels: usize| {
        format!(
            "x = {}1{};",
            "[(a) => { if (b) return c ? d : `${({ m() { return ".repeat(levels),
            "} })}` }]".repeat(levels)
        )
    };
    let levels = deepest(&mixed);
    assert!(parse_within_the_budget(mixed(levels)), "mixed at {levels}");
    // Destructuring patterns: the brackets of one assignment target.
    let object_pattern =
        |levels: usize| format!("({}b{} = c);", "{a: ".repeat(levels), "}".repeat(levels));
    let array_pattern =
        |levels: usize| format!("{}b{} = c;", "[".repeat(levels), "]".repeat(levels));
    let parameters = |levels: usize| {
        format!(
            "x = ({}b{}) => 1;",
            "{a: [".repeat(levels),
            "]}".repeat(levels)
        )
    };
    for (name, build) in [
        (
            "object patterns",
            &object_pattern as &dyn Fn(usize) -> String,
        ),
        ("array patterns", &array_pattern),
        ("patterns in parameters", &parameters),
    ] {
        let levels = deepest(build);
        assert!(
            parse_within_the_budget(build(levels)),
            "{name} at {levels} is no valid script"
        );
    }
}

#[cfg(feature = "script-syntax")]
#[test]
fn long_flat_code_is_bounded_shallow_and_parsed() {
    let n = 20_000;
    let flat: [(&str, String); 12] = [
        ("a sum", format!("x = a{};", " + b".repeat(n))),
        ("a member chain", format!("x = a{};", ".b".repeat(n))),
        ("a call chain", format!("x = a{};", "()".repeat(n))),
        ("a comma list", format!("x = [a{}];", ", b".repeat(n))),
        ("arguments", format!("f(a{});", ", b = 1".repeat(n))),
        (
            "an object",
            format!("x = {{a: 1{}}};", ", b: c ? d : e".repeat(n)),
        ),
        ("statements", "x = a ? b : c + d * e;\n".repeat(n)),
        (
            "statements without semicolons",
            "const x = a ? b : c + d * e\nfoo(x)\n".repeat(n),
        ),
        (
            "function declarations",
            "function f(a) { return a ? b : c }\n".repeat(n),
        ),
        (
            "exported classes",
            (0..n).fold(String::new(), |mut source, i| {
                source.push_str("export class A");
                source.push_str(&i.to_string());
                source.push_str(" extends B { m() { return 1 } }\n");
                source
            }),
        ),
        (
            "if blocks",
            "if (a) { b = c } else { d = e }\nwhile (f) { g = !h }\n".repeat(n),
        ),
        (
            "switch cases",
            format!(
                "switch (a) {{{}}}",
                " case 1: b = c; break; case 2: d = e;".repeat(n)
            ),
        ),
    ];
    for (name, source) in flat {
        let stack = super::bound::parser_stack(&source, u64::MAX).unwrap();
        assert!(stack <= 32 * 1024, "{name}: bounded at {stack} bytes");
        assert!(parse_within_the_budget(source), "{name} is no valid script");
    }
}

#[test]
fn the_stack_bound_is_linear_in_the_file() {
    use super::bound::{STACK_BUDGET, parser_stack};
    // Each input stays within the budget, so the whole of it is scanned.
    let bound = |source: &String| assert!(parser_stack(source, STACK_BUDGET).is_some());
    bedrock_addon_core::test_cost::assert_linear_at(
        "script stack bound, many imports",
        4_000,
        many_imports,
        bound,
    );
    // Forty-eight brackets deep, where a `/` both divides and starts a regular expression:
    // two readings, joined level by level at every one.
    let forks = |n: usize| {
        format!(
            "{}{}{}",
            "({[".repeat(16),
            "a++ / 2 / /x/.y; return /z/ / w; {} / 3 / `${c}` / 4;\n".repeat(n),
            "]})".repeat(16)
        )
    };
    bedrock_addon_core::test_cost::assert_linear_at(
        "script stack bound, two readings",
        4_000,
        forks,
        bound,
    );
    // Past the budget the scan stops; up to it the readings hold at most the budget's
    // brackets, however many the file opens.
    let stops = |source: &String| parser_stack(source, STACK_BUDGET);
    bedrock_addon_core::test_cost::assert_linear_at(
        "script stack bound, closers and openers",
        8_000,
        |n| "x = /[\"/]/ + `${(a)}`) / (b\n".repeat(n),
        stops,
    );
}

#[test]
fn the_stack_bound_follows_every_reading_of_a_slash() {
    let stack = |source: &str| super::bound::parser_stack(source, u64::MAX).unwrap();
    let deep = 3 * 4608;
    // After a `}`, a head's `)`, a keyword and `++`, the `/` may divide or start a regular
    // expression: the brackets count in the reading that has them.
    for source in [
        "x = function(){} / (((1))) / 1;",
        "x = {} / (((1))) / 1;",
        "if (a) /(((/.test(b)",
        "if (a) /\"/.test(b), (((1))); \"\"",
        "return /\"/.test(b), (((1))); \"\"",
        "for (a of /\"/.exec(b)) (((1))); \"\"",
        "a++ / (((1))) / 1;",
        "var a\n/\"/.test(b), (((1))); \"\"",
    ] {
        assert!(stack(source) >= deep, "{source}");
    }
    // After a name, a number, a string, `]` and a call's `)` on the same line it divides, and
    // after an operator it starts a regular expression: one reading, three brackets deep
    // (four in the last).
    for (source, expected) in [
        ("x = a / b; y = (((1))); z = c / d;", 2048 + deep),
        ("x = f(a) / 2 + (((1))) / g[0] / 3;", 2048 + 2 * 1400 + deep),
        ("x = /[(]/.test(a) + (((1)));", 2048 + 1400 + deep),
        ("x = a.replace(/\"/g, '') + (((1)));", 2048 + 1400 + deep),
        ("x = (/\\(/, (((1))), /[/(\"]/);", 2048 + 4608 + deep),
    ] {
        assert_eq!(stack(source), expected, "{source}");
    }
    // What only one reading takes for brackets is counted all the same.
    assert_eq!(stack("x = /(((/;"), 2048);
    assert_eq!(stack("x = `(((`;"), 2048);
    assert_eq!(stack("x = '(((' // (((\n/* ((( */;"), 2048);
    assert_eq!(stack("x = `${(((1)))}`;"), 2048 + 4 * 4608);
    assert_eq!(stack("x = `${`${1}`}`;"), 2048 + 2 * 4608);
}

#[test]
fn the_stack_bound_counts_what_is_open_and_forgets_what_has_closed() {
    let stack = |source: &str| super::bound::parser_stack(source, u64::MAX).unwrap();
    // One open bracket is 4608 bytes, one pending operator 2048, one precedence step 1400.
    assert_eq!(stack("a"), 0);
    assert_eq!(stack("(a)"), 4608);
    assert_eq!(stack("((a))"), 2 * 4608);
    assert_eq!(stack("(a); (b); (c);"), 4608);
    assert_eq!(stack("x = y;"), 2048);
    assert_eq!(stack("x = y = z;"), 2 * 2048);
    assert_eq!(stack("x = y; z = w;"), 2048);
    assert_eq!(stack("a + b + c;"), 1400);
    assert_eq!(stack("a + b * c;"), 2 * 1400);
    assert_eq!(stack("a + b; c * d;"), 1400);
    assert_eq!(stack("f(a = b, c = d)"), 4608 + 2048);
    // Unbraced statements stay open until the statement ends; `else` goes on past the `;`.
    assert_eq!(stack("if (a) if (b) x;"), 2 * 2048 + 4608);
    assert_eq!(stack("if (a) x; if (b) y;"), 2048 + 4608);
    // Deepest inside `(b)`: `if`, `else`, `if` pending and one bracket open.
    assert_eq!(stack("if (a) x; else if (b) y; else z;"), 3 * 2048 + 4608);
    assert_eq!(
        stack("if (a) x; else if (b) y; else if (c) z;"),
        5 * 2048 + 4608
    );
    // A function costs its brackets; a comma ends what waited for its right side.
    assert_eq!(stack("function f() {} function g() {}"), 4608);
    assert_eq!(
        stack("x = function () {}, y = function () {};"),
        2048 + 4608
    );
    // Without semicolons, a word that can only begin a statement starts afresh after an
    // operand and after a `}`, and nowhere else.
    assert_eq!(stack("const a = b\nconst c = d\n"), 2048);
    assert_eq!(stack("a = b\nc = d\n"), 2 * 2048);
    assert_eq!(stack("if (a) { b = c } if (d) { e = f }"), 2 * 2048 + 4608);
    assert_eq!(stack("if (a) if (b) return (c)"), 3 * 2048 + 4608);
    // A label waits like a statement: past a `;` when `else` or `while` follows.
    assert_eq!(stack("a: b: do x; while (c)"), 4 * 2048 + 4608);
    assert_eq!(stack("a: b: x; (c)"), 4608);
    // The keys of an object and the arms of a ternary end at the comma.
    assert_eq!(stack("x = {a: 1, b: 2, c: d ? e : f};"), 5 * 2048 + 4608);
    // A clause of a switch starts afresh.
    assert_eq!(
        stack("switch (a) { case 1: case 2: b = c; case 3: d = e }"),
        4608 + 4 * 2048
    );
    // A property name is no keyword.
    assert_eq!(stack("a.if.for.delete.new.typeof.in"), 0);
    // The `.` that ends a number is no member access: the word on the next line is a keyword.
    assert_eq!(stack("x = 1.\nif (a) if (b) y;"), 3 * 2048 + 4608);
    assert_eq!(stack("x = a?.b?.c;"), 2048);
}

/// Every `.js` file below `dir`.
#[cfg(feature = "script-syntax")]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn js_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            js_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "js") {
            out.push(path);
        }
    }
}

/// The lexer against the parser, the plain implementation that is its oracle: for every script
/// file both read, the static imports and re-exports the lexer finds are the modules the
/// parser's module record requests. The files are the fixture packs and, when
/// `BEDROCK_ADDON_CORPUS` is set, every `.js` file below that directory (the wrappers of the
/// game's library packs are minified bundles: the regular-expression rule at work).
#[cfg(feature = "script-syntax")]
#[test]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn the_lexer_finds_the_static_imports_the_parser_finds() {
    use super::bound::STACK_BUDGET;
    use super::syntax::requested_modules;
    let mut files = Vec::new();
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../addon/tests/fixtures/script/script_bundle/packs");
    js_files(&fixtures, &mut files);
    let in_fixtures = files.len();
    assert!(in_fixtures >= 60, "{in_fixtures} fixture scripts");
    if let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") {
        js_files(std::path::Path::new(&corpus), &mut files);
    }
    files.sort();
    // The test gives the parser a stack no file overflows, so that the files
    // the library keeps from it (nested past the budget) are compared too.
    let compare = move || {
        let (mut compared, mut sites, mut too_deep, mut unreadable) = (0, 0, 0, 0);
        for path in &files {
            let Ok(text) = String::from_utf8(std::fs::read(path).unwrap()) else {
                unreadable += 1;
                continue;
            };
            let lexed = lex(&text);
            if super::bound::parser_stack(&text, STACK_BUDGET).is_none() {
                println!("past the parser's budget: {}", path.display());
                too_deep += 1;
            }
            let Some(wanted) = requested_modules(&text) else {
                // The parser refuses the file: the fixtures with a syntax error.
                unreadable += 1;
                continue;
            };
            assert_eq!(lexed.error, None, "{}", path.display());
            let mut found: Vec<String> = lexed
                .imports
                .iter()
                .filter(|site| site.kind.is_static())
                .map(|site| site.raw.clone())
                .collect();
            sites += found.len();
            found.sort();
            found.dedup();
            assert_eq!(found, wanted, "{}", path.display());
            compared += 1;
        }
        (compared, sites, too_deep, unreadable)
    };
    let (compared, sites, too_deep, unreadable) = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(compare)
        .unwrap()
        .join()
        .unwrap();
    println!(
        "lexer against parser: {compared} files compared ({in_fixtures} fixture scripts), {sites} static import sites, {too_deep} nested past the parser's budget, {unreadable} the parser or UTF-8 refuses"
    );
    // The fixtures: 3 files are not UTF-8 or no valid script by design (latin1_entry, unused_bad
    // and the three syntax fixtures), none is deep.
    assert!(compared >= in_fixtures - 6, "{compared} of {in_fixtures}");
}

/// The bundles of a stack of one behaviour pack with `manifest` and an empty
/// `scripts/main.js`, loaded as the stack's loader loads them, on an engine of `engine`.
fn bundles_on_engine(
    manifest: &str,
    engine: bedrock_addon_core::version::FormatVersion,
) -> Vec<super::ScriptBundle> {
    use bedrock_addon_core::load::{
        ListedFile, LoadContext, LoadOptions, StackCx, StackPack, WalkFile,
    };
    use bedrock_addon_core::pack_identity::{
        PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
    };
    use bedrock_addon_core::version::SemVersion;
    let mut cx = LoadContext::detached();
    cx.engine_version = engine;
    cx.pack_type = PackType::Behavior;
    let id = PackIdVersion::new(
        uuid::Uuid::from_u128(1),
        SemVersion::new(1, 0, 0),
        PackType::Behavior,
    );
    let pack = PackRef::new(0, id, "relaxed");
    let paths = ["manifest.json", "scripts/main.js"].map(PackPath::from_static);
    let entry = StackPack::new(pack.clone(), PackOrigin::User, cx.clone()).with_files(
        paths.iter().map(|path| ListedFile {
            path: path.clone(),
            key: path.clone(),
        }),
    );
    let loader = super::LOADER;
    let file = |path: &PackPath, bytes: &'static [u8]| WalkFile {
        pack: pack.clone(),
        side: PackSide::Behavior,
        path: path.clone(),
        key: path.clone(),
        bytes: bytes::Bytes::from_static(bytes),
    };
    let manifest: &'static str = Box::leak(manifest.to_owned().into_boxed_str());
    let parsed = vec![
        (loader.parse)(&file(&paths[0], manifest.as_bytes()), &cx),
        (loader.parse)(&file(&paths[1], b""), &cx),
    ];
    let options = LoadOptions::default();
    let packs = [entry];
    (loader.combine)(parsed, &StackCx::new(&options, &packs))
}

/// A stack entry built by hand whose `LoadContext::runs_scripts` nobody set. The walk
/// then reads the manifest alone, the module still makes a plugin, and the bundle's graph is
/// drawn over files nobody read: one finding says so.
#[test]
fn a_plugin_whose_script_files_the_walk_left_unread_is_told_once() {
    use bedrock_addon_core::diagnostic::{Code, Severity};
    use bedrock_addon_core::load::{
        ListedFile, LoadContext, LoadOptions, StackCx, StackPack, WalkFile,
    };
    use bedrock_addon_core::pack_identity::{
        PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
    };
    use bedrock_addon_core::version::SemVersion;
    const MANIFEST: &str = r#"{
    "format_version": 2,
    "header": { "name": "unread", "description": "",
        "uuid": "00000000-0000-0000-0000-000000000001", "version": [1, 0, 0],
        "min_engine_version": [1, 21, 0] },
    "modules": [{ "type": "script", "uuid": "00000000-0000-0000-0000-000000000002",
        "version": [1, 0, 0], "entry": "scripts/main.js" }]
}"#;
    let bundles = |runs_scripts: bool| {
        let mut cx = LoadContext::detached();
        cx.pack_type = PackType::Behavior;
        cx.runs_scripts = runs_scripts;
        let id = PackIdVersion::new(
            uuid::Uuid::from_u128(1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        );
        let pack = PackRef::new(0, id, "unread");
        let paths =
            ["manifest.json", "scripts/lib.js", "scripts/main.js"].map(PackPath::from_static);
        let entry = StackPack::new(pack.clone(), PackOrigin::User, cx.clone()).with_files(
            paths.iter().map(|path| ListedFile {
                path: path.clone(),
                key: path.clone(),
            }),
        );
        let loader = super::LOADER;
        let file = |path: &PackPath, bytes: &'static [u8]| WalkFile {
            pack: pack.clone(),
            side: PackSide::Behavior,
            path: path.clone(),
            key: path.clone(),
            bytes: bytes::Bytes::from_static(bytes),
        };
        // What the walk hands over: the manifest always, a script file only of a pack whose
        // context says it runs scripts (`Walk::in_pack`).
        let read = loader.walk.in_pack.expect("the walk's rule");
        let parsed = paths
            .iter()
            .zip([MANIFEST.as_bytes(), b"", b"import \"./lib.js\";\n"])
            .filter(|(path, _)| read(path, &cx))
            .map(|(path, bytes)| (loader.parse)(&file(path, bytes), &cx))
            .collect();
        let options = LoadOptions::default();
        let packs = [entry];
        (loader.combine)(parsed, &StackCx::new(&options, &packs))
    };
    let unread = bundles(false);
    assert_eq!(unread.len(), 1);
    assert!(unread[0].plugin.is_some());
    assert!(unread[0].files.values().all(|file| file.content.is_none()));
    let found: Vec<(&Code, Severity, String)> = unread[0]
        .findings()
        .map(|f| {
            (
                &f.code,
                f.severity,
                f.location
                    .path
                    .as_ref()
                    .map_or(String::new(), |path| path.as_str().to_owned()),
            )
        })
        .collect();
    assert_eq!(
        found,
        [(
            &Code::from(&core_codes::FILE_NOT_LOADED),
            Severity::Info,
            "scripts/main.js".to_owned()
        )],
        "one finding, at the main file, for the two unread files"
    );
    // A context that says so has the files read and nothing to tell.
    let read = bundles(true);
    assert!(read[0].plugin.is_some());
    assert!(read[0].files.values().all(|file| file.content.is_some()));
    assert_eq!(read[0].findings().count(), 0);
}

#[test]
fn m9_on_an_engine_before_1_26_30_the_relaxed_form_refuses_the_manifest_and_no_script_runs() {
    use bedrock_addon_core::diagnostic::{Code, Severity};
    use bedrock_addon_core::version::FormatVersion;
    let manifest = r#"{
    "format_version": 3,
    "header": { "name": "relaxed", "description": "",
        "uuid": "00000000-0000-0000-0000-000000000001", "version": "1.0.0",
        "min_engine_version": "1.26.30" },
    "modules": [{ "type": "script", "uuid": "00000000-0000-0000-0000-000000000002",
        "version": "1.0.0", "entry": "scripts/main.js" }],
    "dependencies": [{ "module_name": "@minecraft/server", "version": "2.0" }]
}"#;
    // A stack is always loaded at the engine the crate tracks; an older engine is a context
    // a caller sets.
    let refused = bundles_on_engine(manifest, FormatVersion::new(1, 26, 29));
    assert_eq!(refused.len(), 1);
    let found: Vec<(&Code, Severity, String, String)> = refused[0]
        .findings()
        .map(|f| {
            (
                &f.code,
                f.severity,
                f.location
                    .path
                    .as_ref()
                    .map_or(String::new(), |path| path.as_str().to_owned()),
                f.location.json_path.to_string(),
            )
        })
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].0.is(&core_codes::DEPENDENCY_INVALID_VERSION));
    assert_eq!(
        (found[0].1, found[0].2.as_str(), found[0].3.as_str()),
        (Severity::Error, "manifest.json", "dependencies/0/version")
    );
    assert_eq!(
        (refused[0].entry.as_ref(), refused[0].graph.as_ref()),
        (None, None)
    );
    // From 1.26.30 the same manifest starts its script.
    let accepted = bundles_on_engine(manifest, known::V1_26_30);
    assert_eq!(accepted[0].findings().count(), 0);
    assert!(accepted[0].graph.is_some());
}

#[test]
fn scan_options_set_imports_through_the_builder() {
    use super::ScanOptions;
    assert!(ScanOptions::default().imports);
    assert!(!ScanOptions::default().with_imports(false).imports);
}
