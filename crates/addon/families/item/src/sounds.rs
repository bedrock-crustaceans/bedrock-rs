//! The level sound event names the item sound members are checked against: a stable item
//! document names its sounds by level sound event, so only these names are valid there.

/// The canonical spelling of the level sound event `name` (the table's, lower case), compared
/// without case; `None` for a name that is no level sound event.
pub(crate) fn level_sound_event(name: &str) -> Option<String> {
    bedrock_addon_core::vanilla::sound_events::level_sound_event(name).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::level_sound_event;

    #[test]
    fn a_level_sound_event_is_spelled_as_the_table_spells_it() {
        assert_eq!(level_sound_event("Ambient").as_deref(), Some("ambient"));
        assert_eq!(level_sound_event("custom.boom"), None);
    }
}
