//! The normaliser against `normalize.toml`: one row per line of the observed table of
//! The schema, and its cost.

use bedrock_addon::families::script::{EngineCrash, ScriptPath, normalize};

use super::common::{fixtures, member, tables};

/// The rows of `normalize.toml`: `(referrer, spec, known, expected)`.
fn rows() -> Vec<(String, String, String, String)> {
    let text = std::fs::read_to_string(fixtures().join("normalize.toml")).unwrap();
    tables(&text)
        .into_iter()
        .filter(|(name, _)| name == "rows")
        .map(|(_, table)| {
            (
                member(&table, "referrer"),
                member(&table, "spec"),
                member(&table, "known"),
                member(&table, "expected"),
            )
        })
        .collect()
}

#[test]
fn every_row_of_the_observed_table_normalises_as_observed() {
    let rows = rows();
    // 46 rows of the table and 12 for the branches it has no row for.
    assert_eq!(rows.len(), 58);
    for (referrer, spec, known, expected) in rows {
        let found = normalize(&ScriptPath::new(referrer.as_str()), &spec, &|name| {
            !known.is_empty() && name == known
        });
        let wanted = if expected == "!" {
            Err(EngineCrash)
        } else {
            Ok(expected)
        };
        let shown: String = spec.chars().take(40).collect();
        assert_eq!(found, wanted, "`{shown}` from {referrer}");
    }
}

#[test]
fn a_path_of_1023_bytes_is_looked_up_and_one_of_1024_is_not() {
    let main = ScriptPath::new("main.js");
    let none = |_: &str| false;
    // Bare: the specifier is the path.
    let fits = "a".repeat(1023);
    assert_eq!(normalize(&main, &fits, &none), Ok(format!("{fits}.js")));
    assert_eq!(
        normalize(&main, &"a".repeat(1024), &none),
        Ok(".js".to_owned())
    );
    // Relative: the path is what follows the leading run, behind the importing file's folder.
    let deep = ScriptPath::new("dir/main.js");
    let fits = format!("./{}", "b".repeat(1023 - 4));
    assert_eq!(
        normalize(&deep, &fits, &none),
        Ok(format!("dir/{}.js", "b".repeat(1019)))
    );
    let over = format!("./{}", "b".repeat(1024 - 4));
    assert_eq!(normalize(&deep, &over, &none), Ok(String::new()));
    // More `../` than folders: the whole specifier is the path.
    let fits = format!("../{}", "c".repeat(1020));
    assert_eq!(normalize(&main, &fits, &none), Ok(format!("{fits}.js")));
    let over = format!("../{}", "c".repeat(1021));
    assert_eq!(normalize(&main, &over, &none), Ok(".js".to_owned()));
}

#[test]
fn the_leading_run_takes_one_folder_per_dot_dot() {
    let from = ScriptPath::new("a/b/c/file.js");
    let none = |_: &str| false;
    let found = |spec: &str| normalize(&from, spec, &none);
    assert_eq!(found("./x.js"), Ok("a/b/c/x.js".to_owned()));
    assert_eq!(found("../x.js"), Ok("a/b/x.js".to_owned()));
    assert_eq!(found("../../x.js"), Ok("a/x.js".to_owned()));
    // As many as there are folders: the path is what follows the run.
    assert_eq!(found("../../../x.js"), Ok("x.js".to_owned()));
    // One more: the whole specifier.
    assert_eq!(found("../../../../x.js"), Ok("../../../../x.js".to_owned()));
    assert_eq!(found("./.././../x.js"), Ok("a/x.js".to_owned()));
    // Only a leading run counts: `..` further in stays as written.
    assert_eq!(found("./d/../x.js"), Ok("a/b/c/d/../x.js".to_owned()));
}

#[test]
fn trailing_slashes_are_dropped_before_the_extension_is_looked_at() {
    // Two choices the folder is silent on: every trailing slash goes, not one; and
    // the extension is asked for of the specifier as written, where a trailing slash hides it.
    let main = ScriptPath::new("main.js");
    let none = |_: &str| false;
    let found = |spec: &str| normalize(&main, spec, &none);
    assert_eq!(found("./lib//"), Ok("lib.js".to_owned()));
    assert_eq!(found("./lib/"), Ok("lib.js".to_owned()));
    assert_eq!(found("./lib.js/"), Ok("lib.js.js".to_owned()));
    assert_eq!(found("./lib.js"), Ok("lib.js".to_owned()));
}

#[test]
fn normalising_is_linear_in_the_specifier_and_the_referrer() {
    let input = |n: usize| {
        let referrer = ScriptPath::new(format!("{}file.js", "d/".repeat(n * 50)));
        let spec = format!("{}{}", "../".repeat(n * 25), "x".repeat(600));
        (referrer, spec)
    };
    super::cost::assert_linear_at("script normalise", 2_000, input, |(referrer, spec)| {
        // Under the path limit only when the folders left are few; the run is scanned whole.
        normalize(referrer, spec, &|_| false)
    });
}
