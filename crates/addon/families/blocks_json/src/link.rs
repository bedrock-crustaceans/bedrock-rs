//! What the link stage reads of one pack's `blocks.json` beside the merged registry: which of
//! the four drawing members each entry has, block by block. The registry holds each block's
//! fields merged over every pack, so it cannot say which pack wrote a member, nor that a
//! member was written when its value was refused.

use super::resolve;
use bedrock_addon_core::id::BlockId;

/// The drawing members of one `blocks.json` entry, for one block the entry applies to: what
/// the client's tessellation bookkeeping records of it. A member counts when the entry has
/// it, whatever its value (`null` too): the client tests for the member's presence only.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntrySettings {
    /// The block.
    pub id: BlockId,
    /// The key the entry is written under.
    pub key: String,
    /// The entry has `textures`; without it, it has `blockshape`,
    /// `ambient_occlusion_exponent` or `isotropic`.
    pub textures: bool,
}

/// The entries of the `blocks.json` text `bytes` that have at least one of `textures`,
/// `blockshape`, `ambient_occlusion_exponent` and `isotropic`, once per block each applies to,
/// in the order the client applies them. `base_game` says whether the file is a base-game
/// pack's. A file the client skips gives none.
#[must_use]
pub fn entry_settings(bytes: &[u8], base_game: bool) -> Vec<EntrySettings> {
    let read = resolve::read(bytes, base_game);
    let Some(file) = read.file else {
        return Vec::new();
    };
    let mut settings = Vec::new();
    for applied in read.applied {
        let Some(entry) = file
            .entries
            .get(&applied.key)
            .and_then(|value| value.as_object())
        else {
            continue;
        };
        let textures = entry.get("textures").is_some();
        let other = ["blockshape", "ambient_occlusion_exponent", "isotropic"]
            .iter()
            .any(|member| entry.get(member).is_some());
        if textures || other {
            settings.push(EntrySettings {
                id: applied.id,
                key: applied.key,
                textures,
            });
        }
    }
    settings
}

/// Which of some members one `blocks.json` entry has, for one block the entry applies to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntryMembers {
    /// The block.
    pub id: BlockId,
    /// The key the entry is written under.
    pub key: String,
    /// The members of the asked ones that the entry has, whatever their values.
    pub members: Vec<&'static str>,
}

/// The entries of the `blocks.json` text `bytes` that have at least one of `members`, once
/// per block each applies to, in the order the client applies them. `base_game` says whether
/// the file is a base-game pack's. A file the client skips gives none.
#[must_use]
pub fn entry_members(bytes: &[u8], base_game: bool, members: &[&'static str]) -> Vec<EntryMembers> {
    let read = resolve::read(bytes, base_game);
    let Some(file) = read.file else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for applied in read.applied {
        let Some(entry) = file
            .entries
            .get(&applied.key)
            .and_then(|value| value.as_object())
        else {
            continue;
        };
        let has: Vec<&'static str> = members
            .iter()
            .copied()
            .filter(|member| entry.get(member).is_some())
            .collect();
        if !has.is_empty() {
            found.push(EntryMembers {
                id: applied.id,
                key: applied.key,
                members: has,
            });
        }
    }
    found
}

#[cfg(test)]
mod derive_tests {
    use super::*;
    use crate::{FaceBools3, FaceBools6, FaceTextures, Isotropic};
    use std::fmt::Debug;
    use std::hash::Hash;

    fn assert_value_traits<T: Clone + Debug + PartialEq + Eq + Hash>() {}

    #[test]
    fn entry_and_face_types_have_the_common_traits() {
        assert_value_traits::<EntrySettings>();
        assert_value_traits::<EntryMembers>();
        assert_value_traits::<FaceBools3>();
        assert_value_traits::<FaceBools6>();
        assert_value_traits::<Isotropic>();
        assert_value_traits::<FaceTextures>();
    }
}
