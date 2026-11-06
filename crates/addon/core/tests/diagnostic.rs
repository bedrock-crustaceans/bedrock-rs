//! `Diagnostic`, `Diagnostics`, `Location`, `JsonPath`, `Span` and `PackReport`. Assertions match
//! on codes and values; the content-log line is checked for its shape (segment order and
//! separators) only.

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, PackReport,
    Phase, ReferenceHint, Severity, Span,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::load::{Place, RootId};
use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
};
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

fn finding(code: DiagnosticCode, severity: Severity) -> Diagnostic {
    Diagnostic::new(code, severity, Phase::Validate, Location::default())
}

fn friction_path() -> JsonPath {
    let mut path = JsonPath::new();
    path.push("minecraft:block");
    path.push("components");
    path.push("minecraft:friction");
    path
}

/// A warning, an error, an info and a second error, in that order.
fn mixed_findings() -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.push(finding(
        DiagnosticCode::InvalidValue { text: None },
        Severity::Error,
    ));
    diagnostics.push(finding(DiagnosticCode::NoSplashes, Severity::Info));
    diagnostics.push(finding(
        DiagnosticCode::MissingRequiredField { text: None },
        Severity::Error,
    ));
    diagnostics
}

fn report_at_memory_root() -> PackReport {
    PackReport::new(
        Place {
            root: RootId::MEMORY,
            path: PackPath::ROOT,
        },
        PackType::Behavior,
        PackOrigin::World,
    )
}

#[test]
fn severity_orders_debug_info_warning_error() {
    assert!(Severity::Debug < Severity::Info);
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert_eq!(
        [
            Severity::Error,
            Severity::Debug,
            Severity::Warning,
            Severity::Info
        ]
        .into_iter()
        .max(),
        Some(Severity::Error)
    );
}

#[test]
fn diagnostic_new_has_no_detail() {
    let diagnostic = finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    );
    assert_eq!(diagnostic.code, DiagnosticCode::UnknownSound { text: None });
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(diagnostic.phase, Phase::Validate);
    assert_eq!(diagnostic.location, Location::default());
    assert_eq!(diagnostic.detail, None);
}

#[test]
fn location_new_keeps_its_parts() {
    let span = Span { offset: 4, len: 2 };
    let location = Location::new(None, None, friction_path(), Some(span));
    assert_eq!(location.pack, None);
    assert_eq!(location.path, None);
    assert_eq!(location.json_path, friction_path());
    assert_eq!(location.span, Some(span));
}

#[test]
fn diagnostics_keep_insertion_order() {
    let diagnostics = mixed_findings();
    assert_eq!(diagnostics.len(), 4);
    assert!(!diagnostics.is_empty());
    let codes: Vec<_> = diagnostics.iter().map(|d| d.code.name()).collect();
    assert_eq!(
        codes,
        [
            "UnknownSound",
            "InvalidValue",
            "NoSplashes",
            "MissingRequiredField"
        ]
    );
    let by_ref: Vec<_> = (&diagnostics).into_iter().map(|d| d.severity).collect();
    let owned: Vec<_> = diagnostics.into_iter().map(|d| d.severity).collect();
    assert_eq!(by_ref, owned);
}

#[test]
fn empty_diagnostics_have_no_errors_and_no_severity() {
    let diagnostics = Diagnostics::default();
    assert!(diagnostics.is_empty());
    assert_eq!(diagnostics.len(), 0);
    assert!(!diagnostics.has_errors());
    assert_eq!(diagnostics.max_severity(), None);
    assert_eq!(diagnostics.first_error(), None);
    assert_eq!(diagnostics.at_least(Severity::Debug).count(), 0);
}

#[test]
fn has_errors_needs_an_error_level_finding() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    assert!(!diagnostics.has_errors());
    assert!(mixed_findings().has_errors());
}

