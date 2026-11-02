//! A set of names that shares what it inherits: adding a name to a set makes a new set in
//! logarithmic time and leaves the old one as it was, so each material of a parent chain holds
//! its own variant names without copying its ancestors'.

use std::fmt;
use std::sync::Arc;

/// An immutable set of names; a clone shares the whole set.
#[derive(Clone, Default)]
pub(super) struct NameSet {
    root: Option<Arc<Node>>,
}

/// One name of the set, with the names before and after it: a node of a height-balanced
/// tree, so that no order of insertion makes the tree deeper than a few dozen levels.
struct Node {
    name: Arc<str>,
    height: u8,
    len: usize,
    left: Link,
    right: Link,
}

/// A subtree.
type Link = Option<Arc<Node>>;

/// The height of a subtree.
fn height(link: &Link) -> u8 {
    link.as_ref().map_or(0, |node| node.height)
}

/// The number of names of a subtree.
fn len(link: &Link) -> usize {
    link.as_ref().map_or(0, |node| node.len)
}

/// A node over two subtrees that are balanced against each other.
fn node(left: Link, name: Arc<str>, right: Link) -> Arc<Node> {
    Arc::new(Node {
        height: height(&left).max(height(&right)) + 1,
        len: len(&left) + len(&right) + 1,
        name,
        left,
        right,
    })
}

/// A node over two subtrees whose heights differ by at most three: one rotation, single or
/// double, restores the balance.
fn balanced(left: Link, name: Arc<str>, right: Link) -> Arc<Node> {
    let (lh, rh) = (height(&left), height(&right));
    if lh > rh + 2 {
        if let Some(l) = &left {
            if height(&l.left) >= height(&l.right) {
                let inner = node(l.right.clone(), name, right);
                return node(l.left.clone(), l.name.clone(), Some(inner));
            }
            if let Some(lr) = &l.right {
                let low = node(l.left.clone(), l.name.clone(), lr.left.clone());
                let high = node(lr.right.clone(), name, right);
                return node(Some(low), lr.name.clone(), Some(high));
            }
        }
    } else if rh > lh + 2
        && let Some(r) = &right
    {
        if height(&r.right) >= height(&r.left) {
            let inner = node(left, name, r.left.clone());
            return node(Some(inner), r.name.clone(), r.right.clone());
        }
        if let Some(rl) = &r.left {
            let low = node(left, name, rl.left.clone());
            let high = node(rl.right.clone(), r.name.clone(), r.right.clone());
            return node(Some(low), rl.name.clone(), Some(high));
        }
    }
    node(left, name, right)
}

/// The subtree with `name` added; the path to it is copied, the rest is shared. `name` is not
/// in the subtree.
fn inserted(link: &Link, name: &str) -> Arc<Node> {
    let Some(at) = link else {
        return node(None, Arc::from(name), None);
    };
    if name < &*at.name {
        let left = inserted(&at.left, name);
        balanced(Some(left), at.name.clone(), at.right.clone())
    } else {
        let right = inserted(&at.right, name);
        balanced(at.left.clone(), at.name.clone(), Some(right))
    }
}

impl NameSet {
    /// How many names.
    pub(super) fn len(&self) -> usize {
        len(&self.root)
    }

    /// Whether `name` is in the set.
    pub(super) fn contains(&self, name: &str) -> bool {
        let mut at = &self.root;
        while let Some(node) = at {
            at = match name.cmp(&node.name) {
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Less => &node.left,
                std::cmp::Ordering::Greater => &node.right,
            };
        }
        false
    }

    /// The set with `name` in it, and whether it was there already.
    pub(super) fn with(&self, name: &str) -> (Self, bool) {
        if self.contains(name) {
            return (self.clone(), true);
        }
        let root = Some(inserted(&self.root, name));
        (Self { root }, false)
    }

    /// The names, bytewise.
    pub(super) fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        let mut pending: Vec<&Node> = Vec::new();
        let mut at = self.root.as_deref();
        std::iter::from_fn(move || {
            while let Some(node) = at {
                pending.push(node);
                at = node.left.as_deref();
            }
            let node = pending.pop()?;
            at = node.right.as_deref();
            Some(&*node.name)
        })
    }
}

/// The number of names only.
impl fmt::Debug for NameSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} names", self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_added_is_in_the_new_set_and_not_in_the_old() {
        let empty = NameSet::default();
        let (one, was) = empty.with("skinning");
        assert!(!was);
        assert!(one.contains("skinning"));
        assert!(!empty.contains("skinning"));
        assert_eq!((empty.len(), one.len()), (0, 1));
        let (two, was) = one.with("skinning_color");
        assert!(!was);
        assert!(two.contains("skinning") && two.contains("skinning_color"));
        assert!(!one.contains("skinning_color"));
        assert_eq!(two.len(), 2);
    }

    #[test]
    fn adding_a_name_twice_says_so_and_keeps_one() {
        let (one, _) = NameSet::default().with("a");
        let (again, was) = one.with("a");
        assert!(was);
        assert_eq!(again.len(), 1);
        assert_eq!(again.iter().collect::<Vec<_>>(), ["a"]);
    }

    #[test]
    fn names_iterate_bytewise_whatever_the_order_they_came_in() {
        let mut set = NameSet::default();
        for name in ["m", "b", "z", "a", "q", "c", "y", "B"] {
            set = set.with(name).0;
        }
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            ["B", "a", "b", "c", "m", "q", "y", "z"]
        );
        assert!(!set.contains("d"));
    }

    #[test]
    fn sets_hold_the_same_names_whatever_their_order_and_print_their_size() {
        // A set has no equality of its own: a material is compared as a link, and reading two
        // sets name by name is what a long chain cannot afford per link.
        let a = NameSet::default().with("x").0.with("y").0;
        let b = NameSet::default().with("y").0.with("x").0;
        assert!(a.iter().eq(b.iter()));
        assert!(
            !a.iter()
                .eq(NameSet::default().with("x").0.with("z").0.iter())
        );
        assert_eq!(format!("{a:?}"), "2 names");
    }

    #[test]
    fn a_hundred_thousand_names_stay_shallow_and_complete() {
        // Sorted input is the worst case of an unbalanced tree: it must neither overflow the
        // stack nor lose a name.
        let mut set = NameSet::default();
        for n in 0..100_000 {
            set = set.with(&format!("v{n:06}")).0;
        }
        assert_eq!(set.len(), 100_000);
        assert!(set.contains("v000000") && set.contains("v099999"));
        assert_eq!(set.iter().count(), 100_000);
        assert!(set.iter().zip(set.iter().skip(1)).all(|(a, b)| a < b));
    }
}
