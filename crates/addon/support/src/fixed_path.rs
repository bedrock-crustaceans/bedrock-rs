//! What the fixed-path files of the menu texts, the fonts and the UI images' side files share:
//! core's [`kit::fixed_path`](bedrock_addon_core::json::kit::fixed_path), its findings'
//! [`builders`](bedrock_addon_core::diagnostic::builders) and its writers'
//! [`build`](bedrock_addon_core::json::build), kept at this path for their users.

pub use bedrock_addon_core::diagnostic::builders::{
    file_finding, invalid, invalid_text, malformed,
};
pub use bedrock_addon_core::json::build::{Object, array, json_string, json_text};
pub use bedrock_addon_core::json::kit::fixed_path::{
    Authored, as_int, as_real, below, iterated, kept_unknown, raw_member, reader, testing, to_text,
};
