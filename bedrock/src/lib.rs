#![allow(unused_imports)]

pub mod core {
    pub use ::bedrock_shared::*;
}

#[cfg(feature = "level")]
pub mod level {
    pub use ::bedrock_level::*;
}

#[cfg(feature = "auth")]
pub mod auth {
    pub use ::bedrock_auth::*;
}

#[cfg(feature = "protocol")]
pub mod protocol {
    pub use ::bedrock_protocol::*;
    pub use ::bedrock_protocol_core::*;

    pub mod error {
        pub use ::bedrock_protocol_core::error::*;
    }
}

#[cfg(feature = "network")]
pub mod network {
    pub use ::bedrock_network::*;
    pub mod error {
        pub use ::bedrock_network::error::*;
    }
}

#[cfg(feature = "form")]
pub mod form {
    pub use ::bedrock_form::*;
}

/// The sans-IO add-on toolkit (`bedrock_addon`): packs, stacks, loaders and every content family.
#[cfg(feature = "addon")]
pub use ::bedrock_addon as addon;

/// The std driver of the add-on toolkit (`bedrock_addon_std`): file system, threads and archives.
#[cfg(feature = "addon-std")]
pub use ::bedrock_addon_std as addon_std;
