//! The entity documents of a pass as the rules read them: every entity definition of the
//! behaviour stack, the references the entity family finds in each
//! ([`Entity::references`]), and the place in its file of each.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, Origin};

use crate::families::entity::Entity;
use crate::families::entity::component::EntityComponent;
use bedrock_addon_core::versioning::Family;

use crate::families::entity::links::{ReferenceKind, Referenced, References};
use crate::pack::link::prelude::*;

/// The payload key of an entity document.
pub(super) const PAYLOAD: &str = <Entity as Family>::PAYLOAD_KEY;

/// One entity definition of the stack: a stable document or a beta one.
#[derive(Debug)]
pub(super) struct Definition<'a> {
    /// Where the document is.
    pub(super) origin: &'a Origin,
    /// The entity.
    pub(super) entity: &'a Entity,
    /// The document's findings: they give the place in the file of a member the entity
    /// holds (`Diagnostics::place_of`).
    pub(super) places: &'a Diagnostics,
}

/// Every effective entity definition of the behaviour stack, stable then beta, in load order.
pub(super) fn definitions<'a>(cx: &LinkCx<'a>) -> Vec<Definition<'a>> {
    let registry = cx.behavior().entities();
    let mut all: Vec<Definition<'a>> = registry
        .iter()
        .map(|(_, document)| Definition {
            origin: document.origin(),
            entity: document.value(),
            places: document.diagnostics(),
        })
        .collect();
    if let Some(beta) = registry.beta() {
        all.extend(beta.iter().map(|(_, document)| Definition {
            origin: document.origin(),
            entity: &document.value().0,
            places: document.diagnostics(),
        }));
    }
    all
}

/// The sections of an entity that hold components: `components` and each component group,
/// with the JSON path of each.
#[cfg_attr(
    not(feature = "block"),
    allow(dead_code, reason = "the typed goal of the block rule reads them")
)]
pub(super) fn sections(
    entity: &Entity,
) -> impl Iterator<Item = (JsonPath, &ComponentMap<EntityComponent>)> + '_ {
    let own = std::iter::once((path(&[PAYLOAD, "components"]), &entity.components));
    let groups = entity.component_groups.iter().map(|(name, group)| {
        (
            path(&[PAYLOAD, "component_groups", name]),
            &group.components,
        )
    });
    own.chain(groups)
}

/// What the rules over entity documents read, built once per pass.
#[derive(Debug)]
pub(super) struct Entities<'a> {
    /// The definitions.
    pub(super) definitions: Vec<Definition<'a>>,
    /// The references of each definition, in the order of [`Entities::definitions`]: the
    /// typed components of each are written back to JSON once per pass.
    references: Vec<References<'a>>,
}

impl<'a> Entities<'a> {
    /// The entities of the behaviour stack of `cx`.
    pub(super) fn new(cx: &LinkCx<'a>) -> Self {
        let definitions = definitions(cx);
        let references = definitions
            .iter()
            .map(|definition| definition.entity.references())
            .collect();
        Self {
            definitions,
            references,
        }
    }

    /// Calls `found` with every member that holds a reference of `kind`, in document order:
    /// the definition, the member's path in its file and the member as the entity holds it.
    ///
    /// The members are walked in the shape the entity holds, and each is named where the
    /// file has it: the lenient conversion of a document below a component's typed version
    /// leaves a list element out that the typed member does not take, a step moves a member,
    /// and the document keeps where each member it holds is in the file.
    pub(super) fn each(
        &self,
        kind: ReferenceKind,
        found: &mut dyn FnMut(&Definition<'a>, JsonPath, &Referenced<'_>),
    ) {
        for (definition, references) in self.definitions.iter().zip(&self.references) {
            for referenced in references.of(kind) {
                found(
                    definition,
                    definition.places.place_of(&referenced.at),
                    &referenced,
                );
            }
        }
    }

    /// Calls `found` with the particle of every `emit_particle` response of every
    /// definition's events, in document order, at its path in the entity.
    #[cfg_attr(
        not(feature = "particle"),
        allow(dead_code, reason = "the emitted particles are the particle rule's")
    )]
    pub(super) fn emitted_particles(
        &self,
        found: &mut dyn FnMut(&Definition<'a>, &Referenced<'_>),
    ) {
        for (definition, references) in self.definitions.iter().zip(&self.references) {
            for referenced in references.emitted_particles() {
                found(definition, &referenced);
            }
        }
    }
}

/// The path of `keys`.
pub(super) fn path(keys: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for key in keys {
        path.push(key);
    }
    path
}
