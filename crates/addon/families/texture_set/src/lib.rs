//! Texture sets: `<texture>.texture_set.json` beside the texture it describes, the layers
//! (`color`, `metalness_emissive_roughness`, `normal`, `heightmap`, …) the deferred renderer
//! reads. The client has no upgrade path: each of its schemas reads only the documents whose
//! `format_version` is exactly its version ([`history`]).
//!
//! The group a stack loads is the sets of every pack by the texture each sits beside, held to
//! the rules of the stack: each layer's image is in the set's own pack, and the capability lints
//! of the packs that ship sets. Which file the client loads for a texture path, and the images
//! of the layers, are the stack's composition of this family with the texture files.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
mod read;
mod stack;
mod texts;
mod types;
mod write;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath};
use bedrock_addon_core::document::unversioned::{lone, rejected};
use bedrock_addon_core::document::{Document, JsonDocument, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, Family, Identity, Loaded, MissingVersion, NoComponents, Parsed,
    RangeModel, finish,
};

pub use types::{LayerKind, LayerValue, Rgb, Rgba, TextureName, TextureSet};

use crate::history::{Any, TextureSetFamily};
use crate::read::{PAYLOAD, Schema, V1_16_100, read};
use crate::stack::{LintPack, capability_lints, check_layer_images, texture_path};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_support::texture::files::{
    IndexedFile, StackFiles, is_texture_file, listed_files,
};
#[doc(hidden)]
pub use stack::{PackSets, SetCopy, layer_path};

// ---- the file -------------------------------------------------------------------------------

impl Family for TextureSetFamily {
    const NAME: &'static str = "texture_set";
    const PAYLOAD_KEY: &'static str = PAYLOAD;
    const FLOOR: FormatVersion = V1_16_100;
    // The client compares the version with `==` against each schema's own: no range, no
    // closest lower struct, and the ceiling is the newest struct's version.
    const RANGE_MODEL: RangeModel = RangeModel::ExactOnly;
    type Latest = TextureSet;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    const ENVELOPE: Envelope = Envelope {
        // A missing version is an error here ("Invalid 'format_version' in JSON file").
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    fn identity(_: bedrock_addon_core::json::JsonSlice<'_>) -> Identity {
        Identity::default()
    }
}

impl JsonDocument for TextureSet {
    type Family = TextureSetFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    const NAME: &'static str = "texture set";

    /// Reads one `.texture_set.json` with the file's own reader (the version the client
    /// compares with `==`, its texts), then the chain: every rule of the file. Whether the
    /// layers' images exist is the stack's to say.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<TextureSetFamily> {
        match parse_file(origin, bytes, cx) {
            Ok(document) => DocumentRead::Stable(document),
            Err(rejected) => DocumentRead::Rejected(None, rejected),
        }
    }

    /// Parses one texture-set file without upgrading it: the struct its `format_version`
    /// selects, read by the file's own reader.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        _: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = lone(path);
        let mut diagnostics = Diagnostics::default();
        let read = read(bytes, path.as_str(), &mut diagnostics);
        match (read.value, read.schema) {
            (Some(value), Some(schema)) => {
                locate(&mut diagnostics, &origin);
                Ok(Document::new(
                    value,
                    origin,
                    version_info(schema),
                    diagnostics,
                ))
            }
            _ => Err(rejected(origin, diagnostics)),
        }
    }
}

impl TextureSet {
    /// The payload object of the set as JSON, without the envelope.
    #[must_use]
    pub fn to_value(&self) -> Value {
        write::write_layers(self)
    }
}

/// What a file of `schema`'s version says about its version.
fn version_info(schema: Schema) -> VersionInfo {
    let version = schema.version();
    VersionInfo::new(Some(version), false, version, version, false)
}

/// The path of the payload in the document.
#[doc(hidden)]
pub fn payload_at() -> JsonPath {
    JsonPath::new().child(PAYLOAD)
}

/// The chain's parse stage of a value already read at `version`.
fn parsed(value: Any, version: VersionInfo, diagnostics: Diagnostics) -> Parsed<TextureSetFamily> {
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    Parsed::from_value(Some(value), version, written, payload_at(), diagnostics)
}

