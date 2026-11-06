//! Content keys for marketplace (encrypted) packs, supplied by the caller.

use std::collections::BTreeMap;
use std::fmt;

/// A content key: the 32 bytes of a marketplace key string. The pack key decrypts
/// `contents.json`; each asset has its own key, listed there.
///
/// The cipher uses the 32 bytes as the AES-256 key and their first 16 as the IV. `Debug`
/// prints `ContentKey(..)`, never the key, so logging an input does not leak key material.
#[derive(Clone, PartialEq, Eq)]
pub struct ContentKey {
    bytes: [u8; 32],
}

impl ContentKey {
    /// A key from its 32 bytes.
    pub fn new(bytes: [u8; 32]) -> ContentKey {
        ContentKey { bytes }
    }

    /// A key from the first 32 bytes of a key string (the game reads only those); `None` if it
    /// is shorter.
    pub fn from_slice(text: &[u8]) -> Option<ContentKey> {
        let bytes = text.get(..32)?.try_into().ok()?;
        Some(ContentKey { bytes })
    }

    /// The 32 key bytes.
    #[cfg(feature = "encrypted")]
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl From<[u8; 32]> for ContentKey {
    fn from(bytes: [u8; 32]) -> ContentKey {
        ContentKey::new(bytes)
    }
}

impl fmt::Debug for ContentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ContentKey(..)")
    }
}

/// The `ContentIdentity` string of a `contents.json` header: the name a pack's content key is
/// looked up by.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentIdentity {
    text: Box<str>,
}

impl ContentIdentity {
    /// An identity from its text.
    pub fn new(text: impl Into<Box<str>>) -> ContentIdentity {
        ContentIdentity { text: text.into() }
    }

    /// The identity text.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl From<&str> for ContentIdentity {
    fn from(text: &str) -> ContentIdentity {
        ContentIdentity::new(text)
    }
}

impl From<String> for ContentIdentity {
    fn from(text: String) -> ContentIdentity {
        ContentIdentity::new(text)
    }
}

impl AsRef<str> for ContentIdentity {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// The identity text.
impl fmt::Display for ContentIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Content keys by identity: the map `OpenOptions::keys` holds. `Debug` lists the identities
/// and prints each key as `ContentKey(..)`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContentKeys {
    keys: BTreeMap<ContentIdentity, ContentKey>,
}

impl ContentKeys {
    /// An empty map.
    pub fn new() -> ContentKeys {
        ContentKeys::default()
    }

    /// Adds the key for `identity`, returning the key it replaces.
    pub fn insert(&mut self, identity: ContentIdentity, key: ContentKey) -> Option<ContentKey> {
        self.keys.insert(identity, key)
    }

    /// The key for `identity`, if held.
    pub fn get(&self, identity: &ContentIdentity) -> Option<&ContentKey> {
        self.keys.get(identity)
    }

    /// Every held key with its identity, in identity order.
    pub fn iter(&self) -> impl Iterator<Item = (&ContentIdentity, &ContentKey)> + '_ {
        self.keys.iter()
    }

    /// How many keys are held.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// No key is held.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl<'a> IntoIterator for &'a ContentKeys {
    type Item = (&'a ContentIdentity, &'a ContentKey);
    type IntoIter = std::collections::btree_map::Iter<'a, ContentIdentity, ContentKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter()
    }
}

impl FromIterator<(ContentIdentity, ContentKey)> for ContentKeys {
    fn from_iter<I: IntoIterator<Item = (ContentIdentity, ContentKey)>>(iter: I) -> Self {
        ContentKeys {
            keys: iter.into_iter().collect(),
        }
    }
}

impl Extend<(ContentIdentity, ContentKey)> for ContentKeys {
    fn extend<I: IntoIterator<Item = (ContentIdentity, ContentKey)>>(&mut self, iter: I) {
        self.keys.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_enumerate_in_identity_order_and_compare_equal() {
        let keys: ContentKeys = [
            (ContentIdentity::from("b"), ContentKey::from([2; 32])),
            (
                ContentIdentity::from(String::from("a")),
                ContentKey::from([1; 32]),
            ),
        ]
        .into_iter()
        .collect();
        let identities: Vec<&str> = keys.iter().map(|(identity, _)| identity.as_ref()).collect();
        assert_eq!(identities, ["a", "b"]);
        let mut walked = 0;
        for (identity, key) in &keys {
            assert_eq!(keys.get(identity), Some(key));
            walked += 1;
        }
        assert_eq!(walked, keys.len());
        assert_eq!(keys.clone(), keys);
        assert_ne!(keys, ContentKeys::new());
    }
}
