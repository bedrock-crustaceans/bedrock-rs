//! The entity's Molang fields: the specs of the description's expressions and the walk of
//! Molang pass 2 over the newest struct and the beta one.

use bedrock_addon_core::molang::{
    MolangFailure, MolangFieldSpec, MolangFields, MolangVisitor, OpsRule, PROPERTY_DEFAULT_QUERIES,
    Step,
};

use super::{AnimateEntry, Beta, Entity, PropertyDefault};

/// `description.scripts.animate[].<name>`: a Molang string at the pack's version; a rejected
/// expression is kept and evaluates to 0.
// The game logs the rejection at the error level, stores the script anyway and keeps the
// entity: a kept error.
pub(crate) const ANIMATE: MolangFieldSpec = MolangFieldSpec::KEPT.compile_empty();

/// `description.properties.<name>.default` (a string that is no enum value): a Molang string
/// at the pack's version that may call only `query.had_component_group` and assigns nothing;
/// a rejected expression drops the entity's property group.
// The game logs "Error loading Actor Properties" at the error level and keeps the entity
// without its properties: a kept error.
pub(crate) const PROPERTY_DEFAULT: MolangFieldSpec = MolangFieldSpec::KEPT
    .only(PROPERTY_DEFAULT_QUERIES)
    .ops(OpsRule::NoAssignments)
    .compile_empty()
    .on_failure(MolangFailure::DropEntry);

impl MolangFields for Entity {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter(Step::from("description"));
        visitor.enter(Step::from("properties"));
        for (name, property) in &mut self.description.properties {
            if let Some(PropertyDefault::Expression(source)) = &mut property.default {
                visitor.enter(Step::from(name.as_str()));
                visitor.field("default", &PROPERTY_DEFAULT, source);
                visitor.leave();
            }
        }
        visitor.leave();
        if let Some(scripts) = &mut self.description.scripts {
            visitor.enter(Step::from("scripts"));
            visitor.enter(Step::from("animate"));
            // Each entry at the index the file has it at.
            let places: Vec<usize> = (0..scripts.animate.len())
                .map(|index| scripts.place(index))
                .collect();
            for (place, entry) in places.into_iter().zip(&mut scripts.animate) {
                if let AnimateEntry::Conditional(members) = entry {
                    visitor.field(place, &ANIMATE, members);
                }
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
        visitor.nested("components", &mut self.components);
        visitor.enter(Step::from("component_groups"));
        for (name, group) in &mut self.component_groups {
            visitor.nested(name.as_str(), &mut group.components);
        }
        visitor.leave();
    }
}

impl MolangFields for Beta {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        self.0.visit_molang(visitor);
    }
}
