//! The entity family (`bedrock_addon::families::entity`): document, version chain, steps and
//! validation.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "entity/actor_identifiers.rs"]
mod actor_identifiers;

#[path = "entity/callbacks.rs"]
mod callbacks;

#[path = "entity/chain.rs"]
mod chain;

#[path = "entity/common.rs"]
mod common;

#[path = "entity/document.rs"]
mod document;

#[path = "entity/fullchain.rs"]
mod fullchain;

#[path = "entity/goal/mod.rs"]
mod goal;

#[path = "entity/jsonutil.rs"]
mod jsonutil;

#[path = "entity/molang.rs"]
mod molang;

#[path = "entity/puv_beta.rs"]
mod puv_beta;

#[path = "entity/replaced.rs"]
mod replaced;

#[path = "entity/steps.rs"]
mod steps;

#[path = "entity/table.rs"]
mod table;

#[path = "entity/validation.rs"]
mod validation;

#[path = "entity/component/mod.rs"]
mod component;

#[path = "entity/engine.rs"]
mod engine;
