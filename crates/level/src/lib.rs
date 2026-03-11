pub mod biome;
pub mod bits;
pub mod error;
pub mod key;
pub mod packed;
pub mod player;
pub mod settings;
pub mod subchunk;
pub mod traits;
pub mod unpacked;

#[cfg(feature = "mojang-leveldb")]
pub mod mojang;
#[cfg(feature = "mojang-leveldb")]
pub use mojang::*;

// #[cfg(feature = "rusty-leveldb")]
// pub mod rusty;

// #[cfg(feature = "rusty-leveldb")]
// pub use rusty::*;

mod private {
    pub trait Sealed {}
}

pub trait PackingMethod: private::Sealed {
    const IS_PACKED: bool;
}

pub enum Unpacked {}

impl private::Sealed for Unpacked {}

impl PackingMethod for Unpacked {
    const IS_PACKED: bool = false;
}

pub enum Packed {}

impl private::Sealed for Packed {}

impl PackingMethod for Packed {
    const IS_PACKED: bool = true;
}

pub mod prelude {}
