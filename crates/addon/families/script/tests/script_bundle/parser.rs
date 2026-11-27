//! The parser of the `script-syntax` feature recurses; no file may make it overflow the
//! stack. Every source here is scanned on a thread with the megabyte of stack a world load is
//! given (`crates/addon/std/tests/small_stack.rs`), through `ScriptBundle::scan`: an overflow
//! aborts the test binary. Run once with full debug info before a release (the frames are
//! largest then): `CARGO_PROFILE_TEST_DEBUG=2 CARGO_PROFILE_DEV_DEBUG=2`.

use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_script::{
    NativeModuleAccess, ScanOptions, ScriptBundle, ScriptHost, ScriptPath, ScriptSyntaxKind,
};

/// A script module that starts `scripts/main.js`.
struct Host;

impl ScriptHost for Host {
    fn plugin(&self) -> &'static str {
        "probe - 1.0.0"
    }
    fn entry(&self) -> Option<&str> {
        Some("scripts/main.js")
    }
    fn native(&self, _name: &str) -> NativeModuleAccess {
        NativeModuleAccess::Unknown
    }
    fn script_eval(&self) -> bool {
        true
    }
}

/// What the scan of one source came to.
#[derive(Debug, PartialEq, Eq)]
struct Scanned {
    /// The parser read the file.
    parsed: bool,
    /// The parser (not the lexer) found an error.
    parser_error: bool,
    /// The lexer stopped in the file.
    lexer_error: bool,
}

/// Scans each source as the entry of a bundle, all on one thread with a megabyte of stack.
fn scan_on_a_megabyte(sources: Vec<String>) -> Vec<Scanned> {
    let named = sources
        .into_iter()
        .map(|source| (String::new(), source))
        .collect();
    scan_named_on_a_megabyte(named)
}

/// [`scan_on_a_megabyte`] for sources with a name, which is printed before the source is
/// scanned: the last name in the output of a run that aborted is the shape that overflowed.
fn scan_named_on_a_megabyte(sources: Vec<(String, String)>) -> Vec<Scanned> {
    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(move || {
            let path = PackPath::new("scripts/main.js").unwrap();
            sources
                .iter()
                .map(|(name, source)| {
                    if !name.is_empty() {
                        println!("scanning: {name}");
                    }
                    let bundle = ScriptBundle::scan(
                        [(&path, source.as_bytes())],
                        Some(&Host),
                        ScanOptions::default(),
                    );
                    let file = &bundle.files[&ScriptPath::new("main.js")];
                    let parser_error = matches!(
                        file.content
                            .as_ref()
                            .unwrap()
                            .syntax
                            .as_ref()
                            .map(|syntax| &syntax.kind),
                        Some(ScriptSyntaxKind::Parser { .. })
                    );
                    Scanned {
                        parsed: file.content.as_ref().unwrap().parsed,
                        parser_error,
                        lexer_error: file.content.as_ref().unwrap().syntax.is_some()
                            && !parser_error,
                    }
                })
                .collect()
        })
        .unwrap()
        .join()
        .unwrap()
}

/// The file is kept from the parser: too deep for it, whatever the reading.
const KEPT: Scanned = Scanned {
    parsed: false,
    parser_error: false,
    lexer_error: false,
};

/// The parser read the file (with the feature) and found nothing.
const CLEAN: Scanned = Scanned {
    parsed: ScriptBundle::SYNTAX_CHECKED,
    parser_error: false,
    lexer_error: false,
};

/// How often a shape is repeated: more levels than any stack holds.
const DEEP: usize = 100_000;

/// `open` and `close` repeated around `inner`.
fn nest(open: &str, inner: &str, close: &str, levels: usize) -> String {
    format!("{}{inner}{}", open.repeat(levels), close.repeat(levels))
}

#[test]
fn the_three_shapes_of_the_review_are_kept_from_the_parser() {
    // B2 of the review: valid scripts in which the import lexer reads as a regular
    // expression what the parser reads as a division by nested brackets.
    let shape = |levels: usize| {
        vec![
            format!("x = function(){{}} / {} / 1;", nest("(", "1", ")", levels)),
            format!(
                "var r = c ? 0 : {{}} / {} / 3;",
                nest("(", "1", ")", levels)
            ),
            format!("x=class{{}}/{}/1", nest("[", "", "]", levels)),
        ]
    };
    assert_eq!(scan_on_a_megabyte(shape(DEEP)), [KEPT, KEPT, KEPT]);
    // The same shapes, shallow, are scripts the parser accepts.
    assert_eq!(scan_on_a_megabyte(shape(20)), [CLEAN, CLEAN, CLEAN]);
}

