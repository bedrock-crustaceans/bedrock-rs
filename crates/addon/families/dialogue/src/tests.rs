//! Unit tests of the dialogue module's private parts.

use super::command::{CURRENT, command_version};
use bedrock_addon_core::version::{MinEngineVersion, SemVersion};

/// The command version of `major.minor.patch`.
fn at(major: u16, minor: u16, patch: u16) -> u8 {
    command_version(Some(&MinEngineVersion::new(major, minor, patch)))
}

/// The effective table: the first engine version of every command
/// version, in order.
const TABLE: [((u16, u16, u16), u8); 33] = [
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

#[test]
fn every_entry_of_the_command_version_map_starts_at_its_engine_version() {
    let mut below = 8;
    for ((major, minor, patch), version) in TABLE {
        assert_eq!(at(major, minor, patch), version, "{major}.{minor}.{patch}");
        // One patch earlier is still the entry before (the patch is never 0 at a boundary
        // except where the minor steps, where the previous minor's highest patch stands in).
        let before = if patch > 0 {
            at(major, minor, patch - 1)
        } else {
            at(major, minor - 1, u16::MAX)
        };
        assert_eq!(before, below, "just below {major}.{minor}.{patch}");
        below = version;
    }
}

#[test]
fn the_command_version_outside_the_map_is_the_earliest_or_the_current_one() {
    // Vanilla's own unit tests: 1.0.0 and 1.8.1 → 8, 100.0.1 → 46, 1.18.30 → 19, an unset
    // version → 46.
    assert_eq!(at(1, 0, 0), 8);
    assert_eq!(at(0, 0, 0), 8);
    assert_eq!(at(1, 7, u16::MAX), 8);
    assert_eq!(at(1, 8, 1), 8);
    assert_eq!(at(1, 18, 30), 19);
    assert_eq!(at(100, 0, 1), 46);
    assert_eq!(at(1, 26, 9), 45);
    assert_eq!(at(1, 26, 11), 46);
    assert_eq!(command_version(None), 46);
    assert_eq!(CURRENT, 46);
    // `*` sorts above every version.
    let any = MinEngineVersion {
        version: SemVersion::ANY,
    };
    assert_eq!(command_version(Some(&any)), 46);
}

#[test]
fn a_pre_release_engine_version_maps_below_its_release() {
    let pre = |text: &str| MinEngineVersion {
        version: SemVersion::parse(text).unwrap(),
    };
    assert_eq!(command_version(Some(&pre("1.16.0-beta"))), 11);
    assert_eq!(command_version(Some(&pre("1.16.0"))), 12);
    assert_eq!(command_version(Some(&pre("1.8.0-beta"))), 8);
    assert_eq!(command_version(Some(&pre("1.26.10-beta"))), 45);
}
