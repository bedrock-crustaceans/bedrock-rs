pub mod biome;
pub mod bits;
pub mod color;
pub mod db;
pub mod error;
pub mod greedy;
pub mod height_map;
pub mod iter;
pub mod key;
pub mod lazy;
pub mod player;
pub mod provider;
pub mod serde_helpers;
pub mod settings;
pub mod subchunk;
pub mod traits;
pub mod types;
pub mod version;

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