/// Scripts in which a careless reading takes brackets for the text of a string, a template, a
/// regular expression or a comment, or text for brackets: `(name, source)` with `@` where
/// the nested brackets go. Each is a valid script.
const MISREADINGS: [(&str, &str); 38] = [
    // The parser reads a regular expression; a reader that divides opens a string.
    (
        "of before a regex",
        "for (const m of /\"/g.exec(s)) @; \"end\";",
    ),
    (
        "return before a regex",
        "function f() { return /\"/.test(s) + @ }",
    ),
    ("typeof before a regex", "x = typeof /\"/ + @;"),
    (
        "case before a regex",
        "switch (x) { case /'/.source: y = @ }",
    ),
    ("else before a regex", "if (a) b(); else /`/.test(s), @;"),
    ("do before a regex", "do /\"/.test(s), @; while (0);"),
    ("in before a regex", "x = (\"a\" in /'/) + @;"),
    ("a head before a regex", "if (a) /\"/.test(s), @;"),
    (
        "a for await head before a regex",
        "for await (const a of b) /\"/.test(a), @;",
    ),
    ("a block before a regex", "{}\n/\"/.test(s), @;"),
    ("a declaration before a regex", "var x\n/\"/.test(s), @;"),
    (
        "an import before a regex",
        "import \"./a.js\"\n/'/.test(s), @;",
    ),
    ("a class in a regex", "x = /[\"/]/.test(s) + @;"),
    ("an escaped slash in a regex", "x = /\\//.test(\"//\") + @;"),
    (
        "a comment opener in a regex",
        "x = /[/*]/.test(s) + @; //*/",
    ),
    ("a regex after a multiplication", "x = b /a*/1/ + @;"),
    // The parser divides; a reader that starts a regular expression skips the brackets.
    ("a function before a division", "x = function(){} / @ / 1;"),
    ("an object before a division", "var r = c ? 0 : {} / @ / 3;"),
    ("a class before a division", "x = class{} / @ / 1;"),
    ("a parenthesis before a division", "x = (a) / @ / 2;"),
    ("a call named if before a division", "x = a.if(b) / @ / 2;"),
    ("a bracket before a division", "x = a[0] / @ / 2;"),
    ("a name before a division", "x = of / @ / 2;"),
    ("a name on the line before a division", "x = a\n/ @ / 2;"),
    ("a string before a division", "x = \"s\" / @ / 2;"),
    ("a template before a division", "x = `t` / @ / 2;"),
    ("a number before a division", "x = 1 / @ / 2;"),
    ("a postfix operator before a division", "x = a++ / @ / 2;"),
    ("this before a division", "x = this / @ / 2;"),
    // Strings, templates and comments that end where a careless reader does not look.
    (
        "an escaped backslash ends a string",
        "x = \"\\\\\" + @ + \"\\\\\";",
    ),
    (
        "a line continuation in a string",
        "x = \"a\\\n\" + @ + \"b\";",
    ),
    ("a quote in a comment", "x = a //\"\n + @;"),
    ("a quote in a block comment", "x = a /*\"*/ + @;"),
    ("a comment opener in a string", "x = \"/*\" + @ + \"*/\";"),
    (
        "a backquote in a substitution's string",
        "x = `${\"`\"}` + @;",
    ),
    ("escapes in a template", "x = `\\`` + @ + `\\${`;"),
    ("an HTML comment opener", "x = 1 <!--a\n+ @;"),
    (
        "a block comment that ends in two stars",
        "x = a /***/ + @ /**/;",
    ),
];

#[test]
fn brackets_are_counted_in_every_reading_of_a_slash_a_quote_and_a_comment() {
    let build = |levels: usize| -> Vec<(String, String)> {
        MISREADINGS
            .iter()
            .flat_map(|(name, source)| {
                [("(", "1", ")"), ("[", "1", "]"), ("{a: ", "1", "}")].map(
                    |(open, inner, close)| {
                        (
                            format!("{name}, {levels} of {open}"),
                            source.replace('@', &nest(open, inner, close, levels)),
                        )
                    },
                )
            })
            .collect()
    };
    for (scanned, (name, _)) in scan_named_on_a_megabyte(build(DEEP))
        .iter()
        .zip(build(DEEP))
    {
        assert!(!scanned.parsed, "{name}: {scanned:?}");
    }
    // Shallow, each is a script the lexer scans whole and the parser accepts.
    let wrong: Vec<_> = scan_named_on_a_megabyte(build(3))
        .into_iter()
        .zip(build(3))
        .filter(|(scanned, _)| *scanned != CLEAN)
        .map(|(scanned, (name, _))| (name, scanned))
        .collect();
    assert_eq!(wrong, [], "not clean");
}

