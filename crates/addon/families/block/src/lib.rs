//! Blocks: `Block`, `Permutation`, `StateValues`, `Traits`, `component::*`, `history::*`.
//!
//! Custom blocks: `blocks/**/*.json` of a behaviour pack (`minecraft:block`). `Block` is the
//! newest document (format 1.26.20); older files are upgraded when they load, and `history` has
//! the older shapes.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod component;
pub mod history;

pub use states::{
    Connection, MultiBlock, PlacementDirection, PlacementPosition, StateValues, Traits,
};

mod read;
mod states;
mod types;
mod validate;
mod write;

#[cfg(feature = "nbt")]
mod network;

pub use types::{Block, Description, MaterialType, Permutation, VanillaBlockData};

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{
    Contribution, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Origin, Registry, default_path,
};
use bedrock_addon_core::id::{BlockId, kind};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BeyondWidth, ComponentSection, ComponentTable, DocumentRead, Envelope, EnvelopeCheck, Family,
    InputKind, Loaded, Reading, Scalars, StructIndex, ValidateCx, VersionChain,
};
use component::BlockComponent;
use history::{BlockFamily, BlockHistory};
use read::{CEREAL, read_file};

/// The floor: below 1.16.0 the game's upgrades discard most components, so the crate rejects the
/// file.
const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);

impl Family for BlockFamily {
    const NAME: &'static str = "block";
    const PAYLOAD_KEY: &'static str = "minecraft:block";
    const FLOOR: FormatVersion = FLOOR;
    type Latest = Block;
    type Beta = NoBeta<kind::Block>;
    type Component = BlockComponent;
    type History = BlockHistory;
    const ENVELOPE: Envelope = Envelope {
        // Every block file is read as a document whose version path and size limits are the
        // file's own.
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with an information at it.
        clamps_beyond_double: true,
        // Rule 11 reads a component written twice ("Duplicate component … found"); every other
        // member written twice keeps its last value.
        duplicates_kept_in: &["components"],
        // The game reads a block file as a versioned document: the raw text below 1.26.20, exactly
        // two members from it.
        check: EnvelopeCheck::From(CEREAL),
        ..Envelope::DEFAULT
    };

    // The v1_26_20 and v1_26_60 slices are strict under rule N2: an unknown member or component
    // of a cereal document rejects it. The legacy slice is the hand-written parser's (the crate's
    // legacy reader), which takes `null` itself and does not reject unknown members: its findings
    // stay as they are. A cereal document's scalars are the strict reader's; the legacy component
    // parser reads every cereal-registered description with the non-strict schema reader (a bool
    // and a number read as each other, an integer by its low bits, a real by its truncation).
    // Both refuse a number beyond its float's width.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        reading_of(is_cereal(index))
    }

    fn components() -> &'static ComponentTable<Self> {
        BlockComponent::TABLE
    }

    fn component_sections(v: &mut Block) -> Vec<ComponentSection<'_, BlockComponent>> {
        let mut sections = Vec::with_capacity(1 + v.permutations.len());
        let at = JsonPath::new().child("components");
        sections.push(ComponentSection::new(at, false, &mut v.components));
        for (index, permutation) in v.permutations.iter_mut().enumerate() {
            let at = JsonPath::new()
                .child("permutations")
                .index(index)
                .child("components");
            sections.push(ComponentSection::new(at, true, &mut permutation.components));
        }
        sections
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(block) = v {
            validate::validate(block, cx);
        }
    }
}

/// Whether `index` names a cereal struct (1.26.20 and later).
/// The reading of a cereal slice or a legacy one ([`Family::reading`]); the components of a
/// payload are read by it too (`read::section`), whichever slice the payload is read for.
pub(crate) fn reading_of(cereal: bool) -> Reading {
    if cereal {
        Reading::DOCUMENT.with_scalars(Scalars::Strict)
    } else {
        Reading::OWN.with_scalars(Scalars::NonStrict)
    }
    .with_beyond_width(BeyondWidth::Error)
}

fn is_cereal(index: StructIndex) -> bool {
    match index {
        StructIndex::Stable(i) => <BlockHistory as VersionChain<BlockFamily>>::VERSIONS
            .get(i)
            .is_some_and(|version| *version >= CEREAL),
        StructIndex::Beta => false,
    }
}

impl JsonDocument for Block {
    type Family = BlockFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// `read::read_file`: `parse_document`'s steps, with a legacy file's findings at the payload.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<BlockFamily> {
        read_file(origin, bytes, cx)
    }

    /// The document of the format the block is written in (`Block::document`): a block the
    /// 1.26.20 format cannot hold is written as a legacy document at its own version, whose
    /// payload is not [`VersionStruct::write`](bedrock_addon_core::versioning::VersionStruct::write).
    fn write_root(&self) -> Result<Value, EncodeError> {
        self.document()
    }

    fn write_root_with(
        &self,
        unknown: &bedrock_addon_core::document::Unknown,
    ) -> Result<(Value, Vec<bedrock_addon_core::diagnostic::Diagnostic>), EncodeError> {
        self.document_with(unknown)
    }
}

impl JsonPlaced for Block {
    /// `blocks/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path("blocks", &self.description.identifier)
    }
}

impl JsonDefinition for Block {
    type Kind = kind::Block;

    fn identifier(&self) -> &BlockId {
        &self.description.identifier
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `blocks/**/*.json`, packs bottom-up (the higher pack's definition replaces the lower's).
/// Vanilla's loader takes `.json` only, as written (`Accept::Ext`).
const WALK: Walk = Walk::json_tree_in(&["blocks"]);

/// The loader of `bp.blocks()`: `blocks/**/*.json` of every behaviour pack, combined top pack
/// wins.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Block>, Contribution<Block>> =
    load::registry::<Block>(WALK);

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn block_reads_as_pinned() {
        assert_reading_table(
            "BlockFamily",
            &reading_table::<super::BlockFamily>(),
            r#"
                text: Document, clamps beyond a double: true, duplicates kept in: ["components"]
                envelope: From(FormatVersion { major: 1, minor: 26, patch: 20 })
                missing version: at the floor, told at Info: None
                below floor: UseFormatVersion
                empty document: Error
                1.16.0..=1.26.0 (0..=13): json Own/NonStrict/Error/Warn/reports/lenient/Refuse, mcb Own/NonStrict/Error/Warn/reports/lenient/Refuse
                1.26.20..=1.26.60 (14..=15): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            "#,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::versioning::testing::slice_reading;

    /// The components of a payload are read by the reading of its kind of slice: the slice's
    /// `Reading` is the one place that says how block reads its scalars.
    #[test]
    fn a_slice_reads_its_components_by_its_reading() {
        let stable = <BlockHistory as VersionChain<BlockFamily>>::VERSIONS.len();
        let indices = (0..stable)
            .map(StructIndex::Stable)
            .chain([StructIndex::Beta]);
        for index in indices {
            let reading = slice_reading::<BlockFamily>(index, DocumentSource::Json);
            assert_eq!(
                reading_of(is_cereal(index)).scalar_rules(),
                reading.scalar_rules(),
                "{index:?}"
            );
        }
    }
}
