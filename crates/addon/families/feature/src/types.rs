//! The `description` every feature type shares, with its constructor. [`Feature`](crate::Feature),
//! one variant per feature type, is declared by its rows in `family.rs`.

use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::schema::Schema;

/// A feature's `description`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, facet::Facet, Schema)]
pub struct Description {
    /// The feature's identifier, `namespace:name` (JSON `identifier`). References to the
    /// feature use it; the file's name and folder do not matter.
    pub identifier: FeatureId,
}

impl Description {
    /// The description of the feature `identifier`.
    pub fn new(identifier: FeatureId) -> Self {
        Self { identifier }
    }
}
