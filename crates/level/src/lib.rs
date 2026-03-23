pub use vek;

pub mod biome;
pub mod bits;
pub mod error;
pub mod greedy;
pub mod key;
pub mod lazy;
pub mod player;
pub mod settings;
pub mod subchunk;
pub mod traits;

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

pub trait UnpackingMethod: private::Sealed {
    const IS_LAZY: bool;
}

pub enum Greedy {}

impl private::Sealed for Greedy {}

impl UnpackingMethod for Greedy {
    const IS_LAZY: bool = false;
}

pub enum Lazy {}

impl private::Sealed for Lazy {}

impl UnpackingMethod for Lazy {
    const IS_LAZY: bool = true;
}

pub mod prelude {}
