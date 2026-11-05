//! Loading texture sets: the family's version-chain parameters, `Content`, one file through
//! the reader and the chain, and the checks that need the stack (each layer's image in the
//! set's own pack, the capability lints).

use indexmap::IndexMap;

use super::history::{Any, TextureSetFamily};
use super::set::TextureSet;
use super::set_read::{PAYLOAD, Schema, V1_16_100, read};
use super::texts;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Content, Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::load::StackPack;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    Envelope, Family, Identity, Loaded, MissingVersion, NoComponents, Parsed, RangeModel, finish,
};
use bedrock_addon_support::texture::document::{detached, locate, rejected};
use bedrock_addon_support::texture::files::{
    PackFiles, StackFiles, TEXTURE_SET, VERBATIM_OR_IMAGE,
};
use bedrock_addon_support::texture::texts::fixed;

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

impl bedrock_addon_core::__private::Sealed for TextureSet {}

impl Content for TextureSet {
    const NAME: &'static str = "texture set";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// Reads one `.texture_set.json` on its own: every rule of the file. Whether the layers'
    /// images exist is the stack's to say.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(detached(path), bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for TextureSet {}

impl TextureSet {
    /// Parses one texture-set file without upgrading it: the struct its `format_version`
    /// selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        _: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = detached(path);
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

    /// The payload object of the set as JSON, without the envelope.
    #[must_use]
    pub fn to_value(&self) -> Value {
        super::set_write::write_layers(self)
    }
}

/// What a file of `schema`'s version says about its version.
fn version_info(schema: Schema) -> VersionInfo {
    let version = schema.version();
    VersionInfo::new(Some(version), false, version, version, false)
}

/// The path of the payload in the document.
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
pub(super) fn upgrade_document(
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
pub(super) fn parse_file(
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

// ---- the stack ------------------------------------------------------------------------------

/// One pack's copy of a texture set: the document, or why the client shows the missing
/// texture instead.
pub type SetCopy = Result<Document<TextureSet>, Rejected>;

/// The texture sets of one pack, by the texture path each sits beside (the file's key without
/// `.texture_set.json`).
pub type PackSets = IndexMap<String, SetCopy>;

/// The texture path a texture-set file at `key` belongs to.
pub(super) fn texture_path(key: &str) -> Option<&str> {
    key.strip_suffix(TEXTURE_SET)
}

/// The path a layer named `name` of the texture set beside `texture` is loaded from, before
/// the extensions: the set's folder, a `/`, and the name as written.
// The value is joined to the set's folder as written and is not normalised.
pub fn layer_path(texture: &str, name: &str) -> String {
    match texture.rsplit_once('/') {
        Some((folder, _)) => format!("{folder}/{name}"),
        None => name.to_owned(),
    }
}

/// The layer images of one pack: a set with a layer whose image is not in its own pack is not
/// loaded. The whole texture is replaced by the missing texture, and an image of the same name
/// in another pack is not used.
pub(super) fn check_layer_images(sets: &mut PackSets, stack_index: usize, files: &StackFiles) {
    for (texture, copy) in sets.iter_mut() {
        let Ok(document) = copy else {
            continue;
        };
        let mut missing = Diagnostics::default();
        for (layer, name) in document.textures() {
            let path = layer_path(texture, name.as_str());
            if files
                .in_pack(stack_index, &path, &VERBATIM_OR_IMAGE)
                .is_none()
            {
                let at = payload_at().child(layer.key());
                let code = DiagnosticCode::TextureNotFound {
                    text: fixed(texts::SET_NO_IMAGE),
                };
                // The game logs this at the content log's information level; the set is still
                // replaced by the missing texture.
                missing.push(Diagnostic::info(code).at(at).phase(Phase::Validate));
            }
        }
        if missing.is_empty() {
            continue;
        }
        let origin = document.origin().clone();
        let mut diagnostics = document.diagnostics().clone();
        locate(&mut missing, &origin);
        for finding in missing {
            diagnostics.push(finding);
        }
        *copy = Err(Rejected::new(origin, diagnostics));
    }
}

/// The capability that makes the client use the layers besides `color` in the deferred
/// renderer.
const PBR: &str = "pbr";
/// The capability of the ray-traced renderer.
const RAYTRACED: &str = "raytraced";
/// From this `min_engine_version` on, a resource pack without `pbr` turns Vibrant Visuals off
/// for the whole stack.
const PBR_REQUIRED_FROM: SemVersion = SemVersion::new(1, 21, 110);

/// One pack as the capability lints see it.
pub(super) struct LintPack<'a> {
    /// The stack entry.
    pub(super) entry: &'a StackPack,
    /// The pack's files.
    pub(super) files: &'a PackFiles,
}

/// The pack lints of the texture sets: one for each pack that turns Vibrant Visuals off, and one
/// for each pack that ships texture sets in a stack where no pack declares a capability that
/// uses them.
pub(super) fn capability_lints(packs: &[LintPack<'_>], d: &mut Diagnostics) {
    // The capabilities are the manifest's as the pack was opened with it: a
    // `manifest.json` of the active subpack is no manifest.
    let declares = |pack: &LintPack<'_>, name: &str| {
        let mut declared = pack.entry.capabilities.iter();
        declared.any(|capability| &**capability == name)
    };
    let any_renderer = packs
        .iter()
        .any(|pack| declares(pack, PBR) || declares(pack, RAYTRACED));
    for pack in packs {
        let at_manifest = |code: DiagnosticCode| {
            let location = Location::new(
                Some(pack.entry.pack.clone()),
                Some(PackPath::from_static("manifest.json")),
                JsonPath::new(),
                None,
            );
            Diagnostic::new(code, Severity::Info, Phase::Validate, location)
        };
        let context = &pack.entry.context;
        let from_required = context
            .min_engine_version
            .as_ref()
            .is_some_and(|version| version.version >= PBR_REQUIRED_FROM);
        if !context.is_base_game_pack && from_required && !declares(pack, PBR) {
            d.push(at_manifest(DiagnosticCode::VibrantVisualsDisabledByPack));
        }
        let ships_sets = pack
            .files
            .files
            .keys()
            .any(|key| key.ends_with(TEXTURE_SET));
        if ships_sets && !any_renderer {
            // The game has no sentence for this lint; the text is core's.
            d.push(at_manifest(DiagnosticCode::CapabilityHint { text: None }));
        }
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
