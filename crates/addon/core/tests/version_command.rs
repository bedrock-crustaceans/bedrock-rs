//! `MinEngineVersion::command_version`: the game maps a pack's `min_engine_version` to the
//! command-system version its `.mcfunction` commands parse at. The tests cover the whole table,
//! both ends of every range.

use bedrock_addon_core::version::{CURRENT_COMMAND_VERSION, MinEngineVersion, SemVersion};

/// The effective version table: the lowest `min_engine_version` of
/// each command version, duplicate keys collapsed to the value the lookup returns.
const EFFECTIVE: [((u16, u16, u16), u8); 33] = [
    ((1, 8, 0), 8),
    ((1, 9, 0), 9),
    ((1, 13, 0), 10),
    ((1, 15, 0), 11),
    ((1, 16, 0), 12),
    ((1, 16, 100), 13),
    ((1, 16, 210), 14),
    ((1, 16, 220), 15),
    ((1, 17, 30), 16),
    ((1, 17, 40), 17),
    ((1, 18, 10), 18),
    ((1, 18, 20), 19),
    ((1, 19, 0), 20),
    ((1, 19, 10), 21),
    ((1, 19, 20), 22),
    ((1, 19, 30), 23),
    ((1, 19, 40), 24),
    ((1, 19, 50), 25),
    ((1, 19, 60), 26),
    ((1, 19, 70), 32),
    ((1, 19, 80), 33),
    ((1, 20, 0), 34),
    ((1, 20, 10), 35),
    ((1, 20, 30), 36),
    ((1, 20, 60), 37),
    ((1, 20, 70), 38),
    ((1, 21, 0), 39),
    ((1, 21, 10), 41),
    ((1, 21, 20), 42),
    ((1, 21, 70), 43),
    ((1, 21, 90), 44),
    ((1, 21, 130), 45),
    ((1, 26, 10), 46),
];

fn at(major: u16, minor: u16, patch: u16) -> u8 {
    MinEngineVersion::new(major, minor, patch).command_version()
}

/// The version one patch below `major.minor.patch`.
fn below((major, minor, patch): (u16, u16, u16)) -> (u16, u16, u16) {
    if patch > 0 {
        (major, minor, patch - 1)
    } else {
        (major, minor - 1, u16::MAX)
    }
}

#[test]
fn f23_every_key_maps_to_its_command_version() {
    for ((major, minor, patch), expected) in EFFECTIVE {
        assert_eq!(at(major, minor, patch), expected, "{major}.{minor}.{patch}");
    }
}

#[test]
fn f23_one_patch_below_a_key_is_the_previous_command_version() {
    // Below the first key there is no previous entry: vanilla returns the first value.
    let mut previous = 8;
    for (key, value) in EFFECTIVE {
        let (major, minor, patch) = below(key);
        assert_eq!(at(major, minor, patch), previous, "{major}.{minor}.{patch}");
        previous = value;
    }
}

#[test]
fn f23_vanillas_unit_test_versions() {
    // "1.0.0" is the earliest supported version, "100.0.1" the current one.
    assert_eq!(at(1, 0, 0), 8);
    assert_eq!(at(1, 8, 0), 8);
    assert_eq!(at(1, 8, 1), 8);
    assert_eq!(at(100, 0, 1), 46);
    assert_eq!(at(1, 18, 0), 17);
    assert_eq!(at(1, 18, 30), 19);
}

#[test]
fn f23_the_fixture_list_versions() {
    assert_eq!(at(1, 19, 75), 32);
    assert_eq!(at(1, 21, 15), 41);
    assert_eq!(at(1, 7, 0), 8);
    assert_eq!(at(1, 26, 20), 46);
}

#[test]
fn f23_an_unset_version_is_the_current_command_version() {
    // Vanilla's default `MinEngineVersion` holds 46: the caller's `None`.
    let unset: Option<MinEngineVersion> = None;
    assert_eq!(
        unset
            .as_ref()
            .map_or(CURRENT_COMMAND_VERSION, MinEngineVersion::command_version),
        46
    );
    assert_eq!(CURRENT_COMMAND_VERSION, 46);
}

#[test]
fn f23_a_pre_release_sorts_below_its_release() {
    let beta = SemVersion::parse("1.16.0-beta").unwrap();
    assert_eq!(MinEngineVersion { version: beta }.command_version(), 11);
    let release = SemVersion::parse("1.16.0").unwrap();
    assert_eq!(MinEngineVersion { version: release }.command_version(), 12);
}

#[test]
fn f23_the_any_version_is_the_current_command_version() {
    let any = MinEngineVersion {
        version: SemVersion::ANY,
    };
    assert_eq!(any.command_version(), 46);
}

#[test]
fn a_duplicate_key_gives_its_last_value() {
    // 27 to 31 share 1.19.70 with 32, and 40 shares 1.21.10 with 41: no version maps to them.
    let reachable: Vec<u8> = EFFECTIVE.iter().map(|(_, value)| *value).collect();
    for unreachable in [27, 28, 29, 30, 31, 40] {
        assert!(!reachable.contains(&unreachable));
    }
    assert_eq!(at(1, 19, 70), 32);
    assert_eq!(at(1, 21, 10), 41);
}

#[test]
fn the_command_version_never_decreases_with_the_engine_version() {
    let mut previous = 8;
    for minor in 0..40 {
        for patch in [
            0, 1, 9, 10, 19, 20, 29, 30, 40, 50, 60, 70, 80, 90, 100, 130, 210, 220,
        ] {
            let version = at(1, minor, patch);
            assert!(version >= previous, "1.{minor}.{patch}");
            assert!((8..=46).contains(&version), "1.{minor}.{patch}");
            previous = version;
        }
    }
}
