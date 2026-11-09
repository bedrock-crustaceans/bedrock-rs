//! The point of interest a Brain memory sensor looks for: a POI type, a POI tag with a
//! document, or a tag a type carries (the game makes such a tag on its first use). A name
//! that is none of these gets no line from the sensor ("Failed to load POI Type" is the type
//! loader's): the game writes no content-log line for this, so the crate reports it as an
//! information.
use std::collections::HashSet;

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::Id;

use super::walk::Entities;
use crate::families::entity::links::{Reference, ReferenceKind};
use crate::pack::link::prelude::*;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &Entities<'_>) {
    // The tags the types carry, gathered on the first name that is no type and no tag
    // document.
    // A tag only a type names is a leaf tag.
    let mut carried: Option<HashSet<&str>> = None;
    entities.each(ReferenceKind::Poi, &mut |definition, at, found| {
        let Reference::Poi(name) = found.reference else {
            return;
        };
        let Target::Missing(hint) = cx.poi_type(&Id::parse(name)) else {
            return;
        };
        if !cx.poi_tag(&Id::parse(name)).is_missing() {
            return;
        }
        let carried = carried.get_or_insert_with(|| {
            let types = &cx.behavior().poi().types;
            types
                .iter()
                .flat_map(|(_, document)| document.value().tags.iter().flatten())
                .map(Id::as_str)
                .collect()
        });
        if carried.contains(name) {
            return;
        }
        let at = At::new(definition.origin, at, Severity::Info);
        out.push(unresolved(at, kinds::POI_TYPE, name, hint));
    });
}