#[test]
fn max_severity_is_the_most_severe() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(DiagnosticCode::NoSplashes, Severity::Info));
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.push(finding(DiagnosticCode::LiteralIgnored, Severity::Debug));
    assert_eq!(diagnostics.max_severity(), Some(Severity::Warning));
    assert_eq!(mixed_findings().max_severity(), Some(Severity::Error));
}

#[test]
fn at_least_filters_by_severity_in_order() {
    let diagnostics = mixed_findings();
    let warnings_up: Vec<_> = diagnostics
        .at_least(Severity::Warning)
        .map(|d| d.code.name())
        .collect();
    assert_eq!(
        warnings_up,
        ["UnknownSound", "InvalidValue", "MissingRequiredField"]
    );
    assert_eq!(diagnostics.at_least(Severity::Error).count(), 2);
    assert_eq!(diagnostics.at_least(Severity::Debug).count(), 4);
}

#[test]
fn first_error_is_the_earliest_error() {
    let diagnostics = mixed_findings();
    let first = diagnostics.first_error().expect("two errors");
    assert!(matches!(first.code, DiagnosticCode::InvalidValue { .. }));
}

#[test]
fn extend_appends_in_order() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.extend(mixed_findings());
    assert_eq!(diagnostics.len(), 5);
    assert!(matches!(
        diagnostics.iter().nth(1).map(|d| &d.code),
        Some(DiagnosticCode::UnknownSound { text: None })
    ));
}

/// A kept error then a rejecting one: the game keeps the document after the first, so the
/// error that decides about the document is the second.
fn kept_then_rejecting() -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push_kept_error(finding(
        DiagnosticCode::InvalidValue { text: None },
        Severity::Error,
    ));
    diagnostics.push(finding(
        DiagnosticCode::MissingRequiredField { text: None },
        Severity::Error,
    ));
    diagnostics
}

#[test]
fn first_error_skips_a_kept_error() {
    let diagnostics = kept_then_rejecting();
    let first = diagnostics.first_error().expect("a rejecting error");
    assert!(first.rejects(), "the first error is the first that rejects");
    assert!(matches!(
        first.code,
        DiagnosticCode::MissingRequiredField { .. }
    ));
}

#[test]
fn summary_names_a_rejecting_error_before_a_kept_one() {
    let diagnostics = kept_then_rejecting();
    let rejecting = diagnostics
        .iter()
        .find(|d| d.rejects())
        .expect("a rejecting error")
        .to_string();
    assert_eq!(diagnostics.summary().to_string(), rejecting);
}

#[test]
fn summary_of_only_kept_errors_names_the_first_of_them() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.push_kept_error(finding(
        DiagnosticCode::InvalidValue { text: None },
        Severity::Error,
    ));
    let kept = diagnostics
        .iter()
        .nth(1)
        .expect("the kept error")
        .to_string();
    assert_eq!(diagnostics.summary().to_string(), kept);
}

#[test]
fn first_error_of_only_kept_errors_is_none() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push_kept_error(finding(
        DiagnosticCode::InvalidValue { text: None },
        Severity::Error,
    ));
    assert!(!diagnostics.has_errors());
    assert!(diagnostics.first_error().is_none());
    // The kept error stays visible at its severity.
    assert_eq!(diagnostics.at_least(Severity::Error).count(), 1);
}

#[test]
fn summary_with_no_findings() {
    assert_eq!(Diagnostics::default().summary().to_string(), "no findings");
}

#[test]
fn summary_names_the_first_error() {
    let diagnostics = mixed_findings();
    let first = diagnostics.first_error().expect("two errors").to_string();
    assert_eq!(diagnostics.summary().to_string(), first);
}

#[test]
fn summary_without_errors_names_the_most_severe() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(DiagnosticCode::NoSplashes, Severity::Info));
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.push(finding(DiagnosticCode::PersonaPiecePack, Severity::Warning));
    let warning = finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    )
    .to_string();
    assert_eq!(diagnostics.summary().to_string(), warning);
}

