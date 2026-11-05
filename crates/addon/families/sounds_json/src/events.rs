//! The level sound event names `sounds.json` binds sounds to.

/// The events `individual_named_sounds` also takes from `individual_event_sounds`, by name.
// `AmbientCave` and the mood, loop and addition events of the five nether biomes.
pub(crate) const LEGACY_AMBIENT_EVENTS: &[&str] = &[
    "ambient.basalt_deltas.additions",
    "ambient.basalt_deltas.loop",
    "ambient.basalt_deltas.mood",
    "ambient.cave",
    "ambient.crimson_forest.additions",
    "ambient.crimson_forest.loop",
    "ambient.crimson_forest.mood",
    "ambient.nether_wastes.additions",
    "ambient.nether_wastes.loop",
    "ambient.nether_wastes.mood",
    "ambient.soulsand_valley.additions",
    "ambient.soulsand_valley.loop",
    "ambient.soulsand_valley.mood",
    "ambient.warped_forest.additions",
    "ambient.warped_forest.loop",
    "ambient.warped_forest.mood",
];

/// Whether `name`, already lower-cased, is a level sound event (the game's table, in the
/// vanilla data crate).
pub fn is_level_sound_event(name: &str) -> bool {
    bedrock_addon_core::vanilla::sound_events::is_level_sound_event(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_sound_event_is_the_game_s_name_as_written() {
        assert!(is_level_sound_event("ice_ball.break"));
        assert!(is_level_sound_event("undefined"));
        assert!(!is_level_sound_event("Ice_Ball.Break"));
    }

    #[test]
    fn every_legacy_ambient_event_is_a_level_sound_event() {
        assert_eq!(LEGACY_AMBIENT_EVENTS.len(), 16);
        for name in LEGACY_AMBIENT_EVENTS {
            assert!(is_level_sound_event(name), "{name}");
        }
    }
}
