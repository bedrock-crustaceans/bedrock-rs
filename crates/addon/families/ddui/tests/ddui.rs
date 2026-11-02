#![allow(clippy::pedantic, missing_docs)]
//! Data-driven UI (`ddui/root`, `ddui/composition`, `ddui/styling`): the rows of its fixtures,
//! the loader's rules, the control catalogue, the chains, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "ddui/catalogue.rs"]
mod catalogue;

#[path = "ddui/chains.rs"]
mod chains;
