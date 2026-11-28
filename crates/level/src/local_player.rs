//! Optional, off-by-default normalization helpers for `~local_player`'s
//! coordinate and armor-slot layout -- see the Phase 6 "`~local_player`
//! slot/coordinate normalization helpers" task in `TODO.md`.
//!
//! Nothing here is wired into [`crate::player::PlayerData`] or called by
//! any read/write path in this crate: these are free functions a caller
//! opts into explicitly, because every one of them is lossy in some way
//! (see each item below) -- silently applying any of them inside the
//! normal read/write path would be exactly the kind of unannounced
//! normalization architecture decision 7 exists to rule out.
//!
//! # The player-height Y offset
//!
//! See [`pos_y_to_feet_y`]/[`feet_y_to_pos_y`].
//!
//! `PlayerData::pos`'s Y component is not the player's feet position --
//! it sits [`PLAYER_HEIGHT_OFFSET`] (`1.620_01`) blocks above it. This is
//! not a guess: one reference implementation subtracts exactly this
//! constant when converting a Bedrock player position to its own
//! universal form, and adds it back on write; a comment there records it
//! as an empirically-derived player-height-plus-epsilon offset. This
//! crate's own fixture corpus independently corroborates the same
//! constant without reading that number back into a test: five of the
//! fourteen worlds spawn their local player standing exactly on an
//! integer-Y block, and every one of those five stores a `Pos[1]` whose
//! fractional part is exactly `.62001` (`v1_16`: `73.62001`; `v1_18`:
//! `72.62001`; `v1_18_updated_superflat` and `v1_19_30`: `5.62001` each;
//! `v1_17_40_superflat`: `5.6200104`, the same value to the last
//! representable `f32` digit) -- i.e. `floor(Pos[1]) +
//! PLAYER_HEIGHT_OFFSET`, to the last representable `f32` digit.
//!
//! **Why this is lossy, and off by default.** The offset is a constant:
//! it does not account for the player's actual pose. A crouching player's
//! true eye/camera height is lower than a standing player's, so
//! `pos_y_to_feet_y` applied to a `Pos` captured while [`Sneaking`] (the
//! `sneaking` field on [`crate::player::PlayerData`]) is set overstates
//! how far above the feet the stored Y actually sits by however much the
//! sneaking pose lowers it -- this module does not know that delta and
//! does not attempt to correct for it. The conversion is exactly invertible
//! as arithmetic ([`feet_y_to_pos_y`] undoes [`pos_y_to_feet_y`] bit for
//! bit), but its *meaning* -- "this is the feet Y" -- is only reliable when
//! the player was standing normally when the record was written, which is
//! not observable from the stored data alone.
//!
//! [`Sneaking`]: crate::player::PlayerData::sneaking
//!
//! # Armor slot order
//!
//! `PlayerData::armor`'s four array positions are not head-to-foot or
//! foot-to-head in on-disk order -- they are **reversed** relative to the
//! natural equip order. One reference implementation remaps Bedrock armor
//! index `i` (`0..=3`) to its own universal slot `100 + (3 - i)` when
//! reading, and walks `i` from 3 down to 0 to rebuild the same list when
//! writing -- both directions commented there as reversing the order.
//! Cross-referenced against Java Edition's own well-documented armor slot
//! numbering (`100`=boots, `101`=leggings, `102`=chestplate, `103`=helmet,
//! the same convention that universal numbering is built to line up
//! with), Bedrock's on-disk array order is therefore: index `0` = helmet,
//! `1` = chestplate, `2` = leggings, `3` = boots -- see [`ArmorSlot`].
//!
//! **This is reference-established, not corpus-verified.** Every one of
//! the fourteen fixture worlds' local players has all four armor slots
//! empty (`Count: 0, Name: ""`), so nothing in this crate's own corpus can
//! confirm which physical slot a non-empty entry at a given array index
//! actually holds -- there is no equipped-armor record anywhere in the
//! corpus to check against. Treat [`ArmorSlot`] as carrying the same
//! confidence as any other reference-derived-but-corpus-unconfirmed claim
//! elsewhere in this crate, not as independently verified the way the
//! Y-offset above is.
//!
//! One further corpus observation, unexplained and out of scope for this
//! task: the 1.26 seed world's local player carries **five** `Armor`
//! entries, not four -- every other world's local player has exactly four.
//! [`crate::player::PlayerData::armor`] is a fixed `[InventoryItem; 4]`
//! and does not decode that record strictly (a pre-existing gap, not
//! introduced or fixed here); [`ArmorSlot`] below still only names the
//! four slots the reference mapping above accounts for.
//!
//! # Offhand
//!
//! No normalization helper is offered for offhand. The only documented
//! "offset" either reference applies to it is that reference's own
//! universal slot-numbering convention (`150 + Slot`) -- an artifact of
//! its own cross-edition intermediate model, not a Bedrock-native
//! constant this crate could point to as a format fact the way
//! [`PLAYER_HEIGHT_OFFSET`] and the armor reversal are. There is nothing
//! else to normalize: [`crate::player::PlayerData::offhand`] is already a
//! plain list in on-disk order.