#[test]
fn display_has_the_content_log_shape() {
    let mut diagnostic = finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    );
    diagnostic.location.json_path = friction_path();
    let message = DiagnosticCode::UnknownSound { text: None }.to_string();
    assert_eq!(
        diagnostic.to_string(),
        format!("[warning] minecraft:block/components/minecraft:friction | {message}")
    );
}

#[test]
fn display_without_location_is_severity_and_message() {
    let diagnostic = finding(
        DiagnosticCode::MissingRequiredField { text: None },
        Severity::Error,
    );
    let message = DiagnosticCode::MissingRequiredField { text: None }.to_string();
    assert_eq!(diagnostic.to_string(), format!("[error] {message}"));
}

#[test]
fn display_names_each_severity_in_lower_case() {
    for (severity, word) in [
        (Severity::Debug, "[debug] "),
        (Severity::Info, "[info] "),
        (Severity::Warning, "[warning] "),
        (Severity::Error, "[error] "),
    ] {
        let text = finding(DiagnosticCode::NoSplashes, severity).to_string();
        assert!(text.starts_with(word), "{text}");
    }
}

#[test]
fn display_appends_the_reference_detail() {
    let mut diagnostic = finding(
        DiagnosticCode::UnresolvedReference { text: None },
        Severity::Warning,
    );
    diagnostic.detail = Some(Box::new(DiagnosticDetail::Reference {
        kind: "geometry",
        id: "geometry.Ruby".into(),
        hint: Some(ReferenceHint::CaseMismatch("geometry.ruby".into())),
    }));
    let message = DiagnosticCode::UnresolvedReference { text: None }.to_string();
    let text = diagnostic.to_string();
    assert!(
        text.starts_with(&format!("[warning] {message}: geometry 'geometry.Ruby'")),
        "{text}"
    );
    assert!(text.contains("geometry.ruby"), "{text}");
}

/// The tried paths of a file lookup are a list, and print as one joined reference.
#[test]
fn display_appends_the_tried_paths_detail() {
    let code = DiagnosticCode::FontFileMissing { text: None };
    let mut diagnostic = finding(code.clone(), Severity::Warning);
    let detail = DiagnosticDetail::TriedPaths {
        kind: "font file",
        paths: vec!["font/a.ttf".into(), "font/a.otf".into()],
    };
    assert_eq!(detail.to_string(), ": font file 'font/a.ttf, font/a.otf'");
    diagnostic.detail = Some(Box::new(detail));
    assert_eq!(
        diagnostic.to_string(),
        format!("[warning] {code}: font file 'font/a.ttf, font/a.otf'")
    );
}

#[test]
fn json_path_push_pop_and_display() {
    let mut path = JsonPath::new();
    assert!(path.is_empty());
    assert_eq!(path.to_string(), "");
    path.push("minecraft:loot_table");
    path.push("pools");
    path.push_index(3);
    path.push("entries");
    assert_eq!(path.len(), 4);
    assert_eq!(path.to_string(), "minecraft:loot_table/pools/3/entries");
    assert!(path.pop());
    assert_eq!(path.to_string(), "minecraft:loot_table/pools/3");
    assert!(path.pop());
    assert!(path.pop());
    assert!(path.pop());
    assert!(path.is_empty());
    assert!(!path.pop(), "popping an empty path reports nothing popped");
    assert_eq!(path, JsonPath::default());
}

#[test]
fn json_path_append_continues_with_keys_and_indices_unchanged() {
    let mut base = JsonPath::new();
    base.push("minecraft:block");
    let mut relative = JsonPath::new();
    relative.push("components");
    relative.push_index(2);
    relative.push("3");
    base.append(&relative);
    let mut expected = JsonPath::new();
    expected.push("minecraft:block");
    expected.push("components");
    expected.push_index(2);
    expected.push("3");
    assert_eq!(base, expected);
    base.append(&JsonPath::new());
    assert_eq!(base, expected, "an empty path appends nothing");
}

#[test]
fn json_path_of_the_17_example() {
    assert_eq!(
        friction_path().to_string(),
        "minecraft:block/components/minecraft:friction"
    );
}