/// The rest of the load: the upgrade step and the document.
fn finished(
    parsed: Parsed<TextureSetFamily>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<TextureSet>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    locate(&mut diagnostics, &origin);
    match outcome.value {
        Some(Loaded::Stable(set)) => Ok(Document::new(set, origin, outcome.version, diagnostics)),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// Rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<TextureSet>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    finished(
        parsed(document.into_value(), version, diagnostics),
        origin,
        cx,
    )
}

/// Loads the texture-set file at `origin`.
fn parse_file(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<TextureSet>, Rejected> {
    let mut diagnostics = Diagnostics::default();
    let read = read(bytes, origin.path.as_str(), &mut diagnostics);
    match (read.value, read.schema) {
        (Some(value), Some(schema)) => {
            finished(parsed(value, version_info(schema), diagnostics), origin, cx)
        }
        _ => Err(rejected(origin, diagnostics)),
    }
}

/// Gives every finding the document's pack and file, taken out and pushed back as the
/// unversioned kit does (so the moves and places the list held are dropped).
// TODO(phase 1 step 4): `Diagnostics::locate` in place, which keeps them; behaviour-bearing,
// with `unversioned`'s own switch.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
}

// ---- the group ------------------------------------------------------------------------------

/// The texture sets of a resource stack (`rp.texture_sets()`).
///
/// Hand-written: the sets are indexed by the texture each sits beside, per pack, and held to
/// rules that read the packs' listings (each layer's image in the set's own pack) and their
/// capabilities. The file itself is a [`JsonDocument`](bedrock_addon_core::document::JsonDocument).
#[derive(Debug)]
#[non_exhaustive]
pub struct TextureSets {
    /// The texture sets of each pack, in stack order, by the texture path each sits beside.
    sets: Vec<PackSets>,
    /// The findings that belong to no set.
    diagnostics: Diagnostics,
}

impl TextureSets {
    /// The sets of a stack with the findings of its rules.
    pub(crate) fn new(sets: Vec<PackSets>, diagnostics: Diagnostics) -> Self {
        Self { sets, diagnostics }
    }

    /// The texture sets of the stack that loaded, top pack first, each with the texture path
    /// it sits beside. A set a higher pack overrides is listed too: the stack's resolver of a
    /// texture path says which one the client uses.
    pub fn sets(&self) -> impl Iterator<Item = (&str, &Document<TextureSet>)> + '_ {
        self.set_copies()
            .filter_map(|(texture, copy)| Some((texture, copy.as_ref().ok()?)))
    }

    /// The texture sets of the stack that did not load, top pack first: for each the client
    /// shows the missing texture.
    pub fn rejected_sets(&self) -> impl Iterator<Item = (&str, &Rejected)> + '_ {
        self.set_copies()
            .filter_map(|(texture, copy)| Some((texture, copy.as_ref().err()?)))
    }

    /// Every texture set file of the stack, top pack first.
    fn set_copies(&self) -> impl Iterator<Item = (&str, &SetCopy)> + '_ {
        self.sets
            .iter()
            .flat_map(|pack| pack.iter().map(|(texture, copy)| (texture.as_str(), copy)))
    }

    /// The set of the pack at `stack_index` beside the texture `texture`, as it loaded.
    #[doc(hidden)]
    pub fn set_of(&self, stack_index: usize, texture: &str) -> Option<&SetCopy> {
        self.sets.get(stack_index)?.get(texture)
    }

    /// The sets of each pack, in stack order.
    #[doc(hidden)]
    pub fn packs(&self) -> &[PackSets] {
        &self.sets
    }

    /// Every finding of the group: the sets, then the rest.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        let sets = self.set_copies().flat_map(|(_, copy)| match copy {
            Ok(set) => set.diagnostics().iter(),
            Err(rejected) => rejected.diagnostics.iter(),
        });
        sets.chain(self.diagnostics.iter())
    }
}

