//! `script::config` (families/script_config): the BDS `config/` folder and `server.properties`.
//!
//! Every row of the folder's fixtures is a server root under
//! `tests/fixtures/script/script_config/<row>/` with the findings it must give in
//! `expected.toml`; each mirrors a probe run on BDS 1.26.36.1.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "script")]

#[path = "script_config/common.rs"]
mod common;

#[path = "script_config/reader.rs"]
mod reader;

#[path = "script_config/net.rs"]
mod net;

#[path = "script_config/commands.rs"]
mod commands;

#[path = "script_config/stack.rs"]
mod stack;

#[path = "script_config/properties.rs"]
mod properties;

#[path = "script_config/write.rs"]
mod write;
