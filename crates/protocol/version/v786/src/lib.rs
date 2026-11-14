//! `r/21_u7`

pub mod enums;
pub mod packets;

#[cfg(feature = "version")]
mod generated;
#[cfg(feature = "version")]
pub use generated::*;
