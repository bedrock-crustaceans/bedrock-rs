//! The iterator types of [`Files`](super::Files): `Layered`'s,
//! named by the content.

use super::layered;
use super::{Document, PackPath};

/// `(path, effective document)`.
pub type Iter<'a, T> = layered::Iter<'a, PackPath, Document<T>>;

/// Every copy of one path, the effective one first
/// ([`Files::all_layers`](super::Files::all_layers)).
pub type Layers<'a, T> = layered::Layers<'a, Document<T>>;
