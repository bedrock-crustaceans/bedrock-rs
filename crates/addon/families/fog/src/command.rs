//! The server's half of the `/fog` command: the
//! per-player stack of fog identifiers the server keeps, checks and sends to the client. The
//! server never loads `fogs/` and does not check that a pushed identifier names a definition.

use super::jsonutil::is_namespaced;

/// The most entries a player's fog stack holds: a push onto 16 entries is refused.
pub const FOG_STACK_LIMIT: usize = 16;

/// Whether `/fog … push` takes `identifier`: lower-cased, it must be `namespace:name` over
/// `a`–`z`, digits, `.` and `_`.
pub fn is_valid_fog_identifier(identifier: &str) -> bool {
    is_namespaced(&identifier.to_ascii_lowercase())
}

/// Why a `/fog` operation failed; [`FogCommandError::lang_key`] is the message the game shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum FogCommandError {
    /// The fog identifier is not `namespace:name`.
    #[error("invalid fog setting id")]
    InvalidFogId,
    /// The stack already holds [`FOG_STACK_LIMIT`] entries.
    #[error("size limit reached for fog identifiers")]
    LimitReached,
    /// No entry has the user-provided id.
    #[error("no fog settings with the id were found")]
    InvalidUserId,
    /// A saved stack with an odd number of strings (identifier and user id come in pairs).
    #[error("the saved fog stack has an odd number of items")]
    OddSavedStack,
}

impl FogCommandError {
    /// The lang key of the command's message (`commands.fog.invalidFogId`, …); `None` for a
    /// failure of loading a saved stack, which the game only logs.
    pub fn lang_key(self) -> Option<&'static str> {
        match self {
            Self::InvalidFogId => Some("commands.fog.invalidFogId"),
            Self::LimitReached => Some("commands.fog.limitReached"),
            Self::InvalidUserId => Some("commands.fog.invalidUserId"),
            Self::OddSavedStack => None,
        }
    }
}

/// One player's stack of fog settings, bottom first: each entry is a fog identifier and the
/// id the command's user gave it. The client draws the top-most entry that sets a value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FogStack {
    /// The entries, bottom first: (fog identifier, user-provided id).
    entries: Vec<(String, String)>,
}

impl<'a> IntoIterator for &'a FogStack {
    type Item = &'a (String, String);
    type IntoIter = std::slice::Iter<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl FogStack {
    /// The entries, bottom first: (fog identifier, user-provided id).
    pub fn iter(&self) -> std::slice::Iter<'_, (String, String)> {
        self.entries.iter()
    }

    /// How many entries the stack holds, at most [`FOG_STACK_LIMIT`].
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the stack holds no entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// An empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// `/fog <player> push <fogId> <userProvidedId>`: puts the pair on top.
    ///
    /// # Errors
    ///
    /// Returns [`FogCommandError::InvalidFogId`] when `fog_id` is not `namespace:name`, and
    /// [`FogCommandError::LimitReached`] when the stack already holds [`FOG_STACK_LIMIT`] entries.
    pub fn push(&mut self, fog_id: &str, user_id: &str) -> Result<(), FogCommandError> {
        if !is_valid_fog_identifier(fog_id) {
            return Err(FogCommandError::InvalidFogId);
        }
        if self.entries.len() >= FOG_STACK_LIMIT {
            return Err(FogCommandError::LimitReached);
        }
        self.entries.push((fog_id.to_owned(), user_id.to_owned()));
        Ok(())
    }

    /// `/fog <player> pop <userProvidedId>`: removes the top-most entry with that id.
    ///
    /// # Errors
    ///
    /// Returns [`FogCommandError::InvalidUserId`] when no entry has `user_id`.
    pub fn pop(&mut self, user_id: &str) -> Result<(), FogCommandError> {
        let index = self
            .entries
            .iter()
            .rposition(|(_, user)| user == user_id)
            .ok_or(FogCommandError::InvalidUserId)?;
        self.entries.remove(index);
        Ok(())
    }

    /// `/fog <player> remove <userProvidedId>`: removes every entry with that id.
    ///
    /// # Errors
    ///
    /// Returns [`FogCommandError::InvalidUserId`] when no entry has `user_id`.
    pub fn remove(&mut self, user_id: &str) -> Result<(), FogCommandError> {
        let before = self.entries.len();
        self.entries.retain(|(_, user)| user != user_id);
        if self.entries.len() == before {
            return Err(FogCommandError::InvalidUserId);
        }
        Ok(())
    }

    /// The fog identifiers, bottom to top: the `fog_stack` of `PlayerFogPacket`, which the
    /// server sends after every change.
    pub fn fog_ids(&self) -> Vec<&str> {
        self.entries.iter().map(|(fog, _)| fog.as_str()).collect()
    }

    /// The strings of the player's saved `fogCommandStack` list: identifier and user id of
    /// each entry, in turn.
    pub fn to_saved(&self) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|(fog, user)| [fog.clone(), user.clone()])
            .collect()
    }

    /// The stack a saved `fogCommandStack` list holds. A pair whose fog identifier is not valid
    /// is left out; an odd number of strings is an error (the game logs it and the player
    /// starts with an empty stack).
    ///
    /// # Errors
    ///
    /// Returns [`FogCommandError::OddSavedStack`] when `saved` holds an odd number of strings.
    pub fn from_saved<S: AsRef<str>>(saved: &[S]) -> Result<Self, FogCommandError> {
        let (pairs, rest) = saved.as_chunks::<2>();
        if !rest.is_empty() {
            return Err(FogCommandError::OddSavedStack);
        }
        let entries = pairs
            .iter()
            .filter(|[fog, _]| is_valid_fog_identifier(fog.as_ref()))
            .map(|[fog, user]| (fog.as_ref().to_owned(), user.as_ref().to_owned()))
            .collect();
        Ok(Self { entries })
    }
}
