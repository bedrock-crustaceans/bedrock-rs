#![cfg_attr(docsrs, feature(doc_auto_cfg))] // docs.rs shows "Available on feature X" badges

pub use ::bedrock_shared as shared;

#[cfg(feature = "level")]
pub use ::bedrock_level as level;

#[cfg(feature = "auth")]
pub use ::bedrock_auth as auth;

#[cfg(feature = "form")]
pub use ::bedrock_form as form;

/// The sans-IO add-on toolkit: packs, stacks, loaders and every content family.
#[cfg(feature = "addon")]
pub use ::bedrock_addon as addon;

/// The std driver of the add-on toolkit: file system, threads and archives.
#[cfg(feature = "addon-std")]
pub use ::bedrock_addon_std as addon_std;

#[cfg(feature = "network")]
pub use ::bedrock_network as network;

#[cfg(feature = "protocol")]
pub mod protocol {
    pub use ::bedrock_network_protocol::*;
    pub use ::bedrock_network_core as codec;
    pub use ::bedrock_network_core::error;
}
