//! Privileges: in which execution contexts a script may call a function, read or write a
//! property, or in which a callback runs.

use std::fmt;
use std::ops::{BitAnd, BitOr, BitOrAssign};

use super::MetaError;
use super::model::script::PrivilegeName;

/// A set of privilege flags over the contexts an API member allows
/// beyond the default one. [`DEFAULT`](Self::DEFAULT) is the empty set (`default` in the files);
/// [`RESTRICTED_EXECUTION`](Self::RESTRICTED_EXECUTION) and
/// [`EARLY_EXECUTION`](Self::EARLY_EXECUTION) are the two flags, and a member may have both.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Privilege(u8);

impl Privilege {
    /// `default`: no flag.
    pub const DEFAULT: Self = Self(0);
    /// `restricted_execution` (bit 0).
    pub const RESTRICTED_EXECUTION: Self = Self(1);
    /// `early_execution` (bit 1).
    pub const EARLY_EXECUTION: Self = Self(2);

    /// The flags as vanilla's bitmask.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// The set with the bits of `bits`; `None` when a bit other than the two flags is set.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !3 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Every flag of `other` is in this set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// No flag: the `default` privilege.
    #[must_use]
    pub const fn is_default(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for Privilege {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for Privilege {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl BitAnd for Privilege {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl fmt::Binary for Privilege {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for Privilege {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Debug for Privilege {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names = Vec::new();
        if self.contains(Self::RESTRICTED_EXECUTION) {
            names.push("RESTRICTED_EXECUTION");
        }
        if self.contains(Self::EARLY_EXECUTION) {
            names.push("EARLY_EXECUTION");
        }
        if names.is_empty() {
            names.push("DEFAULT");
        }
        write!(f, "Privilege({})", names.join(" | "))
    }
}

/// A privilege list as written (`call_privilege`, `get_privilege`, `set_privilege`, or a
/// closure's single `call_privilege` object): the flags, and any name outside the generator's
/// three (only `read_only`, in script-generated members of older tags).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PrivilegeList {
    /// The flags the list names.
    pub bits: Privilege,
    /// Names outside `default`, `restricted_execution` and `early_execution`, in file order.
    pub overlay: Vec<String>,
}

impl PrivilegeList {
    /// Reads a list of privilege names the way the generator writes them: `default` alone, or
    /// `restricted_execution` and `early_execution`, each at most once and in that order; any
    /// other name goes to `overlay`. An empty list, `default` beside another name, or a repeated
    /// or reordered flag is an error.
    ///
    /// # Errors
    ///
    /// Returns [`MetaError::EmptyPrivilegeList`] for an empty list, [`MetaError::DefaultNotAlone`]
    /// for `default` beside another name, and [`MetaError::PrivilegeOrder`] for a repeated or
    /// reordered flag.
    pub fn from_names(names: &[PrivilegeName]) -> Result<Self, MetaError> {
        // The game writes a privilege value as a list: value 0 is `default`; bit 0 is
        // `restricted_execution`, then bit 1 `early_execution`; any other value adds nothing.
        if names.is_empty() {
            return Err(MetaError::EmptyPrivilegeList);
        }
        if names.iter().any(|name| name.name == "default") {
            return if names.len() == 1 {
                Ok(Self::default())
            } else {
                Err(MetaError::DefaultNotAlone)
            };
        }
        let mut list = Self::default();
        for name in names {
            let flag = match name.name.as_str() {
                "restricted_execution" => Privilege::RESTRICTED_EXECUTION,
                "early_execution" => Privilege::EARLY_EXECUTION,
                other => {
                    list.overlay.push(other.to_owned());
                    continue;
                }
            };
            // A flag after an equal or higher one is a repeat or the other order.
            if list.bits.0 >= flag.0 {
                return Err(MetaError::PrivilegeOrder);
            }
            list.bits |= flag;
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privilege_combines_and_formats_as_a_bit_set() {
        let mut flags = Privilege::DEFAULT;
        flags |= Privilege::EARLY_EXECUTION;
        flags |= Privilege::RESTRICTED_EXECUTION;
        assert_eq!(flags.bits(), 3);
        assert_eq!(
            flags & Privilege::EARLY_EXECUTION,
            Privilege::EARLY_EXECUTION
        );
        assert_eq!(flags & Privilege::DEFAULT, Privilege::DEFAULT);
        assert_eq!(format!("{flags:b}"), "11");
        assert_eq!(format!("{:#04x}", Privilege::EARLY_EXECUTION), "0x02");
    }
}
