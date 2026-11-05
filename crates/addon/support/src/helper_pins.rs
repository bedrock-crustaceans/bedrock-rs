//! Golden values of the support crate's finding and document helpers, each built by hand the
//! long way (`Diagnostic::new`, `Origin::new`, `VersionInfo::new`): the helpers are written over
//! core's builder and `document::unversioned`, and must give exactly these.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

/// A pack (the crate has no `uuid` of its own to name another with).
fn pack() -> PackRef {
    PackRef::app_data()
}

fn path() -> PackPath {
    PackPath::from_static("font/glyph_E0.json")
}

fn at() -> JsonPath {
    JsonPath::of_keys(["a", "b"])
}

fn code() -> DiagnosticCode {
    DiagnosticCode::InvalidValue { text: None }
}

/// The finding of `code` the long way.
fn long(severity: Severity, phase: Phase, location: Location) -> Diagnostic {
    Diagnostic::new(code(), severity, phase, location)
}

fn here(at: JsonPath) -> Location {
    Location::new(None, None, at, None)
}

fn in_file(at: JsonPath) -> Location {
    Location::new(Some(pack()), Some(path()), at, None)
}

/// Every field of two findings, `kept` and the detail included.
fn same(left: &Diagnostic, right: &Diagnostic) {
    assert_eq!(left, right);
    assert_eq!(left.kept, right.kept, "kept");
    assert_eq!(left.detail, right.detail, "detail");
}

fn unversioned_origin(pack: PackRef, side: PackSide) -> Origin {
    Origin::new(pack, side, path(), None, Vec::new())
}

#[test]
fn the_fixed_path_findings_and_paths() {
    use crate::fixed_path::{file_finding, finding, index, key};
    for severity in [Severity::Info, Severity::Warning, Severity::Error] {
        same(
            &finding(code(), severity, at()),
            &long(severity, Phase::Parse, here(at())),
        );
        same(
            &file_finding(code(), severity, &pack(), &path()),
            &long(severity, Phase::Validate, in_file(JsonPath::new())),
        );
    }
    let mut keyed = at();
    keyed.push("c");
    assert_eq!(key(&at(), "c"), keyed);
    let mut indexed = at();
    indexed.push_index(4);
    assert_eq!(index(&at(), 4), indexed);
}

#[test]
fn the_fixed_path_is_int_is_jsoncpp_s_token_rule() {
    use crate::fixed_path::as_int;
    let read =
        |text: &str| as_int(&bedrock_addon_core::json::from_str_jsonc::<Value>(text).unwrap());
    assert_eq!(read("7"), Some(7));
    assert_eq!(read("-2147483648"), Some(i32::MIN));
    assert_eq!(read("2147483648"), None);
    assert_eq!(read("2.0"), None, "a real token is never an int");
    assert_eq!(read("1e2"), None);
    assert_eq!(read("true"), None);
    assert_eq!(read("\"7\""), None);
    assert_eq!(read("null"), None);
}

#[test]
fn the_closing_lines_and_shared_findings() {
    let fixed = |text: &'static str| DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Fixed(text)),
    };
    let mut lines = Diagnostics::default();
    crate::chain_file::closing_line(&mut lines, "closing");
    let lines: Vec<_> = lines.iter().cloned().collect();
    assert_eq!(
        lines,
        [Diagnostic::new(
            fixed("closing"),
            Severity::Error,
            Phase::Parse,
            Location::default()
        )]
    );
    same(
        &crate::spawn::load::closing("closing"),
        &Diagnostic::new(
            fixed("closing"),
            Severity::Error,
            Phase::Validate,
            Location::default(),
        ),
    );
    same(
        &crate::spawn::load::error(code()),
        &long(Severity::Error, Phase::Parse, Location::default()),
    );
    let origin = unversioned_origin(pack(), PackSide::Behavior);
    same(
        &crate::puv::shadowed(&origin),
        &Diagnostic::new(
            DiagnosticCode::ShadowedDefinition { text: None },
            Severity::Info,
            Phase::Validate,
            in_file(JsonPath::new()),
        ),
    );
    for severity in [Severity::Info, Severity::Warning, Severity::Error] {
        same(
            &crate::shape::finding(code(), severity, at()),
            &long(severity, Phase::Parse, here(at())),
        );
        let mut d = Diagnostics::default();
        crate::spawn::read::finding(code(), severity, &at(), &mut d);
        crate::animation::validate::push(&mut d, code(), severity, at());
        let told: Vec<_> = d.iter().cloned().collect();
        assert_eq!(
            told,
            [
                long(severity, Phase::Parse, here(at())),
                long(severity, Phase::Validate, here(at()))
            ]
        );
    }
}

fn json(text: &str) -> Value {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text).unwrap()
}

/// The texture readers' `asInt` and the sound fork's agree but on an integer token past `i64`:
/// its low 32 bits for the texture readers, 0 for the sound fork.
#[test]
fn the_two_as_int_copies_and_their_one_difference() {
    use crate::sound::fork::as_int as fork;
    use crate::texture::jsoncpp::as_int as texture;
    let wide = (u64::MAX - 1).to_string();
    for (text, low_word, zero) in [
        ("7", 7, 7),
        ("-7", -7, -7),
        ("4294967297", 1, 1),
        (wide.as_str(), -2, 0),
        ("2.9", 2, 2),
        ("-2.9", -2, -2),
        ("1e300", i32::MAX, i32::MAX),
        ("-1e300", i32::MIN, i32::MIN),
        ("true", 1, 1),
        ("false", 0, 0),
        ("\"7\"", 0, 0),
        ("[1]", 0, 0),
        ("{}", 0, 0),
        ("null", 5, 5),
    ] {
        let value = json(text);
        assert_eq!(texture(Some(&value), 5), low_word, "texture {text}");
        assert_eq!(fork(&value, 5), zero, "fork {text}");
    }
    assert_eq!(texture(None, 5), 5, "a missing member is the default");
}

/// `sound::merge::located` gives each finding the pack and file and moves it into the list, by
/// push: the place of a move the findings held is not carried over.
#[test]
fn sound_located_pushes_without_the_places() {
    let mut found = Diagnostics::default();
    found.moved(JsonPath::of_keys(["old"]), JsonPath::of_keys(["new"]));
    found.push(long(
        Severity::Warning,
        Phase::Upgrade,
        here(JsonPath::of_keys(["new"])),
    ));
    found.name_as_the_file_has_it();
    assert_eq!(
        found.place_of(&JsonPath::of_keys(["new"])),
        JsonPath::of_keys(["old"])
    );
    let origin = unversioned_origin(pack(), PackSide::Resource);
    let mut into = Diagnostics::default();
    crate::sound::merge::located(found, &origin, &mut into);
    let told: Vec<_> = into.iter().cloned().collect();
    assert_eq!(
        told,
        [long(
            Severity::Warning,
            Phase::Upgrade,
            in_file(JsonPath::of_keys(["old"]))
        )]
    );
    assert_eq!(
        into.place_of(&JsonPath::of_keys(["new"])),
        JsonPath::of_keys(["new"]),
        "the place is not carried over"
    );
}