/// What a parser recurses on, as `(name, what one level opens, the innermost expression or
/// statement, what one level closes)`: valid scripts at any depth.
const NESTED: [(&str, &str, &str, &str); 59] = [
    ("parentheses", "(", "1", ")"),
    ("arrays", "[", "1", "]"),
    ("objects", "({a: ", "1", "})"),
    ("blocks", "{ ", "", " }"),
    ("calls", "f(", "1", ")"),
    ("optional calls", "f?.(", "1", ")"),
    ("computed members", "a[", "1", "]"),
    ("templates", "`${", "1", "}`"),
    ("tagged templates", "t`a${", "1", "}b`"),
    ("templates in strings' clothes", "`\"${", "1", "}'`"),
    ("function declarations", "function f() { ", "", " }"),
    ("function expressions", "(function () { return ", "1", " })"),
    ("generators", "(function* () { yield ", "1", " })"),
    ("async functions", "(async function () { await ", "1", " })"),
    ("arrows with a block", "(() => { return ", "1", " })"),
    ("async arrows", "(async () => await ", "1", ")"),
    ("classes", "class A { m() { ", "", " } }"),
    ("class expressions", "(class { static m = ", "1", " })"),
    (
        "class heritage in parentheses",
        "(class extends ",
        "Object",
        " {})",
    ),
    ("static blocks", "class A { static { ", "", " } }"),
    ("object methods", "({ m() { return ", "1", " } })"),
    ("getters", "({ get m() { return ", "1", " } })"),
    ("default parameters", "(function (a = ", "1", ") {})"),
    ("if with blocks", "if (a) { ", "", " }"),
    ("else with blocks", "if (a) {} else { ", "", " }"),
    ("loops with blocks", "for (;;) { ", "", " }"),
    ("try blocks", "try { ", "", " } catch {}"),
    ("catch blocks", "try {} catch { ", "", " }"),
    ("finally blocks", "try {} finally { ", "", " }"),
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
    ("not", "!", "a", ""),
    ("complement", "~", "a", ""),
    ("minus", "- ", "a", ""),
    ("plus", "+ ", "a", ""),
    ("minus on lines", "-\n", "a", ""),
    ("typeof", "typeof ", "a", ""),
    ("void", "void ", "a", ""),
    ("delete", "delete ", "a", ""),
    ("new", "new ", "X", ""),
    ("await", "await ", "a", ""),
    ("assignments", "a = ", "1", ""),
    ("compound assignments", "a ||= b += c >>>= d **= ", "1", ""),
    ("ternaries", "a ? ", "1", " : 2"),
    ("ternaries on the right", "a ? 1 : ", "2", ""),
    ("arrows", "a => ", "1", ""),
    ("async arrows without parentheses", "async a => ", "1", ""),
    ("exponents", "a ** ", "1", ""),
    ("class heritage", "class A extends ", "B", " {}"),
    ("if", "if (a) ", "x;", ""),
    ("else if", "if (a) x; else ", "y;", ""),
    ("for", "for (;;) ", "x;", ""),
    ("while", "while (a) ", "x;", ""),
    ("do", "do ", "x;", " while (a);"),
    ("labels", "l: ", "x;", ""),
];

