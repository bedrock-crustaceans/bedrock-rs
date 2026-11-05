//! Versions as a user of `bedrock_addon_core::version` sees them: `SemVersion` parsing in every
//! form the game accepts and its ordering, `FormatVersion` and what an envelope said about it,
//! `VersionRange` containment under the four range models, and the manifest version types.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{
    BaseGameVersion, ENGINE_VERSION, FormatVersion, FormatVersionInput, MinEngineVersion,
    SemVersion, VersionRange,
};

fn sem(text: &str) -> SemVersion {
    SemVersion::parse(text).unwrap_or_else(|| panic!("{text:?} parses"))
}

fn json(text: &str) -> Value {
    facet_json::from_str(text).unwrap()
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

const fn fv(major: u16, minor: u16, patch: u16) -> FormatVersion {
    FormatVersion::new(major, minor, patch)
}

mod sem_version_parsing {
    use super::*;

    #[test]
    fn a_full_version_string() {
        let v = sem("1.21.60");
        assert_eq!((v.major, v.minor, v.patch), (1, 21, 60));
        assert_eq!(v.pre_release, "");
        assert!(!v.is_any());
        assert_eq!(v, SemVersion::new(1, 21, 60));
    }

    #[test]
    fn a_partial_version_string_fills_zeros() {
        // "1.10" loads as 1.10.0.
        assert_eq!(sem("1.10"), SemVersion::new(1, 10, 0));
        assert_eq!(sem("2"), SemVersion::new(2, 0, 0));
    }

    #[test]
    fn pre_release_and_build_metadata() {
        let v = sem("1.21.60-beta.3+build.7");
        assert_eq!(v.pre_release, "beta.3");
        assert_eq!(v.build, "build.7");
        assert_eq!(v.to_string(), "1.21.60-beta.3+build.7");
    }

    #[test]
    fn the_any_version_and_the_beta_literal() {
        assert!(sem("*").is_any());
        assert_eq!(sem("*").to_string(), "*");
        // The literal "beta" is the full 9999.9999.9999 sentinel, the newest version there is.
        let beta = sem("beta");
        assert_eq!((beta.major, beta.minor, beta.patch), (9999, 9999, 9999));
        assert_eq!(beta.pre_release, "beta");
    }

    #[test]
    fn malformed_strings_are_rejected() {
        for text in [
            "",
            "abc",
            "1.",
            "1..2",
            "1.2.3.4",
            "-1.0.0",
            "1.0.0-",
            "1.0.0+",
            "1.0.0-a..b",
            "1.2.x",
            " 1.2.3",
            "70000.0.0",
        ] {
            assert_eq!(SemVersion::parse(text), None, "{text:?} is not a version");
        }
    }

    /// The game's semantic-version regex: a number is `0` or 1–5 digits
    /// without a leading zero; a numeric pre-release identifier has no leading zero (an
    /// alphanumeric one may start with `0`); the pre-release and the build come only after a
    /// patch. Build identifiers may have leading zeros.
    #[test]
    fn the_engine_regex_rejects_leading_zeros_and_tags_without_a_patch() {
        for text in [
            "01.21.60",
            "1.021.0",
            "1.0.00",
            "00.0.0",
            "1.21.60-01",
            "1.2.3-beta.01",
            "100000.0.0",
            "1.2-beta",
            "1-beta",
            "1.2+build",
        ] {
            assert_eq!(SemVersion::parse(text), None, "{text:?} is not a version");
        }
        for (text, pre, build) in [
            ("1.21.60-0", "0", ""),
            ("1.21.60-0a", "0a", ""),
            ("1.21.60-00a.1", "00a.1", ""),
            ("1.21.60-x-y", "x-y", ""),
            ("1.2.3+001", "", "001"),
            ("0.0.0", "", ""),
            ("65535.0.0", "", ""),
        ] {
            let version = sem(text);
            assert_eq!(
                (version.pre_release.as_str(), version.build.as_str()),
                (pre, build),
                "{text}"
            );
        }
    }

    #[test]
    fn an_array_of_three_numbers() {
        assert_eq!(
            SemVersion::from_json(&json("[1, 20, 0]")),
            Some(SemVersion::new(1, 20, 0))
        );
    }

    #[test]
    fn an_array_truncates_reals() {
        assert_eq!(
            SemVersion::from_json(&json("[1.9, 20.5, 0.0]")),
            Some(SemVersion::new(1, 20, 0))
        );
    }

    #[test]
    fn an_array_element_above_0xffff_is_rejected() {
        assert_eq!(SemVersion::from_json(&json("[1, 65536, 0]")), None);
        assert_eq!(SemVersion::from_json(&json("[1, 65536.5, 0]")), None);
        assert_eq!(
            SemVersion::from_json(&json("[1, 65535.5, 0]")),
            Some(SemVersion::new(1, u16::MAX, 0)),
            "truncated first"
        );
        assert_eq!(
            SemVersion::from_json(&json("[65535, 65535, 65535]")),
            Some(SemVersion::new(u16::MAX, u16::MAX, u16::MAX))
        );
    }

    #[test]
    fn an_array_needs_exactly_three_non_negative_numbers() {
        for text in [
            "[1, 2]",
            "[1, 2, 3, 4]",
            "[1, -2, 3]",
            r#"[1, "2", 3]"#,
            "[1, true, 3]",
            "[1, null, 3]",
            "[-0.5, 0, 0]",
            "[1, -1.0, 0]",
            "[]",
        ] {
            assert_eq!(SemVersion::from_json(&json(text)), None, "{text}");
        }
    }

    #[test]
    fn a_json_string_parses_as_a_string_and_other_values_are_rejected() {
        assert_eq!(
            SemVersion::from_json(&json(r#""1.2.3""#)),
            Some(SemVersion::new(1, 2, 3))
        );
        assert_eq!(SemVersion::from_json(&json("1")), None);
        assert_eq!(SemVersion::from_json(&json("{}")), None);
    }
}

// Reading a version from a JSON array the way the game does, with jsoncpp's conversions of each
// element, beside the numbers-only `from_json`.
mod sem_version_json_array {
    use super::*;

    fn read(text: &str) -> Option<SemVersion> {
        SemVersion::from_json_array(json(text).as_array().unwrap().as_slice())
    }

    #[test]
    fn null_and_booleans_convert_and_reals_truncate() {
        assert_eq!(read("[1, 21, 0]"), Some(SemVersion::new(1, 21, 0)));
        assert_eq!(read("[true, false, true]"), Some(SemVersion::new(1, 0, 1)));
        assert_eq!(read("[null, 1, 0]"), Some(SemVersion::new(0, 1, 0)));
        assert_eq!(read("[1.5, 14, 0.9]"), Some(SemVersion::new(1, 14, 0)));
        assert_eq!(
            read("[65535.9, -0.0, 0]"),
            Some(SemVersion::new(65535, 0, 0))
        );
    }

    #[test]
    fn what_does_not_convert_to_a_u16_is_unreadable() {
        for text in [
            "[1, 2]",
            "[1, 2, 3, 4]",
            "[]",
            "[-1, 0, 0]",
            "[-0.5, 0, 0]",
            "[65536, 0, 0]",
            "[65536.0, 0, 0]",
            r#"["1", 0, 0]"#,
            "[[1], 0, 0]",
            "[{}, 0, 0]",
        ] {
            assert_eq!(read(text), None, "{text}");
        }
    }

    #[test]
    fn from_json_stays_numbers_only() {
        assert_eq!(SemVersion::from_json(&json("[true, 0, 0]")), None);
        assert_eq!(SemVersion::from_json(&json("[null, 1, 0]")), None);
    }
}

mod sem_version_ordering {
    use super::*;

    #[test]
    fn numbers_compare_numerically() {
        assert!(sem("1.9.0") < sem("1.10.0"));
        assert!(sem("1.21.60") < sem("1.21.100"));
        assert!(sem("2.0.0") > sem("1.99.99"));
    }

    #[test]
    fn a_pre_release_sorts_below_its_release() {
        assert!(sem("1.21.60-beta.3") < sem("1.21.60"));
        assert!(sem("1.21.60-beta.3") > sem("1.21.50"));
    }

    #[test]
    fn a_pre_release_list_that_is_a_prefix_sorts_lower() {
        assert!(sem("1.0.0-alpha") < sem("1.0.0-alpha.1"));
        assert!(sem("1.0.0-alpha.1") < sem("1.0.0-alpha.beta"));
        assert!(sem("1.0.0-beta.2") < sem("1.0.0-beta.11"));
    }

    #[test]
    fn build_metadata_takes_no_part() {
        assert_eq!(sem("1.2.3+a"), sem("1.2.3+b"));
        assert_eq!(hash_of(&sem("1.2.3+a")), hash_of(&sem("1.2.3")));
        assert_eq!(sem("1.2.3+a").cmp(&sem("1.2.3")), Ordering::Equal);
    }

    #[test]
    fn the_any_version_is_the_maximum_and_equal_only_to_itself() {
        assert!(SemVersion::ANY > sem("65535.65535.65535"));
        assert!(SemVersion::ANY > sem("beta"));
        assert_eq!(SemVersion::ANY, sem("*"));
        assert_ne!(SemVersion::ANY, SemVersion::new(0, 0, 0));
        assert_eq!(SemVersion::ANY.cmp(&SemVersion::ANY), Ordering::Equal);
    }

    #[test]
    fn the_beta_literal_sorts_above_every_released_engine() {
        assert!(sem("beta") > SemVersion::from(ENGINE_VERSION));
    }

    /// Equal versions hash alike: build metadata takes no part. Leading zeros do not
    /// parse, so `beta.01` and `beta.1` are not a pair.
    #[test]
    fn equal_versions_hash_equally() {
        let pairs = [
            ("1.0.0-beta.1", "1.0.0-beta.1+x"),
            ("1.0.0-rc.7.x", "1.0.0-rc.7.x"),
            ("1.2.3+a", "1.2.3+b"),
            ("*", "*"),
        ];
        for (a, b) in pairs {
            assert_eq!(sem(a), sem(b), "{a} == {b}");
            assert_eq!(hash_of(&sem(a)), hash_of(&sem(b)), "{a} and {b} hash alike");
        }
        assert_ne!(sem("1.0.0-beta.1"), sem("1.0.0-beta.1a"));
    }

    /// `satisfies(other)` holds when the major is equal and the minor is at least `other`'s.
    #[test]
    fn satisfies_needs_the_same_major_and_at_least_the_minor() {
        assert!(sem("1.2.0").satisfies(&sem("1.2.0")));
        assert!(sem("1.3.0").satisfies(&sem("1.2.5")));
        assert!(sem("1.2.0").satisfies(&sem("1.2.9")), "patch takes no part");
        assert!(!sem("1.1.9").satisfies(&sem("1.2.0")));
        assert!(!sem("2.2.0").satisfies(&sem("1.2.0")));
    }
}

mod format_version {
    use super::*;

    #[test]
    /// The engine version is the game's current format version, where every open-ended slice
    /// sequence of a family closes.
    fn the_engine_version_is_the_current_release() {
        assert_eq!(ENGINE_VERSION, fv(1, 26, 60));
        assert_eq!(ENGINE_VERSION.to_string(), "1.26.60");
    }

    /// A struct holding a format version as a member, as facet reads and writes it.
    #[derive(Debug, PartialEq, facet::Facet)]
    struct Held {
        version: FormatVersion,
    }

    #[test]
    fn a_format_version_is_written_as_its_string_form() {
        let mut out = Vec::new();
        bedrock_addon_core::json::write_json(
            &Held {
                version: fv(1, 21, 60),
            },
            &mut out,
            bedrock_addon_core::json::WriteStyle {
                indent: bedrock_addon_core::json::Indent::Compact,
                trailing_newline: false,
            },
        )
        .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), r#"{"version":"1.21.60"}"#);
    }

    #[test]
    fn a_format_version_reads_its_string_form_back() {
        let held: Held = facet_json::from_str(r#"{"version":"1.21.60"}"#).unwrap();
        assert_eq!(held.version, fv(1, 21, 60));
        assert!(facet_json::from_str::<Held>(r#"{"version":"beta"}"#).is_err());
        assert!(facet_json::from_str::<Held>(r#"{"version":"one"}"#).is_err());
    }

    /// The member read is `SemVersion`'s: a partial version completes with zeros and a
    /// pre-release tag is dropped; the array form `FormatVersionInput::from_json` takes is
    /// refused, as is the beta literal.
    #[test]
    fn a_format_version_member_completes_a_partial_version_and_drops_the_pre_release() {
        let read = |text: &str| -> FormatVersion {
            facet_json::from_str::<Held>(&format!(r#"{{"version":"{text}"}}"#))
                .unwrap()
                .version
        };
        assert_eq!(read("1.21"), fv(1, 21, 0));
        assert_eq!(read("1"), fv(1, 0, 0));
        assert_eq!(read("1.21.60-beta.3"), fv(1, 21, 60));
        assert!(facet_json::from_str::<Held>(r#"{"version":[1, 21, 60]}"#).is_err());
        let refused = FormatVersion::try_from("one".to_owned()).unwrap_err();
        assert_eq!(refused.text, "one");
        assert_eq!(
            refused.to_string(),
            "`one` is not a format version: expected a version string such as \"1.21.60\" \
             (missing numbers are 0, a pre-release tag is dropped)"
        );
    }

    #[test]
    fn the_default_format_version_is_the_least() {
        assert_eq!(FormatVersion::default(), fv(0, 0, 0));
        assert_eq!(FormatVersion::default().max(fv(1, 19, 50)), fv(1, 19, 50));
    }

    #[test]
    fn format_versions_order_by_their_numbers() {
        assert!(fv(1, 21, 60) < fv(1, 21, 100));
        assert!(fv(1, 26, 0) > fv(1, 21, 130));
    }

    #[test]
    fn a_string_format_version() {
        let input = FormatVersionInput::parse("1.21.60").unwrap();
        assert_eq!(input.version, Some(fv(1, 21, 60)));
        assert!(!input.beta_literal);
        assert!(!input.prerelease);
    }

    #[test]
    fn a_prerelease_format_version_is_an_ordinary_version_flagged_prerelease() {
        let input = FormatVersionInput::parse("1.21.60-beta.3").unwrap();
        assert_eq!(input.version, Some(fv(1, 21, 60)));
        assert!(input.prerelease);
        assert!(!input.beta_literal);
    }

    #[test]
    fn the_beta_literal_has_no_version() {
        let input = FormatVersionInput::parse("beta").unwrap();
        assert_eq!(input.version, None);
        assert!(input.beta_literal);
        assert!(!input.prerelease);
    }

    #[test]
    fn malformed_format_versions() {
        for text in ["abc", "*", "", "1.2.3.4"] {
            assert_eq!(FormatVersionInput::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn a_json_format_version_takes_a_string_or_three_integers() {
        let read = |text: &str| FormatVersionInput::from_json(&json(text));
        assert_eq!(read(r#""1.20.80""#).unwrap().version, Some(fv(1, 20, 80)));
        assert_eq!(read("[1, 20, 80]").unwrap().version, Some(fv(1, 20, 80)));
        assert!(read(r#""beta""#).unwrap().beta_literal);
        // Integer-tagged, non-negative elements only; anything else is malformed
        for text in [
            "[1.0, 20, 80]",
            "[1, -20, 80]",
            "[1, 20]",
            "[1, 4294967296, 0]",
            "1.21",
            "true",
            "null",
            "{}",
        ] {
            assert_eq!(read(text), None, "{text}");
        }
    }

    /// The game stores each array element in 16 bits, so an element that fits `uint32` is taken
    /// modulo 65 536.
    #[test]
    fn an_array_element_above_65535_is_truncated() {
        let read = |text: &str| FormatVersionInput::from_json(&json(text));
        assert_eq!(read("[1, 70000, 0]").unwrap().version, Some(fv(1, 4464, 0)));
        assert_eq!(
            read("[4294967295, 65536, 65535]").unwrap().version,
            Some(fv(65535, 0, 65535))
        );
    }

    /// `"9999.9999.9999-beta"` is the beta sentinel itself (the game spells it either way).
    #[test]
    fn the_spelled_out_beta_sentinel_is_the_beta_literal() {
        let input = FormatVersionInput::parse("9999.9999.9999-beta").unwrap();
        assert!(input.beta_literal);
        assert_eq!(input.version, None);
        // The game compares the numbers and the pre-release only: build metadata takes no
        // part.
        assert!(
            FormatVersionInput::parse("9999.9999.9999-beta+x")
                .unwrap()
                .beta_literal
        );
        assert!(
            !FormatVersionInput::parse("9999.9999.9999-beta.1")
                .unwrap()
                .beta_literal
        );
    }

    #[test]
    fn a_format_version_converts_to_a_sem_version() {
        assert_eq!(SemVersion::from(fv(1, 2, 3)), SemVersion::new(1, 2, 3));
    }
}

/// The four range models over three struct versions.
mod ranges {
    use super::*;

    fn input(text: &str) -> FormatVersionInput {
        FormatVersionInput::parse(text).unwrap()
    }

    #[test]
    fn a_right_open_range_excludes_its_end_and_takes_the_end_prerelease() {
        let range = VersionRange::right_open(fv(1, 20, 0), fv(1, 21, 60));
        assert!(range.contains(fv(1, 20, 0)));
        assert!(range.contains(fv(1, 21, 50)));
        assert!(!range.contains(fv(1, 21, 60)));
        assert!(!range.contains(fv(1, 19, 0)));
        // `1.21.60-beta.3` sorts below 1.21.60, so it selects the struct below
        assert!(range.contains_input(&input("1.21.60-beta.3")));
        assert!(!range.contains_input(&input("1.20.0-beta.1")));
    }

    #[test]
    fn a_closed_range_includes_both_ends() {
        let range = VersionRange::closed(fv(1, 21, 60), ENGINE_VERSION);
        assert!(range.contains(fv(1, 21, 60)));
        assert!(range.contains(ENGINE_VERSION));
        assert!(!range.contains(fv(1, 26, 61)));
        assert!(range.contains_input(&input("1.26.60-beta.1")));
        assert!(
            !range.contains_input(&input("1.21.60-beta.1")),
            "FLOOR-beta.n is uncovered"
        );
    }

    #[test]
    fn an_exact_range_takes_only_its_version() {
        let range = VersionRange::exact(fv(1, 21, 30));
        assert!(range.contains(fv(1, 21, 30)));
        assert!(!range.contains(fv(1, 21, 31)));
        assert!(!range.contains(fv(1, 21, 29)));
        assert!(!range.contains_input(&input("1.21.30-beta.1")));
        assert!(range.contains_input(&input("1.21.30")));
    }

    #[test]
    fn the_beta_literal_is_in_no_range() {
        let range = VersionRange::closed(fv(0, 0, 0), fv(u16::MAX, u16::MAX, u16::MAX));
        assert!(!range.contains_input(&input("beta")));
    }

    #[test]
    fn a_half_open_to_engine_chain_has_no_struct_for_the_engine_version() {
        let chain = [
            VersionRange::right_open(fv(1, 21, 60), fv(1, 26, 0)),
            VersionRange::right_open(fv(1, 26, 0), ENGINE_VERSION),
        ];
        assert!(chain.iter().any(|r| r.contains(fv(1, 26, 49))));
        assert!(!chain.iter().any(|r| r.contains(ENGINE_VERSION)));
    }

    #[test]
    fn a_range_shows_its_bounds() {
        assert_eq!(
            VersionRange::right_open(fv(1, 20, 0), fv(1, 21, 60)).to_string(),
            "[1.20.0, 1.21.60)"
        );
        assert_eq!(
            VersionRange::exact(fv(1, 21, 30)).to_string(),
            "[1.21.30, 1.21.30]"
        );
    }
}

/// `min_engine_version` and `base_game_version`.
mod manifest_versions {
    use super::*;

    #[test]
    fn a_min_engine_version_from_numbers() {
        assert_eq!(
            MinEngineVersion::new(1, 26, 20).version,
            SemVersion::new(1, 26, 20)
        );
    }

    #[test]
    fn format_1_defaults_a_missing_version_to_1_2_5() {
        assert_eq!(
            MinEngineVersion::format_1(None),
            MinEngineVersion::new(1, 2, 5)
        );
        assert_eq!(
            MinEngineVersion::FORMAT_1_DEFAULT,
            MinEngineVersion::new(1, 2, 5)
        );
    }

    #[test]
    fn format_1_caps_1_13_0_and_above_to_1_12_0() {
        assert_eq!(
            MinEngineVersion::format_1(Some(SemVersion::new(1, 13, 0))),
            MinEngineVersion::new(1, 12, 0)
        );
        assert_eq!(
            MinEngineVersion::format_1(Some(SemVersion::new(1, 20, 0))),
            MinEngineVersion::FORMAT_1_CAP
        );
        assert!(MinEngineVersion::new(1, 13, 0).exceeds_format_1_cap());
        assert!(!MinEngineVersion::new(1, 12, 9).exceeds_format_1_cap());
    }

    #[test]
    fn format_1_keeps_a_version_below_the_cap() {
        assert_eq!(
            MinEngineVersion::format_1(Some(SemVersion::new(1, 12, 0))),
            MinEngineVersion::new(1, 12, 0)
        );
        assert_eq!(
            MinEngineVersion::format_1(Some(sem("1.13.0-beta"))).version,
            sem("1.13.0-beta"),
            "a pre-release of 1.13.0 sorts below it"
        );
    }

    #[test]
    fn the_molang_version_of_a_min_engine_version() {
        use bedrock_addon_core::molang::MolangVersion;
        // The Molang version table: [1.20.70, 1.21.100) → 12; ≥ 1.21.100 → 13; 1.2.5 → 0; "*" → 13
        assert_eq!(
            MinEngineVersion::new(1, 21, 0).molang_version(),
            MolangVersion::V12
        );
        assert_eq!(
            MinEngineVersion::new(1, 26, 20).molang_version(),
            MolangVersion::V13
        );
        assert_eq!(
            MinEngineVersion::FORMAT_1_DEFAULT.molang_version(),
            MolangVersion::V0
        );
        assert_eq!(
            MinEngineVersion {
                version: SemVersion::ANY
            }
            .molang_version(),
            MolangVersion::V13
        );
        assert_eq!(
            MinEngineVersion {
                version: sem("1.21.100-beta")
            }
            .molang_version(),
            MolangVersion::V12
        );
    }

    #[test]
    fn a_world_version_includes_the_slices_at_or_below_it() {
        let world = BaseGameVersion::new(SemVersion::new(1, 20, 0));
        assert!(world.includes(&SemVersion::new(1, 19, 0)));
        assert!(world.includes(&SemVersion::new(1, 20, 0)));
        assert!(!world.includes(&SemVersion::new(1, 20, 10)));
    }

    #[test]
    fn the_any_and_the_empty_world_version_include_every_slice() {
        assert!(BaseGameVersion::ANY.is_any());
        assert!(BaseGameVersion::ANY.includes(&SemVersion::new(1, 21, 0)));
        assert_eq!(BaseGameVersion::default(), BaseGameVersion::EMPTY);
        assert!(BaseGameVersion::EMPTY.includes(&SemVersion::new(1, 21, 0)));
        assert!(!BaseGameVersion::INCOMPATIBLE.includes(&SemVersion::new(1, 0, 0)));
    }
}

/// The write-back class "version respelt": the model holds the version, not
/// its spelling, and writes it in full as a string.
mod version_respelt {
    use super::*;

    #[test]
    fn a_format_version_is_written_in_full_whatever_its_spelling() {
        let written = |text: &str| {
            FormatVersionInput::from_json(&json(text))
                .unwrap()
                .version
                .unwrap()
                .to_string()
        };
        for (spelling, full) in [
            (r#""1.21.130""#, "1.21.130"),
            // A string with leading zeros is no version (the game's regex rejects it); the
            // array's numbers are the JSON reader's, which reads `01` as 1.
            ("[01, 021, 0130]", "1.21.130"),
            // An element above 65 535 is truncated to 16 bits.
            ("[1, 21, 65666]", "1.21.130"),
            // A short string and the array form.
            (r#""1.26""#, "1.26.0"),
            ("[1, 26, 0]", "1.26.0"),
        ] {
            assert_eq!(written(spelling), full, "{spelling}");
        }
    }
}
