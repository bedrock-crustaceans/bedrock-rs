//! `blocks.json` as written in every format version.
//!
//! There is no chain: [`v1_1_0::BlocksJson`] reads the file at every version, and the version
//! only decides how legacy block names are read.

pub mod v1_1_0;
