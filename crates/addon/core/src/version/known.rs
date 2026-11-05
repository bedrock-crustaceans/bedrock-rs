//! The format versions the families name most, declared once.
//!
//! A family that means something by a version keeps its own name for it and its own
//! documentation (`const TYPED: FormatVersion = known::V1_26_30;`); only the literal comes from
//! here. Each constant says what the tree uses it for, counted when the module was made; the
//! game's meaning of a version is the family's to state.

use super::FormatVersion;

/// 0.0.0: no version at all. The `VERSION` of an unversioned document and the `FLOOR` of a
/// family that reads every version (eight declarations in seven crates).
pub const V0_0_0: FormatVersion = FormatVersion::new(0, 0, 0);

/// 1.8.0: the `FLOOR` of entity (its first version with `description.identifier`) and of
/// attachable, and the version of several first slices (ten declarations in seven crates).
pub const V1_8_0: FormatVersion = FormatVersion::new(1, 8, 0);

/// 1.10.0: a slice version and a family floor (seven declarations).
pub const V1_10_0: FormatVersion = FormatVersion::new(1, 10, 0);

/// 1.16.100: the first texture-set schema, and the version item documents are told
/// data-driven from (six declarations).
pub const V1_16_100: FormatVersion = FormatVersion::new(1, 16, 100);

/// 1.21.30: the cereal slice of trade tables, and a version written inline fifty times.
pub const V1_21_30: FormatVersion = FormatVersion::new(1, 21, 30);

/// 1.21.90: the version several deprecations and namespace rules date from (eight
/// declarations).
pub const V1_21_90: FormatVersion = FormatVersion::new(1, 21, 90);

/// 1.21.100: a component typing version and a family floor (nine declarations).
pub const V1_21_100: FormatVersion = FormatVersion::new(1, 21, 100);

/// 1.21.110: a component typing version and a family floor; also used as a `SemVersion`
/// gate (eighteen declarations in six crates).
pub const V1_21_110: FormatVersion = FormatVersion::new(1, 21, 110);

/// 1.21.120: a component typing version (fourteen declarations in seven crates).
pub const V1_21_120: FormatVersion = FormatVersion::new(1, 21, 120);

/// 1.21.130: a component typing version and a modern-slice boundary (seven declarations).
pub const V1_21_130: FormatVersion = FormatVersion::new(1, 21, 130);

/// 1.26.0: a component typing version and the newest legacy slice of several families
/// (thirty declarations in eight crates).
pub const V1_26_0: FormatVersion = FormatVersion::new(1, 26, 0);

/// 1.26.10: a component typing version (twenty-nine declarations in three crates).
pub const V1_26_10: FormatVersion = FormatVersion::new(1, 26, 10);

/// 1.26.20: a component typing version and the first version blocks are read with the cereal
/// document (forty-seven declarations in five crates).
pub const V1_26_20: FormatVersion = FormatVersion::new(1, 26, 20);

/// 1.26.30: the version most components are typed at (`TYPED`, forty-two of its fifty-seven
/// declarations, in nine crates).
pub const V1_26_30: FormatVersion = FormatVersion::new(1, 26, 30);

/// 1.26.40: a component typing version and the version of several newest slices
/// (thirty-six declarations in eight crates).
pub const V1_26_40: FormatVersion = FormatVersion::new(1, 26, 40);

/// 1.26.50: the version of several newest slices (sixteen declarations in ten crates).
pub const V1_26_50: FormatVersion = FormatVersion::new(1, 26, 50);

/// 1.26.60: the engine version the crate tracks today ([`ENGINE_VERSION`](super::ENGINE_VERSION)
/// is declared from it) and the newest slice of several families.
pub const V1_26_60: FormatVersion = FormatVersion::new(1, 26, 60);

#[cfg(test)]
mod tests {
    use super::*;

    /// Each name is its number, and the list is in order.
    #[test]
    fn each_constant_is_the_version_its_name_spells() {
        let all = [
            ("V0_0_0", V0_0_0),
            ("V1_8_0", V1_8_0),
            ("V1_10_0", V1_10_0),
            ("V1_16_100", V1_16_100),
            ("V1_21_30", V1_21_30),
            ("V1_21_90", V1_21_90),
            ("V1_21_100", V1_21_100),
            ("V1_21_110", V1_21_110),
            ("V1_21_120", V1_21_120),
            ("V1_21_130", V1_21_130),
            ("V1_26_0", V1_26_0),
            ("V1_26_10", V1_26_10),
            ("V1_26_20", V1_26_20),
            ("V1_26_30", V1_26_30),
            ("V1_26_40", V1_26_40),
            ("V1_26_50", V1_26_50),
            ("V1_26_60", V1_26_60),
        ];
        for (name, version) in all {
            assert_eq!(format!("V{}", version.to_string().replace('.', "_")), name);
        }
        assert!(all.windows(2).all(|pair| pair[0].1 < pair[1].1));
    }

    #[test]
    fn the_engine_version_is_today_v1_26_60() {
        assert_eq!(super::super::ENGINE_VERSION, V1_26_60);
    }
}
