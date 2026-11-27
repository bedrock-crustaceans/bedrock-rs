//! Components: the map of one definition's components in file order, and the
//! traits `#[derive(Components)]` implements to move between a component type and its
//! family's enum.

use std::fmt;

use indexmap::IndexMap;
use indexmap::map::Entry as MapEntry;

use crate::json;
use crate::molang::{MolangFields, MolangVisitor};

/// The closed set of typed components of one family (its component enum).
pub trait ComponentSet: Sized + 'static {
    /// The JSON key of a typed variant (`"minecraft:friction"`); `None` for the custom,
    /// unknown and dynamic variants, whose key is not the type's.
    fn name(&self) -> Option<&'static str>;
}

/// A typed component. Implemented by `#[derive(Components)]` for every variant of a family's
/// component enum.
pub trait Component: Sized {
    /// The family's component set.
    type Set: ComponentSet;
    /// The JSON key, `"minecraft:friction"`.
    const NAME: &'static str;

    /// The component, if `set` is its variant.
    fn from_set(set: &Self::Set) -> Option<&Self>;
    /// The component, mutably, if `set` is its variant.
    fn from_set_mut(set: &mut Self::Set) -> Option<&mut Self>;
    /// The component, if `set` is its variant; `set` back otherwise.
    ///
    /// # Errors
    ///
    /// `set` itself, when it is another variant.
    fn try_from_set(set: Self::Set) -> Result<Self, Self::Set>;
    /// The component as its family's enum.
    fn into_set(self) -> Self::Set;
}

/// A custom component (`ns:name` with parameters).
///
/// `Default` is an unnamed component with `null` parameters, for `..Default::default()`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomComponent {
    /// The JSON key, `"demo:glow"`.
    pub name: String,
    /// The parameters as written.
    pub parameters: json::Value,
    /// Made by an upgrade from an entry of the 1.21.60 `minecraft:custom_components` list,
    /// not written keyed by the author. The component is written keyed like any other; a
    /// family that keeps the list itself (the item side-car) need not write the list again.
    pub from_v1_list: bool,
}

/// One entry of a component map.
#[derive(Clone, Debug, PartialEq)]
enum Entry<C> {
    Typed { value: C, synthesized: bool },
    Custom(CustomComponent),
    Dynamic(json::Value),
    Unknown(json::Value),
}

/// The components of one definition (or permutation, or component group), in file order.
///
/// Two maps are equal when they hold equal entries in the same order.
pub struct ComponentMap<C: ComponentSet> {
    entries: IndexMap<Box<str>, Entry<C>>,
}

