//! `json::clamp_non_finite` and `json::span_at`: the one walk that replaces
//! the numbers no double holds in a value kept as written, and the span of a value by its
//! path.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::{
    Clamped, JsonSlice, Value, clamp_non_finite, clamp_non_finite_warned, from_str_jsonc, span_at,
    to_json,
};

fn path(steps: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for step in steps {
        match step.parse::<usize>() {
            Ok(index) if !step.starts_with('k') => path.push_index(index),
            _ => path.push(step),
        }
    }
    path
}

fn read(text: &str) -> Value {
    from_str_jsonc(text).unwrap()
}

/// The text a span covers.
fn covered(text: &str, span: Span) -> &str {
    &text[span.offset as usize..(span.offset + span.len) as usize]
}

#[test]
fn an_infinity_is_replaced_by_the_largest_finite_double_of_its_sign() {
    let wide = "9".repeat(310);
    let text = format!(
        r#"{{"a": 1e999, "b": [1, -1e999, {{"c": {wide}, "d": -{wide}}}], "e": 1e308, "f": 1e39,
            "g": "1e999", "h": 18446744073709551616, "i": [], "j": null, "k": -1e400}}"#
    );
    let mut value = read(&text);
    let clamped = clamp_non_finite(&mut value, None);
    let paths: Vec<String> = clamped.iter().map(|c| c.path.to_string()).collect();
    // Document order.
    assert_eq!(paths, ["a", "b/1", "b/2/c", "b/2/d", "k"]);
    assert!(clamped.iter().all(|c| c.span.is_none()));
    let expected = read(
        r#"{"a": 1.7976931348623157e308, "b": [1, -1.7976931348623157e308,
            {"c": 1.7976931348623157e308, "d": -1.7976931348623157e308}], "e": 1e308,
            "f": 1e39, "g": "1e999", "h": 18446744073709551616, "i": [], "j": null,
            "k": -1.7976931348623157e308}"#,
    );
    assert_eq!(value, expected);
    // What is written reads back equal, with nothing left to replace.
    let written = to_json(&value).unwrap();
    let mut again = read(&written);
    assert_eq!(again, value);
    assert!(clamp_non_finite(&mut again, None).is_empty());
}

#[test]
fn a_value_without_an_infinity_is_left_as_it_is() {
    for text in [
        "1",
        "1e308",
        "-1.7976931348623157e308",
        "[1, 2.5, \"x\", null, true, {\"a\": []}]",
        "{}",
        "\"1e999\"",
    ] {
        let mut value = read(text);
        let before = value.clone();
        // The text is not looked at: one that is not the value's own changes nothing.
        let clamped = clamp_non_finite(&mut value, Some(JsonSlice::new("[1e999]")));
        assert!(clamped.is_empty(), "`{text}`");
        assert_eq!(value, before, "`{text}`");
    }
    // A root that is itself the number.
    let mut root = read("-1e999");
    let clamped = clamp_non_finite(&mut root, Some(JsonSlice::new("-1e999")));
    assert_eq!(clamped.len(), 1);
    assert!(clamped[0].path.is_empty());
    assert_eq!(clamped[0].span, Some(Span { offset: 0, len: 6 }));
    assert_eq!(root, read("-1.7976931348623157e308"));
}

#[test]
fn each_replaced_number_comes_with_the_span_of_its_token() {
    let wide = "9".repeat(310);
    let text = format!(
        "{{ // c 1e999\n  \"a\\n\\u00e9\": 1e999, /* 1e999 */ \"b\": [ {wide} , \"1e999\", \
         -1E+999 ], \"\": {{\"k\": [[1.5e999]]}}, \"z\": 1e30 }} 1e999"
    );
    let file = format!("  \n{text}");
    let slice = JsonSlice {
        text: &file[3..],
        offset: 3,
    };
    let mut value = read(&text);
    let clamped = clamp_non_finite(&mut value, Some(slice));
    let found: Vec<(String, &str)> = clamped
        .iter()
        .map(|c| (c.path.to_string(), covered(&file, c.span.unwrap())))
        .collect();
    assert_eq!(
        found,
        [
            ("a\né".to_owned(), "1e999"),
            ("b/0".to_owned(), wide.as_str()),
            ("b/2".to_owned(), "-1E+999"),
            ("/k/0/0".to_owned(), "1.5e999"),
        ]
    );
    // The first is the member's token, not the comment's.
    assert_eq!(
        clamped[0].span.unwrap().offset as usize,
        file.find(": 1e999").unwrap() + 2
    );
    assert_eq!(clamped[0].path, path(&["a\né"]));
    assert_eq!(clamped[3].path, path(&["", "k", "0", "0"]));
}

