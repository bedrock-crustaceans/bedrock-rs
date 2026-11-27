//! The newest `description` (format version 1.26.0 and up), and the document that holds it:
//! what client entities and attachables share, generic over the identifier kind and over the
//! members only one of them has.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

use crate::VisualSlots;
use crate::codes;
use crate::read::{Named, identifier, min_engine_version};

/// What a client entity's `description` has beside the shared members: nothing. A family's own
/// members are flattened into the description at its end, in every era.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Facet, Schema)]
pub struct NoOwn {}

/// `description` of a client entity or an attachable (format version 1.26.0 and up): the
/// identifier of kind `K`, `min_engine_version`, the shared members and the family's own `O`.
///
/// Every member is kept as written. Vanilla lower-cases the identifier, every slot name and
/// the values of `materials`, `geometry`, `animations` and `particle_effects` when it loads
/// them; the accessors of the families' documents give that reading. No member beyond these is
/// allowed: vanilla skips a file that has one.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Description<I: Named, O> {
    /// `identifier`: the name the document defines. Compared in lower case.
    #[facet(schema::read_with = identifier)]
    pub identifier: I,
    /// `min_engine_version`: a version string or `[major, minor, patch]`. It decides which
    /// definition of an identifier wins across packs, never which members are read; a text
    /// that is no version counts as 0.0.0.
    #[facet(schema::read_with = min_engine_version)]
    pub min_engine_version: Option<Value>,
    /// The members client entities and attachables share, at the description's own level
    /// (`materials` to `hide_armor`, in that order).
    #[facet(flatten)]
    pub slots: VisualSlots,
    /// The family's own members.
    #[facet(flatten)]
    pub own: O,
}

impl<I: Named, O: Default> Description<I, O> {
    /// A description of `identifier` with every other member absent.
    pub fn new(identifier: I) -> Self {
        Self {
            identifier,
            min_engine_version: None,
            slots: VisualSlots::default(),
            own: O::default(),
        }
    }
}

/// The object of a payload: `description` and nothing else.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
pub struct Payload<D> {
    /// `description`.
    pub description: D,
}

/// A document of a family: its payload holds the description, and a twin payload the family
/// does not read is any value.
pub trait Root<D>: Sized {
    /// The description, and whether the twin payload is written.
    fn into_parts(self) -> (D, bool);
}

/// The root of a client entity file: `format_version`, the client entity's payload, and the
/// attachable's, which is not read.
#[derive(Facet, Schema)]
pub struct ClientEntityRoot<D> {
    /// `format_version`: the pipeline's, read before.
    pub format_version: Option<Value>,
    /// `minecraft:client_entity`: a file without it has no entity definition.
    #[facet(rename = "minecraft:client_entity", schema::missing = codes::NO_ENTITY_DEFINITION)]
    pub client_entity: Payload<D>,
    /// `minecraft:attachable`: the twin, loaded by no one here.
    #[facet(rename = "minecraft:attachable")]
    pub attachable: Option<Value>,
}

impl<D> Root<D> for ClientEntityRoot<D> {
    fn into_parts(self) -> (D, bool) {
        (self.client_entity.description, self.attachable.is_some())
    }
}

/// The root of an attachable file: `format_version`, the attachable's payload, and the client
/// entity's, which is not read.
#[derive(Facet, Schema)]
pub struct AttachableRoot<D> {
    /// `format_version`: the pipeline's, read before.
    pub format_version: Option<Value>,
    /// `minecraft:attachable`: a file without it has no entity definition.
    #[facet(rename = "minecraft:attachable", schema::missing = codes::NO_ENTITY_DEFINITION)]
    pub attachable: Payload<D>,
    /// `minecraft:client_entity`: the twin, loaded by no one here.
    #[facet(rename = "minecraft:client_entity")]
    pub client_entity: Option<Value>,
}

impl<D> Root<D> for AttachableRoot<D> {
    fn into_parts(self) -> (D, bool) {
        (self.attachable.description, self.client_entity.is_some())
    }
}