#[test]
fn span_line_col_on_lf_text() {
    let text = "{\n  \"a\": 1,\n  \"b\": 2\n}";
    let b = u32::try_from(text.find("\"b\"").unwrap()).unwrap();
    assert_eq!(Span { offset: 0, len: 1 }.line_col(text), (1, 1));
    assert_eq!(Span { offset: b, len: 3 }.line_col(text), (3, 3));
}

#[test]
fn span_line_col_on_crlf_text_matches_lf() {
    let lf = "{\n  \"a\": 1,\n  \"b\": 2\n}";
    let crlf = lf.replace('\n', "\r\n");
    let b_lf = u32::try_from(lf.find("\"b\"").unwrap()).unwrap();
    let b_crlf = u32::try_from(crlf.find("\"b\"").unwrap()).unwrap();
    assert_eq!(
        Span {
            offset: b_crlf,
            len: 3
        }
        .line_col(&crlf),
        (3, 3)
    );
    assert_eq!(
        Span {
            offset: b_crlf,
            len: 3
        }
        .line_col(&crlf),
        Span {
            offset: b_lf,
            len: 3
        }
        .line_col(lf)
    );
    let close = u32::try_from(crlf.len() - 1).unwrap();
    assert_eq!(
        Span {
            offset: close,
            len: 1
        }
        .line_col(&crlf),
        (4, 1)
    );
}

#[test]
fn span_line_col_counts_characters_not_bytes() {
    let text = "\"ä\": x";
    let x = u32::try_from(text.find('x').unwrap()).unwrap();
    assert_eq!(Span { offset: x, len: 1 }.line_col(text), (1, 6));
}

#[test]
fn span_line_col_past_the_end_clamps_to_the_end() {
    assert_eq!(Span { offset: 99, len: 0 }.line_col("ab\nc"), (2, 2));
}

#[test]
fn pack_report_has_vanillas_field_set() {
    let report = report_at_memory_root();
    assert!(report.diagnostics.is_empty());
    assert!(!report.was_upgraded);
    assert!(!report.attempted_upgrade);
    assert_eq!(report.location.root, RootId::MEMORY);
    assert_eq!(report.required_base_game_version, None);
    assert_eq!(report.original_name, "");
    assert_eq!(report.original_version, "");
    assert_eq!(report.identity, None);
    assert_eq!(report.pack_type, PackType::Behavior);
    assert_eq!(report.pack_origin, PackOrigin::World);
}

#[test]
fn pack_report_summary_with_no_findings() {
    let report = report_at_memory_root();
    assert!(!report.has_errors());
    assert!(report.diagnostics().is_empty());
    assert_eq!(report.summary().to_string(), "no findings");
}

#[test]
fn pack_report_summary_names_the_first_error() {
    let mut report = report_at_memory_root();
    report.diagnostics = mixed_findings();
    assert!(report.has_errors());
    assert_eq!(
        report.summary().to_string(),
        report.diagnostics().summary().to_string()
    );
    let first = report
        .diagnostics()
        .first_error()
        .expect("errors")
        .to_string();
    assert_eq!(report.summary().to_string(), first);
}

/// The file segment sits between the pack and the JSON path.
#[test]
fn display_names_the_file_before_the_json_path() {
    let mut diagnostic = finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    );
    diagnostic.location.path = Some(PackPath::from_static("blocks/ruby.json"));
    diagnostic.location.json_path = friction_path();
    let message = DiagnosticCode::UnknownSound { text: None }.to_string();
    assert_eq!(
        diagnostic.to_string(),
        format!(
            "[warning] blocks/ruby.json | minecraft:block/components/minecraft:friction | {message}"
        )
    );
}

