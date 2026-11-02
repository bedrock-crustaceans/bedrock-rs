//! `MemoryTier`: the hardware memory tier a subpack is meant for.

/// The hardware memory tier a subpack is meant for, `Tier0` (least memory) to `Tier5` (most):
/// a subpack's `memory_performance_tier` (format 3) or the tier of its legacy `memory_tier`.
///
/// A device or server uses the highest-tier subpack at or below its own tier.
#[doc(alias = "HardwareMemoryTier")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum MemoryTier {
    /// 0: the lowest tier, and the tier of a legacy `memory_tier` from 0 to 10.
    #[default]
    Tier0 = 0,
    /// 1: legacy `memory_tier` 11.
    Tier1 = 1,
    /// 2: legacy `memory_tier` 12 to 17.
    Tier2 = 2,
    /// 3: legacy `memory_tier` 18 to 23.
    Tier3 = 3,
    /// 4: legacy `memory_tier` 24 to 31.
    Tier4 = 4,
    /// 5: the highest tier, legacy `memory_tier` 32 and above.
    Tier5 = 5,
}

impl MemoryTier {
    /// The tier numbered `number` (`memory_performance_tier`); `None` above 5.
    #[must_use]
    pub const fn from_number(number: u8) -> Option<MemoryTier> {
        match number {
            0 => Some(Self::Tier0),
            1 => Some(Self::Tier1),
            2 => Some(Self::Tier2),
            3 => Some(Self::Tier3),
            4 => Some(Self::Tier4),
            5 => Some(Self::Tier5),
            _ => None,
        }
    }

    /// The tier's number, 0 to 5.
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// The tier of a legacy `memory_tier` (a memory size): 32 and above is `Tier5`, 24 `Tier4`,
    /// 18 `Tier3`, 12 `Tier2`, 11 `Tier1`, anything lower `Tier0`.
    pub(super) const fn from_legacy_size(size: i64) -> MemoryTier {
        match size {
            32.. => Self::Tier5,
            24.. => Self::Tier4,
            18.. => Self::Tier3,
            12.. => Self::Tier2,
            11 => Self::Tier1,
            _ => Self::Tier0,
        }
    }

    /// The smallest legacy `memory_tier` of the tier, which a legacy manifest is written with.
    pub(super) const fn legacy_size(self) -> u8 {
        match self {
            Self::Tier0 => 0,
            Self::Tier1 => 11,
            Self::Tier2 => 12,
            Self::Tier3 => 18,
            Self::Tier4 => 24,
            Self::Tier5 => 32,
        }
    }
}