/// One walked texture set file: where it is and what its parse job made of it.
#[derive(Debug)]
pub struct SetFileParsed {
    /// The pack.
    pack: bedrock_addon_core::pack_identity::PackRef,
    /// The path in the pack.
    path: PackPath,
    /// The key the stack reads the file at.
    key: PackPath,
    /// The texture it sits beside and the document, when the file is some texture's set.
    set: Option<(String, Box<SetCopy>)>,
}

/// The files whose content the group reads: every texture set.
fn is_read(path: &PackPath) -> bool {
    path.as_str().ends_with("texture_set.json")
}

/// One file of the walk.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> SetFileParsed {
    let set = texture_path(file.key.as_str()).map(|texture| {
        let copy = parse_file(origin_of(file), &file.bytes, cx);
        (texture.to_owned(), Box::new(copy))
    });
    SetFileParsed {
        pack: file.pack.clone(),
        path: file.path.clone(),
        key: file.key.clone(),
        set,
    }
}

/// The stack rules of the texture sets: each layer's image in the set's own pack, and
/// the capability lints.
fn check_sets(
    sets: &mut [PackSets],
    files: &StackFiles,
    stack: &StackCx<'_>,
    diagnostics: &mut Diagnostics,
) {
    for (index, sets) in sets.iter_mut().enumerate() {
        check_layer_images(sets, index, files);
    }
    let lint_packs = stack.packs.iter().zip(&files.packs);
    let lint_packs: Vec<LintPack<'_>> = lint_packs
        .map(|(entry, files)| LintPack { entry, files })
        .collect();
    capability_lints(&lint_packs, diagnostics);
}

/// The texture sets, from every parsed file in walk order.
fn combine(parsed: Vec<SetFileParsed>, stack: &StackCx<'_>) -> TextureSets {
    let mut files = listed_files(stack);
    let mut sets: Vec<PackSets> = stack.packs.iter().map(|_| PackSets::new()).collect();
    for file in parsed {
        let index = file.pack.index();
        // A file the walk read is a file of its pack even where the listing lacks it.
        let pack = index.and_then(|index| files.packs.get_mut(index));
        if let Some(pack) = pack.filter(|_| is_texture_file(&file.key)) {
            let path = file.path.clone();
            pack.files
                .insert(file.key.as_str().to_owned(), IndexedFile { path });
        }
        let slot = index.and_then(|index| sets.get_mut(index));
        if let (Some((texture, copy)), Some(slot)) = (file.set, slot) {
            slot.insert(texture, *copy);
        }
    }
    let mut diagnostics = Diagnostics::default();
    check_sets(&mut sets, &files, stack, &mut diagnostics);
    TextureSets::new(sets, diagnostics)
}

/// The loader of `rp.texture_sets()`: every texture set of every resource pack, held to the
/// rules of the stack.
#[doc(hidden)]
pub const LOADER: CollectionLoader<TextureSets, SetFileParsed> = CollectionLoader {
    walk: Walk::json_tree_in(&[""]).accept(Accept::Fn(is_read)),
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl bedrock_addon_core::load::Reported for TextureSets {
    fn reported(
        &self,
    ) -> Box<dyn Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn texture_set_reads_as_pinned() {
        assert_reading_table(
            "TextureSetFamily",
            &reading_table::<super::TextureSetFamily>(),
            r"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Error
                1.16.100..=1.21.30 (0..=1): json Own/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
            ",
        );
    }
}

#[cfg(test)]
mod identity {
    //! What the family's documents say about themselves before they are parsed.

    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::{Family, Identity, parse_as_written};

    /// The family reads no identity from its payload, not even a `description.identifier`
    /// written in it (the default would).
    #[test]
    fn texture_set_reads_no_identity_from_the_payload() {
        let text = format!(
            r#"{{"format_version": "1.21.0", "{}": {{"description": {{"identifier": "demo:x"}}}}}}"#,
            <super::TextureSetFamily as Family>::PAYLOAD_KEY,
        );
        let parsed = parse_as_written::<super::TextureSetFamily>(
            JsonSlice::new(&text),
            &LoadContext::detached(),
        );
        assert_eq!(parsed.identity, Identity::default());
    }
}
