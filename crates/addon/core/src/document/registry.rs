//! The iterator types of [`Registry`](super::Registry): `Layered`'s,
//! named by the definition.

use super::layered;
use super::{Definition, Document};
use crate::id::Id;

/// `(id, effective document)` in load order.
pub type Iter<'a, T> = layered::Iter<'a, Id<<T as Definition>::Kind>, Document<T>>;

/// Every definition of one id, effective first.
pub type Layers<'a, T> = layered::Layers<'a, Document<T>>;
