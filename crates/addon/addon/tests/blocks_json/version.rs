//! Rows bj03 to bj05: what the client makes of
//! `format_version`. Each file has the keys `stone` (gate 1.20.50) and `wool` (gate 1.21.30),
//! so the effective version shows in how they are read.

use bedrock_addon::document::Content;
use bedrock_addon::families::blocks_json::{
    BLOCKS_JSON_DEFAULT_VERSION, BLOCKS_JSON_MAX_FORMAT_VERSION, BlocksJson,
};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;
use std::fmt::Write as _;

use super::common::{E, Found, I, W, file_at, load_one, ours, six, vanilla};

const NOT_GIVEN: &str = "Warning: The blocks.json \"format_version\" field was not given. \"format_version\" 1.1.0 will be used.";
const INVALID: &str = "Warning: The blocks.json \"format_version\" field is invalid. \"format_version\" 1.1.0 will be used.";

/// V2 at `format_version`.
fn invalid() -> Found {
    vanilla("MalformedFormatVersion", W, "format_version", INVALID)
}

/// V3 for the version text `written`.
fn above(written: &str) -> Found {
    vanilla(
        "UnsupportedVersion",
        W,
        "format_version",
        &format!(
            "Warning: The blocks.json \"format_version\": {written} exceeds largest currently supported version: 1.21.40.\"format_version\" 1.1.0 will be used.\n"
        ),
    )
}

/// L1 at the legacy key `key`.
fn split_warning(key: &str) -> Found {
    ours("DeprecatedMember", I, key)
}

/// V6 for `wool`.
fn wool_is_an_old_name() -> Found {
    let names: String = [
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ]
    .iter()
    .fold(String::new(), |mut out, colour| {
        write!(out, "\n - minecraft:{colour}_wool").unwrap();
        out
    });
    vanilla(
        "DeprecatedMember",
        E,
        "wool",
        &format!(
            "blocks.json file contains a block using an old name that was used for multiple blocks: minecraft:wool. \nPlease use their new names instead: {names}"
        ),
    )
}

const STONE_AND_WOOL: &str = r#""stone": {"textures": "s"}, "wool": {"textures": "w"}"#;

/// How a file with [`STONE_AND_WOOL`] at `version` read its two keys.
#[derive(Debug, PartialEq, Eq)]
enum Read {
    /// Split onto the post-split blocks with `name:<variant>`.
    Split,
    /// Applied to the block of that name.
    Direct,
    /// Skipped.
    Skipped,
}

/// `(stone, wool)` of a file at the JSON `version`, and its findings at warning level or above.
fn read(version: Option<&str>) -> ((Read, Read), Vec<Found>) {
    let text = match version {
        Some(version) => file_at(version, STONE_AND_WOOL),
        None => format!("{{{STONE_AND_WOOL}}}"),
    };
    let loaded = load_one(&text);
    let stone = match (loaded.get("stone"), loaded.get("granite")) {
        (Some(stone), Some(granite)) => {
            assert_eq!(stone.face_textures(), ["s:0"; 6]);
            assert_eq!(granite.face_textures(), ["s:1"; 6]);
            assert_eq!(loaded.textures("polished_andesite"), six("s:6"));
            Read::Split
        }
        (Some(stone), None) => {
            assert_eq!(stone.face_textures(), ["s"; 6]);
            Read::Direct
        }
        (None, _) => Read::Skipped,
    };
    let wool = match (loaded.get("white_wool"), loaded.get("wool")) {
        (Some(white), None) => {
            assert_eq!(white.face_textures(), ["w:0"; 6]);
            assert_eq!(loaded.textures("black_wool"), six("w:15"));
            Read::Split
        }
        (None, Some(_)) => Read::Direct,
        (None, None) => Read::Skipped,
        (Some(_), Some(_)) => panic!("both"),
    };
    ((stone, wool), loaded.found())
}

