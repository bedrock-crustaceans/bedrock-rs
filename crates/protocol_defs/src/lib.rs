pub mod schemas;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./defs"]
pub struct ProtocolDefs;