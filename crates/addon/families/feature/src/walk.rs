//! The walk over a feature's Molang fields: the expressions of world-generation members and the
//! `tags` of every block descriptor. Each struct lists the members that can
//! hold one; [`walk!`] writes the impls.

use super::values::{BlockList, WeightedBlock};
use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangFailure, MolangFieldSpec, MolangVersionSource, MolangVisitor,
    OpsRule, ReaderKind, Step,
};
use bedrock_addon_core::shared::{BlockDescriptor, BlockSpecifier};

/// A descriptor's `tags`: the pack's Molang version, the `tags` query set; vanilla logs the
/// parser's lines at level 3 and keeps the descriptor, so a rejection is a kept error.
pub(crate) const DESCRIPTOR_TAGS: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    version: MolangVersionSource::Pack,
    admission: Admission::Tags,
    ops: OpsRule::All,
    empty: EmptyRule::Compile,
    on_failure: MolangFailure::KeepLogged,
    error_severity: Severity::Error,
    error_kept: true,
};

/// A typed world-generation member (`width_modifier`, `condition`): the pack's
/// version, the `world_gen` query set; a failed compile fails the schema, so the feature is not
/// registered.
pub(crate) const WORLD_GEN: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    version: MolangVersionSource::Pack,
    admission: Admission::WorldGen,
    ops: OpsRule::All,
    empty: EmptyRule::Compile,
    on_failure: MolangFailure::RejectDocument,
    error_severity: Severity::Error,
    error_kept: false,
};

/// A scatter member of the `distribution` (format 1.21.10 and later): read leniently, compiled
/// at the pack's version even in the object form, the `world_gen` query set. A failed compile
/// is logged at level 3 and the feature loads, so it is a kept error: the client converts each
/// member and goes on, with no failure path. An empty string is no expression.
pub(crate) const SCATTER: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::LenientVersioned,
    version: MolangVersionSource::PackIgnoringObject,
    admission: Admission::WorldGen,
    ops: OpsRule::All,
    empty: EmptyRule::NoExpression,
    on_failure: MolangFailure::KeepLogged,
    error_severity: Severity::Error,
    error_kept: true,
};

/// A scatter member of the root layout below format 1.21.10: the legacy schema's typed
/// children, which take no object form. A failed compile is a kept error as for [`SCATTER`];
/// whether the client instead refuses the feature for a text its parser rejects is not known.
pub(crate) const SCATTER_LEGACY: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    ..SCATTER
};

/// A member that may hold Molang fields: block descriptors (their `tags`) or expressions.
pub(crate) trait Walk {
    /// Hands every Molang field of the value to `v`, at its path below the current one.
    fn walk(&mut self, v: &mut MolangVisitor<'_>);
}

impl Walk for BlockDescriptor {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = self {
            v.field("tags", &DESCRIPTOR_TAGS, tags);
        }
    }
}

impl Walk for BlockList {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        match self {
            BlockList::One(one) => one.walk(v),
            BlockList::Many(many) => many.walk(v),
        }
    }
}

impl Walk for WeightedBlock {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        v.enter(Step::Index(0));
        self.block.walk(v);
        v.leave();
    }
}

impl<T: Walk> Walk for Option<T> {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        if let Some(inner) = self {
            inner.walk(v);
        }
    }
}

impl<T: Walk> Walk for Vec<T> {
    fn walk(&mut self, v: &mut MolangVisitor<'_>) {
        for (index, inner) in self.iter_mut().enumerate() {
            v.enter(Step::Index(index));
            inner.walk(v);
            v.leave();
        }
    }
}

/// Steps into `key` and walks `value` there.
pub(crate) fn member<T: Walk + ?Sized>(v: &mut MolangVisitor<'_>, key: &str, value: &mut T) {
    v.enter(Step::Key(key));
    value.walk(v);
    v.leave();
}

/// `impl Walk` (and, with `molang`, `impl MolangFields`) for a struct, from the members that can
/// hold Molang fields: `"json key" => field`. The same members are the ones that can hold a
/// reference to a block; a second list, after `+`, has the members that hold a reference of
/// another kind and no Molang field (`impl Refer`, for the link pass).
macro_rules! walk {
    ($ty:ty { $($key:literal => $field:ident),* $(,)? }) => {
        walk!($ty { $($key => $field),* } + {});
    };
    // The second list and the `Refer` impl
    ($ty:ty { $($key:literal => $field:ident),* $(,)? } + { $($more_key:literal => $more:ident),* $(,)? }) => {
        impl $crate::walk::Walk for $ty {
            #[allow(unused_variables)]
            fn walk(&mut self, v: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                $( $crate::walk::member(v, $key, &mut self.$field); )*
            }
        }
        impl $crate::walk::Refer for $ty {
            #[allow(unused_variables)]
            fn refer<'a>(&'a self, out: &mut $crate::walk::References<'a>) {
                $( out.member($key, &self.$field); )*
                $( out.member($more_key, &self.$more); )*
            }
        }
    };
    (fields $ty:ty { $($key:literal => $field:ident),* $(,)? }) => {
        walk!(fields $ty { $($key => $field),* } + {});
    };
    (fields $ty:ty { $($key:literal => $field:ident),* $(,)? } + { $($more_key:literal => $more:ident),* $(,)? }) => {
        walk!($ty { $($key => $field),* } + { $($more_key => $more),* });
        impl ::bedrock_addon_core::molang::MolangFields for $ty {
            fn visit_molang(&mut self, v: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                $crate::walk::Walk::walk(self, v);
            }
        }
    };
}

pub(crate) use walk;

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

impl<T: Refer> Refer for Vec<T> {
    fn refer<'a>(&'a self, out: &mut References<'a>) {
        for (index, inner) in self.iter().enumerate() {
            out.element(index, inner);
        }
    }
}

/// Every reference of `feature` to another feature or to a block, each with the JSON path of
/// its member in the file (`minecraft:<type>/<member>/…`).
#[must_use]
pub fn references(feature: &super::Feature) -> Vec<(JsonPath, Reference<'_>)> {
    let mut at = JsonPath::new();
    at.push(feature.type_key());
    let mut out = References {
        at,
        found: Vec::new(),
    };
    feature.refer(&mut out);
    out.found
}
