//! The default biome of a dimension (`minecraft:default_biome.biome`, from format
//! 1.26.50).

use crate::diagnostic::{JsonPath, Severity};
use crate::pack::link::prelude::*;

/// A default biome that is no biome of the stack and none of the game's is an
/// `UnresolvedReference` information: a lint, since whether the game checks the name is not
/// known. A dimension that names none takes `minecraft:ocean` and references nothing.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    for (_, document) in cx.behavior().dimensions() {
        let Some(default) = &document.value().components.default_biome else {
            continue;
        };
        if let Target::Missing(hint) = cx.biome(&default.biome) {
            let mut path = JsonPath::new();
            for key in [
                "minecraft:dimension",
                "components",
                "minecraft:default_biome",
                "biome",
            ] {
                path.push(key);
            }
            let at = At::new(document.origin(), path, Severity::Info);
            out.push(unresolved(at, kinds::BIOME, default.biome.as_str(), hint));
        }
    }
}