/// A pack on a stack, as the stack builders hand it out.
fn ruby_pack() -> PackRef {
    PackRef::new(
        0,
        PackIdVersion::new(
            Uuid::from_u128(0x5eed),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        "Ruby Pack",
    )
}

/// The pack's display name is the first segment of the content-log line.
#[test]
fn display_names_the_pack_first() {
    let pack = ruby_pack();
    let name = pack.name().to_owned();
    let mut diagnostic = finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    );
    diagnostic.location.pack = Some(pack);
    diagnostic.location.path = Some(PackPath::from_static("blocks/ruby.json"));
    diagnostic.location.json_path = friction_path();
    let message = DiagnosticCode::UnknownSound { text: None }.to_string();
    assert_eq!(
        diagnostic.to_string(),
        format!(
            "[warning] {name} | blocks/ruby.json | minecraft:block/components/minecraft:friction | {message}"
        )
    );
}

// A path's segments are readable without parsing its `Display` text.
#[test]
fn a_json_path_lists_its_segments() {
    use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment};
    let mut path = JsonPath::new();
    path.push("minecraft:camera_preset");
    path.push("a/b");
    path.push_index(3);
    path.push("");
    let segments: Vec<JsonPathSegment<'_>> = path.segments().collect();
    assert_eq!(
        segments,
        [
            JsonPathSegment::Key("minecraft:camera_preset"),
            JsonPathSegment::Key("a/b"),
            JsonPathSegment::Index(3),
            JsonPathSegment::Key(""),
        ]
    );
    // `Display` keeps the key `a/b` one segment; the empty key is an empty segment.
    assert_eq!(path.to_string(), "minecraft:camera_preset/a~1b/3/");
    assert_eq!(JsonPath::new().segments().count(), 0);
}

/// A key that holds `/` is one segment in the text too. `Display` writes a key as a JSON
/// pointer (RFC 6901) token: `~` as `~0`, `/` as `~1`; an index and every other key as before.
#[test]
fn json_path_display_escapes_a_slash_and_a_tilde_in_a_key() {
    use bedrock_addon_core::diagnostic::JsonPath;
    let of = |keys: &[&str]| {
        let mut path = JsonPath::new();
        for key in keys {
            path.push(key);
        }
        path
    };
    assert_eq!(of(&["textures", "a/b"]).to_string(), "textures/a~1b");
    assert_eq!(of(&["c~d"]).to_string(), "c~0d");
    // `~` first, so a key that spells an escape is not read back as a slash.
    assert_eq!(of(&["~1"]).to_string(), "~01");
    assert_eq!(of(&["a/b~/"]).to_string(), "a~1b~0~1");
    // The two paths that printed alike print apart.
    assert_ne!(of(&["a/b"]).to_string(), of(&["a", "b"]).to_string());
    assert_eq!(of(&["a", "b"]).to_string(), "a/b");
    // A namespaced key, a dotted key and an index are written as they are.
    let mut path = of(&["minecraft:block", "geometry.x", "q.is_a('b')"]);
    path.push_index(10);
    assert_eq!(
        path.to_string(),
        "minecraft:block/geometry.x/q.is_a('b')/10"
    );
    // The text splits back into the segments: split at `/`, then `~1` to `/`, then `~0` to `~`.
    let path = of(&["a/b~/", "~1", "plain"]);
    let read_back: Vec<String> = path
        .to_string()
        .split('/')
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect();
    assert_eq!(read_back, ["a/b~/", "~1", "plain"]);
}

/// A path of `segments` steps, a third of them keys the text escapes and a third indices, and
/// the same path once more: what a comparison is given.
fn long_path(segments: usize) -> (JsonPath, JsonPath) {
    let mut path = JsonPath::new();
    for step in 0..segments {
        match step % 3 {
            0 => path.push("minecraft:member"),
            1 => path.push("a/b~c"),
            _ => path.push_index(step),
        }
    }
    (path.clone(), path)
}