#[test]
fn every_construct_the_parser_recurses_on_is_kept_from_it_when_deep() {
    let build = |levels: usize| -> Vec<(String, String)> {
        let mut sources: Vec<(String, String)> = NESTED
            .iter()
            .map(|(name, open, inner, close)| {
                let nested = nest(open, inner, close, levels);
                // The shapes that are statements stand alone; the others are the right
                // side of `x =`.
                let statement =
                    inner.is_empty() || inner.ends_with(';') || open.starts_with("class");
                let source = if statement {
                    nested
                } else {
                    format!("x = {nested};")
                };
                ((*name).to_owned(), source)
            })
            .collect();
        sources.push((
            "yield".to_owned(),
            format!("function* g() {{ {}1 }}", "yield ".repeat(levels)),
        ));
        sources.push((
            "await in a function".to_owned(),
            format!("async function f() {{ {}1 }}", "await ".repeat(levels)),
        ));
        sources.push((
            "nested ternaries".to_owned(),
            format!("x = {}1{};", "a ? ".repeat(levels), " : 1".repeat(levels)),
        ));
        sources.push((
            "else if with blocks".to_owned(),
            format!("if (a) {{}}{}", " else if (a) {}".repeat(levels)),
        ));
        sources.push((
            "object patterns".to_owned(),
            format!("({} = c);", nest("{a: ", "b", "}", levels)),
        ));
        sources.push((
            "array patterns".to_owned(),
            format!("{} = c;", nest("[", "b", "]", levels)),
        ));
        sources.push((
            "patterns in parameters".to_owned(),
            format!("x = ({}) => 1;", nest("{a: [", "b", "]}", levels)),
        ));
        sources.push((
            "decorated class expressions".to_owned(),
            format!("x = {};", nest("(@", "a", " class {})", levels)),
        ));
        sources.push((
            "mixed".to_owned(),
            format!(
                "x = {}1{};",
                "[(a) => { if (b) return c ? d : `${({ m() { return ".repeat(levels),
                "} })}` }]".repeat(levels)
            ),
        ));
        sources
    };
    for (scanned, (name, _)) in scan_named_on_a_megabyte(build(DEEP))
        .iter()
        .zip(build(DEEP))
    {
        assert_eq!(*scanned, KEPT, "{name}");
    }
    // Three levels of each are a script the parser accepts.
    let wrong: Vec<_> = scan_named_on_a_megabyte(build(3))
        .into_iter()
        .zip(build(3))
        .filter(|(scanned, _)| *scanned != CLEAN)
        .map(|(scanned, (name, _))| (name, scanned))
        .collect();
    assert_eq!(wrong, [], "not clean");
}

/// Guards the one reading the bound leaves opaque: the bound counts nothing inside a
/// regular expression literal, which is only safe while `oxc_parser` does not descend into the
/// pattern. If a version of `oxc_parser` parsed regex internals by default (its
/// `parse_regular_expression` option), 100 000 nested groups, `v`-flag classes or lookbehinds
/// would pass the bound and overflow the stack; this test would then abort, and the bound would
/// have to count the pattern's brackets.
#[test]
fn nested_regular_expression_literals_are_parsed_without_the_parser_descending_into_them() {
    let sources = vec![
        (
            "groups".to_owned(),
            format!("x = /{}a{}/;", "(".repeat(DEEP), ")".repeat(DEEP)),
        ),
        (
            "v-flag classes".to_owned(),
            format!("x = /{}a{}/v;", "[".repeat(DEEP), "]".repeat(DEEP)),
        ),
        (
            "lookbehinds".to_owned(),
            format!("x = /{}a{}/;", "(?<=".repeat(DEEP), ")".repeat(DEEP)),
        ),
    ];
    assert_eq!(scan_named_on_a_megabyte(sources), [CLEAN, CLEAN, CLEAN]);
}

#[test]
fn long_flat_scripts_are_parsed() {
    // What a parser loops over costs no stack: these are read at any length.
    let flat: Vec<(String, String)> = [
        ("a sum", format!("x = a{};", " + b".repeat(DEEP))),
        ("a quotient", format!("x = a{};", "/b".repeat(DEEP))),
        ("a relation", format!("x = a{};", " in b".repeat(DEEP))),
        ("a member chain", format!("x = a{};", ".b".repeat(DEEP))),
        ("an optional chain", format!("x = a{};", "?.b".repeat(DEEP))),
        ("a call chain", format!("x = a{};", "()".repeat(DEEP))),
        (
            "a tagged template chain",
            format!("x = a{};", "`x`".repeat(DEEP)),
        ),
        (
            "a comma list",
            format!("x = [a{}];", ", b = 1".repeat(DEEP)),
        ),
        (
            "a sequence",
            format!("a = 1{};", ", b = c ? d : e".repeat(DEEP)),
        ),
        ("statements", "x = a ? b : c + d * e;\n".repeat(DEEP)),
        (
            "statements without semicolons",
            "const x = a ? b : c + d * e\nfoo(x)\n".repeat(DEEP),
        ),
        (
            "function declarations",
            "function f(a) { return a ? b : c }\n".repeat(DEEP),
        ),
        (
            "if blocks",
            "if (a) { b = c } else { d = e }\nwhile (f) { g = !h }\n".repeat(DEEP),
        ),
        (
            "switch cases",
            format!(
                "switch (a) {{{}}}",
                " case 1: case 2: b = c; break;".repeat(DEEP)
            ),
        ),
        ("decorators", format!("{}class A {{}}", "@a ".repeat(DEEP))),
        (
            "class members",
            format!(
                "class A {{{}}}",
                " m() { return 1 } static x = 2;".repeat(DEEP)
            ),
        ),
        (
            "regular expressions and divisions",
            "x = a / b / /[/]\"/.test(c) + (d) / 2;\n".repeat(DEEP),
        ),
    ]
    .into_iter()
    .map(|(name, source)| (name.to_owned(), source))
    .collect();
    let names: Vec<String> = flat.iter().map(|(name, _)| name.clone()).collect();
    for (scanned, name) in scan_named_on_a_megabyte(flat).iter().zip(names) {
        assert_eq!(*scanned, CLEAN, "{name}");
    }
}

