#![allow(clippy::pedantic, missing_docs)]
//! The atomic client entity family
//! (`bedrock_addon::families::atomic_client_entity::AtomicClientEntity`): every row of its
//! fixtures, the chain, round trips and the stack resolution.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "atomic_client_entity")]

#[path = "atomic_client_entity/common.rs"]
mod common;
#[path = "common/responder.rs"]
mod responder;

#[path = "../../families/client_entity/tests/client_entity/generate.rs"]
mod generate;
#[path = "atomic_client_entity/generated.rs"]
mod generated;
#[path = "atomic_client_entity/link.rs"]
mod link;
#[path = "atomic_client_entity/rows.rs"]
mod rows;