impl<C: ComponentSet> ComponentMap<C> {
    /// Empty; `Default` too.
    pub fn new() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }

    /// The component of type `T`, if present: `block.components.get::<Friction>()`.
    pub fn get<T: Component<Set = C>>(&self) -> Option<&T> {
        match self.entries.get(T::NAME)? {
            Entry::Typed { value, .. } => T::from_set(value),
            _ => None,
        }
    }

    /// The component of type `T`, mutably.
    pub fn get_mut<T: Component<Set = C>>(&mut self) -> Option<&mut T> {
        match self.entries.get_mut(T::NAME)? {
            Entry::Typed { value, .. } => T::from_set_mut(value),
            _ => None,
        }
    }

    /// Whether a component of type `T` is present.
    pub fn contains<T: Component<Set = C>>(&self) -> bool {
        self.get::<T>().is_some()
    }

    /// Inserts or replaces, keeping the position of a replaced entry.
    ///
    /// Returns the replaced component of type `T`, if there was one.
    pub fn insert<T: Component<Set = C>>(&mut self, component: T) -> Option<T> {
        match self.insert_entry(
            T::NAME,
            Entry::Typed {
                value: component.into_set(),
                synthesized: false,
            },
        )? {
            Entry::Typed { value, .. } => T::try_from_set(value).ok(),
            _ => None,
        }
    }

    /// Removes the component of type `T`, keeping the order of the others.
    pub fn remove<T: Component<Set = C>>(&mut self) -> Option<T> {
        self.get::<T>()?;
        match self.entries.shift_remove(T::NAME)? {
            Entry::Typed { value, .. } => T::try_from_set(value).ok(),
            _ => None,
        }
    }

    /// Every entry in file order: typed, custom (`ns:name` with parameters) or unknown.
    #[allow(clippy::iter_without_into_iter)] // a component map is iterated through `iter`, which yields `ComponentRef`s
    pub fn iter(&self) -> impl Iterator<Item = ComponentRef<'_, C>> + '_ {
        self.entries.iter().map(|(name, entry)| match entry {
            Entry::Typed { value, synthesized } => ComponentRef::Typed {
                name,
                value,
                synthesized: *synthesized,
            },
            Entry::Custom(custom) => ComponentRef::Custom(custom),
            Entry::Dynamic(json) => ComponentRef::Dynamic { name, json },
            Entry::Unknown(json) => ComponentRef::Unknown { name, json },
        })
    }

    /// How many entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// No entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Inserts or replaces a typed component read from a file or inserted by an upgrade
    /// step (`synthesized`), keyed by its type's name; returns the replaced one. A value
    /// without a typed key (`ComponentSet::name` is `None`) is returned untouched.
    pub fn insert_typed(&mut self, value: C, synthesized: bool) -> Option<C> {
        let Some(name) = value.name() else {
            return Some(value);
        };
        match self.insert_entry(name, Entry::Typed { value, synthesized })? {
            Entry::Typed { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Inserts or replaces a typed component under the key it was written with, for a
    /// family whose key carries a suffix after its type's name (the entity's generic
    /// memory goal `minecraft:memory_behavior.move_to_position.<name>`): one entry per key.
    /// Returns the replaced one; [`get`](Self::get) finds only an entry keyed by the type's
    /// own name, [`iter`](Self::iter) yields every entry with its key.
    pub fn insert_typed_as(&mut self, name: &str, value: C, synthesized: bool) -> Option<C> {
        match self.insert_entry(name, Entry::Typed { value, synthesized })? {
            Entry::Typed { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Inserts or replaces a custom component, keyed by its name.
    pub fn insert_custom(&mut self, component: CustomComponent) {
        let name = component.name.clone();
        self.insert_entry(&name, Entry::Custom(component));
    }

    /// Inserts or replaces a key the family parses at run time, kept as JSON.
    pub fn insert_dynamic(&mut self, name: &str, json: json::Value) {
        self.insert_entry(name, Entry::Dynamic(json));
    }

    /// Inserts or replaces a key no table knows, kept as JSON for write-back.
    pub fn insert_unknown(&mut self, name: &str, json: json::Value) {
        self.insert_entry(name, Entry::Unknown(json));
    }

    /// Removes the entry keyed `name`, whatever its kind, keeping the order of the others;
    /// `false` when there was none. What validation uses to drop a deprecated, vanilla-only or
    /// superseded component it knows only by name.
    pub fn remove_named(&mut self, name: &str) -> bool {
        self.entries.shift_remove(name).is_some()
    }

    /// Inserts `entry` under `name`, in the place of a replaced entry; returns the replaced
    /// one.
    fn insert_entry(&mut self, name: &str, entry: Entry<C>) -> Option<Entry<C>> {
        match self.entries.entry(name.into()) {
            MapEntry::Occupied(mut occupied) => Some(occupied.insert(entry)),
            MapEntry::Vacant(vacant) => {
                vacant.insert(entry);
                None
            }
        }
    }
}

/// The Molang walk descends into each typed component below its key; custom, dynamic and
/// unknown entries hold JSON, not typed Molang fields.
impl<C: ComponentSet + MolangFields> MolangFields for ComponentMap<C> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (name, entry) in &mut self.entries {
            if let Entry::Typed { value, .. } = entry {
                visitor.nested(&**name, value);
            }
        }
    }
}

impl<C: ComponentSet> Default for ComponentMap<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: ComponentSet + Clone> Clone for ComponentMap<C> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<C: ComponentSet + fmt::Debug> fmt::Debug for ComponentMap<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.entries.iter()).finish()
    }
}

/// Equal entries in the same order.
impl<C: ComponentSet + PartialEq> PartialEq for ComponentMap<C> {
    fn eq(&self, other: &Self) -> bool {
        self.entries.len() == other.entries.len() && self.entries.iter().eq(other.entries.iter())
    }
}

/// Typed components keyed by their type's name, in iteration order; a later component of a
/// type replaces the earlier one in its place. Values without a typed key (the custom,
/// unknown and dynamic variants, whose key is not the type's) are skipped.
impl<C: ComponentSet> FromIterator<C> for ComponentMap<C> {
    fn from_iter<I: IntoIterator<Item = C>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

/// As [`FromIterator`]: typed components inserted or replaced in place, the rest skipped.
impl<C: ComponentSet> Extend<C> for ComponentMap<C> {
    fn extend<I: IntoIterator<Item = C>>(&mut self, iter: I) {
        for value in iter {
            self.insert_typed(value, false);
        }
    }
}

/// One entry of a [`ComponentMap`].
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ComponentRef<'a, C> {
    /// A typed component.
    Typed {
        /// The JSON key.
        name: &'a str,
        /// The component, as the family's enum.
        value: &'a C,
        /// Inserted by an upgrade step, not written by the author.
        synthesized: bool,
    },
    /// A custom component (`ns:name` with parameters).
    Custom(&'a CustomComponent),
    /// Entity keys vanilla parses at runtime from JSON (142 keys); known, so not reported.
    Dynamic {
        /// The JSON key.
        name: &'a str,
        /// The value as read.
        json: &'a json::Value,
    },
    /// A key no table knows; kept for write-back, reported as `UnrecognizedComponent`.
    Unknown {
        /// The JSON key.
        name: &'a str,
        /// The value as read.
        json: &'a json::Value,
    },
}

impl<C> Clone for ComponentRef<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ComponentRef<'_, C> {}