/// `open` repeated a thousand levels deep.
fn deep(open: &str) -> String {
    open.repeat(DEEP)
}

/// Sources that nest or never close one construct inside another, a thousand levels deep.
fn nesting_sources() -> Vec<(&'static str, String)> {
    let parens = nest("(", "1", ")", DEEP);
    vec![
        ("unclosed parentheses", deep("(")),
        ("unclosed braces", deep("{")),
        ("unclosed brackets", format!("x={}", deep("["))),
        ("unclosed substitutions", deep("`${")),
        ("closers only", deep(")")),
        ("closing braces only", deep("}")),
        (
            "closers, then openers",
            format!("{}{}", deep(")]}"), deep("([{")),
        ),
        ("mismatched brackets", deep("(]")),
        ("mismatched brackets, the other way", deep("[)")),
        ("mismatched braces", deep("{)")),
        ("openers of three kinds", deep("([{")),
        ("not", format!("x={}", deep("!"))),
        ("minus", format!("x={}", deep("- "))),
        ("decrements", format!("x={}a", deep("--"))),
        ("increments", format!("x={}a", deep("++"))),
        ("increments on lines", format!("x={}a", deep("++\n"))),
        (
            "an arrow's block before a division",
            format!("x = () => {{}}\n/ {parens} / 1;"),
        ),
        (
            "a postfix operator, a line, a quote in slashes",
            format!("a++\n/\"/.test(s), {parens};"),
        ),
        ("increments on a name", format!("x=a{}", deep("++"))),
        ("new", format!("x={}", deep("new "))),
        ("arrows", format!("x={}", deep("a=>"))),
        ("ternaries", format!("x={}", deep("a?b:"))),
        ("questions", format!("x={}", deep("a?"))),
        ("colons", format!("x={}", deep("a:"))),
    ]
}

