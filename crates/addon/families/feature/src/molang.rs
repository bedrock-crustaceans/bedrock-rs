//! The Molang specs of a feature's members (the expressions of world-generation members and the
//! `tags` of every block descriptor, which the structs' `schema::molang` name), and the list of
//! the references a feature makes to other definitions, for the link pass.

use super::values::{BlockList, WeightedBlock};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::molang::{MolangFieldSpec, MolangVersionSource, ReaderKind};
use bedrock_addon_shared_shapes::{BlockDescriptor, BlockSpecifier};

/// A descriptor's `tags`: the pack's Molang version, the `tags` query set; vanilla logs the
/// parser's lines at level 3 and keeps the descriptor, so a rejection is a kept error.
pub(crate) const DESCRIPTOR_TAGS: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::SchemaValidated)
    .tags()
    .compile_empty();

/// A typed world-generation member (`width_modifier`, `condition`): the pack's
/// version, the `world_gen` query set; a failed compile fails the schema, so the feature is not
/// registered.
pub(crate) const WORLD_GEN: MolangFieldSpec = MolangFieldSpec::REJECTING
    .reader(ReaderKind::SchemaValidated)
    .world_gen()
    .compile_empty();

/// A scatter member of the `distribution` (format 1.21.10 and later): read leniently, compiled
/// at the pack's version even in the object form, the `world_gen` query set. A failed compile
/// is logged at level 3 and the feature loads, so it is a kept error: the client converts each
/// member and goes on, with no failure path. An empty string is no expression.
pub(crate) const SCATTER: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::LenientVersioned)
    .version(MolangVersionSource::PackIgnoringObject)
    .world_gen();

/// A scatter member of the root layout below format 1.21.10: the legacy schema's typed
/// children, which take no object form. A failed compile is a kept error as for [`SCATTER`];
/// whether the client instead refuses the feature for a text its parser rejects is not known.
pub(crate) const SCATTER_LEGACY: MolangFieldSpec = SCATTER.reader(ReaderKind::SchemaValidated);

/// `impl Refer` for a struct, from the members that can hold a reference to another
/// definition (a feature, a block, the blocks a tag expression matches): `"json key" => field`,
/// in document order.
macro_rules! refers {
    ($ty:ty { $($key:literal => $field:ident),* $(,)? }) => {
        impl $crate::molang::Refer for $ty {
            #[allow(unused_variables)]
            fn refer<'a>(&'a self, out: &mut $crate::molang::References<'a>) {
                $( out.member($key, &self.$field); )*
            }
        }
    };
}

pub(crate) use refers;

// ---- references, for the link pass -------------------------------------------------------------

/// What a member of a feature names in another definition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reference<'a> {
    /// Another feature.
    Feature(&'a FeatureId),
    /// A block, or the blocks a tag expression matches.
    Descriptor(&'a BlockDescriptor),
    /// A block.
    Specifier(&'a BlockSpecifier),
}

/// The references of one feature document, each with the JSON path of its member from the
/// document's root, in document order of the struct.
pub(crate) struct References<'a> {
    at: JsonPath,
    found: Vec<(JsonPath, Reference<'a>)>,
}

impl<'a> References<'a> {
    /// Steps into `key` and collects the references of `value` there.
    pub(crate) fn member<T: Refer + ?Sized>(&mut self, key: &str, value: &'a T) {
        self.at.push(key);
        value.refer(self);
        self.at.pop();
    }

    /// Steps into the element `index` and collects the references of `value` there.
    pub(crate) fn element<T: Refer + ?Sized>(&mut self, index: usize, value: &'a T) {
        self.at.push_index(index);
        value.refer(self);
        self.at.pop();
    }

    /// A reference at the current path.
    fn here(&mut self, reference: Reference<'a>) {
        self.found.push((self.at.clone(), reference));
    }
}

/// A member that may hold references to other definitions.
pub(crate) trait Refer {
    /// Hands every reference of the value to `out`, at its path below the current one.
    fn refer<'a>(&'a self, out: &mut References<'a>);
}

impl Refer for FeatureId {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        out.here(Reference::Feature(self));
    }
}

impl Refer for BlockDescriptor {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        out.here(Reference::Descriptor(self));
    }
}

impl Refer for BlockSpecifier {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        out.here(Reference::Specifier(self));
    }
}

impl Refer for BlockList {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        match self {
            BlockList::One(one) => one.refer(out),
            BlockList::Many(many) => many.refer(out),
        }
    }
}

impl Refer for WeightedBlock {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        out.element(0, &self.block);
    }
}

impl<T: Refer> Refer for Option<T> {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        if let Some(inner) = self {
            inner.refer(out);
        }
    }
}

impl<T: Refer> Refer for Box<T> {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        (**self).refer(out);
    }
}

impl<T: Refer> Refer for Vec<T> {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        for (index, inner) in self.iter().enumerate() {
            out.element(index, inner);
        }
    }
}

/// Every reference of `feature` to another feature or to a block, each with the JSON path of
/// its member in the file (`minecraft:<type>/<member>/…`).
pub fn references(feature: &super::Feature) -> Vec<(JsonPath, Reference<'_>)> {
    let at = JsonPath::new().child(feature.type_key());
    let mut out = References {
        at,
        found: Vec::new(),
    };
    feature.refer(&mut out);
    out.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::molang::{Admission, EmptyRule, MolangFailure, OpsRule};

    /// Each spec, field by field, as the game reads the member.
    #[test]
    fn the_molang_specs_keep_their_fields() {
        use Admission::{Tags, WorldGen};
        use EmptyRule::{Compile, NoExpression};
        use MolangFailure::{KeepLogged, RejectDocument};
        use MolangVersionSource::{Pack, PackIgnoringObject};
        use ReaderKind::{LenientVersioned, SchemaValidated};
        let spec = |reader, version, admission, empty, on_failure, error_kept| MolangFieldSpec {
            reader,
            version,
            admission,
            ops: OpsRule::All,
            empty,
            on_failure,
            error_severity: Severity::Error,
            error_kept,
        };
        assert_eq!(
            DESCRIPTOR_TAGS,
            spec(SchemaValidated, Pack, Tags, Compile, KeepLogged, true)
        );
        assert_eq!(
            WORLD_GEN,
            spec(
                SchemaValidated,
                Pack,
                WorldGen,
                Compile,
                RejectDocument,
                false
            )
        );
        assert_eq!(
            SCATTER,
            spec(
                LenientVersioned,
                PackIgnoringObject,
                WorldGen,
                NoExpression,
                KeepLogged,
                true
            )
        );
        assert_eq!(
            SCATTER_LEGACY,
            spec(
                SchemaValidated,
                PackIgnoringObject,
                WorldGen,
                NoExpression,
                KeepLogged,
                true
            )
        );
    }
}
