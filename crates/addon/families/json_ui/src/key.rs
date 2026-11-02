//! The key of a definition in a UI file: `name`, `name@base` or `name@namespace.base`.

use std::fmt;

/// The key of a definition: its name and the definition it inherits from.
///
/// Written `name`, `name@base` or `name@namespace.base`; the text after `@` may hold
/// `$variables`.
// The name is the text before the first `@`; everything after it is the base.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefKey {
    /// The definition's name: the key up to its first `@`.
    pub name: String,
    /// The definition it inherits from: the key after its first `@`.
    pub base: Option<DefRef>,
}

/// A reference to a definition: `base` or `namespace.base`.
// The reference is split at its first `.`; without one the namespace is the file's.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefRef {
    /// The namespace before the first `.`; `None` means the namespace of the file.
    pub namespace: Option<String>,
    /// The name of the referenced definition.
    pub name: String,
}

impl DefKey {
    /// The key of a definition without a base.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base: None,
        }
    }

    /// Splits a key as written at its first `@`.
    #[must_use]
    pub fn parse(key: &str) -> Self {
        match key.split_once('@') {
            None => Self::new(key),
            Some((name, reference)) => Self {
                name: name.to_owned(),
                base: Some(DefRef::parse(reference)),
            },
        }
    }
}

impl DefRef {
    /// Splits a reference as written at its first `.`.
    #[must_use]
    pub fn parse(reference: &str) -> Self {
        match reference.split_once('.') {
            None => Self {
                namespace: None,
                name: reference.to_owned(),
            },
            Some((namespace, name)) => Self {
                namespace: Some(namespace.to_owned()),
                name: name.to_owned(),
            },
        }
    }
}

/// The key as a file writes it.
impl fmt::Display for DefKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        match &self.base {
            None => Ok(()),
            Some(base) => write!(f, "@{base}"),
        }
    }
}

/// The reference as a key writes it after `@`.
impl fmt::Display for DefRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{namespace}.")?;
        }
        f.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_splits_at_the_first_at_and_the_first_dot() {
        let key = DefKey::parse("hud@common.base.x@y");
        assert_eq!(key.name, "hud");
        let base = key.base.clone().unwrap();
        assert_eq!(base.namespace.as_deref(), Some("common"));
        assert_eq!(base.name, "base.x@y");
        assert_eq!(key.to_string(), "hud@common.base.x@y");
    }

    #[test]
    fn every_spelling_writes_back_as_written() {
        for text in [
            "a", "a@", "a@b", "a@b.", "a@.b", "@", "", "a@$ref", "a@n.$ref",
        ] {
            assert_eq!(DefKey::parse(text).to_string(), text);
        }
        assert_eq!(DefKey::parse("a").base, None);
        assert_eq!(DefKey::parse("a@b").base.unwrap().namespace, None);
    }
}
