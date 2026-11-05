//! Data-driven UI (`ddui/root`, `ddui/composition`, `ddui/styling`): the rows of its fixtures,
//! the loader's rules, the control catalogue, the chains, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "ddui")]

#[path = "common/responder.rs"]
mod responder;

#[path = "ddui/common.rs"]
mod common;

#[path = "ddui/controls.rs"]
mod controls;

#[path = "ddui/envelope.rs"]
mod envelope;

#[path = "ddui/extension.rs"]
mod extension;

#[path = "ddui/generated.rs"]
mod generated;

#[path = "ddui/namespace.rs"]
mod namespace;

#[path = "ddui/round_trip.rs"]
mod round_trip;

#[path = "ddui/stack.rs"]
mod stack;

#[path = "ddui/style.rs"]
mod style;

#[path = "ddui/located.rs"]
mod located;
