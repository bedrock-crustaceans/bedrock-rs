//! What a definition inherits through its `@base` chain, shared instead of copied: every
//! definition keeps the members it writes and a link to what its base hands on, so the table
//! of a stack is as large as its files, whatever the chains look like.
//!
//! The game copies the inherited members into each definition; here the same view is read
//! through the links, member by member ([`Inherited::get`]) or as a whole
//! ([`Inherited::members`]).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use indexmap::IndexMap;

use super::value::member;
use bedrock_addon_core::json::Value;

/// What a definition inherits: the members its base hands on.
#[derive(Clone, Default)]
pub(crate) enum Inherited {
    /// Nothing: no base, or one that does not exist.
    #[default]
    Nothing,
    /// What a definition outside every cycle hands on.
    Chain(Arc<Link>),
    /// What the member at this position of a cycle hands on.
    Ring(Arc<Ring>, usize),
}

/// A definition as a base: what it hands on to the ones that inherit from it. A `null`
/// member of its own is not handed on, and what its own base has under that key is.
#[derive(Debug)]
pub(crate) struct Link {
    /// The namespace and name of the definition.
    id: (String, String),
    /// The members the definition writes.
    own: Value,
    /// What it inherits itself.
    base: Inherited,
}

/// The definitions of a cycle of bases, each one's base the next and the last one's the
/// first. Each inherits around the cycle once: per member the nearest definition that has
/// it wins.
#[derive(Debug)]
pub(crate) struct Ring {
    /// The namespace and name of each definition, by position.
    ids: Vec<(String, String)>,
    /// The members each definition writes, by position.
    own: Vec<Value>,
    /// Per key, the positions that have a value for it (not `null`), ascending; the keys in
    /// the order the cycle first writes them.
    holders: IndexMap<String, Vec<usize>>,
}

impl Link {
    pub(crate) fn new(id: (String, String), own: Value, base: Inherited) -> Self {
        Self { id, own, base }
    }
}

/// A chain is taken apart link by link: dropping the last definition of a long chain must
/// not recurse through every base.
impl Drop for Link {
    fn drop(&mut self) {
        let mut next = std::mem::take(&mut self.base);
        while let Inherited::Chain(link) = next {
            // A link something else still holds stays; so does everything below it.
            let Ok(mut link) = Arc::try_unwrap(link) else {
                break;
            };
            next = std::mem::take(&mut link.base);
        }
    }
}

impl Ring {
    /// The cycle of the definitions given in base order: each one's id and members.
    pub(crate) fn new(definitions: Vec<((String, String), Value)>) -> Self {
        let mut holders: IndexMap<String, Vec<usize>> = IndexMap::new();
        for (position, (_, own)) in definitions.iter().enumerate() {
            for (key, value) in members_of(own) {
                if !value.is_null() {
                    holders.entry(key.to_owned()).or_default().push(position);
                }
            }
        }
        let (ids, own) = definitions.into_iter().unzip();
        Self { ids, own, holders }
    }

    /// The value of `key` the definition at `from` hands on: its own or, going around the
    /// cycle once, the nearest one.
    fn get(&self, from: usize, key: &str) -> Option<&Value> {
        Self::nearest(self.holders.get(key)?, from)
            .and_then(|holder| member(&self.own[holder], key))
    }

    /// Of the ascending `holders`, the first at or after `from`, else the first of all.
    fn nearest(holders: &[usize], from: usize) -> Option<usize> {
        let at = holders.partition_point(|position| *position < from);
        holders.get(at).or(holders.first()).copied()
    }
}

/// The members of an object in the order written; none for anything else.
fn members_of(value: &Value) -> impl Iterator<Item = (&str, &Value)> {
    value
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter().map(|(key, value)| (key.as_str(), value)))
}

impl Inherited {
    /// The namespace and name of the definition that hands this on.
    pub(crate) fn source(&self) -> Option<(&str, &str)> {
        let (namespace, name) = match self {
            Self::Nothing => return None,
            Self::Chain(link) => &link.id,
            Self::Ring(ring, position) => ring.ids.get(*position)?,
        };
        Some((namespace, name))
    }

    /// The inherited member `key`: the nearest base's that has a value for it. The walk is
    /// as long as the chain above the definition that has it.
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        let mut at = self;
        loop {
            match at {
                Self::Nothing => return None,
                Self::Ring(ring, from) => return ring.get(*from, key),
                Self::Chain(link) => match member(&link.own, key) {
                    Some(value) => return Some(value),
                    None => at = &link.base,
                },
            }
        }
    }

    /// Every inherited member whose key `seen` does not hold yet; `seen` holds their keys
    /// afterwards. The order is no part of the result (a chain gives the nearest base's
    /// first, a cycle its keys as first written): every reader sorts by key. The walk is as
    /// long as everything the chain writes.
    pub(crate) fn members<'a>(&'a self, seen: &mut HashSet<&'a str>) -> Vec<(&'a str, &'a Value)> {
        let mut found = Vec::new();
        let mut at = self;
        loop {
            match at {
                Self::Nothing => return found,
                Self::Ring(ring, from) => {
                    for (key, holders) in &ring.holders {
                        let value = Ring::nearest(holders, *from)
                            .and_then(|holder| member(&ring.own[holder], key));
                        if let Some(value) = value
                            && seen.insert(key)
                        {
                            found.push((key.as_str(), value));
                        }
                    }
                    return found;
                }
                Self::Chain(link) => {
                    for (key, value) in members_of(&link.own) {
                        if !value.is_null() && seen.insert(key) {
                            found.push((key, value));
                        }
                    }
                    at = &link.base;
                }
            }
        }
    }
}

/// The base by its name: a link never prints the chain behind it.
impl std::fmt::Debug for Inherited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.source() {
            None => f.write_str("Nothing"),
            Some((namespace, name)) => write!(f, "From({namespace:?}, {name:?})"),
        }
    }
}

/// Two definitions inherit the same when they inherit from the definition of one namespace
/// and name: a link never compares the chain behind it. That definition is an entry of the
/// table itself and is compared there.
impl PartialEq for Inherited {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source()
    }
}

/// Inherited members looked up for many definitions of one table: every link a lookup
/// walked remembers the answer, so the lookups of one key cost the table's size together,
/// however long its chains are.
#[derive(Default)]
pub(crate) struct Lookups<'a> {
    found: HashMap<(*const Link, &'static str), Option<&'a Value>>,
}

impl<'a> Lookups<'a> {
    /// [`Inherited::get`], remembered.
    pub(crate) fn get(&mut self, inherited: &'a Inherited, key: &'static str) -> Option<&'a Value> {
        let mut walked = Vec::new();
        let mut at = inherited;
        let value = loop {
            match at {
                Inherited::Nothing => break None,
                Inherited::Ring(ring, from) => break ring.get(*from, key),
                Inherited::Chain(link) => {
                    let link_key = (Arc::as_ptr(link), key);
                    if let Some(known) = self.found.get(&link_key) {
                        break *known;
                    }
                    walked.push(link_key);
                    match member(&link.own, key) {
                        Some(value) => break Some(value),
                        None => at = &link.base,
                    }
                }
            }
        };
        for link_key in walked {
            self.found.insert(link_key, value);
        }
        value
    }
}
