//! `entity_types`: one mob descriptor or a list of them, shared by the goals and
//! `wither_target_highest_damage`. Each names its own descriptor: [`goal::EntityTypes`] and
//! [`component::EntityTypes`].
//!
//! [`goal::EntityTypes`]: crate::goal::EntityTypes
//! [`component::EntityTypes`]: crate::component::EntityTypes

use facet::Facet;

/// `entity_types`: the mobs a goal reacts to, one descriptor or a list of them (vanilla
/// `EntityTypes`, read as a list and also as a single descriptor). Written back in the form
/// read.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "unboxed so the unknown-member walk keeps the object form's members: behind a `Box` they are reported but not kept for write-back"
)]
pub enum EntityTypes<D> {
    /// `[descriptor, …]`.
    Many(Vec<D>),
    /// A single descriptor object.
    One(D),
}

impl<D> Default for EntityTypes<D> {
    /// The empty list.
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl<D> EntityTypes<D> {
    /// The descriptors, in document order.
    pub fn as_slice(&self) -> &[D] {
        match self {
            Self::Many(list) => list,
            Self::One(one) => std::slice::from_ref(one),
        }
    }

    /// The descriptors, in document order.
    pub fn iter(&self) -> std::slice::Iter<'_, D> {
        self.as_slice().iter()
    }
}

impl<'a, D> IntoIterator for &'a EntityTypes<D> {
    type Item = &'a D;
    type IntoIter = std::slice::Iter<'a, D>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::EntityTypes;

    #[test]
    fn the_descriptors_are_read_in_document_order() {
        let two = EntityTypes::Many(vec![1, 2]);
        assert_eq!(two.as_slice(), [1, 2]);
        assert_eq!((&two).into_iter().count(), 2);
        let one = EntityTypes::One(7);
        assert_eq!(one.iter().copied().collect::<Vec<_>>(), [7]);
        assert_eq!(EntityTypes::<u8>::default(), EntityTypes::Many(Vec::new()));
    }
}
