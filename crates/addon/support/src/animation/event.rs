//! Event strings (`timeline`, `on_entry`, `on_exit`): vanilla's split into command, entity event
//! and Molang, and the pack's command version.

use super::types::EventString;
use bedrock_addon_core::molang::{MolangSource, RawVersion};
use bedrock_addon_core::version::{CURRENT_COMMAND_VERSION, MinEngineVersion};

impl EventString {
    /// Splits `text` the way the game does: `""` and `@…` other than `@s ` are nothing, `/…` a
    /// command, `@s <name>` an entity event, anything else Molang.
    #[must_use]
    pub fn classify(text: &str, command_version: u8) -> Self {
        let bytes = text.as_bytes();
        match bytes.first() {
            None => Self::Ignored(String::new()),
            Some(b'/') => Self::Command {
                command: text[1..].to_owned(),
                command_version,
            },
            Some(b'@') if bytes.len() >= 3 && bytes[1] == b's' && bytes[2] == b' ' => {
                Self::EntityEvent(text[3..].to_owned())
            }
            Some(b'@') => Self::Ignored(text.to_owned()),
            // The context version is set by pass 1 (the pack's, or the controller's effective).
            Some(_) => Self::Molang(MolangSource::string(text, RawVersion(-1))),
        }
    }

    /// The string as it is written back: the inverse of [`EventString::classify`].
    #[must_use]
    pub fn to_text(&self) -> String {
        match self {
            Self::Command { command, .. } => format!("/{command}"),
            Self::EntityEvent(name) => format!("@s {name}"),
            Self::Molang(source) => source.as_str().to_owned(),
            Self::Ignored(text) => text.clone(),
        }
    }
}

/// The pack's command version, from its `min_engine_version`: 46 without a version.
pub fn command_version(min_engine_version: Option<&MinEngineVersion>) -> u8 {
    min_engine_version.map_or(CURRENT_COMMAND_VERSION, MinEngineVersion::command_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_version_follows_the_games_table() {
        let at = |a, b, c| command_version(Some(&MinEngineVersion::new(a, b, c)));
        assert_eq!(at(1, 0, 0), 8);
        assert_eq!(at(1, 8, 0), 8);
        assert_eq!(at(1, 8, 1), 8);
        assert_eq!(at(100, 0, 1), 46);
        assert_eq!(command_version(None), 46);
        assert_eq!(at(1, 18, 30), 19);
        assert_eq!(at(1, 16, 0), 12);
        assert_eq!(at(1, 21, 0), 39);
    }

    #[test]
    fn classify_splits_as_the_event_constructor() {
        assert!(matches!(EventString::classify("", 1), EventString::Ignored(t) if t.is_empty()));
        assert!(
            matches!(EventString::classify("/say hi", 1), EventString::Command { command, .. } if command == "say hi")
        );
        assert!(
            matches!(EventString::classify("@s  x", 1), EventString::EntityEvent(e) if e == " x")
        );
        assert!(matches!(
            EventString::classify("@s", 1),
            EventString::Ignored(_)
        ));
        assert!(matches!(
            EventString::classify("@S x", 1),
            EventString::Ignored(_)
        ));
        assert!(matches!(
            EventString::classify("v.x = 1;", 1),
            EventString::Molang(_)
        ));
    }
}