#[test]
fn bj03_a_missing_version_is_1_1_0_with_vanilla_s_warning() {
    let loaded = load_one(r#"{"dirt": {"textures": "x"}}"#);
    // `dirt` is below its 1.21.20 gate: split onto `dirt` and `coarse_dirt`.
    assert_eq!(loaded.textures("dirt"), six("x:0"));
    assert_eq!(loaded.textures("coarse_dirt"), six("x:1"));
    assert_eq!(loaded.registry().len(), 2);
    assert_eq!(
        loaded.found(),
        [
            vanilla("MissingFormatVersion", W, "", NOT_GIVEN),
            split_warning("dirt")
        ]
    );
    let version = loaded.block("dirt").version();
    assert_eq!(version.written, None);
    assert_eq!(version.effective, FormatVersion::new(1, 1, 0));
    // The same for the two test keys.
    let (keys, found) = read(None);
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(found[0].code, "MissingFormatVersion");
}

#[test]
fn bj04_a_version_that_is_not_one_is_1_1_0() {
    for version in [
        "7",
        r#""1.21.40.1""#,
        "[1, 21]",
        "true",
        "{}",
        r#""1.21.40 ""#,
        "[1, 21, 40, 0]",
        "[1, -1, 0]",
        "[1, 65536, 0]",
        r#"["1", "21", "40"]"#,
    ] {
        let (keys, found) = read(Some(version));
        assert_eq!(keys, (Read::Split, Read::Split), "{version}");
        assert_eq!(
            found,
            [invalid(), split_warning("stone"), split_warning("wool")],
            "{version}"
        );
    }
}

#[test]
fn bj04_the_any_version_star_is_0_0_0_and_gates_like_1_1_0() {
    let (keys, found) = read(Some(r#""*""#));
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(
        found,
        [invalid(), split_warning("stone"), split_warning("wool")]
    );
    let loaded = load_one(&file_at(r#""*""#, STONE_AND_WOOL));
    assert_eq!(
        loaded.block("stone").version().effective,
        FormatVersion::new(0, 0, 0)
    );
}

#[test]
fn bj04_a_partial_version_is_kept_and_skips_the_ceiling_deviation() {
    // `vanilla = "misleading V2"`: the client says 1.1.0 and reads 1.21.0. L15 says so.
    let partial = ours("UnusualFormatVersion", I, "format_version");
    let (keys, found) = read(Some(r#""1.21""#));
    assert_eq!(
        keys,
        (Read::Direct, Read::Split),
        "1.21.0: only stone's gate is passed"
    );
    assert_eq!(found, [invalid(), partial.clone(), split_warning("wool")]);
    let loaded = load_one(&file_at(r#""1.21""#, STONE_AND_WOOL));
    assert_eq!(
        loaded.block("stone").version().effective,
        FormatVersion::new(1, 21, 0)
    );
    assert_eq!(
        loaded.block("stone").version().written,
        None,
        "not a whole version"
    );
    // `"2"` is 2.0.0: above the ceiling, yet no V3 and no fallback, so `wool` is an old name.
    let (keys, found) = read(Some(r#""2""#));
    assert_eq!(keys, (Read::Direct, Read::Skipped));
    assert_eq!(found, [invalid(), partial.clone(), wool_is_an_old_name()]);
    // A partial version below every gate still splits, with L1.
    let (keys, found) = read(Some(r#""1""#));
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(
        found,
        [
            invalid(),
            partial,
            split_warning("stone"),
            split_warning("wool")
        ]
    );
}

#[test]
fn bj04_the_beta_literal_is_above_the_ceiling() {
    let (keys, found) = read(Some(r#""beta""#));
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(
        found,
        [
            above("9999.9999.9999-beta"),
            split_warning("stone"),
            split_warning("wool")
        ]
    );
}

#[test]
fn bj05_a_version_above_1_21_40_is_read_as_1_1_0_not_as_1_21_40() {
    let loaded = load_one(&file_at(
        r#""1.26.0""#,
        r#""stone": {"textures": "s"}, "granite": {"textures": "g"}"#,
    ));
    // `granite` resolves directly; `stone`, after it in key order, is split (L1), and the
    // split's `s:1` is applied to `granite` last.
    assert_eq!(loaded.textures("stone"), six("s:0"));
    assert_eq!(loaded.textures("granite"), six("s:1"));
    assert_eq!(loaded.found(), [above("1.26.0"), split_warning("stone")]);
    let version = loaded.block("stone").version();
    assert_eq!(version.written, Some(FormatVersion::new(1, 26, 0)));
    assert_eq!(version.effective, FormatVersion::new(1, 1, 0));
    // Without a legacy key the warning is all there is.
    let loaded = load_one(&file_at(r#""1.26.0""#, r#""granite": {"textures": "g"}"#));
    assert_eq!(loaded.textures("granite"), six("g"));
    assert_eq!(loaded.found(), [above("1.26.0")]);
}

#[test]
fn the_ceiling_is_exactly_1_21_40() {
    assert_eq!(
        BLOCKS_JSON_MAX_FORMAT_VERSION,
        FormatVersion::new(1, 21, 40)
    );
    assert_eq!(BLOCKS_JSON_DEFAULT_VERSION, FormatVersion::new(1, 1, 0));
    // At the ceiling: no finding, and every legacy name is an old name.
    let (keys, found) = read(Some(r#""1.21.40""#));
    assert_eq!(keys, (Read::Direct, Read::Skipped));
    assert_eq!(found, [wool_is_an_old_name()]);
    // One patch above: V3 and the 1.1.0 split.
    let (keys, found) = read(Some(r#""1.21.41""#));
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(found[0], above("1.21.41"));
    // A pre-release of the ceiling is below it: no V3, read as written.
    let (keys, found) = read(Some(r#""1.21.40-beta.1""#));
    assert_eq!(keys, (Read::Direct, Read::Skipped));
    assert_eq!(found, [wool_is_an_old_name()]);
    // A pre-release above it is above it, printed with its tag.
    let (_, found) = read(Some(r#""1.21.41-rc.2+build""#));
    assert_eq!(found[0], above("1.21.41-rc.2"));
    // The array form is a whole version too.
    let (keys, found) = read(Some("[1, 21, 40]"));
    assert_eq!(keys, (Read::Direct, Read::Skipped));
    assert_eq!(found.len(), 1);
    let (_, found) = read(Some("[1, 21, 41]"));
    assert_eq!(found[0], above("1.21.41"));
    let (keys, found) = read(Some("[1, 1, 0]"));
    assert_eq!(keys, (Read::Split, Read::Split));
    assert_eq!(found, [], "vanilla's own old files: no warning, no L1");
}

#[test]
fn an_array_version_reads_its_elements_as_jsoncpp_does() {
    // `isConvertibleTo(uintValue)`: a real in range is truncated, a boolean is 0 or 1, `null` 0.
    for (array, effective) in [
        ("[1.9, 21, 40]", FormatVersion::new(1, 21, 40)),
        ("[true, false, null]", FormatVersion::new(1, 0, 0)),
        ("[0, 0, 65535]", FormatVersion::new(0, 0, 65535)),
        // 65536 does not fit: not a version.
        ("[0, 0, 65536]", FormatVersion::new(1, 1, 0)),
        ("[0, -0.5, 0]", FormatVersion::new(1, 1, 0)),
        (r#"[1, "21", 40]"#, FormatVersion::new(1, 1, 0)),
    ] {
        let file = BlocksJson {
            format_version: Some(facet_json::from_str::<Value>(array).unwrap()),
            ..BlocksJson::default()
        };
        assert_eq!(file.effective_version(), effective, "{array}");
    }
}

#[test]
fn the_file_type_tells_its_effective_version() {
    let version = |json: &str| {
        BlocksJson {
            format_version: Some(facet_json::from_str::<Value>(json).unwrap()),
            ..BlocksJson::default()
        }
        .effective_version()
    };
    assert_eq!(
        BlocksJson {
            format_version: None,
            ..BlocksJson::default()
        }
        .effective_version(),
        FormatVersion::new(1, 1, 0)
    );
    assert_eq!(version(r#""1.21.40""#), FormatVersion::new(1, 21, 40));
    assert_eq!(version(r#""1.20.50""#), FormatVersion::new(1, 20, 50));
    assert_eq!(version(r#""1.26.0""#), FormatVersion::new(1, 1, 0));
    assert_eq!(version(r#""1.21""#), FormatVersion::new(1, 21, 0));
    assert_eq!(version(r#""2""#), FormatVersion::new(2, 0, 0));
    assert_eq!(version(r#""*""#), FormatVersion::new(0, 0, 0));
    assert_eq!(version(r#""beta""#), FormatVersion::new(1, 1, 0));
    assert_eq!(version("7"), FormatVersion::new(1, 1, 0));
    // A new file carries the one version at which no legacy name is split.
    assert_eq!(
        BlocksJson::new().effective_version(),
        FormatVersion::new(1, 21, 40)
    );
    assert_eq!(BlocksJson::FORMAT_VERSION, FormatVersion::new(1, 21, 40));
}

#[test]
fn one_file_parses_with_its_findings() {
    let text = file_at(
        "7",
        r#""wool": {"textures": "w", "brightness_gamma": 1}, "x": 3"#,
    );
    let document = BlocksJson::parse(
        &PackPath::from_static("blocks.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .unwrap();
    assert_eq!(document.entries.len(), 2);
    assert_eq!(
        document.entries.keys().collect::<Vec<_>>(),
        ["wool", "x"],
        "file order, `format_version` apart"
    );
    assert_eq!(document.version().effective, FormatVersion::new(1, 1, 0));
    let codes: Vec<&str> = document
        .diagnostics()
        .iter()
        .map(|f| f.code.name())
        .collect();
    assert_eq!(
        codes,
        [
            "MalformedFormatVersion",
            "IgnoredMember",
            "DeprecatedMember",
            "InvalidType"
        ]
    );
    assert!(
        document
            .diagnostics()
            .iter()
            .all(|f| f.location.path.as_ref().map(PackPath::as_str) == Some("blocks.json"))
    );
    assert_eq!(BlocksJson::NAME, "blocks.json");
}

#[test]
fn the_default_file_is_the_new_file() {
    assert_eq!(BlocksJson::default(), BlocksJson::new());
    assert_eq!(
        BlocksJson::default().effective_version(),
        BLOCKS_JSON_MAX_FORMAT_VERSION
    );
}