/// The text of a path and what a path answers for its segments cost the
/// segments, each once: `Display` writes each key to the formatter in the pieces between its
/// `~` and `/` (a key without either in one piece), and a comparison walks the two lists of
/// segments. By the crate's one protocol: four times the segments in under eight times the
/// time.
#[test]
fn the_text_and_the_comparisons_of_a_json_path_cost_its_segments() {
    use std::fmt::Write as _;
    let display = |(path, _): &(JsonPath, JsonPath)| {
        let mut text = String::new();
        write!(text, "{path}").unwrap();
        assert!(text.len() > 9 * path.len());
    };
    let compare = |(path, same): &(JsonPath, JsonPath)| {
        for _ in 0..40 {
            assert!(path == same && path.starts_with(same) && path.ends_with(same));
            assert_eq!(path.index_after("no such key"), None);
            assert!(!path.is_key("minecraft:member"));
        }
    };
    cost::assert_each_linear_at(
        120_000,
        long_path,
        &[
            ("the text of a path, segments", &display),
            ("the comparisons of a path, segments", &compare),
        ],
    );
}

/// A key without `~` and `/` is written as it is, in one piece and without a
/// search of its characters one by one: the text of 2000 paths of such keys (the paths of
/// nearly every finding) against the same keys and separators pushed to a `String` by hand.
/// Two times compared under the lock of the cost tests, not a bound on growth: each the best
/// of seven taken in turn so that both see the same neighbours, required to stay under eight
/// times the hand-written text (a search for the two characters a character at a time
/// costs about 24 times, looking through the key as bytes first about 3), and a time over the
/// bound is measured once more before it is believed.
#[test]
fn the_text_of_a_path_of_plain_keys_costs_little_more_than_its_keys() {
    use std::fmt::Write as _;
    const KEYS: [&str; 4] = [
        "minecraft:entity",
        "components",
        "minecraft:behavior.eat_block",
        "success_chance",
    ];
    let paths: Vec<JsonPath> = (0..2000).map(|_| JsonPath::of_keys(KEYS)).collect();
    let displayed = |text: &mut String| {
        for path in &paths {
            write!(text, "{path}").unwrap();
        }
    };
    let by_hand = |text: &mut String| {
        for _ in &paths {
            for (at, key) in KEYS.iter().enumerate() {
                if at > 0 {
                    text.push('/');
                }
                text.push_str(key);
            }
        }
    };
    // Twenty writings of the 2000 texts, timed; the text is the last writing's.
    let timed = |write: &dyn Fn(&mut String), text: &mut String| {
        cost::time_of(|| {
            for _ in 0..20 {
                text.clear();
                write(text);
            }
        })
    };
    let (mut text, mut expected) = (String::new(), String::new());
    let mut ratios = Vec::new();
    for _attempt in 0..2 {
        let (display, hand) = cost::alone_for(|| {
            let mut best = (std::time::Duration::MAX, std::time::Duration::MAX);
            for _ in 0..7 {
                best.0 = best.0.min(timed(&displayed, &mut text));
                best.1 = best.1.min(timed(&by_hand, &mut expected));
            }
            best
        });
        assert_eq!(text, expected);
        let ratio = display.as_secs_f64() / hand.as_secs_f64().max(1e-9);
        eprintln!(
            "cost: the text of 2000 paths {display:?}, their keys by hand {hand:?}, ratio {ratio:.2}"
        );
        if ratio < 8.0 {
            return;
        }
        ratios.push(ratio);
    }
    panic!("the text of the paths took {ratios:?} times the keys pushed by hand, twice");
}