/// How far above the player's feet Y coordinate `PlayerData::pos`'s Y
/// component sits, in blocks. See the module docs for how this is
/// established and why applying it is lossy.
pub const PLAYER_HEIGHT_OFFSET: f32 = 1.620_01;

/// Converts a `PlayerData::pos` Y component to an approximate feet Y.
/// See the module docs' "player-height Y offset" section for why this is
/// lossy (pose-dependent) despite being exact arithmetic.
#[must_use]
pub fn pos_y_to_feet_y(pos_y: f32) -> f32 {
    pos_y - PLAYER_HEIGHT_OFFSET
}

/// Converts an approximate feet Y back to a `PlayerData::pos`-shaped Y --
/// the exact inverse of [`pos_y_to_feet_y`], sharing the same lossiness
/// caveat for what "feet Y" means to begin with.
#[must_use]
pub fn feet_y_to_pos_y(feet_y: f32) -> f32 {
    feet_y + PLAYER_HEIGHT_OFFSET
}

/// The physical identity of one of `PlayerData::armor`'s four array
/// positions -- see the module docs' "Armor slot order" section for the
/// reference derivation and its corpus-unconfirmed caveat.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

impl ArmorSlot {
    /// All four slots, in `PlayerData::armor`'s on-disk array order.
    pub const ALL: [ArmorSlot; 4] = [
        ArmorSlot::Helmet,
        ArmorSlot::Chestplate,
        ArmorSlot::Leggings,
        ArmorSlot::Boots,
    ];

    /// The `PlayerData::armor` array index this slot occupies on disk.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            ArmorSlot::Helmet => 0,
            ArmorSlot::Chestplate => 1,
            ArmorSlot::Leggings => 2,
            ArmorSlot::Boots => 3,
        }
    }

    /// The slot occupying `PlayerData::armor[index]` on disk, or `None` if
    /// `index` is out of range (only `0..=3` name a real slot -- see the
    /// module docs' note on the 1.26 seed world's five-entry anomaly,
    /// which this deliberately does not try to explain).
    #[must_use]
    pub const fn from_index(index: usize) -> Option<ArmorSlot> {
        match index {
            0 => Some(ArmorSlot::Helmet),
            1 => Some(ArmorSlot::Chestplate),
            2 => Some(ArmorSlot::Leggings),
            3 => Some(ArmorSlot::Boots),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_height_offset_matches_the_corpus_to_the_last_f32_digit() {
        // v1_16 and v1_18: local player spawned standing on an integer-Y
        // block (72 and 71 respectively -- see the module docs for the
        // full corpus table); Pos[1] is that block's Y plus one, i.e. the
        // block *above* it, since a standing player's feet rest on top of
        // the block below.
        assert_eq!(feet_y_to_pos_y(72.0), 73.62001);
        assert_eq!(feet_y_to_pos_y(71.0), 72.62001);
        assert_eq!(feet_y_to_pos_y(4.0), 5.62001);
    }

    #[test]
    fn feet_y_and_pos_y_conversions_are_exact_inverses() {
        for pos_y in [0.0f32, 1.5, -30.25, 100.75, 5.62001] {
            assert_eq!(feet_y_to_pos_y(pos_y_to_feet_y(pos_y)), pos_y);
        }
    }

    #[test]
    fn armor_slot_index_round_trips() {
        for slot in ArmorSlot::ALL {
            assert_eq!(ArmorSlot::from_index(slot.index()), Some(slot));
        }
        assert_eq!(ArmorSlot::from_index(4), None);
    }

    #[test]
    fn armor_slot_order_matches_the_documented_reversal() {
        assert_eq!(ArmorSlot::ALL[0], ArmorSlot::Helmet);
        assert_eq!(ArmorSlot::ALL[1], ArmorSlot::Chestplate);
        assert_eq!(ArmorSlot::ALL[2], ArmorSlot::Leggings);
        assert_eq!(ArmorSlot::ALL[3], ArmorSlot::Boots);
    }
}
