// The hostile texts of the lexer characterization, shared by `tests/characterize_lex.rs` and
// the crate's own `json::characterize_lex` (for the scanners that are not public). Each is run
// through every scanner of the door that the fold of phase 7 touches; the expected lines are
// what the scanners answer at main cbcf47859.

/// The texts, short enough to print whole.
const TEXTS: &[&str] = &[
    r#"{"a":1}"#,
    r#"{"a":"\q"}"#,
    r#"{"a":"\u12"}"#,
    r#"{"a":"\u12"#,
    r#"{"a":"abc"#,
    r#"{"a":"x\\","b":2}"#,
    r#"{"a":"\/\b\f\n\r\t\u00e9"}"#,
    r#"{"a":1 /* x"#,
    r#"{"a":1} /* x"#,
    "{\"a\":\u{c}1}",
    "{\"a\":1}\u{c}",
    "\u{c}{\"a\":1}",
    "{\"a\":\u{a0}1}",
    "\u{a0}{\"a\":1}",
    r#"{"a":[1}]"#,
    "[}",
    "}",
    "]]]{",
    r#""str""#,
    "12 garbage",
    "-",
    "1.5e+",
    "nul",
    r#"{"a":1} trailing"#,
    "// c\n{\"a\":1}",
    "/* c */ {\"a\":1}",
    "/* c */ [1]",
    r#"{"a/b":1,"c~d":{"e":[0,1e39]}}"#,
    r#"{"a":1,"a":2}"#,
    r#"{"a\u0062":1,"ab":2}"#,
    r#"{"a":{"b":1,"b":2},"a":{"c":3,"c":4}}"#,
    r#"{"k":{"a":1,"a":2},"k":3}"#,
    r#"{"a":1,"a":2,"b":tru}"#,
    r#"{"a":tru,"a":2}"#,
    r#"{"a":[1,2,],}"#,
    r#"{,,"a":1}"#,
    r#"{"a" 1}"#,
    r#"{"a":1 "b":2}"#,
    "",
    "   ",
    "// only",
    "/* only",
    "/* only */",
    "[ /* c */ ]",
    "[ // c\n ]",
    "[ ]",
    "[\u{a0}]",
    r#"["[ ]", 1]"#,
    r#"{"a":1e999,"b":-1e400,"c":"1e999"}"#,
    r#"{"a":1e39,"a":2}"#,
    r#"{"a":2,"a":1e39}"#,
    r#"{"a":1e39,"b":{"c":-1e39},"a":3e39}"#,
    r#"{"a":{"x":1e39},"a":{"y":2e39}}"#,
    r#"{"b":1e39,"a":1e39,"b":2e39}"#,
    r#"[1e39,[-1e39],{"a\"b":1e39}]"#,
    r#"{"a": "]}" // [
, "b": 1}"#,
    "{\"a\":[[[\"\\q\"]]]}",
    "[[\"\\q\",[[[[]]]]]]",
    r#"{"a":{"b":{"c":{"d":1}}}}"#,
];

/// Texts too long to print: the depth limit, with what lies around it.
fn deep_texts() -> Vec<(String, String)> {
    let nest = |n: usize, inner: &str| format!("{}{inner}{}", "[".repeat(n), "]".repeat(n));
    vec![
        ("128 levels".into(), nest(128, "")),
        ("129 levels".into(), nest(129, "")),
        ("129 levels after a bad escape".into(), format!("[\"\\q\",{}]", nest(128, ""))),
        ("129 levels in a comment".into(), format!("[/*{}*/]", "[".repeat(129))),
        ("129 levels in a string".into(), format!("[\"{}\"]", "[".repeat(129))),
        ("129 levels after the root".into(), format!("[] {}", nest(129, ""))),
        ("128 objects".into(), format!("{}1{}", "{\"a\":".repeat(128), "}".repeat(128))),
        ("129 objects".into(), format!("{}1{}", "{\"a\":".repeat(129), "}".repeat(129))),
        (
            "duplicate at 127 levels".into(),
            format!("{}{{\"a\":1,\"a\":2}}{}", "[".repeat(127), "]".repeat(127)),
        ),
        (
            "duplicate at 128 levels".into(),
            format!("{}{{\"a\":1,\"a\":2}}{}", "[".repeat(128), "]".repeat(128)),
        ),
    ]
}
