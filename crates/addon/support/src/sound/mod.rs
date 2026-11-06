//! What the sound families share: core's [`jsoncpp::fork`](bedrock_addon_core::json::jsoncpp::fork),
//! [`load::sound`](bedrock_addon_core::load::sound) and
//! [`kit::sound`](bedrock_addon_core::json::kit::sound), kept at these paths for their users.

pub mod fork;
pub use bedrock_addon_core::json::kit::sound::{read, report, write};
pub use bedrock_addon_core::load::sound as merge;
