//! Version-agnostic helpers for the login exchange.
//!
//! Nothing here sequences packets. A server or client receives, matches its own protocol
//! version's packet variants, calls these helpers with plain data and sends the reply.

pub mod client;
pub mod handshake;
pub mod server;
pub mod settings;

pub use client::ClientIdentity;
pub use handshake::{begin_encryption, client_finish_encryption, server_begin_encryption};
pub use server::LoginOptions;
pub use settings::{NetworkSettings, check_protocol};
