//! What the texture families share: the files and image headers of
//! [`shapes::texture`](bedrock_addon_shared_shapes::texture), the lenient reader of
//! [`jsoncpp::lenient`](bedrock_addon_core::json::jsoncpp::lenient) and the texts of
//! [`diagnostic::texts`](bedrock_addon_core::diagnostic::texts), kept at these paths for their
//! users.

pub use bedrock_addon_shared_shapes::texture::{files, image};
pub mod jsoncpp;
pub mod texts;