/// A key written twice keeps its last value: the span is the last token's.
#[test]
fn a_number_of_a_key_written_twice_has_the_span_of_the_last() {
    let text =
        r#"{"a": 1e999, "b": {"x": -1e999}, "a": 2e999, "b": {"x": -3e999}, "c": [1e999], "c": 5}"#;
    let mut value = read(text);
    let clamped = clamp_non_finite(&mut value, Some(JsonSlice::new(text)));
    let found: Vec<(String, &str)> = clamped
        .iter()
        .map(|c| (c.path.to_string(), covered(text, c.span.unwrap())))
        .collect();
    assert_eq!(
        found,
        [("a".to_owned(), "2e999"), ("b/x".to_owned(), "-3e999")]
    );
}

/// A text that is not the value's own gives no span where it has no number; the value is
/// clamped all the same.
#[test]
fn a_number_the_text_does_not_have_has_no_span() {
    let mut value = read(r#"{"a": 1e999, "b": 1e999}"#);
    let clamped = clamp_non_finite(&mut value, Some(JsonSlice::new(r#"{"a": 1, "b": 1e999"#)));
    assert_eq!(clamped.len(), 2);
    assert_eq!(clamped[0].span, None);
    assert_eq!(clamped[1].span, Some(Span { offset: 14, len: 5 }));
    let clamped: Vec<Clamped> = clamp_non_finite(&mut value, None);
    assert!(clamped.is_empty());
}

/// A hand-built value is walked on a list, not on the stack; a NaN is left.
#[test]
fn a_hand_built_value_is_walked_whatever_it_holds() {
    let mut array = facet_value::VArray::new();
    array.push(Value::from(f64::INFINITY));
    array.push(Value::from(f64::NAN));
    array.push(Value::from(f64::NEG_INFINITY));
    let mut object = facet_value::VObject::new();
    object.insert("list", Value::from(array));
    let mut value = Value::from(object);
    let clamped = clamp_non_finite(&mut value, None);
    let paths: Vec<String> = clamped.iter().map(|c| c.path.to_string()).collect();
    assert_eq!(paths, ["list/0", "list/2"]);
    let list = value.as_object().unwrap().get("list").unwrap();
    let numbers: Vec<f64> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_number().unwrap().to_f64_lossy())
        .collect();
    assert_eq!(numbers[0].to_bits(), f64::MAX.to_bits());
    assert!(numbers[1].is_nan());
    assert_eq!(numbers[2].to_bits(), f64::MIN.to_bits());
}

/// The walk keeps one path, stepping in and out: the path of a number below a long chain of
/// arrays and objects is its own, and the one of a number after the chain is not the
/// chain's, so the path is not copied at every node.
#[test]
fn a_number_below_and_after_a_deep_chain_has_its_own_path() {
    const DEPTH: usize = 600;
    let mut expected = JsonPath::new();
    expected.push("chain");
    let mut inner = Value::from(f64::NEG_INFINITY);
    for level in (0..DEPTH).rev() {
        if level % 2 == 0 {
            let mut array = facet_value::VArray::new();
            array.push(Value::from(1.5));
            array.push(inner);
            inner = Value::from(array);
        } else {
            let mut object = facet_value::VObject::new();
            object.insert("k", inner);
            object.insert("after", Value::from(2.5));
            inner = Value::from(object);
        }
    }
    for level in 0..DEPTH {
        if level % 2 == 0 {
            expected.push_index(1);
        } else {
            expected.push("k");
        }
    }
    let mut root = facet_value::VObject::new();
    root.insert("chain", inner);
    root.insert("after", Value::from(f64::INFINITY));
    let mut value = Value::from(root);
    let clamped = clamp_non_finite(&mut value, None);
    assert_eq!(clamped.len(), 2);
    assert_eq!(clamped[0].path, expected);
    assert_eq!(clamped[0].path.len(), DEPTH + 1);
    assert_eq!(clamped[1].path, path(&["after"]));
    assert!(clamp_non_finite(&mut value, None).is_empty());
}

#[test]
fn span_at_is_the_span_of_the_value_a_path_names() {
    let text = "{\"a\": [1, {\"b\\n\": \"x y\" /* c */ }, [ ] ], \"a2\": 1e999, \"\": null}";
    let file = format!("\u{feff}{text}");
    let slice = JsonSlice {
        text: &file[3..],
        offset: 3,
    };
    let at = |steps: &[&str]| span_at(slice, &path(steps)).map(|span| covered(&file, span));
    assert_eq!(at(&[]), Some(text));
    assert_eq!(at(&["a", "0"]), Some("1"));
    assert_eq!(at(&["a", "1", "b\n"]), Some("\"x y\""));
    assert_eq!(at(&["a", "2"]), Some("[ ]"));
    assert_eq!(at(&["a2"]), Some("1e999"));
    assert_eq!(at(&[""]), Some("null"));
    // A step that is not there.
    assert_eq!(at(&["a", "3"]), None);
    assert_eq!(at(&["b"]), None);
    assert_eq!(at(&["a", "0", "x"]), None);
    assert_eq!(at(&["a2", "0"]), None);
    assert_eq!(span_at(slice, &path(&["a", "0"])).unwrap().offset, 3 + 7);
    // A key written twice: the last, the one the readers keep.
    let twice = r#"{"k": 1, "k": [2, 3]}"#;
    let span = span_at(JsonSlice::new(twice), &path(&["k"])).unwrap();
    assert_eq!(covered(twice, span), "[2, 3]");
    let span = span_at(JsonSlice::new(twice), &path(&["k", "1"])).unwrap();
    assert_eq!(covered(twice, span), "3");
}

// ---- clamp_text: numbers beyond their member, clamped in the text ----

mod clamp_text {
    use bedrock_addon_core::diagnostic::{
        Diagnostics, JsonPath, JsonPathSegment, Phase, Severity, Span,
    };
    use bedrock_addon_core::json::{self, JsonSlice, Value, Width, clamp_text};

    fn width(path: &[JsonPathSegment<'_>]) -> Option<Width> {
        match path.first() {
            Some(JsonPathSegment::Key("float" | "list")) => Some(Width::Float),
            Some(JsonPathSegment::Key("kept")) => Some(Width::Double),
            _ => None,
        }
    }

    #[test]
    fn a_number_beyond_its_width_is_replaced_in_the_text_with_one_information_at_its_token() {
        let text =
            r#"{"float": 1e39, "other": 1e39, "kept": [1e39, -1e999], "list": [{"a": -1E400}, 2]}"#;
        let json: Value = json::from_str(text).unwrap();
        let mut at = JsonPath::new();
        at.push("root");
        let mut d = Diagnostics::default();
        // The slice lies 100 bytes into its file.
        let slice = JsonSlice { text, offset: 100 };
        let clamped = clamp_text(slice, &json, &width, &at, &mut d).unwrap();
        assert_eq!(
            clamped.text,
            r#"{"float": 3.4028235e38, "other": 1e39, "kept": [1e39, -1.7976931348623157e308], "list": [{"a": -3.4028235e38}, 2]}"#
        );
        let found: Vec<(String, &str)> = d
            .iter()
            .map(|f| {
                assert_eq!(f.code.id(), "core/invalid_value");
                assert_eq!(f.code.to_string(), "invalid value");
                assert_eq!(f.severity, Severity::Info);
                assert_eq!(f.phase, Phase::Parse);
                let span = f.location.span.unwrap();
                let from = span.offset as usize - 100;
                (
                    f.location.json_path.to_string(),
                    &text[from..from + span.len as usize],
                )
            })
            .collect();
        assert_eq!(
            found,
            [
                ("root/float".to_owned(), "1e39"),
                ("root/kept/1".to_owned(), "-1e999"),
                ("root/list/0/a".to_owned(), "-1E400"),
            ]
        );
        // A place in the clamped text is named as the same place in the text as written.
        let in_clamped = clamped.text.find(r#""other""#).unwrap();
        let as_written = clamped.written_span(
            Span {
                offset: u32::try_from(100 + in_clamped).unwrap(),
                len: 7,
            },
            100,
        );
        assert_eq!(
            as_written.offset as usize,
            100 + text.find(r#""other""#).unwrap()
        );
        assert_eq!(as_written.len, 7);
        // A span inside a replaced number is the number as written.
        let inside = clamped.text.find("3.4028235e38").unwrap() + 3;
        let as_written = clamped.written_span(
            Span {
                offset: u32::try_from(100 + inside).unwrap(),
                len: 2,
            },
            100,
        );
        assert_eq!(as_written.offset as usize, 100 + text.find("1e39").unwrap());
        assert_eq!(as_written.len, 4);
    }

    #[test]
    fn a_text_whose_numbers_fit_is_left_alone() {
        for text in [
            r#"{"float": 3.4028235e38, "kept": [1e39, 1e308], "other": 1e999}"#,
            r#"{"float": "1e39", "list": [true, null, -3.4028235e38]}"#,
        ] {
            let json: Value = json::from_str(text).unwrap();
            let mut d = Diagnostics::default();
            let clamped = clamp_text(
                JsonSlice::new(text),
                &json,
                &width,
                &JsonPath::new(),
                &mut d,
            );
            assert!(clamped.is_none(), "{text}");
            assert!(d.is_empty(), "{text}");
        }
    }
}

// ---- clamp_beyond_double: a whole document read at the width of a double ----

mod clamp_beyond_double {
    use bedrock_addon_core::diagnostic::{
        Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, Span, codes,
    };
    use bedrock_addon_core::json::{ClampPlan, ClampedText, JsonSlice, clamp_tokens};

    /// The clamp of a document read at the width of a double.
    fn clamp_beyond_double(
        slice: JsonSlice<'_>,
        at: &JsonPath,
        d: &mut Diagnostics,
    ) -> Option<ClampedText> {
        clamp_tokens(slice, &ClampPlan::doubles(), at, d)
    }

    #[test]
    fn a_text_without_a_number_beyond_a_double_is_left_alone() {
        let wide = "9".repeat(308);
        for text in [
            "{}".to_owned(),
            r#"{"a": 1e308, "b": 1e39, "c": -1.7976931348623157e308, "d": [0.5, 12]}"#.to_owned(),
            // In a string and in a comment it is no number.
            r#"{"a": "1e999", "1e999": 1 /* 1e999 */, "b": 2 // -1e999
            }"#
            .to_owned(),
            format!(r#"{{"a": {wide}}}"#),
            // No JSON: nothing to clamp, the reader says what it is.
            r#"{"a": "\q", "b": 1e999}"#.to_owned(),
            "{\"a\": 1e999".to_owned(),
        ] {
            let mut d = Diagnostics::default();
            let clamped = clamp_beyond_double(JsonSlice::new(&text), &JsonPath::new(), &mut d);
            assert!(clamped.is_none(), "{text}");
            assert!(d.is_empty(), "{text}");
        }
    }

    #[test]
    fn every_number_beyond_a_double_is_replaced_with_an_information_on_its_token() {
        let wide = "1234567890".repeat(40);
        let text = format!(
            r#"{{"a": 1e999, /* 1e999 */ "b": [0, -1E400, {{"c": {wide}}}], "d": 1e39, "e": "1e999", "f": true}}"#
        );
        let mut at = JsonPath::new();
        at.push("root");
        let mut d = Diagnostics::default();
        let slice = JsonSlice {
            text: &text,
            offset: 10,
        };
        let clamped = clamp_beyond_double(slice, &at, &mut d).expect("three numbers");
        assert_eq!(
            clamped.text,
            r#"{"a": 1.7976931348623157e308, /* 1e999 */ "b": [0, -1.7976931348623157e308, {"c": 1.7976931348623157e308}], "d": 1e39, "e": "1e999", "f": true}"#
        );
        let found: Vec<(String, &str)> = d
            .iter()
            .map(|f| {
                assert_eq!(f.code.id(), "core/invalid_value");
                assert_eq!(f.severity, Severity::Info);
                let span = f.location.span.unwrap();
                let from = span.offset as usize - 10;
                (
                    f.location.json_path.to_string(),
                    &text[from..from + span.len as usize],
                )
            })
            .collect();
        assert_eq!(
            found,
            [
                ("root/a".to_owned(), "1e999"),
                ("root/b/1".to_owned(), "-1E400"),
                ("root/b/2/c".to_owned(), wide.as_str()),
            ]
        );

        // A reading of the clamped text names `true`; the finding is of the file as written.
        let in_clamped = clamped.text.find("true").unwrap();
        let mut later = Diagnostics::default();
        for span in [
            Some(Span {
                offset: u32::try_from(10 + in_clamped).unwrap(),
                len: 4,
            }),
            None,
        ] {
            later.push(Diagnostic::new(
                Code::from(&codes::INVALID_TYPE),
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, JsonPath::new(), span),
            ));
        }
        clamped.restore_spans(&mut later, 10);
        let spans: Vec<Option<Span>> = later.iter().map(|f| f.location.span).collect();
        assert_eq!(
            spans,
            [
                Some(Span {
                    offset: u32::try_from(10 + text.find("true").unwrap()).unwrap(),
                    len: 4
                }),
                None
            ]
        );
    }
}

/// The finding of each replaced number is an information of the crate's own: the game's
/// jsoncpp reads the token with `sscanf("%lf")` and holds the infinity in silence, so the game
/// reports nothing here; only the write-back needs the finite number.
#[test]
fn a_replaced_number_is_told_as_an_information() {
    let text = r#"{"a": [1e999, -1e400]}"#;
    let mut value = read(text);
    let mut d = Diagnostics::default();
    let at = path(&["root"]);
    assert!(clamp_non_finite_warned(
        &mut value,
        Some(JsonSlice::new(text)),
        &at,
        &mut d
    ));
    let found: Vec<(String, Severity, &str)> = d
        .iter()
        .map(|f| {
            (
                f.location.json_path.to_string(),
                f.severity,
                covered(text, f.location.span.unwrap()),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("root/a/0".to_owned(), Severity::Info, "1e999"),
            ("root/a/1".to_owned(), Severity::Info, "-1e400"),
        ]
    );
}