/// What code asks of a path it asks of its segments, never of its text.
#[test]
fn a_json_path_answers_for_its_keys_prefixes_and_steps() {
    use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment};
    let path = JsonPath::of_keys(["minecraft:block", "a/b", "c~d"]);
    assert_eq!(
        path.segments().collect::<Vec<_>>(),
        [
            JsonPathSegment::Key("minecraft:block"),
            JsonPathSegment::Key("a/b"),
            JsonPathSegment::Key("c~d"),
        ]
    );
    assert_eq!(JsonPath::of_keys(Vec::<String>::new()), JsonPath::new());
    // The last key is the key as written, not its escaped text.
    assert_eq!(path.last_key(), Some("c~d"));
    assert_eq!(JsonPath::of_keys(["a/b"]).last_key(), Some("a/b"));
    assert_eq!(JsonPath::new().last_key(), None);
    let mut element = JsonPath::of_keys(["list"]);
    element.push_index(2);
    assert_eq!(element.last_key(), None);
    // One member of the root.
    assert!(JsonPath::of_keys(["a/b"]).is_key("a/b"));
    assert!(!JsonPath::of_keys(["a", "b"]).is_key("a/b"));
    assert!(!JsonPath::of_keys(["a/b"]).is_key("a~1b"));
    assert!(!JsonPath::new().is_key(""));
    assert!(JsonPath::of_keys([""]).is_key(""));
    let mut zero = JsonPath::new();
    zero.push_index(0);
    assert!(!zero.is_key("0"));
    assert!(JsonPath::of_keys(["0"]).is_key("0"));
    // Prefixes and suffixes go by whole segments.
    let block = JsonPath::of_keys(["minecraft:block"]);
    assert!(path.starts_with(&block));
    assert!(path.starts_with(&path));
    assert!(path.starts_with(&JsonPath::new()));
    assert!(!path.starts_with(&JsonPath::of_keys(["minecraft:blo"])));
    assert!(!block.starts_with(&path));
    assert!(!JsonPath::of_keys(["a/b"]).starts_with(&JsonPath::of_keys(["a"])));
    assert!(path.ends_with(&JsonPath::of_keys(["a/b", "c~d"])));
    assert!(path.ends_with(&JsonPath::new()));
    assert!(!path.ends_with(&JsonPath::of_keys(["b", "c~d"])));
    assert!(!JsonPath::of_keys(["x/sound"]).ends_with(&JsonPath::of_keys(["sound"])));
    assert_eq!(
        path.strip_prefix(&block),
        Some(JsonPath::of_keys(["a/b", "c~d"]))
    );
    assert_eq!(path.strip_prefix(&path), Some(JsonPath::new()));
    assert_eq!(path.strip_prefix(&JsonPath::of_keys(["other"])), None);
    // The index after a member.
    let mut condition = JsonPath::of_keys(["minecraft:block", "permutations"]);
    condition.push_index(3);
    condition.push("condition");
    assert_eq!(condition.index_after("permutations"), Some(3));
    assert_eq!(condition.index_after("condition"), None);
    assert_eq!(condition.index_after("components"), None);
    assert_eq!(
        JsonPath::of_keys(["permutations", "3"]).index_after("permutations"),
        None,
        "a key of digits is no index"
    );
}

/// The two things the text of a path cannot tell apart, which its segments can.
#[test]
fn the_text_of_a_json_path_does_not_tell_a_digit_key_from_an_index_nor_one_empty_key_from_the_root()
{
    use bedrock_addon_core::diagnostic::JsonPath;
    let mut index = JsonPath::new();
    index.push_index(0);
    let key = JsonPath::of_keys(["0"]);
    assert_eq!(index.to_string(), key.to_string());
    assert_ne!(index, key);
    let empty_key = JsonPath::of_keys([""]);
    assert_eq!(empty_key.to_string(), JsonPath::new().to_string());
    assert_ne!(empty_key, JsonPath::new());
    assert!(!empty_key.is_empty());
    assert_eq!(JsonPath::of_keys(["a", "", "b"]).to_string(), "a//b");
}

