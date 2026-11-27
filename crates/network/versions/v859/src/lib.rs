//! `r/21_u12`

pub mod enums;
pub mod packets;
pub mod types;

#[cfg(feature = "version")]
mod generated;
#[cfg(feature = "version")]
pub use generated::*;
