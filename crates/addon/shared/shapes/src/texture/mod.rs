//! What the texture families and the link pass share of a stack's textures: the files of a
//! stack as the texture rules see them ([`files`]) and the image headers ([`image`]). No byte of
//! an image is decoded.

pub mod files;
pub mod image;
