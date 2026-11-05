//! Flipbooks: `textures/flipbook_textures.json`, the animated tiles of the terrain atlas. The
//! file of every pack is merged entry by entry from the lowest pack up
//! and the merged array is read once.
//! What a flipbook says of the terrain atlas and of its image is a link rule: the family
//! knows neither.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod flipbook;
mod load;
#[allow(
    dead_code,
    reason = "the link rule of the flipbooks, the terrain atlas and the texture files quotes them"
)]
#[doc(hidden)]
pub mod texts;

pub use flipbook::{Flipbook, FlipbookKey, Flipbooks};

#[doc(hidden)]
pub use load::LOADER;
