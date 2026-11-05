//! The 1.26.30 format (`v1_26_30::ItemDocument`, the first modern slice, `FIRST_MODERN_SLICE`):
//! the members of [`crate::Item`], whose four sound members
//! (`use_modifiers.start_sound`, `swing_sounds.*`, `durability_thresholds[].sound_event`) bind
//! a `LevelSoundEvent`. The step to 1.26.50 turns each into its name.

/// An item document of the 1.26.30 format (also 1.26.31–1.26.49): [`crate::Item`]'s
/// members, the sound members holding level sound event names.
pub type Item = crate::ModernItem<{ super::pack(1, 26, 30) }>;
