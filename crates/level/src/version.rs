//! Bedrock game version as stored in level.dat.
//!
//! `lastOpenedWithVersion` and `MinimumCompatibleClientVersion` are both NBT
//! int lists of the same shape: `[major, minor, patch, revision, beta]`,
//! where `beta` is non-zero for a beta build. [`GameVersion`] gives that
//! shape a name and an ordering instead of leaving callers to index into a
//! raw `[i32; 5]`.

use std::fmt;

use facet::Facet;

/// Note on the wire format: on disk this is an `IntArray`, not a compound.
/// nbtx reflects any struct as a compound, so the array/struct conversion is
/// done in [`LevelSettings::read`](crate::settings::LevelSettings::read) — see
/// `settings::unpack_game_versions`. Reading a `GameVersion` through nbtx
/// directly looks for a compound of the five fields below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
pub struct GameVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub revision: i32,
    pub beta: i32,
}

impl GameVersion {
    pub const fn new(major: i32, minor: i32, patch: i32, revision: i32, beta: i32) -> Self {
        Self {
            major,
            minor,
            patch,
            revision,
            beta,
        }
    }

    pub const fn is_beta(&self) -> bool {
        self.beta != 0
    }
}

impl From<GameVersion> for [i32; 5] {
    fn from(version: GameVersion) -> Self {
        [
            version.major,
            version.minor,
            version.patch,
            version.revision,
            version.beta,
        ]
    }
}

impl TryFrom<[i32; 5]> for GameVersion {
    type Error = std::convert::Infallible;

    fn try_from(value: [i32; 5]) -> Result<Self, Self::Error> {
        Ok(Self {
            major: value[0],
            minor: value[1],
            patch: value[2],
            revision: value[3],
            beta: value[4],
        })
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if self.revision != 0 {
            write!(f, ".{}", self.revision)?;
        }

        if self.is_beta() {
            write!(f, "-beta")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_dotted_form() {
        assert_eq!(GameVersion::new(1, 21, 44, 0, 0).to_string(), "1.21.44");
        assert_eq!(GameVersion::new(1, 26, 13, 1, 0).to_string(), "1.26.13.1");
        assert_eq!(GameVersion::new(1, 26, 13, 0, 1).to_string(), "1.26.13-beta");
    }

    #[test]
    fn orders_by_component() {
        assert!(GameVersion::new(1, 21, 0, 0, 0) < GameVersion::new(1, 21, 44, 0, 0));
        assert!(GameVersion::new(1, 20, 99, 0, 0) < GameVersion::new(1, 21, 0, 0, 0));
    }

    #[test]
    fn round_trips_through_int_array() {
        let version = GameVersion::new(1, 26, 13, 1, 0);
        let array: [i32; 5] = version.into();
        assert_eq!(array, [1, 26, 13, 1, 0]);
        assert_eq!(GameVersion::try_from(array).unwrap(), version);
    }
}