/// Sources of statements and expressions that nest, or never end, a thousand levels deep.
fn statement_sources() -> Vec<(&'static str, String)> {
    let parens = nest("(", "1", ")", DEEP);
    vec![
        ("if", deep("if(a)")),
        ("if without a head", deep("if ")),
        ("else", deep("else ")),
        ("else if", format!("if(a){{}}{}", deep("else if(a)"))),
        ("labels", deep("a:")),
        ("await", format!("x={}", deep("await "))),
        ("typeof", format!("x={}", deep("typeof "))),
        ("exponents", format!("x=1{}", deep("**1**"))),
        ("assignments", deep("a=")),
        ("assignments to numbers", deep("1=")),
        ("yield", format!("function* g(){{{}", deep("yield "))),
        ("do", deep("do ")),
        ("do with a separator outside ASCII", deep("do\u{a0}")),
        ("if with a separator outside ASCII", deep("if(a)\u{2003}")),
        ("for", deep("for(;;)")),
        ("for of", deep("for(a of b)")),
        ("while", deep("while(a)")),
        ("with", deep("with(a)")),
        ("spread", format!("x=[{}", deep("..."))),
        ("import calls", deep("import(")),
        ("tags", deep("<a>")),
        ("class heritage", deep("class A extends ")),
        (
            "class heritage, unnamed",
            format!("x={}", deep("class extends ")),
        ),
        ("class bodies", deep("class A{m(){")),
        ("functions", deep("function f(){")),
        ("function heads", deep("function f(a=")),
        ("arrows with blocks", format!("x={}", deep("()=>{"))),
        ("case", format!("switch(a){{{}", deep("case "))),
        (
            "case clauses",
            format!("switch(a){{{}", deep("case 1:if(a)")),
        ),
        ("switches", deep("switch(a){case 1:")),
        ("try", deep("try{")),
        ("const", deep("const a=")),
        ("if const", deep("if(a)const b=1;else ")),
        ("return", deep("return ")),
        ("throw", deep("throw ")),
        ("exports", deep("export default ")),
        ("decorators", deep("@")),
        (
            "decorators of a class",
            format!("x = {}a class {{}};", deep("@")),
        ),
        ("decorated classes", deep("x=@a class{}/")),
        ("private names", deep("#a in ")),
        ("optional calls", deep("a?.(")),
        ("templates", deep("`")),
        ("template tails", deep("}`${")),
        (
            "an unterminated regular expression, then brackets",
            format!("x = /[\n{parens};"),
        ),
        (
            "an unterminated string, then brackets",
            format!("x = \"a\n{parens};"),
        ),
        (
            "an unterminated string after a regex, then brackets",
            format!("x = /a/ + 'b\n{parens};"),
        ),
        (
            "a regular expression per line",
            format!("x = /a\n{}", "(/[\n".repeat(DEEP)),
        ),
        ("a string per line", "(\"\n".repeat(DEEP)),
        ("a stray backslash", format!("x = \\ {parens};")),
        ("a number with a name", format!("x = 1a + {parens};")),
        ("a keyword escaped", format!("x = \\u0074ypeof {parens};")),
    ]
}

/// Sources that hide the closers of fifty open parentheses in what one reading takes for text.
fn closer_sources() -> Vec<(&'static str, String)> {
    // A block of fifty levels that an error or a second reading may leave open, a thousand
    // times over: `@` is what stands between the openers and the closers.
    let block = |between: &str, tail: &str| {
        format!(
            "{}{}{tail}",
            "(".repeat(50),
            between.replace('@', &")".repeat(50))
        )
        .repeat(2_000)
    };
    vec![
        // The closers stand in what one reading takes for text.
        ("a name before a regex-like run", block(" x /@/ ", "")),
        ("a name, a line, a regex-like run", block(" x\n/@/ ", "")),
        (
            "a field before a regex-like run",
            format!("class A {{ {} }}", block(" x /@/ ", ";")),
        ),
        ("closers in a string that does not end", block(" \"@\n", "")),
        ("closers in a string", block(" \"@\" ", "")),
        ("closers in a template", block(" `@` ", "")),
        ("closers in a comment", block(" /*@*/ ", "")),
        ("closers in a line comment", block(" //@\n", "")),
        (
            "closers behind an HTML comment opener",
            block(" <!--@\n", ""),
        ),
        (
            "closers behind an HTML comment closer",
            block("\n-->@\n", ""),
        ),
        (
            "closers in a regex after a parenthesis",
            block(" (a) /@/ ", ""),
        ),
        ("closers in a regex after a brace", block(" {} /@/ ", "")),
        (
            "closers in a regex after a keyword",
            block(" return /@/ ", ""),
        ),
        ("closers in a regex class", block(" /[@/]/ ", "")),
        ("closers in a regex that does not end", block(" = /@\n", "")),
        (
            "closers after a quote in a regex",
            block(" = /\"/ + \"@\" ", ""),
        ),
        (
            "closers in a substitution's string",
            block(" `${\"@\"}` ", ""),
        ),
        ("closers after a template's brace", block(" `${a}@` ", "")),
        ("closers after an escaped quote", block(" \"\\\"@\" ", "")),
        (
            "closers after an escaped backslash",
            block(" \"\\\\\"@\"\\\\\" ", ""),
        ),
    ]
}

#[test]
fn no_script_with_a_syntax_error_overflows_the_stack() {
    let mut sources = nesting_sources();
    sources.extend(statement_sources());
    sources.extend(closer_sources());
    let named: Vec<(String, String)> = sources
        .into_iter()
        .map(|(name, source)| (name.to_owned(), source))
        .collect();
    // Whatever the scan says of each, it returns.
    let count = named.len();
    assert_eq!(scan_named_on_a_megabyte(named).len(), count);
}
