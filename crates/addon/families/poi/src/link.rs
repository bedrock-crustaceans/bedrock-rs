//! The POI references the link pass resolves: the checker the stack set's link pass calls with
//! the block registry.

use super::Poi;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::id::{BlockId, Kind, kind};

/// What the POI references resolve against outside the group.
pub trait Targets {
    /// A block of the stack or a built-in one.
    fn block(&self, id: &BlockId) -> bool;
}

/// Every unresolved reference of the group, in the types' load order: a `blocks` entry of an
/// effective type that names no block is an `UnresolvedReference` error at the entry with the
/// registry's text and the name, as the registry logs it; it rejects nothing: the block is
/// skipped and the rest of the type is kept.
///
/// The game looks every `blocks` name up in the block registry with logging on; a miss is
/// logged as an error, adds no block permutation for the name and still registers the type. A
/// link-phase finding rejects no document, so the error grades as the game logs it.
pub fn link(poi: &Poi, targets: &dyn Targets) -> Diagnostics {
    let mut out = Diagnostics::default();
    for (_, document) in &poi.types {
        let origin = document.origin();
        for (i, block) in document.blocks.iter().enumerate() {
            if targets.block(block) {
                continue;
            }
            let mut at = JsonPath::new();
            at.push("minecraft:poi_block");
            at.push("blocks");
            at.push_index(i);
            let mut finding = Diagnostic::new(
                DiagnosticCode::UnresolvedReference {
                    text: Some(VanillaText::Formatted {
                        template: "BlockTypeRegistry::lookupByName() failed to find block \"{}\"",
                        args: vec![block.as_str().to_owned()],
                    }),
                },
                Severity::Error,
                Phase::Link,
                Location::new(
                    Some(origin.pack.clone()),
                    Some(origin.path.clone()),
                    at,
                    None,
                ),
            );
            finding.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: <kind::Block as Kind>::NAME,
                id: block.as_str().to_owned(),
                hint: None,
            }));
            out.push(finding);
        }
    }
    out
}