/// A file that starts with a byte-order mark: the mark is no column of its first line, as no
/// editor shows it; spans are offsets into the file as written, the mark counted.
#[test]
fn span_line_col_does_not_count_a_byte_order_mark_as_a_column() {
    let text = "\u{feff}{\"a\": é1,\n \"b\": 2}";
    let at = |needle: &str| Span {
        offset: u32::try_from(text.find(needle).unwrap()).unwrap(),
        len: 1,
    };
    assert_eq!(at("{").line_col(text), (1, 1));
    // Columns are characters: `é` is one.
    assert_eq!(at("1").line_col(text), (1, 8));
    assert_eq!(at("\"b\"").line_col(text), (2, 2));
    // The mark itself, and an offset inside it, are the start of the line.
    assert_eq!(Span { offset: 0, len: 3 }.line_col(text), (1, 1));
    assert_eq!(Span { offset: 2, len: 1 }.line_col(text), (1, 1));
    // Only at the start of the text is it a mark.
    let inner = "ab\n\u{feff}c";
    let c = Span {
        offset: u32::try_from(inner.find('c').unwrap()).unwrap(),
        len: 1,
    };
    assert_eq!(c.line_col(inner), (2, 2));
}

#[test]
fn display_appends_a_note_after_the_message() {
    let mut diagnostic = finding(
        DiagnosticCode::ValueRewritten { text: None },
        Severity::Info,
    );
    let message = DiagnosticCode::ValueRewritten { text: None }.to_string();
    assert_eq!(diagnostic.to_string(), format!("[info] {message}"));
    diagnostic.detail = Some(Box::new(DiagnosticDetail::Note("written as `x`")));
    assert_eq!(
        diagnostic.to_string(),
        format!("[info] {message}: written as `x`")
    );
    // A note is the detail's whole text, and two notes compare by their words.
    assert_eq!(DiagnosticDetail::Note("a").to_string(), ": a");
    assert_ne!(DiagnosticDetail::Note("a"), DiagnosticDetail::Note("b"));
}

/// The file a located list is put in.
fn fog_origin() -> Origin {
    Origin::file(
        PackRef::app_data(),
        PackSide::Resource,
        PackPath::from_static("fogs/a.json"),
    )
}

/// Two findings named where the file has them, after an upgrade moved `old` to `new`: the
/// list knows that the member the document holds at `new` is at `old` in the file.
fn moved_findings() -> (Diagnostics, JsonPath, JsonPath) {
    let (from, to) = (JsonPath::of_keys(["old"]), JsonPath::of_keys(["new"]));
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding(
        DiagnosticCode::UnknownSound { text: None },
        Severity::Warning,
    ));
    diagnostics.moved(from.clone(), to.clone());
    diagnostics.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(to.clone()));
    diagnostics.name_as_the_file_has_it();
    assert_eq!(diagnostics.place_of(&to), from);
    (diagnostics, from, to)
}

#[test]
fn locate_puts_every_finding_in_the_file_of_the_origin() {
    let (mut diagnostics, from, _) = moved_findings();
    let origin = fog_origin();
    diagnostics.locate(&origin);
    assert_eq!(diagnostics.len(), 2);
    for found in &diagnostics {
        assert_eq!(found.location.pack, Some(origin.pack.clone()));
        assert_eq!(found.location.path, Some(origin.path.clone()));
    }
    assert_eq!(diagnostics.iter().nth(1).unwrap().location.json_path, from);
    assert_eq!(diagnostics.clone().located(&origin), diagnostics);
}

/// In place, the list keeps the places of the moves it has undone; the families'
/// take-and-repush copy (`std::mem::take`, then a push of each finding) drops them, and the
/// link pass then sees the member where the document holds it.
#[test]
fn locate_in_place_keeps_the_places_take_and_repush_drops() {
    let origin = fog_origin();
    let (mut in_place, from, to) = moved_findings();
    in_place.locate(&origin);
    assert_eq!(in_place.place_of(&to), from);
    assert_eq!(moved_findings().0.located(&origin).place_of(&to), from);

    let (mut repushed, _, _) = moved_findings();
    let found = std::mem::take(&mut repushed);
    for mut f in found {
        f.location.pack = Some(origin.pack.clone());
        f.location.path = Some(origin.path.clone());
        repushed.push(f);
    }
    assert_eq!(repushed, in_place, "the findings are the same either way");
    assert_eq!(repushed.place_of(&to), to, "the places are gone");
}
